# The Beta — five screens, drawn

*24 November 2026. A prototype of the Beta that `docs/product-brief.md` Part 6
hands off. New files under `exploration/` only; nothing in `crates/`,
`frontend/`, `android/`, `desktop/` was touched, and nothing was committed.*

---

## 1 · What is here

| file | what it is |
|---|---|
| `exploration/beta.html` | the Beta: the five screens, one at a time, with a nav between them (`#this-week`, `#where-the-time-went`, `#training-paces`, `#the-load`, `#the-runs`) |
| `exploration/beta-this-week.html` · `beta-where-the-time-went.html` · `beta-training-paces.html` · `beta-training-load.html` · `beta-the-runs.html` | the same five screens, one file each — the same code, one screen in each |
| `exploration/beta.js` | the drawings. Every number and every string is read from `exploration/athlete.js`; nothing is written down here, and every value is recomputed on each read |
| `exploration/beta.css` | the tokens — azure and amber only, no colour of its own |

Plain HTML, CSS, vanilla JS and inline SVG. No framework, no build step, no
network request, no new dependency. It runs from `file://`.

Every screen has **one caption** — the period, and the reason something is
missing where something is missing — and **one line of attribution** naming
where the numbers came from. There is no notes layer.

A **prototype switch** at the foot of every screen shows the states: *with
everything*, *without the race*, *without the zones*, *without the plan*
(the same states can be opened directly with `?state=norace`,
`?state=nozones`, `?state=noplan`).

---

## 2 · The five screens, and what each one answers

| screen | the question | the picture |
|---|---|---|
| **1 · This week** | what does the plan you follow say this week, and what has been run | seven day columns; each session's own length is the bar's height, its kind is the bar's fill, and what was recorded is the darker part of the same bar. Under it, the plan's pacing on one scale, each session's pace band its own bar — the pacing **is** that bar's length |
| **2 · Where the time went** | what do the last 12 weeks contain, and what do they lack | the zone split printed once as one bar, then the 12 weeks × 3 bands matrix: a week with nothing in a band is **white space**, the last occurrence carries an amber marker and its date, and the long runs run as their own series below |
| **3 · The training paces** | at what paces does the athlete train, and from what race | the ladder: heart rate on the left with your zones marked, pace on the right, the goal race's pace as the anchor line, each rung with its band, its pace and the rule it comes from |
| **4 · Training load** | how much has the training been, and where does it sit | the last 7 days against the last 6 weeks, the difference between them filled in, today's dot on each curve, the model's own scenario as the caption |
| **5 · The runs** | what did the athlete run, and when | the window's days as marks (a day with no run is white), and one run — the long run of 21 Nov — with its band line and the zone times' own line for that run |

The picture carries it: the ledger's absence is white space, the load's
difference is a filled area, the plan's week is seven columns. Every number in
a picture is also in the text beside it, and each screen carries a screen-reader
sentence with the same numbers.

---

## 3 · The rules it obeys

* **five screens, no more** — and no screen needs a scroll to be read: the
  tallest is 516 px at a 390 px-wide phone, the shortest 450.
* **the plain vocabulary** of `exploration/language.md`: *your zones*, *the last
  7 days*, *the last 6 weeks*, *the difference*, *the plan you follow*, *the
  kinds of run*, *the time you were running*.
* **one caption per screen** — the period and the reason something is missing,
  and nothing else (D22). The ladder's axis caption and the model constant stay
  in the notes layer, which the Beta does not have.
* **no notes layer** — the Beta is the plain version; `exploration/end-to-end.html`
  is where the explanations live.
* **no account, no network, no new dependency, no stored derived value** —
  everything is recomputed on read.
* **azure and amber only** — no colour that marks a problem.
* **the ban list** (`docs/contract/voice.md`) — zero hits across all five
  screens and all four states, checked word-boundary over every rendered string.

---

## 4 · The acceptance criteria from Part 6.5

