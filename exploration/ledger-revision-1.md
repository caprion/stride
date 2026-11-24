Parent decisions on your eight open questions, plus three revisions. Please apply them to `exploration/stimulus-ledger.md` and `exploration/stimulus-ledger.html` (prototype only; still no changes under `crates/`, `frontend/` or `android/`).

## The decisions

1. **Window** — **calendar weeks**, expressed as the core's own rolling window so both hold at once: `RollingWindow::new(7 * weeks, end_of_the_current_week)`. That is exactly the Progress wall's calendar-week span, and `RollingWindow::previous()` then gives the same number of calendar weeks before it. Say in the note that the wall and the ledger now share one window definition, and flag (for the cleanup pass, not for this slice) that the Progress *totals* and the rolling route currently use two different window definitions — that is a product-wide inconsistency to unify.
2. **Zone model** — **the model in force on each run's date**, not the most recent one, because `crates/stride-core/src/zones.rs` already argues that two runs must not get two scales, and the store keeps dated models. Two consequences to write down: (a) a run dated before the first stored model has **no zone time** (a missing value with a reason, never zero, never band 1); (b) the product-wide rule should become "each run is measured with the zone model in force on the day of the run" everywhere — the zones route, the run detail's zone table, the ledger and the load model. I am taking that to the cleanup pass; for now the note states the rule the ledger will follow and flags the routes that must be migrated with it, so the ledger and the run detail cannot disagree.
3. **Denominator** — the **record stream's movement**, so the bands and the outside add up to 100 %. Agreed.
4. **Last date** — **history-wide** (at or before the end of the window), with the panel saying when the date falls outside the window ("40 days before the end of the window"). Agreed — this is the negative space, and it is the point.
5. **Longest run** — keep the row, but call it a **value in the shape family**, not a kind: the enum should not pretend it partitions anything. Note the count is 1 by definition.
6. **Repeated-work thresholds** — do not freeze them yet. Check them against the `fixtures/fit/` corpus before they are written into the catalogue; report what the fixtures actually contain.
7. **Where it lives** — the ledger is **not a new card**. It is the Progress wall's **"Where the time went"** card, extended (see reconciliation below), and it also gets one line on the run detail (the run's own row). The parallel surface thread (`thr_q6tcjhp6wm`, done) cut the wall from 24 cards to 11 and merged the polarised split into the zone card; your ledger and their card are the same card.
8. **Condition axis** — hold. See the temperature finding below.

## Reconciliation with the surface cut

Read `exploration/surface-inventory.md` §6 (the consolidated Progress, card by card). The card **"Where the time went"** (which merges `zones` + `split` + the zone-time distribution) becomes **the ledger card**: the five zone shares, the polarised three-band summary folded in as context, and then your rows — the last date per band, the longest run, the repeated work. One card, no new surface. Adjust the note's §6 and the prototype's frames to show it as that card, and make sure the merged card does not print the zone shares twice.

## Three revisions

1. **Check the watch's own workout structure first.** RUNALYZE now classifies laps by the FIT **workout step type** (warmup / interval / recovery / cooldown / rest) — Garmin records that on the lap message. Our FIT lap definition numbers are read by number in `crates/stride-ingest/src/fit.rs` (the lap message is `MESG_LAP`); the workout-step field on a lap is definition number 24 (`wkt_step_name`). **Verify from the fixture corpus** whether our lap messages carry definition 24, and whether the three real Garmin files in `fixtures/fit/` carry it. If they do, the repeated-work rule should **prefer the watch's own step structure** and fall back to the lap-shape heuristic only for files without it — that is a fact from the athlete's own watch rather than our threshold, and it is a better answer. Rewrite kind 5 with the two paths and state which one answered.
2. **Temperature.** Your finding is important and I am taking it to the user: the fixture corpus carries **no** temperature field, so the condition axis may have zero coverage on this athlete's watch. Add a short paragraph to §5 saying what would have to be true for the axis to exist (a watch with a thermometer **and** temperature recording enabled in the watch's own settings, so that field 13 is written into the FIT), and that until a real file shows the field, no condition-axis design should start. Keep §5 as it is otherwise — it is the best-written part of the note.
3. **Proof-read the wording against the ban list** now that the sentences are final, and keep the ban-list check table in §3.3.

## What I need back

* the note and prototype updated (paths unchanged);
* the answer on **lap definition 24** in the fixture corpus, with the exact files and what they carry;
* what the repeated-work rule answers on the real fixture files (`mixed_run.fit`, `short_effort.fit`) and whether the thresholds survive;
* any place where decisions 1, 2 or 7 broke something in the note — I would rather know now than at implementation time.
