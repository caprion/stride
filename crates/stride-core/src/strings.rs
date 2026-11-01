//! The string catalogue: every user-facing string, typed, in one place — and
//! the lint that holds it to `docs/contract/voice.md`.
//!
//! The catalogue is **data**, not format strings scattered through the code.
//! Two things follow from that:
//!
//! * a string's wording can be checked by a machine, and the check is the same
//!   check everywhere — the rules live here and are exported with the
//!   catalogue, so the frontend's lint reads the same rules this module
//!   applies;
//! * the closed direction set has exactly one definition. The three words are
//!   constants, the catalogue entries use them, and the lint refuses anything
//!   else.
//!
//! The registers, from `docs/contract/voice.md`:
//!
//! | type | what it is |
//! |---|---|
//! | `vital` | label · value · unit |
//! | `direction` | a word from the closed set, with the arithmetic and the travel beside it |
//! | `option` | a session from the athlete's own plan, unranked, never conditioned on the vitals |
//! | `pattern` | what usually happened, always with its sample size |
//! | `status` | connect/sync/staleness and empty-state lines |
//!
//! The rules the lint enforces:
//!
//! * a `direction` string comes from the closed set and nothing else;
//! * a `pattern` string carries its sample size;
//! * an `option` string never references a vital;
//! * no imperatives, no verdicts, no "you should", no "missed", no score.

use serde::{Deserialize, Serialize};

/// The closed set of direction words. Nothing may be added to it.
pub const BETTER_THAN_USUAL: &str = "better than usual";
pub const ABOUT_USUAL: &str = "about usual";
pub const COULD_BE_BETTER: &str = "could be better";

/// The closed set, as a list.
pub const DIRECTION_SET: [&str; 3] = [BETTER_THAN_USUAL, ABOUT_USUAL, COULD_BE_BETTER];

/// The ban list, from `docs/contract/voice.md`.
///
/// Note `rest day`, `rest up` and `rest today`: the contract bans "rest" **as
/// a command**, never as the noun. The product's own summary row is called
/// "Rest" — "Rest — could be better · 2 of 3 signals below your usual" — and
/// that is a name, not an instruction, so it needs no exemption.
pub const BANNED_PHRASES: [&str; 23] = [
    "should",
    "must",
    "take the day",
    "rest day",
    "rest up",
    "rest today",
    "go easy",
    "missed",
    "bad",
    "poor",
    "fail",
    "behind",
    "good",
    "keep it easy",
    "ready to",
    "recommended",
    "better to",
    "need to",
    "red",
    "readiness score",
    "not recovered",
    "you are ready",
    "you are not ready",
];

/// The list used to be two: `voice.md`'s and `PLAN.md` §0's additions. They
/// are one now — the contract's list, word for word — because two lists is
/// how a lint ends up passing a string the contract bans.
pub const EXTRA_BANNED_PHRASES: [&str; 0] = [];

/// The words that name a vital. An `option` string may not contain any of
/// them: "because your sleep was low" is the banned link, and listing an
/// option *because* a vital was poor would be the app judging the vitals.
pub const VITAL_WORDS: [&str; 8] = [
    "sleep",
    "hrv",
    "heart rate",
    "resting hr",
    "resting heart rate",
    "readiness",
    "recovery",
    "stress",
];

/// The fewest comparable days a pattern may be stated from. Two is "twice so
/// far", not a pattern.
pub const MIN_PATTERN_SAMPLE: usize = 3;

/// The register a user-facing string belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StringKind {
    /// Label · value · unit.
    Vital,
    /// A word from the closed set, with its arithmetic and its travel.
    Direction,
    /// A session or alternative from the athlete's own plan.
    Option,
    /// What usually happened, with its sample size.
    Pattern,
    /// A factual sentence about the data — the distance comparison, the
    /// Progress headline, the Compare basis.
    ///
    /// A statement carries no verdict and no instruction. It may name a vital
    /// (unlike an `option`, which must not: an option is a choice, and a
    /// statement is a fact), and it is always rendered by the core, with the
    /// values it compares already in the sentence.
    Statement,
    /// Connect, sync, staleness and empty-state lines.
    Status,
}

