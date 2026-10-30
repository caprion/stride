//! The plan importer.
//!
//! An athlete writes their plan as markdown, as a CSV or as JSON, and the
//! app turns it into the plan tables. The reference is
//! `backend/app/ingest/plan_importer.py`, and this is a port of it — quirks
//! included, because the reference is the definition of what the importer
//! does.
//!
//! **Two of those quirks are worth knowing about**, because they look like
//! bugs and are reproduced here on purpose:
//!
//! * the phase dates never parse. The reference looks for a three-letter
//!   month (`[A-Z][a-z]{2}`) and then looks that three-letter name up in a
//!   table of *full* month names, so a phase's `start_date` and `end_date`
//!   are always `null`;
//! * a week with no dated rows gets a template instead — four sessions a
//!   week — and a warning saying so.
//!
//! The importer is a pure function over the document: no clock, no
//! randomness. The only thing it needs from the caller is the year, when
//! the document does not name one.
//!
//! **A row that names no kind of session is unknown.** "Session 4" and
//! "Zona 3" are not easy runs; the importer says `unknown`, and the plan
//! engine, the comparison and the screens all read that as "not known"
//! rather than as an easy run somebody prescribed.

use serde_json::{json, Map, Value};

use crate::backend::{
    Backend, PlanBlockFields, PlanPhaseFields, PlanSessionFields, PlanWeekFields,
};
use crate::error::BackendError;

/// The months the reference knows, by their full names.
const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// The days of the week, as the reference maps them.
const DAYS: [(&str, i64); 8] = [
    ("Mon", 0),
    ("Tue", 1),
    ("Wed", 2),
    ("Thu", 3),
    ("Fri", 4),
    ("Sat", 5),
    ("Sun", 6),
    ("Sat/Sun", 6),
];

/// Parse a plan document.
///
/// `year` is the year the document is about when it does not name one.
pub fn import_plan(
    content: &str,
    format: &str,
    name: Option<&str>,
    year: i32,
) -> Result<Value, String> {
    match format {
        "json" => {
            let data: Value =
                serde_json::from_str(content).map_err(|err| format!("invalid JSON: {err}"))?;
            let object = data
                .as_object()
                .ok_or_else(|| "the document is not an object".to_string())?;
            if has_keys(object, &["block", "phases", "weeks", "sessions"]) {
                // The reference hands the document back as it is.
                return Ok(Value::Object(object.clone()));
            }
            Ok(normalize_json(object, name))
        }
        "csv" => {
            let rows = parse_csv(content);
            let mut data = Map::new();
            data.insert("sessions".to_string(), Value::Array(rows));
            data.insert("name".to_string(), json!(name.unwrap_or("Imported plan")));
            Ok(normalize_json(&data, name))
        }
        "markdown" => Ok(import_markdown(content, name, year)),
        _ => Err("format must be markdown, csv, or json".to_string()),
    }
}

/// Whether an object has all of these keys.
fn has_keys(object: &Map<String, Value>, keys: &[&str]) -> bool {
    keys.iter().all(|key| object.contains_key(*key))
}

/// The reference's `_normalize_json`.
fn normalize_json(data: &Map<String, Value>, name: Option<&str>) -> Value {
    let block_name = name
        .map(str::to_string)
        .or_else(|| data.get("name").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| "Imported plan".to_string());
    json!({
        "block": {
            "name": block_name,
            "start_date": Value::Null,
            "end_date": Value::Null,
            "status": "active",
            "source": "imported",
        },
        "phases": data.get("phases").cloned().unwrap_or(Value::Array(Vec::new())),
        "weeks": data.get("weeks").cloned().unwrap_or(Value::Array(Vec::new())),
        "sessions": data.get("sessions").cloned().unwrap_or(Value::Array(Vec::new())),
        "warnings": Value::Array(Vec::new()),
    })
}

/// A CSV document, read as the reference reads it.
fn parse_csv(content: &str) -> Vec<Value> {
    let mut lines = content.lines();
    let Some(header) = lines.next() else {
        return Vec::new();
    };
    let columns: Vec<String> = header
        .split(',')
        .map(|name| name.trim().to_string())
        .collect();
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cells: Vec<&str> = line.split(',').collect();
        let mut row = Map::new();
        for (index, column) in columns.iter().enumerate() {
            let value = cells.get(index).map(|cell| cell.trim()).unwrap_or("");
            row.insert(column.clone(), json!(value));
        }
        rows.push(Value::Object(row));
    }
    rows
}

/// The reference's `parse_pace`: written `min:sec` paces, in seconds per
/// kilometre.
pub fn parse_pace(value: Option<&str>) -> (Option<i64>, Option<i64>) {
    let Some(value) = value else {
        return (None, None);
    };
    if value.is_empty() {
        return (None, None);
    }
    // The `/km` suffix is removed before the paces are read.
    let text = strip_km_suffix(value);
    let paces = find_paces(&text);
    let numbers: Vec<i64> = paces
        .iter()
        .take(2)
        .map(|(minutes, seconds)| minutes * 60 + seconds)
        .collect();
    match numbers.len() {
        0 => (None, None),
        1 => (Some(numbers[0]), Some(numbers[0])),
        _ => (
            Some(numbers[0].min(numbers[1])),
            Some(numbers[0].max(numbers[1])),
        ),
    }
}

