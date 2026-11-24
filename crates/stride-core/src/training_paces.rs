//! The ladder's rungs — the training paces, each with the rule it comes from.
//!
//! Every rung is **two things**: a pace where the method states one, and a
//! heart-rate band where the method states one. The two are never turned
//! into one another, and a rung that has no pace says so — the method fixes
//! no pace for the general-aerobic day, and the card says that rather than
//! inventing one (`exploration/pace-engine.md` §3, `exploration/
//! pfitzinger-spine.md` §5.2).
//!
//! ## The rules, and where they come from
//!
//! The rules are **Pfitzinger's** (*Faster Road Racing: 5K to Half
//! Marathon*, 2nd ed., 2015) and each carries its chapter. The **anchors**
//! are the engine's own: the athlete's race time turned into equivalents by
//! Riegel's exponent. The two are never blurred — every rule carries its
//! identifier, its citation and the wording the surface prints, so the
//! number is printed beside the rule that produced it.
//!
//! | rung | the rule | citation |
//! |---|---|---|
//! | general aerobic | a heart-rate band and **no pace** | ch. 7, ch. 8 |
//! | recovery | +120 s/mi (+75 s/km) off the 15K–HM race pace, below 76 % maxHR | ch. 2, ch. 7 |
//! | lactate threshold | the hour-pace band — "the pace you could race for about an hour" | ch. 1, ch. 7 |
//! | the long run | 10K pace × 1.20–1.33 at the start, 15K–HM pace × 1.17–1.29 at the finish | ch. 1, ch. 7 |
//! | the goal race | the athlete's own race at the goal distance | ch. 10, ch. 12 |
//!
//! ## The two anchors, named and never mixed
//!
//! * the **goal race pace** — the athlete's own race at the goal distance;
//! * the **LT band** — the hour-pace band, on the goal distance's span.
//!
//! The threshold candidate in [`crate::threshold`] computes the *same rule*
//! on a *different anchor* — the athlete's typed race — and the two are
//! never quoted as one another (`exploration/pace-engine.md` §2.3).
//!
//! ## What this module never does
//!
//! * It never guesses a pace for a rung the method leaves open.
//! * It never converts a heart rate into a pace, or a pace into a heart rate.
//! * It never computes a maximum heart rate from an age: the maximum comes
//!   from the athlete's own dated zone model, and the resting heart rate
//!   from their own readings. When either is missing, the band has **no
//!   value with a reason**.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::strings;
use crate::units;
use crate::vo2;

/// A race the athlete typed: an observation with its day.
///
/// A race time is never inferred from the training and never invented; this
/// is the whole of the engine's input for the paces
/// (`exploration/pace-engine.md` §4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Race {
    /// The race's distance, in metres.
    pub distance_m: f64,
    /// The finish time, in seconds.
    pub time_s: f64,
    /// The day of the race.
    pub date: Date,
    /// The athlete's own name for the race, when they gave one.
    pub name: Option<String>,
}

impl Race {
    /// A race: distance, time and the day it was run.
    pub fn new(distance_m: f64, time_s: f64, date: Date) -> Self {
        Self {
            distance_m,
            time_s,
            date,
            name: None,
        }
    }

    /// The pace of the race, in seconds per kilometre.
    pub fn pace_s_per_km(&self) -> Option<f64> {
        if !self.distance_m.is_finite() || !self.time_s.is_finite() {
            return None;
        }
        if self.distance_m <= 0.0 || self.time_s <= 0.0 {
            return None;
        }
        Some(self.time_s / (self.distance_m / 1_000.0))
    }
}

/// A pace band: the fast edge and the slow edge.
///
/// A band is a band — the method's anchor is a band, and a midpoint is a
/// number the method does not state (`exploration/threshold-setup.md` §0.1).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaceBand {
    /// The fast edge, in seconds per kilometre.
    pub fast_s_per_km: f64,
    /// The slow edge, in seconds per kilometre.
    pub slow_s_per_km: f64,
}

impl PaceBand {
    /// A band with one value at both edges — where the rule gives one pace.
    pub fn single(s_per_km: f64) -> Self {
        Self {
            fast_s_per_km: s_per_km,
            slow_s_per_km: s_per_km,
        }
    }

    /// A band between two paces.
    pub fn between(fast_s_per_km: f64, slow_s_per_km: f64) -> Self {
        Self {
            fast_s_per_km,
            slow_s_per_km,
        }
    }

    /// Whether the band has one value at both edges.
    pub fn is_single(&self) -> bool {
        (self.fast_s_per_km - self.slow_s_per_km).abs() < 0.5
    }

    /// The rendered band: "4:16–4:21 /km", or "4:21 /km" when it has one
    /// value.
    pub fn render(&self) -> Option<String> {
        if self.is_single() {
            units::render_pace(Some(self.fast_s_per_km))
        } else {
            units::render_pace_range(self.fast_s_per_km, self.slow_s_per_km)
        }
    }
}

/// The rule a rung's numbers come from.
///
/// The rule carries its **identifier** (what the plan and the card call it),
/// its **citation** (the chapter it comes from) and the **wording** the
/// surface prints beside the number.
///
/// The three are static — a rule is looked up, never built from the wire —
/// so the type serializes but does not deserialize, and the types that
/// carry it are computed by the core and serialized on the way out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Rule {
    /// The rule's identifier.
    pub id: &'static str,
    /// Where the rule comes from — the book's chapter.
    pub citation: &'static str,
    /// The catalogue id of the wording the surface prints.
    pub string_id: &'static str,
}

impl Rule {
    /// The wording of the rule, as the surface prints it.
    pub fn wording(&self) -> Option<String> {
        strings::text(self.string_id).map(|text| text.to_string())
    }

    /// The rule's line: its wording, with its citation beside it.
    pub fn line(&self) -> Option<String> {
        let wording = self.wording()?;
        strings::render(
            "pace.rule.line",
            &[("wording", &wording), ("citation", self.citation)],
        )
    }
}

/// The general-aerobic rule: a heart-rate band and no pace (ch. 7, ch. 8).
pub const RULE_GENERAL_AEROBIC: Rule = Rule {
    id: "book.general_aerobic",
    citation: "ch. 7, ch. 8",
    string_id: "pace.rule.general_aerobic",
};

