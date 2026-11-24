# Setting up the athlete's threshold — candidates, a choice, and a dated fact

*24 November 2026. **Prototype only** — no code under `crates/`, `frontend/`, `android/` or
`desktop/`. The two files are this note and `exploration/threshold-setup.html`. Nothing is
committed.*

*The surface copy in this note and in its prototype follows the language pass — `exploration/language.md`.*

*Read with:* `exploration/decisions.md` (**D3** the zone model's dated facts, **D10** the
judgements kept out, **D11** the picture first, **D13** the first drop is Pfitzinger's plan,
**D14** the two bases, one at a time), `docs/contract/voice.md` (the ban list),
`exploration/pace-engine.md` (the chain and its anchor), `exploration/pfitzinger-spine.md`
(**§5.5.1** the LT-pace rule, the book's five ways to find LT pace, and the band's correction),
`exploration/methods-and-products.md` **§2.6** (the Norwegian family — RCRS — and the rules
the research could not verify), `exploration/stimulus-ledger.md` (the ledger's rows and their
model), `crates/stride-core/src/zones.rs`, `crates/stride-core/src/load.rs` and
`crates/stride-core/src/plan.rs`.

*The six decisions of 24 November are in §13.1; this note carries them.*

*This note is the annotation layer for the prototype: the product surface carries the picture,
the labels, the numbers, the period and the reasons something is missing, and the reasoning
lives here and in the prototype's notes (§15).*

---

## 0. The line this design holds

The user's sentence was: *"We could help runners set up their current threshold pace/HR
somewhere and calculate all of this for them."*

The design keeps the first half of that sentence and declines the second, because the second
half is the app deciding for the athlete:

> **The app computes candidates. The athlete chooses one. The choice is stored as a dated
> fact with its source.**

Three negatives hold everywhere below:

1. the app never says which candidate is right, and never marks one as the one to pick;
2. the app never warns that a candidate is old, that a threshold needs retesting, or that
   anything is stale — a date is a date;
3. the app never computes a threshold (or a maximum heart rate) from an **age**. There is no
   birthdate in Stride, and no date of birth is added by this design.

So the screen is not "we calculate your threshold". It is **"here are the numbers your data
gives, and here is the one you use."**

### 0.1 What a threshold is, here

A threshold is **a value with a unit**, in one of two kinds:

| kind | unit | what it is read against |
|---|---|---|
| **heart rate** | bpm | the athlete's own zones; the 80/20 base's bands |
| **pace** | s/km | the ladder's threshold rung; a plan's lactate-threshold session |

