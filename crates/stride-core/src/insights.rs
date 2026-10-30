//! Insights: the plain-language layer over the runs.
//!
//! Three jobs, one module:
//!
//! * **Decide today**: [`today`] turns the run history and last night's
//!   recovery into one call, one reason and one guardrail.
//! * **Understand each run**: [`run_story`] says what a run meant against the
//!   athlete's own recent running: a chip for the list and a few sentences
//!   for the detail screen.
//! * **Stay motivated**: [`progress`] finds proof of getting fitter, forgiving
//!   week-by-week consistency, best runs and lifetime milestones.
//!
//! There is no Python reference for this module. The Python oracle is frozen
//! for engines that did not exist in it; these are tested here.
//!
//! # Evidence the rules lean on
//!
//! * **The spike guard.** In the 2025 Garmin-RUNSAFE cohort (5,205 runners,
//!   BJSM), a single run more than 10% longer than the longest run of the
//!   previous 30 days raised the rate of overuse injury (HRR 1.64; more than
//!   double, 2.28). Week-to-week change and the acute/chronic ratio did not.
//!   So the guardrail names "the longest run of the last 30 days, plus 10%"
//!   as where the observed risk rose, never as a safe zone: the study is an
//!   association, not a threshold, its authors float a lower cut-off, and a
//!   rolling window lets runs ratchet up just under the line. The copy says
//!   so.
//! * **No cramming.** A missed week is never made up: catching up is how a
//!   single run spikes.
//! * **Forgiving consistency.** A broken streak that can be repaired keeps
//!   people going; a reset to zero makes them stop. A week counts, and one
//!   quiet week in six is a grace week, not a break.
//! * **Recovery is a personal trend, not a verdict.** HRV-guided training has
//!   not beaten plain training in meta-analyses, and wrist HRV is noisy, so a
//!   recovery signal only ever softens the call; it never forbids a run.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::numeric::{median, round_to};

/// The single-run spike ratio: a run longer than this multiple of the
/// longest run of the previous 30 days is a spike.
pub const SPIKE_RATIO: f64 = 1.1;

/// How close a run has to be to the spike limit for the distance guardrail
/// to be worth showing.
///
/// The guardrail exists because a jump in distance is the one thing the
/// studies link to overuse injury. A run at 40% of the limit is not a
/// jump, and a warning about it is noise on the runner's first screen.
pub const GUARDRAIL_NEAR_RATIO: f64 = 0.9;

/// The smallest efficiency a percentage change may be computed against.
///
/// Efficiency is metres per minute per beat: about 1.2 for an easy run at
/// 150 bpm. A number far below that is not a run — it is a run with missing
/// data, a walk, or a division that went wrong — and a percentage against
/// it is an artefact of the denominator, not a change in the athlete.
pub const EFFICIENCY_FLOOR: f64 = 0.5;

/// The largest efficiency change worth showing as a percentage.
///
/// The trends the app reads are a few per cent; a change of half the
/// number is a data problem, not a fitness one.
pub const MAX_EFFICIENCY_CHANGE_PCT: f64 = 50.0;

/// The percentage change between two efficiency readings, when it means
/// something.
///
/// A percentage against a denominator that is nearly zero is not a change
/// in the athlete, it is an artefact of the division — and it is the kind
/// of number that reaches a runner as "1,847% more efficient". So the
/// baseline has to be a real run's worth of efficiency, and the change
/// has to be a change a body makes.
fn efficiency_change(now: f64, then: f64) -> Option<f64> {
    if then < EFFICIENCY_FLOOR || now <= 0.0 {
        return None;
    }
    let delta = (now / then - 1.0) * 100.0;
    if !delta.is_finite() || delta.abs() > MAX_EFFICIENCY_CHANGE_PCT {
        return None;
    }
    Some(delta)
}

/// A gap this many days long makes the next run a comeback.
pub const COMEBACK_DAYS: i64 = 14;

/// The lifetime distance milestones, in kilometres.
pub const DISTANCE_TIERS_KM: [f64; 10] = [
    50.0, 100.0, 250.0, 500.0, 1000.0, 2000.0, 3000.0, 5000.0, 7500.0, 10000.0,
];

/// The best-run distances: label and metres.
pub const BEST_DISTANCES: [(&str, f64); 4] = [
    ("5K", 5000.0),
    ("10K", 10000.0),
    ("Half", 21097.5),
    ("Marathon", 42195.0),
];

/// One run, as the insights read it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Run {
    /// The activity id.
    pub id: i64,
    /// The activity's name.
    #[serde(default)]
    pub name: String,
    /// The local date, `YYYY-MM-DD`.
    pub date: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The moving time, in seconds.
    pub moving_time_s: f64,
    /// The average heart rate, when the watch recorded one.
    #[serde(default)]
    pub avg_hr: Option<f64>,
}

impl Run {
    /// The pace, in seconds per kilometre.
    pub fn pace_s_km(&self) -> Option<f64> {
        if self.distance_m > 0.0 && self.moving_time_s > 0.0 {
            Some(self.moving_time_s / (self.distance_m / 1000.0))
        } else {
            None
        }
    }

    /// Metres per minute per heartbeat: how much running each beat buys.
    ///
    /// Higher is fitter. It moves with heat, hills and a bad wrist reading,
    /// which is why it is only ever compared across many runs.
    pub fn efficiency(&self) -> Option<f64> {
        let hr = self.avg_hr.filter(|hr| *hr >= 80.0 && *hr <= 220.0)?;
        if self.distance_m <= 0.0 || self.moving_time_s <= 0.0 {
            return None;
        }
        Some(self.distance_m / (self.moving_time_s / 60.0) / hr)
    }

    fn day(&self) -> Option<Date> {
        Date::parse(&self.date)
    }
}

/// One night of recovery data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Recovery {
    /// The local date, `YYYY-MM-DD`.
    pub date: String,
    /// The night's sleep, in seconds.
    #[serde(default)]
    pub sleep_s: Option<f64>,
    /// The overnight HRV, in milliseconds.
    #[serde(default)]
    pub hrv_ms: Option<f64>,
    /// The resting heart rate.
    #[serde(default)]
    pub resting_hr: Option<f64>,
}

/// How a chip should read: `good`, `warn` or `neutral`.
pub type Tone = &'static str;

/// What one run meant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunStory {
    /// The activity id.
    pub id: i64,
    /// The one chip the list shows ("Usual run", "Big jump in distance").
    pub chip: String,
    /// `good`, `warn` or `neutral`.
    pub tone: String,
    /// The sentences the detail screen shows, most important first.
    pub story: Vec<String>,
    /// Whether this run was a spike past the 30-day longest.
    pub spike: bool,
    /// The longest run of the 30 days before this one, in metres.
    pub longest_prior_30d_m: Option<f64>,
    /// How many earlier runs of a similar distance it was compared with.
    pub similar_runs: usize,
    /// Its efficiency against those runs, as a percentage (+ is fitter).
    pub efficiency_delta_pct: Option<f64>,
    /// Its pace against those runs, in seconds per km (- is faster).
    pub pace_delta_s_km: Option<f64>,
    /// Its average heart rate against those runs, in bpm.
    pub hr_delta_bpm: Option<f64>,
    /// The best-run distances it set a one-year best for ("5K").
    pub bests: Vec<String>,
}

fn km(metres: f64) -> String {
    format!("{:.1} km", metres / 1000.0)
}

fn hours(seconds: f64) -> String {
    let minutes = (seconds / 60.0).round() as i64;
    format!("{}h{:02}", minutes / 60, minutes % 60)
}

fn days_between(earlier: &str, later: &str) -> Option<i64> {
    let a = Date::parse(earlier)?;
    let b = Date::parse(later)?;
    Some(a.days_until(b))
}

/// The longest run in the `days` days before `day` (not counting `day`).
pub fn longest_before(runs: &[Run], day: &str, days: i64) -> Option<f64> {
    runs.iter()
        .filter(|run| days_between(&run.date, day).is_some_and(|gap| gap >= 1 && gap <= days))
        .map(|run| run.distance_m)
        .fold(None, |best: Option<f64>, value| {
            Some(best.map_or(value, |best| best.max(value)))
        })
}

/// The runs before `run` in the `days` days before its date.
fn prior<'a>(runs: &'a [Run], run: &Run, days: i64) -> Vec<&'a Run> {
    runs.iter()
        .filter(|other| other.id != run.id)
        .filter(|other| {
            days_between(&other.date, &run.date)
                .is_some_and(|gap| (gap >= 1 && gap <= days) || (gap == 0 && other.id < run.id))
        })
        .collect()
}

