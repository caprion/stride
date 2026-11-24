//! The adopted plan: the store's own rows.
//!
//! An adopted plan is a **dated copy** of a programme, and the sessions
//! that were worked out on the day it was adopted are stored with it. The
//! tests here hold that copy to what it says it is: the programme's id and
//! version, the days, the paces, and the rule and the threshold the paces
//! came from.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_store::{NewAdoptedPlan, NewAdoptedSession, Result};

/// The plan the tests write: the half-marathon tier 2, starting on the
/// second of November 2026, with the race on the twenty-fourth of January
/// 2027.
fn plan() -> NewAdoptedPlan {
    NewAdoptedPlan {
        template_id: "pfitzinger-half-marathon-tier-2".to_string(),
        template_version: "1".to_string(),
        label: "half marathon tier 2".to_string(),
        method: "Pfitzinger".to_string(),
        tier: "tier 2".to_string(),
        goal_distance_m: 21_097.5,
        goal_label: "half marathon".to_string(),
        race_date: "2027-01-24".to_string(),
        start_date: "2026-11-02".to_string(),
        end_date: "2027-02-07".to_string(),
        adopted_on: "2026-11-04".to_string(),
        threshold_value: Some("165 bpm".to_string()),
        threshold_date: Some("2026-11-03".to_string()),
        threshold_source: Some("watch_zone_model".to_string()),
        threshold_base: Some("pfitzinger".to_string()),
        rule_value: Some("10K".to_string()),
        rule_date: Some("2026-10-12".to_string()),
        paced: true,
        pacing_missing: None,
        created_at: common::instant(),
    }
}

/// The plan's own sessions: three of them, one of them without a pace.
fn sessions(plan_id: i64) -> Vec<NewAdoptedSession> {
    vec![
        NewAdoptedSession {
            plan_id,
            date: "2026-12-28".to_string(),
            week: 3,
            kind: "rest_cross".to_string(),
            distance_m: None,
            pace_low_s_per_km: None,
            pace_high_s_per_km: None,
            finish_low_s_per_km: None,
            finish_high_s_per_km: None,
            pace_rule: None,
            threshold_value: None,
            detail: None,
            lt_last_m: None,
        },
        NewAdoptedSession {
            plan_id,
            date: "2026-12-30".to_string(),
            week: 3,
            kind: "endurance".to_string(),
            distance_m: Some(28_500.0),
            pace_low_s_per_km: Some(300.0),
            pace_high_s_per_km: Some(332.0),
            finish_low_s_per_km: Some(299.0),
            finish_high_s_per_km: Some(337.0),
            pace_rule: Some("book.long_run".to_string()),
            threshold_value: Some("165 bpm".to_string()),
            detail: None,
            lt_last_m: None,
        },
        NewAdoptedSession {
            plan_id,
            date: "2026-12-31".to_string(),
            week: 3,
            kind: "lt".to_string(),
            distance_m: Some(12_900.0),
            pace_low_s_per_km: Some(239.0),
            pace_high_s_per_km: Some(261.0),
            finish_low_s_per_km: None,
            finish_high_s_per_km: None,
            pace_rule: Some("book.lt_pace_band".to_string()),
            threshold_value: Some("165 bpm".to_string()),
            detail: Some("3 × 8 min, jog 3".to_string()),
            lt_last_m: None,
        },
    ]
}

#[test]
fn the_plan_and_its_sessions_are_stored_together() -> Result<()> {
    let (dir, store) = common::store("adopted-write")?;
    let id = store.write_adopted_plan(&plan(), &sessions(0))?;
    assert!(id > 0);

    let plans = store.adopted_plans()?;
    assert_eq!(plans.len(), 1);

    let stored = store.adopted_plan(id)?;
    let stored = match stored {
        Some(stored) => stored,
        None => return Err(stride_store::StoreError::Invariant("no plan".to_string())),
    };
    assert_eq!(stored.template_id, "pfitzinger-half-marathon-tier-2");
    assert_eq!(stored.template_version, "1");
    assert_eq!(stored.tier, "tier 2");
    assert_eq!(stored.start_date, "2026-11-02");
    assert_eq!(stored.race_date, "2027-01-24");
    assert_eq!(stored.threshold_value.as_deref(), Some("165 bpm"));
    assert_eq!(stored.threshold_date.as_deref(), Some("2026-11-03"));
    assert_eq!(stored.threshold_source.as_deref(), Some("watch_zone_model"));
    assert_eq!(stored.threshold_base.as_deref(), Some("pfitzinger"));
    assert!(stored.paced);
    assert_eq!(stored.pacing_missing, None);
    assert_eq!(stored.source, "prescribed");
    assert_eq!(stored.status, "active");

    drop(dir);
    Ok(())
}

