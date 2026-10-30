//! The network route, and the trust configuration.
//!
//! ## The route is the deployment's business
//!
//! Garmin throttles datacentre addresses hard, so the app can route its
//! Garmin traffic through an exit node on another machine. **Which tool
//! performs that switch is the operator's business, not the application's.**
//! The app asks for an executable named by `STRIDE_EXIT_NODE_TOOL` and speaks
//! a small JSON protocol to it:
//!
//! ```text
//! <tool> status --json   {"exit_node": "<address>|null", "active": true|false}
//! <tool> nodes --json    [{"ip": "...", "hostname": "...", "online": true, "selected": false}]
//! <tool> set --exit-node=<address|none>
//! ```
//!
//! No networking product is named here, and **none is detected**: no address
//! is discovered at startup and no command is run to work out where the
//! network is. With no tool configured the feature is simply unavailable, and
//! the app says so rather than guessing.
//!
//! The reference is `backend/app/network.py`; this is a port of it — the same
//! protocol, the same parsing, the same answers.
//!
//! The one thing the reference gets that this cannot is the *public
//! identity* — the machine's public IP and network operator — because that
//! needs an HTTPS call to a third party, and this crate has no HTTP client
//! in it on purpose (the Android build would carry a TLS stack). The
//! reference's own answer when the call fails is `{"ip": null, "org": null}`,
//! and that is what this answers; a host that wants the real thing
//! overrides [`Backend::public_identity`].
//!
//! ## The trust configuration
//!
//! Whether a client from another network may use the API without a pairing
//! code is a *deployment* decision, and it is read from the environment:
//!
//! | variable | meaning | default |
//! |---|---|---|
//! | `STRIDE_TRUST_REMOTE` | clients from the trusted networks are trusted | `false` |
//! | `STRIDE_TRUSTED_NETWORKS` | the CIDRs those clients come from | empty |
//!
//! An empty list means **nothing extra is trusted**: only the loopback
//! interface is. There is no hard-coded range anywhere — the range is the
//! operator's to write down.

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::backend::{Backend, Caller, SettingsPatch};
use crate::error::BackendError as Error;

/// The environment variable that names the exit-node tool.
pub const EXIT_NODE_TOOL_ENV: &str = "STRIDE_EXIT_NODE_TOOL";

/// The name the tool is looked up under on `PATH` when the environment does
/// not name one. A deployment provides it; the repository does not.
pub const DEFAULT_TOOL_NAME: &str = "stride-exit-node";

/// The environment variable that turns the trust of remote clients on.
pub const TRUST_REMOTE_ENV: &str = "STRIDE_TRUST_REMOTE";

/// The environment variable that lists the trusted networks.
pub const TRUSTED_NETWORKS_ENV: &str = "STRIDE_TRUSTED_NETWORKS";

/// How long the tool may take.
const COMMAND_TIMEOUT_SECONDS: u64 = 20;

/// A backend that knows about the machine's network route.
pub struct NetworkBackend {
    inner: Arc<dyn Backend>,
}

impl NetworkBackend {
    /// Wrap a backend.
    pub fn new(inner: Arc<dyn Backend>) -> Self {
        Self { inner }
    }

    /// The backend underneath.
    pub fn inner(&self) -> &Arc<dyn Backend> {
        &self.inner
    }
}

/// Is this caller on this machine?
///
/// Both have to say so: the address the connection came from **and** the
/// name the caller used. The peer address alone is not enough — a page the
/// athlete happens to have open can point a name it controls at 127.0.0.1
/// and then talk to Stride as the origin it came from, and the `Host`
/// header is what tells the two apart.
pub fn caller_is_local(caller: &Caller) -> bool {
    let peer = caller.peer.is_some_and(|peer| peer.is_loopback());
    let host = caller.host.as_deref().is_some_and(host_is_loopback);
    peer && host
}

/// Does the `Host` header name a loopback host?
pub fn host_is_loopback(host: &str) -> bool {
    let host = host.trim();
    if host.is_empty() {
        return false;
    }
    let name = if let Some(rest) = host.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else if host.matches(':').count() > 1 {
        host
    } else {
        host.split(':').next().unwrap_or("")
    };
    if name.is_empty() {
        return false;
    }
    if name.eq_ignore_ascii_case("localhost") {
        return true;
    }
    name.parse::<IpAddr>()
        .is_ok_and(|address| address.is_loopback())
}

