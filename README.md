# Stride

A Strava-class running webapp that pulls your data straight off a Garmin watch.

Plug in a Garmin credential and everything works: activities sync automatically
with full splits, heart rate, GPS tracks and elevation. No cloud account, no
subscription, your data stays on your machine.

```
┌────────────┐        ┌──────────────────┐        ┌──────────────────┐
│ Garmin     │  OAuth │  Stride backend  │  SQL  │  Stride frontend │
│ Connect    │───────▶│  FastAPI+SQLite  │───────▶│  React SPA       │
│ (watch)    │  FIT   │  ingest + stats  │        │  (port 8000)     │
└────────────┘  file  └──────────────────┘        └──────────────────┘
```

---

## Quick start

```bash
cd ~/stride
./scripts/run.sh            # builds the SPA if needed, starts the app
```

Then open **http://localhost:8000**.

The app seeds itself with a full year of demo data (161 runs with splits, heart
rate, elevation and synthesised GPS routes) so it is never a blank slate. Connect
Garmin in **Settings → Garmin connection** and the real data takes over.

| Command | What it does |
|---|---|
| `./scripts/run.sh` | Build (if needed) and serve on `:8000` |
| `./scripts/run.sh dev` | API on `:8000` with reload + Vite HMR on `:5173` |
| `./scripts/run.sh stop` | Stop everything |
| `./scripts/run.sh status` | Show what is running |

API docs live at **http://localhost:8000/api/v1/docs**.

---

## Connecting your Garmin

There are three ways in. All three are supported; none is required to run the app.

### 1. Developer API key (recommended)

