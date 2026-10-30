//! The FIT importer: parse a file, segment it, rebuild the laps and streams.
//!
//! This is a direct port of `backend/app/ingest/fit_importer.py`. The
//! reference implementation is the specification; the golden files under
//! `backend/tests/golden/golden/` are the proof that this port agrees with
//! it.
//!
//! The three Garmin quirks the port exists to get right:
//!
//! 1. **Running cadence is stored halved.** A watch showing 170 spm writes
//!    85. The correction applies to running sports only — a cycling cadence
//!    of 90 rpm stays 90.
//! 2. **Positions are semicircles**, not degrees:
//!    `degrees = round(semicircles * (180 / 2^31), 7)`.
//! 3. **Lap speed is sometimes absent.** It is derived from the distance and
//!    the duration only when the watch did not record it, and the running
//!    pass replaces it with the speed of the running samples.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::decoder::{self, Message, FIT_EPOCH_UNIX};
use crate::error::FitError;
use crate::profile::{self, FieldKind};
use crate::round::{mean, round_to, trunc_to_int};

/// The largest FIT file the importer will read.
pub const MAX_FIT_INPUT_BYTES: usize = 20 * 1024 * 1024;

/// The largest number of record messages the importer will read.
pub const MAX_FIT_RECORDS: usize = 250_000;

/// How a record is classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SegmentKind {
    /// The athlete was running.
    Run,
    /// The athlete was walking.
    Walk,
    /// The athlete was standing still.
    Stand,
}

impl SegmentKind {
    /// The kind's name, as the contract spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            SegmentKind::Run => "run",
            SegmentKind::Walk => "walk",
            SegmentKind::Stand => "stand",
        }
    }
}

/// One contiguous stretch of samples of the same kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    /// What the athlete was doing.
    pub kind: SegmentKind,
    /// Elapsed time of the first sample of the segment.
    pub start_s: f64,
    /// Elapsed time of the **last** sample of the segment.
    pub end_s: f64,
    /// How many samples the segment holds.
    pub samples: i64,
    /// `end_s - start_s`.
    pub duration_s: f64,
}

/// One record message, as the analysis sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    /// The sample's position in the record stream.
    pub index: i64,
    /// Seconds since the first record.
    pub elapsed_s: f64,
    /// How the sample was classified.
    pub kind: SegmentKind,
    /// The cumulative distance at the sample, in metres.
    pub distance_m: Option<f64>,
    /// The speed at the sample, in metres per second.
    pub speed_ms: Option<f64>,
    /// The cadence **as stored in the file** — halved for running sports.
    pub stored_cadence: Option<f64>,
    /// The cadence in steps per minute, halved where the sport calls for it.
    pub cadence_spm: Option<f64>,
    /// The heart rate at the sample, in beats per minute.
    pub heart_rate_bpm: Option<f64>,
    /// The altitude at the sample, in metres.
    pub altitude_m: Option<f64>,
}

/// A time series over the running samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stream {
    /// Seconds since the first record, one per value.
    pub seconds: Vec<i64>,
    /// The values themselves.
    pub values: Vec<f64>,
    /// The unit of the values.
    pub unit: String,
}

/// A lap, rebuilt over the running samples of the lap's window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lap {
    /// The lap's number, starting at one.
    pub lap_index: i64,
    /// The lap's full distance, in metres (carried over from the file).
    pub distance_m: f64,
    /// The running time of the lap, in seconds.
    pub duration_s: f64,
    /// The speed over the running samples, in metres per second.
    pub avg_speed_ms: Option<f64>,
    /// The fastest speed recorded in the lap, in metres per second.
    pub max_speed_ms: Option<f64>,
    /// The mean heart rate over the running samples.
    pub avg_hr: Option<f64>,
    /// The highest heart rate over the running samples.
    pub max_hr: Option<f64>,
    /// The mean cadence over the running samples, in steps per minute.
    pub avg_cadence: Option<f64>,
    /// The highest cadence over the running samples, in steps per minute.
    pub max_cadence: Option<f64>,
    /// Always `None`; the file does not carry it.
    pub stride_length_m: Option<f64>,
    /// The ascent recorded in the file, in metres.
    pub elevation_gain_m: Option<f64>,
    /// The descent recorded in the file, in metres.
    pub elevation_loss_m: Option<f64>,
    /// Always `None`; the file does not carry it.
    pub start_elevation_m: Option<f64>,
    /// Always `None`; the file does not carry it.
    pub end_elevation_m: Option<f64>,
    /// The lap's span in the activity's elapsed time.
    pub elapsed_s: f64,
    /// The distance covered while running, in metres.
    pub running_distance_m: Option<f64>,
    /// The pace over the running samples, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
}

