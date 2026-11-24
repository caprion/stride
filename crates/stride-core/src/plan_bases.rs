//! The programmes: the catalogue the athlete adopts from.
//!
//! A **programme** is a method's own template, read cell by cell from the
//! book's schedule: the 10K tiers 1–2 and the half-marathon tiers 1–2,
//! with their twelve weeks counted down from eleven to one and then the
//! race week, their entry requirement, their tune-up races and the weeks
//! that follow the race. The catalogue is **data**, not code: it is read
//! from a file that is compiled into the crate, and it is versioned.
//!
//! ## Where the data lives
//!
//! The canonical copy is inside the crate — `plan_bases.json`, read with
//! `include_str!` and parsed when it is first needed — so the phone pays
//! nothing for it at startup and the data cannot go missing from the
//! binary. The **exploration copy** (`exploration/plan-templates.json`) is
//! the verification copy, and a test in this crate reads both files and
//! holds them to the same catalogue: the data and the code cannot drift.
//!
//! The core takes no JSON dependency — it is pure functions over plain
//! data, and every dependency it takes is a cost the phone pays — so the
//! reader for the one file the core reads lives here, in the crate. The
//! test reads the same file with a second, independent parser, so a fault
//! in this reader cannot quietly change the programme the athlete adopts.
//!
//! ## Why four programmes and not five
//!
//! The 80/20 plan is **not** in the catalogue, and its absence is the
//! point. Its schedule has not been read cell by cell: the published plans
//! are level-based and time-based, and a cell holds minutes, not distances
//! (`exploration/plan-bases-spec.md` §10). A catalogue entry would have to
//! invent the distances, and this catalogue invents nothing — every cell
//! traces to a row of the book's own table, and every week reconciles
//! against the mileage the book prints beside it.
//!
//! The 80/20 base also wants things the shape does not yet carry: a
//! duration on a session, a band scale set from the threshold rather than
//! from the watch, an entry requirement in hours rather than miles, and a
//! recovery week every third week. Until those are settled, the 80/20 base
//! stays a spec — and the app says so rather than offering a programme it
//! has not read.
//!
//! ## What the catalogue never does
//!
//! * It never suggests a programme. The tier is the athlete's own, and the
//!   entry requirement is **stated** beside the athlete's own mileage, never
//!   compared with it.
//! * It never invents a distance, a week or a kind.
//! * It never holds a date: a programme is undated, and an adoption is
//!   what gives it its days.

use std::sync::OnceLock;

use crate::date::{Date, Weekday};
use crate::plan::{PlanSource, SessionKind};
use crate::strings;
use crate::threshold::ThresholdChoice;
use crate::training_paces::{
    self, LadderInput, LongRunKind, MissingInput, PaceBand, PaceValue, Race, RungKind,
    METRES_PER_MILE,
};
use crate::units;

/// The catalogue's version.
///
/// A programme is a **versioned** catalogue entry (B1): an adopted plan
/// keeps the id and the version it was adopted from, so a later correction
/// to the catalogue cannot alter a plan that is running.
pub const CATALOGUE_VERSION: &str = "1";

/// The catalogue's own file, compiled into the crate.
const CATALOGUE_JSON: &str = include_str!("plan_bases.json");

/// The catalogue: the programmes the athlete can adopt.
#[derive(Clone, Debug, PartialEq)]
pub struct Catalogue {
    /// The catalogue's version.
    pub version: String,
    /// The schema the file carries.
    pub schema: String,
    /// The programmes, in the file's own order.
    pub programmes: Vec<Programme>,
    /// The reason the catalogue could not be read, when it could not.
    ///
    /// A catalogue that cannot be read is a **no value with a reason** —
    /// never a programme that is not there, and never a panic.
    pub error: Option<String>,
}

/// One programme: the method's own template, undated.
#[derive(Clone, Debug, PartialEq)]
pub struct Programme {
    /// The catalogue's stable key: "pfitzinger-half-marathon-tier-2".
    pub id: String,
    /// The programme's own name: "half marathon tier 2".
    pub label: String,
    /// Whose method it is: "Pfitzinger".
    pub method: String,
    /// The goal distance, in metres.
    pub distance_m: f64,
    /// The goal distance's name: "half marathon".
    pub distance_label: String,
    /// How many weeks the programme runs: twelve.
    pub weeks: usize,
    /// The tier's **entry requirement** — the weekly mileage the level
    /// asks the athlete to be at.
    pub entry_mileage: Option<MileageRange>,
    /// The band the plan itself runs in.
    pub plan_mileage: Option<MileageRange>,
    /// The goal race: the distance, and the options the programme prints.
    pub goal_race: GoalRace,
    /// Where the programme was read from.
    pub source: ProgrammeSource,
    /// The facts a reader needs about it.
    pub notes: Vec<String>,
    /// The weeks, in the file's own order: eleven down to one, then the
    /// race week, then the weeks after the race.
    pub weeks_detail: Vec<TemplateWeek>,
}

impl Programme {
    /// The week with this number: eleven down to one, zero for race week,
    /// and negative for the weeks after the race.
    pub fn week(&self, number: i32) -> Option<&TemplateWeek> {
        self.weeks_detail.iter().find(|week| week.week == number)
    }

    /// The race week — the twelfth week of the plan.
    pub fn race_week(&self) -> Option<&TemplateWeek> {
        self.week(0)
    }

    /// The weeks the programme carries after the race.
    pub fn post_race_weeks(&self) -> usize {
        self.weeks_detail
            .iter()
            .filter(|week| !week.in_plan)
            .count()
    }
}

/// A weekly mileage band.
#[derive(Clone, Debug, PartialEq)]
pub struct MileageRange {
    /// The lower edge, in metres a week.
    pub min_m: f64,
    /// The upper edge, in metres a week.
    pub max_m: f64,
    /// The band as the book prints it, when the book prints one.
    pub printed: Option<String>,
}

impl MileageRange {
    /// The band, rendered in the core's own unit: "74-101.4 km".
    pub fn render(&self) -> Option<String> {
        crate::units::render_distance_range(Some(self.min_m), Some(self.max_m))
    }
}

/// The goal race a programme prints.
#[derive(Clone, Debug, PartialEq)]
pub struct GoalRace {
    /// The race's own label: "8K or 10K goal race".
    pub label: String,
    /// The distance the programme prints.
    pub distance_m: f64,
    /// The distance's name: "10K".
    pub distance_label: String,
    /// The distances the athlete may choose between: the 8K or the 10K.
    pub options: Vec<RaceOption>,
    /// Where it was read from.
    pub source: String,
}

/// One distance the goal race may be run at.
#[derive(Clone, Debug, PartialEq)]
pub struct RaceOption {
    /// The distance's name: "8K".
    pub label: String,
    /// The distance, in metres.
    pub distance_m: f64,
}

/// Where a programme was read from.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammeSource {
    /// The spine's section the tables were read from.
    pub spine_section: String,
    /// The book's chapter.
    pub book_chapter: String,
}

/// One week of a programme, as the book prints it.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateWeek {
    /// The week's number: eleven down to one, zero for race week, and
    /// negative for the weeks after the race.
    pub week: i32,
    /// The weeks to go: eleven down to one, zero for race week, and absent
    /// for the weeks after the race.
    pub weeks_to_go: Option<i32>,
    /// The week's own label: "3 weeks to go", "race week".
    pub label: String,
    /// Whether the week is one of the plan's twelve.
    pub in_plan: bool,
    /// Whether the book prints the week as a recovery week.
    pub recovery_week: bool,
    /// The week's printed volume, in metres. `None` where the book prints
    /// "?" or "—": absence is not zero.
    pub printed_volume_m: Option<f64>,
    /// The reconciliation of the week's seven days against its printed
    /// volume.
    pub volume_check: VolumeCheck,
    /// Where the week was read from.
    pub source: String,
    /// The week's seven days, Monday first.
    pub days: Vec<TemplateDay>,
}

/// The check on a week: do its seven days add up to the mileage the book
/// prints beside them?
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VolumeCheck {
    /// The sum of the week's printed days, in metres.
    pub cells_m: Option<f64>,
    /// Whether the days add up to the printed volume.
    pub reconciles: Option<bool>,
    /// The note, when the week does not add up.
    pub note: Option<String>,
}