/// The executable that performs the switch, if one is configured.
///
/// The environment names it, or it is looked up on `PATH` under
/// [`DEFAULT_TOOL_NAME`]. A name with a path separator in it is a path; a
/// bare name is looked up in `PATH`.
pub fn tool_path() -> Option<PathBuf> {
    let configured = std::env::var(EXIT_NODE_TOOL_ENV)
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
        .unwrap_or_else(|| DEFAULT_TOOL_NAME.to_string());
    if configured.contains('/') {
        let candidate = Path::new(&configured);
        return if candidate.is_file() {
            Some(candidate.to_path_buf())
        } else {
            None
        };
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join(&configured))
        .find(|candidate| candidate.is_file())
}

/// Is the tool there?
pub fn tool_available() -> bool {
    tool_path().is_some()
}

/// Run the tool, and answer with its exit code and its output.
///
/// The output is stdout and stderr together, the way the reference reads it.
/// A tool that takes too long is killed: the app must not hang on a route
/// change.
fn run_tool(args: &[&str]) -> Option<(i32, String)> {
    let tool = tool_path()?;
    let mut child = std::process::Command::new(tool)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .ok()?;
    // The pipes are drained on their own threads: a tool that writes more
    // than a pipe buffer holds would otherwise block and never exit.
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let out = std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = stdout {
            use std::io::Read;
            let _ = pipe.read_to_string(&mut text);
        }
        text
    });
    let err = std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = stderr {
            use std::io::Read;
            let _ = pipe.read_to_string(&mut text);
        }
        text
    });
    let deadline = Instant::now() + Duration::from_secs(COMMAND_TIMEOUT_SECONDS);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break None,
        }
    };
    let stdout = out.join().unwrap_or_default();
    let stderr = err.join().unwrap_or_default();
    let code = status.and_then(|status| status.code()).unwrap_or(-1);
    Some((code, format!("{stdout}{stderr}")))
}

/// The exit node the machine is using, and whether it is using one.
fn tool_status() -> (Option<String>, bool) {
    let Some((code, out)) = run_tool(&["status", "--json"]) else {
        return (None, false);
    };
    if code != 0 {
        return (None, false);
    }
    let Ok(data) = serde_json::from_str::<Value>(&out) else {
        return (None, false);
    };
    let address = data
        .get("exit_node")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|address| !address.is_empty())
        .map(str::to_string);
    let active = data.get("active").and_then(Value::as_bool).unwrap_or(false);
    (address, active)
}

/// The exit nodes the tool advertises.
///
/// The reference: `network.available_exit_nodes`.
pub fn parse_exit_nodes(output: &str) -> Vec<Value> {
    let Ok(data) = serde_json::from_str::<Value>(output) else {
        return Vec::new();
    };
    let Some(entries) = data.as_array() else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|entry| {
            let address = entry.get("ip").and_then(Value::as_str).unwrap_or_default();
            if address.is_empty() {
                return None;
            }
            let hostname = entry
                .get("hostname")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Some(json!({
                "id": if hostname.is_empty() { address } else { hostname },
                "ip": address,
                "hostname": hostname,
                "online": entry.get("online").and_then(Value::as_bool).unwrap_or(true),
                "selected": entry.get("selected").and_then(Value::as_bool).unwrap_or(false),
            }))
        })
        .collect()
}

/// The exit nodes the tool advertises.
fn tool_nodes() -> Vec<Value> {
    let Some((code, out)) = run_tool(&["nodes", "--json"]) else {
        return Vec::new();
    };
    if code != 0 {
        return Vec::new();
    }
    parse_exit_nodes(&out)
}

/// A network, as an address and a prefix length.
///
/// The reference: `ipaddress.ip_network`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cidr {
    /// The network address, with the host bits cleared.
    address: IpAddr,
    /// How many bits of the address are the network.
    prefix: u8,
}