/// A parsed FIT activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedActivity {
    /// The file's serial number, or the file name when the file has none.
    pub external_id: String,
    /// Always `"garmin_fit"`.
    pub source: String,
    /// The file name, with `_` and `-` turned into spaces.
    pub name: String,
    /// The normalised sport, e.g. `"running"`.
    pub sport_type: String,
    /// The raw FIT sport, e.g. `"running"` or `"cycling"`.
    pub sport: Option<String>,
    /// The start of the activity.
    pub start_time: DateTime<Utc>,
    /// The UTC date of the start, `YYYY-MM-DD`.
    pub local_date: String,
    /// The English weekday name of the start.
    pub day_of_week: String,
    /// The total distance, in metres.
    pub distance_m: f64,
    /// The moving time, in seconds.
    pub moving_time_s: f64,
    /// The elapsed time, in seconds.
    pub elapsed_time_s: f64,
    /// The ascent, in metres.
    pub elevation_gain_m: f64,
    /// The mean heart rate, in beats per minute.
    pub avg_hr: Option<f64>,
    /// The highest heart rate, in beats per minute.
    pub max_hr: Option<f64>,
    /// The calories burned, in kilocalories.
    pub calories: Option<f64>,
    /// The mean cadence **as stored in the file**, in steps per minute.
    pub avg_cadence: Option<f64>,
    /// The highest cadence **as stored in the file**, in steps per minute.
    pub max_cadence: Option<f64>,
    /// The mean speed, in metres per second.
    pub avg_speed_ms: Option<f64>,
    /// The highest speed, in metres per second.
    pub max_speed_ms: Option<f64>,
    /// Never set by the FIT importer.
    pub perceived_exertion: Option<f64>,
    /// The watch that recorded the file.
    pub device: Option<String>,
    /// Never set by the FIT importer.
    pub gear: Option<String>,
    /// Never set by the FIT importer.
    pub is_trail: bool,
    /// Never set by the FIT importer.
    pub is_commute: bool,
    /// The time spent running, in seconds.
    pub running_time_s: Option<f64>,
    /// The time spent walking, in seconds.
    pub walking_time_s: Option<f64>,
    /// The time spent standing still, in seconds.
    pub stopped_time_s: Option<f64>,
    /// The distance covered while running, in metres.
    pub running_distance_m: Option<f64>,
    /// The GPS track, in degrees.
    pub geo_points: Option<Vec<[f64; 2]>>,
    /// The laps, rebuilt over the running samples.
    pub laps: Vec<Lap>,
    /// The cumulative elapsed time at the end of each lap.
    pub lap_bounds: Vec<f64>,
    /// The per-second series over the running samples.
    pub streams: BTreeMap<String, Stream>,
    /// The run / walk / stand segments.
    pub segments: Vec<Segment>,
    /// One row per record message.
    pub samples: Vec<Sample>,
}

/// Parse a FIT file into a canonical activity.
///
/// `filename` is used for the activity's name and, when the file carries no
/// serial number, for its external id.
pub fn parse_fit(bytes: &[u8], filename: &str) -> Result<ParsedActivity, FitError> {
    if bytes.len() > MAX_FIT_INPUT_BYTES {
        return Err(FitError::TooLarge {
            size: bytes.len(),
            limit: MAX_FIT_INPUT_BYTES,
        });
    }
    let mut importer = Importer::new(filename);
    decoder::for_each_message(bytes, |message| importer.visit(message))?;
    importer.finish()
}

// ---------------------------------------------------------------- the file

/// One record message, decoded.
#[derive(Debug, Clone)]
struct RecordRow {
    ts: f64,
    distance: Option<f64>,
    speed: Option<f64>,
    cadence: Option<f64>,
    heart_rate: Option<f64>,
    altitude: Option<f64>,
}

/// A record message with its classification.
#[derive(Debug, Clone)]
struct ClassifiedRow {
    elapsed: f64,
    distance: Option<f64>,
    speed: Option<f64>,
    cadence: Option<f64>,
    heart_rate: Option<f64>,
    altitude: Option<f64>,
    kind: SegmentKind,
}

/// The raw lap message.
#[derive(Debug, Clone, Default)]
struct RawLap {
    total_elapsed_time: Option<f64>,
    total_timer_time: Option<f64>,
    total_distance: Option<f64>,
    avg_speed: Option<f64>,
    max_speed: Option<f64>,
    avg_heart_rate: Option<f64>,
    max_heart_rate: Option<f64>,
    avg_cadence: Option<f64>,
    max_cadence: Option<f64>,
    total_ascent: Option<f64>,
    total_descent: Option<f64>,
}

/// The raw session message.
#[derive(Debug, Clone, Default)]
struct RawSession {
    total_elapsed_time: Option<f64>,
    total_timer_time: Option<f64>,
    total_distance: Option<f64>,
    avg_speed: Option<f64>,
    max_speed: Option<f64>,
    avg_heart_rate: Option<f64>,
    max_heart_rate: Option<f64>,
    avg_cadence: Option<f64>,
    max_cadence: Option<f64>,
    total_calories: Option<f64>,
    total_ascent: Option<f64>,
}

/// The state built while walking the file's messages.
struct Importer<'a> {
    filename: &'a str,
    file_ids: Vec<String>,
    start_ts: Option<f64>,
    end_ts: Option<f64>,
    last_distance: f64,
    records: Vec<RecordRow>,
    geo: Vec<[f64; 2]>,
    laps: Vec<RawLap>,
    session: Option<RawSession>,
    sport: Option<String>,
    device: Option<String>,
    file_id_count: usize,
    record_count: usize,
    lap_count: usize,
    session_count: usize,
    device_info_count: usize,
}