/// What one run meant, against the athlete's own recent running.
///
/// `runs` is the history (it may include the run itself; it is skipped).
/// `recovery` is the night before the run, with the baselines it is read
/// against.
pub fn run_story(run: &Run, runs: &[Run], recovery: Option<&RecoveryRead>) -> RunStory {
    let mut story: Vec<String> = Vec::new();
    let mut chip: Option<(String, Tone)> = None;

    // The spike guard.
    let longest = longest_before(runs, &run.date, 30);
    let spike =
        longest.is_some_and(|longest| longest > 0.0 && run.distance_m > longest * SPIKE_RATIO);
    if let (true, Some(longest)) = (spike, longest) {
        let pct = (run.distance_m / longest - 1.0) * 100.0;
        story.push(format!(
            "{:.0}% longer than your longest run of the previous 30 days ({}). Jumps of more than 10% are linked to more overuse injuries, so keep the next few days easy.",
            pct,
            km(longest)
        ));
        chip = Some(("Big jump in distance".to_string(), "warn"));
    }

    // One-year bests by distance: the fastest average pace over at least
    // that distance.
    let year = prior(runs, run, 365);
    let mut bests: Vec<String> = Vec::new();
    if let Some(pace) = run.pace_s_km() {
        for (label, distance) in BEST_DISTANCES {
            if run.distance_m < distance * 0.995 {
                continue;
            }
            let rivals: Vec<f64> = year
                .iter()
                .filter(|other| other.distance_m >= distance * 0.995)
                .filter_map(|other| other.pace_s_km())
                .collect();
            if rivals.len() >= 2 && rivals.iter().all(|other| pace < *other) {
                bests.push(label.to_string());
            }
        }
    }
    if let Some(best) = bests.last() {
        story.push(format!(
            "Your fastest run of {best} or longer in the past year, by average pace."
        ));
        if chip.is_none() {
            chip = Some((format!("Fastest {best}+ this year"), "good"));
        }
    }

    // Against similar runs: within 30% of the distance, the last 90 days.
    let recent = prior(runs, run, 90);
    let similar: Vec<&&Run> = recent
        .iter()
        .filter(|other| {
            run.distance_m > 0.0 && (other.distance_m / run.distance_m - 1.0).abs() <= 0.3
        })
        .collect();
    let mut efficiency_delta_pct = None;
    let mut pace_delta_s_km = None;
    let mut hr_delta_bpm = None;
    if similar.len() >= 3 {
        let paces: Vec<f64> = similar
            .iter()
            .filter_map(|other| other.pace_s_km())
            .collect();
        if let (Some(pace), Some(usual)) = (run.pace_s_km(), median(&paces)) {
            pace_delta_s_km = Some(round_to(pace - usual, 0));
        }
        let hrs: Vec<f64> = similar.iter().filter_map(|other| other.avg_hr).collect();
        if let (Some(hr), Some(usual)) = (run.avg_hr, median(&hrs)) {
            hr_delta_bpm = Some(round_to(hr - usual, 0));
        }
        let effs: Vec<f64> = similar
            .iter()
            .filter_map(|other| other.efficiency())
            .collect();
        if let (Some(eff), Some(usual)) = (run.efficiency(), median(&effs)) {
            if effs.len() >= 3 {
                // A percentage against a baseline that is nearly zero is an
                // artefact of the division, not a change in the athlete, so
                // it is not shown as one — and when the readings cannot
                // carry a percentage, the story says nothing about effort
                // rather than claiming the run was "in line".
                if let Some(delta) = efficiency_change(eff, usual) {
                    efficiency_delta_pct = Some(round_to(delta, 1));
                    if delta >= 4.0 {
                        story.push(format!(
                            "More speed per heartbeat than your usual run of this length ({:+.0}%): {}.",
                            delta,
                            pace_hr_phrase(pace_delta_s_km, hr_delta_bpm)
                        ));
                        if chip.is_none() {
                            chip = Some(("Fitter than usual".to_string(), "good"));
                        }
                    } else if delta <= -5.0 {
                        let why = recovery.and_then(RecoveryRead::likely_cause);
                        story.push(format!(
                            "Harder than your usual run of this length ({:+.0}%): {}.{}",
                            delta,
                            pace_hr_phrase(pace_delta_s_km, hr_delta_bpm),
                            why.map(|why| format!(" {why}")).unwrap_or_else(|| {
                                " Heat, hills or a noisy wrist reading move this too, so one run says little."
                                    .to_string()
                            })
                        ));
                        if chip.is_none() {
                            chip = Some(("Harder than usual".to_string(), "neutral"));
                        }
                    } else {
                        // Pace and heart rate are medians of their own, so
                        // quoting them here can read "in line" next to "10 s/km
                        // faster"; the efficiency is the one honest number.
                        story.push("In line with your usual effort for this distance.".to_string());
                    }
                }
            }
        }
    }

    // Distance against the usual.
    let distances: Vec<f64> = recent.iter().map(|other| other.distance_m).collect();
    if distances.len() >= 4 {
        if let Some(usual) = median(&distances) {
            if run.distance_m > usual * 1.4 && !spike {
                story.push(format!("Longer than your usual {} run.", km(usual)));
                if chip.is_none() {
                    chip = Some(("Longer than usual".to_string(), "neutral"));
                }
            } else if run.distance_m < usual * 0.6 && chip.is_none() {
                chip = Some(("Short one".to_string(), "neutral"));
            }
        }
    }

    if story.is_empty() {
        if recent.len() < 4 {
            story.push("Not enough recent runs yet to compare this one with.".to_string());
        } else {
            story.push("A usual run for you. Nothing stood out.".to_string());
        }
    }

    let (chip, tone) = chip.unwrap_or_else(|| ("Usual run".to_string(), "neutral"));
    RunStory {
        id: run.id,
        chip,
        tone: tone.to_string(),
        story,
        spike,
        longest_prior_30d_m: longest,
        similar_runs: similar.len(),
        efficiency_delta_pct,
        pace_delta_s_km,
        hr_delta_bpm,
        bests,
    }
}

fn pace_hr_phrase(pace: Option<f64>, hr: Option<f64>) -> String {
    let pace_part = pace.map(|delta| {
        if delta.abs() < 3.0 {
            "the same pace".to_string()
        } else if delta < 0.0 {
            format!("{:.0} s/km faster", -delta)
        } else {
            format!("{:.0} s/km slower", delta)
        }
    });
    let hr_part = hr.map(|delta| {
        if delta.abs() < 2.0 {
            "the same heart rate".to_string()
        } else if delta < 0.0 {
            format!("{:.0} bpm lower heart rate", -delta)
        } else {
            format!("{:.0} bpm higher heart rate", delta)
        }
    });
    match (pace_part, hr_part) {
        (Some(pace), Some(hr)) => format!("{pace} at {hr}"),
        (Some(pace), None) => pace,
        (None, Some(hr)) => hr,
        (None, None) => "no pace or heart rate to compare".to_string(),
    }
}

/// One recovery signal against its normal range.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RecoverySignal {
    /// `sleep`, `hrv` or `resting_hr`.
    pub key: String,
    /// Inside the athlete's own normal range.
    pub normal: bool,
}

/// How far outside its normal range a signal is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Concern {
    /// Just outside the range: worth easing for, not worth a day off.
    Mild,
    /// Well outside the range: worth taking seriously.
    Marked,
}

/// One recovery signal that is outside its normal range, and by how much.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecoveryConcern {
    /// `sleep`, `hrv` or `resting_hr`.
    pub key: String,
    /// The sentence the runner reads: "HRV 54 ms vs your usual 61".
    pub detail: String,
    /// How far outside the normal range it is.
    pub concern: Concern,
}

/// How far below the baseline sleep has to be before the night counts as off.
const SLEEP_OFF_SECONDS: f64 = 3600.0;
/// How far below the baseline sleep makes it marked rather than mild.
const SLEEP_MARKED_SECONDS: f64 = 7200.0;
/// How far below the baseline HRV has to be before the night counts as off.
const HRV_OFF_RATIO: f64 = 0.85;
/// How far below the baseline HRV makes it marked rather than mild.
const HRV_MARKED_RATIO: f64 = 0.75;
/// How far above the baseline resting heart rate has to be before the night
/// counts as off.
const RHR_OFF_BPM: f64 = 5.0;
/// How far above the baseline resting heart rate makes it marked rather than
/// mild.
const RHR_MARKED_BPM: f64 = 10.0;

/// A night of recovery read against the athlete's 30-day baselines.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RecoveryRead {
    /// The night, `YYYY-MM-DD`.
    pub date: String,
    /// The night's sleep, in seconds.
    pub sleep_s: Option<f64>,
    /// The 30-day median sleep, in seconds.
    pub sleep_baseline_s: Option<f64>,
    /// The overnight HRV.
    pub hrv_ms: Option<f64>,
    /// The 30-day mean HRV.
    pub hrv_baseline_ms: Option<f64>,
    /// The resting heart rate.
    pub resting_hr: Option<f64>,
    /// The 30-day mean resting heart rate.
    pub resting_hr_baseline: Option<f64>,
}

