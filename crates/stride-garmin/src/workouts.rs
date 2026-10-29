//! Garmin's scheduled workouts and workout templates.
//!
//! The reference is `backend/app/ingest/garmin_workouts.py`. This is a port
//! of it: the same tolerance for the field names Garmin sends in more than
//! one shape, the same pace units, and the same session types.
//!
//! Two things are worth stating, because they are the ones that are easy to
//! get wrong:
//!
//! * **A pace is not a distance.** Garmin sends the target of a step in one
//!   field, and what it means depends on the step: a distance step's target
//!   is metres, a pace step's target is a pace. A band comes from
//!   `targetValueLow`/`targetValueHigh`; a single `targetValue` on a
//!   distance step is never read as a pace.
//! * **A pace is in seconds per kilometre.** Garmin sends it either as
//!   seconds per metre (0.3) or as `m:ss` ("5:00"), and a value outside
//!   150–600 s/km is a distance that was mislabelled, not a pace.

use serde_json::Value;

use crate::types::{ScheduledWorkout, WorkoutDetail};

/// The keys a collection of scheduled workouts may hide behind.
const SCHEDULED_KEYS: [&str; 4] = ["scheduledWorkouts", "workouts", "calendarItems", "items"];

/// The keys a workout's steps may hide behind.
const STEP_KEYS: [&str; 2] = ["workoutSteps", "steps"];

/// The scheduled workouts in a payload.
///
/// Garmin sends them under more than one key, and the shape of a row varies
/// too, so every field is looked up under each of the names it is known by.
pub fn scheduled_workouts(payload: &Value) -> Vec<ScheduledWorkout> {
    rows(payload, &SCHEDULED_KEYS)
        .into_iter()
        .map(|row| ScheduledWorkout {
            workout_id: first_string(&row, &["workoutId", "id", "scheduledWorkoutId"]),
            name: first_string(&row, &["workoutName", "name", "title"]),
            date: first_string(&row, &["date", "scheduledDate", "calendarDate"]),
            sport: first_string(&row, &["sportType", "sportTypeKey", "activityType"]),
            status: first_string(&row, &["status", "workoutStatus"]),
        })
        .collect()
}

