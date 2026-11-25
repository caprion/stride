# Brief — step 4a: the plan programmes, as data (exploration only)

*Written by the parent thread, 24 November 2026. **Data and a spec — no
code changes under `crates/`, `frontend/`, `android/` or `desktop/`.** This
runs alongside the API step; it prepares the material for step 4b.*

## What to produce

### 1. `exploration/plan-templates.json` — the four shipping programmes

The **read cell by cell** schedules from `exploration/pfitzinger-spine.md`
§2.3, transcribed into data:

* the **10K programme, tier 1** (30–42 mi/wk) and **tier 2** (45–57 mi/wk);
* the **half-marathon programme, tier 1** (31–47 mi/wk) and **tier 2**
  (46–63 mi/wk).

Each programme carries:

| field | what it is |
|---|---|
| `id`, `label` | "10K tier 1", "half marathon tier 2" |
| `method` | "Pfitzinger" — the base's name, so the app can show whose method it is |
| `distance_m` | the goal distance |
| `weeks` | 12, counted down from 11 to 1, then race week |
| `entry_mileage` | the tier's entry requirement — the weekly mileage range the athlete is expected to be at |
| `weeks_detail[]` | per week: the **week number** (weeks to go), the **printed weekly volume** (in miles and kilometres), and the **seven days** |

Per day: the **kind** of run in the book's own vocabulary — *rest or
cross-training · GA · GA+speed · endurance · progression long run (last k
miles at LT pace) · LT · VO₂max · speed repetitions · recovery · race pace ·
the tune-up race* — the **distance** in miles and kilometres, and the
**detail** the spine's cells carry (for example *"3 × 8 min, jog 3"* for an
LT session, *"6 × 12 s uphill + 6 × 100 m strides"* for GA+speed, *"2 sets of
4 × 200 m @ 800 m–mile race pace, jog 200 m, 4 min between sets"* for speed
repetitions).

**Transcribe, do not summarise.** The spine's tables are the source; the
check column there reconciles each daily cell with the printed weekly volume
— **keep that check** in the data (a per-week `volume_check` field) and
report any week that does not reconcile.

**Where a cell could not be read** (the spine flags some), the day carries a
`readability` note rather than a guess.

### 2. `exploration/plan-bases-spec.md` — the spec for the core's module

How the programmes become adoptable, dated plans:

* the shape of a **template** and of a **week** and a **session** in it;
* how a template is **adopted**: the athlete picks the programme, the tier
  (the app states the **entry requirement** beside their own weekly mileage
  and the athlete chooses — the app never suggests, D8/D24), the **start
  date**, and the goal race's distance and date;
* how the **dates** are worked out from the plan's 12 weeks and the race
  week — and which week is the race week;
* how a session carries its **pace**: the rule that gives it, and the
  **threshold it was paced with** (value and date — D20), and what happens
  when the threshold changes afterwards (the plan keeps the bands it was
  given);
* what an **adopted plan** records, and what the plan card shows: the week's
  sessions, the plan's source, the pacing's threshold, the **kinds of run**
  in the runner's language, and the **entry requirement** beside the
  athlete's own mileage;
* how a **tune-up race** and the **race week** appear, and the post-race
  recovery and transition weeks;
* the **80/20 base** as a second base in the same shape — but only as a
  **spec**: its schedule is not read cell by cell yet, so say what it would
  need and do not invent its weeks.

## Rules

* **Exploration only.** Two new files under `exploration/`. No code changes
  anywhere else. No commit.
* **Transcribe, do not invent.** Every cell traces to the spine's table; the
  spine's citations travel with it. Where it is silent, the field is absent,
  not guessed.
* Miles and kilometres both, so the app can print in the athlete's own unit.
* The names a runner would recognise — *"GA+speed"*, *"endurance"*, *"LT"* —
  are the **method's** names, and the app's own wording for them (the
  vocabulary in `exploration/language.md`) is the runner's.

## Report back (last message, under ~400 words)

* the two files;
* the four programmes, each with its weeks and its printed volumes — and
  **any week whose daily cells do not reconcile with its printed volume**;
* the days you could not read, and what you put instead;
* the spec's five decisions in five lines;
* what the 80/20 base would need before it could be a second base.
