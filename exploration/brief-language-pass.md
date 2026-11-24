# Brief — the language pass on every product-facing string

*Written by the parent thread, 24 November 2026. **Prototypes only** — new
and edited files under `exploration/`; nothing under `crates/`, `frontend/`,
`android/`, `desktop/`, `PLAN.md` or `docs/`.*

## Why this thread exists

The user approved **option A** (two blocks: the app's own zone time, and the
watch's own zone minutes for the runs with no model in force) and then said:

> **"The language again - the watch's own zone minutes, elapsed time. We need
> to work on everything like that if we are showing that in the surface.
> Overall go with A. Check the language everywhere."**

So: a **language pass** over every product-facing string in every prototype.
Not a proofread — a rewrite, with one test.

## The rule

**Precision belongs in the notes. The surface speaks the runner's language.**

* The **notes** (the annotation layer, off by default) may use the precise
  terms — "elapsed time", "moving time", "the zone model's floors", "the
  source's `hrTimeInZone` field" — because a note is written for someone
  reading about the product.
* The **surface** — a label, a caption, a title, a state, a table header, a
  button, a tooltip — is written for a runner looking at their own training.
  No field names, no model names, no jargon, no internal vocabulary, and no
  phrase that only makes sense to whoever wrote the code.

**The test:** *would a runner who has never read the code understand this line
without it being explained?* If not, it is either too technical, too
abstract, or it is explanation that belongs in a note (D22).

## The strings to fix, and the shape of the fix

The worst offender is the one the user named:

| now | why it fails | the runner's language |
|---|---|---|
| "the watch's own zone minutes · elapsed time" | "zone minutes" is a field, not a thing; "elapsed time" is a statistic | something like **"the watch's own zones · the whole run, stops included"** |
| "your zone model · moving time" | "moving time" is the core's term | **"your zones · the time you were running"** |
| "3 minutes apart — the app's zone time is measured over the moving time, the watch's zone minutes over the elapsed time" | three pieces of jargon in one sentence | **"3 minutes apart — one counts the time you were running, the other the whole run."** |

The second column is a **direction, not a copy** — find the plainest true
phrasing yourself, and keep it short.

Other classes of string to sweep:

* **field and module names** — `hrTimeInZone`, `zones.models`, `zones.source`,
  "the model in force", "the source's own number";
* **statistic names** — "VDOT", "Riegel", "TRIMP", "CTL", "HRR", "Banister k",
  "monotony", "strain", "the acute-to-chronic ratio" (these may appear in
  the *notes* freely; on a surface, say what the number **is**);
* **internal nouns** — "the zone model", "the plan base", "the session kind",
  "the basis of the comparison" (a comparison's basis stays — it is what makes
  a number a fact — but say it plainly: "the same distance, 3 weeks apart");
* **sentences that explain the drawing** — these move to the notes under D22;
* **states** — every "no value" line must be readable by a runner: say what
  is missing and where it would come from, in their words;
* **plural and unit forms** — "1 h 08 m", "84%", "40 days" — the app's own
  conventions (`crates/stride-core/src/units.rs`) are the authority for the
  *format*; do not change the format, only the words around it.

## What to produce

### 1. `exploration/language.md` — the pass

* **The rule** (above), the test, and the precedent: D22 (explanation is not
  product content) and D23 (option A).
* **A table of every product-facing string** in the prototypes: the string as
  it stands, where it appears (file and surface), what is wrong with it, and
  the rewrite. Grouped by surface (Today · Runs · run detail · Progress ·
  the pace card · the threshold · the plan · profile · Body · Compare · the
  year · the states).
* **A short vocabulary** of the recurring hard words, each with its plain
  form — one place, so the whole product says the same thing: *zone model*,
  *moving time*, *elapsed time*, *the watch's own zones*, *VDOT*, *the load*,
  *form*, *ratio*, *a tier*, *the base*, *the session kinds*, and anything
  else that appears more than once.
* **What stays technical on a surface, and why** — a short list. Some things
  genuinely belong to the runner: the name of their own watch's zone, their
  race time, a pace, a heart rate, a date. Say where the line falls.

### 2. The rewrites, applied

Apply them to the prototypes: `exploration/athlete.js`'s **copy block** (the
shared strings — most of the surface copy lives there) and the **literal
strings in each prototype's own markup**. The prototypes to sweep:

* `exploration/end-to-end.html` + `end-to-end.md`
* `exploration/stimulus-ledger.html` + `.md`
* `exploration/pace-engine.html` + `.md`
* `exploration/threshold-setup.html` + `.md`
* `exploration/cards-as-pictures.html` + `.md`
* `exploration/two-bases.html` + `.md`

**Numbers must not change** — every number is read from `athlete.js`, and the
rewrites are words only. If a string is generated from a number, keep the
number and change the words around it.

### 3. The check

* **ban list**: `docs/contract/voice.md` — zero hits in the surfaces.
* **D22**: no surface line that only explains the drawing; everything
  explanatory sits in the notes.
* **the runner test**: read every surface string aloud in your head — would a
  runner understand it without an explanation? List any you could not make
  pass, and why you left it.
* The **notes** keep their precise language — that is their job. Where a
  surface string was rewritten because of a term, the note may define that
  term once.

## Rules

* **Prototypes and their copy only.** Nothing under `crates/`, `frontend/`,
  `android/`, `desktop/`, `PLAN.md` or `docs/`. No commit.
* **Words, not numbers, not structure.** Do not re-lay-out a card, do not
  move a chart, do not change a value. If a string is fine, leave it.
* The ban list applies to the surfaces **and** the notes.
* Read `exploration/decisions.md` (D11, D21, D22, D23) first.

## Report back (last message, under ~400 words)

* the files created and changed;
* **the vocabulary** — the hard words and their plain forms, in a table;
* the three rewrites you think are best, and the three you are least sure of;
* any surface string you could not make pass the runner test, and why;
* the ban-list and D22 check results.