/// One day of a programme: a cell of the book's own schedule.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateDay {
    /// The day of the week.
    pub weekday: Weekday,
    /// The method's kind — the book's own vocabulary.
    pub kind: SessionKind,
    /// The book's short form for the kind: "GA+s".
    pub legend: String,
    /// The day's distance, in metres. `None` on a rest day: a day with no
    /// distance is a day with no distance, never a day of zero.
    pub distance_m: Option<f64>,
    /// The cell's own detail: "3 × 8 min, jog 3".
    pub detail: Option<String>,
    /// On a race day: the race's own distance.
    ///
    /// The number the schedule prints beside a race is the mileage the day
    /// counts for in the weekly total; it is **not** the race's distance
    /// (B5).
    pub race_distance_m: Option<f64>,
    /// On a race day: the race's own name: "5K".
    pub race_distance_label: Option<String>,
    /// On a progression long run: how many of the last miles are at the
    /// threshold pace.
    pub lt_last_m: Option<f64>,
    /// The whole cell, as the book prints it.
    pub cell: String,
}

/// The catalogue.
///
/// The file is parsed once, when it is first needed, and the result is
/// kept: the parse is a pure function of the compiled-in text.
pub fn catalogue() -> &'static Catalogue {
    static CATALOGUE: OnceLock<Catalogue> = OnceLock::new();
    CATALOGUE.get_or_init(|| read_catalogue(CATALOGUE_JSON))
}

/// The programmes the athlete can adopt.
pub fn programmes() -> &'static [Programme] {
    &catalogue().programmes
}

/// One programme, by its catalogue id.
pub fn programme(id: &str) -> Option<&'static Programme> {
    programmes().iter().find(|programme| programme.id == id)
}

/// The catalogue's summaries: what the api's route reads.
///
/// The summary is the programme's own face — its id, its name, whose
/// method it is, the goal distance, the entry requirement and the length.
/// The weeks come from [`programme`], which holds all of them.
pub fn summaries() -> &'static [crate::plan::PlanBase] {
    static SUMMARIES: OnceLock<Vec<crate::plan::PlanBase>> = OnceLock::new();
    SUMMARIES.get_or_init(|| {
        programmes()
            .iter()
            .map(|programme| crate::plan::PlanBase {
                id: programme.id.clone(),
                label: programme.label.clone(),
                method: programme.method.clone(),
                distance_m: programme.distance_m,
                distance_label: programme.distance_label.clone(),
                entry_m_per_week: programme
                    .entry_mileage
                    .as_ref()
                    .map(|range| (range.min_m, range.max_m)),
                weeks: programme.weeks,
            })
            .collect()
    })
}