/// The detail of one workout: the session type, the distance and the pace band.
pub fn workout_detail(payload: &Value) -> WorkoutDetail {
    let Some(object) = payload.as_object() else {
        return WorkoutDetail::default();
    };
    if object.is_empty() {
        // Nothing to read: there is no workout here.
        return WorkoutDetail::default();
    }
    let steps = flatten_steps(
        STEP_KEYS
            .iter()
            .find_map(|key| object.get(*key))
            .unwrap_or(&Value::Null),
    );

    let title = first_string(payload, &["workoutName", "name"]).unwrap_or_default();
    let raw_titles: Vec<String> = steps
        .iter()
        .map(|step| {
            first_string(step, &["description", "stepName", "stepType"]).unwrap_or_default()
        })
        .collect();
    let text = format!("{} {}", title, raw_titles.join(" ")).to_lowercase();

    let mut session_type = if text.contains("race") || text.contains("competition") {
        "race"
    } else if ["tempo", "threshold", "lactate"]
        .iter()
        .any(|term| text.contains(term))
    {
        "tempo"
    } else if text.contains("strength") || text.contains("weight") {
        "strength"
    } else if text.contains("rest") {
        "rest"
    } else if text.contains("recovery") {
        "recovery"
    } else if steps.iter().any(|step| {
        first_string(step, &["stepType"])
            .unwrap_or_default()
            .to_lowercase()
            .contains("repeat")
    }) || steps
        .iter()
        .any(|step| step.get("stepList").is_some_and(Value::is_array))
    {
        "interval"
    } else {
        "easy"
    };

    let mut distance = 0.0;
    let mut paces: Vec<f64> = Vec::new();
    let mut zones: Vec<f64> = Vec::new();
    let mut long_duration = false;

    for step in &steps {
        let duration_type = first_string(step, &["durationType"])
            .unwrap_or_default()
            .to_lowercase();
        let target_value = safe_float(step.get("targetValue"));
        if duration_type == "distance" {
            if let Some(value) = target_value {
                distance += value;
            }
        }
        if duration_type == "time" {
            if let Some(value) = target_value {
                if value > 3600.0 {
                    long_duration = true;
                }
            }
        }
        let target_type = first_string(step, &["targetType", "targetValueType"])
            .unwrap_or_default()
            .to_lowercase();
        if target_type.contains("pace") || target_type == "speed" || target_type == "pace_zone" {
            // A band comes from targetValueLow/High. targetValue alone is a
            // single point target — and on a distance step it is a distance,
            // so it must never be read as a pace.
            let mut low = pace(step.get("targetValueLow"));
            let mut high = pace(step.get("targetValueHigh"));
            if low.is_none() && high.is_none() {
                if let Some(single) = pace(step.get("targetValue")) {
                    low = Some(single);
                    high = Some(single);
                }
            }
            if low.is_none() {
                low = high;
            }
            if high.is_none() {
                high = low;
            }
            for value in [low, high].into_iter().flatten() {
                paces.push(value);
            }
        }
        let zone = if duration_type.starts_with("hr_") {
            safe_float(step.get("targetValue"))
        } else {
            safe_float(step.get("targetZone"))
        };
        if let Some(zone) = zone {
            if duration_type.starts_with("hr_") || target_type.contains("heart") {
                zones.push(zone);
            }
        }
    }

    if session_type == "easy" && (distance > 12_000.0 || long_duration) {
        session_type = "long";
    }

    let details = raw_titles
        .iter()
        .filter(|title| !title.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");

    WorkoutDetail {
        session_type: Some(session_type.to_string()),
        target_distance_m: if distance > 0.0 { Some(distance) } else { None },
        target_pace_low_s_km: paces.iter().cloned().fold(None, |low: Option<f64>, pace| {
            Some(low.map_or(pace, |value| value.min(pace)))
        }),
        target_pace_high_s_km: paces.iter().cloned().fold(None, |high: Option<f64>, pace| {
            Some(high.map_or(pace, |value| value.max(pace)))
        }),
        hr_zone_target: zones.iter().cloned().fold(None, |low: Option<f64>, zone| {
            Some(low.map_or(zone, |value| value.min(zone)))
        }),
        title: if title.is_empty() {
            raw_titles
                .iter()
                .find(|candidate| !candidate.is_empty())
                .cloned()
        } else {
            Some(title)
        },
        details: if details.is_empty() {
            None
        } else {
            Some(details)
        },
    }
}

/// The rows of a collection, whatever it is wrapped in.
fn rows(payload: &Value, keys: &[&str]) -> Vec<Value> {
    if let Some(list) = payload.as_array() {
        return list.clone();
    }
    for key in keys {
        if let Some(list) = payload.get(*key).and_then(Value::as_array) {
            return list.clone();
        }
    }
    Vec::new()
}

/// The steps of a workout, flattened.
///
/// A repeat step holds its repetitions in a nested `stepList`, and the order
/// matters, so the nested steps follow their parent.
fn flatten_steps(steps: &Value) -> Vec<Value> {
    let Some(list) = steps.as_array() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for step in list {
        if !step.is_object() {
            continue;
        }
        out.push(step.clone());
        if let Some(nested) = step.get("stepList").and_then(Value::as_array) {
            out.extend(flatten_steps(&Value::Array(nested.clone())));
        }
    }
    out
}

/// The first of the named fields that holds something.
///
/// An id is sometimes a number and sometimes a string, so both are read and
/// both come out as text.
fn first_string(payload: &Value, names: &[&str]) -> Option<String> {
    for name in names {
        let Some(value) = payload.get(*name) else {
            continue;
        };
        let text = match value {
            Value::String(text) => text.trim().to_string(),
            Value::Number(number) => number.to_string(),
            _ => continue,
        };
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

/// A number from a payload, or nothing.
///
/// A value that is not a number is absent, and zero is a number.
fn safe_float(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// A pace target, in seconds per kilometre.
///
/// Garmin sends it either as seconds per metre (0.3) or as `m:ss` ("5:00").
/// Both must land in the same unit, and the result must be physically
/// possible for running.
fn pace(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::String(text) if text.contains(':') => {
            let (minutes, seconds) = text.split_once(':')?;
            let candidate =
                minutes.trim().parse::<f64>().ok()? * 60.0 + seconds.trim().parse::<f64>().ok()?;
            (150.0..=600.0).contains(&candidate).then_some(candidate)
        }
        other => {
            let number = safe_float(Some(other))?;
            let candidate = if number > 0.0 && number < 20.0 {
                number * 1000.0
            } else {
                number
            };
            (150.0..=600.0).contains(&candidate).then_some(candidate)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A repeat workout with one paced interval inside it.
    fn repeat_workout(low: Value, high: Value) -> Value {
        json!({
            "workoutName": "Track session",
            "workoutSteps": [
                {
                    "stepType": "repeat",
                    "numberOfIterations": 4,
                    "stepList": [
                        {
                            "stepType": "interval",
                            "targetType": "pace",
                            "targetValueLow": low,
                            "targetValueHigh": high,
                        },
                    ],
                },
            ],
        })
    }

    /// A nested repeat is an interval, and the pace band is in seconds per
    /// kilometre however Garmin sent it.
    #[test]
    fn a_repeat_workout_is_an_interval_with_a_sane_pace_band() {
        for (low, high) in [
            (json!(0.3), json!(0.36)),
            (json!(300), json!(360)),
            (json!("5:00"), json!("6:00")),
        ] {
            let detail = workout_detail(&repeat_workout(low, high));
            assert_eq!(detail.session_type.as_deref(), Some("interval"));
            assert_eq!(detail.target_pace_low_s_km, Some(300.0));
            assert_eq!(detail.target_pace_high_s_km, Some(360.0));
            let low = detail.target_pace_low_s_km.expect("a low bound");
            let high = detail.target_pace_high_s_km.expect("a high bound");
            assert!(150.0 <= low && low <= high && high <= 600.0);
        }
    }

    /// A distance target is a distance, never a pace.
    #[test]
    fn a_distance_target_is_not_a_pace() {
        let mut payload = repeat_workout(json!(0.3), json!(0.36));
        payload["workoutSteps"][0]["stepList"][0]["durationType"] = json!("distance");
        payload["workoutSteps"][0]["stepList"][0]["targetValue"] = json!(1000);
        let detail = workout_detail(&payload);
        assert_eq!(
            detail.target_pace_low_s_km,
            Some(300.0),
            "the band comes from the low and the high"
        );
        assert_eq!(detail.target_pace_high_s_km, Some(360.0));
    }

    /// A distance workout's distance is the sum of its distance steps.
    #[test]
    fn a_distance_workout_sums_its_distance_steps() {
        let payload = json!({
            "workoutName": "Long run",
            "workoutSteps": [
                {"durationType": "distance", "targetValue": 5000},
                {"durationType": "distance", "targetValue": 8000},
            ],
        });
        let detail = workout_detail(&payload);
        assert_eq!(detail.target_distance_m, Some(13000.0));
        assert_eq!(detail.session_type.as_deref(), Some("long"));
    }

    /// The title says what the session is.
    #[test]
    fn the_title_says_what_the_session_is() {
        let detail = workout_detail(&json!({
            "workoutName": "Threshold fartlek",
            "workoutSteps": [{"stepType": "interval"}],
        }));
        assert_eq!(detail.session_type.as_deref(), Some("tempo"));
        assert_eq!(detail.title.as_deref(), Some("Threshold fartlek"));
    }

    /// The scheduled workouts are read out of every shape Garmin sends.
    #[test]
    fn the_scheduled_workouts_are_read_out_of_every_shape() {
        let payload = json!({
            "scheduledWorkouts": [
                {
                    "workoutId": 42,
                    "workoutName": "Tempo",
                    "date": "2026-03-02",
                    "sportType": "running",
                    "status": "scheduled",
                },
                {
                    "scheduledWorkoutId": 43,
                    "name": "Rest",
                    "scheduledDate": "2026-03-03",
                    "sportTypeKey": "rest",
                    "workoutStatus": "scheduled",
                },
            ],
        });
        let rows = scheduled_workouts(&payload);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].workout_id.as_deref(), Some("42"));
        assert_eq!(rows[0].name.as_deref(), Some("Tempo"));
        assert_eq!(rows[0].date.as_deref(), Some("2026-03-02"));
        assert_eq!(rows[0].sport.as_deref(), Some("running"));
        assert_eq!(rows[0].status.as_deref(), Some("scheduled"));
        assert_eq!(rows[1].workout_id.as_deref(), Some("43"));
        assert_eq!(rows[1].date.as_deref(), Some("2026-03-03"));
        assert_eq!(rows[1].sport.as_deref(), Some("rest"));
    }

    /// A collection under another key is still a collection.
    #[test]
    fn a_collection_under_another_key_is_still_a_collection() {
        let payload = json!({"items": [{"id": 7, "calendarDate": "2026-03-04"}]});
        let rows = scheduled_workouts(&payload);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].workout_id.as_deref(), Some("7"));
        assert_eq!(rows[0].date.as_deref(), Some("2026-03-04"));
    }

    /// An answer that is not a workout is no detail.
    #[test]
    fn an_answer_that_is_not_a_workout_is_no_detail() {
        let detail = workout_detail(&json!({}));
        assert_eq!(detail.session_type, None);
        assert_eq!(detail.target_distance_m, None);
    }
}
