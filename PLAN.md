# Stride — a Strava-class running webapp with native Garmin ingest

> Plan of record. Status: **executing**.

---

## 1. Goal

A high-fidelity, self-hosted Strava clone for running. The user plugs in a Garmin
credential and everything works: activities flow in automatically, splits/HR/GPS are
preserved, and the app presents them in a Strava-grade UI.

Success criterion: **the app runs on one command, is populated with real data, and
looks like Strava.**

---

## 2. Discovery findings (this is the part that shaped everything)

Reusing prior art in `~/running` (a Garmin sync pipeline + Streamlit dashboard,
8.2k lines) was the highest-leverage move — but it also exposed hard constraints.

| Finding | Evidence | Consequence |
|---|---|---|
| `~/running` has **never synced real Garmin data** | `tracking/unified-cache.json` is byte-identical to `sample-data/unified-cache.json` (891532 B); no `.garth` session dir | The old pipeline is unproven against Garmin. Cannot assume it works. |
| Prior work is **Streamlit**, not a webapp | `dashboard/` = 10 Streamlit pages | Strava fidelity needs a real SPA. Streamlit cannot deliver it. |
| **Garmin blocks datacenter IPs** | `garminconnect` login: all 5 strategies fail — `429 Too Many Requests` (mobile), `403 Cloudflare bot challenge` (portal) | Sync must not run from the Azure IP (AS8075, Pune). |
| **Residential egress fixes it** | After `tailscale set --exit-node=100.90.192.46` (raspberrypi @ home, AS24309 Atria Broadband): `429` → `401`, and the portal strategy reached Garmin's real login form | The app needs a **network-route** concept, and the fleet's exit node is the escape hatch. |
| **Official OAuth1 developer API is reachable** | `connectapi.garmin.com/oauth-service/oauth/request_token` → `401` (not `404`/`429`); `.../access_token` → `400`; `/activity-service/activity/{id}` → `403` | A real Consumer Key/Secret gives a clean 3-legged flow. This is the "plug in a key" path. |
| **No OAuth2 client_credentials** | `/oauth2/token` → `404` on both hosts | OAuth1 only. |
| **Garmin paused new developer-program applications (2026)** | Garmin revamped the Connect Developer Program; approvals now gated | Cannot depend on the user getting a key. Must ship fallback ingest paths. |
| Stored credentials are **rejected** | portal login reached Garmin's form → `401 Invalid Username or Password` | Password may be stale. App must accept credentials via UI, not `.env` only. |

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
| Frontend | React 18 · TypeScript · Vite · Tailwind · Recharts · react-leaflet · Zustand · TanStack Query | Strava-grade SPA. Recharts + Leaflet cover the whole chart/map surface. |
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

## 5. Frontend surfaces (Strava parity)

| Route | Surface | Strava analogue |
|---|---|---|
| `/` | Dashboard: hero tiles, weekly volume, recent activity feed | Dashboard |
| `/feed` | Activity feed with route thumbnails | Feed |
| `/activities/:id` | Map hero, stat tiles, HR/pace/elevation charts, splits table, description | Activity detail |
| `/training` | Calendar + training log | Training Log |
| `/stats` | Totals & gains, personal bests, pace/HR trends | Stats |
| `/settings` | Connect Garmin, sync config, ingest, data | Settings |

Design language: Strava orange `#fc5200`, white cards, left icon sidebar, dense
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