/// Read the catalogue out of the file.
///
/// A programme that cannot be read is not a programme that is half there:
/// the whole catalogue is a no value with a reason.
fn read_catalogue(text: &str) -> Catalogue {
    let unreadable = || Catalogue {
        version: CATALOGUE_VERSION.to_string(),
        schema: String::new(),
        programmes: Vec::new(),
        error: strings::text("plan.bases.unreadable").map(|text| text.to_string()),
    };
    let root = match json::Reader::read(text) {
        Some(root) => root,
        None => return unreadable(),
    };
    let schema = root
        .get("schema")
        .and_then(json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let templates = match root.get("templates").and_then(json::Value::as_array) {
        Some(templates) => templates,
        None => return unreadable(),
    };
    let mut programmes = Vec::with_capacity(templates.len());
    for template in templates {
        match read_programme(template) {
            Some(programme) => programmes.push(programme),
            None => return unreadable(),
        }
    }
    Catalogue {
        version: CATALOGUE_VERSION.to_string(),
        schema,
        programmes,
        error: None,
    }
}

/// Read one programme.
fn read_programme(value: &json::Value) -> Option<Programme> {
    let id = value.get("id")?.as_str()?.to_string();
    let label = value.get("label")?.as_str()?.to_string();
    let method = value.get("method")?.as_str()?.to_string();
    let distance_m = value.get("distance_m")?.as_f64()?;
    let weeks = value.get("weeks")?.as_f64()? as usize;
    let goal_race = read_goal_race(value.get("goal_race")?)?;
    let distance_label = goal_race.distance_label.clone();
    let entry_mileage = value.get("entry_mileage").and_then(read_mileage_range);
    let plan_mileage = value.get("plan_mileage").and_then(read_mileage_range);
    let source = match value.get("source") {
        Some(source) => ProgrammeSource {
            spine_section: source
                .get("spine_section")
                .and_then(json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            book_chapter: source
                .get("book_chapter")
                .and_then(json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
        },
        None => ProgrammeSource {
            spine_section: String::new(),
            book_chapter: String::new(),
        },
    };
    let notes = value
        .get("notes")
        .and_then(json::Value::as_array)
        .map(|notes| {
            notes
                .iter()
                .filter_map(json::Value::as_str)
                .map(|note| note.to_string())
                .collect()
        })
        .unwrap_or_default();
    let mut weeks_detail = Vec::new();
    for week in value.get("weeks_detail")?.as_array()? {
        weeks_detail.push(read_template_week(week)?);
    }
    Some(Programme {
        id,
        label,
        method,
        distance_m,
        distance_label,
        weeks,
        entry_mileage,
        plan_mileage,
        goal_race,
        source,
        notes,
        weeks_detail,
    })
}

/// Read the goal race.
fn read_goal_race(value: &json::Value) -> Option<GoalRace> {
    let label = value.get("label")?.as_str()?.to_string();
    let distance_m = value.get("distance_m")?.as_f64()?;
    let distance_label = value
        .get("distance_label")
        .and_then(json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut options = Vec::new();
    for option in value.get("options")?.as_array()? {
        options.push(RaceOption {
            label: option.get("label")?.as_str()?.to_string(),
            distance_m: option.get("distance_m")?.as_f64()?,
        });
    }
    let source = value
        .get("source")
        .and_then(json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    Some(GoalRace {
        label,
        distance_m,
        distance_label,
        options,
        source,
    })
}

/// Read a weekly mileage band.
///
/// The book prints miles; the core's own unit is the metre, and the
/// conversion happens here, where the data is read.
fn read_mileage_range(value: &json::Value) -> Option<MileageRange> {
    let min_mi = value.get("min_mi")?.as_f64()?;
    let max_mi = value.get("max_mi")?.as_f64()?;
    let printed = value
        .get("printed_km")
        .and_then(json::Value::as_str)
        .map(|text| text.to_string());
    Some(MileageRange {
        min_m: min_mi * METRES_PER_MILE,
        max_m: max_mi * METRES_PER_MILE,
        printed,
    })
}

/// Read one week of the file.
fn read_template_week(value: &json::Value) -> Option<TemplateWeek> {
    let week = value.get("week")?.as_f64()? as i32;
    let weeks_to_go = value
        .get("weeks_to_go")
        .and_then(json::Value::as_f64)
        .map(|number| number as i32);
    let label = value.get("label")?.as_str()?.to_string();
    let in_plan = value
        .get("in_plan")
        .and_then(json::Value::as_bool)
        .unwrap_or(false);
    let recovery_week = value
        .get("recovery_week")
        .and_then(json::Value::as_bool)
        .unwrap_or(false);
    let printed_volume_m = value
        .get("printed_volume_mi")
        .and_then(json::Value::as_f64)
        .map(|miles| miles * METRES_PER_MILE);
    let volume_check = match value.get("volume_check") {
        Some(check) => VolumeCheck {
            cells_m: check
                .get("cells_mi")
                .and_then(json::Value::as_f64)
                .map(|miles| miles * METRES_PER_MILE),
            reconciles: check.get("reconciles").and_then(json::Value::as_bool),
            note: check
                .get("note")
                .and_then(json::Value::as_str)
                .map(|text| text.to_string()),
        },
        None => VolumeCheck::default(),
    };
    let source = value
        .get("source")
        .and_then(json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut days = Vec::new();
    for day in value.get("days")?.as_array()? {
        days.push(read_day(day)?);
    }
    Some(TemplateWeek {
        week,
        weeks_to_go,
        label,
        in_plan,
        recovery_week,
        printed_volume_m,
        volume_check,
        source,
        days,
    })
}

/// Read one day of a week.
fn read_day(value: &json::Value) -> Option<TemplateDay> {
    let weekday = weekday_of(value.get("day")?.as_str()?)?;
    let kind = SessionKind::from_key(value.get("kind")?.as_str()?)?;
    let legend = value
        .get("kind_label")
        .and_then(json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let distance_m = value
        .get("distance_mi")
        .and_then(json::Value::as_f64)
        .map(|miles| miles * METRES_PER_MILE);
    let detail = value
        .get("detail")
        .and_then(json::Value::as_str)
        .map(|text| text.to_string());
    let race_distance_m = value.get("race_distance_m").and_then(json::Value::as_f64);
    let race_distance_label = value
        .get("race_distance_label")
        .and_then(json::Value::as_str)
        .map(|text| text.to_string());
    let lt_last_m = value
        .get("lt_last_mi")
        .and_then(json::Value::as_f64)
        .map(|miles| miles * METRES_PER_MILE);
    let cell = value.get("cell")?.as_str()?.to_string();
    Some(TemplateDay {
        weekday,
        kind,
        legend,
        distance_m,
        detail,
        race_distance_m,
        race_distance_label,
        lt_last_m,
        cell,
    })
}

/// The day of the week the catalogue's own key names.
fn weekday_of(text: &str) -> Option<Weekday> {
    let weekday = match text {
        "mon" => Weekday::Monday,
        "tue" => Weekday::Tuesday,
        "wed" => Weekday::Wednesday,
        "thu" => Weekday::Thursday,
        "fri" => Weekday::Friday,
        "sat" => Weekday::Saturday,
        "sun" => Weekday::Sunday,
        _ => return None,
    };
    Some(weekday)
}

/// The small JSON reader.
///
/// The core reads exactly one JSON file — the catalogue's own — and takes
/// no JSON dependency, so the reader is here. It reads the whole of JSON
/// the file uses: objects, arrays, strings (with their escapes), numbers,
/// booleans and null.
///
/// The crate's own test reads the same file with a second, independent
/// parser and holds the two to the same catalogue, so a fault here cannot
/// quietly change the programme the athlete adopts.
mod json {
    /// A JSON value.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Value {
        /// `null`.
        Null,
        /// `true` or `false`.
        Bool(bool),
        /// A number.
        Number(f64),
        /// A string.
        String(String),
        /// An array.
        Array(Vec<Value>),
        /// An object: its members, in the order the file lists them.
        Object(Vec<(String, Value)>),
    }

    impl Value {
        /// The member with this key, when the value is an object.
        pub fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Object(members) => members
                    .iter()
                    .find(|(name, _)| name == key)
                    .map(|(_, value)| value),
                _ => None,
            }
        }

        /// The value's text, when it is a string.
        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(text) => Some(text),
                _ => None,
            }
        }

        /// The value's number, when it is one.
        pub fn as_f64(&self) -> Option<f64> {
            match self {
                Value::Number(number) => Some(*number),
                _ => None,
            }
        }

        /// The value's truth, when it is a boolean.
        pub fn as_bool(&self) -> Option<bool> {
            match self {
                Value::Bool(flag) => Some(*flag),
                _ => None,
            }
        }

        /// The value's items, when it is an array.
        pub fn as_array(&self) -> Option<&[Value]> {
            match self {
                Value::Array(items) => Some(items),
                _ => None,
            }
        }
    }

    /// The reader: a cursor over the text.
    pub struct Reader<'a> {
        bytes: &'a [u8],
        at: usize,
    }

    impl<'a> Reader<'a> {
        /// The whole document, when it is one value and nothing else.
        pub fn read(text: &'a str) -> Option<Value> {
            let mut reader = Reader {
                bytes: text.as_bytes(),
                at: 0,
            };
            reader.skip_space();
            let value = reader.value()?;
            reader.skip_space();
            if reader.at == reader.bytes.len() {
                Some(value)
            } else {
                None
            }
        }

        /// The byte at the cursor.
        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.at).copied()
        }

        /// Skip the space between values.
        fn skip_space(&mut self) {
            while let Some(byte) = self.peek() {
                if matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
                    self.at += 1;
                } else {
                    break;
                }
            }
        }

        /// The next value.
        fn value(&mut self) -> Option<Value> {
            match self.peek()? {
                b'{' => self.object(),
                b'[' => self.array(),
                b'"' => self.string().map(Value::String),
                b't' => self.literal("true").map(|_| Value::Bool(true)),
                b'f' => self.literal("false").map(|_| Value::Bool(false)),
                b'n' => self.literal("null").map(|_| Value::Null),
                _ => self.number().map(Value::Number),
            }
        }

        /// A literal word: `true`, `false`, `null`.
        fn literal(&mut self, word: &str) -> Option<()> {
            if self.bytes[self.at..].starts_with(word.as_bytes()) {
                self.at += word.len();
                Some(())
            } else {
                None
            }
        }

        /// An object.
        fn object(&mut self) -> Option<Value> {
            self.at += 1;
            let mut members = Vec::new();
            self.skip_space();
            if self.peek()? == b'}' {
                self.at += 1;
                return Some(Value::Object(members));
            }
            loop {
                self.skip_space();
                let key = self.string()?;
                self.skip_space();
                if self.peek()? != b':' {
                    return None;
                }
                self.at += 1;
                self.skip_space();
                let value = self.value()?;
                members.push((key, value));
                self.skip_space();
                match self.peek()? {
                    b',' => self.at += 1,
                    b'}' => {
                        self.at += 1;
                        return Some(Value::Object(members));
                    }
                    _ => return None,
                }
            }
        }

        /// An array.
        fn array(&mut self) -> Option<Value> {
            self.at += 1;
            let mut items = Vec::new();
            self.skip_space();
            if self.peek()? == b']' {
                self.at += 1;
                return Some(Value::Array(items));
            }
            loop {
                self.skip_space();
                items.push(self.value()?);
                self.skip_space();
                match self.peek()? {
                    b',' => self.at += 1,
                    b']' => {
                        self.at += 1;
                        return Some(Value::Array(items));
                    }
                    _ => return None,
                }
            }
        }

        /// A string, with its escapes.
        fn string(&mut self) -> Option<String> {
            if self.peek()? != b'"' {
                return None;
            }
            self.at += 1;
            let mut out = String::new();
            loop {
                let byte = self.peek()?;
                self.at += 1;
                match byte {
                    b'"' => return Some(out),
                    b'\\' => {
                        let escape = self.peek()?;
                        self.at += 1;
                        match escape {
                            b'"' => out.push('"'),
                            b'\\' => out.push('\\'),
                            b'/' => out.push('/'),
                            b'b' => out.push('\u{8}'),
                            b'f' => out.push('\u{c}'),
                            b'n' => out.push('\n'),
                            b'r' => out.push('\r'),
                            b't' => out.push('\t'),
                            b'u' => out.push(self.escape()?),
                            _ => return None,
                        }
                    }
                    _ => {
                        // A character outside ASCII: the file's own text is
                        // UTF-8, and the character is copied whole.
                        let start = self.at - 1;
                        let width = utf8_width(byte);
                        let end = start + width;
                        let slice = self.bytes.get(start..end)?;
                        out.push_str(std::str::from_utf8(slice).ok()?);
                        self.at = end;
                    }
                }
            }
        }

        /// A `\u` escape, with its surrogate pair.
        fn escape(&mut self) -> Option<char> {
            let first = self.hex4()?;
            let code = if (0xD800..0xDC00).contains(&first) {
                if self.peek()? != b'\\' {
                    return None;
                }
                self.at += 1;
                if self.peek()? != b'u' {
                    return None;
                }
                self.at += 1;
                let second = self.hex4()?;
                if !(0xDC00..0xE000).contains(&second) {
                    return None;
                }
                0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00)
            } else {
                first
            };
            char::from_u32(code)
        }

        /// Four hexadecimal digits.
        fn hex4(&mut self) -> Option<u32> {
            let mut value = 0u32;
            for _ in 0..4 {
                let byte = self.peek()?;
                let digit = match byte {
                    b'0'..=b'9' => u32::from(byte - b'0'),
                    b'a'..=b'f' => u32::from(byte - b'a') + 10,
                    b'A'..=b'F' => u32::from(byte - b'A') + 10,
                    _ => return None,
                };
                self.at += 1;
                value = value * 16 + digit;
            }
            Some(value)
        }

        /// A number.
        fn number(&mut self) -> Option<f64> {
            let start = self.at;
            if self.peek() == Some(b'-') {
                self.at += 1;
            }
            while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
                self.at += 1;
            }
            if self.peek() == Some(b'.') {
                self.at += 1;
                while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
                    self.at += 1;
                }
            }
            if matches!(self.peek(), Some(b'e') | Some(b'E')) {
                self.at += 1;
                if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                    self.at += 1;
                }
                while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
                    self.at += 1;
                }
            }
            let text = std::str::from_utf8(self.bytes.get(start..self.at)?).ok()?;
            text.parse::<f64>().ok()
        }
    }

    /// The width of a UTF-8 character, from its first byte.
    fn utf8_width(byte: u8) -> usize {
        match byte {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            _ => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::json::{Reader, Value};
    use super::*;

    /// The crate's own file, read by the reader the core uses.
    fn crate_file() -> &'static str {
        CATALOGUE_JSON
    }

    /// The same text, read by a second, independent parser.
    fn read_with_serde(text: &str) -> Value {
        let parsed: serde_json::Value =
            serde_json::from_str(text).expect("the catalogue's own file is JSON");
        from_serde(&parsed)
    }

    /// A JSON value read by `serde_json`, as the reader's own value.
    ///
    /// The two trees are compared member by member and **not** in order:
    /// `serde_json` keeps an object's members in a map, and the reader
    /// keeps them in the file's own order.
    fn from_serde(value: &serde_json::Value) -> Value {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(flag) => Value::Bool(*flag),
            serde_json::Value::Number(number) => Value::Number(number.as_f64().unwrap_or(f64::NAN)),
            serde_json::Value::String(text) => Value::String(text.clone()),
            serde_json::Value::Array(items) => Value::Array(items.iter().map(from_serde).collect()),
            serde_json::Value::Object(members) => Value::Object(
                members
                    .iter()
                    .map(|(name, value)| (name.clone(), from_serde(value)))
                    .collect(),
            ),
        }
    }

    /// Whether two values are the same, member by member.
    fn same(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(one), Value::Bool(other)) => one == other,
            (Value::Number(one), Value::Number(other)) => (one - other).abs() < f64::EPSILON,
            (Value::String(one), Value::String(other)) => one == other,
            (Value::Array(one), Value::Array(other)) => {
                one.len() == other.len() && one.iter().zip(other.iter()).all(|(a, b)| same(a, b))
            }
            (Value::Object(one), Value::Object(other)) => {
                one.len() == other.len()
                    && one.iter().all(|(name, value)| {
                        other
                            .iter()
                            .find(|(other_name, _)| other_name == name)
                            .map(|(_, other_value)| same(value, other_value))
                            .unwrap_or(false)
                    })
            }
            _ => false,
        }
    }

    #[test]
    fn the_reader_reads_the_catalogue_exactly_as_a_second_parser_does() {
        let mine = Reader::read(crate_file()).expect("the catalogue's own file is JSON");
        let theirs = read_with_serde(crate_file());
        assert!(
            same(&mine, &theirs),
            "the reader and the second parser do not agree on the catalogue's file"
        );
    }

    #[test]
    fn the_catalogue_reads_the_four_programmes() {
        let catalogue = catalogue();
        assert!(catalogue.error.is_none(), "{:?}", catalogue.error);
        assert_eq!(catalogue.version, CATALOGUE_VERSION);
        assert_eq!(catalogue.schema, "stride.plan-templates/v1".to_string());
        let ids: Vec<&str> = catalogue
            .programmes
            .iter()
            .map(|programme| programme.id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "pfitzinger-10k-tier-1",
                "pfitzinger-10k-tier-2",
                "pfitzinger-half-marathon-tier-1",
                "pfitzinger-half-marathon-tier-2",
            ]
        );
    }

    #[test]
    fn every_week_of_every_programme_adds_up_to_its_printed_mileage() {
        for programme in programmes() {
            assert_eq!(programme.weeks, 12, "{} runs twelve weeks", programme.id);
            assert_eq!(
                programme.weeks_detail.len(),
                programme.weeks + programme.post_race_weeks(),
                "{} carries the weeks after the race",
                programme.id
            );
            for week in programme.weeks_detail.iter() {
                assert_eq!(week.days.len(), 7, "{} week {}", programme.id, week.week);
                let cells: f64 = week
                    .days
                    .iter()
                    .map(|day| day.distance_m.unwrap_or(0.0))
                    .sum();
                if let (Some(printed), Some(recorded)) =
                    (week.printed_volume_m, week.volume_check.cells_m)
                {
                    assert!(
                        (cells - recorded).abs() < 1.0,
                        "{} week {}: the days add up to {cells}, and the file records {recorded}",
                        programme.id,
                        week.week
                    );
                    assert!(
                        (cells - printed).abs() < 1.0,
                        "{} week {}: the days add up to {cells}, and the book prints {printed}",
                        programme.id,
                        week.week
                    );
                }
                assert_eq!(
                    week.volume_check.reconciles,
                    Some(true),
                    "{} week {} reconciles",
                    programme.id,
                    week.week
                );
            }
        }
    }

    #[test]
    fn a_rest_day_has_no_distance_and_never_a_zero() {
        let programme = programme("pfitzinger-10k-tier-1").expect("a programme");
        let week = programme
            .weeks_detail
            .iter()
            .find(|week| week.week == 11)
            .expect("week eleven");
        let monday = week
            .days
            .iter()
            .find(|day| day.weekday == crate::date::Weekday::Monday)
            .expect("a monday");
        assert_eq!(monday.kind, crate::plan::SessionKind::RestCross);
        assert_eq!(monday.distance_m, None);
    }

    #[test]
    fn a_race_day_keeps_the_races_own_distance_apart_from_the_printed_miles() {
        let programme = programme("pfitzinger-half-marathon-tier-2").expect("a programme");
        let week = programme
            .weeks_detail
            .iter()
            .find(|week| week.week == 0)
            .expect("race week");
        let race = week
            .days
            .iter()
            .find(|day| day.kind == crate::plan::SessionKind::GoalRace)
            .expect("the goal race");
        // The book's number is the mileage the day counts for in the week's
        // total; the race's own distance is the half marathon.
        assert_eq!(race.distance_m, Some(17.0 * METRES_PER_MILE));
        assert_eq!(race.race_distance_m, Some(21_097.5));
    }
}

