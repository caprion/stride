Parent: a transcription fault in the spine's own tables. The plan-data thread
re-parsed your four shipping schedules independently and cross-checked every
day against its JSON — its cells are exact — and found that **your rows do
not reconcile with your own printed mileage** in one programme, and that two
weeks are impossible.

## What it found

* **HM tier 2, weeks 7, 6, 5, 4, 3, 2 and 1** — the daily cells do not sum
  to the printed weekly volume: short by 3, 7, 5, 17, 6 and 4 miles, and
  week 3 **over** by 1.
* **HM tier 2, weeks 4 and 2** — **identical cells** with volumes 63 and 52.
  Two different weeks cannot have the same days and different totals; one of
  the two is mis-transcribed.
* **10K tier 1, week 5** — the spine prints `?` for the volume and no
  numbers for the Saturday `Rec` and the Sunday `End`; the readable part is
  26 miles.

Your §2.3 and §6.4 say every cell reconciles with the printed mileage. For
the HM tier 2 they do not — so the fault is in the **transcription**, not in
the book.

## What to do

1. **Re-read the HM tier 2 schedule from the book's schedule image** — every
   day, every week, cell by cell — and **replace the table** in
   `exploration/pfitzinger-spine.md`. Read the image at the highest
   resolution you can; the appendices are rendered images, and the weekly
   volumes are printed alongside the days, so the check is available: each
   week's days must sum to its printed mileage.
2. **Re-read 10K tier 1 week 5** — the volume and the Saturday and Sunday
   cells.
3. **Check the other three programmes the same way**, cell by cell, against
   their printed volumes — the data thread's check found them sound, but it
   only *re-parsed* them; you should *re-read* them.
4. If a cell **cannot** be read with confidence, say so in the table and in
   §6.4's list — **do not fill it from the arithmetic**; a volume that does
   not sum is a finding, not a licence to adjust a cell.
5. Update §6.4 to state, per programme, **which weeks reconcile and which do
   not**.

Then the data thread will re-transcribe from your corrected tables.

## Also

* `exploration/athlete.js` carries **VDOT 49.7**; the core's relations give
  **49.6** (`vo2.rs`, step 2 of the rig-v2 build). The difference is
  rounding, but the file and the code should agree — please set the file to
  **49.6** and note the change.
* Keep the rule you have used throughout: **the book's rules are quoted and
  cited**; the **programmes' weeks are transcribed**, and where the reading
  is uncertain, the uncertainty is stated.

## Report back

* the HM tier 2 table, re-read, with the per-week reconciliation;
* the 10K tier 1 week 5 cells;
* the reconciliation status of all four programmes;
* the VDOT correction.
