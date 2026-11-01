//! The statements: factual sentences about the data, rendered by the core.
//!
//! A statement is the sixth kind of user-facing string, and it is the one
//! that sits between the vitals and the plan: it says what the numbers are
//! doing, without judging them and without telling the athlete anything.
//!
//! * the **distance comparison** — "2.4 km more than the 10 km target", and
//!   only when the gap from the target is beyond 10% (contract §5);
//! * the **Progress headline** — "fitter than 8 weeks ago", with its
//!   evidence clause ("same heart rate, 24 s/km faster");
//! * the **Compare basis** — "Same distance, 3 weeks apart".
//!
//! The rules a statement lives by:
//!
//! * it carries no verdict and no instruction — the ban list applies to it
//!   like to every other kind;
//! * it **may** name a vital: an `option` must not, because an option is a
//!   choice and naming a vital is how a choice becomes a judgement; a
//!   statement is a fact, and a fact may be about the sleep;
//! * it is **rendered by the core**, with the values it compares already in
//!   the sentence. A hole left in a statement is a place for a second
//!   implementation to grow back, so the lint refuses one.

use serde::{Deserialize, Serialize};

use crate::strings;
use crate::types::Distance;
use crate::units;

/// The gap from the target above which a distance comparison is news.
///
/// Contract §5: *shown only when the gap from the plan's target exceeds
/// **10%***. Within 10% it is absent — 10.2 km against 10 km is not news, and
/// neither more nor less is good or bad.
pub const DISTANCE_COMPARISON_THRESHOLD: f64 = 0.10;

/// The distance comparison: "2.4 km more than the 10 km target".
///
/// `None` when the gap is within ten per cent, and `None` when there is no
/// target to compare against: a near-zero baseline yields no percentage,
/// ever.
pub fn distance_comparison(actual: Distance, target: Distance) -> Option<String> {
    if target.metres() <= 0.0 {
        return None;
    }
    let gap = (actual.metres() - target.metres()).abs();
    let share = gap / target.metres();
    if share <= DISTANCE_COMPARISON_THRESHOLD {
        return None;
    }
    let gap_text = format!("{} km", units::format_distance_km_short(gap));
    let target_text = format!("{} km", units::format_distance_km_short(target.metres()));
    let id = if actual.metres() >= target.metres() {
        "statement.distance.more"
    } else {
        "statement.distance.less"
    };
    strings::render(id, &[("gap", &gap_text), ("target", &target_text)])
}

/// The distance comparison as the contract prints it:
/// "12.4 km · 2.4 km more than the 10 km target".
///
/// `None` exactly when [`distance_comparison`] is `None`.
pub fn distance_comparison_line(actual: Distance, target: Distance) -> Option<String> {
    let comparison = distance_comparison(actual, target)?;
    let distance = format!("{} km", units::format_distance_km_short(actual.metres()));
    strings::render(
        "statement.distance.line",
        &[("distance", &distance), ("comparison", &comparison)],
    )
}

/// The pace at the same heart rate, now and then: the honest fitness metric.
///
/// A lower pace at the same heart rate is a fitter athlete; a higher one is
/// not. The core renders the sentence; the numbers come from the fitness
/// engine.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaceAtHr {
    /// The pace now, in seconds per kilometre.
    pub now_s_per_km: f64,
    /// The pace then, in seconds per kilometre, at the same heart rate.
    pub then_s_per_km: f64,
    /// How many weeks apart the two readings are.
    pub weeks: u32,
}

/// The Progress headline and the evidence under it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressHeadline {
    /// The headline: "fitter than 8 weeks ago".
    pub headline: String,
    /// The evidence clause, when there is one: "same heart rate, 24 s/km
    /// faster".
    pub evidence: Option<String>,
}

/// The Progress headline.
///
/// `None` for the comparison means there is not enough history yet, and the
/// headline says so plainly rather than inventing a trend.
pub fn progress_headline(comparison: Option<PaceAtHr>) -> ProgressHeadline {
    let comparison = match comparison {
        Some(comparison) => comparison,
        None => {
            let headline = strings::text("status.not_enough_data_yet")
                .unwrap_or("not enough data yet")
                .to_string();
            return ProgressHeadline {
                headline,
                evidence: None,
            };
        }
    };
    let ago = weeks_ago(comparison.weeks);
    let difference = comparison.then_s_per_km - comparison.now_s_per_km;
    if difference > 0.0 {
        let pace = format_pace(difference);
        ProgressHeadline {
            headline: strings::render("statement.progress.fitter", &[("ago", &ago)])
                .unwrap_or_default(),
            evidence: strings::render("statement.progress.evidence.faster", &[("pace", &pace)]),
        }
    } else if difference < 0.0 {
        let pace = format_pace(-difference);
        ProgressHeadline {
            headline: strings::render("statement.progress.slower", &[("ago", &ago)])
                .unwrap_or_default(),
            evidence: strings::render("statement.progress.evidence.slower", &[("pace", &pace)]),
        }
    } else {
        ProgressHeadline {
            headline: strings::render("statement.progress.unchanged", &[("ago", &ago)])
                .unwrap_or_default(),
            evidence: None,
        }
    }
}

