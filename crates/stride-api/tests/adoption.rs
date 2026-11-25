// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The adoption: the athlete adopts a programme, and the plan is stored.
//!
//! The **paces are worked out once**, at the adoption, from the threshold
//! in force that day, and they are stored with the plan: a later threshold
//! does not re-pace a running plan.

mod common;

use common::Harness;
use serde_json::Value;

/// The canonical athlete's own facts: the watch's own zone model and the
/// 10K of the autumn.
fn athlete(harness: &Harness) {
    harness.write_zone_model(common::zone_model());
    harness.write_race(common::race());
    // The athlete's own threshold choice, made before the adoption and
    // in force on the day of it.
    harness.write_threshold_choice(&common::choice(), true);
    // Four weeks of the athlete's own runs, so that the plan's answer can
    // state their weekly mileage beside the tier's entry requirement.
    for (index, (date, distance)) in [
        ("2026-04-20", 60_800.0),
        ("2026-04-27", 61_100.0),
        ("2026-05-04", 50_800.0),
        ("2026-05-11", 6_000.0),
    ]
    .iter()
    .enumerate()
    {
        harness.write_run(common::run(
            &format!("a-run-{index}"),
            date,
            *distance,
            3_000.0,
        ));
    }
}

/// The adoption: the half-marathon tier 2, the race on the Sunday of the
/// twenty-fourth of January 2027.
fn body() -> String {
    serde_json::json!({
        "programme_id": "pfitzinger-half-marathon-tier-2",
        "tier": 2,
        "race_date": "2027-01-24",
        "goal_distance_m": 21_097.5,
    })
    .to_string()
}

/// A post, with its status.
fn post(harness: &Harness, path: &str, body: &str) -> (u16, Value) {
    let answer = harness.router.call("POST", path, Some(body));
    (answer.status, answer.body)
}

/// The sessions of week three, from the plan's own week.
fn week_three(answer: &Value) -> &Vec<Value> {
    let sessions = answer["adopted"]["sessions"]
        .as_array()
        .expect("the adopted week's sessions");
    assert_eq!(sessions.len(), 7, "the week has seven sessions");
    sessions
}

#[test]
fn the_athlete_adopts_a_programme_and_the_plan_is_stored() {
    let harness = Harness::empty("adopt-write");
    athlete(&harness);

    let (status, answer) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200, "the adoption was refused: {answer}");
    assert_eq!(answer["adopted"], true);

    let plan = &answer["plan"];
    assert_eq!(plan["programme_id"], "pfitzinger-half-marathon-tier-2");
    assert_eq!(plan["programme_version"], "1");
    assert_eq!(plan["label"], "half marathon tier 2");
    assert_eq!(plan["method"], "Pfitzinger");
    assert_eq!(plan["tier"], "tier 2");
    // The race date anchors the plan: the race week is the twelfth, and
    // the plan begins eleven weeks before it.
    assert_eq!(plan["race_date"], "2027-01-24");
    assert_eq!(plan["start_date"], "2026-11-02");
    assert_eq!(plan["end_date"], "2027-02-07");
    assert_eq!(plan["adopted_on"], common::TODAY);
    assert_eq!(plan["goal_label"], "half marathon");
    assert_eq!(plan["source"], "prescribed");

    // The threshold it was paced with: the watch's own zone model, read
    // on the day of the adoption.
    assert_eq!(plan["pacing"]["paced"], true);
    assert_eq!(plan["pacing"]["threshold"]["value"], "165 bpm");
    assert_eq!(plan["pacing"]["threshold"]["source"], "watch_zone_model");
    assert_eq!(plan["pacing"]["threshold"]["base"], "pfitzinger");
    assert_eq!(plan["pacing"]["threshold"]["date"], "2026-05-01");
    // And the rule it was read from: the athlete's own race.
    assert_eq!(plan["pacing"]["rule"]["value"], "10K");
    assert_eq!(plan["pacing"]["rule"]["date"], "2026-10-12");

    // The plan is in the store, with its own sessions.
    let plans = harness.store.adopted_plans().expect("the plans");
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].template_id, "pfitzinger-half-marathon-tier-2");
    assert_eq!(plans[0].tier, "tier 2");
    let sessions = harness
        .store
        .adopted_sessions(plans[0].id)
        .expect("the sessions");
    // Twelve weeks of seven days, and the two weeks after the race.
    assert_eq!(sessions.len(), 14 * 7);
}

