# The end-to-end — one athlete, every screen, annotated

*24 November 2026. A prototype thread: new files under `exploration/` only. Nothing under
`crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/` was changed, and nothing
was committed.*

*The surface copy in this note and in its prototype follows the language pass — `exploration/language.md`.*

**The three files**

| file | what it is |
|---|---|
| `exploration/athlete.js` | **the one source of the athlete's data.** The window, 58 runs, the weeks, the zone model, the races, the wellness, the plan, the threshold choice — and the product's own strings. |
| `exploration/end-to-end.html` | **the walk-through.** Twelve screens plus the states, each at phone and desktop width, with the annotations on and off. |
| `exploration/end-to-end.md` | **this file.** What each screen answers, the annotation texts with their KEEP/DROP marks, the state catalogue, and what is not here. |

Open `end-to-end.html` from `file://`. No network, no framework, no build step.

---

## 0 · How to read the page

* **One athlete.** Every number on every screen is read from `exploration/athlete.js` — the run
  table, the zone model, the ledger's aggregates, the load, the race, the ladder's rungs, the
  curve, the threshold's four candidates and its history, the plan's week, the wellness, the
  featured run's splits and series, the compare pair, and the year's months. The markup holds no
  numbers of its own: where a screen needs a figure, the string comes from `athlete.js`'s
  `copy` block.
* **Two widths.** Each screen is rendered twice — a 308 px phone frame and a 660 px desktop
  panel — from one definition, so the two cannot drift apart. The desktop is the same content
  laid out wide; it is not a second design.
* **The annotations.** Every element carries a numbered pin. Clicking a pin opens its note; the
  rail on the right lists the notes of the screen you are in. The **annotations** switch in the
  header turns the pins and the rail off, and the page reads as the product.
* **The product surface carries no explanation.** A number, the label that carries it, the
  period, the source where the source is the fact, and the reason a value is missing — nothing
  else. Every line that explained the *drawing* has moved to the notes (§9), or, where it
  belongs with the picture, to a small caption inside the picture's frame.
* **The navigation.** A contents rail on the desktop, a jump menu on a narrow window.

---

## 1 · The athlete

One person, used on every screen. The window is **7 Sep – 24 Nov 2026**: **58 runs**,
**49 h 32 m** of moving time, 564.8 km.

| what | the value | where it came from |
|---|---|---|
| the zone model | `HR_MAX`, max heart rate **183 bpm**, read **3 Nov 2026** | `exploration/stimulus-ledger.html` (the source line), scaled to the floors of `exploration/threshold-setup.html` |
| the zone floors | 115 · 128 · 143 · 159 · 165 bpm | the same prototype's shape, scaled to the 183 bpm maximum |
| the typed race | **10K in 41:36 on 12 Oct 2026**, entered 13 Oct 2026, changed 3 Nov 2026 | `exploration/threshold-setup.html` |
| the goal | a **half marathon** | the brief, and `exploration/pace-engine.html` |
| the derived paces | 5 km 19:57 · 15 km 1:03:56 · half 1:31:47 · marathon 3:11:22 | Riegel, exponent 1.06, computed in `athlete.js` |
| VDOT | **49.7** | the pace-engine chain, re-anchored on the 10K (see §6) |
| the threshold | **165 bpm**, the watch's fifth zone floor, chosen **3 Nov 2026**, made for the Pfitzinger 10K plan | `exploration/threshold-setup.html` |
| the plan | the **Pfitzinger 10K plan**, adopted 30 Sep 2026, built from the 10K | `exploration/pfitzinger-spine.md` |
| the load | acute 118 · chronic 84 · form +34 · ratio 1.40 · Banister k = 1.92 | `exploration/stimulus-ledger.html` |
| the ledger | Z1 42.4 % · Z2 41.7 % · Z3 13.9 % · Z4 2.0 % · Z5 0.3 %; Z1–Z2 41 h 32 m · Z3 6 h 52 m · Z4–Z5 1 h 08 m (4 runs, the last on **20 Oct 2026**) | `exploration/stimulus-ledger.html` |
| the longest run | 20.4 km in 1 h 59 m, 21 Nov 2026 | the same |
| the repeated work | 2 h 15 m across 3 runs, the last on **15 Oct 2026** | the same |
| the run table | 58 runs, 12 weeks, per-run zone minutes, workout steps, average heart rate and hour | `exploration/cards-as-pictures.html` |
| last night | Sleep 6 h 10 m · resting heart rate 54 bpm · HRV 48 ms | the wellness block in `athlete.js` |

**Two corrections were made to the prototype's run table** so that the table and the ledger
agree, and both are marked in `athlete.js`:

1. Two runs carried an average heart rate that would put them in Z4–Z5 while their own zone
   minutes said otherwise (the run of 23 Nov, at 168 bpm, and the run of 16 Nov, at 171 bpm).
   Their averages were moved into the band their minutes place them in, so that *"no run in
   Z4–Z5 since 20 Oct 2026"* is true of the table as well as of the ledger.
2. The 26 Oct week's long run carried 9 minutes of Z3 in the table where the prototype's own
   ledger has 10; it was corrected, which is what makes the window's Z3 time 6 h 52 m.

---

## 2 · The screens, in the order a runner meets them

| # | screen | what it answers | the notes |
|---|---|---|---|
| 1 | **Today** | *What happened, where you stand, what the plan has on the table, and how last night went* — in that order, with nothing linked to anything else. | 1–11 |
| 2 | **Runs** | *Which runs exist.* One question: the calendar, the filters, the compare tray, the import, the list. | 12–16 |
| 3 | **Run detail** | *Everything about one run* — and only about that run. The sleep before it is a link, not a card. | 17–29 |
| 4 | **Progress** | *What the window says.* Eleven cards, each read as a picture first. | 30–33 |
| 5 | **The pace card** | *What pace to hold for what, and what the racing costs.* The ladder and the curve. | 34–41 |
| 6 | **The threshold** | *What the app is using as the threshold, where that number came from, and what it is not used for.* | 42–51 |
| 7 | **The plan** | *What the athlete's own plan says for this week* — sessions, kinds, paces, and where the paces came from. | 52–57 |
| 8 | **The athlete profile** | *The shape of the athlete.* Five cards, each a picture first. | 58–62 |
| 9 | **Body** | *How the body has been* — the night, the three signals, and what usually happens. | 63–66 |
| 10 | **Compare** | *How two runs differ*, on one basis, with the numbers side by side. | 67–71 |
| 11 | **The year** | *What the year held* — the months, the calendar, and what changed. | 72–74 |
| 12 | **The film and the heatmap** | *The deep view of one run, and where the athlete runs.* Two contextual surfaces. | 75–76 |
| + | **The states** | *The same screens with their absences.* | 77–82 |

**What was left out of the walk-through, and why**

* **The run detail's three charts became one.** The design in
  `exploration/consolidated-chart.html` is the authority; the run detail shows it with the
  signal switch and one cursor.
* **The film's splits table is not drawn** — the inventory retires it in favour of the run
  detail's table, and the film's lap column becomes a link there.
* **The plan's day detail, the adherence counts and the paused note** are not drawn; the plan
  screen here carries the week's sessions and their paces, which is what the brief asked for.
* **Settings, the three-question setup, the Garmin connection, the import history, the export
  and delete** are out of scope for a walk-through of the product's readings.
* **The wall's retired cards** (the radar, the polarised split, the activity heatmap calendar,
  yearly stats, most recent activities, body last night) are gone, exactly as
  `exploration/surface-inventory.md` cut them.
* **Nothing is interactive** except the annotation toggle and the navigation. The threshold's
  "use this", the filters, the chart's signal switch and the compare picker are drawn as
  states, not wired.

---

## 3 · The annotation texts

Each note says **what the element is**, **where it comes from** (the core function and the
file), and **what it may not claim**. The mark is my judgement of whether the note is worth
keeping in the real rig: **KEEP** — the athlete could be misled without it; **DROP** — it
explains the build, not the product.

