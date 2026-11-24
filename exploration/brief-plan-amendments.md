# Brief — the plan of record: amendments for everything we built

*Written by the parent thread, 24 November 2026. **Documentation only** — the
`PLAN.md` file and, if strictly needed, `docs/`. Nothing under `crates/`,
`frontend/`, `android/`, `desktop/`.*

## Why this thread exists

`PLAN.md` is the **plan of record**. It still describes the product as it was
before this week: the twenty-four-card wall, "zone model" as the surface's
word, no ledger, no pace engine, no threshold, no two bases.

Three commits have landed since — the window and the zone model, the surface
cut, and the vocabulary — and each travelled with a note. What has not been
written is the **whole** new direction: the wedge, the ledger, the pace
engine, the threshold, the two bases, and the two rules that now govern every
surface (**picture first** and **explanation is not product content**).

The repository's style: amendments are written as **"### Proposed amendment —
<the change> (date)"** sections, each stating **the rule it serves**, and
each marked *"Proposal, not approved. The approvals below are the user's to
give."* until the user approves it. `PLAN.md` already has a long series of
these; follow their shape and their voice exactly.

## What to write

Read these first, in this order:

* `exploration/the-wedge.md` — the choice, and the decisions;
* `exploration/decisions.md` — **D1–D25**, each naming the threads it binds;
* `PLAN.md` — the house style, and what is already there;
* `docs/contract/voice.md` — the rules a new surface must obey.

Then write **one amendment per subject**, each with: the change, the rule it
serves, what it may and may not say, the files it touches, and what is still
open. Suggested subjects:

1. **The stimulus ledger** — the card, its rows, the absence drawn as white
   space, the source line hoisted once per screen, the repeated-work row's
   dependence on the watch's workout steps.
2. **The pace engine** — the chain (race time → VDOT → the race-pace
   equivalents → LT pace → the rungs), the ladder's two axes, the
   physiological-cost curve in its own route, and the rule that the method's
   heart-rate bands are **never** drawn on a %maxVO₂ axis.
3. **The threshold** — the four candidates, the athlete's choice, the dated
   fact, and the rule that the watch's zone model and the athlete's threshold
   never merge.
4. **The two bases** — the app's own zone time and the watch's own zone
   minutes, their different bases, the overlap where the app checks itself,
   and the storage proposal.
5. **The picture-first rule** (D11) and **explanation is not product
   content** (D22) and **precision belongs in the notes** (D24) — as three
   short amendments, or one, whichever reads better in the file's voice.
6. **The plan bases** — Pfitzinger first, 80/20 as a separate second, one at
   a time, the athlete's choice, the 10K and half-marathon programmes, and
   that the plan's pacing names the threshold it was paced with.
7. **The audience** — the intermediate runner, and what that means for what
   the product offers.

Each amendment must also say **what it does not do** — the refusals are half
the value of this file.

## Rules

* **`PLAN.md` only**, unless a contract change is genuinely required (it is
  not, so far — say so if you find one).
* Follow the file's existing structure and voice. Read three existing
  amendments before you write, and match them.
* Every amendment is a **proposal** until the user approves it — that is the
  file's convention, not a formality.
* **Do not restate the whole plan.** Each amendment is a change, with the
  rule it serves. Link to the exploration documents by path rather than
  copying them.
* If an amendment would conflict with an existing one in `PLAN.md`, **say so**
  and propose the reconciliation rather than silently overriding it.
* No commit; the parent reviews.

## Report back (last message, under ~350 words)

* the amendments you wrote, by title;
* any existing amendment that your new ones contradict, and how you propose to
  reconcile them;
* anything in `PLAN.md` you think is now **wrong** and should be retired
  rather than amended;
* the questions the user must still answer to approve the set.