impl Cidr {
    /// Parse a CIDR, or a bare address.
    ///
    /// A bare address is a single-host network. The host bits of a CIDR are
    /// cleared, the way `ip_network(..., strict=False)` does it.
    pub fn parse(text: &str) -> Result<Cidr, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("an empty network".to_string());
        }
        let (address, prefix) = match text.split_once('/') {
            Some((address, prefix)) => {
                let address: IpAddr = address
                    .trim()
                    .parse()
                    .map_err(|_| format!("{text} is not an address"))?;
                let prefix: u8 = prefix
                    .trim()
                    .parse()
                    .map_err(|_| format!("{text} has no prefix length"))?;
                let widest = if address.is_ipv4() { 32 } else { 128 };
                if prefix > widest {
                    return Err(format!("{text} has a prefix length above {widest}"));
                }
                (address, prefix)
            }
            None => {
                let address: IpAddr = text
                    .parse()
                    .map_err(|_| format!("{text} is not an address"))?;
                let widest = if address.is_ipv4() { 32 } else { 128 };
                (address, widest)
            }
        };
        Ok(Cidr {
            address: masked(address, prefix),
            prefix,
        })
    }

    /// The network address.
    pub fn address(&self) -> IpAddr {
        self.address
    }

    /// The prefix length.
    pub fn prefix(&self) -> u8 {
        self.prefix
    }

    /// Is this address inside the network?
    pub fn contains(&self, address: &IpAddr) -> bool {
        match (self.address, address) {
            (IpAddr::V4(network), IpAddr::V4(candidate)) => {
                let mask = if self.prefix == 0 {
                    0
                } else {
                    u32::MAX << (32 - self.prefix)
                };
                (u32::from(network) & mask) == (u32::from(*candidate) & mask)
            }
            (IpAddr::V6(network), IpAddr::V6(candidate)) => {
                let mask = if self.prefix == 0 {
                    0
                } else {
                    u128::MAX << (128 - self.prefix)
                };
                (u128::from(network) & mask) == (u128::from(*candidate) & mask)
            }
            // An IPv4 address is not inside an IPv6 network, and the other way
            // round. There is no mapping between them here.
            _ => false,
        }
    }
}

/// Clear the host bits of an address.
fn masked(address: IpAddr, prefix: u8) -> IpAddr {
    match address {
        IpAddr::V4(value) => {
            let bits = u32::from(value);
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            IpAddr::V4(std::net::Ipv4Addr::from(bits & mask))
        }
        IpAddr::V6(value) => {
            let bits = u128::from(value);
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            IpAddr::V6(std::net::Ipv6Addr::from(bits & mask))
        }
    }
}

/// The trust configuration: who may use the API without a pairing code.
///
/// The trade-off is real and worth stating plainly: trusting a network means
/// *every device on it can read the athlete's health data*. That is a far
/// smaller group than "the whole internet" and a far larger group than "this
/// device". It is off by default, because a stranger who clones this
/// repository should not find it trusting their network.
#[derive(Debug, Clone, Default)]
pub struct TrustConfig {
    /// Whether clients from the trusted networks are trusted at all.
    pub remote: bool,
    /// The networks those clients come from. Empty means loopback only.
    pub networks: Vec<Cidr>,
}

impl TrustConfig {
    /// The trust configuration, from the environment.
    pub fn from_env() -> Self {
        let remote = env_flag(TRUST_REMOTE_ENV);
        let raw = std::env::var(TRUSTED_NETWORKS_ENV).unwrap_or_default();
        let mut networks = Vec::new();
        for item in raw.split(',') {
            let text = item.trim();
            if text.is_empty() {
                continue;
            }
            match Cidr::parse(text) {
                Ok(network) => networks.push(network),
                // A typo must not widen the trust boundary, and it must not
                // stop the process from starting either.
                Err(reason) => {
                    eprintln!("stride: ignoring {text:?} in {TRUSTED_NETWORKS_ENV}: {reason}")
                }
            }
        }
        TrustConfig { remote, networks }
    }

    /// Is this address in one of the trusted networks?
    ///
    /// The list is empty by default, so this is false for every address.
    pub fn trusts(&self, address: &IpAddr) -> bool {
        self.remote
            && self
                .networks
                .iter()
                .any(|network| network.contains(address))
    }

    /// Is nothing trusted at all?
    pub fn is_empty(&self) -> bool {
        !self.remote && self.networks.is_empty()
    }
}

/// Is a name set to a true value?
fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

impl Backend for NetworkBackend {
    fn network_status(&self) -> Result<Value, Error> {
        let available = tool_available();
        let (exit_node, active) = tool_status();
        Ok(json!({
            "exit_node_tooling_available": available,
            "exit_node": exit_node,
            "exit_node_active": active,
            "public_ip": Value::Null,
            "public_org": Value::Null,
            // The *actual* route, as opposed to the configured preference.
            "route": if active { "exit_node" } else { "direct" },
        }))
    }

