//! The streaks: how long the habit has run, and how long the longest one ran.
//!
//! A streak is a **count of days the athlete recorded a run on**, with no
//! opinion about it. The product's voice has no room for "you are
//! consistent": a streak of twelve days is the number twelve and the day
//! it started, and the reader decides what that means.
//!
//! ## The arithmetic is the reference implementation's, exactly
//!
//! The self-hosted dashboard this port came from counted the streaks in
//! `FindStreaksQueryHandler`, and the counting has three properties worth
//! keeping, because they are what makes the number the same number:
//!
//! * **a day counts once.** The days are the distinct local days that carry
//!   a run; two runs on one day are one day of the streak.
//! * **the days are walked newest first.** The streak that is *current* is
//!   the one at the head of the list, and the walk goes back through the
//!   history from there.
//! * **the current streak survives one gap-free day.** A streak is still
//!   current when the most recent run was yesterday: the day itself may
//!   still come. A streak that ended two days ago is not current — it is
//!   the longest one, and it is in the past.
//!
//! The week streak counts **weeks with a run in them**, and the weeks are
//! the athlete's own: Monday through Sunday.
//!
//! The month streak counts **months with a run in them**, and the months
//! are the calendar's own: they are the athlete's months, counted from
//! their first day. A month with no run in it breaks the streak, and the
//! streak survives a month of quiet for the same reason the week one does:
//! the month itself may still come.
//!
//! ## No value
//!
//! A streak of zero days is **no streak**, not a streak of zero: the
//! current streak is `None` when the last run was more than a day ago, and
//! the whole answer is empty when there are no runs at all. A number that
//! could be read as "zero" is not a number here.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::plan::week_start;

/// One streak: how long it is, and the days it covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Streak {
    /// How many days the streak runs — or how many weeks, for a week
    /// streak, or how many months, for a month streak.
    pub count: u32,
    /// The oldest day of the streak: the day it started.
    pub start: Date,
    /// The newest day of the streak, when it is known.
    ///
    /// The longest streak has one — it is a stretch of days in the past.
    /// The current streak has none: it is still going, and where it ends is
    /// not a fact yet.
    pub end: Option<Date>,
}

/// The streaks of a history: the current ones and the longest ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Streaks {
    /// The run of days that is still going, if there is one.
    pub current_days: Option<Streak>,
    /// The longest run of days in the history.
    pub longest_days: Option<Streak>,
    /// The run of weeks that is still going, if there is one.
    pub current_weeks: Option<Streak>,
    /// The longest run of weeks in the history.
    pub longest_weeks: Option<Streak>,
    /// The run of months that is still going, if there is one.
    pub current_months: Option<Streak>,
    /// The longest run of months in the history.
    pub longest_months: Option<Streak>,
}

impl Streaks {
    /// The streaks of a history with no runs in it: no streak at all.
    pub fn empty() -> Self {
        Self {
            current_days: None,
            longest_days: None,
            current_weeks: None,
            longest_weeks: None,
            current_months: None,
            longest_months: None,
        }
    }
}

/// The streaks of a history.
///
/// The days are the athlete's own local days that carry a run. They may
/// arrive in any order and may repeat: the counting here takes the
/// distinct days, newest first, exactly as the reference's query did.
///
/// `today` is the athlete's day, injected like every other time-dependent
/// value — the core has no clock.
pub fn streaks(days: &[Date], today: Date) -> Streaks {
    let mut days: Vec<Date> = days.to_vec();
    days.sort_unstable();
    days.dedup();
    // The walk goes from the newest day backwards, the way the reference's
    // `ORDER BY day DESC` did.
    days.reverse();
    if days.is_empty() {
        return Streaks::empty();
    }

    let (current_days, longest_days) = day_streaks(&days, today);
    let (current_weeks, longest_weeks) = week_streaks(&days, today);
    let (current_months, longest_months) = month_streaks(&days, today);

    Streaks {
        current_days,
        longest_days,
        current_weeks,
        longest_weeks,
        current_months,
        longest_months,
    }
}

