Parent: a rule from the user, after walking the end-to-end. It applies to all
three of your threads, so please apply it in each.

## The rule

> **"overall i think great visuals. I am seeing that we are adding text where
> it is not required."**

> **Explanation is not product content.**

The product surface carries: **the picture, its labels, the numbers, the
period, and — only where something is missing — the reason something is
missing.** Nothing else.

Every line that explains *the design* rather than *the data* moves to the
**annotation layer** — the notes, off by default in the clean view. That
includes, non-exhaustively:

* **"candidates, drawn two kinds, two lanes"** — a header that explains the
  drawing. It goes.
* **"these are numbers about different things, so they are never converted
  into one another"** — the reasoning behind the two axes. It is a **caption
  under the chart**, in the chart's own voice, or an annotation. It is not a
  header above it.
* **"it cannot know · the app cannot check what was put into the watch"** —
  the candidate's limitation. It is an **annotation about that candidate**,
  not a line on the card. The card shows the value and its source.
* anything phrased as *"these are two different numbers"*, *"the model is
  dated"*, *"measured over the moving time"* — unless it is the reason a
  value is missing. A basis line that only explains the drawing goes.

**The test:** if the line would still make sense on a card that shows a
different measurement, it is explanation, not content — cut it. If it is the
**period**, or the **reason a value is absent**, it stays.

## What stays, always

* the numbers, and the labels that carry them;
* the **period** (the window, the date range) — it is what makes a number a
  fact;
* the **source** of a value where the source *is* the fact (the race it came
  from, the model it was read with, the date it was entered);
* the **reason something is missing**, when something is missing;
* the states' one-line explanations of what is absent.

## What each thread should do

**Threshold setup** — the four candidate cards become: the value, the source,
the date. The "what it cannot know" lines move to the annotation layer, one
note per candidate. The header about two kinds and two lanes goes.

**Pace engine** — the two-axis explanation becomes a **caption inside the
chart's frame** (small, under the chart), not a header above it, and the same
sentence exists in the annotation layer. The rungs keep their rule labels —
those are the source of the number, which stays.

**End-to-end** — the same sweep across all twelve screens and the states:
every explanatory line off the product surface and into the notes. The
**annotations stay and grow** — that is where the reasoning belongs, and the
toggle now carries the whole explanation.

## Rules

* Same files only; nothing outside `exploration/`; no commit.
* The ban list still applies to everything, annotations included.
* Do **not** delete the explanation — **move** it. Every line removed from a
  product surface should appear as a note (or as a caption where it belongs
  with the picture).
* After the sweep, check each screen for lines that explain the *drawing*
  rather than the *data*, and say which ones you moved and where each went.
