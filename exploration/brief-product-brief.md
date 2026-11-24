# Brief — the product brief: what Stride is, and the brief for the Beta

*Written by the parent thread, 24 November 2026. One document, high quality,
for a reader who has the repository but has not been in this conversation.*

## Why this thread exists

The user asked for **a high-quality document on what this app is really
about**, pulling through **all the features, ideas and optionality** — and
then for **"a really snappy simple application that can pull through all of
this as a separate beta"**, to be built by a coding agent (**Copilot**) that
has access to the repository. The document is the hand-off: it must be
readable on its own, and it must be the thing that agent works from.

**Write one file: `docs/product-brief.md`.** It is a canonical document (the
`docs/` folder, next to the contract and the capability notes), not a scratch
file in `exploration/`.

## What to read first

Everything the work has produced, in this order:

| file | what it gives you |
|---|---|
| `exploration/the-wedge.md` | **the choice**: the wedge, the three pillars, the decisions |
| `exploration/decisions.md` | **D1–D15**: every decision taken, each naming the threads it binds |
| `exploration/methods-and-products.md` | what runners actually ask and complain about; how the coaching methods map to computable data; the fact-only method lens (now deferred) |
| `exploration/surface-inventory.md` | every visible panel, counted, with its merge target; the cut list; the consolidated chart |
| `exploration/stimulus-ledger.md` | the stimulus ledger: taxonomy, rules, wording, states, the HR-lag caveat, the repeated-work row |
| `exploration/pace-engine.md` | race time → VDOT → marathon pace → the training paces; the ladder; the states |
| `exploration/pfitzinger-spine.md` | Pfitzinger's numbers and the 80/20 plan shape (a parallel thread is finishing it — read what is there, and say if it is incomplete) |
| `exploration/garmin-temperature-and-activity-fields.md` | what the Garmin API carries that we discard; the temperature finding |
| `exploration/cleanup-plan.md` | the ordered cleanup, every path named, with the gates |
| `docs/contract/voice.md` | **the voice**: seven string types, the closed direction set, the ban list |
| `docs/contract/core-contract.md`, `docs/contract/api-paths.md` | the frozen contract |
| `PLAN.md` | the plan of record |
| `README.md` | what the product is today, in the user's words |

The **prototypes** — `exploration/consolidated-chart.html`,
`exploration/stimulus-ledger.html`, `exploration/method-lens.html`,
`exploration/pace-engine.html` — are the visual truth for the surfaces. Look
at them; the brief should point at them by path.

## The shape of the document

### Part 1 — What this app is (about 400 words)

The essence, in prose a new reader can hold: what Stride is, what it is not,
**the wedge** (audit the negative space of your training — what the recent
weeks contain, what they do not, and what the athlete chose to follow), who
it is for (**the intermediate runner** — entry level is served by Strava and
Garmin), and the one promise that defines it: **facts, never verdicts** —
every number names its source, its period and its model, and absence is
always named, never zero.

Then a short "what it is not" list: not a coaching app, not a social feed,
not a dashboard of scores, not an account, not a subscription, not a service.

### Part 2 — The principles the product runs on (six, one line each, then a paragraph)

From `docs/contract/voice.md` and the decisions: **facts first**; **absence is
not zero**; **the picture first, the sentence second** (D11); **local-first,
no account**; **one method at a time — the athlete's** (D13/D14); **every
number names its model, its period and its source**.

### Part 3 — The objects (the data model, in plain language)

