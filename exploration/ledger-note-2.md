Parent: one change to the ledger card's design, from the user, on the source
line.

**The change:** the card does **not** repeat the model and its read date on
every row. There is **one source line for the screen** — at the top, naming
the model and the day it was read — with the details available on a click
through. The rows stay clean.

The rule this replaces (D3's "every answer names the model it used and the
day that model was read") becomes:

* **one source line per screen**, naming the model and its read date, at the
  top of the screen;
* **an answer that used a different model says so** — if a run in the window
  predates the first stored model, that run's row still says so, because a
  silent difference would be a wrong number.

The same hoisting applies to the other screens: the run detail's zone table,
the window's split, the load card's scenario. One place per screen, not one
place per answer. This is the product's "less verbose, more visual" rule
applied to provenance — the picture carries the numbers, the source line
carries the model, and the detail is one click away.

Please update `exploration/stimulus-ledger.md` and the prototype accordingly:
move the per-row model naming to a single source line at the top of the card,
keep the per-row exception where the model genuinely differs, and check the
states still name what is missing.

Same two files only; no commit; the ban list still applies.
