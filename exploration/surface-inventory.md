# The surface inventory — what Stride shows today, and what should survive

*24 November 2026. A prototype thread: nothing under `crates/` or `frontend/src/`
was changed. Every count below was read out of the code, not estimated.*

---

## 0. How the count was made

**The rule.** One *panel* = one visible block a reader can point at: a card, a
chart inside a card, a table inside a card, a list, a map. A card that holds a
chart **and** a table counts as two panels, because it is two things to read.
A control row with no heading of its own (a range selector, a window line) is
counted with the block it belongs to.

**The places read.**

| what | where |
|---|---|
| the surfaces | `frontend/src/pages/`, `frontend/src/components/`, `frontend/src/app/App.tsx`, `frontend/src/app/destinations.ts` |
| the wall's cards and their areas | `frontend/src/pages/taskAreas.ts` (`WALL_CARDS`, `TASK_AREAS`, `WALL_PANELS`) |
| the wall's drawing | `frontend/src/pages/progressWall.tsx` |
| the panels behind the cards | `frontend/src/pages/progressPanels.tsx`, `frontend/src/pages/loadCharts.tsx` |
| the phone's indexes | `frontend/src/pages/ProgressIndex.tsx` |
| the panel routes | `frontend/src/pages/ProgressPanelPage.tsx`, `frontend/src/pages/ProfilePage.tsx` |
| the numbers | `crates/stride-core/src/` (`load.rs`, `zones.rs`, `pace.rs`, `rolling.rs`, `fingerprint.rs`, `year.rs`, `profile.rs`, `streaks.rs`, `eddington.rs`, `heatmap.rs`) |
| the wire | `crates/stride-api/src/routes/`, `crates/stride-api/src/router.rs`, `crates/stride-api/src/dto.rs` |
| the segmentation | `crates/stride-ingest/src/segment.rs` |
| the strings | `frontend/src/i18n/catalogue.ts`, `crates/stride-core/src/strings.rs` |

**The headline.**

| | today | after the cut |
|---|---|---|
| **the Progress wall's cards** | **24** | **11** (6 on Progress, 5 on the athlete profile) |
| every visible panel, all surfaces | **104** | **63** |

The wall is the number that matters: it is the only surface where the reader
is asked to *choose* between twenty-four things. Everything else is one
question per screen.

**One contradiction to flag up front.** The brief says *Today, Runs, Progress
and Body remain the four destinations*. The shipped code has **five**
(`frontend/src/app/destinations.ts`: Today, Runs, Progress, **Profile**, Body),
the router carries `/profile` (`App.tsx`), and `taskAreas.ts` splits the
wall's areas across the two pages. This inventory keeps the code's five and
sizes the cut so the four-destination reading is one edit away — see §5.

---

## 1. The inventory

Legend — **K** keep · **M** merge (into the named target) · **R** retire.

### Today — `/` · `TodayPage.tsx` · 6 panels

| # | panel (named) | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The hero — the latest run** (name, distance, time, pace, HR, elevation, zone ribbon, route map) | the run detail's headline numbers | — | **K** | Today is a summary; the detail is the whole run. The hero is the app's front door and the only place a run appears before it is opened. |
| 2 | **Where you stand** — three rows: *This week* · *Form* · *Habit* | Form = the load card on Progress; Habit = the streaks + Eddington cards on the profile; This week = the *week* card on Progress | — | **K, deliberately** | One value (the last state) against Progress's series. A number and its trend are different questions; §4 says why this stays. |
| 3 | **What's on the table** (the plan's session, the other options) | — | — | **K** | The plan is the only source of a *choice* in the product, and it must stay unranked and unlinked to the vitals. |
| 4 | **Rest — last night** (the summary line) | the three vitals directly beneath it | → one *Last night* card | **M** | The Rest line is a count and a direction computed **from** the three vitals below it. Two blocks, one fact. |
| 5 | **The vitals** (three signals, rows) | the Body page's signal cards; the profile's *Body, last night* wall card | → the *Last night* card, then Body | **M** | Keep the readings on Today; drop the duplicate wall card (see the wall). |
| 6 | **Sleep composition** (the stages) | the Body page's sleep card | → the *Last night* card | **M** | Same data as Body, same night. One copy on Today, one on Body (any date). |

**After: 4 panels** — hero · where you stand · what's on the table · last night (Rest + three signals + sleep, one card, one link out to Body).