### 1 · Today

**1 · The source line**

> your zones · running · max heart rate 183 bpm · taken from your watch on 3 Nov 2026

*What it is.* The screen's one source line. It names the model every number on the screen was measured against, and the day the model was read.

*Where it comes from.* The dated zone model the core stores — `crates/stride-core/src/zones.rs` — read from the watch's own configuration and kept with the day it was read.

*What it may not claim.* It may not claim the model is current, and it may not be repeated on any row below. One screen, one line.

**DROP** — the line speaks for itself; the note is provenance

**2 · The hero — the last run**

> 24 Nov 2026 · 6.0 km · 30 m

*What it is.* The last run the watch recorded. Today opens with the run that happened, not with a plan or a vital.

*Where it comes from.* The most recent activity in the store — `crates/stride-core/src/rolling.rs` and the run list on `/runs`.

*What it may not claim.* It may not be the biggest, the best or the one the plan asked for, and it may not be described.

**DROP** — the hero is the last run — there is nothing to explain

**3 · The hero's figures**

> 6.0 km · 30 m · 5:00 /km · 134 bpm

*What it is.* The run's own facts: distance, moving time, pace and average heart rate, in the order a runner reads them.

*Where it comes from.* The run record and its heart-rate summary — `crates/stride-core/src/types.rs` and the pace helpers in `units.rs`.

*What it may not claim.* Pace is moving time over distance. It is not a grade, and the heart rate is the average, never a judgement.

**KEEP** — pace is moving time over distance, never elapsed time

**4 · Where you stand · this week**

> 11.0 km · −5.4 km against the same days a week earlier

*What it is.* The same two days of this week and of the week before, compared. It is a difference, not a verdict.

*Where it comes from.* The weekly aggregate and the period comparison — `crates/stride-core/src/rolling.rs`; the picture it sits on is the <i>This week</i> card in `exploration/cards-as-pictures.html`.

*What it may not claim.* It may not say the week is going badly, and it may not link the difference to sleep or to any other vital.

**DROP** — the arithmetic is on the screen

**5 · Where you stand · form**

> last 7 days 118 · last 6 weeks 84 · the difference +34 · 1.40 times the last 6 weeks

*What it is.* The load engine's own numbers: TRIMP over the athlete's zone model, with Banister k = 1.92. The scenario under it is the model's expectation, not a prediction.

*Where it comes from.* `crates/stride-core/src/load.rs`; the card is the <i>The load</i> card in `exploration/cards-as-pictures.html`.

*What it may not claim.* It may not say the athlete is fresh, tired or ready — the direction words are the only judgement allowed, and this card does not use them.

**KEEP** — the scenario is the model's expectation, and the card may not judge

**6 · Where you stand · habit**

> 4 days in a row · 12 km on 13 days

*What it is.* Two answers to one question — how steady the running has been. The streak is days in a row; the Eddington is the distance that has been reached on at least that many days.

*Where it comes from.* `crates/stride-core/src/streaks.rs` and `eddington.rs`; the picture is the <i>The habit</i> card.

*What it may not claim.* It may not be a score, and the two numbers may not be added together or compared with anything else.

**DROP** — two numbers, no reading needed

**7 · What's on the table**

> Endurance · 8.0 km · 5:12–5:38 /km  ·  Recovery · 6.0 km · 5:36 /km

*What it is.* The plan's own sessions for today, listed as they are — two options, side by side, in no order.

*Where it comes from.* The adopted plan's session list — `crates/stride-core/src/plan.rs` — and the `option` register in `docs/contract/voice.md`.

*What it may not claim.* It may not rank the two, may not mark one as the one to take, and may not mention a vital. A plan option is never conditioned on how the athlete slept. Both options come from the plan itself, and neither is tied to a vital.

**KEEP** — options are unranked and are never tied to a vital

**8 · Rest — the summary line**

> could be better · 2 of 3 signals below your usual

*What it is.* A count and a direction, computed from the three vitals beneath it: how many of them sit below their own usual band.

*Where it comes from.* The direction words and the band rule — `crates/stride-core/src/direction.rs` and the median ± 1.5 × MAD band in `docs/contract/core-contract.md` §4.

*What it may not claim.* It may not become a conclusion. <i>Rest</i> is the name of the row here; it is never an instruction, and the summary may not say the athlete should do anything.

**KEEP** — “Rest” is the row's name, not an instruction, and the line is a count

**9 · The three vitals**

> Sleep — 6 h 10 m · could be better   Resting heart rate — 54 bpm · about usual   HRV — 48 ms · could be better

*What it is.* Each vital on its own: the value, a direction from the closed set, and the arithmetic that produced the direction.

*Where it comes from.* The wellness record and the direction rule — `crates/stride-core/src/rest.rs`, `direction.rs`, `pattern.rs`.

*What it may not claim.* The three may never be combined into a verdict. There is no readiness score, and no line may say the athlete is or is not recovered.

**KEEP** — the three vitals are never combined into a verdict; there is no score

**10 · The sleep chart**

> 6 h 10 m — deep 78 m · REM 94 m · light 174 m · awake 24 m

*What it is.* One night, drawn as the stages the watch recorded, over the hours they happened.

*Where it comes from.* The wellness sleep record — the same one Body shows for any date; `crates/stride-core/src/body.rs` is not a module, so this is the wellness surface's own read.

*What it may not claim.* It may not be scored, and the stages may not be turned into a rating of the night.

**DROP** — a chart is a chart

**11 · The link to Body**

> see all of it in Body →

*What it is.* The way in to the full Body screen, where sleep, the signals and the patterns live. Body holds sleep for any date, the three signals, and the patterns.

*Where it comes from.* The destination list in `frontend/src/app/destinations.ts`; Body is the screen in `exploration/surface-inventory.md` §1.

*What it may not claim.* Today holds the reading; Body holds the history. Neither copy may become a second copy of the other.

**DROP** — the link is a link


### 2 · Runs

**12 · The browse calendar**

> November 2026 — one mark per day that holds a run

*What it is.* The month as a grid, with a mark on every day the watch recorded a run. The calendar is the screen's own control.

*Where it comes from.* The day index over the stored activities — the calendar panel on `/runs`, from the same store the run list reads.

*What it may not claim.* A day without a mark is a day without a recorded run — never a missed day, never a rest day.

**KEEP** — a day without a mark is not a missed day

**13 · The filters**

> search · from 7 Sep 2026 · through 24 Nov 2026 · showing: runs

*What it is.* The four ways the list can be narrowed. They change what the list holds; they do not change what a run means.

*Where it comes from.* The runs query in `crates/stride-api/src/routes/` and the filter controls on `RunsPage.tsx`.

*What it may not claim.* A filter may not rank, colour or hide a run to make a list look better, and an empty result is never an error.

**DROP** — the filters are visible

**14 · The compare tray**

> 21 Nov 2026 · 20.4 km  +  31 Oct 2026 · 19.6 km

*What it is.* Two runs held side by side, waiting to be compared. It is the way into Compare that does not need a run open first.

*Where it comes from.* The selection state on `RunsPage.tsx` and the `/compare/:id` route.

*What it may not claim.* It may not suggest the pair is interesting, and it may not carry a verdict between them — the comparison's own basis sentence does that.

**DROP** — the tray is visible

**15 · Import a FIT file**

> a file the watch or another app wrote

*What it is.* The way a run that the watch did not sync — or that came from somewhere else — enters the store. A file the watch, or another app, wrote.

*Where it comes from.* The ingest path — `crates/stride-ingest/src/` and the FIT parser it uses.

*What it may not claim.* An imported run is a run: it is not marked as second class, and the import may not be used to edit a run that already exists.

**DROP** — the import is visible

**16 · The run list**

> This week · Last week · 16 – 22 Nov — each run with its distance, time, pace and average heart rate

*What it is.* The destination: every run in the window, grouped by week, newest first.

*Where it comes from.* The activity list over the store — the same rows the Progress wall's retired <i>Most recent activities</i> card used to hold.

