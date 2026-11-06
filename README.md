<div align="center">

# Stride

**A private, local-first running log for Android and desktop.**

Your running history, sleep and recovery observations, without a training verdict.

[Android releases](https://github.com/caprion/stride/releases) | [Desktop setup](desktop/README.md) | [Screenshots](docs/media/README.md) | [AGPL-3.0](LICENSE)

</div>

---

## What this is

Stride stores one athlete's history in a local SQLCipher database. Connect the
Garmin Personal API to pull supported activity and wellness data, or import FIT
files. The Rust core powers the same responsive React UI on both hosts: Kotlin
and Capacitor on Android, a local HTTP server on desktop.

There is no Stride account or hosted Stride data store. Saved history can be
viewed offline; Garmin sync needs a network connection. GPS outlines are local.
The optional **Street map** view loads OpenStreetMap tiles and reveals the
viewed area to that provider.

## See it

[![Stride desktop with entirely fictional demo data](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-today.png)](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-desktop-demo.mp4)

**[Watch/download the short desktop demo](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-desktop-demo.mp4)**
or browse the [desktop and compact screenshot gallery](docs/media/README.md).
These show the actual rc.3 UI with 23 invented runs and 39 invented nights.
HR, HRV, sleep and GPS are fabricated, not anonymized personal observations.
The street map uses an illustrative public-park loop, not an actual recorded run.

## What is available

| Screen | What you can explore |
|---|---|
| Today | Recorded weekly distance, linked run dates, the latest run, plan and available body observations |
| Runs | Monthly calendar, inclusive history date filters, FIT import and two-run selection |
| Run detail | Route and optional street map, linked pace/HR exploration, splits, elevation, notes and the matching night's sleep |
| Body | Selected dates, inline sleep stages, HRV and resting HR, with explicit missing observations |
| Progress | Period totals, weekly history and links back to saved runs |
| Compare | Aligned metric differences, heart-rate or pace traces, paired desktop charts and compact A/B controls |
| Plan and Settings | Plan week, local storage, Garmin connection, training preferences and host-owned file workflows |

Today, Runs, Progress and Body remain the four destinations at every width.
Compare and Plan are contextual. The layouts share the compact Azure/amber style.

## What it does *not* do

- It does not tell you what to do. There are no imperatives, no verdicts, no readiness score, no
  traffic light. The app reports a value and which way it moved; you read it and decide.
- It does not judge your training. A session you did not run is "not done", not "missed". A run 2% over
  a target is not news.
- It does not rank you against other people. Every comparison is against **your own** history.

## The voice

Four content registers plus two supporting types: six string types in all.

| type | what it is | example |
|---|---|---|
| **vital** | the measurement | *Sleep — 6 h 10 m* |
| **direction** | which way it moved against your own usual | *could be better · 1 h 10 m less than your usual* |
| **option** | what is in your own plan, unranked | *Today — 10 km · easy · from your imported plan* |
| **pattern** | what usually happened to you, with its sample size | *the 5 other times your sleep was below your usual, you ran 8.4 km or less* |
| **statement** | a fact about a comparison | *2.4 km more than the 10 km target* |
| **status** | connection, sync and empty-state information | *No activities yet* |

`statement` and `status` support the four registers; status covers connect,
sync, staleness and empty states. The direction word comes
from a closed set — **better than usual · about usual · could be better** — and every user-facing string
is typed (`vital · direction · option · pattern · statement · status`) and linted in CI. See
[the voice rules](PLAN.md#0-the-voice--vitals-direction-options-no-verdicts).

## Status

The stable release is **v1.0.0**, promoting the v0.12.0-rc.3 application without
feature changes. The original candidate remains available for reference.

| | |
|---|---|
| Stable | [v1.0.0 APK and release notes](https://github.com/caprion/stride/releases/tag/v1.0.0) |
| Previous candidate | [rc.3 APK and release notes](https://github.com/caprion/stride/releases/tag/v0.12.0-rc.3) |
| Current scope | [PLAN.md](PLAN.md), including the approved responsive-design amendments |
| Garmin data | [capabilities and integration gaps](docs/garmin-api-capabilities.md) |
| Current appearance | [Screenshots and demo provenance](docs/media/README.md) |
| Original design | [Historical v2 wireframes](docs/plan.html), not current screenshots |

Browser checks and a signed APK do not establish physical-phone acceptance.
Upgrade/startup, background work, locking, map gestures and cross-phone recovery
still need device checks. Historical import is bounded and resumable; its
365-day target is not a claim that every requested record has arrived.

This release does **not** deliver a standalone installable PWA. The desktop
browser UI still needs its local Rust host.

## The phone app

Download the stable Android release:

```bash
gh release download v1.0.0 --repo caprion/stride --pattern '*.apk' -D .
```

The release page is the authority on the artefact's name, its size and its
SHA-256 — this file deliberately does not repeat them, because a document
that copies a changing fact is a document that is wrong on the next release.

The release retains the original package and signing certificate. Back up
important history before upgrading; do not uninstall to work around an upgrade
error. The [v0.11.0 migration notes](docs/release-notes-v0.11.0.md) describe the
one-time importer for older tables and its limits. A fresh Garmin pull is not a
substitute for a backup or proof of complete historical coverage.

| | |
|---|---|
| the package | `app.stride.running` |
| the signing certificate | `87:E5:7B:81:F8:ED:7C:FE:1B:A6:0B:8F:B5:E2:E7:3F:B3:E6:41:C8:71:08:46:ED:87:74:1B:7B:9B:23:BC:26` |

Install it on the phone (allow installs from that source), open it, and paste
your Garmin Personal API key into the one native screen that asks for it.
The key is wrapped by a non-exportable Keystore key and never enters the
WebView. The phone runs its Rust core in-process and does not depend on the
desktop. Native passphrase-encrypted recovery backups and dated readable exports
are described in the [Android guide](android/README.md).

## Run on desktop

The desktop host serves the shared app and router against its own local
encrypted database. With Rust and the required native build dependencies:

1. **Build and run it with one command.** After cloning, run
   `cargo run -p stride-desktop`. There is no Python environment, Node, npm,
   or second server process; the app bundle is already in the repository and
   is compiled into the binary.
2. **Give it a Garmin Personal API key.** The first-run Connect page asks for
   the consumer key, tests it with Garmin, and then keeps it on this machine
   in a private file (`0600` on POSIX systems). The key is never put in a URL,
   a log line, an API
   response, or another page.
3. **The trust model is three lines.** The loopback is always answered; an
   address inside a CIDR in `STRIDE_TRUSTED_NETWORKS` is answered; everything
   else is refused with a message naming that variable. To use the rig from a
   phone on the athlete's own network, trust the phone's CIDR and serve TLS
   with a certificate the phone trusts. The full command is in the
   [rig guide](desktop/README.md).
4. **Use host-owned backup tools.** Data & connection links to `/rig` for
   desktop export and merge import. Desktop export directories include an
   export-only key and must stay private. They are not the Android
   passphrase-recovery archive format; do not assume interchangeability.

See [the desktop guide](desktop/README.md) for the exact first-run steps, the
network command, what an export contains, and what the import refuses.

## Building

The core is Rust; the UI is React; the phone shell is Kotlin + Capacitor.

```bash
# the core
cargo build --release

# the desktop rig (one binary, serves the app on loopback)
cargo run -p stride-desktop
```

Android builds need the NDK and the Android SDK — see
[CONTRIBUTING.md](CONTRIBUTING.md).

## Contributing

Issues are welcome. **Features are the maintainer's call** — see [SUPPORT.md](SUPPORT.md). Pull requests
are welcome if they keep the gates green and the voice intact.

Before you open one, read [CONTRIBUTING.md](CONTRIBUTING.md) and [the plan of record](PLAN.md).

To report a bug, leave feedback, or make a request, use the
[issue templates](https://github.com/caprion/stride/issues/new/choose):
[a bug](https://github.com/caprion/stride/issues/new?template=bug_report.yml),
[feedback](https://github.com/caprion/stride/issues/new?template=feedback.yml), or
[a request](https://github.com/caprion/stride/issues/new?template=feature_request.yml).
Redact before you attach anything — this is a public tracker.

## Licence

**[AGPL-3.0](LICENSE).** Its source-sharing obligations also apply to modified
versions offered over a network. The licence does not prohibit hosted forks;
local-first storage is this project's product choice.
