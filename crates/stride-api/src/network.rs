//! The network route.
//!
//! Garmin throttles datacentre addresses hard, so the app can route its
//! Garmin traffic through a residential exit node on the tailnet. That is a
//! property of the *machine*, not of the data: it is read from Tailscale and
//! set with Tailscale.
//!
//! The reference is `backend/app/network.py`. This is a port of it — the same
//! commands, the same parsing, the same answers — and it shells out to the
//! `tailscale` binary rather than adding a crate: the approved dependency set
//! has no Tailscale client in it, and the binary is already on the machine.
//!
//! The one thing the reference gets that this cannot is the *public
//! identity* — the machine's public IP and network operator — because that
//! needs an HTTPS call to a third party, and this crate has no HTTP client
//! in it on purpose (the Android build would carry a TLS stack). The
//! reference's own answer when the call fails is `{"ip": null, "org": null}`,
//! and that is what this answers; a host that wants the real thing
//! overrides [`Backend::public_identity`].

use std::sync::Arc;

use serde_json::{json, Value};

use crate::backend::{Backend, SettingsPatch};
use crate::error::BackendError as Error;

/// How long a Tailscale command may take.
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

    /// Run the Tailscale binary, if it is there.
    fn tailscale(args: &[&str]) -> Option<(i32, String)> {
        let output = std::process::Command::new("tailscale")
            .args(args)
            .output()
            .ok()?;
        Some((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).into_owned(),
        ))
    }

    /// The exit node the machine is using, and whether it is using one.
    fn exit_node(&self) -> (Option<String>, bool) {
        let Some((code, out)) = Self::tailscale(&["status", "--json"]) else {
            return (None, false);
        };
        if code != 0 {
            return (None, false);
        }
        let Ok(data) = serde_json::from_str::<Value>(&out) else {
            return (None, false);
        };
        let status = data.get("ExitNodeStatus");
        let mut address = None;
        let mut active = false;
        if let Some(status) = status {
            if let Some(addresses) = status.get("TailscaleIPs").and_then(Value::as_array) {
                if let Some(first) = addresses.first().and_then(Value::as_str) {
                    address = Some(first.split('/').next().unwrap_or(first).to_string());
                }
            }
            active = status
                .get("ID")
                .and_then(Value::as_str)
                .is_some_and(|id| !id.is_empty());
        }
        if address.is_none() {
            // Older Tailscale versions report the selected node on `Self`.
            if let Some(older) = data
                .get("Self")
                .and_then(|self_node| self_node.get("ExitNodeIP"))
                .and_then(Value::as_str)
            {
                address = Some(older.to_string());
                active = !older.is_empty();
            }
        }
        (address, active)
    }

    /// The exit nodes the tailnet offers.
    fn nodes(&self) -> Vec<Value> {
        let Some((code, out)) = Self::tailscale(&["exit-node", "list"]) else {
            return Vec::new();
        };
        if code != 0 {
            return Vec::new();
        }
        parse_exit_nodes(&out)
    }
}

/// The exit nodes in the table `tailscale exit-node list` prints.
///
/// The reference: `network.available_exit_nodes`.
pub fn parse_exit_nodes(output: &str) -> Vec<Value> {
    let mut nodes = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with("IP ") {
            continue;
        }
        if !line.starts_with(' ') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let ip = parts[0];
        let hostname = parts[1];
        let status = parts[parts.len() - 1].to_lowercase();
        nodes.push(json!({
            "id": hostname,
            "ip": ip,
            "hostname": hostname,
            "online": status != "offline" && status != "unavailable",
            "selected": status == "selected",
        }));
    }
    nodes
}

impl Backend for NetworkBackend {
    fn network_status(&self) -> Result<Value, Error> {
        let available = Self::tailscale(&["version"]).is_some();
        let (exit_node, active) = self.exit_node();
        Ok(json!({
            "tailscale_available": available,
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
        Ok(self.nodes())
    }

    fn set_network_route(&self, route: &str) -> Result<Value, Error> {
        let (ok, message) = match route {
            "direct" => {
                let (code, out) = Self::tailscale(&["set", "--exit-node=none"])
                    .unwrap_or((127, "tailscale is not installed".to_string()));
                (code == 0, out.trim().to_string())
            }
            "exit_node" => {
                // The pinned node, or the first one the tailnet offers.
                let pinned = self
                    .inner
                    .public_status()
                    .ok()
                    .and_then(|status| status.exit_node);
                let node = pinned.or_else(|| {
                    self.nodes()
                        .iter()
                        .find(|node| {
                            node.get("online").and_then(Value::as_bool).unwrap_or(false)
                                && node.get("ip").and_then(Value::as_str).is_some()
                        })
                        .and_then(|node| node.get("ip").and_then(Value::as_str).map(str::to_string))
                });
                let Some(node) = node else {
                    return Err(Error::Domain(
                        "No exit node is available on this tailnet.".to_string(),
                    ));
                };
                let (code, out) = Self::tailscale(&["set", &format!("--exit-node={node}")])
                    .unwrap_or((127, "tailscale is not installed".to_string()));
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

    /// The table `tailscale exit-node list` prints, as the reference reads it.
    const TABLE: &str = "\n IP                HOSTNAME                          COUNTRY     CITY      STATUS       \n 100.90.192.46     raspberrypi.tail62fa14.ts.net     -           -         selected     \n\n# To view the complete list of exit nodes for a country, use `tailscale exit-node list --filter=` followed by the country name.\n";

    /// The nodes are read out of the table.
    #[test]
    fn the_exit_nodes_are_read() {
        let nodes = parse_exit_nodes(TABLE);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["ip"], "100.90.192.46");
        assert_eq!(nodes[0]["hostname"], "raspberrypi.tail62fa14.ts.net");
        assert_eq!(nodes[0]["id"], "raspberrypi.tail62fa14.ts.net");
        assert_eq!(nodes[0]["online"], true);
        assert_eq!(nodes[0]["selected"], true);
    }

    /// A node that is offline is offline.
    #[test]
    fn an_offline_node_is_offline() {
        let table = " 100.1.2.3   node.tail.ts.net   -   -   offline\n";
        let nodes = parse_exit_nodes(table);
        assert_eq!(nodes[0]["online"], false);
        assert_eq!(nodes[0]["selected"], false);
    }
}