*What it may not claim.* The list may not be a leaderboard and may not be sorted by anything but time. There is no place in this list for a judgement about a run.

**KEEP** — the list is not a leaderboard, and no row may carry a judgement about a run


### 3 · Run detail

**17 · The headline numbers**

> 20.4 km · 1 h 59 m · 5:50 /km · 145 bpm

*What it is.* The run's own facts at the top: distance, moving time, pace and average heart rate.

*Where it comes from.* The run record — `crates/stride-core/src/types.rs`; moving time and pace come from `units.rs`.

*What it may not claim.* Moving time is not elapsed time, and it may not be described. The heart rate is the average of the samples the watch took while moving.

**KEEP** — moving time is not elapsed time

**18 · The core's sentence**

> 20.4 km in 1 h 59 m — the longest run of the last 12 weeks

*What it is.* One sentence the core writes about the run, from facts it has already computed. The app owns this sentence; no model writes it.

*Where it comes from.* The story builder — `crates/stride-core/src/story.rs`, using the `statement` register from `docs/contract/voice.md`.

*What it may not claim.* It may not compare the run to the plan, may not call it hard or easy, and may not mention a vital.

**KEEP** — the core owns this sentence; it never compares the run to the plan

**19 · The one chart**

> pace · heart rate · altitude — one cursor at 14:20

*What it is.* The run's shape, with the three signals on one panel and one cursor across them. The run detail used to carry three stacked charts; it carries one now. One panel, one cursor, three signals: the signals are the same run read three ways, and they are not three pictures.

*Where it comes from.* The consolidated chart — the design in `exploration/consolidated-chart.html`, whose signal switch and shared cursor are the same control here.

*What it may not claim.* A chart carries a shape. It may not be read as a verdict, and the three signals may not be given three different scales that invite a comparison.

**KEEP** — one chart and one cursor: no scale may invite a comparison

**20 · The signal switch**

> pace · heart rate · altitude

*What it is.* Which signal the chart is drawing. The cursor stays where it is when the signal changes.

*Where it comes from.* The same switch as the Compare chart — one chart, two places, one control.

*What it may not claim.* It may not reorder the signals so that the flattering one comes first, and it may not carry a default that hides a signal.

**DROP** — the switch is visible

**21 · The cursor**

> 14:20 · 10.2 km · 5:48 /km · 148 bpm · 27 m

*What it is.* One point on the run, read across all three signals at once.

*Where it comes from.* The sample index — the same data the splits and the film read.

*What it may not claim.* It may not be an average of averages, and it may not carry a comment of its own.

**DROP** — the cursor's numbers are visible

**22 · The effort table**

> Z1 20 m · Z2 65 m · Z3 34 m · Z4 0 m · Z5 0 m · outside 0 m · not moving 3 m

*What it is.* The run's time in each of the athlete's five zones, measured against the zone model in force on the run's date, with the time the athlete was not moving kept apart.

*Where it comes from.* `crates/stride-core/src/zones.rs` — the same function that produces the ledger's band times, so the run and the window cannot disagree.

*What it may not claim.* A band with no time is a measured zero, and it is shown as one. A run with no model is <i>no value</i>, never zero. <i>Not moving</i> is time no heart rate describes. The model in force on the run's date is the one the run is measured against, so the run and the window cannot disagree.

**KEEP** — zones are measured against the model in force; a measured zero is not a no value; “not moving” is time no heart rate describes

**23 · The route**

> a 20.4 km loop · 86 m of climbing

*What it is.* The path the watch recorded, drawn from the stored positions.

*Where it comes from.* The activity's position stream — the same positions the heatmap grids and the film colours by pace.

*What it may not claim.* It may not be smoothed into a shape the athlete did not run, and it may not be coloured by anything that judges.

**DROP** — the route is visible

**24 · The splits**

> lap 1 · 4.1 km · 23:44 · 141 bpm  …  lap 5 · 4.0 km · 23:08 · 143 bpm

*What it is.* The watch's own laps: distance, time, pace and average heart rate for each.

*Where it comes from.* The lap records the watch wrote — `crates/stride-core/src/types.rs`.

*What it may not claim.* A lap is a lap, not a kilometre marker, and the laps are numbered from one. The count is the watch's, not the app's.

**KEEP** — a lap is a lap, not a kilometre marker

**25 · The form observations**

> cadence 178 spm · pace drift −0:06 /km · heart-rate drift +3 bpm · decoupling 2.1 %

*What it is.* The run's mechanics, as measurements: cadence, variability, the two drifts, decoupling, stride length and efficiency factor. Every value is the mean over the run, measured on the moving samples.

*Where it comes from.* The fingerprint — `crates/stride-core/src/fingerprint.rs`, whose signs are fixed and whose labels are observations.

*What it may not claim.* There is no form score and no efficiency grade. Each observation is a number with a direction, and the direction words are the app's own.

**KEEP** — there is no form score; the signs are the app's own

**26 · The run update**

> The second half was faster than the first by 0:06 /km, and your heart rate was higher by 3 bpm over the run.

*What it is.* The one place a model writes. It arranges the facts the app computed — the drifts, the decoupling, the laps — and does nothing else. A model wrote this sentence from the numbers above and nothing else.

*Where it comes from.* The update payload and its checks — `crates/stride-core/src/update.rs`, with the register and the fixed signs set out in `docs/contract/voice.md`.

*What it may not claim.* It may not use the closed direction set, may not call the run good or bad, may not mention the plan, and may not carry the run's title — a title is the athlete's own words.

**KEEP** — the run update's register, and the signs the model may only repeat

**27 · Against the plan**

> 18.0 km planned · 20.4 km recorded · 2.4 km more than the 18 km target

*What it is.* The plan's session for that day, and the run beside it. A statement about the athlete's own plan.

*Where it comes from.* The plan's dated session and the run — `crates/stride-core/src/plan.rs`.

*What it may not claim.* It may not say the run met the plan or missed it. The number is the difference; the reader draws the conclusion.

**KEEP** — the difference is stated; the plan is never judged

**28 · The note**

> the last 5 km were into a headwind

*What it is.* The only free text the product stores. It belongs to the run and to the athlete.

*Where it comes from.* The note field on the activity record.

*What it may not claim.* The app never writes it, never reads it back as a signal, and never turns it into a tag or a mood.

**KEEP** — the note is the athlete's own words and the app never reads it back

**29 · The night's sleep**

> Sleep on 20 – 21 Nov 2026 — 6 h 45 m

*What it is.* A link to the night before the run, on Body. It is context, and it is a link rather than a card. The night before the run is on Body; it is a link, not a card.

*Where it comes from.* The wellness sleep record, shown on Body for any date.

*What it may not claim.* It may not sit next to the run's numbers and imply a cause. The run's page holds facts about the run; Body holds the night.

**DROP** — the link is a link


### 4 · Progress

**30 · The source line, once**

> your zones · running · max heart rate 183 bpm · taken from your watch on 3 Nov 2026

*What it is.* The wall's one source line, at the top of the screen. Every card below is measured against the model named here.

*Where it comes from.* `crates/stride-core/src/zones.rs`; the wall's own layout in `exploration/cards-as-pictures.html`.

*What it may not claim.* No card below may name the model again. Where an answer used a different model, the card says so in place, because a silent difference would be a wrong number.

**KEEP** — one source line per screen

**31 · The eleven cards**

> this week · where the time went · training load · the weeks · the pace · bests · your shape · the months · the schedule · the habit · the goals

*What it is.* The wall after the cut: six cards on Progress and five on the athlete profile, each read as a picture first. Where a card carries a signed figure, it is the change against the period before, and the sign is the direction of the change.

*Where it comes from.* The eleven-card design in `exploration/cards-as-pictures.html`; the cut list in `exploration/surface-inventory.md` §1.

*What it may not claim.* No card is a list of sentences, and no caption repeats a number the picture already carries. A card that cannot be drawn is merged or dropped.

**KEEP** — the signed figure is the change against the period before

