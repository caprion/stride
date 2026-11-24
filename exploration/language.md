# The language pass — every product-facing string in the prototypes

*24 November 2026. One rule, one test, one vocabulary — and the pass applied to
`exploration/athlete.js` and to the six prototypes.*

---

## 1 · The rule

> **Precision belongs in the notes. The surface speaks the runner's language.**

* The **notes** — the annotation layer, off by default — may use the precise terms:
  *"elapsed time"*, *"moving time"*, *"the zone model's floors"*, *"the source's
  `hrTimeInZone` field"*, *"TRIMP"*, *"Banister k"*. A note is written for someone
  reading **about** the product, and it is where a term is defined once.
* The **surface** — a label, a caption, a title, a state, a table header, a button,
  a tooltip, an accessible label — is written for a runner looking at their own
  training. No field names, no model names, no internal vocabulary, and no phrase
  that only makes sense to whoever wrote the code.

### The test

> *Would a runner who has never read the code understand this line without it being
> explained?*

If not, the line is one of three things, and each has a fix:

| it is | the fix |
|---|---|
| **too technical** | say what the number **is** ("your fitness number 49.7", not "VDOT 49.7") |
| **too abstract** | say the thing in the runner's words ("the time you were running", not "moving time") |
| **explanation** | it is not product content — it belongs in a note (**D22**) |

### The precedent

* **D22 — explanation is not product content.** The product surface carries the
  picture, its labels, the numbers, the period, and — only where something is
  missing — the reason something is missing. Every line that explains **the design**
  rather than **the data** moves to the notes. *Explanation is moved, never deleted.*
* **D23 — option A.** Two blocks, each naming its own basis, never added together:
  the app's own zone time and the watch's own zone minutes. The pass keeps that
  shape and renames both blocks in the runner's language:
  **"Your zones · the time you were running"** and **"The watch's own zones · the
  whole run, stops included"**.

### What counts as a surface, and what counts as a note

| layer | where it is | rule |
|---|---|---|
| **surface** | anything a runner reads on a screen: card headers, labels, values, captions, table headers, chips, buttons, states, and the `aria-label`s that screen readers read | the runner's language |
| **notes** | the annotation layer: the pins and the notes pane in `end-to-end.html`, the `note()` blocks in `threshold-setup.html`, the `.annotation` paragraphs and the notes panel in `pace-engine.html` and `two-bases.html` | precise language allowed |
| **design documentation** | the prototypes' own prose: the ledes, the `<p class="note">` sections of `cards-as-pictures.html`, the frames' captions, the `.md` notes' analysis | precise language allowed; the ban list still applies |

The pass touched the first two. The third was left alone, except that the ban
list was honoured in it as well.

---

## 2 · The vocabulary

One place, so the whole product says the same thing. Every entry is a word the
product uses more than once.

| the hard word | the plain form on a surface | what it actually is |
|---|---|---|
| the zone model | **your zones** | the watch's configured heart-rate bands, read once and dated |
| moving time | **the time you were running** | the run's own moving samples |
| elapsed time | **the whole run** (stops included) | the watch's own session duration |
| the watch's own zone minutes | **the watch's own zones** | the source's own per-zone time record |
| the model in force | **the zones your watch was set to that day** | the dated model |
| your zone floors | **where your zones start** | the lower edge of each band |
| VDOT | **your fitness number** | the one number a race time gives |
| Riegel | *(the note)* — on a surface: **the times your race gives** | the engine's relation between distances |
| TRIMP | *(the note)* | the load model's per-sample weighting |
| Banister k | *(the note)* | the load model's constant |
| the load | **your training load** | the training-load card |
| acute | **the last 7 days** | the short-window load |
| chronic | **the last 6 weeks** | the long-window load |
| form | **the difference** | the last 7 days against the last 6 weeks |
| ratio | **times the last 6 weeks** | the short window over the long one |
| the acute-to-chronic ratio | **1.40 times the last 6 weeks** | the same number, named |
| the base | **the plan you follow** | the adopted training plan |
| a tier | **the level of the plan** | where a plan starts you |
| the session kinds | **the kinds of run** | the plan's own session names |
| the window | **the last 12 weeks** (or the dates, which the screen shows) | the period every card answers about |
| the period before | **the same span, one week earlier** | the comparison period |
| no value | **say what is missing**: *not recorded*, *not measured*, *not worked out*, *no pace yet* | an absent measurement |
| the ladder | **the training-paces card** | the pace ladder |
| the ledger | **the zone times** | the window's zone rows |
| the method's rule | **the plan's rule** | the published rule of the adopted plan |
| the engine's own | **worked out from your race** | the app's own arithmetic |
| Eddington | **12 km on 13 days** | a distance reached on at least that many days |
| no zone model | **no zones yet** — your watch's heart-rate zones have not come through | nothing has been read from the watch |

