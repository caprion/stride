# Stride v2 — the plan of record

> The full, designed version of this plan — including the **screen-by-screen specification**
> (purpose, why, layout at every width, states, acceptance checks) — is in
> **[`docs/plan.html`](docs/plan.html)**. Open it in a browser.

This is a clean-room re-implementation of Stride in **Rust only**, in a **new open-source repository**,
keeping the mobile app the user already likes. Every screen is designed before code, and every screen is
designed to resize: one information architecture, many widths.

**Status:** plan of record. No product code written yet. Slice 0 is next, on approval.

---

## 1. The product in one page

Stride is a private, local-first running log for one athlete. It answers two questions: **what should I do
today**, and **what did that last run actually mean**. Everything else is evidence for those answers.

- **Loop:** Plan → Execute → Understand → Adapt.
- **Promise:** "Your data stays on this phone." One credential — a Garmin Personal API key — in the Android
  Keystore. No account, no cloud, no server.
- **Four destinations, at every width:** Today · Runs · Progress · Body. Compare and the Plan week are
  reached **contextually**, never as extra tabs.
- **The rule that kills drift:** *if a screen cannot exist on the phone, it does not exist.* The desktop is the
  same four destinations with more room; it never grows features of its own.

## 2. Scope

**In:** Connect (Garmin key) · Sync (incremental) · Today · Runs · Activity detail · Progress · Body ·
Plan week · Compare · FIT import (any watch) · Settings (goal, pause, note, connect, about).

**Out (deferred, with reasons):** Garmin account login/MFA and the scheduled-workout calendar · MCP/agent
harness · running-form fingerprint page · gear · heatmap · tiles proxy · monthly stats table · HR-zone
pages · network egress routing · phone plan editor · desktop-only screens.

## 3. Architecture

```
stride-core    pure domain: plan engine, recovery, story, fitness, compare   (no I/O, no clock)
stride-ingest  Garmin Personal API client · FIT decoder · segmentation
stride-store   SQLCipher schema, migrations, repositories, FIT blobs
stride-api     the one router · DTOs · the FFI surface · the trust gate
stride-testkit frozen golden corpus · parity differ · UI fixtures
desktop        one binary: embedded SPA + router + trust model
android/…/stride-ffi   UniFFI bindings, panic catching, JSON in / JSON out
```

- The dependency graph is a DAG: core ← ingest, core ← store, api → {core, ingest, store}. No cycles.
- **One core, two hosts:** the phone reaches it in-process through the FFI; the desktop reaches it over loopback
  HTTP. Neither can disagree about a number.
- **FIT is the source of truth.** The original file is stored; derived series are recomputable.
- **Views are computed on read** from stored primitives — a changed engine needs no re-sync, and a stored score
  can never rot.
- **The old Rust core is the oracle** during the migration; Python is retired and its outputs frozen as JSON
  fixtures first.

### The engine contract

| engine | contract |
|---|---|
| recovery | never a 0–100 score; deviations graded (one signal ≠ several) |
| today's call | exactly one sentence; Body and Today read the same sentence from the core |
| plan | "missed" only against a prescription — an inferred baseline never produces one |
| story | a sentence's numbers are the same numbers the charts show |
| fitness | no trend percentage without a real baseline; consistency is forgiving |
| compare | the basis is always stated; swapping A and B flips every delta |

## 4. The screen system — one IA, every width

**Bands:** compact `<600` (bottom tabs) · medium `600–1023` (top bar, column capped 680 px) ·
wide `≥1024` (left rail, content column 880 px).

Bands are decided by **container** width, not viewport — split-screen, unfolded foldables and resized windows
all land correctly without a reload.

**Reflow rules:** destination set constant · no page-level horizontal scroll · tap targets ≥44 px at compact ·
safe-area insets honoured · `dvh` not `vh` · no device sniffing for layout (a build flag only for genuinely
native capabilities).