impl RecoveryRead {
    /// Read one night against the 30 nights before it.
    pub fn from_history(rows: &[Recovery], day: &str) -> Option<RecoveryRead> {
        let row = rows.iter().find(|row| row.date == day)?;
        let window: Vec<&Recovery> = rows
            .iter()
            .filter(|other| {
                days_between(&other.date, day).is_some_and(|gap| (1..=30).contains(&gap))
            })
            .collect();
        let pick = |f: fn(&Recovery) -> Option<f64>| -> Vec<f64> {
            window
                .iter()
                .filter_map(|row| f(row))
                .filter(|v| *v > 0.0)
                .collect()
        };
        let mean = |values: Vec<f64>| -> Option<f64> {
            if values.len() < 7 {
                None
            } else {
                Some(values.iter().sum::<f64>() / values.len() as f64)
            }
        };
        let sleeps = pick(|row| row.sleep_s);
        Some(RecoveryRead {
            date: day.to_string(),
            sleep_s: row.sleep_s.filter(|v| *v > 0.0),
            sleep_baseline_s: if sleeps.len() >= 7 {
                median(&sleeps)
            } else {
                None
            },
            hrv_ms: row.hrv_ms.filter(|v| *v > 0.0),
            hrv_baseline_ms: mean(pick(|row| row.hrv_ms)),
            resting_hr: row.resting_hr.filter(|v| *v > 0.0),
            resting_hr_baseline: mean(pick(|row| row.resting_hr)),
        })
    }

    /// The signals that are off, as short phrases, most telling first.
    pub fn flags(&self) -> Vec<String> {
        self.concerns()
            .into_iter()
            .map(|concern| concern.detail)
            .collect()
    }

    /// The signals that are outside their normal range, and by how much.
    ///
    /// The thresholds that decide *whether* a signal is off are the ones
    /// the app has always used. What is new is the grade: a signal that is
    /// a little outside its range is a different thing from one that is
    /// well outside it, and the advice cannot be the same for both.
    pub fn concerns(&self) -> Vec<RecoveryConcern> {
        let mut out = Vec::new();
        if let (Some(rhr), Some(base)) = (self.resting_hr, self.resting_hr_baseline) {
            let over = rhr - base;
            if over >= RHR_OFF_BPM {
                out.push(RecoveryConcern {
                    key: "resting_hr".to_string(),
                    detail: format!("resting heart rate {:.0} vs your usual {:.0}", rhr, base),
                    concern: if over >= RHR_MARKED_BPM {
                        Concern::Marked
                    } else {
                        Concern::Mild
                    },
                });
            }
        }
        if let (Some(sleep), Some(base)) = (self.sleep_s, self.sleep_baseline_s) {
            let short = base - sleep;
            if short >= SLEEP_OFF_SECONDS {
                out.push(RecoveryConcern {
                    key: "sleep".to_string(),
                    detail: format!("slept {} vs your usual {}", hours(sleep), hours(base)),
                    concern: if short >= SLEEP_MARKED_SECONDS {
                        Concern::Marked
                    } else {
                        Concern::Mild
                    },
                });
            }
        }
        if let (Some(hrv), Some(base)) = (self.hrv_ms, self.hrv_baseline_ms) {
            if hrv < base * HRV_OFF_RATIO {
                out.push(RecoveryConcern {
                    key: "hrv".to_string(),
                    detail: format!("HRV {:.0} ms vs your usual {:.0}", hrv, base),
                    concern: if hrv < base * HRV_MARKED_RATIO {
                        Concern::Marked
                    } else {
                        Concern::Mild
                    },
                });
            }
        }
        out
    }

    /// Each readable signal and whether it is inside its normal range. The
    /// one definition of "normal": Body reads it from the API instead of
    /// re-deriving the thresholds, so Today and Body cannot drift apart.
    pub fn signals(&self) -> Vec<RecoverySignal> {
        let mut out = Vec::new();
        if let (Some(sleep), Some(base)) = (self.sleep_s, self.sleep_baseline_s) {
            out.push(RecoverySignal {
                key: "sleep".to_string(),
                normal: sleep >= base - 3600.0,
            });
        }
        if let (Some(hrv), Some(base)) = (self.hrv_ms, self.hrv_baseline_ms) {
            out.push(RecoverySignal {
                key: "hrv".to_string(),
                normal: hrv >= base * 0.85,
            });
        }
        if let (Some(rhr), Some(base)) = (self.resting_hr, self.resting_hr_baseline) {
            out.push(RecoverySignal {
                key: "resting_hr".to_string(),
                normal: rhr < base + 5.0,
            });
        }
        out
    }

    /// How many of sleep, HRV and resting HR are inside their normal range,
    /// and how many could be read.
    pub fn normal_count(&self) -> (usize, usize) {
        let mut read = 0;
        let mut normal = 0;
        if let (Some(sleep), Some(base)) = (self.sleep_s, self.sleep_baseline_s) {
            read += 1;
            if sleep >= base - 3600.0 {
                normal += 1;
            }
        }
        if let (Some(hrv), Some(base)) = (self.hrv_ms, self.hrv_baseline_ms) {
            read += 1;
            if hrv >= base * 0.85 {
                normal += 1;
            }
        }
        if let (Some(rhr), Some(base)) = (self.resting_hr, self.resting_hr_baseline) {
            read += 1;
            if rhr < base + 5.0 {
                normal += 1;
            }
        }
        (normal, read)
    }

    fn likely_cause(&self) -> Option<String> {
        let flags = self.flags();
        let first = flags.first()?;
        let mut sentence = first.clone();
        if let Some(first_char) = sentence.get(..1) {
            sentence = format!("{}{}", first_char.to_uppercase(), &sentence[1..]);
        }
        Some(format!(
            "The night before: {sentence}, which likely played a part."
        ))
    }
}

/// One day of the current week.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeekDay {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The running distance that day, in metres.
    pub distance_m: f64,
    /// Whether there was a run.
    pub ran: bool,
    /// Whether the day is in the future.
    pub future: bool,
}

/// The week so far.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeekSoFar {
    /// Monday to Sunday.
    pub days: Vec<WeekDay>,
    /// The week's running distance so far, in metres.
    pub distance_m: f64,
    /// The week's runs so far.
    pub runs: usize,
    /// The athlete's usual week: the median of the last eight complete weeks
    /// that had a run, in metres. `None` before there are four such weeks.
    pub usual_distance_m: Option<f64>,
}

/// Today's call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Today {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// `empty`, `done`, `easy`, `go`, `nudge` or `comeback`.
    pub state: String,
    /// The one call.
    pub headline: String,
    /// The one reason, when there is one.
    pub reason: Option<String>,
    /// The one guardrail, when there is one.
    pub guardrail: Option<String>,
    /// The longest run of the last 30 days, in metres.
    pub longest_30d_m: Option<f64>,
    /// The spike limit: the longest of the last 30 days plus 10%, in metres.
    pub spike_limit_m: Option<f64>,
    /// Days since the last run, when there was one.
    pub days_since_last_run: Option<i64>,
    /// The usual gap between runs, in days.
    pub usual_gap_days: Option<f64>,
    /// The latest run and what it meant.
    pub last_run: Option<LastRun>,
    /// The week so far.
    pub week: WeekSoFar,
    /// Last night against the 30-day baselines: normal signals, read signals.
    pub recovery_normal: Option<(usize, usize)>,
    /// Each recovery signal read last night, normal or not.
    pub recovery_signals: Vec<RecoverySignal>,
    /// The graded advice about last night, when a signal is outside its
    /// normal range.
    ///
    /// **One sentence, one source.** Today shows it as the reason and Body
    /// shows it under the count, so the two screens cannot disagree about
    /// how serious the night was.
    pub recovery_advice: Option<String>,
    /// Today's session on a written plan, reconciled with the call.
    pub plan: Option<PlanLine>,
}

/// The session a written plan has for today, as Today needs it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Planned {
    /// The session's title, or its type.
    pub label: String,
    /// `easy`, `tempo`, `interval`, `long`, `race`, `recovery`, `rest`.
    pub session_type: String,
    /// The target distance, in metres.
    pub target_distance_m: Option<f64>,
}

/// The plan, in one line under the call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanLine {
    /// The planned session.
    pub session: Planned,
    /// What to do with it today.
    pub line: String,
}