#[test]
fn the_plan_that_is_adopted_is_read_back() {
    let harness = Harness::empty("adopt-read");
    athlete(&harness);
    let (status, _) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200);

    let answer = harness.ok_get("/plan/adopt");
    let plan = &answer["plan"];
    assert_eq!(plan["programme_id"], "pfitzinger-half-marathon-tier-2");
    assert_eq!(plan["tier"], "tier 2");
    assert_eq!(plan["race_date"], "2027-01-24");
    assert_eq!(plan["start_date"], "2026-11-02");
    assert_eq!(plan["pacing"]["threshold"]["value"], "165 bpm");

    // The tier's entry requirement, beside the athlete's own mileage:
    // three facts side by side, unranked.
    let entry = &plan["entry"];
    assert_eq!(
        entry["requirement"],
        "this level asks for 64.4-80.5 km a week"
    );
    assert_eq!(entry["plan_band"], "this plan runs 74-101.4 km a week");
    assert_eq!(
        entry["athlete"],
        "your last 4 weeks: 60.8, 61.1, 50.8, 6 km"
    );
}

#[test]
fn no_plan_answers_with_the_reason_and_not_with_nothing() {
    let harness = Harness::empty("adopt-none");
    let answer = harness.ok_get("/plan/adopt");
    assert!(answer["plan"].is_null());
    assert_eq!(answer["unavailable"], "no plan has been adopted");
}

#[test]
fn the_adopted_week_carries_its_seven_sessions_with_their_paces() {
    let harness = Harness::empty("adopt-week");
    athlete(&harness);
    let (status, _) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200);

    // The week of the third of December 2026 — the plan's third week.
    let answer = harness.ok_get("/plan/week?date=2026-12-30");
    let adopted = &answer["adopted"];
    assert_eq!(adopted["week"], 3);
    assert_eq!(adopted["label"], "3 weeks to go");
    assert_eq!(adopted["source"], "prescribed");
    assert_eq!(
        adopted["source_line"],
        "the Pfitzinger half marathon tier 2 plan"
    );

    let sessions = week_three(&answer);
    // The rest day: no distance, and no pace to explain.
    assert_eq!(sessions[0]["weekday"], "Monday");
    assert_eq!(sessions[0]["kind"], "rest or cross-training");
    assert!(sessions[0]["distance"].is_null());
    assert!(sessions[0]["no_pace"].is_null());

    // The general aerobic day: the runner's wording, and the method's own
    // name beside it. The band is the method's answer, and the reason
    // says so.
    assert_eq!(sessions[1]["kind"], "easy");
    assert_eq!(sessions[1]["method_kind"], "GA");
    assert_eq!(sessions[1]["distance"], "14.5 km");
    assert!(sessions[1]["pace"].is_null());
    assert_eq!(
        sessions[1]["no_pace"],
        "the method fixes a heart-rate band for this run and no pace"
    );

    // The long run: its two ends, and the line that carries the first.
    assert_eq!(sessions[2]["kind"], "long run");
    assert_eq!(sessions[2]["method_kind"], "endurance");
    assert_eq!(sessions[2]["distance"], "17.7 km");
    assert_eq!(sessions[2]["pace"], "5:00–5:32 /km");
    assert_eq!(sessions[2]["finish_pace"], "4:59–5:37 /km");
    assert_eq!(sessions[2]["line"], "17.7 km · long run · 5:00–5:32 /km");

    // The intervals, with the detail the book prints for the day.
    assert_eq!(sessions[3]["kind"], "intervals");
    assert_eq!(sessions[3]["method_kind"], "VO₂max");
    assert_eq!(sessions[3]["pace"], "3:59 /km");
    assert_eq!(
        sessions[3]["detail"],
        "6 × 1,200 m @ 3K–5K race pace, jog 50–90 %"
    );

    // The recovery run.
    assert_eq!(sessions[4]["kind"], "recovery");
    assert_eq!(sessions[4]["method_kind"], "recovery");

    // The race-pace run.
    assert_eq!(sessions[5]["kind"], "easy, with strides");
    assert_eq!(sessions[5]["method_kind"], "GA+speed");
    assert!(sessions[5]["pace"].is_null());

    // And the long run again.
    assert_eq!(sessions[6]["kind"], "long run");
    assert_eq!(sessions[6]["pace"], "5:00–5:32 /km");
}

