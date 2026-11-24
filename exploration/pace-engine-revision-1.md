Parent: the spine thread has finished (`exploration/pfitzinger-spine.md`), and its
findings **change your ladder**. Please revise `exploration/pace-engine.md`
and `exploration/pace-engine.html` accordingly.

## What the spine established (cite these; do not re-derive them)

The book's rules are **percentages of a named race pace** — not fixed s/km
constants, and **not anchored on marathon pace** for either shipping plan
(the 10K and the half marathon). The five placeholders you left:

| rung | the sourced value | citation |
|---|---|---|
| `EASY` | **no pace at all** — the general-aerobic day is a heart-rate rule: **70–81 % maxHR / 62–75 % HRR**. The pace comes from the athlete's own pace-at-HR, or the athlete types it. The provisional `+120 s/km` has no citation. | ch. 7, ch. 8 |
| `THRESHOLD` | **LT pace = "the pace you could race for about an hour"**, ≈15K–10-mile race pace; for shorter-distance runners, 10K pace + 10–15 s/mi (or 5K + 20–30 s/mi). **80–91 % maxHR / 75–88 % HRR.** | ch. 1, ch. 7, App. C |
| `LONG_START` | **10K race pace × 1.20–1.33** ("20 to 33 % slower than your 10K race pace"). **74–84 % maxHR / 65–78 % HRR.** | ch. 1, ch. 7, App. C |
| `FINISH` | **15K–HM race pace × 1.17–1.29** for a standard long run; **LT pace for the last ~3 miles** for a progression long run. Neither is at marathon pace. | ch. 1, ch. 7, App. C |
| `RECOVERY` | **15K–HM race pace + 120 s/mi (+75 s/km)**, plus **below 76 % maxHR / below 70 % HRR**. | ch. 2, ch. 7 |

Also established: the two bases differ structurally — **Pfitzinger = one hard
session a week**, easy days banded by **maxHR**, paced from the athlete's own
race distance; **80/20 = two hard sessions a week**, easy days banded against
**threshold**, the gray zone deliberately avoided.

## What changes in the design

### 1. The anchor is the race pace, not the marathon pace

Keep the VDOT chain — it is the engine's own and it is sound — but the
**anchors** are now:

* **the athlete's race pace at the goal distance** (10K or HM — the two
  shipped programmes), and
* **LT pace**, derived by the book's own rule from the same race time: the
  pace for a ~15K (10-mile) race, or 10K pace + 10–15 s/mi for a
  shorter-distance athlete. Say in the note how you compute it from the race
  time (the VDOT/15K-equivalent route), and that the rule is cited, not
  invented.

The marathon pace may remain as **one more rung** (it is a real thing runners
train at) but it is **not the anchor**, and nothing is computed as an offset
from it.

### 2. Every rung carries a heart-rate band as well as a pace

This is the structural insight: the method's easy day is an **HR band**, and
Stride already knows the athlete's **pace at heart rate**. So a rung is:

* an **HR band** (where the method states one) — and
* a **pace**, either from the method's percentage rule, **or** from the
  athlete's own pace at that heart rate, taken from their history.

The ladder therefore draws **two axes**: heart rate and pace, with each rung
showing its band on the HR axis and the pace the athlete actually runs at
that HR. Where the method states a pace rule, the rung shows **the rule's
range**; where it states only an HR rule, the rung shows **the athlete's own
pace at that HR** — and says which it is.

If the athlete has no history at a given heart rate, that rung's pace is a
**no value** with the reason ("no runs at that heart rate yet"), and the band
still stands. The band is the method's; the pace is the athlete's; the card
must not blur them.

### 3. The inputs change

The athlete types (per the spine): **goal race distance + date · one race
time (10K or HM) · max HR · resting HR · plan tier + start date.** Max HR and
resting HR already exist in Stride (the zone model; the wellness readings) —
say which are read rather than typed, and keep the typed ones dated.

The **plan tier** and **start date** are new inputs; the plan engine's
`PlanSource` gains a method value and the session kinds gain the book's
vocabulary (the spine has the list of thirteen run types). Sketch that in the
note — do not implement it.

### 4. What the ladder should now look like

* Two axes: **heart rate** (with the athlete's own zones marked on it) and
  **pace**.
* The rungs: easy, recovery, threshold (LT), long-run start, long-run finish,
  and the race pace the athlete is training for — each labelled with **the
  rule that produced it**, and each with its HR band where the method gives
  one.
* Azure = the athlete's own data; amber = the method's rule; no judgement
  colour; no "easy/hard" verdicts. The athlete's zone boundaries fall where
  they fall.
* The **no-race-time state** stays exactly as it is.

### 5. What to write down

* the **revised chain**: race time → VDOT → race-pace equivalents → LT pace →
  the rungs, with each link's citation or its own formula;
* a table of the **rungs, their rules, their HR bands, and where each number
  comes from** (the method's rule / the athlete's history / the engine's
  formula);
* the **plan inputs** (typed vs read), and how the ladder changes when the
  race time is replaced;
* the note that the **two bases produce different ladders** — and that the
  app holds one at a time, named on the plan card.

## Rules

* Same two files only. No changes under `crates/`, `frontend/`, `android/`,
  `desktop/`, `PLAN.md` or `docs/`. No commit.
* Every rule you print carries its citation (chapter) or is marked as the
  engine's own formula.
* The ban list still applies to every sentence in the prototype.
* Do not re-research the book: read `exploration/pfitzinger-spine.md` and use
  its numbers.

## Report back

* the revised chain, one line per link,
* the rung table (rule · HR band · where the number comes from),
* what the two-axis ladder looks like now, in three lines,
* which numbers are the method's, which are the athlete's, and which are
  yours.