impl<'a> Importer<'a> {
    fn new(filename: &'a str) -> Self {
        Importer {
            filename,
            file_ids: Vec::new(),
            start_ts: None,
            end_ts: None,
            last_distance: 0.0,
            records: Vec::new(),
            geo: Vec::new(),
            laps: Vec::new(),
            session: None,
            sport: None,
            device: None,
            file_id_count: 0,
            record_count: 0,
            lap_count: 0,
            session_count: 0,
            device_info_count: 0,
        }
    }

    fn visit(&mut self, message: Message) -> Result<(), FitError> {
        match message.mesg_num {
            profile::MESG_FILE_ID => {
                self.file_id_count += 1;
                if self.file_id_count > 100 {
                    return Err(FitError::TooManyMessages {
                        kind: "file_id",
                        limit: 100,
                    });
                }
                if let Some(serial) = number(&message, profile::MESG_FILE_ID, "serial_number") {
                    let serial = trunc_to_int(serial);
                    if serial != 0 {
                        self.file_ids.push(serial.to_string());
                    }
                }
            }
            profile::MESG_RECORD => {
                self.record_count += 1;
                if self.record_count > MAX_FIT_RECORDS {
                    return Err(FitError::TooManyMessages {
                        kind: "record",
                        limit: MAX_FIT_RECORDS,
                    });
                }
                self.visit_record(&message);
            }
            profile::MESG_LAP => {
                self.lap_count += 1;
                if self.lap_count > 10_000 {
                    return Err(FitError::TooManyMessages {
                        kind: "lap",
                        limit: 10_000,
                    });
                }
                self.laps.push(RawLap {
                    total_elapsed_time: number(&message, profile::MESG_LAP, "total_elapsed_time"),
                    total_timer_time: number(&message, profile::MESG_LAP, "total_timer_time"),
                    total_distance: number(&message, profile::MESG_LAP, "total_distance"),
                    avg_speed: number(&message, profile::MESG_LAP, "avg_speed"),
                    max_speed: number(&message, profile::MESG_LAP, "max_speed"),
                    avg_heart_rate: number(&message, profile::MESG_LAP, "avg_heart_rate"),
                    max_heart_rate: number(&message, profile::MESG_LAP, "max_heart_rate"),
                    avg_cadence: number(&message, profile::MESG_LAP, "avg_cadence"),
                    max_cadence: number(&message, profile::MESG_LAP, "max_cadence"),
                    total_ascent: number(&message, profile::MESG_LAP, "total_ascent"),
                    total_descent: number(&message, profile::MESG_LAP, "total_descent"),
                });
            }
            profile::MESG_SESSION => {
                self.session_count += 1;
                if self.session_count > 100 {
                    return Err(FitError::TooManyMessages {
                        kind: "session",
                        limit: 100,
                    });
                }
                // The reference reads the first session message and stops.
                if self.session.is_none() {
                    self.session = Some(RawSession {
                        total_elapsed_time: number(
                            &message,
                            profile::MESG_SESSION,
                            "total_elapsed_time",
                        ),
                        total_timer_time: number(
                            &message,
                            profile::MESG_SESSION,
                            "total_timer_time",
                        ),
                        total_distance: number(&message, profile::MESG_SESSION, "total_distance"),
                        avg_speed: number(&message, profile::MESG_SESSION, "avg_speed"),
                        max_speed: number(&message, profile::MESG_SESSION, "max_speed"),
                        avg_heart_rate: number(&message, profile::MESG_SESSION, "avg_heart_rate"),
                        max_heart_rate: number(&message, profile::MESG_SESSION, "max_heart_rate"),
                        avg_cadence: number(&message, profile::MESG_SESSION, "avg_cadence"),
                        max_cadence: number(&message, profile::MESG_SESSION, "max_cadence"),
                        total_calories: number(&message, profile::MESG_SESSION, "total_calories"),
                        total_ascent: number(&message, profile::MESG_SESSION, "total_ascent"),
                    });
                    if let Some(raw) = enum_number(&message, profile::MESG_SESSION, "sport") {
                        self.sport = Some(profile::sport_name(raw));
                    }
                }
            }
            profile::MESG_DEVICE_INFO => {
                self.device_info_count += 1;
                if self.device_info_count > 100 {
                    return Err(FitError::TooManyMessages {
                        kind: "device_info",
                        limit: 100,
                    });
                }
                // The reference reads the first device_info message and stops.
                if self.device.is_none() {
                    let name = text(&message, profile::MESG_DEVICE_INFO, "product_name")
                        .map(str::to_string)
                        .or_else(|| {
                            enum_number(&message, profile::MESG_DEVICE_INFO, "manufacturer")
                                .map(profile::manufacturer_name)
                        });
                    if let Some(name) = name {
                        if !name.is_empty() {
                            let rendered = enum_name(&name);
                            self.device = Some(if rendered.is_empty() { name } else { rendered });
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_record(&mut self, message: &Message) {
        let ts = number(message, profile::MESG_RECORD, "timestamp")
            .map(|raw| FIT_EPOCH_UNIX as f64 + raw)
            .or_else(|| {
                message
                    .compressed_timestamp
                    .map(|raw| FIT_EPOCH_UNIX as f64 + f64::from(raw))
            });
        if let Some(ts) = ts {
            if self.start_ts.is_none() {
                self.start_ts = Some(ts);
            }
            self.end_ts = Some(ts);
        }

        let raw_distance = number(message, profile::MESG_RECORD, "distance");
        if let Some(distance) = raw_distance {
            if (0.0..=10_000_000.0).contains(&distance) {
                self.last_distance = distance;
            }
        }

        let latitude = number(message, profile::MESG_RECORD, "position_lat");
        let longitude = number(message, profile::MESG_RECORD, "position_long");
        if let (Some(latitude), Some(longitude)) = (latitude, longitude) {
            if (-2_147_483_648.0..=2_147_483_648.0).contains(&latitude)
                && (-2_147_483_648.0..=2_147_483_648.0).contains(&longitude)
            {
                self.geo.push([
                    semicircles_to_degrees(latitude),
                    semicircles_to_degrees(longitude),
                ]);
            }
        }

        let heart_rate = in_range(
            number(message, profile::MESG_RECORD, "heart_rate"),
            0.0,
            300.0,
        );
        let cadence = in_range(number(message, profile::MESG_RECORD, "cadence"), 0.0, 300.0);

        // `speed` is preferred; `enhanced_speed` is the fallback.
        let speed = number(message, profile::MESG_RECORD, "speed")
            .or_else(|| number(message, profile::MESG_RECORD, "enhanced_speed"));
        let speed = in_range(speed, 0.0, 150.0);

        // `altitude` is preferred; `enhanced_altitude` is the fallback.
        let altitude = number(message, profile::MESG_RECORD, "altitude")
            .or_else(|| number(message, profile::MESG_RECORD, "enhanced_altitude"));
        let altitude = in_range(altitude, -1000.0, 10_000.0);

        let distance = match raw_distance {
            Some(distance) if (0.0..=10_000_000.0).contains(&distance) => Some(distance),
            _ => None,
        };

        self.records.push(RecordRow {
            ts: ts.unwrap_or(0.0),
            distance,
            speed,
            cadence,
            heart_rate,
            altitude,
        });
    }

    fn finish(self) -> Result<ParsedActivity, FitError> {
        let session = self.session.clone().unwrap_or_default();
        let sport = self.sport.clone();

        let mut activity = ParsedActivity {
            external_id: self
                .file_ids
                .first()
                .cloned()
                .unwrap_or_else(|| self.filename.to_string()),
            source: "garmin_fit".to_string(),
            name: activity_name(self.filename),
            sport_type: "running".to_string(),
            sport: sport.clone(),
            start_time: DateTime::<Utc>::UNIX_EPOCH,
            local_date: String::new(),
            day_of_week: String::new(),
            distance_m: 0.0,
            moving_time_s: 0.0,
            elapsed_time_s: 0.0,
            elevation_gain_m: 0.0,
            avg_hr: None,
            max_hr: None,
            calories: None,
            avg_cadence: None,
            max_cadence: None,
            avg_speed_ms: None,
            max_speed_ms: None,
            perceived_exertion: None,
            device: self.device.clone(),
            gear: None,
            is_trail: false,
            is_commute: false,
            running_time_s: None,
            walking_time_s: None,
            stopped_time_s: None,
            running_distance_m: None,
            geo_points: None,
            laps: Vec::new(),
            lap_bounds: Vec::new(),
            streams: BTreeMap::new(),
            segments: Vec::new(),
            samples: Vec::new(),
        };

        // ---------------------------------------------------------- totals

        activity.distance_m = match bounded(session.total_distance, 10_000_000.0) {
            Some(distance) if distance != 0.0 => distance,
            _ => self.last_distance,
        };

        let timer = bounded(session.total_timer_time, 604_800.0);
        let elapsed = bounded(session.total_elapsed_time, 604_800.0);
        let mut moving = match timer {
            Some(timer) if timer != 0.0 => Some(timer),
            _ => elapsed,
        };
        if moving.unwrap_or(0.0) == 0.0 {
            if let (Some(start), Some(end)) = (self.start_ts, self.end_ts) {
                if start != 0.0 && end != 0.0 {
                    moving = Some(end - start);
                }
            }
        }
        activity.moving_time_s = moving.unwrap_or(0.0);
        activity.elapsed_time_s = match elapsed {
            Some(elapsed) if elapsed != 0.0 => elapsed,
            _ => activity.moving_time_s,
        };
        activity.elevation_gain_m = bounded(session.total_ascent, 100_000.0).unwrap_or(0.0);
        activity.avg_hr =
            bounded(session.avg_heart_rate, 300.0).map(|value| trunc_to_int(value) as f64);
        activity.max_hr =
            bounded(session.max_heart_rate, 300.0).map(|value| trunc_to_int(value) as f64);
        activity.calories = bounded(session.total_calories, 1_000_000.0);
        // The activity's cadence is the session value **as stored** — it is not
        // halved, even though the laps' and the stream's are.
        activity.avg_cadence = bounded(session.avg_cadence, 300.0);
        activity.max_cadence = bounded(session.max_cadence, 300.0);
        activity.avg_speed_ms = bounded(session.avg_speed, 150.0);
        activity.max_speed_ms = bounded(session.max_speed, 150.0);

        if let Some(sport) = &sport {
            if !sport.is_empty() {
                activity.sport_type = normalise_sport(sport);
            }
        }

        if let Some(start) = self.start_ts {
            if start != 0.0 {
                if let Some(start_time) = DateTime::from_timestamp(start as i64, 0) {
                    activity.start_time = start_time;
                    activity.local_date = start_time.format("%Y-%m-%d").to_string();
                    activity.day_of_week = start_time.format("%A").to_string();
                }
            }
        }

        if !self.geo.is_empty() {
            activity.geo_points = Some(self.geo.clone());
        }

        // ----------------------------------------------------------- laps

        let mut cumulative_elapsed = 0.0;
        let mut laps: Vec<Lap> = Vec::with_capacity(self.laps.len());
        for (index, lap) in self.laps.iter().enumerate() {
            let lap_distance = bounded(lap.total_distance, 10_000_000.0).unwrap_or(0.0);
            let lap_duration = match bounded(lap.total_timer_time, 604_800.0) {
                Some(duration) if duration != 0.0 => duration,
                _ => bounded(lap.total_elapsed_time, 604_800.0).unwrap_or(0.0),
            };
            let mut lap_speed = bounded(lap.avg_speed, 150.0);
            if lap_speed.unwrap_or(0.0) == 0.0 && lap_distance != 0.0 && lap_duration != 0.0 {
                // Not every device records a lap's speed; it follows from the
                // distance and the time.
                let derived = lap_distance / lap_duration;
                lap_speed = if derived > 150.0 { None } else { Some(derived) };
            }
            let cadence = bounded(lap.avg_cadence, 300.0);
            let max_cadence = bounded(lap.max_cadence, 300.0);
            cumulative_elapsed += lap
                .total_elapsed_time
                .filter(|value| *value != 0.0)
                .or(lap.total_timer_time)
                .filter(|value| *value != 0.0)
                .unwrap_or(0.0);

            // The reference asks the lap for a `record_number`, which is not a
            // field of the FIT lap message, and falls back to the lap's
            // position in the file.
            let lap_index = index as i64 + 1;

            laps.push(Lap {
                lap_index,
                distance_m: lap_distance,
                duration_s: lap_duration,
                avg_speed_ms: lap_speed,
                max_speed_ms: bounded(lap.max_speed, 150.0),
                avg_hr: bounded(lap.avg_heart_rate, 300.0).map(|value| trunc_to_int(value) as f64),
                max_hr: bounded(lap.max_heart_rate, 300.0).map(|value| trunc_to_int(value) as f64),
                avg_cadence: cadence
                    .filter(|value| *value != 0.0)
                    .map(|value| running_cadence(value, sport.as_deref())),
                max_cadence: max_cadence
                    .filter(|value| *value != 0.0)
                    .map(|value| running_cadence(value, sport.as_deref())),
                stride_length_m: None,
                elevation_gain_m: bounded(lap.total_ascent, 100_000.0),
                elevation_loss_m: bounded(lap.total_descent, 100_000.0),
                start_elevation_m: None,
                end_elevation_m: None,
                elapsed_s: round_to(cumulative_elapsed, 3),
                running_distance_m: None,
                pace_s_per_km: None,
            });
        }
        activity.laps = laps;

        // ------------------------------------------------------- segments

        let (segments, classified, totals) = segment_records(&self.records, self.start_ts);
        activity.segments = segments;
        activity.running_time_s = Some(round_to(totals.run_seconds, 1));
        activity.walking_time_s = Some(round_to(totals.walk_seconds, 1));
        activity.stopped_time_s = Some(round_to(totals.stand_seconds, 1));
        activity.running_distance_m = Some(round_to(totals.run_metres, 1));
        activity.samples = classified
            .iter()
            .enumerate()
            .map(|(index, row)| Sample {
                index: index as i64,
                elapsed_s: row.elapsed,
                kind: row.kind,
                distance_m: row.distance,
                speed_ms: row.speed,
                stored_cadence: row.cadence,
                cadence_spm: row
                    .cadence
                    .map(|value| running_cadence(value, sport.as_deref())),
                heart_rate_bpm: row.heart_rate,
                altitude_m: row.altitude,
            })
            .collect();

        // -------------------------------------------------------- streams

        if let Some(start_ts) = self.start_ts {
            if start_ts != 0.0 {
                let running: Vec<&RecordRow> = self
                    .records
                    .iter()
                    .filter(|record| {
                        classify_sample(record.speed, record.cadence) == SegmentKind::Run
                    })
                    .collect();
                if !running.is_empty() {
                    let heart_rate: Vec<(f64, f64)> = running
                        .iter()
                        .filter_map(|record| record.heart_rate.map(|value| (record.ts, value)))
                        .collect();
                    activity
                        .streams
                        .insert("hr".to_string(), series(&heart_rate, start_ts, "bpm"));

                    let cadence: Vec<(f64, f64)> = running
                        .iter()
                        .filter_map(|record| {
                            record
                                .cadence
                                .map(|value| (record.ts, running_cadence(value, sport.as_deref())))
                        })
                        .collect();
                    activity
                        .streams
                        .insert("cadence".to_string(), series(&cadence, start_ts, "spm"));

                    let pace: Vec<(f64, f64)> = running
                        .iter()
                        .filter_map(|record| {
                            record
                                .speed
                                .filter(|speed| *speed > 0.1)
                                .map(|speed| (record.ts, 1000.0 / speed))
                        })
                        .collect();
                    if !pace.is_empty() {
                        activity
                            .streams
                            .insert("pace".to_string(), series(&pace, start_ts, "s/km"));
                    }

                    let distance: Vec<(f64, f64)> = running
                        .iter()
                        .filter_map(|record| record.distance.map(|value| (record.ts, value)))
                        .collect();
                    activity
                        .streams
                        .insert("distance".to_string(), series(&distance, start_ts, "m"));

                    let elevation: Vec<(f64, f64)> = running
                        .iter()
                        .filter_map(|record| record.altitude.map(|value| (record.ts, value)))
                        .collect();
                    if !elevation.is_empty() {
                        activity
                            .streams
                            .insert("elevation".to_string(), series(&elevation, start_ts, "m"));
                    }
                }
            }
        }

        // ------------------------------------------- running-only laps

        if !classified.is_empty() && !activity.laps.is_empty() {
            activity.laps = running_laps(activity.laps, &classified, sport.as_deref());
        }

        // ---------------------------------------------------- lap bounds

        if let Some(start_ts) = self.start_ts {
            if start_ts != 0.0 && !self.laps.is_empty() {
                let mut cursor = 0.0;
                let mut bounds = Vec::with_capacity(self.laps.len());
                for lap in &self.laps {
                    cursor += lap
                        .total_elapsed_time
                        .filter(|value| *value != 0.0)
                        .or(lap.total_timer_time)
                        .filter(|value| *value != 0.0)
                        .unwrap_or(0.0);
                    bounds.push(round_to(cursor, 3));
                }
                activity.lap_bounds = bounds;
            }
        }

        Ok(activity)
    }
}

// ------------------------------------------------------------- segmentation

/// The time and distance per classification.
#[derive(Debug, Default)]
struct Totals {
    run_seconds: f64,
    walk_seconds: f64,
    stand_seconds: f64,
    run_metres: f64,
}

/// Classify one record from its own speed and its own stored cadence.
///
/// The thresholds are deliberately conservative: running cadence is stored
/// halved on many devices, so a stored value of 80 is a displayed 160 spm,
/// and a stored value below 60 is walking or shuffling.
pub fn classify_sample(speed_ms: Option<f64>, stored_cadence: Option<f64>) -> SegmentKind {
    if let Some(speed) = speed_ms {
        if speed < 0.30 {
            return SegmentKind::Stand;
        }
    }
    if let Some(cadence) = stored_cadence {
        if cadence == 0.0 {
            return SegmentKind::Stand;
        }
    }
    if let Some(cadence) = stored_cadence {
        if cadence < 60.0 && speed_ms.map_or(true, |speed| speed < 2.2) {
            return SegmentKind::Walk;
        }
    }
    if let Some(speed) = speed_ms {
        if speed < 1.55 {
            return SegmentKind::Walk;
        }
    }
    SegmentKind::Run
}

/// Split the record stream into run / walk / stand segments.
///
/// A sample owns the interval that follows it, up to the next sample: a hole
/// in the stream is charged to the sample before it.
fn segment_records(
    records: &[RecordRow],
    start_ts: Option<f64>,
) -> (Vec<Segment>, Vec<ClassifiedRow>, Totals) {
    let mut segments: Vec<Segment> = Vec::new();
    let mut classified: Vec<ClassifiedRow> = Vec::with_capacity(records.len());
    let mut totals = Totals::default();
    if records.is_empty() {
        return (segments, classified, totals);
    }

    let base = start_ts.unwrap_or_else(|| records[0].ts);
    let kinds: Vec<SegmentKind> = records
        .iter()
        .map(|record| classify_sample(record.speed, record.cadence))
        .collect();

    for (index, record) in records.iter().enumerate() {
        let elapsed = record.ts - base;
        let kind = kinds[index];
        let next_elapsed = if index + 1 < records.len() {
            records[index + 1].ts - base
        } else {
            elapsed
        };
        let span = (next_elapsed - elapsed).max(0.0);
        match kind {
            SegmentKind::Run => totals.run_seconds += span,
            SegmentKind::Walk => totals.walk_seconds += span,
            SegmentKind::Stand => totals.stand_seconds += span,
        }

        if index + 1 < records.len() {
            if let (Some(previous), Some(current)) = (record.distance, records[index + 1].distance)
            {
                if current > previous {
                    match kind {
                        SegmentKind::Run => totals.run_metres += current - previous,
                        SegmentKind::Walk => {}
                        SegmentKind::Stand => {}
                    }
                }
            }
        }

        match segments.last_mut() {
            Some(segment) if segment.kind == kind => {
                segment.end_s = round_to(elapsed, 3);
                segment.samples += 1;
            }
            _ => segments.push(Segment {
                kind,
                start_s: round_to(elapsed, 3),
                end_s: round_to(elapsed, 3),
                samples: 1,
                duration_s: 0.0,
            }),
        }

        classified.push(ClassifiedRow {
            elapsed,
            distance: record.distance,
            speed: record.speed,
            cadence: record.cadence,
            heart_rate: record.heart_rate,
            altitude: record.altitude,
            kind,
        });
    }

    for segment in &mut segments {
        segment.duration_s = round_to(segment.end_s - segment.start_s, 3);
    }

    (segments, classified, totals)
}

/// Rebuild every lap over the running samples of its window.
///
/// A lap keeps its index and its distance, but its time, pace, heart rate
/// and cadence describe only the running. A lap that was mostly walking
/// comes back with very little running in it — which is the truth.
fn running_laps(laps: Vec<Lap>, classified: &[ClassifiedRow], sport: Option<&str>) -> Vec<Lap> {
    let mut out = Vec::with_capacity(laps.len());
    let mut cursor = 0.0;
    for lap in laps {
        let end = if lap.elapsed_s != 0.0 {
            lap.elapsed_s
        } else {
            cursor
        };
        let start = cursor;
        cursor = end;
        let duration = (end - start).max(0.0);

        let window: Vec<&ClassifiedRow> = classified
            .iter()
            .filter(|row| row.elapsed >= start && row.elapsed <= end)
            .collect();
        let running: Vec<&ClassifiedRow> = window
            .iter()
            .copied()
            .filter(|row| row.kind == SegmentKind::Run)
            .collect();

        let mut running_time = 0.0;
        let mut distance = 0.0;
        for index in 1..window.len() {
            let previous = window[index - 1];
            let current = window[index];
            // Only an interval between two running samples counts as running;
            // a gap that spans a rest stop is not running time.
            if previous.kind != SegmentKind::Run || current.kind != SegmentKind::Run {
                continue;
            }
            running_time += (current.elapsed - previous.elapsed).max(0.0);
            if let (Some(previous), Some(current)) = (previous.distance, current.distance) {
                distance += (current - previous).max(0.0);
            }
        }

        let heart_rates: Vec<f64> = running.iter().filter_map(|row| row.heart_rate).collect();
        let cadences: Vec<f64> = running
            .iter()
            .filter_map(|row| row.cadence.map(|value| running_cadence(value, sport)))
            .collect();

        let pace = if distance > 1.0 && running_time != 0.0 {
            Some(running_time / (distance / 1000.0))
        } else {
            None
        };

        out.push(Lap {
            duration_s: round_to(running_time, 3),
            running_distance_m: Some(round_to(distance, 2)),
            avg_speed_ms: if running_time > 0.0 && distance > 0.0 {
                Some(distance / running_time)
            } else {
                None
            },
            avg_hr: mean(&heart_rates).map(|value| trunc_to_int(value) as f64),
            max_hr: heart_rates
                .iter()
                .copied()
                .fold(None, |best: Option<f64>, value| {
                    Some(trunc_to_int(best.map_or(value, |b| b.max(value))) as f64)
                }),
            avg_cadence: mean(&cadences).map(|value| round_to(value, 1)),
            max_cadence: cadences
                .iter()
                .copied()
                .fold(None, |best: Option<f64>, value| {
                    Some(best.map_or(value, |b| b.max(value)))
                })
                .map(|value| round_to(value, 1)),
            pace_s_per_km: pace
                .filter(|value| *value != 0.0)
                .map(|value| round_to(value, 2)),
            elapsed_s: round_to(duration, 3),
            ..lap
        });
    }
    out
}

// ------------------------------------------------------------------ helpers

/// Read a field's value, applying the profile's scale and offset.
fn number(message: &Message, mesg_num: u16, name: &str) -> Option<f64> {
    let spec = profile::spec(mesg_num, name)?;
    if spec.kind == FieldKind::Text {
        return None;
    }
    let raw = message.number(spec.def_num)?;
    Some(spec.apply(raw))
}

/// Read a field's text.
fn text<'a>(message: &'a Message, mesg_num: u16, name: &str) -> Option<&'a str> {
    let spec = profile::spec(mesg_num, name)?;
    message.text(spec.def_num)
}

/// Read an enumeration field's raw value.
///
/// The value is kept at the field's own width: the manufacturer is a
/// `uint16`, and a manufacturer above 255 (the profile knows one) must not
/// be truncated into the name of another watch.
fn enum_number(message: &Message, mesg_num: u16, name: &str) -> Option<u16> {
    let spec = profile::spec(mesg_num, name)?;
    message
        .number(spec.def_num)
        .filter(|raw| *raw >= 0.0 && *raw <= f64::from(u16::MAX))
        .map(|raw| raw as u16)
}

/// `value` when it is inside `low..=high`, otherwise `None`.
fn in_range(value: Option<f64>, low: f64, high: f64) -> Option<f64> {
    match value {
        Some(value) if value >= low && value <= high => Some(value),
        _ => None,
    }
}

/// The reference's `_bounded_number`: `None` when the value is missing or out
/// of range.
fn bounded(value: Option<f64>, maximum: f64) -> Option<f64> {
    match value {
        Some(value) if value >= 0.0 && value <= maximum => Some(value),
        _ => None,
    }
}

/// FIT positions are semicircles: `2^31` semicircles is 180 degrees.
fn semicircles_to_degrees(semicircles: f64) -> f64 {
    round_to(semicircles * 180.0 / 2_147_483_648.0, 7)
}

/// Normalise a FIT sport into Stride's vocabulary.
fn normalise_sport(value: &str) -> String {
    if value.is_empty() {
        return "running".to_string();
    }
    let key = value.trim().to_lowercase().replace(' ', "_");
    let mapped = match key.as_str() {
        "running" => Some("running"),
        "trail_running" => Some("trail_running"),
        "treadmill_running" => Some("treadmill_running"),
        "street_running" | "track_running" => Some("running"),
        "virtualrun" => Some("virtualrun"),
        "walking" => Some("walking"),
        "hiking" => Some("hiking"),
        "cycling" => Some("cycling"),
        "lap_swimming" | "open_water_swimming" | "pool_swimming" => Some("swimming"),
        "strength_training" => Some("strength_training"),
        _ => None,
    };
    if let Some(mapped) = mapped {
        return mapped.to_string();
    }
    if key.ends_with("running") {
        return "running".to_string();
    }
    if key.is_empty() {
        return "running".to_string();
    }
    key
}

/// Sports whose cadence field counts stride cycles (two footfalls) rather
/// than steps.
const HALVED_CADENCE_SPORTS: &[&str] = &[
    "running",
    "trail_running",
    "treadmill_running",
    "street_running",
    "track_running",
    "virtualrun",
];

/// Normalise a FIT cadence to steps per minute.
///
/// Some devices record running cadence in *stride cycles* rather than steps:
/// one cycle is two footfalls, so the watch displays 155 spm while the file
/// says 77. A running cadence below 100 is therefore doubled.
///
/// Cycling cadence is revolutions per minute and is **not** doubled, so the
/// correction is only applied to running sports.
pub fn running_cadence(value: f64, sport: Option<&str>) -> f64 {
    if let Some(sport) = sport {
        if !HALVED_CADENCE_SPORTS.contains(&sport) {
            return value;
        }
    }
    if value > 0.0 && value < 100.0 {
        value * 2.0
    } else {
        value
    }
}

/// Build one stream over the running samples.
fn series(samples: &[(f64, f64)], start_ts: f64, unit: &str) -> Stream {
    Stream {
        seconds: samples
            .iter()
            .map(|(ts, _)| trunc_to_int(round_to(ts - start_ts, 0)))
            .collect(),
        values: samples.iter().map(|(_, value)| *value).collect(),
        unit: unit.to_string(),
    }
}

/// The activity's name: the file name without its extension, with `_` and
/// `-` turned into spaces.
fn activity_name(filename: &str) -> String {
    let stem = match filename.rsplit_once('.') {
        Some((stem, _)) => stem,
        None => filename,
    };
    let name = stem.replace(['_', '-'], " ").trim().to_string();
    if name.is_empty() {
        "Imported run".to_string()
    } else {
        name
    }
}

/// The reference's `enum_name`: `sport:fitness_equipment` -> `fitness_equipment`.
fn enum_name(value: &str) -> String {
    match value.split_once(':') {
        Some((_, rest)) => rest.to_string(),
        None => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycling_cadence_is_not_halved() {
        assert_eq!(running_cadence(90.0, Some("cycling")), 90.0);
        assert_eq!(running_cadence(90.0, Some("running")), 180.0);
        assert_eq!(running_cadence(90.0, None), 180.0);
        assert_eq!(running_cadence(120.0, Some("running")), 120.0);
        assert_eq!(running_cadence(0.0, Some("running")), 0.0);
    }

    #[test]
    fn classification_matches_the_reference() {
        use SegmentKind::{Run, Stand, Walk};
        assert_eq!(classify_sample(Some(0.0), Some(80.0)), Stand);
        assert_eq!(classify_sample(Some(3.0), Some(0.0)), Stand);
        assert_eq!(classify_sample(Some(1.3), Some(30.0)), Walk);
        assert_eq!(classify_sample(Some(2.0), Some(30.0)), Walk);
        assert_eq!(classify_sample(Some(2.5), Some(30.0)), Run);
        assert_eq!(classify_sample(Some(1.0), None), Walk);
        assert_eq!(classify_sample(Some(3.0), Some(78.0)), Run);
    }

    #[test]
    fn semicircles_become_degrees() {
        // 12.9716 degrees is 154 757 216 semicircles.
        assert_eq!(semicircles_to_degrees(154_757_216.0), 12.9716);
        assert_eq!(semicircles_to_degrees(0.0), 0.0);
    }
}
