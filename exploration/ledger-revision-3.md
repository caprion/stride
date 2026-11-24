Parent decisions from revision 3, plus the cleanup thread's brief. Please apply them to `exploration/stimulus-ledger.md` and the prototype (same two files), and keep the scope to those files.

## 1. Repeated work — path 1 only

Your finding settles it: path 2 (the lap-shape heuristic) answers **"repeated work"** for a plain run with three equal auto-laps — auto-laps are equal by construction, so a shape rule cannot tell a workout from a lapped run. A false positive on a real Garmin run is exactly the kind of defect this product exists to avoid.

So: **path 1 only** — the watch's own workout steps (lap definition 24, step name `interval`). Delete path 2 from the note and the prototype, or keep it in one clearly-marked paragraph as *the rule we rejected and why* (that paragraph is worth keeping: it says what the app will not pretend to know).

Then the row's states become:
* runs with steps → the row answers, measured by the watch's own structure;
* runs without steps → **no value** for the row, with the reason ("the runs in the window carry no workout steps"), and the card says which measurement answered.

**Consequence to state plainly in the note:** until the athlete records sessions that carry the watch's own workout steps, this row has little to say — and that is the correct answer, not a bug. I am asking the user to record one structured interval workout on their watch; the next sync brings the file, and the row becomes real.

## 2. The zone model — product-wide migration, staged

You are right that the rule cannot be the ledger's alone: `/fitness/zones`, the run detail's zone table, `load.rs` and the fingerprint's efficiency factor all still use the *current* model, so the ledger and the run detail can disagree for runs before the first model was read.

Decision, in two stages, so it can be done in the cleanup pass without one enormous change:

* **Stage 1 (the cleanup pass):** every run is measured with **the zone model in force on the day of the run** for the **zone time** answers — the run detail's zone table, the window's zone split, the ledger. The store already keeps the dated models in `hr_zone_models`.
* **Stage 2 (later, separately):** the **load model** (`load.rs`: TRIMP's threshold and max heart rate) moves to the dated model per run. That one changes the load history the product has shown since v1, so it is its own change with its own announcement, not a quiet refactor.

**Rule that must hold in both stages:** every answer names the model it used and the day that model was read. The core already has `zones::model_source` for this; the card's source line should carry it.

The repo's own capability note already says it — *"today's settings cannot be passed off as historical settings"* (`docs/garmin-api-capabilities.md`). So this is alignment, not invention.

## 3. The window's tail — one rule, ending today

You found the disagreement: the ledger's window ended on the current week's Sunday (7 Sep – 29 Nov) while the Progress totals end today, leaving up to six days of disagreement at the tail.

Decision — **one window for the whole Progress wall**: the window is **the last N calendar weeks, from the start of the earliest one, through today**, and the period it is compared against is **the same span ending seven days earlier** (so the partial week compares with the partial week).

So: `window_start = week_start(today) - 7 * (weeks - 1)`, `window_end = today`; the previous period is the same number of days ending one week earlier. Use that same definition for the Progress totals, the ledger and the rolling comparisons, so no card describes a different span from its neighbour. The calendar-week shape survives (the window still begins on a Monday); only the end moves from Sunday to today.

## 4. One more small thing

Your duplication fix (the distribution printed once; the band rows carrying only time, runs, last date and change) is right — keep it. And the coverage lines stay **with the rows they qualify**, not in a footer.

## Report back

* the two files updated,
* what the repeated-work row now says in its three states,
* the window's two dates for a 12-week window as of today (24 Nov 2026), and the previous period's,
* anything in the note that now contradicts these three decisions.
