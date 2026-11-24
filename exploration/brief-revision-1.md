Parent: the brief is good — and the contradictions you listed are the right ones.
Here are the resolutions; please apply them to `docs/product-brief.md` so the
canonical document is accurate.

## 1. The pace chain — one chain, settled

**race time (typed, dated) → VDOT (the engine's own) → race-pace equivalents
(VDOT/Riegel) → LT pace (the spine's rule) → the rungs (the spine's
percentages).**

* **VDOT is the engine; the spine is the rulebook.** The spine's §5.4 chain
  ("race time → Appendix B → Appendix C") is the **book's own lookup
  tables**. We compute the same quantities instead — the pace for a 10K, a
  15K, a half — from the athlete's race time — and say so in the document:
  **our chain replaces the book's lookup tables; the book's *rules* are
  quoted and cited.**
* **The anchors are the goal race pace and LT pace**, never marathon pace.
  The pace engine's MP-offset placeholders (`MP+2:00`, `MP−0:45`, `MP+1:30`,
  `MP+3:00`) are **deleted** — they were placeholders and they are wrong for
  the method. Marathon pace may appear as one more rung, not as the anchor.
* **LT pace** = the pace for a ~15K (10-mile) race, from the same VDOT; or
  10K pace + 10–15 s/mi for a shorter-distance athlete (the spine's rule,
  cited). The pace engine thread is rebuilding the ladder on this chain now.
* **Rung values** come from the spine's table: long-run start = 10K pace
  ×1.20–1.33; long-run finish = 15K–HM pace ×1.17–1.29, or LT pace for the
  last ~3 miles of a progression long run; recovery = 15K–HM pace +120 s/mi
  with below 76 % maxHR; the easy day = **no pace at all**, only the band
  (70–81 % maxHR / 62–75 % HRR).
* **Every rung carries a heart-rate band and a pace**, and the card keeps
  them apart: the band is the method's; the pace is the athlete's own
  pace-at-HR, or the method's percentage rule — and where the athlete has no
  history at that heart rate, the pace is a **no value with a reason** while
  the band still stands.

## 2. The vocabulary is the spine's

Use the spine's **run types** — endurance, general aerobic, recovery,
progression long run, lactate threshold, VO₂max, speed, tune-up race — not
the pace engine's placeholder names ("Easy, Threshold, Long run, Recovery").
The **ladder** shows the paces subset (general aerobic, recovery, lactate
threshold, the long run's two ends, and the goal race pace); the other run
types live in the **plan's session kinds**, which is where they belong.

## 3. History and the plan — two sources, never merged

`exploration/the-wedge.md` §7.4's "history only" is **superseded by D13**
(I am correcting that file too). The resolution to state in the brief:

* the **ledger's rows** answer against the athlete's **own history**;
* the **plan card** carries the athlete's **intent** — the adopted base, its
  sessions, its pacing;
* they sit **side by side and are never merged into one number**, and the
  app never says whether the athlete did what they were told.

## 4. The surface cut supersedes `task-areas-plan.md` rule 1

The instruction to reduce the surface supersedes rule 1 ("no card is lost —
all twenty-four survive"). Say that plainly, with the user's instruction as
the reason. The inventory wins.

## 5. The Beta shows the ledger's answer, not two

Until D3's stage 1 lands, the ledger and the run detail measure a run with
different zone models. The Beta shows **the ledger's**; do not put both on
one screen.

## 6. The method lens — design done, shipping deferred

State it exactly that way: the design exists (`exploration/method-lens.md`
and its prototype), D13 defers it out of the first drop, and **its rules
still govern the rest of the product** (no verdict, no target on a shared
scale, absence as a date, picture first). A reader of `method-lens.md` alone
should not think the lens is next.

## 7. What the Beta must not claim

The spine's schedules were read cell by cell for the **half marathon 46–63**
and the **8K–10K 30–42** only. So the brief must say plainly: **the Beta
must not claim a verified weekly skeleton for any other tier** — and the
10K programme's schedule still needs reading before its template is
authoritative. (I have asked the spine thread for that; when it lands, the
brief should point at it.)

## 8. One more drift, for the cleanup list

`README.md` says "six string types in all"; `docs/contract/voice.md` says
**seven** (the `instruction` type — the run update's prompt — is missing
from the README's list). The README also still says "four destinations".
Both are README fixes; add the string count to the cleanup list's
documentation step.

## Also

* In Part 6, name the two routes the Beta needs and mark them as work:
  `/insights/stimulus` (the ledger — a cleanup step) and `/insights/paces`
  (the ladder — not yet planned anywhere). Say that the Beta is served by
  the existing rig at `/beta` and reads **read-only through the existing
  API**, because the route table is frozen and a second reader would be a
  second implementation of the store's semantics — and note that this makes
  the Beta reachable from a phone browser over the mesh, which is how it can
  be tried without shipping in the Android app.
* Keep the "contradictions" section, rewritten as "how the sources were
  reconciled" — it is the most useful part of the document for a future
  reader, and the resolutions belong in it.

## Report back

* the file updated, and the line ranges of the sections you rewrote,
* the final wording of the pace chain, in three lines,
* anything else you found that contradicts another source.