### Runs — `/runs` · `RunsPage.tsx` · 5 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **Browse calendar** | — | — | **K** | The range is the screen's own control. |
| 2 | **Filters** (search, from, through, scope) | — | — | **K** | — |
| 3 | **Compare tray** (two runs selected) | — | — | **K** | The only entry into Compare that does not need a run first. |
| 4 | **Import a FIT file** | Settings → Import | — | **K** | Importing a file is a Runs act. Settings keeps the *history* import and the storage settings. |
| 5 | **The run list** (grouped, with the route thumbnail) | the wall's *Most recent activities* card | — | **K** | The destination. The wall's copy of it is cut, not this. |

**After: 5 panels** — unchanged. Runs is one question and already asks it.

### Activity — the run detail — `/runs/:id` · `ActivityPage.tsx` · 11 cards, **13 panels**

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The headline numbers** (distance, time, pace, HR) | the Today hero | — | **K** | The run's facts, on the run's page. |
| 2 | **What it meant** (the core's sentence, and the extra fact) | — | — | **K** | One sentence from the core. |
| 3 | **The route** (the flyby) | the film's route | — | **K** | The spatial truth of the run. |
| 4 | **Effort → the heart-rate chart** (with the zone bands behind it) | — | → the one chart | **M** | The zone bands move out of the drawing (see below). |
| 5 | **Effort → the zone time table** (each zone, outside, **not moving**, the model's source line) | the Progress wall's *Time in each zone* and *Polarised split* cards | — | **K** | This is where the zones belong: a time, a share and a source. The chart shows a shape; the table shows the bands. |
| 6 | **Effort → the pace chart** | the Today hero's pace; the Pace page; the profile's facets | → the one chart | **M** | The second of the three stacked charts. |
| 7 | **The elevation chart** (a distance axis) | — | → the one chart | **M** | The third. Its own scale, its own treatment — but the *same* cursor. |
| 8 | **Splits** (distance, pace, time, HR per lap) | the film's splits table | — | **K**, and the film's copy goes | The table stays here; the film's split column becomes a link that jumps the playhead. |
| 9 | **Form** (the fingerprint's observations: cadence, CV, drift, decoupling, stride length, efficiency factor) | — | — | **K** | The run's mechanics as observations, never a score. |
| 10 | **The run's update** (the model's own reading, with its checks) | — | — | **K** | The one place a model writes. It is attributed and it can be dismissed. |
| 11 | **Against the plan** (the core's line) | the Plan page's adherence | — | **K** | A fact about the athlete's own plan. |
| 12 | **The note** | — | — | **K** | The only free text the product stores. It belongs to the run. |
| 13 | **Sleep composition** (the night before the run) | Today's sleep; Body's sleep | — | **R** | Context, not the run. Body answers it for any date. |

**After: 10 cards, 10 panels** — headline · what it meant · **the one chart** · effort (the zone table) · route · splits · form · update · against the plan · note.

### Progress, the wall — `/progress` (desktop) · `progressWall.tsx` + `taskAreas.ts` · 1 headline + **24 cards**

The card ids, their titles and their areas, exactly as `WALL_CARDS` and
`TASK_AREAS` have them:

| # | id | title (the catalogue's own) | area → destination | repeats | merge target | | why |
|---|---|---|---|---|---|---|---|
| 1 | `load` | Training load | The work → Progress | Today's *Form* row | — | **K** | A derived series with a scenario and an assumption. The scenario is one of the only two sentences a chart cannot carry. |
| 2 | `volume` | Weekly volume | The work | the Pace page's *training, week by week* | → *The weeks* | **M** | Same x (weeks), same window, same question. |
| 3 | `consistency` | Consistency | The work | — | → *The weeks* | **M** | It is the same weeks drawn a second way. |
| 4 | `zones` | Time in each zone | Where the time went | the run detail's zone table; the year's month cards | → *Where the time went* | **M** | — |
| 5 | `split` | Polarised split | Where the time went | card 4 | → *Where the time went* | **M** | **The polarised split is the same five zone numbers regrouped into three.** It is not a second fact. |
| 6 | `week` | This week against last week | The comparison | Today's *This week* row | → *This week* | **M** | — |
| 7 | `window` | Rolling window | The comparison | — | → *This week* | **M** | The same card, two periods; a card may name two windows. |
| 8 | `pace` | The pace | The pace and the results | the Pace page; the profile's facets; the run detail | → *The pace* | **M** | The pace appears three times today; after the cut it appears twice (the trend here, the run's own pace on the run detail) and the third is a facet value on the profile. |
| 9 | `bests` | Bests & milestones | The pace and the results | — | — | **K** | A list of dates and distances. Cutting a milestone list would be a real loss. |
| 10 | `profile` | The athlete profile (the **radar**) | The shape → Profile | card 11 | → *Your shape* | **M→R** | The radar draws the same seven numbers as card 11, as a polygon over fixed anchors — which `crates/stride-core/src/profile.rs` explicitly argues against ("**Not a radar.** A polygon over fixed anchors is a claim about a shape, and the anchors are somebody else's coaching assumptions"). The drawing goes; the numbers stay. |
| 11 | `facets` | Training profile (the seven facets) | The shape | card 10 | → *Your shape* | **M** | The facet bar list is the honest shape. |
| 12 | `challenges` | Your challenges | The goals | card 13 | → *The goals* | **M** | — |
| 13 | `adherence` | Adherence | The goals | the Plan page's adherence | → *The goals* | **M** | Both are "the plan against what happened". |
| 14 | `weekday` | Stats per weekday | The schedule | — | → *The schedule* | **M** | — |
| 15 | `daytime` | Daytime stats | The schedule | — | → *The schedule* | **M** | — |
| 16 | `distances` | Distance breakdown | The schedule | — | → *The schedule* | **M** | — |
| 17 | `year` | The year (the calendar) | The seasons | card 18 | → *The months* | **M** | Two calendars of the same year, coloured differently. One survives. |
| 18 | `heatmap` | Activity heatmap (the distance calendar) | The seasons | card 17 | → *The months* | **M** | **Cut.** The distance calendar and the year's calendar are the same grid; the distance colouring is the simpler one and the year's month cards already carry the zone mix. (This is **not** the `/heatmap` route heatmap — see below.) |
| 19 | `months` | Monthly stats | The seasons | the year's month cards | → *The months* | **M** | — |
| 20 | `years` | Yearly stats | The seasons | the year page's *What changed* | — | **R** | Two years of distance, and a page that compares years. One place. |
| 21 | `streaks` | Streaks | The habit | Today's *Habit* row | → *The habit* | **M** | — |
| 22 | `eddington` | Eddington number | The habit | Today's *Habit* row | → *The habit* | **M** | The streak and the Eddington are the same question — *how steady is this* — answered two ways. |
| 23 | `activities` | Most recent activities | Somewhere else | the Runs list | — | **R** | The Runs destination already holds it, with the calendar, the filters and the search. |
| 24 | `body` | Body, last night | Somewhere else | Today's vitals and sleep; the Body page | — | **R** | A third copy of last night. |

**After: 1 headline + 11 cards** —

*Progress (6):* **This week** (the comparison) · **The load** · **The weeks** (volume + consistency) · **Where the time went** (zones + the polarised split) · **The pace** · **Bests & milestones**.

*The athlete profile (5):* **Your shape** (the facets; the radar retired) · **The months** (the monthly bars + one calendar) · **The schedule** (weekday + time of day + distance buckets) · **The habit** (streaks + the Eddington number) · **The goals** (challenges + adherence).

### Progress, the index — the phone — `ProgressIndex.tsx` · 2 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **Six questions, and your answer to each** (window · load · year · pace · zones · habit) | the wall's cards | — | **K** | This is the phone's good idea: a question, an answer, a window. |
| 2 | **The areas** (ten headings, a link per panel) | the wall's cards' *open →* | — | **R** | Once the wall has eleven cards, the area link list is a second, worse index of the same eleven things. The index rows become the card list, in the wall's order. |

**After: 1 panel** — the index is the wall's card list, one row per card, in the wall's order.

### The athlete profile — the wall — `/profile` · 13 cards

Covered by cards 10–22 above. **After: 5 cards.**

### The athlete profile — the index — the phone · 3 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **Your training, in one picture** (the radar) | the profile's *Your shape* card | → the shape bar list | **M→R** | The radar goes with the wall's radar. |
| 2 | **Four questions** (goals · schedule · seasons · the months in a row) | the profile's cards | — | **K** | — |
| 3 | **The areas** (the link list) | the profile's cards | — | **R** | As above. |

**After: 2 panels** — the shape (as a list, not a polygon) + the questions.

### The pace page — `/pace` · `PacePage.tsx` · 4 blocks, **5 panels**

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The training, week by week** (distance + the longest run inside it + the pace line) | the wall's *Weekly volume* card | → the wall's *The weeks* card | **M** | The same weeks, the same window, a third drawing of them. |
| 2 | **Pace at 151 bpm** (the scatter) | — | → the wall's *The pace* panel | **M** | Already exists as `/progress/pace`. The page duplicates a panel. |
| 3 | **Where the time went, by pace** (the pace bands) | — | → the wall's *The pace* panel | **M** | The same panel: the pace-at-heart-rate and the pace bands are both "where did the time go, by pace". |
| 4 | **The range selector + the window line** | — | → the wall's range selector | **M** | — |
| 5 | **The summary line** | — | — | **R** | A sentence about a chart that is two panels below it. |

**After: 0 panels — the page is retired**, and its three charts become Progress panels (one panel: the pace-at-HR cloud and the pace bands; one card: the weekly bars). *This is the single biggest duplication in the product: the pace lives on the run detail, on a page of its own, and in the profile's facets.*

### The year — `/year` · `YearPage.tsx` · 3 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The months** (twelve month cards: the weekly strip, the distance, the zone mix) | the wall's *Monthly stats*; the wall's *Activity heatmap* | — | **K** | The month card is the richest unit in the product: a week strip, a distance and a zone mix in one thing. |
| 2 | **The year as a calendar** | the wall's *The year* and *Activity heatmap* | — | **K** | One calendar survives, and this is it. |
| 3 | **What changed** | the wall's *Yearly stats* | — | **K** | The year-over-year comparison; it replaces the retired *Yearly stats* card. |

**After: 3 panels** — unchanged, and now the only place the years are compared.

### Heatmap — `/heatmap` · `HeatmapPage.tsx` · 1 panel

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The heatmap** (the grid over the stored route positions) | nothing — the wall's *Activity heatmap* is a **calendar**, not this | — | **K** | One panel, one question (*where do I run?*), and nothing else in the product answers it. It is reached from Runs, the way the film is reached from a run. |

**After: 1 panel** — unchanged. Note the name collision: two different things are called "heatmap". The wall's one is a **calendar of days coloured by distance**; this one is a **grid over the route positions**. One of them must be renamed in the catalogue whatever happens to the cards.

### The film — `/runs/:id/film` · `FilmPage.tsx` · 3 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The film** (the timeline, the zone ribbon, the playhead) | — | — | **K** | The deep view of one run. |
| 2 | **The route** (the flyby, coloured by pace) | the run detail's route | — | **K** | Coloured by pace, with the playhead — a different drawing from the run detail's plain route. |
| 3 | **Splits** (the lap table, each lap a jump into the playhead) | the run detail's splits table | → the run detail's table | **M** | The table lives on the run detail; the film's lap column becomes a link that opens the film at that lap's time. |

**After: 2 panels.**

### Body — `/body` · `BodyPage.tsx` · 5 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The controls** (the date, the window) | — | — | **K** | — |
| 2 | **Sleep on {date}** | Today's sleep; the run detail's sleep | — | **K** | Body is the home of the sleep chart. |
| 3 | **The three signals** | Today's vitals; the wall's *Body, last night* | — | **K** | — |
| 4 | **How to read this** (what the direction words mean) | — | → the page's lede | **M** | Two sentences of explanation about the closed direction set. They belong with the page's own question, not as a fourth card. |
| 5 | **Patterns** (with their sample sizes) | — | — | **K** | The `pattern` register, and it is Body-only by the contract. |

**After: 4 panels.**

### Compare — `/compare/:id` · `ComparePage.tsx` · 6 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The basis** ("same distance, 12 days apart") | — | — | **K** | The sentence that says what the comparison is. |
| 2 | **The metric table** (metric · A · B · difference) | — | — | **K** | — |
| 3–4 | **Run A** and **Run B** | — | — | **K** | — |
| 5 | **The effort chart** (with the A/B switch and the heart-rate / pace signal switch) | the run detail's two charts | → the one chart, with the toggles | **M** | Compare's chart becomes the *same* consolidated chart as the run detail's, with the signal switch replaced by the series toggles. One chart, two places. |
| 6 | **Choose the other run** (the picker) | — | — | **K** | Contextual, and Compare stays contextual. |

**After: 6 panels** — same count, one chart less to understand.

### Plan — `/plan` · `PlanWeekPage.tsx` · 5 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **This week** (the sessions) | Today's *What's on the table* | — | **K** | Today shows the day; the Plan shows the week. |
| 2 | **The day detail** | — | — | **K** | — |
| 3 | **Adherence** (done · planned · not done) | the wall's *Adherence* card; the profile's *The goals* | — | **K** | The plan's own counts; the profile's card is the same fact over a longer window. |
| 4 | **Sources** (typed · prescribed · inferred baseline) | — | — | **K** | The plan's provenance. |
| 5 | **The paused note** | — | — | **K** | A fact, in the app's voice. |

**After: 5 panels** — unchanged. Plan stays contextual.

### Settings — `/settings` · `SettingsPage.tsx` · 9 panels

| # | panel | repeats | merge target | | why |
|---|---|---|---|---|---|
| 1 | **The local summary** (what is on this device) | the Storage group | → *Storage* | **M** | — |
| 2 | **Garmin** (the connection) | — | — | **K** | — |
| 3 | **Run updates** (the model's settings) | — | — | **K** | — |
| 4 | **Your challenges** | the profile's *The goals* card | — | **K** | Setting a challenge is a setting; reading it is a card. |
| 5 | **Weekly goal** | — | — | **K** | — |
| 6 | **Pause** | Today's paused note | — | **K** | — |
| 7 | **Import** | Runs' import | — | **K** | — |
| 8 | **About** (and the promise: your data stays here) | — | — | **K** | The statement is the product's posture in one line. |
| 9 | **Storage** (export · delete) | — | — | **K** | — |

**After: 8 panels.**

---

## 2. The cut list, in priority order

### 2a. Cut first — high confidence, low loss

| # | the cut | what it removes | why it is safe |
|---|---|---|---|
| 1 | **The radar** (the wall's *The athlete profile* card, and the phone index's *Your training, in one picture*) | 1 wall card, 1 index panel, and the polygon | The core's own module says a polygon over fixed anchors is a claim about a shape that somebody else made. The seven facets behind it are the same numbers and are not a claim. |
| 2 | **The three stacked run-detail charts → one** | 2 panels, 2 axes, 2 cursors | They are one run, read three ways. The consolidated chart keeps every series and adds one shared cursor and one tooltip. |
| 3 | **The polarised split as its own card** | 1 wall card | `rolling.rs`'s `polarised_shares` is the five zone shares regrouped into three. It is a second *drawing* of one fact, not a second fact. It goes under *Where the time went*. |
| 4 | **The splits table on the film** | 1 panel | The same table, twice, in two places. The lap number becomes a link into the playhead. |
| 5 | **The `/pace` page** | 1 route, 4 blocks, 5 panels | It duplicates a Progress panel that already exists (`/progress/pace`) and a wall card that already exists. Its third chart duplicates the weekly card. The pace is the clearest case of the product showing the same thing three times. |
| 6 | **The phone's *areas* link lists** (both indexes) | 2 panels | They are a worse index of the same cards the wall already draws. The index rows become the card list. |
| 7 | **The wall's *Activity heatmap* calendar** | 1 wall card | Two calendars of the same year. One survives, coloured by distance. |
| 8 | **The wall's *Most recent activities* table** | 1 wall card | The Runs destination holds it, with a calendar, filters and search. |
| 9 | **The wall's *Body, last night* card** | 1 wall card | A third copy of last night, on a screen whose question is not about last night. |
| 10 | **The wall's *Yearly stats* card** | 1 wall card | The year page compares the years. |
| 11 | **The run detail's sleep composition** | 1 panel | Context, not the run. Body answers it for any date. |
| 12 | **The wall's *Consistency* card** | 1 wall card | The same weeks drawn a second way; it goes under *The weeks*. |
| 13 | **The wall's *Rolling window* card** | 1 wall card | It is the comparison of two windows; the comparison card can name two windows. |
| 14 | **Today's *Rest*, *vitals* and *sleep* as three blocks** | 2 panels | One *Last night* card: the Rest summary, the three signals, the sleep chart. |
| 15 | **Body's *How to read this*** | 1 panel | Two sentences about the closed direction set, folded into the page's own lede. |

### 2b. Never cut

* **The run's own facts** — the headline numbers, the splits table, the form observations, the note. The note is the only free text the product stores and it belongs to the run.
* **The zone time table, with the *not moving* bucket and the model's source line.** The contract's rule is that a number is reported with its source; that line is the product's posture in one sentence and it is the reason a zone time can be audited.
* **The route, the flyby and the film.** The spatial truth of the run, and the only place the run is seen in motion.
* **The run update and its checks.** The one place a model writes. It is attributed, it carries what it was given, and it can be dismissed.
* **The plan's session on Today, unranked and unlinked to the vitals.** The only choice in the product; the link between a choice and a vital is exactly what the contract forbids.
* **Every "no value" reason.** Absence is not zero. A cut that removes the reason turns an absent number into a zero, and the zero is a lie.
* **The load's scenario and its assumption.** The model's expectation and the model's provenance are the only two sentences a chart cannot carry.
* **The Eddington number's window, and the days still needed for the next one.** A number is only a fact against a stated period.
* **Settings' *About* promise and the *Storage* controls.** The storage is the promise; the promise is the product.
* **The empty states.** Each one names what is missing and what to do.

### 2c. Deliberately kept even though it looks redundant

| what is kept twice | why |
|---|---|
| **The load on Today and on Progress** | Today shows *the last state* — the form, the ratio, the band, the day it was read. Progress shows *the series and the scenario*. A number and its trend are different questions, and Today is the only screen where a number is read with no range selector in sight. |
| **The streak on Today and on *The habit*** | Same reason: a count on the cockpit, the Eddington and the longest run beside it on the profile. |
| **The zone split on the run detail and on Progress** | The run's bands are the run's own hours; the window's split is the window's share. Different periods, one vocabulary — which is why the *source line* matters more than the chart. |
| **The pace on the run detail and on *The pace*** | A run's pace is a run's pace; the panel is the trend. The third home (the profile's facet) is a value, not a chart. |
| **The year card and the year page** | The card is the calendar; the page is the months and what changed. |
| **Compare** | Contextual, and the only place two runs are read against each other. |
| **The route heatmap** | One panel, one question, and nothing else answers it. |

---

## 3. What the consolidated chart needs from the core and the api

The prototype in `exploration/consolidated-chart.html` is plain HTML, but it is
not a mock-up: it is drawn from the fields the product already has.

**What exists today.** `/activities/{id}` returns **two** arrays with **two**
x-axes (`crates/stride-api/src/dto.rs`):

* `samples: Vec<SamplePoint>` — `offset_s`, `pace`, `heart_rate`;
* `elevation: Vec<ElevationPoint>` — `distance`, `elevation`.

The run detail then draws three charts from them: the heart-rate line with the
zone bands (`ActivityPage.tsx`, `activity-effort`), the pace line (same card,
`syncId` shared with the heart rate), and the elevation area on the **distance**
axis (a third chart, its own axis, its own cursor).

**What one chart needs.** A single sample list carrying `offset_s`,
`distance_m`, `pace`, `heart_rate`, `altitude` and the movement kind.

**The good news: the data is already decoded.**
`crates/stride-ingest/src/segment.rs` classifies every sample as **run /
walk / stand** (`classify_sample`: speed < 0.30 m/s or a stored cadence of 0 →
stand; cadence < 60 with speed < 2.2 m/s, or speed < 1.55 m/s → walk; else
run), and each sample already carries `elapsed_s`, `distance_m`, `speed_ms`,
`stored_cadence`, `heart_rate_bpm` and `altitude_m`.

**What changes.**

1. **`stride-api`**: `SamplePoint` gains `distance_m`, `altitude_m` and
   `kind` (`run` / `walk` / `stand`). The `elevation` array becomes a
   projection of the samples and stops being a separate series — and stops
   being a second x-axis.
2. **`stride-core`: nothing new.** The segmentation is the ingest's; the zone
   time already measures only the moving samples (`zone_samples` keeps the
   stopped flag so the stopped span is charged to its own bucket, not merged
   into the running one); the pace bands skip the stopped spans; the
   fingerprint already drops the walked laps (`running_only`) and the
   fragments under 300 m (`measurable`).
3. **The chart reads the same classification the zones do.** That is the point
   of the cut: the gap in the pace line is exactly the *not moving* time in
   the Effort table, and the dashed stretch is exactly the walking the
   fingerprint excluded. One classification, three places.

**One thing to decide.** The chart's walk/stand marks come from the
**sample-level** segmentation (`segment.rs`), while the fingerprint's
observations use the **lap-level** `running_only` test
(`fingerprint.rs`, cadence < 125 spm *and* speed < 2.2 m/s). Two thresholds,
two granularities, in two modules. The chart should use the sample-level one
— it is the one the zone time and the pace bands already use — but the two
thresholds should eventually be one rule with two views of it.

**The x-axis, and why it is time.** Not distance.

* Every observation the core makes about a run is a difference of **times**:
  the pace drift and the heart-rate drift are "second half minus first half"
  (`fingerprint.rs`), the decoupling is the heart rate's rise against the
  pace's, and the time in a zone is seconds of moving time.
* On a distance axis, "the second half" is not the second half the core
  measured: a fast first half covers more ground, so the halves are not
  comparable — the drift the product reports could not be read off the chart.
* A stopped stretch covers **no distance** but is **real elapsed time**, and
  the product has a whole bucket for it. On a distance axis it would vanish;
  on a time axis it is a gap, which is what it is.
* The zone bands behind the heart rate are bands of **time**, and the Effort
  table's numbers are times.

**The altitude.** Altitude is not a pace and cannot share its axis: metres
(28–52) and seconds per kilometre (308–339) would flatten the pace curve
into a straight line and imply a comparison the data does not support. The
prototype gives it **its own scale, its own band and its own treatment** — a
filled area with a hatched edge, anchored to the bottom of the plot, with
its two labels (metres) inside its own band — while the **pace and the heart
rate** share the effort band above it and **one cursor** runs through both.
The reader reads "at 22 minutes: 5:08 /km, 162 bpm, 44 m" from one
crosshair, and the ground under it is scenery, not an effort signal.

---

## 4. Where this disagrees with `exploration/task-areas-plan.md`

The plan is good work and this thread uses its ordering. Four disagreements.

1. **"No card is lost. All twenty-four survive."** (the plan's rule 1) is a
   *reorganisation*, not a cut. The user's instruction here is to reduce the
   surface, and rule 1 forbids the only move that reduces it. **Keep the
   areas' order and their headings; break rule 1.** The plan's own
   diagnosis — "every card is the same size and the same weight", "there is
   no answer to *where do I look for X?*" — is a diagnosis about **card
   count**, and grouping 24 cards under 10 headings does not answer it: the
   reader still has to hold ten names to find one card.
2. **"An area is a heading and nothing more."** Right, and it is also the
   limit of the plan. Ten headings is an index; the answer should be
   **eleven rows** (the phone's), not ten headings plus twenty-four cards.
   After the cut, the phone's index rows *are* the wall's cards, in the
   wall's order — one list, one order, two widths.
3. **The pace is in two areas at once.** The plan puts "The pace and the
   results" on the work side and the pace is also one of the profile's
   facets. That duplication is deliberate and correct — a facet is a value
   inside a shape, not a chart — but it should be *said*, because otherwise
   it reads as an oversight.
4. **"The decision about a fifth destination — later"** (the plan's stage 4)
   is stale: `destinations.ts` and `App.tsx` already ship **five**. The
   decision was made in code without the plan recording it. This thread
   keeps five and says so.

What this thread takes from the plan: the **order** (happening now →
comparison → the shape of the months → the habits), the **areas' names**,
and the rule that the phone and the wall read **one model**. What it drops:
rule 1.

---

## 5. The four-destination question (the contradiction, priced)

The brief says four destinations; the code ships five. Here is what each
reading costs, with the cut above applied.

| | five destinations (the code today) | four destinations (the brief's wording) |
|---|---|---|
| Progress wall | headline + **6** cards | headline + **11** cards |
| Profile wall | **5** cards | — (its cards move to Progress) |
| Phone indexes | **2** panels (Progress's rows, the Profile's shape + rows) | **2** panels (one list of 11 rows; the shape becomes a row) |
| Panels after the cut | **63** | **63** |
| Reading | the long pattern is a place of its own; two indexes | one index, one wall, eleven cards to choose from |

**The panel count is the same either way.** The four-destination reading
**moves** panels, it does not remove them. That is the whole of the decision:
does the athlete profile keep a door of its own, or does its five cards join
Progress's six?

My recommendation: **keep five destinations**, because (a) it is what ships
and what the athlete has learned, (b) the *shape* is the one answer in the
product that is a picture rather than a number, and giving it a page of its
own is what makes it legible, and (c) an eleven-card wall is the thing the
user asked to get rid of. But it is the user's call, and it is the first
question in the report.

---

## 6. The consolidated Progress, card by card

What each merged card **says**. Every sentence is a placeholder for a string
the core would own; the wording follows the voice contract (facts, no
verdict, no colour as a judgement, the window named).

### Progress — 6 cards

**1. This week** — *(merges `week` + `window`)*
> **24.6 km** this week · 3 runs
> the same days last week: 19.1 km · 2 runs
> **+5.5 km**
> *Mon 17 Nov – Sun 23 Nov, against Mon 10 Nov – Sun 16 Nov · 84 days against the 84 days before*

**2. The load** — *(keeps `load`)*
> acute **118** · chronic **84** · form **+34** · ratio **1.40**
> *Banister k = 1.92 · max heart rate 197 bpm from your zone model, read on 12 Nov 2026 · with no training for 7 days the ratio is expected back inside 0.8 to 1.3 in about a day*

**3. The weeks** — *(merges `volume` + `consistency`)*
> the weekly bars, with the typical week as a line above them, and the days of each week that
> held a run drawn as marks under the bar
> *12 weeks · 612 km · your typical week 22.1 km*

**4. Where the time went** — *(merges `zones` + `split` + the zone-time distribution)*
> Z1 **18%** · Z2 **41%** · Z3 **26%** · Z4 **11%** · Z5 **4%** · outside **0%**
> *low 59% · moderate 26% · high 15%*
> *measured over the moving time · your zone model, read on 12 Nov 2026*

**5. The pace** — *(merges `pace` + the `/pace` page's cloud and bands)*
> **5:24 /km** this week at **152 bpm**
> *where the time went, by pace: 5% above 4:50 · 26% between 5:00 and 5:30 · 69% slower than 5:30*
> *12 weeks · 23 Feb – 13 May*

**6. Bests & milestones** — *(keeps `bests`)*
> 5 km — **20:48** on 3 Nov · 10 km — **43:12** on 9 Nov · longest run **21.4 km** on 2 Nov
> *the last 12 weeks*

### The athlete profile — 5 cards

**7. Your shape** — *(merges `profile` + `facets`; the radar retired)*
> seven facets, each a value and the direction it moved against the window before
> *volume 612 km · consistency 0.84 · intensity 0.41 · duration 51 min · density 4.1 · variety 3 · days of the week 4*
> *28 days, against the 28 days before*

**8. The months** — *(merges `months` + `year` + `heatmap`; `years` retired to the year page)*
> the monthly bars, and the year as a calendar coloured by distance
> *Feb 38.4 km · Mar 61.2 km · Apr 54.8 km · May 72.1 km*
> *the last 12 months*

**9. The schedule** — *(merges `weekday` + `daytime` + `distances`)*
> Mon **4** runs · **48.2 km** · Tue **3** · 34.9 km · Wed **3** · 41.0 km …
> *and the time of day: before 09:00 — 6 runs, 71.4 km · 09:00–12:00 — 8 runs, 112.9 km · 12:00–18:00 — 9 runs, 138.2 km*
> *and by distance: under 5 km — 11 runs · 5–10 km — 9 runs · 10–15 km — 4 runs · over 15 km — 2 runs*

**10. The habit** — *(merges `streaks` + `eddington`)*
> **6** days in a row · longest **11** · **2** months in a row
> *Eddington **7** — 7 days of at least 7 km · 12 weeks · 5 days still needed for 8*

**11. The goals** — *(merges `challenges` + `adherence`)*
> *A 10 km a week — met in **5** of 12 weeks · A half marathon — met in **1** of 12 weeks*
> *the plan — **9** of 10 sessions done · typed · Mon 10 km · Wed easy · Fri 8 km*

---

## 7. What the cut does **not** touch

* The four destinations (or five) and their order.
* The contract: no new string type, no new direction word, no word off the
  ban list, no score, no grade, no traffic light, no red.
* The core's modules. `load.rs`, `zones.rs`, `pace.rs`, `rolling.rs`,
  `fingerprint.rs`, `year.rs`, `profile.rs`, `streaks.rs`, `eddington.rs` and
  `heatmap.rs` keep their jobs; the cut is in **which answers reach a
  screen**, not in how an answer is computed.
* The empty states and the "no value" reasons.
* The run update, the note, the film, the route heatmap, the plan and
  Settings' storage and promise.

The cut is: **twenty-four cards become eleven, thirteen run-detail charts
become one, and one page (the pace page) stops being a page.**
