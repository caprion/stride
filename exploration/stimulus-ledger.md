# The stimulus ledger — a design note

*24 November 2026. Revision 4: the parent's revision-3 decisions (repeated work is **path 1 only**;
the zone model's **staged, product-wide migration**; **one window for the Progress wall, ending
today**) and the cleanup thread's brief.*

*The surface copy in this note and in its prototype follows the language pass — `exploration/language.md`.*

**Prototype only.** Nothing under `crates/`, `frontend/` or `android/` has been touched. The two
files are this note and `exploration/stimulus-ledger.html`.

Read with: `exploration/the-wedge.md` (the choice), `PLAN.md` (the plan of record),
`docs/contract/voice.md` (the frozen voice), `docs/garmin-api-capabilities.md` (the zone-model
rule), `exploration/surface-inventory.md` §6 (the cut wall),
`exploration/methods-and-products.md` (the research), and
`crates/stride-core/src/{zones,pace,rolling,fingerprint,story,statement,strings,units,date}.rs`.

---

## 0. What this note decides now

The three decisions the user locked:

1. **Intensity distribution goes first.** This note is the taxonomy and the ledger for it.
2. **Weather comes from FIT bytes only** — **superseded**: the environment axis is **dropped**, not
   deferred (§5). The user has clarified that "running too hot or too cold" means *exertion and
   training load*, not weather ("are you over exerting basically load or not"). §5 keeps the
   **measurement** that closed the question and is the record of why it closed.
3. **History only.** The ledger anchors on the athlete's own recorded history. §1.4 says what that
   costs.

The parent's decisions, as they now stand:

| decision | where it is |
|---|---|
| **The window is the last N calendar weeks, from the start of the earliest one, through today**; the comparison period is the same span ending **seven days earlier** — one window for the whole Progress wall | §2.1 |
| **Each run is measured with the zone model in force on the day of the run**, product-wide, in **two stages**: the zone-time answers first (the cleanup pass), the load model later and separately. **One source line per screen** names the model and the day it was read; an answer that used a different model says so in place | §1.2, §3.3, §6, §6.2 |
| **The share is over the record stream's movement**, so the bands and the outside add up to 100 % | §2.2 |
| **The last date is history-wide** (at or before the end of the window), and the card says when it falls outside the window | §2.4 |
| **Repeated work is path 1 only** — the watch's own workout steps. The lap-shape heuristic is **rejected**, and the note keeps one paragraph saying why | §1.2, §7 |
| **The longest run is a value, not a kind** | §1.1 |
| **The ledger is not a new card** — it is the cut wall's "Where the time went" card, extended, plus one line on the run detail | §6 |
| **The distribution is printed once**; the band rows carry only time, runs, last date and change; the coverage lines stay with the rows they qualify | §3, §6 |
| **The environment axis is dropped** | §5 |

---

## 1. The taxonomy

### 1.1 The kinds and the values

**Three kinds** (the intensity family) and **two values** (the shape family).

| # | | family | what it is | derived from |
|---|--|--------|-----------|--------------|
| 1 | **Z1–Z2 low** | kind | the time the athlete spent in their own first two heart-rate zones | the zone model **in force on the run's date** + the run's own moving samples |
| 2 | **Z3 moderate** | kind | the time in their own third zone | the same |
| 3 | **Z4–Z5 high** | kind | the time in their own fourth and fifth zones | the same |
| — | **Longest run** | value | the run in the window that moved for the longest time | the run's own stored distance and moving time |
| — | **Repeated work** | value | the runs the watch recorded as **workouts with interval steps** | the run's own workout steps (FIT lap definition 24) |

**The longest run is a value, not a kind.** It is one run: it partitions nothing, its count is 1 by
definition, and it must not sit in an enum beside rows that do partition. The enum carries the three
kinds; the shape family is a second list of values on the card.

The three kinds are the product's **existing** grouping — `rolling::PolarisedZone`
(`Low` = Z1–Z2, `Moderate` = Z3, `High` = Z4–Z5) and the catalogue's own names
(`zones.band.low` = "Z1–Z2 low", `zones.band.moderate` = "Z3 moderate",
`zones.band.high` = "Z4–Z5 high"). The ledger counts the bands the product already draws; it invents
no vocabulary.

**Two families, two denominators, and the card says which is which.** The three kinds partition the
window's moving time, so their shares add up. The two values overlap them — a long run is also
mostly Z1–Z2 — so their rows must not be added to the kinds' rows. The prototype draws them in two
blocks for exactly that reason.

### 1.2 The rules, one at a time

Every rule below is deterministic: the same runs, the same models, the same window, the same answer.

---

#### Kinds 1–3 — the intensity kinds (Z1–Z2, Z3, Z4–Z5)

**Rule.** For every run whose date falls in the window:

1. Build the run's samples: `zones::ZoneSample { elapsed_s, heart_rate_bpm, moving }` from the
   decoded FIT, using the api's own helper `routes::series::zone_samples(&decoded)` — the same one
   the zones route uses, so the ledger and the run detail cannot disagree.
2. Compute the run's time in zone: `zones::time_in_zone(floors, &samples) -> Option<TimeInZone>`,
   where `floors` is the athlete's **stored** `zones::ZoneFloors` **in force on the run's date**.
3. The kind's **time** is the sum over the window's runs of the relevant fields of `TimeInZone`:
   * Z1–Z2 → `zone1_s + zone2_s`
   * Z3 → `zone3_s`
   * Z4–Z5 → `zone4_s + zone5_s`
4. The kind's **count** is the number of runs with at least one second in the band
   (`TimeInZone::per_zone()`).
5. The kind's **share** is the time over the window's **record stream's movement** (§2.2).
6. The kind's **last date** is §2.4.

**The zone model is the one in force on the run's date.** The repo's own capability note already
says it: *"today's settings cannot be passed off as historical settings"*
(`docs/garmin-api-capabilities.md`). The store keeps a **dated history** of models
(`hr_zone_models`, with `read_at`), so the model in force on a run's day can be read. The ledger
uses it, run by run.

Two consequences:

* **(a) A run dated before the first stored model has no zone time.** It is a missing value with a
  reason — never zero, never band 1. The card names the coverage (§4.5) and the run's own row on
  the run detail says why.
