# Stride — the product brief, and the brief for the Beta

*24 November 2026. Written for two readers: someone deciding what to build, and an
agent that has never seen the conversation that produced this. Every claim here is
checkable — a path, a function name, a citation. Where a document is incomplete or a
decision is pending, it says so.*

*Sources read: `exploration/the-wedge.md` · `exploration/decisions.md` ·
`exploration/methods-and-products.md` · `exploration/surface-inventory.md` ·
`exploration/stimulus-ledger.md` · `exploration/pace-engine.md` ·
`exploration/pfitzinger-spine.md` · `exploration/garmin-temperature-and-activity-fields.md` ·
`exploration/cleanup-plan.md` · `exploration/method-lens.md` ·
`docs/contract/voice.md` · `docs/contract/core-contract.md` ·
`docs/contract/api-paths.md` · `PLAN.md` · `README.md`, and the four prototypes.*

---

# Part 1 — What this app is

Stride is a private, local-first running log for one athlete. It keeps that athlete's
history — the runs, the sleep, the heart-rate observations — in an encrypted SQLite
database on the athlete's own machine, and it reads the data back as **facts with their
sources attached**. There is no Stride account, no hosted store, no subscription and no
service behind it: Garmin sync is a key the athlete pastes in themselves, and everything
else is local bytes (`README.md`, `docs/contract/core-contract.md` §7–§9).

The thing Stride is about is **the negative space of a training**. Not how the last run
felt, and not whether the training was "good" — but what the recent weeks *contain*, what
they *do not contain*, and what the athlete *chose to follow*. The answers are dates,
counts and shares: "the last run in Z4–Z5 was on 20 Oct 2026 — 35 days before the end of
the window", "the longest run in the window was 20.4 km", "the plan had a threshold
session on 14 October". That is the wedge, stated in `exploration/the-wedge.md` §4, and it
is the only thing competitors do not answer: Strava, Garmin Connect, TrainingPeaks and
RUNALYZE all report what happened; none of them says what did not.

The athlete it is for is the **intermediate runner** — someone already training, with a
base, chasing a distance. Entry level is served by Strava and Garmin, and the app says so
in the user's own words (`exploration/decisions.md` D14). The first method base is
Pfitzinger's; 80/20 is a second base, and the app holds **one plan at a time** — the
athlete's, never the app's recommendation (D13, D14).

The promise that defines it: **facts, never verdicts.** Every number names its source, its
period and its model; absence is always named, never zero; and the app never tells the
athlete what to do. The rules are frozen and machine-linted: seven string types, a closed
set of three direction words, and a 23-phrase ban list that fails the build
(`docs/contract/voice.md`, `crates/stride-core/src/strings.rs`).

**What it is not.** Not a coaching app — it never issues an instruction. Not a social
feed — there is no other athlete in it. Not a dashboard of scores — there is no readiness
score, no grade, no traffic light, no red. Not an account, not a subscription, not a
service: the athlete's history never leaves the machine unless the athlete moves it.

---

# Part 2 — The principles the product runs on

1. **Facts first.** The app reports values and how they moved, and stops there
   (`docs/contract/voice.md`).
2. **Absence is not zero.** A missing measurement is `null` with a reason, never `0`
   (`docs/contract/core-contract.md` §2).
3. **The picture first, the sentence second.** A card is read as a picture; the sentence is
   the caption and the accessible fallback (`exploration/decisions.md` D11).
4. **Local-first, no account.** The database is on the athlete's machine; the key is the
   athlete's own (`README.md`).
5. **One method at a time — the athlete's.** The ledger's rows stay method-agnostic; a
   method is never asserted, and a plan is adopted, not recommended (D13, D14).
6. **Every number names its model, its period and its source.** "measured over the moving
   time · your zone model, read on 12 Nov 2026" is the shape of a source line (D3, D5,
   `docs/contract/core-contract.md` §8).

Each of these is enforced somewhere mechanical rather than by review. The ban list and the
closed direction set live in `crates/stride-core/src/strings.rs` and are exported to
`crates/stride-core/strings.json`, which the frontend's lint reads — one rule, one owner.
The "no value" rule is the store's schema and the API's JSON: `null` is the only
representation of an absent measurement, and there is no sentinel value anywhere. The
picture-first rule is a requirement on the drawings (`exploration/cleanup-plan.md` §2.9:
six of the eleven surviving wall cards are sentence-only or tables today and must be
redrawn). The local-first rule is the shape of the crate graph — the store writes to a
local file and the app reads it, with no network path that is not the athlete's own sync
pull. The one-method rule is a type: `PlanSource` is `Baseline | Prescribed | Typed`, and
D14 adds the method identity beside it. The naming rule is the source line under every
card, and the core's `zones::model_source` is what produces it.

---

# Part 3 — The objects

## 3.1 What the app holds

| object | what it is | where it lives | how it is computed | when an input is missing |
|---|---|---|---|---|
| **Runs** | one activity: the watch's own numbers and the original file | `activities` table (`crates/stride-store/src/schema.rs:44`), `fit_blob` + `fit_sha256`; `crates/stride-store/src/repos/activity.rs` | decoded from the FIT original on read (`crates/stride-ingest/src/fit.rs`); derived values are **never stored** (`docs/contract/core-contract.md` §9) | an empty `fit_blob` means "no file": the run shows what it has and says it has no file; a re-download happens once, on the first pull that finds it missing |
| **Zone model** | the athlete's own five floors + maximum heart rate, with the day it was read | `hr_zone_models` (`schema.rs:160`), written by `repos/zones.rs:71 write_hr_zone_model_if_new` | `zones::ZoneFloors::is_usable`, `zones::time_in_zone`, `zones::zone_of`, `zones::model_source` (`crates/stride-core/src/zones.rs`) | no stored model → `zones.no_model` ("zones unavailable — no zone model has been read"); floors not in order → `zones.unusable_model`. **Never** band 1, never zero |
| **Wellness observations** | sleep, HRV, resting HR, stress — per day, per source | `wellness_daily`, `wellness_observations` (`schema.rs:75`, `schema.rs:195`); `crates/stride-store/src/repos/wellness.rs` | the band: `band::band`, `band::band_for`, `band::median`, `band::median_absolute_deviation` (`crates/stride-core/src/band.rs`) | fewer than three comparable nights → "not enough data for a comparison yet"; the raw value alone |
| **Race times** | the athlete's own typed result: distance, time, day of the race, the day it was entered and last changed | **does not exist yet.** `race_observations` is a sketch in `exploration/pace-engine.md` §3.2 | the pace engine's chain: race time → VDOT → the race-pace equivalents → LT pace → the training paces (§3.2 below) | no race → no value, with the reason ("no race time entered"). Stride never invents a race time and never infers one from training |
| **Plans** | the adopted base: dated sessions, with the plan's pacing | `plan_blocks`, `plan_sessions` (`schema.rs:89`, `schema.rs:101`); `crates/stride-store/src/repos/plan.rs` | `plan::adherence_of`, `plan::session_label`, `plan::PlanSource::banner` (`crates/stride-core/src/plan.rs`) | a session with no target distance is a session with no distance — not a zero |
| **Activity observations** | the source's own facts about an activity: training effect, its time in zone, the athlete's own effort (`directWorkoutRpe`) | **does not exist yet.** `activity_observations` is proposed in `exploration/garmin-temperature-and-activity-fields.md` §5.1(a); D10 | stored with provenance: source, the endpoint it came from, the day it was observed, the day it was read | a field the source never sent → no row; a field it sent as null → a row whose value is null. The two absences stay distinct |
| **Notes** | the athlete's free text on a run — the only free text the product stores | `notes` (`schema.rs:115`); `crates/stride-store/src/repos/notes.rs` | — | — |

## 3.2 The derived series — recomputed on read, never stored