**Viewport matrix gate** (six widths, every PR): 360×640 · 390×844 · 430×932 · 768×1024 · 1024×768 · 1280×800.
Each asserts: no horizontal scroll, no clipped labels, tap targets, **identical destination set**, charts
legible, safe-area applied.

## 5. Screens (specified in full in `docs/plan.html`)

Today · Runs · Activity · Progress · Body · Plan week · Compare · Connect · Settings — each with purpose, what
it must never do, content blocks, layout per width, states, and acceptance checks.

**Compare is the canonical form to get right:** metric rows (`metric · A · B · Δ`) at every width; two columns
and side-by-side charts only when there is room; an A/B chart toggle on narrow screens; the basis always named.

## 6. Security and trust

- Garmin key: native field → non-exportable Keystore key → `noBackupFilesDir`. Never in the WebView, the
  database, or a log. The 24 h token is disposable: a 401 triggers exactly one re-exchange and one retry.
- Database: SQLCipher, key from the caller, never logged/returned/persisted; every query parameterised.
- **Desktop trust model:** loopback always allowed · explicitly trusted CIDRs via
  `STRIDE_TRUSTED_NETWORKS` · everything else refused with a message naming the variable. No pairing codes, no
  QR, no device tokens — ever.

## 7. Open-source posture

**AGPL-3.0.** Public repository **from day one** (no history scrub later — the old repo's history contains
personal data from the demo-seed incident). `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`,
`SUPPORT.md` ("issues welcome, features are mine").

CI gates from commit #1: cargo test + clippy (deny `unwrap`/`expect`/`panic`) · **golden parity** ·
**viewport matrix** · **secret scan** · **privacy scan** · APK-carries-the-core check and the live security
check (release only).

> The old repo's `android.yml` is **not** fork-safe — its APK job needs the signing secret, so every outside
> contributor's build fails, and it grants `contents: write` to pull requests. v2: `contents: read` globally,
> secrets only in the release job, release only from upstream tags.

Releases ship the signed APK, the desktop binaries, and **generated** release notes (so the notes cannot
describe a deleted flow — the old notes still told people to run a removed pairing command).

## 8. Delivery plan

| Slice | Content | Done when |
|---|---|---|
| 0 | Contract freeze: public repo, corpus extracted + sanitised, fork-safe CI, schema/API frozen | every gate runs on empty crates; the old repo is untouched |
| 1 | Thin vertical line: key → sync → one run → one Today sentence → APK | a real key syncs a real run and the app tells the truth about it |
| 2 | FIT decode + segmentation + store | golden parity on every corpus activity |
| 3 | Recovery + the one graded sentence + Body | Today and Body return the byte-identical sentence |
| 4 | Plan engine (sources, active block, adherence, results) | adherence matches the old core number-for-number |
| 5 | Runs feed + activity detail | every row carries a story sentence |
| 6 | Progress + fitness + bests | no percentage without a baseline |
| 7 | Compare + FIT import + dedupe | deltas flip on swap; a duplicate import adds no row |
| 8 | Settings, goal, pause, notes, export/import | the desktop export opens on the phone |
| 9 | Responsive pass + viewport matrix gate | all six widths pass; identical IA everywhere |
| 10 | Release: signed APK, binaries, docs, first public tag | a stranger clones, builds and runs it |

Rules: the old core is the oracle until parity is 100% · one slice at a time · every slice ships and is gated ·
no Python in the tree · acceptance is a golden assertion wherever possible.

## 9. Defaults applied

AGPL-3.0 · public from day one · old repo kept private as the oracle, sanitised public archive at the end ·
Rust only · five crates · desktop as dev rig + community tool (loopback + CIDR allowlist) · four destinations
everywhere with Compare and Plan contextual · Garmin account flow out · MCP out · "issues welcome, features are
mine".

**Still open to change on request:** MIT instead of AGPL · anything from the cut list · the destination names ·
the plan-source set · a native phone UI (a separate project, not v2).
