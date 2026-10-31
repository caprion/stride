//! The phone's own tests.
//!
//! They exercise the path the phone takes: open the database with a key, ask
//! the router for a route, and check the answer is the shape the SPA reads.
//!
//! No fixture is invented here: the numbers come from the routes, and the
//! routes are the contract.

// The tests are allowed to expect: they are a separate crate, not a
// `#[cfg(test)]` module, so the workspace's lint configuration does not
// reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Mutex, MutexGuard, OnceLock};

use serde_json::Value;
use stride_store::model::{Activity, ActivityLap};
use stride_store::Store;

/// The lock the tests share.
///
/// The backend is process-wide — one phone, one database — so the tests take
/// turns rather than fight over it.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let mutex = LOCK.get_or_init(|| Mutex::new(()));
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// A temporary directory that goes away with the test.
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("stride-ffi-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the temporary directory could not be made");
        Self(path)
    }

    fn db(&self) -> String {
        self.0.join("stride.db").to_string_lossy().into_owned()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A 32-byte key, hex-encoded, the way the Keystore hands it over.
fn key_hex(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

/// A run, as the store keeps it.
fn a_run() -> Activity {
    Activity {
        source: "garmin_personal".to_string(),
        external_id: "run-1".to_string(),
        name: "Morning run".to_string(),
        sport_type: "running".to_string(),
        start_time: chrono::Utc::now(),
        local_date: "2026-03-02".to_string(),
        day_of_week: Some("Monday".to_string()),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 120.0,
        avg_hr: Some(150.0),
        max_hr: Some(172.0),
        calories: Some(700.0),
        avg_cadence: Some(170.0),
        ..Activity::default()
    }
}

#[test]
fn the_database_opens_and_the_router_answers() {
    let _guard = lock();
    let dir = TempDir::new("open");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(7));
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {answer}");
    assert_eq!(parsed["body"]["schema"], "migrated");

    let health = uniffi_stride::stride_call("GET".into(), "/api/v1/health".into(), None);
    let parsed: Value = serde_json::from_str(&health).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {health}");
    assert_eq!(parsed["status"], 200);
}

#[test]
fn a_wrong_key_is_an_error_and_not_a_crash() {
    let _guard = lock();
    let dir = TempDir::new("wrong-key");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(9));
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {answer}");

    // A different key cannot open the file that is already there.
    let other = TempDir::new("wrong-key-other");
    std::fs::copy(dir.db(), other.db()).expect("the database could not be copied");
    let answer = uniffi_stride::stride_open(other.db(), key_hex(8));
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], false, "the answer was {answer}");
    assert!(parsed["error"]["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("could not be opened"));
}

#[test]
fn the_routes_answer_from_the_phone_database() {
    let _guard = lock();
    let dir = TempDir::new("routes");
    let key = key_hex(3);
    let answer = uniffi_stride::stride_open(dir.db(), key.clone());
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // Two runs, written the way the sync writes them.
    {
        let store = Store::open(std::path::Path::new(&dir.db()), &[3u8; 32])
            .expect("the store could not be opened");
        let run = a_run();
        let id = store
            .upsert_activity(&run)
            .expect("the run was not written");
        let lap = ActivityLap {
            activity_id: id,
            lap_index: 1,
            distance_m: 1_000.0,
            duration_s: 300.0,
            avg_hr: Some(150.0),
            avg_cadence: Some(170.0),
            ..ActivityLap::default()
        };
        store
            .replace_activity_laps(id, &[lap])
            .expect("the lap was not written");
    }

    let list =
        uniffi_stride::stride_call("GET".into(), "/api/v1/activities?per_page=10".into(), None);
    let parsed: Value = serde_json::from_str(&list).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {list}");
    let items = parsed["body"]["items"].as_array().expect("there are items");
    assert_eq!(items.len(), 1, "the list was {list}");
    assert_eq!(items[0]["name"], "Morning run");
    assert_eq!(items[0]["distance_m"], 10_000.0);

    let detail = uniffi_stride::stride_call("GET".into(), "/api/v1/activities/1".into(), None);
    let parsed: Value = serde_json::from_str(&detail).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {detail}");
    assert_eq!(parsed["body"]["name"], "Morning run");
    assert_eq!(parsed["body"]["distance_m"], 10_000.0);
    // The splits come with the run.
    let laps = parsed["body"]["laps"].as_array().expect("there are laps");
    assert_eq!(laps.len(), 1, "the detail was {detail}");
    assert_eq!(laps[0]["distance_m"], 1_000.0);

    // The fingerprint is the domain's answer, computed on the phone.
    let fingerprint =
        uniffi_stride::stride_call("GET".into(), "/api/v1/fitness/fingerprint".into(), None);
    let parsed: Value = serde_json::from_str(&fingerprint).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {fingerprint}");
    let items = parsed["body"]["items"].as_array().expect("there are items");
    assert_eq!(items.len(), 1, "the fingerprint list was {fingerprint}");
    assert_eq!(items[0]["activity_id"], 1);
    assert_eq!(items[0]["distance_m"], 10_000.0);
}

#[test]
fn the_routes_the_phone_cannot_answer_say_so() {
    let _guard = lock();
    let dir = TempDir::new("unsupported");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(5));
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // The basemap: the phone ships no tiles, and says so.
    let policy = uniffi_stride::stride_call("GET".into(), "/api/v1/tiles/policy".into(), None);
    let parsed: Value = serde_json::from_str(&policy).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {policy}");
    assert_eq!(parsed["body"]["policy"], "none");

    // A tile itself is not there: the phone ships no basemap, and says so.
    let tile = uniffi_stride::stride_call("GET".into(), "/api/v1/tiles/3/4/5.png".into(), None);
    let parsed: Value = serde_json::from_str(&tile).expect("the answer is JSON");
    assert_eq!(parsed["ok"], false, "the answer was {tile}");
    assert_eq!(parsed["status"], 501);
    assert!(parsed["error"]["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("basemap"));
}

#[test]
fn a_pull_is_answered_the_way_the_spa_expects() {
    let _guard = lock();
    let dir = TempDir::new("pull");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(11));
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // A background pull is a job: the route acknowledges it and the phone's
    // scheduler does the work.
    let queued = uniffi_stride::stride_call(
        "POST".into(),
        "/api/v1/sync/run".into(),
        Some("{\"days\":30,\"background\":true}".to_string()),
    );
    let parsed: Value = serde_json::from_str(&queued).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {queued}");
    assert_eq!(parsed["body"]["started"], true, "the answer was {queued}");
    assert_eq!(parsed["body"]["reason"], "queued");

    // A pull the athlete is watching needs the key, and says so when there
    // is none.
    let inline = uniffi_stride::stride_call(
        "POST".into(),
        "/api/v1/sync/run".into(),
        Some("{\"days\":30,\"background\":false}".to_string()),
    );
    let parsed: Value = serde_json::from_str(&inline).expect("the answer is JSON");
    assert_eq!(parsed["ok"], false, "the answer was {inline}");
    assert_eq!(parsed["status"], 400);
    let detail = parsed["error"]["detail"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        detail.contains("no Garmin key") || detail.contains("not connected"),
        "the detail was {detail}"
    );
}

/// The path of a golden fixture, from this crate.
fn fixture(name: &str) -> std::path::PathBuf {
    // android/app/src/main/rust/stride-ffi -> the repository root
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..6 {
        path = path
            .parent()
            .expect("the crate has five parents")
            .to_path_buf();
    }
    path.join("backend/tests/golden/fixtures").join(name)
}

#[test]
fn a_fit_file_imported_the_way_the_bridge_sends_it_gives_the_golden_numbers() {
    let _guard = lock();
    let dir = TempDir::new("fit");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(13));
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // The body is the one the phone's bridge sends: the file name and its
    // bytes, base64-encoded.
    let bytes = std::fs::read(fixture("hr_gps_run.fit")).expect("the fixture is there");
    let body = serde_json::json!({
        "files": [{
            "filename": "hr_gps_run.fit",
            "content_base64": stride_api::util::base64_encode(&bytes),
        }]
    })
    .to_string();

    let answer = uniffi_stride::stride_call(
        "POST".into(),
        "/api/v1/auth/garmin/fit-import".into(),
        Some(body),
    );
    let parsed: Value = serde_json::from_str(&answer).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {answer}");
    assert_eq!(parsed["body"]["imported"], 1, "the answer was {answer}");
    assert_eq!(parsed["body"]["errors"].as_array().map(Vec::len), Some(0));

    // The run is in the phone's own database now, with the numbers the
    // golden file says.
    let detail = uniffi_stride::stride_call("GET".into(), "/api/v1/activities/1".into(), None);
    let parsed: Value = serde_json::from_str(&detail).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {detail}");
    let run = &parsed["body"];
    assert_eq!(run["name"], "hr gps run", "the run was {run}");
    assert_eq!(run["sport_type"], "running");
    assert_eq!(run["source"], "garmin_fit");
    assert_eq!(run["external_id"], "3117002");
    assert_eq!(run["local_date"], "2026-03-03");
    assert_eq!(run["day_of_week"], "Tuesday");
    assert_eq!(run["distance_m"], 1980.0);
    assert_eq!(run["moving_time_s"], 600.0);
    assert_eq!(run["elapsed_time_s"], 600.0);
    assert_eq!(run["avg_hr"], 145.0);
    assert_eq!(run["max_hr"], 150.0);
    assert_eq!(run["calories"], 640.0);
    // The cadence is the halved one, corrected: the file stores 40.
    assert_eq!(run["avg_cadence"], 80.0);
    assert_eq!(run["max_cadence"], 80.0);
    assert_eq!(run["running_time_s"], 599.0);
    assert_eq!(run["walking_time_s"], 0.0);
    assert_eq!(run["stopped_time_s"], 0.0);
    assert_eq!(run["running_distance_m"], 1976.7);
    assert_eq!(run["device"], "Forerunner 265");
}

