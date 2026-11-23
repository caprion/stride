# Splitting Progress into task areas

*23 November 2026. Written before the work, so the plan can be checked
against what was built.*

## The problem

The wall is **twenty-four cards in one grid**. Every card answers a
question about the athlete's history, and every card is the same size and
the same weight. The reader has to work out for themselves which cards
belong together, and there is no answer to "where do I look for X?" —
the wall is ordered by when the panels happened to be written, which is
not an order a reader can use.

The phone's index is better: six questions, each with its answer, in the
order a runner asks them. But **the phone can reach six panels and the
wall twenty-four**, so the phone is missing most of what the app knows,
and the wall has no shape.

## The shape

Two screens, one split:

| | The work | The athlete profile |
|---|---|---|
| **Question** | How is the recent training going? | What does my training look like over months? |
| **Horizon** | Weeks | Months and seasons |
| **Wall** | the work areas | the profile areas |

The split is a **split of the wall, not a split of the app**: the areas
of both sides live on Progress, in one order, and the phone's index
carries the same areas. Whether the athlete profile later becomes a
destination of its own is a separate decision, and nothing here forecloses
it — the model already says which side each area belongs to.

## The areas

Ten areas, in this order. The order is the order a runner asks in: what
is happening now, then how it compares, then the shape of the last few
months, then the habits and the seasons.

### The work

| Area | Cards | What it answers |
|---|---|---|
| **The work** | training load, weekly volume, consistency | How much am I doing, and how hard |
| **Where the time went** | time in each zone, polarised split | Where did the time go |
| **The comparison** | this week against last week, the rolling window | Is it more or less than before |
| **The pace and the results** | the pace, bests and milestones | How fast, and what is the best of it |

### The athlete profile

| Area | Cards | What it answers |
|---|---|---|
| **The shape** | the athlete profile, the training profile | What does my training look like |
| **The goals** | the challenges, the adherence to the plan | What am I aiming at, and how am I doing |
| **The schedule and the run lengths** | weekday, daytime, distance breakdown | When and how far do I run |
| **The seasons** | the year, the activity heatmap, the months, the years | What do the last months and years look like |
| **The habit** | the streaks, the Eddington number | How steady is this, really |

### Elsewhere

| Area | Cards | What it answers |
|---|---|---|
| **Somewhere else** | the most recent activities, the body | The two doors that leave Progress — to the runs, and to the body |

## The rules

1. **No card is lost.** All twenty-four survive, each exactly once. A
   test says so.
2. **No card is moved into a different panel.** Every card still opens
   the panel that answers it, by the same route, with the same test id.
3. **The order is the model's.** The areas live in one module, and both
   the wall and the phone's index read them from there. A test says so
   too.
4. **The area is a heading and nothing more.** It does not judge, does
   not summarise, and does not add a number.
5. **The wall's grid does not change.** The areas are blocks inside the
   same grid, with the same column counts and the same wide cards.

## The layout

The wall's grid is one, two, two and four columns by band. An area is a
**block that spans the wall** and holds its own grid of the same column
counts, so a wide card is still two columns of its area and the columns
still line up across the wall.

The alternative — a heading that spans the wall's own columns and the
cards flowing on after it — leaves a section with an odd number of cards
with one card alone on the last row, which reads as a mistake. The
area's own grid does not have that problem.

## The stages

| Stage | What | Where |
|---|---|---|
| 0 | The model, and the strings for the headings | `frontend/src/pages/taskAreas.ts`, `frontend/src/i18n/catalogue.ts` |
| 1 | The wall reads the model and draws the areas | `frontend/src/pages/progressWall.tsx`, `frontend/src/index.css` |
| 2 | The phone's index carries the same areas | `frontend/src/pages/ProgressIndex.tsx` |
| 3 | Every area ends in an action — the runs behind a change, a week to inspect | the panels, and the api behind them |
| 4 | The decision about a fifth destination | later |

## What is not decided here

- **The athlete profile as a destination.** The model carries the side,
  so the wall can be split into two pages without redoing anything.
- **Whether the phone's index gets the area headings.** It has six
  questions now, and each falls in an area of its own — so headings on
  the phone would be six headings of one row each. The phone's index may
  keep one list of questions and gain the areas as a way into the rest.
- **The load's own tab.** It is a card of "The work" for now, and it
  stays there until the load is understood well enough to be a place of
  its own.