---

## 3 · The pass, surface by surface

### The source line — every screen

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `your zone model · running · HR_MAX · max heart rate 183 bpm · read on 3 Nov 2026` | `athlete.js` `meta.zoneModelLine`; every screen's source bar | "the zone model" is a field's name; `HR_MAX` is the watch's method id; "read" is the app's verb | `your zones · running · max heart rate 183 bpm · taken from your watch on 3 Nov 2026` |
| `your zone model · running · HR_MAX · max heart rate 183 bpm · read on 10 Sep 2026` | `stimulus-ledger.html` (the runs-before-the-model state) | as above | `your zones · running · max heart rate 183 bpm · taken from your watch on 10 Sep 2026` |
| `your zone model · running · HR_MAX · max heart rate 183 bpm · read on 12 Nov 2026` | `cards-as-pictures.html` | as above | `your zones · running · max heart rate 183 bpm · taken from your watch on 12 Nov 2026` |

### Today

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `100 % of the moving time` | Today hero, the band row | "moving time" is the core's term | `100 % of the time you were running` |
| `moving time` (row label) | Today, the four figures | the core's term | `time you were running` |
| `Mon 23 Nov and Tue 24 Nov, against the same two days of the previous week.` | Today, "Where you stand" | "the previous week" is not the comparison — the same span, one week earlier, is | `Mon 23 Nov and Tue 24 Nov, against the same two days a week earlier.` |
| `Form` / `+34` / `118 acute / 84 chronic · 1.40` | Today, "Where you stand" | "form" is a statistic name; "acute" and "chronic" are the load model's window names | `Training load` / `+34` / `the difference · last 7 days 118 · last 6 weeks 84 · 1.40 times the last 6 weeks` |
| `acute 118 · chronic 84 · form +34 · ratio 1.40` | Today, the load card; the wall's load card; `stimulus-ledger.html`; `cards-as-pictures.html` | four statistic names in one line | `last 7 days 118 · last 6 weeks 84 · the difference +34 · 1.40 times the last 6 weeks` |
| `Banister k = 1.92 · with no training for 7 days the ratio is expected back inside 0.8 to 1.3 in about a day` | the load card's caption | a model constant and an unnamed ratio; the note already carries the constant | `with no training for 7 days, 1.40 is expected to come back inside 0.8 to 1.3 in about a day` |
| `Eddington 12` | Today, the habit row; the profile's habit card | the statistic's own name | `12 km on 13 days` — the number is the distance; 13 is the count of days that reached it |
| `58 days recorded in the window of 12 weeks.` | Today, the habit card | "the window" is internal | `58 days with a run recorded, in the last 12 weeks.` |
| `the model’s own words` | the run update card's sub-header | names the mechanism | `written from the run’s own numbers` |
| `the athlete’s own words` | the run note card's sub-header | third person where the product says "you" | `your own words` |

### Runs

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `scope: runs` | the filters | "scope" is a query term | `showing: runs` |
| `58 in the window` | the list header | "the window" is internal | `58 runs in the last 12 weeks` |

### Run detail

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `moving time` (row label) | the run's rows | the core's term | `time you were running` |
| `elapsed` (row label) | the run's rows | a statistic's name | `the whole run` |
| `20.4 km in 1 h 59 m — the longest run of the window.` | the run's sentence | "the window" is internal | `20.4 km in 1 h 59 m — the longest run of the last 12 weeks.` |
| `hr` (table header) | the splits table | an abbreviation | `heart rate` |