#[test]
fn the_sessions_keep_their_paces_and_where_they_came_from() -> Result<()> {
    let (dir, store) = common::store("adopted-sessions")?;
    let id = store.write_adopted_plan(&plan(), &sessions(0))?;

    let rows = store.adopted_sessions(id)?;
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].kind, "rest_cross");
    assert_eq!(rows[0].distance_m, None);
    assert_eq!(rows[0].pace_low_s_per_km, None);

    assert_eq!(rows[1].kind, "endurance");
    assert_eq!(rows[1].distance_m, Some(28_500.0));
    assert_eq!(rows[1].pace_low_s_per_km, Some(300.0));
    assert_eq!(rows[1].pace_high_s_per_km, Some(332.0));
    assert_eq!(rows[1].pace_rule.as_deref(), Some("book.long_run"));
    assert_eq!(rows[1].threshold_value.as_deref(), Some("165 bpm"));

    assert_eq!(rows[2].kind, "lt");
    assert_eq!(rows[2].pace_rule.as_deref(), Some("book.lt_pace_band"));
    assert_eq!(rows[2].detail.as_deref(), Some("3 × 8 min, jog 3"));
    assert_eq!(rows[2].week, 3);

    drop(dir);
    Ok(())
}

#[test]
fn the_plan_in_force_is_the_one_that_covers_the_day() -> Result<()> {
    let (dir, store) = common::store("adopted-in-force")?;
    store.write_adopted_plan(&plan(), &[])?;

    // A day inside the plan.
    let in_force = store.adopted_plan_in_force("2026-12-30")?;
    assert!(in_force.is_some());

    // A day before it and a day after it.
    assert!(store.adopted_plan_in_force("2026-10-31")?.is_none());
    assert!(store.adopted_plan_in_force("2027-02-08")?.is_none());

    assert_eq!(store.count_adopted_plans()?, 1);
    assert_eq!(store.count_adopted_sessions()?, 0);

    drop(dir);
    Ok(())
}

#[test]
fn a_plan_without_paces_says_why() -> Result<()> {
    let (dir, store) = common::store("adopted-unpaced")?;
    let mut plan = plan();
    plan.paced = false;
    plan.pacing_missing = Some("no race time entered — the paces have no value yet".to_string());
    plan.threshold_value = None;
    plan.threshold_date = None;
    plan.threshold_source = None;
    let id = store.write_adopted_plan(&plan, &[])?;

    let stored = store.adopted_plan(id)?;
    let stored = match stored {
        Some(stored) => stored,
        None => return Err(stride_store::StoreError::Invariant("no plan".to_string())),
    };
    assert!(!stored.paced);
    assert_eq!(
        stored.pacing_missing.as_deref(),
        Some("no race time entered — the paces have no value yet")
    );
    assert_eq!(stored.threshold_value, None);

    drop(dir);
    Ok(())
}

#[test]
fn one_plan_is_in_force_at_a_time() -> Result<()> {
    let (dir, store) = common::store("adopted-one-at-a-time")?;
    store.write_adopted_plan(&plan(), &[])?;

    // A second plan: the first one goes to the archive.
    let mut later = plan();
    later.start_date = "2027-03-01".to_string();
    later.end_date = "2027-04-05".to_string();
    later.race_date = "2027-03-21".to_string();
    store.write_adopted_plan(&later, &[])?;

    let active = store.active_adopted_plans()?;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].start_date, "2027-03-01");

    // And the plan that was in force is still there, in the archive.
    let all = store.adopted_plans()?;
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|row| row.status == "archived"));

    drop(dir);
    Ok(())
}
