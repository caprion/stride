# Brief — step 2: the core engines (the maths)

*Written by the parent thread, 24 November 2026. **Implementation.** Step 2 of
the rig-v2 build; `exploration/build-plan.md` has the order and the rules.
Step 1 (the storage) has landed — `race_observations`, `threshold_observations`
and `activity_observations` exist, and the sync captures the source's own
fields with no extra request.*

## What to build — four core modules

Everything here is **the core's arithmetic**, with the **rule it comes from**
named and cited, so that the surfaces can print the rule beside the number
(D22, D24). No new dependencies, no I/O, no clock: every time-dependent thing
is an argument.

### 1. `crates/stride-core/src/vo2.rs` — the effort model

* **VDOT from a race time** — Daniels' relations, already used by the pace
  engine's prototype: `VO2 = −4.60 + 0.182258·v + 0.000104·v²` and
  `%max = 0.8 + 0.1894393·e^(−0.012778·t) + 0.2989558·e^(−0.1932605·t)`, with
  `VDOT = VO2 ÷ %max`, at the distance's pace `v`.
* **The race-pace equivalents** — the pace at a given distance for that VDOT,
  by the same relations inverted, and the **Riegel** form
  (`T₂ = T₁·(D₂/D₁)^1.06`) as the independent check. The prototype's note
  (`exploration/pace-engine.md` §1) has both and the agreement they reach
  (3–15 seconds) — **cite them**.
* **The cost curve** — `%maxVO₂` against time, so that any duration has a
  physiological cost and any pace has a cost.

### 2. `crates/stride-core/src/training_paces.rs` — the ladder's rungs

The rungs, each with its **rule** (the identifier, the citation and the
wording the surface prints), its **heart-rate band** where the method states
one, and its **pace** where the method states one:

| rung | the rule |
|---|---|
| general aerobic | **no pace** — the band only (70–81 % maxHR / 62–75 % HRR) |
| recovery | the 15K–HM race pace **+ 75 s/km**, below 76 % maxHR |
| lactate threshold | the **hour-pace band** — "the pace you could race for about an hour"; the 15K-to-10-mile route is the book's approximation, and the **band is a band** (its fast and slow edges), not a midpoint |
| the long run | **10K pace × 1.20–1.33** at the start, **15K–HM pace × 1.17–1.29** at the finish — and the progression long run's last miles at **LT pace** |
| the goal race | the athlete's own race pace |

Sources: `exploration/pfitzinger-spine.md` (chapters cited) and
`exploration/pace-engine.md` (the chain and the rungs). **Every rule carries
its citation** — the surface prints it beside the number.

**The two anchors are named and never mixed**: the **goal race pace** (the
athlete's own race at the goal distance) and the **LT band** (the hour-pace
band, on the goal distance's span). The band is computed on the athlete's
**own** heart rate and resting heart rate where the band is a percentage of
either — and if the athlete has not given them, the band is **no value with a
reason**.

### 3. `crates/stride-core/src/threshold.rs` — the four candidates

Each candidate is a **source** with a value, a date, and one line on what it
cannot know (which lives in the **notes**, not in the number):

1. **the watch's own zone model** — the fifth zone's floor (the core already
   reads it as the load's threshold);
2. **from a race** — the hour-pace band, computed from the athlete's typed
   race;
3. **from a field test** — the **book's 45-minute time trial**: the pace and
   the heart rate over the **last 30 minutes** of the trial, from the run's
   samples. (The rule is cited; the prototype's note has it.)
4. **typed** — a value, or a **pair** (a heart rate and/or a pace band), with
   the source label from a closed list.

**One choice is in force at a time**, dated, tied to the base it was made for;
the history stays. A candidate with a missing input is **no value with a
reason** — never zero.

### 4. `crates/stride-core/src/stimulus.rs` — the ledger

The kinds (the athlete's own bands, plus the shape rows), the rules, the
**weeks matrix** (per week, per band, the time in that band — absent for a
band a week held none, so absence is drawable as white space), the **last
occurrence** searched over the history, the **coverage** (how many runs carry
heart-rate samples, how many carry the source's own zone minutes), the
**longest run**, and the **repeated work** row (the watch's own workout steps
only — the shape heuristic was rejected, and the note says why).

**The two bases** (D23): the app's own zone time (over the time the athlete
was moving) and the source's own zone minutes (over the whole run) — each
with its own coverage, **never added together**, and the **overlap** where
both exist, so the app can check its own arithmetic and state the distance
between the two without saying which is right.

## Rules

* **The core renders the sentences** — every user-facing string is in
  `crates/stride-core/src/strings.rs`, and `strings.json` is **generated**
  from it, never hand-edited. The UI never writes a sentence.
* **The ban list** (`docs/contract/voice.md`): no verdict, no advice, no
  score, no ranking, no *should*, no *missed*, no red. The **closed direction
  set** stays closed.
* **The surface speaks the runner's language** (D24): *"your zones"*, *"the
  last 7 days"*, *"the difference"*, *"the time you were running"*, *"the
  whole run"*. The notes may use the precise terms.
* **Absence is not zero** — a missing value is `None` with a reason, never
  `0`, never a zero band.
* **Every answer names its period, its model and its source.**
* **No clock**: every date and window is an argument.
* **Citations live in the code** — the rule's identifier and its wording, so
  the surface can print the rule beside the number without the API inventing
  it.
* **Tests for every module**, using the synthetic values from
  `exploration/athlete.js` (the zone model read 3 Nov 2026, max HR 183, the
  10K in 41:36 on 12 Oct 2026, VDOT 49.7) — and the **fixtures** for the FIT
  corpus must not change (`scripts/check-corpus.sh`).
* **The gates**, per commit: fmt, clippy `-D warnings`, `cargo test
  --workspace`, check-deps, the contract greps, the secret and privacy scans
  and their self-tests, `tokens:check`, `lint:voice`, typecheck, unit,
  viewport. **A gate that did not run is a gate that did not pass.**
* **Commit your work** on `rig/v2`, one commit per module, with a message
  saying what changed and which gates passed.

## Report back (last message, under ~450 words)

* the four modules, and for each: the public functions, their signatures, and
  the rules they implement;
* **the numbers**: for the canonical athlete, the VDOT, the equivalents, the
  LT band's two edges, and each rung's value and band;
* how the **two bases** are kept apart in the data, and what the overlap
  returns;
* the **missing-value reasons** you defined, and where each is used;
* the commits and the gates each passed;
* anything the prototypes and the plan disagreed about.
