# Brief — cross-check the Android port, and write the release notes

*Written by the parent thread, 24 November 2026. **Cross-check first; nothing
is pushed until you have checked everything.** The parent tags and publishes.*

## The situation

The rig-v2 build is in `master` (472 commits, the full gate green). The
**Android app is the primary release target** ("Android remains the primary
release target; desktop is not a second UI" — `desktop/README.md`), and it
ships the **same React UI** through Capacitor with the **same Rust core and the
same router** through the FFI seam. So most of the build reaches the phone
already — **the question is what does not, and what would break.**

The release pipeline (`/.github/workflows/release.yml`) runs on a `v*` tag: the
gate, the web build, the Capacitor staging, the release APK with the real
signing key, `scripts/check_apk.py` (the core in the APK must be cargo's, and
the signature must match the published certificate), and the notes from
**`docs/release-notes-<tag>.md`** — a tag without that file **fails**.

## What to check — the port cross-check

### 1. What actually reaches the phone

* the **web bundle**: the release job builds it from the repository and
  stages it (`webDir: frontend/dist` in `android/app/src/main/assets/
  capacitor.config.json`) — so the new cards, the ladder, the curve, the
  threshold, the ledger's two blocks and the week-against-the-plan card are in
  the APK. **Confirm it**, and confirm the Capacitor library staging picks up
  the new bundle.
* the **FFI**: `stride_call(method, path, body)` dispatches over the **same
  router**, so the new routes reach the phone without registration. Confirm it
  against `android/app/src/main/rust/stride-ffi/src/lib.rs`, and confirm the
  phone's frozen table (`PhoneRoutes.kt`) names every route the app calls —
  `scripts/check-contract-greps.sh` compares it with the router, so the
  parity is mechanical.
* the **schema**: v8 adds tables; the store migrates on open. Confirm the
  phone's open path (`stride_open`) reaches the migration, and that an
  existing installation **upgrades in place** — no reset, no lost history.

### 2. What might still speak the old product

**Search the Android host and the shared code for anything that names the
product as it was before this build**, and fix what is a fact about the old
product:

* the retired **`/pace` page** and the old **wall cards** (24 → 11);
* the **four-versus-five destinations** (the code ships five: Today, Runs,
  Progress, **Profile**, Body) — the Android shell, the launcher, the
  notifications, the backup flow, the file delivery;
* the old **band vocabulary** — *acute*, *chronic*, *form*, *ratio* — in any
  Kotlin string, notification title, or webview title;
* the old **schema version** or any hard-coded count;
* the old **day's four parts** (the night's 23:00–06:00) anywhere a
  notification or a native screen names a time of day;
* anything on the Kotlin side that still names a **zone model's** absence as
  *"No zones yet"* rather than the new register.

### 3. What the phone cannot do yet, and should

* **The twelve-month backfill.** The FFI already exposes
  `stride_garmin_backfill(key, days)` — but **can the athlete reach it from
  the phone's UI?** On the rig we pulled 105 runs and twelve months by calling
  it repeatedly; the phone must be able to do the same, or the phone's ledger
  will be as empty as the rig's was. **Check the phone's path** (Settings →
  Data & connection, or wherever the sync lives), and if there is no door,
  **build one**: a control that runs the backfill, bounded, with the state
  shown. Say what you find and what you build.
* **The athlete's heart rates** must be enterable on the phone (Settings ›
  Training › Your heart rates). Confirm the surface exists in the shared UI
  and that the phone can reach it (no desktop-only door anywhere in it).
* The **timezone offset** — the phone already sets it (`stride_set_utc_offset`); confirm the desktop's new behaviour did not change the phone's.

### 4. What cannot be checked here — say so plainly

The **Android SDK is not on this machine**, so `:app:lintDebug`, the Kotlin
unit tests and an APK build **cannot run here**. They run in the release
pipeline on the tag. So:

* **read** the Kotlin tests and state which of their expectations this build
  invalidates (the route list, the schema version, any count);
* run everything that *can* run — the Rust FFI tests (`cargo test -p
  stride-ffi`, part of the workspace), the shell gates, the viewport matrix
  (the phone's layout, at the phone's widths);
* list, in your report, **exactly what the release pipeline will be the first
  to catch** that this check could not.

### 5. The release notes — `docs/release-notes-v1.1.0.md`

The tag will be **`v1.1.0`** (say so in your report if you think it should be
otherwise; the notes file's name must match the tag). Write them in the house
style of `docs/release-notes-v1.0.0.md` — **what an upgrade really carries,
table by table**:

* what is included, in the athlete's terms: the eleven-card wall, the one
  chart, the ledger and its absence rows, the training paces and the cost
  curve, the threshold's four candidates, the adopted programmes, the week
  against the plan, the zone times in two bases, the day's four parts read in
  the athlete's own hours;
* **the upgrade and privacy note**: the schema moves to v8, new tables, the
  migration is in place, **no reset and no lost history**, the data stays on
  the device, and what the athlete must now enter for themselves (their heart
  rates; a race, if they want the paces);
* **the known limits**, honestly: the repeated-work row needs a workout the
  watch recorded with steps; the 80/20 base is a specification and not yet a
  second programme; the app's bands are derived from the athlete's declared
  heart rates and may differ from the watch's own configuration; the
  condition axis is not built; the ledger reads a bounded number of runs.
* the **back up before you upgrade** line, as the previous notes carry.

## Rules

* **Nothing is published.** Do not tag, do not push a tag, do not touch the
  releases. The parent does that after reading your report.
* **Fix only what is a fact about the old product.** If a change would alter
  behaviour on the phone in a way the plan has not settled, **report it**
  instead of changing it.
* **No new dependencies**; no Garmin writes; no production data in tests.
* The gates, as always: fmt, clippy, the workspace tests, check-deps, the
  contract greps, the secret and privacy scans and their self-tests, the
  history scan, `tokens:check`, `lint:voice`, typecheck, unit, viewport.
* **Commit on `rig/v2`**, and keep the commit boundaries legible — several
  threads have shared this tree.

## Report back (last message, under ~450 words)

* **the port cross-check**, as a table: what reaches the phone, what needed
  work, what you fixed;
* **the twelve-month backfill on the phone**: what you found and what you
  built, and how the athlete reaches it;
* everything on the **Kotlin side that still spoke the old product**, and what
  you did about it;
* **what could not be checked here**, and what the release pipeline will catch
  first;
* the release notes' **outline** and the file's path;
* your recommendation on the version number.
