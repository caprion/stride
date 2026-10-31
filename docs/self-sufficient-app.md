# The self-sufficient Android app

**The goal: an APK that is the whole app.** It holds the Garmin credential in
the phone's own Keystore, its own database, its own copy of the analysis, and
it talks to Garmin directly. No server. No pairing code. It works on a plane,
in the hills, with the VM switched off.

**It is built.** This document is now a description of what exists, and of
the four things that are still missing.

---

## 1. What it is

The domain logic — the FIT parser, the run/walk/stand segmentation, the
running fingerprint, the readiness score, the plan-vs-actual engine, the
Garmin client, the database — lives in a **Rust core** that is compiled
twice: once for the phone, where it runs *inside* the app, and once for the
desktop, where it runs as one binary that serves the same API to the same
React app. One implementation, two hosts. The Android app is not a browser
pointed at a server; it is the server.

| | measured |
|---|---|
| the APK | **8.7 MB** (8,670,620 bytes), arm64-v8a |
| the Rust core inside it | **8.69 MiB** — SQLCipher and OpenSSL vendored |
| the desktop binary | **9.2 MB** |
| the Rust gate | **368 tests passed, 38 golden assertions**, 9 of 9 checks green |
| the route parity | **37 of 38 routes** answer with the same shape as the reference |

---

## 2. The decisions, and what each one became

### D1. The core language — **Rust** · done

The port is the same size in Rust or Kotlin, and the difference is what
happens to the **desktop**. With Kotlin the desktop stays a Python server
and the domain logic exists twice, forever; a half-cadence in the port is
invisible in a screenshot and wrong in every decision. With Rust the whole
system collapses onto one implementation.

The Python app did not disappear: it became the **oracle**, and the golden
fixtures it produces are what the Rust core is measured against.

### D2. The UI — a bundled WebView · done, and measured

The app does not depend on a remote server, and that comes from the core,
not from the UI: a WebView loading **bundled** assets is completely
self-sufficient. The React app is a static bundle inside the APK.

The cost is Chrome's renderer on the phone, **about 100 MB** — the real
weight on the device. Rewriting ten screens in Compose is weeks of work
that buys back that memory and buys a native feel, and it is worth doing
later, on evidence, rather than first, on principle.

### D3. The bridge — one generic call · done

The SPA calls `fetch('/api/v1/...')`. On the phone there is no HTTP server,
so the calls are carried into the core by a **UniFFI** surface with one
generic method:

```
stride_call(method, path, body) -> result_json
```

The routing, the authorisation and the SQL all stay in Rust. The bridge is
one method, not forty — so the SPA's `api.ts` swaps its *transport*, not its
surface, and every page keeps working unchanged.

**Why not a local HTTP server on the phone?** Any app on the phone can
connect to `127.0.0.1`, so a local server would expose the Garmin data to
every other app on the device. The bridge has no port at all.

### D4. The store — `rusqlite` + bundled SQLCipher · done

One database file in the app's private storage, encrypted with a key from
the Android Keystore. `rusqlite` with the vendored SQLCipher compiles
SQLCipher from source through the NDK; there is no system dependency to
ship, and OpenSSL is vendored too.

The schema is the one the Python app has always had, and it does not
change: the desktop's export has to open on the phone. The port is checked
against the reference's own schema, table by table and column by column,
by `crates/stride-store/tests/schema_matches_python.rs`.

### D5. The FIT parser — written in-crate · done, **against the original decision**

The decision was to use the `fitparse` crate. **That was wrong, and it was
changed.** The Rust crate called `fitparse` is a *different project* from
the Python `fitparse` the reference uses — the same name, different field
semantics — and it panics on the fixtures. The decoder is therefore written
in-crate, and the three Garmin quirks are the contract:

1. **running cadence is stored halved** — and only for running sports, so a
   cycling cadence of 90 rpm stays 90;
2. **positions are semicircles**, not degrees;
3. **lap speed is sometimes absent**, and it is not invented.

### D6. The Garmin client — `reqwest` + `rustls` · done

`rustls` rather than OpenSSL for HTTP: no system TLS dependency. The Garmin
surface is small — a token exchange, a handful of reads, and a FIT download.

The token is a 24-hour JWT and there are **no refresh tokens**: refreshing
means exchanging the Personal API Key again. On a 401 the client exchanges
once and retries once; on a second 401 it reports `needs_reconnect`. It
never loops.

### D7. The desktop — the same core, serving the same API · done