One choice may carry **both** (a field test gives a heart rate and a pace from the same run) or
**one** (a typed number is one thing; the watch's zone model holds a heart rate and no pace).

**A pace threshold is a band, not a point.** The book's own anchor is a band — the 15K-to-10-mile
race pace, and for elite runners the 15K-to-half-marathon pace (S1 ch. 1). A "15K pace" alone
is the **fast edge** of that band. So a pace candidate carries **two numbers**, the band, and
the picture draws it **as a band** — never collapsed to a midpoint, because a midpoint is a
number the method does not state. Where a rule gives one pace (a field test's pace over its
window), the band is one number and its two ends are equal.

**The app never converts between the two kinds.** A pace candidate is not turned into a heart
rate and a heart rate is not turned into a pace, because any such conversion is a model, and a
model that speaks for the athlete's physiology is the thing this design refuses. The two kinds
sit in two lanes of the picture, side by side, unconverted.

---

## 1. The four candidates

| # | the candidate | what it is | it needs | it yields | the rule it is named by |
|---|---|---|---|---|---|
| 1 | **The watch's own zone model** | the floor of the top zone in the zone model the watch was set to | the stored zone model (already read) | a **heart rate** | the product's own convention (`load.rs`: the fifth zone's floor is the threshold) |
| 2 | **From a race** | a **band of paces**, read off a race time the athlete typed | a race observation | a **pace band** | the book's LT-pace rule (S1 ch. 1, ch. 7; spine §5.5.1) and Daniels' own statement (F4) |
| 3 | **From a field test** | an estimate from a run the athlete points at | the run's stored samples (FIT) | a **heart rate** and a **pace** | the book's 45-minute time trial (S1 ch. 1, ch. 7) — the one rule this candidate computes |
| 4 | **Typed by the athlete** | a number the athlete brings | one typed value and a picked label | a **heart rate**, a **pace band**, or both | none — the athlete's own number |

Each section below gives: what it is · what it needs · how it is computed (the formula, named)
· what it cannot know · how it is stored · how it is named in a sentence.

**The four candidates are four sources, not four rules.** Each candidate is computed by
**one** rule, and the card names that rule. For the Pfitzinger base the rule is the book's.
The app does not compute four estimates and hold them against each other — that would be the
app choosing, and it does not choose.

**What a card carries: the value, the source and the date.** Nothing else. Each candidate's
limitation is an **annotation** under the card — off by default in the clean view, because it
explains the candidate rather than carrying its number (§15). The same holds everywhere on
the product surface: the picture, its labels, the numbers, the period, and the reason
something is missing. Everything else belongs to the annotation layer.

### 1.1 The book's five ways, and where these four sit

The spine's §5.5.1 tabulates the book's **five methods for finding LT pace** (S1 ch. 1, ch. 7).
They are the same conversation the athlete is already having, and the four candidates line up
with them:

| # | the book's method | what it gives | where it sits here |
|---|---|---|---|
| 1 | **a lactate test** — six 4-minute treadmill stages with a 1-minute rest and a blood sample at each | the pace and the heart rate | **candidate 4**, with the label *lab test* — the number arrives from outside |
| 2 | **a race of about 45–60 minutes** — the book calls this a close estimate of LT pace | the pace (the 15K/10-mile route) | **candidate 2** — from a race |
| 3 | **a 45-minute time trial** on a flat course, in the conditions the book asks for — "your pace and heart rate during the last 30 minutes" | the pace and the heart rate | **candidate 3** — from a field test |
| 4 | **an estimate from training software** | the pace | **candidate 4**, with the label *another app* |
| 5 | **an estimate from heart rate** — LT pace "generally occurs at about 82 to 90 percent of maximal heart rate, which coincides with about 76 to 87 percent of heart rate reserve" | the heart rate, and the pace implied by it | **candidate 1** — the same input as the watch's own number, read through the book's own band |

Two things follow from the mapping, and both are design decisions rather than asides:

* **The book's method 5 sits beside the watch's own number, and does not become a card of
  its own.** The stored zone model carries the maximum heart rate, and the wellness readings
  carry the resting heart rate, so the book band — 82–90 % of the maximum, or 76–87 % of the
  heart-rate reserve — is computable from the athlete's own data. It is **a different number
  from the watch's zone floor** (candidate 1): one is the athlete's own configuration, the
  other is the book's band applied to that configuration. They are the same kind of thing —
  an estimate from a heart rate — so they belong to the same candidate, and there stay **four
  cards**.
* **The book's methods 1 and 4 both arrive as typed numbers.** A lactate test and another
  app's estimate are the same shape to the app: a number the athlete brings, with a label
  the athlete picks. A lactate test brings **both** a pace and a heart rate, which is why the
  typed candidate may carry a pair (§5.2).

### 1.2 The rule under the first drop — and the door left open

**The first drop uses the book's rule only.** The field-test candidate computes the book's
45-minute time trial; the race candidate computes the book's hour-pace band; the watch's own
candidate reads the stored zone model. No other rule is computed by the app in this pass.

The other published rules — Friel's 30-minute test (F1), Anderson's 30-minute estimate (F2)
and 80/20's 20-minute test (F3) — stay **named in this note as published alternatives**. A
card may say that other rules exist; it does not compute them.

**RCRS and the Norwegian family are a future base, not a rule for this one.** They are in
`exploration/methods-and-products.md` §2.6: Bakken's model, Norwegian Singles, and Ørnskæg's
"Fthreshold 2:1 and 3:2" ratios and 4×4 / 2×4 / 3×4 cycles — the last of which the research
thread could not verify from any source it reached, which is one of the reasons the spine
refused the family. The decision is that RCRS and the Norwegian family sit **with the 80/20
family as future bases**, and that **a base brings its own rule** — the way the Pfitzinger
base brings the 45-minute trial. When one of those bases ships, its rule becomes the rule its
field-test candidate computes, and the card names it.

So the door is open and nothing is closed by accident: the app computes **one** rule per
candidate now, and the rule is the base's own.

---

## 2. Candidate 1 — the watch's own zone model

### 2.1 What it is

The athlete's watch carries a **zone configuration**: five heart-rate floors and a maximum
heart rate. Stride already reads it and stores it as a **dated fact** — `hr_zone_models`, one
row per configuration change, with `read_at` (`crates/stride-store/src/schema.rs`, v3). The
model is the athlete's own configuration, not a series derived from a run.

**Which floor is which.** The five columns are the **lower bounds** of zones 1–5:
`zone1_floor` is where the first zone begins, and so on up to `zone5_floor`, which is where the
fifth zone begins; the fifth zone ends at the model's `max_heart_rate`. A reading exactly on a
floor belongs to the zone below it (`zones::zone_of` — the floor is the first value of the zone
above it).

**The threshold, in this product's own words, is the fifth zone's floor.** That is already the
convention in the code: `load.rs` reads `model.floors.zone5` as "the threshold — the lactate
threshold heart rate the model was built around". So:

| the stored column | what it is | what it is **not** |
|---|---|---|
| `zone4_floor` | the boundary **below** the top zone — the floor of the fourth zone | not a threshold; under a lactate-threshold configuration it is the start of the zone under the threshold |
| `zone5_floor` | **the threshold**, by the product's existing convention | not a measurement the app can check |
| `max_heart_rate` | the maximum the model was built on (read from the watch) | not a threshold, and never derived from an age |

**The canonical athlete's model** (`exploration/athlete.js`): `HR_MAX`, maximum heart rate
**183**, floors **115 · 128 · 143 · 159 · 165**, read on **3 Nov 2026**. The fifth floor —
**165 bpm** — is the threshold this prototype shows.

The model's own `training_method` says how the watch was configured — `HR_MAX`,
`LACTATE_THRESHOLD`, `HR_RESERVE` or `CUSTOM` — and the card shows it. Under `HR_MAX` and
`HR_RESERVE` the fifth floor is a share of the maximum rather than a threshold the athlete
measured, and the card says so in those words.

**What the watch's own threshold estimate is not.** Garmin's activity payload carries a
lactate-threshold estimate. D10 lists it among the **judgements kept out**, and it stays out.
So this candidate is never the watch's estimate — it is the floor of the athlete's own
configuration, and the athlete is the one who put that number there.

### 2.2 What it needs

Nothing to type. The model has to have been read from the watch; when it has not, this
candidate has **no value with a reason** — the same reason the rest of the product uses:
`zones.no_model` — "no zones yet — your watch's heart-rate zones have not come through".

### 2.3 How it is computed

No arithmetic. The candidate **is** `zone5_floor` of the stored model, read as the day it was
read (`read_at`). The card's source line is the product's own:

```
your zones · running · max heart rate 183 bpm · taken from your watch on 3 Nov 2026
### OLD
| `threshold.card.watch` | statement | "{value} — the start of your watch's fifth zone, from your zones read on {date}." |
### NEW
| `threshold.card.watch` | statement | "{value} — the start of your watch's fifth zone, from your zones read on {date}." |
### OLD
| `threshold.missing.base` | status | "the easy-day band is not set — the plan's bands come from a threshold heart rate, and no choice has been made for this plan. It could come from your watch's zones, a race, a field test, or a number you enter." |
### NEW
| `threshold.missing.base` | status | "the easy-day band is not set — the plan's bands come from a threshold heart rate, and no choice has been made for this plan. It could come from your watch's zones, a race, a field test, or a number you enter." |
### OLD
| `threshold.ways` | status | "four ways a threshold can arrive: your watch's zones, a race, a field test, or a number you enter" |
### NEW
| `threshold.ways` | status | "four ways a threshold can arrive: your watch's zones, a race, a field test, or a number you enter" |
### OLD
| the **80/20 plan's easy-day band** | "the easy-day band is not set — the plan's bands come from a threshold heart rate, and no choice has been made for this plan. It could come from your watch's zones, a race, a field test, or a number you enter." |
### NEW
| the **80/20 plan's easy-day band** | "the easy-day band is not set — the plan's bands come from a threshold heart rate, and no choice has been made for this plan. It could come from your watch's zones, a race, a field test, or a number you enter." |
### OLD
zones.check.heads          "your zones · the time you were running" | "the watch's own zones · the whole run, stops included"
### NEW
zones.check.heads          "your zones · the time you were running" | "the watch's own zones · the whole run, stops included"
```

### 2.4 What it cannot know

* **What was put into the watch.** The number is the athlete's own — or a formula's, or a lab's
  — and the app has no way to check it. The card states this in one line.
* **When it last was true.** The model is a configuration, read once; it says nothing about the
  athlete's fitness on any day.
* **Whether the watch was configured around a threshold at all.** A model built on `HR_MAX`
  carries no threshold.

### 2.5 How it is stored

**It is already stored** — `hr_zone_models`, dated, one row per change. The candidate is a
**read** of the stored model, not a new row. When the athlete accepts it, the *choice* is
stored (§6.2) and the model stays where it is.

### 2.6 How it is named in a sentence

> **165 bpm — the floor of your watch's fifth zone, from the zone model read on 3 Nov 2026.**

with the model's own line under it, and the limit as an annotation:

> *The app cannot check what was put into the watch.*

And the **one action**: a single control on the card that writes the choice — no typing, no
confirmation dialog, nothing to fill in. The card then marks itself as the one in use.

---

## 3. Candidate 2 — from a race

### 3.1 What it is

A **band of paces**, derived from a race time the athlete typed, by a named rule.

### 3.2 What it needs

A race observation — distance, time and day — the way `exploration/pace-engine.md` §3.2
proposes to store it (`race_observations`, proposed; not in the code yet). Stride never invents
a race time and never infers one from training.

### 3.3 How it is computed

**The book's rule** (S1 ch. 1 "Lactate threshold", ch. 7 "Lactate Threshold (Tempo) Runs";
spine §5.5.1), in the book's own words:

> "For experienced runners, LT pace is the pace you could race for about an hour."
> "For faster runners, your LT pace is approximately your race pace for 15K or 10 miles, and
> for elite runners LT pace typically is between 15K and half marathon pace."
> "If your experience is mostly with shorter races, LT pace is generally 10 to 15 seconds per
> mile slower than 10K race pace or 20 to 30 seconds per mile slower than 5K race pace."

**Daniels' own statement of the same rule** (F4, *Daniels' Running Formula*, adapted for
Runner's World): *"Most runners can figure that their threshold pace is equal to a pace they
could race at for 50 to 60 minutes."*

So the chain is the pace engine's chain with the book's last link, and the last link is
**three branches**, chosen by the distance the athlete races:

| the athlete's race distance | the branch | what the candidate carries |
|---|---|---|
| 15K or 10 miles | the **15K-to-10-mile race-pace band** (for elite runners, 15K-to-half marathon) | a band between two equivalent race paces |
| shorter-distance racing (5K, 10K) | the **offset**: 10K pace **+10–15 s/mi**, or 5K pace +20–30 s/mi | a band, because the offset is itself a range |
| — | the **race's own duration**: the book's method 2 is "a race of about 45–60 minutes" | the app says whether the typed race falls in that window |

The band, in the first branch, is **the 15K pace and the 10-mile pace** — two numbers from the
athlete's own race time (the engine's equivalents), and the band sits between them, slightly
faster than the 15K pace for most athletes, whose 15K is a little over an hour (spine §5.5.1).

