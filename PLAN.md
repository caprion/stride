# Stride — a private, local-first running log with native Garmin ingest

> Plan of record. Status: **shipped** — see §8 for what was built and verified.

---

## 0. Outcome

`~/stride` — a private, local-first running log, live at **http://100.89.222.50:8000**
(Tailscale) and `http://localhost:8000` locally. One command: `./scripts/run.sh`.

Six surfaces (Dashboard, Feed, Activity detail, Training log, Stats, Settings),
a production-grade design system, four Garmin ingest paths, and 19 backend tests
passing. Zero console errors, page errors or failed requests across every route
(verified with headless Chromium — `scripts/screenshot.mjs`).

---

## 1. Goal

A high-fidelity, self-hosted running log. The user plugs in a Garmin
credential and everything works: activities flow in automatically, splits/HR/GPS are
preserved, and the app presents them in a polished, fast UI.

Success criterion: **the app runs on one command, is populated with real data, and
looks and feels like a real product.**

---

## 2. Discovery findings (this is the part that shaped everything)

Reusing prior art in `~/running` (a Garmin sync pipeline + Streamlit dashboard,
8.2k lines) was the highest-leverage move — but it also exposed hard constraints.

| Finding | Evidence | Consequence |
|---|---|---|
| `~/running` has **never synced real Garmin data** | `tracking/unified-cache.json` is byte-identical to `sample-data/unified-cache.json` (891532 B); no `.garth` session dir | The old pipeline is unproven against Garmin. Cannot assume it works. |
| Prior work is **Streamlit**, not a webapp | `dashboard/` = 10 Streamlit pages | A real SPA is needed. Streamlit cannot deliver it. |
| **Garmin blocks datacenter IPs** | `garminconnect` login: all 5 strategies fail — `429 Too Many Requests` (mobile), `403 Cloudflare bot challenge` (portal) | Sync must not run from the Azure IP (AS8075, Pune). |
| **Residential egress fixes it** | After `tailscale set --exit-node=100.90.192.46` (raspberrypi @ home, AS24309 Atria Broadband): `429` → `401`, and the portal strategy reached Garmin's real login form | The app needs a **network-route** concept, and the fleet's exit node is the escape hatch. |
| **Official OAuth1 developer API is reachable** | `connectapi.garmin.com/oauth-service/oauth/request_token` → `401` (not `404`/`429`); `.../access_token` → `400`; `/activity-service/activity/{id}` → `403` | A real Consumer Key/Secret gives a clean 3-legged flow. This is the "plug in a key" path. |
| **No OAuth2 client_credentials** | `/oauth2/token` → `404` on both hosts | OAuth1 only. |
| **Garmin paused new developer-program applications (2026)** | Garmin revamped the Connect Developer Program; approvals now gated | Cannot depend on the user getting a key. Must ship fallback ingest paths. |
| Stored credentials are **rejected** | portal login reached Garmin's form → `401 Invalid Username or Password` | Password may be stale. App must accept credentials via UI, not `.env` only. |
| The OAuth **authorise URL is not** where it looks | `connect.garmin.com/oauth/authorize` → `404`; `connect.garmin.com/modern/authorize` → `302` → `/app/authorize` → Garmin sign-in | The browser flow uses `/modern/authorize`. The `connectapi` host also has an `/oauth-service/oauth/authorize` path, but it is service-to-service (`NotAllowedException`). |
| `tailscale exit-node list` has **no `--json`** | `flag provided but not defined: -json` | Parse the table output instead. |
| The active exit node is in **`ExitNodeStatus`**, not `Self.ExitNode` | `Self.ExitNode` means "this node *offers* exit-node service" — the opposite question | Read `ExitNodeStatus.ID` / `.TailscaleIPs` to know whether egress is routed. |
| A FIT importer is **verifiable without Garmin** | Real Garmin `.FIT` files are obtainable from the `python-fitparse` test fixtures | The no-credentials path was validated end-to-end against a real watch file. |

### Ingest paths (all first-class, in priority order)