The Rust core exposes one **request router**. Two hosts use it: the desktop
runs it over HTTP beside the SPA, and the phone calls it in-process through
the bridge. **One** API implementation and **one** SPA.

### D8. The Garmin key — the Android Keystore · done

A non-exportable AES-256-GCM key in the Keystore wraps one credential
envelope in `noBackupFilesDir`. Plain DataStore is not encryption, and
`EncryptedSharedPreferences` is deprecated. The key never enters the
WebView, never enters a log, and never leaves the device except over TLS to
Garmin.

The key is in memory for the length of one pull and no longer: the Kotlin
shell unwraps it, hands it over, and clears it again.

### D9. The harness — a local MCP server inside the app · **the crate is built; no host wires it**

`crates/stride-mcp` holds the **nine** question-shaped tools, over the
router, read-only, bounded, and written so that only the fields an athlete
would want in a model's context are disclosed. The desktop and the phone
both hold a `stride_api::Api` over their own store and would hand it to
this crate the same way.

**What is missing:** neither the desktop binary nor the phone starts the
MCP server yet. The tools exist and are tested; the socket does not.

---

## 3. The onboarding, on the phone

**One screen. Add the key. The sync follows. It says connected.**

| state | what it says |
|---|---|
| validating | "Checking…" |
| pulling | "Syncing…" and a progress line — "12 runs" |
| done | "Connected · 160 runs", and the screen goes away |
| refused | "Garmin did not accept this key." and the field stays |

The key is entered **once**, into a native field. It is validated, wrapped
by a Keystore key and written to `noBackupFilesDir`; it never enters the
WebView. The screen says **"Your data stays on this phone."** and nothing
else about privacy.

The full contract is `docs/phone-simplification.md`.

---

## 4. What the phone shows

**Six destinations.** Today, Plan, Runs, Body, Form and Stats. The cut is
Compare, the training calendar and the server settings — a phone cut, not a
product cut. The desktop keeps all of them.

### v1 reads the data; it does not author it

The phone is a **reader** for the athlete's own history and for the intent
behind it.

| what v1 does | what v1 does not do |
|---|---|
| connect a key, in one native screen | write a plan by hand |
| sync activities, wellness, and the plan it finds | edit a run, rename it, describe it |
| show the plan against what was actually done | keep notes on a run |
| show readiness, with its drivers | author anything at all |

Everything that **authors or edits** — a manual plan editor, editing a run,
notes — is the next iteration, and it is deliberately not designed yet.

### The Plan screen does not need a document

The Plan screen is the reason the app exists, and it must not be empty for
anyone who does not write their plan in a markdown file. A plan is *intent*:
a dated list of sessions with a target. Three sources, and the order
matters:

1. **The plan you already have — inferred from the athlete's own history.**
   Their weekly volume, their rest days, their long-run day, their usual
   session length per weekday; twelve weeks of history gives all of it.
   **Zero setup, and it works for everybody.**

   And it is not a prescription, so the words must not pretend it is. An
   inferred plan is a *baseline*: **"less than usual"**, **"more than
   usual"**, **"a usual day"** — not "missed" and "hit".
2. **The plan you wrote down — Garmin's scheduled workouts.** A
   *prescription*: the athlete wrote them down.
3. **The plan you type in — a manual week.** Real, and a separate piece of
   work: an editor, an API and the storage for it. **Not v1.**

The distinction between the first two is not cosmetic: mixing them would
apply "missed" — a judgement about a plan somebody else wrote — to a plan
nobody wrote at all.

**Work in flight.** The reference implementation is
`backend/app/services/plan_sources.py`; the port is
`crates/stride-core/src/plan_sources.rs`, with the golden fixtures in
`crates/stride-core/tests/fixtures/plan_sources/`. They are being built now
and are not in the published build yet.

---

## 5. What is missing

1. **The MCP server is not started by either host.** The nine tools exist;
   the socket does not.
2. **There is no pairing at all.** The route, the state, the device list
   and the QR code are gone from both implementations. The app has one
   credential — the Garmin API key — and a browser on a network the
   operator has trusted is signed in like any other.
3. **The developer-key and account-login flows are not in the Rust core.**
   `/auth/garmin/developer-key`, `/auth/garmin/login` and
   `/auth/garmin/mfa` answer **501**. The phone does not care — it uses the
   Personal API key and nothing else.
4. **The network route is not in the Rust core.** `/network/status` and
   `/network/route` answer **501** on the desktop. The reference app
   implements the egress switch.

None of the four is on the phone's critical path. The phone is the app, and
the app works.