/// What two runs are being compared on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareBasis {
    /// The same distance, a number of days apart.
    SameDistance {
        /// How many days apart the two runs are.
        days_apart: i64,
    },
    /// The same route, in a different week.
    SameRoute,
    /// Different distances.
    DifferentDistances,
}

impl CompareBasis {
    /// The basis sentence: "Same distance, 3 weeks apart".
    pub fn sentence(&self) -> String {
        match self {
            CompareBasis::SameDistance { days_apart } => {
                let gap = days_apart_text(*days_apart);
                strings::render("statement.compare.same_distance", &[("gap", &gap)])
                    .unwrap_or_default()
            }
            CompareBasis::SameRoute => strings::text("statement.compare.same_route")
                .unwrap_or("Same route, different week")
                .to_string(),
            CompareBasis::DifferentDistances => strings::text("statement.compare.different")
                .unwrap_or("Different distances — read with care")
                .to_string(),
        }
    }
}

/// The basis for two runs, from what is known about them.
///
/// The same route is the strongest basis; the same distance is the next; two
/// different distances are compared anyway, and the sentence says so.
pub fn compare_basis(same_distance: bool, same_route: bool, days_apart: i64) -> CompareBasis {
    if same_route {
        CompareBasis::SameRoute
    } else if same_distance {
        CompareBasis::SameDistance { days_apart }
    } else {
        CompareBasis::DifferentDistances
    }
}

/// How long ago, in weeks: "8 weeks ago", "1 week ago".
fn weeks_ago(weeks: u32) -> String {
    if weeks == 1 {
        "1 week ago".to_string()
    } else {
        format!("{weeks} weeks ago")
    }
}

/// How far apart two runs are: "3 weeks apart", "10 days apart", "on the same
/// day".
fn days_apart_text(days_apart: i64) -> String {
    let days = days_apart.abs();
    if days == 0 {
        "on the same day".to_string()
    } else if days < 14 {
        if days == 1 {
            "1 day apart".to_string()
        } else {
            format!("{days} days apart")
        }
    } else {
        let weeks = units::round_half_even(days as f64 / 7.0, 0) as i64;
        if weeks == 1 {
            "1 week apart".to_string()
        } else {
            format!("{weeks} weeks apart")
        }
    }
}

