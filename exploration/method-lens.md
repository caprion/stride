# The method lens — a picture of the athlete's history, and a method's own rule beside it

*24 November 2026. **Prototype only.** Nothing under `crates/`, `frontend/` or `android/` has been
touched. The two new files are this note and `exploration/method-lens.html`.*

Read with: `exploration/methods-and-products.md` (the research this rests on),
`exploration/stimulus-ledger.md` (the ledger the lens sits beside), `exploration/the-wedge.md`
(the choice), `exploration/decisions.md` (the standing calls — **D1, D2, D3, D5 and D11 bind this
note**), `docs/contract/voice.md` (the frozen voice), and
`crates/stride-core/src/{zones,pace,rolling,fingerprint,strings,units,date}.rs`.

---

## 0. What this note decides, and what it does not

It decides five things:

1. **The lens catalogue** — a small set of method rules that Stride can compute from what it
   already holds, each with the published rule quoted, its source, its units, and a strict answer
   to *is this computable from a watch*.
2. **What each lens draws** (§6) — the picture comes first (D11), the caption under it, the words
   as the accessible fallback.
3. **The caption and the words**, per lens, with every value inline.
4. **The caveats the card carries**, in the product's own voice, including the one that keeps the
   absence honest (Seiler 2010, on heart-rate lag).
5. **The states** — lens off, lens on, lens on whose rule cannot be computed, window with no
   heart-rate data.

It does not decide where the card lives (a route of its own, or a panel on Progress beside the
ledger), and it does not write code. The lens is a **second, explicitly-labelled projection** over
the same history the ledger already measures — the ledger's rows stay method-agnostic, and the lens
is a second reading of them with a rule quoted on top.

**The one rule that governs everything else:** a lens is **opt-in**, **never on by default**, and
**never produces a verdict**. A default lens is the app asserting a method. A verdict — "9 % short
of the 80/20 target", "your easy runs are too fast", "you have not had a threshold run for five
weeks" — is the app doing the athlete's reasoning for them.

---

## 1. The two shapes

### Shape A — the rule and the history, side by side, as two separate facts

The card states the method's **published rule**, quoted and attributed, and then states what the
athlete's **own history** contains, in the athlete's own units. There is **no link between the
two blocks**: no arrow, no colour, no "so", no comparison, no verdict. The reader draws the
conclusion.

The rule block is a fact about a **published rule** — a `statement` about someone else's writing.
The history block is a fact about **the athlete's runs** — a `statement` about the data. Two
facts, side by side. The app states neither about the other.

### Shape B — the athlete chooses a lens; the app reports counts, dates and shares under it only

No rule text, no evaluation, nothing that says whether the result is high or low. Counts, dates,
shares, and the last date a thing occurred. The lens's own number may sit in the header — it is
the lens's identity, and it is a fact: *"Bakken's easy days · 128 bpm"* is 70 % of the maximum
heart rate in the athlete's own zone model.

### The rules both shapes obey

| rule | why |
|---|---|
| **the picture first** (D11) | the card is read as a picture before it is read as text; the sentence is the caption under it |
| **absence is drawn** (D11) | a gap in a strip, or a marker on a small timeline — the reader sees the hole before the sentence names the date |
| **the caption does not repeat the picture** (D11) | the picture carries the shape, the band and the gap; the caption carries the counts and the dates |
| the rule block and the history block stay **visibly separate** | no arrow, and no shared axis that draws the comparison the app must not draw; where the method's number and the athlete's numbers share a scale, that scale is the **method's own** scale and nothing on it is coloured or labelled as a judgement |
| every value is inline | a rendered `statement` may not carry a hole (`docs/contract/voice.md`); a template hole is where a second implementation grows back |
| no banned word | the ban list applies to every string in the catalogue |
| no score, no grade, no traffic light, no colour judgement | there is no readiness in v2 and no verdict in a lens |
| no advice, no imperative | the app never tells the athlete what to do |
| no option linked to a vital | a lens is not an option; it is a reading |
| the absence is a **date** | "the last run with time above 128 bpm was on 21 November 2026" — never a list of what was not done |
| the app never classifies a run | the lens never says a run was an "easy run" or a "threshold run"; it counts time against a number the method published |

**"Above" and "below" are the method's own arithmetic, never the app's judgement.** The words are
allowed because the number is the method's; what is not allowed is a verdict built on them.

---

## 2. The lens catalogue

Three lenses are computed. Three methods are **not** — §3 says why, and what the app says when one
of them is named in the picker.

| # | lens | the method | the rule's units | computable from what Stride holds? | the band it resolves to |
|---|---|---|---|---|---|
| **L1** | Threshold band | Jack Daniels' VDOT, as published by V.O2 | % of maximum heart rate | **yes** — from the maximum heart rate in the zone model in force on the day of the run | 88 %–92 % of that maximum |
| **L2** | The cap | Phil Maffetone's MAF 180 Formula | bpm, from an age formula | **with one athlete-entered number** — Stride holds no age, and the formula's categories are a health self-assessment | the cap the athlete entered |
| **L3** | Easy-day ceiling | Marius Bakken's Norwegian model | % of maximum heart rate **and** blood lactate | **in part** — the heart-rate clause is computable; the lactate clause and the talk test are not | 70 % of that maximum |

