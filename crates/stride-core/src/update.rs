//! The run update: the payload a model reads, and the three checks that
//! stand between the model and the screen.
//!
//! ## What this module is
//!
//! A run gets a short update — four to six sentences about what the run was
//! and how it went — and a language model writes it from the run's own
//! numbers. **The model never computes a number.** It is handed facts that
//! are already computed, and it may only arrange them into sentences.
//!
//! So there are two halves here, and they belong together:
//!
//! * the **payload**: the facts, rendered in this crate's own wording, and
//!   nothing else — never the positions, never the athlete's note, never
//!   their name;
//! * the **checks**: three mechanical rules that decide whether a draft may
//!   be shown at all.
//!
//! ## The three checks
//!
//! | check | what it asks |
//! |---|---|
//! | groundedness | does every number in the draft appear in the payload? |
//! | direction | does a claim with a sign agree with the payload's signed value? |
//! | register | does the draft use a word the product forbids? |
//!
//! They are mechanical on purpose. A rule that needs a reader is a rule
//! that is skipped under pressure, and the three above are the ones that
//! cannot be argued with: a number is either in the payload or it is not.
//!
//! The **register**'s word list is this crate's own ban list — the same one
//! `docs/contract/voice.md` freezes and the same one the frontend's lint
//! reads out of the export — plus the words that would turn an observation
//! into a prediction. One list, two readers: the app's lint and the
//! reviewer's check cannot drift apart, because they read the same list.
//!
//! ## The payload is rendered, not assembled
//!
//! The payload crosses as rendered values ("8.4 km", "5:24 /km") and as
//! sentences in this crate's own words. Nothing here formats a number for
//! itself: the units are [`crate::units`]'s, and the words are
//! [`crate::strings`]'s.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::strings;
use crate::strings::lint;
use crate::units;

/// The words an update may not use, beyond the product's own ban list.
///
/// The product's ban list (`docs/contract/voice.md`) holds the commands, the
/// verdicts and the conclusions. An update adds one more thing the list
/// does not cover: **a prediction**. "The pace will drop" is not an
/// observation of the run, and an observation is the whole of what an
/// update is.
pub const UPDATE_FORBIDDEN: [&str; 14] = [
    "will",
    "going to",
    "likely",
    "predict",
    "predicted",
    "prediction",
    "forecast",
    "injury risk",
    "recovery score",
    "training readiness",
    "next time",
    "next week",
    "from now on",
    "in the coming",
];

/// The words the reviewer may answer with.
///
/// The verdict is the reviewer's own: three words, and the rig reads the
/// first one it finds. The prompt names them, and this list is the one the
/// rig parses — a test holds the two together.
pub const VERDICTS: [&str; 3] = ["ship", "repair", "discard"];

/// The check a finding belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Check {
    /// Every number in the draft appears in the payload.
    Groundedness,
    /// A claim with a sign agrees with the payload's signed value.
    Direction,
    /// The draft uses no word the product forbids.
    Register,
}

impl Check {
    /// The name of the check, as the stored update carries it.
    pub fn name(self) -> &'static str {
        match self {
            Check::Groundedness => "groundedness",
            Check::Direction => "direction",
            Check::Register => "register",
        }
    }

    /// The three checks, in the order they are listed.
    pub const ALL: [Check; 3] = [Check::Groundedness, Check::Direction, Check::Register];
}

/// One thing the checks found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The check that found it.
    pub check: Check,
    /// What is wrong, in this crate's own words.
    pub detail: String,
}

/// The result of the checks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// What the checks found. Empty when they all passed.
    pub findings: Vec<Finding>,
}

impl Report {
    /// Did every check pass?
    pub fn passed(&self) -> bool {
        self.findings.is_empty()
    }

    /// The findings of one check.
    pub fn findings_of(&self, check: Check) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|finding| finding.check == check)
            .collect()
    }
}

/// What a signed observation counts.
///
/// The quantity is what tells a **faster** pace from a **lower** heart
/// rate: the same sign means different things for different measurements,
/// and the direction the payload states is the direction of the quantity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quantity {
    /// A pace, in seconds per kilometre. Lower is faster.
    Pace,
    /// A heart rate, in beats per minute.
    HeartRate,
    /// A percentage.
    Percent,
}