/// ## The adoption
///
/// An adopted plan is a **dated copy** of a programme (B1): the race date
/// anchors it, the sessions carry the paces worked out once at that moment,
/// and the plan keeps the record of what it was paced with. Nothing about
/// it is read again from the catalogue afterwards — a later correction to
/// the catalogue cannot alter a plan that is running.
///
/// ## What the athlete chose, and the facts the adoption reads
///
/// Every date is an argument: the core has no clock, and the adoption asks
/// for the day it happened.
#[derive(Clone, Debug, PartialEq)]
pub struct AdoptionInput {
    /// The programme the athlete chose, by its catalogue id.
    pub programme_id: String,
    /// The goal race's distance, in metres — one the programme prints.
    pub goal_distance_m: f64,
    /// The day of the goal race.
    pub race_date: Date,
    /// The day the plan was adopted.
    ///
    /// The threshold **in force that day** paces the plan (B4), and a
    /// later threshold does not re-pace it.
    pub adopted_on: Date,
    /// The first day of the plan, when the athlete named one.
    ///
    /// It must agree with the race date: the race week is the twelfth
    /// week, and the two cannot both be true otherwise.
    pub start_date: Option<Date>,
    /// The threshold in force on the day of the adoption.
    pub threshold: Option<ThresholdChoice>,
    /// The race the plan's own rule is read from.
    ///
    /// It is the athlete's own race, and it is the anchor the method's
    /// bands are worked out from when no threshold carries a pace.
    pub race: Option<Race>,
    /// The athlete's own recent weekly mileage, in metres, oldest first.
    ///
    /// The app computes it from the athlete's runs; the core only states it
    /// beside the tier's entry requirement.
    pub weekly_mileage_m: Vec<f64>,
}

