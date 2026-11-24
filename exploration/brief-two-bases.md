Parent: a finding that changes the ledger's data, and a design question to
solve. **Design + prototype only** — no code in `crates/`, `frontend/`, and
no schema change yet.

## The finding

The Garmin Personal API serves the zone configuration **as it stands now** —
one record, no date, no effective-from. **There is no zone-model history to
backfill.** So Z's rule ("each run is measured with the model in force on the
day of the run") leaves **every imported run** with no zone times — and the
ledger's intensity rows would be empty over exactly the history the athlete
has.

The fourth path: the **watch's own zone minutes per run**
(`hrTimeInZone_1..5`) ride on the **search payload Stride already fetches** —
**zero extra requests** — and fill 106 of 107 imported runs coherently (the
zone seconds sum to the elapsed duration within 2 %).

What they cannot do:

* no zone **boundaries** — the app does not know what heart rate was "zone 3"
  on the day of that run;
* they sum to **elapsed** time, not **moving** time — a different basis from
  the app's own zone time;
* they do not say **which configuration** produced them;
* they do not exist for a run with no heart rate.

`exploration/zone-model-history.md` has the evidence.

## The question you are designing for

**How does the product show zone time over a history that has two different
sources of zone time?** The parent has asked the user, and **option A is the
recommendation**: **two blocks, each naming its basis, never added together.**
Option B (strict: nothing before the first model) is the alternative.

Design **option A**, and make the design work even if the user picks B — the
two blocks should be able to switch off independently.

## What to produce

### 1. `exploration/two-bases.md` — the design note

* **The two series, precisely**: what each is, what basis it is on (moving
  time vs elapsed time), where it comes from (the app's own computation over
  the stored model; the source's own minutes from the search payload), and
  **the overlap** — the period where both exist and can be **compared**. That
  overlap is gold: it is the only place the app can *check* the source's
  numbers against its own, and it should be designed as such, not as an
  accident.
* **The comparison, and what it may say.** If the two disagree, what does the
  app state? (Boundaries differ, elapsed ≠ moving, the watch may have been
  configured differently.) The app may state the disagreement; it may not
  declare which is right, and it may not silently prefer one.
* **The bands.** For the imported history the app knows **the watch's zone
  numbers** (1–5) but **not the boundaries**. So the rows must be named
  honestly — the watch's own zones, not "Z1–Z2" — and the boundary gap named.
  Say what the bands are called, what the card says about the boundaries it
  does not hold, and what happens for the period where the app *does* hold
  the model.
* **The two-block card**: the ledger's own zone time (moving basis, the model
  in force), and the watch's own zone minutes (elapsed basis, the runs before
  the model). How they sit in one card, how they are never added, and what
  the caption says.
* **The overlap check** as a first-class thing: over the weeks where both
  exist, the app can state how close they are. Say what that looks like as a
  *fact* — and that it is the app checking its own arithmetic, not a verdict
  on the watch.
* **The storage side, as a proposal only**: where `hrTimeInZone_1..5` would
  live (the `activity_observations` table from D10 — one row per activity per
  zone, with the source, the day the response carried it and the day it was
  read), and what the sync change costs (no extra requests; the search
  response already carries it). **Propose, do not implement** — the schema
  change waits for the user's answer and for Pass 3.
* **What the states become**: no model at all; a model but no watch minutes;
  watch minutes but no model; neither (a run with no heart rate).

### 2. `exploration/two-bases.html` — the prototype

Picture-first, and **the two blocks must be visually distinct** so nobody
adds them in their head:

* the ledger's own zone time — the existing weeks × bands matrix;
* the watch's own zone minutes — a second matrix, marked as the other basis;
* the **overlap** shown as a third element: the weeks where both exist;
* the caption naming the basis of each block, and the period;
* the states above;
* phone and desktop widths, in the style of `exploration/stimulus-ledger.html`
  (copy its palette and phone frame). Plain HTML/CSS, no frameworks, no
  network, `file://`.
* **read the numbers from `exploration/athlete.js`** — do not type them in.

## Rules

* **Design and prototype only.** New files under `exploration/` only. Do not
  touch `crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/`.
  No commit.
* **Explanation is not product content** (D22): the surfaces carry the
  picture, the labels, the numbers, the period, and the reason something is
  missing. Everything that explains the design goes in the notes — this
  prototype should have a **notes toggle, off by default**, as the others do.
* The ban list applies (`docs/contract/voice.md`). No verdict, no score, no
  "which is right", no advice.
* Read `exploration/zone-model-history.md`, `exploration/stimulus-ledger.md`,
  `exploration/decisions.md` (D11, D21, D22) and `exploration/athlete.js`
  first.

## Report back (last message, under ~400 words)

* the two files created,
* the two blocks' names and bases, in one line each,
* **the overlap check**: what it would say, in a sentence, and what it may
  not say,
* how the band names differ between the blocks, and the wording that carries
  the boundary gap,
* the storage proposal in four lines,
* the open questions.
