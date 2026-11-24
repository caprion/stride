# Brief — simplify the surface (existing work), prototype first

*Written by the parent thread, 24 November 2026. Follow it exactly; ask if
something is contradictory.*

## Why this thread exists

Stride today shows too much. The Progress wall carries roughly twenty-four
cards, several of which say the same thing in different shapes (the load
appears on Today and again on Progress; the zone split appears on the run
detail, on Progress and again in the year; the pace appears on the run detail,
on the Pace page and again in the profile). The runner has to work out where
to look.

The user's instruction: **religiously reduce the surface area.** Concretely —
consolidate different graphs into **one**, with the series the user can
switch on and off (pace, altitude, heart rate), rather than three stacked
graphs that repeat each other.

The wedge rationale is in `exploration/the-wedge.md` (read it first), then
`PLAN.md` (the plan of record) and `docs/contract/voice.md` (the frozen voice
and ban list). `exploration/task-areas-plan.md` is the earlier split of
Progress into work areas and the athlete profile — use it, and say where you
disagree with it.

## What to produce

### 1. A surface inventory (markdown, `exploration/surface-inventory.md`)

A table of **every visible surface today** — Today, Runs, Activity (run
detail), Progress (wall and task areas), the pace page, the profile, the year,
the film, the heatmap, Body, Compare, Plan, Settings — and for each:

* what it shows (the panels, named),
* which of those panels repeat something shown elsewhere,
* the **merge target** (where that content should live after the cut),
* keep / merge / retire, with a one-line reason.

Ground this in the real code, not in memory: `frontend/src/pages/`,
`frontend/src/components/`, `crates/stride-core/src/` (the core owns the
numbers), and `crates/stride-api/src/routes/` (one route per surface). Count
the panels as they exist today — do not estimate.

Then a **cut list** in priority order: what to cut first, what can never be
cut, and what is deliberately kept even if it looks redundant (and why).

### 2. The consolidated chart prototype (one HTML file, `exploration/consolidated-chart.html`)

The centrepiece: **one chart** that carries pace, altitude/elevation and
heart rate, with the series switched **on and off** by the user.

* One shared horizontal cursor (time, or distance — say which and why), one
  crosshair, one tooltip; the series share the cursor rather than each having
  their own.
* Toggles: show/hide each series; the default state is the one a runner would
  want at a glance; say what the default is and why.
* The tricky part to solve in the prototype: **altitude is not a pace**. It
  needs its own scale, its own visual treatment (filled area, not a line),
  and it must not flatten the pace curve. Show how you handled it.
* Also handle the case the product already knows about: a stopped stretch is
  not time in a zone, and a walking lap is not a running lap (see
  `crates/stride-core/src/fingerprint.rs` and `crates/stride-ingest/src/segment.rs`).
* Phone width and desktop width side by side, in the style of the existing
  prototypes (`exploration/information-shapes.html`, `exploration/charts-first.html`
  — copy their palette and phone frame), using the Stride azure/amber identity.
* Plain HTML/CSS/vanilla JS, no frameworks, no network, works from `file://`.
  The toggles must be real controls: keyboard reachable, with a visible focus
  state, and legible without relying on colour alone.

### 3. A consolidated Progress prototype (same HTML file, separate frames)

The Progress wall after the cut: fewer cards, in the order a runner asks
(happening now → comparison → shape of the months → habits). Show which cards
survive, which merge into one card, and which disappear — and for every
merged card, what the merged card says.

Keep the product's information architecture: **Today, Runs, Progress and Body
remain the four destinations** at every width. Compare and Plan stay
contextual.

## Rules

* **Prototype only.** Do not modify anything under `crates/` or
  `frontend/src/`. Do not touch `PLAN.md`, the contract, or the schema. Write
  only new files under `exploration/`.
* Do not commit or push. Leave the files in the working tree and report the
  paths.
* Synthetic data only. No real athlete data, no network, no external assets.
* The charts must stay honest: a shared cursor, a named window, no chart that
  implies a comparison the data does not support. Do not invent a score, a
  grade, or a "good/bad" colour. Azure and amber, no red.
* Say plainly what you would **not** cut and why.

## Report back (last message, under ~400 words)

* the files you created (paths),
* the panel count today versus the count after the cut,
* the default toggle state you chose and the one chart decision you are least
  sure about,
* the three cuts you would make first,
* the open questions you need answered before real code is written.