| # | criterion | this prototype |
|---|---|---|
| 1 | no race entered → the ladder shows no value and names what is missing | **ticked** — the rails and the axis are drawn, with no rungs and no anchor line; the caption names the fields that can be entered |
| 2 | no zone model → the intensity rows have no value and name the reason | **ticked** — the split is an empty outline, and the runs are still drawn |
| 3 | the absence row is a date, never a list | **ticked** — *the last run in Z4–Z5 was on 20 Oct 2026 — 35 days before the end of this period*; no row counts what the athlete did not do |
| 4 | every sentence survives the ban list | **ticked** — zero hits |
| 5 | works offline, network off | **ticked** — every file is local; the pages open from `file://` |
| 6 | no new dependency | **ticked** — no `Cargo.toml`, no `package.json`, no import; the dependency gate is untouched |
| 7 | nothing is written | **ticked** — the pages read `athlete.js` and hold nothing |
| 8 | the picture comes first; every number is also in text | **ticked** — every screen carries a screen-reader sentence with the same numbers |
| 9 | every answer names its period, its model and its source | **ticked** — the period is in the caption, the model in the attribution line |
| 10 | a missing measurement is null with a reason, never zero | **ticked** — the absences are empty marks and named reasons; the only zeros are *measured* zeros (`Z4–Z5 0 m` on a run that ran no time in that band) |
| 11 | no colour marks a problem | **ticked** |
| 12 | no new route | **n/a for a drawing** — the Beta's two routes (`/insights/stimulus`, `/insights/paces`) are still work for the build |
| 13 | the ladder is on the settled chain | **ticked** — the anchor is the goal race's pace, marathon pace is one more rung, no MP offset appears, and each rung keeps its band and its pace apart |
| 14 | one answer for a run's zone time — the ledger's | **ticked** — screen 5 shows the ledger's line for the run and no second answer |
| 15 | no weekly skeleton presented as verified for an unread tier | **ticked** — the picture shows the sessions of the plan the athlete adopted, as the file holds them; no template is presented as read from the book |

---

## 5 · Where the Beta differs from the main app, on purpose

| the Beta | the main app keeps |
|---|---|
| five screens | five destinations, plus the run detail, compare, plan, settings, the year, the heatmap and the film |
| no notes layer | the notes and pins that carry every explanation |
| no settings | settings, backup, export, import, delete, and the Garmin key |
| no plan editing | the plan screen: adoption, sessions, pacing, the threshold's four sources and their history |
| nothing written | the store, the sync and the ingest |
| no account, no network of its own | the same local store, read over the loopback API |

The main app also keeps the eleven cards the wall was cut to, the effort
curve, the race equivalents, the sleep and vitals, the seven facets, the
habit, the schedule, the months and the year. The Beta drops all of them and
keeps the four things the wedge is about: **the plan you follow, the zone
times, the training paces and the load**.

---

## 6 · What did not survive the cut down to five

* **The load's curve is calibrated, not computed.** The two curves are the
  model's own shape over the athlete's runs, rescaled so that today reads the
  file's own numbers (the last 7 days 118, the last 6 weeks 84, the difference
  +34). The file's 1.40 is not reproducible from these runs under any window
  and weighting tried, so the picture and the numbers are anchored on the file
  rather than on a model that contradicts them. **The build has to settle this:
  the picture and the numbers have to be the same computation.**
* **The plan's pacing lane groups sessions.** Five days hold the same pace
  band, so they share one bar on the pace scale; the link from a bar to its day
  is the kind's colour. One bar per session would need a second lane and a
  scroll.
* **The ladder's rules are two short lines.** A rung's rule can be longer than
  the left column, so it is wrapped and, where it still does not fit, cut —
  the full rule stays in the notes layer.
* **The week strip loses the session's own pace beside each day.** The pace is
  on the shared scale below the columns instead.
* **The runs screen shows one run.** The list, the filters, the compare tray
  and the route did not fit beside the band line.