/// Remove a `/km` suffix, wherever it is written.
fn strip_km_suffix(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let lower = value.to_lowercase();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'/' && lower[index..].starts_with("/km") {
            // The suffix and the whitespace before it go.
            while out.ends_with(char::is_whitespace) {
                out.pop();
            }
            index += 3;
            // A word boundary: `km` must not be the start of a longer word.
            if index < bytes.len() && bytes[index].is_ascii_alphanumeric() {
                out.push_str("/km");
                index += 3;
            }
            continue;
        }
        let character = value[index..].chars().next().unwrap_or(' ');
        out.push(character);
        index += character.len_utf8();
    }
    out
}

/// The `min:sec` paces in a text, in order.
fn find_paces(text: &str) -> Vec<(i64, i64)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            // One or two digits, then a colon, then two digits.
            let mut digits = 0;
            while index + digits < bytes.len() && bytes[index + digits].is_ascii_digit() {
                digits += 1;
            }
            let start = index;
            let minute_end = (start + digits).min(bytes.len());
            // More than two digits is not a pace.
            let minutes = if minute_end - start > 2 {
                String::new()
            } else {
                text[start..minute_end].to_string()
            };
            if !minutes.is_empty()
                && minute_end < bytes.len()
                && bytes[minute_end] == b':'
                && minute_end + 2 < bytes.len()
                && bytes[minute_end + 1].is_ascii_digit()
                && bytes[minute_end + 2].is_ascii_digit()
            {
                let preceded_by_digit = start > 0 && bytes[start - 1].is_ascii_digit();
                let followed_by_digit =
                    minute_end + 3 < bytes.len() && bytes[minute_end + 3].is_ascii_digit();
                if !preceded_by_digit && !followed_by_digit {
                    let seconds: i64 = text[minute_end + 1..minute_end + 3]
                        .parse()
                        .unwrap_or_default();
                    out.push((minutes.parse().unwrap_or_default(), seconds));
                    index = minute_end + 3;
                    continue;
                }
            }
        }
        index += 1;
    }
    out
}

/// The reference's `parse_distance`: a distance written in a title.
pub fn parse_distance(value: Option<&str>) -> Option<f64> {
    let value = value?;
    if value.is_empty() {
        return None;
    }
    if let Some(kilometres) = find_kilometres(value) {
        return Some(kilometres * 1000.0);
    }
    if let Some(kilometres) = find_kilometre_abbreviation(value) {
        return Some(kilometres * 1000.0);
    }
    if value.to_lowercase().contains("half marathon") {
        return Some(21_097.5);
    }
    None
}

/// The first `N km` in a text, in kilometres.
///
/// The reference's pattern is `N km`, with an optional `N-M` range, and it
/// takes the *first* number of the range.
fn find_kilometres(value: &str) -> Option<f64> {
    let lower = value.to_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find("km") {
        let start = from + found;
        let after = start + 2;
        if after >= lower.len() || !lower[after..].starts_with(|c: char| c.is_alphanumeric()) {
            if let Some(number) = number_before_km(&value[..start]) {
                return Some(number);
            }
        }
        from = start + 2;
    }
    None
}

/// The first `N k` in a text, in kilometres.
fn find_kilometre_abbreviation(value: &str) -> Option<f64> {
    let lower = value.to_lowercase();
    let bytes = lower.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'k' {
            let after = index + 1;
            let boundary_after = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
            if boundary_after {
                if let Some(number) = number_before_unit(&value[..index], true) {
                    return Some(number);
                }
            }
        }
        index += 1;
    }
    None
}

/// The number before a `km`, in kilometres.
///
/// The reference's pattern is `N km`, with an optional `N-M` range: the
/// range's *first* number is the distance. The number must be the last
/// thing before the unit, so `5:00/km` is not a distance.
fn number_before_km(head: &str) -> Option<f64> {
    number_before_unit(head, false)
}

/// The number before a unit.
///
/// With `word_boundary` the number must start at a word boundary, which is
/// what the reference's `\b` asks for in the `N k` pattern.
fn number_before_unit(head: &str, word_boundary: bool) -> Option<f64> {
    let text = head.trim_end();
    let bytes = text.as_bytes();
    // The number at the end of the text: the digits are scanned back to
    // where they start.
    let mut start = text.len();
    while start > 0 && (bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.') {
        start -= 1;
    }
    if start == text.len() {
        // The text does not end with a number, so it is not a distance.
        return None;
    }
    let value: f64 = text[start..].parse().ok()?;
    let before = text[..start].trim_end();
    if let Some(range) = before.strip_suffix('-') {
        // A range: `6-8 km` is six kilometres.
        return range.trim_end().parse::<f64>().ok();
    }
    if word_boundary
        && before
            .chars()
            .last()
            .is_some_and(|character| character.is_alphanumeric())
    {
        return None;
    }
    Some(value)
}

/// Every number in a text, as the reference's `findall` finds them.
fn all_numbers(text: &str) -> Vec<f64> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if index < bytes.len() && bytes[index] == b'.' {
                let mut probe = index + 1;
                while probe < bytes.len() && bytes[probe].is_ascii_digit() {
                    probe += 1;
                }
                if probe > index + 1 {
                    index = probe;
                }
            }
            if let Ok(number) = text[start..index].parse::<f64>() {
                out.push(number);
            }
            continue;
        }
        index += 1;
    }
    out
}

/// The reference's `_clean`: strip the emphasis and the emoji from a cell.
fn clean(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '*' | '_' | '⭐' | '⚠' | '🏁' | '✅' | '🔴' | '🟡' | '🟢' => {}
            '→' => out.push_str(" to "),
            '—' => out.push('-'),
            other => out.push(other),
        }
    }
    out.trim().to_string()
}

