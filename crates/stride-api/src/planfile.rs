//! The plan files the athlete may import.
//!
//! Four formats are read: **markdown**, **CSV**, **JSON** and the **Garmin
//! workout schedule** CSV. A file that cannot be read is **refused**: the
//! import says which line it could not read, and it changes nothing.
//!
//! ## The rule about kinds
//!
//! A session's kind comes from the words in the file, and only from words
//! the vocabulary knows:
//!
//! | the file says | the session is |
//! |---|---|
//! | easy | easy |
//! | long | long |
//! | rest | rest |
//! | tempo | tempo |
//! | interval | interval |
//! | race | race |
//! | strength | strength |
//! | quality | quality |
//! | anything else | **unclassified** |
//!
//! **A workout the vocabulary does not know is unclassified.** It keeps the
//! athlete's own words as its title, and it is never given a kind it did
//! not have. The old implementation guessed "easy" for anything it did not
//! recognise, and a guess in a plan is a plan that says something false.
//!
//! ## The rule about distances
//!
//! A distance is a number and a unit: "10", "10 km", "10.5km", "10000 m".
//! A bare number is **kilometres**, because that is how a plan is written;
//! a number with "m" is metres. A distance that cannot be read is not a
//! distance, and the session keeps whatever else the file said about it.

use stride_core::date::Date;

use crate::error::{ApiError, ApiResult};

/// The formats an import reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// A plan written as markdown: a table, or a list of days.
    Markdown,
    /// A plan written as CSV, with a header row.
    Csv,
    /// A plan written as JSON: the app's own form, or a list of sessions.
    Json,
    /// The Garmin workout schedule: a CSV whose rows are workouts.
    GarminCsv,
}

impl Format {
    /// The format with this name.
    pub fn from_name(name: &str) -> Option<Format> {
        match name
            .trim()
            .to_ascii_lowercase()
            .replace(['-', ' '], "_")
            .as_str()
        {
            "markdown" | "md" => Some(Format::Markdown),
            "csv" => Some(Format::Csv),
            "json" => Some(Format::Json),
            "garmin" | "garmin_csv" | "garmin_workout_schedule" | "workout_schedule" => {
                Some(Format::GarminCsv)
            }
            _ => None,
        }
    }

    /// The format a file name names.
    pub fn from_file_name(name: &str) -> Option<Format> {
        let extension = name.rsplit_once('.').map(|(_, extension)| extension)?;
        Format::from_name(extension)
    }

    /// The format's name, as it travels.
    pub fn name(self) -> &'static str {
        match self {
            Format::Markdown => "markdown",
            Format::Csv => "csv",
            Format::Json => "json",
            Format::GarminCsv => "garmin_csv",
        }
    }
}

/// One session, as a plan file carries it.
#[derive(Clone, Debug, PartialEq)]
pub struct FileSession {
    /// The day of the session.
    pub date: Date,
    /// The kind, as the vocabulary names it.
    pub session_type: String,
    /// The session's title, in the athlete's own words.
    pub title: Option<String>,
    /// The target distance, in metres. `None` when the file carried none.
    pub target_distance_m: Option<f64>,
    /// The session's details, as the file carried them.
    pub details: Option<String>,
}

/// A plan, as a file carries it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilePlan {
    /// The plan's name, when the file carried one.
    pub name: Option<String>,
    /// The plan's first day, when the file carried one.
    pub start_date: Option<String>,
    /// The plan's last day, when the file carried one.
    pub end_date: Option<String>,
    /// The plan's goal, when the file carried one.
    pub goal: Option<String>,
    /// The sessions the file carried.
    pub sessions: Vec<FileSession>,
}

/// The words a plan file may use for a session's kind.
///
/// The list is closed. A word that is not in it names nothing, and the
/// session it is on is unclassified.
const KINDS: [(&str, &str); 8] = [
    ("easy", "easy"),
    ("long", "long"),
    ("rest", "rest"),
    ("tempo", "tempo"),
    ("interval", "interval"),
    ("race", "race"),
    ("strength", "strength"),
    ("quality", "quality"),
];

/// The kind a text names.
///
/// `None` when the text names no kind the vocabulary knows: the session is
/// then unclassified, and it keeps the athlete's own words.
pub fn kind_of_text(text: &str) -> Option<&'static str> {
    for word in words_of(text) {
        for (needle, kind) in KINDS.iter() {
            if word == *needle {
                return Some(kind);
            }
        }
    }
    None
}

