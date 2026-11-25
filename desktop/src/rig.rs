//! The rig's own page.
//!
//! ## What it is
//!
//! The app is the athlete's face. **This page is the host's face**: what
//! version this rig is serving, where the athlete's data is, whether a
//! Garmin key is kept, and the three decisions that only the athlete can
//! make.
//!
//! It is the rig's own landing surface, and it has four parts:
//!
//!   - **the index** — the screens the rig serves, so the new screens are
//!     reachable and named;
//!   - **the status** — the version being served, the database's schema
//!     version, and what the log holds;
//!   - **the storage** — where the files are, and how big the database is;
//!   - **the connection** — whether a Garmin key is kept, and when the log
//!     was last filled from the source.
//!
//! ## What it says
//!
//! **Facts, never verdicts.** Every line is a fact about this machine or
//! about what the athlete has already decided. There is no score, no
//! grade, no ranking, and no recommendation.
//!
//! ## The three decisions
//!
//! The new screens need three things that cannot be worked out from the
//! training: **a race to type, a threshold to choose, a programme to
//! adopt**. The page offers them as the athlete's own choices:
//!
//!   - they are **listed side by side**, in the same shape, with no
//!     preference between them — the page never says which one to take
//!     first, and never grades what the athlete has already chosen;
//!   - the **vitals are never linked to them**: the page states what is in
//!     force and where it came from, and it never says what a choice
//!     implies for the athlete's training;
//!   - the page is **not a wizard**: nothing on it presumes that the
//!     athlete wants any of the three, and every one of them can stay
//!     empty.
//!
//! The race is typed here, because the app has no screen for it and this
//! page is the desktop's own. The threshold and the programme are made in
//! the app, where the four candidates and the four programmes live; the
//! page says which screen, and what is in force now.
//!
//! ## Where the numbers come from
//!
//! **The router's own answers.** The races, the threshold, the programme
//! and the log's counts are read from `stride-api`'s routes, so the page
//! says what the app says — the core's words, the same rows — and the rig
//! holds no second reading of its own.
//!
//! ## The key
//!
//! The page says **whether** a Garmin key is kept, and where the file is.
//! It never says what the key is: the key's value has no path onto this
//! page, and the page's own tests hold it to that.

use std::fmt::Write;
use std::path::PathBuf;

use serde_json::Value;

use crate::api::{Request, Router};
use crate::workspace::Workspace;

/// The race labels the app knows.
///
/// **The list is closed** — the label names the distance, and the app
/// never invents a distance for a race it cannot name.
pub const RACE_LABELS: [&str; 6] = stride_store::model::race_label::ALL;

/// The facts the rig's page states.
#[derive(Debug, Clone, Default)]
pub struct RigFacts {
    /// The version this rig is serving.
    pub version: String,
    /// The schema version of the database.
    pub schema_version: Option<i64>,
    /// The directory the rig keeps its files in.
    pub data_dir: PathBuf,
    /// The database file.
    pub database: PathBuf,
    /// The size of the database file.
    pub database_bytes: Option<u64>,
    /// The file the database key lives in.
    pub database_key: PathBuf,
    /// The file the Garmin key may live in.
    pub garmin_key: PathBuf,
    /// Whether a Garmin key is kept.
    pub key_kept: bool,
    /// The first-run state: "fresh", "skipped" or "connected".
    pub first_run: String,
    /// The runs in the log.
    pub runs: Option<i64>,
    /// The wellness days in the log.
    pub wellness_days: Option<i64>,
    /// The races the athlete has typed.
    pub races: Vec<RaceFact>,
    /// The threshold in force.
    pub threshold: Option<ThresholdFact>,
    /// The programme in force.
    pub programme: Option<ProgrammeFact>,
    /// When the log was last filled from the source.
    pub last_sync: Option<String>,
    /// A line about the last thing that happened on the rig.
    pub notice: Option<Notice>,
}

/// A race the athlete typed.
#[derive(Debug, Clone)]
pub struct RaceFact {
    /// The athlete's own label: "10K".
    pub label: String,
    /// The finish time, in the core's words.
    pub time: String,
    /// The day of the race, in the app's own words.
    pub date: String,
    /// Whether the paces come from this race.
    pub anchor: bool,
}

