# Two bases — the ledger's own zone time, and the watch's own zone minutes

*Design note and prototype, 24 November 2026. **Design and prototype only.** Nothing under
`crates/`, `frontend/`, `android/` or `desktop/` was touched; no schema change was made; no commit
was made. The two files are this note and `exploration/two-bases.html`.*

*The surface copy in this note and in its prototype follows the language pass — `exploration/language.md`.*

Read with: `exploration/zone-model-history.md` (the evidence for the finding), `exploration/stimulus-ledger.md`
(the ledger this extends), `exploration/decisions.md` (D3, D10, D11, D21, D22) and
`exploration/athlete.js` (the canonical athlete, the one source of every number).

---

## 0. The question, and the answer in one line

**The question.** The zone model has no history in the source: one record, no date, no effective-from.
So Z's rule — *every run is measured with the model in force on the day of the run* — leaves every
imported run with no zone time, and the ledger's intensity rows would be empty over exactly the
history the athlete has.

**The answer (option A).** The card carries **two blocks**, each naming its own basis, and the two are
**never added together**:

| block | what it is | basis | where it comes from |
|---|---|---|---|
| **Your zones** | the app's own zone time, in the athlete's own bands | **the moving time** — the record stream's movement | the app's computation over the stored dated model (`hr_zone_models`), run by run |
| **The watch's own zones** | the source's own record of each run, in the watch's own zone numbers | **the elapsed time** — the watch's own session duration | `hrTimeInZone_1..5` on the **activity search payload Stride already fetches** — zero extra requests |

And a **third element**, drawn between them: **where both exist** — the weeks in which the app holds
both numbers for the same runs, and can therefore **check its own arithmetic against the watch's
record**. That period is the only place the two series can meet, and it is designed as a check, not
as an accident.

**Option B stays one switch away.** If the user picks the strict reading — nothing before the first
model — the second block is switched off and the card shows what it shows today: the ledger's own
zone time, empty over the period that has no model, with the reason in place. Nothing else in the
design changes: the two blocks are independent, each with its own coverage, its own basis and its own
reasons. §7 lists what that independence means concretely, and §9 lists what is still open.

---

## 1. The two series, precisely

### 1.1 Series 1 — "Your zone model": the app's own zone time

| | |
|---|---|
| **what it is** | the time the athlete spent in each of their own bands, per run |
| **basis** | **moving time** — the record stream's movement; the five bands plus the outside add up to 100 % |
| **computed from** | the run's own heart-rate samples (`zones::time_in_zone`) under the **stored model in force on the run's date** |
| **availability** | only the runs whose date has a model in force. For this athlete the first stored model was read on 3 Nov 2026, so the eight weeks before it hold no value in this block |
| **band names** | **Z1–Z2 low · Z3 moderate · Z4–Z5 high** — the athlete's own bands, the boundaries held in the model in force |
| **what it may not do** | it may not borrow the model's boundaries for a run whose date has no model; it may not be summed with the second block |

### 1.2 Series 2 — "The watch's own zone minutes": the source's own record

| | |
|---|---|
| **what it is** | the minutes the watch itself recorded in each of its own zones, per run |
| **basis** | **elapsed time** — the watch's own session duration, stopped stretches included |
| **comes from** | `hrTimeInZone_1..5` on `SummarizedActivityDTO` — the **activity search** payload, the one the sync already reads to discover runs. **Zero extra requests** |
| **availability** | every run that recorded a heart rate (106 of the 107 imported runs); a run with no heart rate has none, and no option invents one |
| **band names** | **watch zone 1 … watch zone 5** — the watch's own numbers, and **no roll-up** (see §3) |
| **what it may not do** | it may not be presented as a model; it may not carry boundaries the app does not hold; it may not be summed with the first block |

**The three things it cannot do** (the finding, restated as design constraints):

1. **no boundaries.** The app does not know what heart rate was "zone 3" on the day of that run.
2. **a different basis.** The source's zone seconds sum to the **elapsed** duration; the app's own
   zone time sums to the **moving** time. These are two measurements of two different spans, not two
   estimates of one thing.
3. **no configuration.** The payload names no model, no floors, no sport-to-model link. It can be
   **compared** with a model, never **identified** with one.