/// The reference's `_tables`: the markdown tables in a document.
fn tables(text: &str) -> Vec<Vec<Vec<String>>> {
    let mut tables: Vec<Vec<Vec<String>>> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in text.lines() {
        if !line.contains('|') {
            if !rows.is_empty() {
                tables.push(std::mem::take(&mut rows));
            }
            continue;
        }
        let trimmed = line.trim().trim_matches('|');
        let cells: Vec<String> = trimmed.split('|').map(|cell| clean(cell.trim())).collect();
        if cells.iter().all(|cell| is_separator(cell)) {
            continue;
        }
        rows.push(cells);
    }
    if !rows.is_empty() {
        tables.push(rows);
    }
    tables
}

/// Whether a cell is a table separator (`---`, `:---`, …).
fn is_separator(cell: &str) -> bool {
    !cell.is_empty()
        && cell
            .chars()
            .all(|character| character == '-' || character == ':' || character == ' ')
}

/// The reference's `_phase_key`.
fn phase_key(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for character in text.chars() {
        match character {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            _ if depth == 0 => out.push(character),
            _ => {}
        }
    }
    clean(&out)
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase()
}

/// The reference's `_week_date`: a date range in a heading.
fn week_date(heading: &str, year: i32) -> Option<(String, String)> {
    let start = heading.find('(')?;
    let end = heading.rfind(')')?;
    if end <= start {
        return None;
    }
    let inside = &heading[start + 1..end];
    // `Month D - Month D`, with the second month optional.
    let (first, second) = match inside.split_once('-') {
        Some((first, second)) => (first.trim(), second.trim()),
        None => return None,
    };
    let (month_one, day_one) = split_month_day(first)?;
    let (month_two, day_two) = match split_month_day(second) {
        Some((month, day)) => (month, day),
        None => (month_one.clone(), day_one),
    };
    let month_one = month_number(&month_one)?;
    let month_two = month_number(&month_two)?;
    let year_two = if month_two < month_one {
        year + 1
    } else {
        year
    };
    let first = chrono::NaiveDate::from_ymd_opt(year, month_one, day_one)?;
    let second = chrono::NaiveDate::from_ymd_opt(year_two, month_two, day_two)?;
    Some((
        first.format("%Y-%m-%d").to_string(),
        second.format("%Y-%m-%d").to_string(),
    ))
}

/// A month name and a day, from `"March 2"`.
fn split_month_day(text: &str) -> Option<(String, u32)> {
    let (name, day) = text.split_once(' ')?;
    let day: u32 = day.trim().parse().ok()?;
    Some((name.trim().to_lowercase(), day))
}

/// The number of a month, from its full name.
fn month_number(name: &str) -> Option<u32> {
    MONTHS
        .iter()
        .position(|month| *month == name)
        .map(|index| index as u32 + 1)
}

/// The reference's `_classify`: what kind of session a row describes.
///
/// A row whose text names no kind of session at all — "Session 4", "Zona
/// 3" — is **unknown**, not easy. The importer used to call it "easy",
/// which is a guess: the plan then said the athlete had an easy run
/// prescribed, and every verdict, every pace band and every comparison
/// downstream treated that guess as the athlete's own words.
fn classify(title: &str, details: &str) -> String {
    let haystack = format!("{title} {details}").to_lowercase();
    let lowered = title.to_lowercase();
    if lowered.contains("rest") {
        // "Rest or 2km walk" is not a rest day: it is a light recovery
        // session with an optional distance.
        if has_word(&lowered, &["or", "walk", "run", "jog", "shakeout"]) {
            return "recovery".to_string();
        }
        return "rest".to_string();
    }
    if haystack.contains("strength")
        || haystack.contains("session a")
        || haystack.contains("session b")
    {
        return "strength".to_string();
    }
    if lowered.contains("race") {
        return "race".to_string();
    }
    // Distance and the word "long" decide a long run before anything else.
    if haystack.contains("long") || parse_distance(Some(title)).unwrap_or(0.0) > 14_000.0 {
        return "long".to_string();
    }
    if has_repetition(&haystack)
        || ["interval", "fartlek", "sharpener"]
            .iter()
            .any(|word| haystack.contains(word))
    {
        return "interval".to_string();
    }
    if lowered.contains("easy") || lowered.contains("recovery") {
        return "easy".to_string();
    }
    if ["tempo", "threshold", "steady"]
        .iter()
        .any(|word| haystack.contains(word))
    {
        return "tempo".to_string();
    }
    if haystack.contains("recovery") {
        return "recovery".to_string();
    }
    // Nothing in the row names a kind of session, so the kind is unknown.
    "unknown".to_string()
}

/// Whether a text contains any of these words, whole.
fn has_word(text: &str, words: &[&str]) -> bool {
    for word in words {
        let mut from = 0;
        while let Some(found) = text[from..].find(word) {
            let start = from + found;
            let end = start + word.len();
            let before_ok = start == 0
                || !text[..start]
                    .chars()
                    .next_back()
                    .is_some_and(|character| character.is_alphanumeric());
            let after_ok = end >= text.len()
                || !text[end..]
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_alphanumeric());
            if before_ok && after_ok {
                return true;
            }
            from = start + 1;
        }
    }
    false
}