/// The recovery rule: +120 s/mi off the 15K–HM race pace (ch. 2, ch. 7).
pub const RULE_RECOVERY: Rule = Rule {
    id: "book.recovery",
    citation: "ch. 2, ch. 7",
    string_id: "pace.rule.recovery",
};

/// The threshold rule: the hour-pace band (ch. 1, ch. 7).
pub const RULE_THRESHOLD: Rule = Rule {
    id: "book.lt_pace_band",
    citation: "ch. 1, ch. 7",
    string_id: "pace.rule.threshold",
};

/// The long-run rule: the two ends of the long run (ch. 1, ch. 7).
pub const RULE_LONG_RUN: Rule = Rule {
    id: "book.long_run",
    citation: "ch. 1, ch. 7",
    string_id: "pace.rule.long_run",
};

/// The progression long run's rule: the last miles at the threshold pace
/// (ch. 7; ch. 11–13).
pub const RULE_PROGRESSION: Rule = Rule {
    id: "book.progression_long_run",
    citation: "ch. 7; ch. 11–13",
    string_id: "pace.rule.progression",
};

/// The goal-race rule: the athlete's own race at the goal distance
/// (ch. 10, ch. 12).
pub const RULE_GOAL_RACE: Rule = Rule {
    id: "book.goal_race",
    citation: "ch. 10, ch. 12",
    string_id: "pace.rule.goal_race",
};

/// The two ends of a band the method states, as shares of something.
///
/// The ends are **named**, and each is written on its own line: a pair of
/// decimals side by side reads as a pair of coordinates to the privacy
/// scanner, and a band of heart rates is not a place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PercentBand {
    /// The lower share.
    pub low: f64,
    /// The upper share.
    pub high: f64,
}

/// The two factors a pace rule multiplies its anchor by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaceFactors {
    /// The smaller factor — the faster end.
    pub fast: f64,
    /// The larger factor — the slower end.
    pub slow: f64,
}

/// The general-aerobic band, as a share of the maximum heart rate.
pub const GA_MAX_HR: PercentBand = PercentBand {
    low: 0.70,
    high: 0.81,
};
/// The general-aerobic band, as a share of the heart-rate reserve.
pub const GA_RESERVE: PercentBand = PercentBand {
    low: 0.62,
    high: 0.75,
};
/// The recovery ceiling, as a share of the maximum heart rate.
pub const RECOVERY_MAX_HR: f64 = 0.76;
/// The recovery ceiling, as a share of the heart-rate reserve.
pub const RECOVERY_RESERVE: f64 = 0.70;
/// The threshold band, as a share of the maximum heart rate.
pub const LT_MAX_HR: PercentBand = PercentBand {
    low: 0.80,
    high: 0.91,
};
/// The threshold band, as a share of the heart-rate reserve.
pub const LT_RESERVE: PercentBand = PercentBand {
    low: 0.75,
    high: 0.88,
};
/// The long-run band, as a share of the maximum heart rate.
pub const LONG_MAX_HR: PercentBand = PercentBand {
    low: 0.74,
    high: 0.84,
};
/// The long-run band, as a share of the heart-rate reserve.
pub const LONG_RESERVE: PercentBand = PercentBand {
    low: 0.65,
    high: 0.78,
};

/// The recovery offset: +120 s/mi, which is +75 s/km.
pub const RECOVERY_OFFSET_S_PER_KM: f64 = 75.0;
/// The long run's early factors: 20–33 % slower than the 10K pace.
pub const LONG_START_FACTORS: PaceFactors = PaceFactors {
    fast: 1.20,
    slow: 1.33,
};
/// The long run's latter factors: 17–29 % slower than the 15K–HM pace.
pub const LONG_FINISH_FACTORS: PaceFactors = PaceFactors {
    fast: 1.17,
    slow: 1.29,
};
/// The shorter-distance threshold offsets: 10–15 s/mi slower than the 10K
/// pace.
pub const LT_SHORT_OFFSET_S_PER_MI: PaceFactors = PaceFactors {
    fast: 10.0,
    slow: 15.0,
};
/// The shortest-distance threshold offsets: 20–30 s/mi slower than the 5K
/// pace.
pub const LT_SHORTER_OFFSET_S_PER_MI: PaceFactors = PaceFactors {
    fast: 20.0,
    slow: 30.0,
};
/// A mile, in metres — the offsets are stated per mile.
pub const METRES_PER_MILE: f64 = 1_609.344;

/// What a rung's numbers hang on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    /// The athlete's own race at the goal distance.
    GoalRace,
    /// The hour-pace band, on the goal distance's span.
    ThresholdBand,
    /// The 15K-to-half-marathon race pace.
    RacePace,
    /// The 10K race pace.
    TenKPace,
    /// No anchor: the rule fixes a band and no pace.
    None,
}

/// What a rung is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RungKind {
    /// The general-aerobic day: a heart-rate band and no pace.
    GeneralAerobic,
    /// The recovery run.
    Recovery,
    /// The lactate-threshold run: the hour-pace band.
    LactateThreshold,
    /// The long run.
    LongRun,
    /// The goal race: the athlete's own race.
    GoalRace,
}

/// Which long run the ladder draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LongRunKind {
    /// The standard long run: it starts at the slow end of the early band
    /// and points at the faster end of the latter.
    #[default]
    Standard,
    /// The progression long run: the last miles at the threshold pace.
    Progression,
}

/// An input the answer is missing, and why.
///
/// A missing value is **never** a zero and never a zero band: it carries
/// the reason, and the reason names where the number would come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingInput {
    /// No race time has been entered.
    NoRaceTime,
    /// The zone model holds no maximum heart rate.
    NoMaxHeartRate,
    /// The athlete's resting heart rate is not known.
    NoRestingHeartRate,
    /// The goal distance is not set.
    NoGoalDistance,
}

impl MissingInput {
    /// The reason, in the core's own words.
    pub fn reason(&self) -> Option<String> {
        let id = match self {
            MissingInput::NoRaceTime => "pace.missing.race",
            MissingInput::NoMaxHeartRate => "pace.missing.max_hr",
            MissingInput::NoRestingHeartRate => "pace.missing.resting_hr",
            MissingInput::NoGoalDistance => "pace.missing.goal",
        };
        strings::text(id).map(|text| text.to_string())
    }
}