### L1 — Threshold band (Daniels / VDOT O2)

**The rule, quoted.**

> "Threshold Pace — Intensity: Generally, in the range of 83-88% of VO2max or 88-92% of HRmax."

**Source:** the VDOT Running Calculator, V.O2 — [vdoto2.com/calculator](https://www.vdoto2.com/calculator/),
read 24 November 2026. The page states each training pace's intensity, its purpose and a sample
workout; the intensity line above is the threshold pace's.

**Units:** per cent of maximum heart rate.

**Computable: yes.** The zone model carries a maximum heart rate
(`hr_zone_models.max_heart_rate`, method `HR_MAX` — `crates/stride-store/src/schema.rs`), and the
rule is a constant percentage of it. Under **D3** the percentage is resolved **per run**, from the
model in force on that run's day, so the band edges can move inside a window when the model was
re-read. The card names the model and the day it was read.

**What the card carries about the rule itself:**

* the percentage is of **a maximum heart rate**, and Stride's maximum is the number the athlete's
  **watch configuration** carries, not a measured maximum;
* the same page publishes a tolerance for the easy pace — "You may be up to 20 seconds per mile
  slower or faster than the specified pace on a given day" — so the bands are a range and not a
  line;
* the app cannot tell whether a session was run **at** the intensity the rule names.

### L2 — The cap (Maffetone)

**The rule, quoted.**

> "Subtract your age from 180, then modify from one of the categories below"
>
> The categories, as published: −10 for major illness, rehabilitation, regular medication or Stage 3
> overtraining; −5 for injury, regression, frequent infection, allergies or asthma, being overfat,
> Stage 1–2 overtraining, or being inconsistent or new; no modification when training consistently
> at least four times weekly for up to two years; +5 after more than two years without those
> problems.
>
> "For athletes 16 years of age and under, the formula is not applicable; rather, an MAF HR of 165
> has been used."

**Source:** Phil Maffetone, *The MAF 180 Formula* —
[philmaffetone.com/180-formula](https://philmaffetone.com/180-formula), updated July 2020. The
headline sentence and the 16-and-under sentence are quoted verbatim; the four categories are
condensed from the four bullets on that page, with every number left as published, and the page
carries the exact wording.

**Units:** bpm, produced by an arithmetic rule over the athlete's **age** plus a **self-assessed
category**.

**Computable: with one athlete-entered number.** Stride holds no age, and the categories are a
health self-assessment that no watch and no store can check. So the lens takes the **cap itself**
as the athlete's own number (their MAF HR, which a runner using the method already knows), and the
app measures the history against it. The app never computes 180 − age itself — the same posture
`fingerprint` already takes with the maximum heart rate: it is "never an age formula".

**The cap is a dated setting, and it behaves like the zone model** (the ninth decision). It is
**entered by the athlete** and stored as a **dated fact** — the value, the day it was entered, the
day it last changed — beside the zone model's dated facts. Each day of a window is measured
against **the cap in force that day**, and the card names the cap and the day it was read. When
the cap changes, the answers before it change too: the same runs measured against a different cap
are different numbers, and the app says which cap it used.

**The rule's own caveat, which the card quotes in full:**

> "The use of the number 180 is not significant other than as a means of finding the end heart
> rate. Plus, 180 minus age itself is not a meaningful number; for example, it is not associated
> with VO2max, lactate threshold, or other traditional measurements."

That sentence is the reason this lens is honest: it is the method saying out loud that its own
number is a convenience rather than a measurement.

### L3 — Easy-day ceiling (Bakken)

**The rule, quoted.**

> "In terms of lactate, we are below 1.0 mml/l, heart rate below 70 % of max for the easy runs"

**Source:** Marius Bakken, *The Norwegian model of lactate threshold training and lactate controlled
approach to training*, Kristiansand, 23 January 2022 —
[mariusbakken.com/the-norwegian-model.html](https://www.mariusbakken.com/the-norwegian-model.html).

**Units:** per cent of maximum heart rate, **and** blood lactate in mmol/l.

**Computable: in part — and the card says so.** The rule has **two clauses**. The heart-rate
clause resolves to 70 % of the maximum heart rate in the model in force on the run's day — 128 bpm
for this athlete. The lactate clause ("below 1.0 mml/l") does not resolve at all: no watch records
lactate. And the method's own control protocol requires the heart rate to be cross-checked against
the talk test and perceived effort, which the app does not hold either.

**The card names both clauses**, says which one resolved against the athlete's own zone model,
and says the lactate clause is not recorded by a watch. Nothing is approximated, and the method is
not quietly halved.

**What the card carries about the rule itself:**

* "easy" here is **the method's own word for the session type**, in a quotation — not the app's
  word, and not an instruction;
* the method's threshold work — "lowering the lactate level from the standard level of 4.0 mmol/l
  down initially to below 3.0 – usually staying from 2.3 up to 3.0" — is a **different rule** and is
  **not** part of this lens.

### What the lenses are *not*

* **not a default** — off until the athlete turns it on;
* **not a recommendation** — the picker lists the methods without ranking them, and the app never
  says which one to use;
* **not a classification** — the app never labels a run "threshold" or "easy";
* **not a score** — there is no number that says how well the athlete did against the rule;
* **not a plan** — the lens has no say over what comes next;
* **not on a run's own page** (the ninth decision) — a run's page shows the run's own facts; a lens
  is a reading of a window.

---

## 3. The methods that are refused, and how they appear

Three of the six methods in the research define their intensity by a **lactate number**, or in a
**race time** Stride does not have. A lens whose rule cannot be computed says so and offers
**nothing** — no approximation, no proxy, no "roughly equivalent".

| method | the rule | why it is refused |
|---|---|---|
| **Seiler / 80-20 polarisation** | "about 80% of training sessions are LIT intensity and the remaining 20% are performed as ThT or HIT", where LIT is "work eliciting a stable lactate concentration of less than approximately 2 mM" ([Seiler 2010, *Int J Sports Physiol Perform* 5(3):276–291](https://umh1617.umh.es/files/2016/05/02-IJSPP_2010_Seiler_0009.pdf)) | the rule is stated in **blood lactate**, and no watch records lactate. It is also stated in **sessions**, not time, so the "80 %" has no single denominator. And the paper itself says that a heart-rate reading "overestimate[s] the amount of time spent training at low intensity" — so an 80/20 figure built from heart rate would contradict the source's own method |
| **Pfitzinger** | general aerobic runs "15% to 25% slower than your marathon pace", or "70% to 81% of maximum heart rate"; long runs "10% to 20% slower than your marathon pace"; threshold "82% to 91% of your max heart rate" (as summarised at [runningwithrock.com/pfitz-marathon-training-explained](https://runningwithrock.com/pfitz-marathon-training-explained)) | the rule is stated in **marathon pace**, which needs a target race time Stride does not hold. The heart-rate percentages exist only in a **third-party summary**, not in *Advanced Marathoning*, 4th ed. — quoting a blog as the book's rule would misattribute it. And the method's own "general aerobic" and "recovery" runs have **no measurable boundary** between them, so the lens would be measuring a label |
| **Norwegian Singles** | sub-threshold is **LT1** ("~2 mmol/L"), with "RPE 5–6/10" and "you can still speak in full, but short, sentences"; the method's own guidance says "heart rate isn't a reliable tool mid-workout" and that **pace** is the steady proxy (per [marathonhandbook.com/norwegian-singles-training](https://marathonhandbook.com/norwegian-singles-training), as quoted in `exploration/methods-and-products.md` §2.6) | the band is defined by **lactate**, and the two controls that police it — perceived effort and the talk test — are not recorded by a watch. The rule that *is* about pace still needs the **LT1 line**, which comes from a lactate test |

### How a refused method appears in the picker

**Listed, with its reason, and not selectable** (the second decision). A refused method does not
vanish silently: the reader learns that it exists and why it is not there.

> ○ No lens — your own zones only
> ○ Threshold band — Daniels / VDOT O2
> ○ The cap — Maffetone
> ○ Easy-day ceiling — Bakken
> — 80/20 polarisation — Seiler · *the rule is stated in blood lactate*
> — Norwegian singles — Copeland / Bakken · *the rule needs RPE and the talk test*
> — Pfitzinger's plans — Pfitzinger & Douglas · *the rule is stated in marathon pace*

The refused rows are drawn without a radio, in muted type, with the reason on the same line. No
method is ever the app's recommendation: the list is in the order this note uses, and the app
asserts none of them.

### What the app would show for a rule it cannot compute

The card states the rule, states why it does not resolve, and gives **no numbers under it** and
draws **nothing**:

> "This rule is stated in blood lactate: low intensity below 2 mmol/l, threshold 2 to 4 mmol/l, high
> intensity 4 mmol/l and above. No watch records lactate, so this rule is not checked here, and no
> numbers are given under it."

**This state does not arise for the refused methods today** — they are listed in the picker with
their reason and are not selectable (the second decision). It is the shape the card takes for any
rule in the catalogue the watch cannot measure.

**And there is no picture for it.** A refused rule has no scale the athlete's data can be plotted
on, so nothing is drawn — a chart for a rule that is not measured would be decoration (D11).

### A second reason for refusing 80/20, which is about the drawing

**A rule that states a target share cannot be drawn on the same scale as the athlete's share.**
The 80/20 rule is a **target**; putting the athlete's low-intensity share and an 80 % line on one
scale would *be* the verdict the app is not allowed to draw — the picture would say "9 % short"
without a sentence saying it. The only fact-only way to show it would be two separate pictures
(the target as a number, the athlete's share as a bar), and a target with no scale of its own has
no picture.

So 80/20 is refused twice over: the rule is stated in lactate, and the rule is stated as a
target, and a target cannot share a scale with the athlete's data. The same argument retires the
Pfitzinger "MP run" idea and any future lens whose published form is a target the athlete is
measured against.

---

## 4. What each lens asks the app to compute

Only what Stride already holds, plus the athlete's own entries.

### The inputs, once

| input | where it comes from |
|---|---|
| the zone model in force on each run's day (five floors, method, maximum heart rate, the day it was read) | `hr_zone_models`, read per run's date (**D3**) |
| each run's moving samples, with heart rate | `routes::series::zone_samples(&decoded)` — the same helper the zones route uses, so the lens and the run detail cannot disagree |
| each run's stored distance and moving time | `activities` |
| the window | the ledger's window (**D1**): the last N calendar weeks through today; the compared period is the same span ending seven days earlier (**D2**) |
| the athlete's entered cap, with the day it was entered and the day it last changed | a dated setting beside the zone model's dated facts (the ninth decision) |

### The computation, once, for all three lenses

All three lenses reduce to the same question: **how much of the athlete's moving time fell below or
inside a heart-rate band, and when was the last time it did not.** So one function serves all three:

* **`zones::time_in_band(from_bpm: Option<f64>, to_bpm: f64, samples: &[ZoneSample]) -> Option<BandTime>`**
  — a sibling of `zones::time_in_zone`, in the same module, over the same `ZoneSample` list and
  with the same boundary discipline (`(from, to]` — a reading exactly on the lower edge belongs to
  the band below, a reading exactly on the upper edge belongs to the band). It returns the time
  below the band, the time inside it, the time above it, the count of runs that held time above it,
  the longest single stretch inside or above it, and the last date on which time above it was
  recorded.

Because the band is resolved per run (**D3**, and the dated cap), the function is called once per
run with that day's band.

`zones::time_in_zone` itself is **not** reusable for a lens band: it snaps readings to the watch's
five floors, and a lens band (161 bpm to 168 bpm) does not line up with the watch's floors. The
lens band and the watch's zones are two different rulers over the same samples, and the card says
so.

### Per lens

| lens | band | what is computed |
|---|---|---|
| **L1** Threshold band | `(0.88 × max, 0.92 × max]`, per run's model | the time inside the band; the share of the window's moving time; the runs that held time in it; the **longest single stretch** inside it and its date; the last date with time in it (searched over the **whole retained history**, as the ledger's §2.4 rule says) |
| **L2** The cap | `(0, cap]`, per day's cap | the time below the cap; the share; the runs that held time above it; the last date with time above it; the longest stretch above it |
| **L3** Easy-day ceiling | `(0, 0.70 × max]`, per run's model | the same shape as L2 |

### The core functions, by name

| need | the existing thing |
|---|---|
| the zone model and its source line | `zones::ZoneFloors`, `zones::model_source`, `zones::zone_edges` |
| the samples | `zones::ZoneSample`, `routes::series::zone_samples` |
| the time in a band | **`zones::time_in_band` — new**, a sibling of `zones::time_in_zone` |
| the window and its compared period | the ledger's calendar-week window (**D1**), `rolling::totals_in` |
| the athlete's own band shares (the ledger's rows, beside the lens) | `rolling::ZoneTimes`, `rolling::polarised_shares`, `rolling::PolarisedZone::label` |
| the pace bands (context only, never the lens's denominator) | `pace::pace_bands`, `pace::PaceBand::label` |
| the laps (the ledger's repeated-work kind, beside the lens) | `fingerprint::LapReading`, `fingerprint::running_only`, `fingerprint::measurable` |
| the dates | `date::Date::{parse, add_days, days_since}`, `routes::day_label` |
| the rendering of the numbers | `units::render_duration`, `units::render_share`, `units::render_heart_rate` |
| the sentences | `strings::render`, `strings::text`, `strings::StringKind`, `strings::lint::check_rendered_statement` |

### The one new module, and the two dated facts

* **core** — `crates/stride-core/src/lens.rs`: the catalogue of rules (a `lens.json` beside
  `strings.json`, with the quotes, the sources and the read dates), the resolution of a rule to a
  band for a given day, and the window projection.
* **store** — two dated facts: the **zone model** (already there, `hr_zone_models`) and the
  **athlete's cap** (new: the value, the day it was entered, the day it last changed). The lens
  choice itself is a setting, off by default.
* **api** — `stride_api::routes::series::zone_samples` is reused; a `lens_runs(api, &runs)` sits
  beside `stride_api::pace::pace_runs`. The cost is the same as the pace page's: one decode per run
  in the window.

---

## 5. The caption, and the words under it

Each card carries three text pieces, in this order:

1. **the caption** — one sentence, the reading of the picture, carrying the counts and the dates.
2. **the rule block** — the published rule, quoted and attributed. Visibly separate.
3. **the words** — the accessible fallback: the numbers with their period and their reason, for a
   screen reader and for anyone reading the card as text.

The caption never repeats what the picture already shows. The picture shows the shape, the band
and the gap; the caption names the count and the date.

### 5.1 Per lens

#### L1 — Threshold band

**The caption:**

> 4 runs held time in that band, and the last of them was on 15 October 2026 — 40 days before the
> end of the window.

**The rule block:**

> "Threshold Pace — Intensity: Generally, in the range of 83-88% of VO2max or 88-92% of HRmax."
> — VDOT Running Calculator, V.O2 · [vdoto2.com/calculator](https://www.vdoto2.com/calculator/) ·
> read 24 November 2026

**The words:**

> In the last 12 weeks, 41 minutes of moving time was between 161 bpm and 168 bpm — 1% of the
> window's moving time — across 4 runs. The last run with time in that band was on 15 October 2026
> — 40 days before the end of the window.

#### L2 — The cap

**The caption:**

> 9 runs held time above the cap, and the last of them was on 24 November 2026.

**The rule block:**

> "Subtract your age from 180, then modify from one of the categories below"
>
> The categories, as published: −10 for major illness, rehabilitation, regular medication or Stage 3
> overtraining; −5 for injury, regression, frequent infection, allergies or asthma, being overfat,
> Stage 1–2 overtraining, or being inconsistent or new; no modification when training consistently
> at least four times weekly for up to two years; +5 after more than two years without those
> problems.
>
> — Phil Maffetone, The MAF 180 Formula · [philmaffetone.com/180-formula](https://philmaffetone.com/180-formula) ·
> updated July 2020

**The words:**

> The cap you entered on 12 June 2026 is 139 bpm. In the last 12 weeks, 46 h 51 m of moving time was
> below 139 bpm — 89% of the window's moving time — across 61 runs. 9 runs recorded time above
> 139 bpm, and the last of them was on 24 November 2026.

#### L3 — Easy-day ceiling

**The caption:**

> 14 runs held time above the ceiling, and the last of them was on 21 November 2026.

**The rule block:**

> "In terms of lactate, we are below 1.0 mml/l, heart rate below 70 % of max for the easy runs"
> — Marius Bakken, The Norwegian model of lactate threshold training · Kristiansand, 23 January 2022 ·
> [mariusbakken.com/the-norwegian-model.html](https://www.mariusbakken.com/the-norwegian-model.html)

**The words:**

> 70% of the maximum heart rate in the zone model in force on the day of each run is 128 bpm. In the
> last 12 weeks, 38 h 05 m of moving time was below 128 bpm — 72% of the window's moving time —
> across 61 runs. 14 runs recorded time above 128 bpm, and the last of them was on 21 November 2026.

**And the clause that does not resolve, on the card:**

> The rule has two clauses. The heart-rate clause is measured against your own zone model; the
> lactate clause — "below 1.0 mml/l" — is not recorded by a watch, and it is not checked here.

### 5.2 Shape B, per lens

No rule text, no evaluation, nothing that says whether the result is high or low. The lens is named
— it is the athlete's own choice — and the number may sit in the header, because it is the lens's
identity and a fact.

> **Bakken's easy days · 128 bpm**
> 61 runs · 52 h 46 m of moving time
> 38 h 05 m below the ceiling — 72% of the window's moving time
> 14 runs held time above the ceiling
> The last run above the ceiling was on 21 November 2026
> The longest run with no time above the ceiling was 8.2 km, on 17 October 2026

The same shape for the other two: "Threshold band · 161–168 bpm" and "The cap · 139 bpm".

**Shape B draws one picture: the strip.** The absence is drawn the same way, and the numbers are
reported as counts, dates and shares. It does not quote the rule, and it does not draw the
method's number on the athlete's distribution — the header already names the lens and its number,
and a second picture would repeat it.

### 5.3 The catalogue keys these sentences would live under

```
lens.picker.none            "No lens"                       (statement)
lens.rule.quote             "{quote}"                       (statement — about a published rule)
lens.rule.source            "{author} · {publication} · {date}"  (statement — about a published rule)
lens.band.from_max          "{percent}% of the maximum heart rate in the zone model in force on the day of each run is {bpm}."
lens.cap.entered            "The cap you entered on {date} is {bpm}."
lens.history.band           "{time} of moving time was {relation} {band} — {share}% of the window's moving time — across {runs} runs."
lens.history.above          "{runs} runs recorded time {relation} {band}, and the last of them was on {date}."
lens.history.longest        "The longest stretch {relation} {band} was {time}, on {date}."
lens.history.last           "The last run with time {relation} {band} was on {date}."
lens.coverage               "{with} of {runs} runs in the window carry heart-rate samples — the times are of the moving time in those {with} runs ({time})."
lens.not_computed           "This rule is stated in blood lactate: {zones}. No watch records lactate, so this rule is not checked here, and no numbers are given under it."
lens.not_computed.input     "The cap has not been entered, so this rule has no number to measure against, and no numbers are given under it."
lens.clause                 "The rule has two clauses. The heart-rate clause is measured against your own zone model; the lactate clause — {quote} — is not recorded by a watch, and it is not checked here."
lens.caveat.hr_lag          "Heart rate lags the effort: a time-in-zone reading under-counts hard running and over-counts easy running (Seiler 2010)."
lens.caveat.not_recorded    "Blood lactate, perceived effort, the talk test, how the run felt, and whether the session was run at the intended intensity are not recorded here and are not checked."
lens.caveat.max_hr          "{bpm} comes from the maximum heart rate in the zone model — {max_hr} bpm, read on {date}. It is the number your watch carries, not a measured maximum."
lens.caveat.coverage        "{with} of {runs} runs in the window carry heart-rate samples."
lens.source.line            "moving time measured over the record stream's movement · {time} across {runs} runs · the zone model in force on the day of each run: {model} · the cap in force: {cap}"
```

### 5.4 The ban-list check on every sentence above

| banned | present? |
|---|---|
| should · must · need to · recommended · better to · ready to | no |
| missed · bad · poor · fail · behind · good | no |
| red · readiness score · not recovered · you are ready · you are not ready | no |
| take the day · rest day · rest up · rest today · go easy · keep it easy | no |

**Two words that need a note rather than a clean pass.**

*"easy"* — it appears in the L3 quotation ("for the easy runs"), in the lens's name and in the
Shape B header ("Bakken's easy days"). It is the **method's own word for its own session type**,
inside a quotation and inside a name the athlete picked. `docs/contract/voice.md` keeps "easy"
alive as vocabulary and bans it as a command ("go easy", "keep it easy"). Neither appears.

*"your"* — the sentences say "your zone model" and "the cap you entered". The ledger's own source
line says "your zone model · HR_MAX · max heart rate 183 bpm · read on 3 Nov 2026", so this is the
product's existing second person, not a new register.

---

## 6. What the lens draws

D11 binds this card: **the picture first, the caption under it, and the words as the accessible
fallback.** Three rules shape what the picture may be.

1. **The picture is the athlete's own numbers, and the method's number is marked on the scale.**
   One scale per lens: **heart rate in beats per minute** — the unit in which the method states
   its own number. The athlete's own numbers (the window's moving time, by heart rate) are the
   bars; the method's number is a marker on the same axis. Nothing on the scale is coloured as a
   verdict and nothing on it is labelled as a judgement: the athlete's time is azure, the method's
   number is a neutral mark, and the words "above" and "below" are the method's own arithmetic
   wherever they appear.
2. **Absence is drawn.** A twelve-week strip under the chart: a filled cell for each week in which
   the heart rate reached the method's number, an empty cell for each week in which it did not. A
   hole in the strip is the absence, before any sentence is read. The date is a mark on the last
   filled cell; the caption names it.
3. **The caption does not repeat the picture.** The picture carries the shape, the band and the
   gap; the caption carries the count and the date; the words underneath carry the numbers with
   their period and their reason.

### L1 — Threshold band

* **The chart.** The window's moving time by heart rate — eight bars from 100 bpm to the athlete's
  183 — with the method's own band marked on the axis as an outlined interval at **161–168 bpm**.
  The 41 minutes the athlete has inside that band are a sliver at the far right of their own
  distribution, and the picture says so without a word.
* **The strip.** Twelve cells, one per week of the window; four filled (the weeks that held time
  in the band), two single-week gaps earlier in the window, and then a **five-cell hole** from
  27 October to the end of it, with a mark on the last filled cell.
* **The caption.** "4 runs held time in that band, and the last of them was on 15 October 2026 —
  40 days before the end of the window."

### L2 — The cap

* **The chart.** The same distribution, with a single mark at **139 bpm** — the cap the athlete
  entered on 12 June 2026. The athlete's time sits almost entirely to the left of it; the picture
  shows that and calls it nothing.
* **The strip.** Twelve cells; nine filled (the weeks with a run above the cap) and three
  single-week gaps, with a mark on the last filled cell — 24 November.
* **The caption.** "9 runs held time above the cap, and the last of them was on 24 November 2026."

### L3 — Easy-day ceiling

* **The chart.** The same distribution, with a single mark at **128 bpm** — 70 % of the maximum
  heart rate in the zone model in force on each run's day.
* **The strip.** Twelve cells, every week filled, with a mark on the last one — 21 November. There
  is no hole here, and the picture does not invent one.
* **The caption.** "14 runs held time above the ceiling, and the last of them was on 21 November
  2026."

### The lens off

**One bar, printed once** (D11): the athlete's own zone split as a single stacked bar — Z1–Z2,
Z3, Z4–Z5 — with a legend under it. That bar is the ledger's own card: drawn once, and not drawn
again when a lens is on. Above it, one line: "No method's rule is applied to the numbers below."

### The states with no picture

The **refused** state and the **no heart-rate data** state have no picture: there is nothing to
plot, and a chart for a rule that is not measured would be decoration. Both are text.

### What could not be drawn, and stayed fact-only

Three things in this note's material cannot be drawn at all, and each one was left as words rather
than turned into an approximation:

| what | why it cannot be drawn | what the card does instead |
|---|---|---|
| **A rule that states a target** — 80 % of the time at low intensity | putting the target and the athlete's share on one scale would *be* the verdict; the picture would say "9 % short" without a sentence saying it | the rule is refused as a lens, and the method is listed in the picker with its reason |
| **A rule stated in blood lactate, RPE or the talk test** | there is no scale to plot the athlete's data on, and the quantity is not recorded at all | the rule is quoted, the reason is named, no numbers are given and nothing is drawn |
| **The clause of a rule that does not resolve** — L3's lactate clause | a picture of the heart-rate clause would be read as a picture of the whole rule | the card draws the heart-rate clause only, and names the clause it did not draw |

**And one thing that is drawn, deliberately:** the athlete's 41 minutes at the method's number are
a **sliver** at the far right of a 52-hour distribution, and the picture shows them as a sliver. They
are not enlarged, not smoothed and not given a second axis. The gap in the strip is the same size
as the gap in the training.

---

## 7. The caveats the card carries

Four blocks, in the product's own voice, below the picture — not folded into a footnote, and not
mixed into the words.

### 7.1 What the heart rate does (the caveat that keeps the absence honest)

> Heart rate lags the effort: a time-in-zone reading under-counts hard running and over-counts easy
> running (Seiler 2010).

> "heart rate time-in-zone tends to underestimate the time spent working at high intensity (due to
> heart rate lag time during intervals)" — and, in the same paper, "typical software-based heart
> rate analysis methods overestimate the amount of time spent training at low intensity and
> underestimate the time spent at very high workloads" — Seiler, *What is Best Practice for Training
> Intensity and Duration Distribution in Endurance Athletes?*, Int J Sports Physiol Perform 2010;
> 5(3):276–291.

**Why it is on the card and not in a footnote.** A run's heart rate arrives after the effort. So a
stretch of weeks with no time above a ceiling in this card is partly a fact about the **watch** and
not only about the running. Without this line, the absence sentence would be read as a fact about
the athlete; with it, it is a fact about a measurement.

This is also why the ledger's **repeated work** kind (the run's own lap structure) sits beside the
lens: it is the one part of the history that does not depend on the heart rate.

### 7.2 What the watch does not record

> Blood lactate, perceived effort, the talk test, how the run felt, and whether the session was run
> at the intended intensity are not recorded here and are not checked.

Never "the app checks that you felt it was easy". Never an implication that anything was verified.
The sentence is a list of what is absent, and it says *not checked* in the plainest words available.

### 7.3 The number the band is built on

> 161 bpm and 168 bpm come from the maximum heart rate in the zone model — 183 bpm, read on
> 1 September 2026. It is the number your watch carries, not a measured maximum.

(L1 and L3 only; L2's cap comes from the athlete and says where it came from instead.)

### 7.4 Coverage

> 61 of 61 runs in the window carry heart-rate samples — the times are of the moving time in those
> 61 runs (52 h 46 m).

and, in a window where the coverage is partial:

> 58 of 61 runs in the window carry heart-rate samples — the times are of the moving time in those
> 58 runs (51 h 01 m).

When the two windows do not hold the same runs, the change against the period before is not
reported — the ledger's rule, kept (**D2**).

### 7.5 One more, for L2

> The cap you entered on 12 June 2026 is 139 bpm. The rule that produces it is Maffetone's, and its
> own words are on this card: the number is "not associated with VO2max, lactate threshold, or
> other traditional measurements".

---

## 8. The states

| # | state | what the card shows |
|---|---|---|
| **S1** | **no lens chosen** — the default | the control, one line saying that no method's rule is applied, and the athlete's own facts as one drawn bar. No rule text anywhere |
| **S2** | **a lens chosen, and its rule resolves** | the chart, the strip, the caption, the rule block (quote + attribution), the words, then the caveats |
| **S3** | **a rule the watch cannot measure** | the rule block, the reason in one sentence, **nothing under it** and **no picture**. No numbers, no proxies, no "close enough". The refused methods are not selectable, so this is the shape the card takes for any rule in the catalogue the watch cannot compute |
| **S4** | **a lens chosen that needs an input the athlete has not given** (L2 with no cap entered) | the rule block and "the cap has not been entered, so this rule has no number to measure against" — and nothing under it |
| **S5** | **a window whose runs carry no heart-rate data** | the rule block (if a lens is on) and the core's own line — "the split has no value — the runs in the window have no heart-rate samples". No times, no shares, no dates, and **no picture** |
| **S6** | **no stored zone model** | "zones unavailable — no zone model has been read". The lens bands that need the maximum heart rate have no value either; L2's cap is the athlete's own number and survives |

### S1, in full

> **Lens** — None
>
> No method's rule is applied to the numbers below.
>
> *(the athlete's own facts, as one bar: Z1–Z2 84%, Z3 14%, Z4–Z5 2% — 44 h 11 m, 7 h 26 m, 1 h 08 m;
> and the last date each kind occurred.)*

The ledger is **not** duplicated by the lens. The lens adds a rule and a reading on top; the
ledger's own bar is drawn once and stays on the ledger's card.

### S3, in full

> **A rule the watch cannot measure**
>
> "about 80% of training sessions are LIT intensity and the remaining 20% are performed as ThT or
> HIT" — Seiler, Int J Sports Physiol Perform 2010; 5(3):276–291
>
> This rule is stated in blood lactate: low intensity below 2 mmol/l, threshold 2 to 4 mmol/l, high
> intensity 4 mmol/l and above. No watch records lactate, so this rule is not checked here, and no
> numbers are given under it.

Nothing else, and no picture. The athlete's own facts are still on the ledger beside it.

---

## 9. Open questions

1. **Is the heart-rate axis the right scale?** The note reads the parent's instruction — "the scale
   must be the method's own number and the athlete's own number plotted on it" — as: the scale is
   the unit in which the **method** states its number (beats per minute), and the athlete's own
   numbers are plotted on it. The alternative reading — a scale made only of the method's number —
   would leave nothing to plot. Worth confirming, because it decides the whole picture.
2. **A histogram, or one split bar?** The note draws the athlete's time as a histogram by heart
   rate, because it shows *where* the athlete's time sits and how little of it is at the method's
   number. A single bar split at the method's number is simpler, prints one number, and loses the
   shape. The histogram is proposed; the bar is the alternative.
3. **Is the words block visible, or screen-reader only?** The prototype shows it behind a small
   "in words" disclosure. In the app it may be the accessible text only.
4. **Is the strip per lens, and are its cells weeks?** Twelve weekly cells per lens, because the
   weeks the heart rate reached the method's number are what differ. A strip of runs instead of
   weeks would be finer and longer.
5. **Where does the card live** — a route of its own, or a panel on Progress beside the ledger?
6. **Does the lens's picture replace the ledger's bar on the same panel?** D11's "one bar, printed
   once" says no: the ledger's bar stays on the ledger's card, and the lens card carries the
   lens's own picture.
7. **Does a changed cap re-state the earlier answers?** The ninth decision says the answers before
   a change change too. Whether the card also shows what the old cap would have said is not
   decided — it would be a comparison, and it is the athlete's own number either way.

---

## Appendix — the synthetic data behind the prototype

* **One athlete**, the same one as `exploration/stimulus-ledger.html`: 25 weeks, 135 runs, 8 June –
  24 November 2026.
* **Window** (**D1**): the ledger's window — 2 September – 24 November 2026, 61 runs, 52 h 46 m of
  moving time, measured over the record stream's movement.
* **Zone model** (**D3**): HR_MAX, maximum heart rate 183 bpm, read on 1 September 2026, in force
  for the whole window.
* **The cap** (the ninth decision): 139 bpm, entered by the athlete on 12 June 2026, unchanged
  through the window.
* **The ledger's own numbers** (unchanged): Z1–Z2 low 44 h 11 m — 84% — 61 runs; Z3 moderate
  7 h 26 m — 14%; Z4–Z5 high 1 h 08 m — 2% — 4 runs, the last on 15 October 2026, 40 days before
  the end of the window; longest run 20.4 km in 1 h 59 m on 21 November; repeated work 2 h 15 m
  across 3 runs, the last on 1 October 2026.
* **The window's moving time by heart rate** (the chart's own data):
  100–110 bpm 8 h 00 m · 110–120 16 h 50 m · 120–130 16 h 40 m · 130–140 5 h 21 m · 140–150
  2 h 50 m · 150–160 1 h 31 m · 160–170 0 h 59 m · 170–183 0 h 35 m.
* **The bands the three lenses resolve to:**
  * L1 — 88 %–92 % of 183 bpm = **161 bpm to 168 bpm**. 41 minutes of moving time inside the band
    across 4 runs; the longest stretch inside it 14 minutes, on 8 October 2026; the last run with
    time in it on 15 October 2026.
  * L2 — the cap in force: **139 bpm**. 46 h 51 m below the cap — 89% of the moving time — across
    61 runs; 9 runs held time above it, the last on 24 November 2026.
  * L3 — 70 % of 183 bpm = **128 bpm**. 38 h 05 m below the ceiling — 72% of the moving time —
    across 61 runs; 14 runs held time above it, the last on 21 November 2026.
* **The strips** (weeks of the window, filled where the heart rate reached the method's number):
  L1 — weeks 1, 2, 5 and 7 filled, then five empty cells; L2 — nine filled, three single-week
  gaps; L3 — all twelve filled.
* **Coverage:** the main state has heart-rate samples on all 61 runs; the partial-coverage state
  has 58 of 61 (51 h 01 m of measured moving time).
* Everything is synthetic. No real athlete data, no Garmin call, no network, no file under
  `crates/`, `frontend/` or `android/` touched, nothing committed.