/// The words of a text, in lower case, without their punctuation.
fn words_of(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect()
}

/// Read a plan file.
///
/// The file is read whole: a file that cannot be read is refused, and the
/// refusal says where the reading stopped.
pub fn read(format: Format, content: &str) -> ApiResult<FilePlan> {
    match format {
        Format::Markdown => read_markdown(content),
        Format::Csv => read_csv(content, false),
        Format::GarminCsv => read_csv(content, true),
        Format::Json => read_json(content),
    }
}

/// Read a plan written as markdown.
///
/// Two shapes are read: a **table** — the same table a CSV holds, between
/// pipes — and a **list** of days, one per line, with the date, the kind
/// and the distance on the line.
fn read_markdown(content: &str) -> ApiResult<FilePlan> {
    let lines: Vec<&str> = content.lines().collect();
    let has_table = lines.iter().any(|line| line.trim_start().starts_with('|'));
    if has_table {
        let rows: Vec<Vec<String>> = lines
            .iter()
            .filter(|line| line.trim_start().starts_with('|'))
            .map(|line| {
                line.trim()
                    .trim_start_matches('|')
                    .trim_end_matches('|')
                    .split('|')
                    .map(|cell| cell.trim().to_string())
                    .collect()
            })
            .collect();
        return read_rows(&rows, false);
    }

    let mut sessions: Vec<FileSession> = Vec::new();
    for (number, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let is_item = trimmed.starts_with('-')
            || trimmed.starts_with('*')
            || trimmed.starts_with('#')
            || trimmed.starts_with('+');
        if !is_item {
            continue;
        }
        let text = trimmed
            .trim_start_matches(['-', '*', '#', '+', ' '])
            .trim_start_matches(|character: char| {
                character == '-'
                    || character == '*'
                    || character == '#'
                    || character == '+'
                    || character == ' '
            })
            .trim();
        if text.is_empty() {
            continue;
        }
        sessions.push(session_of_line(text, number + 1)?);
    }
    if sessions.is_empty() {
        return Err(ApiError::bad_request(
            "the markdown plan could not be read: it holds no day",
        ));
    }
    Ok(FilePlan {
        sessions,
        ..FilePlan::default()
    })
}

/// One line of a markdown list: a date, a kind and a distance.
fn session_of_line(text: &str, line: usize) -> ApiResult<FileSession> {
    let date = date_in(text).ok_or_else(|| {
        ApiError::bad_request(format!(
            "the markdown plan could not be read: line {line} has no date"
        ))
    })?;
    let kind = kind_of_text(text);
    // The distance is looked for beside the date, not inside it: a date
    // carries numbers of its own.
    let without_date = text.replacen(&date.to_iso(), " ", 1);
    let target_distance_m = distance_in(&without_date);
    Ok(FileSession {
        date,
        session_type: kind.unwrap_or("unclassified").to_string(),
        title: if kind.is_none() {
            Some(text.to_string())
        } else {
            None
        },
        target_distance_m,
        details: None,
    })
}

/// Read a plan written as CSV — or as the Garmin workout schedule.
///
/// The CSV has a header row, and the columns are found by their names: the
/// day, the kind, the distance, the title and the notes. The Garmin
/// schedule is the same table with the workout's name where the kind is.
fn read_csv(content: &str, garmin: bool) -> ApiResult<FilePlan> {
    let rows: Vec<Vec<String>> = split_csv(content);
    if rows.is_empty() {
        return Err(ApiError::bad_request(
            "the plan could not be read: the file is empty",
        ));
    }
    read_rows(&rows, garmin)
}