**32 · The window selector**

> 7 Sep – 24 Nov 2026 · last 12 weeks

*What it is.* The range the wall is answering about. Every card on the wall is about this window.

*Where it comes from.* The range state on `frontend/src/pages/progressWall.tsx`.

*What it may not claim.* Changing the window changes every card, including the sentences. A card may name two windows; it may not mix them.

**KEEP** — the window changes every card, including the sentences

**33 · A card's caption**

> 7 Sep – 24 Nov 2026 · measured over the time you were running

*What it is.* The line under a picture: the period, and the reason something is missing. The caption names the period, and the reason something is missing — nothing else.

*Where it comes from.* The card captions in `exploration/cards-as-pictures.html`.

*What it may not claim.* It may not interpret the picture. If the caption has to explain the picture, the picture is the wrong picture.

**KEEP** — a caption names the period and the absence; it may not interpret


### 5 · The pace card

**34 · The two-axis ladder**

> threshold 4:16–4:21 · the goal race 4:21 · marathon 4:32 · long run 5:32 → 5:00 · general aerobic 5:35–5:50 · recovery 5:36

*What it is.* The training paces and the heart-rate bands on two axes: the athlete's own heart rate on the left, the pace on the right, each rung kept apart. The threshold rung's band is the hour-pace band on the half-marathon span, because the adopted plan is a half; the race candidate's band is the same rule on the shorter distance — the athlete's typed 10K. Each names its anchor, so the two numbers are two anchors of one rule and not two answers to one question.

*Where it comes from.* The ladder design in `exploration/pace-engine.html`; the numbers come from the chain in `crates/stride-core/src/pace.rs` — race time → VDOT → race paces → the book's percentages.

*What it may not claim.* The band is the method's rule and the pace is either the method's rule or the athlete's own; they are never merged, and the athlete's own pace is never presented as the method's number.

**KEEP** — whose number each rung is — the method's rule or the athlete's own

**35 · The two axes**

> heart rate 115–190 bpm on the left · pace 4:10–5:50 /km on the right

*What it is.* A heart rate and a pace are numbers about different things, so they get a scale each, and the app never converts between them. Heart rate on the left, from the zone model's own floors; pace on the right, from the race the athlete typed. They are numbers about different things, so they are never converted into one another.

*Where it comes from.* The zone model's floors for the heart-rate axis; `crates/stride-core/src/pace.rs` for the pace axis.

*What it may not claim.* No rung may be drawn where a heart rate has been converted into a pace, or the reverse.

**KEEP** — a heart rate and a pace are never converted into one another

**36 · The colours**

> amber — the plan's rule · azure — your own data · grey — worked out from your race

*What it is.* Three colours, and none of them is a verdict. Amber is the book, azure is the athlete, grey is the engine's arithmetic. Amber is the method's rule. Azure is the athlete's own data. Grey is the engine's own derivation.

*Where it comes from.* The chart tokens in `frontend/src/tokens.css`; the rule in `exploration/pace-engine.html` §1.

*What it may not claim.* No colour may mean better or worse, and no colour may rank one rung above another.

**KEEP** — the colours are provenance, never verdicts

**37 · The ladder's source**

> From your 10K in 41:36 on 12 Oct 2026 · entered 13 Oct 2026 · changed 3 Nov 2026

*What it is.* Which race the paces came from, when it was typed, and when it was changed.

*Where it comes from.* The stored race record and its change history — `crates/stride-core/src/pace.rs`.

*What it may not claim.* A race time is typed by the athlete. The app never estimates one from the training, and never invents one.

**KEEP** — the race was typed by the athlete; the app never estimates one

**38 · The race equivalents**

> 5 km 19:57 · 10 km 41:36 · 15 km 1:03:56 · half 1:31:47 · marathon 3:11:22

*What it is.* The times the engine computes for the other distances from the one race the athlete typed.

*Where it comes from.* Riegel, exponent 1.06, from the stored race — `crates/stride-core/src/pace.rs`.

*What it may not claim.* A model with its assumption named — not a forecast. These are equivalents of a typed race, not predictions of what the athlete will run.

**KEEP** — the equivalents are a model with its assumption named, not a forecast

**39 · The physiological-cost curve**

> the share of your maximum oxygen uptake each duration holds, against time on a log scale

*What it is.* A second reading of the same athlete: the cost a duration holds, with the athlete's own races sitting on the curve at the share their own pace cost.

*Where it comes from.* The curve design in `exploration/pace-engine.html` §3, on its own route `/insights/pace-curve`; the VO₂max it is measured against comes from `pace.rs`.

*What it may not claim.* It may not be drawn on a calendar or a training-week axis, and it never carries a readiness reading.

**DROP** — the curve is visible

**40 · The rule on the curve**

> the training plan's heart-rate bands are named in the caption and never drawn on this axis

*What it is.* Pfitzinger's bands are percentages of maximal heart rate; the curve's axis is percentages of VO₂max. The two are not convertible without the athlete's lactate relation, which Stride does not have.

*Where it comes from.* The rule is stated in `exploration/pace-engine.md` and in `exploration/pfitzinger-spine.md`.

*What it may not claim.* No band may be drawn on this axis, and the curve may not be used to check a heart-rate band.

**KEEP** — the method's heart-rate bands are never drawn on the %maxVO₂ axis

**41 · The curve's own route**

> /insights/pace-curve

*What it is.* The curve has a page of its own, reached from the pace card — the pace card and the curve are two screens, not one crowded card. The curve is a screen of its own, reached from this card; the pace card and the curve are two screens, not one crowded card.

*Where it comes from.* The route in `frontend/src/app/destinations.ts`; the decision is recorded in `exploration/pace-engine.html` §3.

*What it may not claim.* It may not be folded back into the pace card, and it may not be shown without its own assumptions listed.

**KEEP** — the curve is a screen of its own, not part of the pace card


### 6 · The threshold

**42 · The threshold you use**

> 165 bpm · the start of your watch's fifth zone · made for the Pfitzinger 10K plan · chosen on 3 Nov 2026

*What it is.* The chosen threshold, stored as a dated fact with its source and the base it was made for.

*Where it comes from.* The stored choice and its history — `crates/stride-core/src/plan.rs` and the threshold store behind it; the design is in `exploration/threshold-setup.html`.

*What it may not claim.* It may not be updated silently, and it may not be checked for whether the number still applies — the app does not ask.

**KEEP** — the threshold is a dated fact, and the app does not ask whether it still applies

**43 · The four candidates**

> your watch's zones · from your 10K · from your 45-minute time trial · a number you entered

*What it is.* Four sources, each computed by one rule, each card naming the rule it used. They are listed side by side, and the arrangement is not a ranking.

*Where it comes from.* The threshold candidates — `crates/stride-core/src/`'s threshold rules; the card design in `exploration/threshold-setup.html` §1.

*What it may not claim.* They are never ranked, no chip may name one as the one to take, and no candidate may be called stale or wrong.

**KEEP** — the candidates are unranked, and none is named as the one to take

**44 · The two lanes**

> heart rate 158–176 bpm above · pace 4:00–4:30 /km below

*What it is.* The same four candidates drawn: a heart rate and a pace on a lane each, because they are not interchangeable. A heart rate and a pace are numbers about different things, so they get a lane each.

*Where it comes from.* The lanes picture in `exploration/threshold-setup.html`.

*What it may not claim.* A pace band is drawn as a band — its two ends, never a midpoint — and the app never converts a heart rate into a pace.

**KEEP** — a pace band is drawn as a band — its two ends, never a midpoint

**45 · The history**

> 3 Nov 2026 · 165 bpm · 3 Mar 2026 · 172 bpm

*What it is.* Each choice that has been made, with its own date and its own source. A change writes another row and the old one stays. Each row is one choice, made for a base, on its own day: the app does not say the threshold has changed, and does not ask whether a value still applies.

*Where it comes from.* The threshold history — the store behind `exploration/threshold-setup.html` §2.