/// An adopted plan: a dated copy of a programme.
#[derive(Clone, Debug, PartialEq)]
pub struct AdoptedPlan {
    /// The catalogue id of the programme.
    pub template_id: String,
    /// The catalogue version the plan was adopted from.
    pub template_version: String,
    /// The programme's label: "half marathon tier 2".
    pub label: String,
    /// The programme's method: "Pfitzinger".
    pub method: String,
    /// The goal distance, in metres.
    pub distance_m: f64,
    /// The goal distance's name: "half marathon".
    pub distance_label: String,
    /// The tier's entry requirement, as it was at the adoption.
    ///
    /// It is a **snapshot**: a later change to the catalogue does not
    /// rewrite the athlete's plan.
    pub entry_mileage: Option<MileageRange>,
    /// The plan's own mileage band.
    pub plan_mileage: Option<MileageRange>,
    /// The goal race's distance, as the athlete chose it.
    pub goal_distance_m: f64,
    /// The goal race's distance, named.
    pub goal_distance_label: String,
    /// The day of the goal race.
    pub race_date: Date,
    /// The day the plan was adopted.
    pub adopted_on: Date,
    /// The first day of the plan.
    pub start_date: Date,
    /// The last day of the plan — the last day of the last week after the
    /// race.
    pub end_date: Date,
    /// The weeks, in date order.
    pub weeks: Vec<AdoptedWeek>,
    /// What the plan was paced with.
    pub pacing: Pacing,
    /// The athlete's own recent weekly mileage, in metres, oldest first.
    pub athlete_mileage_m: Vec<f64>,
    /// Where the plan came from.
    ///
    /// An adopted programme is a **prescription**: the athlete chose it.
    pub source: PlanSource,
}

/// One week of an adopted plan.
#[derive(Clone, Debug, PartialEq)]
pub struct AdoptedWeek {
    /// The week's number: eleven down to one, zero for race week, and
    /// negative for the weeks after the race.
    pub week: i32,
    /// The weeks to go: eleven down to one, zero for race week, and absent
    /// for the weeks after the race.
    pub weeks_to_go: Option<i32>,
    /// The week's own label: "3 weeks to go", "race week".
    pub label: String,
    /// Whether the week is one of the plan's twelve.
    pub in_plan: bool,
    /// Whether the book prints the week as a recovery week.
    pub recovery_week: bool,
    /// The first day of the week.
    pub start_date: Date,
    /// The last day of the week.
    pub end_date: Date,
    /// The week's printed volume, in metres.
    pub printed_volume_m: Option<f64>,
    /// The reconciliation of the week's days against its printed volume.
    pub volume_check: VolumeCheck,
    /// The week's sessions, in date order.
    pub sessions: Vec<AdoptedSession>,
    /// The shift, when the race week was moved to the athlete's race day.
    pub shift: Option<RaceWeekShift>,
}

/// One session of an adopted plan.
#[derive(Clone, Debug, PartialEq)]
pub struct AdoptedSession {
    /// The day the session falls on.
    pub date: Date,
    /// The method's kind — the book's own vocabulary.
    pub kind: SessionKind,
    /// The session's distance, in metres.
    ///
    /// On a race day this is the **race's own distance** and not the
    /// number the schedule prints beside it (B5).
    pub distance_m: Option<f64>,
    /// The cell's own detail: "3 × 8 min, jog 3".
    pub detail: Option<String>,
    /// The session's pace.
    ///
    /// A session with no pace is a session with no pace — never a zero
    /// pace — and it says why.
    pub pace: PaceValue,
    /// The second end of a session with two: the long run's finish pace.
    pub finish_pace: PaceValue,
    /// On a progression long run: how many of the last miles are at the
    /// threshold pace.
    pub lt_last_m: Option<f64>,
}

/// The race week, moved to the day the athlete races.
///
/// The book's own rule for a race on another day is to move the race week
/// alone: no other week moves, and the shift is stated as a fact.
#[derive(Clone, Debug, PartialEq)]
pub struct RaceWeekShift {
    /// The days the week moved: negative is earlier.
    pub days: i64,
    /// The day the goal race falls on.
    pub race_weekday: String,
    /// The sessions that are not in this week.
    pub dropped: Vec<DroppedSession>,
}

/// A session the shift left out of the race week.
#[derive(Clone, Debug, PartialEq)]
pub struct DroppedSession {
    /// The day it was printed on.
    pub weekday: String,
    /// The runner's wording for its kind.
    pub kind: String,
}

/// What the plan was paced with, and where the numbers came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Pacing {
    /// The threshold that was in force when the plan was adopted.
    ///
    /// The plan keeps it, and a later threshold does not re-pace it.
    pub threshold: Option<PacingSource>,
    /// The race the plan's own rule was read from.
    pub race: Option<PacingSource>,
    /// The reason the sessions that need a race have no pace.
    pub missing: Option<String>,
}

/// One thing the plan's paces came from: a value and the day it was read.
#[derive(Clone, Debug, PartialEq)]
pub struct PacingSource {
    /// The value, rendered: "165 bpm" or "4:16–4:21 /km".
    pub value: String,
    /// The day it was read.
    pub date: Date,
}

/// Why an adoption could not be made.
///
/// A refusal is a **fact with a reason**, never a verdict.
#[derive(Clone, Debug, PartialEq)]
pub enum AdoptionRefusal {
    /// The programme is not in the catalogue.
    UnknownProgramme {
        /// The id the athlete named.
        id: String,
    },
    /// The goal race's distance is not one the programme prints.
    GoalDistance {
        /// The distances the programme prints.
        printed: Vec<String>,
        /// The distance that was given.
        given_m: f64,
    },
    /// The start date and the race date cannot both be true.
    StartDate {
        /// The Monday the plan's first week has to begin on.
        expected: Date,
        /// The first day the athlete named.
        given: Date,
    },
    /// The programme's own week does not add up to the mileage the book
    /// prints beside it.
    Volume {
        /// The week that does not add up.
        week: i32,
        /// What its days add up to, in metres.
        cells_m: f64,
        /// What the book prints, in metres.
        printed_m: Option<f64>,
    },
}

impl AdoptionRefusal {
    /// The refusal's reason, in the runner's words.
    pub fn reason(&self) -> Option<String> {
        match self {
            AdoptionRefusal::UnknownProgramme { id } => {
                strings::render("statement.plan.refusal.programme", &[("id", id)])
            }
            AdoptionRefusal::GoalDistance { printed, given_m } => {
                let options = printed.join(" or ");
                let distance = units::render_distance(Some(*given_m))?;
                strings::render(
                    "statement.plan.refusal.goal",
                    &[("options", &options), ("distance", &distance)],
                )
            }
            AdoptionRefusal::StartDate { expected, given } => {
                let expected = expected.to_iso();
                let given = given.to_iso();
                strings::render(
                    "statement.plan.refusal.start",
                    &[("start", &expected), ("race", &given)],
                )
            }
            AdoptionRefusal::Volume {
                week,
                cells_m,
                printed_m,
            } => {
                let week = week.to_string();
                let cells = units::render_distance(Some(*cells_m)).unwrap_or_default();
                let volume = printed_m
                    .and_then(|printed| units::render_distance(Some(printed)))
                    .unwrap_or_default();
                strings::render(
                    "statement.plan.refusal.volume",
                    &[("week", &week), ("cells", &cells), ("volume", &volume)],
                )
            }
        }
    }
}

/// The paces the plan was worked out with.
#[derive(Clone, Debug, PartialEq)]
struct PlanPaces {
    /// The threshold pace.
    threshold: PaceValue,
    /// The long run's first end.
    long_start: PaceValue,
    /// The long run's second end.
    long_finish: PaceValue,
    /// The recovery pace.
    recovery: PaceValue,
    /// The VO₂max pace: the 3K-to-5K race pace.
    vo2: PaceValue,
    /// The speed-repetition pace: the 800 m-to-mile race pace.
    speed: PaceValue,
    /// The goal race's own pace.
    race: PaceValue,
    /// The tune-up race's own pace.
    tune_up: PaceValue,
}

