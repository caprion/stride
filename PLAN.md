# Stride v2 — the plan of record

> The full, designed version of this plan — including the **screen-by-screen specification**
> (purpose, why, layout at every width, states, acceptance checks) — is in
> **[`docs/plan.html`](docs/plan.html)**. Open it in a browser.

This is a clean-room re-implementation of Stride in **Rust only**, in a **new open-source repository**,
keeping the mobile app the user already likes. Every screen is designed before code, and every screen is
designed to resize: one information architecture, many widths.

**Status:** the plan of record, and the code is following it. Slice 0 (the contracts, the CI and the
gates) is merged; so are the domain, the store, the Garmin client and the FIT decoder, the router,
the app, the phone's shell and the desktop rig. What is left is the wiring — the router hosted by
both hosts — and the first vertical line: key → sync → one run → today's readout.

### Approved product-reset amendments (2026-11-04)

The user approved a staged, in-place reset retaining the local Rust foundation,
Android-first delivery and original azure/amber identity. The historical design
below remains applicable except where these amendments explicitly supersede it.
This approval is not permission to publish, install, migrate or sync production data.

- Today leads with recorded weekly activity on phone and desktop. Only an
  explicitly imported or typed plan supplies sessions; inferred habits are not
  prescriptions. A session comparison uses its matched actual result, never an
  unrelated last run.
- Independent weekly pace/heart-rate medians remain descriptive. They cannot
  substantiate a matched-effort fitness headline; no such claim is emitted without
  validated paired evidence. Chart windows use calendar weeks and preserve missing
  observations rather than compressing gaps or substituting zero.
- An incomplete listing never authorizes deletion or a successful activity-scan
  timestamp. Wellness failures are surfaced, preserve previous successful coverage,
  and stop subsequent provider requests instead of masquerading as absent readings.
- Initial history targets the last 12 months, arriving incrementally; mobile data
  is permitted and older history is on request. The import now stores dated
  checkpoints independently for activities and wellness. Each job is bounded;
  Android continues unfinished history through one separate WorkManager owner.
  This is implementation progress, not production/device acceptance.
- Azure evolved light is the accepted default, including native setup/data
  screens. Available measurements lead; wholly absent signals stay in collapsed
  disclosure, while historical readings and available activity details remain.
- Phone backup/restore now uses a native user-chosen passphrase independent of
  the original device key and Android document delivery. Separate dated CSV/JSON
  exports are readable, not recovery backups. Isolated recovery tests do not
  establish physical-device document-provider or process-death acceptance.
- Body explores selected dates and 7/28/90/365 calendar-day windows, preserving
  gaps. Today shows recorded-session dates, not inferred rest days. Progress uses
  the core's calendar anchor and offers 52 weeks. Android refreshes the current
  phone offset before requests and sync without reopening the store.
- Run-detail routes are decoded from every valid position in the stored FIT,
  bounded by the response sample limit across the full route with endpoints
  retained. The import summary's first/last GPS metadata is not the route.
  The GPS outline renders locally without tile requests. An explicit Street map
  action enables OpenStreetMap tiles with attribution and disclosure of the
  viewed area, pan/zoom and automatic local-outline fallback on tile failure.

### Runner-first interaction refinement (2026-11-05)

The user accepted Stitch's compact Azure style as a design reference, not its
invented measurements, diagnostics or unverified capability claims. Apply the
shared responsive UI to the signed Android testing APK first; assess PWA
delivery separately afterwards. The same destinations remain available at phone
and desktop widths.

Body puts selected-date sleep stages inline, with dated charts and explicit
missing observations. Today and run detail reuse that presentation, with the
run's night matched to its date. Stages do not establish sleep quality; do not
invent a score or chronological timeline. Pace and heart rate share an elapsed
time cursor; distance-based elevation and untimed route geometry remain separate.