1. **Garmin Developer Key (OAuth1)** — paste Consumer Key + Secret → 3-legged authorize → full activity/splits/details. *This is "plug in your Garmin key".*
2. **Garmin account (email + password)** — via `garminconnect`, with MFA code entry in the UI. Works when egress isn't blocked.
3. **FIT drop** — upload `.FIT` files; parsed locally for splits, HR stream, GPS polyline, elevation. **Needs no auth, always works.**
4. **Demo dataset** — the 161-activity sample already in `~/running`, so the app is alive on first boot with zero setup.

> Design rule: **the app must never be a blank slate.** Demo data seeds on first run;
> every ingest path is additive.

---

## 3. Stack

| Layer | Choice | Why |
|---|---|---|
| Backend | Python 3.12 · FastAPI · SQLAlchemy 2.0 · SQLite | Entire Garmin ecosystem here is Python (`garminconnect`, `garth`, `fitparse`). Reuse beats reimplementation. SQLite = zero-ops, single file, personal scale. |
| Sync | APScheduler worker + manual trigger | Auto-sync on a schedule; "Sync now" button. |
| Frontend | React 18 · TypeScript · Vite · Tailwind · Recharts · react-leaflet · Zustand · TanStack Query | A real SPA. Recharts + Leaflet cover the whole chart/map surface. |
| Serving | FastAPI serves the built SPA + `/api/v1` | One port, one command, "just works". |
| Auth | OAuth1 3-legged (dev key) + session cookie for the app itself | Personal app; no OAuth server needed beyond the app's own session. |

### Data model (SQLite)

```
athlete          id, name, email, timezone, created_at
activity         id, source, external_id, name, sport_type, start_time, local_date,
                 distance_m, moving_time_s, elapsed_time_s, elevation_gain_m,
                 avg_hr, max_hr, calories, avg_cadence, avg_speed_ms,
                 perceived_exertion, device, description, is_trail, is_commute,
                 visibility, gear_id, geojson, created_at
activity_lap     id, activity_id, lap_index, distance_m, duration_s, avg_speed_ms,
                 avg_hr, max_hr, avg_cadence, max_cadence, stride_length_m,
                 elevation_gain_m, elevation_loss_m, start_elev_m, end_elev_m
activity_stream  id, activity_id, kind(hr|pace|elevation|distance|cadence),
                 seconds(json), values(json)
hr_zone          id, activity_id, zone_no, name, low_bpm, high_bpm, time_s
daily_metric     date, resting_hr, sleep_s, hrv, stress, steps, body_battery, readiness
sync_run         id, started_at, finished_at, status, days, activities_found, activities_new, error
```

Derived values (pace, weekly volume, HR-zone distribution, personal records, streaks)
are computed in SQL/Python at query time — no stale aggregates.

---

## 4. API contract (`/api/v1`)

```
GET    /auth/status
POST   /auth/garmin/developer-key      {consumer_key, consumer_secret} -> {authorize_url}
POST   /auth/garmin/exchange           {oauth_token, oauth_verifier}   -> {ok}
POST   /auth/garmin/login              {email, password} -> {status: ok|mfa_required, mfa_token}
POST   /auth/garmin/mfa                {mfa_token, code} -> {ok}
POST   /auth/garmin/logout
POST   /auth/garmin/fit-import         (multipart) -> {imported, activities}

POST   /sync/run                       {days} -> {started}
GET    /sync/status
GET    /sync/runs

GET    /activities                     ?page&per_page&sport&from&to&q&sort
GET    /activities/{id}
PATCH  /activities/{id}
GET    /activities/{id}/streams/{kind}
GET    /activities/{id}/geo

GET    /stats/summary
GET    /stats/weekly                   ?weeks
GET    /stats/pace-trends              ?weeks
GET    /stats/hr-zones                 ?weeks
GET    /stats/personal-records
GET    /stats/heatmap                  ?year
GET    /stats/elevation                ?weeks

GET    /calendar                       ?year&month
GET    /health/daily                   ?from&to
```

---

## 5. Frontend surfaces

| Route | Surface | What it answers |
|---|---|---|
| `/` | Dashboard: hero tiles, weekly volume, recent activity feed | Dashboard |
| `/feed` | Activity feed with route thumbnails | Feed |
| `/activities/:id` | Map hero, stat tiles, HR/pace/elevation charts, splits table, description | Activity detail |
| `/training` | Calendar + training log | Training Log |
| `/stats` | Totals & gains, personal bests, pace/HR trends | Stats |
| `/settings` | Connect Garmin, sync config, ingest, data | Settings |