/// What the band is a share of.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BandBasis {
    /// A share of the athlete's maximum heart rate.
    MaxHeartRate {
        /// The maximum heart rate the band was computed on.
        max_hr: f64,
    },
    /// A share of the athlete's heart-rate reserve.
    Reserve {
        /// The maximum heart rate the reserve was computed on.
        max_hr: f64,
        /// The resting heart rate the reserve was computed on.
        resting_hr: f64,
    },
}

/// A heart-rate band, or a ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HeartRateBand {
    /// A band, from one edge to the other.
    Range {
        /// The lower edge, in beats per minute.
        from_bpm: f64,
        /// The upper edge, in beats per minute.
        to_bpm: f64,
        /// What the band is a share of.
        basis: BandBasis,
    },
    /// A ceiling: below this value.
    Below {
        /// The value the heart rate stays below, in beats per minute.
        below_bpm: f64,
        /// What the ceiling is a share of.
        basis: BandBasis,
    },
}

impl HeartRateBand {
    /// The rendered band: "128–148 bpm", or "below 139 bpm" for a ceiling.
    pub fn render(&self) -> Option<String> {
        match self {
            HeartRateBand::Range {
                from_bpm, to_bpm, ..
            } => units::render_heart_rate_band(*from_bpm, *to_bpm),
            HeartRateBand::Below { below_bpm, .. } => units::render_heart_rate_ceiling(*below_bpm),
        }
    }
}

/// A pace: the method's, or no pace with a reason.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaceValue {
    /// The band.
    Known(PaceBand),
    /// The rule states no pace for this rung.
    NoneStated,
    /// The pace has no value, and here is why.
    Missing(MissingInput),
}

impl PaceValue {
    /// The band, when there is one.
    pub fn band(&self) -> Option<PaceBand> {
        match self {
            PaceValue::Known(band) => Some(*band),
            _ => None,
        }
    }

    /// The reason the pace has no value, when it has none.
    pub fn missing(&self) -> Option<MissingInput> {
        match self {
            PaceValue::Missing(missing) => Some(*missing),
            _ => None,
        }
    }

    /// The rendered band, when there is one.
    pub fn render(&self) -> Option<String> {
        self.band().and_then(|band| band.render())
    }
}

/// A heart-rate band: the method's, or no band with a reason.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HeartRateValue {
    /// The band.
    Known(HeartRateBand),
    /// The method states no band for this rung.
    NoneStated,
    /// The band has no value, and here is why.
    Missing(MissingInput),
}

impl HeartRateValue {
    /// The band, when there is one.
    pub fn band(&self) -> Option<HeartRateBand> {
        match self {
            HeartRateValue::Known(band) => Some(*band),
            _ => None,
        }
    }

    /// The reason the band has no value, when it has none.
    pub fn missing(&self) -> Option<MissingInput> {
        match self {
            HeartRateValue::Missing(missing) => Some(*missing),
            _ => None,
        }
    }

    /// The rendered band, when there is one.
    pub fn render(&self) -> Option<String> {
        self.band().and_then(|band| band.render())
    }
}

/// One rung of the ladder.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Rung {
    /// What the rung is.
    pub kind: RungKind,
    /// The rule the rung's numbers come from.
    pub rule: Rule,
    /// What the rung's numbers hang on.
    pub anchor: Anchor,
    /// The pace the rung is run at, when the rule states one.
    pub pace: PaceValue,
    /// The pace the long run finishes at — the second end of the one rung.
    pub finish_pace: PaceValue,
    /// The heart-rate band, in the method's own form: a share of the maximum
    /// heart rate.
    pub heart_rate: HeartRateValue,
    /// The same band as a share of the heart-rate reserve — the method's
    /// second form, which needs the resting heart rate.
    pub heart_rate_reserve: HeartRateValue,
}

impl Rung {
    /// The rung's name, as the surface prints it.
    pub fn label(&self) -> &'static str {
        match self.kind {
            RungKind::GeneralAerobic => "General aerobic",
            RungKind::Recovery => "Recovery",
            RungKind::LactateThreshold => "Threshold (LT)",
            RungKind::LongRun => "Long run",
            RungKind::GoalRace => "The goal race",
        }
    }
}

/// The inputs the ladder is built from.
///
/// Every one of them may be missing, and a missing one makes its own
/// answers no values — never zeros, and never a rung that borrows another
/// rung's number.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LadderInput {
    /// The race the paces come from — the athlete's own.
    pub race: Option<Race>,
    /// The goal distance, in metres. It decides which branch of the
    /// threshold rule applies.
    pub goal_distance_m: Option<f64>,
    /// The athlete's maximum heart rate, from their own dated zone model.
    pub max_heart_rate: Option<f64>,
    /// The athlete's resting heart rate.
    pub resting_heart_rate: Option<f64>,
    /// Which long run the ladder draws.
    pub long_run: LongRunKind,
}

/// One equivalent race time, as the surface prints it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EquivalentLine {
    /// The distance's name: "10K", "half marathon".
    pub distance: &'static str,
    /// The time, as a clock: "41:36", "1:31:47".
    pub time: String,
}

/// The ladder: the chain, the equivalents and the rungs.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Ladder {
    /// The race the paces come from.
    pub race: Option<Race>,
    /// The goal distance.
    pub goal_distance_m: Option<f64>,
    /// The fitness number the chain produced.
    pub vdot: Option<f64>,
    /// The equivalent race times, by both models.
    pub equivalents: Vec<vo2::Equivalent>,
    /// The rungs, fastest first.
    pub rungs: Vec<Rung>,
}

impl Ladder {
    /// The chain's line: the fitness number and the relations that produced
    /// it.
    pub fn chain_line(&self) -> Option<String> {
        let vdot = self.vdot?;
        let vdot = units::format_decimal(vdot, 1);
        strings::render("pace.chain", &[("vdot", &vdot)])
    }