impl StringKind {
    /// The name of the register, as the exported catalogue writes it.
    pub fn name(&self) -> &'static str {
        match self {
            StringKind::Vital => "vital",
            StringKind::Direction => "direction",
            StringKind::Option => "option",
            StringKind::Pattern => "pattern",
            StringKind::Statement => "statement",
            StringKind::Status => "status",
        }
    }
}

/// The role a `direction` string plays.
///
/// The primary word is the direction itself and comes from the closed set. The
/// arithmetic ("14 below your usual") and the travel ("3rd night down") are
/// descriptive tails beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectionRole {
    /// The direction word itself.
    Primary,
    /// The muted arithmetic beside the word.
    Arithmetic,
    /// The direction of travel beside the word.
    Travel,
}

impl DirectionRole {
    /// The name of the role, as the exported catalogue writes it.
    pub fn name(&self) -> &'static str {
        match self {
            DirectionRole::Primary => "primary",
            DirectionRole::Arithmetic => "arithmetic",
            DirectionRole::Travel => "travel",
        }
    }
}

/// One entry in the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The catalogue id: `kind.name`, dotted.
    pub id: &'static str,
    /// The register the string belongs to.
    pub kind: StringKind,
    /// The role, for a `direction` string.
    pub role: Option<DirectionRole>,
    /// The wording. `{placeholders}` are filled in by the caller.
    pub template: &'static str,
}

/// The catalogue.
pub const CATALOGUE: [Entry; 40] = [
    // The vitals.
    Entry {
        id: "vital.sleep.label",
        kind: StringKind::Vital,
        role: None,
        template: "Sleep",
    },
    Entry {
        id: "vital.hrv.label",
        kind: StringKind::Vital,
        role: None,
        template: "HRV",
    },
    Entry {
        id: "vital.resting_hr.label",
        kind: StringKind::Vital,
        role: None,
        template: "Resting HR",
    },
    Entry {
        id: "vital.rest.label",
        kind: StringKind::Vital,
        role: None,
        template: "Rest",
    },
    Entry {
        id: "vital.sleep.value",
        kind: StringKind::Vital,
        role: None,
        template: "{value}",
    },
    Entry {
        id: "vital.hrv.value",
        kind: StringKind::Vital,
        role: None,
        template: "{value} ms",
    },
    Entry {
        id: "vital.resting_hr.value",
        kind: StringKind::Vital,
        role: None,
        template: "{value} bpm",
    },
    // The closed set.
    Entry {
        id: "direction.better_than_usual",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Primary),
        template: BETTER_THAN_USUAL,
    },
    Entry {
        id: "direction.about_usual",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Primary),
        template: ABOUT_USUAL,
    },
    Entry {
        id: "direction.could_be_better",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Primary),
        template: COULD_BE_BETTER,
    },
    // The arithmetic beside the word.
    Entry {
        id: "direction.above_usual",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{value} above your usual",
    },
    Entry {
        id: "direction.below_usual",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{value} below your usual",
    },
    Entry {
        id: "direction.more_than_usual",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{value} more than your usual",
    },
    Entry {
        id: "direction.less_than_usual",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{value} less than your usual",
    },
    // The travel beside the word.
    Entry {
        id: "direction.travel.steady",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Travel),
        template: "steady",
    },
    Entry {
        id: "direction.travel.down",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Travel),
        template: "{ordinal} night down",
    },
    Entry {
        id: "direction.travel.up",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Travel),
        template: "{ordinal} night up",
    },
    // The "Rest" line: a count and a direction.
    Entry {
        id: "rest.count.below",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{count} of {total} signals below your usual",
    },
    Entry {
        id: "rest.count.above",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{count} of {total} signals above your usual",
    },
    Entry {
        id: "rest.count.outside",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{count} of {total} signals outside your usual",
    },
    Entry {
        id: "rest.count.about",
        kind: StringKind::Direction,
        role: Some(DirectionRole::Arithmetic),
        template: "{count} of {total} signals about your usual",
    },
    // The options.
    Entry {
        id: "option.today_session",
        kind: StringKind::Option,
        role: None,
        template: "Today — {summary}",
    },
    Entry {
        id: "option.other_session",
        kind: StringKind::Option,
        role: None,
        template: "{day} — {summary}",
    },
    Entry {
        id: "option.other_sessions",
        kind: StringKind::Option,
        role: None,
        template: "Also in your plan — {summary}",
    },
    // The patterns.
    Entry {
        id: "pattern.usual_streak",
        kind: StringKind::Pattern,
        role: None,
        template: "your {vital} has been {side} your usual for {n} nights",
    },
    Entry {
        id: "pattern.comparable_outcome",
        kind: StringKind::Pattern,
        role: None,
        template:
            "the {n} other times your {vital} was {side} your usual, you ran {distance} or less",
    },
    Entry {
        id: "pattern.comparable_outcome.at_least",
        kind: StringKind::Pattern,
        role: None,
        template:
            "the {n} other times your {vital} was {side} your usual, you ran {distance} or more",
    },
    // The statements: factual sentences about the data, rendered by the core.
    Entry {
        id: "statement.distance.more",
        kind: StringKind::Statement,
        role: None,
        template: "{gap} more than the {target} target",
    },
    Entry {
        id: "statement.distance.less",
        kind: StringKind::Statement,
        role: None,
        template: "{gap} less than the {target} target",
    },
    Entry {
        id: "statement.distance.line",
        kind: StringKind::Statement,
        role: None,
        template: "{distance} · {comparison}",
    },
    Entry {
        id: "statement.progress.fitter",
        kind: StringKind::Statement,
        role: None,
        template: "fitter than {ago}",
    },
    Entry {
        id: "statement.progress.slower",
        kind: StringKind::Statement,
        role: None,
        template: "slower than {ago}",
    },
    Entry {
        id: "statement.progress.unchanged",
        kind: StringKind::Statement,
        role: None,
        template: "the same pace as {ago}",
    },
    Entry {
        id: "statement.progress.evidence.faster",
        kind: StringKind::Statement,
        role: None,
        template: "same heart rate, {pace} s/km faster",
    },
    Entry {
        id: "statement.progress.evidence.slower",
        kind: StringKind::Statement,
        role: None,
        template: "same heart rate, {pace} s/km slower",
    },
    Entry {
        id: "statement.compare.same_distance",
        kind: StringKind::Statement,
        role: None,
        template: "Same distance, {gap}",
    },
    Entry {
        id: "statement.compare.same_route",
        kind: StringKind::Statement,
        role: None,
        template: "Same route, different week",
    },
    Entry {
        id: "statement.compare.different",
        kind: StringKind::Statement,
        role: None,
        template: "Different distances — read with care",
    },
    // The statuses.
    Entry {
        id: "status.not_enough_data",
        kind: StringKind::Status,
        role: None,
        template: "not enough data for a comparison yet",
    },
    Entry {
        id: "status.not_enough_data_yet",
        kind: StringKind::Status,
        role: None,
        template: "not enough data yet",
    },
];