impl Quantity {
    /// The name the payload carries.
    pub fn name(self) -> &'static str {
        match self {
            Quantity::Pace => "pace",
            Quantity::HeartRate => "heart_rate",
            Quantity::Percent => "percent",
        }
    }

    /// The quantity a name means.
    pub fn parse(name: &str) -> Option<Quantity> {
        match name {
            "pace" => Some(Quantity::Pace),
            "heart_rate" => Some(Quantity::HeartRate),
            "percent" => Some(Quantity::Percent),
            _ => None,
        }
    }

    /// The direction of a signed value, in this crate's own words.
    ///
    /// The word is the catalogue's: the payload states the direction, and
    /// the direction check compares the draft against this word.
    pub fn direction(self, value: f64) -> &'static str {
        let id = match self {
            Quantity::Pace => {
                if value < 0.0 {
                    "update.direction.faster"
                } else if value > 0.0 {
                    "update.direction.slower"
                } else {
                    "update.direction.held"
                }
            }
            Quantity::HeartRate | Quantity::Percent => {
                if value < 0.0 {
                    "update.direction.lower"
                } else if value > 0.0 {
                    "update.direction.higher"
                } else {
                    "update.direction.held"
                }
            }
        };
        strings::text(id).unwrap_or_default()
    }
}

/// One fact of the payload: a label and a rendered value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    /// The label, in this crate's own words.
    pub label: String,
    /// The value, rendered by [`crate::units`].
    pub value: String,
}

/// One split of the payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitFact {
    /// The split's number, from one.
    pub index: usize,
    /// The distance, rendered.
    pub distance: String,
    /// The pace, rendered.
    pub pace: String,
    /// The heart rate, rendered.
    pub heart_rate: String,
}

/// One zone of the payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneFact {
    /// The zone's name: "Z1" … "Z5".
    pub label: String,
    /// The time in the zone, rendered.
    pub time: String,
    /// The share of the moving time, rendered.
    pub share: String,
}

/// One observation of the payload: the run's own mechanics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservationFact {
    /// The label, in this crate's own words.
    pub label: String,
    /// The value, rendered.
    pub value: String,
    /// The direction of a signed value, when the observation has one.
    #[serde(default)]
    pub direction: Option<String>,
    /// The signed number behind the value, when the observation has one.
    #[serde(default)]
    pub signed: Option<f64>,
    /// What the number counts, when the observation has a signed value.
    #[serde(default)]
    pub quantity: Option<String>,
}

/// The payload: the facts a model may use, and nothing else.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    /// The writer's model, by name.
    pub model: String,
    /// The day the run was read, in the api's own rendering.
    pub read_on: String,
    /// The run's own facts.
    pub facts: Vec<Fact>,
    /// The run's splits.
    pub splits: Vec<SplitFact>,
    /// The time in the athlete's own zones.
    pub zones: Vec<ZoneFact>,
    /// The run's own mechanics.
    pub observations: Vec<ObservationFact>,
}

impl Payload {
    /// The payload as the model reads it: this crate's own wording, one
    /// fact per line.
    ///
    /// This is the text the prompt carries, and the text the hash is taken
    /// over — so the hash names what the model was actually given.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for fact in &self.facts {
            out.push_str(&format!("{} — {}\n", fact.label, fact.value));
        }
        if !self.splits.is_empty() {
            out.push_str(&format!("{}\n", heading("update.payload.splits")));
            for split in &self.splits {
                out.push_str(&format!(
                    "{} — {} — {} — {}\n",
                    split.index, split.distance, split.pace, split.heart_rate
                ));
            }
        }
        if !self.zones.is_empty() {
            out.push_str(&format!("{}\n", heading("update.payload.zones")));
            for zone in &self.zones {
                out.push_str(&format!(
                    "{} — {} — {}\n",
                    zone.label, zone.time, zone.share
                ));
            }
        }
        if !self.observations.is_empty() {
            out.push_str(&format!("{}\n", heading("update.payload.observations")));
            for observation in &self.observations {
                match &observation.direction {
                    Some(direction) => out.push_str(&format!(
                        "{} — {} — {}\n",
                        observation.label, observation.value, direction
                    )),
                    None => out.push_str(&format!(
                        "{} — {}\n",
                        observation.label, observation.value
                    )),
                }
            }
        }
        out.trim_end().to_string()
    }

    /// Every number the payload carries.
    ///
    /// The groundedness check compares the draft's numbers with this set:
    /// a number the model was not given is a number it computed, and the
    /// model does not compute.
    pub fn numbers(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for text in self.texts() {
            for number in numbers_in(&text) {
                out.insert(number);
            }
        }
        out
    }

    /// Every text the payload carries, one by one.
    fn texts(&self) -> Vec<String> {
        let mut out = vec![self.model.clone(), self.read_on.clone()];
        for fact in &self.facts {
            out.push(fact.label.clone());
            out.push(fact.value.clone());
        }
        for split in &self.splits {
            out.push(split.distance.clone());
            out.push(split.pace.clone());
            out.push(split.heart_rate.clone());
        }
        for zone in &self.zones {
            out.push(zone.label.clone());
            out.push(zone.time.clone());
            out.push(zone.share.clone());
        }
        for observation in &self.observations {
            out.push(observation.label.clone());
            out.push(observation.value.clone());
            if let Some(direction) = &observation.direction {
                out.push(direction.clone());
            }
        }
        out
    }
}