/// Whether a text has a repetition like `6 x 1km`.
fn has_repetition(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let mut probe = index;
        while probe < bytes.len() && bytes[probe] == b' ' {
            probe += 1;
        }
        if probe < bytes.len() && (bytes[probe] == b'x' || bytes[probe] == b'X') {
            probe += 1;
            while probe < bytes.len() && bytes[probe] == b' ' {
                probe += 1;
            }
            if probe < bytes.len() && bytes[probe].is_ascii_digit() {
                let _ = start;
                return true;
            }
        }
    }
    false
}

/// The paces of a phase, by the kind of session they are for.
fn phase_paces(phase: &Map<String, Value>) -> Vec<(String, Option<i64>, Option<i64>)> {
    ["easy", "tempo", "interval", "long"]
        .iter()
        .map(|kind| {
            (
                kind.to_string(),
                phase
                    .get(&format!("pace_{kind}_low"))
                    .and_then(Value::as_i64),
                phase
                    .get(&format!("pace_{kind}_high"))
                    .and_then(Value::as_i64),
            )
        })
        .collect()
}

/// The reference's `_session`: one planned session.
fn session(
    title: &str,
    details: &str,
    day: chrono::NaiveDate,
    paces: &[(String, Option<i64>, Option<i64>)],
) -> Value {
    let kind = classify(title, details);
    let pace_source = format!("{title} {details}");
    let race_pace = if kind == "race" {
        find_race_pace(&pace_source)
    } else {
        None
    };
    let pace = if kind == "race" {
        match &race_pace {
            Some(pace) => parse_pace(Some(pace)),
            None if pace_source.to_lowercase().contains("half marathon") => (
                Some((2.0_f64 * 3600.0 / 21.0975).round() as i64),
                Some(((2.0_f64 * 3600.0 + 3.0 * 60.0) / 21.0975).round() as i64),
            ),
            None => (None, None),
        }
    } else {
        parse_pace(Some(&pace_source))
    };
    let pace = if pace == (None, None) {
        // The phase's pace bands belong to the kinds they were written for.
        // An unknown session belongs to none of them, so it gets no band:
        // the easy band would be the same guess, one step later.
        let group = match kind.as_str() {
            "unknown" => None,
            "easy" | "recovery" => Some("easy"),
            "long" => Some("long"),
            "tempo" => Some("tempo"),
            "interval" => Some("interval"),
            _ => Some("easy"),
        };
        group
            .and_then(|group| paces.iter().find(|(name, _, _)| name == group))
            .map(|(_, low, high)| (*low, *high))
            .unwrap_or((None, None))
    } else {
        pace
    };
    let repetition = if kind == "interval" {
        find_repetition(title)
    } else {
        None
    };
    let distance = match &repetition {
        Some((count, size, unit)) => {
            let size: f64 = *size;
            let metres = count * size * if unit == "km" { 1000.0 } else { 1.0 };
            Some(metres)
        }
        None => parse_distance(Some(details)),
    };
    let target = distance
        .or_else(|| parse_distance(Some(title)))
        .or_else(|| parse_distance(Some(details)));
    json!({
        "date": day.format("%Y-%m-%d").to_string(),
        "session_type": kind,
        "title": title.chars().take(160).collect::<String>(),
        "details": if details.is_empty() { Value::Null } else { json!(details) },
        "target_distance_m": target,
        "pace_low_s_km": pace.0,
        "pace_high_s_km": pace.1,
        "source": "imported",
    })
}

/// The `min:sec/km` pace of a race.
fn find_race_pace(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find("/km") {
        let start = from + found;
        let head = &text[..start];
        // The pace is the last `min:sec` before the suffix.
        let paces = find_paces(head);
        if let Some((minutes, seconds)) = paces.last() {
            return Some(format!("{minutes}:{seconds:02}"));
        }
        from = start + 3;
    }
    None
}

/// A repetition like `6 x 1km` in a title.
fn find_repetition(title: &str) -> Option<(f64, f64, String)> {
    let lower = title.to_lowercase();
    let bytes = lower.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let count: f64 = lower[start..index].parse().ok()?;
        let mut probe = index;
        while probe < bytes.len() && bytes[probe] == b' ' {
            probe += 1;
        }
        if probe >= bytes.len() || (bytes[probe] != b'x' && bytes[probe] != b'X') {
            continue;
        }
        probe += 1;
        while probe < bytes.len() && bytes[probe] == b' ' {
            probe += 1;
        }
        let size_start = probe;
        while probe < bytes.len() && (bytes[probe].is_ascii_digit() || bytes[probe] == b'.') {
            probe += 1;
        }
        if probe == size_start {
            continue;
        }
        let size: f64 = lower[size_start..probe].parse().ok()?;
        let mut unit_start = probe;
        while unit_start < bytes.len() && bytes[unit_start] == b' ' {
            unit_start += 1;
        }
        let unit = if lower[unit_start..].starts_with("km") {
            "km".to_string()
        } else if lower[unit_start..].starts_with('m') {
            "m".to_string()
        } else {
            continue;
        };
        return Some((count, size, unit));
    }
    None
}