*What it may not claim.* The app does not say the threshold has changed, and it does not ask whether a value still applies.

**KEEP** — each row is one choice on its own day, and the app never says it has changed

**46 · The disagreement**

> 165 bpm and 168 bpm — both are shown

*What it is.* Two candidates that disagree, drawn at their own values with a marker over the span between them. Both marks stand at their own values; nothing is written about the distance between them.

*Where it comes from.* The disagreement state in `exploration/threshold-setup.html` §2.

*What it may not claim.* The app does not reconcile them and does not write about the distance between them. It may not suggest which is closer to the truth.

**KEEP** — a disagreement is shown, not reconciled

**47 · Where the threshold is used**

> the plan's pacing and the 80/20 plan's easy-day bands — and nowhere else

*What it is.* The chosen threshold is read in two places. The ladder's rung is the method's own rule on the race time; the ledger's rows and the load model answer against the athlete's own stored zones. The choice does not re-band the ledger, and the load model does not read it: two numbers, each named where it is used.

*Where it comes from.* `exploration/threshold-setup.html` §3 — the same separation the core keeps between `zones.rs` and `pace.rs`.

*What it may not claim.* The choice does not re-band the ledger, and the load model does not read it. Two numbers, each named where it is used.

**KEEP** — the choice does not re-band the ledger; two numbers, each named where it is used

**48 · The watch's zone model — what it cannot know**

> the app cannot check what was put into the watch

*What it is.* The candidate taken from the watch's own fifth zone floor. The card shows the value and the source; this note holds what that source cannot tell the app.

*Where it comes from.* The candidate's own rule and limit — the candidate block in `exploration/threshold-setup.html` §1, stored with the threshold choice.

*What it may not claim.* The app may not check what was put into the watch, and may not treat the floor as a measured threshold.

**KEEP** — what the watch's own model cannot tell the app

**49 · From your 10K — what it cannot know**

> a race is a race, not a test — the pace carries that day's conditions

*What it is.* The candidate taken from the typed race — the athlete's own 10K — by the book's own rule: the pace you could race for about an hour, on the shorter-distance branch, between the 15K pace and the 10-mile pace. The card shows the value, the source and the rule; this note holds what the rule cannot know.

*Where it comes from.* The candidate's own rule and limit — the candidate block in `exploration/threshold-setup.html` §1.

*What it may not claim.* A race is a race, not a test: the pace carries that day's conditions, and the app may not present it as a measured threshold.

**KEEP** — a race is a race, not a test

**50 · From your 45-minute time trial — what it cannot know**

> the estimate carries the run's conditions, the effort across the window, and the watch's heart-rate accuracy

*What it is.* The candidate computed from a field test by one published rule. The card shows the value, the source and the rule; this note holds what the estimate carries with it.

*Where it comes from.* The candidate's own rule and limit — the candidate block in `exploration/threshold-setup.html` §1; the rule assumes a 45-minute time trial on a flat course.

*What it may not claim.* The estimate may not be presented as a measurement of the athlete's threshold on any other day.

**KEEP** — what a field test's estimate carries with it

**51 · A number you entered — what it cannot know**

> the app cannot check what the number was taken from

*What it is.* The candidate the athlete typed in. The card shows the value and where it came from; this note holds what the app cannot know about it.

*Where it comes from.* The candidate's own rule and limit — the candidate block in `exploration/threshold-setup.html` §1.

*What it may not claim.* The app may not check what the number was taken from, and may not weigh it above or below the other candidates.

**KEEP** — the app cannot check what a typed number was taken from


### 7 · The plan

**52 · The base**

> the Pfitzinger 10K plan · adopted 30 Sep 2026

*What it is.* Which training base the athlete follows. The app holds one base at a time.

*Where it comes from.* The adopted plan record — `crates/stride-core/src/plan.rs`; the base's rules are in `exploration/pfitzinger-spine.md`.

*What it may not claim.* The app may not hold two bases at once, and it may not compare the athlete against a base they are not following.

**KEEP** — one base at a time

**53 · The week's sessions**

> Mon 23 Nov … Sun 29 Nov 2026 — seven sessions with their kinds

*What it is.* The plan's own week, in the order the days fall. Two days of it have happened: 5.0 km on Monday and 6.0 km on Tuesday, against 8.0 km planned on each. The plan is not restated as a verdict.

*Where it comes from.* The plan's dated sessions — `crates/stride-core/src/plan.rs`.

*What it may not claim.* A session may not be marked as recommended, and the week may not be ordered by anything but the calendar.

**KEEP** — the plan's week is never restated as a verdict

**54 · The session kinds**

> Endurance · Intervals · Long run

*What it is.* The book's own names for the training components. They are the plan's vocabulary, and they are not commands. Endurance, Intervals and Long run are the book's own names for the training components; they are the plan's vocabulary — never commands.

*Where it comes from.* The run types in `exploration/pfitzinger-spine.md` §1.1, each with its chapter citation.

*What it may not claim.* <i>Easy</i> and <i>very easy</i> survive as session types only. They are never commands, and never linked to a vital.

**KEEP** — “easy” survives as a session type only — never a command

**55 · The session paces**

> 5:12–5:38 /km · 4:16–4:21 /km · 5:32 → 5:00 /km

*What it is.* The pace each session asks for, taken from the ladder's rungs and the heart-rate band beside it. Each pace comes from the ladder's rungs, and each carries the heart-rate band beside it.

*Where it comes from.* The rungs in `crates/stride-core/src/pace.rs`, applied to the base's own rules.

*What it may not claim.* A pace may not be turned into an instruction, and a session may not be given a pace it does not have a rule for.

**DROP** — the paces are visible

**56 · The pacing's threshold**

> 165 bpm · 3 Nov 2026

*What it is.* The threshold the plan's pacing was built from, named with the day it was chosen.

*Where it comes from.* The stored threshold choice — `crates/stride-core/src/plan.rs`; the candidate is in `exploration/threshold-setup.html` §1.

*What it may not claim.* The choice is not merged with the watch's zone model, and the two numbers may not be shown as one.

**KEEP** — the threshold is named with its date, and is never merged with the zone model

**57 · The plan's source**

> your 10K in 41:36 on 12 Oct 2026

*What it is.* What the plan was built from: the athlete's own typed race time. A race time is typed by the athlete: the plan is never built from an estimated one, and it is never re-scaled silently.

*Where it comes from.* The plan's provenance record — `crates/stride-core/src/plan.rs`; the rules are in `exploration/pfitzinger-spine.md`.

*What it may not claim.* The plan may not be re-scaled silently when the race time changes, and it may not be built from an estimated race.

**KEEP** — the plan is built from a typed race and is never re-scaled silently


### 8 · The athlete profile

**58 · Your shape**

> volume · consistency · intensity · duration · density · variety · days of the week

*What it is.* Seven facets, drawn as seven gauges on one common scale, each with the way it moved against the previous window.

*Where it comes from.* `crates/stride-core/src/profile.rs` — which argues explicitly against the radar this card replaced; the picture is in `exploration/cards-as-pictures.html` card 7.

*What it may not claim.* The seven are not a shape and not a score. There is no polygon, and no facet may be added to make a picture look better.

**KEEP** — the seven facets are not a shape and not a score

**59 · The months**

> Dec 186 km · … · Sep 208.9 km · Oct 280.1 km · Nov 75.8 km

*What it is.* Twelve columns of monthly volume, each split by the zone mix.

*Where it comes from.* `crates/stride-core/src/year.rs`; the picture is in `exploration/cards-as-pictures.html` card 8.

*What it may not claim.* November is a partial month and says so. A month without a zone model carries the distance and no mix.

**DROP** — the months are visible

**60 · The schedule**

> seven rows for the days of the week, twenty-four columns for the hours

*What it is.* Every run in the window placed by the weekday and the hour it was run. That is the shape of the athlete's week.

*Where it comes from.* The weekday and time-of-day aggregates — `crates/stride-core/src/profile.rs`; the picture is in `exploration/cards-as-pictures.html` card 9.