/// The threshold in force.
#[derive(Debug, Clone)]
pub struct ThresholdFact {
    /// The value, in the core's words.
    pub value: String,
    /// Where the choice came from, in the runner's words.
    pub source: String,
    /// The day the choice was made.
    pub entered_on: String,
}

/// The programme in force.
#[derive(Debug, Clone)]
pub struct ProgrammeFact {
    /// The programme's own name: "half marathon tier 2".
    pub label: String,
    /// Whose method it is: "Pfitzinger".
    pub method: String,
    /// The goal race's own distance.
    pub goal: String,
    /// The day of the goal race.
    pub race_date: String,
    /// The day the programme was adopted.
    pub adopted_on: String,
}

/// A line about the last thing that happened on the rig.
#[derive(Debug, Clone)]
pub struct Notice {
    /// Whether it went through.
    pub ok: bool,
    /// The line itself.
    pub text: String,
}

/// The screens the rig serves, and where each one is.
///
/// **The new screens are here too** — the ladder, the cost curve, the
/// ledger, the threshold and the programmes — so the rig's own index says
/// what the app now holds.
const SCREENS: [(&str, &str); 12] = [
    ("Today", "/"),
    ("Runs", "/runs"),
    ("Progress", "/progress"),
    ("The pace", "/progress/pace"),
    ("The effort curve", "/progress/curve"),
    ("Where the time went", "/progress/zone-time"),
    ("My running", "/profile"),
    ("Body", "/body"),
    ("Plan", "/plan"),
    ("Adopting a programme", "/plan/bases"),
    ("Your threshold", "/settings?section=training"),
    ("Backup and restore", "/rig/tools"),
];

/// Read the facts the rig's page states.
///
/// **Every number comes from the router.** The rig reads the answers the
/// app reads and states them; it computes none of them.
pub fn facts(workspace: &Workspace, router: &dyn Router) -> RigFacts {
    let health = get(router, "/health");
    let sync = get(router, "/sync/status");

    RigFacts {
        version: text_at(&health, &["version"]).unwrap_or_default(),
        schema_version: workspace.store().schema_version().ok(),
        data_dir: workspace.data_dir().to_path_buf(),
        database: workspace.store().path().to_path_buf(),
        database_bytes: workspace.store().size_bytes(),
        database_key: crate::paths::database_key_path(workspace.data_dir()),
        garmin_key: workspace.garmin_key_path(),
        key_kept: workspace.keys().is_set(),
        first_run: first_run_word(workspace),
        runs: count_at(&health, "activities"),
        wellness_days: count_at(&health, "wellness_days"),
        races: races_of(&get(router, "/athlete/races")),
        threshold: threshold_of(&get(router, "/insights/threshold")),
        programme: programme_of(&get(router, "/plan/adopt")),
        last_sync: text_at(&sync, &["last_sync_at"]),
        notice: None,
    }
}

/// The first-run state, as a word.
fn first_run_word(workspace: &Workspace) -> String {
    match workspace.first_run() {
        crate::workspace::FirstRun::Fresh => "fresh".to_string(),
        crate::workspace::FirstRun::Skipped => "skipped".to_string(),
        crate::workspace::FirstRun::Connected => "connected".to_string(),
    }
}

/// Ask the router for one route, and read the answer as JSON.
fn get(router: &dyn Router, path: &str) -> Value {
    let call = Request {
        method: "GET",
        path,
        query: "",
        body: &[],
    };
    let answer = router.call(&call);
    serde_json::from_slice(&answer.body).unwrap_or(Value::Null)
}

/// A string field of an answer.
fn text_at(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for step in path {
        current = current.get(step)?;
    }
    current.as_str().map(str::to_string)
}

/// A count in an answer.
fn count_at(value: &Value, name: &str) -> Option<i64> {
    value.get(name).and_then(Value::as_i64)
}

