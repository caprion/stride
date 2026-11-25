//! The athlete's own heart rates: the declaration, the bands it yields,
//! and the two facts they are compared with.
//!
//! The athlete declares their **maximum heart rate**, their **resting
//! heart rate**, or **both**, with where each number came from and when it
//! was measured. That is one small fact, entered once, and the app takes
//! care of it afterwards (`exploration/heart-rate-setup.md`).
//!
//! ## The rule, and where it comes from
//!
//! Garmin publishes the five-zone ladder — **50 %, 60 %, 70 %, 80 %,
//! 90 %** — and publishes the two ranges it can be expressed on: the
//! maximum heart rate, and the **heart-rate reserve** (the maximum minus
//! the resting rate). The app applies **the same published percentages** to
//! **the range the declaration supports**, so there is **one rule with two
//! ranges** and not two rules.
//!
//! ```text
//! both numbers:  floor(n) = resting + share(n) × (maximum − resting)
//! a maximum:    floor(n) = share(n) × maximum
//! ```
//!
//! Each floor is rounded to the nearest beat per minute, halves up, so two
//! runs of the app on the same number give the same bands.
//!
//! ## Absence is not zero
//!
//! * **no maximum** → **no bands at all**, and the reason says why;
//! * **no resting rate** → the bands are a **share of the maximum**, and
//!   the answer says which range it used;
//! * the bands are **never stored**: they are recomputed on read from the
//!   declaration in force.
//!
//! ## Two facts, never blended
//!
//! The **watch's own zones** (the floors configured in the watch) and the
//! **bands from the heart rates the athlete entered** are two dated facts.
//! The comparison returns the two **separately**, with the difference
//! between them, and every answer says **which one it used**. Nothing here
//! ever mixes them, and nothing here says which one is right.
//!
//! ## What this module never does
//!
//! * It never computes a heart rate from an age, and it never uses
//!   Garmin's own age estimate: the watch's maximum is a fact about the
//!   watch's configuration, not about the athlete's physiology.
//! * It never blends the two bases into one set of bands.
//! * It never ranks them, and it never says one is stale.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::strings;
use crate::training_paces::Rule;
use crate::units;
use crate::zones::ZoneFloors;

/// The five shares of Garmin's published five-zone ladder.
///
/// The ladder is Garmin's own: 50–60 %, 60–70 %, 70–80 %, 80–90 % and
/// 90–100 % of the range the bands are expressed on.
pub const LADDER: [f64; 5] = [0.50, 0.60, 0.70, 0.80, 0.90];

/// The name the declared bands carry where a model names its method.
///
/// The watch's own models name Garmin's training method (`HR_MAX`,
/// `HR_RESERVE`); the bands the athlete declared name **the basis they
/// came from**, and the source line says which.
pub const DECLARED_METHOD: &str = "declared";

/// The rule the bands are derived by, and where it comes from.
///
/// The citation is Garmin's own published ladder: the *Forerunner 245
/// Owner's Manual*, "About Heart Rate Zones" → "Heart Rate Zone
/// Calculations", and the blog page that prints the same five bands.
pub const RULE: Rule = Rule {
    id: "garmin.five_zone_ladder",
    citation: "Garmin's published five-zone ladder",
    string_id: "heart_rates.rule",
};

/// The kind of heart rate a declaration carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartRateKind {
    /// The athlete's maximum heart rate.
    Max,
    /// The athlete's resting heart rate.
    Resting,
}

impl HeartRateKind {
    /// The kind's name, as it is stored.
    pub fn key(&self) -> &'static str {
        match self {
            HeartRateKind::Max => "max",
            HeartRateKind::Resting => "resting",
        }
    }

    /// The kind's label, in the runner's words.
    pub fn label(&self) -> &'static str {
        match self {
            HeartRateKind::Max => "your maximum heart rate",
            HeartRateKind::Resting => "your resting heart rate",
        }
    }
}

/// Where a declared heart rate came from.
///
/// **A closed list of five**, so the app has nothing to interpret: the
/// athlete picks a label and the app stores it and shows it. No source is
/// ranked, and no source is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartRateSource {
    /// The athlete's own watch.
    Watch,
    /// A test.
    Test,
    /// A doctor or a lab.
    DoctorLab,
    /// Somebody else's watch — the training partner's.
    PartnerWatch,
    /// A number the athlete brought.
    Typed,
}

