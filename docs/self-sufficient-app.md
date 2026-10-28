# The self-sufficient Android app

**A decision document. Nothing here is implemented yet.**

The goal: **an APK that is the whole app.** It holds the Garmin credential in
the phone's own keystore, its own database, its own copy of the analysis, and
it talks to Garmin directly. No server. No tailnet. No pairing code. It works
on a plane, in the hills, with the VM switched off.

This is the plan for getting there, and the decisions that have to be made
first.

---

## 1. What changes, in one paragraph

The domain logic — the FIT parser, the run/walk/stand segmentation, the
running fingerprint, the readiness score, the plan-vs-actual engine, the
Garmin client, the database — **moves out of Python and into a Rust core**.
That core is compiled twice: once for the phone, where it runs *inside* the
app; once for the desktop, where it runs as a local binary that serves the
same API to the same React app. One implementation, two hosts. The Android
app is no longer a browser pointed at a server; it is the server.

---

## 2. The decisions

Nine decisions. Each has a recommendation and the reason. The ones marked
**DECIDE** need your answer before the work starts.

### D1. The core language — **Rust** · DECIDE

The port is the same size in Rust or Kotlin (~3,700 lines). The difference
is what happens to the **desktop**:

| | Kotlin core | Rust core |
|---|---|---|
| phone | local app | local app |
| desktop | still a Python server | **a local binary** — the server disappears |
| implementations of the domain logic | **two, forever** | **one** |
| the desktop's setup | venv + npm + a process to keep alive | **one binary** |

Rust wins because it collapses the *whole system* onto one implementation.
With Kotlin you carry a second copy of the FIT parser and the engines
forever, and a half-cadence in the port is invisible in a screenshot and
wrong in every decision.

Cost: the Android platform APIs — the Keystore, WorkManager, the Activity
lifecycle — are Java, so there is a Kotlin layer regardless. It is a few
hundred lines of platform glue, and the domain logic is not in it.

**Also decided by this:** the desktop's Python backend becomes a *reference
implementation*, not a product. It stays as the oracle the differential
tests compare against, and it is retired once the port is proven.

### D2. The UI — a bundled WebView now, native later · DECIDE

Two separate questions are hiding inside "not just a web view":

1. **Must the app not depend on a remote server?** Yes — and that comes from
   the core, not from the UI. A WebView loading **bundled** assets is
   completely self-sufficient.
2. **Must the app not use a WebView at all?** That is a separate, much larger
   question.

**Recommendation: keep the WebView, bundle the assets.** The React app
becomes a static bundle inside the APK; there is no server to reach. The
cost is Chrome's renderer on the phone — roughly 100 MB, which is the real
weight on the device, and which the memory investigation is measuring now.

Rewriting ten screens in Compose is weeks of work that buys back that
memory and buys a native feel. It is worth doing *later*, on evidence,
rather than first, on principle. **The thing you actually asked for — an app
that runs on its own — comes from the core, not from the UI.**

If the answer here is "no WebView at all", the plan grows by roughly the
whole UI layer and the colour system has to be re-expressed in Kotlin.

### D3. The bridge — one Capacitor plugin method · DECIDE

The SPA calls `fetch('/api/v1/...')`. On the phone there is no HTTP server,
so something must carry those calls into the core.

**Recommendation: a Capacitor plugin with a single generic method.**

```ts
apiCall(method: 'GET'|'POST'|..., path: string, body?: unknown): Promise<unknown>
```

The routing, the authorisation and the SQL all stay in Rust. The bridge is
one method, not forty — so the SPA's `api.ts` swaps its *transport*, not its
surface, and every page keeps working unchanged.

**Why not a local HTTP server on the phone?** Any app on the phone can
connect to `127.0.0.1`, so a local server would expose your Garmin data to
every other app on the device. The plugin bridge has no port at all.

**Why not forty typed methods?** That is forty places for the two
implementations to drift apart.

### D4. The store — `rusqlite` + bundled SQLCipher

One database file in the app's private storage, encrypted with a key in the
Android Keystore. `rusqlite` with the `bundled-sqlcipher` feature compiles
SQLCipher from source through the NDK; there is no system dependency to ship.

The schema is the one the Python app already has — the port must not change
it, or the desktop's snapshot will not open on the phone.