/// The reference's `import_plan` for a markdown document.
fn import_markdown(content: &str, name: Option<&str>, year: i32) -> Value {
    let year = year_of(content, year);
    let tables = tables(content);

    // The phases, in the order they were read.
    let mut phase_rows: Vec<(String, Map<String, Value>)> = Vec::new();
    for table in &tables {
        let Some(header) = table.first() else {
            continue;
        };
        let joined = header.join(" ").to_lowercase();
        if header
            .first()
            .is_some_and(|cell| phase_key(cell) == "phase")
            && header.len() >= 4
            && joined.contains("duration")
        {
            for row in &table[1..] {
                if row.len() < 4 {
                    continue;
                }
                let weeks = week_range(&row[1]);
                let kilometres = all_numbers(&row[3]);
                let mut phase = Map::new();
                phase.insert("name".into(), json!(row[0]));
                phase.insert(
                    "week_start".into(),
                    weeks.map(|(start, _)| json!(start)).unwrap_or(Value::Null),
                );
                phase.insert(
                    "week_end".into(),
                    weeks.map(|(_, end)| json!(end)).unwrap_or(Value::Null),
                );
                phase.insert("duration".into(), json!(row[2]));
                phase.insert(
                    "target_weekly_km_min".into(),
                    kilometres
                        .first()
                        .map(|value| json!(value))
                        .unwrap_or(Value::Null),
                );
                phase.insert(
                    "target_weekly_km_max".into(),
                    kilometres
                        .last()
                        .map(|value| json!(value))
                        .unwrap_or(Value::Null),
                );
                set_phase(&mut phase_rows, phase_key(&row[0]), phase);
            }
        } else if header
            .first()
            .is_some_and(|cell| phase_key(cell) == "phase")
            && joined.contains("easy")
        {
            for row in &table[1..] {
                if row.len() >= 5 {
                    let key = phase_key(&row[0]);
                    let mut phase = phase_of(&phase_rows, &key).cloned().unwrap_or_default();
                    if !phase.contains_key("name") {
                        phase.insert("name".into(), json!(row[0]));
                    }
                    for (index, kind) in ["easy", "tempo", "interval", "long"].iter().enumerate() {
                        let (low, high) = parse_pace(Some(&row[index + 1]));
                        phase.insert(format!("pace_{kind}_low"), json!(low));
                        phase.insert(format!("pace_{kind}_high"), json!(high));
                    }
                    set_phase(&mut phase_rows, key, phase);
                }
            }
        }
    }

    // The week progression: the volume of each week.
    let mut progression: Vec<(i64, Map<String, Value>)> = Vec::new();
    for table in &tables {
        let Some(header) = table.first() else {
            continue;
        };
        if !header.first().is_some_and(|cell| phase_key(cell) == "week") {
            continue;
        }
        if !header
            .iter()
            .any(|cell| cell.to_lowercase().contains("running"))
        {
            continue;
        }
        for row in &table[1..] {
            let Some(number) = leading_number(&row[0]) else {
                continue;
            };
            let phase = row.get(1).map(|cell| json!(cell)).unwrap_or(Value::Null);
            let distance = row
                .get(2)
                .and_then(|cell| parse_distance(Some(cell.as_str())))
                .or_else(|| {
                    // A volume written without a unit still reads as one.
                    row.get(2)
                        .map(|cell| all_numbers(cell.as_str()))
                        .and_then(|numbers| numbers.first().copied())
                });
            let key_workout = row.get(4).map(|cell| json!(cell)).unwrap_or(Value::Null);
            let mut entry = Map::new();
            entry.insert("phase".into(), phase);
            entry.insert(
                "target_km".into(),
                distance
                    .map(|metres| json!(metres / 1000.0))
                    .unwrap_or(Value::Null),
            );
            entry.insert("key_workout".into(), key_workout);
            match progression.iter_mut().find(|(week, _)| *week == number) {
                Some((_, existing)) => {
                    *existing = entry;
                }
                None => progression.push((number, entry)),
            }
        }
    }

    // The phases, with their dates.
    let mut phases: Vec<Map<String, Value>> = Vec::new();
    for (index, (_, phase)) in phase_rows.iter().enumerate() {
        let mut phase = phase.clone();
        let duration = phase
            .get("duration")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        // The reference looks for a three-letter month and then looks that
        // name up in a table of full month names, so this never parses.
        let dates = phase_dates(&duration);
        phase.insert(
            "start_date".into(),
            dates
                .as_ref()
                .map(|(start, _)| json!(start))
                .unwrap_or(Value::Null),
        );
        phase.insert(
            "end_date".into(),
            dates
                .as_ref()
                .map(|(_, end)| json!(end))
                .unwrap_or(Value::Null),
        );
        phase.insert("_idx".into(), json!(index));
        phases.push(phase);
    }

    // The week headings and the dated rows under them.
    let mut week_heading: Option<i64> = None;
    let mut detail_rows: Vec<(i64, String, String, String)> = Vec::new();
    for line in content.lines() {
        if let Some(heading) = week_heading_of(line) {
            week_heading = Some(heading);
            continue;
        }
        if line.trim_start().starts_with('#') {
            week_heading = None;
            continue;
        }
        let Some(week) = week_heading else {
            continue;
        };
        if !line.contains('|') {
            continue;
        }
        let trimmed = line.trim().trim_matches('|');
        let cells: Vec<String> = trimmed.split('|').map(|cell| clean(cell.trim())).collect();
        if cells.is_empty() || cells[0] == "Day" {
            continue;
        }
        if cells.iter().all(|cell| is_separator(cell)) {
            continue;
        }
        if cells.len() >= 2 {
            detail_rows.push((
                week,
                cells[0].clone(),
                cells[1].clone(),
                cells.get(2).cloned().unwrap_or_default(),
            ));
        }
    }

    // The weeks, and the sessions in them.
    let mut weeks: Vec<Value> = Vec::new();
    let mut sessions: Vec<Value> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut progression: Vec<(i64, Map<String, Value>)> = progression;
    progression.sort_by_key(|(week, _)| *week);

    for (number, entry) in &progression {
        let phase_name = entry
            .get("phase")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let key = phase_key(&phase_name);
        let mut phase = phase_of(&phase_rows, &key).cloned().unwrap_or_default();
        // A phase whose name is a prefix of another one still counts.
        if phase.is_empty() {
            for (candidate_key, candidate) in &phase_rows {
                if key.starts_with(candidate_key.as_str())
                    || candidate_key.as_str().starts_with(key.as_str())
                {
                    phase = candidate.clone();
                    break;
                }
            }
        }
        // The phase whose week range holds this week wins.
        let start_week = phase.get("week_start").and_then(Value::as_i64);
        let end_week = phase.get("week_end").and_then(Value::as_i64);
        if let (Some(_), Some(_)) = (start_week, end_week) {
            if let Some(candidate) = phases.iter().find(|candidate| {
                let candidate_start = candidate.get("week_start").and_then(Value::as_i64);
                let candidate_end = candidate.get("week_end").and_then(Value::as_i64);
                candidate_start.is_some_and(|value| value <= *number)
                    && candidate_end.map_or(true, |value| *number <= value)
            }) {
                phase = candidate.clone();
            }
        }
        let parent = phase.get("_idx").and_then(Value::as_i64).unwrap_or(0);
        let start = match (phase.get("start_date").and_then(Value::as_str), start_week) {
            (Some(start_date), Some(start_week)) => {
                chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
                    .ok()
                    .map(|date| date + chrono::Duration::weeks(*number - start_week))
            }
            _ => None,
        }
        .unwrap_or_else(|| {
            // The date comes from the plan's declared week progression, which
            // begins on the fifth of January.
            chrono::NaiveDate::from_ymd_opt(year, 1, 5)
                .map(|date| date + chrono::Duration::weeks(*number - 1))
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year, 1, 5).unwrap_or_default())
        });
        let end = start + chrono::Duration::days(6);
        weeks.push(json!({
            "week_number": number,
            "start_date": start.format("%Y-%m-%d").to_string(),
            "end_date": end.format("%Y-%m-%d").to_string(),
            "target_km": entry.get("target_km").cloned().unwrap_or(Value::Null),
            "key_workout": entry.get("key_workout").cloned().unwrap_or(Value::Null),
            "phase_index": parent,
        }));

        let paces = phase_paces(&phase);
        let rows: Vec<&(i64, String, String, String)> = detail_rows
            .iter()
            .filter(|(week, _, _, _)| *week == *number)
            .collect();
        if rows.is_empty() {
            // A week with no dated rows gets the documented template.
            let key_workout = entry
                .get("key_workout")
                .and_then(Value::as_str)
                .unwrap_or("Easy running");
            let template = [
                (1i64, "Easy run", "6-8km"),
                (3, "Quality", key_workout),
                (5, "Long run", "10km"),
                (6, "Easy run", "3-5km"),
            ];
            for (offset, title, details) in template {
                sessions.push(session(
                    title,
                    details,
                    start + chrono::Duration::days(offset),
                    &paces,
                ));
            }
            warnings.push(format!(
                "Week {number}: day sessions generated from the documented weekly template; no dated rows in source"
            ));
            continue;
        }
        for (_, day_name, title, details) in rows {
            let Some(offset) = DAYS
                .iter()
                .find(|(name, _)| *name == day_name.as_str())
                .map(|(_, offset)| *offset)
            else {
                warnings.push(format!(
                    "Week {number}: could not map day {}",
                    serde_json::to_string(&day_name).unwrap_or_default()
                ));
                continue;
            };
            sessions.push(session(
                title,
                details,
                start + chrono::Duration::days(offset),
                &paces,
            ));
        }
    }

    for phase in &mut phases {
        phase.remove("_idx");
    }

    let starts: Vec<String> = weeks
        .iter()
        .filter_map(|week| week["start_date"].as_str().map(str::to_string))
        .collect();
    let ends: Vec<String> = weeks
        .iter()
        .filter_map(|week| week["end_date"].as_str().map(str::to_string))
        .collect();
    json!({
        "block": {
            "name": name.unwrap_or("Imported training plan"),
            "start_date": starts
                .iter()
                .min()
                .map(|value| json!(value))
                .unwrap_or(Value::Null),
            "end_date": ends
                .iter()
                .max()
                .map(|value| json!(value))
                .unwrap_or(Value::Null),
            "goal": Value::Null,
            "status": "active",
            "source": "imported",
        },
        "phases": phases,
        "weeks": weeks,
        "sessions": sessions,
        "warnings": warnings,
    })
}