impl HeartRateSource {
    /// Every source, in the order the athlete meets them.
    pub const ALL: [HeartRateSource; 5] = [
        HeartRateSource::Watch,
        HeartRateSource::Test,
        HeartRateSource::DoctorLab,
        HeartRateSource::PartnerWatch,
        HeartRateSource::Typed,
    ];

    /// The source's name, as it is stored.
    pub fn key(&self) -> &'static str {
        match self {
            HeartRateSource::Watch => "watch",
            HeartRateSource::Test => "test",
            HeartRateSource::DoctorLab => "doctor_lab",
            HeartRateSource::PartnerWatch => "partner_watch",
            HeartRateSource::Typed => "typed",
        }
    }

    /// The source's label, in the runner's words.
    pub fn label(&self) -> &'static str {
        match self {
            HeartRateSource::Watch => "from my watch",
            HeartRateSource::Test => "from a test",
            HeartRateSource::DoctorLab => "from a doctor or a lab",
            HeartRateSource::PartnerWatch => "from my training partner's watch",
            HeartRateSource::Typed => "typed",
        }
    }

    /// The source the store holds, as the core reads it.
    ///
    /// A source the app does not know is `None`: the label is a closed
    /// list, and a row that carries something else is a row the app cannot
    /// show.
    pub fn of(key: &str) -> Option<HeartRateSource> {
        match key {
            "watch" => Some(HeartRateSource::Watch),
            "test" => Some(HeartRateSource::Test),
            "doctor_lab" => Some(HeartRateSource::DoctorLab),
            "partner_watch" => Some(HeartRateSource::PartnerWatch),
            "typed" => Some(HeartRateSource::Typed),
            _ => None,
        }
    }
}

/// One declared heart rate: the value, where it came from, and the day it
/// was measured.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeclaredRate {
    /// The heart rate, in beats per minute.
    pub value_bpm: f64,
    /// Where it came from.
    pub source: HeartRateSource,
    /// The day it was measured. `None` when the athlete did not say.
    pub measured_on: Option<Date>,
}

/// One declaration: the athlete's own statement about their heart rates.
///
/// **A pair entered together is one declaration.** A change writes a new
/// declaration and never edits the earlier one, so a declaration is always
/// one coherent statement — *"my maximum is 186 and my resting rate is
/// 52"* — and the earlier ones stay in the dated history with their own
/// dates and their own sources.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    /// The declaration's own id: the two rows of one declaration share it.
    pub id: i64,
    /// The maximum heart rate, when the declaration carries one.
    pub max: Option<DeclaredRate>,
    /// The resting heart rate, when the declaration carries one.
    pub resting: Option<DeclaredRate>,
    /// The day the declaration was made.
    pub entered_on: Date,
    /// The day the declaration was last changed.
    pub updated_on: Date,
}

impl Declaration {
    /// The declaration's value, when it carries one.
    ///
    /// A declaration with neither number is not a declaration the app can
    /// show: it holds no fact at all.
    pub fn carries_a_value(&self) -> bool {
        self.max.is_some() || self.resting.is_some()
    }

    /// The rate of one kind, when the declaration carries it.
    pub fn rate(&self, kind: HeartRateKind) -> Option<DeclaredRate> {
        match kind {
            HeartRateKind::Max => self.max,
            HeartRateKind::Resting => self.resting,
        }
    }
}

/// The declaration in force: **the most recent one that carries a value**.
///
/// One declaration is in force at a time. The earlier declarations stay in
/// the dated history, and the answers computed under them stay as they
/// were, dated.
pub fn in_force(declarations: &[Declaration]) -> Option<&Declaration> {
    declarations
        .iter()
        .filter(|declaration| declaration.carries_a_value())
        .max_by_key(|declaration| (declaration.entered_on, declaration.id))
}

/// The declarations in their dated history, newest first.
pub fn history(declarations: &[Declaration]) -> Vec<&Declaration> {
    let mut dated: Vec<&Declaration> = declarations.iter().collect();
    dated.sort_by_key(|declaration| std::cmp::Reverse((declaration.entered_on, declaration.id)));
    dated
}