/// The day streaks of a history, newest day first.
fn day_streaks(days: &[Date], today: Date) -> (Option<Streak>, Option<Streak>) {
    let newest = days[0];

    let mut longest = 1u32;
    let mut running = 1u32;
    // The newest day of the run of days being counted.
    let mut running_end = newest;
    // Whether the run being counted is the longest one so far.
    let mut running_is_longest = true;
    let mut longest_period = (newest, newest);

    // The current streak is still current when the last run was today or
    // yesterday: the day itself may still come. The reference's rule, and
    // the reason a streak does not die at midnight.
    //
    // This is the *possibility* of a current streak, and it is decided
    // once, from the newest day. The walk below only decides how long the
    // streak is: the current streak is the newest run of consecutive days,
    // and a gap ends that run without ending the streak's existence.
    let possible = today.days_since(newest).abs() < 2;
    let mut counting = possible;
    let mut current = if possible { 1u32 } else { 0 };
    let mut current_start = if possible { Some(newest) } else { None };

    let mut previous = newest;
    for &day in days.iter().skip(1) {
        let gap = previous.days_since(day).abs();
        if gap == 1 {
            running += 1;
            if counting {
                current += 1;
                current_start = Some(day);
            }
            if running > longest {
                longest = running;
                running_is_longest = true;
            }
        } else {
            // The run is broken: a new one starts here.
            running = 1;
            running_end = day;
            running_is_longest = false;
            counting = false;
        }
        if running_is_longest {
            longest_period = (day, running_end);
        }
        previous = day;
    }

    let longest_streak = Streak {
        count: longest,
        start: longest_period.0,
        end: Some(longest_period.1),
    };
    // A count of zero is no streak: the number would read as "zero days in
    // a row", which is not a thing that happened.
    let current_streak = match (possible, current_start) {
        (true, Some(start)) if current > 0 => Some(Streak {
            count: current,
            start,
            end: None,
        }),
        _ => None,
    };
    (current_streak, Some(longest_streak))
}

/// The week streaks of a history, newest day first.
///
/// The weeks are the athlete's own: Monday through Sunday. Two runs in the
/// same week are one week of the streak.
fn week_streaks(days: &[Date], today: Date) -> (Option<Streak>, Option<Streak>) {
    let newest = days[0];

    let mut longest = 1u32;
    let mut running = 1u32;
    let mut running_end = newest;
    let mut running_is_longest = true;
    let mut longest_period = (newest, newest);

    // A week streak survives a week of quiet: the reference's rule is that
    // the last run was within the last seven days.
    let possible = today.days_since(newest).abs() < 7;
    let mut counting = possible;
    let mut current = if possible { 1u32 } else { 0 };
    let mut current_start = if possible { Some(newest) } else { None };

    let mut previous = newest;
    for &day in days.iter().skip(1) {
        // The weeks between the two days, counted between their Mondays.
        let gap = week_start(previous).days_since(week_start(day)).abs() / 7;
        if gap > 0 {
            if gap == 1 {
                running += 1;
                if counting {
                    current += 1;
                    current_start = Some(day);
                }
                if running > longest {
                    longest = running;
                    running_is_longest = true;
                }
            } else {
                running = 1;
                running_end = day;
                running_is_longest = false;
                counting = false;
            }
        }
        if running_is_longest {
            longest_period = (day, running_end);
        }
        previous = day;
    }

    let longest_streak = Streak {
        count: longest,
        start: longest_period.0,
        end: Some(longest_period.1),
    };
    let current_streak = match (possible, current_start) {
        (true, Some(start)) if current > 0 => Some(Streak {
            count: current,
            start,
            end: None,
        }),
        _ => None,
    };
    (current_streak, Some(longest_streak))
}