/// The facts the payload is built from.
///
/// The caller has already computed every one of them; this crate only
/// renders them and arranges them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UpdateInput {
    /// Does the run have a file to read?
    ///
    /// A run with no file has no facts: there is nothing to write about,
    /// and the answer is "no value" — never an empty update.
    pub has_file: bool,
    /// The writer's model, by name.
    pub model: String,
    /// The day the run was read, in the api's own rendering.
    pub read_on: String,
    /// The distance, in metres.
    pub distance_m: Option<f64>,
    /// The moving time, in seconds.
    pub duration_s: Option<f64>,
    /// The average pace, in seconds per kilometre.
    pub pace_s_km: Option<f64>,
    /// The average heart rate, in beats per minute.
    pub average_heart_rate: Option<f64>,
    /// The highest heart rate, in beats per minute.
    pub highest_heart_rate: Option<f64>,
    /// The elevation gain, in metres.
    pub elevation_gain_m: Option<f64>,
    /// The run's splits.
    pub splits: Vec<SplitInput>,
    /// The time in the athlete's own zones.
    pub zones: Vec<ZoneInput>,
    /// The run's own mechanics.
    pub observations: Vec<ObservationInput>,
}

/// One split of the input.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SplitInput {
    /// The split's number, from one.
    pub index: usize,
    /// The distance, in metres.
    pub distance_m: f64,
    /// The pace, in seconds per kilometre.
    pub pace_s_km: Option<f64>,
    /// The heart rate, in beats per minute.
    pub heart_rate: Option<f64>,
}

/// One zone of the input.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZoneInput {
    /// The zone's number, from one.
    pub index: usize,
    /// The time in the zone, in seconds.
    pub seconds: f64,
    /// The share of the moving time, as a percentage.
    pub share_pct: Option<f64>,
}

/// One observation of the input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObservationInput {
    /// The label, in this crate's own words.
    pub label: String,
    /// The value, rendered.
    pub value: String,
    /// The signed number behind the value, when there is one.
    pub signed: Option<f64>,
    /// What the number counts, when it is signed.
    pub quantity: Option<Quantity>,
}