### D5. The FIT parser — the `fitparse` crate

There is no good Kotlin FIT library, so that decision is made for us.
`fitparse` is a Rust port of the Python `fitparse` the current implementation
already uses, which is exactly what you want when the differential tests
compare the two: the same library, so the *parsing* is identical and the
tests are about **our** logic — the segmentation, the halved cadence, the
semicircle positions.

### D6. The Garmin client — `reqwest` + `rustls`

`rustls` rather than OpenSSL: no system TLS dependency, and the whole
toolchain is one `cargo build`. The Garmin surface is small — a token
exchange, a handful of reads, and a FIT download — so there is no need for
anything heavier.

The token is a 24-hour JWT and there are **no refresh tokens**: refreshing
means exchanging the Personal API Key again. The client therefore keeps the
key (encrypted, in the Keystore) and exchanges it on demand, which is
already how the Python client behaves.

### D7. The desktop — the same core, serving the same API

The Rust core exposes one **request router** — the thing that turns
`(method, path, body)` into a result. Two hosts use it:

- the desktop runs a tiny HTTP server on `127.0.0.1` and serves the React
  app and the API from it. The desktop experience does not change at all.
- the phone calls the same router in-process, through the bridge.

So there is **one** API implementation and **one** SPA, and the desktop is
not a different product — it is the same core with an HTTP skin.

### D8. The Garmin key — the Android Keystore

A non-exportable AES-256-GCM key in the Keystore wraps one credential
envelope (the Personal API key) in `noBackupFilesDir`. Plain DataStore is
not encryption, and `EncryptedSharedPreferences` is deprecated. The key never
enters the WebView, never enters a log, and never leaves the device except
over TLS to Garmin.

### D9. The harness — a local MCP server inside the app

The MCP sidecar for Claude Code / Codex currently talks to a running
instance. With a local core, the app can expose the same tools on a
loopback socket, so a harness on the same machine can query the phone's data
— or the desktop's — through one interface.

**This is the piece that most benefits from one core**: the tools are
written once, against the core, and work on both hosts.

---

## 3. The shape of the code

```
stride/
├── crates/
│   ├── stride-core/      the domain: fingerprint, readiness, plan engine, comparison
│   ├── stride-fit/       the FIT parser and the run/walk/stand segmentation
│   ├── stride-garmin/    the Personal API client, token exchange, FIT download
│   ├── stride-store/     the schema, the migrations, the repositories
│   ├── stride-api/       the request router — one method, path, body in, result out
│   └── stride-mcp/       the harness tools, over the router
├── android/              the app: Kotlin shell, Keystore, WorkManager, the bridge
├── desktop/              a small binary: the router over HTTP, serving the SPA
├── frontend/             the React app — unchanged
└── backend/              the Python reference implementation, and the oracle
```

### The FFI surface

`stride-core` and `stride-api` are `cdylib` + `staticlib`. The Kotlin
shell calls them through **UniFFI** — a UDL file describes the surface and
the Kotlin bindings are generated, so there is no hand-written JNI to get
wrong.

The surface is deliberately small:

```
call(method, path, body_json) -> result_json
device_status() -> {connected, athlete, last_sync_at, needs_reconnect}
sync_start(days) -> {started}
pairing_status() -> ...
```

Everything else is a route. One generic entry point means the Kotlin side
never has to change when a route is added.

---

## 4. The phases, and what runs in parallel

### Phase 0 — Freeze the reference · **running now**

| thread | owns | doing |
|---|---|---|
| `thr_hbwgvkrmnh` | `backend/tests/golden/**`, `docs/core-contract.md` | freeze the Python behaviour as exact golden JSON |

Nothing may be ported until this lands. It is the thing that makes the port
*provable* rather than hopeful.

### Phase 1 — The core, in parallel, against the frozen contract

Six workstreams, disjoint directories, one contract:

| workstream | owns | the hard part |
|---|---|---|
| **W1** | `crates/stride-fit/` | the parser and the segmentation — the highest risk, so it goes first |
| **W2** | `crates/stride-core/` | fingerprint, readiness, plan engine, comparison — pure functions over the frozen model |
| **W3** | `crates/stride-garmin/` | the API client and the token lifecycle |
| **W4** | `crates/stride-store/` | the schema and the repositories |
| **W5** | `crates/stride-api/` + `crates/stride-mcp/` | the router and the harness tools |
| **W6** | `crates/*/tests/golden/` | the differential tests — **the gate for all the others** |