### Progress — the wall

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `the moving time of the window, by zone` | the ledger card's picture | the core's term | `the time you were running, by zone` |
| `the signed figure is the change vs the period before` | the ledger card's key | "the period before" is not the comparison | `the signed figure is the change against a week earlier` |
| `By intensity · your zones` | the ledger card's first block | "intensity" is a statistic's name | `By heart rate · your zones` |
| `By shape · the run's own structure` | the ledger card's second block | "the run's own structure" is internal | `By the runs themselves` |
| `a measurement that does not use heart rate` | the same block's coverage line | a clause about the mechanism | `measured without a heart rate` |
| `58 of 58 runs carry heart-rate samples` | the same block's coverage line | "carry heart-rate samples" is a field's phrase | `58 of 58 runs recorded a heart rate` |
| `37 m more against the period before (41 h 32 m against 40 h 54 m)` | every changed row | "the period before" is not the comparison | `37 m more than the same span, one week earlier (41 h 32 m against 40 h 54 m)` |
| `84% of the moving time` | every share pill | the core's term | `84% of the time you were running` |
| `measured over the moving time` | the ledger card's basis line | the core's term | `measured over the time you were running` |
| `the athlete's own zones` (row sub-label) | the desktop ledger table | third person | `your own zones` |
| `Against the period before` (table header) | the desktop ledger table | as above | `Against the same span, one week earlier` |
| `35 days before the end of the window` | the Z4–Z5 row | "the window" is internal | `35 days before the end of this period` |
| `The load` | the wall's card title | the card's own noun is fine; the *statistics* inside it were not | `Training load` |
| `The Eddington has no value` | the habit card's empty state | the statistic's name plus a bare absence | `not enough days yet` |
| `no zone model has been read — the band rows have no value` | the wall's empty states | a field's name; "no value" alone | `your watch's zones have not come through — the band rows are not worked out` |