/// The phase with this key, if there is one.
fn phase_of<'a>(
    phases: &'a [(String, Map<String, Value>)],
    key: &str,
) -> Option<&'a Map<String, Value>> {
    phases
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, phase)| phase)
}

/// Put a phase into the list, replacing the one with the same key.
fn set_phase(
    phases: &mut Vec<(String, Map<String, Value>)>,
    key: String,
    phase: Map<String, Value>,
) {
    match phases.iter_mut().find(|(candidate, _)| *candidate == key) {
        Some((_, existing)) => *existing = phase,
        None => phases.push((key, phase)),
    }
}

/// The year the document names, in its first 500 characters.
fn year_of(content: &str, fallback: i32) -> i32 {
    let head: String = content.chars().take(500).collect();
    let bytes = head.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index - start != 4 {
            continue;
        }
        let candidate = &head[start..index];
        if candidate.starts_with("20") {
            if let Ok(year) = candidate.parse::<i32>() {
                return year;
            }
        }
    }
    fallback
}

/// The week range in a cell, e.g. `1 - 4`.
fn week_range(cell: &str) -> Option<(i64, i64)> {
    let bytes = cell.as_bytes();
    let mut index = 0;
    let mut first: Option<i64> = None;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            let number: i64 = cell[start..index].parse().ok()?;
            if first.is_none() {
                first = Some(number);
                // A dash, and then the second number.
                let mut probe = index;
                while probe < bytes.len() && bytes[probe] == b' ' {
                    probe += 1;
                }
                if probe < bytes.len() && bytes[probe] == b'-' {
                    probe += 1;
                    while probe < bytes.len() && bytes[probe] == b' ' {
                        probe += 1;
                    }
                    let second_start = probe;
                    while probe < bytes.len() && bytes[probe].is_ascii_digit() {
                        probe += 1;
                    }
                    if probe > second_start {
                        let second: i64 = cell[second_start..probe].parse().ok()?;
                        return Some((first.unwrap_or_default(), second));
                    }
                }
            }
            continue;
        }
        index += 1;
    }
    None
}