/// Read a table of rows: the first row is the header.
fn read_rows(rows: &[Vec<String>], garmin: bool) -> ApiResult<FilePlan> {
    let header = match rows.first() {
        Some(header) => header,
        None => {
            return Err(ApiError::bad_request(
                "the plan could not be read: the file has no header",
            ))
        }
    };
    let names: Vec<String> = header
        .iter()
        .map(|name| name.trim().to_ascii_lowercase())
        .collect();
    let date_column = column_of(&names, &["date", "day", "start date", "start_date"]);
    let distance_column = column_of(
        &names,
        &[
            "distance",
            "distance_km",
            "distance km",
            "km",
            "target",
            "target_distance",
            "target distance",
            "target_distance_m",
        ],
    );
    // The Garmin schedule names the workout where a plan names the kind.
    let kind_column = if garmin {
        column_of(
            &names,
            &[
                "workout name",
                "workout",
                "name",
                "session",
                "session name",
                "type",
            ],
        )
    } else {
        column_of(
            &names,
            &[
                "type",
                "session",
                "session type",
                "kind",
                "workout",
                "activity",
                "workout name",
            ],
        )
    };

    let date_column = date_column.ok_or_else(|| {
        ApiError::bad_request("the plan could not be read: the file has no date column")
    })?;

    let mut sessions: Vec<FileSession> = Vec::new();
    for (index, row) in rows.iter().enumerate().skip(1) {
        if row.iter().all(|cell| cell.trim().is_empty()) {
            continue;
        }
        // The line under a markdown table's header is a rule, not a row.
        if row.iter().all(|cell| {
            cell.chars()
                .all(|character| character == '-' || character == ':')
        }) {
            continue;
        }
        let line = index + 1;
        let date_text = cell_of(row, date_column).ok_or_else(|| {
            ApiError::bad_request(format!(
                "the plan could not be read: line {line} has no date"
            ))
        })?;
        let date = Date::parse(date_text.trim()).ok_or_else(|| {
            ApiError::bad_request(format!(
                "the plan could not be read: line {line} has a date that could not be read"
            ))
        })?;
        let words = kind_column
            .and_then(|column| cell_of(row, column))
            .unwrap_or_default();
        let kind = kind_of_text(&words);
        let target_distance_m = distance_column
            .and_then(|column| cell_of(row, column))
            .and_then(|cell| distance_in(&cell));
        let title = kind_column
            .and_then(|column| cell_of(row, column))
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .filter(|_| kind.is_none());
        let details = column_of(&names, &["notes", "note", "details", "description"])
            .and_then(|column| cell_of(row, column))
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty());
        sessions.push(FileSession {
            date,
            session_type: kind.unwrap_or("unclassified").to_string(),
            title,
            target_distance_m,
            details,
        });
    }
    if sessions.is_empty() {
        return Err(ApiError::bad_request(
            "the plan could not be read: it holds no session",
        ));
    }
    Ok(FilePlan {
        sessions,
        ..FilePlan::default()
    })
}

/// Read a plan written as JSON.
///
/// Two shapes are read: the app's own form — an object with a `sessions`
/// array — and a plain array of sessions.
fn read_json(content: &str) -> ApiResult<FilePlan> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|_| ApiError::bad_request("the plan could not be read: it is not JSON"))?;
    let (object, sessions) = match &value {
        serde_json::Value::Array(items) => (None, items.clone()),
        serde_json::Value::Object(map) => {
            let items = map
                .get("sessions")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            (Some(map.clone()), items)
        }
        _ => {
            return Err(ApiError::bad_request(
                "the plan could not be read: it is not a list of sessions",
            ))
        }
    };
    if sessions.is_empty() {
        return Err(ApiError::bad_request(
            "the plan could not be read: it holds no session",
        ));
    }
    let mut read: Vec<FileSession> = Vec::with_capacity(sessions.len());
    for (index, session) in sessions.iter().enumerate() {
        let line = index + 1;
        let date_text = ["date", "day", "start_date", "startDate"]
            .iter()
            .find_map(|name| session.get(*name).and_then(|value| value.as_str()))
            .ok_or_else(|| {
                ApiError::bad_request(format!(
                    "the plan could not be read: session {line} has no date"
                ))
            })?;
        let date = Date::parse(date_text.trim()).ok_or_else(|| {
            ApiError::bad_request(format!(
                "the plan could not be read: session {line} has a date that could not be read"
            ))
        })?;
        let words = [
            "session_type",
            "sessionType",
            "type",
            "kind",
            "workout",
            "name",
            "title",
        ]
        .iter()
        .find_map(|key| session.get(*key).and_then(|value| value.as_str()))
        .unwrap_or_default();
        let kind = kind_of_text(words);
        let target_distance_m = [
            "target_distance_m",
            "targetDistanceM",
            "distance_m",
            "distanceM",
            "distance_km",
            "distanceKm",
            "distance",
            "target",
        ]
        .iter()
        .find_map(|key| session.get(*key))
        .and_then(|value| match value {
            serde_json::Value::Number(number) => number.as_f64(),
            serde_json::Value::String(text) => distance_in(text),
            _ => None,
        });
        let title = ["title", "name", "notes", "details"]
            .iter()
            .find_map(|key| session.get(*key).and_then(|value| value.as_str()))
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .filter(|_| kind.is_none());
        read.push(FileSession {
            date,
            session_type: kind.unwrap_or("unclassified").to_string(),
            title,
            target_distance_m,
            details: None,
        });
    }
    let name = object
        .as_ref()
        .and_then(|map| map.get("name"))
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    let start_date = object
        .as_ref()
        .and_then(|map| map.get("start_date"))
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    let end_date = object
        .as_ref()
        .and_then(|map| map.get("end_date"))
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    let goal = object
        .as_ref()
        .and_then(|map| map.get("goal"))
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    Ok(FilePlan {
        name,
        start_date,
        end_date,
        goal,
        sessions: read,
    })
}

