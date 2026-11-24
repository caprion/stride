# The four decisions waiting on you

*24 November 2026. One page, four questions, each with the options, the
consequence, the evidence — and a line for your answer.*

Everything below is settled in design; what is missing is your word. Write an
answer at the end of any section, or just tell the parent thread.

---

## 1 · Pass 2 — the surface cut

**The question:** may the biggest visual change to the product go ahead?

**What it does**

| | today | after |
|---|---|---|
| Progress wall cards | 24 | **11** (6 on Progress, 5 on the athlete profile) |
| Run detail panels | 13 | **10** — three stacked charts become **one** |
| `/pace` page | a page | **retired** into Progress panels |
| The radar | drawn | **retired** — the seven facets stay |
| The zone card | zones + polarised split | becomes **the ledger card** |

**What is lost:** the radar's polygon, the `/pace` page's own layout, the
wall's two calendars, its *Most recent activities* table and its *Body, last
night* card — each is duplicated elsewhere, which is why it goes.

**What is never cut:** the run's own facts, the zone table with its *not
moving* bucket and source line, the route and the film, the run update, the
plan's options, every "no value" reason, the empty states, the load's
scenario.

**See it:** `exploration/consolidated-chart.html` (the one chart) ·
`exploration/surface-inventory.md` §6 (every card, before and after, with the
words it would say).

**The plan:** `exploration/cleanup-plan.md`, Pass 2.

**Your call:** ☐ go ☐ not yet ☐ changes: ______________

---

## 2 · W — the window migration

**The question:** may the app use **one** window definition everywhere?

**The rule:** the window is the last *N* calendar weeks, from the start of the
earliest one **through today**; it is compared with **the same span ending
seven days earlier** — so the partial week compares with the partial week.

**Why:** the Progress totals and the pace page already work this way, the
rolling route does not, and the ledger needs the same comparison. Today the
three describe slightly different spans, which is how a card ends up
disagreeing with its neighbour.

**What changes in the numbers**

* the **profile's volume-a-week** and its **consistency** (the window's
  denominator becomes the window's length, which is 79 days for 12 weeks on a
  Wednesday — not 84);
* the **year's comparison** moves from a 12-month shift to a week shift.

**What does not change:** the load, the zone split, the pace bands, the
ledger's own comparison.

**See it:** `exploration/decisions.md` D1–D2 · `exploration/cleanup-plan.md`
(W).

**Your call:** ☐ go ☐ not yet ☐ changes: ______________

---

## 3 · Z — the zone model, stage 1

**The question:** may every run be measured with **the zone model in force on
the day of the run**, instead of today's model?

**Why:** the model is a **dated fact** about the athlete's watch — the
repository's own capability note already says *"today's settings cannot be
passed off as historical settings"*. Today the zones route, the run detail's
table, the load and the fingerprint all use the *current* model, so two runs
can be measured on two scales.

**What changes:** **every zone number changes retroactively** — the time in
each zone, the window's split, the ledger's rows, the fingerprint's efficiency
factor.

**What is deliberately not in this pass:** the **load model** (`load.rs` —
TRIMP's threshold and max heart rate). It changes the load history the
product has shown since v1, so it is its own change with its own
announcement, later.

**The rule in both stages:** every answer names the model it used and the day
that model was read.

**See it:** `exploration/decisions.md` D3 · `exploration/cleanup-plan.md` (Z).

**Your call:** ☐ go ☐ not yet ☐ changes: ______________

---

## 4 · The plan tier

**The question:** when the athlete adopts a plan, does the app **suggest** a
tier, or only **state the requirements** and let them pick?

**The book's own rule:** a tier is chosen by **current weekly mileage**. The
10K tiers want 30–42 / 45–57 / 60–76 miles a week and expect the athlete to
be at 25–35 / 40–50 / 55–65; the half-marathon tiers want 31–47 / 46–63 /
61–84 / 81–100 and expect 25–35 / 40–50 / 55–65 / 75–85.

**Option A — state both facts, let the athlete pick (recommended).** The plan
card says: *"the 10K tier 1 expects 25–35 miles a week; your last eight weeks
averaged 34."* Two facts, side by side, no arrow — the athlete chooses.

**Option B — the app suggests.** The plan card says *"tier 1 fits you"*.
That is a judgement the app has no standing to make, and it is the one thing
in this list that would put a verdict in the product.

**Option C — the app asks for the tier up front and says nothing else.** The
simplest, but the athlete loses the requirement that makes the choice
meaningful.

**Note:** miles or kilometres, the app's own unit — the same number the
athlete's own history is measured in.

**See it:** `exploration/pfitzinger-spine.md` §2.3 (the schedules and their
entry requirements).

**Your call:** ☐ A ☐ B ☐ C ☐ other: ______________

---

## Not waiting on you

* **One structured interval workout** on your watch, synced — it is what makes
  the repeated-work row a fact instead of "no value".
* The **spine** is reading the remaining schedules cell by cell.
* The **threshold setup** and the **pace curve** are in design.