*What it may not claim.* It may not be read as a routine or a discipline, and it may not suggest the athlete should move their runs.

**KEEP** — the schedule may not be read as a routine or a discipline

**61 · The habit**

> 4 days in a row · 12 km on 13 days · 58 days recorded

*What it is.* The streak as a row of day marks, and the Eddington as a ladder of ticks — one tick per distance, as tall as the number of days that reached it.

*Where it comes from.* `crates/stride-core/src/streaks.rs` and `eddington.rs`; the picture is in `exploration/cards-as-pictures.html` card 10.

*What it may not claim.* The Eddington needs three comparable days before it is drawn at all, and it is not a score the athlete is graded on.

**KEEP** — the Eddington needs three comparable days, and is not a grade

**62 · The goals**

> a 10 km a week · a half marathon — 12 weeks of marks, and the plan's sessions under them

*What it is.* The athlete's own challenges, drawn as marks, with the plan's sessions for the week beneath them.

*Where it comes from.* `crates/stride-core/src/challenges.rs`; the picture is in `exploration/cards-as-pictures.html` card 11.

*What it may not claim.* A week without the distance is a quiet empty square, never a red cross, and a session that was not done is not a miss.

**KEEP** — a week without the distance is an empty square, not a miss


### 9 · Body

**63 · Sleep on 24 Nov 2026**

> 6 h 10 m — deep 78 m · REM 94 m · light 174 m · awake 24 m

*What it is.* One night of sleep, drawn, for any date the athlete picks.

*Where it comes from.* The wellness sleep record — the same data Today shows for last night.

*What it may not claim.* It may not be scored or rated, and one night may not be read as a trend.

**DROP** — the night is visible

**64 · The three signals**

> Resting heart rate — 54 bpm · about usual   HRV — 48 ms · could be better

*What it is.* Each signal on its own: the value, its direction against the athlete's own usual band, and the arithmetic behind it.

*Where it comes from.* `crates/stride-core/src/direction.rs` — the band is the median of the last 30 nights ± 1.5 × MAD.

*What it may not claim.* The signals may not be combined into a verdict, and there is no readiness score.

**KEEP** — the signals are never combined, and there is no readiness score

**65 · The patterns**

> On the 5 other nights you ran before 06:00, your sleep was 7 h 02 m — longer than your usual.

*What it is.* What usually happened, as a fact about the past, with the number of comparable days beside it. A pattern is a fact about the past, and it is only ever drawn here.

*Where it comes from.* `crates/stride-core/src/pattern.rs`; the `pattern` register in `docs/contract/voice.md`.

*What it may not claim.* A pattern never appears outside Body, never next to a suggestion, and never from fewer than three comparable days.

**KEEP** — a pattern carries its sample size and is drawn nowhere else

**66 · The date and the window**

> 24 Nov 2026 · last 8 nights

*What it is.* The controls: which night, and how much history beside it. The date, and how much history beside it.

*Where it comes from.* The Body page's own state — `frontend/src/pages/BodyPage.tsx`.

*What it may not claim.* The controls may not choose the window to flatter the athlete, and a window with too few nights says so rather than drawing less.

**DROP** — the controls are visible


### 10 · Compare

**67 · The basis**

> Same distance, 3 weeks apart

*What it is.* One sentence saying what the comparison is. It is what makes the rest of the screen legible.

*Where it comes from.* The comparison's own builder — `crates/stride-core/src/statement.rs`; the design is in `exploration/surface-inventory.md` §1.

*What it may not claim.* It may not say one run was better. It states the similarity or the difference and nothing more.

**KEEP** — the basis states the comparison and no verdict

**68 · The metric table**

> distance 20.4 / 19.6 km · pace 5:50 / 5:49 /km · heart rate 145 / 144 bpm

*What it is.* Metric by metric, the two runs side by side and the difference beside them.

*Where it comes from.* The run facts and the arithmetic — `crates/stride-core/src/types.rs` and `units.rs`.

*What it may not claim.* No row may be a score, and the difference may not be coloured as better or worse.

**KEEP** — no row is a score, and the difference is not coloured

**69 · The two runs**

> 21 Nov 2026 · 20.4 km  ·  31 Oct 2026 · 19.6 km

*What it is.* The two runs themselves, so the reader knows what is being compared without opening either.

*Where it comes from.* The run records — `crates/stride-core/src/types.rs`.

*What it may not claim.* Neither run may be the one that is called the better one.

**DROP** — the two runs are visible

**70 · The shared chart**

> the same one chart as the run detail, with the A/B switch

*What it is.* The run detail's chart, with a switch between the two runs. One chart, two places, one scale for both runs.

*Where it comes from.* The consolidated chart in `exploration/consolidated-chart.html`, used by both `ActivityPage.tsx` and `ComparePage.tsx`.

*What it may not claim.* It may not use a different scale for each run, and it may not hide a signal that one run has and the other does not.

**KEEP** — one chart and one scale for two runs, with no signal hidden

**71 · Choose the other run**

> 31 Oct 2026 · 19.6 km

*What it is.* The picker: the other run can be changed without leaving the screen. The picker is a list, not a ranking.

*Where it comes from.* The picker on `ComparePage.tsx`.

*What it may not claim.* It may not rank the candidates, and it may not propose a pair it thinks is interesting.

**KEEP** — the picker is a list, not a ranking


### 11 · The year

**72 · The months**

> twelve month columns, each with its distance and its zone mix

*What it is.* The year as twelve columns — the richest unit in the product: a week strip, a distance and a zone mix in one thing.

*Where it comes from.* `crates/stride-core/src/year.rs`; the month card in `exploration/surface-inventory.md` §1.

*What it may not claim.* A month is not a grade, and a partial month says so.

**DROP** — the months are visible

**73 · The year as a calendar**

> 2026 — one cell per day, filled by distance

*What it is.* The year as a calendar. It is the only calendar in the product.

*Where it comes from.* `crates/stride-core/src/year.rs`; the retired wall calendar is in `exploration/surface-inventory.md` §1.

*What it may not claim.* A day with no run is an empty cell, never a red one, and the calendar may not be coloured as a problem.

**KEEP** — an empty cell is never a red one

**74 · What changed**

> Jan – Aug · 1 676 km · 187 runs   —   Sep – 24 Nov · 565 km · 58 runs

*What it is.* The year over year, or the span over the span before it, as a set of figures.

*Where it comes from.* `crates/stride-core/src/year.rs` and the `statement` register.

*What it may not claim.* It may not say the athlete got better or worse, and the two periods must be named so the reader can see they are different lengths.

**KEEP** — the two periods are named because they are different lengths; no better, no worse


### 12 · The film and the heatmap

**75 · The film**

> 21 Nov 2026 — the timeline, the zone ribbon, the playhead at 1 h 15 m

*What it is.* The deep view of one run: the timeline, the band the run was in, and a playhead that moves. The playhead stands at 1 h 15 m.

*Where it comes from.* The film page — `frontend/src/pages/FilmPage.tsx`; the design is in `exploration/surface-inventory.md` §1.

*What it may not claim.* The film carries no reading of its own. Its lap column is a link into the run detail, not a second table.

**KEEP** — the film carries no reading of its own

**76 · The heatmap**

> a grid over the stored positions of the run

*What it is.* Where the athlete runs: the run's recorded positions, laid on a grid and counted. Each cell counts the recorded positions that fall in it, and the grid is over the run, not over the calendar.

*Where it comes from.* `crates/stride-core/src/heatmap.rs`; the panel is the `/heatmap` route in `exploration/surface-inventory.md` §1.

*What it may not claim.* It is not the calendar of days coloured by distance — that name is a collision the catalogue still has to settle. A grid cell may not be read as a favourite place.

**KEEP** — the heatmap is a grid over route positions, not the calendar


### The states

**77 · No zone model**

> No zones yet — your watch's heart-rate zones have not come through.

*What it is.* The intensity family has no value and says why, in the core's own words. The shape family does not depend on the model, so it still answers.

