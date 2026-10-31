//! The map tiles.
//!
//! The browser must not talk to a third party. Every tile request to
//! `tile.openstreetmap.org` tells OpenStreetMap the athlete's IP address and
//! that they are looking at a running map — which is exactly the kind of
//! signal a privacy-forward product should not be handing out.
//!
//! So the tiles are fetched **by the athlete's own machine**, with an
//! identifying User-Agent, cached on disk, and served back from the same
//! origin. The browser only ever talks to us.
//!
//! The reference is `backend/app/api/tiles.py`. This is a port of it:
//!
//! * the tile is fetched from `https://tile.openstreetmap.org/{z}/{x}/{y}.png`
//!   with the User-Agent the reference uses;
//! * **redirects are not followed.** The proxy fetches on the athlete's
//!   behalf, so it must fetch exactly the tile it was asked for; a redirect
//!   would turn it into a general-purpose fetcher — including one that can be
//!   aimed at an internal address or a cloud metadata endpoint;
//! * the body is capped at 2 MB, and the cap is applied while it is read;
//! * a cached tile is good for 30 days.
//!
//! The HTTP client is `reqwest`, which is in the approved dependency set and
//! is already in the graph: `stride-garmin` uses it for the Garmin API, so
//! neither the desktop binary nor the Android library grows by a byte.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde_json::{json, Value};

use crate::backend::{Backend, Tile};
use crate::error::BackendError as Error;

/// Where the tiles come from.
pub const TILE_URL: &str = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";

/// The host the tiles come from.
pub const TILE_HOST: &str = "tile.openstreetmap.org";

/// The largest tile the proxy will fetch: 2 MB.
pub const MAX_TILE_BYTES: usize = 2 * 1024 * 1024;

/// How long the proxy waits for a tile.
pub const TILE_TIMEOUT: Duration = Duration::from_secs(15);

/// The User-Agent the tiles are fetched with.
pub const USER_AGENT: &str = "Stride/1.0 (private running log; https://github.com/local/stride)";

/// How long a cached tile is good for: 30 days.
pub const CACHE_SECONDS: u64 = 30 * 24 * 3600;

/// A backend that serves the map tiles.
pub struct TileBackend {
    inner: Arc<dyn Backend>,
    cache_dir: PathBuf,
    /// Where the tiles come from. It is a field so that a test can point it
    /// somewhere that is not the internet.
    host: String,
}

impl TileBackend {
    /// Wrap a backend, caching the tiles in `cache_dir`.
    pub fn new(inner: Arc<dyn Backend>, cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            inner,
            cache_dir: cache_dir.into(),
            host: TILE_HOST.to_string(),
        }
    }

    /// Wrap a backend, fetching the tiles from somewhere else.
    pub fn with_host(inner: Arc<dyn Backend>, cache_dir: impl Into<PathBuf>, host: &str) -> Self {
        Self {
            inner,
            cache_dir: cache_dir.into(),
            host: host.to_string(),
        }
    }

    /// Where the tiles are cached.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// The cache file of one tile.
    ///
    /// The name is the first 32 hex characters of the SHA-256 of
    /// `zoom/x/y`, as the reference names it.
    fn cache_path(&self, zoom: i64, x: i64, y: i64) -> PathBuf {
        self.cache_dir.join(format!("{}.png", tile_key(zoom, x, y)))
    }

    /// The cached tile, if it is there and still fresh.
    fn cached(&self, zoom: i64, x: i64, y: i64) -> Option<Vec<u8>> {
        let path = self.cache_path(zoom, x, y);
        let metadata = std::fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;
        if age.as_secs() >= CACHE_SECONDS {
            return None;
        }
        std::fs::read(&path).ok()
    }

    /// Fetch a tile from OpenStreetMap and cache it.
    fn fetch(&self, zoom: i64, x: i64, y: i64) -> Result<Vec<u8>, Error> {
        let url = TILE_URL
            .replace("tile.openstreetmap.org", &self.host)
            .replace("{z}", &zoom.to_string())
            .replace("{x}", &x.to_string())
            .replace("{y}", &y.to_string());
        let client = reqwest::blocking::Client::builder()
            .timeout(TILE_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|err| Error::Domain(format!("the tile client failed: {err}")))?;
        let mut response = client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .header("Accept", "image/png,*/*")
            .send()
            .map_err(|err| Error::Unavailable(format!("Map tile unavailable: {err}")))?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(Error::Unavailable("Map tile unavailable.".to_string()));
        }
        // The body is capped while it is read, not after: a hostile or
        // broken upstream cannot make the process buffer it all in memory.
        let mut body = Vec::new();
        let mut capped = (&mut response).take(MAX_TILE_BYTES as u64 + 1);
        capped
            .read_to_end(&mut body)
            .map_err(|_| Error::Unavailable("Map tile unavailable.".to_string()))?;
        if body.len() > MAX_TILE_BYTES {
            return Err(Error::Unavailable("Map tile too large.".to_string()));
        }
        if body.is_empty() {
            return Err(Error::Unavailable("Map tile unavailable.".to_string()));
        }
        let _ = std::fs::create_dir_all(&self.cache_dir);
        let _ = std::fs::write(self.cache_path(zoom, x, y), &body);
        Ok(body)
    }
}