* **(b) The rule is product-wide, and it is staged** (§6.2): **Stage 1** — every run is measured
  with the model in force on the day of the run for the **zone-time answers** (the run detail's
  zone table, the window's zone split, the ledger); **Stage 2** — the **load model** (`load.rs`:
  TRIMP's threshold and maximum heart rate) moves to the dated model per run, as **its own change
  with its own announcement**, because it changes the load history the product has shown since v1.
  The rule that holds in both stages: **one source line per screen names the model and the day it
  was read** — the core already has `zones::model_source` for it — **and an answer that used a
  different model says so in place.** The model is not repeated on every row: the picture carries
  the numbers, the source line carries the model, and the model's detail is one click away
  (§3.3, §6).

**When an input is missing.**

| missing | what the ledger answers |
|---|---|
| no stored zone model at all | the three kinds have **no value**, and the reason in the core's own words: `zones.no_model` — "no zones yet — your watch's heart-rate zones have not come through". Never band 1, never zero. |
| the stored model's floors are not strictly increasing | no value: `zones.unusable_model` — "zones unavailable — the stored zone model's floors are not in order". (`ZoneFloors::is_usable` is the test.) |
| a run dated before the first stored model | that run has **no zone time** and is not counted in any kind. It is not a zero in the band; it is a run the ledger cannot measure. The coverage line names the gap. |
| a run's file carries no heart-rate samples | as above. |
| no run in this period has any heart-rate samples | the whole intensity block has no value: `rolling.zones.no_samples` — "no heart rate recorded — none of the runs in this period recorded a heart rate". |
| the run's file is missing or unreadable | as above. |

**A measured zero is not a missing value.** The core already draws this line, and the ledger keeps
it: `zones::TimeInZone` says a field "can be `0.0` (a real measurement of no time in that zone),
but the whole result is `None` when the run has no model". So a window with one easy run answers
**0 m** of Z4–Z5 with **no date** — a measurement — while a window with no stored model answers
**no value**. The prototype shows both.

**Derived from a stated fact, or heuristic?** *Derived.* The band boundaries are the athlete's own
watch configuration, dated to the day; the time is the run's own samples. The only judgement is
which bands group together, and that grouping is already the product's frozen one
(`rolling::PolarisedZone`).

**Why "Z4–Z5" and not "above Z4".** Because the core's boundary rule puts a reading *exactly on* a
floor in the lower zone (`zones::zone_of`: the interval of zone *n* is `(floor_n, floor_{n+1}]`),
the time above the Z4 floor **is** Z4 + Z5 exactly. "Above Z4" and "in Z4–Z5" are the same set of
readings, so the ledger can say either; it says the band, because the band is the athlete's name
for it.

---

#### Value 1 — the longest run

**Rule.** The run in the window with the greatest moving time. Where two runs are equally long, the
**earlier** one — the rule is stated so the answer is deterministic.

* **time** — the run's moving time (`rolling::RollingActivity::moving_time_s`).
* **share** — that moving time over the window's moving time.
* **count** — 1, by definition.
* **date** — the run's date.
* the row also carries the run's **distance**, because "the longest run was 20.4 km" is the fact a
  runner reads; the time and the share alone would not say it.

**Inputs.** The stored activities: `local_date`, `moving_time_s`, `distance_m`.

**When an input is missing.** A window with no runs → no longest run → the row is absent, and the
window itself answers `stimulus.no_runs` (§4.1). A run with no distance still has a moving time, so
the row exists with the distance absent (a missing distance is a missing distance, not 0 km).

**Derived or heuristic?** *Derived.* The run's own stored numbers; the only rule is the tie-break.

---

#### Value 2 — repeated work: **the watch's own workout steps**

**The rule.** A run is repeated work when its own file carries the watch's **workout steps** and
those steps include **`interval`**. The FIT `lap` message carries the field **definition 24,
`wkt_step_name`** — an enum naming the step the lap belongs to: `warmup`, `cooldown`, `recovery`,
`interval`, `rest`, `work` (and `custom`). The `workout_step` message (global message number 27)
carries the same names for the steps of the whole session.

So the rule in one line: **two or more of the run's laps carry the step name `interval`.** That is
the watch's own statement about its own session, read from the athlete's own file. The product adds
no threshold, and the row is a fact rather than an inference.

**The row's states.**

| the window's runs | what the row answers |
|---|---|
| some runs carry the watch's workout steps | the row answers, measured by the watch's own structure: the time, the count, the last date, and the change against the comparison period. The card says which measurement answered ("3 runs carry the watch's workout steps"). |
| **no run carries workout steps** | the row has **no value**, and the reason in the core's own words: "not recorded — the runs in this period carry no workout steps". Never "0 m of repeated work", never a zero. |
| a run carries steps but none of them is `interval` | that run is not repeated work — a measured fact about the file, not a missing value. |

**The consequence, stated plainly.** Until the athlete records sessions that carry the watch's own
workout steps, this row has little to say — and **that is the correct answer, not a bug**. The
product is not going to guess a structure into a file that does not have one. The user has been
asked to record one structured interval workout on their watch; the next sync brings the file, and
the row becomes real.

**The lap-shape heuristic is rejected.** The rule that was in the earlier revision — "three
consecutive laps of at least 1 km within ±10 % of each other, holding at least half the run" — is
**not used**, and §7 keeps the reason in one paragraph: applied to the corpus it answers "repeated
work" for a **plain Garmin run with three equal auto-laps**, because auto-laps are equal by
construction. A false positive on a real run is exactly the defect this product exists to avoid,
and the paragraph is worth keeping because it says what the app will not pretend to know.

**What the card says about the measurement.** The coverage line carries it: "{runs} runs carry the
watch's workout steps". A row answered by the watch is a fact from the watch; a row with no value
says why.

---

### 1.3 The kinds that were **not** adopted, and why

The parent asked that the taxonomy stay method-agnostic: a kind that exists only because a coaching
method defines it must not be baked in. The research (`methods-and-products.md` §1.6, §2.7, §3.1)
confirms the rule and gives the reason.

| candidate | status | why |
|---|---|---|
| **Recovery days / short runs** | **not adopted — method-specific.** | Any definition of a recovery day is a method's: Maffetone's 80/20, Pfitzinger's recovery days, Daniels' "recovery run" (a different thing from Pfitzinger's "easy" run). The research's disagreement table (§2.7) shows the same run being "recovery" under one method and "easy" under another. A method-agnostic substitute would have to come from the athlete's own history — "a run shorter than the median of your own runs" — and that is a threshold that moves as the history grows and a label ("recovery") that carries a prescription with it. The ledger answers the underlying question with a **value** instead: the shortest run of the window, alongside the longest. |
| **Sub-threshold volume** | **renamed, not adopted as a label.** | "Sub-threshold" is the Norwegian method's word. The kind exists — it is Z3 — but the ledger calls it **Z3 moderate**, the athlete's own band and the product's existing name (`zones.band.moderate`). |
| **Threshold volume / threshold work** | **renamed.** | "Threshold" is Daniels' and Pfitzinger's word, and the research's table shows the disagreement is a full zone wide: Daniels' threshold sits at 88–92 % of max HR, Norwegian Singles' LT1 at 78–82 % — *the same run* is "threshold" to one and "sub-threshold" to the other. The kind exists — it is Z4–Z5 — and the ledger calls it **Z4–Z5 high**. |
| **MAFS run / MP run** | **not adopted.** | Both require a pace table and a heart-rate cap that only a method defines. Stride has no MAFS pace and no cap. |
| **Fthreshold sessions (2:1, 3:2 ratios)** | **not adopted.** | A ratio of two session types inside a week needs the *plan* to know which session is which, and v1 has no plan anchor. It is also a method-specific structure. |
| **80/20 polarisation as a kind** | **not adopted as a kind; the shape already exists.** | `rolling::polarised_shares` already gives the three-band split with the comparison period beside it. A "polarisation" kind would be a **verdict** ("you are 70/30, which is not 80/20"), and verdicts are banned. The ledger states the shares and the last date; the athlete compares. |
| **Taper / recovery week / race week** | **not adopted.** | Needs a plan, a race, or a calendar event — none of which are anchors in v1. |
| **The pace band as a kind** | **not adopted.** | The pace bands are a property of the **window** (`pace::pace_bands` measures the window's own grid), and they answer a different question: where the time went, by pace. The pace page already owns that. The ledger names the window's fastest band as context, and no more. |
| **Terrain / hill work** | **not adopted.** | The elevation gain is there (`FitLap::elevation_gain_m`, and the climb the file's own records support after the barometer's resets are taken out), but "hills" needs a threshold, and the environment axis is gone (§5). |
| **Repeated work from the lap shape** | **rejected — it misfires** (§7). | Auto-laps are equal by construction; a shape rule cannot tell a workout from a lapped run. The watch's own step names can. |

**The rule the research settles, in one line:** *the kinds are the athlete's own bands, and they
stay that way.* A method's vocabulary may be quoted **beside** the athlete's numbers — that is the
method lens (`methods-and-products.md` §3.3, and the separate `exploration/brief-method-lens.md`),
and it is **never merged into a kind**. The disagreement table is the reason: on the two words
runners care about most — "threshold" and "easy" — the credible sources differ by a full zone, so
a kind named by a method would be a claim about whose method. The ledger's rows are the athlete's
own bands, the counts, the shares and the dates; a lens is a **second, explicitly labelled
projection** over the same history, quoting the method's own published rule, and it is never on by
default.

The mapping the parent asked for, in one line each:

| the parent's list | the ledger's answer |
|---|---|
| easy volume | kind 1, **Z1–Z2 low** |
| sub-threshold volume | kind 2, **Z3 moderate** (the kind exists; the *name* is the method's) |
| threshold-ish volume | kind 3, **Z4–Z5 high** (same) |
| long run | the **longest run** value |
| high-intensity work | the **repeated work** value — the session's own structure, from the watch's workout steps |
| recovery days | **not a kind** — method-specific; the ledger answers with the shortest run of the window instead |

---

### 1.4 Two limits the research puts on the ledger

**The heart-rate lag (Seiler 2010) — what the absence row is reporting.** The research quotes
Seiler: heart-rate time-in-zone "tends to **underestimate the time spent working at high
intensity** (due to heart rate lag time during intervals)"; the software-based methods
"**overestimate the amount of time spent training at low intensity and underestimate the time
spent at very high workloads**". So "the last run in Z4–Z5 was 35 days before the end of the
window" is reporting a property of the watch's heart-rate lag as much as a property of the
training: a session that was hard but short, or run with a lag, can sit below the athlete's Z4
floor.

**The card therefore carries the caveat next to the absence row**, in the core's own words and
with no verdict:

> Heart-rate time in zone counts hard running slowly and easy running generously — the last run in
> Z4–Z5 is what the watch recorded.

And the absence is **corroborated by the run's own structure where the structure is available**: the
watch's workout steps do not depend on heart-rate lag, so a second, independent measurement can
stand beside the first. Where the workout steps are missing, the card says so rather than implying
agreement (§4.5).

**The plan-versus-reality gap (Foster et al., cited in Seiler 2010) — the limit of a history-only
ledger.** The research quotes Foster: mid-level athletes "train **harder than planned on easy days
and at lower intensity than planned on hard days**, relative to coach prescriptions". So an absent
kind has two possible readings, and a history-only ledger cannot tell them apart: the athlete did
not do that kind of work, **or** the athlete did that work differently from the plan. v1 anchors on
history only (decision 3), so the ledger says what the history contains and stops there.

This is a **known limit, not a gap to be papered over**: the question the plan anchor would answer
later is precisely *"did the training match what was intended?"* — the disagreement between the
plan's sessions and the runs that were recorded. That is a different card from this one, it needs
the plan, and it is not built here. The ledger's own vocabulary is neutral about it: it states
**last dates, counts and shares**, never "missed".

---

## 2. The ledger

### 2.1 The window — one window for the whole Progress wall

**The rule.**

```
window_start = week_start(today) - 7 * (weeks - 1)
window_end   = today
previous_start = window_start - 7 days
previous_end   = window_end   - 7 days
```

The window is the last **N calendar weeks**, from the start of the earliest one, **through today**,
and the period it is compared against is **the same span ending seven days earlier** — so the
partial current week compares with the partial week before it. The calendar-week shape survives
(the window still begins on a Monday); only the end moves from Sunday to today.

`weeks` is the Progress route's own parameter (default 12, bound 1..=104, at
`routes::insight::progress`).

**The type.** `rolling::RollingWindow` **cannot express this window**: its length is a fixed number
of days (`from() = to - (days - 1)`), and the parent's window is 79 days for a 12-week window
because the current week is partial — 12 weeks from the Monday of the earliest week through today is
84 days minus the days of the current week that have not happened. The ledger's window is therefore
a **`Period`** — `stride_core::eddington::Period::new(from, to)`, which already has
`contains(date)` and is what `pace::pace_bands` already takes.

> **Flagged for the cleanup pass, not for this slice.** This window is not new to the wall: the
> **Progress totals** (`routes::insight::progress`) and the **pace page** (`routes::pace::pace`)
> already compute `week_start(today) − 7 × (weeks − 1)` through today. The **rolling route**
> (`routes::rolling::rolling`) does not: it uses `RollingWindow::new(7 × weeks, today)`, a rolling
> N-day window, and the streaks and the comparison beside it inherit that. So the ledger now sits
> with the Progress totals and the pace page, and **the rolling route is the odd one out** — it has
> to move onto this window, or two cards on the same screen will describe different spans.

**What the comparison period means.** The two periods **overlap by all but seven days**: for a
12-week window, the current window is 7 September – 24 November 2026 and the comparison period is
31 August – 17 November 2026, so 72 of the 79 days are shared. The change therefore answers
*"what changed in the week that came in, against the week that fell out"* — the same rule the
Progress cockpit's "this week" row already uses ("the same days last week"), generalised from one
week to the window. It is **not** the rolling view's `RollingWindow::previous()`, which shifts the
whole window back by its full length; the ledger's comparison is a **one-week shift**, and the note
says so wherever the comparison is described.

### 2.2 The denominator of the share

The share is over the window's **moving time**, and there are two moving times in Stride:

* the **watch's timer** — `rolling::WindowTotals::moving_time_s`, from the stored activity;
* the **record stream's movement** — the sum of the spans the samples own, which is what
  `zones::TimeInZone` measures the zones over (`TimeInZone::moving_s()` = the zones + the outside).

They differ by one or two per cent on real runs; PLAN's 23 November amendment says so plainly ("the
watch's own timer and the record stream's movement are two measurements").

**Decision: the record stream's movement**, so the numerator and the denominator are the same
measurement and the five bands plus the outside add up to exactly 100 %. The card names the basis
("measured over the moving time"), and the window's stored total is still the store's own number.

**Each share is over its own measurement.** The three kinds' times are measured over the record
stream's movement, so their shares are over it. The **longest run's** time is the watch's own timer
(`moving_time_s`, the stored activity's number — the run detail shows the same), so **its** share is
over the window's stored moving time. A share whose numerator and denominator come from two
different measurements would carry a one-or-two-per-cent error, and the card does not need one.

### 2.3 The ledger, over the Progress window

The synthetic athlete of the prototype (details in the appendix). Window: **7 September – 24
November 2026** (12 calendar weeks through today, 79 days), 58 runs, 49 h 33 m of moving time. The
comparison period: **31 August – 17 November 2026** (the same span, ending seven days earlier),
58 runs, 49 h 20 m.

| kind | time | share of the moving time | runs | last date | against the period before |
|---|---|---|---|---|---|
| Z1–Z2 low | 41 h 32 m | 84 % | 58 | 24 Nov 2026 | +37 m (41 h 32 m against 40 h 54 m) · 84 % against 83 % |
| Z3 moderate | 6 h 52 m | 14 % | 58 | 24 Nov 2026 | −8 m (6 h 52 m against 7 h 01 m) · 14 % against 14 % |
| Z4–Z5 high | 1 h 08 m | 2 % | 4 | **20 Oct 2026** — 35 days before the end of the window | −15 m (1 h 08 m against 1 h 23 m) · 2 % against 3 % · 4 runs against 5 |

| value | time | share | count | date | against the period before |
|---|---|---|---|---|---|
| Longest run | 1 h 59 m (20.4 km) | 4 % | 1 | 21 Nov 2026 | +0.8 km (20.4 km against 19.6 km on 31 Oct 2026) |
| Repeated work | 2 h 15 m | 5 % | 3 | 15 Oct 2026 — 40 days before the end of the window | −45 m (2 h 15 m against 3 h 00 m) · 3 runs against 4 |

And the distribution the card draws above them, over the same window: Z1 **42.4 %** · Z2 **41.7 %**
· Z3 **13.9 %** · Z4 **2.0 %** · Z5 **0.3 %** · outside **0 %** — with the roll-up **low 84.1 % ·
moderate 13.9 % · high 2.3 %** (the period before: Z1 42.5 % · Z2 41.2 % · Z3 14.3 % · Z4 2.4 % ·
Z5 0.4 %, roll-up low 83.7 % · moderate 14.3 % · high 2.8 %).

The three kinds' rows and the two values' rows are **not** added together: the first family
partitions the moving time, the second overlaps it (§1.1).

### 2.4 The last date: searched over the history, not over the window

**The rule.** The last date a kind occurred is the most recent date, **at or before the end of the
window**, on which a run of that kind was recorded — searched over the athlete's **whole retained
history**, not over the window alone.

**Why.** This is the row the wedge is about ("last run above Z4: 41 days ago"). If the last date
were searched inside the window only, a kind that has been absent for longer than the window would
show *nothing at all* — the absence would be invisible, which is the opposite of the point. With the
history-wide search, the window says "4 runs" and the date says "the last one was 35 days before
the end of the window".

**The two facts are different and the card keeps them apart.** The window's own numbers are the
in-window ones. The last date is a fact about the athlete's history, and it may point outside the
window; when it does, the card says so ("35 days before the end of the window"). When a kind occurs
inside the window, the last date is simply the last day of the window's own last run of that kind.

### 2.5 The change against the period before

The change is computed against the **same span, ending seven days earlier** (§2.1) and is reported
in the product's own shape, the shape the year page already uses (`year.changed.distance`:
"{difference} against the period before ({current} against {previous})").

* the **time** — the difference, with its sign;
* the **share** — the difference in percentage points;
* the **count** — the difference in runs;
* the **movement word** — `rolling::Movement::of(current, previous)` gives `Up` / `Down` / `Steady`,
  and the word is the movement and nothing else. It is not a verdict: a share that grew and a count
  that fell are both "up" and "down".

**What the change measures, now that the comparison is a one-week shift.** The two periods share 72
of their 79 days, so the difference is the **week that came in against the week that fell out** —
the cockpit's "this week against the same days last week", generalised to the window. In the
prototype's data the Z4–Z5 row's change (−15 m, 4 runs against 5) is the tempo session of 20
October leaving the window, and the repeated-work row's change (−45 m, 3 runs against 4) is the
1 October workout leaving it. The change is small by construction, and the card must not present it
as a period-over-period comparison.

The change is reported **only when both periods measure the same thing**. If the coverage of the
window and of the period before it differs (different runs carry heart-rate samples), the change is
**not reported** — the two numbers are not comparable and the card says so (§4.5).

---

## 3. The wording

### 3.1 The card's lines

The ledger is the **"Where the time went" card** of the cut Progress wall (§6). Drafted in the
core's voice, in the catalogue's shape. Each is a `statement` — a factual sentence about the data —
except the empty-window line, which is a `status`.

**The screen's source line — one per screen, at the top, above the cards:**

```
stimulus.source              "your zones · {sport} · max heart rate {max_hr} · taken from your watch on {date}"
### OLD
stimulus.zones.basis         "measured over the time you were running"
### NEW
stimulus.zones.basis         "measured over the time you were running"
### OLD
stimulus.coverage.hr         "{runs} of {total} runs recorded a heart rate"
### NEW
stimulus.coverage.hr         "{runs} of {total} runs recorded a heart rate"
### OLD
stimulus.band.none           "no run in this period was in {band}"
### NEW
stimulus.band.none           "no run in this period was in {band}"
### OLD
stimulus.band.before_model   "no zone time for {runs} runs in this period — your watch's zones were first read on {date}"
### NEW
stimulus.band.before_model   "no zone time for {runs} runs in this period — your watch's zones were first read on {date}"
### OLD
stimulus.changed             "{difference} against the same span, one week earlier ({current} against {previous})"
### NEW
stimulus.changed             "{difference} against the same span, one week earlier ({current} against {previous})"
### OLD
stimulus.run.before_model    "no zone time — your watch's zones were not read on this run's date"
### NEW
stimulus.run.before_model    "no zone time — your watch's zones were not read on this run's date"
### OLD
stimulus.longest.changed     "{difference} more than the longest run in the same span, one week earlier ({current} against {previous})"
### NEW
stimulus.longest.changed     "{difference} more than the longest run in the same span, one week earlier ({current} against {previous})"
### OLD
stimulus.steps.not_corroborated "the runs in this period carry no workout steps — the date above is the heart rate's answer alone"
### NEW
stimulus.steps.not_corroborated "the runs in this period carry no workout steps — the date above is the heart rate's answer alone"
### OLD
stimulus.no_runs             "no runs recorded"                                (status)
stimulus.steps.no_data       "not recorded — the runs in this period carry no workout steps"
### NEW
stimulus.no_runs             "no runs recorded"                                (status)
stimulus.steps.no_data       "not recorded — the runs in this period carry no workout steps"
### OLD
> **41 h 32 m** of the time you were running was in Z1–Z2 — 84% of it — across 58 runs. The last run
> in Z1–Z2 was on 24 Nov 2026.
### NEW
> **41 h 32 m** of the time you were running was in Z1–Z2 — 84% of it — across 58 runs. The last run
> in Z1–Z2 was on 24 Nov 2026.
### OLD
> **1 h 08 m** was in Z4–Z5 — 2% of the time you were running — across 4 runs. The last run in Z4–Z5 was
> on 20 Oct 2026, 35 days before the end of this period.
### NEW
> **1 h 08 m** was in Z4–Z5 — 2% of the time you were running — across 4 runs. The last run in Z4–Z5 was
> on 20 Oct 2026, 35 days before the end of this period.
### OLD
* **The absence is a date.** The ledger states **when** the kind last occurred. It never states
  what was not done, and it does not need the word the ban list reserves for that.
### NEW
* **The absence is a date.** The ledger states **when** the kind last occurred. It never states
  what was not done, and it does not need the word the ban list reserves for that.
### OLD
`zones.no_model` — "no zones yet — your watch's heart-rate zones have not come through".
### NEW
`zones.no_model` — "no zones yet — your watch's heart-rate zones have not come through".
```

The line is the model's **identity and its date**: the sport and the method, the maximum heart rate,
and the day it was read. The core's `zones::model_source` already renders the method, the maximum
heart rate and the read day; the sport is added by the api, which carries it on the model. **"details
›" opens the model's detail** — the five floors, the method, the read day.

**One per screen, not one per card.** The Progress wall's six cards share the line, because they
share the model; repeating it six times would be exactly the clutter the change was meant to remove.
A screen with **no model carries no line at all** — there is nothing to name, and the card says what
is missing.

**The distribution, and the roll-up (printed once each):**

```
stimulus.zones.distribution  "Z1 {z1} · Z2 {z2} · Z3 {z3} · Z4 {z4} · Z5 {z5} · outside {outside}"
stimulus.zones.rollup        "low {low} · moderate {moderate} · high {high}"
stimulus.zones.basis         "measured over the time you were running"
```

**The coverage, with the rows it qualifies (never in a footer):**

```
stimulus.coverage.hr         "{runs} of {total} runs recorded a heart rate"
stimulus.coverage.steps      "{runs} runs carry the watch's workout steps"
```

**The kinds:**

```
stimulus.band.time           "{time} · {runs} runs"
stimulus.band.last           "the last run in {band} was on {date}"
stimulus.band.last_gap       "the last run in {band} was on {date} — {days} days before the end of the window"
stimulus.band.none           "no run in this period was in {band}"
stimulus.band.before_model   "no zone time for {runs} runs in this period — your watch's zones were first read on {date}"
stimulus.changed             "{difference} against the same span, one week earlier ({current} against {previous})"
stimulus.changed.runs        "{current} runs against {previous} runs"
```

and, on the run detail, the same exception for one run:

```
stimulus.run.before_model    "no zone time — your watch's zones were not read on this run's date"
```

**The values:**

```
stimulus.longest             "{distance} · {duration} · on {date}"
stimulus.longest.changed     "{difference} more than the longest run in the same span, one week earlier ({current} against {previous})"
stimulus.repeated            "{time} · {runs} runs"
stimulus.repeated.last       "the last run with a workout step was on {date}"
```

**The caveat, and the corroboration:**

```
stimulus.zone.lag            "heart-rate time in zone counts hard running slowly and easy running generously — the last run in {band} is what the watch recorded"
stimulus.steps.corroborates  "the last run with a workout step was on {date} — {runs} runs carry the watch's workout steps"
stimulus.steps.not_corroborated "the runs in this period carry no workout steps — the date above is the heart rate's answer alone"
```

**The states:**

```
stimulus.no_runs             "no runs recorded"                                (status)
stimulus.steps.no_data       "not recorded — the runs in this period carry no workout steps"
```

### 3.2 The three sentences the prototype shows, in full

> **41 h 32 m** of the time you were running was in Z1–Z2 — 84% of it — across 58 runs. The last run
> in Z1–Z2 was on 24 Nov 2026.

> **1 h 08 m** was in Z4–Z5 — 2% of the time you were running — across 4 runs. The last run in Z4–Z5 was
> on 20 Oct 2026, 35 days before the end of this period.

> The longest run in the window was **20.4 km** in 1 h 59 m, on 21 Nov 2026. The longest run in the 12
> weeks before it was 19.6 km.

And the two lines that sit with the absence:

> Heart-rate time in zone counts hard running slowly and easy running generously — the last run in
> Z4–Z5 is what the watch recorded.

> the last run with a workout step was on 15 Oct 2026 — 3 runs carry the watch's workout steps

### 3.3 The rules the wording follows, and the ban-list check

* **Facts only.** Every sentence is checkable against the stored runs. There is no verdict, no
  score, no readiness, no form, and nothing that tells the athlete what to do.
* **The absence is a date.** The ledger states **when** the kind last occurred. It never states
  what was not done, and it does not need the word the ban list reserves for that.
* **"No run recorded", never "rest".** A window with no runs says "no run recorded in the window".
  A day without a recorded run may be a forgotten watch, not a day off — the research states it
  plainly, and the vocabulary follows (§4.1).
* **The caveat is a statement about the measurement.** Not an excuse, not a hedge, not a verdict: it
  says what the heart rate does, and that the date is the watch's answer.
* **The watch's word, the watch's name.** The repeated-work row says "workout step", because that
  is the name in the athlete's own file; the product does not rename the watch's structure.
* **One source line per screen, not one per row and not one per card.** The model and the day it
  was read are named **once, at the top of the screen, above the cards**, and the rows stay clean:
  the picture carries the numbers, the source line carries the model, and the model's detail is one
  click away. The wall's cards share the line because they share the model. **The exception is an
  answer that used a different model** — a run that predates the first stored model has no zone
  time, and its row says so in place, because a silent difference would be a wrong number. The same
  hoisting applies to the run detail's zone table, the window's split and the load card's scenario:
  one place per screen, not one place per answer.
* **No "you".** The sentences are about the runs. The one place the athlete is named is the source
  line, which the product already owns: `zones.model.source` — "your zones · max heart
  rate 183 bpm · read on 3 Nov 2026".
* **The window is named.** A number is only a fact against a stated period, so the card names the
  window and the comparison period with it.
* **The rendering is the core's.** Every sentence comes out of `strings::render(...)` with the
  values already in it — the lint refuses a rendered statement that still carries a hole
  (`strings::lint::check_rendered_statement`). The app writes no sentence of its own.
* **The dates are the api's.** "24 Nov 2026" is `routes::day_label` (`%-d %b %Y`); the core's own
  canonical form (`2026-11-24`) stays inside the core.

| banned | in the drafted lines? |
|---|---|
| should · must · need to · recommended · better to · ready to | no |
| missed · bad · poor · fail · behind · good | no |
| red · readiness score · not recovered · you are ready | no |
| take the day · rest day · rest up · rest today · go easy · keep it easy | no |
| "rest" as a noun for an unrecorded day | no — the card says "no run recorded" |

The word "less" appears ("15 m less against the period before"), which is arithmetic and not a
verdict — the same word the year page already uses in `year.changed.distance`.

---

## 4. The states

### 4.1 No run recorded in the window

A window with no runs. The card shows the window's name and one line:

> **no run recorded in the window**

(`stimulus.no_runs`, a `status` string — the catalogue's `status.*` family is exactly this: the
connect/sync/staleness/empty-state lines.)

* **No rows.** The kinds are not shown with zeros. There is nothing for them to hold.
* **No second reading.** The card does not say the athlete did not run, and does not compare with
  the period before: "the period before holds 58 runs" is a fact about a different period and would
  be a comparison the reader would have to undo.
* **"no run recorded", never "rest".** A day with no recorded run may be a forgotten watch. The
  card's vocabulary is the recorded fact, not an inference about the athlete's week.
* The zone model is not shown either — a model with no window is not a fact about the window.

### 4.2 One run in the window

The card answers for the run that is there.

* The kinds the run carries carry their numbers: "33 m", "95% of the moving time", "1 run", "the
  last run in Z1–Z2 was on 8 Nov 2026".
* A kind the run does not carry shows a **measured zero**: "0 m", "0% of the moving time", "0 runs",
  and **no date**. The sentence is "no run in the window was in Z4–Z5" — a fact about the window,
  and not a complaint.
* The repeated-work row shows **no value** and its reason: "not recorded — the runs in this period carry
  no workout steps". Not "0 m of repeated work".
* The change against the period before is reported normally — the two periods are the same length,
  and the product's rule is the same length, not the same content.

### 4.3 No stored zone model

* The three kinds have **no value**: "no zones yet — your watch's heart-rate zones have not come through"
  (`zones.no_model`, the string the zones route already uses). No time, no share, no run count, no
  last date, and no distribution line.
* The two values **still answer** — the longest run and the repeated work do not depend on the zone
  model, and the card does not withhold them. **The card can show one family and not the other.**
* The model is not replaced by a guess. The core's rule: "A run with **no stored model is not
  calibrated**: the result is 'no value', never band 1 and never zero" (`zones::time_in_zone`
  returns `None`; `ZoneFloors::is_usable` is the test).

### 4.4 A window whose runs carry no heart-rate data

The same shape as §4.3, for a different reason: the model is there, but the runs carry no
heart-rate samples.

* The intensity block has **no value**: `rolling.zones.no_samples` — "no heart rate recorded — none of the
  runs in the window have no heart-rate samples". No distribution line, no band rows, no last dates.
* The shape block **still answers**: the longest run and the repeated work come from the runs'
  distance, time and workout steps, and none of those needs a heart rate. **One family answers and
  the other does not**, and the card says which is which.
* The absence is not the athlete's: no band row means "no value", never "0 m", and the reason is
  named.

### 4.5 Partial coverage, and no workout steps

Some of the window's runs carry heart-rate samples and some do not.

* The card **names the coverage** with the rows it qualifies: "55 of 58 runs carry heart-rate
  samples", and the shares are measured over the measured movement ("measured over the moving time"
  — the movement the bands were measured over).
* The kind's run count is the count of the runs that carry it.
* **The change against the period before is not reported** when the two periods do not hold the
  same runs — the two numbers are not comparable, and a comparison between them would be a fact
  about the calendar, not about the running.
* **The workout-step coverage travels with the repeated-work row**, and the corroboration line
  changes with it:
  * **with workout steps** — "the last run with a workout step was on 15 Oct 2026 — 3 runs carry the
    watch's workout steps": the second, independent measurement stands beside the first, and the
    card does not say whether they agree;
  * **without workout steps** — the repeated-work row shows **no value** ("not recorded — the runs in
    the window carry no workout steps"), and one line under it says what could not be measured:
    "the runs in the window carry no workout steps — the date above is the heart rate's answer
    alone". The card says so rather than implying agreement it cannot support.
* The heart-rate lag line stays in both cases: the heart rate's answer is the heart rate's answer,
  whether or not something else was measured beside it.

### 4.6 When the ledger cannot be trusted

| situation | what the card shows |
|---|---|
| **No heart-rate samples anywhere in the window** | the intensity block has no value: `rolling.zones.no_samples`. |
| **The stored model's floors are not in order** | no value: `zones.unusable_model`. |
| **A run dated before the first stored model** | that run has no zone time, and the band's row says so **in place** — "no zone time for 3 runs in the window — the first stored zone model was read on 10 Sep 2026". The screen's source line still names the model the rest of the window was measured against, and the two are not merged. |
| **Partial coverage** | as §4.5. |
| **No run carries the watch's workout steps** | the repeated-work row has no value, with the reason (§4.5). |
| **The zone model was read after the window** | the ledger measures each run with the model **in force on its date** (§1.2), so this cannot arise inside the ledger. It **does** arise between the ledger and the run detail until Stage 1 of the migration lands (§6.2) — that is the disagreement this rule exists to remove, and it is flagged, not hidden. |

---

## 5. The environment axis — dropped, and the measurement that closed it

**The user's clarification.** "Running too hot or too cold" means **exertion and training load**,
not weather ("are you over exerting basically load or not"). The environment axis is therefore
**out of scope**, not deferred. There is no FIT temperature decoding, no weather source, no heat
adjustment, and nothing to design later in this thread.

**The measurement, kept as the record of why it closed.** The FIT `record` message carries the
ambient temperature as **definition number 13**, base type **`sint8`**, in degrees Celsius. The
decoder in `crates/stride-ingest/src/fit.rs` has every piece it would need — `RawMessage::number`
reads fields by definition number, `decode_value` already handles `sint8` (base type 0x01, with the
FIT invalid value 0x7F mapped to "no value"), and `Importer::visit_record` reads the record's
fields with the helper `scaled(message, definition, scale, offset)` — so one more call,
`in_range(scaled(message, 13, 1.0, 0.0), -40.0, 60.0)`, would read it. The original file is kept on
the activity row (`activities.fit_blob`), so it would be recomputed on read with no re-download and
no schema change.

**And here is what the corpus says.** I parsed the record-message definitions of all sixteen FIT
files in `fixtures/fit/`: **not one carries definition number 13.** The three real Garmin files —
`real_garmin_run.fit`, `real_garmin_gap.fit` and `real_garmin_nulls.fit` — carry, between them, the
record fields 0, 1, 2, 3, 4, 5, 6, 53, 73, 78, 87, 90, 134, 135, 136 and 253, and no 13. So on the
evidence available, this athlete's watch records **no temperature at all**, and the coverage for a
condition axis would be zero.

**What would have to be true for the axis to exist, if it ever came back:** a watch with a
thermometer **and** temperature recording enabled in the watch's own settings, so that field 13 is
written into the file; and a real file on this device that shows the field. Until a real file shows
it, the axis has no input. That is the whole of the forward statement — the axis is closed, not
queued.

**Where the question actually lives now.** The "too hot" side of the user's question is already
answered by the existing **load card** on the cut wall: acute, chronic, form, the ratio, the
monotony and the strain, computed by `crates/stride-core/src/load.rs`, with the model's assumptions
named where the numbers are read (`k = 1.92`, the maximum heart rate from the stored zone model, the
two time constants) and the scenario — what the ratio is expected to do with a rest day. That is
exertion and training load, measured from the athlete's own history, with the model's assumptions
stated. **The ledger answers the other side**: what the weeks contain, what they lack, and where the
training has been quiet. **The two cards are neighbours, not rivals** — the load card says how much
and how the load sits; the ledger says of what, and when.

**What altitude already gives, and why it is not a substitute.** `fit.rs` decodes the altitude from
the record message (field 2, with the enhanced field 78 as the fallback) and **rebases** it:
`rebased_altitudes` calls `rebased_elevation`, which takes out the barometer's own resets (a change
of more than `RESET_STEP_M` = 50 m is the sensor losing its signal, and the readings until they come
back within `RESET_TOLERANCE_M` = 30 m are replaced by the straight line between the two trusted
levels), and the climb the file's own records support is then `climb_of(&rebased)`. So the product
has a **cleaned altitude series** and a trustworthy elevation gain — a fact about the ground, which
is worth having on its own terms, and which the ledger's card does not use. It is not a heat model:
altitude is a position rather than an air temperature; the two are related only through a lapse rate
that is an average of a standard atmosphere; the barometric level is itself weather-dependent; the
variation within a run is a degree or two; and humidity — which is most of the heat load — is
absent from the FIT record message entirely.

---

## 6. Where it lives: the "Where the time went" card

**The ledger is not a new card.** The surface cut (`exploration/surface-inventory.md` §6) took the
Progress wall from 24 cards to 6, and card 4 is:

> **4. Where the time went** — *(merges `zones` + `split` + the zone-time distribution)*
> Z1 **18%** · Z2 **41%** · Z3 **26%** · Z4 **11%** · Z5 **4%** · outside **0%**
> *low 59% · moderate 26% · high 15%*
> *measured over the time you were running · your zones, taken from your watch on 12 Nov 2026*

**That card becomes the ledger card**, extended with the rows this note describes:

* the **screen's source line, above the cards** — the model and the day it was read, **once for the
  screen**, with the model's detail one click away. The card does not carry it, and no row names
  the model again;
* the **five zone shares** (printed once), with the **polarised roll-up** as the one line that names
  the bands the ledger's rows are grouped into — the card must not print the zone shares twice;
* the **three kind rows** — time, run count, the last date, and the change against the period
  before. **The rows do not repeat the shares**: the share is read from the roll-up above;
* the **two value rows** — the longest run and the repeated work;
* the **coverage lines**, with the rows they qualify: the heart-rate coverage with the kind rows,
  the workout-step coverage with the repeated work;
* the **heart-rate lag line**, with the absence row it qualifies;
* the **corroboration line**, with the absence row.

**One card, no new surface, and no number printed twice.** The prototype shows exactly this shape.

**Where the rule applies.** Wherever the model was named per answer, it is named once per screen:
the ledger card, the run detail's Effort panel, the window's zone split, and the **load card's
scenario** — which today repeats the maximum heart rate and the read day in its own line and must
not: it keeps the model's own constants (Banister k = 1.92) and the scenario, and the model is
named once at the top of the screen.

**And one line on the run detail:** the run's own row in the ledger — which band its time fell in,
how long, and the last date that band occurred. The run detail's own zone table and the ledger
must not disagree, which is why §1.2's dated-model rule has to reach the run detail in the cleanup
pass.

### 6.1 Where the code would go

**One new core module** — `crates/stride-core/src/stimulus.rs` — beside the other window-shaped
analyses, with:

```
pub enum StimulusKind { Easy, Steady, Hard }              // the kinds; the values are a second list
pub enum StimulusValue { LongestRun, RepeatedWork }
pub struct StimulusRun { date, distance_m, moving_time_s, zone_time: Option<TimeInZone>,
                         workout_steps: Option<Vec<WorkoutStep>> }
pub fn ledger(runs: &[StimulusRun], window: Period, previous: Period) -> Option<StimulusLedger>
pub struct StimulusRow { kind, time_s, share_percent: Option<f64>, runs: usize, last_on: Option<Date>,
                         previous_time_s, previous_share_percent, previous_runs, movement: Option<Movement> }
```

and the rendering functions that ask the catalogue for the sentences.

**What it reuses, by name:**

| need | the existing thing |
|---|---|
| the zone model and the time in zone | `zones::ZoneFloors`, `zones::time_in_zone`, `zones::TimeInZone::{per_zone, moving_s}`, `zones::zone_of`, `zones::model_source` |
| the window and the comparison | `eddington::Period` (the window), `rolling::totals_in`, `rolling::compare_windows`, `rolling::ZoneTimes`, `rolling::polarised_shares`, `rolling::Movement::of`, `rolling::RollingActivity` |
| the pace bands (context only) | `pace::pace_bands`, `pace::PaceBand::label`, `pace::BAND_WIDTH_S_PER_KM` |
| the workout steps | the ingest's lap message: definition 24 (`wkt_step_name`) and the `workout_step` message (27) — **both to be read** (§6.3) |
| the sentences | `strings::render`, `strings::text`, `strings::CATALOGUE`, `strings::StringKind`, `strings::lint::check_rendered_statement` |
| the rendering of the numbers | `units::render_duration`, `units::render_distance`, `units::render_share`, `units::round_half_even` |
| the dates | `date::Date`, `date::Date::{parse, to_iso, add_days, days_since}` |
| the run's own words (for the run detail's line) | `story::story`, `story::story_with_why`, `story::RunFacts`, `statement::distance_comparison`, `statement::CompareBasis` |