**Worked example (the canonical athlete).** The athlete's 10K in 41:36 on 12 Oct 2026, typed on
13 Oct 2026 and changed on 3 Nov 2026; **VDOT 49.7**. The engine computes the equivalents from
that race by Riegel's exponent (1.06): **1:03:56** for the 15K, at **4:16 /km**, and **1:08:53**
for the 10-mile, at **4:17 /km**. The candidate carries the band **4:16–4:17 /km**.

The heart-rate band the book's LT session runs in sits beside it: **80–91 % of maxHR, 75–88 % of
HR reserve** (S1 ch. 7), and the narrower **82–90 % maxHR / 76–87 % HRR** of ch. 1.

### 3.4 What it cannot know

* **The athlete's day.** A race carries its own conditions — heat, terrain, wind, what was
  eaten that morning.
* **The heart rate.** A race gives a pace. It gives no heart rate, and the app does not
  manufacture one from it.
* **Which branch applies.** The rule branches on the distance the athlete races; the app uses
  the distance of the race the athlete typed, names the branch it used, and says the band is
  the route to the hour pace rather than the hour pace itself.

### 3.5 How it is stored

Not stored as its own row: the race observation is stored (with its provenance), and the band
is **derived from it** — the same way the pace engine's rungs are derived and never stored.
When the athlete chooses this candidate, the choice row carries the race's id, the day of the
race, and the branch that ran.

### 3.6 How it is named in a sentence

> **4:16–4:17 /km — threshold pace from your 10K in 41:36 on 12 Oct 2026: the band between your
> 15K pace (1:03:56) and your 10-mile pace (1:08:53), by the book's rule — the pace you could
> race for about an hour.**

with the limit as an annotation:

> *A race is a race, not a test — the pace carries that day's conditions.*

