# Stride — the plan of record

> Status: **the phone is the app.** The Rust core runs inside the APK, the
> Garmin key lives in the phone's Keystore, the phone has its own encrypted
> database and talks to Garmin directly. What is left is in §4.

---

## 1. What the product is

A private, local-first running log built around one loop:

**Plan → Execute → Understand → Adapt.**

The plan is a first-class object: every session gets a verdict *and* a
cause. The athlete's physiology and running mechanics are modelled. The app
tells you what to do today.

There is no social layer, and there is not going to be one. A social layer
would mean the data stopped being yours alone.

---

## 2. The shape of the system

One domain implementation, two hosts.

```
crates/stride-fit      the FIT parser and the run/walk/stand segmentation
crates/stride-core     fingerprint, readiness, plan engine, comparison
crates/stride-garmin   the Garmin Personal API client
crates/stride-store    the schema, the migrations, the repositories
crates/stride-api      the request router — the one router, two hosts
crates/stride-mcp      the harness tools
android/               the Kotlin shell: Keystore, bridge, notifications
desktop/               one binary: the router over HTTP, serving the SPA
backend/               the Python reference implementation, and the oracle
frontend/              the React app
```

- The **desktop** runs the router over HTTP and serves the SPA beside it.
- The **phone** calls the same router in-process, through a UniFFI bridge.
- The **React app** is unchanged. Only its transport swaps.

The Python app is the **oracle**, not the product: it is what the golden
fixtures were produced from, and it is what the Rust core is compared
against. It is not what the athlete runs.

---

## 3. What shipped

### The phone

| | |
|---|---|
| the Rust core runs inside the APK | `libuniffi_stride.so`, **8.58 MiB**, SQLCipher and OpenSSL vendored |
| the APK | **8.6 MB** (8,609,784 bytes), arm64-v8a |
| the Garmin key | a non-exportable Keystore key wraps it; it never enters the WebView |
| the database | the phone's own, encrypted, in the app's private storage |
| the onboarding | **one screen**: the key, then `Checking…` → `Syncing…` → `Connected · 160 runs` |
| the surfaces | **six** — Today, Plan, Runs, Body, Form, Stats |

The cut is the phone's: Compare, the training calendar, the server settings.

**v1 reads the data; it does not author it.** Connect a key, it syncs, and
you read your own history and the intent behind it. A manual plan editor,
editing a run and notes are the next iteration, and they are deliberately
not designed yet.

The Plan screen does not need a document. A plan comes from one of three
sources, and the order matters: the plan the athlete **already has**,
inferred from their own history (a **baseline**: "less than usual", not
"missed"); the plan they **wrote down**, Garmin's scheduled workouts (a
**prescription**); and the plan they **type in**, a manual week — real, and
separate work, and not v1. The plan sources are being built now: the
reference is `backend/app/services/plan_sources.py` and the port is
`crates/stride-core/src/plan_sources.rs`.

The desktop keeps Compare and the training calendar. The cut is the
phone's, not the product's: the desktop has the room, and those screens are
good there.

### The gates

| gate | result |
|---|---|
| `./scripts/rust_gate.sh` | **9 passed, 0 failed, 0 warnings** — 355 tests, 38 golden assertions |
| `./scripts/security_check.sh` | **26 passed, 0 failed, 1 warning** |
| `python3 scripts/check_apk.py <apk>` | the APK carries the Rust core and points at nothing |
| `python3 scripts/api_parity.py` | **38 of 38 routes** answer with the same shape as the Python reference |
| `backend/.venv/bin/python -m pytest -q` | **178 passed**, 2 skipped, 2 xfailed, 1 xpassed |
| the desktop binary | **9.1 MB** |

### The contracts that are frozen

| contract | where |
|---|---|
| the onboarding flow and the surface cut | `docs/phone-simplification.md` |
| the crates, the data model, the route table | `docs/rust-core-spec.md` |
| the units, the rounding and the representation of `None` | `docs/core-contract.md` |
| the API response shapes | the OpenAPI document the Python app generates |

Three rules that do not move: **the API shapes do not change** (the React app
consumes them), **the schema does not change** (a desktop export has to open
on the phone), and **no new dependency** without saying so and justifying it.

---

## 4. What is open

Ranked by what it costs the athlete.

1. **The Rust desktop cannot pair a second device.** `GET
   /api/v1/auth/pair-code` answers **501** — "the pairing code is minted by
   the application state", and the application state is not wired to it. The
   reference app still mints one (`./scripts/prod.sh pair`); the Rust
   desktop serves the browser on its own machine and nothing else.
2. **The developer-key and account-login flows are not in the Rust core.**
   `/auth/garmin/developer-key`, `/auth/garmin/login` and `/auth/garmin/mfa`
   answer **501**. The phone does not care — it uses the Personal API key
   and nothing else — but the desktop cannot use them yet.
3. **The network route is not in the Rust core.** `/network/status` and
   `/network/route` answer **501** on the desktop ("the network status is
   read from the machine, not from the database"). The reference app
   implements the egress switch; the Rust desktop does not.
4. **The FIT is not yet the phone's storage format.** The design review
   measured the exploded per-second series at **115 KB per activity** against
   **20 KB** for the original file. The phone should keep the FIT and derive
   from it; that is not done.
5. **The renderer costs about 100 MB on the phone.** Measured by the memory
   workstream. A native UI is the answer if it matters; it is not yet a
   decision.
6. **Two smaller design items are open**: the readiness grade is still not
   one traffic light everywhere, and the fingerprint page is still 14.8
   screens. See `docs/design-review.md` §6.

> The design review's other five items — the fingerprint table as cards on a
> phone, units on every driver number, volume bars coloured by tier, range
> selectors on the trend charts and the map as the hero of the activity page
> — **are done**, with the file and the mechanism in `docs/design-review.md`
> §5.

---

## 5. Risks

| risk | mitigation |
|---|---|
| the port is subtly wrong | the golden fixtures decide, and every crate is gated on them |
| two cores drift apart | there is one core; the Python app is the oracle, not a second product |
| the phone's database cannot open the desktop's export | the schema does not change, and the export is tested |
| the app is rejected from a store | it is not a website wrapper: it has no server and no network dependency for its data |

---

## 6. The plan of record before this one

The original plan — the Python app, the four ingest paths, the six surfaces
— is in the git history. It is kept because the reference implementation it
produced is the oracle the Rust core is measured against.
