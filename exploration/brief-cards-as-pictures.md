# Brief — the cards as pictures (redraw the wall)

*Written by the parent thread, 24 November 2026. **Prototype only** — new
files under `exploration/`, nothing under `crates/`, `frontend/`, `android/`
or `desktop/`.*

## The complaint, and the standard

The user, looking at the ledger card prototype:

> **"the cards at this time look very verbose. we need to find a way to make
> them look like charts or other things."**

The cards are still **tables of sentences with a chart bolted on**. The
ledger card's rows read like sentences — "1 h 08 m was in Z4–Z5 — 2% of the
moving time — across 4 runs. The last run in Z4–Z5 was on 15 Oct 2026, 40
days before the end of the window." — which is the very thing D11 was written
against.

**The standard (D11):** the card is read as a **picture** first; the caption
is the caption and the accessible fallback. Now go further:

* **the picture carries the fact; the numbers are labels on the picture, not
  sentences**;
* **absence is drawn, not described** — white space, not a clause;
* **the caption says only what the picture cannot** — the period, the model,
  and the reason something is missing. It never repeats a number the picture
  already shows;
* **no card may be a list of sentences**. If a card cannot be drawn, it is
  merged or it is dropped.

## The centrepiece — the ledger card, drawn

The idea to prototype (change it if you find better, but keep the principle):

### A weeks × bands matrix

* **columns = the weeks** of the window (12 weeks, labelled sparsely);
* **rows = the bands** — Z1–Z2, Z3, Z4–Z5 — plus the shape rows where they
  earn a place;
* **each cell is filled in proportion to the time in that band that week**
  (a density, not a category) — so the mix is a picture, and a week with no
  hard running is **white space**;
* **a marker on the last filled cell of each row** — the "last date" becomes a
  position on a line, not a clause;
* **the row label carries the numbers, compactly**: `Z4–Z5 · 1 h 08 m · 2% ·
  4 runs` — set as a label, not as a sentence;
* **the change against the previous window** is a small signed figure beside
  the label (the product's own `rolling::Movement` and the muted arithmetic
  the load card already uses) — **no arrow, no colour that judges**.

The absence then reads as **the last five columns of a row being empty** —
which is the wedge, visible at a glance, before a word is read.

### The shape rows

* **the longest run** — a small bar series of the window's long runs (the top
  few, by length), the largest highlighted, with its date and length as the
  label; the previous window's longest as a faint reference line;
* **repeated work** — marks on a strip of weeks where the watch recorded a
  workout step; where the watch recorded none, the row says so in one short
  line.

### The caption

**One line, and only what the picture cannot say**: the period, and (if
needed) the reason a row is empty. Never a number already drawn.

## Then the rest of the wall — the same treatment

The cut wall has six Progress cards and five profile cards
(`exploration/surface-inventory.md` §6 lists each card and the words it would
say today). The cleanup plan flagged **six as sentence-only** — *This week*,
*Where the time went*, *Bests & milestones*, *Your shape*, *The schedule*,
*The habit*.

**Redraw every one of the eleven as a picture**, and say for each: the
picture, the caption (one line), and the states it must handle (empty, one
run, no zone model, no race time, no plan). Some ideas to start from — take
or replace them:

| card | the picture to draw |
|---|---|
| This week | seven columns, one per day, each holding the session drawn to its kind and length; the plan's pacing as the bar's own length |
| Where the time went | the weeks × bands matrix above |
| The load | the fitness–fatigue series with the **form area filled** between the two curves; today's value as a dot; the scenario as the caption |
| The weeks | the weekly bars, the typical week as a line, the days that held a run as marks under each bar |
| The pace | pace against heart rate — the athlete's own scatter, the training paces as the ladder's marks |
| Bests & milestones | a **timeline** of the distances with the personal bests marked, not a list |
| Your shape | the seven facets as seven small horizontal gauges, each with its direction arrow |
| The months | twelve columns of monthly volume, the zone mix as each column's split |
| The schedule | a **week × hour** grid, the days and times that hold runs as marks |
| The habit | the streak as a row of day marks, the Eddington as a ladder of ticks |
| The goals | the challenge weeks as a small grid of met/not met marks — never red |

**A card that resists being a picture gets merged or dropped** — say so
plainly if that happens, and which one.

## The states

Every card must state its absence: an empty window, one run, no zone model,
no race time, no plan adopted. The state is part of the card's design, not an
afterthought — and it is a **fact**, not a placeholder.

## Rules

* **Prototype only.** New files under `exploration/` only. Nothing under
  `crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/`. No
  commit.
* Phone and desktop widths, in the style of
  `exploration/information-shapes.html` and `exploration/consolidated-chart.html`
  (copy their palette and phone frame). Plain HTML/CSS/vanilla JS, no
  frameworks, no network, works from `file://`. Synthetic data.
* **Azure and amber, no red.** No colour that judges. No score, no grade, no
  traffic light.
* The **ban list applies to every string** (`docs/contract/voice.md`) —
  including the labels, the captions and the states.
* **No sentence repeats a number the picture shows.** That is the test for
  whether a card is finished.
* Read `exploration/decisions.md` (D11, D18), `exploration/surface-inventory.md`
  §6, `exploration/stimulus-ledger.md` and `exploration/cleanup-plan.md` §2.9
  first.

## Deliverables

1. `exploration/cards-as-pictures.html` — the eleven cards, phone and desktop
   widths, with their states.
2. `exploration/cards-as-pictures.md` — a short note: for each card, the
   picture, the caption, the states, and what was dropped or merged; plus the
   **one sentence you would add to D11** that this exercise taught.

## Report back (last message, under ~400 words)

* the two files created,
* for each of the eleven cards, its picture in one line,
* which card you could not draw as a picture, and what you did with it,
* the sentence you would add to D11,
* the states you found hardest to draw, and what they say.