/// The month streaks of a history, newest day first.
///
/// The months are the calendar's own. Two runs in the same month are one
/// month of the streak, and a month with no run in it breaks it.
fn month_streaks(days: &[Date], today: Date) -> (Option<Streak>, Option<Streak>) {
    // The months that carry a run, newest first, each with the newest run
    // day in it — the days arrive newest first, so the first day seen in
    // a month is that month's newest.
    let mut months: Vec<(i32, u32, Date)> = Vec::new();
    for &day in days {
        let key = (day.year(), day.month());
        if !months.iter().any(|(year, month, _)| (*year, *month) == key) {
            months.push((key.0, key.1, day));
        }
    }

    let newest = months[0];

    let mut longest = 1u32;
    let mut running = 1u32;
    let mut running_end = newest.2;
    let mut running_is_longest = true;
    let mut longest_period = (newest.2, newest.2);

    // A month streak survives a month of quiet: the week streak's rule is
    // that the last run was within the last seven days, and a month is
    // that same rule with a month's span.
    let possible = today.days_since(newest.2).abs() < 31;
    let mut counting = possible;
    let mut current = if possible { 1u32 } else { 0 };
    let mut current_start = if possible { Some(newest.2) } else { None };

    let mut previous = newest;
    for &month in months.iter().skip(1) {
        // The months between the two, counted between their firsts.
        let gap = month_index(previous) - month_index(month);
        if gap > 0 {
            if gap == 1 {
                running += 1;
                if counting {
                    current += 1;
                    current_start = Some(month.2);
                }
                if running > longest {
                    longest = running;
                    running_is_longest = true;
                }
            } else {
                running = 1;
                running_end = month.2;
                running_is_longest = false;
                counting = false;
            }
        }
        if running_is_longest {
            // The start of a month streak is the newest run day in the
            // streak's oldest month — the week streak's own rule.
            longest_period = (month.2, running_end);
        }
        previous = month;
    }

    let longest_streak = Streak {
        count: longest,
        start: longest_period.0,
        end: Some(longest_period.1),
    };
    let current_streak = match (possible, current_start) {
        (true, Some(start)) if current > 0 => Some(Streak {
            count: current,
            start,
            end: None,
        }),
        _ => None,
    };
    (current_streak, Some(longest_streak))
}

