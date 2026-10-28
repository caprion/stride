# Stride

> **Most running apps tell you what you did. Stride tells you whether that was
> the right thing to do — and what to do tomorrow.**

A private, local-first running log with native Garmin ingest, built around one loop:

**Plan → Execute → Understand → Adapt.**

The plan exists as a first-class object. Every session gets a verdict *and* a
cause. The athlete's physiology and running mechanics are modelled. The app
tells you what to do today.

A social running app is a **social activity log**: it has no concept of intent, no physiology
and no causality. That is the gap this fills.

```
┌────────────┐        ┌──────────────────┐        ┌──────────────────┐
│ Garmin     │  OAuth │  Stride backend  │  SQL  │  Stride frontend │
│ Connect    │───────▶│  FastAPI+SQLite  │───────▶│  React SPA       │
│ (watch)    │  FIT   │  ingest + stats  │        │  (port 8000)     │
└────────────┘  file  └──────────────────┘        └──────────────────┘
```

---

## Showcase

A one-page collage of every surface: **[`showcase/stride-collage.png`](file:///home/sumit/stride/showcase/stride-collage.png)**.

## Design documents

| Document | What it decides |
|---|---|
| [`DESIGN.md`](file:///home/sumit/stride/DESIGN.md) | the product: plan-vs-actual, physiology, fingerprint, comparison |
| [`docs/android-v1.md`](file:///home/sumit/stride/docs/android-v1.md) | **the small, secure, Garmin-only Android app (target < 10 MB)** |
| [`docs/android-app.md`](file:///home/sumit/stride/docs/android-app.md) | the full Android architecture and threat model |
| [`docs/garmin-on-device.md`](file:///home/sumit/stride/docs/garmin-on-device.md) | key entry, Keystore storage, pull cadence, error copy |
| [`docs/harness-integration.md`](file:///home/sumit/stride/docs/harness-integration.md) | MCP sidecar so your own Claude Code / Codex can query your data |
| [`docs/working-agreement.md`](file:///home/sumit/stride/docs/working-agreement.md) | how we run big changes with parallel agents |
| [`docs/ai-sign-in-discussion.md`](file:///home/sumit/stride/docs/ai-sign-in-discussion.md) | whether the Android app should offer "Sign in with ChatGPT" — and why the answer dissolves |
| [`docs/piping-to-chatgpt.md`](file:///home/sumit/stride/docs/piping-to-chatgpt.md) | how to pipe the data through ChatGPT — MCP, and the four ways it can run |
| [`docs/android-packaging.md`](file:///home/sumit/stride/docs/android-packaging.md) | **how the Android app is packaged, signed, built in CI and deployed** |
| [`docs/production-deployment.md`](file:///home/sumit/stride/docs/production-deployment.md) | **the production instance: the vault key, the setup, the pre-flight** |

## No social layer, on purpose

There is no feed, no followers, no comments and no sharing to a network — and
there is not going to be one. A social layer would mean the data stopped being
yours alone. **Sharing is Garmin's business**: every activity links back to its
original in Garmin Connect, and Garmin decides who may open it. Stride never
publishes, never mirrors and never asks for an account.

## Protecting the athlete's data

The history in `backend/data/` is the only copy of a year of training. Two
mechanisms guard it.

**Snapshots.**

```bash
./scripts/snapshot.sh              # take one (verified, checksummed)
./scripts/snapshot.sh --list
./scripts/snapshot.sh --restore <name>
```

A snapshot is the database (WAL-checkpointed first, so it is a consistent
set), the state file with the Garmin credential, and a manifest with SHA-256
checksums. The snapshot is written `0600` and its database is integrity-checked
after the copy — a snapshot that cannot be restored is worse than none. A
restore takes a safety snapshot of the current state first.

**Parallel instances, on their own port and their own database.**

```bash
./scripts/instance.sh next 8010 --clone   # a copy of the current data
./scripts/instance.sh --list
./scripts/instance.sh --stop next
```

Each instance gets `backend/instances/<name>/data` as its own `STRIDE_DATA_DIR`,
so a new iteration **cannot** touch the live history. The Vite proxy target is
configurable (`VITE_API_TARGET`), so the two versions can run side by side —
`localhost:8010` for the API, `localhost:8011` for its frontend — and the
instance can be deleted afterwards without a trace.

The live instance is the one on **:8000**. Leave it alone.

## The differentiator

| | A social running app | Stride |
|---|---|---|
| **Plan** | none — "Training Log" is a calendar of the past | block → phase → week → session, with distance and pace targets |
| **Completion** | n/a | every session gets a verdict: `hit` · `missed_pace` · `short` · `missed` · `substituted` · `bonus` |
| **Why** | n/a | causes from sleep, HRV, resting HR, cadence stability and load — with a confidence |
| **The athlete** | one run at a time | readiness score with visible drivers; a running fingerprint (cadence stability, aerobic decoupling, efficiency, stride length) |
| **Tomorrow** | n/a | one actionable recommendation for today |

> "Thu tempo: planned 9 km @ 5:40–5:45, ran 9.0 km @ 6:02 (−17 s/km). Night
> before: sleep 5 h 40 m (baseline 7 h 32 m), HRV 34 ms (baseline 48 ms,
> −29 %), RHR 63 bpm (baseline 59 bpm). → **Fatigue-driven fade, not a fitness
> regression.**"

See [`DESIGN.md`](file:///home/sumit/stride/DESIGN.md) for the full design.

---

## Quick start

```bash
cd ~/stride
./scripts/run.sh            # builds the SPA if needed, starts the app
```

Then open **http://localhost:8000**.

On this VM it is already running as a managed bb terminal (`Stride API`), so it
is reachable over Tailscale at **http://100.89.222.50:8000** from any device on
the tailnet.

The live instance is the **production** one: 160 real Garmin activities, 371
days of physiology, the real training plan, and no synthetic data. It gets its
Garmin Personal API Key from Azure Key Vault, so the key is never written to
disk and never reaches the browser.

> The demo dataset is now **opt-in** (`STRIDE_SEED_DEMO=1`) and off by
> default. A production database holds only what Garmin actually returned.

| Command | What it does |
|---|---|
| `./scripts/prod.sh setup` | First run: schema, vault key, real history, plan |
| `./scripts/prod.sh start` | Harden permissions, start, health-check |
| `./scripts/prod.sh status` | What is running and what is in the database |
| `./scripts/prod.sh doctor` | The full pre-flight — every check, or it fails |
| `./scripts/prod.sh stop` | Stop the instance |
| `./scripts/run.sh` | Build (if needed) and serve on `:8000` |
| `./scripts/run.sh dev` | API on `:8000` with reload + Vite HMR on `:5173` |
| `./scripts/run.sh stop` | Stop everything |
| `./scripts/run.sh status` | Show what is running |
| `./scripts/import-fit.sh *.fit` | Import FIT files from the shell |
| `node scripts/screenshot.mjs` | Render every route in headless Chromium and report errors |

The backend also has a production tool:

```bash
cd backend
.venv/bin/python -m tools.production setup        # first run
.venv/bin/python -m tools.production status       # what is in the database
.venv/bin/python -m tools.production backfill-fit # rebuild FIT for old activities
.venv/bin/python -m tools.production purge-demo   # remove synthetic activities
```

See [`docs/production-deployment.md`](file:///home/sumit/stride/docs/production-deployment.md).

API docs live at **http://localhost:8000/api/v1/docs**.

### Running it as a managed service

```bash
bb terminal create --machine host_8ej6yymhhj --cwd /home/sumit/stride \
  --title "Stride API" \
  --command "cd /home/sumit/stride/backend && exec /home/sumit/stride/backend/.venv/bin/uvicorn app.main:app --host 0.0.0.0 --port 8000"
```

### Tests

```bash
cd backend && .venv/bin/python -m pytest -q     # 74 tests
cd frontend && npm run typecheck                # zero errors
```

---

## Connecting your Garmin

There are three ways in. All three are supported; none is required to run the app.

### 1. Personal API key (recommended — easiest, and the only one that covers wellness)

Garmin's own first-party API for your own account. You generate a key at
`https://connect.garmin.com/app/settings/personalApiToken` while signed in to
Garmin Connect Web. **No developer-program registration, no consumer secret, no
browser authorisation dance** — and it covers *activities and wellness*:
sleep, resting heart rate, stress, body battery, HRV and scheduled workouts.

Stride reads it from **Azure Key Vault** (`claw-agent` → `garmin-api`) on this
VM, so it connects itself at startup; you can also paste one in Settings.

Two steps happen behind the scenes:

1. `POST /personalapitoken-service/personal-api/personal-api-key/token` with
   `Authorization: ApiKey <key>` returns a JWT valid for 24 hours (scopes:
   `CONNECT_READ`, `PERSONAL_API_FITNESS_READ`, `PERSONAL_API_WELLNESS_READ`,
   `PERSONAL_API_USER_READ`, `PERSONAL_API_NUTRITION_READ`). Stride refreshes it
   automatically.
2. Every other endpoint is a plain bearer-authenticated GET against
   `personal.connectapi.garmin.com`.

The original `.FIT` file is downloaded per activity, so each run keeps its full
GPS track, per-second heart rate and kilometre splits — not just a summary.

### 2. Developer API key (for people who already have one)

Garmin's official OAuth1 Activity API. Requires a **Consumer Key** and
**Consumer Secret** from [developer.garmin.com](https://developer.garmin.com).

1. In the app: **Settings → Garmin connection → Developer API key**.
2. Paste the key and secret, click **Connect with key**.
3. The app opens Garmin's authorisation page. Sign in and approve.
4. Garmin redirects back to the app and the connection is live.

> Garmin paused new applications to the Connect Developer Program in 2026, so
> you may not be able to register a new app. If you already have a key, this is
> the best path — it never touches your password.

### 3. Your Garmin account (the fallback)

Email and password, with an inline MFA step when Garmin demands a 6-digit code.
Uses the `garminconnect` library under the hood and stores a refreshable session
token so you only do this once.

Use this when a Personal API key is not available. It unlocks the same wellness
data (sleep, resting heart rate, stress, body battery, HRV) plus Garmin's
*scheduled workouts*, which Stride reads as a plan source — at the cost of
handing Garmin your password.

### 4. FIT files (no credentials at all)

Export `.FIT` files from Garmin Connect (an activity → **⋯** → **Download FIT**)
and drop them into the app. The importer decodes the full file: GPS track,
per-second heart rate, cadence, elevation and laps.

```bash
# or from the command line
curl -X POST http://localhost:8000/api/v1/auth/garmin/fit-import \
     -F "files=@activity.fit"

./scripts/import-fit.sh ~/Downloads/*.fit
```

### Importing your training plan

**Plan → Import a plan**, then paste a markdown (or CSV, or JSON) plan document.
Stride parses the weekly volume table, the phase pace bands and the day-level
session tables, and builds the plan hierarchy from it.

The importer is tested against a real 20-week plan and yields **8 phases, 20
weeks and 83 day-level sessions**, with pace bands on 67 of them. Imported plans
are **anchored to your activity history**, so the plan and your runs sit on the
same timeline instead of the plan reading as "every session missed".

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
│   │   ├── models.py            ORM schema (activities, laps, streams, zones)
│   │   ├── models_phase2.py     plan hierarchy · results · physiology · fingerprint · gear
│   │   ├── network.py           Tailscale exit-node management
│   │   ├── api/                 auth · activities · stats · calendar · sync · plan · physiology · fitness
│   │   ├── ingest/              garmin_oauth · garmin_account · garmin_health ·
│   │   │                        garmin_workouts · fit_importer · plan_importer · demo
│   │   └── services/            persistence · analytics · sync · plan_engine · fingerprint · readiness
│   ├── tests/                   74 tests (Phase 1 + Phase 2)
│   └── data/                    stride.db + state.json (gitignored)
├── frontend/                    React 18 + TS + Vite + Tailwind + Recharts + Leaflet
│   ├── src/components/          design system: ui · charts · ActivityCard · RouteThumb
│   ├── src/lib/                 api client + formatters
│   ├── src/pages/               Dashboard · Plan · Feed · Activity · Training ·
│   │                            Physiology · Fitness · Stats · Settings
│   ├── CONTRACT.md              the frozen API/design contract
│   └── types_phase2.ts          the frozen Phase-2 types
├── scripts/run.sh               one-command launcher
├── scripts/import-fit.sh        FIT import from the shell
├── scripts/screenshot.mjs       headless render + error check for every route
├── DESIGN.md                    the Phase-2 design of record
└── PLAN.md                      scope, discovery findings, risks
```

### Surfaces

| Route | What it is |
|---|---|
| `/` | **Today** — the plan for today, readiness, one recommendation; then volume, pace, records, recent runs |
| `/plan` | **Plan vs actual** — the week strip (planned beside done), per-session verdicts with causes, weekly adherence, import |
| `/physiology` | Readiness score with its drivers, sleep stages, HRV against a 30-day baseline |
| `/fitness` | **Running fingerprint** — cadence stability, HR drift, aerobic decoupling, efficiency, zone split, shoe rotation |
| `/feed` | Activity feed with route thumbnails, filters, pagination |
| `/activities/:id` | Interactive map, stat strip, HR/pace/elevation charts, splits table, notes |
| `/training` | Month calendar with volume tiers, day detail, streaks |
| `/stats` | Totals & gains, personal records, sport split, year heatmap, trends |
| `/settings` | Connect Garmin, network route, sync settings, sync history |

---

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