| series | computed by | from | when an input is missing |
|---|---|---|---|
| **Load**: TRIMP impulse, acute (ATL), chronic (CTL), form, ratio, monotony, strain | `load::load_report`, `load::recovery_scenario`, `load::ratio_band_text` (`crates/stride-core/src/load.rs`) | the runs' heart rates and the load model's own assumptions (Banister *k*, the maximum heart rate from the zone model) | the answer names its scenario and its assumptions; the model's own recovery scenario is part of the card, not a recommendation |
| **Time in zone** | `zones::time_in_zone`, `zones::TimeInZone::{per_zone, moving_s}` | the run's samples and the zone model **in force on the run's day** (D3 stage 1) | the whole result is `None` when the run has no model; a *measured* zero is a real measurement and keeps a value |
| **Pace bands** | `pace::pace_bands`, `pace::PaceBand::label` (`crates/stride-core/src/pace.rs`) | the window's moving time on a 30 s/km grid | skipped spans are skipped, not zero-filled |
| **Window totals, comparison, polarised split** | `rolling::totals_in`, `rolling::compare_windows`, `rolling::polarised_shares`, `rolling::Movement::of` (`crates/stride-core/src/rolling.rs`) | the stored activities in the window | — |
| **The stimulus ledger** | **planned**: `crates/stride-core/src/stimulus.rs`, `stimulus::ledger` (`exploration/stimulus-ledger.md` §6.1) | each run's zone time, the workout steps, the window and the comparison period | a kind with no zone model has no value and names the reason; a measured zero keeps a value and no date |
| **The training paces** | **planned**: `crates/stride-core/src/pace_engine.rs` (`exploration/pace-engine.md` §10), rebuilt on the chain **race time (typed, dated) → VDOT → the race-pace equivalents → LT pace → the rungs** — see §6.2, Screen 3 | the athlete's typed race time, and the method's own percentages and heart-rate bands | a rung with no value is an empty slot on the ladder, and the caption names what the rule needs; a rung whose *pace* has no value (the athlete has no history at that heart rate) keeps its **band** and says the pace has none |
| **Race predictions** | Riegel `T₂ = T₁·(D₂/D₁)^1.06`; the same VDOT the ladder is built on, for the equivalents (`exploration/pace-engine.md` §2.1–§2.4) | a race time | no value; the card names the form and the exponent it used |
| **Streaks, the Eddington number** | `streaks::streaks`, `eddington::eddington`, `eddington::days_needed_for_next` (`crates/stride-core/src/streaks.rs`, `eddington.rs`) | the days with a recorded run, the day's distance | the number is only a fact against a stated period, and the days still needed for the next one are part of the answer |
| **The year** | `year::days_of`, `year::months_of`, `year::what_changed` (`crates/stride-core/src/year.rs`) | the runs in the year | — |
| **The seven facets** | `profile::facets` (`crates/stride-core/src/profile.rs`) | the window's runs against the previous window's | — |
| **The heatmap** | `heatmap::heatmap`, `heatmap::cell_of` (`crates/stride-core/src/heatmap.rs`) | the stored route positions | — |
| **A run's fingerprint** | `fingerprint::fingerprint`, `fingerprint::running_only`, `fingerprint::measurable` (`crates/stride-core/src/fingerprint.rs`) | the run's laps and samples | each observation carries its own "no value" reason (`fingerprint.rs:200–240`) |
| **A run's sentence** | `story::story`, `story::story_with_why` (`crates/stride-core/src/story.rs`) | the run's own facts | the "why" is said only when the slowest or fastest split stands out by more than ten per cent |

**One rule over all of it.** A derived value is recomputed from the bytes on every read and
is never written down, so it cannot drift from the source it names
(`docs/contract/core-contract.md` §9, and the `purity` test: the core has no I/O, no clock
and no randomness).

---

# Part 4 — The surfaces

## 4.1 The destinations

| destination | what it answers | where |
|---|---|---|
| **Today** | what is on the table, where the athlete stands against their own usual, and last night | `/` · `frontend/src/pages/TodayPage.tsx` · `/insights/today` |
| **Runs** | the history: a calendar, filters, the list, and a FIT file brought in by hand | `/runs` · `RunsPage.tsx` · `/activities` · `/auth/garmin/fit-import` |
| **Progress** | the window: the load, the weeks, where the time went, the pace, the records | `/progress` · `progressWall.tsx` · `/insights/progress`, `/insights/load`, `/insights/rolling`, `/insights/paces`, `/insights/pacing-curve`, `/insights/stimulus` |
| **The athlete profile** | the shape: the seven facets, the months, the schedule, the habit, the goals | `/profile` · `ProfilePage.tsx` · `/insights/rolling`, `/insights/streaks`, `/insights/challenges` |
| **Body** | the vitals: sleep, HRV, resting heart rate, by date, with their sample sizes | `/body` · `BodyPage.tsx` · `/physiology/*` |

The contextual surfaces: the **run detail** (`/runs/:id`), **Compare** (`/compare/:id`),
**Plan** (`/plan`), **Settings** (`/settings`), the **year** (`/year`), the **heatmap**
(`/heatmap`) and the **film** (`/runs/:id/film`).

**Five destinations, not four.** The code ships five (`frontend/src/app/destinations.ts`,
`App.tsx`); `PLAN.md` and `README.md` still say four. D8.1 decides five, and the fix is a
documentation step in the cleanup plan (`exploration/cleanup-plan.md` §3.1, steps P1–P2).

## 4.2 The cut

Counted from the code, not estimated (`exploration/surface-inventory.md` §0–§2):

| what | today | after | the source |
|---|---|---|---|
| the Progress wall's cards | **24** | **11** — six on Progress, five on the profile | D7 |
| every visible panel, all surfaces | **104** | **63** | `surface-inventory.md` §0 |
| the run detail's panels | **13** (11 cards) | **10 cards** | `surface-inventory.md` §1 |
| the run detail's charts | three stacked, two axes, two cursors | **one** chart: pace / heart rate / altitude, one shared cursor, **time on the x-axis** | `exploration/consolidated-chart.html`, `surface-inventory.md` §3 |
| the `/pace` page | a page of its own, four blocks, five panels | **retired** into Progress panels | `surface-inventory.md` §2a #5 |
| the radar | a polygon over fixed anchors | **retired** — the seven facets stay as bars | D7, `crates/stride-core/src/profile.rs` |
| the polarised split | its own card | merged into *Where the time went*, printed **once** | D11(d) |