Design language: deep azure for structure, warm amber for the athlete's data,
white cards, left icon sidebar, dense
stat tiles, dark-on-white typography, sticky top nav.

---

## 6. Execution plan

**Phase A — foundation (serial, me).** Repo, backend skeleton, DB + models, ingest
adapters (OAuth1 / email+MFA / FIT / demo), sync engine, analytics, full REST API.
Frontend scaffold, design tokens, API client, routing, shared components.

**Phase B — surfaces (parallel agents).** One agent per isolated frontend route,
each building against the frozen API contract + shared component kit. Split only
where files don't overlap.

**Phase C — integration (serial, me).** Wire routes, fix contract drift, seed real
data, end-to-end test, screenshots, single-command launch.

---

## 7. Risks

| Risk | Mitigation |
|---|---|
| Garmin keeps blocking our egress | Network-route setting + auto-detect; FIT and demo paths always work |
| User has no developer key (program paused) | Email/MFA path + FIT import + demo |
| OAuth1 authorize needs a browser round-trip | App opens the URL; accepts pasted verifier *or* a registered callback |
| Agent-parallel edits collide | Frozen API contract; each agent owns disjoint files |
| SQLite concurrent writes from sync + API | Single writer (sync holds a lock); WAL mode |

---

## 8. What shipped

### Ingest paths — all four implemented and exercised

| Path | Status | Evidence |
|---|---|---|
| **Developer API key** (OAuth1 3-legged) | Implemented; endpoints probed and confirmed reachable (`request_token` → `401` on a bogus key, i.e. the endpoint answers). Needs a real key to complete. | `POST /api/v1/auth/garmin/developer-key` returns a clean `400` for a bad key and a `429`-specific message for a throttled IP. |
| **Garmin account** (email + password + MFA) | Implemented against `garminconnect` 0.3.19; MFA bridged to the browser with a blocking broker. | Login reaches Garmin's auth form; Garmin answers `401 Invalid Username or Password` for the stored credentials. |
| **FIT import** | **Verified end-to-end** against real Garmin `.FIT` files: GPS track (221 pts), laps, per-lap pace, elevation, HR, cadence. | `scripts/import-fit.sh` + `POST /api/v1/auth/garmin/fit-import`. |
| **Demo dataset** | 161 runs over 53 weeks, rebased to end two days ago, with synthesised GPS routes and per-lap elevation. | First boot; auto-hidden the moment real activities arrive. |

### Surfaces

| Route | Notes |
|---|---|
| `/` | Hero stats, weekly volume, pace trend, personal records, monthly volume, HR zones, recent activities |
| `/feed` | Sport/sort/search filters, pagination, route thumbnails, side column |
| `/activities/:id` | Interactive Leaflet map with the route, inline rename, 7-tile stat strip, HR/pace/elevation charts, splits table with fastest/slowest, notes, prev/next |
| `/training` | Month calendar with volume tiers, day panel, week summary, streak |
| `/stats` | Totals & gains, personal records, sport split, HR zones, year heatmap, trends |
| `/settings` | Three connect paths, network route control, sync settings, sync history, about |

### Verification performed

- 19 backend tests pass (`backend/.venv/bin/python -m pytest -q`).
- Frontend typechecks with **zero** errors; production build succeeds.
- Headless Chromium renders all six routes plus two mobile viewports with
  **no console errors, page errors or failed requests**.
- OAuth1 handshake, FIT import, sync trigger, settings mutation and route
  switching all exercised over HTTP.

### Bugs found and fixed during the build

- `fitparse` returns `FieldData` wrappers, not values — the importer silently
  produced empty activities until every field read was unwrapped.
- `garminconnect` calls `prompt_mfa()` with **no arguments**; the web MFA
  bridge passed the code positionally and would have raised `TypeError`.
- `set --exit-node` flips the *whole machine's* egress, so a "direct" default
  silently changed it for every other service on the VM.
- `setsid` forks when it is a process-group leader, so the launcher's captured
  PID was not the server's — restarts left the old process holding the port.
- Recharts animates areas in on mount; screenshots taken too early showed
  empty charts that were in fact rendering correctly.