Runs supports an explicit two-run comparison tray. Date filters now search stored
history; text search remains scoped to loaded pages. Progress describes saved
activity history, marks the partial current week
and distinguishes separate weekly medians from matched-effort evidence. Average
session counts are scoped to recorded weeks. Data & connection leads with
local storage and actual Garmin status; diagnostics are secondary and connection
failures are not interpreted as disconnection. Saving and loading failures remain
visible and do not claim success or replace saved records with invented emptiness.

The user authorized publishing a signed GitHub pre-release for physical-phone
testing. This does not establish device acceptance or promote the build to stable.

The rc.3 interaction pass adds shared date/history links, monthly run browsing,
bounded local route previews and preserved history return paths. Progress totals
follow the selected period, and its weekly bars open saved runs. Compare uses an
aligned table, paired desktop charts and compact A/B switches for heart rate or
pace. Settings separates data, preferences, imports and About; desktop backup
opens the host-owned tools rather than an unconfigured export destination.
Desktop/mobile browser acceptance does not establish physical-phone acceptance
or pixel-identical reproduction of the synthetic Stitch prototype.

The initial target is a 365-day lookback, approximately twelve months, inclusive
of today. A foreground pull checks the latest fourteen activity dates and up to
three wellness dates; continuation visits older checkpointed dates. The target
can be extended explicitly to 3650 days without resetting completed history.
Default per-attempt bounds are 20 FIT download attempts, 14 historical wellness
dates, 80 actual HTTP attempts, 32 MiB response-body bytes and 45 seconds. Byte
and elapsed-time limits stop the next request; an already bounded in-flight
response can exceed those soft limits. They are not wire-byte measurements.

Successful no-data days count as checked dates, not measurements or a provider
retention guarantee. Wellness data and its cursor commit together; missing
provider fields preserve existing observations. Activity reconciliation requires
an entirely enumerated dated window. Capped windows shrink, and an unenumerable
single day produces an explicit failure instead of an endless automatic loop.
Partial provider failures remain failures even when some data landed.

The Android `history_days` preference is new. The former `sync_days` value
bounded a single recent pull and is preserved, not reinterpreted or overwritten.
Automatic history continuation requires enabled network and auto-sync, allows
mobile data and waits for sufficient battery and an unlocked phone. WorkManager
backoff paces continuations, so completion is inexact and can span days. A failed
key stops automatic retries; rate limits and connection failures defer instead.
No publish, install or production-data pull is part of verification.

Optional explainable nudges are accepted product direction, but no new nudge
contract or wording is introduced by this slice; existing voice checks remain.

---

## 0. The voice — vitals, direction, options; no verdicts

Not a data dump, and not a set of orders. The app shows the vitals, says which way each has moved against the
athlete's own usual, and lists what is on the table. It draws no conclusion. Enforced in CI.

**1 · Vital — the measurement.** Label, value, unit. Nothing added: *Sleep — 6 h 10 m · HRV — 48 ms ·
Resting HR — 63 bpm.* No score, no band colour, no icon standing in for a word.

**2 · Direction — which way it moved.** A direction word from a **closed set** — *better than usual · about usual ·
could be better* — beside the muted arithmetic (*5 above your usual*) and the direction of travel (*3rd night down*,
*steady*). "Usual" is defined once and drawn on every chart: the median of the last 30 nights, band = median ±
1.5 × MAD. Which side is better is a fact about the measurement (more sleep, higher HRV, lower resting HR) — and
because the band is drawn, the athlete can disagree with it.

**3 · Options — what is on the table.** The plan's own sessions and alternatives, listed unranked and **never
conditioned on the vitals**. *Today — 10 km · easy · from your imported plan · Also in your plan — a 5 km on
Friday, a 30-minute one on Sunday.* The app never says which to pick, and never says "because your sleep was low" —
that link would be the judgement.

**4 · Pattern — the history, in the history screen.** What usually happened to this athlete, as a fact about the
past, always with its sample size: *"the 5 other times your sleep was below your usual, you ran 8.4 km or less."* It lives on
**Body**, over the visible history — not on the morning screen, and never next to a suggestion.