#[test]
fn the_status_answers_before_anything_is_open() {
    let status = uniffi_stride::stride_status();
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["body"]["core"], "stride-ffi");
}

#[test]
fn the_app_can_see_that_garmin_is_connected() {
    let _guard = lock();
    let dir = TempDir::new("connected");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(17));
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // No key: not connected, and the app is told so.
    let status = uniffi_stride::stride_call("GET".into(), "/api/v1/auth/status".into(), None);
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(
        parsed["body"]["garmin_connected"], false,
        "the answer was {status}"
    );
    assert_eq!(
        parsed["body"]["garmin_method"],
        Value::Null,
        "the answer was {status}"
    );

    // The key is there: the app can see that Garmin is connected, and by
    // what. The phone never sends the key to the page.
    uniffi_stride::stride_set_garmin_key("a-personal-api-key".to_string());
    let status = uniffi_stride::stride_call("GET".into(), "/api/v1/auth/status".into(), None);
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(
        parsed["body"]["garmin_connected"], true,
        "the answer was {status}"
    );
    assert_eq!(
        parsed["body"]["garmin_method"], "personal",
        "the answer was {status}"
    );

    // The copy in the core's memory goes when the pull is over: the key is
    // in memory for the length of one pull and no longer.
    //
    // **That is not the same as disconnecting.** The key itself lives in the
    // Keystore, and the phone is still connected: the copy in the memory
    // was only ever there for the pull. So the connection stays, and the
    // test says so — a phone that said "not connected" after every pull
    // would be telling the athlete a lie.
    uniffi_stride::stride_set_garmin_key(String::new());
    let status = uniffi_stride::stride_call("GET".into(), "/api/v1/auth/status".into(), None);
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(
        parsed["body"]["garmin_connected"], true,
        "the answer was {status}"
    );
    assert_eq!(
        parsed["body"]["garmin_method"], "personal",
        "the answer was {status}"
    );

    // Disconnecting is a separate fact, and it is said separately.
    uniffi_stride::stride_set_garmin_connected(false);
    let status = uniffi_stride::stride_call("GET".into(), "/api/v1/auth/status".into(), None);
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(
        parsed["body"]["garmin_connected"], false,
        "the answer was {status}"
    );
    assert_eq!(
        parsed["body"]["garmin_method"],
        Value::Null,
        "the answer was {status}"
    );
}

