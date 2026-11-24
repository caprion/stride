# Brief — the end-to-end: one athlete, every screen, annotated

*Written by the parent thread, 24 November 2026. **Prototype only** — new
files under `exploration/`, nothing under `crates/`, `frontend/`, `android/`,
`desktop/`, `PLAN.md` or `docs/`.*

## Why this thread exists

The user, looking at the pieces:

> **"can you pull together a complete end to end of this please. its easier
> to see all the cards populated together. and then we can get feedback or
> approve it. Also annotate it right now, on what you are showing so that i
> understand completely. We can remove this annotation or keep some in the
> actual rig."**

So: **one page that shows the whole product, populated, in order, with
annotations explaining every element** — and an annotation toggle so the same
page can be read clean afterwards.

## Deliverables

1. **`exploration/end-to-end.html`** — the whole product, one synthetic
   athlete, phone and desktop widths, every destination and every contextual
   surface, with an **annotations toggle**.
2. **`exploration/end-to-end.md`** — the walkthrough: what each screen
   answers, the annotation texts, the state catalogue, and what is **not** in
   this end-to-end.
3. **`exploration/athlete.js`** — **the single source of the synthetic
   athlete's data**, as one clearly-marked block at the top of the file
   (runs, weeks, laps, zones, races, wellness, the zone model, the plan, the
   threshold choice). A later thread will build the Beta's five screens from
   this same file, so **every screen must read from it** — no numbers typed
   into the markup. Keep it small and legible.

## The athlete (one person, used everywhere)

Take the athlete the existing prototypes already use, so the pages agree with
each other: the window **7 Sep – 24 Nov 2026**, ~58 runs, ~49 h 33 m of moving
time, the zone model read **3 Nov 2026** (max HR 183, `HR_MAX`), a **10K in
41:36 on 12 Oct 2026** as the typed race time, a half marathon as the goal,
the **Pfitzinger 10K base** adopted, a threshold chosen from the watch's zone
model, and the gap the ledger exists to show — **no run in Z4–Z5 since
15 Oct 2026**. Read `exploration/stimulus-ledger.html`,
`exploration/pace-engine.html`, `exploration/threshold-setup.html` and
`exploration/cards-as-pictures.html` for the existing numbers and reuse them
exactly.

## The screens, in the order a runner meets them

| # | screen | what it must show |
|---|---|---|
| 1 | **Today** | the source line at the top (the zone model, once); the hero (the last run); *where you stand* (this week · form · habit); the plan's sessions as **unranked options**; *last night* (the Rest summary, the three vitals, the sleep chart) |
| 2 | **Runs** | the calendar, the filters, the compare tray, FIT import, the run list |
| 3 | **Run detail** | the headline numbers, the core's story sentence, **the one chart** (pace / heart rate / altitude toggled, one cursor), the effort table (zones, *not moving*), the route, the splits, the form observations, the run update, *against the plan*, the note, the night's sleep as a **link** |
| 4 | **Progress** | the wall, the **eleven cards as pictures** (`exploration/cards-as-pictures.html` is the design), the window selector, the source line once at the top |
| 5 | **The pace card** | the **two-axis ladder** (`exploration/pace-engine.html`), and the **physiological-cost curve** in its own panel with its own route |
| 6 | **The threshold** | the four candidates side by side, the choice, the dated history, the disagreement state (`exploration/threshold-setup.html`) |
| 7 | **The plan** | the adopted Pfitzinger 10K plan: the week's sessions with their kinds and paces, the plan's source, the pacing's threshold named with its date |
| 8 | **The athlete profile** | the five cards as pictures (shape, months, schedule, habit, goals) |
| 9 | **Body** | sleep, the three signals, the patterns |
| 10 | **Compare** | the basis, the metric table, the two runs, the shared chart |
| 11 | **The year** | the months, the calendar, what changed |
| 12 | **The film** and **the heatmap** | one frame each — the deep view of a run, and the grid over the route positions |

Each screen is a **section on one page**, with a simple navigation (a contents
rail on desktop, a jump menu on phone) so the whole thing reads as one
continuous walk-through rather than eleven separate files.

## The annotations

**Every element gets a note.** Two modes, one toggle:

* **annotated** — numbered pins on the elements, and the notes in a side rail
  (desktop) or in an expandable sheet (phone). Each note says **what the
  element is**, **where it comes from** (the core function and the file), and
  **what it may not claim**.
* **clean** — the pins and the rail disappear; the product reads as the
  product.

Suggested note, to show the depth:

> **4 · The load** — acute 118 · chronic 84 · form +34 · ratio 1.40.
> *The load engine's own numbers: TRIMP over the athlete's zone model, with
> Banister k = 1.92. The scenario under it is the model's expectation, not a
> prediction. From `crates/stride-core/src/load.rs`. It may not say the
> athlete is fresh, tired or ready — the direction words are the only
> judgement allowed, and this card does not use them.*

The notes are **for the user now**; the toggle lets the same page be read
clean, and the note list tells us which ones might be worth keeping in the
real rig (mark each note **KEEP** or **DROP** in the note's own list — the
note text itself stays clean; the marking lives in `end-to-end.md`).

## The states

After the walk-through, an **appendix of states**: the same screens with
their absences — no zone model, no race time, no plan adopted, no threshold
chosen, an empty window, a run with no heart-rate data. Each state says what
it is and what it says. These are the product's honesty, so they belong in
the end-to-end, not in a separate file.

## Rules

* **Prototype only.** New files under `exploration/` only. Nothing under
  `crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/`. Do
  not commit or push.
* Plain HTML/CSS/vanilla JS, inline SVG, **no frameworks, no network**, works
  from `file://`.
* Phone and desktop widths, in the style of `exploration/information-shapes.html`
  (copy its palette and phone frame) — azure and amber, **no red**, no colour
  that judges, no score, no grade, no traffic light.
* **The ban list applies to every product string** (`docs/contract/voice.md`).
  The annotations are *explanatory text about the product*, not product
  strings — but they still must not imply a verdict or a recommendation about
  the athlete.
* **Every number comes from `athlete.js`.** If a number is missing from it,
  add it there; do not invent numbers in the markup.
* Do not re-derive the analytics: the designs exist. This thread **assembles**
  them — the ledger, the ladder, the curve, the threshold, the cards as
  pictures. Where a design's prototype is the authority, follow it.

## Report back (last message, under ~400 words)

* the three files created, and the total size;
* the screen list as it appears, and anything you had to leave out;
* how the annotation toggle works, and how many notes you wrote;
* the states you drew, and any that turned out to be hard to say without a
  verdict;
* anything you found inconsistent between two prototypes — that is valuable.
