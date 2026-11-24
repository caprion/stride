# Brief — research: how runners talk about analytics, and how coaching methods map to data

*Written by the parent thread, 24 November 2026. Follow it exactly.*

## Why this thread exists

We are choosing the shape of Stride's new analysis (the **stimulus ledger** —
an inventory of what the recent training contained and what it lacked) and, in
parallel, cutting the app's surface area down. Both decisions need one thing
we do not have yet: **what runners actually say they want, and how the
established coaching methods define the same things we are computing.**

Two questions to answer:

1. **What do users say about the existing analytics products** — TrainingPeaks,
   RUNALYZE, Garmin Connect, intervals.icu, Strava — in their own words? What
   do they find genuinely useful, what do they complain about, what do they
   still have to work out for themselves?
2. **How closely do those products (and the coaching methods their users
   follow) resemble each other?** Specifically Pfitzinger, Jack Daniels,
   Maffetone, Seiler's 80/20 polarisation, and the **Norwegian methods** —
   Ørjan Ørnskæg's Run Faster Run Smarter, Freshold, and the newer Norwegian
   Singles / sub-threshold volume work.

The output is a **reference document for the parent and for the two design
threads** — not code, not a prototype.

## Method rules (important for the quality of this)

* Prefer **primary sources**: Reddit threads (r/AdvancedRunning, r/running,
  r/AdvancedRunning's monthly "training plan" threads), coach forums
  (LetsRun, Runners World, Slowtwitch, the TrainingPeaks community), and the
  product's own documentation for what a feature actually computes.
* **Quote short verbatim user language** (a sentence or two per point), with
  the URL and the date. The point of this thread is to hear how runners
  actually speak, in their words, not a summary of a blog post.
* Separate three things explicitly and label them: **published research**,
  **coaching convention**, and **forum folklore**. Many claims about intensity
  distribution and overtraining are folklore; say which is which.
* Note where credible sources **disagree** — this matters more than a single
  consensus, because our product must not pick a side.
* If a fetch fails (Reddit blocks some fetching), say so and use the search
  snippets or an alternative mirror; never invent a quote.

## Deliverable — `exploration/methods-and-products.md`

### Section 1 — What runners say (user language)

* **What they use today and why.** Which platform, for which question.
* **What they like** (name the specific feature and the specific phrase they
  use about it: "the load chart", "intensity distribution", "the pace at HR
  chart", "monotony", "decoupling", "CTL/ATL", "VDOT paces", "the plan").
* **What they complain about** (again with their words: dashboards that show
  numbers without meaning, "black box" scores, plans that do not match life,
  apps that require a subscription or an account, paywalls, privacy,
  over-claiming — e.g. "readiness" scores).
* **The questions they actually ask of an analytics tool.** This is the most
  valuable section: quote them verbatim. We expect things like "am I training
  too hard?", "is my easy run too easy?", "how do I know if I need more
  threshold work?", "why is this week different?", "is my mileage enough for
  my race?", "how do I know I'm overreaching?" — find the real ones.
* **How they check a session**: what they look at first (HR drift, pace
  drift, splitting, decoupling, "did the last rep fall apart?").
* Note anything that suggests the runner wants **comparison to a method**
  ("is this week Pfitzinger?") rather than raw numbers.

### Section 2 — The methods, mapped

One row per method, then a paragraph each. Use a table with the columns:
*method | weekly structure (days/week, types) | intensity rule | long-run rule |
recovery rule | what it asks the runner to feel | computable from HR zones + pace + time + distance alone? | needs what it needs (lactate, RPE, a coach)*.

Cover:
* **Pfitzinger** (Pfitzinger Plan / *Advanced Marathoning*): MAFS and the
  "slow down to feel easy" rule, MP/mile, threshold and threshold-pace
  training, the long run (start slow, finish faster), recovery days, the
  "eight weeks out / three weeks down" structure, and the well-known
  complaints about MAFS being too slow for faster runners.
* **Jack Daniels** (Lydian, VDOT, "run at the intensity of the workout",
  recovery / easy / steady / interval / repetition, and the *pace at HR*
  foundation — which is exactly the "pace at heart rate" chart Stride already
  has).
* **Maffetone** (MAF method, 180 bpm cap, no HR above, "no workouts with
  speed"? state it precisely).
* **Seiler's 80/20 polarisation** (three-zone model: grey, moderate, white;
  the 80/20 rule; and the 2018 Stöggl/Seiler marathon study as evidence).
* **Norwegian methods** — the current state, in two parts:
  * *classic*: Ørjan Ørnskæg's "Norwegian method" (double threshold days,
    4-week cycles of 4×4 / 2×4 / 3×4 minutes, "run at threshold 3×/week");
  * *current* (2024-2026): **Norwegian Singles** — one sub-threshold session
    a day plus easy volume, the Ingebrigtsen brothers' and Bashir Abdi's
    results, the marathonhandbook.com analysis of the actual literature
    (LT1 volume, twice-daily), and Ørnskæg's own current guidance (RPE /
    lactate control, "Fthreshold" 2:1 and 3:2 ratios).
  * Include the **criticism**: e.g. the Runner's World piece with Coach
    Bakken — "running too fast/hard is the most common mistake", and "threshold
    training at an intensity that is not threshold is worse than conventional
    training".
* For each method, mark clearly which parts a **watch alone can verify**.

### Section 3 — What this means for our stimulus ledger

* Which **stimulus kinds are method-agnostic** (derivable from the athlete's
  own zones, pace bands, time and distance: easy volume, sub-threshold volume,
  threshold-ish volume, long run, high-intensity work, recovery days) and
  which kinds exist only because a **method** defines them ("MAFS run",
  "MP run", "Fthreshold 3:2 session"). Give the taxonomy you would use.
* Where the methods **disagree** about the same run (an easy run under Pfitzinger
  is slower than under Daniels' "recovery" run; a long run under Hanson's
  marathon pace is a different session from Pfitzinger's fast-finish long run).
  Conclude whether a method-agnostic ledger is possible, and what a
  "method lens" would have to add.
* **The design constraint**: our voice contract forbids verdicts, advice and
  scores. So propose how a method comparison could be shown as *fact only* —
  for example, the athlete **chooses** a lens and the app states what the
  history shows under that lens (counts, dates, shares), or the app states the
  method's own published rule next to the athlete's own numbers, as two
  separate facts side by side. Write out two or three example sentences in
  Stride's voice that would survive the ban list in `docs/contract/voice.md`.
* Say plainly what **cannot** be derived from watch data, so we never imply
  it (lactate, RPE, terrain conditions, how the run felt, motivation,
  sleep debt beyond what is stored).

## Rules

* **Research only.** Do not modify anything under `crates/`, `frontend/` or
  `android/`. Do not touch `PLAN.md`, the contract, or the schema. The only
  file you create is `exploration/methods-and-products.md`.
* Do not commit or push.
* Every non-obvious claim carries a link. Where two credible sources
  disagree, show both and say so.
* No invented quotes, no invented statistics, no invented URLs. If you cannot
  verify something, write "unverified" and move on.
* Length: as long as the content needs, but no filler. Aim for something a
  designer and a developer can both act on in an hour.

## Report back (last message, under ~400 words)

* the file you created,
* the ten or so **verbatim user questions** you found most valuable, quoted
  short,
* the three findings that most change the stimulus-ledger design,
* the biggest disagreement between methods you found, and how a fact-only
  product can hold both,
* what you could not verify.