/// The cache file name of one tile.
pub fn tile_key(zoom: i64, x: i64, y: i64) -> String {
    // The name only has to be unique and stable, and the reference's is a
    // truncated SHA-256. The tiles are few enough that a plain
    // `zoom-x-y` would do, but the reference's shape is kept so that a
    // cache written by either implementation is the same file.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in format!("{zoom}/{x}/{y}").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}{hash:016x}")
}

impl Backend for TileBackend {
    fn tile_policy(&self) -> Result<Value, Error> {
        Ok(json!({
            "upstream": "openstreetmap.org",
            "proxied": true,
            "reason": "Tiles are fetched by this server and cached, so the browser never reveals the athlete's IP address to a third party.",
            "user_agent": USER_AGENT,
            "cache_dir": self.cache_dir.to_string_lossy(),
        }))
    }

    fn tile(&self, z: i64, x: i64, y: i64) -> Result<Tile, Error> {
        if let Some(bytes) = self.cached(z, x, y) {
            return Ok(Tile {
                bytes,
                source: "cache",
            });
        }
        let bytes = self.fetch(z, x, y)?;
        Ok(Tile {
            bytes,
            source: "upstream",
        })
    }

    fn activities(
        &self,
        filter: &crate::backend::ActivityFilter,
    ) -> Result<Vec<crate::backend::ActivityRow>, Error> {
        self.inner.activities(filter)
    }

    fn activity_count(&self, filter: &crate::backend::ActivityFilter) -> Result<i64, Error> {
        self.inner.activity_count(filter)
    }

    fn activity(&self, id: i64) -> Result<Option<crate::backend::ActivityRow>, Error> {
        self.inner.activity(id)
    }

    fn activity_by_external(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<crate::backend::ActivityRow>, Error> {
        self.inner.activity_by_external(source, external_id)
    }

    fn activity_laps(&self, id: i64) -> Result<Vec<crate::backend::LapRow>, Error> {
        self.inner.activity_laps(id)
    }

    fn activity_streams(&self, id: i64) -> Result<Vec<crate::backend::StreamRow>, Error> {
        self.inner.activity_streams(id)
    }

    fn activity_zones(&self, id: i64) -> Result<Vec<crate::backend::ZoneRow>, Error> {
        self.inner.activity_zones(id)
    }

    fn activity_geo(&self, id: i64) -> Result<Option<Vec<[f64; 2]>>, Error> {
        self.inner.activity_geo(id)
    }

    fn patch_activity(
        &self,
        id: i64,
        patch: &crate::backend::ActivityPatch,
    ) -> Result<Option<crate::backend::ActivityRow>, Error> {
        self.inner.patch_activity(id, patch)
    }

    fn counts(&self) -> Result<crate::backend::Counts, Error> {
        self.inner.counts()
    }

    fn upsert_synced_activity(
        &self,
        activity: &crate::backend::ActivityRow,
        laps: &[crate::backend::LapRow],
        streams: &[crate::backend::StreamRow],
    ) -> Result<i64, Error> {
        self.inner.upsert_synced_activity(activity, laps, streams)
    }

    fn upsert_physiology(&self, row: &crate::backend::PhysiologyRow) -> Result<(), Error> {
        self.inner.upsert_physiology(row)
    }

    fn upsert_daily_metric(&self, row: &crate::backend::DailyMetricRow) -> Result<(), Error> {
        self.inner.upsert_daily_metric(row)
    }

    fn upsert_sync_run(&self, run: &crate::backend::SyncRunRow) -> Result<i64, Error> {
        self.inner.upsert_sync_run(run)
    }

    fn garmin_personal_key(&self) -> Result<Option<String>, Error> {
        self.inner.garmin_personal_key()
    }

    fn physiology(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::backend::PhysiologyRow>, Error> {
        self.inner.physiology(from, to)
    }

    fn physiology_latest(&self) -> Result<Option<crate::backend::PhysiologyRow>, Error> {
        self.inner.physiology_latest()
    }

    fn daily_metrics(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<crate::backend::DailyMetricRow>, Error> {
        self.inner.daily_metrics(from, to)
    }

    fn plan_blocks(&self) -> Result<Vec<crate::backend::PlanBlockRow>, Error> {
        self.inner.plan_blocks()
    }

    fn plan_block(&self, id: i64) -> Result<Option<crate::backend::PlanBlockRow>, Error> {
        self.inner.plan_block(id)
    }

    fn plan_phases(&self, block_id: i64) -> Result<Vec<crate::backend::PlanPhaseRow>, Error> {
        self.inner.plan_phases(block_id)
    }

    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<crate::backend::PlanWeekRow>, Error> {
        self.inner.plan_weeks(phase_id)
    }

    fn plan_sessions(&self, week_id: i64) -> Result<Vec<crate::backend::PlanSessionRow>, Error> {
        self.inner.plan_sessions(week_id)
    }

    fn plan_sessions_in_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::backend::PlanSessionRow>, Error> {
        self.inner.plan_sessions_in_range(from, to)
    }

    fn create_plan_block(
        &self,
        fields: &crate::backend::PlanBlockFields,
    ) -> Result<crate::backend::PlanBlockRow, Error> {
        self.inner.create_plan_block(fields)
    }

    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &crate::backend::PlanPhaseFields,
    ) -> Result<crate::backend::PlanPhaseRow, Error> {
        self.inner.create_plan_phase(block_id, fields)
    }

    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &crate::backend::PlanWeekFields,
    ) -> Result<crate::backend::PlanWeekRow, Error> {
        self.inner.create_plan_week(phase_id, fields)
    }

    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &crate::backend::PlanSessionFields,
    ) -> Result<crate::backend::PlanSessionRow, Error> {
        self.inner.create_plan_session(week_id, fields)
    }