### 1.3 The overlap — the period where both exist

**The rule.** The overlap is the set of **runs** that satisfy both:

* a model was **in force on the run's date** (the app holds the boundaries), **and**
* the source carried its own zone minutes for that run (the watch recorded a heart rate).

For this athlete: the model was first read on **3 Nov 2026**, so the overlap is the runs from
3 Nov 2026 on — **8 runs in 4 calendar weeks** (the weeks of 2 Nov, 9 Nov, 16 Nov and 23 Nov; the
week of 9 Nov holds no runs).

**Why this period is gold.** Everywhere else, the two series are not comparable at all: one has
boundaries the other does not, and they are over different spans. Over the overlap they are two
answers to the same question about the same runs on the same days — the only place the app can
**check its own arithmetic** against the watch's record. §5 designs that check as a first-class
element.

**The overlap is not a third series.** The watch's minutes are shown as a series **only where they
are the answer** (the runs with no model in force). Where the app holds the model, the watch's
minutes appear **only inside the check** — so that no week ever carries two zone times that could be
added.

---

## 2. The comparison, and what it may say

### 2.1 What the app compares

| comparison | over | what it answers |
|---|---|---|
| **the totals** | each run in the overlap, and each week | do the two cover the same time? (they cover different spans, so a small difference is expected) |
| **the zones** | each run in the overlap | do the two place the same time in the same zones? — the watch's zone *n* against the app's Z*n* |

### 2.2 If the two disagree, what the app states

The app **states the disagreement and names what could account for it**, in one line, beside the
numbers:

> **3 minutes apart** — one counts the time you were running, the other the whole run.

The three possible accounts, all of them **statements about the data** and none of them a finding:

1. the two are measured over **different spans** of the same run (moving vs elapsed);
2. the **boundaries** the watch used on those days may not be the model's boundaries;
3. the watch may have been **configured differently** then.

### 2.3 What the app may not do

* it may **not declare which is right** — not the model, not the watch, not the app;
* it may **not silently prefer one** — the card never shows the app's number and hides the watch's,
  or the reverse; the two are both shown, or the second block is off and says so;
* it may **not score the agreement** — no "accurate", no "consistent", no percentage that reads as a
  grade;
* it may **not treat a difference as an error** — a difference is a difference between two
  measurements, and the card says what it is measuring;
* it may **not extend a difference into the periods where only one series exists** — outside the
  overlap there is nothing to compare, and the card says that rather than extending a result.

**The check is the app checking itself.** Its subject is the app's own arithmetic — *is the number I
computed from the samples consistent with the number the watch wrote down at the time?* The watch is
the **witness**, not the defendant: a disagreement is a fact about two measurements, and the card
does not turn it into a finding about either.

---

## 3. The bands: what they are called, and the boundary gap

### 3.1 The names differ between the blocks, deliberately

| block | row names | why |
|---|---|---|
| **Your zone model** | **Z1–Z2 low · Z3 moderate · Z4–Z5 high** | the athlete's own bands, named by the product's existing vocabulary (`zones.band.low/moderate/high`). The boundaries are held — the model in force carries the floors — so the roll-up is the athlete's own grouping and may be drawn |
| **The watch's own zones** | **the watch's zone 1 · the watch's zone 2 · the watch's zone 3 · the watch's zone 4 · the watch's zone 5** | the watch's own numbers. **No roll-up**: "low / moderate / high" is the product's grouping of the **model's** bands, and applying it to the watch's zones would import the model's boundaries into rows that do not hold them |

So the second block has **five rows where the first has three**, and the two row sets are never
mixed. The reader sees five differently-named rows under a different heading, on a different basis,
covering a different period. That is the whole of the separation, and it is carried by the naming
rather than by a warning.

### 3.2 The wording that carries the boundary gap

The second block names the gap in one line, under its rows:

> **the watch's own zone numbers — the boundaries of these zones are not recorded**

This is a **fact about the data** (the boundaries are absent), not an explanation of the design, and
it stays on the surface: a reader who sees "watch zone 3 — 4 h 12 m" must not be left to assume the
app knows what zone 3 was.