/// One line reconciling today's call with the planned session. The call
/// says *how hard* (recovery and load); the plan says *what*. They never
/// disagree silently: on an easy day a hard session is softened or moved,
/// and a planned distance past the spike limit is capped.
pub fn plan_line(
    state: &str,
    planned: &Planned,
    spike_limit_m: Option<f64>,
    call_distance_m: Option<f64>,
) -> String {
    let what = match planned.target_distance_m {
        Some(distance) if distance > 0.0 => format!("{}, {}", planned.label, km(distance)),
        _ => planned.label.clone(),
    };
    // Normalise the stored type so a hand-written or third-party plan's
    // "Intervals" or "long_run" reads the same as the canonical "interval".
    let kind = crate::plan::canonical_session_type(&planned.session_type);
    if kind == "rest" {
        return if state == "done" {
            "Your plan said rest. Today's run was extra; keep tomorrow easy.".to_string()
        } else {
            "Your plan says rest. Take it.".to_string()
        };
    }
    let hard = matches!(kind.as_str(), "tempo" | "interval" | "race" | "long");
    // A session whose kind is not known is not a kind of session. The
    // line says what the plan says and stops there: it does not call it an
    // easy run, and it does not call it hard.
    if kind == "unknown" {
        return match state {
            "done" => format!("Your plan had {what} today."),
            _ => format!("Your plan says {what}. It does not say what kind of session it is."),
        };
    }
    match state {
        "done" => format!("Your plan had {what} today."),
        "easy" if hard => format!(
            "Your plan says {what}. Recovery says go easy: keep it gentle, or swap it for an easy run."
        ),
        "easy" => format!("Your plan says {what}. Keep it easy, and shorter if you feel flat."),
        "comeback" => format!(
            "Your plan says {what}. After the break, run the short easy run above instead."
        ),
        // The call is a short one, so the line must not say "run it as
        // written": the two have to agree about what today is.
        "nudge" => match call_distance_m {
            Some(distance) => format!(
                "Your plan says {what}. Today is a short one — {} is enough.",
                km(distance)
            ),
            None => format!("Your plan says {what}. Today is a short one."),
        },
        _ => match (planned.target_distance_m, spike_limit_m) {
            (Some(target), Some(limit)) if target > limit => format!(
                "Your plan says {what}. That is past your spike limit: cap it at {}.",
                km(limit)
            ),
            _ => format!("Your plan says {what}. Nothing says otherwise: run it as written."),
        },
    }
}

/// The latest run, with its story.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LastRun {
    /// The run.
    pub run: Run,
    /// What it meant.
    pub story: RunStory,
}

/// The usual gap between runs over the last 90 days, in days.
fn usual_gap(runs: &[Run], today: &str) -> Option<f64> {
    let mut dates: Vec<Date> = runs
        .iter()
        .filter(|run| days_between(&run.date, today).is_some_and(|gap| (0..=90).contains(&gap)))
        .filter_map(Run::day)
        .collect();
    dates.sort();
    dates.dedup();
    let gaps: Vec<f64> = dates
        .windows(2)
        .map(|pair| pair[0].days_until(pair[1]) as f64)
        .collect();
    if gaps.len() < 3 {
        None
    } else {
        median(&gaps)
    }
}

/// The current week, Monday to Sunday, and the usual week.
pub fn week_so_far(runs: &[Run], today: &str) -> WeekSoFar {
    let day = Date::parse(today);
    let monday = day.map(|day| day.iso_week_start());
    let mut days = Vec::new();
    let mut total = 0.0;
    let mut count = 0;
    if let (Some(day), Some(monday)) = (day, monday) {
        for offset in 0..7 {
            let date = monday.add_days(offset);
            let key = date.to_string();
            let on: Vec<&Run> = runs.iter().filter(|run| run.date == key).collect();
            let distance: f64 = on.iter().map(|run| run.distance_m).fold(0.0, |a, b| a + b);
            if date <= day {
                total += distance;
                count += on.len();
            }
            days.push(WeekDay {
                date: key,
                distance_m: distance,
                ran: !on.is_empty(),
                future: date > day,
            });
        }
    }
    let usual = monday.and_then(|monday| {
        let mut weeks: Vec<f64> = Vec::new();
        for back in 1..=8 {
            let start = monday.add_days(-7 * back);
            let end = start.add_days(6);
            let distance: f64 = runs
                .iter()
                .filter(|run| run.day().is_some_and(|d| d >= start && d <= end))
                .map(|run| run.distance_m)
                .fold(0.0, |a, b| a + b);
            if distance > 0.0 {
                weeks.push(distance);
            }
        }
        if weeks.len() >= 4 {
            median(&weeks)
        } else {
            None
        }
    });
    WeekSoFar {
        days,
        distance_m: total,
        runs: count,
        usual_distance_m: usual,
    }
}

/// Today's call: one headline, one reason, one guardrail.
///
/// `recovery` is last night read against the 30-day baselines.
pub fn today(
    today: &str,
    runs: &[Run],
    recovery: Option<&RecoveryRead>,
    planned: Option<&Planned>,
) -> Today {
    let recovery_signals = recovery.map(RecoveryRead::signals).unwrap_or_default();
    let week = week_so_far(runs, today);
    let mut sorted: Vec<&Run> = runs.iter().collect();
    sorted.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));
    let last = sorted
        .iter()
        .rev()
        .find(|run| days_between(&run.date, today).is_some_and(|gap| gap >= 0))
        .copied();
    let recovery_normal = recovery
        .map(RecoveryRead::normal_count)
        .filter(|(_, read)| *read > 0);

    let Some(last) = last else {
        return Today {
            date: today.to_string(),
            state: "empty".to_string(),
            headline: "Connect Garmin to see your runs.".to_string(),
            reason: None,
            guardrail: None,
            longest_30d_m: None,
            spike_limit_m: None,
            days_since_last_run: None,
            usual_gap_days: None,
            last_run: None,
            week,
            recovery_normal,
            recovery_signals,
            recovery_advice: None,
            plan: None,
        };
    };

    // The longest of the last 30 days *including* today, for the guardrail.
    let longest_30d = runs
        .iter()
        .filter(|run| days_between(&run.date, today).is_some_and(|gap| (0..=30).contains(&gap)))
        .map(|run| run.distance_m)
        .fold(None, |best: Option<f64>, value| {
            Some(best.map_or(value, |b| b.max(value)))
        });
    let spike_limit = longest_30d.map(|longest| longest * SPIKE_RATIO);
    let gap = days_between(&last.date, today);
    let usual = usual_gap(runs, today);
    let story = run_story(last, runs, None);
    let last_run = Some(LastRun {
        run: last.clone(),
        story: story.clone(),
    });

    let recent: Vec<f64> = runs
        .iter()
        .filter(|run| days_between(&run.date, today).is_some_and(|g| (0..=60).contains(&g)))
        .map(|run| run.distance_m)
        .collect();
    let typical = median(&recent);
    let concerns = recovery.map(RecoveryRead::concerns).unwrap_or_default();

    // The distance this call is about: what the plan says, or what the
    // nudge suggests, or the athlete's usual run when there is no plan.
    //
    // The distance guardrail is only worth the runner's attention when the
    // run is near the line. A warning about a distance that is nowhere near
    // the limit is noise, and noise on the first screen is noise the runner
    // learns to skip.
    let planned_distance_m = planned
        .and_then(|session| session.target_distance_m)
        .filter(|distance| *distance > 0.0);
    let nudge_m = typical
        .map(|t| (t * 0.6).clamp(3000.0, 6000.0))
        .unwrap_or(5000.0);
    let nudged = gap.is_some_and(|days| usual.is_some_and(|u| days >= 3 && days as f64 >= u * 2.0));
    let call_distance_m = planned_distance_m.or(if nudged { Some(nudge_m) } else { typical });

    let guardrail_text = match (spike_limit, call_distance_m) {
        (Some(limit), Some(distance)) if distance >= limit * GUARDRAIL_NEAR_RATIO => Some(
            format!(
                "Runs past {} (your longest in 30 days, plus 10%) were linked to more overuse injuries in a large study. Under it is not proven safe, and nudging up run after run adds up.",
                km(limit)
            ),
        ),
        _ => None,
    };

    let (state, headline, reason, guardrail) = if gap == Some(0) {
        let ran: f64 = runs
            .iter()
            .filter(|run| run.date == today)
            .map(|run| run.distance_m)
            .fold(0.0, |a, b| a + b);
        let tomorrow = if story.spike
            || story.tone == "warn"
            || concerns.iter().any(|c| c.concern == Concern::Marked)
        {
            "Tomorrow: rest or 30 minutes very easy."
        } else if !concerns.is_empty() {
            "Tomorrow: keep it easy."
        } else {
            "Tomorrow: whatever's next. There's nothing to make up."
        };
        (
            "done",
            format!("Done for today: {} logged.", km(ran)),
            story.story.first().cloned(),
            Some(tomorrow.to_string()),
        )
    } else if gap.is_some_and(|gap| gap >= COMEBACK_DAYS) {
        let days = gap.unwrap_or(0);
        let cap = typical
            .map(|t| (t * 0.5).clamp(2000.0, 5000.0))
            .unwrap_or(3000.0);
        (
            "comeback",
            "Welcome back. Start with 20 to 30 minutes, easy.".to_string(),
            Some(format!(
                "{days} days since your last run. The first week back is about rhythm, not pace."
            )),
            Some(format!(
                "Keep the first runs under {}, and skip pace targets this week.",
                km(cap)
            )),
        )
    } else if !concerns.is_empty() {
        let (headline, advice) = recovery_advice(&concerns);
        (
            "easy",
            headline,
            Some(format!("Last night: {advice}")),
            guardrail_text,
        )
    } else if let (Some(days), Some(usual)) = (gap, usual) {
        if nudged {
            (
                "nudge",
                format!("A short easy {} today keeps the rhythm going.", km(nudge_m)),
                Some(format!(
                    "{days} days since your last run; you usually run every {} days. No catching up needed.",
                    usual.round()
                )),
                guardrail_text,
            )
        } else {
            go_state(typical, recovery_normal, guardrail_text, planned_distance_m)
        }
    } else {
        go_state(typical, recovery_normal, guardrail_text, planned_distance_m)
    };

    Today {
        date: today.to_string(),
        state: state.to_string(),
        headline,
        reason,
        guardrail,
        longest_30d_m: longest_30d,
        spike_limit_m: spike_limit,
        days_since_last_run: gap,
        usual_gap_days: usual,
        last_run,
        week,
        recovery_normal,
        recovery_signals,
        recovery_advice: (!concerns.is_empty()).then(|| recovery_note(&concerns)),
        plan: planned.map(|session| PlanLine {
            session: session.clone(),
            line: plan_line(state, session, spike_limit, call_distance_m),
        }),
    }
}