/// What a declaration supports.
///
/// The two ranges and the two ways a declaration falls short of them.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Supported {
    /// **Both numbers**: the ladder on the heart-rate reserve.
    Reserve {
        /// The maximum heart rate, in beats per minute.
        max_bpm: f64,
        /// The resting heart rate, in beats per minute.
        resting_bpm: f64,
    },
    /// **A maximum alone**: the ladder on the share of the maximum.
    MaxShare {
        /// The maximum heart rate, in beats per minute.
        max_bpm: f64,
    },
    /// **A resting rate alone**: there is no range without a maximum.
    RestingOnly {
        /// The resting heart rate, in beats per minute.
        resting_bpm: f64,
    },
    /// **Nothing at all.**
    None,
}

impl Supported {
    /// The maximum heart rate, when the declaration carries one.
    pub fn max_bpm(&self) -> Option<f64> {
        match self {
            Supported::Reserve { max_bpm, .. } => Some(*max_bpm),
            Supported::MaxShare { max_bpm } => Some(*max_bpm),
            _ => None,
        }
    }

    /// The resting heart rate, when the declaration carries one.
    pub fn resting_bpm(&self) -> Option<f64> {
        match self {
            Supported::Reserve { resting_bpm, .. } => Some(*resting_bpm),
            Supported::RestingOnly { resting_bpm } => Some(*resting_bpm),
            _ => None,
        }
    }

    /// The range the bands are expressed on.
    pub fn range(&self) -> Option<BandRange> {
        match self {
            Supported::Reserve { .. } => Some(BandRange::Reserve),
            Supported::MaxShare { .. } => Some(BandRange::MaxShare),
            _ => None,
        }
    }
}

/// The range the ladder is applied to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BandRange {
    /// The **heart-rate reserve**: the maximum minus the resting rate.
    Reserve,
    /// The **share of the maximum** heart rate.
    MaxShare,
}

impl BandRange {
    /// The range's name, as it is carried in an answer.
    pub fn key(&self) -> &'static str {
        match self {
            BandRange::Reserve => "reserve",
            BandRange::MaxShare => "max_share",
        }
    }

    /// The range, in the runner's words.
    pub fn label(&self) -> &'static str {
        match self {
            BandRange::Reserve => "from your maximum heart rate and your resting heart rate",
            BandRange::MaxShare => "from your maximum heart rate",
        }
    }
}

/// Why a declaration falls short of the bands.
///
/// The reasons are **facts about what is missing**, and never a verdict on
/// the declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingReason {
    /// There is **no maximum heart rate**: there is no range without one,
    /// so there are no bands at all.
    NoMaximum,
    /// There is **no resting heart rate**: the bands can only be a share
    /// of the maximum.
    NoRestingRate,
}

impl MissingReason {
    /// The reason, in the core's own words.
    pub fn reason(&self) -> Option<String> {
        let id = match self {
            MissingReason::NoMaximum => "heart_rates.missing.max",
            MissingReason::NoRestingRate => "heart_rates.missing.resting",
        };
        strings::text(id).map(|text| text.to_string())
    }
}

/// What a declaration supports, and what it does not.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Derived {
    /// What the declaration supports.
    pub supported: Supported,
    /// The bands, when the declaration yields any.
    pub bands: Option<Bands>,
    /// Why there are no bands.
    pub missing: Option<MissingReason>,
    /// Why the bands are not a share of the heart-rate reserve.
    pub missing_reserve: Option<MissingReason>,
}

/// The bands a declaration yields.
///
/// The five floors, the maximum they were built on, and the range the
/// ladder was applied to.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bands {
    /// The five floors, in beats per minute.
    pub floors: ZoneFloors,
    /// The maximum heart rate the bands were built on.
    pub max_heart_rate: Option<f64>,
    /// The range the ladder was applied to.
    pub range: BandRange,
}

/// What a declaration supports, and the bands it yields.
///
/// **Absence is not zero.** A declaration with no maximum yields no bands
/// at all, and the reason says why. A declaration with a maximum but no
/// resting rate yields the ladder on the **share of the maximum**, and the
/// reserve — the other range — has no value, with its own reason.
pub fn derive(declaration: &Declaration) -> Derived {
    let supported = match (declaration.max, declaration.resting) {
        (Some(max), Some(resting)) => Supported::Reserve {
            max_bpm: max.value_bpm,
            resting_bpm: resting.value_bpm,
        },
        (Some(max), None) => Supported::MaxShare {
            max_bpm: max.value_bpm,
        },
        (None, Some(resting)) => Supported::RestingOnly {
            resting_bpm: resting.value_bpm,
        },
        (None, None) => Supported::None,
    };
    let bands = bands_of(&supported);
    let missing = match supported {
        Supported::RestingOnly { .. } | Supported::None => Some(MissingReason::NoMaximum),
        _ => None,
    };
    let missing_reserve = match supported {
        Supported::MaxShare { .. } => Some(MissingReason::NoRestingRate),
        _ => None,
    };
    Derived {
        supported,
        bands,
        missing,
        missing_reserve,
    }
}