/// The races the athlete has typed.
fn races_of(answer: &Value) -> Vec<RaceFact> {
    let Some(rows) = answer.get("races").and_then(Value::as_array) else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(|row| {
            Some(RaceFact {
                label: row.get("label")?.as_str()?.to_string(),
                time: rendered(row.get("time"))?,
                date: row
                    .get("date_label")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                anchor: row.get("anchor").and_then(Value::as_bool).unwrap_or(false),
            })
        })
        .collect()
}

/// A rendered value, in the core's words.
fn rendered(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    value
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// The threshold in force.
fn threshold_of(answer: &Value) -> Option<ThresholdFact> {
    let choice = answer.get("in_force").filter(|value| value.is_object())?;
    Some(ThresholdFact {
        value: choice
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        source: choice
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        entered_on: choice
            .get("entered_on")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
}

/// The programme in force.
fn programme_of(answer: &Value) -> Option<ProgrammeFact> {
    let plan = answer.get("plan")?;
    Some(ProgrammeFact {
        label: plan.get("label").and_then(Value::as_str)?.to_string(),
        method: plan
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        goal: plan
            .get("goal_label")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        race_date: plan
            .get("race_date_label")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        adopted_on: plan
            .get("adopted_on")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
}

/// The rig's own page.
pub fn page(facts: &RigFacts) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str(&head());
    out.push_str("<main>\n");
    out.push_str("<p class=\"mark\">Stride</p>\n");
    out.push_str("<h1>This rig</h1>\n");
    let _ = writeln!(
        out,
        "<p class=\"lede\">Version {version}. Stored on this device: your history is in this \
         machine's own files, and nowhere else.</p>",
        version = escape(&facts.version)
    );
    if let Some(notice) = &facts.notice {
        let _ = writeln!(
            out,
            "<p class=\"notice\" data-kind=\"{kind}\">{text}</p>",
            kind = if notice.ok { "ok" } else { "refused" },
            text = escape(&notice.text)
        );
    }
    out.push_str(&index_section());
    out.push_str(&status_section(facts));
    out.push_str(&storage_section(facts));
    out.push_str(&connection_section(facts));
    out.push_str(&decisions_section(facts));
    out.push_str("</main>\n");
    out.push_str(FOOT);
    out
}

/// The head of the page.
fn head() -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <meta name=\"referrer\" content=\"no-referrer\">\n\
         <title>Stride - this rig</title>\n<style>\n{}\n</style>\n</head>\n<body>\n",
        STYLE
    )
}

/// The index: the screens the rig serves.
fn index_section() -> String {
    let mut out = String::new();
    out.push_str("<section id=\"index\">\n<h2>What this rig serves</h2>\n<ul class=\"links\">\n");
    for (label, path) in SCREENS {
        let _ = writeln!(
            out,
            "<li><a href=\"{path}\">{label}</a></li>",
            path = escape(path),
            label = escape(label)
        );
    }
    out.push_str("</ul>\n</section>\n");
    out
}

/// The status: the version, the schema and what the log holds.
fn status_section(facts: &RigFacts) -> String {
    let mut out = String::new();
    out.push_str("<section id=\"status\">\n<h2>What this rig holds</h2>\n<dl>\n");
    let _ = writeln!(
        out,
        "<dt>The version</dt><dd>{}</dd>",
        escape(&facts.version)
    );
    let schema = match facts.schema_version {
        Some(version) => format!("schema version {version}"),
        None => "the schema version could not be read".to_string(),
    };
    let _ = writeln!(out, "<dt>The database</dt><dd>{}</dd>", escape(&schema));
    let _ = writeln!(out, "<dt>The log</dt><dd>{}</dd>", escape(&counts(facts)));
    let _ = writeln!(
        out,
        "<dt>The first run</dt><dd>{}</dd>",
        escape(&first_run_line(facts))
    );
    out.push_str("</dl>\n</section>\n");
    out
}

/// The counts of the log, in the app's own words.
fn counts(facts: &RigFacts) -> String {
    let runs = match facts.runs {
        Some(count) => count.to_string(),
        None => "no value".to_string(),
    };
    let days = match facts.wellness_days {
        Some(count) => count.to_string(),
        None => "no value".to_string(),
    };
    format!("{runs} runs · {days} wellness days")
}

/// The first run, in the app's own words.
fn first_run_line(facts: &RigFacts) -> String {
    match facts.first_run.as_str() {
        "fresh" => "nobody has opened this rig yet".to_string(),
        "skipped" => "the first-run page was set aside; the app opens as it is".to_string(),
        "connected" => "a Garmin key was accepted".to_string(),
        _ => "no value".to_string(),
    }
}

/// The storage: where the files are.
fn storage_section(facts: &RigFacts) -> String {
    let mut out = String::new();
    out.push_str("<section id=\"storage\">\n<h2>Where your data is</h2>\n<dl>\n");
    let _ = writeln!(
        out,
        "<dt>The folder</dt><dd><code>{}</code></dd>",
        escape(&facts.data_dir.display().to_string())
    );
    let _ = writeln!(
        out,
        "<dt>The database</dt><dd><code>{}</code>{}</dd>",
        escape(&facts.database.display().to_string()),
        escape(&size_line(facts))
    );
    let _ = writeln!(
        out,
        "<dt>The keys</dt><dd><code>{}</code>{}</dd>",
        escape(&facts.database_key.display().to_string()),
        garmin_key_line(facts)
    );
    out.push_str("</dl>\n");
    out.push_str(
        "<p class=\"muted\">The database is encrypted, and the key that opens it is a file of \
         its own. Nothing here leaves this machine.</p>\n",
    );
    out.push_str("<p><a href=\"/rig/tools\">Backup and restore</a></p>\n</section>\n");
    out
}

/// The size of the database, in the app's own words.
fn size_line(facts: &RigFacts) -> String {
    match facts.database_bytes {
        Some(bytes) => format!(" · {bytes} bytes"),
        None => String::new(),
    }
}

/// The Garmin key's file, in the app's own words.
///
/// The line is **markup, not text**: the path is escaped and the tags are
/// the page's own.
fn garmin_key_line(facts: &RigFacts) -> String {
    let path = escape(&facts.garmin_key.display().to_string());
    if facts.key_kept {
        format!(" · the Garmin key is in <code>{path}</code>")
    } else {
        format!(" · no Garmin key is kept (<code>{path}</code> is where it would go)")
    }
}

/// The connection: the key and the last sync.
fn connection_section(facts: &RigFacts) -> String {
    let mut out = String::new();
    out.push_str("<section id=\"connection\">\n<h2>The connection</h2>\n<dl>\n");
    let _ = writeln!(
        out,
        "<dt>The Garmin key</dt><dd>{}</dd>",
        escape(if facts.key_kept {
            "a key is kept on this machine"
        } else {
            "no key is kept"
        })
    );
    let _ = writeln!(
        out,
        "<dt>The last sync</dt><dd>{}</dd>",
        escape(
            facts
                .last_sync
                .as_deref()
                .unwrap_or("nothing has been synced yet")
        )
    );
    out.push_str("</dl>\n</section>\n");
    out
}

/// The three decisions.
///
/// **They are listed side by side**, in the same shape, with no preference
/// between them and nothing graded. The page says what is in force and
/// where the choice is made; it never says which one comes first.
fn decisions_section(facts: &RigFacts) -> String {
    let mut out = String::new();
    out.push_str("<section id=\"decisions\">\n");
    out.push_str("<h2>Three things only you can give it</h2>\n");
    out.push_str(
        "<p class=\"muted\">Each one is yours to give or to leave. Nothing here is asked for, \
         and nothing here is measured.</p>\n",
    );
    out.push_str("<div class=\"choices\">\n");

    // 1 · a race to type.
    out.push_str("<article>\n<h3>A race</h3>\n");
    match facts.races.is_empty() {
        true => out.push_str("<p>No race time entered.</p>\n"),
        false => {
            out.push_str("<ul class=\"races\">\n");
            for race in &facts.races {
                let _ = writeln!(
                    out,
                    "<li>{} · {} · {}{}</li>",
                    escape(&race.label),
                    escape(&race.time),
                    escape(&race.date),
                    if race.anchor {
                        " · the paces come from this race"
                    } else {
                        ""
                    }
                );
            }
            out.push_str("</ul>\n");
        }
    }
    out.push_str(&race_form());
    out.push_str("</article>\n");

    // 2 · a threshold to choose.
    out.push_str("<article>\n<h3>A threshold</h3>\n");
    match &facts.threshold {
        Some(threshold) => {
            let _ = writeln!(
                out,
                "<p>In force since {date}: {value} · from {source}.</p>",
                date = escape(&threshold.entered_on),
                value = escape(&threshold.value),
                source = escape(&threshold.source)
            );
        }
        None => out.push_str("<p>No threshold is in force.</p>\n"),
    }
    out.push_str(
        "<p class=\"muted\">The four candidates are in the app, under Settings &rsaquo; Training \
         preferences &mdash; the app states them and does not choose.</p>\n",
    );
    out.push_str("</article>\n");

    // 3 · a programme to adopt.
    out.push_str("<article>\n<h3>A programme</h3>\n");
    match &facts.programme {
        Some(programme) => {
            let _ = writeln!(
                out,
                "<p>In force since {date}: {label} &middot; {method} &middot; the race is {goal} \
                 on {race}.</p>",
                date = escape(&programme.adopted_on),
                label = escape(&programme.label),
                method = escape(&programme.method),
                goal = escape(&programme.goal),
                race = escape(&programme.race_date)
            );
        }
        None => out.push_str("<p>No programme has been adopted.</p>\n"),
    }
    out.push_str(
        "<p class=\"muted\">The four programmes are in the app, under Plan &rsaquo; Adopting a \
         programme &mdash; the entry requirement sits beside your own mileage, and the app does \
         not rank them.</p>\n",
    );
    out.push_str("</article>\n");

    out.push_str("</div>\n</section>\n");
    out
}

/// The form a race is typed in.
///
/// **The label is the app's own closed list**, and the time and the day are
/// the two facts a race needs.
fn race_form() -> String {
    let mut out = String::new();
    out.push_str("<form method=\"post\" action=\"/rig/race\" class=\"race\">\n");
    out.push_str("<label for=\"race-label\">The race</label>\n");
    out.push_str("<select id=\"race-label\" name=\"label\">\n");
    for label in RACE_LABELS {
        let _ = writeln!(
            out,
            "<option value=\"{value}\">{label}</option>",
            value = escape(label),
            label = escape(label)
        );
    }
    out.push_str("</select>\n");
    out.push_str("<label for=\"race-time\">The time</label>\n");
    out.push_str(
        "<input id=\"race-time\" name=\"time\" type=\"text\" inputmode=\"numeric\" \
         placeholder=\"43:10\" autocomplete=\"off\" spellcheck=\"false\">\n",
    );
    out.push_str("<label for=\"race-date\">The day</label>\n");
    out.push_str("<input id=\"race-date\" name=\"date\" type=\"date\" autocomplete=\"off\">\n");
    out.push_str("<button type=\"submit\">Add the race</button>\n");
    out.push_str("</form>\n");
    out
}

/// The foot of the page.
const FOOT: &str = "</body>\n</html>\n";

/// The page's own styling.
///
/// The rig's pages are the host's own, and they carry their own small
/// stylesheet — the app's tokens belong to the app.
const STYLE: &str = ":root {\n  color-scheme: light;\n  --paper: #f4f7fa;\n  --card: #ffffff;\n  --ink: #17324d;\n  --muted: #61788d;\n  --line: #e4ebf0;\n  --accent: #124a7c;\n  --accent-ink: #ffffff;\n  --error: #8b2635;\n}\n* { box-sizing: border-box; }\nbody {\n  margin: 0;\n  min-height: 100dvh;\n  background: var(--paper);\n  color: var(--ink);\n  font: 16px/1.5 system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif;\n}\nmain { width: min(100%, 1000px); margin: 0 auto; padding: 24px 16px 48px; }\n.mark { margin: 0 0 8px; color: var(--accent); font-size: 13px; font-weight: 700; letter-spacing: .12em; text-transform: uppercase; }\nh1 { margin: 16px 0 8px; font-size: 28px; letter-spacing: -.025em; }\nh2 { margin: 0 0 8px; font-size: 17px; }\nh3 { margin: 0 0 6px; font-size: 15px; }\n.lede { margin: 0 0 30px; color: var(--muted); }\nsection { margin-top: 12px; padding: 16px; border: 1px solid var(--line); border-radius: 12px; background: var(--card); }\na { color: var(--accent); min-height: 44px; display: inline-flex; align-items: center; }\np { margin: 0 0 14px; }\n.muted { color: var(--muted); font-size: 14px; }\ncode { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 13px; }\ndl { display: grid; grid-template-columns: minmax(0, 12rem) minmax(0, 1fr); gap: 6px 16px; margin: 0 0 14px; }\ndt { color: var(--muted); font-size: 14px; }\ndd { margin: 0; overflow-wrap: anywhere; }\nul { margin: 0 0 14px; padding-left: 20px; }\nul.links { columns: 2; }\n@media (max-width: 640px) { ul.links { columns: 1; } dl { grid-template-columns: minmax(0, 1fr); } dt { margin-top: 8px; } }\n.choices { display: grid; gap: 16px; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); }\n.choices article { border: 1px solid var(--line); border-radius: 10px; padding: 14px; }\n.races { padding-left: 18px; }\nform.race { display: grid; gap: 6px; }\nlabel { color: var(--muted); font-size: 13px; }\ninput, select, button {\n  min-height: 44px;\n  border-radius: 8px;\n  font: inherit;\n}\ninput, select { padding: 9px 11px; border: 1px solid var(--line); background: var(--paper); color: var(--ink); }\nbutton { padding: 9px 15px; border: 1px solid var(--accent); background: var(--accent); color: var(--accent-ink); font-weight: 650; cursor: pointer; }\nbutton:focus-visible, input:focus-visible, select:focus-visible, a:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }\n.notice { padding: 10px 12px; border-radius: 8px; background: var(--paper); color: var(--ink); }\n.notice[data-kind=\"refused\"] { color: var(--error); }\n";

/// The text of a tag, with the characters that would break the page escaped.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// A time, as the athlete wrote it, in seconds.
///
/// **Two shapes**: `43:10` and `1:12:30`. A plain number is read as
/// seconds. Anything else is refused, and the refusal says so.
pub fn parse_time(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() > 3 {
        return None;
    }
    let mut seconds = 0.0;
    for part in parts {
        let value: f64 = part.trim().parse().ok()?;
        if value < 0.0 || !value.is_finite() {
            return None;
        }
        seconds = seconds * 60.0 + value;
    }
    if seconds <= 0.0 {
        return None;
    }
    Some(seconds)
}

/// A form's fields, read out of a `application/x-www-form-urlencoded` body.
pub fn form_fields(body: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(body);
    let mut fields = Vec::new();
    for pair in text.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (name, value) = match pair.split_once('=') {
            Some((name, value)) => (name, value),
            None => (pair, ""),
        };
        let name = crate::http::percent_decode(&name.replace('+', " "));
        let value = crate::http::percent_decode(&value.replace('+', " "));
        fields.push((name, value));
    }
    fields
}

/// One field of a form.
pub fn field(fields: &[(String, String)], name: &str) -> Option<String> {
    fields
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn a_time_is_read_in_seconds() {
        assert_eq!(parse_time("43:10"), Some(2590.0));
        assert_eq!(parse_time("1:12:30"), Some(4350.0));
        assert_eq!(parse_time("90"), Some(90.0));
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time("43:xx"), None);
        assert_eq!(parse_time("0:00"), None);
    }

    #[test]
    fn a_form_is_read_field_by_field() {
        let fields = form_fields(b"label=10K&time=43%3A10&date=2026-03-03");
        assert_eq!(field(&fields, "label"), Some("10K".to_string()));
        assert_eq!(field(&fields, "time"), Some("43:10".to_string()));
        assert_eq!(field(&fields, "date"), Some("2026-03-03".to_string()));
        assert_eq!(field(&fields, "name"), None);
    }

    #[test]
    fn the_page_escapes_what_it_prints() {
        assert_eq!(escape("<b>&\"</b>"), "&lt;b&gt;&amp;&quot;&lt;/b&gt;");
    }
}