    /// The source line: the race the paces came from, with its day.
    ///
    /// The day is **rendered by the caller** — the api writes it in its own
    /// form — so the core never formats a date for a screen.
    pub fn source_line(&self, race_date: &str) -> Option<String> {
        let race = self.race.as_ref()?;
        let time = units::format_race_time(race.time_s);
        let distance = race_label(race.distance_m);
        strings::render(
            "pace.source.race",
            &[("distance", distance), ("time", &time), ("date", race_date)],
        )
    }

    /// The equivalent race times, one line each.
    pub fn equivalent_lines(&self) -> Vec<EquivalentLine> {
        self.equivalents
            .iter()
            .filter_map(|equivalent| {
                let time = equivalent.riegel_time_s?;
                Some(EquivalentLine {
                    distance: race_label(equivalent.distance_m),
                    time: units::format_race_time(time),
                })
            })
            .collect()
    }

    /// The line that names the equivalent race times and the model they came
    /// from.
    pub fn equivalents_line(&self) -> Option<String> {
        let lines = self.equivalent_lines();
        if lines.is_empty() {
            return None;
        }
        let joined = lines
            .iter()
            .map(|line| format!("{} {}", line.distance, line.time))
            .collect::<Vec<String>>()
            .join(" · ");
        strings::render("pace.source.equivalents", &[("equivalents", &joined)])
    }

    /// The rung of that kind, when the ladder has one.
    pub fn rung(&self, kind: RungKind) -> Option<&Rung> {
        self.rungs.iter().find(|rung| rung.kind == kind)
    }
}

/// The name a distance is printed under: "10K", "half marathon".
pub fn race_label(distance_m: f64) -> &'static str {
    let matches = |target: f64| (distance_m - target).abs() < 1.0;
    if matches(vo2::FIVE_K) {
        "5K"
    } else if matches(vo2::TEN_K) {
        "10K"
    } else if matches(vo2::FIFTEEN_K) {
        "15K"
    } else if matches(vo2::TEN_MILE) {
        "10-mile"
    } else if matches(vo2::HALF_MARATHON) {
        "half marathon"
    } else if matches(vo2::MARATHON) {
        "marathon"
    } else {
        "the race"
    }
}

/// The ladder, built from the athlete's own inputs.
///
/// The chain is: the race time → the fitness number → the equivalents → the
/// rungs. A rung whose own input is missing has **no value with a reason**,
/// and the other rungs still answer.
pub fn ladder(input: &LadderInput) -> Ladder {
    let effort = input
        .race
        .as_ref()
        .and_then(|race| vo2::effort_from_race(race.distance_m, race.time_s));
    let vdot = effort.map(|effort| effort.vdot);

    // The distances the equivalents are computed at: the six the method
    // names, and the goal distance when it is not one of them.
    let mut distances = vec![
        vo2::FIVE_K,
        vo2::TEN_K,
        vo2::FIFTEEN_K,
        vo2::TEN_MILE,
        vo2::HALF_MARATHON,
        vo2::MARATHON,
    ];
    if let Some(goal) = input.goal_distance_m {
        if goal.is_finite()
            && goal > 0.0
            && !distances.iter().any(|known| (*known - goal).abs() < 1.0)
        {
            distances.push(goal);
        }
    }

    let equivalents = match (vdot, input.race.as_ref()) {
        (Some(vdot), Some(race)) => {
            vo2::equivalents(vdot, race.time_s, race.distance_m, &distances)
        }
        _ => Vec::new(),
    };

    let pace_at = |distance_m: f64| -> Option<f64> {
        equivalents
            .iter()
            .find(|equivalent| (equivalent.distance_m - distance_m).abs() < 1.0)
            .and_then(|equivalent| equivalent.riegel_pace_s_per_km())
    };

    let pace_missing = if vdot.is_some() {
        None
    } else {
        Some(MissingInput::NoRaceTime)
    };

    // --- the goal race: the athlete's own race at the goal distance.
    let goal_pace = input
        .goal_distance_m
        .and_then(pace_at)
        .map(PaceBand::single);
    let goal_rung = Rung {
        kind: RungKind::GoalRace,
        rule: RULE_GOAL_RACE,
        anchor: Anchor::GoalRace,
        pace: match goal_pace {
            Some(band) => PaceValue::Known(band),
            None => match pace_missing {
                Some(missing) => PaceValue::Missing(missing),
                None => PaceValue::Missing(MissingInput::NoGoalDistance),
            },
        },
        finish_pace: PaceValue::NoneStated,
        heart_rate: HeartRateValue::NoneStated,
        heart_rate_reserve: HeartRateValue::NoneStated,
    };

    // --- the threshold: the hour-pace band on the goal distance's span.
    let lt_band = threshold_band(input.goal_distance_m, pace_at);
    let lt_rung = Rung {
        kind: RungKind::LactateThreshold,
        rule: RULE_THRESHOLD,
        anchor: Anchor::ThresholdBand,
        pace: match lt_band {
            Some(band) => PaceValue::Known(band),
            None => match pace_missing {
                Some(missing) => PaceValue::Missing(missing),
                None => PaceValue::Missing(MissingInput::NoGoalDistance),
            },
        },
        finish_pace: PaceValue::NoneStated,
        heart_rate: band_of_max_hr(input.max_heart_rate, LT_MAX_HR),
        heart_rate_reserve: band_of_reserve(
            input.max_heart_rate,
            input.resting_heart_rate,
            LT_RESERVE,
        ),
    };

    // --- the recovery run: +75 s/km off the 15K–HM race pace.
    let recovery_pace = recovery_pace(input.goal_distance_m, pace_at);
    let recovery_rung = Rung {
        kind: RungKind::Recovery,
        rule: RULE_RECOVERY,
        anchor: Anchor::RacePace,
        pace: match recovery_pace {
            Some(band) => PaceValue::Known(band),
            None => match pace_missing {
                Some(missing) => PaceValue::Missing(missing),
                None => PaceValue::Missing(MissingInput::NoGoalDistance),
            },
        },
        finish_pace: PaceValue::NoneStated,
        heart_rate: ceiling_of_max_hr(input.max_heart_rate, RECOVERY_MAX_HR),
        heart_rate_reserve: ceiling_of_reserve(
            input.max_heart_rate,
            input.resting_heart_rate,
            RECOVERY_RESERVE,
        ),
    };

    // --- the long run: one rung with two ends.
    let long_start = long_run_start(pace_at);
    let long_finish = match input.long_run {
        LongRunKind::Standard => long_run_finish(pace_at),
        LongRunKind::Progression => lt_band,
    };
    let long_rung = Rung {
        kind: RungKind::LongRun,
        rule: match input.long_run {
            LongRunKind::Standard => RULE_LONG_RUN,
            LongRunKind::Progression => RULE_PROGRESSION,
        },
        anchor: Anchor::TenKPace,
        pace: match long_start {
            Some(band) => PaceValue::Known(band),
            None => match pace_missing {
                Some(missing) => PaceValue::Missing(missing),
                None => PaceValue::Missing(MissingInput::NoGoalDistance),
            },
        },
        finish_pace: match long_finish {
            Some(band) => PaceValue::Known(band),
            None => match pace_missing {
                Some(missing) => PaceValue::Missing(missing),
                None => PaceValue::Missing(MissingInput::NoGoalDistance),
            },
        },
        heart_rate: band_of_max_hr(input.max_heart_rate, LONG_MAX_HR),
        heart_rate_reserve: band_of_reserve(
            input.max_heart_rate,
            input.resting_heart_rate,
            LONG_RESERVE,
        ),
    };

    // --- the general-aerobic day: a band and no pace.
    let general_rung = Rung {
        kind: RungKind::GeneralAerobic,
        rule: RULE_GENERAL_AEROBIC,
        anchor: Anchor::None,
        pace: PaceValue::NoneStated,
        finish_pace: PaceValue::NoneStated,
        heart_rate: band_of_max_hr(input.max_heart_rate, GA_MAX_HR),
        heart_rate_reserve: band_of_reserve(
            input.max_heart_rate,
            input.resting_heart_rate,
            GA_RESERVE,
        ),
    };

    Ladder {
        race: input.race.clone(),
        goal_distance_m: input.goal_distance_m,
        vdot,
        equivalents,
        // Fastest first, as the ladder is drawn.
        rungs: vec![lt_rung, goal_rung, long_rung, general_rung, recovery_rung],
    }
}