/// Adopt a programme.
///
/// The plan is a **dated copy** of the catalogue's programme:
///
/// * the **race date anchors it** (B3) — the race week is the twelfth
///   week, and the goal race falls on the book's own day for the
///   programme. A race on another day moves the race week alone, and the
///   move is stated as a fact;
/// * the **paces are worked out once, here** (B4), from the threshold in
///   force on the day of the adoption, and they are stored with it. A
///   later threshold does not re-pace a running plan;
/// * a **race day's distance is the race's own distance** (B5) and not the
///   number the schedule prints beside it.
///
/// The adoption is **refused**, with the reason, when the start date and
/// the race date cannot both be true, when the goal race's distance is not
/// one the programme prints, or when the programme's own week does not add
/// up to the mileage the book prints beside it.
pub fn adopt(input: &AdoptionInput) -> Result<AdoptedPlan, AdoptionRefusal> {
    let programme =
        programme(&input.programme_id).ok_or_else(|| AdoptionRefusal::UnknownProgramme {
            id: input.programme_id.clone(),
        })?;

    // The goal race's distance must be one the programme prints: the 10K
    // programme's race is the 8K or the 10K, and the half-marathon
    // programme's race is the half marathon.
    let option = programme
        .goal_race
        .options
        .iter()
        .find(|option| (option.distance_m - input.goal_distance_m).abs() < 1.0)
        .ok_or_else(|| AdoptionRefusal::GoalDistance {
            printed: programme
                .goal_race
                .options
                .iter()
                .map(|option| option.label.clone())
                .collect(),
            given_m: input.goal_distance_m,
        })?;

    // The race week is the twelfth week, and the plan begins eleven weeks
    // before it.
    let race_monday = week_start(input.race_date);
    let start = race_monday.add_days(-((programme.weeks as i64) - 1) * 7);
    if let Some(given) = input.start_date {
        if given != start {
            return Err(AdoptionRefusal::StartDate {
                expected: start,
                given,
            });
        }
    }

    // A programme whose own week does not add up is not adopted: the gap
    // is reported, and no cell is adjusted to make it sum.
    for week in programme.weeks_detail.iter() {
        if week.volume_check.reconciles == Some(false) {
            return Err(AdoptionRefusal::Volume {
                week: week.week,
                cells_m: week.volume_check.cells_m.unwrap_or(0.0),
                printed_m: week.printed_volume_m,
            });
        }
    }

    // The tune-up race's own distance: the pace it is run at.
    let tune_up_distance_m = programme
        .weeks_detail
        .iter()
        .flat_map(|week| week.days.iter())
        .find(|day| day.kind == SessionKind::TuneUpRace)
        .and_then(|day| day.race_distance_m);

    let paces = plan_paces(input, option.distance_m, tune_up_distance_m);
    let pacing = pacing_of(input);

    // The book's own day for the goal race, and the shift the athlete's
    // race day asks for.
    let book_race_day = programme
        .race_week()
        .and_then(|week| {
            week.days
                .iter()
                .find(|day| day.kind == SessionKind::GoalRace)
        })
        .map(|day| day.weekday);
    let shift_days = match book_race_day {
        Some(book) => weekday_index(input.race_date.weekday()) - weekday_index(book),
        None => 0,
    };

    let mut weeks = Vec::with_capacity(programme.weeks_detail.len());
    for template_week in programme.weeks_detail.iter() {
        let week_start_date = race_monday.add_days(-(template_week.week as i64) * 7);
        let mut sessions = Vec::with_capacity(template_week.days.len());
        let mut dropped = Vec::new();
        for day in template_week.days.iter() {
            let mut index = weekday_index(day.weekday);
            if template_week.week == 0 && shift_days != 0 {
                index += shift_days;
                if !(0..=6).contains(&index) {
                    // The session falls outside the week: it is not in this
                    // week, and the week says which one it was.
                    dropped.push(DroppedSession {
                        weekday: day.weekday.name().to_string(),
                        kind: day.kind.word(),
                    });
                    continue;
                }
            }
            let date = week_start_date.add_days(index);
            let distance_m = match day.kind {
                // A race day's distance is the race's own distance (B5).
                SessionKind::GoalRace => Some(option.distance_m),
                SessionKind::TuneUpRace => day.race_distance_m.or(day.distance_m),
                _ => day.distance_m,
            };
            let (pace, finish_pace) = pace_for(&day.kind, &paces);
            sessions.push(AdoptedSession {
                date,
                kind: day.kind.clone(),
                distance_m,
                detail: day.detail.clone(),
                pace,
                finish_pace,
                lt_last_m: day.lt_last_m,
            });
        }
        let shift = if template_week.week == 0 && shift_days != 0 {
            Some(RaceWeekShift {
                days: shift_days,
                race_weekday: input.race_date.weekday().name().to_string(),
                dropped,
            })
        } else {
            None
        };
        weeks.push(AdoptedWeek {
            week: template_week.week,
            weeks_to_go: template_week.weeks_to_go,
            label: template_week.label.clone(),
            in_plan: template_week.in_plan,
            recovery_week: template_week.recovery_week,
            start_date: week_start_date,
            end_date: week_start_date.add_days(6),
            printed_volume_m: template_week.printed_volume_m,
            volume_check: template_week.volume_check.clone(),
            sessions,
            shift,
        });
    }

    let end_date = weeks
        .last()
        .map(|week| week.end_date)
        .unwrap_or(race_monday.add_days(6));

    Ok(AdoptedPlan {
        template_id: programme.id.clone(),
        template_version: catalogue().version.clone(),
        label: programme.label.clone(),
        method: programme.method.clone(),
        distance_m: programme.distance_m,
        distance_label: programme.distance_label.clone(),
        entry_mileage: programme.entry_mileage.clone(),
        plan_mileage: programme.plan_mileage.clone(),
        goal_distance_m: option.distance_m,
        goal_distance_label: option.label.clone(),
        race_date: input.race_date,
        adopted_on: input.adopted_on,
        start_date: start,
        end_date,
        weeks,
        pacing,
        athlete_mileage_m: input.weekly_mileage_m.clone(),
        // An adopted programme is a prescription: the athlete chose it.
        source: PlanSource::Prescribed,
    })
}

/// The paces the plan was worked out with.
///
/// The **threshold in force on the day of the adoption** gives the
/// threshold pace when it carries one (B4). Everything else comes from the
/// method's own rule on the athlete's race, read through the ladder. A
/// session whose pace cannot be worked out has **no pace with a reason** —
/// never a zero pace.
fn plan_paces(
    input: &AdoptionInput,
    goal_distance_m: f64,
    tune_up_distance_m: Option<f64>,
) -> PlanPaces {
    let ladder = training_paces::ladder(&LadderInput {
        race: input.race.clone(),
        goal_distance_m: Some(goal_distance_m),
        max_heart_rate: None,
        resting_heart_rate: None,
        long_run: LongRunKind::Standard,
    });
    let rung = |kind| {
        ladder
            .rung(kind)
            .map(|rung| rung.pace)
            .unwrap_or(PaceValue::Missing(MissingInput::NoRaceTime))
    };

    // The threshold pace: the threshold's own band when it carries one,
    // and the method's rule on the race when it does not.
    let threshold = match input.threshold.as_ref().and_then(|choice| choice.pace) {
        Some(band) => PaceValue::Known(band),
        None => rung(RungKind::LactateThreshold),
    };

    let long = ladder.rung(RungKind::LongRun);
    let long_start = long
        .map(|rung| rung.pace)
        .unwrap_or(PaceValue::Missing(MissingInput::NoRaceTime));
    let long_finish = long
        .map(|rung| rung.finish_pace)
        .unwrap_or(PaceValue::Missing(MissingInput::NoRaceTime));

    // The VO₂max pace: the 3K-to-5K race pace. The engine reads the 5K
    // equivalent — the fastest of the distances the method's own relations
    // give — and the 3K pace is not one of them.
    let vo2 = match equivalent_pace(&ladder, crate::vo2::FIVE_K) {
        Some(pace) => PaceValue::Known(PaceBand::single(pace)),
        None => PaceValue::Missing(MissingInput::NoRaceTime),
    };

    // The speed-repetition pace: the 800 m-to-mile race pace. The mile is
    // not one of the ladder's equivalents, so it is read from the
    // athlete's own race by the same relation the ladder uses.
    let speed = match input.race.as_ref().and_then(mile_race_pace) {
        Some(pace) => PaceValue::Known(PaceBand::single(pace)),
        None => PaceValue::Missing(MissingInput::NoRaceTime),
    };

    // The tune-up race's own pace: the race's distance, as the athlete's
    // own race gives it.
    let tune_up = match tune_up_distance_m.and_then(|distance| equivalent_pace(&ladder, distance)) {
        Some(pace) => PaceValue::Known(PaceBand::single(pace)),
        None => PaceValue::Missing(MissingInput::NoRaceTime),
    };

    PlanPaces {
        threshold,
        long_start,
        long_finish,
        recovery: rung(RungKind::Recovery),
        vo2,
        speed,
        race: rung(RungKind::GoalRace),
        tune_up,
    }
}

