# Brief — step 4b: the core's plan module (the programmes, adoptable)

*Written by the parent thread, 24 November 2026. **Implementation.** Step 4b of
the rig-v2 build; `exploration/build-plan.md` has the order and the rules.
The data is ready: `exploration/plan-templates.json` — four programmes, cell by
cell, verified (54 week-rows, 378 days, every week reconciling). The spec is
`exploration/plan-bases-spec.md` (its decisions B1–B5 stand).*

## What to build

### 1. `crates/stride-core/src/plan_bases.rs` — the catalogue

The four programmes as a **versioned catalogue** inside the core, and the
model of a template, a week and a session:

* **the programme** — its id, its label ("10K tier 1"), its **method**
  ("Pfitzinger" — so the app can show whose method the athlete is
  following), the goal distance, the 12 weeks (counted down from 11 to 1, then
  race week), the **entry requirement** (the weekly mileage range the tier
  expects), the tune-up races, and the post-race weeks;
* **the week** — its number, its printed volume, and its **seven days**;
* **the day** — the **kind** in the book's vocabulary (*rest or cross-training
  · GA · GA+speed · endurance · progression long run (last k miles at LT pace)
  · LT · VO₂max · speed repetitions · recovery · race pace · the tune-up
  race*), its distance, and the **detail** the cell carries ("3 × 8 min,
  jog 3"; "6 × 12 s uphill + 6 × 100 m strides").

**Where the data lives.** Put the canonical copy **inside the crate**
(`crates/stride-core/src/plan_bases.json`, included with `include_str!` and
parsed at read time), and keep `exploration/plan-templates.json` as the
verification copy — with a **test that the two agree**, so the data and the
code cannot drift (the same discipline the prototypes have with
`athlete.js`). If the crate already depends on a JSON parser, use it; **no
new dependency**.

### 2. The session kinds

The core's `plan::SessionKind` must carry the book's kinds. Extend it in the
core's own style, with the **catalogue strings** in
`crates/stride-core/src/strings.rs` (and `strings.json` **generated**, never
hand-edited). The kinds are the **method's** words; the runner-facing wording
for them is the app's (D24 — "the kinds of run").

### 3. The adoption — `adopt(...)`

Given the athlete's choices and facts, produce an **adopted plan**: dated
sessions with their kinds and their paces, and the record of what it was
paced with.

* **B1** — a template is a versioned catalogue; an adopted plan is a **dated
  copy** of it.
* **B2** — the **tier is the athlete's**. The app states the tier's **entry
  requirement** beside the athlete's own weekly mileage and **never suggests**
  one (D8, D24).
* **B3** — the **race date anchors the plan**: the race week is the twelfth,
  and the race falls on the book's day; if the athlete's race is on another
  day, the race week alone shifts.
* **B4** — the **paces are worked out once, at adoption**, from the threshold
  **in force that day**, and stored with that threshold's **value and date**.
  **A later threshold does not re-pace a running plan** — and the plan says
  which threshold it was paced with.
* **B5** — a race day's printed miles are the **day's mileage**, not the
  race's distance.

The paces come from `training_paces.rs`: the GA day has **no pace** (the band
only); the LT session is paced at the **LT band**; the long run at its start
and finish paces; the recovery run at the recovery pace; the speed work at
the race pace. Where a session's pace cannot be worked out — no threshold
chosen, no race entered, no goal distance — the session carries the **kind
and the distance** and **no pace**, with the reason.

### 4. The plan's own reading

A function that answers, for a given week: the sessions with their kinds,
their distances, their paces (where they have one), the plan's source, the
**threshold the plan was paced with**, and the **entry requirement beside the
athlete's own mileage**. All of it in the core's catalogue, in the runner's
language.

### 5. The 80/20 base

**Not a template.** Its schedule has not been read cell by cell, so it stays a
spec (`exploration/plan-bases-spec.md`). Say in the code's comments why the
catalogue holds four programmes and not five — the refusal is the point.

## Rules

* **The core renders the sentences.** Every user-facing string is in the
  core's catalogue; the UI never writes one.
* **The ban list** (`docs/contract/voice.md`) and the **closed direction
  set**: no verdict, no advice, no score, no ranking, no *should*, no
  *missed*, no red.
* **The surface speaks the runner's language** (D24).
* **Absence is not zero** — a session with no pace is a session with no pace,
  with the reason; never a zero pace.
* **Every answer names its period, its model and its source** — the plan's
  source, the threshold's value and date.
* **No clock**: every date is an argument.
* **The gates**, per commit: fmt, clippy `-D warnings`, `cargo test
  --workspace`, check-deps, the contract greps, the secret and privacy scans
  and their self-tests, `tokens:check`, `lint:voice`, typecheck, unit,
  viewport. **A gate that did not run is a gate that did not pass.**
* **Commit your work** on `rig/v2`, one commit per piece, with a message
  saying what changed and which gates passed.

## Report back (last message, under ~450 words)

* the module's public functions and their signatures;
* how the catalogue is stored, and how the test keeps it in step with
  `exploration/plan-templates.json`;
* the **adopted plan** for the canonical athlete — the tier, the start date,
  the race, and **week 3's seven sessions with their paces**;
* which sessions have **no pace**, and the reason each one gives;
* the session kinds you added, and how the runner-facing wording reads;
* the commits and the gates each passed.