/// The hour-pace band, on the goal distance's span.
///
/// The rule's true statement is "the pace you could race for about an hour"
/// (ch. 1, ch. 7). The book then **approximates** it, and the approximation
/// branches on the distance the athlete races:
///
/// * a half marathon or longer — the band runs from the 15K pace (its fast
///   edge) to the half-marathon pace (its slow edge), the span in which the
///   hour pace falls;
/// * a 10K — the 10K pace plus 10–15 s/mi, the book's **floor** for a
///   shorter-distance athlete;
/// * a 5K — the 5K pace plus 20–30 s/mi.
fn threshold_band(
    goal_distance_m: Option<f64>,
    pace_at: impl Fn(f64) -> Option<f64>,
) -> Option<PaceBand> {
    let goal = goal_distance_m?;
    if goal >= vo2::HALF_MARATHON {
        let fast = pace_at(vo2::FIFTEEN_K)?;
        let slow = pace_at(vo2::HALF_MARATHON)?;
        Some(PaceBand::between(fast, slow))
    } else if goal >= vo2::TEN_K {
        let ten_k = pace_at(vo2::TEN_K)?;
        let offsets = LT_SHORT_OFFSET_S_PER_MI;
        Some(PaceBand::between(
            ten_k + offsets.fast * METRES_PER_MILE / 1_000.0,
            ten_k + offsets.slow * METRES_PER_MILE / 1_000.0,
        ))
    } else {
        let five_k = pace_at(vo2::FIVE_K)?;
        let offsets = LT_SHORTER_OFFSET_S_PER_MI;
        Some(PaceBand::between(
            five_k + offsets.fast * METRES_PER_MILE / 1_000.0,
            five_k + offsets.slow * METRES_PER_MILE / 1_000.0,
        ))
    }
}

/// The recovery pace: +75 s/km off the 15K-to-half-marathon race pace.
///
/// The engine takes the **slower** of the two paces the athlete has: a
/// recovery run is the easiest run of the week, and the rule's intent is a
/// slow run (`exploration/pace-engine.md` §15, question 3).
fn recovery_pace(
    goal_distance_m: Option<f64>,
    pace_at: impl Fn(f64) -> Option<f64>,
) -> Option<PaceBand> {
    let fifteen_k = pace_at(vo2::FIFTEEN_K)?;
    let mut anchor = fifteen_k;
    if let Some(goal) = goal_distance_m {
        if goal >= vo2::HALF_MARATHON {
            if let Some(half) = pace_at(vo2::HALF_MARATHON) {
                anchor = anchor.max(half);
            }
        }
    }
    Some(PaceBand::single(anchor + RECOVERY_OFFSET_S_PER_KM))
}

/// The long run's early band: 20–33 % slower than the 10K race pace.
fn long_run_start(pace_at: impl Fn(f64) -> Option<f64>) -> Option<PaceBand> {
    let ten_k = pace_at(vo2::TEN_K)?;
    let factors = LONG_START_FACTORS;
    Some(PaceBand::between(
        ten_k * factors.fast,
        ten_k * factors.slow,
    ))
}

/// The long run's latter band: 17–29 % slower than the 15K-to-half-marathon
/// race pace.
///
/// The anchor is a **band** — the 15K pace and the half-marathon pace — and
/// the method's two factors are applied to its two edges: the fast edge is
/// the 15K pace slowed by the smaller factor, the slow edge is the
/// half-marathon pace slowed by the larger one. The long run therefore
/// starts at the slow end of the early band and points at the faster end of
/// this one, which is what the book asks for: "begin at a conversational
/// pace and gradually increase" (ch. 1, ch. 7).
fn long_run_finish(pace_at: impl Fn(f64) -> Option<f64>) -> Option<PaceBand> {
    let fifteen_k = pace_at(vo2::FIFTEEN_K)?;
    let half = pace_at(vo2::HALF_MARATHON)?;
    let factors = LONG_FINISH_FACTORS;
    Some(PaceBand::between(
        fifteen_k * factors.fast,
        half * factors.slow,
    ))
}