The first block, over the period where the app **does** hold the model, carries the model's own
identity at the screen's source line — the model and the day it was read, named **once per screen**
(Z's hoisting rule, D18.3 / D22) — and the run's own row in the ledger says so **in place** where a
different basis was used.

### 3.3 What happens where the app *does* hold the model

* The **first block** answers, over the moving time, in the athlete's own bands.
* The **second block** is silent for those weeks — not empty, not zero: **not shown**, because the
  watch's minutes are not the answer there. The check carries them instead (§5).
* The **second block's** boundary gap applies only to the rows it shows. In the overlap the app
  holds the model's boundaries — but the *watch's* boundaries on those days are still not held, so
  the check's per-zone comparison names that gap too.

---

## 4. The two-block card

### 4.1 The shape

One card — the cut Progress wall's **"Where the time went"** card, not a new surface. Inside it,
three elements stacked in this order, each with its own heading, its own basis line and its own
coverage:

```
Where the time went
7 Sep – 24 Nov 2026 · last 12 weeks

┌ Your zones ───────────────────── the time you were running
│  Z1–Z2 low     5 h 10 m   85%   ▓▓▓▓▓▓▓▓······
│  Z3 moderate     53 m     15%   ········▓▓▓▓▓▓▓·
│  Z4–Z5 high      0 m      0%   ·············▓▓▓·
│  8 of 58 runs measured · your watch's zones were not read before 3 Nov 2026
└
┌ The watch's own zones ───────── the whole run, stops included
│  the watch's zone 1   18 h 20 m  42%   ▓▓▓▓▓▓▓▓······
│  the watch's zone 2   18 h 02 m  42%   ▓▓▓▓▓▓▓▓······
│  the watch's zone 3    5 h 59 m  14%   ▓▓▓▓▓········
│  the watch's zone 4      59 m    2%   ▓▓▓▓·········
│  the watch's zone 5       9 m    0%   ▓············
│  50 of 58 runs · the watch's own zones — where each of them starts is not recorded
└
┌ Where both exist ─────────────  8 runs · 8 Nov – 24 Nov 2026
│  your zones against the watch's own zones
│  week      moving     elapsed
│  2 Nov      30 m       30 m
│  16 Nov   4 h 40 m   4 h 43 m
│  23 Nov      53 m       53 m
│  the period  6 h 03 m   6 h 06 m
│  3 minutes apart — one counts the time you were running, the other the
│  whole run
└
```

*(the numbers above are the prototype's; the prototype computes them from `exploration/athlete.js` —
see the appendix)*

### 4.2 The rules the card holds to

1. **Never added.** The two blocks carry separate totals and separate bases. Nothing on the card sums
   them, and no row of one block sits in a column of the other.
2. **One block per period.** The first block answers the runs with a model in force; the second
   answers the runs without one. Their coverage does not overlap — the overlap lives in the third
   element, which is a comparison and not a series.
3. **Each block names its own basis**, in the same place, in the same shape: *over the moving time* /
   *over the elapsed time*. The card does not carry a single "measured over …" line for both.
4. **Each block names its own coverage**, so the reader can see the split of the window without
   reading a sentence: the first block's columns are filled for the weeks with a model; the second
   block's columns are filled for the weeks without one; the third element marks the weeks where both
   exist.
5. **Absence is drawn, not described** (D11). The eight weeks the app cannot measure are **empty
   cells**, not a paragraph; the one line under the block says why, in the app's own words.
6. **The period is on the card** — the window, and each block's own period.
7. **The source is named once per screen** (Z's hoisting rule): the model, its method, its maximum
   heart rate and the day it was read, at the top of the screen — not repeated on a row.

### 4.3 The caption

The card's caption carries the period, the basis of each block, and the reason something is missing.
It carries **no** explanation of why the design has two blocks, and **no** instruction to the reader
about how to read the card — that is the note layer (D22).

### 4.4 The picture (D11)

The card is read as a picture first; the numbers and the reasons sit under it.

| element | drawn as | why |
|---|---|---|
| the app's own zone time | a **solid** fill, each week's cell in proportion to the time that band held that week | the app's own measurement, on the moving time |
| the watch's own zone minutes | a **hatched** fill, the same proportion | the other basis — the fill is different so the eye does not add the two |
| the weeks where both exist | an **amber** marker strip, aligned to the same week columns | the overlap, drawn where it is, before it is read |
| a week the app cannot measure | an **empty** cell | the hole is drawn, not described (D11) |
| a week with no runs | a **blank** cell | no run recorded is not the same as a run the app cannot measure |
| a measured zero | a **light** cell | a measurement, not an absence |

Two blocks, two fills, one set of columns — the reader sees the split of the window before reading a
single number, and the two fills cannot be mistaken for two parts of one bar.

---

## 5. The overlap check, as a first-class thing

### 5.1 What it is

A third element in the card, drawn over the weeks where both series exist. It is **not** a third
series and **not** a footnote: it is the only place in the product where the app compares two
measurements of the same runs on the same days.

### 5.2 What it says, as a fact

* **the period and the coverage** — "where both exist · 3 Nov – 24 Nov 2026 · 8 runs";
* **the two numbers, per week and in total**, each with its basis on the column head —
  *the app's own zone time · over the moving time* against *the watch's zone minutes · over the
  elapsed time*;
* **the difference, as an arithmetic fact** — "3 minutes apart";
* **what could account for it**, in one line — the two spans, the boundaries the app does not hold,
  the watch's own configuration of that day.

### 5.3 What it may not say

* not "the watch is right" / "the app is right";
* not "accurate" / "inaccurate" / "consistent" / "inconsistent" — a grade, a score, a verdict;
* not "error" — a difference between two measurements over two spans is not an error until something
  says which span was meant;
* not "the model has not changed" — the app cannot know what the watch was configured with on a past
  day, and it does not claim the comparison proves anything about the model's history;
* not a claim about the runs **outside** the overlap — the check's result is not extended to the
  period where only one series exists.

### 5.4 The wording

```
zones.check.title          "where both exist"
zones.check.coverage       "{runs} runs · {from} – {to}"
zones.check.heads          "your zones · the time you were running" | "the watch's own zones · the whole run, stops included"
zones.check.total          "{app} against {watch}"
zones.check.difference     "{difference} apart — one counts the time you were running, the other the whole run."
### OLD
zones.check.none           "not recorded — the watch's own zones are not recorded for these runs"
### NEW
zones.check.none           "not recorded — the watch's own zones are not recorded for these runs"
### OLD
┌ Your zones ───────────────────── the time you were running
### NEW
┌ Your zones ───────────────────── the time you were running
### OLD
│  8 of 58 runs measured · your watch's zones were not read before 3 Nov 2026
### NEW
│  8 of 58 runs measured · your watch's zones were not read before 3 Nov 2026
### OLD
┌ The watch's own zones ───────── the whole run, stops included
│  the watch's zone 1   18 h 20 m  42%   ▓▓▓▓▓▓▓▓······
│  the watch's zone 2   18 h 02 m  42%   ▓▓▓▓▓▓▓▓······
│  the watch's zone 3    5 h 59 m  14%   ▓▓▓▓▓········
│  the watch's zone 4      59 m    2%   ▓▓▓▓·········
│  the watch's zone 5       9 m    0%   ▓············
│  50 of 58 runs · the watch's own zones — where each of them starts is not recorded
### NEW
┌ The watch's own zones ───────── the whole run, stops included
│  the watch's zone 1   18 h 20 m  42%   ▓▓▓▓▓▓▓▓······
│  the watch's zone 2   18 h 02 m  42%   ▓▓▓▓▓▓▓▓······
│  the watch's zone 3    5 h 59 m  14%   ▓▓▓▓▓········
│  the watch's zone 4      59 m    2%   ▓▓▓▓·········
│  the watch's zone 5       9 m    0%   ▓············
│  50 of 58 runs · the watch's own zones — where each of them starts is not recorded
### OLD
│  your zones against the watch's own zones
### NEW
│  your zones against the watch's own zones
### OLD
│  3 minutes apart — one counts the time you were running, the other the
│  whole run
### NEW
│  3 minutes apart — one counts the time you were running, the other the
│  whole run
### OLD
> **3 minutes apart** — one counts the time you were running, the other the whole run.
### NEW
> **3 minutes apart** — one counts the time you were running, the other the whole run.
### OLD
| **The watch's own zones** | **the watch's zone 1 · the watch's zone 2 · the watch's zone 3 · the watch's zone 4 · the watch's zone 5** | the watch's own numbers.
### NEW
| **The watch's own zones** | **the watch's zone 1 · the watch's zone 2 · the watch's zone 3 · the watch's zone 4 · the watch's zone 5** | the watch's own numbers.
zones.check.boundaries     "the boundaries the watch used on those days are not recorded here"
zones.check.none           "not recorded — the watch's own zones are not recorded for these runs"
```

The last line is the state where the check cannot be run at all (no watch minutes, §7).

### 5.5 Why it is worth having

* It is the **only** place the app can find out whether its own computation agrees with the watch's
  own record. A disagreement over the overlap is the app's earliest warning that a stored model is
  not the configuration of a past day — and the app has no other way to find that out.
* It costs **nothing at query time**: the comparison is arithmetic over two stored sets.
* It is the honest shape for the finding: the app can say *here is what I computed, here is what the
  watch wrote down, here is how far apart they are* — and stop there.

---

## 6. The storage side — a proposal, not an implementation

**Proposed shape.** The watch's zone minutes go into **`activity_observations`** (D10's table,
already proposed in `exploration/garmin-temperature-and-activity-fields.md` §5.1(a)), one row per
activity per zone:

| column | value |
|---|---|
| `activity_id` | the run |
| `source` | `garmin_personal` |
| `endpoint` | the activity-search path the value was read from |
| `field` | the source's own field name — `hrTimeInZone_1` … `hrTimeInZone_5` |
| `value` | the seconds, as reported; `NULL` when the source sent null |
| `unit` | `s` |
| `observed_at` | the run's own date — the day the response carried it |
| `retrieved_at` | the day it was read |

**The chores, the repository's normal ones.** A schema version bump (5 → 6) in
`crates/stride-store/src/schema.rs`; the matching edit to `docs/contract/schema.sql` (the two are
compared statement for statement by `crates/stride-store/tests/schema_contract.rs`, so one without
the other fails the build); a row in the export table list so the backup carries the table.

**The sync change: none of substance.** The values ride on the **search entry already in hand** in
the sync's listing loop — the same payload the sync reads to discover the runs. **No extra HTTP
request.** A pull stays at roughly 70 of its 80-request ceiling; the alternative (fetching each
activity's detail for its zone times) would add ~20 requests a pull and break the budget.

**The absences stay apart.** A field the source never sent → no row. A field it sent as null → a row
whose value is null. **Never zero** for "no time in that zone", and never "band 1" for a missing
value.

**The basis.** "Elapsed" is a property of the source's field, not a per-row choice, so the first
proposal adds no column. If a second source with a **moving** basis ever lands, the table needs a
`basis` column — flagged, not added now.

**The seam.** In the one reader (`crates/stride-api/src/zonemodel.rs`), the "no model in force"
branch gains a third case: **no model in force, and the run has the source's own zone time** →
answer with the source's numbers, under the second basis, with the zone edges rendered as unknown;
the reason id becomes *reported* rather than *no value*.

**What the proposal does not do.** It does not bend Z's rule: no run is ever measured against a
model that did not exist on its day. It does not widen the sync's request budget. It does not change
the zone-time computation itself.

---

## 7. The states

| state | first block (the app's own) | second block (the watch's own) | the check |
|---|---|---|---|
| **no model at all** | **not worked out** — "no zones yet — your watch's heart-rate zones have not come through" | **answers** — the watch's minutes exist for the runs that recorded a heart rate | not worked out — "your watch's zones have not come through, so there is nothing to compare against" |
| **a model, but no watch minutes** (the source did not carry them, or the run was imported from a FIT file) | **answers** — the runs are measured with the model in force | **not recorded** — "not recorded — the watch's own zones are not recorded for these runs" | **not recorded** — "not recorded — the watch's own zones are not recorded for these runs" |
| **watch minutes, but no model** (the runs predate the first stored model) | **not worked out in that period** — "your watch's zones were not read before 3 Nov 2026"; the model-period part of the window still answers | **answers** — the watch's own record of those runs | answers over the model period only, and says which period that is |
| **neither** (a run that recorded no heart rate) | that run has no zone time — the cell is empty and the line says why | the same | the run is not in the overlap: it has no heart rate, so there is nothing to compare |
| **option B (strict)** — the second block switched off | answers over the model period; the earlier period is empty, with the reason in place | **not shown** — and the check, which reads the watch's minutes, is not shown with it | not shown |

**Rules the states hold to.**

* A block that cannot answer says so **in place**, in its own words, and does not borrow the other
  block's numbers to fill the gap.
* A **measured zero** is not a missing value: a week with a model in force and no time in Z4–Z5 is
  "0 m", not "no value". The prototype shows one.
* The card **can answer one block and not the other** — that is the normal case, not an error state.
* When the second block is off, the card does **not** show an empty matrix "for symmetry": the block
  is not there, and the first block's reason line carries the gap.

---

## 8. What the card does not say

* **No verdict.** Nothing says the training was unbalanced, that the athlete needs threshold work,
  or that eight weeks without the app's own zone time is a problem.
* **No score, no grade.** The check does not rate the watch, the model or the agreement.
* **No "which is right".** The card states the two numbers and the difference; it does not pick.
* **No advice.** No instruction, no plan, no "next".
* **No zero where a value is missing** — and no silent zero where a *block* is missing.
* **No summing of the two bases.** No row of one block sits in a column of the other, and no total on
  the card adds them.
* **No boundaries the app does not hold.** The second block's rows are the watch's own zone numbers,
  and the line under them says so.
* **No "missed"** — the period the app cannot measure is a period, not a failure.

---

## 9. Open questions

1. **Under option B, does the check survive?** The design says no: the check reads the watch's
   minutes, and B is "nothing from the watch before the first model" — but the check is over the
   *model* period, so it could be kept. This is the user's call.
2. **Does the first block keep a "last date" row while its coverage is partial?** With the model read
   on 3 Nov, the Z4–Z5 row has no date at all — the last run above the floor predates the model, and
   the app cannot place a run in a zone without one. The design shows "no run in this period was in
   Z4–Z5" and puts the reason in the coverage line; confirm the wording.
3. **Should the second block's rows carry their own last dates** (the watch's own last date per
   zone)? The design says yes — it is the same fact in the second basis — but it adds a line per row.
4. **Should the watch's minutes also be shown as a series over the overlap weeks** (a third matrix),
   or only inside the check? The design says only inside the check, so that no week carries two zone
   times.
5. **How much of the check is on the phone?** The design shows the period, the per-week totals and
   the difference. The per-zone comparison is available and could be one row more.
6. **Is the check's difference stated per week, per run, or only in total?** The design shows per
   week and in total; the per-run figure is available and noisier.
7. **The run detail.** The run's own row in the ledger has the same two-base question: a run before
   the first model shows the watch's own minutes, with its own basis and no boundaries. Not designed
   here — flagged for the pass that lands the seam (§6).
8. **The load model** (stage 2 of the zone-model migration) is untouched by this design; it keeps the
   model's own constants, and the second block has no part in it.

---

## Appendix — what the prototype computes, and from where

Every number on the prototype comes from `exploration/athlete.js`; none is typed into the markup.
The prototype draws **synthetic** figures that stand in for the source's own per-zone seconds, which
are **not** in the canonical file. The substitutions, in full:

| quantity | in the prototype | in the product |
|---|---|---|
| the app's own zone time per run | the run's own band minutes from `athlete.js` (low / mod / hard), with Z1–Z2 and Z4–Z5 split by the window's own five-zone share (`ledger.zoneSplit`) | `zones::time_in_zone` over the model in force, per zone |
| the watch's own zone minutes per run | the run's recorded duration (its elapsed basis) spread over the run's own band share | `hrTimeInZone_1..5` from the search payload |
| elapsed vs moving | the long run of 21 Nov carries both in `athlete.js` (`featured.elapsed` "2 h 02 m" against `featured.moving` "1 h 59 m"); every other run has one duration in the file, so both bases use it | the source's sum is the elapsed duration; the app's is the record stream's movement |
| the model in force | `zoneModel.readOn` ("3 Nov 2026") — every run on or after that date has a model in force | the dated history in `hr_zone_models` |
| the watch's minutes | every run in the file carries a heart rate, so every run has the watch's minutes | the runs that recorded a heart rate (106 of 107 in the import) |
