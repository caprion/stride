Parent: the spine's transcription fault is **corrected** — re-read from the
book's schedule images by row-index alignment, and every week now reconciles.
Please re-transcribe from the corrected tables.

## What changed in `exploration/pfitzinger-spine.md`

* **HM tier 2** — the 63-mile week is **week 3** (the book "builds up to 63
  miles with three weeks to go"), and **weeks 4 and 2 are the same week
  twice, both 52 miles** — the "identical cells, different totals" was the
  transcription's error, not the book's. Weeks 7, 6 and 5 had their
  Friday/Saturday cells transposed. Corrected volumes: 46, 49, 52, 47
  (recovery, wk 8), 55, 58, 61, 52, 63, 52, 43, 42 (race) + 37 recovery + 50
  transition.
* **10K tier 1, week 5** — **41 miles**: Friday race pace 9 mi (4 × 1,200 m +
  2 × 1,000 m @ 8K–10K race pace, jog 50 % interval time recovery), Saturday
  recovery 4 mi, Sunday endurance 11 mi. The post-race week is **32 miles**.
* **10K tier 2** — the two endurance days are **Thursday and Sunday**, and its
  week-1 quality session sits on **Thursday** (the other weeks' on Wednesday).
* The spine now states the **method**: the schedules print as two images per
  plan with different row heights, so the halves are aligned by **row index**
  (the Mon–Thu half carries the week label, the Fri–Sun half the printed
  volume) and checked by sum — and that *a volume that does not sum is a
  finding, not a licence to adjust a cell*.
* All four programmes now reconcile: **54 week-rows parsed from the file, 0
  mismatching**.

## What to do

1. **Re-parse the four programmes** from the corrected tables into
   `exploration/plan-templates.json` — every day, every week, exactly as
   before, with the miles and kilometres, the kind in the book's vocabulary,
   and the detail the cells carry.
2. **Keep the check.** Every week carries `volume_check` with the printed
   volume and the sum of its days. **All of them must reconcile now** — and if
   one does not, **report it as a finding and leave the cells as they are**;
   do not adjust a cell to make it add up.
3. **The readability notes may go** where the cells are now read — except
   where the book genuinely does not print a number.
4. **Re-check the JSON against the corrected file** the way you did before:
   parse the spine's tables independently and cross-check every day. That
   check found the fault last time; keep it.
5. `exploration/plan-bases-spec.md` — **no change is needed** unless the
   correction changes something in it (the five decisions B1–B5 stand).

## Rules

* **Transcribe, do not invent.** No code changes; the two exploration files
  only. No commit.
* The **80/20 base stays a spec** — its weeks are not read cell by cell.

## Report back

* the file rewritten;
* **the reconciliation table** — every programme, every week, and the result;
* anything you could not read;
* the volume series for each programme, so the parent can check them against
  the book's shape (a recovery week at 8 weeks to go, a peak, a taper, the
  race week, and the post-race recovery and transition weeks).