/// The bands a supported declaration yields.
///
/// `None` when the declaration supports no range at all: there is no
/// maximum, and a range without a maximum is not a range.
fn bands_of(supported: &Supported) -> Option<Bands> {
    match supported {
        Supported::Reserve {
            max_bpm,
            resting_bpm,
        } => {
            let reserve = max_bpm - resting_bpm;
            if !reserve.is_finite() || reserve <= 0.0 {
                return None;
            }
            Some(Bands {
                floors: floors_of(&LADDER, |share| resting_bpm + share * reserve),
                max_heart_rate: Some(*max_bpm),
                range: BandRange::Reserve,
            })
        }
        Supported::MaxShare { max_bpm } => Some(Bands {
            floors: floors_of(&LADDER, |share| share * max_bpm),
            max_heart_rate: Some(*max_bpm),
            range: BandRange::MaxShare,
        }),
        Supported::RestingOnly { .. } | Supported::None => None,
    }
}

/// The five floors of the ladder, on a range.
fn floors_of(ladder: &[f64; 5], floor: impl Fn(f64) -> f64) -> ZoneFloors {
    ZoneFloors {
        zone1: round_half_up(floor(ladder[0])),
        zone2: round_half_up(floor(ladder[1])),
        zone3: round_half_up(floor(ladder[2])),
        zone4: round_half_up(floor(ladder[3])),
        zone5: round_half_up(floor(ladder[4])),
    }
}

/// A number rounded to the nearest whole, halves up.
///
/// The rounding is part of the rule, so two runs of the app on the same
/// number give the same bands.
fn round_half_up(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    (value + 0.5).floor()
}

/// Which basis an answer used.
///
/// **One basis per answer.** The two bases are never mixed, and every
/// answer names the one it used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Basis {
    /// The bands from the heart rates the athlete entered.
    Declared {
        /// The range the ladder was applied to.
        range: BandRange,
        /// The day the declaration was made.
        entered_on: Date,
    },
    /// The watch's own configured floors.
    Watch {
        /// The day the model was read from the watch.
        read_on: Date,
    },
    /// **No basis at all**: there are no bands to measure against.
    None,
}

impl Basis {
    /// The basis's name, as it is carried in an answer.
    pub fn key(&self) -> &'static str {
        match self {
            Basis::Declared { .. } => "declared",
            Basis::Watch { .. } => "watch",
            Basis::None => "none",
        }
    }

    /// The basis, in the runner's words.
    pub fn label(&self) -> &'static str {
        match self {
            Basis::Declared { .. } => "the bands from the heart rates you entered",
            Basis::Watch { .. } => "your watch's zones",
            Basis::None => "no bands",
        }
    }
}

/// The watch's own model, as the core takes it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchBands {
    /// The watch's five floors.
    pub floors: ZoneFloors,
    /// The maximum heart rate the watch's model was built on.
    pub max_heart_rate: Option<f64>,
    /// The day the model was read.
    pub read_on: Date,
}

/// The bands an answer is measured on, and the basis they came from.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectedBands {
    /// The five floors, in beats per minute.
    pub floors: ZoneFloors,
    /// The maximum heart rate the bands were built on.
    pub max_heart_rate: Option<f64>,
    /// The basis the bands came from.
    pub basis: Basis,
}

/// The bands an answer is measured on, and the basis they came from.
///
/// **The rule.** Where a declaration exists, the **athlete's declared
/// bands** are the bands — applied to **every** run, so the series is
/// comparable across the whole window. The **watch's own model** is used
/// where one was in force and **no declaration exists**.
///
/// The two are **never mixed**: the function returns one set of floors and
/// the basis it came from, and the caller names the basis in the answer.
pub fn select(
    declaration: Option<&Declaration>,
    watch: Option<WatchBands>,
) -> Option<SelectedBands> {
    if let Some(declaration) = declaration {
        let derived = derive(declaration);
        if let Some(bands) = derived.bands {
            return Some(SelectedBands {
                floors: bands.floors,
                max_heart_rate: bands.max_heart_rate,
                basis: Basis::Declared {
                    range: bands.range,
                    entered_on: declaration.entered_on,
                },
            });
        }
    }
    watch.map(|watch| SelectedBands {
        floors: watch.floors,
        max_heart_rate: watch.max_heart_rate,
        basis: Basis::Watch {
            read_on: watch.read_on,
        },
    })
}