/// The leading number of a cell.
fn leading_number(cell: &str) -> Option<i64> {
    let bytes = cell.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index == 0 {
        return None;
    }
    cell[..index].parse().ok()
}

/// The dates in a phase's duration.
///
/// The reference looks for a three-letter month (`[A-Z][a-z]{2}`) and then
/// looks that three-letter name up in a table of *full* month names, so the
/// two never agree and the phase dates are always `null`. The code is ported
/// as it is, so the behaviour is the reference's behaviour.
fn phase_dates(duration: &str) -> Option<(String, String)> {
    let bytes = duration.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        // A capital letter and two lowercase ones: a three-letter month.
        if !(bytes[index].is_ascii_uppercase()
            && index + 2 < bytes.len()
            && bytes[index + 1].is_ascii_lowercase()
            && bytes[index + 2].is_ascii_lowercase())
        {
            index += 1;
            continue;
        }
        let month_one = &duration[index..index + 3];
        let mut cursor = index + 3;
        let mut whitespace = 0;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
            whitespace += 1;
        }
        if whitespace == 0 {
            index += 1;
            continue;
        }
        let day_one_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor == day_one_start {
            index += 1;
            continue;
        }
        let day_one = &duration[day_one_start..cursor];
        // The dash, with optional spaces around it.
        let mut probe = cursor;
        while probe < bytes.len() && bytes[probe].is_ascii_whitespace() {
            probe += 1;
        }
        if probe >= bytes.len() || bytes[probe] != b'-' {
            index += 1;
            continue;
        }
        probe += 1;
        while probe < bytes.len() && bytes[probe].is_ascii_whitespace() {
            probe += 1;
        }
        // The second month, if there is one.
        let mut month_two = month_one;
        if probe + 2 < bytes.len()
            && bytes[probe].is_ascii_uppercase()
            && bytes[probe + 1].is_ascii_lowercase()
            && bytes[probe + 2].is_ascii_lowercase()
        {
            let mut after = probe + 3;
            while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                after += 1;
            }
            if after > probe + 3 && after < bytes.len() && bytes[after].is_ascii_digit() {
                month_two = &duration[probe..probe + 3];
                probe = after;
            }
        }
        let day_two_start = probe;
        while probe < bytes.len() && bytes[probe].is_ascii_digit() {
            probe += 1;
        }
        if probe == day_two_start {
            index += 1;
            continue;
        }
        let day_two = &duration[day_two_start..probe];
        let heading = format!("({month_one} {day_one}-{month_two} {day_two})");
        return week_date(&heading, 0);
    }
    None
}

