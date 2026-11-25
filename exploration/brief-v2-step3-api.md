# Brief — step 3: the API routes

*Written by the parent thread, 24 November 2026. **Implementation.** Step 3 of
the rig-v2 build; `exploration/build-plan.md` has the order and the rules.
Steps 1 and 2 have landed: the storage (schema v6) and the core engines
(`vo2.rs`, `training_paces.rs`, `threshold.rs`, `stimulus.rs`).*

## What to build

### The read routes

| route | what it answers |
|---|---|
| `GET /insights/paces` | the **ladder**: the rungs, each with its value, its heart-rate band, its rule and its citation; the anchors (the goal race pace and the LT band); the source lines; the missing-value reasons |
| `GET /insights/pacing-curve` | the **cost curve**: `%maxVO₂` against time, the athlete's own races as points, the method's sessions as bands, the field test's marker, the pace equivalents |
| `GET /insights/threshold` | the four **candidates** (each with its value, its source and its date), the **choice in force**, and the dated **history** |
| `GET /insights/stimulus` | the **ledger**: the weeks × bands matrix, the last occurrence per kind, the coverage, the longest run, the repeated work, and the **two bases** with the overlap |

Each route takes the same window parameter the rest of the Progress wall uses
(`weeks`, the calendar-week window through today) and names the window it
answers for. The **source line** is hoisted: one per screen, naming the zone
model and its read day, with the list behind a click (Z already ships
`zones.source` and `zones.models` — reuse them).

### The write routes (the athlete's own facts)

| route | what it does |
|---|---|
| `POST /athlete/races` · `GET /athlete/races` | the athlete's typed races: the distance, the time, the date, the label from a closed list, and the name |
| `POST /athlete/threshold` | the athlete's **choice** of a threshold — which candidate, or a typed value or pair — dated, tied to the base it was made for |
| `GET /plan/bases` | the **programmes** — the adoptable Pfitzinger 10K and half-marathon templates, with their entry requirement, their length and their weeks (step 4 lands the data; this route serves it) |

**The athlete's own facts are never synced.** They are entered in the app and
stored locally. Every write carries the same discipline the wellness
observations have: the value, the day it was entered, the day it last
changed, and the source.

**A choice with no value is refused**, and so is a second choice that would
leave **two in force** — one is in force at a time, and the older ones stay
in the history.

## The things that will bite

* **The route table is frozen, and the phone's bridge carries a copy of it.**
  `scripts/check-contract-greps.sh` compares the router's patterns with
  `PhoneRoutes.kt`'s table — so **every new route must be added to the
  bridge's table too**, or the gate fails. The phone may not call the new
  routes yet (the pace page is the precedent: a route with no screen), but
  the table must name them. Update `PhoneRoutesTest.kt` with it.
* **The contract**: `docs/contract/api-paths.md` is frozen — a change travels
  with a `PLAN.md` note in the same change, and the "shapes the app does not
  read" list must be corrected if any of the new routes are not read by the
  app.
* **The decode cost.** The ledger reads every run in the window, and each run
  is a FIT file decoded on read — a 12-week window is ~60 decodes per
  request. The pace route has the same shape of cost today. **Implement the
  decode cache** (the plan's C3: the single choke point at
  `crates/stride-api/src/routes/series.rs::decoded`) so the reads are
  bounded — a bounded cache keyed by the activity's identity, with the
  budget and the eviction stated. If the cache cannot be made correct and
  bounded, **say so** rather than shipping an unbounded read.
* **The strings** come from the core's catalogue (`crates/stride-core/src/
  strings.rs`); `strings.json` is **generated**, never hand-edited. The
  frontend's `frontend/src/i18n/catalogue.ts` is CI-checked.
* **The ban list** and the **closed direction set** are unchanged.

## The rules that bind

* **Facts, never verdicts.** No advice, no score, no ranking, no *should*,
  no *missed*, no red.
* **The surface speaks the runner's language** (D24): *"your zones"*, *"the
  last 12 weeks"*, *"the time you were running"*, *"the whole run"*.
* **Absence is not zero** — a missing value is `None` with a reason.
* **Every answer names its period, its model and its source.**
* **The gates**, per commit: fmt, clippy `-D warnings`, `cargo test
  --workspace`, check-deps, the contract greps, the secret and privacy scans
  and their self-tests, `tokens:check`, `lint:voice`, typecheck, unit,
  viewport. **A gate that did not run is a gate that did not pass.**
* **Commit your work** on `rig/v2`, one commit per route group.

## Report back (last message, under ~450 words)

* the routes, their parameters and their response shapes;
* the **decode cache**: what it is keyed on, what its bound is, and the
  evidence that the ledger's read is bounded;
* the bridge and the contract changes, and how the frozen-file rule was met;
* the missing-value reasons each route can return;
* the commits and the gates each passed;
* anything you had to decide that the plan did not settle.