    fn public_identity(&self) -> Result<Value, Error> {
        // The reference's own answer when it cannot reach the identity
        // service: the machine's public address is a diagnostic, not a
        // requirement, and this crate makes no network calls of its own.
        Ok(json!({ "ip": Value::Null, "org": Value::Null }))
    }

    fn exit_nodes(&self) -> Result<Vec<Value>, Error> {
        Ok(tool_nodes())
    }

    fn set_network_route(&self, route: &str) -> Result<Value, Error> {
        let missing = format!(
            "no exit-node tool found ({EXIT_NODE_TOOL_ENV} or {DEFAULT_TOOL_NAME} on PATH)"
        );
        let (ok, message) = match route {
            "direct" => {
                let (code, out) = run_tool(&["set", "--exit-node=none"]).unwrap_or((127, missing));
                (code == 0, out.trim().to_string())
            }
            "exit_node" => {
                // The pinned node, or the first one the tool advertises.
                let pinned = self
                    .inner
                    .public_status()
                    .ok()
                    .and_then(|status| status.exit_node);
                let node = pinned.or_else(|| {
                    tool_nodes()
                        .iter()
                        .find(|node| {
                            node.get("online").and_then(Value::as_bool).unwrap_or(false)
                                && node.get("ip").and_then(Value::as_str).is_some()
                        })
                        .and_then(|node| node.get("ip").and_then(Value::as_str).map(str::to_string))
                });
                let Some(node) = node else {
                    return Err(Error::Domain("No exit node is available.".to_string()));
                };
                let (code, out) =
                    run_tool(&["set", &format!("--exit-node={node}")]).unwrap_or((127, missing));
                (code == 0, out.trim().to_string())
            }
            other => {
                return Err(Error::Domain(format!(
                    "Route must be 'direct' or 'exit_node', not '{other}'."
                )))
            }
        };
        if ok {
            // The preference follows the machine.
            self.inner.update_settings(&SettingsPatch {
                network_route: Some(route.to_string()),
                ..SettingsPatch::default()
            })?;
        }
        let status = self.network_status()?;
        Ok(json!({ "ok": ok, "message": message, "status": status }))
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

    fn update_settings(&self, patch: &SettingsPatch) -> Result<crate::backend::StatusView, Error> {
        self.inner.update_settings(patch)
    }

    fn gear(&self) -> Result<Vec<Value>, Error> {
        self.inner.gear()
    }

    fn pair_code(&self) -> Result<Value, Error> {
        self.inner.pair_code()
    }

    fn may_mint_pairing_code(&self, caller: &Caller) -> bool {
        self.inner.may_mint_pairing_code(caller)
    }

    fn pair(&self, code: &str) -> Result<Option<Value>, Error> {
        self.inner.pair(code)
    }

    fn devices(&self) -> Result<Vec<Value>, Error> {
        self.inner.devices()
    }

    fn revoke_device(&self, device_id: &str) -> Result<Value, Error> {
        self.inner.revoke_device(device_id)
    }

    fn connect_personal_key(&self, api_key: &str) -> Result<Value, Error> {
        self.inner.connect_personal_key(api_key)
    }

    fn test_personal_key(&self, api_key: &str) -> Result<Value, Error> {
        self.inner.test_personal_key(api_key)
    }

    fn tile_policy(&self) -> Result<Value, Error> {
        self.inner.tile_policy()
    }

    fn tile(&self, z: i64, x: i64, y: i64) -> Result<crate::backend::Tile, Error> {
        self.inner.tile(z, x, y)
    }

    fn garmin_oauth(&self, action: &str, body: &Value) -> Result<Value, Error> {
        self.inner.garmin_oauth(action, body)
    }

    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, Error> {
        self.inner.import_fit(filename, bytes)
    }
}

/// The time a Tailscale command may take.
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(COMMAND_TIMEOUT_SECONDS);

#[cfg(test)]
mod tests {
    use super::*;

    /// The nodes the tool advertises, in the protocol the reference speaks.
    const NODES: &str =
        r#"[{"ip":"100.90.192.46","hostname":"raspberrypi","online":true,"selected":true}]"#;

    /// The nodes are read out of the tool's answer.
    #[test]
    fn the_exit_nodes_are_read() {
        let nodes = parse_exit_nodes(NODES);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["ip"], "100.90.192.46");
        assert_eq!(nodes[0]["hostname"], "raspberrypi");
        assert_eq!(nodes[0]["id"], "raspberrypi");
        assert_eq!(nodes[0]["online"], true);
        assert_eq!(nodes[0]["selected"], true);
    }

