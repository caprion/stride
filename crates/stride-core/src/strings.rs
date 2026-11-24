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
//! | `instruction` | the product's instruction to a model — the run update's prompt |
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
    /// The product's instruction to a model.
    ///
    /// The one string the app never shows: it is the prompt the writer and
    /// the reviewer are given, and it names the register the model is to
    /// write in. It is a template — it carries holes, because the payload
    /// and the draft go into it — and it is here because the prompt is the
    /// product's instruction, not the model's own wording.
    Instruction,
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
            StringKind::Instruction => "instruction",
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
pub const CATALOGUE: [Entry; 209] = [
    // The vitals.
    Entry {
        id: "vital.rest.label",
        kind: StringKind::Vital,
        role: None,
        template: "Rest",
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
    // The rendered values. The number is already formatted by `units.rs`; the
    // template adds the unit, so the API sends "8.4 km" and nobody downstream
    // rounds it again.
    Entry {
        id: "value.distance",
        kind: StringKind::Statement,
        role: None,
        template: "{value} km",
    },
    Entry {
        id: "value.duration",
        kind: StringKind::Statement,
        role: None,
        template: "{value}",
    },
    Entry {
        // A distance in a week: the volume of a window, as a rate, so
        // that two windows of different lengths can be read against each
        // other. The rate names its own unit — a number that does not say
        // what it is says nothing.
        id: "value.distance_per_week",
        kind: StringKind::Statement,
        role: None,
        template: "{value} km a week",
    },
    Entry {
        // A share of the window's days: the consistency of a window, as a
        // rate, for the same reason.
        id: "value.share_of_days",
        kind: StringKind::Statement,
        role: None,
        template: "{value}% of the days",
    },
    Entry {
        id: "value.heart_rate",
        kind: StringKind::Statement,
        role: None,
        template: "{value} bpm",
    },
    Entry {
        id: "value.pace",
        kind: StringKind::Statement,
        role: None,
        template: "{value} /km",
    },
    Entry {
        id: "value.elevation",
        kind: StringKind::Statement,
        role: None,
        template: "{value} m",
    },
    Entry {
        id: "value.difference",
        kind: StringKind::Statement,
        role: None,
        template: "{sign}{value}",
    },
    Entry {
        id: "value.pace_difference",
        kind: StringKind::Statement,
        role: None,
        template: "{value} s/km",
    },
    // The fingerprint's own values: a cadence, a percentage, a stride
    // length and the efficiency factor.
    Entry {
        id: "value.cadence",
        kind: StringKind::Statement,
        role: None,
        template: "{value} spm",
    },
    Entry {
        id: "value.percent",
        kind: StringKind::Statement,
        role: None,
        template: "{value} %",
    },
    Entry {
        id: "value.length",
        kind: StringKind::Statement,
        role: None,
        template: "{value} cm",
    },
    Entry {
        // The efficiency factor: the speed over the heart rate as a
        // fraction of the maximum. The unit is written out, because a
        // bare number would say nothing about what was measured.
        id: "value.efficiency",
        kind: StringKind::Statement,
        role: None,
        template: "{value} m/s per unit of max heart rate",
    },
    Entry {
        // The heart-rate bands: the range, and the honest answer when the
        // bands cannot be computed at all.
        id: "value.heart_rate_range",
        kind: StringKind::Statement,
        role: None,
        template: "{from}-{to} bpm",
    },
    Entry {
        // The band's name: "Z1" … "Z5". The number is the api's, the word is
        // the core's — a label composed from an index is a second place where
        // a word is put together.
        id: "value.zone_label",
        kind: StringKind::Statement,
        role: None,
        template: "Z{index}",
    },
    // The days of the week: the names the app writes.
    Entry {
        id: "weekday.monday",
        kind: StringKind::Vital,
        role: None,
        template: "Monday",
    },
    Entry {
        id: "weekday.tuesday",
        kind: StringKind::Vital,
        role: None,
        template: "Tuesday",
    },
    Entry {
        id: "weekday.wednesday",
        kind: StringKind::Vital,
        role: None,
        template: "Wednesday",
    },
    Entry {
        id: "weekday.thursday",
        kind: StringKind::Vital,
        role: None,
        template: "Thursday",
    },
    Entry {
        id: "weekday.friday",
        kind: StringKind::Vital,
        role: None,
        template: "Friday",
    },
    Entry {
        id: "weekday.saturday",
        kind: StringKind::Vital,
        role: None,
        template: "Saturday",
    },
    Entry {
        id: "weekday.sunday",
        kind: StringKind::Vital,
        role: None,
        template: "Sunday",
    },
    // The story: one sentence about what the run was, and never a judgement
    // about the athlete.
    Entry {
        id: "story.usual.more",
        kind: StringKind::Statement,
        role: None,
        template: "{distance}, {gap} more than your usual {weekday}",
    },
    Entry {
        id: "story.usual.less",
        kind: StringKind::Statement,
        role: None,
        template: "{distance}, {gap} less than your usual {weekday}",
    },
    Entry {
        // The why: the run's own shape, the fact the detail knows and the
        // feed does not.
        id: "why.slowest_split",
        kind: StringKind::Statement,
        role: None,
        template: "the last {distance} were the slowest of the run",
    },
    Entry {
        id: "why.slowest_split_one",
        kind: StringKind::Statement,
        role: None,
        template: "the last {distance} was the slowest of the run",
    },
    Entry {
        id: "why.fastest_split",
        kind: StringKind::Statement,
        role: None,
        template: "the first {distance} were the fastest of the run",
    },
    Entry {
        id: "why.fastest_split_one",
        kind: StringKind::Statement,
        role: None,
        template: "the first {distance} was the fastest of the run",
    },
    Entry {
        id: "story.plain",
        kind: StringKind::Statement,
        role: None,
        template: "{distance} in {duration}, at an average of {heart_rate}",
    },
    Entry {
        id: "story.plain_no_heart_rate",
        kind: StringKind::Statement,
        role: None,
        template: "{distance} in {duration}",
    },
    Entry {
        id: "story.plain_heart_rate_only",
        kind: StringKind::Statement,
        role: None,
        template: "{distance}, at an average of {heart_rate}",
    },
    Entry {
        id: "story.plain_distance_only",
        kind: StringKind::Statement,
        role: None,
        template: "{distance}",
    },
    Entry {
        id: "story.plain_no_distance",
        kind: StringKind::Statement,
        role: None,
        template: "{duration}, at an average of {heart_rate}",
    },
    Entry {
        id: "story.plain_duration_only",
        kind: StringKind::Statement,
        role: None,
        template: "{duration}",
    },
    Entry {
        id: "story.nothing_unusual",
        kind: StringKind::Statement,
        role: None,
        template: "nothing unusual about this one",
    },
    Entry {
        // The honest answer when the bands cannot be computed. The maximum is
        // *derived* from the run's own samples, not stored, and no threshold
        // is stored either: what is missing is a sample that carries a heart
        // rate, so no maximum can be read.
        //
        // The line says **what is missing**, in the runner's words: there are
        // no zones yet, and here is why.
        id: "zones.unavailable",
        kind: StringKind::Statement,
        role: None,
        template: "No zones yet — this run recorded no heart rate",
    },
    Entry {
        // No model has been read yet: the run is not calibrated, and the
        // answer is "not recorded" — never band 1 and never a zero.
        id: "zones.no_model",
        kind: StringKind::Statement,
        role: None,
        template: "No zones yet — your watch's heart-rate zones have not come through",
    },
    Entry {
        // A model is stored, but its floors are not strictly increasing, so
        // it cannot be used.
        //
        // "Where your zones start" is the runner's phrase for the floors:
        // the line says that the order is wrong, not that the bands are.
        id: "zones.unusable_model",
        kind: StringKind::Statement,
        role: None,
        template: "No zones yet — your watch's zones do not start in order",
    },
    Entry {
        // The split of a period of runs: the model is there, but the runs
        // carry no heart-rate samples to measure. There is no share, and
        // the answer says why.
        id: "rolling.zones.no_samples",
        kind: StringKind::Statement,
        role: None,
        template: "No heart rate recorded — none of the runs in this period recorded a heart rate",
    },
    Entry {
        // The seven facets of the athlete's shape: the names of the
        // values, and nothing that says what they mean.
        id: "profile.volume",
        kind: StringKind::Statement,
        role: None,
        template: "Volume",
    },
    Entry {
        id: "profile.consistency",
        kind: StringKind::Statement,
        role: None,
        template: "Consistency",
    },
    Entry {
        id: "profile.intensity",
        kind: StringKind::Statement,
        role: None,
        template: "Heart rate",
    },
    Entry {
        id: "profile.duration",
        kind: StringKind::Statement,
        role: None,
        template: "Duration",
    },
    Entry {
        id: "profile.density",
        kind: StringKind::Statement,
        role: None,
        template: "Density",
    },
    Entry {
        id: "profile.variety",
        kind: StringKind::Statement,
        role: None,
        template: "Sport variety",
    },
    Entry {
        id: "profile.week_days",
        kind: StringKind::Statement,
        role: None,
        template: "Week days",
    },
    Entry {
        // The range of distances a bucket of the distance breakdown holds:
        // "0-5 km".
        id: "value.distance_range",
        kind: StringKind::Statement,
        role: None,
        template: "{from}-{to} km",
    },
    Entry {
        // A range of paces: the band of the pace distribution, "4:30–5:00
        // /km". The seconds are written as a clock, the way a pace is
        // read.
        id: "value.pace_range",
        kind: StringKind::Statement,
        role: None,
        template: "{from}–{to} /km",
    },
    Entry {
        // Not worked out for the pace bands: the period holds no moving
        // samples, so there is no distribution to draw. Never a band of zero.
        id: "pace.no_samples",
        kind: StringKind::Statement,
        role: None,
        template: "No pace bands yet — none of the runs in this period recorded a pace",
    },
    Entry {
        // The three bands the training is distributed across. They are the
        // athlete's own zones, and the names describe what the bands are:
        // no target range and no prescription lives near them.
        id: "zones.band.low",
        kind: StringKind::Statement,
        role: None,
        template: "Z1–Z2 low",
    },
    Entry {
        id: "zones.band.moderate",
        kind: StringKind::Statement,
        role: None,
        template: "Z3 moderate",
    },
    Entry {
        id: "zones.band.high",
        kind: StringKind::Statement,
        role: None,
        template: "Z4–Z5 high",
    },
    Entry {
        // A share of a whole: the time in the zones, the time of the week.
        id: "value.share",
        kind: StringKind::Statement,
        role: None,
        template: "{value}%",
    },
    Entry {
        // The times of day, in the athlete's own local time.
        id: "daytime.morning",
        kind: StringKind::Statement,
        role: None,
        template: "Morning",
    },
    Entry {
        id: "daytime.afternoon",
        kind: StringKind::Statement,
        role: None,
        template: "Afternoon",
    },
    Entry {
        id: "daytime.evening",
        kind: StringKind::Statement,
        role: None,
        template: "Evening",
    },
    Entry {
        id: "daytime.night",
        kind: StringKind::Statement,
        role: None,
        template: "Night",
    },
    Entry {
        // The model's own source line: where the bands came from and when
        // the model was read.
        //
        // The date is **rendered by the caller** — the api renders it in the
        // api's own form ("10 May 2026") and passes the string in. The core
        // does not format a date for a screen: its own canonical form stays
        // inside it.
        id: "zones.model.source",
        kind: StringKind::Statement,
        role: None,
        template: "your zones · {method} · max heart rate {max_hr} · taken from your watch on {date}",
    },
    Entry {
        // The same line for a model Garmin reported no maximum heart rate
        // for: the method and the day are still the model's own.
        id: "zones.model.source.no_max",
        kind: StringKind::Statement,
        role: None,
        template: "your zones · {method} · taken from your watch on {date}",
    },
    Entry {
        // The line for a window that was measured against **more than one**
        // model: the days the models were read, so the reader can see that
        // the window spans a change of the athlete's own configuration.
        //
        // The days are **rendered by the caller**, like the day of the
        // single model's line.
        id: "zones.models.source",
        kind: StringKind::Statement,
        role: None,
        template: "your zones · {count} readings · taken from your watch on {dates}",
    },
    Entry {
        // The reason a split has no value when a model **has** been read
        // but none of them was in force for the runs in question: a run
        // that predates the first stored model is not calibrated.
        id: "zones.no_model_in_force",
        kind: StringKind::Statement,
        role: None,
        template: "No zones yet — your watch's zones were not read for these runs",
    },
    // The plan's statements: what the athlete intended, what happened, and
    // where the intent came from. They are facts, never verdicts.
    Entry {
        id: "statement.plan.source.baseline",
        kind: StringKind::Statement,
        role: None,
        template: "your usual week, inferred from your history",
    },
    Entry {
        id: "statement.plan.source.prescribed",
        kind: StringKind::Statement,
        role: None,
        template: "from your imported plan",
    },
    Entry {
        id: "statement.plan.source.typed",
        kind: StringKind::Statement,
        role: None,
        template: "from your typed plan",
    },
    Entry {
        id: "statement.plan.baseline.description",
        kind: StringKind::Statement,
        role: None,
        template: "your usual week, inferred from your history ({weeks} weeks)",
    },
    Entry {
        id: "statement.plan.baseline.not_enough_history",
        kind: StringKind::Statement,
        role: None,
        template: "not enough history to infer a usual week",
    },
    Entry {
        id: "statement.plan.kind.easy",
        kind: StringKind::Statement,
        role: None,
        template: "easy",
    },
    Entry {
        id: "statement.plan.kind.long",
        kind: StringKind::Statement,
        role: None,
        template: "long",
    },
    Entry {
        id: "statement.plan.kind.quality",
        kind: StringKind::Statement,
        role: None,
        template: "quality",
    },
    Entry {
        id: "statement.plan.kind.rest",
        kind: StringKind::Statement,
        role: None,
        template: "rest",
    },
    Entry {
        id: "statement.plan.kind.other",
        kind: StringKind::Statement,
        role: None,
        template: "{kind}",
    },
    Entry {
        id: "statement.plan.session.label",
        kind: StringKind::Statement,
        role: None,
        template: "{distance} · {kind}",
    },
    Entry {
        id: "statement.plan.session.no_target",
        kind: StringKind::Statement,
        role: None,
        template: "{kind} · no distance target",
    },
    Entry {
        id: "statement.plan.day.monday",
        kind: StringKind::Statement,
        role: None,
        template: "Monday",
    },
    Entry {
        id: "statement.plan.day.tuesday",
        kind: StringKind::Statement,
        role: None,
        template: "Tuesday",
    },
    Entry {
        id: "statement.plan.day.wednesday",
        kind: StringKind::Statement,
        role: None,
        template: "Wednesday",
    },
    Entry {
        id: "statement.plan.day.thursday",
        kind: StringKind::Statement,
        role: None,
        template: "Thursday",
    },
    Entry {
        id: "statement.plan.day.friday",
        kind: StringKind::Statement,
        role: None,
        template: "Friday",
    },
    Entry {
        id: "statement.plan.day.saturday",
        kind: StringKind::Statement,
        role: None,
        template: "Saturday",
    },
    Entry {
        id: "statement.plan.day.sunday",
        kind: StringKind::Statement,
        role: None,
        template: "Sunday",
    },
    Entry {
        id: "statement.plan.adherence",
        kind: StringKind::Statement,
        role: None,
        template: "{done} of {planned} sessions run",
    },
    Entry {
        id: "statement.plan.adherence.baseline",
        kind: StringKind::Statement,
        role: None,
        template: "{done} sessions in your usual week",
    },
    Entry {
        id: "statement.plan.adherence.baseline.one",
        kind: StringKind::Statement,
        role: None,
        template: "1 session in your usual week",
    },
    Entry {
        id: "statement.plan.result.target",
        kind: StringKind::Statement,
        role: None,
        template: "{actual} of a {target} target",
    },
    Entry {
        id: "statement.plan.result.no_target",
        kind: StringKind::Statement,
        role: None,
        template: "{actual} · no distance target",
    },
    Entry {
        id: "statement.plan.result.no_run",
        kind: StringKind::Statement,
        role: None,
        template: "{target} target · no run recorded",
    },
    Entry {
        id: "statement.plan.result.empty",
        kind: StringKind::Statement,
        role: None,
        template: "no run recorded · no distance target",
    },
    Entry {
        id: "statement.plan.result.no_distance_with_target",
        kind: StringKind::Statement,
        role: None,
        template: "run recorded · no distance recorded · {target} target",
    },
    Entry {
        id: "statement.plan.result.no_distance",
        kind: StringKind::Statement,
        role: None,
        template: "run recorded · no distance recorded",
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
    // The training load. A scenario is a statement with its assumption
    // attached: it says what it assumes, and it never names a verdict.
    Entry {
        id: "statement.load.scenario.ratio.now",
        kind: StringKind::Statement,
        role: None,
        template:
            "the last {acute} days are already inside {band} times the last {chronic}",
    },
    Entry {
        id: "statement.load.scenario.ratio.days",
        kind: StringKind::Statement,
        role: None,
        template:
            "the last {acute} days are expected back inside {band} times the last {chronic} in {days}",
    },
    Entry {
        id: "statement.load.scenario.ratio.undetermined",
        kind: StringKind::Statement,
        role: None,
        template:
            "the last {acute} days are not back inside {band} times the last {chronic} within {days} days",
    },
    Entry {
        id: "statement.load.scenario.form.now",
        kind: StringKind::Statement,
        role: None,
        template: "the difference is positive now",
    },
    Entry {
        id: "statement.load.scenario.form.days",
        kind: StringKind::Statement,
        role: None,
        template: "the difference turns positive in {days}",
    },
    Entry {
        id: "statement.load.scenario.form.undetermined",
        kind: StringKind::Statement,
        role: None,
        template: "the difference does not turn positive within {days} days",
    },
    Entry {
        id: "statement.load.assumptions",
        kind: StringKind::Statement,
        role: None,
        template: "no training for {forecast} days · Banister k = {k} · max heart rate {max_hr} from your zones, taken from your watch on {date} · the last {acute} days · the last {chronic}",
    },
    // The fingerprint: the labels of the observations, and the reasons a
    // run has none of them. They are facts about the run and never about
    // the athlete.
    Entry {
        id: "fingerprint.label.cadence",
        kind: StringKind::Vital,
        role: None,
        template: "Cadence",
    },
    Entry {
        id: "fingerprint.label.cadence_variability",
        kind: StringKind::Vital,
        role: None,
        template: "Cadence variability",
    },
    Entry {
        id: "fingerprint.label.efficiency",
        kind: StringKind::Vital,
        role: None,
        template: "Efficiency factor",
    },
    Entry {
        id: "fingerprint.label.pace_variability",
        kind: StringKind::Vital,
        role: None,
        template: "Pace variability",
    },
    Entry {
        id: "fingerprint.label.cadence_drift",
        kind: StringKind::Vital,
        role: None,
        template: "Cadence drift",
    },
    Entry {
        id: "fingerprint.label.heart_rate_drift",
        kind: StringKind::Vital,
        role: None,
        template: "Heart rate drift",
    },
    Entry {
        id: "fingerprint.label.pace_drift",
        kind: StringKind::Vital,
        role: None,
        template: "Pace drift",
    },
    Entry {
        id: "fingerprint.label.decoupling",
        kind: StringKind::Vital,
        role: None,
        template: "Aerobic decoupling",
    },
    Entry {
        id: "fingerprint.label.stride_length",
        kind: StringKind::Vital,
        role: None,
        template: "Stride length",
    },
    Entry {
        id: "fingerprint.no_cadence",
        kind: StringKind::Statement,
        role: None,
        template: "the run has no cadence readings",
    },
    Entry {
        id: "fingerprint.no_heart_rate",
        kind: StringKind::Statement,
        role: None,
        template: "the run has no heart-rate readings",
    },
    Entry {
        id: "fingerprint.no_pace",
        kind: StringKind::Statement,
        role: None,
        template: "the run has no pace readings",
    },
    Entry {
        id: "fingerprint.too_few_laps",
        kind: StringKind::Statement,
        role: None,
        template: "the run has too few laps to measure a variation from",
    },
    Entry {
        id: "fingerprint.too_few_laps_to_compare",
        kind: StringKind::Statement,
        role: None,
        template: "the run has too few laps to compare one end with the other",
    },
    Entry {
        id: "fingerprint.no_stride_length",
        kind: StringKind::Statement,
        role: None,
        template: "the run's file carries no stride length",
    },
    Entry {
        id: "fingerprint.no_reference",
        kind: StringKind::Statement,
        role: None,
        template: "no maximum heart rate to measure the efficiency factor against",
    },
    Entry {
        id: "fingerprint.no_samples",
        kind: StringKind::Statement,
        role: None,
        template: "the run has no samples to read",
    },
    Entry {
        id: "fingerprint.no_time_in_zones",
        kind: StringKind::Statement,
        role: None,
        template: "the run has no time in any of the zones",
    },
    Entry {
        id: "fingerprint.unavailable",
        kind: StringKind::Statement,
        role: None,
        template: "the run has no file to read",
    },
    Entry {
        // Where the zone model came from, and when it was read. A number
        // that cannot say where its input came from is not inspectable.
        id: "fingerprint.zones.source",
        kind: StringKind::Statement,
        role: None,
        template: "your zones, taken from your watch on {date}",
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
    Entry {
        id: "status.saved_running_history",
        kind: StringKind::Status,
        role: None,
        template: "Your saved activity history",
    },
    Entry {
        // The version's label. The version itself is not the core's business —
        // it comes from the build environment — but how it is shown is, and
        // this is the only place a version string is put together.
        id: "status.version",
        kind: StringKind::Status,
        role: None,
        template: "Version {version}",
    },
    Entry {
        id: "year.changed.distance",
        kind: StringKind::Statement,
        role: None,
        template: "{difference} against the same days, a week earlier ({current} against {previous})",
    },
    Entry {
        id: "year.changed.sessions",
        kind: StringKind::Statement,
        role: None,
        template: "{difference} recorded sessions ({current} against {previous})",
    },
    Entry {
        id: "year.changed.longest.grew",
        kind: StringKind::Statement,
        role: None,
        template: "the longest run grew by {gap} ({current} against {previous})",
    },
    Entry {
        id: "year.changed.longest.shortened",
        kind: StringKind::Statement,
        role: None,
        template: "the longest run shortened by {gap} ({current} against {previous})",
    },
    Entry {
        id: "year.changed.longest.same",
        kind: StringKind::Statement,
        role: None,
        template: "the longest run was {current} in both periods",
    },
    Entry {
        id: "year.changed.pace",
        kind: StringKind::Statement,
        role: None,
        template: "the average pace was {current} against {previous}",
    },
    Entry {
        id: "year.changed.pace.one",
        kind: StringKind::Statement,
        role: None,
        template: "the average pace was {current}",
    },
    Entry {
        id: "year.changed.form",
        kind: StringKind::Statement,
        role: None,
        template:
            "the difference was {current} at the end of this period against {previous} at the end of the same days, a week earlier",
    },

    Entry {
        // The space **outside** the athlete's zones: a reading at or below
        // the first floor, or no reading at all.
        //
        // It is a name and not a verdict: the run's film draws it as a band
        // behind the pace and the heart rate, and the athlete reads where
        // they were. Nothing here says the reading was too low.
        id: "film.zone.outside",
        kind: StringKind::Statement,
        role: None,
        template: "outside your zones",
    },

    Entry {
        id: "update.payload.distance",
        kind: StringKind::Statement,
        role: None,
        template: "Distance",
    },
    Entry {
        id: "update.payload.duration",
        kind: StringKind::Statement,
        role: None,
        template: "Duration",
    },
    Entry {
        id: "update.payload.pace",
        kind: StringKind::Statement,
        role: None,
        template: "Average pace",
    },
    Entry {
        id: "update.payload.heart_rate",
        kind: StringKind::Statement,
        role: None,
        template: "Average heart rate",
    },
    Entry {
        id: "update.payload.highest_heart_rate",
        kind: StringKind::Statement,
        role: None,
        template: "Highest heart rate",
    },
    Entry {
        id: "update.payload.elevation",
        kind: StringKind::Statement,
        role: None,
        template: "Elevation gain",
    },
    Entry {
        id: "update.payload.splits",
        kind: StringKind::Statement,
        role: None,
        template: "The laps",
    },
    Entry {
        id: "update.payload.zones",
        kind: StringKind::Statement,
        role: None,
        template: "Zones",
    },
    Entry {
        id: "update.payload.observations",
        kind: StringKind::Statement,
        role: None,
        template: "Observations",
    },
    Entry {
        id: "update.direction.faster",
        kind: StringKind::Statement,
        role: None,
        template: "faster",
    },
    Entry {
        id: "update.direction.slower",
        kind: StringKind::Statement,
        role: None,
        template: "slower",
    },
    Entry {
        id: "update.direction.higher",
        kind: StringKind::Statement,
        role: None,
        template: "higher",
    },
    Entry {
        id: "update.direction.lower",
        kind: StringKind::Statement,
        role: None,
        template: "lower",
    },
    Entry {
        id: "update.direction.held",
        kind: StringKind::Statement,
        role: None,
        template: "held",
    },
    Entry {
        id: "update.check.groundedness",
        kind: StringKind::Status,
        role: None,
        template: "the number {value} is not in the payload",
    },
    Entry {
        id: "update.check.direction",
        kind: StringKind::Status,
        role: None,
        template: "the {label} is {stated} in the payload, and the draft says {claim}",
    },
    Entry {
        id: "update.check.register",
        kind: StringKind::Status,
        role: None,
        template: "the word {word} is not one the app uses",
    },
    Entry {
        id: "update.payload.notes",
        kind: StringKind::Statement,
        role: None,
        template: "The notes",
    },
    Entry {
        id: "update.payload.note.laps",
        kind: StringKind::Instruction,
        role: None,
        template:
            "Each line of the laps is one lap of the run: its number, how far that lap covered, its pace and its average heart rate. A lap is not a kilometre — the laps of a run do not add up to a kilometre count.",
    },
    Entry {
        id: "update.payload.note.zones",
        kind: StringKind::Instruction,
        role: None,
        template:
            "The zones are the athlete's own zones, read from the app: the first band is the slowest and the last the fastest. The share is the share of the run's time in that band.",
    },
    Entry {
        id: "update.payload.note.pace_drift",
        kind: StringKind::Instruction,
        role: None,
        template:
            "Pace drift is the second half's average pace minus the first half's, in seconds per kilometre. A negative pace drift means the second half was the faster half.",
    },
    Entry {
        id: "update.payload.note.heart_rate_drift",
        kind: StringKind::Instruction,
        role: None,
        template:
            "Heart rate drift is the second half's average heart rate minus the first half's, in beats per minute. A positive drift means the heart rate rose.",
    },
    Entry {
        id: "update.payload.note.cadence_drift",
        kind: StringKind::Instruction,
        role: None,
        template:
            "Cadence drift is the last third's average cadence minus the first third's, as a percentage. A positive drift means the cadence rose.",
    },
    Entry {
        id: "update.payload.note.aerobic_decoupling",
        kind: StringKind::Instruction,
        role: None,
        template:
            "Aerobic decoupling is the first third's efficiency minus the last third's. A positive decoupling means the efficiency fell as the run went on, and a negative one means it rose.",
    },
    Entry {
        id: "update.payload.note.observations",
        kind: StringKind::Instruction,
        role: None,
        template:
            "Each line of the observations is one measurement of the run, with the direction the app read in it: faster, slower, higher, lower or held.",
    },
    Entry {
        id: "update.check.shape",
        kind: StringKind::Status,
        role: None,
        template: "the draft has {count} sentences, and the shape is four to six",
    },
    Entry {
        id: "update.check.shape.you",
        kind: StringKind::Status,
        role: None,
        template: "the draft does not address the athlete",
    },
    Entry {
        id: "update.check.shape.bullets",
        kind: StringKind::Status,
        role: None,
        template: "the draft is a list, and the shape is prose",
    },
    Entry {
        id: "update.prompt.writer",
        kind: StringKind::Instruction,
        role: None,
        template:
            "You are writing the run update for a running log — a private record of the runner's own runs.\n\nHere is the run, exactly as the app recorded it. Every number you may use is in this payload. Do not use any other number, do not estimate, and do not carry a number over from another run. The notes inside the payload say what the app's fields mean; follow them.\n\n{payload}\n\nWrite the update in four to six sentences.\n\n- Say what the run was and how it went.\n- If something in the laps or the mechanics stands out, say what stands out.\n- Address the runner as \"you\".\n- Observations only: no advice, no verdict, no score, no commands. Do not tell the runner what to do, and do not call the run good or bad.\n- Do not use these words: should, must, missed, bad, poor, fail, behind, good, red, recommended, better to, need to, ready to.\n- Plain prose. No preamble, no markdown, no bullet points, no labels, no sign-off.\n\nReturn only the update.",
    },
    Entry {
        id: "update.prompt.reviewer",
        kind: StringKind::Instruction,
        role: None,
        template:
            "You are the reviewer of a run update for a private running log. Another model wrote the draft below from the payload, and the draft reaches the runner only after you have checked it.\n\nHere is the run, exactly as the app recorded it, with the app's own notes on what the fields mean:\n\n{payload}\n\nHere is the draft:\n\n{draft}\n\nCheck the draft against the payload, for these things:\n- any number that is not in the payload;\n- any claim whose direction contradicts the payload (a negative pace drift is a faster second half; a positive heart rate drift is a heart rate that rose; a negative aerobic decoupling is an efficiency that rose);\n- any banned word (should, must, missed, bad, poor, fail, behind, good, red, recommended, better to, need to, ready to);\n- any advice, verdict, score or command;\n- any preamble, markdown, bullet, label or reasoning that leaked into the text;\n- a length outside four to six sentences;\n- anything that contradicts the app's own numbers.\n\nAnswer with one word first: ship, repair or discard. Say ship when the draft is true to the payload and written in the register. Say repair when the draft can be corrected, and name what to correct. Say discard when it cannot be corrected.\n\nHere are the findings the checks found: {findings}",
    },
    Entry {
        id: "update.prompt.repair",
        kind: StringKind::Instruction,
        role: None,
        template:
            "You are writing the run update for a running log — a private record of the runner's own runs. Another model wrote a draft from the same payload, and a reviewer has said what is wrong with it.\n\nHere is the run, exactly as the app recorded it. Every number you may use is in this payload. Do not use any other number, do not estimate, and do not carry a number over from another run. The notes inside the payload say what the app's fields mean; follow them.\n\n{payload}\n\nHere is the draft:\n\n{draft}\n\nHere is what the reviewer found:\n\n{findings}\n\nWrite the update again, in four to six sentences.\n\n- Keep what the findings do not question, and correct what they question.\n- Address the runner as \"you\".\n- Observations only: no advice, no verdict, no score, no commands. Do not tell the runner what to do, and do not call the run good or bad.\n- Do not use these words: should, must, missed, bad, poor, fail, behind, good, red, recommended, better to, need to, ready to.\n- Plain prose. No preamble, no markdown, no bullet points, no labels, no sign-off.\n\nReturn only the update.",
    },


    // ---- The effort model, the ladder and the threshold (rig v2, step 2).
    //
    // The heart-rate **band** is a computed boundary — the method's
    // percentage of the athlete's own maximum heart rate — so its edges are
    // written as a band with an en dash, and the ceiling is written as one.
    Entry {
        id: "value.heart_rate_band",
        kind: StringKind::Statement,
        role: None,
        template: "{from}–{to} bpm",
    },
    Entry {
        id: "value.heart_rate_below",
        kind: StringKind::Statement,
        role: None,
        template: "below {value} bpm",
    },
    Entry {
        id: "pace.chain",
        kind: StringKind::Statement,
        role: None,
        template: "VDOT {vdot} · Daniels' relations",
    },
    Entry {
        id: "pace.source.race",
        kind: StringKind::Statement,
        role: None,
        template: "from your {distance} in {time} on {date}",
    },
    Entry {
        id: "pace.source.equivalents",
        kind: StringKind::Statement,
        role: None,
        template: "{equivalents} — the equivalent race times, by Riegel's exponent 1.06",
    },
    Entry {
        id: "pace.rule.line",
        kind: StringKind::Statement,
        role: None,
        template: "{wording} ({citation})",
    },
    Entry {
        id: "pace.rule.general_aerobic",
        kind: StringKind::Statement,
        role: None,
        template: "70–81 % of your maximum heart rate · 62–75 % of your heart-rate reserve — the method fixes no pace",
    },
    Entry {
        id: "pace.rule.recovery",
        kind: StringKind::Statement,
        role: None,
        template: "+120 s/mi (+75 s/km) slower than your 15K-to-half-marathon race pace · below 76 % of your maximum heart rate",
    },
    Entry {
        id: "pace.rule.threshold",
        kind: StringKind::Statement,
        role: None,
        template: "the pace you could race for about an hour — the book places it at your 15K-to-10-mile race pace",
    },
    Entry {
        id: "pace.rule.long_run",
        kind: StringKind::Statement,
        role: None,
        template: "20–33 % slower than your 10K race pace at the start · 17–29 % slower than your 15K-to-half-marathon race pace at the finish",
    },
    Entry {
        id: "pace.rule.progression",
        kind: StringKind::Statement,
        role: None,
        template: "the last 3 miles at the threshold pace",
    },
    Entry {
        id: "pace.rule.goal_race",
        kind: StringKind::Statement,
        role: None,
        template: "your own race at the goal distance",
    },
    Entry {
        id: "pace.missing.race",
        kind: StringKind::Status,
        role: None,
        template: "no race time entered — the paces have no value yet",
    },
    Entry {
        id: "pace.missing.max_hr",
        kind: StringKind::Status,
        role: None,
        template: "no maximum heart rate — your watch's zones hold none, so the heart-rate bands have no value",
    },
    Entry {
        id: "pace.missing.resting_hr",
        kind: StringKind::Status,
        role: None,
        template: "no resting heart rate — the bands as a share of your heart-rate reserve have no value",
    },
    Entry {
        id: "pace.missing.goal",
        kind: StringKind::Status,
        role: None,
        template: "no goal distance — the rule that places the threshold band needs to know which distance you race",
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

        // The ban list holds what the **athlete** reads. An `instruction` is
        // the one string the athlete never sees: it is the product's own
        // instruction to a model, and a prompt that forbids a word has to
        // name it — "do not use these words: should, must, …" — which is
        // the opposite of using one.
        if kind != StringKind::Instruction {
            for phrase in banned_phrases() {
                if contains_phrase(text, phrase) {
                    violations.push(Violation {
                        rule: Rule::BannedWord,
                        entry_id,
                        detail: format!("the banned phrase \"{phrase}\""),
                    });
                }
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
    fn the_zones_s_honest_answer_says_what_is_actually_missing() {
        // The maximum is derived from the run's own samples and no threshold
        // is stored: what is missing is a sample that carries a heart rate.
        assert_eq!(
            text("zones.unavailable"),
            Some("No zones yet — this run recorded no heart rate")
        );
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
