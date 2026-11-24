# Brief — pin Pfitzinger's numbers, and the 80/20 plan structure

*Written by the parent thread, 24 November 2026. Research only.*

## Why this thread exists

The user has chosen: **no lens in the first drop; Pfitzinger's plan first,
maybe 80/20 as well — nothing else.** We are moving the product toward
Pfitzinger.

A Pfitzinger plan is written in **marathon pace** (MP), and Stride has no
marathon pace yet — no VDOT, no race times. So the numbers of the method
have to be pinned before any of it can be computed, and every one of them
needs a source.

`exploration/methods-and-products.md` already covers Pfitzinger at §2.2 and
**refused** it as a lens precisely because its rules are stated in marathon
pace, and because "the quotable HR percentages exist only in a third-party
summary, not the book". **That refusal was right for a lens and wrong for a
plan** — a plan is followed by the athlete and paced from the athlete's own
race time, so MP is exactly what we must be able to compute.

## What to produce — `exploration/pfitzinger-spine.md`

### Part 1 — Every number a Pfitzinger plan needs

A table. One row per rule, with columns:

| the rule | the exact number | the unit it is stated in | source (author, book/chapter or published interview, date) | confidence | can Stride compute it, and what would it need |

The rules to cover:

1. **The easy days / MAFS.** The actual rule — how "slow down to feel easy"
   becomes a pace. Pfitzinger's MAFS is defined relative to **marathon pace**
   (minutes slower than MP, or a percentage slower), *not* by Phil
   Maffetone's `180 − age`. Get the real form of it, the range, and how it
   changes with the athlete's fitness or the plan's phase.
2. **Threshold training.** The threshold session's length, its frequency in
   the week, and the **threshold pace** rule relative to MP.
3. **The long run.** Its length progression, and the **long-run pace rule**
   (start slower, finish at MP — how much slower, how long to hold it).
4. **The weekly structure.** What a week contains: which days are quality
   days, which are "go easy" days, which are recovery days, and where the
   doubling / back-to-back pattern sits. How many quality sessions per week,
   and how that changes across the plan.
5. **The 12-week and 18-week plans.** Their actual shape: weekly mileage
   progression, where the long runs fall, where the threshold sessions fall,
   the cutback weeks, the peak, and the taper ("eight weeks out, three weeks
   down" — verify it).
6. **The recovery-day rule** and any rule about what the athlete may and may
   not do on recovery days.
7. **What "race week" looks like** in the plan.

**Sourcing rules, strictly:**

* Prefer the **author's own published statements**: Fast Running interviews,
  the author's own site and newsletters, published Q&A, talks, and the
  publisher's material.
* The book (*Advanced Marathoning*) is **copyrighted** — do not reproduce
  long passages. Short quotations of a single rule, with attribution, are
  fine; paraphrase and cite.
* Where a number comes from a **third-party summary** (a coach's blog, a
  forum, a plan comparison table such as the Fellrnr marathon-plan
  comparison), **label it as such**, give the link, and say it is
  unverified against the book.
* **Never invent a number.** If a rule cannot be sourced, write "not
  verified" and say what is unknown. A gap here is a real finding.
* Note where the sources disagree with each other, and where Pfitzinger's
  own editions differ from each other.

### Part 2 — 80/20 as a **plan**, not a lens

The user may also want 80/20. It was refused as a lens for two independent
reasons (the rule is stated in blood lactate; and a target and the athlete's
share cannot share a scale without the picture becoming the verdict). As a
**plan** it is a different thing. Cover:

* the structure of a polarized week (how many hard sessions, how many easy,
  rest days, the long run) and where it comes from (Seiler's 80/20, Stöggl
  & Seiler's studies, and the practical plans published from it) — with the
  same sourcing discipline;
* **how a polarized plan and a Pfitzinger plan differ in their weekly mix**,
  because the app can hold only **one plan at a time** and the athlete must
  be able to see which one they are following;
* what the athlete's own history would have to show for either plan to be
  plausible — the facts, not a verdict.

### Part 3 — What this means for the pace engine

The **input** list for a Pfitzinger plan, as a table: for each rule, the
input it needs (race time? marathon pace? a week number? the plan's phase?)
and whether that input is **in Stride today**, **derivable from what Stride
has**, or **must be typed by the athlete**.

End with the shortest honest list of **what the athlete must supply** for
the app to pace a Pfitzinger week.

## Rules

* **Research only.** Create only `exploration/pfitzinger-spine.md`. Do not
  modify anything under `crates/`, `frontend/`, `android/`, `desktop/`,
  `PLAN.md` or `docs/`.
* Do not commit or push.
* Every number carries a source and a confidence. Quotes are short and
  attributed. No invented URLs, no invented quotes, no invented numbers.
* If Reddit or another source refuses to fetch, say so and use the search
  result's text or an alternative source; never reconstruct a quote from
  memory.
* Read `exploration/methods-and-products.md` (§2.2, §2.5, §2.7, §3.4) first
  so you do not repeat that work, and `exploration/decisions.md` so you know
  what is already decided.

## Report back (last message, under ~400 words)

* the file you created,
* the **ten numbers that matter** for a Pfitzinger plan, each with its source
  and its confidence,
* what you could **not** verify, and what a reader should therefore not
  trust yet,
* how the polarized week differs from a Pfitzinger week, in three lines,
* the shortest list of what the athlete must type in for the app to pace a
  week.