What the app holds and how each thing is *earned*: **runs** (the FIT original
retained; derived values recomputed on read, never stored); the **zone model**
(a dated fact about the athlete's own watch); **wellness observations**
(sleep, HRV, resting HR, with provenance); **race times** (the athlete's own
typed input, dated); **plans** (adopted base, dated sessions, with the plan's
pacing); **activity observations** (the source's own training effect, effort,
time in zone — coming, D10); and the **derived series**: load (TRIMP, CTL,
ATL, form, ratio, monotony, strain), time in zone, pace bands, the ledger,
the training paces, the race predictions.

For each: **where it lives** (the file path), **how it is computed** (the
function name), and **what happens when an input is missing**.

### Part 4 — The surfaces (what the runner sees)

The destinations (**Today, Runs, Progress, Body** and the contextual ones),
each with one line on what it answers; the **cut** (24 wall cards → 11; the
run detail's three charts → one; the `/pace` page retired; the radar
retired) and the **six sentence-only cards that must be redrawn** as pictures
(D11). Name the file paths for the prototypes that show each.

### Part 5 — The full inventory: shipped · in flight · optional · out

The "pull through all the features, ideas and optionality" section. **Four
lists, in tables**, each row with: the feature, what it answers, where it is
documented (path), and its state.

* **Shipped today** — the local store, Garmin sync and FIT import, the run
  detail, the load, the zone time, the pace bands, the profile facets, the
  streaks and the Eddington number, the year, the film, the heatmap, the
  plan engine, the run update with its grounding checks, compare, body, the
  local backup and export.
* **In flight** (with the thread and the file that holds the design): the
  **stimulus ledger** (thr_efgucevebk · `exploration/stimulus-ledger.md`), the
  **pace engine** (thr_crvvnnhwtz · `exploration/pace-engine.md`), the
  **Pfitzinger spine** (thr_nxc9e9nxms · `exploration/pfitzinger-spine.md`),
  the **surface cut** (thr_q6tcjhp6wm · `exploration/surface-inventory.md`),
  the **cleanup** (thr_8xwzkx39xu · `exploration/cleanup-plan.md`), the
  **bridge fix** (landed: `android/.../bridge/PhoneRoutes.kt` + the parity
  check), the **activity observations** (D10).
* **Optional — with the condition that unlocks each** (this is the
  "optionality" the user asked for; be explicit that an optional item is
  *not* a promise): the **method lens** (deferred by D13; the design exists
  in `exploration/method-lens.md` and its rules still govern the rest), the
  **environment axis** (dropped by D6 — and the measurement that closed it),
  **race predictions** (Riegel and VDOT — the engine exists, the surface does
  not), **effective VO₂max** (a number with a model version, never a
  headline), **the athlete's own effort** (from the source's `directWorkoutRpe`
  — waiting on D10), **training effect** (the source's own), **plan-vs-reality**
  (the Foster finding: runners train harder than planned on easy days), a
  **third method base**, and anything else the documents name.
* **Out of scope, and why** — a readiness or recovery score (banned and not
  validatable for one athlete); machine learning (no population dataset);
  a weather source; social features; a hosted account; a subscription; the
  5K and 8K plan programmes (cut by D15).

### Part 6 — The Beta brief (the hand-off)

This is the part the coding agent works from. It must be **complete enough to
build from**, and it must be **snappy** — the whole point is a simple,
fast app that carries the ideas, not another twenty-card wall.

Write it as:

1. **What the Beta is** — one paragraph: a **separate, minimal application**
   that embodies the product's ideas (the plan the athlete adopted, the
   ledger's absence rows, the training paces, the load) with a small screen
   set, reading the same local data, offline, with no account and no new
   dependency. Say where it lives (its own directory, its own entry point)
   and how it reads the data (**read-only** from the existing local store
   through the existing API, or a documented read-only reader — pick one and
   justify it in two lines).
2. **The screens** — four or five, each with: the question it answers, the
   picture it draws, the caption it says, and the states it must handle
   (empty, one run, no zone model, no race time, no plan adopted).
   The **ledger card** and the **pace ladder** are the centrepieces — both
   have prototypes; point at them.
3. **What it must never do** — the hard constraints, each with its source:
   no verdict, no score, no readiness, no advice, no ranking, no banned word,
   no red, no account, no network for its own sake, no new dependency, no
   stored derived value (everything recomputed on read), no implied absence.
4. **The data it reads** — a table: what it needs, where it is in the store
   today, and what it needs that does not exist yet.
5. **The acceptance criteria** — a checklist an agent can tick: e.g. "with no
   race entered, the pace ladder shows no value and says what is missing";
   "with no zone model, the ledger's intensity rows have no value and name
   the reason"; "the absence row is a date, never a list"; "every sentence
   survives the ban list"; "the app works offline with the network off";
   "no new dependency appears in `Cargo.toml` or `package.json`".
6. **How to build it in this repository** — the rules a new agent must not
   get wrong: the crate dependency DAG (`core ← ingest`, `core ← store`,
   `api → {core, ingest, store}`, `desktop → {api, store}`, `stride-ffi →
   {api, store}`) and `scripts/check-deps.sh`; the clippy lints (no unwrap,
   no expect, no panic, no todo); the frozen contract and what a change to it
   requires; the string catalogue (`crates/stride-core/src/strings.json` and
   `frontend/src/i18n/catalogue.ts` are both CI-checked and the core's
   strings are rendered by the core, not the UI); the privacy rules and the
   secret scan; "no production data, no Garmin writes, no publish"; the
   synthetic-fixture rule; the commit gates (list them).
7. **A map of the sources** — a table of every document and prototype, what
   it settles, and which part of the Beta it feeds.

### Part 7 — The open questions

What is still undecided (the destinations question was decided — five; the
zone bands, the altitude scale, the plan programmes — decided), what is
waiting on the user (the green light for the surface cut; W and Z), and what
is deliberately open (which method base the athlete picks; whether the Beta
reads through the API or directly).

## Rules

* **One file**: `docs/product-brief.md`. Do not modify anything else — not
  `PLAN.md`, not the contract, not `crates/`, not `frontend/`. (The brief
  may quote the contract's rules; it does not change them.)
* Do not commit or push.
* **Quality over length**: this document will be read by someone deciding
  what to build, and by an agent that has never seen this conversation. Every
  claim should be checkable — a path, a function name, a citation. Where a
  document is incomplete or a decision is pending, say so plainly; the
  document is more useful honest than tidy.
* Write like the product speaks: plain, specific, no hype, no adjectives
  where a number would do. **No verdict language in the document either** —
  do not write "great", "powerful", "seamless", "game-changing". The
  document describes what the app does and what it refuses to do.
* Keep the tables tight; do not pad the prose.
* If `exploration/pfitzinger-spine.md` is still being written when you read
  it, note that in the document and describe the shape of what it will
  contain, so the hand-off stays accurate.

## Report back (last message, under ~350 words)

* the file you created, and its length,
* the Beta's screen list in one line each,
* the three acceptance criteria you think matter most,
* what you could not settle from the sources, and what you wrote instead,
* anything in the sources you think contradicts another source — that is
  valuable and the parent needs to know.