### The pace card

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `VDOT 49.7 · Daniels' chain` | the card's sub-header | a statistic's name and an author's name | `your fitness number 49.7 · from your 10K in 41:36` |
| `the method's band · your own zone · your zone floors · the method's pace · your own pace · the engine's own` | the legend | five internal nouns | `the plan's band · your own band · where your zones start · the plan's pace · your own pace · worked out from your race` |
| `Riegel, exponent 1.06, from your 10K in 41:36 on 12 Oct 2026.` | the equivalents card's caption | names the relation; the source is what the runner needs | `from your 10K in 41:36 on 12 Oct 2026.` — Riegel stays in the note |
| `Heart rate on the left, from the zone model's own floors; pace on the right, from the race the athlete typed. They are numbers about different things, so they are never converted into one another.` | the ladder's caption | "the zone model's own floors"; third person; a sentence that repeats itself | `Heart rate on the left, from your own zones; pace on the right, from the race you entered. A heart rate and a pace are different things, and one is never turned into the other.` |
| `the engine's own` (a rung's rule) | the ladder | names the mechanism | `worked out from your race` |
| `no pace rule · yours` (a rung's rule) | the ladder | internal | `yours — no rule of its own` |
| `+75 s/km off HM pace` (a rung's rule) | the ladder | an abbreviation and a formula | `+75 s/km slower than your half-marathon pace` |
| `10K × 1.20–1.33` (a rung's rule) | the ladder | a formula | `your 10K pace × 1.20–1.33` — the digits are unchanged |
| `the physiological-cost curve` | the curve card, its title, its link, the route card | a physiological term | `the effort curve` |
| `% maxVO₂` (the curve's y-axis label) | the curve | see §4 — kept | kept |
| `the method's sessions` | the curve's legend | internal | `the plan's sessions` |
| `the share of VO₂max each duration holds` | the curve's legend | a physiological term in prose | `the share of your maximum oxygen uptake each duration holds` |
| `No race time entered — the curve has no value. Every point on it is measured against your VDOT, and the VDOT comes from your race.` | the curve's empty state | three statistic names; "no value" | `No race entered yet — the curve is not drawn. Every point on it comes from the number your race gives.` |
| `no value` (a rung's pace cell) | the ladder | a bare absence | `no pace yet` |
| `The general aerobic pace has no value — no runs at that heart rate yet.` | the ladder's caption | a bare absence | `The general aerobic pace has nothing to show yet — no runs at that heart rate yet.` |
| `typed by the athlete` | the race-fields card | third person | `entered by you` |
| `the equivalent of your 10K in 41:36 is 1:31:47` | the desktop self-check | "equivalent" is the engine's noun | `the time your 10K in 41:36 gives for it is 1:31:47` |

### The threshold

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `your watch's zone model` | the candidate's title | a field's name | `your watch's zones` |
| `the floor of your watch's fifth zone · zone model read on 3 Nov 2026` | the candidate's and the choice's source | "the floor" is the core's noun; "zone model" is a field's name | `the start of your watch's fifth zone · read from your watch on 3 Nov 2026` |
| `the book's rule — the pace you could race for about an hour…` | the candidates' rule line | "the book" is the source's name | `the plan's rule — the pace you could race for about an hour…` |
| `the rule ·` (the rule line's label) | the candidates | internal | `how it was worked out ·` |
| `The ladder's threshold rung` / `the method's rule` | "where the threshold is used" | "the ladder" is the card's internal name | `The threshold pace on the training-paces card` / `the plan's own rule` |
| `the 80/20 base's bands` | "where the threshold is used" | "the base" is internal | `the 80/20 plan's easy-day bands` |
| `the ledger's rows` / `the athlete's own zones` | "where the threshold is used" | internal, third person | `the zone times on your runs` / `your own zones` |
| `The base you follow` | the used frame | "the base" is internal | `The plan you follow` |
| `The ladder's threshold rung` / `the method's rule` | the used frame | internal | `The threshold pace on the training-paces card` / `the plan's own rule` |
| `no value` (the 80/20 band's slot) | the used frame | a bare absence | `not set` |
| `the easy-day band has no value — the base's bands are set from a threshold heart rate…` | the used frame | "no value"; "the base" | `the easy-day band is not set — the plan's bands come from a threshold heart rate…` |
| `The ledger's rows, and the load` / `the athlete's own stored zones` | the used frame | internal | `The zone times on your runs, and the training load` / `your own zones` |
| `No threshold is chosen.` | the header card | past tense for a state that is still open | `No threshold chosen yet.` |
| `the watch's zone model` (the four ways) | the header card and the empty frame | a field's name | `your watch's zones` |
| `On the physiological-cost curve` | the field-test frame | a physiological term | `On the effort curve` |

### The plan

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `The base` | the plan card | "the base" is internal | `The plan you follow` |
| `The kinds` | the session-kinds card | internal | `The kinds of run` |

### The athlete profile

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `4 days in a row · Eddington 12 · 58 days recorded` | the habit card | the statistic's name | `4 days in a row · 12 km on 13 days · 58 days recorded` |

### Body

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `370 min` | the sleep card's sub-header | not the app's duration format (`units.rs`) | `6 h 10 m` |

### Compare

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `moving time` (metric name) | the metric table | the core's term | `time you were running` |

### The year

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `The two periods are different lengths; the comparison is between the same span of the year and the nine months before it.` | "what changed" | a semicolon-clause that names the mechanism | `The two periods are different lengths — this is the same span of the year, and the nine months before it.` |

### The film and the heatmap

| now | where | what is wrong | the rewrite |
|---|---|---|---|
| `The heatmap` / `the stored positions` | the context screen | "heatmap" is a chart's internal name | `Where you run` / `the positions your watch recorded` |

### The states

Every state now says **what is missing** and **where it would come from**.

| now | what is wrong | the rewrite |
|---|---|---|
| `Zones unavailable — no zone model has been read.` | a field's name; "unavailable" is a system's word | `No zones yet — your watch's heart-rate zones have not come through.` |
| `No race time entered — the race paces and the training paces have no value.` | "no value" alone; "race paces" is the engine's plural | `No race entered yet — the race times and the training paces are not worked out.` |
| `No plan adopted — the plan's sessions have no value.` | "no value" alone | `No plan adopted yet — there are no sessions to show.` |
| `No threshold is chosen. Every answer that needs one is a no value with a reason.` | "no value with a reason" is the core's phrase; the reason is not given | `No threshold chosen yet. Everything that needs one says what is missing.` + `it could come from your watch's zones, a race, a field test, or a number you enter.` |
| `No run recorded in the window.` | "the window" is internal | `No runs recorded.` |
| `The split has no value — the runs in the window have no heart-rate samples.` | "the split has no value"; "samples" is a field's word | `No heart rate recorded — none of the runs in this period recorded a heart rate.` |
| `no value — the runs in the window carry no workout steps` | a bare absence | `not recorded — the runs in this period carry no workout steps` |
| `no zone time for 3 runs in the window — the first stored zone model was read on 10 Sep 2026` | two internal nouns | `no zone time for 3 runs in this period — your watch's zones were first read on 10 Sep 2026` |
| `the change against the period before is not reported — the two periods do not hold the same runs` | "the period before" | `the change against a week earlier is not reported — the two periods do not hold the same runs` |

### The two-bases card — the user's own complaint

| now | what is wrong | the rewrite |
|---|---|---|
| `Your zone model` / `the app's own zone time · over the moving time` | the first block's name and basis | `Your zones` / `the time you were running` |
| `The watch's own zone minutes` / `the source's own record of each run · over the elapsed time` | "zone minutes" is a field; "the elapsed time" is a statistic | `The watch's own zones` / `the whole run, stops included` |
| `watch zone 1 … watch zone 5` | an internal row name | `the watch's zone 1 … the watch's zone 5` |
| `the app's zone time against the watch's zone minutes` | the check's basis | `your zones against the watch's own zones` |
| `the app's zone time · over the moving time` / `the watch's zone minutes · over the elapsed time` | the check table's two heads | `your zones · the time you were running` / `the watch's own zones · the whole run, stops included` |
| `3 minutes apart — the app's zone time is measured over the moving time, the watch's zone minutes over the elapsed time` | three pieces of jargon in one sentence | `3 minutes apart — one counts the time you were running, the other the whole run.` |
| `the app's zone time is measured over the moving time, the watch's zone minutes over the elapsed time` | the desktop reading | `your zones — the time you were running` / `the watch's own zones — the whole run, stops included` |
| `The two blocks are measured over two different spans of the same runs, and are never added together.` | explains the drawing (**D22**) | `The two blocks count the same runs two ways, and are never added together.` |
| `no zone model was in force on the runs before 3 Nov 2026` | internal | `your watch's zones were not read before 3 Nov 2026` |
| `the watch's own zone numbers — the boundaries of these zones are not recorded` | internal | `the watch's own zones — where each of them starts is not recorded` |

---

## 4 · What stays technical on a surface, and why

The line is drawn at **the runner's own device and the runner's own data**.

| it stays | why |
|---|---|
| **`Z1`–`Z5`, `Z1–Z2`, `Z4–Z5`** | they are the watch's own zone labels — the names on the athlete's wrist. The app groups them (`Z1–Z2 low`); the watch does not. |
| **the watch's own metric names** — cadence, stride length, aerobic decoupling, efficiency factor, HRV | the athlete's watch already shows these under these names. Renaming them would make the app's numbers unmatchable to the device that produced them. Each is defined once in the note beside it. |
| **the plan's own session names** — Endurance, Intervals, Long run | the athlete's own plan's vocabulary; the contract already exempts them ("easy" and "very easy" survive as session types). |
| **the name of the adopted plan** — *the Pfitzinger 10K plan* | the athlete's own plan's name, like a race's name. |
| **race names, distances and times** — 10K, half marathon, 41:36 | the athlete's own facts. |
| **`% maxVO₂`** — the curve's y-axis label | the standard name of the measure, and the chart has no other name for it. The prose caption says it in full ("the share of your maximum oxygen uptake each duration holds"). |
| **units and formats** — bpm, /km, ms, spm, `1 h 08 m`, `84 %`, `40 days` | the app's own conventions (`crates/stride-core/src/units.rs`) are the authority; the pass changed the words around them, never the format. |
| **dates** | `12 Oct 2026`, `35 days before the end of this period`. |

What does **not** stay: field names (`hrTimeInZone`, `zones.models`, `zones.source`),
the engine's statistic names (`VDOT`, `Riegel`, `TRIMP`, `CTL`, `HRR`, `Banister k`),
the internal nouns (`the zone model`, `the plan base`, `the session kind`, `the
ladder`, `the ledger`, `the window`, `the period before`), and the phrase
`no value` standing alone.

---

## 5 · The check

### Ban list — `docs/contract/voice.md`

**Zero hits in the surfaces.** Zero in the annotation layer. The only remaining hits
in the six prototypes are in `cards-as-pictures.html`, where the page **displays the
ban list itself** — the rule, quoted verbatim, on the page that demonstrates it.
That is the one exception, and it is deliberate.

Three words needed care because the ban list bans a *construction* rather than a
word: `rest` is banned as a command and survives as the name of Today's row;
`red` is banned as a judgement and survives inside `read`; `bad` survives inside
`badly`. The notes' prohibitions were rewritten so that they do not use the banned
words to state the prohibition — *"never a day marked as skipped, never a day named
as a day off"*, *"may not judge the run"*, *"the store that holds it"*.

### D22 — explanation is not product content

Three lines were **moved to the notes** rather than rewritten:

| the line | where it went |
|---|---|
| `Banister k = 1.92` | the load card's note (it is the model's constant, and it would still make sense on a card about any measurement) |
| `Riegel, exponent 1.06` | the equivalents card's note |
| `the boundaries the watch used on those days are not recorded here` | the two-bases check's note |

Two lines were **rewritten** rather than moved, because they are the product's own
fact and not an explanation of the drawing: the curve's caption (*"the plan's bands
are named here and never drawn on it"*) and the ladder's caption (*"a heart rate and
a pace are different things"*). **D22 allows a caption inside the chart's frame**,
and both stay.

### The runner test — the strings that could not pass

Read aloud, one by one. These are the ones that did not pass cleanly, and why they
were left:

| the string | why it did not pass | what was done |
|---|---|---|
| `your 10K pace × 1.20–1.33` (the long-run rung's rule) | a runner has to know that a *larger* number is a *slower* pace, and the digits are the book's | the digits are unchanged on purpose — "numbers must not change" — so the plainest true reading of the same digits was kept and the note defines it |
| `% maxVO₂` (the curve's y-axis) | a chart axis has to name its measure | kept; the caption says it in full, and the note defines it once |
| `the method's rule` → `the plan's rule` (the ladder's bands) | "the plan's rule" is still a rule, and the word "rule" is what the book's chapter calls it | kept: the *rule* is the product's fact — the band and the pace are kept apart, and the runner is told which is which |
| `the physiological-cost curve` → `the effort curve` | "the effort curve" is a coined name, not a term a runner uses | kept as the shortest true name; the note keeps the physiological name and its definition |
| `Eddington 12` → `12 km on 13 days` | the plain form gives the fact but loses the name a runner may know | the name is gone from the surface and kept in the note |
| `Z1–Z2 low`, `Z3 moderate`, `Z4–Z5 high` | "low / moderate / high" is the app's grouping of the watch's zones, not the watch's own | kept: the watch's labels stay, and the grouping is stated beside them |
| `the whole run` (the elapsed row) | on its own it does not say *stops included* | the note says it; on the two-bases card the phrase is the full `the whole run, stops included` |
| `/insights/pace-curve` (the route card) | a route is not a runner's language | the card's surface is now a link, *open the effort curve →*; the route stays in the note |

### The numbers

**No number changed.** Every figure is still read from `exploration/athlete.js`, and
the pass changed words only. Two things look like number changes and are not: the
load card's caption lost the constant `Banister k = 1.92` (it moved to the note),
and the curve's `Daniels' chain` sub-header was replaced by the race it comes from.
Both are explained above.

---

## 6 · Where the pass was applied

| file | what changed |
|---|---|
| `exploration/athlete.js` | the shared copy block (`copy.*`), the source line, the ledger's change lines, the load's scenario, the ladder's rules and legend, the curve's assumptions, the threshold's sources and rules, the plan's pacing source, the compare metric names, the year's note, and all six states |
| `exploration/end-to-end.html` | the notes' quoted strings, the notes' ban-list wording, the Today cards, the pace and curve cards, the threshold and plan screens, the Body sleep header, the heatmap card, the states appendix, and the two charts' own labels |
| `exploration/stimulus-ledger.html` | the source lines, the load card, the ledger card's blocks, shares and change lines, the desktop table and its reading, and all six states |
| `exploration/pace-engine.html` | the ladder and curve cards, the equivalents, the legends, the empty states, the aria-labels, and the rung rules |
| `exploration/threshold-setup.html` | the candidate titles and sources, the zone picture, the header card, the used frame, and the field-test frame |
| `exploration/cards-as-pictures.html` | all eleven cards' captions, labels and empty states, and the load card's title |
| `exploration/two-bases.html` | both blocks' names and bases, the check's heads and difference line, the desktop reading, and the row names |
| the six `.md` notes | the string catalogues, the quoted surface strings, and the ASCII card sketch, so the design record matches the prototypes |