#[test]
fn a_race_on_another_day_moves_the_race_week_alone() {
    let harness = Harness::empty("adopt-shift");
    athlete(&harness);
    // The race on the Saturday instead of the Sunday.
    let (status, _) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-half-marathon-tier-2", "race_date": "2027-01-23"}"#,
    );
    assert_eq!(status, 200);

    // The race week is still the twelfth week, and its sessions moved a
    // day earlier.
    let answer = harness.ok_get("/plan/week?date=2027-01-18");
    let adopted = &answer["adopted"];
    assert_eq!(adopted["week"], 0);
    assert_eq!(adopted["label"], "race week");
    let sessions = adopted["sessions"].as_array().expect("the sessions");
    // The week moved a day earlier: the Monday's session falls outside the
    // week and is not in it, and the goal race is on the Saturday.
    assert_eq!(sessions[0]["date"], "2027-01-18");
    let race = sessions
        .iter()
        .find(|session| session["kind"] == "race")
        .expect("the goal race");
    assert_eq!(race["date"], "2027-01-23");
    // And the race's own distance is the race's, not the printed miles.
    assert_eq!(race["distance"], "21.1 km");
}

#[test]
fn a_programme_the_catalogue_does_not_hold_is_refused() {
    let harness = Harness::empty("adopt-unknown");
    athlete(&harness);
    let (status, answer) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-80-20-tier-1", "race_date": "2027-01-24"}"#,
    );
    assert_eq!(status, 400, "the adoption was made: {answer}");
    assert_eq!(answer["code"], "bad_request");
    assert_eq!(
        answer["detail"],
        "there is no programme called pfitzinger-80-20-tier-1 in the catalogue"
    );
}

#[test]
fn a_tier_the_programme_does_not_have_is_refused() {
    let harness = Harness::empty("adopt-tier");
    athlete(&harness);
    let (status, answer) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-half-marathon-tier-2", "tier": 3, "race_date": "2027-01-24"}"#,
    );
    assert_eq!(status, 400, "the adoption was made: {answer}");
    assert_eq!(answer["code"], "bad_request");
    assert_eq!(
        answer["detail"],
        "pfitzinger-half-marathon-tier-2 is the tier 2 programme"
    );
}

#[test]
fn a_start_date_that_leaves_no_room_is_refused() {
    let harness = Harness::empty("adopt-start");
    athlete(&harness);
    let (status, answer) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-half-marathon-tier-2", "race_date": "2027-01-24", "start_date": "2026-10-26"}"#,
    );
    assert_eq!(status, 400, "the adoption was made: {answer}");
    assert_eq!(answer["code"], "bad_request");
    let detail = answer["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("2026-11-02"), "{detail}");
    assert!(detail.contains("2027-01-24"), "{detail}");
}

#[test]
fn a_goal_race_the_programme_does_not_print_is_refused() {
    let harness = Harness::empty("adopt-goal");
    athlete(&harness);
    let (status, answer) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-half-marathon-tier-2", "race_date": "2027-01-24", "goal_distance_m": 42195.0}"#,
    );
    assert_eq!(status, 400, "the adoption was made: {answer}");
    assert_eq!(answer["code"], "bad_request");
    let detail = answer["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("half marathon"), "{detail}");
}

#[test]
fn a_second_plan_while_one_is_in_force_is_refused() {
    let harness = Harness::empty("adopt-second");
    athlete(&harness);
    let (status, _) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200);

    let (status, answer) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 400, "the second plan was adopted: {answer}");
    assert_eq!(answer["code"], "bad_request");
    assert_eq!(
        answer["detail"],
        "a plan is already in force until 2027-02-07"
    );
}