The Progress headline is the factual mirror of the evidence, in one of three wordings — *fitter than 8 weeks
ago*, *the same pace as 8 weeks ago*, *slower than 8 weeks ago* — with the evidence in a clause. It never
claims a change it cannot see.

**Forbidden:** commands (*take the day*, *rest day*, *rest up*, *rest today*, *go easy*, *you should*, *it'd be better to*) · verdicts (*good*,
*bad*, *poor*, *missed*, *behind*, a readiness score) · **conclusions** (the app never combines vitals into a
verdict — it reports each direction and the athlete reads the combination) · ranking the options (*recommended*) ·
shame (red, *should have*).

```
TODAY · Tue 12 May
Rest         could be better · 2 of 3 signals below your usual
            (the count names the side the signals are literally on — "above" when a resting heart rate is the one
             on its worse side, "outside" when they disagree; the sentence must be true as written)
Sleep        6 h 10 m   · could be better · 1 h 10 m less than your usual
HRV          48 ms      · could be better · 14 below your usual · 3rd night down
Resting HR   63 bpm     · about usual · 5 above your usual
Today        10 km · easy · from your imported plan
Also in plan a 5 km on Friday · a 30-minute one on Sunday
Last run     8.4 km · 42 m · Sat 10 May
```

Every user-facing string lives in one catalogue and is typed (`vital`, `direction`, `option`, `pattern`, `status`).
The lint fails the build on banned words (`should`, `must`, `take the day`, `rest day`, `rest up`, `rest today`, `go easy`, `missed`, `bad`,
`poor`, `fail`, `behind`, `ready to`, `recommended`, `better to`, `need to`), requires a `direction` string to come
from the closed set, and requires a `pattern` string to carry its sample size.

| old product | v2 |
|---|---|
| "Take the day, or make it very easy." | "Rest — could be better · 2 of 3 signals below your usual", and the plan's options, unlinked |
| "Keep it easy today." | "Sleep — 6 h 10 m · could be better · 1 h 10 m less than your usual" |
| "You missed Tuesday's 10 km." | "Tuesday — 10 km planned · not done" |
| "0.62 km over your target." | "10.6 km — 0.6 km more than the 10 km target" (and only when the gap is over 10%) |
| Readiness 63/100 | no score exists |

## 1. The product in one page

Stride is a private, local-first running log for one athlete. It shows you **your own numbers against your own
normal**, in plain language — and leaves the decision with you.

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
stride-core    pure domain: plan engine, vitals & direction, story, fitness, compare   (no I/O, no clock)
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
| vitals & direction | never a 0–100 score, never a grade, never a recommendation — a value and a direction word from a closed set, with the band drawn on the chart |
| pattern | the athlete's own history of comparable states; always carries its n; never from fewer than three cases; lives on Body, not Today |
| today's state | vitals first, then the plan's own options. No commands, no verdicts, no conclusions, no link between an option and a vital. Today and Body read the same **values and directions** from the core |
| plan | plain fact: *done* · *not done* · *planned*. Never a red "missed". An inferred baseline never yields "not done" |
| story | descriptive, never evaluative; reads the same primitives the charts read; the feed shows the sentence, the detail shows it plus the why |
| fitness | no trend percentage without a real baseline; consistency is stated, never graded |
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

**Today** leads with the vitals: each with its value, its direction word (*better than usual · about usual ·
could be better*) and the muted arithmetic (*14 below your usual · 3rd night down*), then a **Rest** summary
(*2 of 3 signals below your usual*), then the plan's own session and its other options, unranked and unlinked.
The **distance comparison appears only when the gap exceeds 10%**, stated neutrally ("2.4 km more than the 10 km
target"); within 10% it is absent — 10.2 km against a 10 km target is simply not news, and neither more nor less is
good or bad.

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

CI gates from commit #1: cargo test + clippy (deny `unwrap`/`expect`/`panic`) · **corpus integrity** (FIT
SHA-256) · **secret scan** · **privacy scan**. Later slices add **golden parity** (full engine parity against
the frozen corpus, Slice 2) and the **viewport matrix** (Slice 9). APK-carries-the-core check and the live
security check are release-only.