/// The pace of a session, and the second end of a session with two.
fn pace_for(kind: &SessionKind, paces: &PlanPaces) -> (PaceValue, PaceValue) {
    let none = PaceValue::NoneStated;
    match kind {
        // The general-aerobic day: the method fixes a heart-rate band and
        // no pace.
        SessionKind::GeneralAerobic | SessionKind::GeneralAerobicSpeed => (none, none),
        // The rest day is not a run: it carries no pace at all.
        SessionKind::RestCross | SessionKind::Rest => (none, none),
        // The long run: from the start pace to the finish pace.
        SessionKind::Endurance => (paces.long_start, paces.long_finish),
        // The progression long run: the long run, then the last miles at
        // the threshold pace.
        SessionKind::ProgressionLongRun => (paces.long_start, paces.threshold),
        SessionKind::Threshold => (paces.threshold, none),
        SessionKind::Vo2Max => (paces.vo2, none),
        SessionKind::SpeedRepetitions => (paces.speed, none),
        SessionKind::Recovery => (paces.recovery, none),
        SessionKind::RacePace | SessionKind::RacePaceSpeed => (paces.race, none),
        SessionKind::TuneUpRace => (paces.tune_up, none),
        SessionKind::GoalRace => (paces.race, none),
        // The app's own kinds are not the method's: an adopted plan's
        // sessions always carry one of the method's.
        _ => (none, none),
    }
}

/// What the plan was paced with.
fn pacing_of(input: &AdoptionInput) -> Pacing {
    let threshold = input.threshold.as_ref().and_then(|choice| {
        choice.render().map(|value| PacingSource {
            value,
            date: choice.origin_date.unwrap_or(choice.entered_on),
        })
    });
    let race = input.race.as_ref().map(|race| PacingSource {
        value: training_paces::race_label(race.distance_m).to_string(),
        date: race.date,
    });
    let missing = match input.race {
        Some(_) => None,
        None => MissingInput::NoRaceTime.reason(),
    };
    Pacing {
        threshold,
        race,
        missing,
    }
}

/// The pace of an equivalent race, in seconds per kilometre.
fn equivalent_pace(ladder: &training_paces::Ladder, distance_m: f64) -> Option<f64> {
    ladder
        .equivalents
        .iter()
        .find(|equivalent| (equivalent.distance_m - distance_m).abs() < 1.0)
        .and_then(|equivalent| equivalent.riegel_pace_s_per_km())
}

/// The pace of a mile race, worked out from the athlete's own race.
///
/// The speed repetitions are run at the 800 m-to-mile race pace. The mile
/// is not one of the distances the ladder's own equivalents carry, so it is
/// read by the same relation the ladder uses for every other distance.
fn mile_race_pace(race: &Race) -> Option<f64> {
    let time_s = crate::vo2::riegel_time(race.time_s, race.distance_m, METRES_PER_MILE)?;
    Some(time_s / (METRES_PER_MILE / 1_000.0))
}

/// The Monday of the calendar week containing a day.
fn week_start(day: Date) -> Date {
    day.add_days(-weekday_index(day.weekday()))
}

/// The weekday's index, Monday is zero.
fn weekday_index(weekday: Weekday) -> i64 {
    match weekday {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    }
}

/// ## The plan's own reading
///
/// A reading is the plan answering for one week: the seven sessions with
/// their kinds, their distances and their paces, the plan's own source, the
/// threshold the plan was paced with, and the tier's entry requirement
/// beside the athlete's own mileage.
///
/// Every sentence is the core's, and the dates are the caller's: the core
/// never formats a day for a screen, so the reading takes the words it
/// needs in [`DateWords`].
///
/// The entry requirement and the athlete's own mileage are **three facts
/// side by side, unranked** (B2). The app states them and never compares
/// them, never marks one as the right one, and never says which level
/// fits.
/// The caller's own wording for the days the plan's lines carry.
///
/// The api writes dates in its own form; the core keeps its own canonical
/// form inside itself and asks for the words it needs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DateWords {
    /// The threshold's day: "3 Nov 2026".
    pub threshold: Option<String>,
    /// The race's day: "12 Oct 2026".
    pub race: Option<String>,
}

/// The plan's own reading of one week.
#[derive(Clone, Debug, PartialEq)]
pub struct WeekReading {
    /// The week's number: eleven down to one, zero for race week, and
    /// negative for the weeks after the race.
    pub week: i32,
    /// The weeks to go.
    pub weeks_to_go: Option<i32>,
    /// The week's own label: "3 weeks to go", "race week".
    pub label: String,
    /// The first day of the week.
    pub start_date: Date,
    /// The last day of the week.
    pub end_date: Date,
    /// The week's printed volume, rendered: "42 km".
    pub printed_volume: Option<String>,
    /// The week's own reconciliation: the seven days against the mileage
    /// the book prints beside them.
    pub volume: Option<String>,
    /// The week's seven sessions, in date order.
    pub sessions: Vec<SessionReading>,
    /// The plan's own source.
    pub source: PlanReading,
    /// What the plan was paced with.
    pub pacing: PacingReading,
    /// The tier's entry requirement beside the athlete's own mileage.
    pub entry: EntryReading,
    /// The shift, when the race week was moved to the athlete's race day.
    pub shift: Option<ShiftReading>,
}

/// One session, as the plan reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionReading {
    /// The day the session falls on.
    pub date: Date,
    /// The day's name: "Wednesday".
    pub weekday: String,
    /// The runner's wording for the kind: "tempo".
    pub kind: String,
    /// The method's own name, when the kind is one of the method's.
    pub method_kind: Option<String>,
    /// The distance, rendered: "17.7 km".
    pub distance: Option<String>,
    /// The session's pace: "4:16–4:21 /km".
    pub pace: Option<String>,
    /// The second end of a session with two: the long run's finish pace.
    pub finish_pace: Option<String>,
    /// The cell's own detail: "3 × 8 min, jog 3".
    pub detail: Option<String>,
    /// The line the surface leads with: "17.7 km · tempo".
    pub label: String,
    /// The line with the pace beside it, when the session has one.
    pub line: Option<String>,
    /// The reason the session has no pace, when it has none.
    pub no_pace: Option<String>,
    /// On a progression long run: the last miles at the threshold pace.
    pub lt_last: Option<String>,
}

/// The plan's own source: the programme the plan was adopted from.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanReading {
    /// Where the plan came from. An adopted programme is a prescription:
    /// the athlete chose it.
    pub source: PlanSource,
    /// The line: "the Pfitzinger half marathon tier 2 plan".
    pub line: String,
    /// The catalogue id the plan was adopted from.
    pub template_id: String,
    /// The catalogue version the plan was adopted from.
    pub template_version: String,
}

/// What the plan was paced with.
#[derive(Clone, Debug, PartialEq)]
pub struct PacingReading {
    /// The threshold the plan was paced with: its value and the day it was
    /// read.
    pub threshold: Option<PacingSource>,
    /// The race the plan's own rule was read from.
    pub race: Option<PacingSource>,
    /// The reason the sessions that need a race have no pace.
    pub missing: Option<String>,
    /// The line: "paced with the threshold of 3 Nov 2026 · 165 bpm".
    pub threshold_line: Option<String>,
    /// The line: "paced with the plan's rule on your 10K of 12 Oct 2026".
    pub rule_line: Option<String>,
}