/// The advice for a night whose signals are outside their normal range.
///
/// The advice is graded, because the signals are not the same thing: one
/// signal that is just outside its range is not several that are well
/// outside theirs. And it is not the athlete's recovery data that decides
/// whether they run — they are. So a mild signal eases the plan and leaves
/// the day to them; a marked one asks for an easy run or a day off; several
/// say take the day, with the athlete's own feeling able to call it down
/// to an easy run.
fn recovery_advice(concerns: &[RecoveryConcern]) -> (String, String) {
    let advice = recovery_note(concerns);
    let headline = if concerns.len() >= 2 {
        "Take the day, or make it very easy."
    } else if concerns
        .iter()
        .any(|concern| concern.concern == Concern::Marked)
    {
        "Go easy today, or take the day."
    } else {
        "Keep it easy today."
    };
    (headline.to_string(), advice)
}

/// The graded advice, in one sentence.
///
/// **One sentence, one source.** Today and Body both show it, so the two
/// screens cannot say different things about the same night: the advice is
/// written here, once, and both read it.
fn recovery_note(concerns: &[RecoveryConcern]) -> String {
    let details: Vec<String> = concerns
        .iter()
        .map(|concern| concern.detail.clone())
        .collect();
    let joined = details.join("; ");
    let marked = concerns
        .iter()
        .filter(|concern| concern.concern == Concern::Marked)
        .count();
    if concerns.len() >= 2 {
        return format!(
            "{joined}. {} signals outside your normal range — if you feel fine, an easy run is enough.",
            concerns.len()
        );
    }
    if marked == 1 {
        return format!(
            "{joined}. Well outside your normal range — and how you feel has the last word."
        );
    }
    format!("{joined}. Just outside your normal range — if you feel off too, take the day.")
}

fn go_state(
    typical: Option<f64>,
    normal: Option<(usize, usize)>,
    guardrail: Option<String>,
    planned_distance_m: Option<f64>,
) -> (&'static str, String, Option<String>, Option<String>) {
    // The headline quotes the distance the runner will actually do. With a
    // plan on the sheet, that is the plan's distance: quoting the usual
    // next to a plan that says otherwise is two numbers, and the runner
    // has to guess which one the card means.
    let headline = match (planned_distance_m, typical) {
        (Some(planned), _) => format!("Good day for a run. Your plan says {}.", km(planned)),
        (None, Some(typical)) => {
            format!("Good day for a run. Your usual is about {}.", km(typical))
        }
        (None, None) => "Good day for a run.".to_string(),
    };
    let reason = normal.map(|(normal, read)| {
        if normal == read {
            "Sleep and recovery are in your normal range.".to_string()
        } else {
            format!("{normal} of {read} recovery signals in your normal range.")
        }
    });
    ("go", headline, reason, guardrail)
}

/// The fitness headline: efficiency now against six to twelve weeks ago.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fitness {
    /// The headline sentence.
    pub headline: String,
    /// The change in efficiency, as a percentage, when it means something.
    ///
    /// `None` when the readings cannot carry a percentage — the baseline
    /// was not a real run's worth of efficiency, or the change is far
    /// outside anything a body does.
    pub delta_pct: Option<f64>,
    /// The runs in the recent window.
    pub recent_runs: usize,
    /// The runs in the earlier window.
    pub earlier_runs: usize,
    /// The pace at the athlete's typical heart rate now, in s/km.
    pub pace_now_s_km: Option<f64>,
    /// The pace at the same heart rate in the earlier window, in s/km.
    pub pace_then_s_km: Option<f64>,
    /// The typical heart rate the paces are read at.
    pub at_hr: Option<f64>,
}

/// One week of consistency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsistencyWeek {
    /// The Monday.
    pub start: String,
    /// Runs that week.
    pub runs: usize,
    /// Distance that week, in metres.
    pub distance_m: f64,
    /// `active`, `grace`, `gap` or `current`.
    pub status: String,
}

/// Week-by-week consistency, forgiving by design.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Consistency {
    /// The weekly run count that makes a week active.
    pub runs_per_week: usize,
    /// The current run of active weeks (grace weeks included).
    pub current_weeks: usize,
    /// The longest run of active weeks ever.
    pub best_weeks: usize,
    /// Whether a grace week is available now.
    pub grace_available: bool,
    /// The last 52 weeks, oldest first.
    pub weeks: Vec<ConsistencyWeek>,
    /// Active weeks this calendar year.
    pub active_weeks_this_year: usize,
}

/// A best run at a distance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BestRun {
    /// "5K", "10K", "Half", "Marathon".
    pub label: String,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The scope: `all_time` or `this_year`.
    pub scope: String,
    /// The run.
    pub activity_id: i64,
    /// Its date.
    pub date: String,
    /// Its average pace, in s/km.
    pub pace_s_km: f64,
    /// Its distance, in metres.
    pub run_distance_m: f64,
}

/// Lifetime milestones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Milestones {
    /// Total running distance, in metres.
    pub total_m: f64,
    /// Total runs.
    pub total_runs: usize,
    /// The first run's date.
    pub runner_since: Option<String>,
    /// The longest run ever, in metres.
    pub longest_m: f64,
    /// The distance tiers reached, in km.
    pub tiers_reached_km: Vec<f64>,
    /// The next tier, in km.
    pub next_tier_km: Option<f64>,
    /// Distance this calendar year, in metres.
    pub this_year_m: f64,
    /// Runs this calendar year.
    pub this_year_runs: usize,
}

/// A race-time estimate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RaceEstimate {
    /// "5K", "10K", "Half", "Marathon".
    pub label: String,
    /// The faster end of the range, in seconds.
    pub low_s: f64,
    /// The slower end of the range, in seconds.
    pub high_s: f64,
}

/// The Progress screen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    /// The fitness headline, when there are enough runs with heart rate.
    pub fitness: Option<Fitness>,
    /// Week-by-week consistency.
    pub consistency: Consistency,
    /// Best runs by distance.
    pub bests: Vec<BestRun>,
    /// Lifetime milestones.
    pub milestones: Milestones,
    /// Race-time ranges from the best recent runs.
    pub race_estimates: Vec<RaceEstimate>,
    /// What the race estimates are based on.
    pub race_basis: Option<String>,
}

fn in_window(run: &Run, today: &str, from_days: i64, to_days: i64) -> bool {
    days_between(&run.date, today).is_some_and(|gap| gap >= from_days && gap <= to_days)
}

/// Efficiency now (last 6 weeks) against 6 to 12 weeks ago, with the pace at
/// the athlete's typical heart rate in both windows.
pub fn fitness(runs: &[Run], today: &str) -> Option<Fitness> {
    let recent: Vec<&Run> = runs.iter().filter(|r| in_window(r, today, 0, 41)).collect();
    let earlier: Vec<&Run> = runs
        .iter()
        .filter(|r| in_window(r, today, 42, 83))
        .collect();
    let eff = |set: &[&Run]| -> Vec<f64> { set.iter().filter_map(|r| r.efficiency()).collect() };
    let now = eff(&recent);
    let then = eff(&earlier);
    if now.len() < 4 || then.len() < 4 {
        return None;
    }
    let now_m = median(&now)?;
    let then_m = median(&then)?;
    // The earlier stretch is the baseline, so it has to be a run's worth of
    // efficiency for the comparison to mean anything. A baseline that is
    // nearly zero is not a slow runner; it is missing data, and a pace or
    // a percentage read against it would be a number with no runner in it.
    if then_m < EFFICIENCY_FLOOR {
        return None;
    }
    let delta = efficiency_change(now_m, then_m);
    let hrs: Vec<f64> = recent
        .iter()
        .chain(earlier.iter())
        .filter_map(|r| r.avg_hr)
        .collect();
    let at_hr = median(&hrs).map(|hr| round_to(hr, 0));
    let pace_at = |eff: f64| at_hr.map(|hr| round_to(1000.0 / (eff * hr) * 60.0, 0));
    let pace_now = pace_at(now_m);
    let pace_then = pace_at(then_m);
    let headline = match (pace_now, pace_then, at_hr) {
        (Some(now), Some(then), Some(hr)) if (then - now).abs() >= 3.0 => {
            if now < then {
                format!(
                    "Fitter than six weeks ago: {:.0} s/km faster at {:.0} bpm.",
                    then - now,
                    hr
                )
            } else {
                format!(
                    "{:.0} s/km slower at {:.0} bpm than six weeks ago. Heat, illness or a training break all do this.",
                    now - then,
                    hr
                )
            }
        }
        _ => "About as fit as six weeks ago: the same pace for the same heart rate.".to_string(),
    };
    Some(Fitness {
        headline,
        delta_pct: delta.map(|value| round_to(value, 1)),
        recent_runs: now.len(),
        earlier_runs: then.len(),
        pace_now_s_km: pace_now,
        pace_then_s_km: pace_then,
        at_hr,
    })
}