/// One boundary's difference between the two sets of floors.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundaryDifference {
    /// The band: 1 to 5.
    pub zone: usize,
    /// The floor of the declared band, in beats per minute.
    pub declared_bpm: f64,
    /// The floor of the watch's band, in beats per minute.
    pub watch_bpm: f64,
    /// The difference: the declared floor less the watch's.
    pub difference_bpm: f64,
}

/// The two sets of floors, and the difference between them.
///
/// **The two are never blended.** The comparison holds them side by side,
/// boundary by boundary, and says which one an answer used. The
/// difference is a **fact**, and never a correction: the app does not say
/// the watch is mis-set, and it does not say the athlete's number
/// supersedes the watch's.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comparison {
    /// The bands from the athlete's declaration.
    pub declared: Option<Bands>,
    /// The watch's own floors.
    pub watch: Option<ZoneFloors>,
    /// The boundary-by-boundary difference, when both sets exist.
    pub difference: Option<Vec<BoundaryDifference>>,
    /// The basis an answer used.
    pub basis: Basis,
}

/// Compare the athlete's declared bands with the watch's own floors.
///
/// The two sets are returned **separately**, with the difference between
/// them, and the basis an answer used. Nothing is blended and nothing is
/// ranked.
pub fn compare(
    declaration: Option<&Declaration>,
    watch: Option<WatchBands>,
) -> Comparison {
    let declared = declaration.and_then(|declaration| derive(declaration).bands);
    let watch_floors = watch.map(|watch| watch.floors);
    // The basis: the declared bands where there are any, and the watch's
    // own model otherwise. The two are never mixed.
    let basis = match (declaration, declared) {
        (Some(declaration), Some(bands)) => Basis::Declared {
            range: bands.range,
            entered_on: declaration.entered_on,
        },
        // A declaration that yields no bands is no basis at all: the
        // watch's own model is the other fact, and it is never blended
        // with a declaration that holds nothing.
        (Some(_), None) => match watch {
            Some(watch) => Basis::Watch { read_on: watch.read_on },
            None => Basis::None,
        },
        (None, _) => match watch {
            Some(watch) => Basis::Watch { read_on: watch.read_on },
            None => Basis::None,
        },
    };
    let difference = match (declared, watch_floors) {
        (Some(declared), Some(watch)) => Some(differences(&declared.floors, &watch)),
        _ => None,
    };
    Comparison {
        declared,
        watch: watch_floors,
        difference,
        basis,
    }
}

/// The difference between two sets of floors, boundary by boundary.
fn differences(declared: &ZoneFloors, watch: &ZoneFloors) -> Vec<BoundaryDifference> {
    let declared_floors = [declared.zone1, declared.zone2, declared.zone3, declared.zone4, declared.zone5];
    let watch_floors = [watch.zone1, watch.zone2, watch.zone3, watch.zone4, watch.zone5];
    declared_floors
        .iter()
        .zip(watch_floors.iter())
        .enumerate()
        .map(|(index, (declared, watch))| BoundaryDifference {
            zone: index + 1,
            declared_bpm: *declared,
            watch_bpm: *watch,
            difference_bpm: declared - watch,
        })
        .collect()
}

