<div align="center">

# Stride

**A private, local-first running log. Rust only. No account, no cloud, no server.**

Your numbers, against your own normal — and you decide what to do with them.

[the plan of record](PLAN.md) · [the full design, screen by screen](docs/plan.html) · [AGPL-3.0](LICENSE)

</div>

---

## What this is

Stride is a running log for **one athlete**. It syncs a Garmin watch over the
**Connect Personal API** (one long-lived Personal API key), stores everything in a local encrypted
database, and shows you where you stand: sleep, HRV and resting HR **against your own history**, your
plan, and what you actually ran.

There is no cloud service, no account to create, and nothing to sign up for. It works on a plane.

## What it does *not* do

- It does not tell you what to do. There are no imperatives, no verdicts, no readiness score, no
  traffic light. The app reports a value and which way it moved; you read it and decide.
- It does not judge your training. A session you did not run is "not done", not "missed". A run 2% over
  a target is not news.
- It does not rank you against other people. Every comparison is against **your own** history.

## The voice

Four registers, plus one supporting `status` type — five string types in all:

| register | what it is | example |
|---|---|---|
| **vital** | the measurement | *Sleep — 6 h 10 m* |
| **direction** | which way it moved against your own usual | *could be better · 1 h 10 m less than your usual* |
| **option** | what is in your own plan, unranked | *Today — 10 km · easy · from your imported plan* |
| **pattern** | what usually happened to you, with its sample size | *the 5 other times your sleep was this low, you ran 8 km or less* |

`status` is the fifth string type — connect/sync/staleness and empty-state lines. The direction word comes
from a closed set — **better than usual · about usual · could be better** — and every user-facing string
is typed (`vital · direction · option · pattern · status`) and linted in CI. See
[the voice rules](PLAN.md#0-the-voice--vitals-direction-options-no-verdicts).

## Status

**v2 is being built in the open, from a clean sheet.** The design is finished and specified; the code is
coming, one vertical slice at a time.

| | |
|---|---|
| design | [PLAN.md](PLAN.md) — features, architecture, delivery plan |
| screens | [docs/plan.html](docs/plan.html) — every screen, every width, before any code |
| slices | 0 → 10, each ending with a working app and green gates |

The desktop is a **development rig and a community tool**, not a product surface: it is the same Rust
core, one binary, and the phone app is the product.

## The phone app

**If you were using Stride before, the app you had is still here.**

v2 is being built, slice by slice, and until it is finished the phone app
people already have is **v0.8.0** — the last v1 build. It is published in
this repository, under the same release tag, and it is signed with the same
certificate, so the v2 builds that follow will install over it on the phone
without an uninstall and without losing your data.

```bash
gh release download v0.8.0 --repo caprion/stride --pattern '*.apk'
```

| | |
|---|---|
| the APK | **8.4 MB** (`stride-v0.8.0.apk`, arm64-v8a) |
| the package | `app.stride.running` · versionName `0.8.0` |
| SHA-256 | `095ff9cdfebccba9c470444d23bbdaa71b63ec28268835c9200f2c9a0e858f84` |
| signing certificate | `87:E5:7B:81:F8:ED:7C:FE:1B:A6:0B:8F:B5:E2:E7:3F:B3:E6:41:C8:71:08:46:ED:87:74:1B:7B:9B:23:BC:26` |

Install it on the phone (allow installs from that source), open it, and paste
your Garmin Personal API key into the one native screen that asks for it.
The key is wrapped by a non-exportable Keystore key and never enters the
WebView. There is no server to reach, no account to create and nothing to sign
up for — it works on a plane, with this VM switched off.

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

## Licence

**AGPL-3.0.** Use it, fork it, improve it — but if you offer it over a network, publish your source.
That is what keeps the "no cloud version of Stride" promise true rather than aspirational.

The product name and the promise in the README are not covered by the licence; see
[the plan](PLAN.md#8-open-source-posture).