/// The catalogue entry with this id.
pub fn entry(id: &str) -> Option<&'static Entry> {
    CATALOGUE.iter().find(|entry| entry.id == id)
}

/// The wording of the catalogue entry with this id.
pub fn text(id: &str) -> Option<&'static str> {
    entry(id).map(|entry| entry.template)
}

/// Fill in an entry's placeholders.
///
/// `None` when the id is unknown, or when a placeholder has no value: a
/// half-filled string is worse than no string, and a missing argument is a
/// bug rather than a wording.
pub fn render(id: &str, args: &[(&str, &str)]) -> Option<String> {
    let template = text(id)?;
    let mut rendered = template.to_string();
    for (name, value) in args {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    if rendered.contains('{') || rendered.contains('}') {
        return None;
    }
    Some(rendered)
}

/// The lint: the rules of `docs/contract/voice.md`, as code.
pub mod lint {
    use super::{
        DirectionRole, Entry, StringKind, BANNED_PHRASES, CATALOGUE, DIRECTION_SET,
        EXTRA_BANNED_PHRASES, MIN_PATTERN_SAMPLE, VITAL_WORDS,
    };

    /// The rule a string breaks.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Rule {
        /// A banned word or construction.
        BannedWord,
        /// A `direction` string that is not one of the three words.
        DirectionOutsideClosedSet,
        /// A descriptive tail that tries to be the direction word.
        DirectionTailReserved,
        /// A `pattern` string with no sample size in it.
        PatternWithoutSampleSize,
        /// An `option` string that names a vital.
        OptionReferencesVital,
        /// A rendered statement that still carries a hole.
        StatementHole,
    }

    impl Rule {
        /// The name of the rule, as the exported catalogue writes it.
        pub fn name(&self) -> &'static str {
            match self {
                Rule::BannedWord => "banned_word",
                Rule::DirectionOutsideClosedSet => "direction_outside_closed_set",
                Rule::DirectionTailReserved => "direction_tail_reserved",
                Rule::PatternWithoutSampleSize => "pattern_without_sample_size",
                Rule::OptionReferencesVital => "option_references_vital",
                Rule::StatementHole => "statement_hole",
            }
        }
    }

    /// One broken rule.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Violation {
        /// The rule that was broken.
        pub rule: Rule,
        /// The catalogue entry that broke it, when there is one.
        pub entry_id: Option<&'static str>,
        /// What is wrong, in a sentence.
        pub detail: String,
    }

    impl std::fmt::Display for Violation {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.entry_id {
                Some(id) => write!(formatter, "{}: {} ({})", id, self.detail, self.rule.name()),
                None => write!(formatter, "{} ({})", self.detail, self.rule.name()),
            }
        }
    }

    /// Every banned phrase, in one list.
    pub fn banned_phrases() -> Vec<&'static str> {
        BANNED_PHRASES
            .iter()
            .chain(EXTRA_BANNED_PHRASES.iter())
            .copied()
            .collect()
    }

    /// Whether a phrase is banned.
    pub fn is_banned(phrase: &str) -> bool {
        banned_phrases().contains(&phrase)
    }

    /// Whether a text contains a phrase, on word boundaries and without
    /// regard to case.
    ///
    /// Word boundaries matter: "resting" is not the command "rest today", and
    /// "mustard" is not "must".
    pub fn contains_phrase(text: &str, phrase: &str) -> bool {
        let haystack = text.to_lowercase();
        let needle = phrase.to_lowercase();
        if needle.is_empty() || haystack.len() < needle.len() {
            return false;
        }
        let mut start = 0;
        while let Some(found) = haystack[start..].find(&needle) {
            let begin = start + found;
            let end = begin + needle.len();
            let before_ok = begin == 0 || !is_word_byte(haystack.as_bytes()[begin - 1]);
            let after_ok = end == haystack.len() || !is_word_byte(haystack.as_bytes()[end]);
            if before_ok && after_ok {
                return true;
            }
            start = begin + 1;
            if start >= haystack.len() {
                break;
            }
        }
        false
    }

    /// Whether a byte is part of a word.
    fn is_word_byte(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80
    }

    /// Check a string of a known register.
    ///
    /// This is the raw check: no exemptions apply, because a rendered string
    /// has no catalogue id to be exempt under.
    pub fn check_text(kind: StringKind, role: Option<DirectionRole>, text: &str) -> Vec<Violation> {
        check_text_with(kind, role, text, None)
    }

    /// Check one catalogue entry.
    pub fn check_entry(entry: &Entry) -> Vec<Violation> {
        check_text_with(entry.kind, entry.role, entry.template, Some(entry.id))
    }

    /// Check every entry in the catalogue.
    pub fn check_catalogue() -> Vec<Violation> {
        CATALOGUE.iter().flat_map(check_entry).collect()
    }

    /// Check a rendered pattern: it must carry its sample size.
    pub fn check_rendered_pattern(text: &str, sample_size: usize) -> Vec<Violation> {
        if sample_size < MIN_PATTERN_SAMPLE {
            return vec![Violation {
                rule: Rule::PatternWithoutSampleSize,
                entry_id: None,
                detail: format!(
                    "a pattern needs {MIN_PATTERN_SAMPLE} comparable days, not {sample_size}"
                ),
            }];
        }
        if !contains_number(text, sample_size) {
            return vec![Violation {
                rule: Rule::PatternWithoutSampleSize,
                entry_id: None,
                detail: format!("the pattern does not carry its sample size ({sample_size})"),
            }];
        }
        Vec::new()
    }

    /// Check a rendered statement: it must carry no hole.
    ///
    /// A statement is rendered by the core, with the values it compares
    /// already in the sentence. A hole left in it is a place for the second
    /// implementation to grow back, so one is a failure, not a warning.
    pub fn check_rendered_statement(text: &str) -> Vec<Violation> {
        if text.contains('{') || text.contains('}') {
            return vec![Violation {
                rule: Rule::StatementHole,
                entry_id: None,
                detail: format!("the rendered statement still carries a hole: \"{text}\""),
            }];
        }
        Vec::new()
    }

    /// Whether a text carries a number.
    fn contains_number(text: &str, number: usize) -> bool {
        let digits = number.to_string();
        let mut start = 0;
        while let Some(found) = text[start..].find(&digits) {
            let begin = start + found;
            let end = begin + digits.len();
            let before_ok = begin == 0 || !is_word_byte(text.as_bytes()[begin - 1]);
            let after_ok = end == text.len() || !is_word_byte(text.as_bytes()[end]);
            if before_ok && after_ok {
                return true;
            }
            start = begin + 1;
            if start >= text.len() {
                break;
            }
        }
        false
    }

    /// The check itself.
    fn check_text_with(
        kind: StringKind,
        role: Option<DirectionRole>,
        text: &str,
        entry_id: Option<&'static str>,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        for phrase in banned_phrases() {
            if contains_phrase(text, phrase) {
                violations.push(Violation {
                    rule: Rule::BannedWord,
                    entry_id,
                    detail: format!("the banned phrase \"{phrase}\""),
                });
            }
        }

        if kind == StringKind::Direction {
            match role {
                Some(DirectionRole::Primary) => {
                    if !DIRECTION_SET.contains(&text) {
                        violations.push(Violation {
                            rule: Rule::DirectionOutsideClosedSet,
                            entry_id,
                            detail: format!("\"{text}\" is not one of the three direction words"),
                        });
                    }
                }
                _ => {
                    for word in DIRECTION_SET {
                        if contains_phrase(text, word) {
                            violations.push(Violation {
                                rule: Rule::DirectionTailReserved,
                                entry_id,
                                detail: format!("\"{text}\" carries the direction word \"{word}\""),
                            });
                        }
                    }
                }
            }
        }

        if kind == StringKind::Option {
            for word in VITAL_WORDS {
                if contains_phrase(text, word) {
                    violations.push(Violation {
                        rule: Rule::OptionReferencesVital,
                        entry_id,
                        detail: format!("the option names a vital (\"{word}\")"),
                    });
                }
            }
        }

        if kind == StringKind::Pattern && !text.contains("{n}") {
            violations.push(Violation {
                rule: Rule::PatternWithoutSampleSize,
                entry_id,
                detail: "the pattern carries no sample size".to_string(),
            });
        }

        violations
    }
}

