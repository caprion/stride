# Stride

> **Most running apps tell you what you did. Stride tells you whether that was
> the right thing to do — and what to do tomorrow.**

A private, local-first running log, built around one loop:

**Plan → Execute → Understand → Adapt.**

The plan exists as a first-class object. Every session gets a verdict *and* a
cause. The athlete's physiology and running mechanics are modelled. The app
tells you what to do today.

A social running app is a **social activity log**: it has no concept of
intent, no physiology and no causality. That is the gap this fills.

**v1 reads the data. It does not author it.** You connect a key, it syncs,
and you read your own history and the intent behind it. See
[what v1 is](#what-v1-is) below.

---

## The app is the app

**Your data stays on this phone.**

The Android APK carries the whole of Stride. The Rust core — the FIT parser,
the run/walk/stand segmentation, the running fingerprint, the readiness
score, the plan engine, the Garmin client and the encrypted database — runs
*inside* the phone. The Garmin Personal API key lives in the phone's
Keystore. The phone has its own database and talks to Garmin directly.

There is no server to reach, no pairing code to type and no account to
create. It works on a plane, with the VM switched off.

```bash
gh release download v0.4.2 --repo caprion/stride
```

Install the APK on the phone (allow installs from that source), open it, and
paste your Garmin Personal API key into the one native screen that asks for
it. The screen says **Checking…**, then **Syncing…**, then **Connected · 160
runs**, and it goes away. The key never enters the WebView: it is typed into
a native field, wrapped by a non-exportable Keystore key, and written to
`noBackupFilesDir`.

**The promise is one line, everywhere.** The setup screen says it, the About
card says it, and the screenshots say it:

> **Your data stays on this phone.**

| | |
|---|---|
| the APK | **8.6 MB** (8,609,784 bytes), arm64-v8a |
| the Rust core inside it | **8.58 MiB** — SQLCipher and OpenSSL vendored |
| the desktop binary | **9.1 MB** (9,478,400 bytes, `target/release/stride`) |

The app is built in CI and signed with a key whose certificate is committed in
`backend/app/assetlinks.json`; the key itself is only in the repository
secrets. `scripts/check_apk.py` asserts that the published APK really carries
the core and points at nothing — a build that quietly loses it still
succeeds otherwise, and that happened once.

### The desktop is the same core

The desktop is the same Rust core as **one binary**. It serves the same
React app and the same API, out of the same crate, against the same
database. There is no second implementation of the domain logic and no
process to keep alive.

The desktop is the same product with more room. The phone keeps **six
destinations** — Today, Plan, Runs, Body, Form and Stats — and cuts what is
a laptop activity: **Compare** (two runs side by side), the **training
calendar** (the Plan screen carries the schedule), and the **server
settings**. The desktop keeps all of them.

The desktop says the same promise in its own words: **"Your data stays on
this machine."**

### What v1 is

**v1 VIEWS DATA. It does not manage it.**

The phone is a reader for the athlete's own history and for the intent
behind it.

| what v1 does | what v1 does not do |
|---|---|
| connect a Garmin key, in one native screen | write a plan by hand |
| sync: activities, wellness, and the plan it finds | edit a run, rename it, describe it |
| show the plan against what was actually done | keep notes on a run |
| show readiness, with its drivers | author anything at all |

Everything that **authors or edits** — a manual plan editor, editing a run,
notes — is the next iteration, and it is deliberately not designed yet.

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
3. **The plan you type in — a manual week.** Real, and a separate piece of
   work: an editor, an API and the storage for it. **Not v1.**

> **Work in flight.** The plan sources are being built now: the reference is
> `backend/app/services/plan_sources.py` and the port is
> `crates/stride-core/src/plan_sources.rs`. They are not in the published
> build yet.

### The Python app is the reference, not the product

`backend/` is the **oracle**: the original implementation, kept so the Rust
core can be proved equal to it rather than believed equal to it. It answers
the same routes with the same shapes, and the golden fixtures under
`backend/tests/golden/` are what say which of the two is right. It is not
what the athlete runs.

---

## What is measured

Every number below is from a gate in this repository, not from a judgement
call.

| gate | what it says | result |
|---|---|---|
| `./scripts/rust_gate.sh` | format, clippy, release build, tests, golden fixtures, panics, dependency set, size | **9 passed, 0 failed, 0 warnings** — 355 tests, 38 golden assertions |
| `./scripts/security_check.sh` | the Python side: permissions, configuration, dependencies, tests, the live service | **26 passed, 0 failed, 1 warning** |
| `python3 scripts/check_apk.py <apk>` | the APK carries the Rust core and points at nothing | **the APK carries the Rust core and points at nothing** |
| `python3 scripts/api_parity.py` | the Rust router and the Python reference, route by route | **38 of 38 routes answer with the same shape** |

The Rust router and the Python reference now answer **all 38 routes** with
the same shape: the same keys, the same types, the same nesting.

---

## Showcase

A one-page collage of every surface: **[`showcase/stride-collage.png`](showcase/stride-collage.png)** — the app as it is now, with the promise on it:

> **Your data stays on this phone.**

The individual screenshots are in [`showcase/shots/`](showcase/shots/), and
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

The desktop also imports `.FIT` files — the parser is the same
`stride-fit` crate the phone uses — and the reference application still
accepts a developer key and an account login with MFA.

**The gap:** the Rust desktop answers `501 Not Implemented` to the developer
key and the account login ("the Garmin account flow is driven by the Garmin
client"). The phone is not affected: it uses the Personal API key and nothing
else, and never asks for, and never stores, a Garmin password.

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
automatically. A browser on another device is not, so it has to be paired on
purpose. **The phone app does not need this** — it has its own database and
its own Garmin key — but a second browser, or a browser on another machine,
does.

**Today the pairing code is minted by the reference application**, not by the
Rust desktop:

```bash
./scripts/prod.sh pair
```

That prints a **QR code**. Point the phone's camera at it and the phone is
connected — one action, once, and it stays connected. The device gets its
**own** credential, so a lost phone can be revoked on its own in
**Settings → Connected devices** without touching the machine. The code is
carried in the URL *fragment*, never the query string, so it never lands in
an access log; it works **once** and expires in **10 minutes**.

**The gap:** the Rust desktop answers `501 Not Implemented` to
`GET /api/v1/auth/pair-code` — the store backend says "the pairing code is
minted by the application state" and the application state has not been wired
to it. Until that is done, the Rust desktop serves the browser on its own
machine and nothing else.

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
| [`docs/security/AUDIT.md`](docs/security/AUDIT.md) | the security audit and what it found |
| [`docs/working-agreement.md`](docs/working-agreement.md) | how we run big changes with parallel agents |

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
│   ├── stride-core/         fingerprint, readiness, plan engine, comparison
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

`unique(source, external_id)` makes re-syncing idempotent. Derived numbers
(pace, weekly volume, HR-zone distribution, personal records, streaks) are
computed on read, so there are no stale aggregates.

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
./scripts/security_check.sh      # the Python side
cd backend && .venv/bin/python -m pytest -q
cd frontend && npm run typecheck
```

---

## Licence

MIT.