/// The payload for a run — or "no value", when the run has no file.
pub fn payload(input: &UpdateInput) -> Option<Payload> {
    if !input.has_file {
        return None;
    }

    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "update.payload.distance",
        units::render_distance(input.distance_m),
    );
    push_fact(
        &mut facts,
        "update.payload.duration",
        units::render_duration(input.duration_s),
    );
    push_fact(
        &mut facts,
        "update.payload.pace",
        units::render_pace(input.pace_s_km),
    );
    push_fact(
        &mut facts,
        "update.payload.heart_rate",
        units::render_heart_rate(input.average_heart_rate),
    );
    push_fact(
        &mut facts,
        "update.payload.highest_heart_rate",
        units::render_heart_rate(input.highest_heart_rate),
    );
    push_fact(
        &mut facts,
        "update.payload.elevation",
        units::render_elevation(input.elevation_gain_m),
    );

    let splits = input
        .splits
        .iter()
        .map(|split| SplitFact {
            index: split.index,
            distance: units::render_distance(Some(split.distance_m)).unwrap_or_default(),
            pace: units::render_pace(split.pace_s_km).unwrap_or_default(),
            heart_rate: units::render_heart_rate(split.heart_rate).unwrap_or_default(),
        })
        .collect();

    let zones = input
        .zones
        .iter()
        .map(|zone| ZoneFact {
            label: units::render_zone_label(zone.index).unwrap_or_default(),
            time: units::render_duration(Some(zone.seconds)).unwrap_or_default(),
            share: units::render_percent(zone.share_pct).unwrap_or_default(),
        })
        .collect();

    let observations = input
        .observations
        .iter()
        .map(|observation| ObservationFact {
            label: observation.label.clone(),
            value: observation.value.clone(),
            direction: match (observation.signed, observation.quantity) {
                (Some(value), Some(quantity)) => Some(quantity.direction(value).to_string()),
                _ => None,
            },
            signed: observation.signed,
            quantity: observation.quantity.map(|quantity| quantity.name().to_string()),
        })
        .collect();

    Some(Payload {
        model: input.model.clone(),
        read_on: input.read_on.clone(),
        facts,
        splits,
        zones,
        observations,
    })
}

/// Add a fact, when it has a value.
fn push_fact(facts: &mut Vec<Fact>, label: &str, value: Option<String>) {
    if let Some(value) = value {
        facts.push(Fact {
            label: heading(label),
            value,
        });
    }
}

/// A catalogue entry's wording, or an empty string.
fn heading(id: &str) -> String {
    strings::text(id).unwrap_or_default().to_string()
}

// ------------------------------------------------------------------ the checks

/// The three checks, on a draft.
///
/// The draft is measured against the payload it was written from: the
/// numbers it may use, the directions it may claim, and the register it
/// may be written in.
pub fn check_draft(payload: &Payload, draft: &str) -> Report {
    let mut findings = Vec::new();
    findings.extend(groundedness(payload, draft));
    findings.extend(direction(payload, draft));
    findings.extend(register(draft));
    Report { findings }
}

/// The groundedness check: every number in the draft is in the payload.
fn groundedness(payload: &Payload, draft: &str) -> Vec<Finding> {
    let given = payload.numbers();
    let mut findings = Vec::new();
    for number in numbers_in(draft) {
        if given.contains(&number) {
            continue;
        }
        let detail = strings::render(
            "update.check.groundedness",
            &[("value", &number)],
        )
        .unwrap_or_else(|| format!("the number {number} is not in the payload"));
        findings.push(Finding {
            check: Check::Groundedness,
            detail,
        });
    }
    findings
}

/// The direction check: a claim with a sign agrees with the payload.
///
/// The payload states the direction of every signed observation — "faster",
/// "slower", "higher", "lower", "held" — and the draft's own words about
/// it have to say the same thing. A draft that reads a pace drift of
/// −43 s/km as an improvement of 43 s/km has the sign the wrong way
/// round, and this is where that is caught.
fn direction(payload: &Payload, draft: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for observation in &payload.observations {
        let (Some(signed), Some(quantity)) = (observation.signed, observation.quantity.as_deref())
        else {
            continue;
        };
        let Some(quantity) = Quantity::parse(quantity) else {
            continue;
        };
        let stated = quantity.direction(signed);
        let magnitude = canonical_number(&format!("{}", signed.abs())).unwrap_or_default();
        let unit = unit_of(&observation.value);
        for clause in clauses(draft) {
            if !mentions(&clause, &magnitude, unit, &observation.label) {
                continue;
            }
            let words = direction_words(&clause);
            if words.is_empty() {
                continue;
            }
            let agrees = words
                .iter()
                .any(|word| agrees_with(word, stated, quantity));
            if agrees {
                continue;
            }
            let detail = strings::render(
                "update.check.direction",
                &[
                    ("label", &observation.label),
                    ("stated", stated),
                    ("claim", &words[0]),
                ],
            )
            .unwrap_or_else(|| {
                format!(
                    "the draft says {}, and the payload says {stated}",
                    words[0]
                )
            });
            findings.push(Finding {
                check: Check::Direction,
                detail,
            });
        }
    }
    findings
}