The rule clause is the **amber** part (it is the method's rule); the race time, its day and the
two equivalent times are the **azure** part (the athlete's own data).

---

## 4. Candidate 3 — from a field test

This is the candidate that needs the app's help.

### 4.1 What it is

An **estimate**, computed from a run the athlete already has, by a **named rule** that the app
states in the sentence.

### 4.2 What it needs

* A **run** the athlete points at — a synced activity with its stored samples. The app does not
  choose the run: the athlete points at it, because "this run was a test" is a fact only the
  athlete holds.
* The run's **stored samples** — the decoded heart rate and pace the importer already holds
  (`routes::series::zone_samples` gives the moving samples with their heart rates).
* Enough of the run for the rule's window to exist. When the window is short of the rule's,
  the app says so as a fact and still computes what the rule's window covers.

### 4.3 The rule, and the published alternatives

**The rule this candidate computes is the book's** (S1 ch. 1, ch. 7; spine §5.5.1) — the
book's third way of finding LT pace: a **45-minute time trial** on a flat course, and the
pace and heart rate **over the last 30 minutes**.

The other published rules are **named here, as published alternatives**. The card does not
compute them, and the app does not hold them against the book's number and pick: that would
be the app choosing. A card may say that other rules exist.

| the rule | what the athlete runs | the number it produces | source | confidence | in the first drop? |
|---|---|---|---|---|---|
| **the book's 45-minute time trial** | 45 minutes, on a flat course, in the conditions the book asks for | **the pace and the heart rate over the last 30 minutes** | S1 ch. 1, ch. 7 | **[book]** | **yes — the one rule computed** |
| **Friel's 30-minute test** | solo, all out, 30 minutes; the lap button at 10 minutes | **average heart rate over the final 20 minutes** ≈ LTHR | F1 — Joe Friel's own writing, joefrieltraining.com | **[author]** | named as an alternative |
| **Anderson's 30-minute estimate** | the same 30 minutes | **threshold pace = distance covered in the 30 minutes ÷ 1800 s** | F2 — Owen Anderson, Ph.D., Runner's World | **[3rd]** (editorial; it quotes the ECU research) | named as an alternative |
| **80/20's 20-minute test** | 15 minutes of warm-up, then 20 minutes as hard as sustainable | **average heart rate of the final 15 minutes** = LTHR; the distance in 20 minutes = threshold pace | F3 — Matt Fitzgerald, *Intensity Guidelines for 80/20 Running* | **[product documentation]** | named as an alternative |

Each rule is named with its own author and its own window. When a base brings its own rule —
as the 80/20 and the future Norwegian bases will (§1.2) — that rule becomes the one its card
computes, and the card names it.

**The book's method 5** — an estimate from heart rate — is not in this list, because it is not a
run: it is the book band applied to the athlete's own maximum and resting heart rates, and it
sits with candidate 1 (§1.1).

### 4.4 How it is computed

Three numbers, each from the stored samples, each over the window **its rule names**:

```
window W = the last 30 minutes of the run          (the book's rule)

avg_pace(W) = the distance covered in W ÷ the moving time in W      (seconds per km)
avg_hr(W)   = the mean of the moving heart-rate samples in W
hr_drift(W) = the mean heart rate of W's second half − the mean heart rate of W's first half
```

and, for the alternatives, the windows they name: the final 20 minutes of a 30-minute run
(F1), the final 15 minutes of a 20-minute run (F3), the whole 30 minutes (F2).

**The drift is a fact, not a verdict.** It is the app's own drift convention — the second
half's mean minus the first half's mean, in beats per minute — and it is the evidence for the
rule's own assumption that the effort was steady across the window. A drift that is small and
a drift that is large are both printed; neither is coloured and neither is named as a problem.

**The rule's assumption, in the sentence.** The rule assumes a **steady effort across the
window** — a 45-minute time trial on a flat course in the conditions the book asks for, not a
run that eased off. The drift line is the number that speaks to it.

### 4.5 What it cannot know

* **The run's conditions.** The book states the conditions the test is run in — a flat course
  and settled weather — and they are part of the number. 80/20's own words (F3): *"your
  Lactate Threshold Heart Rate test result is specific to the environment and conditions in
  which you test"*. Heat, humidity, time of day, terrain and wind are in the number and are not
  visible in it.
* **The athlete's pacing.** Friel's own words (F1): a test run with other people, or as part
  of a race, *"will change the outcome. Your number will be too high"*, and a solo 30-minute
  effort runs *"about 5% slower"* than the same effort in a race. The estimate inherits the
  effort the athlete actually ran.
* **The watch's heart-rate accuracy.** The estimate is the average of the samples the watch
  recorded. The app does not adjudicate the sensor; the run's own samples are the input.
* **The window's own steadiness.** A run that faded, or one that started too fast, gives a
  different number from a run that was held even. The drift line is what the app can say about
  it, and it says that and nothing else.
* **How the test compares with a race.** ECU's Joe Houmard, quoted by F2: *"the 30-minute test
  is a workout, not an all-out race preceded by a taper and performed in a competitive
  setting"* — it yields a true running speed at lactate threshold, but the computed speed
  *"will not be as fast as the one achieved during a 30-minute race"*.
* **How repeatable it is.** Friel's own words (F1): the number *"will vary a few beats per
  minute from one test to the next"*.
* **A test run is a run.** It goes into the ledger like any other run — in the zone rows, in
  the window, in the history. The app does not mark it exempt, and the fact that it was a test
  is not a column of the ledger.

### 4.6 How it is stored

The **run** is stored already (the activity and its samples). The estimate is **derived** and
never stored on its own; when the athlete chooses this candidate, the choice row carries the
activity's id, the day of the run, the rule's id, and the window the rule names.

### 4.7 How it is named in a sentence

> **166 bpm · 4:22 /km — an estimate from your 45-minute time trial on 3 Mar 2026: your pace
> and heart rate over the last 30 minutes (the book's rule).**

with the limit as an annotation:

> *The estimate carries the run's conditions, the effort across the window, and the watch's
> heart-rate accuracy; the rule assumes a 45-minute time trial on a flat course.*

The rule clause is **amber**; the run, its day and its samples are **azure**.

### 4.8 What the card points at, and what it does not draw

The field-test card does not restate what the physiological-cost curve already shows. On that
curve — **%maxVO₂ against time**, from the athlete's own VDOT — the athlete's **own races sit
as dots** and the **time trial sits as a marker at 45 minutes**. The curve carries the *meaning*
of the estimate ("a 45-minute effort costs this much"); the card carries the **estimate
itself**: the rule named, the number, the date, its limits. The card points at the curve; it
does not redraw the curve and it does not paraphrase it.

**The method's heart-rate bands are never drawn on that curve.** %maxHR and %maxVO₂ are not
convertible without the athlete's lactate relation, and Stride does not have one — so a band
of the method's, laid over the VO₂ axis, would be a number the app has no ground for. Where
the estimate is shown beside heart rates, it is on the **heart-rate axis of the athlete's own
history** — the athlete's own runs at those durations and the heart rates the athlete's own
watch recorded in them — which is the athlete's own data, in the athlete's own colour.

The register is unchanged by any of this: the estimate stays **an estimate from a named rule**,
with the rule's formula and its limits in the sentence, in the same register as the
race-derived LT-pace candidate, and as clearly **not a verdict**.

---

## 5. Candidate 4 — typed by the athlete

### 5.1 What it is

A number the athlete brings: from a laboratory, from a coach, from another app, from a
book's table.

### 5.2 What it needs

One or two values, and **one label the athlete picks** from a closed list — not free text the
app interprets. The labels: `lab test` · `coach` · `another app` · `book or table` ·
`my own number`. The app stores the label and shows it; it never parses it.

**A typed candidate may carry a pair.** The book's LT is a **pace band** and the watch's is a
**heart rate**, so the typed candidate can be either one or **both**: a heart rate, a pace
band (two ends), or a heart rate and a pace band together. The app stores whichever the
athlete entered and shows each one with its own unit; a missing kind is a missing kind, not a
zero.

The book's **lactate test** (method 1) and its **training-software estimate** (method 4) both
arrive here — the same shape to the app: a number from outside, with a label. A lactate test
brings the pace and the heart rate together, which is the pair.

### 5.3 How it is computed

No arithmetic at all. The value is the athlete's.

### 5.4 What it cannot know

* **Where the number came from.** The app cannot check a lab's protocol or a coach's method.
  It records the label and shows it beside the number, every time the number is used.
* **Whether the number is a heart rate or a pace** — the athlete says which, by picking the
  unit for each value entered.
* **Whether it is current.** The app says the day it was entered and the day it was last
  changed, and stops there.

### 5.5 How it is stored

As a **choice row** (§9) with the value, the unit, the label, `entered_on` and `updated_on`.
It is not a run, a race or a model, and it never joins those tables.

### 5.6 How it is named in a sentence

> **168 bpm — entered on 3 Mar 2026, from a lab test.**

and, when the athlete brings both kinds:

> **168 bpm · 4:06–4:12 /km — entered on 3 Mar 2026, from a lab test.**

with the limit as an annotation:

> *The app cannot check what the number was taken from.*

---

## 6. The flow

### 6.1 The list: side by side, never ranked

The candidates are listed as **cards**, each carrying:

* the **value** — big, in the card's own unit; a pace band as a band;
* the **source** — where the number came from, and the rule's name where a rule computed it;
* the **date** — the day of the fact the number came from (the model's read day, the race day,
  the run's day, the day it was entered).

The card's **limitation** is an annotation under it, not a line on it (§15).

There is **no order of merit** — no ranking, no ordering by "accuracy", no chip that names one
as the one to take, and no colour that judges. The card order is fixed by the product (the
watch's own, the race's, the field test's, the typed one) and is not a preference.

**The picture first** (D11): the candidates also appear as **two lanes** — a heart-rate lane
and a pace lane — with each candidate as a mark on its own lane. A pace band is drawn **as a
band**, its two ends labelled, never as a point and never as a midpoint. Two marks in one lane
are two numbers about the same thing, and the distance between them is drawn. Two marks in
different lanes are two kinds of number, and no lane converts into the other.

### 6.2 The pick

One action. The athlete presses the control on the card they want:

1. the card is marked as **the one in use** — a mark and a word, not a colour that judges;
2. a **choice row** is written (§9) carrying the value (or the band's two ends, or both), the
   source, the rule (where one computed it), the origin date, the **base it was made for**,
   and the day the choice was made;
3. every answer that depends on the threshold now answers from that row, and each one names
   the source it used.

The other cards stay exactly where they are, with their values, their sources and their
dates. Choosing one is not deleting the others.

**One threshold at a time, dated, and tied to the base.** The choice row names the base it
was made for — the Pfitzinger plan, the 80/20 base — because the two read the threshold
differently. If the athlete adopts another base later, that base may come with **its own
choice**; the earlier choice stays in the dated history, with its base named. The app holds
one plan at a time (D14), and one threshold in force at a time.

### 6.3 Later changes are facts, not alarms

A change writes **another row**. The previous row stops being in force; it is not deleted and
not edited. The history card then reads as a list of facts:

> *Your threshold on 3 Nov 2026 was 165 bpm. On 3 Mar 2026 it was 172 bpm.*

The app does not say that the threshold has changed, does not ask whether the old value still
applies, and does not mark any row as old. The dates do that work.

### 6.4 A disagreement is shown, not resolved

If the watch's fifth-zone floor says 165 and the typed number says 168, both cards are on the
screen, each with its source and its date, and the two marks stand on the heart-rate lane at
the two values. The app:

* does **not** average them, reconcile them, or pick between them;
* does **not** say which one is right, and does not label the distance between them as a
  problem — the distance is drawn and nothing is written about it;
* does **not** hide the unchosen one after a choice is made. Both remain visible, with the
  chosen one marked as the one in use.

### 6.5 The setup may be skipped

Skipping is a normal state, and it is a **state with a reason**, not an error:

* nothing is stored; no row is written;
* every answer that depends on a threshold is a **no value with a reason**, and each reason
  names what is missing **and where it would come from**:

| the answer | what it says when no threshold is chosen |
|---|---|
| the **80/20 plan's easy-day band** | "the easy-day band is not set — the plan's bands come from a threshold heart rate, and no choice has been made for this plan. It could come from your watch's zones, a race, a field test, or a number you enter." |
| the **plan's lactate-threshold session** | with no threshold chosen for the base, the session is paced from the method's own rule on the plan's anchor race; when the plan has no anchor race either, "the session's pace has no value — it is paced from a threshold pace, and none has been chosen, and the plan carries no race time." |

The **ladder's threshold rung** is not on this list: it answers from the method's own chain on
the race time (§7.2), and when there is no race time it names that missing input the way the
pace engine does.

* the setup card itself names the four ways a threshold could arrive — the watch's own zone
  model, a race, a field test, a typed number — and states that the athlete can come back to
  it. There is no prompt to choose, no empty slot with a warning colour, and no statement that
  the athlete is missing something.

---

## 7. Where the threshold is used — and where it does not go

### 7.1 The rule in one line

> **The watch's zone model and the athlete's threshold are two different numbers. Each is
> named where it is used, and the app does not merge them.**

The zone model is the **watch's configuration**, read once and dated; the threshold is the
**athlete's own choice**, dated. They can be the same number. They can be different numbers.
The app never makes them the same number by editing one, and never substitutes one for the
other.

**And the athlete's threshold is read in two places only:** the **plan's pacing** and the
**80/20 base's bands**. Everywhere else the app keeps the watch's own numbers.

### 7.2 The ladder's threshold rung

The ladder (`exploration/pace-engine.md` §7) draws the paces as rungs against the anchor, and
the anchor is the **race time**. The threshold rung is the **method's rule applied to that
anchor** — the book's hour pace, the 15K-to-10-mile band or the shorter-distance offset
(S1, spine §5.5.1) — so the rung is a **band**, drawn with its two ends, the way the book's
own anchor is a band.

**The canonical rung** (`exploration/athlete.js`, the ladder's threshold rung): the
**hour-pace band**, **4:16–4:21 /km**, with the heart-rate band **146–167 bpm** — the 15K pace
to the half-marathon pace, both computed from the typed 10K by the same chain.

**The ladder does not read the athlete's chosen threshold.** The threshold is the athlete's
number, and the ladder is the method's chain; the app does not put one in place of the other
here. Where the athlete has chosen a threshold, it lives on the candidates card, with its
source and its date, and the plan card names the threshold **it** was paced with (§7.4).

### 7.3 The 80/20 base's easy-day band

80/20's seven zones are set **from the athlete's threshold**, not from a percentage of a
maximum heart rate (F3): Z1 72–81 % of LTHR, Z2 81–90 %, **X 90–95 %** (the gray zone, the one
the base avoids), Z3 95–100 %, Y 100–102 %, Z4 102–105 %, Z5 over 105 %; the pace bands run
60–76 %, 76–87 %, 87–93 %, 93–100 %, 100–102 %, 102–115 %, over 115 % of threshold speed.
The base also defines the threshold itself: *"your LTHR is the highest heart rate you can hold
for about an hour"* (F3) — the same hour Pfitzinger's LT pace is built on (S1), which is worth
noting because the two bases agree on what the word means.

So the base needs **two** numbers: a **threshold heart rate** for the bands and the gray-zone
boundary, and a **threshold pace** for the pace bands. Each is a no value with a reason when
the choice does not carry it.

Note that the first drop is Pfitzinger's plan (D13) and 80/20 is the second base (D14). This
section describes what the base needs when it arrives; the threshold it needs is the same
threshold, and the storage is the same.

### 7.4 The plan's lactate-threshold session

The plan's sessions carry a **pace band** (the store's `plan_sessions.pace_low_s_km` /
`pace_high_s_km`; the core's `PlanSession` holds no pace yet — §8). The lactate-threshold
session's pace band is read from the **chosen threshold's pace band**.

**The plan names the threshold it was paced with — the value and the date.** A plan adopted
in March and still running in July is paced by March's number, and the card says so: which
threshold, which value, and the day it was chosen. When the athlete changes the threshold,
the sessions **from the day of the change** take the new band, and the sessions before it
keep the band they were given.

**The canonical athlete's plan** (`exploration/athlete.js`): the Pfitzinger 10K plan, adopted
30 Sep 2026, anchored on the 10K in 41:36 on 12 Oct 2026, and **paced with the threshold the
athlete chose from the watch's zone model — 165 bpm, 3 Nov 2026**. The threshold carries a
heart rate and no pace, so the session's pace band is the **method's hour-pace band**,
**4:16–4:21 /km** (146–167 bpm), and the card names the threshold it was paced with.

When the chosen threshold carries a pace band of its own, the session is paced at **that**
band. When there is no chosen threshold for this base, the session's pace comes from the
**method's own rule** on the plan's anchor race (§8.1); when the plan has no anchor race
either, the pace is a **no value with a reason** and the session itself still stands: date,
kind and target distance are unaffected (§8).

### 7.5 The ledger — where it does not go

The ledger's rows answer against the athlete's **own stored zone model** — the model in force
on the day of the run (D3), read from `hr_zone_models`. The ledger's zone rows, its band
shares, its absence rows and its coverage lines are about **the watch's zones**.

**The chosen threshold does not enter any of them.** A threshold chosen on 3 March does not
re-band a run from November, does not change the window's split, does not fill an absence row,
and does not move a band edge. If the ledger's numbers and the base's bands are drawn together
anywhere, they are two lanes, each labelled with what it is.

There is one more place the word "threshold" already lives in the product, and it is worth
naming: **the load model** (`load.rs`) reads its threshold as the **zone-5 floor of the stored
model** — `let threshold = model.floors.zone5;`. That is the watch's model, not the athlete's
choice, and **it stays that way in this pass**: the athlete's chosen threshold does not feed
the load, the load history does not move because an athlete chose a different number, and
every load figure names the model it used. The load model's own inputs are examined in
**stage 2** of the zone-model migration (D3), as a separate change with its own
announcement.

---

## 8. The plan and the ledger — the short section

### 8.1 Where a threshold session's pace comes from

In `crates/stride-core/src/plan.rs` today:

* `PlanSession { date, kind, target_distance_m }` — a dated intent, with `SessionKind` =
  `Easy | Long | Quality | Rest | Other(String)`. The book's own vocabulary (general aerobic,
  lactate threshold, VO₂max, …) waits to be added as named kinds, per the spine's §4.3.
* `plan_sessions` already carries `pace_low_s_km` and `pace_high_s_km` (NULL = unbounded) in
  the store, but nothing fills them and the core type cannot hold them.

So a lactate-threshold session's pace comes from **the plan's pacing rule**, and the rule
reads its anchor from the athlete's own number — as a **band**, which is what the store's two
columns are for:

```
LT session pace band  ←  the chosen threshold's pace band      (the athlete's own number)
                       ←  the book's hour-pace band from a race time   (the method's rule)
```

**When both are there, the plan is paced with the athlete's own number**, and the card names
that threshold, its value and the day it was chosen. The method's band is the plan's rule
when the athlete has chosen nothing for that base.

**The canonical athlete** (`exploration/athlete.js`): the plan is the Pfitzinger 10K plan,
adopted 30 Sep 2026, anchored on the 10K in 41:36 on 12 Oct 2026, and paced with the
threshold the athlete chose from the watch's zone model — **165 bpm, 3 Nov 2026**. The
threshold carries no pace, so the sessions take the **hour-pace band, 4:16–4:21 /km**
(146–167 bpm) — the method's rule — while the card names the threshold the plan was paced
with.

**The plan names the threshold it was paced with — the value and the date.** A plan adopted in
March and still running in July is paced by March's number, and the card says so. When the
athlete's threshold changes, the sessions **from the day of the change** take the new band,
and the sessions before it keep the band they were given — the same discipline the pace engine
applies to a changed race time (`exploration/pace-engine.md` §8) and the same discipline D3
states for the zone model ("today's settings cannot be passed off as historical settings").

**Without a threshold for the base**, the lactate-threshold session is paced from the
**method's own rule** on the plan's anchor race — the book's hour-pace band, named with its
source. When the plan has no anchor race either, the session still stands — it has a date, a
kind and a target distance — and its **pace is a no value with a reason**: *"the session's pace
has no value — it is paced from a threshold pace, and none has been chosen, and the plan
carries no race time."* The session is not dropped, and the plan's other sessions are
unaffected.

### 8.2 What the 80/20 base needs from the threshold, and what it does without one

| the base needs | from | without a threshold |
|---|---|---|
| the seven heart-rate zones and the gray-zone boundary (X = 90–95 % of LTHR) | the chosen **threshold heart rate** | the bands have **no value with a reason** — the base's zones are set from a threshold heart rate, and none has been chosen |
| the seven pace bands (60–76 % of threshold speed and up) | the chosen **threshold pace** | the pace bands have **no value with a reason** — the bands are set from a threshold pace, and none has been chosen |
| the weekly structure (two hard sessions, the gray zone avoided) | the method's own rule | unaffected — the structure is the method's |

The base is not blocked by an absent threshold: it is **unbanded**, and says so. The sessions
on the calendar are the method's; the bands that pace them are the athlete's number.

---

## 9. Storage sketch (proposed — not implemented here)

A new table beside `hr_zone_models` and the proposed `race_observations`, holding **the
choice**, not the candidates:

```sql
-- The threshold the athlete uses: one dated fact per choice, with its source.
-- The candidates are computed and never stored; this table holds what the
-- athlete picked, and the history of the picks.
CREATE TABLE threshold_observations (
    id               INTEGER NOT NULL PRIMARY KEY,
    hr_bpm           REAL,                    -- NULL when the choice carries no heart rate
    pace_low_s_per_km REAL,                   -- NULL when the choice carries no pace
    pace_high_s_per_km REAL,                  -- NULL when the choice carries no pace
    source           VARCHAR(24) NOT NULL,    -- 'watch_zone_model' | 'race' | 'field_test' | 'typed'
    rule_id          VARCHAR(48),             -- the named rule that computed it; NULL for 'typed'
    base             VARCHAR(24),             -- the base the choice was made for: 'pfitzinger' | 'eighty_twenty'
    origin_date      VARCHAR(10),             -- the day of the underlying fact (model read, race, run)
    origin_ref       VARCHAR(64),             -- the id of the model / race / activity it came from
    label            TEXT,                    -- the athlete's own label; 'typed' only
    entered_on       VARCHAR(10) NOT NULL,    -- the day the choice was made
    updated_on       VARCHAR(10) NOT NULL,    -- the day the row was last changed
    in_force         INTEGER NOT NULL DEFAULT 0  -- 1 = the threshold in use now
);
```

* **At most one row carries `in_force = 1`** — the same discipline as `race_observations.anchor`.
  One threshold at a time; when the athlete adopts another base, that base may bring its own
  choice, and the earlier row stays in the history with its `base` named.
* **A pace is two numbers.** The band's two ends are the pace; a single pace is a band whose
  ends are equal. The store's `plan_sessions.pace_low_s_km` / `pace_high_s_km` set the
  precedent for the shape.
* **A choice may carry a pair** — a heart rate and a pace band together, both non-null — or
  either one alone. A typed value from a lactate test carries both; a typed number carries
  one.
* `entered_on` and `updated_on` are separate for the same reason they are separate on the race
  observation: "entered 13 Oct 2026, changed 3 Nov 2026" is a fact about a record.
* `rule_id` names the rule that computed the value (`book.lt_pace_band`, `book.45min_tt`,
  `zone_model.zone5_floor`, or none for a typed number). The published alternatives
  (`friel.30min.hr`, `anderson.30min.pace`, `8020.20min.hr`) are named in the catalogue and
  in the note, and are not computed in the first drop.
* The candidates are **not** stored. They are recomputed from the model, the race and the runs
  every time the card is drawn, which is why a candidate's date is the date of the fact it came
  from and not the date the app saw it.

### 9.1 The strings the core would own

Following the catalogue's shape (`crates/stride-core/strings.json`):

| id | kind | template |
|---|---|---|
| `threshold.card.watch` | statement | "{value} — the start of your watch's fifth zone, from your zones read on {date}." |
| `threshold.card.race` | statement | "{band} — threshold pace from your {race} on {date}: the band between your {pace_a} pace ({time_a}) and your {pace_b} pace ({time_b}), by the book's rule — the pace you could race for about an hour." |
| `threshold.card.test` | statement | "{value} — an estimate from your {test} on {date}: your pace and heart rate over the last {window} minutes (the book's rule)." |
| `threshold.card.typed` | statement | "{value} — entered on {date}, from {label}." |
| `threshold.card.base` | statement | "the threshold you chose, made for the {base} base" |
| `threshold.limit.watch` | statement | "the app cannot check what was put into the watch" |
| `threshold.limit.race` | statement | "a race is a race, not a test — the pace carries that day's conditions" |
| `threshold.limit.test` | statement | "the estimate carries the run's conditions, the effort across the window, and the watch's heart-rate accuracy" |
| `threshold.limit.typed` | statement | "the app cannot check what the number was taken from" |
| `threshold.in_use` | status | "in use" |
| `threshold.history` | statement | "your threshold on {date} was {value}" |
| `threshold.paced_with` | statement | "this plan is paced with your threshold of {value}, chosen on {date}" |
| `threshold.other_rules` | status | "other rules are published for this test — the card names the one it used" |
| `threshold.missing.rung` | status | "the threshold pace is not worked out — the rule needs a race time, and none has been entered." |
| `threshold.missing.base` | status | "the easy-day band is not set — the plan's bands come from a threshold heart rate, and no choice has been made for this plan. It could come from your watch's zones, a race, a field test, or a number you enter." |
| `threshold.missing.session` | status | "the session's pace is not worked out — it is paced from a threshold pace, and none has been chosen, and the plan carries no race time." |
| `threshold.drift` | statement | "the heart rate's drift across the window: {drift} bpm" |
| `threshold.ways` | status | "four ways a threshold can arrive: your watch's zones, a race, a field test, or a number you enter" |

Every one of them is a `statement` or a `status`; none of them is a `direction`, an `option`
or a `pattern`, because none of them judges anything.

The four `threshold.limit.*` strings are **annotations**: they are not rendered on the card,
they sit in the note under it (§15). A card renders the value, the source and the date.

---

## 10. The states

| # | state | what the card shows |
|---|---|---|
| 1 | **no zone model read** | the watch's card is **empty** — a hollow slot where the value would be — with the reason the product already uses (`zones.no_model`); the other three cards stand |
| 2 | **no race entered** | the race's card is empty, with the reason "no race time entered"; the watch's card stands on its own |
| 3 | **no test run pointed at** | the field-test card is empty, with the reason "no run chosen for the estimate"; the other cards stand |
| 4 | **candidates, none chosen** | all the cards that have values, each with its source and its date; no card marked; the setup card names the four ways a threshold could arrive |
| 5 | **a choice made** | the chosen card is marked "in use"; every other card is still there, with its value, its source and its date |
| 6 | **a choice changed** | the history card lists both: "your threshold on 3 Nov 2026 was 165 bpm — on 3 Mar 2026 it was 172 bpm"; the old card stays in the list of candidates with its own date |
| 7 | **two candidates disagreeing** | both cards, both marked on the heart-rate lane at their own values; the distance drawn; nothing written about the distance |
| 8 | **a threshold chosen, the dependent answers asked** | the dependent answers use the chosen value and name its source; the unchosen candidates stay visible |
| 9 | **a base adopted with its own choice** | the new choice is marked "in use" and names the base it was made for; the earlier choice stays in the history with its own base and date |
| 10 | **a typed pair** | the typed card carries the heart rate and the pace band, each with its own unit, and the label the athlete picked |

---

## 11. What it never does

* **Never computes a threshold or a maximum heart rate from an age.** There is no birthdate in
  Stride, and this design adds none. `180 − age` is never evaluated, and no candidate carries
  an age.
* **Never asserts which candidate is right**, which one to take, or that one agrees with
  another.
* **Never warns that a candidate is old**, that a threshold needs retesting, or that anything
  is stale. The dates are printed; no freshness rule runs.
* **Never shows a verdict, a score, a readiness, a recommendation, or a colour that judges.**
  The palette is azure for the athlete's own data and amber for a method's rule, and there is
  no third colour with a meaning.
* **Never ranks the candidates**, and never puts a chip on one that names it as the one to
  take.
* **Never computes more than one estimate per candidate**, and never holds several rules
  against each other to pick from — one rule per candidate, and the card names it.
* **Never reads the chosen threshold outside the plan's pacing and the 80/20 base's bands**
  — not the ladder, not the ledger, not the load model.
* **Never merges the watch's zone model with the athlete's threshold**, and never re-bands the
  ledger.
* **Never converts a pace into a heart rate or the reverse**, and never collapses a pace band
  to a single number.
* **Never draws a method's heart-rate band on the %maxVO₂ curve** — %maxHR and %maxVO₂ are
  not convertible without the athlete's lactate relation, which Stride does not have.
* **Never uses a banned word** — `docs/contract/voice.md`'s list, checked word by word in §12.

---

## 12. The ban-list check

The core's list (`crates/stride-core/src/strings.rs`, `BANNED_PHRASES`, 23 entries) is checked
with the core's own word-boundary rule: a phrase matches only when the characters on both
sides are not word characters.

1. this note's prose and the prototype's visible text (tags stripped, `<script>` and `<style>`
   removed, entities decoded, lower-cased) — including the `aria-label`s the screen reader
   reads;
2. the eighteen proposed catalogue strings in §9.1.

**Result: zero hits** in both files and in the proposed strings. The words that come closest
are the plan's own vocabulary — "easy", "long", "recovery" — which the contract keeps as
session types, never as commands; and the noun "rest", which this note does not use at all.

---

## 13. What was decided, and what is still open

### 13.1 The six decisions (24 November 2026)

1. **RCRS.** It is in the repository: `exploration/methods-and-products.md` §2.6 — Bakken's
   model, Norwegian Singles, and Ørnskæg's "Fthreshold 2:1 / 3:2" ratios and 4×4 / 2×4 / 3×4
   cycles, some of which the research thread could not verify, which is why the spine refused
   the family. **The first drop uses the book's rule only.** RCRS and the Norwegian family are
   a **future base** with the 80/20 family, and a base brings **its own rule** (§1.2).
2. **One rule per candidate**, and for the Pfitzinger base that rule is the book's. The other
   published rules stay **named in this note as published alternatives**; the card may say
   other rules exist, and it does not compute four estimates and pick among them (§4.3).
3. **The book's method 5 does not become a fifth card.** The four candidates are four
   **sources**, not four rules; the heart-rate estimate sits beside the watch's own number,
   which is the same kind of thing (§1.1).
4. **The chosen threshold does not feed the load model — not in this pass.** The load model
   keeps the watch's zone model as its reference (the zone-5 floor), the load history does not
   move because an athlete chose a different number, and the load model's own inputs are
   examined in **stage 2** of the zone-model migration, as a separate change with its own
   announcement (§7.5). The athlete's threshold is read in **two places**: the plan's pacing
   and the 80/20 base's bands (§7.1).
5. **One threshold at a time — dated, and tied to the base.** The choice carries the base it
   was made for; another base may come with its own choice, and the earlier one stays in the
   dated history. The plan's pacing **names the threshold it was paced with — the value and
   the date** (§6.2, §7.4, §8.1).
6. **A typed candidate may carry a pair** — a pace band and/or a heart rate (§5.2, §9).

### 13.2 Still open

1. **The ladder's rung and the plan's session are two different paces.** The ladder's
   threshold rung is the **method's rule** on the race time; the plan's lactate-threshold
   session is paced from the **athlete's chosen threshold** (§7.2, §7.4). Both are named where
   they are used, but whether the app draws them together anywhere, so the athlete can see the
   two numbers side by side, is open.
2. **The date a candidate carries.** Each candidate's date is the date of the fact it came from
   (model read, race day, run day, entry day). For the typed candidate the entry day and the
   origin day can differ, and the card shows the entry day. Whether a typed value may also
   carry the day the measurement was taken is open — it is the one place where the athlete
   carries a fact the app could otherwise have read.
3. **The wording of the "other rules exist" line** on the field-test card — how much of the
   alternatives' names belongs in the app, and how much stays in this note.
4. **A base with no threshold.** When a base is adopted and the athlete has chosen nothing for
   it, the setup card names the four ways a threshold could arrive for **that** base, and the
   base's own rule stays visible.

---

## 14. The sources

| id | source | what it gives | confidence |
|---|---|---|---|
| **S1** | Pete Pfitzinger & Philip Latter, *Faster Road Racing: 5K to Half Marathon*, 2nd ed. (Human Kinetics, 2015) — ch. 1, ch. 7, App. B | LT pace = "the pace you could race for about an hour"; the anchor is a **band** — 15K-to-10-mile race pace, for elites 15K-to-HM; 10K pace +10–15 s/mi (or 5K +20–30 s/mi) for shorter-distance athletes; the five ways to find LT pace (a lactate test; a race of about 45–60 minutes; a 45-minute time trial — "your pace and heart rate during the last 30 minutes"; an estimate from training software; an estimate from heart rate — 82–90 % maxHR / 76–87 % HRR); LT band 80–91 % maxHR / 75–88 % HRR; App. B's equivalent race times | **[book]** |
| **F1** | Joe Friel, "Determining your LTHR" and "The 30-Minute Test Is Easy. Really.", joefrieltraining.com | the 30-minute time trial; **average heart rate over the final 20 minutes** ≈ LTHR; the lap button at 10 minutes; a test with others or in a race gives a number too high; a solo effort runs about 5 % slower than a race; the number varies a few beats between tests | **[author]** |
| **F2** | Owen Anderson, Ph.D., "How to Find Your Running Speed at Lactate Threshold", Runner's World | the 30-minute field test; threshold pace = distance in the 30 minutes ÷ 1800 s; the ECU research and Houmard's caveat that the test is a workout, not a race | **[3rd]** (editorial, quoting research) |
| **F3** | Matt Fitzgerald, "Intensity Guidelines for 80/20 Running", 80/20 Endurance | LTHR = "the highest heart rate you can hold for about an hour"; the 20-minute time trial; **average heart rate of the final 15 minutes**; the seven zones from the threshold; the gray zone X at 90–95 % of LTHR; the result is specific to the conditions of the test | **[product documentation]** |
| **F4** | Jack Daniels, "Threshold training", Runner's World (adapted from *Daniels' Running Formula*) | threshold pace = a pace that could be raced for 50–60 minutes; T-pace at 83–88 % VO₂max, 88–92 % maxHR | **[author]** |
| **R1** | the repository | `hr_zone_models` (schema v3), `zones::zone_of` and `zones::model_source`, `load.rs` reading the zone-5 floor as the threshold, `plan_sessions.pace_low_s_km` / `pace_high_s_km`, `PlanSession` and `SessionKind` | **[repo]** |
| **R2** | the repository | `exploration/pace-engine.md` (the chain, the race observation, the ladder, the plan's block anchor), `exploration/pfitzinger-spine.md` (§5.5 the chain, **§5.5.1** the LT-pace rule and the five methods), `exploration/stimulus-ledger.md` (the ledger's rows), `exploration/decisions.md` (D3, D10, D11, D13, D14) | **[repo]** |
| **R3** | the repository | `exploration/methods-and-products.md` §2.6 — the Norwegian family (Bakken's model, Norwegian Singles, Ørnskæg's "Fthreshold 2:1 / 3:2" ratios and 4×4 / 2×4 / 3×4 cycles, marked unverified) — the family RCRS names, and a future base that brings its own rule | **[repo]** |
| **R4** | the repository | `exploration/athlete.js` — the canonical synthetic athlete: the window, the zone model, the 58 runs, the race, the VDOT, the ladder, the curve, the threshold candidates and the plan. The prototype loads it and reads every number from it (§16) | **[repo]** |

---

## 15. The annotation layer

**Explanation is not product content.** The product surface carries the **picture, its labels,
the numbers, the period**, and — where something is missing — **the reason something is
missing**. Everything that explains the design rather than the data lives in the annotation
layer: the notes, off by default in the clean view.

The prototype has a **notes toggle**, and every line below is shown only when the toggle is
on. Nothing was deleted; each line moved.

### 15.1 The lines that moved, and where they went

| the line | from | to |
|---|---|---|
| "the app cannot check what was put into the watch" | the watch's candidate card | the note under that card |
| "a race is a race, not a test — the pace carries that day's conditions" | the race's candidate card | the note under that card |
| "the estimate carries the run's conditions, the effort across the window, and the watch's heart-rate accuracy; the rule assumes a 45-minute time trial on a flat course" | the field-test card | the note under that card |
| "the app cannot check what the number was taken from" | the typed card | the note under that card |
| "the curve carries what the effort costs; this card carries the estimate" | the field-test card | the note under that card |
| "other rules are published for a field test — a 30-minute trial, a 20-minute trial; this card uses one rule and names it" | the field-test card | the note under that card |
| "two kinds, two lanes" (the card's sub-header) | the lanes card's header | cut from the header; the note under the chart carries the reasoning |
| "a heart rate and a pace are two kinds of number, and neither is turned into the other" | the section note above the frames | the note under the lanes chart, and the annotation layer |
| "a pace band is drawn as a band, with its two ends, never as a midpoint" | the section note | the note under the lanes chart |
| "every answer that needs a threshold is a no value with a reason…" | the header card | the note under the header card |
| "a choice is made for a base, and it is dated…" | the header card | the note under the header card |
| "each row is one choice, made for a base, on its own day…" | the history card | the note under the history list |
| "the ladder is the method's chain: the race time, the equivalents, the hour pace. The threshold you chose is not read here." | the ladder card | the note under that card |
| "with a choice made for that base, the bands read Z1 72–81 % of it, and the gray zone X 90–95 %" | the 80/20 card | the note under that card |
| "the session itself still stands: its date, its kind and its target distance" | the plan's session card | the note under that card |
| "the ledger answers against the zone model in force on the day of each run… the load model reads the same model, and the choice does not enter it" | the ledger and load card | the note under that card |
| "your own runs and the heart rates your own watch recorded in them, with the estimate marked… %maxHR and %maxVO₂ are not convertible…" | the own-history card | the note under that card |
| the section notes, the frame captions, and the list of what the prototype does not do | the page | the annotation layer, under the toggle |

### 15.2 What stayed on the product surface, and why

* **The value, and the labels that carry it** — including "the threshold" on the zone model's
  fifth floor, and the units on each lane.
* **The source** where the source *is* the fact: the race and its day, the model it was read
  with and the day it was read, the run and its day, the day a typed number was entered, and
  the label the athlete picked.
* **The rule clause**, amber, on the cards a rule computed — the rule is the source of the
  number, so it stays on the card.
* **The period** — the choice's day, the model's read day, the race day.
* **The reason something is missing** — the 80/20 card's no-value reason, which says what is
  missing *and* where it would come from.
* **The four ways a threshold could arrive**, in the no-choice state.
* **The pointer to the physiological-cost curve** — the effort is the 45-minute marker there;
  the card names it and stops.

### 15.3 The test applied

*If the line would still make sense on a card that shows a different measurement, it is
explanation, not content.* Every line above was checked against that test: the surviving
lines are the numbers, their labels, their sources, their dates, and the one missing-value
reason. The lines that failed the test moved to the notes, and none was deleted.

The rule applies to the **whole product surface**, so it applies to this note too: a note
is the annotation layer's own record, and the prototype's toggle carries the same reasoning
that this file carries in prose.

---

## 16. The canonical athlete — what is read from where

`exploration/athlete.js` is the single source of the synthetic data. The prototype **loads it**
(`<script src="athlete.js">`) and reads every number from it, so the two files cannot drift:
the zone model, the race, the ladder's hour-pace band, the four candidates, the choice, the
history, the plan's pacing and the zone-model line all come from that file.

### 16.1 The canonical set, as it stands here

| | canonical | where it is used here |
|---|---|---|
| the zone model | `HR_MAX`, max heart rate **183**, floors **115 · 128 · 143 · 159 · 165**, read **3 Nov 2026** | the watch's candidate (the fifth floor, **165 bpm**), the zone-model picture, the ledger and load card's source line |
| the typed race | **10K in 41:36 on 12 Oct 2026** (entered 13 Oct, changed 3 Nov) | the race's candidate, the ladder card's source, the plan's anchor |
| VDOT | **49.7** | the chain the equivalents come from |
| the threshold band | the **hour-pace band** in both places | the race candidate's band **4:16–4:17 /km** (the 15K-to-10-mile anchor) and the ladder rung's band **4:16–4:21 /km** (the 15K-to-HM span) — the same rule, two spans |
| the disagreement | **165** and **168** | the watch's fifth-zone floor and the typed number |
| the choice | the watch's floor, **165 bpm**, **3 Nov 2026**, made for **the Pfitzinger 10K plan** | the header card, the history, the plan's pacing line |

### 16.2 The numbers that are derived, and from what

Every number in this prototype that is **not** in the canonical set is derived from the
canonical race and VDOT, and the derivation is the engine's own:

| the number | how it is derived |
|---|---|
| the 15K pace **4:16 /km** (1:03:56) and the 10-mile pace **4:17 /km** (1:08:53) | Riegel's exponent 1.06 from the typed 10K — the engine's chain, not the book's lookup table |
| the race candidate's band **4:16–4:17 /km** | the book's anchor: the 15K pace and the 10-mile pace |
| the ladder rung's band **4:16–4:21 /km** (146–167 bpm) | the hour pace's own span: the 15K pace to the half-marathon pace, with the method's heart-rate band |
| the lane scales on the two-lane chart | derived from the candidates' own numbers, so the chart fits whatever they are |

### 16.3 Two wording flags for the end-to-end thread

1. **The curve's field-test marker.** `athlete.js` places the curve's field-test marker at
   **30 minutes**, because the pace-engine prototype drew a 30-minute test. This design's
   field-test candidate follows the **book's 45-minute time trial** (S1 ch. 1, ch. 7), so its
   marker sits at **45 minutes** — the test's own duration. The two files disagree on that one
   marker, and the book is the rule this file follows.
2. **The race candidate's rule string.** `athlete.js` prints the two equivalent race times
   without the hour — "63:56" and "68:53" — inside the rule clause. As clocks they are
   **1:03:56** and **1:08:53**. The prototype shows `athlete.js`'s string verbatim, so the two
   files cannot drift; the wording is `athlete.js`'s to change.
