//! The payload types the client returns.
//!
//! Garmin's Personal API is not a strongly typed API: the same field turns up
//! under three different names depending on the endpoint. Every type here
//! therefore carries a `raw` map with the untouched payload, so nothing the
//! reference implementation could read is lost.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The athlete's Garmin profile.
///
/// Garmin's profile payload carries a dozen display names for the same person.
/// The four the reference implementation reads are lifted out; everything else
/// stays in [`Profile::raw`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    /// `displayName` — the name the athlete sees in Garmin Connect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// `fullName`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// `userName`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    /// `name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The rest of the payload, untouched.
    #[serde(flatten)]
    pub raw: Map<String, Value>,
}

impl Profile {
    /// Builds a profile from Garmin's raw payload.
    pub fn from_value(payload: &Value) -> Self {
        let mut raw = match payload {
            Value::Object(map) => map.clone(),
            _ => Map::new(),
        };
        let display_name = take_string(&mut raw, "displayName");
        let full_name = take_string(&mut raw, "fullName");
        let user_name = take_string(&mut raw, "userName");
        let name = take_string(&mut raw, "name");
        Self {
            display_name,
            full_name,
            user_name,
            name,
            raw,
        }
    }

    /// The athlete's display name, whichever field Garmin used.
    ///
    /// The order matches the reference implementation: `displayName`,
    /// `fullName`, `userName`, `name`.
    pub fn athlete_name(&self) -> Option<&str> {
        self.display_name
            .as_deref()
            .or(self.full_name.as_deref())
            .or(self.user_name.as_deref())
            .or(self.name.as_deref())
    }
}

/// One heart-rate zone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Zone {
    /// The zone number, 1-based.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_number: Option<i64>,
    /// The zone's name, when Garmin sent one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The lower bound, in beats per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low_bpm: Option<f64>,
    /// The upper bound, in beats per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high_bpm: Option<f64>,
    /// The rest of the zone payload, untouched.
    #[serde(flatten)]
    pub raw: Map<String, Value>,
}

/// The athlete's heart-rate zone model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZoneModel {
    /// The zones, in the order Garmin listed them.
    pub zones: Vec<Zone>,
    /// The rest of the payload, untouched.
    #[serde(flatten)]
    pub raw: Map<String, Value>,
}

impl ZoneModel {
    /// Builds a zone model from Garmin's raw payload.
    ///
    /// Garmin has shipped several shapes of this endpoint, so the parser looks
    /// for the first array it recognises and reads each zone's fields under any
    /// of the names it has used.
    pub fn from_value(payload: &Value) -> Self {
        let raw = match payload {
            Value::Object(map) => map.clone(),
            _ => Map::new(),
        };
        let zones = ["zones", "zoneBounds", "heartRateZones", "zoneBoundsList"]
            .iter()
            .find_map(|key| raw.get(*key).and_then(Value::as_array))
            .map(|items| items.iter().map(zone_from_value).collect())
            .unwrap_or_default();
        Self { zones, raw }
    }
}

/// One activity, as it appears in the activity list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    /// Garmin's activity id. This is the `external_id` the app stores.
    pub activity_id: String,
    /// The activity's name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The sport, when Garmin named one (`activityTypeDTO.typeKey`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sport_type: Option<String>,
    /// The activity's local start time, as Garmin sent it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time_local: Option<String>,
    /// The distance, in metres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_m: Option<f64>,
    /// The duration, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    /// The rest of the payload, untouched.
    #[serde(flatten)]
    pub raw: Map<String, Value>,
}

impl ActivitySummary {
    /// Builds a summary from one item of the activity list.
    pub fn from_value(payload: &Value) -> Option<Self> {
        let object = payload.as_object()?;
        let mut raw = object.clone();
        let activity_id = take_id(&mut raw, "activityId")?;
        let name = take_string(&mut raw, "activityName").or_else(|| take_string(&mut raw, "name"));
        let sport_type = raw
            .get("activityTypeDTO")
            .and_then(Value::as_object)
            .and_then(|dto| dto.get("typeKey"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let start_time_local = take_string(&mut raw, "startTimeLocal");
        let summary = raw.get("summaryDTO").and_then(Value::as_object);
        let distance_m = summary
            .and_then(|dto| dto.get("distance"))
            .and_then(Value::as_f64)
            .or_else(|| raw.get("distance").and_then(Value::as_f64));
        let duration_s = summary
            .and_then(|dto| dto.get("duration"))
            .and_then(Value::as_f64)
            .or_else(|| raw.get("duration").and_then(Value::as_f64));
        Some(Self {
            activity_id,
            name,
            sport_type,
            start_time_local,
            distance_m,
            duration_s,
            raw,
        })
    }
}

fn zone_from_value(payload: &Value) -> Zone {
    let mut raw = match payload {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    };
    let zone_number = [
        "zoneNumber",
        "zoneNumberOrdered",
        "zoneOrder",
        "order",
        "number",
    ]
    .iter()
    .find_map(|key| raw.get(*key).and_then(Value::as_i64));
    let name = take_string(&mut raw, "zoneName").or_else(|| take_string(&mut raw, "name"));
    let low_bpm = ["lowerBound", "low", "min", "lowerBoundInclusive"]
        .iter()
        .find_map(|key| raw.get(*key).and_then(Value::as_f64));
    let high_bpm = ["upperBound", "high", "max", "upperBoundExclusive"]
        .iter()
        .find_map(|key| raw.get(*key).and_then(Value::as_f64));
    Zone {
        zone_number,
        name,
        low_bpm,
        high_bpm,
        raw,
    }
}

/// Removes a string field from the raw map and returns it.
fn take_string(raw: &mut Map<String, Value>, key: &str) -> Option<String> {
    let value = raw.remove(key)?;
    match value {
        Value::String(text) => Some(text),
        other => Some(other.to_string()),
    }
}

/// Removes an identifier field from the raw map and returns it as a string.
///
/// Garmin sends the activity id as a number on some endpoints and as a string
/// on others.
fn take_id(raw: &mut Map<String, Value>, key: &str) -> Option<String> {
    match raw.remove(key) {
        Some(Value::String(text)) => Some(text),
        Some(Value::Number(number)) => Some(number.to_string()),
        Some(other) => Some(other.to_string()),
        None => None,
    }
}
