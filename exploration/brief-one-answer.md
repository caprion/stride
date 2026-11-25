# Brief — one answer per question: the app must not contradict itself

*Written by the parent thread, 24 November 2026. **Implementation.** Found by
a review of the live rig.*

## What the review saw

> *"/pace and /progress both know max heart rate 197 bpm · taken from your
> watch on 25 Nov 2026, yet /runs/1 (24 Nov Bengaluru - Supersets) says 'No
> zones yet — your watch's zones were not read for these runs' and 'no maximum
> heart rate to measure the efficiency factor against'."*
>
> *"/progress displays 'No heart rate recorded — none of the runs in this
> period recorded a heart rate' right above THE WATCH'S OWN ZONES showing
> 36+ hours of Z1–Z5 HR data."*

Both are the same fault: **two answers to one question, on the same screen or
across two screens.**

## The rule

**One basis per answer, and the answer says which basis it used.**

* The **screen's source line** names the model and the day it was read — and
  it must also say **what it applies to**: the zones it names are the
  *watch's configuration as it stood on the day it was read*, and they measure
  **the runs from that day on**. A run before it is a different answer, and
  that answer must say so **where it is shown**, not leave the reader to
  reconcile two screens.
* A **run's own answer** states the basis it used, and when it has none, it
  says why — in the wording the ledger fix already introduced:
  *"not measured — your watch's zones were not read for these runs; the
  watch's own zone minutes are the only zone time here"*.
* **A reason belongs to the block it qualifies.** The ledger's two blocks are
  two measurements; the message about the app's own bands must sit **inside
  the app's own block**, and the watch's block must carry its own coverage —
  never a message above one block that describes the other.
* **No answer may contain a number that another answer on the same screen
  contradicts** — and where two answers legitimately differ (the app's bands
  against the watch's own floors), the difference is **stated** where the two
  appear, not left for the reader to find.

## The dead end on /pace

> *"/pace dead-end UX: Empty horizontal bars for every pace row with 'no race
> time entered — the paces have no value yet', no inline button/input to enter
> or pick a race from history, and the your own race legend entry has nothing
> plotted."*

* **A card must not offer a way that goes nowhere.** The pace card needs a
  **door**: the race can be typed on the rig's page today, so the card must
  **say where** and take the athlete there — a link, with the exact place and
  what happens there ("your race is typed on the rig's own page"), or the
  input itself if the app can own it. Say which you chose and why.
* **A legend entry with nothing plotted must not be drawn** — the legend
  lists what is on the chart.
* **Empty bars are not bars.** A rung with no value should show **no bar at
  all** and the reason beside it, not an empty track that reads as "zero".
  (The product's rule: absence is not zero — an empty bar is a zero wearing
  a costume.)

## The other half of the review's finding

> *"/progress displays 'No heart rate recorded — none of the runs in this
> period recorded a heart rate' right above THE WATCH'S OWN ZONES showing 36+
> hours of Z1–Z5 HR data."*

The message was **wrong** — 101 of the 102 runs carry heart rate — because it
was the reason for the *app's* bands being absent, stated as a reason about
heart rate. The ledger fix has the right reasons now; **make sure the card
places them correctly** and that the two blocks cannot be read as one.

## Rules

* **Facts, never verdicts**; the ban list and the closed direction set hold.
* **The vocabulary** (D24): *"your zones"*, *"the watch's own zones"*,
  *"not measured"*, *"not recorded"*.
* **Do not touch** `crates/stride-core/src/stimulus.rs`,
  `crates/stride-api/src/routes/stimulus.rs` (the ledger thread's) or the
  daytime constants (the day's-parts thread's) — if you need something from
  them, **report it**.
* **The gates**: fmt, clippy `-D warnings`, the workspace tests, check-deps,
  the contract greps, the secret and privacy scans and their self-tests,
  `tokens:check`, `lint:voice`, typecheck, unit, viewport.
* **Commit on `rig/v2`**.

## Report back (last message, under ~400 words)

* what you changed, so that no two answers can contradict;
* **the source line's new wording**, and where it says what it applies to;
* where each reason now sits, and the rule that keeps it there;
* the **door** on the pace card, and what the athlete finds when they take
  it;
* the legend and the empty bars, before and after;
* the gates, and anything you could not change without the other threads.