/// The index of the first column with one of these names.
fn column_of(names: &[String], wanted: &[&str]) -> Option<usize> {
    names
        .iter()
        .position(|name| wanted.iter().any(|candidate| name == candidate))
}

/// One cell of a row.
fn cell_of(row: &[String], index: usize) -> Option<String> {
    row.get(index).map(|cell| cell.to_string())
}

/// The date in a text, if it holds one.
fn date_in(text: &str) -> Option<Date> {
    for word in text.split(|character: char| !(character.is_alphanumeric() || character == '-')) {
        if let Some(date) = Date::parse(word) {
            return Some(date);
        }
    }
    None
}

/// The distance in a text, in metres.
///
/// A bare number is kilometres — that is how a plan is written. A number
/// with "m" is metres, and a number with "km" is kilometres.
fn distance_in(text: &str) -> Option<f64> {
    let characters: Vec<char> = text.trim().chars().collect();
    let mut index = 0;
    while index < characters.len() && !characters[index].is_ascii_digit() {
        index += 1;
    }
    let mut number = String::new();
    while index < characters.len()
        && (characters[index].is_ascii_digit() || characters[index] == '.')
    {
        number.push(characters[index]);
        index += 1;
    }
    if number.is_empty() {
        return None;
    }
    let value: f64 = number.parse().ok()?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    let unit: String = characters[index..]
        .iter()
        .skip_while(|character| character.is_whitespace())
        .take_while(|character| character.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_lowercase();
    match unit.as_str() {
        "" | "km" | "k" | "kms" | "kilometres" | "kilometers" => Some(value * 1000.0),
        "m" | "metres" | "meters" => Some(value),
        _ => None,
    }
}

/// The rows of a CSV document.
///
/// The reader is the plain one: a field may be quoted, a quoted field may
/// hold a comma, and a doubled quote inside a quoted field is one quote.
fn split_csv(content: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut characters = content.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '"' if quoted => {
                if characters.peek() == Some(&'"') {
                    characters.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            }
            '"' => quoted = true,
            ',' if !quoted => {
                row.push(std::mem::take(&mut field));
            }
            '\r' => {}
            '\n' if !quoted => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            other => field.push(other),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_workout_the_vocabulary_does_not_know_is_unclassified(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = read(Format::Csv, "date,type\n2026-05-12,Hill repeats\n")?;
        assert_eq!(plan.sessions.len(), 1);
        assert_eq!(plan.sessions[0].session_type, "unclassified");
        assert_eq!(plan.sessions[0].title.as_deref(), Some("Hill repeats"));
        Ok(())
    }

    #[test]
    fn a_distance_is_read_in_kilometres_and_in_metres() {
        assert_eq!(distance_in("10"), Some(10_000.0));
        assert_eq!(distance_in("10 km"), Some(10_000.0));
        assert_eq!(distance_in("10.5km"), Some(10_500.0));
        assert_eq!(distance_in("10000 m"), Some(10_000.0));
        assert_eq!(distance_in("none"), None);
    }

    #[test]
    fn a_markdown_list_is_read() -> Result<(), Box<dyn std::error::Error>> {
        let plan = read(
            Format::Markdown,
            "- 2026-05-12 — easy 10 km\n- 2026-05-13: long 20 km\n",
        )?;
        assert_eq!(plan.sessions.len(), 2);
        assert_eq!(plan.sessions[0].session_type, "easy");
        assert_eq!(plan.sessions[0].target_distance_m, Some(10_000.0));
        assert_eq!(plan.sessions[1].session_type, "long");
        Ok(())
    }

    #[test]
    fn a_file_without_a_date_is_refused() {
        let answer = read(Format::Csv, "date,type\n,easy\n");
        assert!(answer.is_err());
    }
}