/// The catalogue as JSON, for the frontend's lint.
///
/// The rules travel with the strings: the ban list, the closed set, the
/// minimum sample size, the vital words and the exemptions are all in here,
/// so the lint that runs on the frontend reads the same rules this module
/// applies.
pub fn catalogue_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"version\": 1,\n");
    out.push_str("  \"rules\": {\n");
    out.push_str(&format!(
        "    \"banned_phrases\": {},\n",
        json_array(&lint::banned_phrases())
    ));
    out.push_str(&format!(
        "    \"direction_set\": {},\n",
        json_array(&DIRECTION_SET)
    ));
    out.push_str(&format!(
        "    \"vital_words\": {},\n",
        json_array(&VITAL_WORDS)
    ));
    out.push_str(&format!(
        "    \"min_pattern_sample\": {MIN_PATTERN_SAMPLE},\n"
    ));
    out.push_str("    \"match\": \"case_insensitive_word\"\n");
    out.push_str("  },\n");
    out.push_str("  \"entries\": [\n");
    for (index, entry) in CATALOGUE.iter().enumerate() {
        let comma = if index + 1 == CATALOGUE.len() {
            ""
        } else {
            ","
        };
        let role = match entry.role {
            Some(role) => json_string(role.name()),
            None => "null".to_string(),
        };
        out.push_str(&format!(
            "    {{ \"id\": {}, \"kind\": {}, \"role\": {}, \"template\": {} }}{comma}\n",
            json_string(entry.id),
            json_string(entry.kind.name()),
            role,
            json_string(entry.template)
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

/// A JSON array of strings.
fn json_array(values: &[&str]) -> String {
    let items: Vec<String> = values.iter().map(|value| json_string(value)).collect();
    format!("[{}]", items.join(", "))
}

/// A JSON string.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if (other as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", other as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::lint::{self, Rule};
    use super::*;

    #[test]
    fn the_catalogue_itself_breaks_no_rule() {
        let violations = lint::check_catalogue();
        assert!(
            violations.is_empty(),
            "the catalogue breaks its own rules: {violations:?}"
        );
    }

    #[test]
    fn the_closed_set_is_three_words_and_no_more() {
        assert_eq!(DIRECTION_SET.len(), 3);
        assert_eq!(
            text("direction.better_than_usual"),
            Some("better than usual")
        );
        assert_eq!(text("direction.about_usual"), Some("about usual"));
        assert_eq!(text("direction.could_be_better"), Some("could be better"));
        // The set cannot grow: a fourth word is not in the catalogue.
        assert!(text("direction.much_worse").is_none());
    }

    #[test]
    fn a_direction_string_outside_the_set_is_refused() {
        let violations = lint::check_text(
            StringKind::Direction,
            Some(DirectionRole::Primary),
            "much worse than usual",
        );
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, Rule::DirectionOutsideClosedSet);
    }

    #[test]
    fn an_option_string_that_names_a_vital_is_refused() {
        let violations = lint::check_text(
            StringKind::Option,
            None,
            "a 5 km, because your sleep was low",
        );
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, Rule::OptionReferencesVital);
    }

    #[test]
    fn an_imperative_or_a_verdict_is_refused() {
        for phrase in [
            "take the day",
            "you should rest",
            "keep it easy",
            "a bad night",
        ] {
            let violations = lint::check_text(StringKind::Status, None, phrase);
            assert!(
                violations
                    .iter()
                    .any(|violation| violation.rule == Rule::BannedWord),
                "\"{phrase}\" should have been refused"
            );
        }
    }

    #[test]
    fn a_pattern_without_a_sample_size_is_refused() {
        let violations = lint::check_text(
            StringKind::Pattern,
            None,
            "the other times your sleep was low, you ran 8 km or less",
        );
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, Rule::PatternWithoutSampleSize);
    }

    #[test]
    fn a_pattern_rendered_without_its_sample_size_is_refused() {
        let violations = lint::check_rendered_pattern("you ran 8 km or less", 5);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, Rule::PatternWithoutSampleSize);
        // And one rendered with it is fine.
        assert!(
            lint::check_rendered_pattern("the 5 other times, you ran 8 km or less", 5).is_empty()
        );
        // Two comparable days is not a pattern.
        assert!(
            !lint::check_rendered_pattern("the 2 other times, you ran 8 km or less", 2).is_empty()
        );
    }

    #[test]
    fn the_rest_label_is_a_name_not_a_command() {
        // "Rest" is the product's own summary row; the ban is on the command.
        let entry = match entry("vital.rest.label") {
            Some(entry) => entry,
            None => return,
        };
        assert!(lint::check_entry(entry).is_empty());
        // The commands are refused.
        for command in ["rest today", "rest day", "rest up"] {
            let violations = lint::check_text(StringKind::Status, None, command);
            assert!(
                violations
                    .iter()
                    .any(|violation| violation.rule == Rule::BannedWord),
                "\"{command}\" should have been refused"
            );
        }
    }

    #[test]
    fn a_placeholder_without_a_value_is_not_a_string() {
        assert_eq!(
            render("direction.above_usual", &[("value", "5")]),
            Some("5 above your usual".to_string())
        );
        assert_eq!(render("direction.above_usual", &[]), None);
        assert_eq!(render("not.an.entry", &[]), None);
    }

    #[test]
    fn the_exported_catalogue_is_valid_json_and_matches_the_file(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let exported = catalogue_json();
        // The golden file is written with STRIDE_CATALOGUE_OUT=<path>; the
        // test then checks the file against what the catalogue exports.
        if let Ok(path) = std::env::var("STRIDE_CATALOGUE_OUT") {
            let written = std::fs::write(&path, &exported);
            assert!(written.is_ok(), "could not write {path}");
            return Ok(());
        }
        let parsed: serde_json::Value = serde_json::from_str(&exported)?;
        let entries = parsed["entries"].as_array().map(|entries| entries.len());
        assert_eq!(entries, Some(CATALOGUE.len()));
        assert_eq!(
            parsed["rules"]["direction_set"]
                .as_array()
                .map(|set| set.len()),
            Some(DIRECTION_SET.len())
        );
        // The checked-in file is the same catalogue, so the frontend's lint
        // reads exactly what this module applies.
        assert_eq!(exported, include_str!("../strings.json"));
        Ok(())
    }
}