**W6 is not optional and not last.** Every other workstream's work is only
accepted when the golden fixtures match. That is how six agents work on one
set of numbers without six different answers.

### Phase 2 — The phone

| workstream | owns |
|---|---|
| **W7** | `android/` — the shell: Keystore, WorkManager, the bridge, the notifications |
| **W8** | the build pipeline: `cargo-ndk`, UniFFI codegen, CI, the signing |

### Phase 3 — The desktop

| workstream | owns |
|---|---|
| **W9** | `desktop/` — the binary that serves the SPA and the API from the core |

The desktop becomes a local app with the same look and the same numbers, and
the Python server is retired.

### Phase 4 — Cut over

The phone becomes the primary. The desktop is the analysis companion. The
Python implementation is archived as the reference.

---

## 5. The contracts that must be frozen before Phase 1

| contract | owner | why it matters |
|---|---|---|
| the **data model** — the tables, the fields, the units | the plan | every crate depends on it |
| the **golden fixtures** and the output format | W6 | the only definition of "correct" |
| the **crate boundaries** and the public API of each | the plan | so six agents do not collide |
| the **FFI surface** | W5 + W7 | the seam between Rust and Kotlin |
| the **route table** — the API paths and shapes | W5 | the SPA depends on it; it must not change |

Everything else is a workstream's own business.

---

## 6. What is deliberately not changing

- **The React app.** Every screen, the design system, the colour language,
  the API contract. The SPA is unchanged; only its transport swaps on the
  phone.
- **The palette.** `frontend/src/lib/palette.ts` stays the source of truth.
- **The database schema.** The phone opens a snapshot the desktop exported.
- **The API shapes.** The SPA is a consumer of them; they are not ours to
  reshape.
- **The Garmin scopes.** Whatever Garmin grants, we use; no more.

---

## 7. The risks

| risk | how bad | mitigation |
|---|---|---|
| **the port is subtly wrong** | the worst one — plausible numbers that are not the athlete's | the golden fixtures, and W6 gating every workstream |
| **UniFFI's build is fiddly in CI** | a week of yak-shaving | W8 owns it early, in parallel with everything else |
| **the FIT quirks** — halved cadence, semicircle positions | wrong by a factor of two | the same `fitparse` library as the reference, so the tests isolate *our* logic |
| **Android background sync is unreliable** | the phone is stale for a while | sync on open, on demand, and opportunistically; never pretend it is exact |
| **a 100 MB renderer** | heavy on a low-end phone | measured now by `thr_ichgk3wdi9`; a native UI is the answer if it matters |
| **two cores drifting** | the desktop and the phone disagree | one implementation — the whole point of D1 |
| **the app gets rejected from Play** | a distribution delay | it is not a website wrapper: it has no server and no network dependency for its data. The Play "webview" policy is a risk to manage, not a certainty |

---

## 8. What is still open

**For you to decide:**

1. **D1 — Rust?** The recommendation is Rust, for the desktop as much as the
   phone. If Kotlin, the desktop stays Python and we carry two
   implementations forever.
2. **D2 — the WebView.** Keep it bundled, or rewrite the UI natively? The
   recommendation is to keep it and measure.
3. **D9 — the harness.** Should the MCP tools live inside the app (so a
   harness can query the phone) or stay a sidecar?

**For the team to answer, from measurements we do not have yet:**

4. How much memory does the renderer really cost, and does it change D2?
   (`thr_ichgk3wdi9` is measuring this now.)
5. Does the security review of the new surface change anything in this plan?
   (`thr_prqcfy7hsz` is running.)
6. How big is the port, really, once the golden fixtures exist? The 3,700
   figure is an estimate from the Python side.

---

## 9. Meanwhile

The desktop keeps working exactly as it does. It is the reference
implementation, the test oracle, and the thing that proves the port is right.
Nothing about it is thrown away until the port is proven.

The phone, once the core lands, stops being a remote view and becomes the
app.

And the tailnet trust and the pairing code are not deleted — they are the
bridge for the interim, and they are how a *second* device connects to a
desktop that is still running Python.