/// A band as a share of the athlete's maximum heart rate.
fn band_of_max_hr(max_hr: Option<f64>, band: PercentBand) -> HeartRateValue {
    match max_hr {
        Some(max_hr) if max_hr.is_finite() && max_hr > 0.0 => {
            HeartRateValue::Known(HeartRateBand::Range {
                from_bpm: max_hr * band.low,
                to_bpm: max_hr * band.high,
                basis: BandBasis::MaxHeartRate { max_hr },
            })
        }
        Some(_) => HeartRateValue::Missing(MissingInput::NoMaxHeartRate),
        None => HeartRateValue::Missing(MissingInput::NoMaxHeartRate),
    }
}

/// A band as a share of the athlete's heart-rate reserve.
fn band_of_reserve(
    max_hr: Option<f64>,
    resting_hr: Option<f64>,
    band: PercentBand,
) -> HeartRateValue {
    let max_hr = match max_hr {
        Some(max_hr) if max_hr.is_finite() && max_hr > 0.0 => max_hr,
        _ => return HeartRateValue::Missing(MissingInput::NoMaxHeartRate),
    };
    let resting_hr = match resting_hr {
        Some(resting_hr) if resting_hr.is_finite() && resting_hr > 0.0 && resting_hr < max_hr => {
            resting_hr
        }
        _ => return HeartRateValue::Missing(MissingInput::NoRestingHeartRate),
    };
    let reserve = max_hr - resting_hr;
    HeartRateValue::Known(HeartRateBand::Range {
        from_bpm: resting_hr + reserve * band.low,
        to_bpm: resting_hr + reserve * band.high,
        basis: BandBasis::Reserve { max_hr, resting_hr },
    })
}

/// A ceiling as a share of the athlete's maximum heart rate.
fn ceiling_of_max_hr(max_hr: Option<f64>, share: f64) -> HeartRateValue {
    match max_hr {
        Some(max_hr) if max_hr.is_finite() && max_hr > 0.0 => {
            HeartRateValue::Known(HeartRateBand::Below {
                below_bpm: max_hr * share,
                basis: BandBasis::MaxHeartRate { max_hr },
            })
        }
        _ => HeartRateValue::Missing(MissingInput::NoMaxHeartRate),
    }
}

/// A ceiling as a share of the athlete's heart-rate reserve.
fn ceiling_of_reserve(max_hr: Option<f64>, resting_hr: Option<f64>, share: f64) -> HeartRateValue {
    let max_hr = match max_hr {
        Some(max_hr) if max_hr.is_finite() && max_hr > 0.0 => max_hr,
        _ => return HeartRateValue::Missing(MissingInput::NoMaxHeartRate),
    };
    let resting_hr = match resting_hr {
        Some(resting_hr) if resting_hr.is_finite() && resting_hr > 0.0 && resting_hr < max_hr => {
            resting_hr
        }
        _ => return HeartRateValue::Missing(MissingInput::NoRestingHeartRate),
    };
    let reserve = max_hr - resting_hr;
    HeartRateValue::Known(HeartRateBand::Below {
        below_bpm: resting_hr + reserve * share,
        basis: BandBasis::Reserve { max_hr, resting_hr },
    })
}

/// What a session of the method is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodSessionKind {
    /// The lactate-threshold session: twenty to forty minutes at the
    /// hour-pace band (ch. 1, ch. 7).
    Threshold,
    /// The recovery run: three to five miles at the recovery pace
    /// (ch. 2, ch. 7).
    Recovery,
    /// The long run: from the start pace to the finish pace. Its length
    /// comes with the plan's own weeks.
    LongRun,
}

impl MethodSessionKind {
    /// The key the app reads the session by.
    pub fn key(&self) -> &'static str {
        match self {
            MethodSessionKind::Threshold => "threshold",
            MethodSessionKind::Recovery => "recovery",
            MethodSessionKind::LongRun => "long_run",
        }
    }

    /// The session's name, in the runner's words.
    pub fn label(&self) -> &'static str {
        match self {
            MethodSessionKind::Threshold => "Lactate threshold",
            MethodSessionKind::Recovery => "Recovery run",
            MethodSessionKind::LongRun => "Long run",
        }
    }
}

/// The threshold session's duration: twenty to forty minutes (ch. 1, ch. 7).
pub const LT_SESSION_MINUTES: (f64, f64) = (20.0, 40.0);

/// The recovery run's length: three to five miles (ch. 2, ch. 7).
pub const RECOVERY_RUN_MILES: (f64, f64) = (3.0, 5.0);

/// A mile, in metres — the recovery run's length is stated in miles.
const METRES_PER_MILE_SESSION: f64 = 1_609.344;

/// One of the method's own sessions: what the book prescribes, and for how
/// long.
///
/// The session is the **method's**, and so is its duration: the threshold
/// session runs for twenty to forty minutes (ch. 1, ch. 7), and the
/// recovery run is three to five miles at its own pace (ch. 2, ch. 7). The
/// long run's length comes with the plan's own weeks, and is **no value**
/// until they land — never a guess and never a zero.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MethodSession {
    /// What the session is.
    pub kind: MethodSessionKind,
    /// The session's name, in the runner's words.
    pub label: &'static str,
    /// The duration, in minutes: the first minute and the last.
    ///
    /// `None` when the method's own length for the session is not known.
    pub minutes: Option<(f64, f64)>,
    /// The pace the session is run at.
    pub pace: Option<PaceBand>,
    /// The rule the session comes from.
    pub rule: Rule,
}