The cut **supersedes `exploration/task-areas-plan.md` rule 1** ("no card is lost — all
twenty-four survive"). The instruction to reduce the surface supersedes that rule:
grouping twenty-four cards under ten headings is a reorganisation and not a cut, and the
reader would still have to hold ten names to find one card. The inventory wins, and the
areas' order and headings are kept — only rule 1 is broken
(`exploration/surface-inventory.md` §4).

The **prototype** of the cut wall and the consolidated chart is
`exploration/consolidated-chart.html` ("Stride — one chart, and the Progress wall after the
cut"). It is drawn from the fields the product already has, not mocked up.

## 4.3 The six cards that must be redrawn as pictures

D11 makes this a requirement, not a nicety. Six of the eleven surviving cards are
sentence-only or tables today (`exploration/cleanup-plan.md` §2.9):

| card | today | what it becomes | source |
|---|---|---|---|
| **This week** | a number, a difference, a `<dl>`, a window line | the week strip must carry the comparison — the week against the same elapsed days of the week before | `cleanup-plan.md` §2.9 |
| **Where the time went** | two drawings of one fact (five zone shares + the roll-up) | **one bar, printed once**; the ledger's 12-week strip goes here, coloured by the band each week held, so a five-week hole in the hard band is a hole | `exploration/stimulus-ledger.html` |
| **Bests & milestones** | a `<ul>` of label · value · date | distance against time, the three records as marks | — |
| **Your shape** | the facets as a `<ul>` (the radar already retired) | **seven bars on a common scale**, each labelled, each with its movement | — |
| **The schedule** | three tables (weekday, time of day, distance buckets) | one **7 × 24 strip** — each run a mark on its weekday and hour, sized by distance | — |
| **The habit** | two sentence-only cards (streaks, Eddington) | the **E × E grid** of days, coloured where the day reached E kilometres; the streak as a row of day marks | — |

The two other centrepieces have prototypes of their own: **the ledger card** is
`exploration/stimulus-ledger.html`, and **the pace ladder** is `exploration/pace-engine.html`.
The method lens's card is `exploration/method-lens.html` — deferred by D13, but its design
stands.

---

# Part 5 — The full inventory

## 5.1 Shipped today

| feature | what it answers | where it is documented | state |
|---|---|---|---|
| the local store | where the athlete's history lives, encrypted, on this machine | `crates/stride-store/src/{store.rs,schema.rs,key.rs}`; `docs/contract/core-contract.md` §2 | shipped |
| Garmin sync | pull activities and wellness, bounded and resumable | `crates/stride-api/src/sync.rs`, `crates/stride-store/src/sync.rs`; `/sync/*`; `docs/contract/core-contract.md` §7 | shipped |
| FIT import | bring a file in by hand | `/auth/garmin/fit-import`; `crates/stride-ingest/src/fit.rs` | shipped |
| the run detail | one run's facts: headline, the sentence and the why, the route, the splits, the form, the zone table, the note | `/activities/{id}`; `frontend/src/pages/ActivityPage.tsx`; `docs/contract/api-paths.md` | shipped |
| the load | how much, and where it sits | `/insights/load`; `crates/stride-core/src/load.rs` | shipped |
| the zone time | time in each band, from the athlete's own model, with the model's source line | `/fitness/zones`; `crates/stride-core/src/zones.rs` | shipped |
| the ladder | the training paces, each with the rule it comes from | `/insights/paces`; `crates/stride-core/src/training_paces.rs` | shipped |
| the effort curve | the share of the maximum a duration holds, against the time | `/insights/pacing-curve`; `crates/stride-core/src/vo2.rs` | shipped |
| the ledger | the two bases of zone time: the app's own and the watch's own | `/insights/stimulus`; `crates/stride-core/src/stimulus.rs` | shipped |
| the profile facets | the seven facets of the athlete's shape | `/insights/rolling`; `crates/stride-core/src/profile.rs` | shipped |
| the streaks and the Eddington number | how steady the athlete is | `/insights/streaks`; `crates/stride-core/src/{streaks.rs,eddington.rs}` | shipped |
| the year | the months, the calendar of the days, and what changed | `/insights/year`; `crates/stride-core/src/year.rs` | shipped |
| the film | the run in motion: the timeline, the zone ribbon, the playhead, the route by pace | `/activities/{id}/film`; `docs/contract/api-paths.md` | shipped |
| the heatmap | where the athlete runs | `/insights/heatmap`; `crates/stride-core/src/heatmap.rs` | shipped |
| the plan engine | the plan's sessions, its sources, its adherence | `/plan/*`; `crates/stride-core/src/plan.rs` | shipped |
| the run update, with its grounding checks | the model's own reading of a run, attributed, with the payload it was written from and the checks it passed | `/activities/{id}/update`, `/update/settings`; `docs/contract/voice.md` §"The run update's own register" | shipped |
| compare | two runs read against each other | `/compare`, `/compare/candidates`; `crates/stride-core/src/statement.rs` | shipped |
| body | sleep, HRV, resting heart rate, and the patterns, with their sample sizes | `/physiology/*`; `crates/stride-core/src/{band.rs,pattern.rs}` | shipped |
| the local backup and export | write a backup, read one back, delete everything | `/settings/export`, `/settings/import`, `/settings/delete`; `crates/stride-store/src/export.rs` | shipped |

## 5.2 In flight

| feature | what it answers | the design | the thread |
|---|---|---|---|
| the **stimulus ledger** | what the recent weeks contain and what they lack | `exploration/stimulus-ledger.md` (taxonomy, rules, wording, states, the HR-lag caveat) · prototype `exploration/stimulus-ledger.html` | `thr_efgucevebk` |
| the **pace engine** | race time → VDOT → the race-pace equivalents → LT pace → the training paces; the ladder and its states — **being rebuilt on that chain now** | `exploration/pace-engine.md` · prototype `exploration/pace-engine.html` | `thr_crvvnnhwtz` |
| the **Pfitzinger spine** | the method's numbers, its run vocabulary, the 80/20 week | `exploration/pfitzinger-spine.md` | `thr_nxc9e9nxms` |
| the **surface cut** | 24 cards → 11; the one chart; the page retired; the radar retired | `exploration/surface-inventory.md` · prototype `exploration/consolidated-chart.html` | `thr_q6tcjhp6wm` |
| the **cleanup** | the ordered, gated plan for the code the new features need | `exploration/cleanup-plan.md` (25 steps, each one commit, each with its own test) | `thr_8xwzkx39xu` |
| the **bridge fix** | the phone's route table was missing eleven routes, so phone screens were silently showing their error state | `android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt` + `PhoneRoutesTest.kt` (cleanup step B1) | landed in the working tree, **uncommitted** |
| the **activity observations** | the source's own facts about an activity, kept with provenance | D10; `exploration/garmin-temperature-and-activity-fields.md` §4–§5; cleanup steps O1–O3 | `thr_8xwzkx39xu` |

## 5.3 Optional — with the condition that unlocks each

**An optional item is not a promise.** Each row below names the condition that makes it
possible; until that condition holds, the design exists and nothing ships.

| feature | what it would answer | the condition | where |
|---|---|---|---|
| the **method lens** | what the history looks like through a method's own published rule | **the design exists; the shipping is deferred.** `exploration/method-lens.md` and its prototype are the record of what was designed and why; D13 takes it out of the first drop, not even opt-in. **Its rules still govern the rest of the product**: no verdict, no target on a shared scale, absence as a date, the picture first. A reader of `method-lens.md` alone should not think the lens is next | `exploration/method-lens.md` · prototype `exploration/method-lens.html` · D12 |
| the **environment axis** | heat, humidity and altitude as an effort tax | **dropped by D6**, not deferred. The measurement that closed it: no record message in `fixtures/fit/` carries FIT field 13, and on the Garmin side the temperature key is absent from all 100 activities and all 336 split objects read | `exploration/stimulus-ledger.md` §5; `exploration/garmin-temperature-and-activity-fields.md` §3.1; D9 |
| **race predictions** | what a distance would take, from a race time | the engine exists on paper (Riegel for the form; VDOT is the engine's own and the ladder is built on it); **the surface does not**. D13 makes the pace engine a dependency of the first drop; the prediction rows are a separate decision (Riegel or Daniels below 15 km) | `exploration/pace-engine.md` §2.4, §13 Q1 |
| **effective VO₂max** | a number with a model version | never a headline. D10 keeps Garmin's `vO2MaxValue` **out** — it is a judgement, not an observation | D10; `exploration/garmin-temperature-and-activity-fields.md` §4 |
| **the athlete's own effort** | what the athlete said the session felt like | waiting on D10. `directWorkoutRpe` / `directWorkoutFeel` live on the detail endpoint, so they cost one call per activity; the research says no watch can derive this | `exploration/garmin-temperature-and-activity-fields.md` §4, §5.4 |
| **training effect** | the source's own aerobic and anaerobic effect | D10 keeps it **out of the core** (it is a judgement, and its message is a label with a verdict in it) but available as an attributed observation | D10 |
| **plan-vs-reality** | whether the training matched what the plan intended | the Foster finding — runners train harder than planned on easy days and easier than planned on hard days — means a history-only ledger cannot tell "did not happen" from "happened differently". The ledger answers against the **history** and the plan card carries the **intent**; they sit side by side and are **never merged into one number**, and the app never says whether the athlete did what they were told. A card that compares them is a later, separate piece of work | `exploration/stimulus-ledger.md` §1.4; `exploration/pace-engine.md` §9 |
| **a third method base** | a third weekly structure | D14 fixes **two** bases, one at a time; the app holds one plan, with a method identity beside `PlanSource` | `exploration/pfitzinger-spine.md` §4.3 |
| **a pace-zone model** | the ladder's zone lane, and athlete-owned pace boundaries | the zone model is heart-rate based, so the lane is empty by default. A dated pace-zone model is the twin of the dated heart-rate model | `exploration/pace-engine.md` §7, §13 Q3 |
| **the athlete's own Maffetone cap** | the Maffetone-shaped rules | D12 rule 4: a **dated athlete-typed fact**, stored beside the zone model's dated facts. Stride never computes `180 − age` — no age formula, no birthdate stored | D12; `exploration/pace-engine.md` §3.4 |
| **temperature as a run-detail fact** | one line of context on the run that has it | device-conditional, and **absent on this athlete's account**; if it ever appears it is a fact for the run's context, never an adjustment | D9; `exploration/garmin-temperature-and-activity-fields.md` §6 |

## 5.4 Out of scope, and why

| what | why not |
|---|---|
| a readiness or recovery score | banned by `docs/contract/voice.md` ("readiness score", "you are ready", "not recovered" are on the ban list); and it is **not validatable for one athlete** — there is no population to check it against (`exploration/the-wedge.md` §6) |
| machine learning of any kind | there is no population dataset; the athlete's own history is the dataset (`exploration/the-wedge.md` §6) |
| a weather source | dropped by D6 — "running too hot or too cold" means exertion and load, not weather. The API carries no humidity, no wind, no "feels like" at all |
| social features | the product is one athlete's own history. Every comparison is against the athlete's own history (`README.md`) |
| a hosted account | the store is local and encrypted; there is no account to sign in to (`README.md`) |
| a subscription | there is no service to charge for |
| the 5K and 8K plan programmes | cut by D15 — the first drop ships **the 10K and the half marathon** only. The method's *rules* still come from the whole book; only the plan templates are cut |
| an adherence score or percentage | the contract drops `adherence_score`, `overall_pct` and `adherence_pct`; adherence is a count — "7 of 8 sessions run" — never a percentage (`docs/contract/core-contract.md` §3) |

---

# Part 6 — The Beta brief

## 6.1 What the Beta is

The Beta is a **separate, minimal application** that carries the product's ideas — the plan
the athlete adopted, the ledger's absence rows, the training paces, the load — in a small
screen set, reading the same local data, offline, with no account and no new dependency.
It is not another twenty-card wall: five screens, each one question, each one a picture
with a caption.

**Where it lives.** `desktop/pages/beta.html` — its own entry point, served by the existing
rig at `/beta` by **one line** in `desktop/src/server.rs`, beside the rig's other own pages
(`/rig` from `desktop/pages/tools.html`, `/connect` from `desktop/pages/connect.html`). Its
stylesheet is the app's own generated `frontend/src/tokens.css`, served unchanged, so the
Beta has no colour of its own; its script is inline in the page; it needs no build step and
no npm dependency. It is a second entry point, not a second crate:
`scripts/check-deps.sh` asserts the workspace has **exactly eight members**, so a ninth
crate fails the gate, and a second binary would mean a second process opening the same
SQLCipher file with a second copy of the key.

**How it reads the data: through the existing API, read-only.** The Beta calls the same
`/api/v1/...` routes the app calls, over loopback, offline. Two lines of justification:
the route table is frozen and a route change must travel with a `PLAN.md` edit
(`docs/contract/api-paths.md`), so a Beta that carried its own route set would be a
contract change the Beta is not allowed to make; and the contract's rule is **one owner for
the sentences and the rounding** (`docs/contract/core-contract.md` §8), so a read-only
reader of its own would be a second implementation of the store's semantics — the disease
the whole contract is written against.

**Two routes the Beta needed, and both have landed:**

* `/insights/stimulus` — **the ledger**. Planned as cleanup step L1
  (`exploration/cleanup-plan.md`): the route, the core module
  `crates/stride-core/src/stimulus.rs`, the route table and a **`PLAN.md` edit**, the
  bridge's table, and the panel. The path is the one `exploration/the-wedge.md` §5 names.
  It landed with the fourth insight route, and the ledger's card draws its two blocks.
* `/insights/paces` — **the ladder**. It landed beside it, with
  `/insights/pacing-curve` and `/insights/threshold`, and the pace card draws the
  ladder with the curve behind it.

If neither has landed when the Beta is built, the Beta's two centrepieces ship as
**no-value states with their reasons named** — which is the honest answer, and is the same
answer the main app gives today.

**How it can be tried.** The Beta is served by the existing desktop rig at `/beta`, and
the rig already answers a trusted address inside `STRIDE_TRUSTED_NETWORKS`
(`desktop/src/trust.rs`). So the Beta is reachable from **a phone browser over the
Tailscale mesh**, which is how it can be used and judged **without shipping in the
Android app** — no Kotlin, no bridge change, no release. The Android route (the FFI seam
and the bridge's table) is a separate piece of work, and it is not the Beta's.

## 6.2 The screens

### Screen 1 — **This week**: what did the athlete adopt, and what does the week hold?

* **Question.** What the plan says today and this week, and what the athlete chose to
  follow.
* **Picture.** A seven-day strip: one mark per session, coloured done / planned / not done,
  the day's session held; the plan's pacing beside the session's distance where the plan
  carries one.
* **Caption.** The plan's source (`typed` · `prescribed` · `inferred baseline`), the method
  it is following, and the window it covers. The plan's banner names the base; the app
  recommends nothing.
* **Vocabulary.** The session kinds are the **method's own run types** — endurance, general
  aerobic, recovery, progression long run, lactate threshold, VO₂max, speed, tune-up race
  (`exploration/pfitzinger-spine.md` Part 1 and Part 3). They belong to the plan, and the
  plan is where they live.
* **States.** No plan adopted → the strip is empty and the caption names the absence. A plan
  with no race time → the sessions answer with distance and kind, and their pace rows say
  what is missing. A plan whose paces were derived from a race that has since changed →
  the caption names the race time and the day it was last changed. A paused plan → the
  pause line, in the app's own voice.
* **What the Beta must not claim.** The spine's schedules were read cell by cell for the
  **half marathon 46–63 mi/wk** and the **8K–10K 30–42 mi/wk** tiers only
  (`exploration/pfitzinger-spine.md` §2.3, §6). **No other tier's weekly skeleton may be
  presented as verified**, and the **10K programme's schedule still has to be read** before
  its template is authoritative. Where a tier's schedule is not read, the plan says what it
  is — the method's rules, not a schedule read from the book.

### Screen 2 — **Where the time went** *(centrepiece — the ledger)*

* **Question.** What do the recent weeks contain, and what do they lack?
* **Picture.** The **12-week strip, coloured by the band each week held** — a five-week hole
  in the hard band is a hole before any sentence is read — and beneath it the **five zone
  shares as one bar, printed once**, with the roll-up read off that bar rather than printed
  beside it.
* **Caption.** Three kind rows (time, runs, the last date, the change against the period
  before); two value rows (the longest run; the repeated work); the coverage lines, with the
  rows they qualify; the heart-rate-lag line beside the absence row; the corroboration line
  where the watch's own workout steps exist.
* **States.** Empty window → "no run recorded in the window", no rows. One run → the kinds
  it carries answer; a kind it does not carry shows a **measured zero** with no date. No
  zone model → the intensity block has no value and names the reason; the shape block still
  answers. No heart-rate samples → same shape, different reason. No workout steps → the
  repeated-work row has no value and says so. Partial coverage → the coverage line names
  it and the change against the period before is **not** reported.
* **History and plan, side by side and never merged.** The ledger's rows answer against the
  athlete's **own history**. The **plan card** (Screen 1) carries the athlete's **intent**
  — the adopted base, its sessions, its pacing. The two sit side by side and are never
  merged into one number, and the app never says whether the athlete did what they were
  told.
* **Prototype.** `exploration/stimulus-ledger.html`. The rules and the exact wording:
  `exploration/stimulus-ledger.md` §1–§4.

### Screen 3 — **The paces** *(centrepiece — the ladder)*

* **Question.** At what paces does the athlete train, and from what race time?
* **The chain.** **race time (typed, dated) → VDOT → the race-pace equivalents → LT pace →
  the rungs.** VDOT is the engine's own; the spine is the rulebook. **Our chain replaces
  the book's lookup tables** (Appendix B's equivalent race times, Appendix C's workout
  paces) — we compute the same quantities from the athlete's own race time — and **the
  book's rules are quoted and cited.** The anchors are the **goal race pace** and **LT
  pace**; **marathon pace may appear as one more rung, never as the anchor**, and the
  MP-offset placeholders (`MP+2:00`, `MP−0:45`, `MP+1:30`, `MP+3:00`) are deleted — they
  were placeholders and they are wrong for the method.
* **The rungs**, from the spine's table:
  * **LT pace** — the pace for a ~15K (10-mile) race, from the same VDOT; or 10K pace
    + 10–15 s/mi for a shorter-distance athlete (the spine's rule, cited);
  * **the long run, two ends** — the start at 10K pace × 1.20–1.33; the finish at
    15K–HM pace × 1.17–1.29, or **LT pace for the last ~3 miles of a progression long
    run**;
  * **recovery** — 15K–HM pace + 120 s/mi, with the band below 76 % maxHR;
  * **the easy day — no pace at all.** Only the band: 70–81 % maxHR / 62–75 % HRR. The
    book gives the general aerobic run **no pace rule**, so a pace for it would be the
    app's inference and not the method's.
* **Picture.** The ladder: a horizontal pace scale from 3:30 /km to 8:00 /km, each rung a
  marker, the **anchor plumb-lines** (the goal race pace and LT pace) that every rung is
  read against, and the long run as **one bar with two ends** — where it starts and where
  it finishes. Azure for the anchors, amber for the derived rungs, no green and no colour
  that marks a problem.
* **Band and pace, kept apart.** Every rung carries **a heart-rate band and a pace**, and
  the card keeps them apart: the band is the **method's**; the pace is the **athlete's own
  pace-at-heart-rate**, or the method's percentage rule. Where the athlete has **no
  history at that heart rate**, the pace is a **no value with a reason** — and the band
  still stands.
* **Caption.** The race it came from — distance, time, the day of the race, the day it was
  entered, the day it was last changed — and, once it is old enough, the race's age in
  days. Each rung names the rule behind it, with its citation.
* **States.** No race entered → the ladder's rails and axis with **no rungs and no anchor
  line**: the hole is drawn, and the caption names the fields the athlete can type. One
  race → the ladder with its rungs. A race replaced → the ladder redrawn, and the caption
  names the race time that produced the numbers on screen. A race old enough to matter →
  one line of fact, no colour, no instruction. A rung with no value → an **empty slot** on
  the scale, not a zero, and the caption names what the rule needs. A rung whose band
  stands but whose pace does not → the band is drawn, the pace is an empty slot with its
  reason.
* **Vocabulary.** The ladder shows the **paces subset** of the method's run types — general
  aerobic, recovery, lactate threshold, the long run's two ends, and the goal race pace.
  The other run types (progression long run, VO₂max, speed, tune-up race) live in the
  **plan's session kinds**, which is where they belong.
* **Prototype.** `exploration/pace-engine.html` — the ladder's geometry and its states; the
  values there are the engine's placeholders and are being rebuilt on this chain. The
  rules: `exploration/pace-engine.md` §2, §4, §6–§7, and `exploration/pfitzinger-spine.md`
  Part 1, Part 5.

### Screen 4 — **The load**: how much, and where does it sit?

* **Question.** How much has the training been, and is the acute load above the chronic?
* **Picture.** The fitness–fatigue series with the form area, one shared axis.
* **Caption.** The model's own scenario and its assumptions: the *k*, the maximum heart
  rate and the day it was read, the two time constants, and what the ratio is expected to do
  with a rest day. Those sentences are the ones a chart cannot carry.
* **States.** No zone model → the load has no value and says so (the maximum heart rate
  comes from the athlete's own model). A window with no runs → no value. A ratio inside the
  band → the band is named, not coloured as a problem.

### Screen 5 — **The runs**: what did the athlete run, and when?

* **Question.** The history, at a glance, and one run's own facts.
* **Picture.** The list as marks on a calendar; one run as its headline numbers and the
  **ledger's own line for that run** — which band its time fell in, how long, and the last
  date that band occurred.
* **Caption.** The run's distance, time, pace and heart rate, the date, and the zone
  model's source line.
* **States.** No runs → the calendar is empty and the caption says no run is recorded. A run
  with no file → the run shows what it has and says it has no file. A run dated before the
  first stored zone model → that run has no zone time, and the run detail names the reason.
* **One answer, not two.** Until D3's stage 1 lands, the ledger and the run detail measure
  a run with different zone models. The Beta shows **the ledger's** answer — the run's band
  time and the last date that band occurred — and never puts both on one screen.
* **This screen is the one that may be folded** into Screen 2 if the Beta is to be four
  screens. The four screens are the four the wedge is about.

## 6.3 What it must never do

| the constraint | the source |
|---|---|
| **No verdict.** No readiness, no score, no grade, no traffic light, no sentence about the athlete's condition | `docs/contract/voice.md`; the ban list in `crates/stride-core/src/strings.rs` |
| **No advice.** No imperative addressed to the athlete; the plan's sessions are listed unranked and never linked to a vital | `docs/contract/voice.md` — an `option` string may not reference a vital |
| **No ranking.** Every comparison is against the athlete's own history | `README.md` |
| **No banned word.** `should` `must` `take the day` `rest day` `rest up` `rest today` `go easy` `missed` `bad` `poor` `fail` `behind` `good` `keep it easy` `ready to` `recommended` `better to` `need to` `red` `readiness score` `not recovered` `you are ready` `you are not ready` | `crates/stride-core/src/strings.rs` (`BANNED_PHRASES`, 23 entries), word-boundary matched |
| **No red, and no colour that marks a problem.** Azure and amber only | `docs/contract/voice.md`; D8.3 |
| **No account, no network of its own.** The Beta reads the local store through the loopback API; it never calls Garmin and never reaches the internet | `README.md`; `docs/contract/core-contract.md` §7 |
| **No new dependency.** No crate, no `Cargo.toml` entry, no `package.json` entry | `scripts/check-deps.sh` (eight members, exactly); `scripts/approved-deps.txt` |
| **No stored derived value.** Everything recomputed on read; the Beta writes nothing to the store | `docs/contract/core-contract.md` §9 |
| **No implied absence.** A missing value is `null` with a reason; never zero, never band 1, never "0 m of repeated work" | `docs/contract/core-contract.md` §2; `exploration/stimulus-ledger.md` §4 |
| **No route of its own.** Only the two named above, each travelling with a `PLAN.md` edit | `docs/contract/api-paths.md`; `exploration/cleanup-plan.md` §3.3 |
| **No written sentence of its own.** Every sentence comes out of the core's catalogue, rendered by the core | `docs/contract/core-contract.md` §8; `crates/stride-core/src/strings.rs` |

## 6.4 The data it reads

| what it needs | where it is today | what does not exist yet |
|---|---|---|
| the window's runs, with their dates, distances and moving times | `activities` (`crates/stride-store/src/schema.rs:44`) | — |
| each run's decoded samples, laps and route | the FIT original, `activities.fit_blob`, decoded on read (`crates/stride-ingest/src/fit.rs`) | — |
| the zone model **in force on each run's day** | `hr_zone_models` with `read_at` (`schema.rs:160`); `repos/zones.rs:93` | **D3 stage 1** is not landed: the run detail still measures with the *current* model, so the ledger and the run detail can disagree for runs dated before the first stored model |
| the watch's own workout steps | — | **the ingest does not read them.** `fit.rs` reads the lap message but not field 24 (`wkt_step_name`), and does not read the `workout_step` message (27) at all. No fixture in `fixtures/fit/` carries either, so the repeated-work row cannot be built or tested from the corpus — it needs a real file from a structured workout |
| the athlete's typed race time | — | `race_observations` is a sketch (`exploration/pace-engine.md` §3.2) |
| the method's own pacing rules | the numbers are in `exploration/pfitzinger-spine.md` Part 1 and Part 5 | no code: `crates/stride-core/src/pace_engine.rs` does not exist, the pace engine's MP offsets are **placeholders** and are being **deleted**, and the ladder is being rebuilt on the chain **race time → VDOT → the race-pace equivalents → LT pace → the rungs** (§6.2, Screen 3) |
| the plan's sessions, their kind, their source and their pacing | `plan_blocks`, `plan_sessions` (`schema.rs:89`, `schema.rs:101`) | the store has `pace_low_s_km` and `pace_high_s_km`, but nothing fills them and `PlanSession` cannot hold them (`exploration/pace-engine.md` §8); and the **10K programme's schedule has not been read cell by cell** — only the half marathon 46–63 and the 8K–10K 30–42 have (§6.2, Screen 1) |
| the ledger's rows | — | `crates/stride-core/src/stimulus.rs` does not exist, and neither does the route `/insights/stimulus` (cleanup step L1) |
| the ladder's answer | — | **no route exists** — `/insights/paces` is planned nowhere (§6.1) |
| the load model | `crates/stride-core/src/load.rs` | D3 **stage 2** — the load model still reads the *current* maximum heart rate, not the dated one |
| the wellness observations | `wellness_daily`, `wellness_observations` | — |
| the source's own effort and training effect | — | D10's `activity_observations` table; and the effort fields live on the detail endpoint, one call per activity |
| a temperature | — | absent on this account (D9). If it ever populates it is a fact for the run's context, never an adjustment |

## 6.5 The acceptance criteria

A checklist an agent can tick:

1. **With no race entered, the pace ladder shows no value and says what is missing** — the
   empty ladder is drawn (rails, axis, no rungs, no anchor line), and the caption names the
   fields the athlete can type: distance, time, the day of the race, the name optional.
2. **With no zone model, the ledger's intensity rows have no value and name the reason** —
   "zones unavailable — no zone model has been read"; no time, no share, no run count, no
   last date, no distribution line. The shape rows — the longest run, the repeated work —
   still answer, because they do not need a zone model.
3. **The absence row is a date, never a list.** "the last run in Z4–Z5 was on 20 Oct 2026 —
   35 days before the end of the window". The words "missed", "bad" and "poor" appear
   nowhere, and no row counts what the athlete did not do.
4. **Every sentence in the Beta survives the ban list**, checked with the core's own
   word-boundary rule (`strings::lint`, `contains_phrase`) over the rendered strings, and the
   catalogue is in step with `docs/contract/voice.md`.
5. **The app works offline with the network off** — no request leaves the loopback; the
   Garmin key is never needed for any of the five screens.
6. **No new dependency appears in `Cargo.toml` or `package.json`.** `scripts/check-deps.sh`
   passes, and the workspace still has exactly eight members.
7. **Nothing is written.** The Beta issues no write to the store: the database is opened
   through the same read paths the app uses, and a test asserts that the store's mtime and
   row count are unchanged after a full pass through the five screens.
8. **The picture comes first.** Every card is readable with its caption hidden, and every
   number in the picture is also present in text beside it (the accessible fallback).
9. **Every answer names its period, its model and its source.** A number with no period is
   not shipped.
10. **A missing measurement is `null` with a reason, never zero.** No `0`, no `—`, no
    `0 km`, no "0 m of repeated work".
11. **No colour marks a problem.** Azure and amber only; no green, no red.
12. **The Beta adds no route except the two named**, and each of those travels with a
    `PLAN.md` edit and a bridge-table entry in the same pull request.
13. **The ladder is on the settled chain.** The anchors are the **goal race pace** and **LT
    pace**; marathon pace is at most one more rung; no MP-offset placeholder appears
    anywhere; the rungs carry the **method's heart-rate bands**, and where the athlete has
    no history at that heart rate the **pace** is a no value with a reason while the band
    stands.
14. **One answer for a run's zone time — the ledger's.** The Beta never shows the ledger's
    band time and the run detail's band time on one screen.
15. **No weekly skeleton is presented as verified for any tier other than the half marathon
    46–63 and the 8K–10K 30–42**, and the 10K programme's schedule is named as not yet
    read.

## 6.6 How to build it in this repository

**The dependency DAG.** It is fixed, and CI asserts it
(`scripts/check-deps.sh`, from `cargo metadata`, not from a text parse):

```
core ← ingest   core ← store   api → {core, ingest, store}
desktop → {api, store}
stride-ffi → {api, store}      stride-testkit is dev-only, everywhere
```

The script also asserts: the workspace has **exactly eight members**; a build-dependency
is a real edge; a dev-dependency may only add what the crate may already depend on (or the
testkit); `stride-testkit` is never a hard dependency; and **no crate may override the
workspace lint table**. A cleanup that adds a crate fails this gate; adding a *module* to
an existing crate does not.

**The clippy lints.** `[workspace.lints.clippy]`: `unwrap_used`, `expect_used`, `panic`,
`todo`, `unimplemented` — all **denied**, and CI runs `-D warnings`. Test code opts out with
a module-level `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` — the
one sanctioned place (`CONTRIBUTING.md`).

**The frozen contract.** Four things are frozen: the **route table**
(`docs/contract/api-paths.md`, held to the router by `route_parity` in both directions), the
**schema** (`crates/stride-store/src/schema.rs`; the one approved addition is D10's
`activity_observations`), the **strings** (the catalogue and its rules), and the **corpus**
(`fixtures/fit/`, SHA-256 frozen by `scripts/check-corpus.sh`). Changing a route is a plan
change and **must travel with a `PLAN.md` edit in the same pull request**. The Android
bridge's route table may only **grow**, and it travels with the route table.

**The string catalogue.** Two files, both CI-checked: `crates/stride-core/strings.json` (the
export, carrying the ban list, the closed direction set and the minimum pattern sample) and
`frontend/src/i18n/catalogue.ts` (the frontend's). The core's strings are **rendered by the
core** — the API sends the sentence, not a template with holes in it
(`docs/contract/core-contract.md` §8). A second implementation of the wording is the bug the
contract is built to prevent. The Beta therefore adds its strings to the **core's**
catalogue and asks the core for the rendered sentence; it writes no user-facing text of its
own. Note the frontend's lint also fails an inline user-facing string in any `.tsx`
(`checkInlineText`) — the Beta's equivalent rule is that its HTML holds no sentence, only
values the core rendered.

**Privacy and secrets.** `scripts/privacy-rules.sh` (shared by `scan-privacy.sh` and
`scan-pr.sh`): the only coordinates allowed are within 1° of the synthetic anchor
(−35.0, −140.0); a labelled coordinate, an unlabelled pair, a degree conversion, a Garmin
profile field with a personal value, a non-synthetic serial, an email, a phone number and a
free-text note in `fixtures/`, `crates/*/tests/`, `desktop/tests/`, `*.json`, `*.csv`,
`*.toml` are violations. `scripts/scan-secrets.sh` and `scripts/scan-history.sh` cover the
tree and every commit on every ref — a leak that was added and later removed is exactly what
the tree scanners cannot see.

**No production data, no Garmin writes, no publish.** The Beta calls no Garmin endpoint, ever.
It never writes to the store. It is not published: no release, no tag, no push.

**The synthetic-fixture rule.** Any fixture the Beta adds is synthetic. A new FIT file goes
in a **new directory**, not `fixtures/fit/` (the corpus's SHA-256 list is frozen); its
coordinates sit within 1° of the synthetic anchor; and its field names must not look like a
note, because the privacy scan reads `*.json` and `fixtures/` for `note|comment|remark`
keys.

**The commit gates.** `bash scripts/gate.sh` is the authoritative list, and it runs in CI and
by hand: `cargo fmt --check` · `cargo clippy --workspace --all-targets -- -D warnings` ·
`cargo test --workspace` (which includes `route_parity`, `frontend_shapes`,
`fixture_parity`, `purity`, and `schema_contract` once D10 lands) ·
`scripts/check-deps.sh` · `scripts/check-corpus.sh` ·
`scripts/check-contract-greps.sh` (no pairing/QR routes, no string-built SQL, no
interpolated identifier, no `unsafe` outside the FFI crate) · `scripts/scan-secrets.sh` ·
`scripts/scan-privacy.sh` · the scanner self-tests · `scripts/scan-history.sh` ·
`npm --prefix frontend run tokens:check` · `lint:voice` · `typecheck` · `test:unit` ·
`test:viewport` (six widths) · `scripts/scan-pr-self-test.sh` — and on a pull request,
`scripts/scan-pr.sh origin/master HEAD`.

`CONTRIBUTING.md`'s own rules are gates in everything but name: **one slice or one coherent
change per pull request, and never mix a refactor with a behaviour change**; if it changes a
number, a string, a route shape or the schema, **say so explicitly**; **never edit an
expectation to make a test pass**; **never delete a fixture**.

**The three rig-side changes the Beta needs**, all small and all in the existing crate:

1. one arm in `desktop/src/server.rs` — `("GET", "/beta") => Response::html(200,
   include_str!("../pages/beta.html"))`, the pattern the rig already uses for `/connect`;
2. a `Response::css` constructor in `desktop/src/http.rs` (the rig has `html` and `json`
   today) so the Beta's stylesheet can be served — and it is served **unchanged**: the
   generated `frontend/src/tokens.css`, which the token gate already holds in step with
   `frontend/tokens.ts`;
3. the token gate's scan (`frontend/scripts/check-tokens.mjs`) widened to
   `desktop/pages/beta.html`, so a raw colour in the Beta's page is caught the way it is in
   the app's. A one-line change to the scan's file list, and no new dependency.

The Beta's page holds **no hex and no user-facing sentence**: the values it shows are the
core's rendered text, and the strings it needs are added to the core's catalogue, where the
ban list and the closed direction set are already enforced.

## 6.7 A map of the sources

| source | what it settles | what it feeds the Beta |
|---|---|---|
| `exploration/the-wedge.md` | the choice: the negative space, the three pillars, the vocabulary trap | the Beta's premise; Screen 2 |
| `exploration/decisions.md` | D1–D15, each naming the thread it binds | every rule below |
| `exploration/methods-and-products.md` | what runners ask and complain about; how the methods map to computable data; the fact-only lens (now deferred) | the wording; Screen 2's HR-lag caveat; the method-lens deferral |
| `exploration/surface-inventory.md` | every panel, counted, with its merge target; the cut list; the consolidated chart | the Beta's size: five screens, not a wall |
| `exploration/stimulus-ledger.md` | the taxonomy, the rules, the wording, the states, the HR-lag caveat, the repeated-work row | **Screen 2**, whole |
| `exploration/stimulus-ledger.html` | the ledger card, drawn | Screen 2's picture |
| `exploration/pace-engine.md` | race time → VDOT → the race-pace equivalents → LT pace → the training paces; the ladder; the states | **Screen 3**, whole |
| `exploration/pace-engine.html` | the ladder, drawn | Screen 3's picture |
| `exploration/pfitzinger-spine.md` | Pfitzinger's numbers, the run vocabulary, the 80/20 week, the mapping to the stimulus kinds | Screen 3's rungs, bands and rules; Screen 1's plan vocabulary. **Complete as read** (Parts 1–7); the schedules were read cell by cell for the **half marathon 46–63** and the **8K–10K 30–42** only, and §6's unverified list is open |
| `exploration/method-lens.md` · `.html` | the deferred lens: the catalogue, the rules, the refused methods | nothing ships; the rules govern the rest |
| `exploration/garmin-temperature-and-activity-fields.md` | what the Garmin API carries that Stride discards; the temperature finding | §5.3, §5.4 of this brief; the activity observations (D10) |
| `exploration/cleanup-plan.md` | the ordered, gated plan for the code the features need | §6.4 (what exists and what does not); §6.6 (the gates) |
| `exploration/consolidated-chart.html` | the one chart, and the Progress wall after the cut | Screen 2's card layout; the run detail's chart, if the Beta grows one |
| `docs/contract/voice.md` | the seven string types, the closed direction set, the ban list | §6.3, §6.5, §6.6 |
| `docs/contract/core-contract.md` | units, rounding, no value, the direction band, the human rendering, one owner for the sentences | §6.3, §6.4, §6.6 |
| `docs/contract/api-paths.md` | the frozen route table | §6.1 (the Beta's two routes); §6.6 |
| `PLAN.md` | the plan of record | §6.6 (a route change travels with it) |
| `README.md` | what the product is today, in the user's words | Part 1; §6.6 |
| the code itself | `crates/stride-core/src/*`, `crates/stride-store/src/*`, `crates/stride-api/src/*`, `desktop/src/*`, `frontend/src/*` | Part 3; §6.1; §6.4 |

---

# Part 7 — The open questions

## 7.1 Decided, and not open any more

| the question | the answer | where |
|---|---|---|
| how many destinations | **five** — Today, Runs, Progress, the athlete profile, Body | D8.1 (the code already ships five; `PLAN.md` and `README.md` still say four, and the fix is cleanup steps P1–P2) |
| the zone bands in the chart as well as the table | **yes** — the athlete's own configuration drawn on their own data; the table keeps the numbers, the *not moving* bucket and the model's source line | D8.2 |
| the altitude scale | **its own scale, default off**, with its own metres label, never sharing the effort band's scale; the default is pace + heart rate | D8.3 |
| the plan programmes | **the 10K and the half marathon only**; the 5K and 8K are cut | D15 |
| the window | the last N calendar weeks, from the start of the earliest one, through today; the comparison period is the same span ending seven days earlier | D1, D2 |
| the zone model | **staged**: stage 1 moves the zone-time answers to the model in force on the run's day; stage 2 moves the load model, later and alone; every answer names the model and the day it was read | D3 |
| repeated work | **path 1 only** — the watch's own `interval` step names; the lap-shape heuristic is rejected | D4 |
| the shares | each names its denominator; shares over different denominators are not added together | D5 |
| the environment axis | **dropped**, not deferred | D6 |
| the surface cut | 24 wall cards → 11; the run detail's three charts → one; the `/pace` page retired; the radar retired | D7 |
| Garmin's temperature | the API has it; this account does not; no work is needed | D9 |
| what to keep from the Garmin activity response | training effect, its time in zone, the athlete's own effort, respiration, steps — with provenance; Garmin's estimates stay out | D10 |
| the picture or the sentence | **the picture first**; absence is drawn, not described | D11 |
| the first drop | **Pfitzinger's plan, one base at a time**; no lens ships | D13 |
| the audience | **the intermediate runner**; entry level is served by Strava and Garmin | D14 |
| the pace chain | **race time (typed, dated) → VDOT → the race-pace equivalents → LT pace → the rungs.** Our chain replaces the book's lookup tables (Appendix B, Appendix C); the book's **rules** are quoted and cited. The anchors are the **goal race pace** and **LT pace** — marathon pace is at most one more rung — and the MP-offset placeholders are deleted | the parent's resolution, 24 November 2026; the pace engine thread is rebuilding the ladder on it |
| history and plan | **two sources, never merged**: the ledger's rows answer against the athlete's own history; the plan card carries the intent; the app never says whether the athlete did what they were told | D13 (which supersedes `exploration/the-wedge.md` §7.4) |
| the surface cut against the areas plan | the cut **supersedes rule 1** ("no card is lost"); the areas' order and headings are kept, rule 1 is broken | `exploration/surface-inventory.md` §4 |

## 7.2 Waiting on the user

| what is waiting | what it blocks |
|---|---|
| the **green light for the surface cut** | the cut is a plan change and a contract change together: cleanup steps P1 (the `PLAN.md` amendment), X1 (the route shapes), W2 and Z1 travel in the same pull request. Nothing in the cut can land before that |
| **W — the window migration** | cleanup steps W1 (the window type) and W2 (the contract's prose): the rolling route still computes a rolling N-day window while the Progress totals and the pace page compute the calendar-week window. Until W lands, two cards on the same screen describe different spans — and the ledger would be a third |
| **Z — the zone model's staged migration** | stage 1 is what makes the ledger and the run detail agree. Until it lands, a run dated before the first stored model has no zone time in the ledger and a band in the run detail |
| a **structured workout recorded on the watch** | the repeated-work row. It is *no value* until a real file with `wkt_step_name` exists — the correct answer, but the row has nothing to say |
| the **plan programmes' start dates and levels** | the 10K and half-marathon templates are cut to D15's two distances; which level, and from which date, is the athlete's |

## 7.3 Deliberately open

* **Which method base the athlete picks.** D14 fixes two bases and one plan at a time; the
  app holds one plan, with a method identity beside `PlanSource`, and the athlete switches.
  Which one the athlete adopts is not the app's to choose.
* **Whether the Beta reads through the API or directly.** **Decided here: through the
  existing API** (§6.1), because the route table is frozen and the contract's rule is one
  owner for the sentences and the rounding. A second reader is defensible — it would avoid
  the two route additions — and it is the parent's call to reverse.
* **The method lens's form** — where the card lives, and whether it is ever offered.
  Deferred by D13; the design stands.
* **The zone lane on the ladder** — empty until a dated pace-zone model exists
  (`exploration/pace-engine.md` §13 Q3).
* **Riegel or Daniels below 15 km** for the prediction rows (§13 Q1); **how old is "old"** for
  the race (§13 Q2); **whether the anchor moves** when a newer race is entered (§13 Q6).
* **The ledger's density on a narrow screen** — the full card, or the change lines folded
  away (`exploration/stimulus-ledger.md` §8 Q4).
* **The comparison's overlap** — the window and its comparison share 72 of 79 days, so the
  change reads as "the week that came in against the week that fell out"
  (`exploration/stimulus-ledger.md` §8 Q2).

---

# How the sources were reconciled

*The sources disagreed in eight places. These are the resolutions, each with the evidence
behind it, so that a later reader can check them and does not have to rediscover them.
The resolutions belong to the parent's decision of 24 November 2026; the evidence is
quoted so they can be audited.*

1. **The pace chain — one chain, settled.** `exploration/pace-engine.md` built the ladder
   on **VDOT** and on marathon-pace offsets (`MP+2:00`, `MP−0:45`, `MP+1:30`, `MP+3:00`),
   and its own §12 says the offsets are placeholders and are "not to be quoted as the
   method's". `exploration/pfitzinger-spine.md` §5.4 says the book does **not** use VDOT,
   and that the book's own chain is *race time → Appendix B equivalents → Appendix C
   workout paces*. **Resolution: race time (typed, dated) → VDOT (the engine's own) → the
   race-pace equivalents (VDOT/Riegel) → LT pace (the spine's rule) → the rungs (the
   spine's percentages).** VDOT is the engine; the spine is the rulebook. Our chain
   **replaces the book's lookup tables** — we compute the same quantities from the
   athlete's own race time — and **the book's rules are quoted and cited**. The **anchors
   are the goal race pace and LT pace**; marathon pace may appear as one more rung, never
   as the anchor; and the MP-offset placeholders are **deleted**. The rung values come
   from the spine: the long run's start at 10K pace × 1.20–1.33; its finish at 15K–HM
   pace × 1.17–1.29, or LT pace for the last ~3 miles of a progression long run; recovery
   at 15K–HM pace + 120 s/mi with the band below 76 % maxHR; the easy day with **no pace
   at all**, only the band (70–81 % maxHR / 62–75 % HRR) — the book gives the general
   aerobic run no pace rule, so a pace for it would be the app's inference. Every rung
   carries **a heart-rate band and a pace**, kept apart: the band is the method's, the pace
   is the athlete's own pace-at-heart-rate or the method's percentage rule, and where the
   athlete has no history at that heart rate the **pace** is a no value with a reason while
   the band stands. Written into §6.2, Screen 3 and §3.2.

2. **The vocabulary is the spine's.** The pace engine's ladder named its rungs "Easy,
   Threshold, Long run, Recovery"; the spine names **eight** run types — endurance, general
   aerobic, recovery, progression long run, lactate threshold, VO₂max, speed, tune-up race
   — and its Part 4 maps them to the stimulus kinds, with the column that says which the
   watch can verify. **Resolution: the ladder shows the paces subset** (general aerobic,
   recovery, lactate threshold, the long run's two ends, the goal race pace); **the other
   run types live in the plan's session kinds**, which is where they belong. Written into
   §6.2, Screens 1 and 3.

3. **History and plan — two sources, never merged.** `exploration/the-wedge.md` §7.4 said
   "history only — the ledger anchors on the athlete's own recorded history in v1; the
   typed/prescribed plan is not an anchor yet"; **D13 reverses that** ("the plan anchor
   returns"). The wedge predates the decision, and the parent is correcting that file too.
   **Resolution: the ledger's rows answer against the athlete's own history; the plan card
   carries the athlete's intent** — the adopted base, its sessions, its pacing; **the two
   sit side by side and are never merged into one number**, and the app never says whether
   the athlete did what they were told. Written into §6.2 (Screens 1 and 2), §5.3 and §7.1.

4. **The surface cut supersedes `task-areas-plan.md` rule 1.** The areas plan's rule 1
   says "no card is lost — all twenty-four survive"; `exploration/surface-inventory.md` §4
   breaks it deliberately, because grouping twenty-four cards under ten headings is a
   reorganisation and not a cut. **Resolution: the instruction to reduce the surface
   supersedes rule 1; the inventory wins**, and the areas' order and headings are kept.
   Written into §4.2 and §7.1.

5. **The Beta shows the ledger's answer, not two.** Until D3's stage 1 lands, the ledger
   measures a run with the zone model in force on its day and the run detail with the
   current one (`exploration/stimulus-ledger.md` §6.2). Both name their model, so the
   disagreement is visible rather than silent. **Resolution: the Beta shows the ledger's
   answer and never puts both on one screen.** Written into §6.2, Screen 5, and the
   acceptance criteria.

6. **The method lens — design done, shipping deferred.** `exploration/decisions.md` D13
   takes the lens out of the first drop; `exploration/method-lens.md` §0 presents itself
   as deciding the catalogue, the states and the caveats, so a reader of that file alone
   would think the lens is next. **Resolution: the design exists and the shipping is
   deferred, and the lens's rules still govern the rest of the product** — no verdict, no
   target on a shared scale, absence as a date, picture first. Written into §5.3.

7. **What the Beta must not claim.** The spine's schedules were read cell by cell for the
   **half marathon 46–63 mi/wk** and the **8K–10K 30–42 mi/wk** tiers only. **Resolution:
   the Beta must not present a verified weekly skeleton for any other tier**, and the
   **10K programme's schedule still has to be read** before its template is authoritative.
   Where a tier's schedule is not read, the plan says what it is — the method's rules,
   not a schedule read from the book. Written into §6.2, Screen 1, §6.4, §6.5 and §6.7.

8. **Two drifts left for the cleanup list.** `README.md` says "six string types in all"
   where `docs/contract/voice.md` says **seven** — the `instruction` type (the run update's
   prompt) is missing from the README's list — and the README still says "four
   destinations" where the code ships five. **Resolution: both are README fixes, and the
   string count belongs on the cleanup list's documentation step (P2)** alongside the
   destinations.

**What is still open after the reconciliation** is in §7.3 — the method lens's form, the
ladder's zone lane, Riegel or Daniels below 15 km, how old is "old" for the race, whether
the anchor moves when a newer race is entered, the ledger's density on a narrow screen,
the comparison's overlap, and whether the Beta reads through the API or directly.

**One residual drift, for the parent:** `exploration/pfitzinger-spine.md` §6 item 4 still
says that *only* the half-marathon 46–63 schedule was read cell by cell. The 8K–10K
30–42 read is not in the file yet, so the spine file and this brief do not yet agree on
that point.