**On the api side:** the ledger needs each run's samples and workout steps, decoded on read. The
precedent exists: `stride_api::pace::pace_runs(api, &runs)` decodes every run's stored FIT and
builds the core's `PaceRun`s, and `stride_api::routes::series::{decoded, zone_samples,
pace_points}` are the helpers that turn a decoded file into the core's sample types. A
`stimulus_runs(api, &runs)` would sit beside them. **The cost is real** — a 12-week window is 58
file decodes on every read — and it is worth a look at caching before the card ships (the load route
and the pace page have the same shape of cost today).

### 6.2 The zone model's migration, in two stages

| stage | what moves | when |
|---|---|---|
| **Stage 1** | every run is measured with the zone model **in force on the day of the run** for the **zone-time answers**: the run detail's zone table, the window's zone split, the ledger | the cleanup pass |
| **Stage 2** | the **load model** (`load.rs`: TRIMP's threshold and maximum heart rate) moves to the dated model per run | later, separately — it changes the load history the product has shown since v1, so it is its own change with its own announcement |

**The rule that holds in both stages:** **one source line per screen** — at the top, above the
cards — names the model and the day that model was read, and **an answer that used a different model
says so in place**. The core has `zones::model_source` for the line; the api adds the sport, and
"details ›" opens the model's detail. In the prototype the screen's line reads "your zones ·
running · HR_MAX · max heart rate 183 bpm · read on 3 Nov 2026", once, at the top.

**The alignment, not the invention.** `docs/garmin-api-capabilities.md` already says it: *"today's
settings cannot be passed off as historical settings"*. Stage 1 makes the product's answers obey
that; it does not add a new idea.

**While Stage 1 is pending,** the ledger and the run detail can disagree for runs dated before the
first stored model — the ledger uses the model in force on the day (or no model at all), the run
detail still uses the current one. The screen's source line names the model once and the affected
row says so in place, so the disagreement is visible rather than silent, and it disappears when
Stage 1 lands.

### 6.3 The ingest, for the workout steps

`fit.rs` reads the lap message (`MESG_LAP`) by definition number — 7, 8, 9, 13, 14, 15, 16, 17, 18,
21, 22 today — and does not read **24** (`wkt_step_name`); it does not read the `workout_step`
message (27) at all. Both are one branch and one field away, and both are in `crates/`, so they are
not in this slice. The repeated-work row has no value until they are.

---

## 7. The rule we rejected, and why

One paragraph, kept on purpose: it says what the app will not pretend to know.

**The rejected rule.** A run was *repeated work* when its own laps held a repeating structure: at
least three **consecutive** laps of at least **1 km**, whose lengths were within **±10 %** of each
other, holding at least **half** the run's moving time. It is a plausible rule, it is checkable, and
it is wrong.

**Why it is wrong, from the corpus** (decoded with the product's own `stride_ingest::decode_fit`,
and the rule applied with the core's own `fingerprint::running_only` and `fingerprint::measurable`):

| file | laps | lap lengths | the rule's answer |
|---|---|---|---|
| `real_garmin_run.fit` | 3 | 605 m, 618 m, 656 m | **"repeated work"** — and this is a *plain run with three auto-laps* |
| `hr_gps_run.fit` | 6 | 327 m × 6 | "repeated work" — auto-laps |
| `cycling.fit`, `no_cadence.fit`, `vendor_strava_export.fit`, `vendor_wahoo_export.fit` | 3–6 | equal | "repeated work" — auto-laps |
| `mixed_run.fit` | 2 | 238 m, 202 m | no value (both laps are fragments under 300 m; fewer than three) |
| `short_effort.fit` | 1 | 270 m | no value (a fragment) |
| `real_garmin_gap.fit` | 2 | 1168 m, 271 m | no value |
| `real_garmin_nulls.fit` | 12 | 137–170 m | no value (every lap is a fragment) |
| `gaps.fit`, `hilly.fit`, `low_resolution.fit`, `treadmill.fit`, `vendor_coros_export.fit`, `zz_compressed_timestamp.fit` | 1–3 | 105–459 m | no value |

**Auto-laps are equal by construction**, so a shape rule cannot tell a workout from a run the watch
simply lapped — and the rule's false positive on a *real Garmin run* is exactly the defect this
product exists to avoid. The 1 km floor does not save it: it rejects every repeating shape in the
corpus, so it cannot be validated against anything. **The watch's own step names can tell them
apart**, because the watch wrote them.

**And the corpus has no workout structure to learn from either.** No file carries lap definition 24
(`wkt_step_name`), and no file carries a `workout` (26) or `workout_step` (27) message. The lap
messages carry:

* the thirteen synthetic fixtures: 7, 8, 9, 13, 14, 15, 16, 17, 18, 21, 22, 253 (`no_cadence.fit`
  drops 15 and 16);
* `real_garmin_run.fit`, `real_garmin_gap.fit`, `real_garmin_nulls.fit`: 0, 1, 2, 7, 8, 9, 13, 14,
  15, 16, 17, 18, 21, 22, 110, 111, 253, 254.

So the watch's own path cannot be built or tested from the corpus at all — it needs a real file from
a structured workout, which is what the user has been asked to record.

---

## 8. Open questions, before any code is written

The parent's eight questions and the revision-3 decisions are answered (§0). What is left:

1. **The workout-step path needs a file.** Path 1 cannot be built or tested until a real file that
   carries definition 24 (or a `workout_step` message) exists — a Garmin workout run with interval
   steps. Until then, does the repeated-work row ship at all, or does the card show it with its
   "no value" line until a workout appears?
2. **The comparison's overlap.** The window and its comparison period share 72 of 79 days, so the
   change answers "the week that came in against the week that fell out". Is that the change the
   card should carry, or should the card carry the full-length comparison (`RollingWindow::previous()`)
   as well?
3. **Stage 1's order.** Which moves first — the run detail's zone table, the window's zone split, or
   the ledger? The ledger and the run detail must not disagree, and until Stage 1 lands they can.
4. **The card's density.** The merged card carries a distribution line, a roll-up, three kind rows,
   two value rows, two coverage lines, a caveat line and a corroboration line. On the phone that is
   a long card. Does the phone show the full card or fold the change lines away (the prototype has
   both forms)?
5. **The run detail's line** — which of the ledger's fields travel to the run detail: the run's own
   band time, its last date, the run's own row, or all three?
6. **The window's tail in the wording** — the card's window ends today, and "35 days before the end
   of the window" is measured to today. The Progress wall already marks the partial current week;
   does the ledger's card mark it too?
7. **Where the model's detail opens.** The click-through is settled — "details ›" opens the model's
   detail: the five floors, the method, the read day — but not where it lives: a sheet on the screen
   it was opened from, or the run detail's own model panel.

---

## Appendix — the synthetic data behind the prototype

* **One athlete**, 25 weeks, 135 runs, 8 June – 24 November 2026.
* A weekly pattern — an easy Monday, a short Tuesday, a steady Wednesday, a hard Thursday (every
  week in the summer, every second week in the autumn, then none), a long Saturday that grows from
  14.0 km to 20.4 km, an easy Sunday — with the zone splits typical of each kind of run.
* **The deliberate gap:** the last run with time in Z4–Z5 was on **20 October 2026**, 35 days before
  the end of the window — five weeks without a run above the Z4 floor. The comparison period holds
  five such runs.
* **The workout sessions** — the Thursday hard sessions of 17 September, 1 October and 15 October —
  are the runs the watch recorded as workouts with interval steps. The 20 October tempo is a plain
  run with no steps, so the two measurements on the card point at different dates.
* **Two weeks away from the sport** (2–15 November), of which one run — 6.0 km on 8 November —
  was recorded. That gives the prototype its real empty window (9–15 November) and its real
  one-run window (2–8 November).
* Everything is synthetic. No real athlete data, no Garmin call, no network, no file under
  `crates/` or `frontend/` touched, nothing committed.
