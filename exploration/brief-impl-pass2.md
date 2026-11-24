# Brief — cleanup Pass 2: the surface cut (implementation)

*Written by the parent thread, 24 November 2026. **Implementation.** Pass 1
is committed (`f5cc73c`); work on top of it.*

## What is authorised

The user approved the surface cut, and then saw the wall **redrawn as
pictures** (`exploration/cards-as-pictures.html`) and the run detail's
**consolidated chart** (`exploration/consolidated-chart.html`) and approved
those. The end-to-end (`exploration/end-to-end.html`) is the assembled
reference.

**Pass 2, from `exploration/cleanup-plan.md`:**

| step | what |
|---|---|
| **F3** | the Progress wall: **24 cards → 11** (six on Progress, five on the athlete profile), in the order and with the wording `exploration/surface-inventory.md` §6 gives |
| **F1** | the **radar** retires (`ProfileRadar.tsx` and its axis list) — the seven **facets stay**, drawn as the gauges in the cards-as-pictures prototype |
| **F2** | the **`/pace` page** retires (`PacePage.tsx` and its route) — its two panels fold into Progress's *The pace* card; eighteen `pace.*` catalogue keys die with it |
| **S2** | the **dead catalogue keys** go (the plan lists them) |
| **D11 / §2.9** | the six cards that were **sentence-only** are redrawn as pictures, per `exploration/cards-as-pictures.html` |

Plus the two prototype-driven pieces:

* the **run detail's one chart** — pace, heart rate and altitude on one
  cursor, toggled by the user; the **zone bands stay behind the heart-rate
  line**; the effort table keeps the zone numbers, the *not moving* bucket
  and the model's source line.
* the **ledger card** — "Where the time went" — built from
  `exploration/stimulus-ledger.md`: the weeks × bands matrix with the
  absence drawn as white space, the longest run's bar series, the repeated
  work's marks, and the **source line hoisted once per screen**.

## The rules that decide how it looks

* **D22 — explanation is not product content.** The surface carries the
  picture, its labels, the numbers, the period, and the reason something is
  missing. Every line that explains the *drawing* lives in the notes, not on
  the card. The test: *if the line would still make sense on a card showing a
  different measurement, it is explanation — cut it.*
* **The source line is hoisted** — one per screen, naming the zone model and
  its read day, with the list behind a click (Z's `zones.source` /
  `zones.models` already carry it). No card repeats it. An answer that used a
  **different** model says so in place.
* **Picture first, sentence second** — and **absence is drawn**: white space,
  a missing marker, an empty bar. Not a clause.
* **The ban list applies** (`docs/contract/voice.md`): no verdict, no score,
  no advice, no ranking, no *should*, no *missed*, no *good*, no red. Every
  user-facing string comes from the core's catalogue
  (`crates/stride-core/src/strings.json`); **the core renders the sentences,
  the UI does not write them**.
* **The four destinations stay** (Today, Runs, Progress, Body) with the
  athlete profile as the fifth (D8). Compare and Plan stay contextual.
* **The states are part of the design**: empty window, one run, no zone
  model, no race time, no plan adopted, no threshold chosen, a run with no
  heart-rate data.

## Rules

* **Implement the cut; do not re-plan it.** The plan and the two prototypes
  are the specification. Where they disagree with the code, **stop and
  report** rather than improvising.
* **Small commits, one step each**, in the order: F3 (the wall's card list),
  F1 (the radar), F2 (the pace page), S2 (the dead keys), then the two
  prototype-driven pieces. Each commit passes the gates on its own.
* **The gates are in `exploration/cleanup-plan.md`** — read them before you
  start. Run them per commit and report the results: `cargo fmt --check`,
  `clippy -D warnings`, `cargo test --workspace`, `scripts/check-deps.sh`,
  `scripts/check-contract-greps.sh`, `scripts/privacy-rules.sh`, the secret
  and privacy scans and their self-tests, `scripts/check-corpus.sh` (the FIT
  fixtures' SHA-256 must not change), `scripts/scan-history.sh`,
  `tokens:check`, `lint:voice`, `typecheck`, `test:unit`, `test:viewport`.
  **A gate you did not run is a gate you did not pass.**
* **The contract is frozen.** A change to it is a plan change and must
  travel with a `PLAN.md` note in the same change (`docs/contract/README.md`).
* **Synthetic fixtures only**; no production data, no Garmin writes, no
  publish, no network. **No new dependencies** (`scripts/approved-deps.txt`).
* **Do not commit or push.** Leave the changes in the working tree; the
  parent reviews and commits.

## Report back (last message, under ~450 words)

* the files changed, per step;
* the wall's eleven cards, as built, with the picture each one carries;
* how the run detail's one chart is built — the toggles, the cursor, where
  the zone bands sit, and how the three series share an axis without lying;
* the ledger card's matrix, the longest run's series and the repeated work's
  marks;
* the catalogue keys added and removed;
* the gates each commit passed;
* anything the prototypes and the code could not agree on.
