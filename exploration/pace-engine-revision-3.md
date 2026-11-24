Parent: one addition to your file — a **physiological-cost curve** panel, which
the user asked for after seeing the ladder.

## The chart

Daniels' classic curve: **%maxVO₂ against time (log scale)**, computed from
the athlete's own VDOT (the engine's own relations, already in your note).
Over it:

* the athlete's **own races** as filled dots (their time, their %maxVO₂);
* their **goal race** as a hollow dot if it is not one of the races;
* the **method's sessions** as amber segments — each placed by its
  **duration** and its **prescribed pace**, with the pace turned into
  physiological cost by VDOT. So: the lactate-threshold session (its
  duration range at LT pace), the long run (its duration range, from the
  start pace to the finish pace), the recovery run, and — if the method
  fixes a duration for them — the general-aerobic day;
* the **30-minute field test's position** as a hollow marker at 30 minutes,
  so the test is legible on the curve;
* the **pace equivalents** labelled along the bottom or right (the pace for a
  5K, a 10K, a 15K, a half at those durations).

## The one rule that matters

**Pfitzinger's bands are %maxHR and the curve's axis is %maxVO₂. They cannot
be converted without the athlete's lactate relation, which Stride does not
have.** So the method's HR bands are **quoted in the caption and the legend**,
never drawn on the %maxVO₂ axis. Do not imply a conversion, and do not draw
the bands on the curve. If the athlete's own pace-at-HR history allows a
*pace* to be shown for a session, that pace is the athlete's own (azure) and
it may be labelled — but it is not the HR band.

## Where it lives

**Not on the ladder.** The ladder is already two axes; this is a second
reading of the same athlete. Put the curve in its **own panel**, reachable
from the pace card — a route of its own (say `/insights/pace-curve`), linked
from the pace card — so nothing is crowded. The user's words: *"I don't want
to crowd it too much, but that kind of graph is informational."*

## What it must not become

* a **judgement**: no "this is too hard", no ordering, no red, no "you are
  not ready for a half" — it shows the method's sessions and the athlete's
  races on one curve, and the reader draws the conclusion;
* a **verdict about the method** — the same curve holds both bases, and the
  athlete's choice of base is named on the plan card, not here;
* a **second ladder**: no rung duplication, no pace bands repeated.

## Also

* The caption must **name the assumptions**: VDOT from the athlete's typed
  race time (with its date), the method's prescribed paces (cited), and the
  race equivalences. The ladder's own source line is the model to copy.
* The **empty states**: no race time → the curve has no value and says so;
  race time but no adopted plan → the curve still draws (the athlete's own
  races and the pace equivalents), with no method's sessions on it, and it
  says which is missing.
* Add the panel to the prototype as a new frame — phone and desktop widths —
  and add the section to `exploration/pace-engine.md`.

Same two files only; no commit; the ban list still applies to every string.