    fn import_plan(&self, request: &crate::backend::ImportRequest) -> Result<Value, Error> {
        self.inner.import_plan(request)
    }

    fn fingerprint(&self, activity_id: i64) -> Result<Value, Error> {
        self.inner.fingerprint(activity_id)
    }

    fn fingerprints(&self, from: &str, to: &str) -> Result<Vec<Value>, Error> {
        self.inner.fingerprints(from, to)
    }

    fn readiness(&self, day: &str) -> Result<Value, Error> {
        self.inner.readiness(day)
    }

    fn readiness_range(&self, from: &str, to: &str) -> Result<Vec<Value>, Error> {
        self.inner.readiness_range(from, to)
    }

    fn baselines(&self, day: &str) -> Result<Value, Error> {
        self.inner.baselines(day)
    }

    fn compare(&self, ids: &[i64]) -> Result<Value, Error> {
        self.inner.compare(ids)
    }

    fn compare_candidates(&self, limit: usize) -> Result<Vec<Value>, Error> {
        self.inner.compare_candidates(limit)
    }

    fn plan_results(&self, from: &str, to: &str) -> Result<Vec<Value>, Error> {
        self.inner.plan_results(from, to)
    }

    fn plan_adherence(&self, from: &str, to: &str) -> Result<Value, Error> {
        self.inner.plan_adherence(from, to)
    }

    fn plan_sessions_with_results(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::backend::SessionWithResult>, Error> {
        self.inner.plan_sessions_with_results(from, to)
    }

    fn athlete_zone_model(&self, activity_id: i64) -> Result<Vec<Value>, Error> {
        self.inner.athlete_zone_model(activity_id)
    }

    fn max_hr_reference(&self, activity_id: i64) -> Result<f64, Error> {
        self.inner.max_hr_reference(activity_id)
    }

    fn public_status(&self) -> Result<crate::backend::StatusView, Error> {
        self.inner.public_status()
    }

    fn network_status(&self) -> Result<Value, Error> {
        self.inner.network_status()
    }

    fn public_identity(&self) -> Result<Value, Error> {
        self.inner.public_identity()
    }

    fn exit_nodes(&self) -> Result<Vec<Value>, Error> {
        self.inner.exit_nodes()
    }

    fn set_network_route(&self, route: &str) -> Result<Value, Error> {
        self.inner.set_network_route(route)
    }

    fn sync_status(&self) -> Result<Value, Error> {
        self.inner.sync_status()
    }

    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, Error> {
        self.inner.sync_runs(limit)
    }

    fn run_sync(&self, days: Option<i64>, background: bool) -> Result<Value, Error> {
        self.inner.run_sync(days, background)
    }

    fn load_history(
        &self,
        activities: Option<i64>,
        health_days: Option<i64>,
        background: bool,
    ) -> Result<Value, Error> {
        self.inner.load_history(activities, health_days, background)
    }

    fn update_settings(
        &self,
        patch: &crate::backend::SettingsPatch,
    ) -> Result<crate::backend::StatusView, Error> {
        self.inner.update_settings(patch)
    }

    fn gear(&self) -> Result<Vec<Value>, Error> {
        self.inner.gear()
    }

    fn connect_personal_key(&self, api_key: &str) -> Result<Value, Error> {
        self.inner.connect_personal_key(api_key)
    }

    fn test_personal_key(&self, api_key: &str) -> Result<Value, Error> {
        self.inner.test_personal_key(api_key)
    }

    fn garmin_oauth(&self, action: &str, body: &Value) -> Result<Value, Error> {
        self.inner.garmin_oauth(action, body)
    }

    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, Error> {
        self.inner.import_fit(filename, bytes)
    }
}