/// The register check: the draft uses no word the product forbids.
///
/// The list is this crate's own: the product's ban list, and the words that
/// would make the update a prediction.
fn register(draft: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for phrase in lint::banned_phrases().into_iter().chain(UPDATE_FORBIDDEN) {
        if !lint::contains_phrase(draft, phrase) {
            continue;
        }
        let detail = strings::render("update.check.register", &[("word", phrase)])
            .unwrap_or_else(|| format!("the word {phrase} is not one the app uses"));
        findings.push(Finding {
            check: Check::Register,
            detail,
        });
    }
    findings
}

/// Does the draft's word agree with the payload's direction?
fn agrees_with(word: &str, stated: &str, quantity: Quantity) -> bool {
    let stated = class_of(stated);
    let word = class_of(word);
    match word {
        Some(Claim::Improved) => stated == Some(quantity.improving()),
        Some(claim) => stated == Some(claim),
        None => false,
    }
}

/// The direction a quantity improves in.
impl Quantity {
    /// The class of direction that is an improvement of this quantity.
    fn improving(self) -> Claim {
        match self {
            Quantity::Pace => Claim::Faster,
            Quantity::HeartRate | Quantity::Percent => Claim::Lower,
        }
    }
}

/// The classes of direction a claim can carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Claim {
    Faster,
    Slower,
    Higher,
    Lower,
    Held,
    Improved,
}

/// The class a direction word belongs to.
///
/// The five words the payload states are the catalogue's, and the other
/// words are the ones a model reaches for when it says the same thing in
/// its own way.
fn class_of(word: &str) -> Option<Claim> {
    let word = word.to_lowercase();
    for (id, claim) in [
        ("update.direction.faster", Claim::Faster),
        ("update.direction.slower", Claim::Slower),
        ("update.direction.higher", Claim::Higher),
        ("update.direction.lower", Claim::Lower),
        ("update.direction.held", Claim::Held),
    ] {
        if word == strings::text(id).unwrap_or_default() {
            return Some(claim);
        }
    }
    let claim = match word.as_str() {
        "quicker" | "quickened" => Claim::Faster,
        "slowed" => Claim::Slower,
        "rose" | "risen" | "increased" | "climbed" => Claim::Higher,
        "fell" | "fallen" | "dropped" | "decreased" => Claim::Lower,
        "steady" | "unchanged" | "flat" | "maintained" | "kept" => Claim::Held,
        "improved" => Claim::Improved,
        _ => return None,
    };
    Some(claim)
}

/// The direction words that are two words long.
const PHRASES: [(&str, Claim); 5] = [
    ("sped up", Claim::Faster),
    ("speeded up", Claim::Faster),
    ("slowed down", Claim::Slower),
    ("went up", Claim::Higher),
    ("went down", Claim::Lower),
];

/// The direction words in a clause, in the order they appear.
fn direction_words(clause: &str) -> Vec<String> {
    let mut out = Vec::new();
    for word in clause.split(|character: char| !character.is_alphabetic()) {
        let word = word.trim();
        if word.is_empty() {
            continue;
        }
        if class_of(word).is_some() {
            out.push(word.to_string());
        }
    }
    let lowered = clause.to_lowercase();
    for (phrase, _) in PHRASES {
        if lowered.contains(phrase) {
            out.push(phrase.to_string());
        }
    }
    out
}

/// Does the clause talk about this observation?
///
/// The clause must carry the observation's own number, and either its unit
/// or its name. A number on its own is not enough: "5 splits" says nothing
/// about a heart rate that moved by 5.
fn mentions(clause: &str, magnitude: &str, unit: Option<&str>, label: &str) -> bool {
    let numbers = numbers_in(clause);
    if !numbers.iter().any(|number| number == magnitude) {
        return false;
    }
    if let Some(unit) = unit {
        if clause.to_lowercase().contains(unit) {
            return true;
        }
    }
    let words = keywords(label);
    clause
        .to_lowercase()
        .split(|character: char| !character.is_alphabetic())
        .any(|word| words.iter().any(|keyword| *keyword == word))
}

/// The words of a label that name it.
fn keywords(label: &str) -> Vec<String> {
    label
        .to_lowercase()
        .split(|character: char| !character.is_alphabetic())
        .filter(|word| word.len() >= 4)
        .map(|word| word.to_string())
        .collect()
}

