# Contributing

Issues are welcome. **Features are the maintainer's call** — see [SUPPORT.md](SUPPORT.md). This document
is about the *how*.

## Before you start

Read [PLAN.md](PLAN.md). It is the plan of record: the scope, the architecture, the ten slices, and the
design rules that the code is measured against. Its approved amendments
supersede the [original HTML wireframes](docs/plan.html). Use the
[current screenshot gallery](docs/media/README.md) to understand the implemented
UI; a new screen or capability needs an explicit plan change.

Two rules are not negotiable:

1. **One information architecture.** Today · Runs · Progress · Body — at every width. Compare and the
   Plan week are reached contextually. A screen that cannot exist on the phone does not exist at all.
2. **The voice.** The app informs; it never instructs or judges. Your strings must be typed, catalogued,
   and pass the lint. See [the voice rules](PLAN.md#0-the-voice--vitals-direction-options-no-verdicts).

## Credentials for development

Use synthetic fixtures for normal development. A live provider check needs
explicitly supplied credentials, never a key copied into source, `.env`, tests,
shell commands, logs or an agent transcript.

Use the host's setup UI or a secret manager that injects credentials into the
child process without printing them. The desktop recognizes `STRIDE_GARMIN_KEY`
and `GARMIN_API_KEY`; site-specific vault launchers are not public prerequisites.
An injected key is process-scoped. A key accepted through desktop setup is
deliberately stored in a private host file; Android uses Keystore-wrapped storage.
See [the security model](SECURITY.md) for those distinct persistence boundaries.

`scripts/scan-secrets.sh` is a backstop, not permission to handle a credential
carelessly. Public screenshots, recordings and bug reports must use synthetic
data, including invented HR, HRV and routes, rather than personal observations.

## Building

**One target directory per worktree. Do not share it.**

Cargo's fingerprint for a path package is keyed by the package id, so two
worktrees of the same crate share one fingerprint slot in a shared `target/`.
One worktree's build then marks the other's artefact fresh, and the second
build links the first worktree's `.rlib` — so `cargo test --workspace` fails
with an error that names a function the current worktree does have. A lock
does not fix it: the lock serialises the builds, not the artefacts.

So: let each worktree have its own `target/`, and delete it when the worktree
is finished. (We tried one shared tree to save 14 GB of disk. It cost more
than it saved.)

```bash
# core + desktop
cargo build --release
cargo run -p stride-desktop        # serves the app on loopback

# tests
cargo test --workspace
```

**The phone** (the `android/` shell) and **the app** (the `frontend/` UI) are both in the tree. The
build steps are:

```bash
sdkmanager "platforms;android-36" "build-tools;36.0.0" "ndk;27.2.12479018"
rustup target add aarch64-linux-android
npm --prefix frontend ci
npm --prefix frontend run build
(cd android && ./fetch-capacitor.sh && ./sync-web.sh ../frontend/dist && ./gradlew assembleDebug)
```

Android also needs a compatible JDK and `cargo-ndk`. See
[the Android build/release guide](android/README.md) for signing and native checks.
Build and synchronize the frontend before assembling an APK.

## The gates

Every pull request runs:

| gate | what it protects |
|---|---|
| `cargo test` + clippy | the maths, and the FIT decoder's sentinels |
| corpus integrity | every FIT file matches its frozen SHA-256, and the privacy gate decodes every FIT position |
| the secret scan | no credential, keystore or database in the tree |
| the privacy scan | no real coordinates, names, notes or device names |

Golden parity and the six-width responsive matrix are implemented gates.
`bash scripts/gate.sh` is the authoritative local/CI list; it also checks
frontend types, voice, tests, scanner self-tests and repository history.

A pull request that turns a gate off will not be merged. If a gate is wrong, fix the gate and the code
together.

## The frozen corpus

`fixtures/` holds the golden corpus: the FIT samples and the expected readings, extracted from the
original implementation and **sanitised** — the dates were already synthetic (no shift was needed), the
coordinates were **replaced wholesale** with a synthetic anchor (not jittered), and a fabricated name and
a Garmin id were replaced with placeholders.

* **Do not add real data.** Never a real route, a real name, a real note, a real device name.
* **Do not edit an expectation to make a test pass.** If a number is wrong, the *code* is wrong — unless
  the change is a deliberate contract change, which needs the plan updated in the same pull request.
* **Do not delete a fixture** because it is inconvenient. A fixture that no longer holds is a fixture
  that documents a behaviour nobody wants to lose.

The privacy scan fails the build if a tracked file contains anything that looks like real personal data.

## Commit and pull request style

- One slice or one coherent change per pull request. Do not mix a refactor with a behaviour change.
- Say what changed and why, in the description. Reference the slice or the screen spec.
- If it changes a number, a string, a route shape or the schema, say so explicitly — those are the four
  frozen contracts.
- Keep the diff reviewable: no drive-by reformatting of a file you are not otherwise touching.

## Code style

- Rust: `cargo fmt`. The workspace denies `unwrap`, `expect` and `panic` — these are *clippy* lints, so a
  plain `cargo build` succeeds and it is CI's `-D warnings` that enforces them. A crate may not override
  the workspace lint table (a check asserts it). Test code opts out with a module-level
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` — the one sanctioned place. `unsafe`
  is forbidden outside the implemented FFI crate (a check asserts it).
- The core is **pure**: no I/O, no clock, no randomness. Time is injected. That is what makes the corpus
  reproducible.
- Every query is parameterised. There is no string-built SQL anywhere.
- Comments explain *why*, not what the code says.