/// Week-by-week consistency over the last 52 weeks, with one grace week in
/// any six.
pub fn consistency(runs: &[Run], today: &str) -> Consistency {
    let Some(day) = Date::parse(today) else {
        return Consistency {
            runs_per_week: 1,
            current_weeks: 0,
            best_weeks: 0,
            grace_available: true,
            weeks: Vec::new(),
            active_weeks_this_year: 0,
        };
    };
    let this_monday = day.iso_week_start();
    let first = runs.iter().filter_map(Run::day).min();
    // All weeks from the first run (at most ~20 years) to this one.
    let total_weeks = first
        .map(|first| (first.iso_week_start().days_until(this_monday) / 7 + 1).clamp(1, 1100))
        .unwrap_or(1);
    let mut weeks: Vec<(Date, usize, f64)> = Vec::new();
    for back in (0..total_weeks).rev() {
        let start = this_monday.add_days(-7 * back);
        let end = start.add_days(6);
        let on: Vec<&Run> = runs
            .iter()
            .filter(|run| run.day().is_some_and(|d| d >= start && d <= end))
            .collect();
        weeks.push((
            start,
            on.len(),
            on.iter().map(|r| r.distance_m).fold(0.0, |a, b| a + b),
        ));
    }
    // The bar: two runs a week, or one for an athlete whose usual is one.
    let counts: Vec<f64> = weeks
        .iter()
        .rev()
        .skip(1)
        .take(12)
        .filter(|(_, n, _)| *n > 0)
        .map(|(_, n, _)| *n as f64)
        .collect();
    let runs_per_week = if median(&counts).unwrap_or(2.0) >= 2.0 {
        2
    } else {
        1
    };

    // Walk forward, marking each week.
    let mut statuses: Vec<&'static str> = Vec::new();
    let mut streak = 0usize;
    let mut best = 0usize;
    let mut last_grace: Option<usize> = None;
    let last_index = weeks.len().saturating_sub(1);
    for (index, (_, n, _)) in weeks.iter().enumerate() {
        let current = index == last_index;
        if *n >= runs_per_week {
            statuses.push("active");
            streak += 1;
        } else if current {
            // The week is not over: it never breaks anything yet.
            statuses.push("current");
        } else if streak > 0 && last_grace.map_or(true, |g| index - g >= 6) {
            statuses.push("grace");
            last_grace = Some(index);
            streak += 1;
        } else {
            statuses.push("gap");
            streak = 0;
        }
        best = best.max(streak);
    }
    let grace_available = last_grace.map_or(true, |g| last_index - g >= 6);
    let year = day.to_string().get(..4).unwrap_or("").to_string();
    let active_weeks_this_year = weeks
        .iter()
        .zip(statuses.iter())
        .filter(|((start, _, _), status)| {
            start.add_days(6).to_string().starts_with(&year) && **status == "active"
        })
        .count();
    let keep = weeks.len().saturating_sub(52);
    let out: Vec<ConsistencyWeek> = weeks
        .iter()
        .zip(statuses.iter())
        .skip(keep)
        .map(|((start, n, d), status)| ConsistencyWeek {
            start: start.to_string(),
            runs: *n,
            distance_m: *d,
            status: status.to_string(),
        })
        .collect();
    Consistency {
        runs_per_week,
        current_weeks: streak,
        best_weeks: best,
        grace_available,
        weeks: out,
        active_weeks_this_year,
    }
}

