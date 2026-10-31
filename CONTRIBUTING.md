# Contributing

Issues are welcome. **Features are the maintainer's call** — see [SUPPORT.md](SUPPORT.md). This document
is about the *how*.

## Before you start

Read [PLAN.md](PLAN.md). It is the plan of record: the scope, the architecture, the ten slices, and the
design rules that the code is measured against. The screen specifications in
[docs/plan.html](docs/plan.html) are the contract for the UI — if a screen does not exist there, do not
invent one in a pull request.

Two rules are not negotiable:

1. **One information architecture.** Today · Runs · Progress · Body — at every width. Compare and the
   Plan week are reached contextually. A screen that cannot exist on the phone does not exist at all.
2. **The voice.** The app informs; it never instructs or judges. Your strings must be typed, catalogued,
   and pass the lint. See [the voice rules](PLAN.md#0-the-voice--vitals-direction-options-no-verdicts).

## Building

```bash
# core + desktop
cargo build --release
cargo run -p stride-desktop        # serves the app on loopback

# tests
cargo test --workspace
```

**The phone** needs the Android SDK and the NDK:

```bash
sdkmanager "platforms;android-36" "build-tools;36.0.0" "ndk;27.2.12479018"
rustup target add aarch64-linux-android
cd android && ./gradlew assembleRelease
```

**The UI** needs Node 22:

```bash
npm --prefix frontend ci
npm --prefix frontend run build
```

## The gates

Every pull request runs:

| gate | what it protects |
|---|---|
| `cargo test` + clippy | the maths, and the FIT decoder's sentinels |
| golden parity | every number matches the frozen corpus |
| the viewport matrix | six widths, one information architecture |
| the secret scan | no credential, keystore or database in the tree |
| the privacy scan | no real coordinates, names, notes or device names |

A pull request that turns a gate off will not be merged. If a gate is wrong, fix the gate and the code
together.

## The frozen corpus

`fixtures/` holds the golden corpus: the FIT samples and the expected readings, extracted from the
original implementation and **sanitised** — dates shifted, coordinates jittered, names and notes removed.

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

- Rust: `cargo fmt`, and the workspace lints are strict on purpose — `unwrap`, `expect` and `panic` are
  denied in library code, and `unsafe` is allowed only in the FFI boundary.
- The core is **pure**: no I/O, no clock, no randomness. Time is injected. That is what makes the corpus
  reproducible.
- Every query is parameterised. There is no string-built SQL anywhere.
- Comments explain *why*, not what the code says.