/// The week number of a heading, e.g. `## Week 3 - Base (March 9) 30 km`.
fn week_heading_of(line: &str) -> Option<i64> {
    let lower = line.to_lowercase();
    let start = lower.find("week ")?;
    let rest = &line[start + 5..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    // The heading must carry a distance, or it is not a week heading.
    let after = &rest[digits.len()..];
    if !after.contains("km") {
        return None;
    }
    digits.parse().ok()
}

/// The plan a document describes, in the terms the store holds.
#[derive(Debug, Clone)]
pub struct PlanImport {
    /// The block.
    pub block: PlanBlockFields,
    /// The phases.
    pub phases: Vec<PlanPhaseFields>,
    /// The weeks, each with the phase it hangs on.
    pub weeks: Vec<(usize, PlanWeekFields)>,
    /// The sessions, each with the week it hangs on (by its date).
    pub sessions: Vec<PlanSessionFields>,
    /// What the document did not say, in words.
    pub warnings: Vec<String>,
}

/// The plan a document describes.
///
/// The reference: `personal_sync`-free — this is `plan_importer.import_plan`
/// turned into the rows the store holds.
pub fn parse_plan(
    content: &str,
    format: &str,
    name: Option<&str>,
    year: i32,
) -> Result<PlanImport, String> {
    let document = import_plan(content, format, name, year)?;
    let today = format!("{year}-01-01");
    let block_name = document["block"]["name"]
        .as_str()
        .unwrap_or("Imported plan")
        .to_string();
    let block = PlanBlockFields {
        name: block_name,
        start_date: document["block"]["start_date"]
            .as_str()
            .unwrap_or(&today)
            .to_string(),
        end_date: document["block"]["end_date"].as_str().map(str::to_string),
        goal: document["block"]["goal"].as_str().map(str::to_string),
        status: Some("active".to_string()),
        source: Some("imported".to_string()),
        notes: None,
    };
    let phases: Vec<PlanPhaseFields> = document["phases"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|phase| PlanPhaseFields {
            name: phase["name"].as_str().unwrap_or("Phase").to_string(),
            start_date: phase["start_date"].as_str().map(str::to_string),
            end_date: phase["end_date"].as_str().map(str::to_string),
            target_weekly_km_min: phase["target_weekly_km_min"].as_f64(),
            target_weekly_km_max: phase["target_weekly_km_max"].as_f64(),
            target_sessions: phase["target_sessions"].as_i64(),
            pace_easy_low: phase["pace_easy_low"].as_i64().map(|value| value as f64),
            pace_easy_high: phase["pace_easy_high"].as_i64().map(|value| value as f64),
            pace_tempo_low: phase["pace_tempo_low"].as_i64().map(|value| value as f64),
            pace_tempo_high: phase["pace_tempo_high"].as_i64().map(|value| value as f64),
            pace_interval_low: phase["pace_interval_low"]
                .as_i64()
                .map(|value| value as f64),
            pace_interval_high: phase["pace_interval_high"]
                .as_i64()
                .map(|value| value as f64),
            pace_long_low: phase["pace_long_low"].as_i64().map(|value| value as f64),
            pace_long_high: phase["pace_long_high"].as_i64().map(|value| value as f64),
            hr_zone_target: phase["hr_zone_target"].as_i64(),
            notes: None,
        })
        .collect();
    let weeks: Vec<(usize, PlanWeekFields)> = document["weeks"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|week| {
            (
                week["phase_index"].as_i64().unwrap_or(0).max(0) as usize,
                PlanWeekFields {
                    week_number: week["week_number"].as_i64().unwrap_or(1),
                    start_date: week["start_date"].as_str().map(str::to_string),
                    end_date: week["end_date"].as_str().map(str::to_string),
                    target_km: week["target_km"].as_f64(),
                    key_workout: week["key_workout"].as_str().map(str::to_string),
                    notes: None,
                },
            )
        })
        .collect();
    let sessions: Vec<PlanSessionFields> = document["sessions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|session| PlanSessionFields {
            date: session["date"].as_str().unwrap_or_default().to_string(),
            // A row that names no kind of session is unknown. It is not
            // easy: nothing in the document said so.
            session_type: session["session_type"]
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown")
                .to_string(),
            title: session["title"].as_str().map(str::to_string),
            details: session["details"].as_str().map(str::to_string),
            target_distance_m: session["target_distance_m"].as_f64(),
            pace_low_s_km: session["pace_low_s_km"].as_f64(),
            pace_high_s_km: session["pace_high_s_km"].as_f64(),
            hr_zone_target: session["hr_zone_target"].as_i64(),
            status: session["status"].as_str().map(str::to_string),
            source: session["source"].as_str().map(str::to_string),
            notes: None,
        })
        .collect();
    let warnings = document["warnings"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|warning| warning.as_str().map(str::to_string))
        .collect();
    Ok(PlanImport {
        block,
        phases,
        weeks,
        sessions,
        warnings,
    })
}

/// Write a plan into a backend, and answer the way the reference does.
///
/// The reference: `api/plan.import_document`.
pub fn write_plan(backend: &dyn Backend, plan: &PlanImport) -> Result<Value, BackendError> {
    let block = backend.create_plan_block(&plan.block)?;
    let mut phase_ids: Vec<i64> = Vec::new();
    for phase in &plan.phases {
        phase_ids.push(backend.create_plan_phase(block.id, phase)?.id);
    }
    // The weeks, each under its phase. A document with weeks but no phases
    // still needs one to hang them on.
    let mut week_ids: Vec<(i64, String, String)> = Vec::new();
    for (index, week) in &plan.weeks {
        let phase_id = match phase_ids
            .get(*index)
            .copied()
            .or_else(|| phase_ids.first().copied())
        {
            Some(phase_id) => phase_id,
            None => {
                let written = backend.create_plan_phase(
                    block.id,
                    &PlanPhaseFields {
                        name: "Imported".to_string(),
                        ..PlanPhaseFields::default()
                    },
                )?;
                phase_ids.push(written.id);
                written.id
            }
        };
        let written = backend.create_plan_week(phase_id, week)?;
        week_ids.push((
            written.id,
            written.start_date.clone().unwrap_or_default(),
            written.end_date.clone().unwrap_or_default(),
        ));
    }
    // The sessions, each under the week that holds its date.
    let mut session_count = 0usize;
    for session in &plan.sessions {
        let Some(week_id) = week_ids
            .iter()
            .find(|(_, start, end)| {
                !start.is_empty()
                    && !end.is_empty()
                    && session.date.as_str() >= start.as_str()
                    && session.date.as_str() <= end.as_str()
            })
            .map(|(id, _, _)| *id)
        else {
            // A session with no week is not scheduled.
            continue;
        };
        backend.create_plan_session(week_id, session)?;
        session_count += 1;
    }
    Ok(json!({
        "ok": true,
        "block_id": block.id,
        "name": block.name,
        "phases": phase_ids.len(),
        "weeks": week_ids.len(),
        "sessions": session_count,
        "warnings": plan.warnings,
        "error": Value::Null,
    }))
}