#[test]
fn the_time_of_the_last_pull_survives_in_the_phone_database() {
    let _guard = lock();
    let dir = TempDir::new("last-sync");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(19));
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // Nothing has been pulled yet.
    let status = uniffi_stride::stride_call("GET".into(), "/api/v1/sync/status".into(), None);
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {status}");
    assert_eq!(
        parsed["body"]["last_sync_at"],
        Value::Null,
        "the answer was {status}"
    );

    // A pull that worked, written the way the phone's shell writes it.
    {
        let store = Store::open(std::path::Path::new(&dir.db()), &[19u8; 32])
            .expect("the store could not be opened");
        let now = chrono::Utc::now();
        store
            .upsert_sync_run(&stride_store::model::SyncRun {
                id: None,
                source: "garmin_personal".to_string(),
                started_at: now,
                finished_at: Some(now),
                status: "ok".to_string(),
                days: 365,
                activities_found: 160,
                activities_new: 160,
                activities_updated: 0,
                message: Some("160 runs, 160 new, 0 updated".to_string()),
                error: None,
            })
            .expect("the sync log was not written");
    }

    // The app reads the time of the last pull out of the phone's own
    // database, not out of the memory the key was in.
    let status = uniffi_stride::stride_call("GET".into(), "/api/v1/sync/status".into(), None);
    let parsed: Value = serde_json::from_str(&status).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {status}");
    assert!(
        parsed["body"]["last_sync_at"].is_string(),
        "the answer was {status}"
    );

    // And the log itself is there for the app to read.
    let runs = uniffi_stride::stride_call("GET".into(), "/api/v1/sync/runs".into(), None);
    let parsed: Value = serde_json::from_str(&runs).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {runs}");
    let items = parsed["body"]["items"].as_array().expect("there are items");
    assert_eq!(items.len(), 1, "the log was {runs}");
    assert_eq!(items[0]["activities_found"], 160, "the log was {runs}");
    assert_eq!(items[0]["status"], "ok", "the log was {runs}");
}

