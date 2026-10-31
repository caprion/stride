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

Four registers, and nothing else:

| register | what it is | example |
|---|---|---|
| **vital** | the measurement | *Sleep — 6 h 10 m* |
| **direction** | which way it moved against your own usual | *could be better · 1 h 10 m less than your usual* |
| **option** | what is in your own plan, unranked | *Today — 10 km · easy · from your imported plan* |
| **pattern** | what usually happened to you, with its sample size | *the 5 other times your sleep was this low, you ran 8 km or less* |

The direction word comes from a closed set — **better than usual · about usual · could be better** —
and every user-facing string is typed and linted in CI. See
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