*Where it comes from.* `crates/stride-core/src/zones.rs` — a run with no stored model is not calibrated; the result is no value, never band 1 and never zero.

*What it may not claim.* The absence may not be shown as zero, and the runs themselves may not disappear with the model.

**KEEP** — what the watch's own model cannot tell the app

**78 · No race time**

> No race entered yet — the race times and the training paces are not worked out.

*What it is.* The ladder and the curve drawn empty, each naming the fields the athlete can type.

*Where it comes from.* `crates/stride-core/src/pace.rs`; the states are in `exploration/pace-engine.html` §4.

*What it may not claim.* The app never estimates a race from the training, and an empty ladder is not a failed one.

**KEEP** — a race is a race, not a test

**79 · No plan adopted**

> No plan adopted yet — there are no sessions to show.

*What it is.* The plan's outline is gone from the pictures, the session row is absent, and each says why.

*Where it comes from.* `crates/stride-core/src/plan.rs`; the state is in `exploration/cards-as-pictures.html` card 1.

*What it may not claim.* The runs are still drawn as they are, and the absence may not be filled with a suggestion.

**KEEP** — what a field test's estimate carries with it

**80 · No threshold chosen**

> No threshold chosen yet. Everything that needs one says what is missing.

*What it is.* Nothing is stored. The reasons name where the number would come from, and the setup may be returned to later.

*Where it comes from.* The threshold store — the design is in `exploration/threshold-setup.html` §2.

*What it may not claim.* The setup may not be forced, and no answer may fall back to the watch's zone model silently.

**KEEP** — the app cannot check what a typed number was taken from

**81 · An empty window**

> No runs recorded.

*What it is.* One line, in the core's words — never 'rest': a day without a recorded run may be a forgotten watch.

*Where it comes from.* `crates/stride-core/src/rolling.rs`; the state is in `exploration/stimulus-ledger.html` §3.

*What it may not claim.* It may not say the athlete rested, and it may not list what was not done.

**KEEP** — a missing model is a no value, never zero

**82 · A run with no heart-rate data**

> No heart rate recorded — none of the runs in this period recorded a heart rate.

*What it is.* One family answers and the other does not. The shape rows come from distance, time and the watch's workout steps, and need no heart rate.

*Where it comes from.* `crates/stride-core/src/zones.rs`; the state is in `exploration/stimulus-ledger.html` §3.

*What it may not claim.* The absence is not the athlete's: no band row means no value, never 0 m.

**KEEP** — the app never estimates a race from the training

## 4 · The states

The same screens with their absences. A missing value is never a zero, and a measured zero is
not a missing value. These are the product's honesty, so they belong in the end-to-end.

| state | what it is | what it says | the note |
|---|---|---|---|
| **No zones yet** | the intensity family has no value and says why; the shape family does not depend on the model, so it still answers | *"No zones yet — your watch's heart-rate zones have not come through."* No split, no zone times, no shares, no last dates. The runs themselves are still there. | 77 |
| **No race entered yet** | the ladder and the curve are drawn empty, and each names the fields the athlete can type | *"No race entered yet — the race times and the training paces are not worked out."* A race is entered as distance, time and the day of the race; the name is optional. | 78 |
| **No plan adopted yet** | the plan's outline is gone from the pictures and the session row is absent, and each says why | *"No plan adopted yet — there are no sessions to show."* The runs are still drawn, as they are. | 79 |
| **No threshold chosen yet** | nothing is stored; every answer that depends on one is a no value with a reason | *"No threshold chosen yet. Everything that needs one says what is missing."* It could come from your watch's zones, a race, a field test, or a number you enter. The setup may be skipped, and the card can be returned to. | 80 |
| **An empty window** | one line, in the core's words | *"No runs recorded."* A day without a recorded run may be a run the watch did not record. | 81 |
| **A run with no heart-rate data** | one family answers and the other does not; the shape rows need no heart rate | *"No heart rate recorded — none of the runs in this period recorded a heart rate."* No zone row means nothing was measured, never 0 m. | 82 |

**The states that were hard to say without a verdict.** Two of them wanted a judgement and were
not given one:

* *"No plan adopted"* and *"no threshold chosen"* both want to say what the athlete should do
  next. They do not. The absence is drawn, the reason is named, and the nearest thing the
  product can say is where the missing number would come from.
* *"A run with no heart-rate data"* wants to be a warning about the watch. It is not: the
  absence is not the athlete's, and the card says what it could not measure rather than what the
  athlete did not do.

---

## 5 · What is **not** in this end-to-end

* **Any interaction.** The thresholds, the filters, the chart's signal switch, the compare
  picker and the calendar's days are drawn, not wired. The annotation toggle is the only
  working control.
* **The annotations are not the product.** They are explanatory text *about* the product, for
  this review. They do not imply a verdict about the athlete, and none of them would be
  shipped in this form.
* **Any data that is not the athlete's.** There is no second athlete, no comparison to another
  runner, and no synthetic "typical week" other than the athlete's own median.
* **The setup wizard, Settings, the Garmin connection state, the import history, the export and
  delete, the notification settings, the challenges settings and the weekly-goal setting.**
* **The retired wall cards**, the old `/pace` page and the old profile radar — they are cut in
  `exploration/surface-inventory.md`, and this end-to-end is drawn after the cut.
* **The 80/20 plan.** It is the second base, and it is named only where the threshold's use is
  explained; its plan is not drawn.
* **Real accessibility work.** Every picture carries an `aria-label`, but the contrast, focus
  order and screen-reader passes are not done here.

---

## 6 · Inconsistencies found between the prototypes

These are worth more than the walk-through. Each one is a place where two prototypes of the
same product disagree, and each needed a decision to make one page.

1. **The zone model is dated three ways.** `stimulus-ledger.html` says read **3 Nov 2026**,
   max heart rate 183; `threshold-setup.html` says read **12 Oct 2026**, max heart rate 188,
   floors 118/132/147/163/170; `cards-as-pictures.html` says read **12 Nov 2026**, max 183.
   *Used: 3 Nov 2026, max 183, floors scaled to 115/128/143/159/165.*
2. **The typed race is a different race in every prototype.** `pace-engine.html`'s athlete
   typed a **half marathon in 1:32:21** on 12 Oct 2026 (VDOT 49.5); `threshold-setup.html`'s
   athlete typed a **10K in 41:36** on the same day; `cards-as-pictures.html`'s pace card says
   the training paces came from **the 10 km of 49:12 on 18 Nov 2026**. *Used: the 10K in 41:36,
   and the ladder and the curve were re-anchored on it (Riegel) — which is why the ladder's
   rungs and the VDOT differ from the pace-engine prototype's.*
3. **The threshold band is drawn twice, on two anchors.** `pace-engine.html`'s threshold rung is
   the hour-pace band, from the 15K pace to the half marathon's (4:17–4:23 for their athlete);
   `threshold-setup.html`'s candidate is the band from the 15K pace to the 10-mile pace
   (4:15–4:19). **These are not a contradiction** — they are the same rule on two anchors. *Used:
   the ladder's rung is the half-marathon span (**4:16–4:21**, labelled "for your half
   marathon", because the adopted plan is a half) and the candidate card is the shorter-distance
   branch, from the athlete's typed **10K** (**4:16–4:17**, labelled "from your 10K"). Each
   names its anchor, so no reader sees two numbers for one thing.*
4. **The last run in Z4–Z5.** The brief says **15 Oct 2026**; the ledger says the last run in
   Z4–Z5 was **20 Oct 2026** and the last run with a workout step was **15 Oct 2026**. The run
   table agrees with the ledger: the four hard runs are 17 Sep, 1 Oct, 15 Oct and 20 Oct.
   *Used: 20 Oct for the band, 15 Oct for the workout steps — the brief appears to have merged
   the two dates.*
5. **The window's moving time is a minute out.** The ledger says **49 h 33 m**, but its own
   band times (41 h 32 m + 6 h 52 m + 1 h 08 m) and the run table both sum to **49 h 32 m**.
   *Used: 49 h 32 m, the sum — and the run table was corrected to match it.*