#[test]
fn a_plan_that_has_finished_is_replaced_and_the_old_one_is_archived() {
    let harness = Harness::empty("adopt-replace");
    athlete(&harness);
    // A plan for a race that has already been run.
    let (status, _) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-half-marathon-tier-2", "race_date": "2026-04-12"}"#,
    );
    assert_eq!(status, 200);

    // And one for the race that is coming.
    let (status, _) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200, "the second plan was refused");

    // One plan is in force: the old one is in the archive.
    let active = harness.store.active_adopted_plans().expect("the plans");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].race_date, "2027-01-24");
    let plans = harness.store.adopted_plans().expect("the plans");
    assert_eq!(plans.len(), 2);
    assert!(plans
        .iter()
        .any(|plan| plan.status == "archived" && plan.race_date == "2026-04-12"));
}

#[test]
fn a_plan_may_be_adopted_before_a_race_is_typed() {
    let harness = Harness::empty("adopt-no-race");
    // No race, and no threshold: the plan is adopted anyway, and it says
    // that it has no paces yet.
    let (status, answer) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-half-marathon-tier-2", "race_date": "2027-01-24"}"#,
    );
    assert_eq!(status, 200, "the adoption was refused: {answer}");
    let plan = &answer["plan"];
    assert_eq!(plan["pacing"]["paced"], false);
    assert!(plan["pacing"]["threshold"].is_null());
    assert_eq!(
        plan["pacing"]["missing"],
        "no race time entered — the paces have no value yet"
    );

    // Its sessions carry their kinds and their distances, and no paces.
    let week = harness.ok_get("/plan/week?date=2026-12-30");
    let sessions = week_three(&week);
    assert_eq!(sessions[2]["kind"], "long run");
    assert_eq!(sessions[2]["distance"], "17.7 km");
    assert!(sessions[2]["pace"].is_null());
    assert_eq!(
        sessions[2]["no_pace"],
        "no race time entered — the paces have no value yet"
    );
}

#[test]
fn a_typed_threshold_is_the_threshold_the_plan_is_paced_with() {
    let harness = Harness::empty("adopt-typed");
    let (status, answer) = post(
        &harness,
        "/plan/adopt",
        r#"{"programme_id": "pfitzinger-10k-tier-1", "race_date": "2027-01-24", "goal_distance_m": 10000.0,
            "threshold": {"source": "typed", "heart_rate_bpm": 168, "label": "lab_test"}}"#,
    );
    assert_eq!(status, 200, "the adoption was refused: {answer}");
    let plan = &answer["plan"];
    assert_eq!(plan["pacing"]["threshold"]["value"], "168 bpm");
    assert_eq!(plan["pacing"]["threshold"]["source"], "typed");
    assert_eq!(plan["pacing"]["threshold"]["base"], "pfitzinger");
}

#[test]
fn the_threshold_in_force_is_the_one_the_plan_is_paced_with() {
    let harness = Harness::empty("adopt-in-force");
    harness.write_race(common::race());
    // The athlete's own choice, made before the adoption.
    harness.write_threshold_choice(&common::choice(), true);
    let (status, answer) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200, "the adoption was refused: {answer}");
    let threshold = &answer["plan"]["pacing"]["threshold"];
    assert_eq!(threshold["value"], "165 bpm");
    assert_eq!(threshold["source"], "watch_zone_model");
}

#[test]
fn the_athletes_own_mileage_is_beside_the_entry_requirement_and_never_against_it() {
    let harness = Harness::empty("adopt-mileage");
    athlete(&harness);
    // Four weeks of the athlete's own runs, well under the tier's entry
    // requirement — and the adoption is not refused for it.
    for (index, distance) in [10_000.0, 12_000.0, 8_000.0, 6_000.0].iter().enumerate() {
        harness.write_run(common::run(
            &format!("a-run-{index}"),
            &format!("2026-04-{:02}", 20 + index),
            *distance,
            3_000.0,
        ));
    }

    let (status, answer) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200, "the adoption was refused: {answer}");
    let entry = &answer["plan"]["entry"];
    assert!(entry["requirement"].is_string(), "{entry}");
    assert!(entry["athlete"].is_string(), "{entry}");
    // The requirement is not compared with the athlete's mileage: both are
    // stated, and neither is marked as the right one.
    let requirement = entry["requirement"].as_str().unwrap_or_default();
    let athlete_line = entry["athlete"].as_str().unwrap_or_default();
    assert!(!requirement.contains("below"), "{requirement}");
    assert!(!athlete_line.contains("below"), "{athlete_line}");
}