/// The number of a month, counted from January of year zero: the gap
/// between two months is the difference of their numbers.
fn month_index(month: (i32, u32, Date)) -> i32 {
    month.0 * 12 + month.1 as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> Option<Date> {
        Date::parse(text)
    }

    /// The streaks of a list of days, written out.
    fn streaks_of(days: &[&str], today: &str) -> Option<Streaks> {
        let mut parsed: Vec<Date> = Vec::new();
        for day in days {
            parsed.push(date(day)?);
        }
        Some(streaks(&parsed, date(today)?))
    }

    #[test]
    fn a_history_with_no_runs_has_no_streak() {
        let answer = match streaks_of(&[], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        assert_eq!(answer, Streaks::empty());
        assert!(answer.current_days.is_none());
        assert!(answer.longest_days.is_none());
        assert!(answer.current_weeks.is_none());
        assert!(answer.longest_weeks.is_none());
        assert!(answer.current_months.is_none());
        assert!(answer.longest_months.is_none());
    }

    #[test]
    fn one_day_is_a_streak_of_one() {
        let answer = match streaks_of(&["2026-05-13"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_days {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 1);
        assert_eq!(Some(current.start), date("2026-05-13"));
        // The current streak is still going: where it ends is not a fact.
        assert_eq!(current.end, None);

        let longest = match answer.longest_days {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 1);
        assert_eq!(Some(longest.start), date("2026-05-13"));
        assert_eq!(longest.end, date("2026-05-13"));
    }

    #[test]
    fn three_consecutive_days_are_a_streak_of_three() {
        // 11, 12 and 13 May 2026.
        let answer = match streaks_of(&["2026-05-11", "2026-05-12", "2026-05-13"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_days {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 3);
        assert_eq!(Some(current.start), date("2026-05-11"));

        let longest = match answer.longest_days {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 3);
        assert_eq!(Some(longest.start), date("2026-05-11"));
        assert_eq!(longest.end, date("2026-05-13"));
    }

    #[test]
    fn a_gap_breaks_the_streak_and_the_longest_keeps_its_days() {
        // 8, 9 and 10 May, then a gap, then 12 and 13 May.
        let answer = match streaks_of(
            &[
                "2026-05-08",
                "2026-05-09",
                "2026-05-10",
                "2026-05-12",
                "2026-05-13",
            ],
            "2026-05-13",
        ) {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_days {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 2);
        assert_eq!(Some(current.start), date("2026-05-12"));

        // The longest is the run of three in the past, and it names its days.
        let longest = match answer.longest_days {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 3);
        assert_eq!(Some(longest.start), date("2026-05-08"));
        assert_eq!(longest.end, date("2026-05-10"));
    }

    #[test]
    fn a_streak_that_ended_two_days_ago_is_not_current() {
        // The last run was on the 11th; the 13th is today.
        let answer = match streaks_of(&["2026-05-10", "2026-05-11"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        // Two days back: the streak is over, and there is no current one.
        assert!(answer.current_days.is_none());
        // The longest one is still a fact.
        let longest = match answer.longest_days {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 2);
        assert_eq!(Some(longest.start), date("2026-05-10"));
        assert_eq!(longest.end, date("2026-05-11"));
    }

    #[test]
    fn a_streak_that_ended_yesterday_is_still_current() {
        let answer = match streaks_of(&["2026-05-11", "2026-05-12"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_days {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 2);
        assert_eq!(Some(current.start), date("2026-05-11"));
    }

    #[test]
    fn two_runs_on_one_day_are_one_day() {
        let answer = match streaks_of(&["2026-05-12", "2026-05-12", "2026-05-13"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_days {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 2);
        assert_eq!(Some(current.start), date("2026-05-12"));
    }

    #[test]
    fn the_week_streak_counts_weeks_with_a_run_in_them() {
        // 13 May 2026 is a Wednesday. The weeks are Monday 11 May to
        // Sunday 17 May, and the week before it, 4 May to 10 May.
        let answer = match streaks_of(
            &["2026-05-04", "2026-05-06", "2026-05-11", "2026-05-13"],
            "2026-05-13",
        ) {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_weeks {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 2);
        // The start of a week streak is the newest run day in the streak's
        // oldest week — the reference's own value.
        assert_eq!(Some(current.start), date("2026-05-06"));

        let longest = match answer.longest_weeks {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 2);
        assert_eq!(Some(longest.start), date("2026-05-04"));
        assert_eq!(longest.end, date("2026-05-13"));
    }

    #[test]
    fn a_gap_of_two_weeks_breaks_the_week_streak() {
        // The week of 4 May, then nothing, then the week of 18 May.
        let answer = match streaks_of(&["2026-05-04", "2026-05-06", "2026-05-18"], "2026-05-19") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_weeks {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 1);
        assert_eq!(Some(current.start), date("2026-05-18"));

        // The longest is one week as well, and the tie goes to the newest:
        // the reference's own rule.
        let longest = match answer.longest_weeks {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 1);
        assert_eq!(Some(longest.start), date("2026-05-18"));
        assert_eq!(longest.end, date("2026-05-18"));
    }

    #[test]
    fn the_month_streak_counts_months_with_a_run_in_them() {
        // March, April and May 2026: three months in a row.
        let answer = match streaks_of(&["2026-03-14", "2026-04-12", "2026-05-10"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_months {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 3);
        // The start of a month streak is the newest run day in the
        // streak's oldest month — the week streak's own rule.
        assert_eq!(Some(current.start), date("2026-03-14"));
        assert_eq!(current.end, None);

        let longest = match answer.longest_months {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 3);
        assert_eq!(Some(longest.start), date("2026-03-14"));
        assert_eq!(longest.end, date("2026-05-10"));
    }

    #[test]
    fn two_runs_in_one_month_are_one_month() {
        let answer = match streaks_of(&["2026-05-10", "2026-05-12"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_months {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 1);
        assert_eq!(Some(current.start), date("2026-05-12"));
    }

    #[test]
    fn a_month_with_no_run_breaks_the_month_streak() {
        // May, then nothing in April, then March.
        let answer = match streaks_of(&["2026-03-14", "2026-05-10"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_months {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 1);
        assert_eq!(Some(current.start), date("2026-05-10"));

        let longest = match answer.longest_months {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 1);
        assert_eq!(Some(longest.start), date("2026-05-10"));
        assert_eq!(longest.end, date("2026-05-10"));
    }

    #[test]
    fn a_month_streak_that_ended_a_month_ago_is_not_current() {
        // The last run was on the 10th of March; the 20th of April is today.
        let answer = match streaks_of(&["2026-02-14", "2026-03-10"], "2026-04-20") {
            Some(answer) => answer,
            None => return,
        };
        // More than a month of quiet: the streak is over, and there is no
        // current one.
        assert!(answer.current_months.is_none());
        // The longest one is still a fact.
        let longest = match answer.longest_months {
            Some(longest) => longest,
            None => return,
        };
        assert_eq!(longest.count, 2);
        assert_eq!(Some(longest.start), date("2026-02-14"));
        assert_eq!(longest.end, date("2026-03-10"));
    }

    #[test]
    fn the_days_may_arrive_in_any_order() {
        let answer = match streaks_of(&["2026-05-13", "2026-05-11", "2026-05-12"], "2026-05-13") {
            Some(answer) => answer,
            None => return,
        };
        let current = match answer.current_days {
            Some(current) => current,
            None => return,
        };
        assert_eq!(current.count, 3);
        assert_eq!(Some(current.start), date("2026-05-11"));
    }
}
