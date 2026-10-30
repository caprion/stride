# Stride

> **Most running apps tell you what you did. Stride tells you whether that was
> the right thing to do — and what to do tomorrow.**

A private, local-first running log, built around one loop:

**Plan → Execute → Understand → Adapt.**

Three jobs, in this order: **decide** what to do today (Today), **understand**
what each run meant (Runs), and **stay motivated** with proof you are getting
fitter (Progress). Body is the evidence: sleep, HRV and resting HR against your
own normal. The why behind every choice is in
[docs/philosophy.md](docs/philosophy.md).

A social running app is a **social activity log**: it has no concept of
intent, no physiology and no causality. That is the gap this fills.

**v1 reads the data, and decides with it.** You connect a key, it syncs, and
every morning it says what to do today and why; every run gets a sentence on
what it meant; and Progress shows you getting fitter. See
[what v1 is](#what-v1-is) below.

---

## The app is the app

**Your data stays on this phone.**

The Android APK carries the whole of Stride. The Rust core — the FIT parser,
the run/walk/stand segmentation, the running fingerprint, the readiness
score, the plan engine, the Garmin client and the encrypted database — runs
*inside* the phone. The Garmin Personal API key lives in the phone's
Keystore. The phone has its own database and talks to Garmin directly.

There is no server to reach, no account to create and nothing to sign up for.
It works on a plane, with the VM switched off.

```bash
gh release download v0.8.0 --repo caprion/stride --pattern '*.apk'
```

Install the APK on the phone (allow installs from that source), open it, and
paste your Garmin Personal API key into the one native screen that asks for
it. The screen says **Checking…**, then **Syncing…**, then **Connected**, and
it goes away. The key never enters the WebView: it is typed into a native
field, wrapped by a non-exportable Keystore key, and written to
`noBackupFilesDir`.

**The promise is one line, everywhere.** The setup screen says it, the About
card says it, and the screenshots say it:

> **Your data stays on this phone.**

| | |
|---|---|
| the APK | **8.4 MB** (`stride-v0.8.0.apk`, arm64-v8a) |
| the desktop binary | **9.3 MB** (`target/release/stride`) |
| the phone's destinations | **four** — Today · Runs · Progress · Body |
| the desktop's | all nine, plus the settings |

### The desktop is the same core

The desktop is the same Rust core as **one binary**. It serves the same
React app and the same API, out of the same crate, against the same
database. There is no second implementation of the domain logic and no
process to keep alive.

The desktop is the same product with more room. The phone keeps **four
destinations** — Today, Runs, Progress and Body — and cuts what is a laptop
activity: **Compare** (two runs side by side), the **training calendar** (the
Plan screen carries the schedule), the **server settings** and the plan
editor. The desktop keeps all of them.

The desktop says the same promise in its own words: **"Your data stays on
this machine."**

### What v1 is

**v1 VIEWS DATA. It does not manage it.**

The phone is a reader for the athlete's own history and for the intent
behind it.

| what v1 does | what v1 does not do |
|---|---|
| connect a Garmin key, in one native screen | write a plan by hand on the phone |
| sync: activities, wellness, and the plan it finds | edit a run's data (distance, time, HR) |
| import `.FIT` files from **any** watch, and not add the same run twice | score you against other people |
| **Today**: one call, one reason, and a distance warning only when the run is near the line | show a red "missed" anywhere |
| **Runs**: a chip and a short story for every run | use Garmin's own scores, which need a password |
| **Progress**: pace at the same HR, forgiving consistency, bests, milestones | quote a trend a near-zero reading cannot carry |
| **Body**: sleep, HRV and resting HR against your own normal | show a 0-100 readiness score |
| an optional weekly goal and a pause (travel, sick, injured), on the device | |

The three small inputs (a weekly goal, a pause, and a run's note) are there
because they change what the app should say. Everything else that authors
or edits (a plan editor on the phone, editing a run's numbers) is the next
iteration.

**The advice is graded, and it is one sentence.** One recovery signal that is
just outside its normal range is not three, and a mild one is not a marked
one — so the call says so, and the two screens that show recovery (Today and
Body) read the *same* sentence from the core:

| last night | the call |
|---|---|
| one signal, just outside its range | "Keep it easy today." — *if you feel off too, take the day* |
| one signal, well outside its range | "Go easy today, or take the day." — *how you feel has the last word* |
| several signals | "Take the day, or make it very easy." — *if you feel fine, an easy run is enough* |

And the card never contradicts itself: the headline quotes the distance the
runner will actually do (the plan's, when there is one), and the plan line
under it says the same thing the call says.

**The Plan screen is the reason the app exists, and it does not need a
document.** A plan is *intent*: a dated list of sessions with a target. The
screen compares the intent with what happened and says why it did not match.
Three sources, and the order matters:

1. **The plan you already have — inferred from your own history.** Every
   runner has an implicit plan: their weekly volume, their rest days, their
   long-run day, their usual session length per weekday. Twelve weeks of
   history gives all of it. **Zero setup, and it works for everybody.**

   And it is not a prescription, so the words must not pretend it is. An
   inferred plan is a *baseline*: **"less than usual"**, **"more than
   usual"**, **"a usual day"** — not "missed" and "hit".
2. **The plan you wrote down — Garmin's scheduled workouts.** If you schedule
   workouts in Garmin Connect, those are your plan, and they are a
   *prescription*: the athlete wrote them down.
3. **The plan you imported.** A `.FIT` file, a CSV or a markdown week. The
   importer reads what it can and says what it cannot: a session whose kind
   it does not recognise stays **unknown**, and nothing downstream pretends
   it is an easy run.

### The Python app is the reference, not the product

`backend/` is the **oracle**: the original implementation, kept so the Rust
core can be proved equal to it rather than believed equal to it. It answers
the same routes with the same shapes, and the golden fixtures under
`backend/tests/golden/` are what say which of the two is right. It is not
what the athlete runs.

The insight engines — Today, the run stories, Progress — are **Rust-native**:
they have no Python counterpart to be measured against, and the port says so
rather than pretending.

---

## What is measured

Every number below is from a gate in this repository, not from a judgement
call.

| gate | what it says | result |
|---|---|---|
| `./scripts/rust_gate.sh` | format, clippy, release build, tests, golden fixtures, panics, dependency set, size | **9 passed, 0 failed, 0 warnings** — 423 tests, 41 golden assertions |
| `./scripts/security_check.sh` | secrets, permissions, configuration, dependencies, tests, and **the live service** | **26 passed, 0 failed, 1 warning** |
| `python3 scripts/check_apk.py <apk>` | the APK carries the Rust core and points at nothing | **the APK carries the Rust core and points at nothing** |
| `python3 scripts/api_parity.py` | the Rust router and the Python reference, route by route | **37 of 38 routes answer with the same shape** |
| `node scripts/phone_smoke.mjs` | the four phone surfaces at 390 px, on synthetic data | **29 of 29 checks pass** |

The security gate's live section is not a formality: with the app running it
mints a pairing code on the machine, pairs a throwaway device, reads the API
with it, revokes it, and watches it be signed out — and it checks that a
caller on the network cannot mint a code at all.

The one difference in the parity check is `GET /api/v1/health`: the Rust
desktop reports `garmin.method: null` and the reference reports `"personal"`.
It is the nullability of one field — the desktop copy in that measurement
had no Garmin key configured — and it is open.

---

## Showcase

A one-page collage of every surface: **[`showcase/stride-collage.png`](showcase/stride-collage.png)** — the app as it is now, with the promise on it:

> **Your data stays on this phone.**

The individual screenshots are in [`showcase/shots/`](showcase/shots/), every
screen on both hosts is in [`showcase/review/`](showcase/review/), and
[`showcase/README.md`](showcase/README.md) says how they are captured.

---

## Connecting Garmin

The phone connects with a **Personal API key** — Garmin's own first-party
API for your own account. Generate one at
`https://connect.garmin.com/app/settings/personalApiToken` while signed in
to Garmin Connect Web. **No developer-program registration, no consumer
secret, no browser authorisation dance** — and it covers *activities and
wellness*: sleep, resting heart rate, stress, body battery, HRV and scheduled
workouts.

Two steps happen behind the scenes:

1. `POST /personalapitoken-service/personal-api/personal-api-key/token` with
   `Authorization: ApiKey <key>` returns a JWT valid for 24 hours (scopes:
   `CONNECT_READ`, `PERSONAL_API_FITNESS_READ`, `PERSONAL_API_WELLNESS_READ`,
   `PERSONAL_API_USER_READ`, `PERSONAL_API_NUTRITION_READ`).
2. Every other endpoint is a plain bearer-authenticated GET against
   `personal.connectapi.garmin.com`.

The original `.FIT` file is downloaded per activity, so each run keeps its full
GPS track, per-second heart rate and kilometre splits — not just a summary.

### Files from any watch

A `.FIT` file is a `.FIT` file, whoever wrote it. The importer reads **Strava
exports, COROS, Wahoo, Polar and Garmin** — the file says what it is, and
the watch it came from is part of the run's story, not a reason to refuse it.

A run that is already in the history is not added twice. The run itself is
the key: two rows are the same run when they start within two minutes of
each other and their distances are within two per cent. So the run synced
from Garmin and the file exported from the watch are **one** run, and
re-importing the same file is not a second one.

**The gap:** the Rust desktop answers `501 Not Implemented` to the developer
key and the account login ("the Garmin account flow is driven by the Garmin
client"). The phone is not affected: it uses the Personal API key and
nothing else, and never asks for, and never stores, a Garmin password.

---

## Configuration

Everything is an environment variable. The desktop reads them at startup; the
phone reads the same names from its own settings.

| Variable | Default | Purpose |
|---|---|---|
| `STRIDE_TLS_HOST` | `0.0.0.0` | the address the HTTPS listener binds |
| `STRIDE_TLS_PORT` | `8443` | the port for it |
| `STRIDE_TRUST_REMOTE` | `false` | when true, clients from the trusted networks are trusted without a pairing code |
| `STRIDE_TRUSTED_NETWORKS` | empty | comma-separated CIDRs those clients come from. **Empty means nothing extra is trusted** — loopback only |
| `STRIDE_BIND` | `127.0.0.1` | the address the desktop binary binds. A non-loopback address is refused |
| `STRIDE_PORT` | `8000` | the port the desktop binary binds |
| `STRIDE_DATA_DIR` | `backend/data` | the database and the state file |
| `STRIDE_CORS_ORIGINS` | `http://localhost:5173` | the origins that may call the API from a page. A `*` is refused |

The listener **detects nothing**. It binds `STRIDE_TLS_HOST` and nothing
else; no network's own tooling is run at startup.

### Reaching the app from another device

The phone reaches the desktop over the address the HTTPS listener binds:

```
STRIDE_TLS_HOST    the address the HTTPS listener binds
STRIDE_TLS_PORT    the port for it
```

The certificate on that listener has to be one the browser on the phone
trusts, or the phone will not install the app — a service worker only
registers in a secure context, and plain HTTP leaves the athlete with a
browser tab forever. **The operator's address and certificate are the
operator's business; this repository names neither.**

With the listener up and the certificate trusted, a browser reaches the
desktop at `https://<STRIDE_TLS_HOST>:<STRIDE_TLS_PORT>`.

A browser on the same machine as the desktop is given a session cookie
automatically. A browser on another device is not, so signing one in is a
deliberate act by the operator:

```bash
./scripts/prod.sh pair
```

That prints a **QR code**. Open it in the browser you want to sign in — one
action, once, and it stays connected. The device gets its **own**
credential, so it can be revoked on its own in **Settings → Connected
devices** without touching the machine. The code is carried in the URL
*fragment*, never the query string, so it never lands in an access log; it
works **once** and expires in **10 minutes**.

**The phone app never needs any of this.** It has its own database and its
own Garmin key, and it is signed in from the first pixel: you paste the key
into one native screen and the data stays on the phone. The pairing code is
for a *second browser*, on another machine, that wants to read the
desktop's data — and nothing in the phone's setup asks for one.

Two rules, and the security gate checks both:

- **only a caller on the machine can mint a code.** The peer address *and*
  the `Host` header have to say loopback: a page the athlete happens to have
  open can point a name it controls at 127.0.0.1 and then talk to Stride as
  the origin it came from.
- **the device's credential is stored as a hash, never as the token**, so
  the state file does not hold a credential that works.

---

## The egress route

Garmin throttles **datacentre IPs** hard, and from a datacentre address the
web-login path is blocked by a bot challenge. The same request from a
residential address succeeds. So Stride has a *network route* concept: the
Garmin traffic can be sent out through a machine of the operator's choosing.

| Route | Egress | Use when |
|---|---|---|
| `direct` | the machine itself | Garmin may rate-limit or block |
| `exit_node` | the chosen machine | reliable Garmin access |

The app also detects a `429` or a bot challenge mid-sync and retries once
through the other route automatically. The switch changes the **whole
machine's** egress, not just Stride's traffic — switch it back to `direct`
when other traffic needs the machine's own address.

**The gap:** `/network/status` and `/network/route` answer **501** in the
Rust core — "the network status is read from the machine, not from the
database". The egress switch is the reference application's; the Rust
desktop does not have it yet.

---

## Design documents

| Document | What it decides |
|---|---|
| [`docs/philosophy.md`](docs/philosophy.md) | the thesis, the principles, and what v1 is and is not |
| [`DESIGN.md`](DESIGN.md) | the product: plan-vs-actual, physiology, fingerprint, comparison |
| [`docs/rust-core-spec.md`](docs/rust-core-spec.md) | the crates, the data model and the route table |
| [`docs/core-contract.md`](docs/core-contract.md) | the byte-level contract: units, rounding, `None` |
| [`docs/phone-simplification.md`](docs/phone-simplification.md) | the frozen contract for the phone: one screen, half as much |
| [`docs/self-sufficient-app.md`](docs/self-sufficient-app.md) | what the self-sufficient app is, and what is built |
| [`docs/production-deployment.md`](docs/production-deployment.md) | how the deployment is configured and checked |
| [`docs/design-review.md`](docs/design-review.md) | the footprint, the colour language, every screen on a phone |
| [`docs/android-app.md`](docs/android-app.md) | the Android architecture and the threat model |
| [`docs/android-packaging.md`](docs/android-packaging.md) | how the APK is packaged, signed, built in CI |
| [`docs/garmin-on-device.md`](docs/garmin-on-device.md) | key entry, Keystore storage, pull cadence |
| [`docs/harness-integration.md`](docs/harness-integration.md) | the MCP sidecar, so your own agent can query your data |
| [`docs/security/AUDIT.md`](docs/security/AUDIT.md) | the security audits and what they found |
| [`docs/working-agreement.md`](docs/working-agreement.md) | how we run big changes with parallel agents |
| [`docs/agent-review-loop.md`](docs/agent-review-loop.md) | the review loop: how the work is checked before it ships |

---

## No social layer, on purpose

There is no feed, no followers, no comments and no sharing to a network — and
there is not going to be one. A social layer would mean the data stopped being
yours alone. **Sharing is Garmin's business**: every activity links back to its
original in Garmin Connect, and Garmin decides who may open it. Stride never
publishes, never mirrors and never asks for an account.

---

## Architecture

```
stride/
├── crates/
│   ├── stride-core/         fingerprint, readiness, plan engine, comparison, insights
│   ├── stride-fit/          the FIT parser and the segmentation
│   ├── stride-garmin/       the Garmin Personal API client
│   ├── stride-store/        the schema, the migrations, the repositories
│   ├── stride-api/          the request router — the one router, two hosts
│   └── stride-mcp/          the harness tools
├── android/                 the Kotlin shell: the Keystore, the bridge, the notifications
├── desktop/                 one binary: the router over HTTP, serving the SPA
├── frontend/                the React app
├── backend/                 the Python reference implementation, and the oracle
└── scripts/                 the gates
```

One router, two hosts: the desktop runs it over HTTP, the phone calls it
in-process through the FFI. The SPA is unchanged and only its transport
swaps.

### The data model

The schema is the one the Python app has always had, and it does not change:
the desktop's export has to open on the phone. `activity` is the canonical
row; every ingest path normalises into the same shape before it touches the
database, so nothing downstream branches on source.

```
activity         id, source, external_id, name, sport_type, start_time, local_date,
                 distance_m, moving_time_s, elapsed_time_s, elevation_gain_m,
                 avg_hr, max_hr, calories, avg_cadence, avg_speed_ms,
                 perceived_exertion, device, description, gear, is_trail,
                 is_commute, visibility, geo_points, raw
activity_lap     activity_id, lap_index, distance_m, duration_s, avg_speed_ms,
                 avg_hr, max_hr, avg_cadence, stride_length_m, elevation_*
activity_stream  activity_id, kind(hr|pace|elevation|distance|cadence), seconds, values
hr_zone          activity_id, zone_no, name, low_bpm, high_bpm, time_s
daily_physiology date, resting_hr, sleep_s, hrv_ms, stress, steps, body_battery, readiness
sync_run         started_at, finished_at, status, days, activities_found/new/updated, error
```

`unique(source, external_id)` makes re-syncing idempotent, and the import
dedupe is the same idea one level up: the *run* is the key, not the file it
came in. Derived numbers (pace, weekly volume, HR-zone distribution, personal
records, streaks) are computed on read, so there are no stale aggregates.

---

## Running it

### The desktop

```bash
cargo run --release -p stride-desktop
```

It serves the SPA and the API on `127.0.0.1:8000` and opens the athlete's
own database. The first run converts the plain database into an encrypted
one — the copy is kept, the row counts are checked, and the plain file is
only replaced when the converted one reads back with the same rows.

### The reference implementation

```bash
cd backend && .venv/bin/python -m app.serve      # the Python app, :8000
```

### Tests

```bash
./scripts/rust_gate.sh          # the Rust workspace
./scripts/security_check.sh      # the Python side and the live service
cd backend && .venv/bin/python -m pytest -q
cd frontend && npm run typecheck
node scripts/phone_smoke.mjs     # the phone surfaces at 390 px, on synthetic data
```

---

## Licence

MIT.