/// The week against the week that happened: **a fact, not a verdict**.
#[test]
fn the_plan_s_week_is_compared_with_the_week_that_happened() {
    let harness = Harness::empty("adopt-against");
    athlete(&harness);
    let (status, answer) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200, "the adoption was refused: {answer}");

    // The week of the twenty-eighth of December 2026: a run on the
    // Tuesday and a run on the Wednesday, and nothing on the other days.
    harness.write_run(common::run(
        "a-run-on-the-tuesday",
        "2026-12-29",
        14_500.0,
        4_785.0,
    ));
    harness.write_run(common::run(
        "a-run-on-the-wednesday",
        "2026-12-30",
        15_000.0,
        5_025.0,
    ));

    // The Friday of that week: two sessions done, one not done, and the
    // rest of the week still to come.
    let answer = harness.ok_get("/plan/week?date=2027-01-01");
    let against = &answer["against"];
    assert_eq!(against["state"], "running");
    assert_eq!(
        against["source_line"],
        "the Pfitzinger half marathon tier 2 plan"
    );
    assert_eq!(
        against["paced_with"],
        "paced with the threshold of 1 May 2026 · 165 bpm"
    );

    // The week's totals against the plan's: the days that have gone on
    // both sides.
    let totals = &against["totals"];
    assert_eq!(totals["line"], "29.5 km against the plan's 49.9 km");
    assert_eq!(totals["planned"]["text"], "49.9 km");
    assert_eq!(totals["actual"]["text"], "29.5 km");

    let sessions = week_three(&answer);
    // The rest day: neither done nor not done.
    assert!(sessions[0]["status"].is_null(), "{:?}", sessions[0]);
    assert!(sessions[0]["actual"].is_null(), "{:?}", sessions[0]);

    // The easy run: done, and the distance is what was run.
    assert_eq!(sessions[1]["status"], "done");
    assert_eq!(sessions[1]["actual"]["text"], "14.5 km");

    // The long run: done, short of the plan, and slower than the plan's
    // pace. **The pace is compared only because both of them exist.**
    assert_eq!(sessions[2]["status"], "done");
    assert_eq!(sessions[2]["actual"]["text"], "15 km");
    assert_eq!(
        sessions[2]["difference"],
        "2.7 km less than the 17.7 km target"
    );
    assert_eq!(
        sessions[2]["pace_comparison"],
        "5:35 /km · 3 s/km slower than the plan's 5:00–5:32 /km"
    );

    // The intervals: the day went by and the session was not run.
    assert_eq!(sessions[3]["status"], "not_done");
    assert!(sessions[3]["actual"].is_null(), "{:?}", sessions[3]);

    // The general aerobic day: the method fixes a band and states no
    // pace, so no pace is compared.
    assert!(
        sessions[1]["pace_comparison"].is_null(),
        "{:?}",
        sessions[1]
    );
    assert!(
        sessions[5]["pace_comparison"].is_null(),
        "{:?}",
        sessions[5]
    );
}

/// The four states: **each one says its own thing**.
#[test]
fn the_card_says_which_state_the_week_is_in() {
    let harness = Harness::empty("adopt-states");

    // No plan at all: the app says so, and points at the programmes there
    // are to adopt.
    let answer = harness.ok_get("/plan/week");
    let against = &answer["against"];
    assert_eq!(against["state"], "none");
    assert_eq!(against["state_line"], "no plan has been adopted");
    assert_eq!(against["choose"], "your plan is one of the 4 programmes");
    assert!(against["totals"].is_null(), "{against}");

    athlete(&harness);
    let (status, _) = post(&harness, "/plan/adopt", &body());
    assert_eq!(status, 200);

    // The plan has not started yet: the app says when it starts.
    let answer = harness.ok_get("/plan/week?date=2026-11-01");
    let against = &answer["against"];
    assert_eq!(against["state"], "before");
    assert_eq!(
        against["state_line"],
        "your plan starts on 2 Nov 2026 — this week comes before it"
    );

    // And the plan has passed: the app says when its last week ended.
    let answer = harness.ok_get("/plan/week?date=2027-02-08");
    let against = &answer["against"];
    assert_eq!(against["state"], "after");
    assert_eq!(
        against["state_line"],
        "your plan's last week ended on 7 Feb 2027 — this week comes after it"
    );
}