/// The fastest average pace over at least each best-run distance, all time
/// and this year.
pub fn bests(runs: &[Run], today: &str) -> Vec<BestRun> {
    let year = today.get(..4).unwrap_or("");
    let mut out = Vec::new();
    for (label, distance) in BEST_DISTANCES {
        for scope in ["all_time", "this_year"] {
            let best = runs
                .iter()
                .filter(|run| run.distance_m >= distance * 0.995)
                .filter(|run| scope == "all_time" || run.date.starts_with(year))
                .filter_map(|run| run.pace_s_km().map(|pace| (run, pace)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            if let Some((run, pace)) = best {
                out.push(BestRun {
                    label: label.to_string(),
                    distance_m: distance,
                    scope: scope.to_string(),
                    activity_id: run.id,
                    date: run.date.clone(),
                    pace_s_km: round_to(pace, 1),
                    run_distance_m: run.distance_m,
                });
            }
        }
    }
    out
}

/// Lifetime milestones.
pub fn milestones(runs: &[Run], today: &str) -> Milestones {
    let total: f64 = runs.iter().map(|r| r.distance_m).fold(0.0, |a, b| a + b);
    let total_km = total / 1000.0;
    let year = today.get(..4).unwrap_or("");
    let this_year: Vec<&Run> = runs.iter().filter(|r| r.date.starts_with(year)).collect();
    Milestones {
        total_m: total,
        total_runs: runs.len(),
        runner_since: runs.iter().map(|r| r.date.clone()).min(),
        longest_m: runs.iter().map(|r| r.distance_m).fold(0.0, f64::max),
        tiers_reached_km: DISTANCE_TIERS_KM
            .iter()
            .copied()
            .filter(|tier| total_km >= *tier)
            .collect(),
        next_tier_km: DISTANCE_TIERS_KM
            .iter()
            .copied()
            .find(|tier| total_km < *tier),
        this_year_m: this_year
            .iter()
            .map(|r| r.distance_m)
            .fold(0.0, |a, b| a + b),
        this_year_runs: this_year.len(),
    }
}

/// Race-time ranges from the best run of 5 km or more in the last 90 days,
/// for distances up to 2.2 times that run (a 10K says nothing about a marathon),
/// by Riegel's formula (exponent 1.06), widened 4% on the slow side because
/// predictors run optimistic for anyone who has not trained the distance.
pub fn race_estimates(runs: &[Run], today: &str) -> (Vec<RaceEstimate>, Option<String>) {
    let basis = runs
        .iter()
        .filter(|run| in_window(run, today, 0, 90) && run.distance_m >= 5000.0)
        .filter_map(|run| run.pace_s_km().map(|pace| (run, pace)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let Some((run, _)) = basis else {
        return (Vec::new(), None);
    };
    let estimates = BEST_DISTANCES
        .iter()
        .filter(|(_, distance)| *distance <= run.distance_m * 2.2)
        .map(|(label, distance)| {
            let time = run.moving_time_s * (distance / run.distance_m).powf(1.06);
            RaceEstimate {
                label: label.to_string(),
                low_s: round_to(time, 0),
                high_s: round_to(time * 1.04, 0),
            }
        })
        .collect();
    (
        estimates,
        Some(format!(
            "From your {} on {}, your fastest run of 5 km or more in the last 90 days.",
            km(run.distance_m),
            run.date
        )),
    )
}

/// Everything the Progress screen shows.
pub fn progress(runs: &[Run], today: &str) -> Progress {
    let (race_estimates, race_basis) = race_estimates(runs, today);
    Progress {
        fitness: fitness(runs, today),
        consistency: consistency(runs, today),
        bests: bests(runs, today),
        milestones: milestones(runs, today),
        race_estimates,
        race_basis,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn run(id: i64, date: &str, km: f64, pace: f64, hr: Option<f64>) -> Run {
        Run {
            id,
            name: "Run".to_string(),
            date: date.to_string(),
            distance_m: km * 1000.0,
            moving_time_s: km * pace,
            avg_hr: hr,
        }
    }

    fn history(today: &str, count: i64, every: i64, km: f64) -> Vec<Run> {
        let day = Date::parse(today).unwrap();
        (1..=count)
            .map(|i| {
                run(
                    i,
                    &day.add_days(-every * i).to_string(),
                    km,
                    360.0,
                    Some(145.0),
                )
            })
            .collect()
    }

    #[test]
    fn a_run_past_the_30_day_longest_plus_ten_percent_is_a_spike() {
        let mut runs = history("2026-10-30", 10, 2, 8.0);
        let big = run(100, "2026-10-30", 9.6, 360.0, Some(145.0));
        runs.push(big.clone());
        let story = run_story(&big, &runs, None);
        assert!(story.spike);
        assert_eq!(story.tone, "warn");
        assert_eq!(story.chip, "Big jump in distance");
        assert!(story.story[0].starts_with("20% longer"));
    }

    #[test]
    fn a_run_inside_the_limit_is_not_a_spike() {
        let mut runs = history("2026-10-30", 10, 2, 8.0);
        let fine = run(100, "2026-10-30", 8.7, 360.0, Some(145.0));
        runs.push(fine.clone());
        assert!(!run_story(&fine, &runs, None).spike);
    }

    #[test]
    fn more_speed_at_the_same_heart_rate_reads_fitter() {
        let mut runs = history("2026-10-30", 8, 3, 8.0);
        let quick = run(100, "2026-10-30", 8.0, 340.0, Some(145.0));
        runs.push(quick.clone());
        let story = run_story(&quick, &runs, None);
        // A year-best outranks "fitter" as the chip; the story says both.
        assert_eq!(story.chip, "Fastest 5K+ this year");
        assert!(story
            .story
            .iter()
            .any(|line| line.starts_with("More speed per heartbeat")));
        assert_eq!(story.pace_delta_s_km, Some(-20.0));
    }

    #[test]
    fn a_harder_run_names_a_short_night_as_the_likely_cause() {
        let mut runs = history("2026-10-30", 8, 3, 8.0);
        let slow = run(100, "2026-10-30", 8.0, 390.0, Some(150.0));
        runs.push(slow.clone());
        let night = RecoveryRead {
            date: "2026-10-30".into(),
            sleep_s: Some(5.0 * 3600.0),
            sleep_baseline_s: Some(7.5 * 3600.0),
            ..RecoveryRead::default()
        };
        let story = run_story(&slow, &runs, Some(&night));
        assert_eq!(story.chip, "Harder than usual");
        assert!(story.story[0].contains("Slept 5h00 vs your usual 7h30"));
    }

    #[test]
    fn today_after_a_run_is_done_not_complete_the_session() {
        let mut runs = history("2026-10-30", 10, 2, 8.0);
        runs.push(run(100, "2026-10-30", 8.0, 360.0, Some(145.0)));
        let call = today("2026-10-30", &runs, None, None);
        assert_eq!(call.state, "done");
        assert!(call.headline.starts_with("Done for today: 8.0 km"));
        assert!(call.guardrail.unwrap().contains("nothing to make up"));
    }

    #[test]
    fn two_weeks_off_is_a_comeback_with_a_cap() {
        let runs = history("2026-10-16", 10, 2, 8.0);
        let call = today("2026-10-30", &runs, None, None);
        assert_eq!(call.state, "comeback");
        assert!(call.guardrail.unwrap().contains("4.0 km"));
    }

    #[test]
    fn a_long_gap_for_this_runner_is_a_gentle_nudge() {
        let runs = history("2026-10-25", 10, 2, 8.0);
        let call = today("2026-10-30", &runs, None, None);
        assert_eq!(call.state, "nudge");
        assert!(call.headline.contains("4.8 km"));
        assert!(call.reason.unwrap().contains("No catching up"));
    }

    #[test]
    fn a_short_night_softens_the_call() {
        let runs = history("2026-10-29", 10, 2, 8.0);
        let night = RecoveryRead {
            date: "2026-10-30".into(),
            resting_hr: Some(58.0),
            resting_hr_baseline: Some(51.0),
            ..RecoveryRead::default()
        };
        let call = today("2026-10-30", &runs, Some(&night), None);
        assert_eq!(call.state, "easy");
        assert!(call.guardrail.unwrap().contains("8.8 km"));
    }

    /// A history of short runs with one long one in it: the usual run is
    /// 5 km and the spike limit is 22 km.
    fn short_runs_with_one_long_one() -> Vec<Run> {
        let mut runs = history("2026-10-29", 10, 2, 5.0);
        runs.push(run(100, "2026-10-20", 20.0, 360.0, Some(145.0)));
        runs
    }

    #[test]
    fn the_distance_guardrail_is_silent_when_the_run_is_nowhere_near_the_line() {
        let runs = short_runs_with_one_long_one();
        let short = Planned {
            label: "Easy 5".to_string(),
            session_type: "easy".to_string(),
            target_distance_m: Some(5000.0),
        };
        let call = today("2026-10-30", &runs, None, Some(&short));
        assert!(
            call.guardrail.is_none(),
            "a 5 km run is not a distance risk: {:?}",
            call.guardrail
        );

        // The same history, with a run near the line.
        let near = Planned {
            label: "Long 21".to_string(),
            session_type: "long".to_string(),
            target_distance_m: Some(21_000.0),
        };
        let call = today("2026-10-30", &runs, None, Some(&near));
        assert!(call.guardrail.unwrap().contains("22.0 km"));
    }

    #[test]
    fn the_distance_guardrail_stays_silent_with_no_plan_and_a_short_usual_run() {
        let runs = short_runs_with_one_long_one();
        let call = today("2026-10-30", &runs, None, None);
        assert!(call.guardrail.is_none());
    }

    #[test]
    fn one_mild_signal_eases_the_plan_and_leaves_the_day_to_the_runner() {
        let runs = history("2026-10-29", 10, 2, 8.0);
        let night = RecoveryRead {
            date: "2026-10-30".into(),
            resting_hr: Some(57.0),
            resting_hr_baseline: Some(51.0),
            ..RecoveryRead::default()
        };
        let call = today("2026-10-30", &runs, Some(&night), None);
        assert_eq!(call.state, "easy");
        assert_eq!(call.headline, "Keep it easy today.");
        let reason = call.reason.unwrap();
        assert!(reason.contains("resting heart rate 57 vs your usual 51"));
        assert!(reason.contains("if you feel off too"));
    }

    #[test]
    fn one_marked_signal_asks_for_an_easy_run_or_a_day_off() {
        let runs = history("2026-10-29", 10, 2, 8.0);
        let night = RecoveryRead {
            date: "2026-10-30".into(),
            resting_hr: Some(62.0),
            resting_hr_baseline: Some(51.0),
            ..RecoveryRead::default()
        };
        let call = today("2026-10-30", &runs, Some(&night), None);
        assert_eq!(call.state, "easy");
        assert_eq!(call.headline, "Go easy today, or take the day.");
        let reason = call.reason.unwrap();
        assert!(reason.contains("Well outside your normal range"));
        assert!(reason.contains("how you feel has the last word"));
    }

    #[test]
    fn several_signals_say_take_the_day() {
        let runs = history("2026-10-29", 10, 2, 8.0);
        let night = RecoveryRead {
            date: "2026-10-30".into(),
            resting_hr: Some(62.0),
            resting_hr_baseline: Some(51.0),
            sleep_s: Some(4.0 * 3600.0),
            sleep_baseline_s: Some(7.5 * 3600.0),
            ..RecoveryRead::default()
        };
        let call = today("2026-10-30", &runs, Some(&night), None);
        assert_eq!(call.state, "easy");
        assert_eq!(call.headline, "Take the day, or make it very easy.");
        let reason = call.reason.unwrap();
        assert!(reason.contains("2 signals outside your normal range"));
        assert!(reason.contains("if you feel fine"));
    }

    #[test]
    fn a_tomorrow_after_a_mild_signal_is_only_easy() {
        let mut runs = history("2026-10-30", 10, 2, 8.0);
        runs.push(run(100, "2026-10-30", 8.0, 360.0, Some(145.0)));
        let night = RecoveryRead {
            date: "2026-10-30".into(),
            resting_hr: Some(57.0),
            resting_hr_baseline: Some(51.0),
            ..RecoveryRead::default()
        };
        let call = today("2026-10-30", &runs, Some(&night), None);
        assert_eq!(call.state, "done");
        assert!(call.guardrail.unwrap().contains("keep it easy"));
    }

    #[test]
    fn a_percentage_against_a_near_zero_baseline_is_not_a_fitness_change() {
        // The denominator is nearly zero: the percentage would be enormous
        // and would mean nothing.
        assert_eq!(efficiency_change(1.0, 0.01), None);
        assert_eq!(efficiency_change(1.0, 0.0), None);
        // A change a body makes.
        let change = efficiency_change(1.2, 1.0).expect("a real change is a change");
        assert!((change - 20.0).abs() < 1e-9, "the change is 20%: {change}");
        // A change no body makes: the readings are wrong, not the athlete.
        assert_eq!(efficiency_change(10.0, 0.5), None);
    }

    #[test]
    fn a_fitness_trend_against_a_near_zero_baseline_is_not_read() {
        let mut runs = Vec::new();
        // The earlier stretch: the same pace with a heart rate that makes
        // the efficiency unreadable.
        for index in 1..=6 {
            runs.push(run(
                index,
                &Date::parse("2026-10-30")
                    .unwrap()
                    .add_days(-42 * index)
                    .to_string(),
                8.0,
                3600.0,
                Some(145.0),
            ));
        }
        // The recent stretch: a normal run.
        for index in 1..=6 {
            runs.push(run(
                100 + index,
                &Date::parse("2026-10-30")
                    .unwrap()
                    .add_days(-2 * index)
                    .to_string(),
                8.0,
                360.0,
                Some(145.0),
            ));
        }
        assert!(fitness(&runs, "2026-10-30").is_none());
    }

    #[test]
    fn the_run_story_does_not_claim_a_percentage_it_cannot_carry() {
        let mut runs = history("2026-10-30", 8, 3, 8.0);
        // The usual runs of this length have an efficiency that cannot
        // carry a percentage.
        for run in runs.iter_mut() {
            run.moving_time_s = 8.0 * 3600.0;
        }
        let quick = run(100, "2026-10-30", 8.0, 340.0, Some(145.0));
        runs.push(quick.clone());
        let story = run_story(&quick, &runs, None);
        assert_eq!(story.efficiency_delta_pct, None);
        assert!(
            !story
                .story
                .iter()
                .any(|line| line.contains("speed per heartbeat")),
            "the story must not quote a percentage it cannot carry: {:?}",
            story.story
        );
        assert!(
            !story
                .story
                .iter()
                .any(|line| line.contains("In line with your usual effort")),
            "an unreadable comparison is not 'in line': {:?}",
            story.story
        );
    }

    #[test]
    fn the_headline_quotes_the_plan_when_the_plan_says_otherwise() {
        // The usual run is 8 km and the plan says 10: the card must not
        // quote both, or the runner has to guess which one it means.
        let runs = history("2026-10-29", 10, 2, 8.0);
        let planned = Planned {
            label: "Easy".to_string(),
            session_type: "easy".to_string(),
            target_distance_m: Some(10_000.0),
        };
        let call = today("2026-10-30", &runs, None, Some(&planned));
        assert!(
            call.headline.contains("Your plan says 10.0 km"),
            "the headline was {}",
            call.headline
        );
        assert!(
            !call.headline.contains("usual"),
            "the headline quoted the usual too: {}",
            call.headline
        );

        // With no plan, the usual is what the runner will do.
        let call = today("2026-10-30", &runs, None, None);
        assert!(call.headline.contains("Your usual is about 8.0 km"));
    }

    #[test]
    fn the_plan_line_agrees_with_a_short_call() {
        // The nudge says "a short easy 4.8 km today"; the line below it
        // must not say "run it as written".
        let runs = history("2026-10-25", 10, 2, 8.0);
        let planned = Planned {
            label: "Easy".to_string(),
            session_type: "easy".to_string(),
            target_distance_m: Some(8000.0),
        };
        let call = today("2026-10-30", &runs, None, Some(&planned));
        assert_eq!(call.state, "nudge");
        let line = call.plan.unwrap().line;
        assert!(line.contains("Today is a short one"), "{line}");
        assert!(!line.contains("run it as written"), "{line}");
    }

    #[test]
    fn the_advice_is_one_sentence_for_both_screens() {
        // Today and Body read the same sentence, so they cannot disagree.
        let runs = history("2026-10-29", 10, 2, 8.0);
        let night = RecoveryRead {
            date: "2026-10-30".into(),
            resting_hr: Some(57.0),
            resting_hr_baseline: Some(51.0),
            ..RecoveryRead::default()
        };
        let call = today("2026-10-30", &runs, Some(&night), None);
        let advice = call.recovery_advice.expect("there is advice");
        assert!(
            call.reason
                .as_deref()
                .unwrap_or_default()
                .ends_with(&advice),
            "the reason and the advice are different sentences: {:?} / {advice}",
            call.reason
        );
    }

    #[test]
    fn one_quiet_week_is_grace_not_a_reset() {
        // Two runs a week for eight weeks, then a quiet week, then two more.
        let mut runs = Vec::new();
        let monday = Date::parse("2026-10-26").unwrap();
        let mut id = 0;
        for back in 1..=10 {
            if back == 3 {
                continue;
            }
            for day in [0, 3] {
                id += 1;
                runs.push(run(
                    id,
                    &monday.add_days(-7 * back + day).to_string(),
                    6.0,
                    360.0,
                    None,
                ));
            }
        }
        let c = consistency(&runs, "2026-10-30");
        assert_eq!(c.runs_per_week, 2);
        assert_eq!(c.current_weeks, 10);
        assert!(!c.grace_available);
        let statuses: Vec<&str> = c.weeks.iter().map(|w| w.status.as_str()).collect();
        assert_eq!(statuses.last(), Some(&"current"));
        assert!(statuses.contains(&"grace"));
    }

    #[test]
    fn the_fitness_headline_reads_pace_at_the_same_heart_rate() {
        let mut runs = Vec::new();
        let day = Date::parse("2026-10-30").unwrap();
        for i in 0..6 {
            runs.push(run(
                i,
                &day.add_days(-3 - i * 5).to_string(),
                8.0,
                350.0,
                Some(145.0),
            ));
            runs.push(run(
                100 + i,
                &day.add_days(-50 - i * 5).to_string(),
                8.0,
                370.0,
                Some(145.0),
            ));
        }
        let fit = fitness(&runs, "2026-10-30").unwrap();
        assert!(fit
            .headline
            .starts_with("Fitter than six weeks ago: 20 s/km faster at 145 bpm"));
    }

    #[test]
    fn milestones_and_bests_come_from_the_runs() {
        let runs = vec![
            run(1, "2025-03-01", 5.0, 330.0, None),
            run(2, "2026-02-01", 10.0, 340.0, None),
            run(3, "2026-09-01", 5.2, 320.0, None),
        ];
        let m = milestones(&runs, "2026-10-30");
        assert_eq!(m.runner_since.as_deref(), Some("2025-03-01"));
        assert_eq!(m.next_tier_km, Some(50.0));
        let b = bests(&runs, "2026-10-30");
        let five = b
            .iter()
            .find(|b| b.label == "5K" && b.scope == "all_time")
            .unwrap();
        assert_eq!(five.activity_id, 3);
    }

    #[test]
    fn plan_line_reconciles_the_call_with_the_plan() {
        let intervals = Planned {
            label: "Intervals".to_string(),
            session_type: "interval".to_string(),
            target_distance_m: Some(8000.0),
        };
        let easy = plan_line("easy", &intervals, Some(17600.0), None);
        assert!(easy.contains("swap it for an easy run"), "{easy}");
        let go = plan_line("go", &intervals, Some(17600.0), None);
        assert!(go.contains("run it as written"), "{go}");
        let long = Planned {
            label: "Long run".to_string(),
            session_type: "long".to_string(),
            target_distance_m: Some(22000.0),
        };
        let capped = plan_line("go", &long, Some(17600.0), None);
        assert!(capped.contains("cap it at 17.6 km"), "{capped}");
    }

    #[test]
    fn a_session_with_an_unknown_kind_is_not_called_easy() {
        let unknown = Planned {
            label: "Zona 3".to_string(),
            session_type: "unknown".to_string(),
            target_distance_m: Some(8000.0),
        };
        let line = plan_line("easy", &unknown, Some(17_600.0), None);
        assert!(
            !line.contains("Keep it easy"),
            "an unknown kind is not an easy session: {line}"
        );
        assert!(
            line.contains("does not say what kind of session it is"),
            "{line}"
        );
        let done = plan_line("done", &unknown, Some(17_600.0), None);
        assert!(
            done.contains("Your plan had Zona 3, 8.0 km today."),
            "{done}"
        );
    }

    #[test]
    fn today_carries_the_plan_line() {
        let runs = history("2026-10-30", 20, 2, 8.0);
        let intervals = Planned {
            label: "Intervals".to_string(),
            session_type: "interval".to_string(),
            target_distance_m: Some(8000.0),
        };
        let call = today("2026-10-30", &runs, None, Some(&intervals));
        let plan = call.plan.unwrap();
        assert!(
            plan.line.starts_with("Your plan says Intervals, 8.0 km."),
            "{}",
            plan.line
        );
        assert!(today("2026-10-30", &runs, None, None).plan.is_none());
    }

    #[test]
    fn plan_line_reads_plural_and_variant_session_types_as_hard() {
        for raw in [
            "intervals",
            "Intervals",
            "interval",
            "long_run",
            "threshold",
        ] {
            let planned = Planned {
                label: "6 x 800 m".to_string(),
                session_type: raw.to_string(),
                target_distance_m: Some(9000.0),
            };
            let line = plan_line("easy", &planned, Some(17_600.0), None);
            assert!(line.contains("swap it for an easy run"), "{raw}: {line}");
        }
    }
}