/// The source line of the declared bands.
///
/// The line names **what** the bands are and **when** they were dated. The
/// day is **rendered by the caller**: the core does not format a date for
/// a screen, and a third form would be a form nobody owns.
///
/// The pair's line names the two numbers the bands were built on; the
/// maximum's line names the one it has.
pub fn source_line(
    bands: &Bands,
    resting_bpm: Option<f64>,
    entered_on: &str,
) -> Option<String> {
    let max_hr = units::render_heart_rate(bands.max_heart_rate)?;
    match (bands.range, resting_bpm) {
        (BandRange::Reserve, Some(resting)) => {
            let resting = units::render_heart_rate(Some(resting))?;
            strings::render(
                "heart_rates.source_line",
                &[("max_hr", &max_hr), ("resting_hr", &resting), ("date", entered_on)],
            )
        }
        _ => strings::render(
            "heart_rates.source_line.max",
            &[("max_hr", &max_hr), ("date", entered_on)],
        ),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    /// A day, as the tests write it.
    fn day(year: i32, month: u32, day: u32) -> Date {
        Date::from_ymd(year, month, day).expect("a date")
    }

    /// The canonical athlete's pair: a maximum of 186 and a resting rate
    /// of 52.
    fn pair() -> Declaration {
        Declaration {
            id: 1,
            max: Some(DeclaredRate {
                value_bpm: 186.0,
                source: HeartRateSource::Test,
                measured_on: Some(day(2026, 11, 8)),
            }),
            resting: Some(DeclaredRate {
                value_bpm: 52.0,
                source: HeartRateSource::DoctorLab,
                measured_on: Some(day(2026, 11, 12)),
            }),
            entered_on: day(2026, 11, 24),
            updated_on: day(2026, 11, 24),
        }
    }

    /// The same athlete, with a maximum and no resting rate.
    fn maximum_only() -> Declaration {
        Declaration {
            id: 2,
            max: Some(DeclaredRate {
                value_bpm: 186.0,
                source: HeartRateSource::Watch,
                measured_on: None,
            }),
            resting: None,
            entered_on: day(2026, 11, 3),
            updated_on: day(2026, 11, 3),
        }
    }

    /// The watch's own floors, read on 3 November 2026.
    fn watch() -> WatchBands {
        WatchBands {
            floors: ZoneFloors {
                zone1: 115.0,
                zone2: 128.0,
                zone3: 143.0,
                zone4: 159.0,
                zone5: 165.0,
            },
            max_heart_rate: Some(183.0),
            read_on: day(2026, 11, 3),
        }
    }

    #[test]
    fn the_pair_yields_the_ladder_on_the_reserve() {
        let derived = derive(&pair());
        let bands = derived.bands.expect("the bands");
        // The reserve is 186 − 52 = 134, and the ladder is Garmin's.
        assert_eq!(bands.range, BandRange::Reserve);
        assert_eq!(bands.floors.zone1, 119.0);
        assert_eq!(bands.floors.zone2, 132.0);
        assert_eq!(bands.floors.zone3, 146.0);
        assert_eq!(bands.floors.zone4, 159.0);
        assert_eq!(bands.floors.zone5, 173.0);
        assert_eq!(bands.max_heart_rate, Some(186.0));
    }

    #[test]
    fn a_maximum_alone_yields_the_ladder_on_the_maximum() {
        let derived = derive(&maximum_only());
        let bands = derived.bands.expect("the bands");
        assert_eq!(bands.range, BandRange::MaxShare);
        assert_eq!(bands.floors.zone1, 93.0);
        assert_eq!(bands.floors.zone2, 112.0);
        assert_eq!(bands.floors.zone3, 130.0);
        assert_eq!(bands.floors.zone4, 149.0);
        assert_eq!(bands.floors.zone5, 167.0);
        // And the reserve has no value, with its own reason.
        assert_eq!(derived.missing, None);
        assert_eq!(derived.missing_reserve, Some(MissingReason::NoRestingRate));
        assert!(MissingReason::NoRestingRate
            .reason()
            .unwrap_or_default()
            .contains("resting"));
    }

    #[test]
    fn a_resting_rate_alone_yields_no_bands_at_all() {
        let declaration = Declaration {
            id: 3,
            max: None,
            resting: Some(DeclaredRate {
                value_bpm: 52.0,
                source: HeartRateSource::DoctorLab,
                measured_on: None,
            }),
            entered_on: day(2026, 11, 24),
            updated_on: day(2026, 11, 24),
        };
        let derived = derive(&declaration);
        assert!(derived.bands.is_none());
        assert_eq!(derived.missing, Some(MissingReason::NoMaximum));
        // And the reason is the core's own sentence.
        let reason = MissingReason::NoMaximum.reason().unwrap_or_default();
        assert!(reason.contains("no maximum heart rate"));
    }

    #[test]
    fn the_most_recent_declaration_is_the_one_in_force() {
        let march = Declaration {
            id: 1,
            max: Some(DeclaredRate {
                value_bpm: 183.0,
                source: HeartRateSource::Watch,
                measured_on: None,
            }),
            resting: None,
            entered_on: day(2026, 3, 3),
            updated_on: day(2026, 3, 3),
        };
        let declarations = vec![march.clone(), pair()];
        let in_use = in_force(&declarations).expect("a declaration in force");
        assert_eq!(in_use.id, 1);
        // And the history is newest first, with the earlier one still in it.
        let history = history(&declarations);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].entered_on, pair().entered_on);
        assert_eq!(history[1].entered_on, march.entered_on);
    }

    #[test]
    fn a_declaration_with_no_value_is_not_in_force() {
        let empty = Declaration {
            id: 4,
            max: None,
            resting: None,
            entered_on: day(2026, 11, 30),
            updated_on: day(2026, 11, 30),
        };
        assert!(in_force(&[empty]).is_none());
    }

    #[test]
    fn the_declared_bands_are_used_where_a_declaration_exists() {
        let selected = select(Some(&pair()), Some(watch())).expect("bands");
        assert_eq!(selected.floors.zone1, 119.0);
        assert_eq!(
            selected.basis,
            Basis::Declared {
                range: BandRange::Reserve,
                entered_on: day(2026, 11, 24),
            }
        );
    }

    #[test]
    fn the_watch_s_model_is_used_where_no_declaration_exists() {
        let selected = select(None, Some(watch())).expect("bands");
        assert_eq!(selected.floors.zone1, 115.0);
        assert_eq!(
            selected.basis,
            Basis::Watch {
                read_on: day(2026, 11, 3),
            }
        );
    }

    #[test]
    fn a_declaration_without_a_maximum_leaves_the_watch_s_model_in_place() {
        let resting_only = Declaration {
            id: 5,
            max: None,
            resting: Some(DeclaredRate {
                value_bpm: 52.0,
                source: HeartRateSource::DoctorLab,
                measured_on: None,
            }),
            entered_on: day(2026, 11, 24),
            updated_on: day(2026, 11, 24),
        };
        let selected = select(Some(&resting_only), Some(watch())).expect("bands");
        assert_eq!(selected.floors.zone1, 115.0);
        assert!(matches!(selected.basis, Basis::Watch { .. }));
    }

    #[test]
    fn the_two_bases_are_compared_and_never_blended() {
        let comparison = compare(Some(&pair()), Some(watch()));
        // The two are there, separately.
        let declared = comparison.declared.expect("the declared bands");
        let watch_floors = comparison.watch.expect("the watch's floors");
        assert_eq!(declared.floors.zone1, 119.0);
        assert_eq!(watch_floors.zone1, 115.0);
        // And the difference, boundary by boundary.
        let difference = comparison.difference.expect("the difference");
        assert_eq!(difference.len(), 5);
        assert_eq!(difference[0].difference_bpm, 4.0);
        assert_eq!(difference[1].difference_bpm, 4.0);
        assert_eq!(difference[2].difference_bpm, 3.0);
        assert_eq!(difference[3].difference_bpm, 0.0);
        assert_eq!(difference[4].difference_bpm, 8.0);
        // And the basis an answer used.
        assert!(matches!(comparison.basis, Basis::Declared { .. }));
    }

    #[test]
    fn the_source_line_names_the_basis_and_the_day() {
        let derived = derive(&pair());
        let bands = derived.bands.expect("the bands");
        let line = source_line(&bands, Some(52.0), "24 Nov 2026").expect("a source line");
        assert_eq!(
            line,
            "your zones · the bands from the heart rates you entered · maximum heart rate 186 bpm \
             · resting heart rate 52 bpm · entered on 24 Nov 2026"
                .to_string()
        );
        // And with a maximum alone, the line names the one number it has.
        let derived = derive(&maximum_only());
        let bands = derived.bands.expect("the bands");
        let line = source_line(&bands, None, "3 Nov 2026").expect("a source line");
        assert_eq!(
            line,
            "your zones · the bands from the heart rates you entered · maximum heart rate 186 bpm \
             · entered on 3 Nov 2026"
                .to_string()
        );
    }

    #[test]
    fn the_sources_are_a_closed_list_of_five() {
        assert_eq!(HeartRateSource::ALL.len(), 5);
        assert_eq!(HeartRateSource::Watch.label(), "from my watch");
        assert_eq!(HeartRateSource::Test.label(), "from a test");
        assert_eq!(
            HeartRateSource::DoctorLab.label(),
            "from a doctor or a lab"
        );
        assert_eq!(
            HeartRateSource::PartnerWatch.label(),
            "from my training partner's watch"
        );
        assert_eq!(HeartRateSource::Typed.label(), "typed");
        // And a source the app does not know is not one of them.
        assert!(HeartRateSource::of("a rumour").is_none());
    }
}