/// The four screens the phone has: Today, Runs, Progress, Body.
///
/// They read four routes, and all four are answered by the phone's own
/// database — there is no server behind them. This test writes three
/// synthetic runs and a month of synthetic recovery rows into that
/// database, and then asks the core for them the way the WebView does.
///
/// It is the phone's end-to-end check: the data the screens draw comes out
/// of the phone, not out of a fixture the WebView was handed.
#[test]
fn the_phone_answers_the_four_screens_from_its_own_database() {
    let _guard = lock();
    let dir = TempDir::new("screens");
    let answer = uniffi_stride::stride_open(dir.db(), key_hex(23));
    assert!(answer.contains("\"ok\":true"), "the answer was {answer}");

    // The core counts the days from UTC, so the test does too.
    let today = chrono::Utc::now().date_naive();
    let day = |offset: i64| {
        (today - chrono::Duration::days(offset))
            .format("%Y-%m-%d")
            .to_string()
    };

    {
        let store = Store::open(std::path::Path::new(&dir.db()), &[23u8; 32])
            .expect("the store could not be opened");

        // Three runs: yesterday, three days ago and five days ago.
        for (offset, distance) in [(1i64, 10_000.0), (3, 12_000.0), (5, 8_000.0)] {
            let mut run = a_run();
            run.external_id = format!("run-{offset}");
            run.local_date = day(offset);
            run.start_time = chrono::Utc::now() - chrono::Duration::days(offset);
            run.distance_m = distance;
            store
                .upsert_activity(&run)
                .expect("the run was not written");
        }

        // A month of nights, so last night has a baseline to be read
        // against: seven readings are the least a baseline can be.
        for offset in 0..31 {
            let night = stride_store::model::DailyPhysiology {
                date: day(offset),
                sleep_s: Some(27_000.0),
                hrv_ms: Some(60.0),
                resting_hr: Some(48.0),
                ..Default::default()
            };
            store
                .upsert_physiology(&night)
                .expect("the night was not written");
        }
    }

    // ------------------------------------------------------------ Today
    let today_call =
        uniffi_stride::stride_call("GET".into(), "/api/v1/insights/today".into(), None);
    let parsed: Value = serde_json::from_str(&today_call).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {today_call}");
    let body = &parsed["body"];

    // One call: a headline, a state the screen knows, and no more.
    assert!(
        body["headline"].as_str().unwrap_or_default().len() > 10,
        "the headline was {today_call}"
    );
    assert!(
        ["go", "easy", "done", "nudge", "comeback"]
            .iter()
            .any(|state| state == &body["state"].as_str().unwrap_or_default()),
        "the state was {today_call}"
    );
    // The last run is the one from yesterday.
    assert_eq!(body["last_run"]["run"]["distance_m"], 10_000.0);
    // The week: seven days, Monday to Sunday, and the runs in them.
    // Which of the three runs falls inside this week depends on the day the
    // test runs, so the count is not fixed — but there is always at least
    // one, and there are always seven days.
    let days = body["week"]["days"].as_array().expect("there are days");
    assert_eq!(days.len(), 7, "the week was {today_call}");
    assert!(
        body["week"]["runs"].as_u64().unwrap_or_default() >= 1,
        "the week was {today_call}"
    );
    // The guardrail: the longest run in 30 days, plus ten per cent.
    assert_eq!(body["longest_30d_m"], 12_000.0, "the call was {today_call}");
    // The ten per cent is a multiplication, so it is compared with a
    // tolerance: 12 km * 1.1 is 13200.000000000002.
    let spike = body["spike_limit_m"].as_f64().unwrap_or_default();
    assert!(
        (spike - 13_200.0).abs() < 0.001,
        "the call was {today_call}"
    );
    // The guardrail is not on every card: it speaks when the distance the
    // call is about is near the line. This athlete's usual run is 10 km
    // against a 13.2 km line, so there is nothing to warn about and the
    // card says nothing.
    assert!(
        body["guardrail"].is_null(),
        "the guardrail was {today_call}"
    );
    // Body's headline comes from here: the signals and their normals, read
    // by the core so Today and Body cannot drift apart.
    let signals = body["recovery_signals"]
        .as_array()
        .expect("there are signals");
    assert_eq!(signals.len(), 3, "the signals were {today_call}");
    for signal in signals {
        assert!(
            ["sleep", "hrv", "resting_hr"].contains(&signal["key"].as_str().unwrap_or_default()),
            "the signal was {signal}"
        );
        assert_eq!(signal["normal"], true, "the signal was {signal}");
    }
    assert_eq!(body["recovery_normal"][0], 3, "the call was {today_call}");
    assert_eq!(body["recovery_normal"][1], 3, "the call was {today_call}");

    // ---------------------------------------------------------- Progress
    let progress_call =
        uniffi_stride::stride_call("GET".into(), "/api/v1/insights/progress".into(), None);
    let parsed: Value = serde_json::from_str(&progress_call).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {progress_call}");
    let body = &parsed["body"];
    assert_eq!(
        body["milestones"]["total_runs"], 3,
        "the progress was {progress_call}"
    );
    assert_eq!(
        body["milestones"]["total_m"], 30_000.0,
        "the progress was {progress_call}"
    );
    // The bar: two runs a week, or one for an athlete whose usual is one.
    // Which of the two depends on the weeks the test's runs fall in, so
    // both are right and nothing else is.
    let per_week = body["consistency"]["runs_per_week"]
        .as_u64()
        .unwrap_or_default();
    assert!(
        per_week == 1 || per_week == 2,
        "the progress was {progress_call}"
    );
    assert!(
        !body["consistency"]["weeks"]
            .as_array()
            .expect("there are weeks")
            .is_empty(),
        "the progress was {progress_call}"
    );
    assert!(
        !body["bests"]
            .as_array()
            .expect("there are bests")
            .is_empty(),
        "the progress was {progress_call}"
    );
    assert!(
        body["race_estimates"].is_array(),
        "the progress was {progress_call}"
    );

    // -------------------------------------------------------------- Runs
    let runs =
        uniffi_stride::stride_call("GET".into(), "/api/v1/activities?per_page=10".into(), None);
    let parsed: Value = serde_json::from_str(&runs).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {runs}");
    let items = parsed["body"]["items"].as_array().expect("there are items");
    assert_eq!(items.len(), 3, "the list was {runs}");

    // One chip per run, so the Runs screen has something to say.
    let chips = uniffi_stride::stride_call(
        "GET".into(),
        "/api/v1/insights/chips?days=3650".into(),
        None,
    );
    let parsed: Value = serde_json::from_str(&chips).expect("the answer is JSON");
    assert_eq!(parsed["ok"], true, "the answer was {chips}");
    assert!(
        !parsed["body"]["items"]
            .as_object()
            .expect("there are chips")
            .is_empty(),
        "the chips were {chips}"
    );
}