6. **The zone split cannot be exact from the run table.** The ledger's split (Z1 42.4, Z2 41.7,
   Z3 13.9, Z4 2.0, Z5 0.3) is finer than the table, which carries only low / moderate / hard
   minutes (83.8 % / 13.9 % / 2.3 %). *Used: the ledger's split as the canonical figures and
   the table for the matrix — and the ledger card is drawn with the canonical split, as its own
   prototype does.*
7. **Two runs' average heart rates contradicted their own zone minutes** in the cards
   prototype's table (see §1). *Corrected in `athlete.js`, and flagged in its header.*
8. **The two "heatmap" names still collide** — the wall's calendar of days coloured by
   distance, and the `/heatmap` route grid. The inventory says one of them must be renamed;
   this end-to-end draws the route grid and says so in its note.

---

## 7 · How the annotation toggle works

* The header switch adds or removes the class `annotated` on `<body>`. The pins are
  `display:none` unless that class is present, and the notes rail is hidden below 1380 px
  regardless — on a narrow window the notes open in a sheet at the bottom instead.
* Each annotated element carries `data-note="n"`; the pin is inserted by one pass over the
  document after the screens are rendered, and it takes the note's number, so the numbering
  follows the reading order of the page.
* Clicking a pin marks every copy of that element (the phone frame and the desktop panel) with
  an outline, selects the note in the rail, and — on a narrow window — opens the sheet with the
  note's three parts.
* Clicking a note in the rail scrolls its element into view. The rail shows only the notes of
  the screen you are in, and it re-renders as you scroll.
* In **clean** mode the pins, the outlines and the rail are gone, and the page is the product.

**82 notes** were written, in three registers: the rule the athlete could be misled without
(**KEEP**), the provenance of the element (**DROP**), and the boundary it may not cross (kept
either way, because it is the contract).

The explanation that used to sit on the product surface now lives here — see §9.

---

## 9 · The sweep — what moved, and where each line went

*Rule applied:* the product surface carries the picture, its labels, the numbers, the period,
and — only where something is missing — the reason something is missing. Every line that
explained the **design** rather than the **data** came off the surface. Nothing was deleted:
each line is now a note, or a small caption inside the frame of the picture it belongs to.

### 9.1 · Moved into the notes

| screen | the line that came off | where it went |
|---|---|---|
| Today | "Both are options from the plan. Neither is ranked, and neither is tied to a vital." | note **7** |
| Run detail | "one cursor, three signals" (the chart's sub-header) | note **19** |
| Run detail | "The zone model in force on 21 Nov is the one the run is measured against, so the run and the window cannot disagree." | note **22** |
| Run detail | "Every value is the mean over the run, measured on the moving samples." | note **25** |
| Run detail | "A model wrote this sentence from the numbers above and nothing else." | note **26** |
| Progress | "the signed figure is the change vs the period before" (the matrix's key) | note **31** |
| The pace card | "Amber is the method's rule. Azure is the athlete's own data. Grey is the engine's own derivation." | note **36** (the legend under the ladder carries the labels) |
| The pace card | "The curve is a screen of its own, reached from this card…" | note **41** (the route chip `/insights/pace-curve` stays) |
| The threshold | "four sources, unranked" (the candidates' sub-header) | note **43** |
| The threshold | "two kinds, two lanes" (the lanes' sub-header) | note **44** |
| The threshold | "Both marks stand at their own values. Nothing is written about the distance between them." | note **46** |
| The threshold | "The choice does not re-band the ledger, and the load model does not read it…" | note **47** |
| The threshold | "a dated fact" (the choice's sub-header) | note **42** |
| The threshold | each candidate's **"it cannot know · …"** line | four new notes: **48**, **49**, **50**, **51** — one per candidate |
| The plan | "one at a time" (the base's sub-header) | note **52** |
| The plan | "Two days of the week have happened: 5.0 km on Monday and 6.0 km on Tuesday…" | note **53** (the rows themselves carry "5.0 km recorded") |
| The plan | "Endurance, Intervals and Long run are the book's own names for the training components…" | note **54** (the three kinds stay on the card, as chips) |
| The plan | "Each pace comes from the ladder's rungs, and each carries the heart-rate band beside it." | note **55** |
| The plan | "A race time is typed by the athlete. The plan is never built from an estimated one…" | note **57** (the source line — the 10K, its time and its date — stays) |
| Body | "The date, and how much history beside it." | note **66** (the date and the count of nights stay) |
| Body | "A pattern is a fact about the past, and it is only ever drawn here." | note **65** |
| Compare | "the same one chart as the run detail" (the chart's sub-header) | note **70** |
| Compare | "one chart, two runs, one scale" (in the picture) | note **70** |
| Compare | "the picker is a list, not a ranking" | note **71** |
| The film | "The timeline, the band the run was in, and the playhead at 1 h 15 m." | note **75** |
| The heatmap | "The run's recorded positions, laid on a grid and counted…" | note **76** |
| Runs | "the browse calendar" (the calendar's sub-header) | note **12** |
| Runs | "A file the watch, or another app, wrote." | note **15** |
| Today | "Body holds sleep for any date, the three signals, and the patterns." | note **11** |
| Run detail | "The night before the run is on Body. It is a link, not a card." | note **29** |
| Progress | "The caption names the period, and the reason something is missing. Nothing else." | note **33** |
| The threshold | "Each row is one choice, made for a base, on its own day…" | note **45** |

### 9.2 · Moved into a caption inside the picture's frame

Two lines belong with the picture they explain, so they became **small captions under the
chart, inside its card** — and they are in the notes as well.

| screen | the line | where it is now |
|---|---|---|
| The pace card | "Heart rate on the left, from the zone model's own floors; pace on the right, from the race the athlete typed. They are numbers about different things, so they are never converted into one another." | a caption under the **ladder** (note **35**) |
| The pace card | "Pfitzinger's bands are percentages of maximal heart rate; this curve's axis is percentages of VO₂max…" | a caption under the **curve** (note **40**); the banner card that held it is gone |

### 9.3 · Cut from the captions and the pictures

| where | the line | why it went |
|---|---|---|
| Progress · *Where the time went* | the caption's "· measured over the moving time" | a basis that explains the drawing; the period stays |
| Progress · *The schedule* | the caption's "· the runs by weekday and hour" | the axes already say it |
| The year | the calendar's in-picture caption | it described the drawing; the period is on the screen |
| The heatmap | the grid's in-picture caption | it described the drawing |

### 9.4 · What the sweep did **not** touch

* **The walkthrough's own framing** — the screen's number and its "What it answers" line, and the
  captions under the phone and desktop frames. These are the walk-through talking to the reader,
  outside the product frames.
* **The pictures' own keys** — "the moving time of the window, by zone", "the azure lines are
  your zone floors", "the last 12 weeks, day by day", "the days that reached each distance",
  "one mark per day that holds a run", "both are shown". Each names what a *specific* mark in
  that picture means; none of them would read the same on a card showing a different
  measurement.
* **The states** — each one keeps its line about what is absent, because the absence is the
  content.

### 9.5 · What stayed, and why

* **The numbers and their labels** — the hero's figures, the effort table's bands and the
  *not moving* row, the splits, the metric table, the zone distributions.
* **The period** — the source line, "7 Sep – 24 Nov 2026" under every card, the night's date,
  the week label, "8 nights", "20 Nov 2026" on the 10K.
* **The source, where the source is the fact** — the race the paces came from, the model the
  numbers were read with and the day it was read, the rule that produced a candidate, the plan's
  adoption date, the threshold's date.
* **The reason something is missing** — every state, and the empty rows inside the wall's
  pictures.
* **The basis of a comparison** — "Same distance, 3 weeks apart", "Mon 23 Nov and Tue 24 Nov,
  against the same two days of the previous week", "12 weeks, against the same span one week
  earlier". These say what a number is a fact *about*, which is the same job the period does.