/// The unit of a rendered value: the text after the number.
///
/// A value may carry a sign — "−43 s/km" — and the sign is not the unit.
fn unit_of(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let start = trimmed
        .find(|character: char| character.is_ascii_digit())
        .unwrap_or(0);
    let rest = &trimmed[start..];
    let at = rest.find(|character: char| {
        !character.is_ascii_digit() && character != '.' && character != ','
    })?;
    let unit = rest[at..].trim();
    if unit.is_empty() {
        None
    } else {
        Some(unit)
    }
}

/// The clauses of a draft: the sentences, split at the joins.
fn clauses(draft: &str) -> Vec<String> {
    draft
        .split(|character: char| {
            character == '.' || character == ';' || character == '\n' || character == '!'
        })
        .flat_map(|sentence| sentence.split(" but ").flat_map(|part| part.split(" and ")))
        .map(|clause| clause.trim().to_string())
        .filter(|clause| !clause.is_empty())
        .collect()
}

/// Every number in a text, in the canonical form the checks compare.
fn numbers_in(text: &str) -> Vec<String> {
    let characters: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        if !characters[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < characters.len()
            && (characters[index].is_ascii_digit()
                || characters[index] == '.'
                || characters[index] == ',')
        {
            index += 1;
        }
        let raw: String = characters[start..index].iter().collect();
        let raw = raw.trim_end_matches(['.', ',']);
        if let Some(number) = canonical_number(raw) {
            out.push(number);
        }
    }
    out
}