/// A pace difference in seconds per kilometre: "24", "4.5".
///
/// The contract rounds a pace to two decimals; a difference is read as a
/// person reads it, with the decimals that carry information and no others.
fn format_pace(seconds: f64) -> String {
    let rounded = units::round_half_even(seconds, 2);
    let text = format!("{rounded:.2}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strings::StringKind;

    fn distance(kilometres: f64) -> Option<Distance> {
        Distance::from_kilometres(kilometres)
    }

    #[test]
    fn a_gap_beyond_ten_per_cent_is_stated_and_a_smaller_one_is_not() {
        let actual = match distance(12.4) {
            Some(distance) => distance,
            None => return,
        };
        let target = match distance(10.0) {
            Some(distance) => distance,
            None => return,
        };
        assert_eq!(
            distance_comparison(actual, target),
            Some("2.4 km more than the 10 km target".to_string())
        );
        assert_eq!(
            distance_comparison_line(actual, target),
            Some("12.4 km · 2.4 km more than the 10 km target".to_string())
        );

        // 10.2 km against a 10 km target is a 2% gap: not news.
        let close = match distance(10.2) {
            Some(distance) => distance,
            None => return,
        };
        assert_eq!(distance_comparison(close, target), None);
        assert_eq!(distance_comparison_line(close, target), None);
    }

    #[test]
    fn the_comparison_is_symmetric_and_neutral() {
        let target = match distance(10.0) {
            Some(distance) => distance,
            None => return,
        };
        let under = match distance(8.9) {
            Some(distance) => distance,
            None => return,
        };
        assert_eq!(
            distance_comparison(under, target),
            Some("1.1 km less than the 10 km target".to_string())
        );
        // Exactly ten per cent is not "beyond" ten per cent.
        let edge = match distance(11.0) {
            Some(distance) => distance,
            None => return,
        };
        assert_eq!(distance_comparison(edge, target), None);
    }

    #[test]
    fn a_target_of_zero_yields_no_percentage_and_no_sentence() {
        let actual = match distance(12.4) {
            Some(distance) => distance,
            None => return,
        };
        let zero = match distance(0.0) {
            Some(distance) => distance,
            None => return,
        };
        assert_eq!(distance_comparison(actual, zero), None);
    }

    #[test]
    fn the_progress_headline_states_the_trend_and_its_evidence() {
        let headline = progress_headline(Some(PaceAtHr {
            now_s_per_km: 312.0,
            then_s_per_km: 336.0,
            weeks: 8,
        }));
        assert_eq!(headline.headline, "fitter than 8 weeks ago");
        assert_eq!(
            headline.evidence,
            Some("same heart rate, 24 s/km faster".to_string())
        );
    }

    #[test]
    fn a_slower_pace_is_stated_as_slower_and_an_unchanged_one_is_neither() {
        let slower = progress_headline(Some(PaceAtHr {
            now_s_per_km: 336.0,
            then_s_per_km: 312.0,
            weeks: 8,
        }));
        assert_eq!(slower.headline, "slower than 8 weeks ago");
        assert_eq!(
            slower.evidence,
            Some("same heart rate, 24 s/km slower".to_string())
        );

        let unchanged = progress_headline(Some(PaceAtHr {
            now_s_per_km: 312.0,
            then_s_per_km: 312.0,
            weeks: 8,
        }));
        assert_eq!(unchanged.headline, "the same pace as 8 weeks ago");
        assert_eq!(unchanged.evidence, None);
    }

    #[test]
    fn one_week_reads_as_one_week() {
        let headline = progress_headline(Some(PaceAtHr {
            now_s_per_km: 312.0,
            then_s_per_km: 336.0,
            weeks: 1,
        }));
        assert_eq!(headline.headline, "fitter than 1 week ago");
    }

    #[test]
    fn without_a_comparison_the_headline_says_so_plainly() {
        let headline = progress_headline(None);
        assert_eq!(headline.headline, "not enough data yet");
        assert_eq!(headline.evidence, None);
    }

    #[test]
    fn the_basis_names_what_is_actually_being_compared() {
        assert_eq!(
            compare_basis(true, false, 21).sentence(),
            "Same distance, 3 weeks apart"
        );
        assert_eq!(
            compare_basis(false, true, 21).sentence(),
            "Same route, different week"
        );
        assert_eq!(
            compare_basis(false, false, 21).sentence(),
            "Different distances — read with care"
        );
        // The same route wins over the same distance: it is the stronger basis.
        assert_eq!(compare_basis(true, true, 21), CompareBasis::SameRoute);
    }

    #[test]
    fn a_gap_of_a_few_days_is_stated_in_days() {
        assert_eq!(
            compare_basis(true, false, 10).sentence(),
            "Same distance, 10 days apart"
        );
        assert_eq!(
            compare_basis(true, false, 0).sentence(),
            "Same distance, on the same day"
        );
    }

    #[test]
    fn every_statement_the_core_renders_carries_no_hole() {
        let actual = match distance(12.4) {
            Some(distance) => distance,
            None => return,
        };
        let target = match distance(10.0) {
            Some(distance) => distance,
            None => return,
        };
        let mut rendered: Vec<String> = Vec::new();
        if let Some(line) = distance_comparison_line(actual, target) {
            rendered.push(line);
        }
        let headline = progress_headline(Some(PaceAtHr {
            now_s_per_km: 312.0,
            then_s_per_km: 336.0,
            weeks: 8,
        }));
        rendered.push(headline.headline);
        if let Some(evidence) = headline.evidence {
            rendered.push(evidence);
        }
        rendered.push(compare_basis(true, false, 21).sentence());
        rendered.push(compare_basis(false, true, 21).sentence());
        rendered.push(compare_basis(false, false, 21).sentence());
        for sentence in &rendered {
            assert!(
                strings::lint::check_rendered_statement(sentence).is_empty(),
                "\"{sentence}\" still carries a hole"
            );
        }
    }

    #[test]
    fn a_statement_may_name_a_vital_and_a_banned_word_is_still_refused() {
        // A statement is a fact, and a fact may be about the sleep.
        assert!(strings::lint::check_text(
            StringKind::Statement,
            None,
            "same heart rate, 24 s/km faster"
        )
        .is_empty());
        let violations =
            strings::lint::check_text(StringKind::Statement, None, "you should rest today");
        assert!(!violations.is_empty());
    }
}