Garmin's official OAuth1 Activity API. Requires a **Consumer Key** and
**Consumer Secret** from [developer.garmin.com](https://developer.garmin.com).

1. In the app: **Settings → Garmin connection → Developer API key**.
2. Paste the key and secret, click **Connect with key**.
3. The app opens Garmin's authorisation page. Sign in and approve.
4. Garmin redirects back to the app and the connection is live.

> Garmin paused new applications to the Connect Developer Program in 2026, so
> you may not be able to register a new app. If you already have a key, this is
> the best path — it never touches your password.

### 2. Your Garmin account

Email and password, with an inline MFA step when Garmin demands a 6-digit code.
Uses the `garminconnect` library under the hood and stores a refreshable session
token so you only do this once.

### 3. FIT files (no credentials at all)

Export `.FIT` files from Garmin Connect (an activity → **⋯** → **Download FIT**)
and drop them into the app. The importer decodes the full file: GPS track,
per-second heart rate, cadence, elevation and laps.

```bash
# or from the command line
curl -X POST http://localhost:8000/api/v1/auth/garmin/fit-import \
     -F "files=@activity.fit"
```

---

## The network problem (and why there is a network setting)

Garmin throttles **datacentre IPs** hard. From this VM (Azure, AS8075) every
login attempt returns `429 Too Many Requests` and the web-login path is blocked
by a Cloudflare bot challenge. The same request from a residential IP succeeds.

So Stride can route its Garmin traffic through the **raspberrypi at home**,
which advertises itself as a Tailscale exit node. This is what the *Network
route* control in Settings switches.

| Route | Egress | Use when |
|---|---|---|
| `direct` | Azure (datacentre) | Garmin may rate-limit or block |
| `exit_node` | Home broadband (residential) | Reliable Garmin access |

The app also detects a `429`/bot-challenge mid-sync and retries once through the
exit node automatically.

> The exit-node switch changes the whole machine's egress, not just Stride's
> traffic — it is a machine-level Tailscale setting. Switch it back to `direct`
> when you are done if other traffic needs Azure egress.

---

## Architecture

```
stride/
├── backend/                     FastAPI + SQLAlchemy + SQLite
│   ├── app/
│   │   ├── main.py              app entrypoint, SPA hosting, scheduler
│   │   ├── config.py            env config + persisted runtime state
│   │   ├── db.py                engine, session, WAL pragmas
│   │   ├── models.py            ORM schema
│   │   ├── network.py           Tailscale exit-node management
│   │   ├── api/                 auth · activities · stats · calendar · sync
│   │   ├── ingest/              garmin_oauth · garmin_account · fit_importer · demo
│   │   └── services/            persistence · analytics · sync
│   └── data/                    stride.db + state.json (gitignored)
├── frontend/                    React 18 + TS + Vite + Tailwind + Recharts + Leaflet
│   ├── src/components/          design system: ui · charts · ActivityCard · RouteThumb
│   ├── src/lib/                 api client + formatters
│   ├── src/pages/               Dashboard · Feed · Activity · Training · Stats · Settings
│   └── CONTRACT.md              the frozen API/design contract
├── scripts/run.sh               one-command launcher
└── PLAN.md                      scope, discovery findings, risks
```

### Data model

`activity` is the canonical row. Every ingest path normalises into the same
shape before it touches the database, so nothing downstream branches on source.

```
athlete          id, name, email, timezone
activity         id, source, external_id, name, sport_type, start_time, local_date,
                 distance_m, moving_time_s, elapsed_time_s, elevation_gain_m,
                 avg_hr, max_hr, calories, avg_cadence, avg_speed_ms,
                 perceived_exertion, device, description, gear, is_trail,
                 is_commute, visibility, geo_points, raw
activity_lap     activity_id, lap_index, distance_m, duration_s, avg_speed_ms,
                 avg_hr, max_hr, avg_cadence, stride_length_m, elevation_*
activity_stream  activity_id, kind(hr|pace|elevation|distance|cadence), seconds, values
hr_zone          activity_id, zone_no, name, low_bpm, high_bpm, time_s
daily_metric     date, resting_hr, sleep_s, hrv_ms, stress, steps, body_battery, readiness
sync_run         started_at, finished_at, status, days, activities_found/new/updated, error
```

`unique(source, external_id)` makes re-syncing idempotent — Garmin lets you
rename an activity or add a description, and the next sync updates it in place.

Derived numbers (pace, weekly volume, HR-zone distribution, personal records,
streaks) are computed on read, so there are no stale aggregates.

---

## API

Base URL `/api/v1`. Full OpenAPI docs at `/api/v1/docs`.

| Method | Path | Purpose |
|---|---|---|
| GET | `/auth/status` | Garmin connection + network route |
| POST | `/auth/garmin/developer-key` | Start the OAuth1 handshake → authorise URL |
| GET | `/auth/garmin/callback` | OAuth1 redirect target |
| POST | `/auth/garmin/exchange` | Complete the handshake with the verifier |
| POST | `/auth/garmin/login` | Account login (may return `mfa_required`) |
| POST | `/auth/garmin/mfa` | Submit the MFA code |
| POST | `/auth/garmin/fit-import` | Upload `.FIT` files |
| POST | `/sync/run` | Trigger a sync |
| GET | `/sync/status` · `/sync/runs` | Sync state and history |
| PATCH | `/sync/settings` | Sync days, interval, auto-sync, route |
| GET | `/activities` | Paged, filterable, sortable |
| GET | `/activities/{id}` | Full detail: laps, streams, zones, route |
| PATCH | `/activities/{id}` | Rename, describe, rate effort |
| GET | `/activities/{id}/geo` · `/streams/{kind}` | Track and series |
| GET | `/stats/summary` | Totals & gains, PRs, streaks, sport split |
| GET | `/stats/weekly` · `/monthly` · `/pace-trends` · `/hr-zones` · `/personal-records` · `/heatmap` | Analytics |
| GET | `/calendar?year=&month=` | Calendar with per-day activities |
| GET | `/network/status` · POST `/network/route` | Egress diagnostics and control |

---

## Configuration

Environment variables (all optional):

| Variable | Default | Purpose |
|---|---|---|
| `STRIDE_PORT` | `8000` | Port the app serves on |
| `STRIDE_DATA_DIR` | `backend/data` | SQLite + state location |
| `STRIDE_PUBLIC_URL` | `http://localhost:8000` | OAuth callback base URL |
| `STRIDE_EXIT_NODE` | — | Tailscale node used as the exit node |
| `STRIDE_DEMO_CACHE` | `~/running/sample-data/unified-cache.json` | Demo dataset |

Garmin credentials, OAuth tokens and sync settings are stored in
`backend/data/state.json` (mode `600`, gitignored). Nothing leaves the machine.

---

## Testing the ingest paths

```bash
cd backend
.venv/bin/python -m pytest              # if you add tests
```

Manual checks:

```bash
# FIT import, end to end
curl -X POST http://localhost:8000/api/v1/auth/garmin/fit-import -F "files=@run.fit"

# Developer-key handshake (fails cleanly without a real key)
curl -X POST http://localhost:8000/api/v1/auth/garmin/developer-key \
     -H 'Content-Type: application/json' \
     -d '{"consumer_key":"...","consumer_secret":"..."}'
```

---

## Licence

MIT.