/// A number in the one form the checks compare.
///
/// "8.40" and "8.4" are the same number, and "43" and "43.0" are the same
/// number: the checks compare values, not spellings.
fn canonical_number(raw: &str) -> Option<String> {
    let cleaned = raw.replace(',', ".");
    if cleaned.is_empty() {
        return None;
    }
    let mut candidate = cleaned.as_str();
    loop {
        if let Ok(value) = candidate.parse::<f64>() {
            if !value.is_finite() {
                return None;
            }
            return Some(if value.fract() == 0.0 && value.abs() < 1e15 {
                format!("{}", value as i64)
            } else {
                let mut text = format!("{value:.6}");
                while text.ends_with('0') {
                    text.pop();
                }
                if text.ends_with('.') {
                    text.pop();
                }
                text
            });
        }
        if candidate.len() <= 1 {
            return None;
        }
        candidate = &candidate[..candidate.len() - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A payload with a pace drift of −43 s/km, as the app would read it.
    fn payload_with_drift(drift: f64) -> Payload {
        let input = UpdateInput {
            has_file: true,
            model: "the writer".to_string(),
            read_on: "10 May 2026".to_string(),
            distance_m: Some(12_400.0),
            duration_s: Some(4_020.0),
            pace_s_km: Some(324.19),
            average_heart_rate: Some(148.0),
            highest_heart_rate: Some(171.0),
            elevation_gain_m: Some(42.0),
            splits: vec![SplitInput {
                index: 1,
                distance_m: 1_000.0,
                pace_s_km: Some(320.0),
                heart_rate: Some(150.0),
            }],
            zones: vec![ZoneInput {
                index: 1,
                seconds: 180.0,
                share_pct: Some(8.0),
            }],
            observations: vec![ObservationInput {
                label: "Pace drift".to_string(),
                value: units::render_difference(Some(drift), units::ValueUnit::Pace)
                    .unwrap_or_default(),
                signed: Some(drift),
                quantity: Some(Quantity::Pace),
            }],
        };
        payload(&input).unwrap_or_default()
    }

    #[test]
    fn the_prompt_names_the_verdicts_the_rig_parses() {
        let prompt = strings::text("update.prompt.reviewer").unwrap_or_default();
        for verdict in VERDICTS {
            assert!(
                prompt.contains(verdict),
                "the reviewer's prompt does not name the verdict \"{verdict}\""
            );
        }
    }

    #[test]
    fn a_run_with_no_file_has_no_payload() {
        let input = UpdateInput {
            has_file: false,
            ..UpdateInput::default()
        };
        assert!(payload(&input).is_none());
    }

    #[test]
    fn a_payload_carries_the_facts_and_nothing_else() {
        let payload = payload_with_drift(-43.0);
        assert_eq!(payload.model, "the writer");
        assert_eq!(payload.read_on, "10 May 2026");
        assert!(payload
            .facts
            .iter()
            .any(|fact| fact.value == "12.4 km" && fact.label == "Distance"));
        assert_eq!(payload.splits.len(), 1);
        assert_eq!(payload.zones[0].label, "Z1");
        assert_eq!(payload.observations[0].direction.as_deref(), Some("faster"));
        // The payload holds no position, no note and no name: it is built
        // from the facts and nothing else.
        let text = payload.text();
        assert!(!text.contains('@'));
        assert!(!text.to_lowercase().contains("note"));
    }

    #[test]
    fn a_clean_draft_passes_the_checks() {
        let payload = payload_with_drift(-43.0);
        let draft = "The run was 12.4 km in 1 h 7 m, at an average of 148 bpm. \
                      The pace was 5:24 /km. The first split was 1 km at 5:20 /km and 150 bpm. \
                      The pace drift was −43 s/km, so the second half was faster. \
                      The run spent 3 m in Z1.";
        let report = check_draft(&payload, draft);
        assert!(
            report.passed(),
            "a clean draft was refused: {:?}",
            report.findings
        );
    }

    #[test]
    fn a_number_from_nowhere_is_refused() {
        let payload = payload_with_drift(-43.0);
        let draft = "The run was 12.4 km in 1 h 7 m, at an average of 148 bpm. \
                      It covered 500 m more than usual.";
        let report = check_draft(&payload, draft);
        let findings = report.findings_of(Check::Groundedness);
        assert_eq!(findings.len(), 1, "the number from nowhere was not found");
        assert!(findings[0].detail.contains("500"));
    }

    #[test]
    fn a_sign_that_is_the_wrong_way_round_is_refused() {
        let payload = payload_with_drift(-43.0);
        // The payload says the second half was 43 s/km faster. The draft
        // says it slowed.
        let draft = "The run was 12.4 km in 1 h 7 m, at an average of 148 bpm. \
                      The pace drift was −43 s/km and the pace slowed by 43 s/km.";
        let report = check_draft(&payload, draft);
        let findings = report.findings_of(Check::Direction);
        assert_eq!(findings.len(), 1, "the inverted sign was not found");
        assert!(findings[0].detail.contains("faster"));
    }

    #[test]
    fn a_claim_that_agrees_is_kept() {
        let payload = payload_with_drift(-43.0);
        let draft = "The run was 12.4 km in 1 h 7 m. The pace drift was −43 s/km, \
                      and the second half was 43 s/km faster than the first.";
        let report = check_draft(&payload, draft);
        assert!(
            report.findings_of(Check::Direction).is_empty(),
            "a claim that agrees was refused: {:?}",
            report.findings
        );
    }

    #[test]
    fn a_forbidden_word_is_refused() {
        let payload = payload_with_drift(-43.0);
        let draft = "The run was 12.4 km in 1 h 7 m. You should take the day after a run \
                      like that, and the pace will drop next week.";
        let report = check_draft(&payload, draft);
        let findings = report.findings_of(Check::Register);
        assert!(
            findings.len() >= 2,
            "the forbidden words were not found: {:?}",
            report.findings
        );
    }

    #[test]
    fn the_numbers_are_compared_as_values_and_not_as_spellings() {
        let payload = payload_with_drift(-43.0);
        let draft = "The run was 12.40 km in 1 h 7 m, at an average of 148 bpm.";
        let report = check_draft(&payload, draft);
        assert!(
            report.findings_of(Check::Groundedness).is_empty(),
            "a number written another way was refused: {:?}",
            report.findings
        );
    }

    #[test]
    fn a_claim_about_another_number_is_not_a_claim_about_this_one() {
        let payload = payload_with_drift(-43.0);
        // "5 splits" says nothing about the pace drift, even though 5 is
        // a number the payload carries.
        let draft = "The run was 12.4 km in 1 h 7 m over 5 splits. The pace held steady \
                      at 5:24 /km.";
        let report = check_draft(&payload, draft);
        assert!(
            report.findings_of(Check::Direction).is_empty(),
            "an unrelated number was read as a claim: {:?}",
            report.findings
        );
    }
}
