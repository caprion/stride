# The parent's decisions — 24 November 2026

*One log, so five threads and one user can read the same decisions. Each entry
names the thread it binds. Nothing here is code; the cleanup and the feature
work come after.*

## D1 — The window (binds: ledger, cleanup audit)

One window for the whole Progress wall:

* **window** = the last *N* calendar weeks, from the start of the earliest one
  **through today**: `week_start(today) − 7 × (weeks − 1)` → today.
* **compared period** = the **same span ending seven days earlier** — the
  week that came in against the week that fell out, so the partial week
  compares with the partial week.

The Progress **totals** and the **pace page** already compute the window this
way; the **rolling route** is the odd one out and is migrated in the cleanup
pass.

The window's length is therefore not a round number of days (79 for 12 weeks
on a Wednesday) and cannot be expressed by `rolling::RollingWindow` (fixed
length). The ledger's window is a `stride_core::eddington::Period`. The
product's own comparison is the one-week shift, not the full-length shift.

## D2 — One comparison rule (binds: ledger, cleanup audit)

**Every window-shaped answer on the wall compares with the same span, one
calendar week earlier.** The rolling route's one-day shift (`RollingWindow::
previous()`, which overlaps by all but one day) is not a second rule — it is
the reading the surface cut already retires.

## D3 — Zone model, staged (binds: ledger, cleanup audit)

* **Stage 1 (cleanup pass):** every run is measured with **the zone model in
  force on the day of the run**, for the **zone-time** answers — the run
  detail's zone table, the window's zone split, the ledger.
* **Stage 2 (later, separately, with its own announcement):** the **load
  model** (`load.rs` — TRIMP's threshold and max heart rate) moves to the
  dated model per run. It changes load history the product has shown since
  v1, so it is its own change, not a quiet refactor.
* **Always:** every answer names the model it used and the day that model was
  read (`zones::model_source`).

The repo's own capability note already says it: *"today's settings cannot be
passed off as historical settings"*.

**Sequencing consequence:** the ledger ships **after** Stage 1. The ledger
alone would disagree with the run detail until Stage 1 lands.

## D4 — Repeated work: the watch's own steps only (binds: ledger)

Path 1 only — FIT lap definition 24 (`wkt_step_name`); a run is repeated work
when two or more of its laps carry the step name `interval`.

The lap-shape heuristic is **deleted**: it answers "repeated work" for a plain
run with three equal auto-laps (`real_garmin_run.fit`) and five other files,
because auto-laps are equal by construction. The note keeps one paragraph on
the rule we rejected and why.

States: steps present → the row answers; no run with steps → **no value**
("the runs in the window carry no workout steps"), never "0 m of repeated
work"; steps present but none is `interval` → a measured fact, not a missing
value.

**Blocked on the athlete:** until a structured workout is recorded on the
watch and synced, this row has little to say — which is the correct answer.

## D5 — Shares name their denominator (binds: ledger)

The intensity bands' shares are over the **record stream's movement** (so the
bands and the outside add up to 100 %). The longest run's share is over the
**stored moving time** (the watch's timer — the run's own number). Shares
over different denominators are **not added together**, and each row names
its denominator.

## D6 — The environment axis is dropped, not deferred (binds: all)

"Running too hot or too cold" means **exertion and load**, not weather (the
user's own clarification). No FIT temperature decoding, no weather source, no
heat adjustment, no heat-adjusted pace. The corpus measurement — no record
message in `fixtures/fit/` carries field 13 — stays in the ledger's §5 as the
record that closed the question.

The Garmin API question (does the *service* carry a per-activity temperature?)
is answered separately by its own thread; a temperature that exists would be a
**fact for the run detail's context**, never an adjustment.

## D7 — The surface cut (binds: cleanup audit)

* Progress wall **24 cards → 11** (6 on Progress, 5 on the athlete profile).
* Run detail **13 panels → 10**; three stacked charts become **one** chart
  with pace / heart rate / altitude, toggled by the user, one shared cursor,
  time on the x-axis.
* The `/pace` page is **retired** into Progress panels.
* The **radar is retired** (the seven facets stay).
* The polarised split merges into the zone card; the ledger **is** that card.
* The run detail keeps a **one-line link** to the night's sleep (the panel
  moves to Body).

## D9 — Temperature: the API has it, this account does not (binds: cleanup audit)

The Garmin Personal API **does** expose a per-activity temperature:
`SummaryDTO.averageTemperature` / `maxTemperature` / `minTemperature` (and
the same three per split) on `GET /activity-service/activity/{activityId}`, and
`SummarizedActivityDTO.minTemperature` / `maxTemperature` on the search
endpoint.

**On this account the keys are absent** — not null — from all 100 activities
of a 12-month search page, all 336 split objects and the detail payload,
while training effect, heart rate and elevation were populated. The API
carries **no weather at all** (no humidity, no wind, no "feels like").

So: the environment axis stays dropped (D6), and **no work is needed** — no
schema change, no sync change, no ingestion. If the athlete ever records
temperature on a thermometer-equipped watch, the field may populate; then it
is a **dated fact for the run detail's context**, never an adjustment.

## D10 — The activity search response already carries more than we keep (binds: cleanup audit)

The live read showed that fields Stride currently discards arrive on the
**activity search endpoint we already call** — so keeping them costs **no
extra requests**, only storage and schema:

* **observations worth keeping** (a fact about the activity): aerobic and
  anaerobic **training effect** and their messages, `trainingEffect`,
  `hrTimeInZone_1..5` (the source's own time in zone — an **independent
  cross-check** on our computed zone time, never a replacement for it),
  `directWorkoutRpe` and `directWorkoutFeel` (**the athlete's own recorded
  effort** — the one thing the research says no watch can derive), moderate
  and vigorous minutes, respiration, steps;
* **judgements to keep out** (Garmin's estimates, excluded by the contract):
  VO₂ max, training stress score, activity training load, lactate threshold,
  intensity factor, stress, body battery.

Each kept field is stored **with its provenance** (source, the day the
response carried it, the day it was read), like `wellness_observations`.
Absence is a missing value with a reason — never zero.

## D11 — The picture first, the sentence second (binds: ledger, method lens, cleanup audit)

The user's directive, 24 November 2026: **less verbose, more visual** — in the
app itself.

* A card is read as a **picture** first: the shape, the band, the gap, the
  streak. The sentence is the **caption** under it, and the accessible
  fallback — a number is still a number, and a number needs its period and
  its reason stated in words.
* **Absence is drawn, not described.** The twelve-week strip, coloured by
  what each week held, makes a five-week hole in the hard band visible as a
  hole — before any sentence is read. The sentence then names the date.
* Two bars are not two facts: the five zone shares and the polarised
  roll-up are **one bar, printed once** (D5's siblings in the cut).
* The chart is not decoration: it must be readable on its own, and the
  caption must not repeat what the picture already says.

**Applies to:** the ledger card ("Where the time went"), the method-lens
card, the consolidated chart, and every card that survives the cut — a
surviving card that is only a sentence should be redrawn as a picture, or
merged.

## D12 — The method lens: catalogue and rules (binds: method lens, cleanup audit)

**Kept (three, all computed):**

| lens | the rule | computable? |
|---|---|---|
| **L1 Threshold band** (Daniels / VDOT O₂) | "83–88% of VO₂max or 88–92% of HRmax" | yes, from the stored zone model's maximum heart rate |
| **L2 The cap** (Maffetone) | "Subtract your age from 180…" | only with an **athlete-entered cap** — Stride never computes 180 − age (no age formula, no birthdate stored) |
| **L3 Easy-day ceiling** (Bakken) | "lactate below 1.0 mmol/l, heart rate below 70% of max" | **half** resolves (the heart-rate clause); the lactate clause does not, and the card says so |

**Refused, and listed with the reason, not selectable:** Seiler / 80–20 (stated
in blood lactate; the paper's own HR method contradicts it), Pfitzinger
(stated in marathon pace; the quotable HR percentages exist only in a
third-party summary), Norwegian Singles (LT1 lactate plus RPE and talk test).

**Rules:**

1. A two-clause rule computes **the clause that resolves** and **states the
   clause that does not** — the method is never quietly halved.
2. The picker lists the refused methods **with their reason**, unselectable.
   The app asserts none of them; the lens is the athlete's choice.
3. The band or ceiling value may sit in the Shape B header — it is a fact
   about the athlete's own zone model.
4. The **athlete's own cap is a dated fact** (value, day entered, day last
   changed), stored beside the zone model's dated facts. A window that spans
   a change measures each day against the cap **in force that day** (D3's
   rule), and the card names the cap and the day it was read.
5. **No lens on a run's own page** — a run's page shows the run's own facts;
   a lens reads a window.

## D23 — There is no zone-model history; the watch's own zone minutes are the fourth path (binds: ledger, sync, Pass 3)

**The finding (verified against the spec and one live read):** the Garmin
Personal API serves the zone configuration **as it stands now** — one record,
no date, no effective-from (`/biometric-service/heartRateZones/v2`, its only
parameter `generateDefaults`). **There is nothing to backfill.** The only date
Stride can ever attach is the day it read it.

So Z's rule leaves **every imported run** with no zone times — and the
ledger's intensity rows would be empty over exactly the athlete's history.

**The fourth path:** `hrTimeInZone_1..5` ride on the **search payload Stride
already fetches** — **zero extra requests** — and fill 106 of 107 imported
runs coherently (zone seconds sum to elapsed duration within 2 %). They
cannot: give the zone **boundaries** (the app does not know what heart rate
was zone 3 that day); agree by construction with the app's **moving-time**
basis (the source sums to **elapsed**); say which configuration produced
them; or exist for a run with no heart rate.

**DECIDED by the user: option A.** **Two blocks**, each naming its basis,
never added together: the app's own zone time (over the time the athlete was
moving) and the watch's own zone minutes (over the whole run) for the runs
with no model in force. The same discipline the ledger already uses for its
intensity and shape families. The **overlap** — the weeks where both exist —
is where the app checks its own arithmetic against the watch's, stated as a
fact, with no verdict on which is right. Under option B there would be no
overlap at all, and the app could never check itself.

**Storage (proposal only, for Pass 3):** the watch's zone minutes belong in
the `activity_observations` table (D10) — one row per activity per zone,
with the source, the day the response carried it and the day it was read.

## D24 — Precision belongs in the notes; the surface speaks the runner's language (binds: every surface)

The user, on the language of the two-basis design: *"The language again — the
watch's own zone minutes, elapsed time. We need to work on everything like
that if we are showing that in the surface."*

* The **notes** (the annotation layer) may use the precise terms — "elapsed
  time", "moving time", "the zone model's floors", a field name — because a
  note is written for someone reading *about* the product.
* The **surface** is written for a runner looking at their own training: no
  field names, no model names, no jargon, no internal vocabulary, and no
  phrase that only makes sense to whoever wrote the code.
* **The test:** *would a runner who has never read the code understand this
  line without it being explained?* If not, it is too technical, too
  abstract, or it is explanation that belongs in a note (D22).
* Numbers and their formats are not touched — the app's own conventions
  (`crates/stride-core/src/units.rs`) are the authority. **Words only.**

## D22 — Explanation is not product content (binds: every surface, every implementation)

The user's words, after walking the end-to-end:

> **"overall i think great visuals. I am seeing that we are adding text where it
> is not required."**

**The product surface carries: the picture, its labels, the numbers, the
period, and — only where something is missing — the reason something is
missing.** Nothing else.

Every line that explains **the design** rather than **the data** belongs to
the **annotation layer** — the notes, off by default. That includes: a
header explaining the drawing ("candidates, drawn two kinds, two lanes"); the
reasoning behind two axes ("these are numbers about different things, so
they are never converted into one another" — which belongs as a **caption
inside the chart's frame**); a candidate's limitation ("it cannot know — the
app cannot check what was put into the watch"); and any basis line that only
explains the drawing.

**The test:** if the line would still make sense on a card showing a
different measurement, it is explanation, not content — cut it. If it is the
**period**, or the **source** where the source is the fact, or the **reason
a value is absent**, it stays.

**Explanation is moved, never deleted** — every line that leaves a product
surface appears as a note.

## D21 — The canonical athlete (binds: the prototypes, the end-to-end)

`exploration/athlete.js` is the **single source** for the synthetic athlete in
the prototypes. It resolved seven inconsistencies between prototypes that had
each invented their own athlete:

* the zone model: **read 3 Nov 2026, max HR 183** (three prototypes had
  dated it three ways);
* the typed race: **10K in 41:36 on 12 Oct 2026**, giving **VDOT 49.7** (the
  prototypes had used three different races);
* the threshold band: the **hour-pace band** in both places (it had been
  defined two ways);
* "no Z4–Z5 since 15 Oct" is **two facts**: the band says **20 Oct**, the
  workout step says **15 Oct**;
* the ledger's moving time is the **sum of its bands** (49 h 32 m);
* two runs' average heart rates contradicted their own zone minutes —
  corrected in the data file and flagged there;
* the two "heatmap" names still collide (the wall's calendar of days vs the
  grid over the route positions) — **still to rename**.

## D20 — The threshold's last four questions (binds: threshold setup, plan)

1. **The ladder's rung and the plan's session pace are never drawn
   together.** They are two different numbers — the ladder's is the method's
   rule on the athlete's anchor; the plan's is what the athlete was told to
   run, which uses the athlete's **chosen** threshold when one exists. Put on
   one axis they would imply they are the same. The plan card names the
   threshold it was paced with (value and date); the ladder names the rule.
2. **A typed value carries two dates**: the day the **measurement** was taken
   (when the athlete knows it — optional) and the day it was **entered**
   (always). The sentence names the entered day; the detail shows both.
3. **The "other rules exist" line**, in the product's voice: *"One rule is
   used here: your pace and heart rate over the last 30 minutes. Other rules
   for this test are published; they are not computed here."*
4. **A base may be adopted with no threshold chosen.** Then the plan's pacing
   falls back to **the method's rule on the plan's anchor race**, the ladder
   shows the method's rung, and the threshold card says so and names the four
   ways a threshold could arrive.

**The threshold is read in two places only:** the **plan's pacing** and the
**80/20 bands**. The **ladder keeps the method's rule** on the race time; the
**ledger and the load keep the watch's own numbers**.

## D19 — The threshold: four candidates, one choice, dated (binds: threshold setup, pace engine, plan)

**The four candidates** (sources, not rules): the **watch's own zone model**
(its fifth zone's floor — no typing at all) · **from a race** (a **band** of
paces, the book's rule: "the pace you could race for about an hour", the
15K/10-mile route being the book's approximation) · **from a field test**
(the **book's 45-minute time trial**, computed from the stored FIT) · **typed**
(by the athlete, from a lab or a coach; **a pair is allowed** — a pace band
and/or a heart rate).

**The decisions:**

1. **One rule per candidate, and for the Pfitzinger base it is the book's.**
   Friel's 30-minute, Anderson's and 80/20's 20-minute rules stay **named as
   published alternatives**; the app does not compute four estimates and
   choose among them.
2. **The book's fifth method (HR at 82–90 % maxHR) is not a fifth card** — the
   four candidates are four sources.
3. **The chosen threshold does NOT feed the load model.** The load keeps the
   watch's zone model as its reference, and the load history does not change
   because the athlete chose a different number. The load model's inputs are
   examined in **stage 2** of the zone-model migration, separately.
4. **One threshold at a time — dated, and tied to the base it was made for.**
   A new base may come with a new choice; the old one stays in the dated
   history. **The plan's pacing names the threshold it was paced with** (value
   and date), because a plan adopted in March and still running in July is
   paced by March's number — and the card says so.
5. **RCRS and the Norwegian family are a future base**, with the 80/20 family,
   and they bring their own rule the way Pfitzinger brings the 45-minute
   trial. The research already holds the family (`exploration/methods-and-products.md`
   §2.6), with the "Fthreshold 2:1 / 3:2" ratios and Ørnskæg's cycle marked
   **unverified** — and the "Norwegian 4×4" that circulates publicly is
   **NTNU research**, not Ørnskæg's method.
6. **The watch's zone model and the athlete's threshold are two different
   numbers** and never merge. The ledger's rows keep answering against the
   model in force on each run's day.

## D18 — The user's decisions, 24 November 2026 (all four)

1. **The consolidated chart for the runs — approved** ("thumbsup"). With it,
   the **surface cut** (Pass 2): the wall 24 → 11 cards, the `/pace` page
   retired, the radar retired, the ledger card taking the zone card's place.
2. **W — the window migration: GO.** One window (the last *N* calendar weeks
   through today, compared with the same span ending seven days earlier),
   the rolling route migrated, the profile's volume-a-week and consistency
   changed, the year's comparison moved to a week.
3. **Z — the zone model, stage 1: GO, with one change.** Every run is
   measured with the model in force on its day for the zone-time answers —
   **but the model and its read date are hoisted to ONE source line per
   screen**, not repeated per answer; the details sit on a click through. An
   answer that used a **different** model (a run predating the first stored
   model) still says so, because a silent difference would be a wrong number.
   This is D11 (picture first) applied to provenance. The load model does not
   move — that is stage 2.
4. **The plan tier: option A.** The plan card states the tier's **entry
   requirement** beside the athlete's own weekly mileage, and **the athlete
   picks**. The app never suggests a tier.

**Implementation order:** Pass 1 (W, Z, C1, C2 — the correctness changes) →
Pass 2 (the cut) → Pass 3 (the activity observations). The ledger ships
after Z's stage 1, so the ledger and the run detail cannot disagree.

## D17 — The pace chain, settled; and the eight source contradictions reconciled

**The chain (settled):**

    race time (typed, dated) → VDOT (the engine's own) → the race-pace
    equivalents (VDOT/Riegel) → LT pace (the spine's rule) → the rungs
    (the spine's percentages)

* **VDOT is the engine; the spine is the rulebook.** Our chain **replaces**
  the book's lookup tables (Appendix B, Appendix C) — we compute the pace for
  a 10K, a 15K and a half from the athlete's race time. The book's *rules*
  are quoted and cited.
* **The anchors** are the goal race pace (10K or HM) and **LT pace**;
  marathon pace is at most one more rung, never the anchor. The MP-offset
  placeholders are **deleted** from every file that carried them.
* **Every rung carries a heart-rate band (the method's) and a pace (the
  athlete's own pace-at-HR, or the method's percentage rule)**, kept apart;
  where there is no history at that heart rate the pace is a **no value with
  a reason** and the band stands.
* **The vocabulary is the spine's** run types — endurance, general aerobic,
  recovery, progression long run, lactate threshold, VO₂max, speed, tune-up
  race. The ladder shows the paces subset; the rest are the plan's session
  kinds.

**The other seven resolutions:**

* **History and the plan are two sources, never merged** — the ledger's rows
  answer against history; the plan card carries intent; the app never says
  whether the athlete did what they were told.
* **The surface cut supersedes `task-areas-plan.md` rule 1** ("no card is
  lost"), on the user's instruction to reduce the surface.
* **The Beta shows the ledger's answer, not two** — until stage 1 of the zone
  model lands.
* **The method lens: design done, shipping deferred** (D13) — and its rules
  still govern the rest of the product.
* **The Beta must not claim a verified weekly skeleton** for any tier the
  spine has not read cell by cell.
* **README drift** — "six string types" should be **seven**, and it still
  says four destinations.
* **The Beta** is served by the existing rig at `/beta`, reads **read-only
  through the existing API**, and is therefore reachable from a phone browser
  over the mesh — tryable without shipping in the Android app. It needs two
  routes that do not exist yet: `/insights/stimulus` (a cleanup step) and
  `/insights/paces` (planned nowhere).

## D16 — The anchor is the race pace, and the easy day is a heart-rate band (binds: pace engine)

From the spine (`exploration/pfitzinger-spine.md`, *Faster Road Racing*, 2nd
ed., 2015):

* **Pfitzinger's rules are percentages of a *named race pace*** — not fixed
  s/km constants, and **not anchored on marathon pace** for either shipping
  programme. The anchors are the athlete's **race pace at the goal distance**
  (10K or HM) and **LT pace** ("the pace you could race for about an hour",
  ≈ a 15K–10-mile race, or 10K pace + 10–15 s/mi for a shorter-distance
  athlete).
* **The easy day has no pace at all** — it is a heart-rate rule
  (70–81 % maxHR / 62–75 % HRR). The pace comes from the athlete's **own
  pace at heart rate**, or the athlete types it.
* So a rung is **a heart-rate band (the method's) plus a pace (the
  athlete's, or the method's percentage rule)** — and the card keeps them
  apart. Where the athlete has no history at that heart rate, the pace is a
  **no value with a reason**; the band still stands.
* **The athlete's inputs:** goal race distance + date · one race time (10K or
  HM) · max HR · resting HR · plan tier + start date. Max HR and resting HR
  already exist in Stride (the zone model; the wellness readings) and are
  read, not typed.
* **The two bases differ structurally:** Pfitzinger = **one** hard session a
  week, easy days banded by **maxHR**; 80/20 = **two** hard sessions a week,
  easy days banded against **threshold**, the gray zone avoided. One base at
  a time, named on the plan card. `PlanSource` gains a method value and the
  session kinds gain the book's thirteen run types.
* **Ships:** the 10K template (the book's 8K–10K tiers 1–2) and the
  half-marathon template (HM tiers 1–2). **Extracted but not shipping:** the
  5K and 15K/10-mile plans, the multi-distance plan, the higher tiers, the
  three base-training schedules, and the marathon book's MP-relative rules.

## D15 — The plans: **10K and half marathon only** (binds: spine, plan templates)

The user's decision, 24 November 2026: *"just have 10k and HM plans, cut even
5k for now."* So the product ships **two plan programmes** — the 10K and the
half marathon — and the 5K and 8K programmes are **out of scope** for the
first drop. The method's *rules* (run types, pace rules, recovery, the hard/
easy principle) still come from the whole book; only the **plan templates**
are cut. "Intermediate runners" is confirmed as the audience.

## D14 — Who the app is for, and the two plan bases (binds: spine, pace engine, PLAN amendment)

**The user's decisions, 24 November 2026:**

* **The app is for the intermediate runner.** *"Mark this whole app for
  intermediate runners. For entry level, Strava/Garmin is enough."* The
  audience is an athlete who is already training, has a base, and is chasing
  a specific distance — not a beginner. The plan structures, the vocabulary
  and the analytics all assume that.
* **Pfitzinger is the base model** — the app's first method.
* **80/20 is a separate second method, not a lens and not a variant.** *"80/20
  and Pfitzinger are two separate methods, not the same. So those will be
  two different defaults."*

So the first drop has **two plan bases, one at a time**: the athlete adopts
one; the ledger and the pace engine speak against the base they are
following; the app never recommends either.

**The primary source is now in hand:** Pete Pfitzinger, *Faster Road Racing:
5K to Half Marathon* (the user's own copy, in thread storage). It is the
source for the sub-marathon levels — the distances this app serves — and it
supplies the method's **session vocabulary** (general aerobic, tempo,
intervals, long run, recovery, strides), which is what the plan side of the
ledger speaks in.

**Copyright discipline:** the repository is public. The book stays in thread
storage; only rules, numbers and short attributed quotations go into the
repo, each with a chapter citation.

**What the book gives the ledger:** a mapping from the method's run types to
our stimulus kinds, and — the point of it — **which of them the watch can
verify and which exist only as intent**. A row that is evidence is
distinguishable from a row that is the plan's words.

## D13 — The first drop is Pfitzinger; the lens is out (binds: everything downstream)

**The user's decision, 24 November 2026:** *"no lens or Pfitzinger's plan to
start with. Maybe 80/20 also. Nothing else in the first drop."* And: *"I
personally want to move toward Pfitzinger's plan overall."*

* **No lens ships in the first drop** — not even opt-in. The method-lens
  prototype (`exploration/method-lens.md`, `exploration/method-lens.html`)
  stays in `exploration/` as the record of what was designed and why it was
  deferred.
* **Pfitzinger's plan is the spine.** It is a **plan**, not a lens: dated
  sessions the athlete adopts, with the plan's own pacing.
* **80/20 comes as a second plan**, not a lens. The ledger's polarised bar
  already shows the athlete's own split, so 80/20 needs no quoted rule; what
  it needs is a weekly structure. The app holds **one plan at a time** — the
  athlete's own, never the app's recommendation.
* **The dependency this exposes:** a Pfitzinger plan is written in
  **marathon pace**, and Stride has none — no VDOT, no race time, no race
  prediction. So the first drop needs the **pace engine**:
  race time → VDOT → MP → the training paces (easy, threshold, long run,
  recovery), each naming its source.
* **The plan anchor returns.** The ledger's rows answer against the athlete's
  own history and now also against the plan's intent — the negative space the
  wedge is about. The plan's session kinds are the vocabulary; the app never
  judges whether the athlete did what they were told.
* A **race time is an observation**, stored with provenance (value, day
  entered, day last changed). Stride never invents one, never guesses one
  from training, and never derives a maximum heart rate from an age.

**Threads:** `thr_nxc9e9nxms` (pin Pfitzinger's numbers and the 80/20 plan
shape) · `thr_crvvnnhwtz` (the pace engine — design note and prototype).

## D8 — The three standing calls: **decided by the user, 24 November 2026**

1. **Five destinations.** The code ships five (Profile); `PLAN.md` and
   `README.md` say four. **Decision: five** — the profile keeps a door of
   its own, and the drift is fixed with a `PLAN.md` amendment and a
   `README.md` correction. The surface cut sizes both readings to the same
   panel count, so this is a placement decision, not a size decision.
2. **Zone bands in the chart as well as the table.** The bands are the
   athlete's own configuration drawn on their own data; the table keeps the
   numbers, the *not moving* bucket and the model's source line.
3. **Altitude on its own scale, default off.** Its own metres label, never
   sharing the effort band's scale, and the default is pace + heart rate.

**Consequence for the cleanup:** the five-destination reading means the
profile's five cards stay on the profile, and `PLAN.md` + `README.md` are
corrected as part of the documentation pass. The consolidation thread's
cut (24 wall cards → 11) is unchanged by this decision.

---

## D25 — The rig is recoverable (binds: every deployment)

The user, 24 November 2026: *"as we deploy this, after completion, the current
stride desktop rig — keep it as a separate branch we can recover easily."*

**Two anchors, both on the remote:**

| what | where it lives | how to recover |
|---|---|---|
| the **released rig** (v1.0.0, before any of this work) | the tag **`v1.0.0`** at `66432bb` | `git checkout v1.0.0` |
| the **rig as it stands today** (Pass 1 + Pass 2) | the branch **`rig/window-zone-model-and-the-cut`** at `75d1627` | `git checkout rig/window-zone-model-and-the-cut` |

Both are pushed to `origin` (`github.com/caprion/stride`), so they survive a
lost disk.

**The rule for every deployment from here:** before a release, cut a branch
at the state being released and push it — the tag says *what* was shipped, the
branch says *what it looked like the moment before the next thing landed*.
Recovery is then a checkout, not an archaeology.

---

## D26 — The day's four parts are the athlete's own hours, and the label says which (binds: the rolling view, the schedule card)

The athlete, 24 November 2026, on their own history: *"another sort of a bug is
that night time is taken as 23:00 to 6:00. A lot of my runs would have started
at around 5ish. So everything gets reported to night time. We should move
night to 21:00 to 4:00 am. And label it that way."*

* **The cut points.** Morning **04:00**, afternoon 12:00, evening 17:00, night
  **21:00**. The night **wraps around midnight**, so it runs from 21:00 to
  04:00 — seven hours — and the morning from 04:00 to 12:00 — eight. The
  partition follows from the four constants; the hour a run **started in** is
  still the only thing that places it.
* **The label carries the window.** "Morning (04:00–12:00)", "Afternoon
  (12:00–17:00)", "Evening (17:00–21:00)" and "Night (21:00–04:00)". The
  hours are **rendered from the cut points** and are never typed into the
  wording, so the name of a part and the part itself cannot drift apart.
* **The calendar day does not move.** A run at 05:10 belongs to that calendar
  day — for the load, the ledger and the plan — and it is a *morning* run in
  the day's four parts. **Only the four parts changed**: sleep and wellness
  are read by the calendar day, and they are untouched.

---

## D27 — The athlete's declared heart rates are the bands (binds: the zones, the ledger, onboarding)

The athlete, 24 November 2026, on the fact that Garmin does not expose the
zone model's history: *"we should ask the runner to explicitly add the HR
setup.. either RHR or max HR or both .. we should setup that as part of
onboarding for the first time, thereafter the app should take care of it."*

* **The declared pair is a dated fact the app holds for the whole history.**
  Resting heart rate, maximum heart rate, **either or both**, each with its
  **source from a closed list** (my watch · a test · a doctor or a lab · my
  partner's watch · typed), the day it was measured, and the day it was
  entered. A later declaration does not replace the earlier one; both stay.
* **The bands are derived from it by a cited rule** — Garmin's published
  five-zone ladder: heart-rate reserve (`resting + p × (max − resting)`)
  when both numbers are given, percentages of the maximum when only the
  maximum is, and **no bands at all** when only a resting rate is given,
  with the core's own reason.
* **Two dated facts, never blended.** The watch's configured floors and the
  athlete's declared pair are both stated, the difference between them is
  stated as a fact, and **no answer mixes bands from one with bands from the
  other**.
* **The declared pair is the basis** (option 1 of the design's trade-off):
  it is applied to **every** run, so the ledger's intensity rows are one
  comparable series over the whole window rather than empty over the
  history. The cost is stated and accepted: the app's boundaries are
  **derived**, not the watch's configuration, and the card says so — and the
  watch's own zone minutes remain their own block.
* **This changes Z's rule** (D3, D18.3): a run is now measured against the
  athlete's declared bands where a declaration exists, and against the
  watch's model where one was in force and no declaration exists — and the
  answer names which.
* **Onboarding**: the rig's first-run page gains a **fourth** choice beside
  the race, the threshold and the programme — *"four things only you can give
  it"* — side by side, unranked, and **every one can stay empty**. The
  heart rates are typed in the app, in **Settings › Training › Your heart
  rates**, so there is one place and not two.
* **Thereafter the app looks after it**: the bands are derived on read and
  used wherever a band is needed; they are re-derived when the pair changes,
  and the earlier answers stay as they were; the watch's zones are compared
  with the declared ones and the difference is stated; and **the athlete is
  never asked again** — nothing nags.

**The implementation's own line** (25 November 2026). The decision's shape
held; two things about it were settled by the build:

* **The storage is one row per kind, not one row per declaration.** The
  table is `heart_rate_declarations`, and a row carries the **kind**
  (`max` or `resting`), the **value**, the **source**, the day it was
  measured, the day the declaration was made, the day the row was last
  changed, and the **declaration it belongs to**. A pair entered together
  is one declaration with **two rows**, and the rows name the declaration
  they belong to, so the two can never be read as two declarations. **One
  declaration is in force at a time** — the most recent — and the earlier
  ones stay in the dated history. The schema version moves from 7 to 8.
* **Every answer names the basis, and the seam is in one place.** The
  api reads the stored models and the declaration in one place
  (`crates/stride-api/src/zonemodel.rs`), so the rule is in one place: the
  declared bands where a declaration exists, and the watch's own model
  where one was in force and no declaration exists. The zone answers carry
  the basis (`bands_basis`) beside the two facts, and **the two are never
  mixed inside one answer**.

---

## D28 — The week against the plan: one card, one question, and the plan's own words (binds: Progress, Today, the Plan page)

The review of the live rig (24 November 2026): *"Home (/) and /progress mix
Banister load ratios, streaks, sleep stages, and weekday comparisons (-1.9 km
against 16 Nov – 18 Nov) without answering whether the week's training fits a
goal."*

* **The answer is a fact, not a verdict.** Each planned session is *done* or
  *not done* — the plan engine's own statuses — and for the sessions that
  happened, the distance run is read against the distance planned and the
  pace against the plan's pace. **No grade, no percentage, no traffic
  light, and the word "missed" is not in the app.**
* **The paces are compared only when both of them exist.** The general
  aerobic day carries a heart-rate band and no pace, so that session is
  compared on its distance or on nothing; a run is never compared against a
  pace that is not there.
* **The comparison is like for like.** The days that have gone are compared
  with the days that have gone: the plan's first three days against the
  athlete's first three days, never the plan's whole week against a part of
  one. A session of the day that is still running is not counted against
  the athlete either — the day is not over.
* **One plan.** The athlete's own plan is compared with the week that
  happened. The app never compares two plans, never says which plan is
  better, and never offers a programme the athlete did not adopt.
* **The card is merged into *This week*, not added beside it.** The wall is
  eleven cards and the instruction was to reduce the surface: the question
  "how is this week" and the question "how is this week against the plan"
  are the same question asked of two other things, so the answer lives
  inside the card that already answers it. The panel behind the card, the
  Plan page and Today carry the same answer in full.
* **The four states are the answer's own**: no plan adopted (the app says
  so and points at the programmes there are to adopt), a plan whose first
  day is after this week (the app says when it starts), a week inside the
  plan (the comparison so far), and a plan whose last day is before this
  week (the app says when its last week ended).