/// The tier's entry requirement, beside the athlete's own mileage.
///
/// Three facts, side by side, unranked. The app states them and never
/// compares them.
#[derive(Clone, Debug, PartialEq)]
pub struct EntryReading {
    /// "this level asks for 64.4-80.5 km a week".
    pub requirement: Option<String>,
    /// "this plan runs 74-101.4 km a week".
    pub plan_band: Option<String>,
    /// "your last 4 weeks: 60.8, 61.1, 50.8, 6 km".
    pub athlete: Option<String>,
}

/// The race week, moved to the day the athlete races.
#[derive(Clone, Debug, PartialEq)]
pub struct ShiftReading {
    /// The days the week moved: negative is earlier.
    pub days: i64,
    /// The line: "your race is on Saturday · this week's sessions are
    /// earlier a day".
    pub line: Option<String>,
    /// The sessions that are not in this week.
    pub dropped: Vec<String>,
}

/// The plan's own reading of a week.
///
/// `None` when the plan has no such week.
pub fn read_week(plan: &AdoptedPlan, week: i32, words: &DateWords) -> Option<WeekReading> {
    let found = plan.weeks.iter().find(|candidate| candidate.week == week)?;
    Some(reading_of(plan, found, words))
}

/// The plan's own reading of the week containing a day.
///
/// `None` when the day is not in the plan.
pub fn read_week_at(plan: &AdoptedPlan, day: Date, words: &DateWords) -> Option<WeekReading> {
    let found = plan
        .weeks
        .iter()
        .find(|week| day >= week.start_date && day <= week.end_date)?;
    Some(reading_of(plan, found, words))
}

/// The reading of one week.
fn reading_of(plan: &AdoptedPlan, week: &AdoptedWeek, words: &DateWords) -> WeekReading {
    let sessions = week.sessions.iter().map(session_reading).collect();
    WeekReading {
        week: week.week,
        weeks_to_go: week.weeks_to_go,
        label: week.label.clone(),
        start_date: week.start_date,
        end_date: week.end_date,
        printed_volume: units::render_distance(week.printed_volume_m),
        volume: volume_line(week),
        sessions,
        source: plan_reading(plan),
        pacing: pacing_reading(&plan.pacing, words),
        entry: entry_reading(plan),
        shift: week.shift.as_ref().map(shift_reading),
    }
}

/// One session, as the plan reads it.
fn session_reading(session: &AdoptedSession) -> SessionReading {
    let kind = session.kind.word();
    let distance = units::render_distance(session.distance_m);
    let label = match &distance {
        Some(distance) => strings::render(
            "statement.plan.session.label",
            &[("distance", distance), ("kind", &kind)],
        )
        .unwrap_or_default(),
        None => strings::render("statement.plan.session.no_target", &[("kind", &kind)])
            .unwrap_or_default(),
    };
    let pace = session.pace.render();
    let finish_pace = session.finish_pace.render();
    let line = match &pace {
        Some(pace) => strings::render(
            "statement.plan.session.line",
            &[("label", &label), ("pace", pace)],
        ),
        None => None,
    };
    let lt_last = session.lt_last_m.and_then(|metres| {
        let distance = units::render_distance(Some(metres))?;
        strings::render(
            "statement.plan.progression.last",
            &[("distance", &distance)],
        )
    });
    SessionReading {
        date: session.date,
        weekday: session.date.weekday().name().to_string(),
        kind,
        method_kind: session.kind.method_name(),
        distance,
        pace,
        finish_pace,
        detail: session.detail.clone(),
        label,
        line,
        no_pace: no_pace_reason(&session.kind, &session.pace),
        lt_last,
    }
}

/// The reason a session has no pace, when it has none.
///
/// A session with no pace is a session with no pace, with the reason —
/// never a zero pace. A day that is not a run has no pace to explain.
fn no_pace_reason(kind: &SessionKind, pace: &PaceValue) -> Option<String> {
    if !kind.is_run() {
        return None;
    }
    match pace {
        PaceValue::Known(_) => None,
        PaceValue::NoneStated => {
            strings::text("statement.plan.pace.band_only").map(|text| text.to_string())
        }
        PaceValue::Missing(missing) => missing.reason(),
    }
}

/// The week's own reconciliation.
fn volume_line(week: &AdoptedWeek) -> Option<String> {
    let cells = week.volume_check.cells_m?;
    let cells = units::render_distance(Some(cells))?;
    match week.printed_volume_m {
        Some(printed) => {
            let printed = units::render_distance(Some(printed))?;
            if week.volume_check.reconciles == Some(true) {
                strings::render("statement.plan.volume.check", &[("volume", &cells)])
            } else {
                strings::render(
                    "statement.plan.volume.gap",
                    &[("cells", &cells), ("volume", &printed)],
                )
            }
        }
        None => strings::render("statement.plan.volume.check", &[("volume", &cells)]),
    }
}

/// The plan's own source.
fn plan_reading(plan: &AdoptedPlan) -> PlanReading {
    let line = strings::render(
        "statement.plan.programme",
        &[("method", &plan.method), ("label", &plan.label)],
    )
    .unwrap_or_default();
    PlanReading {
        source: plan.source,
        line,
        template_id: plan.template_id.clone(),
        template_version: plan.template_version.clone(),
    }
}

/// What the plan was paced with.
fn pacing_reading(pacing: &Pacing, words: &DateWords) -> PacingReading {
    let threshold_line = match (&pacing.threshold, &words.threshold) {
        (Some(threshold), Some(date)) => strings::render(
            "statement.plan.pacing.threshold",
            &[("date", date), ("value", &threshold.value)],
        ),
        _ => None,
    };
    let rule_line = match (&pacing.race, &words.race) {
        (Some(race), Some(date)) => strings::render(
            "statement.plan.pacing.rule",
            &[("distance", &race.value), ("date", date)],
        ),
        _ => None,
    };
    PacingReading {
        threshold: pacing.threshold.clone(),
        race: pacing.race.clone(),
        missing: pacing.missing.clone(),
        threshold_line,
        rule_line,
    }
}

/// The tier's entry requirement beside the athlete's own mileage.
fn entry_reading(plan: &AdoptedPlan) -> EntryReading {
    let requirement = plan.entry_mileage.as_ref().and_then(|range| {
        let range = range.render()?;
        strings::render("statement.plan.entry.requirement", &[("range", &range)])
    });
    let plan_band = plan.plan_mileage.as_ref().and_then(|range| {
        let range = range.render()?;
        strings::render("statement.plan.entry.plan", &[("range", &range)])
    });
    let athlete = athlete_mileage_line(&plan.athlete_mileage_m);
    EntryReading {
        requirement,
        plan_band,
        athlete,
    }
}

/// The athlete's own recent weekly mileage.
fn athlete_mileage_line(mileage_m: &[f64]) -> Option<String> {
    if mileage_m.is_empty() {
        return None;
    }
    let values: Vec<String> = mileage_m
        .iter()
        .filter(|metres| metres.is_finite() && **metres >= 0.0)
        .map(|metres| units::format_distance_km_short(*metres))
        .collect();
    if values.is_empty() {
        return None;
    }
    let count = values.len().to_string();
    let values = values.join(", ");
    strings::render(
        "statement.plan.entry.athlete",
        &[("n", &count), ("values", &values)],
    )
}

/// The race week, moved to the day the athlete races.
fn shift_reading(shift: &RaceWeekShift) -> ShiftReading {
    let direction = if shift.days < 0 {
        "statement.plan.race_week.earlier"
    } else {
        "statement.plan.race_week.later"
    };
    let direction = strings::text(direction).unwrap_or_default().to_string();
    let line = strings::render(
        "statement.plan.race_week.shifted",
        &[("weekday", &shift.race_weekday), ("direction", &direction)],
    );
    let dropped = shift
        .dropped
        .iter()
        .filter_map(|dropped| {
            strings::render(
                "statement.plan.race_week.dropped",
                &[("day", &dropped.weekday), ("kind", &dropped.kind)],
            )
        })
        .collect();
    ShiftReading {
        days: shift.days,
        line,
        dropped,
    }
}
