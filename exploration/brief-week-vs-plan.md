# Brief — the answer the review says is missing: does this week fit the plan?

*Written by the parent thread, 24 November 2026. **Implementation.** The
review of the live rig said:*

> *"Home (/) and /progress mix Banister load ratios, streaks, sleep stages,
> and weekday comparisons (-1.9 km against 16 Nov – 18 Nov) without answering
> whether the week's training fits a goal."*

## What to build

**A card, on Progress, that answers one question: how is this week against
the plan?** Not a verdict — a **fact**, in the product's voice.

* **The plan** is now adoptable (`/plan/adopt`): a programme, a tier, a start
  date, a race, and the paces worked out once at adoption from the threshold
  in force. The plan's week is known, dated, and has its kinds of run, its
  distances and its paces.
* **The athlete's week** is known: the runs that happened, their kinds, their
  distances, and their paces where they had one.

So the card compares **the plan's week against the week that happened**:

* each planned session, **done or not done** — a fact, in the plan engine's
  own words (*done* · *not done*), never a grade;
* for the sessions that happened, **how they compared with the plan** — the
  distance against the planned distance, and the pace against the planned pace
  where both have one;
* the **week's totals** against the plan's, the way the week card already
  compares this week with last week;
* and the plan's own **source**, the **threshold the plan was paced with**, and
  the **entry requirement beside the athlete's own mileage** — unranked, as
  everywhere else.

### Where it lives

* **Progress** — it belongs beside *This week* and *The load*, and it may
  **merge with *This week*** rather than add a card: the wall is eleven cards
  and the user's instruction was to reduce the surface. Say which you did and
  why.
* **Today** — the plan's session for the day is already there; it should
  carry the same comparison for that one session.
* **Plan** — the week, in full, with the comparison.

### The states

* **no plan adopted** — the card says so, and points at the four programmes;
* a plan adopted but **the week has not started**;
* a plan adopted and **the week is in progress** — the comparison so far;
* a plan adopted and **the week is over**.

### The rules

* **Done and not done are facts** — no grade, no percentage, no traffic
  light, no "missed".
* **The plan's paces and the athlete's paces are compared only when both
  exist**; a session with no pace (the general-aerobic day has a band and no
  pace) is compared on its **band** or on nothing — never on a pace that does
  not exist.
* **The athlete's own plan** — the app never compares two plans, never says
  which plan is better, and never offers a plan the athlete did not adopt.
* **The vocabulary** (D24): *"your plan"*, *"the kinds of run"*, *"done"*,
  *"not done"*.
* The gates, as always; commit on `rig/v2`; **do not touch** the ledger's
  modules or the daytime constants.

## Report back (last message, under ~400 words)

* the card's shape and where it lives — merged into *This week* or beside it,
  and why;
* the sentences it prints, in a week's worth of examples;
* the states, and what each says;
* the gates.