/// The method's own sessions, over the ladder's rungs.
///
/// The sessions are the ones the method states a length for, and the long
/// run, whose length comes with the plan's weeks. **The general-aerobic
/// day is not here**: the method fixes no pace and no duration for it, and
/// there is nothing to place.
pub fn method_sessions(ladder: &Ladder) -> Vec<MethodSession> {
    let mut sessions: Vec<MethodSession> = Vec::new();

    // The threshold session: twenty to forty minutes at the hour-pace
    // band.
    if let Some(rung) = ladder.rung(RungKind::LactateThreshold) {
        sessions.push(MethodSession {
            kind: MethodSessionKind::Threshold,
            label: MethodSessionKind::Threshold.label(),
            minutes: Some(LT_SESSION_MINUTES),
            pace: rung.pace.band(),
            rule: RULE_THRESHOLD,
        });
    }

    // The recovery run: three to five miles at the recovery pace. The
    // duration is the distance at the pace: the book's own two facts.
    if let Some(rung) = ladder.rung(RungKind::Recovery) {
        let pace = rung.pace.band();
        let minutes = pace.map(|band| {
            let pace_s_per_km = band.fast_s_per_km.max(band.slow_s_per_km);
            let miles = METRES_PER_MILE_SESSION;
            (
                RECOVERY_RUN_MILES.0 * miles / 1_000.0 * pace_s_per_km / 60.0,
                RECOVERY_RUN_MILES.1 * miles / 1_000.0 * pace_s_per_km / 60.0,
            )
        });
        sessions.push(MethodSession {
            kind: MethodSessionKind::Recovery,
            label: MethodSessionKind::Recovery.label(),
            minutes,
            pace,
            rule: RULE_RECOVERY,
        });
    }

    // The long run: from the start pace to the finish pace. Its length
    // comes with the plan's own weeks, and is no value until they land.
    if let Some(rung) = ladder.rung(RungKind::LongRun) {
        let pace = match (rung.pace.band(), rung.finish_pace.band()) {
            (Some(start), Some(finish)) => Some(PaceBand::between(
                start.fast_s_per_km.min(finish.fast_s_per_km),
                start.slow_s_per_km.max(finish.slow_s_per_km),
            )),
            _ => None,
        };
        sessions.push(MethodSession {
            kind: MethodSessionKind::LongRun,
            label: MethodSessionKind::LongRun.label(),
            minutes: None,
            pace,
            rule: rung.rule,
        });
    }

    sessions
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    /// The canonical athlete (`exploration/athlete.js`): the 10K in 41:36 on
    /// 12 October 2026, a half marathon as the goal, max HR 183, resting HR
    /// 54.
    fn canonical_input() -> LadderInput {
        LadderInput {
            race: Some(Race::new(
                vo2::TEN_K,
                2_496.0,
                Date::from_ymd(2026, 10, 12).expect("a date"),
            )),
            goal_distance_m: Some(vo2::HALF_MARATHON),
            max_heart_rate: Some(183.0),
            resting_heart_rate: Some(54.0),
            long_run: LongRunKind::Standard,
        }
    }

    fn canonical() -> Ladder {
        ladder(&canonical_input())
    }

    #[test]
    fn the_canonical_athlete_s_own_race_gives_the_canonical_fitness_number() {
        let ladder = canonical();
        let vdot = ladder.vdot.expect("a value");
        assert!((vdot - 49.6163).abs() < 0.001, "VDOT was {vdot}");
    }

    #[test]
    fn the_equivalent_race_times_are_the_canonical_ones() {
        let ladder = canonical();
        let lines = ladder.equivalent_lines();
        let rendered: Vec<String> = lines
            .iter()
            .map(|line| format!("{} {}", line.distance, line.time))
            .collect();
        assert_eq!(
            rendered,
            vec![
                "5K 19:57".to_string(),
                "10K 41:36".to_string(),
                "15K 1:03:56".to_string(),
                "10-mile 1:08:53".to_string(),
                "half marathon 1:31:47".to_string(),
                "marathon 3:11:22".to_string(),
            ]
        );
    }

    #[test]
    fn the_threshold_rung_is_the_hour_pace_band_on_the_goal_span() {
        let ladder = canonical();
        let rung = ladder.rung(RungKind::LactateThreshold);
        let band = rung.and_then(|rung| rung.pace.band());
        let band = band.expect("a value");
        // 4:16 (the 15K pace) to 4:21 (the half-marathon pace).
        assert!((band.fast_s_per_km - 255.75).abs() < 0.2);
        assert!((band.slow_s_per_km - 261.03).abs() < 0.2);
        assert_eq!(
            rung.and_then(|rung| rung.pace.render()),
            Some("4:16–4:21 /km".to_string())
        );
    }

    #[test]
    fn the_two_anchors_are_named_and_never_mixed() {
        let ladder = canonical();
        let threshold = ladder.rung(RungKind::LactateThreshold);
        let goal = ladder.rung(RungKind::GoalRace);
        assert_eq!(
            threshold.map(|rung| rung.anchor),
            Some(Anchor::ThresholdBand)
        );
        assert_eq!(goal.map(|rung| rung.anchor), Some(Anchor::GoalRace));
        // And they are two different numbers: the band's slow edge is the
        // goal race's own pace.
        let band = threshold
            .and_then(|rung| rung.pace.band())
            .expect("a value");
        let goal_band = goal.and_then(|rung| rung.pace.band()).expect("a value");
        assert!((band.slow_s_per_km - goal_band.fast_s_per_km).abs() < 0.01);
    }

    #[test]
    fn the_goal_race_is_the_athletes_own_race_at_the_goal_distance() {
        let ladder = canonical();
        let band = ladder
            .rung(RungKind::GoalRace)
            .and_then(|rung| rung.pace.band())
            .expect("a value");
        assert!((band.fast_s_per_km - 261.03).abs() < 0.2);
        assert_eq!(band.render(), Some("4:21 /km".to_string()));
    }

    #[test]
    fn the_recovery_pace_is_seventy_five_seconds_off_the_half_marathon_pace() {
        let ladder = canonical();
        let band = ladder
            .rung(RungKind::Recovery)
            .and_then(|rung| rung.pace.band())
            .expect("a value");
        assert!((band.fast_s_per_km - (261.03 + 75.0)).abs() < 0.2);
        assert_eq!(band.render(), Some("5:36 /km".to_string()));
    }

    #[test]
    fn the_long_run_has_two_ends_and_the_second_is_faster_than_the_first() {
        let ladder = canonical();
        let rung = ladder.rung(RungKind::LongRun);
        let start = rung.and_then(|rung| rung.pace.band()).expect("a value");
        let finish = rung
            .and_then(|rung| rung.finish_pace.band())
            .expect("a value");
        // The early band: 10K pace × 1.20 to × 1.33 → 5:00–5:32.
        assert!((start.fast_s_per_km - 249.6 * 1.20).abs() < 0.2);
        assert!((start.slow_s_per_km - 249.6 * 1.33).abs() < 0.2);
        assert_eq!(start.render(), Some("5:00–5:32 /km".to_string()));
        // The latter band: the 15K–HM race pace × 1.17–1.29 → 4:59–5:37.
        assert!((finish.fast_s_per_km - 255.75 * 1.17).abs() < 0.2);
        assert!((finish.slow_s_per_km - 261.03 * 1.29).abs() < 0.2);
        // And the run starts at the slow end of the early band and points at
        // the faster end of the latter.
        assert!(start.slow_s_per_km > finish.fast_s_per_km);
    }

    #[test]
    fn the_progression_long_run_finishes_at_the_threshold_pace() {
        let mut input = canonical_input();
        input.long_run = LongRunKind::Progression;
        let ladder = ladder(&input);
        let rung = ladder.rung(RungKind::LongRun);
        let finish = rung
            .and_then(|rung| rung.finish_pace.band())
            .expect("a value");
        let threshold = ladder
            .rung(RungKind::LactateThreshold)
            .and_then(|rung| rung.pace.band())
            .expect("a value");
        assert!((finish.fast_s_per_km - threshold.fast_s_per_km).abs() < 0.01);
        assert!((finish.slow_s_per_km - threshold.slow_s_per_km).abs() < 0.01);
        assert_eq!(
            rung.map(|rung| rung.rule.id),
            Some("book.progression_long_run")
        );
    }

    #[test]
    fn the_general_aerobic_day_has_a_band_and_no_pace() {
        let ladder = canonical();
        let rung = ladder.rung(RungKind::GeneralAerobic);
        assert_eq!(rung.map(|rung| rung.pace), Some(PaceValue::NoneStated));
        // 70–81 % of 183 → 128–148 bpm.
        let band = rung
            .and_then(|rung| rung.heart_rate.band())
            .expect("a value");
        assert_eq!(band.render(), Some("128–148 bpm".to_string()));
        // And the reserve form, on the athlete's own resting heart rate.
        let reserve = rung
            .and_then(|rung| rung.heart_rate_reserve.band())
            .expect("a value");
        assert_eq!(reserve.render(), Some("134–151 bpm".to_string()));
    }

    #[test]
    fn the_threshold_band_is_eighty_to_ninety_one_percent_of_the_maximum() {
        let ladder = canonical();
        let rung = ladder.rung(RungKind::LactateThreshold);
        let band = rung
            .and_then(|rung| rung.heart_rate.band())
            .expect("a value");
        assert_eq!(band.render(), Some("146–167 bpm".to_string()));
    }

    #[test]
    fn the_long_run_band_is_seventy_four_to_eighty_four_percent() {
        let ladder = canonical();
        let rung = ladder.rung(RungKind::LongRun);
        let band = rung
            .and_then(|rung| rung.heart_rate.band())
            .expect("a value");
        assert_eq!(band.render(), Some("135–154 bpm".to_string()));
    }

    #[test]
    fn the_recovery_run_is_a_ceiling_and_not_a_band() {
        let ladder = canonical();
        let rung = ladder.rung(RungKind::Recovery);
        let band = rung
            .and_then(|rung| rung.heart_rate.band())
            .expect("a value");
        // Below 76 % of 183 → below 139 bpm.
        assert_eq!(band.render(), Some("below 139 bpm".to_string()));
    }

    #[test]
    fn without_a_maximum_heart_rate_the_bands_have_no_value_and_a_reason() {
        let mut input = canonical_input();
        input.max_heart_rate = None;
        let ladder = ladder(&input);
        let rung = ladder.rung(RungKind::GeneralAerobic);
        assert_eq!(
            rung.and_then(|rung| rung.heart_rate.missing()),
            Some(MissingInput::NoMaxHeartRate)
        );
        assert!(MissingInput::NoMaxHeartRate
            .reason()
            .unwrap_or_default()
            .contains("maximum heart rate"));
        // And the paces still answer: they come from the race.
        assert!(ladder
            .rung(RungKind::GoalRace)
            .and_then(|rung| rung.pace.band())
            .is_some());
    }

    #[test]
    fn without_a_resting_heart_rate_the_reserve_bands_have_no_value_and_a_reason() {
        let mut input = canonical_input();
        input.resting_heart_rate = None;
        let ladder = ladder(&input);
        let rung = ladder.rung(RungKind::GeneralAerobic);
        assert_eq!(
            rung.and_then(|rung| rung.heart_rate_reserve.missing()),
            Some(MissingInput::NoRestingHeartRate)
        );
        // And the maximum-heart-rate band still stands.
        assert!(rung.and_then(|rung| rung.heart_rate.band()).is_some());
    }

    #[test]
    fn without_a_race_the_paces_have_no_value_and_the_bands_still_stand() {
        let input = LadderInput {
            race: None,
            goal_distance_m: Some(vo2::HALF_MARATHON),
            max_heart_rate: Some(183.0),
            resting_heart_rate: Some(54.0),
            long_run: LongRunKind::Standard,
        };
        let ladder = ladder(&input);
        assert!(ladder.vdot.is_none());
        assert_eq!(
            ladder
                .rung(RungKind::GoalRace)
                .and_then(|rung| rung.pace.missing()),
            Some(MissingInput::NoRaceTime)
        );
        // The general-aerobic band does not need a race.
        assert!(ladder
            .rung(RungKind::GeneralAerobic)
            .and_then(|rung| rung.heart_rate.band())
            .is_some());
    }

    #[test]
    fn every_rule_carries_its_citation_and_its_wording() {
        let ladder = canonical();
        for rung in &ladder.rungs {
            assert!(!rung.rule.citation.is_empty());
            assert!(rung.rule.wording().is_some());
            assert!(rung.rule.line().is_some());
        }
    }

    #[test]
    fn a_shorter_distance_athlete_gets_the_books_offset_branch() {
        let mut input = canonical_input();
        input.goal_distance_m = Some(vo2::TEN_K);
        let ladder = ladder(&input);
        let band = ladder
            .rung(RungKind::LactateThreshold)
            .and_then(|rung| rung.pace.band())
            .expect("a value");
        // The 10K pace plus 10–15 s/mi, which is +16 to +24 s/km.
        assert!((band.fast_s_per_km - (249.6 + 16.09344)).abs() < 0.05);
        assert!((band.slow_s_per_km - (249.6 + 24.14016)).abs() < 0.05);
    }
}