> The old repo's `android.yml` is **not** fork-safe — its APK job needs the signing secret, so every outside
> contributor's build fails, and it grants `contents: write` to pull requests. v2: `contents: read` globally,
> secrets only in the release job, release only from upstream tags.

Releases ship the signed APK, the desktop binaries, and **generated** release notes (so the notes cannot
describe a deleted flow — the old notes still told people to run a removed pairing command).

## 8. Delivery plan

| Slice | Content | Done when |
|---|---|---|
| 0 | Contract freeze: public repo, corpus extracted + sanitised, fork-safe CI, schema/API frozen | **done** — merged |
| 1 | Thin vertical line: key → sync → one run → today's readout → APK | a real key syncs a real run and the app shows its true numbers |
| 2 | FIT decode + segmentation + store | golden parity on every corpus activity |
| 3 | Vitals & direction engine + pattern engine + string catalogue + voice lint + Body | Today and Body render the same values and directions; the lint forbids every imperative, verdict and ranking word |
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
mine" · **voice: vitals, direction (closed set), options — no verdicts** · **distance comparison only beyond a 10% gap**.

**Still open to change on request:** MIT instead of AGPL · anything from the cut list · the destination names ·
the plan-source set · a native phone UI (a separate project, not v2).

---

## 11 · After the vertical line — the rest of the product

The vertical line (slice 1) is merged and released as **v0.9.0**. The plan engine, the settings, the
notes, the import and the export, and the screens' states are what is left.

### Two rules this stretch earned

**The plan engine has one owner: the core.** `crates/stride-core/src/plan.rs` holds the baseline inference,
the week, the adherence, the results and the distance comparison. The api reads the plan out of the store
and hands it to the core; the frontend renders what the core rendered. The reason is written down because it
has been learned three times in this project: a client that retried *and* a shell that retried; a frontend
that carried a port of `units.rs`; two FFI surfaces. **A rule that lives in two places is a rule that will
drift.**

**The screen inventory is complete; what was missing was the states.** All nine screens of §5 exist. The
states each must cover:

- **empty** — no runs, no wellness, no plan: the screen says what is missing and what to do, in the app's
  voice, with no invented numbers;
- **not enough data** — the honest form of the empty state, where a comparison is impossible;
- **stale** — the data is older than it should be, and the screen says when it was last synced;
- **the plan's states** — a baseline plan, an imported plan and no plan at all, each named as the source it is;
- **a baseline is not a prescription** — a plan inferred from the athlete's own history never produces a "not
  done"; it is what they usually do, not what they were told to do;
- **an unrecognised session is unclassified**, never labelled "easy" by guesswork.

### The delivery plan from here

| slice | what | done when |
|---|---|---|
| **1** | the vertical line — key → sync → one run → today's readout → the phone | **done** (merged; released as v0.9.0) |
| **2** | the plan engine in the core: the baseline, the week, the adherence, the results, the 10% rule | the baseline infers a week from a synthetic history and says how many weeks it used; a thin history yields nothing |
| **3** | the api's plan, settings, notes, import and export — all on the core's engine | every route in the frozen table answers; the import refuses what it cannot read; the shapes are written into the contract |
| **4** | the app's screens: Settings, Connect, and the states on every screen | the viewport matrix passes at six widths; the lint passes; no screen invents a number |
| **5** | the phone's first run, its notifications, and the key's whole life | the key goes in, the pull lands the runs, the app says so; "forget the key" keeps the history |
| **6** | the rig: the connect page, the export, the import, the stranger's README | a stranger clones it, runs it, gives it a key and sees their own runs |
| **7** | the second review, and the release that carries it | an independent review of the whole product, and a release an Instinct can install |

Slices 2 to 6 run in parallel where they touch different crates. 3 and 4 meet at the shapes — the app's
declarations are the contract, and the api is held to them — and 5 and 6 are independent of everything but the
router.
