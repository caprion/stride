# Brief — step 5: the frontend cards, wired to the routes

*Written by the parent thread, 24 November 2026. **Implementation.** Step 5 of
the rig-v2 build; `exploration/build-plan.md` has the order and the rules.
Steps 1–4 have landed: the storage, the core engines, the API routes, and the
plan module.*

## What to build

Every card below exists as a **prototype** — build the picture, not a new
design. The prototypes are the specification, and they are already in the
product's language (D24).

| where | the card | the design | the route |
|---|---|---|---|
| Progress · **The pace** | the **ladder**: the anchors, the rungs with their values, their bands and their rules | `exploration/pace-engine.html` | `/insights/paces` |
| a drill-through panel from **The pace** | the **cost curve**: the athlete's races, the method's sessions, the field test's marker, the equivalents | `exploration/pace-engine.html` §8 | `/insights/pacing-curve` |
| Progress · **Where the time went** | the ledger's **two blocks** — the app's own zone time, and the watch's own zone minutes — with the overlap | `exploration/two-bases.html`, `exploration/stimulus-ledger.html` | `/insights/stimulus` |
| Settings → **Training** | the **threshold**: the four candidates, the choice, the dated history | `exploration/threshold-setup.html` | `/insights/threshold`, `/athlete/threshold` |
| Plan | **adopting a programme**: the four, the tier, the entry requirement beside the athlete's own mileage, the start date, the race | `exploration/plan-bases-spec.md` | `/plan/bases` |
| Plan · the week | the **week with its paces**: the kinds of run, the distances, the paces, and the threshold the plan was paced with | the plan prototypes | the plan route |
| Today · the plan's options | the sessions of the day, with the plan's pacing | `exploration/beta.html` | the plan route |

## The rules that shape the cards

* **Picture first, sentence second** (D11). The ladder is a chart with a
  caption; the threshold's candidates are four cards of value, source and
  date; the ledger's absence is **white space**.
* **Explanation is not product content** (D22) — the card carries the picture,
  the labels, the numbers, the period, and the reason something is missing.
  **What replaces the prototypes' notes layer:** a **details** affordance —
  the source line's click opens the model's detail (as it does now), and a
  rung's rule opens beside it, with its citation. The explanation lives
  *there*, on demand, and nowhere else.
* **The surface speaks the runner's language** (D24) — *"your zones"*, *"the
  last 12 weeks"*, *"the time you were running"*, *"the whole run"*, *"the
  plan you follow"*, *"the kinds of run"*.
* **The source line is hoisted** — once per screen, naming the zone model and
  its read day, with the list behind a click.
* **Absence is not zero** — an empty row says what is missing and where it
  would come from.
* **Azure and amber, no red.** No colour marks a problem; no score, no grade,
  no traffic light.
* **The strings come from the core's catalogue** — `frontend/src/i18n/catalogue.ts`
  is CI-checked against the core's `strings.json`. The UI never writes a
  sentence.
* **The fixtures regenerate** from the router (`frontend/scripts/export-fixtures.mjs`),
  and the tests are updated to the new shapes.

## Also

* **The pace page's old route** (`/insights/pace`) is retired — its panels
  live in the pace card now. Remove what the cut retired, and keep the
  route table honest (the phone's bridge table is a copy, and the parity
  check compares them).
* **`threshold.limit.*`** ships in the catalogue as **statements**: they must
  stay **off the card** — they are the notes, and the card shows the value,
  the source and the date.
* **The 80/20 base is not in the picker** — only the four programmes are.

## The gates

Per commit: fmt, clippy `-D warnings`, `cargo test --workspace`, check-deps,
the contract greps, the secret and privacy scans and their self-tests,
`tokens:check`, `lint:voice`, typecheck, unit, viewport. **A gate that did
not run is a gate that did not pass.**

**Commit your work** on `rig/v2`, one commit per card.

## Report back (last message, under ~450 words)

* the cards, and where each one now lives;
* for the ledger: how the **two blocks** are drawn so nobody adds them in
  their head, and how the overlap reads;
* for the threshold: what the card shows, and what it deliberately does not;
* for the plan: how the **entry requirement** sits beside the athlete's own
  mileage, and how the paces carry the threshold they were worked out from;
* the fixtures and tests updated;
* the commits and the gates each passed.