    /// A node that is offline is offline.
    #[test]
    fn an_offline_node_is_offline() {
        let nodes = parse_exit_nodes(
            r#"[{"ip":"100.1.2.3","hostname":"node","online":false,"selected":false}]"#,
        );
        assert_eq!(nodes[0]["online"], false);
        assert_eq!(nodes[0]["selected"], false);
    }

    /// An answer that is not the protocol is no nodes at all.
    #[test]
    fn an_answer_that_is_not_the_protocol_is_no_nodes() {
        assert!(parse_exit_nodes("not json").is_empty());
        assert!(parse_exit_nodes(r#"{"exit_node":"100.1.2.3"}"#).is_empty());
        assert!(parse_exit_nodes(r#"[{"hostname":"node"}]"#).is_empty());
    }

    /// A node with no hostname is named after its address.
    #[test]
    fn a_node_with_no_hostname_is_named_after_its_address() {
        let nodes = parse_exit_nodes(r#"[{"ip":"100.1.2.3"}]"#);
        assert_eq!(nodes[0]["id"], "100.1.2.3");
        assert_eq!(nodes[0]["online"], true, "online defaults to true");
        assert_eq!(nodes[0]["selected"], false);
    }

    /// A network is parsed, and the host bits are cleared.
    #[test]
    fn a_network_is_parsed() {
        let network = Cidr::parse("100.64.12.0/24").expect("the network");
        assert_eq!(network.prefix(), 24);
        assert_eq!(network.address().to_string(), "100.64.12.0");
        assert!(network.contains(&"100.64.12.7".parse().unwrap_or(network.address())));
        assert!(!network.contains(&"100.64.13.7".parse().unwrap_or(network.address())));
    }

    /// A host address is a network of one.
    #[test]
    fn a_bare_address_is_a_single_host_network() {
        let network = Cidr::parse("192.0.2.10").expect("the network");
        assert_eq!(network.prefix(), 32);
        assert!(network.contains(&"192.0.2.10".parse().unwrap_or(network.address())));
        assert!(!network.contains(&"192.0.2.11".parse().unwrap_or(network.address())));
    }

    /// An address that is not one is refused.
    #[test]
    fn an_address_that_is_not_one_is_refused() {
        assert!(Cidr::parse("not an address").is_err());
        assert!(Cidr::parse("192.0.2.0/40").is_err());
        assert!(Cidr::parse("").is_err());
    }

    /// An IPv6 network is a network.
    #[test]
    fn an_ipv6_network_is_a_network() {
        let network = Cidr::parse("fd00::/8").expect("the network");
        assert_eq!(network.prefix(), 8);
        assert!(network.contains(&"fd00::1".parse().unwrap_or(network.address())));
        assert!(!network.contains(&"192.0.2.1".parse().unwrap_or(network.address())));
    }

    /// Nothing is trusted until the operator says so.
    #[test]
    fn nothing_is_trusted_by_default() {
        let trust = TrustConfig::default();
        assert!(trust.is_empty());
        assert!(!trust.trusts(
            &"100.64.12.7"
                .parse()
                .unwrap_or(IpAddr::from([127, 0, 0, 1]))
        ));
    }

    /// A configured network is trusted, and another one is not.
    #[test]
    fn a_configured_network_is_trusted() {
        let trust = TrustConfig {
            remote: true,
            networks: vec![Cidr::parse("100.64.0.0/10").expect("the network")],
        };
        let inside: IpAddr = "100.89.222.50"
            .parse()
            .unwrap_or(IpAddr::from([127, 0, 0, 1]));
        let outside: IpAddr = "192.0.2.7".parse().unwrap_or(IpAddr::from([127, 0, 0, 1]));
        assert!(trust.trusts(&inside));
        assert!(!trust.trusts(&outside));
    }

    /// The networks are there but the trust is off: nothing is trusted.
    #[test]
    fn the_networks_alone_trust_nothing() {
        let trust = TrustConfig {
            remote: false,
            networks: vec![Cidr::parse("100.64.0.0/10").expect("the network")],
        };
        let inside: IpAddr = "100.89.222.50"
            .parse()
            .unwrap_or(IpAddr::from([127, 0, 0, 1]));
        assert!(!trust.trusts(&inside));
    }
}
