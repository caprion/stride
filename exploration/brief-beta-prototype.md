# Brief — the Beta: five screens, snappy, from the same athlete

*Written by the parent thread, 24 November 2026. **Prototype only** — new
files under `exploration/`.*

## Why this thread exists

`docs/product-brief.md` Part 6 is the hand-off for a coding agent that will
build **a separate, minimal application** — the Beta — out of the same ideas.
Before an agent builds it, someone should **draw it**, so the hand-off is not
only words.

The user asked for it early: *"a really snappy simple application that can
pull through all of this as a separate beta."*

## What to build

`exploration/beta.html` — **five screens**, one per file, reading
`exploration/athlete.js` for every number and every string. Plain HTML/CSS/
vanilla JS, inline SVG, no frameworks, **no network**, works from `file://`.
Phone and desktop widths.

1. **This week** — the plan the athlete adopted: seven day columns, each
   session drawn to its kind and length, the plan's pacing as the bar's own
   length.
2. **Where the time went** — the zone times: the weeks × bands matrix with
   the absence drawn as white space, the last-occurrence marker, the long
   runs' series.
3. **The training paces** — the ladder: the anchor line, the rungs with their
   rules and bands, the two axes (heart rate left, pace right).
4. **The load** — the fitness and fatigue curves with the difference filled
   between them, today's dot, the model's scenario as the caption.
5. **The runs** — the history at a glance, and one run with the zone times'
   band line.

**The rules that make it snappy:**

* **five screens, no more** — and no screen may need a scroll to be read;
* **the picture carries it** — the ledger's absence is white space, the load's
  difference is a filled area, the plan's week is seven columns;
* **one caption per screen** — the period and the reason something is
  missing, and nothing else (D22);
* **the plain vocabulary** (`exploration/language.md`): "your zones", "the
  last 7 days", "the last 6 weeks", "the difference", "the plan you follow",
  "the kinds of run", "the time you were running";
* **no notes layer** — the Beta is the plain version; the end-to-end is where
  the explanations live. (A single line of attribution per screen is fine:
  where the numbers came from.)
* no account, no network, no new dependency, no stored derived value —
  everything recomputed on read.

## Also

* **`exploration/beta.md`** — half a page: the five screens, what each
  answers, the rules it obeys, and **the acceptance criteria from the
  product brief's Part 6, checked off** (no race entered → the ladder is
  empty and says what is missing; no zones yet → the zone times have no
  value and name the reason; the absence row is a date, never a list; every
  sentence survives the ban list; it works offline; no new dependency).
* Note in the document where the Beta **differs from the main app** on
  purpose: fewer screens, no notes, no settings, no plan editing — and what
  the main app keeps that the Beta drops.

## Rules

* **Prototype only.** New files under `exploration/` only. Nothing under
  `crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/`. No
  commit.
* Read `exploration/language.md`, `exploration/athlete.js`,
  `exploration/end-to-end.html` and `docs/product-brief.md` Part 6 first, and
  **reuse the drawings** — the Beta is the same pictures, fewer of them.
* The ban list applies (`docs/contract/voice.md`).
* Synthetic data only.

## Report back (last message, under ~300 words)

* the files created;
* the five screens in one line each;
* what the Beta drops on purpose, compared with the main app;
* the acceptance criteria you could tick, and any you could not;
* anything that did not survive being cut down to five screens.
