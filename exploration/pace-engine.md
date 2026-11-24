# The pace engine — race time → the race paces → the training paces

> **Status: exploration only.** Nothing here is implemented, nothing is committed, nothing is
> merged. Two new files under `exploration/` and nothing else: this note and
> `exploration/pace-engine.html` (the prototype). No code in `crates/`, `frontend/`, `android/`
> or `desktop/`; no change to `PLAN.md`, `docs/` or the contract.

*The surface copy in this note and in its prototype follows the language pass — `exploration/language.md`.*
>
> **Revision 2 — 24 November 2026**, after `exploration/pfitzinger-spine.md` landed. The spine
> changed the design: the method's rules are **percentages of a named race pace**, the shipping
> plans are the **10K and the half marathon**, and the **general-aerobic day has no pace rule at
> all** — it is a heart-rate rule. The ladder is therefore rebuilt on **two axes**, and the
> **marathon pace is no longer the anchor**. Every rule below carries the book's chapter; every
> formula that is the engine's own is marked as such.
>
> The numbers in the prototype are **synthetic** and come from **one place**:
> `exploration/athlete.js`, the canonical athlete (D21). The prototype reads them from there —
> nothing is typed into its markup — so a number cannot drift between this file and the rest
> of the product. The rules are **Pfitzinger's** and cited.
>
> Read with: `exploration/pfitzinger-spine.md` (**the rules and their sources — do not re-derive
> them**), `exploration/decisions.md` (D11 the picture first, D12 the lens that is out, **D13 the
> first drop is Pfitzinger's plan**, **D14 the audience and the two plan bases**),
> `docs/contract/voice.md` (the ban list), `crates/stride-core/src/plan.rs` (the plan engine),
> `crates/stride-core/src/units.rs` (the rendering), `crates/stride-core/src/zones.rs` (the dated
> zone model), `crates/stride-core/src/pace.rs` (the pace grid) and
> `exploration/race-and-level-plans.md` (the plan builder's shape).

### What D13 and D14 bind here

* **D13** makes this engine a **dependency of the first drop**: no lens ships, Pfitzinger's plan is
  the spine, and the plan anchor returns to the ledger. It also states the two rules this note
  is built on: *a race time is an observation, stored with provenance (value, day entered, day
  last changed)*, and *Stride never invents one, never guesses one from training, and never
  derives a maximum heart rate from an age*.
* **D14** sets the audience — the intermediate runner — and the shape: **two plan bases, one at a
  time**. The ledger and this engine speak against the base the athlete is following, and the app
  never recommends either.

---

## 1. What changed, and why

The first revision of this note built the ladder on **marathon pace**, with every rung a fixed
signed offset from it. The spine shows that this was wrong for the plans that ship, and **none of
those offsets survives** — no rung in this design is a constant, and nothing is computed as an
offset from a marathon pace:

* **The shipping plans are the 10K and the half marathon.** The marathon book was not in hand; its
  MP-relative bands are third-party summaries (spine §1.3), and **no marathon plan ships** (spine
  §0.5).
* **The method's rules are percentages of a named race pace**, not fixed offsets from anything:
  the long run is "20–33 % slower than your 10K race pace"; the recovery run is "2 minutes per
  mile slower than 15K-to-half-marathon race pace"; LT pace is "the pace you could race for about
  an hour" (book ch. 1, ch. 2, ch. 7).
* **The general-aerobic day has no pace rule at all** — only a heart-rate band, 70–81 % of
  maximal HR (book ch. 7, ch. 8). Any pace on an easy day is **the athlete's own data** or
  **the athlete's own hand**, and the card has to say which.

So the anchor moves from "the equivalent marathon pace" to **the athlete's own race paces** — the
pace at the goal distance, and the LT pace derived from the same race time by the method's own
rule. The marathon pace stays in the engine as **the normalisation it already is**, and it appears
in the ladder as **one more rung**, not as the thing every other rung is measured from.

One thing did **not** change: the rules that shape everything below come from the contract, not
from the method.

1. **A race time is an observation, not a verdict.** The athlete types it. Stride never invents
   one and never infers one from the training.
2. **Every derived number names its source.** "From your 10K in 41:36 on 12 Oct 2026."
   The card names the race, the day of the race, the day it was entered and the day it was last
   changed.
3. **Absence is not zero.** No race time → no race paces → no rungs → *no value*, with the reason
   stated.
4. **No score, no grade, no verdict**, and **no age, no birthday** — a maximum heart rate is never
   an age formula. The book's own HR bands are computed from the athlete's own max HR (read from
   the dated zone model) and resting HR (read from wellness).

### The canonical athlete (D21)

The prototype's numbers are **not written in this file, and not written into the prototype's
markup**: they are read from **`exploration/athlete.js`**, the canonical athlete every screen in
the product now uses (D21).

| the fact | the canonical value |
|---|---|
| the zone model | read **3 Nov 2026**, method HR_MAX, **max heart rate 183 bpm**, floors 115 · 128 · 143 · 159 · 165 |
| the typed race | a **10K in 41:36 on 12 Oct 2026**, entered 13 Oct 2026, changed 3 Nov 2026 |
| the VDOT | **49.7** |
| the goal | a half marathon |
| the threshold band | the **hour-pace band** — the ladder's threshold rung and the curve's threshold session, the same band in both places |

Everything else the prototype shows is **derived from the canonical race and the canonical
VDOT**, by the engine's own relations: the equivalents (Riegel's form), the LT band (the book's
rule on the 15K and half-marathon equivalents), the long run's two ends, the recovery pace, the
heart-rate bands (the method's percentages of the canonical max HR) and the curve's %maxVO₂ for
every mark. The prototype **draws them from `athlete.js` at run time** — its markup holds no
athlete number of its own — so the two cannot drift again.

---

## 2. The chain

```
race time (typed) ──①──▶ VDOT ──②──▶ the athlete's race paces (10K · 15K · HM · marathon)
                                                  │
                                                  ├─③──▶ LT pace  (the method's rule)
                                                  │
                                                  └─④──▶ the rungs: a pace + a heart-rate band
                                                          │
                                                          └─⑤──▶ a race prediction (Riegel)
```

### 2.1 ① Race time → VDOT — the engine's own chain

Daniels' formulation, computed rather than looked up, so the engine is transparent and a table
update cannot move a number under the athlete's feet. With `v` the speed in metres per minute and
`t` the race time in minutes:

```
v        = distance (m) ÷ t
VO2      = −4.60 + 0.182258·v + 0.000104·v²            (ml/kg/min — the running oxygen cost)
%max     = 0.8 + 0.1894393·e^(−0.012778·t) + 0.2989558·e^(−0.1932605·t)
VDOT     = VO2 ÷ %max
```

*Source of the formulation*: Daniels, ***Running Formula***, 3rd ed. (Human Kinetics, 2014), with
the oxygen-cost and percent-of-VO₂max curves of Daniels & Gilbert, *Oxygen Power* (1979). This is
**the engine's own chain** — the book's chain is different (§2.6) and the card names which one it
used.

* **What it needs:** the athlete's own race — distance, time and the day of the race.
* **What it is:** the athlete's equivalent VO₂max as one number. The card calls it **VDOT** and
  never "VO₂ max" — `exploration/decisions.md` D10 excludes Garmin's VO₂max *estimate* from what
  we keep, and calling our own computed number by that name would put two different things under
  one word.
* **Missing input:** no value, with the reason "no race time entered". Never a default, never a
  VDOT carried over from another race.
* **Worked example (the canonical athlete):** 10 000 m in 41:36 → `v` = 240.38 m/min,
  `VO2` = 45.22, `%max` = 0.9114 → **VDOT 49.6**, which the canonical file carries as
  **49.7** (`exploration/athlete.js`, `race.vdot`). The prototype shows the canonical value.

### 2.2 ② VDOT → the athlete's race paces — the same relations, inverted

The **equivalent race paces** — 5K, 10K, 15K, 10-mile, half marathon, marathon — come from
**Riegel's form** applied at each distance (§2.5's formula), because that is what the canonical
athlete computes (`exploration/athlete.js`). The **VDOT inversion** — finding, at each distance,
the pace at which a race of that distance would produce the same VDOT — is the engine's
**independent check** on them, and the two agree (below).

* **Formula:** the engine's own — the same Daniels relations inverted at each distance
  (`VO2(v) = VDOT × %max(d·p/60000)`), solved for the pace `p`.
* **Self-test:** the inversion reproduces the entered race's own pace exactly (the canonical
  athlete's 10K in 41:36 → 4:10 /km). A round trip that does not return the entered value is a
  bug, not a rounding artefact.
* **Cross-check against the method's own table (this is the important one).** The book's
  Appendix B gives the equivalent performances for a 20:00 5K as **10K 41:36 · 15K 1:03:51 · HM
  1:32:21**. Riegel's form, from that same 20:00 5K, gives **10K 41:50 · 15K 1:03:46 · HM
  1:32:03** — within **18 seconds**, inside the book's own 30-second table steps. The VDOT
  inversion gives **10K 41:28 · 15K 1:03:46 · HM 1:31:50** — the same, within the table's steps.
  Both routes agree with the method's table; the card says which chain produced the number
  (§2.6).
* **The two routes on the canonical athlete.** Riegel's form from the 10K in 41:36 gives a
  **15K 1:03:56 · half 1:31:47 · marathon 3:11:22**; the VDOT inversion on the same VDOT gives
  **1:03:53 · 1:32:00 · 3:11:38** — **3, 12 and 15 seconds** apart. Two different models, one
  answer: the equivalents are sound.
* **Missing input:** no value.

### 2.3 ③ The race pace → LT pace — the method's rule, on the engine's anchor

**LT pace = "the pace you could race for about an hour"** (book ch. 1 "Lactate threshold"; ch. 7
"Lactate Threshold (Tempo) Runs"). That is the rule's true statement. The book then *approximates*
it: "for faster runners, your LT pace is approximately your race pace for **15K or 10 miles**, and
for elite runners LT pace typically is **between 15K and half marathon pace**"; and "if your
experience is mostly with shorter races, LT pace is generally **10 to 15 seconds per mile slower
than 10K race pace**" (ch. 1, ch. 7).

**So the LT rung carries a band of paces, not a single number.** The book's anchor is two paces —
15K *and* 10 miles — and a single "15K pace" is the **fast edge** of that band, not the whole rule.
The engine therefore:

* computes **both** ends from the same race time (the 15K equivalent and the 10-mile equivalent,
  both from ②);
* draws the rung as a **band on the pace axis**, fast edge to slow edge, with the rule named.

| the athlete's goal distance | the LT band the app shows | the rule |
|---|---|---|
| **half marathon** (or longer) | the **15K race pace** (fast edge) to the **half-marathon race pace** (slow edge) — the span in which the hour pace falls for this athlete, with the book's 15K-to-10-mile band inside it | ch. 1 — "the pace you could race for about an hour"; "between 15K and half marathon pace" |
| **10K** (a shorter-distance athlete) | the **10K race pace + 10–15 s/mi** (+16 to +24 s/km) | ch. 1, ch. 7 — the shorter-distance branch, which the book states as a **floor**, not a substitute |
| a 5K goal | the 5K race pace + 20–30 s/mi | ch. 1, ch. 7 |

For the canonical athlete the numbers are: the 15K equivalent **4:16 /km**, the 10-mile
equivalent **4:17 /km** — the book's own 15K-to-10-mile band is **one second wide** for this
athlete — and the half-marathon pace **4:21 /km**, so the rung reads **4:16–4:21 /km**. The card
names the route it used: *"the threshold band is the pace you could race for about an hour; the
book places it at your 15K-to-10-mile race pace"*, and the 45-minute time trial is named as the
book's own route to the same band.

* **The anchor is the engine's; the rule is the method's.** The 15K and 10-mile equivalents come
  from ② (the engine's own inversion); the *choice* of them as the band's edges is the book's,
  cited above.
* **The same rule, two anchors — and the rung names its anchor.** The ladder's threshold rung is
  the hour-pace band **for the athlete's goal distance**: for the canonical athlete, a half
  marathon, **4:16–4:21 /km**. The threshold setup's **race candidate** is the same rule on a
  different anchor — the athlete's **typed race**, the 10K — and gives **4:16–4:17 /km**. Two
  numbers for two anchors, so the rung carries its anchor (*the hour pace · your HM*) and the
  card names it in full (*the threshold band for your half marathon*). Neither of them is ever quoted
  as the other.
* **The book's five ways to find LT pace** (ch. 1, ch. 7 — spine §5.5.1): a **lactate test** (six
  4-minute stages, a blood sample at each); a **race of about 45–60 minutes**; a **45-minute time
  trial** ("your pace and heart rate during the last 30 minutes"); a **training-software estimate**;
  and an **estimate from heart rate** — "about 82 to 90 percent of maximal heart rate, which
  coincides with about 76 to 87 percent of heart rate reserve" (ch. 1's figures; ch. 7 states the
  wider 80–91 % / 75–88 %). The **threshold-setup thread** owns the athlete-facing version of
  these; the ladder names only the route it used — a 45-minute time trial is the book's own test
  for the same band, and the ladder may name it as a route — while the ladder itself stays about
  paces and bands.
* **A useful cross-check, also the method's:** the marathon is "a couple of percent slower than
  lactate threshold" (the author, 2025 interview — spine S3). For the canonical athlete the
  numbers agree: the band sits inside 4:16–4:21 and the marathon equivalent is **4:32 /km** —
  within the method's own tolerance band, and the check is printed in the note rather than
  hidden.

### 2.4 ④ The rungs — a pace **and** a heart-rate band

Each rung is two things:

* an **HR band** where the method states one (the method's rule), and
* a **pace**, either from the method's percentage rule (the method's rule) **or** from the
  athlete's own pace at that heart rate (the athlete's own data).

The band is the method's; the pace may be either. The card does not blur them, so every rung
says which is which. The full table is §3.

### 2.5 ⑤ A race prediction — Riegel, from the goal race

```
T₂ = T₁ · (D₂ ÷ D₁)^1.06
```

with `T₁` = the goal race time (the anchor) and `D₂` the target distance. The exponent **1.06** is
Riegel's and is named in the card. *Missing input:* no value. *Worked example (the canonical
athlete):* from the 10K in 41:36 → 5K 19:57, 15K 1:03:56, 30K 2:13:18, half 1:31:47,
marathon 3:11:22.

### 2.6 Which chain — named, not implied

The spine §5.5 puts three chains on the table and says the app names the one it uses:

| chain | what it is | the numbers it produces |
|---|---|---|
| **(a) the book's own** | race time → Appendix B equivalents → Appendix C workout paces | Pfitzinger's own numbers, citable to the book; costs two transcribed tables and interpolation between 30-second steps |
| **(b) the model's** — **the engine's own, used here** | race time → VDOT (Daniels) → the equivalent race paces → the method's rules applied to them | the *rules* are Pfitzinger's; the *anchors* are Daniels' — and the two agree to within the book's own table step (§2.2) |
| **(c) the author's current framing** | every workout pace set relative to LT pace (his own site's framing, S4) | makes LT pace the single anchor; needs his published LT table |

**This engine is (b), and the card says so** — the header carries "your fitness number 49.7 · from your 10K in 41:36".
**The chain replaces the book's lookup tables**: the app does not transcribe Appendix B and
Appendix C; it computes the 10K, the 15K and the half-marathon pace from the athlete's race time,
and then applies the **book's rules**, which are quoted and cited. The rules are never presented
as Pfitzinger's numbers when they are the engine's: the long run's "20–33 % slower than your 10K
race pace" is the method's rule, and the 10K race pace it is applied to is the engine's
equivalent, named on the card.

**The two bases of the ladder, and the one base at a time.** The ladder above is Pfitzinger's:
its bands are percentages of the athlete's **maximal heart rate** and its paces are read off the
athlete's **own race distance**. The other base (80/20, spine Part 4) bands its easy days
against the athlete's **threshold** (Zones 1–2 = 72–90 % of threshold HR) and deliberately leaves
the middle unused (the gray zone at 90–95 % and 100–102 % of threshold HR is the method's "zones
to avoid"), with **two** hard sessions a week where Pfitzinger's week has **one**. So the same
athlete, on the other base, gets **different bands on the same heart-rate axis** and a different
anchor. The app holds **one base at a time**, the plan card names it, and the ladder's caption
names it too (§10).

### 2.7 The chain as one table

| link | how it is computed | what it needs | when the input is missing |
|---|---|---|---|
| **race time → VDOT** | Daniels' oxygen-cost and percent-of-VO₂max curves (§2.1) | the athlete's own race: distance + time + date | **no value** — "no race time entered" |
| **VDOT → the race paces** | the same relations inverted at each distance (§2.2) | the above | **no value** |
| **race pace → LT pace** | the method's rule, on the engine's 15K-equivalent or 10K pace (§2.3) | the above + the athlete's goal distance | **no value** |
| **race paces + LT pace → the rungs** | each rung's own rule (§3) | the anchor for that rung, and the rule's extra inputs | **no value for that rung only** — the other rungs still answer, and the rung says what it needs |
| **goal race → a race prediction** | Riegel's `T₂ = T₁·(D₂/D₁)^1.06` (§2.5) | the goal race time | **no value** |

---

## 3. The rungs — rule, heart-rate band, and where the number comes from

**The vocabulary is the spine's.** The run types are the book's (ch. 7): **endurance (the long
run), general aerobic, recovery, progression long run, lactate threshold, VO₂max, speed, tune-up
race** — plus the goal race — and they are the **plan's session kinds** (§9). The **ladder draws
the paces subset** of them: the rungs that have a pace an athlete runs at, which is general
aerobic, recovery, lactate threshold, the long run (both forms), the goal race — and, as the one
extra rung, the marathon pace. **The rest belong to the plan's session kinds and not to the
ladder**: VO₂max and speed have a pace but no place on an easy-to-hard ladder (they are the hard
sessions the plan schedules); the tune-up race is a race like the goal race; and the progression
long run is the long run's variant, which the ladder shows when the plan says so. The table below
covers every rung the ladder draws, and the two plan-only run types are carried in §9.

The book's run types are ch. 7's eight, plus the goal race and the plan's own shape.

| rung | anchor | the rule (and its citation) | the heart-rate band | where the pace comes from |
|---|---|---|---|---|
| **Threshold (LT)** | the LT band (§2.3) | "the pace you could race for about an hour" — **a band**: the 15K race pace (fast edge) to the half-marathon race pace for a longer-distance athlete; 10K pace + 10–15 s/mi for a shorter-distance athlete — **book ch. 1, ch. 7** | **80–91 % maxHR / 75–88 % HRR** (ch. 1, ch. 7) | **the method's rule**, on the engine's band |
| **The goal race** | the athlete's own race time | the distance and the time the plan was built for — **book ch. 10, ch. 12** | none from the method; the athlete's own **Z4–Z5** is the context | **the athlete's own data** — their race, typed by them |
| **Marathon pace** | the engine's normalisation (②) | *no method rule ships*: the marathon plan does not ship, and the MP-relative bands are third-party (spine §1.3) | **none** — and the card says so rather than inventing one | **the engine's own** (Daniels) — one rung, **not** the anchor |
| **Long run** | the 10K race pace (early) and the 15K–HM race pace (latter) | early: **20–33 % slower than the 10K race pace**; latter: **17–29 % slower than the 15K-to-HM race pace**; "begin at a conversational pace and gradually increase" — **book ch. 1, ch. 7; App. C** | **74–84 % maxHR / 65–78 % HRR** (ch. 1, ch. 7) | **the method's rule** — one rung, two ends |
| **Progression long run** (the plan's variant) | the LT pace for the last ~3 miles | "increase effort steadily throughout the run, last 3 miles at LT pace" — **book ch. 7; ch. 11–13 schedules** | up to ~84 % maxHR by halfway, then LT | **the method's rule** — the plan says which long run it is |
| **General aerobic (easy)** | **none** | **the method fixes no pace** — the book describes the day as conversational, harder than a recovery run and lighter than a long run — **book ch. 7, ch. 8** | **70–81 % maxHR / 62–75 % HRR** (ch. 7, ch. 8) | **the athlete's own data** — their pace at that heart rate, from their own runs — or **the athlete's own hand** if they type one. Never presented as the method's number |
| **Recovery** | the 15K–HM race pace (the slower end of the book's range) | **+120 s/mi (+75 s/km)** slower than the 15K-to-HM race pace — **book ch. 2 "How Easy?", ch. 7 "Recovery Runs"** | **below 76 % maxHR / below 70 % HRR** (ch. 2, ch. 7) | **the method's rule** |
| **VO₂max** (the hard session) | the 5K/3K race pace | **3K-to-5K race pace**; reps of 2–6 min, 5 000–8 000 m total; recovery jog 50–90 % of the interval time — **book ch. 1, ch. 7; App. C** | **94–98 % maxHR / 92–97 % HRR** (ch. 1, ch. 7) | **the method's rule**; the plan carries the structure |

**Three things this table says out loud.**

1. **The general-aerobic rung has no pace from the method.** If the app shows a pace there, it is
   the athlete's own — from their own runs at that heart rate, or typed by them — and the rung
   says which. This is the same discipline D12's rule 4 applies to Maffetone's cap: an
   athlete-entered number is a dated fact of the athlete's own, never a computed one.
2. **The recovery rule has two forms** — a pace form and a heart-rate ceiling — and the app has
   both, because the method has both (ch. 2, ch. 7).
3. **Two of the method's rules need a choice the book leaves open**, and the app makes it
   explicitly: which of the "15K-to-half-marathon race pace" anchors the recovery rule uses (the
   app takes the **slower** of the two the athlete has — the half-marathon pace — because a
   recovery run is the easiest run of the week and the rule's intent is a slow run), and which
   branch of the LT rule applies (the app's choice follows the athlete's **goal distance**). Both
   choices are named on the card.

---

## 4. The race time is an observation, not a verdict

### 4.1 What the athlete types, and what is read

The spine §5.4 gives the shortest honest list of what the athlete has to supply. Here is where each
of those inputs comes from — **typed** or **read**:

| input | typed or read | where it lives | notes |
|---|---|---|---|
| **goal race distance** (10K or HM) | **typed** | the plan's goal | the two shipping templates (spine §0.5) |
| **goal race date** | **typed** | the plan's anchor | anchors the 12 weeks, the recovery week and the taper (spine §2.2, §2.4) |
| **one race time** (10K or HM; a 5K or 15K time converts) | **typed** | a new race observation, §4.2 | the anchor of every pace |
| **maximal heart rate** | **read** | the **dated zone model** (`crates/stride-core/src/zones.rs`; `max_heart_rate` in the store's model) | **never** an age formula; if the zone model has none, the HR bands have no value and say so |
| **resting heart rate** | **read** | daily wellness (`resting_hr`) | may be absent on some days; the HRR form of a band then has no value for that day |
| **plan tier** (distance + tier) and **plan start date** | **typed** | the plan block | the 10K tiers 1–2 and the HM tiers 1–2 ship (spine §0.5, §2.2) |
| **the athlete's pace at a heart rate** | **read** | their own stored runs (per-sample HR and pace) | this is where the general-aerobic rung's pace comes from |
| **the athlete's own zone floors** | **read** | the dated zone model | drawn on the ladder's heart-rate axis |

Everything else — the equivalent race paces, the LT pace, every rung's pace, every heart-rate
band, the VO₂max pace, the speed splits — is **computed from those inputs**. **No VDOT estimate
of a race time, no age, no guess from the training.**

### 4.2 What is stored — an observation with its provenance

Modelled on `wellness_observations` (`crates/stride-store/src/schema.rs`): the value, the day it
was entered, the day it was last changed, and the source that supplied it.

```sql
CREATE TABLE race_observations (
    id          INTEGER NOT NULL PRIMARY KEY,
    distance_m  REAL NOT NULL,         -- the race's distance (21097.5 for a half marathon)
    time_s      REAL NOT NULL,         -- the finish time
    race_date   VARCHAR(10) NOT NULL,  -- the day of the race
    name        TEXT,                  -- the athlete's own words; optional
    source      VARCHAR(16) NOT NULL,  -- 'typed' — the only source there is
    entered_on  VARCHAR(10) NOT NULL,  -- the day it was entered
    updated_on  VARCHAR(10) NOT NULL,  -- the day it was last changed
    anchor      INTEGER NOT NULL DEFAULT 0  -- 1 when the paces come from this race
);
```

* `source` is always `typed`. A watch does not time races, so there is no other value to store.
* `entered_on` and `updated_on` are separate because the states need them.
* **At most one row carries `anchor = 1`** — the race the paces come from.

### 4.3 The anchor

* The first race the athlete enters becomes the anchor.
* Entering another race **does not silently move the anchor**; the card names the anchor and the
  athlete can point it at another race.
* The **goal race** is a separate fact (§4.1): the goal distance decides which branch of the LT
  rule applies, and the goal race may or may not be the anchor race.

### 4.4 What the engine never does

* **Never invents a race time** and **never infers one from the training**. A run is not a race.
* **Never computes an age-dependent number.** No date of birth is stored; `180 − age` is never
  evaluated.
* **Never presents the athlete's own pace as the method's**, and never the method's rule applied
  to the engine's anchor without saying which is which.

---

## 5. Every derived number names its source

The source line is the caption under the picture:

```
From your 10K in 41:36 on 12 Oct 2026 · entered 13 Oct 2026 · the race time was changed on
3 Nov 2026
```

and, because the anchors are now derived rather than given, one more line names them:

```
10K 41:36 · 15K 1:03:56 · half 1:31:47 · marathon 3:11:22 — the equivalents of that race,
from VDOT 49.7
```

* **The race it came from** — distance, time, day, and the athlete's own name for it if they gave
  one.
* **The day it was entered, and the day it was last changed.**
* **How old the race is**, once it is old enough to matter: *"the race was 194 days ago"*. A fact,
  not a warning. The threshold is ours to choose (**90 days**, §15 Q2).
* **Which chain produced the anchors** — the header carries "VDOT 49.7 · Daniels' chain".

When the race time changes, the answers change with it, because the answers are derived and never
stored: the ladder redraws, and the caption says which race time it used and that the value was
changed on 3 Nov 2026.

---

## 6. The states

| # | state | what the card shows |
|---|---|---|
| 1 | **No race entered** | the same card, **empty**: both axes with no rungs and no anchor line — the hole is drawn (D11) — and the caption says no race time is entered, so the paces have no value, and names the fields the athlete can type: distance, time and the day of the race, with the name optional. |
| 2 | **One race entered** | the ladder with all its rungs, the anchor's plumb-line, and the caption naming the race, the day it was entered, and the day it was last changed if it has been. |
| 3 | **A race entered and later replaced** | the ladder redrawn from the current value, and the caption says the race time was changed on 3 Nov 2026 — or, when the anchor was pointed at a different race, names the race it now uses. |
| 4 | **A race old enough to matter** | the ladder as usual, plus one line of fact: *"the race was 194 days ago"*. |
| 5 | **A rung that cannot be computed** | the rung's **band still stands** (it is the method's), and its **pace** is a hollow marker with the reason: *"the general aerobic pace has no value — no runs at that heart rate yet"*. The other rungs still answer. |

State 5 now has a different case from revision 1, and a better one: the general-aerobic rung's
pace comes from the **athlete's own history**. An athlete who has never run in that heart-rate
band gets the **band** and no pace — and the card says where the pace would come from.

A second case of the same shape: **no max heart rate in the zone model**. Then no band can be
drawn at all, every rung's band has no value, and the card says the band needs the zone model the
watch holds — while the *paces* still answer, because they come from the race.

---

## 7. The ladder — the picture (D11)

The card is the picture; the sentence is the caption.

**Two axes, one set of rows.** The rows are the method's run types, ordered by pace (fastest at
the top). Each row carries two things on two axes:

* **the heart-rate axis** (left): the **method's band** for that rung — amber, the method's rule —
  drawn against the **athlete's own zone floors** — azure vertical lines, the athlete's own data.
  A band the method does not state (the goal race) is drawn as the athlete's own zone; a rung with
  no band at all (the marathon pace) leaves the cell empty and says so.
* **the pace axis** (right): the rung's pace — **amber** when it comes from the method's rule,
  **azure** when it comes from the athlete's own data (their race, their own pace at a heart
  rate), and a **neutral grey** when it is the engine's own (the marathon-pace normalisation).

**The long run is one rung with two ends.** Its bar runs from its start pace to its finish pace,
with the arrow pointing at the faster end: "begin at the slow end of the range and gradually
increase" (book ch. 1). The progression long run's finish is the LT pace for the last ~3 miles —
the plan says which long run it is, and the rung says.

**Where a rule gives a range, the range is drawn.** The threshold rung is a band too — the hour
pace **for the athlete's goal distance**, whose fast edge is the engine's 15K pace — and it is
drawn as one, with its anchor on the label. The long run's band and
the recovery rule's single value are both the method's; the general-aerobic rung's range is the
athlete's own spread at that heart rate. Each says which.

**Colours.** Amber = the method's rule. Azure = the athlete's own data. Grey = the engine's own
derivation. **No green, no colour that marks a problem, no "easy is green" judgement** — the
colours say *whose* number it is, not how it compares.

**Absence is drawn.** The no-race state is an empty ladder — the axes, no rungs, no anchor line —
not a card of zeroes and not a sentence alone. A rung with no pace is a hollow marker on the pace
axis, and its band is still drawn on the heart-rate axis.

**The caption** carries what the picture cannot: the source line (§5), the missing input for a
hollow rung, the age of the race.

**One small caption inside the frame.** Under the chart, in the chart's own voice: *the
heart-rate band and the pace are two different numbers, and they are not converted into one
another.* The same sentence is in the notes (§14). Everything else that explains the design is a
note.

**A second reading, on its own route.** The **physiological-cost curve** (§8) is a second panel
with a route of its own, linked from the pace card. It is not drawn on the ladder: the ladder is
already two axes, and the user's words are *"I don't want to crowd it too much, but that kind of
graph is informational."*

---

## 8. The physiological-cost curve — a second reading of the same athlete

**What it is.** Daniels' classic curve: the share of VO₂max that a duration holds, against time
on a log axis. It is a **second reading**, not a second ladder: no rungs, no pace bands
repeated.

**Where it lives.** **Not on the ladder.** Its own panel, on its own route —
**`/insights/pace-curve`** — linked from the pace card. Nothing is crowded.

### 8.1 What is on the curve

| on the curve | placed by | the colour | whose number |
|---|---|---|---|
| **the curve** | `%max(t)` — the same relation §2.1 uses, read as "the share of VO₂max that duration holds" | grey | the physiological relation (the engine's own) |
| **the athlete's own races** | the race's own time, and the share of their VDOT that the race's own pace cost | azure, filled dots | the athlete's own data |
| **the goal race** | the equivalent time for its distance — a hollow dot when it is not one of the entered races | azure, a ring | the plan's |
| **the method's sessions** | each session's duration **and** its prescribed pace — the threshold session (20–40 min at the hour-pace band, book ch. 1, ch. 7), the long run (its duration range, from the start pace to the finish pace), the recovery run (3–5 mi at its pace, book ch. 2, ch. 7) | amber, a band over its duration and its pace | the method's rule |
| **the 45-minute time trial** | 45 minutes, on the curve | a hollow grey marker | the test's place in time |
| **the pace equivalents** | each distance's equivalent time and pace, along the bottom | azure ticks | the engine's own |

**The general-aerobic day is not on the curve.** The method fixes no pace and no duration for it
(book ch. 7, ch. 8), and Stride does not turn heart rate into %maxVO₂ — so there is nothing to
place, and the panel says so.

### 8.2 The one rule that matters — %maxHR and %maxVO₂ are not interchangeable

Pfitzinger's bands are percentages of **maximal heart rate** (and of heart-rate reserve); this
curve's axis is **%maxVO₂**. Converting one into the other needs the athlete's **lactate
relation**, which Stride does not have and will not estimate. So the method's HR bands are
**quoted in the caption and the legend** and are **never drawn on the curve**. Where the athlete's
own pace-at-heart-rate history allows a **pace** to be named for a session, that pace is **the
athlete's own** (azure) and it is labelled as such — it is not the HR band, and the card does not
pretend the two are the same measurement.

### 8.3 The empty states

* **No race time.** Every point on the curve is measured against the athlete's VDOT, and the VDOT
  comes from their race — so the curve has **no value**. The axes are drawn; the curve is not, and
  the panel says what it needs and where that comes from.
* **A race time but no adopted plan.** The curve still draws — the athlete's own races, the goal
  race and the pace equivalents — with **no method's sessions on it**, and it names what is
  missing.

### 8.4 What it can never become

* **A judgement.** Nothing here says a session is too hard, nothing ranks the athlete's races
  against the method's sessions, and nothing draws a conclusion about the athlete's readiness for
  a distance. It shows the method's sessions and the athlete's races on one curve, and the reader
  draws the conclusion.
* **A verdict about the method.** The same curve holds both bases; the athlete's choice of base
  is named on the plan card, not here.
* **A second ladder.** No rung duplication and no pace bands repeated.

### 8.5 The caption names the assumptions

The curve's caption is the ladder's source line, extended: the **VDOT** and the race time and
the day it was entered; the **method's prescribed paces and durations**, cited; and the **race
equivalences**. And one small caption under the chart, in the chart's own voice: *the method's
heart-rate bands and this curve's %maxVO₂ axis are two different numbers, and they are not
converted into one another.* The rest of the reading — how a race lands where it does, why the
general-aerobic day is absent — is a note (§14).

---

## 9. How the plan uses the paces

A Pfitzinger plan is a **prescribed** plan — an intent, not a baseline — and its sessions carry
the pace **and the heart-rate band** the method prescribes for that week. Today `PlanSession`
(`crates/stride-core/src/plan.rs:124`) is `date + kind + target_distance_m` and the label is
"10 km · easy"; the store already has `plan_sessions.pace_low_s_km` and `pace_high_s_km`, but
nothing fills them and the core type cannot hold them
(`exploration/race-and-level-plans.md` §2). The plan engine therefore gains:

* **a pace band on the session** — the store's existing columns — plus the **heart-rate band**
  the method states for that run type, and the **structure** where the method states one (the
  long run's "last 3 miles at LT pace", the VO₂max reps, the stride sets — book ch. 1, ch. 7).
  The core's label then reads "18 km · long · 5:32 → 5:00 /km · 135–154 bpm".
* **a block-level anchor**: the id of the race the block was paced from, the race's time as it
  stood, and the day it was read, so the block can always say what it was built from.
* **a method value** on `PlanSource` — the spine's §4.3: `PlanSource` gains
  `Pfitzinger | EightyTwenty` so the plan card can name the base, and `SessionKind` gains the
  **book's vocabulary** (endurance, general aerobic, recovery, progression long run, lactate
  threshold, VO₂max, speed, tune-up race, race) because **the plan's kind is what the ledger
  compares the watch's evidence against**.

The plan's **source word** is a product decision left open here: a generated Pfitzinger plan is an
intent and may say "not done", but it is not a file the athlete imported;
`exploration/race-and-level-plans.md` §3.4 already argues for a fourth source (`built`). When the
athlete's race time changes, the plan's pacing changes **with it and in place**: the derivation is
the method's rule plus the current anchor, so sessions on and after the change are re-paced to
the new value and their `updated` day moves, while **sessions before the change keep the pace they
were given** — D3's rule ("today's settings cannot be passed off as historical settings"),
applied to the plan's own history. The same rule applies to the **heart-rate bands**: a zone model
changed on 12 Oct applies from 12 Oct, and the plan's earlier weeks keep the band they were given.

### 9.1 The plan's shape is now known, cell by cell

The spine's schedules for the four shipping tiers — **10K tiers 1–2 and half-marathon tiers 1–2**
— have been read cell by cell (spine §2.3), so the templates can be built from the file's tables
rather than guessed. What the plan side may claim (spine §7):

* a **12-week** plan, weeks-to-go counted 11 → 1 and then race week, with a **recovery week at 8
  weeks to go** in every tier;
* **Monday rest or cross-training** in every week of every tier;
* **one quality session a week**, and its **type moves with the phase** — lactate threshold in the
  early weeks, VO₂max later, race pace in the last third (in the half-marathon tiers the quality
  day moves Friday → Tuesday → Friday);
* the **long run on Sunday**, with the **last 2–3 miles at LT pace** in the half-marathon tiers'
  three progression long runs (weeks 9, 7 and 5) — the ladder's long-run rung has that second end
  for exactly this reason;
* **two tune-up races** — a 5K at 4 and 2 weeks to go in the 10K plans, an 8K–10K at 4 and 2
  weeks to go in the half-marathon plans;
* the **race week's shape**, including the race-pace rehearsal (1 200 m at 8K–10K race pace in
  the 10K plans, 2 miles at half-marathon race pace in the half-marathon plans), and a **recovery
  week and a transition week after the race**.

**What it may not claim:** any weekly skeleton for a tier that is not shipping, **a pace for the
general aerobic run** (the method fixes a heart-rate band only), any marathon-plan claim from the
spine's numbers, any cell value from Appendix C, and **that the athlete's plan was followed, or
not** — the plan's cells are intent; the watch's zones and lap structure are evidence, and the
two are different things.

---

## 10. How the ledger uses the plan, and the two bases

The ledger's rows answer against the athlete's own history — the intensity bands, the longest
run, the repeated work — and the plan adds the **intent** the history is measured against: the
plan's session kinds become the vocabulary, and the negative space the wedge is about becomes
readable — *"the plan had a lactate-threshold session on 14 October; the last run in Z4–Z5 was on
15 October"* is now a statement the card can make, because the plan names the kind and the date
and the history names the last date. The spine's **Part 3 mapping** is what makes the two halves
comparable: the method's run types against our stimulus kinds, with the column that says **which
of them the watch can verify** and which exist only as the plan's words — a row that is evidence
is then distinguishable from a row that is intent. What it can **never** become is a judgement
about whether the athlete did what they were told: no adherence score, no count of what was not
done, no quality verdict on a session, and no count of how far the training was from the plan.

**The two bases produce two different ladders, and the app holds one at a time.** Pfitzinger's week
runs **one** hard session a week; its easy days are banded against the **athlete's own max HR**
(general aerobic 70–81 %, recovery below 76 %, LT 80–91 %, long run 74–84 %); and its paces are
read off the athlete's **own race distance** through the book's tables. The polarised week runs
**two** hard sessions a week; its easy days are banded against the **athlete's threshold**
(Zones 1–2 = 72–90 % of threshold HR), and its **middle is deliberately unused** — the published
seven-zone scale puts the gray zone at 90–95 % and 100–102 % of threshold HR, and the method
calls those the zones to avoid (spine Part 4, R3; the 80 %-of-time rule from Seiler 2010). So the
same athlete, on the other base, gets **different bands on the same heart-rate axis**, a different
set of rungs, and a different anchor: the threshold instead of the race distance. The ladder
therefore **names the base it belongs to** — the plan card names it, and the ladder's caption
names it too — and the app never recommends either (D13, D14).

---

## 11. What the core would own (sketch — not implemented here)

A new module beside `plan.rs` — `crates/stride-core/src/pace_engine.rs` — with the same rules the
rest of the core obeys: no clock (every date is an argument), no value is `None` rendered as `0`,
the core renders the domain's sentences and its numbers, and the frontend renders chrome.

```rust
pub struct RaceObservation {          // one row of race_observations
    pub id: i64,
    pub distance_m: f64,
    pub time_s: f64,
    pub race_date: Date,
    pub name: Option<String>,
    pub entered_on: Date,
    pub updated_on: Date,
}

/// Where a rung's number came from — the card does not blur these.
pub enum NumberSource {
    Method,          // the book's rule, applied to an anchor
    AthleteHistory,  // the athlete's own runs (their pace at a heart rate)
    Engine,          // the engine's own derivation (the marathon-pace normalisation)
}

/// What a rung is anchored on.
pub enum Anchor {
    RacePace { distance_m: f64 },  // the athlete's own race pace at a distance
    LtPace,                       // the method's rule, on the engine's 15K-equivalent
    EngineNormalisation,          // the equivalent marathon pace
}

pub enum PaceRung { Threshold, GoalRace, Marathon, LongStart, LongFinish, GeneralAerobic, Recovery, Vo2Max }

/// One rung: a band, a pace, and the reason it has none.
pub struct RungValue {
    pub rung: PaceRung,
    pub anchor: Option<Anchor>,
    pub hr_band: Option<HrBand>,          // the method's band, or the athlete's own zone
    pub pace_s_per_km: Option<(f64, f64)>, // a single value or a range
    pub source: NumberSource,
    pub missing: Option<MissingInput>,     // NoRaceTime | NoMaxHr | NoHistoryAtThisHr
}

pub struct PaceSet {
    pub anchor_race: Option<RaceObservation>,
    pub vdot: Option<f64>,
    pub race_paces: Vec<(f64, f64)>,       // the equivalent race paces
    pub lt_pace_s_per_km: Option<f64>,
    pub rungs: Vec<RungValue>,
    pub predictions: Vec<Prediction>,
}
```

**Rendering — reuse, do not reinvent.** Pace values use the core's `units::format_pace` ("4:16")
and `units::render_pace` ("4:16 /km"), a range uses `units::render_pace_range` ("4:16–4:21 /km"),
the heart rate uses `units::render_heart_rate` and `units::render_heart_rate_range`, the distance
uses `units::render_distance`, the duration uses `units::format_duration`, and the VDOT uses
`units::format_decimal(value, 1)` ("49.7"). **One new renderer is needed**, a sibling of
`format_pace` beside it in `units.rs`: a race's finish time is read as a clock ("41:36"), and
`format_duration` would give "1 h 32 m", which is not how a race time is written. Dates are the
frontend's existing chrome (`formatDateMedium` — "12 Oct 2026"); the core supplies the ISO day.

**Strings** (the core owns the words; the catalogue lints them):

| id | type | text |
|---|---|---|
| `pace.heading` | status | "Training paces" |
| `pace.chain` | status | "VDOT {vdot} · Daniels' chain" |
| `pace.source.race` | status | "From your {time} {distance} on {date}" |
| `pace.source.entered` | status | "entered {date}" |
| `pace.source.changed` | status | "changed {date}" |
| `pace.source.age` | status | "the race was {days} days ago" |
| `pace.source.equivalents` | status | "the {distance} ({time}) pace is this engine's equivalent of that race" |
| `pace.missing.race` | status | "No race time entered — the paces have no value yet." |
| `pace.missing.history` | status | "The {rung} pace has no value — no runs at that heart rate yet." |
| `pace.missing.maxhr` | status | "The heart-rate bands have no value — the zone model holds no maximum heart rate." |
| `pace.rung.source.method` | status | "{rung} — the method's rule, {citation}" |
| `pace.rung.source.athlete` | status | "{rung} — your own pace at that heart rate" |
| `pace.rung.source.engine` | status | "{rung} — the engine's own equivalent" |
| `pace.prediction` | status | "{distance} {time} — Riegel, exponent 1.06, from your {race}" |
| `pace.selfcheck` | statement | "{distance} on {date}: {time} · the prediction from your {race}: {time}" |

---

## 12. How the ban list was checked

The ban list in `docs/contract/voice.md` is the contract's, and the core enforces it in
`crates/stride-core/src/strings.rs` (`BANNED_PHRASES`, 23 entries) with a **word-boundary**
match (`contains_phrase`: a banned phrase matches only when the characters around it are not word
characters). The same rule was applied to the prototype and to the proposed strings:

1. the prototype's **rendered product surfaces** were drawn from `athlete.js` in a headless run
   and their text extracted (tags stripped, entities decoded, the `aria-label`s included) — **1 028
   words**;
2. the page's own static text, the drawing script's string literals and `athlete.js`'s own strings
   were added to it — **8 187 words** in all;
3. each of the 23 banned phrases was matched with the core's word-boundary rule, on bytes, the
   way `contains_phrase` does it (a phrase matches only when the characters on both sides are not
   word characters — so a word that merely contains a banned fragment, such as "entered" or
   "prediction", is not a hit);
4. the proposed catalogue strings in §11 were checked the same way, plus the rule that a
   `statement` may not carry a hole and that an `option` may not name a vital.

**Result: zero hits** — in the prototype's 8 187 words (1 028 of them the rendered product
surfaces), in the fifteen proposed strings, and in this note's own prose. The words that come closest are the plan's own vocabulary — "easy", "long", "recovery" —
which the contract keeps as *session types*, never as commands and never linked to a vital; and
the noun "Rest", which the contract exempts by name. The prototype contains no imperative, no
score, no colour that marks a problem, no verdict, and no age.

---

## 13. The rules the ladder uses, and where each number comes from

Nothing in the ladder is a placeholder any more: **no rung is a fixed offset from a marathon
pace**, and every rung carries either a cited rule or the athlete's own data. The table in §3 is
the authority; this is the same content in one block, for the record.

**The vocabulary is the spine's, and it is the plan's vocabulary.** The run types are **endurance
(the long run), general aerobic, recovery, progression long run, lactate threshold, VO₂max, speed,
tune-up race** — plus the goal race — and they are the **plan's session kinds** (§9). The ladder
draws the **paces subset**: general aerobic, recovery, lactate threshold, the long run in both its
forms, and the goal race, with the marathon pace as the one extra rung. **The rest belong to the
plan's session kinds and not to the ladder**: VO₂max and speed are the hard sessions the plan
schedules, the tune-up race is a race like the goal race, and the progression long run is the long
run's variant, which the ladder shows when the plan says so.

| rung | the rule | citation | the number's owner |
|---|---|---|---|
| Threshold (LT) | the hour pace — **a band**: the 15K race pace (fast edge) to the HM race pace for a longer-distance athlete, 10K pace + 10–15 s/mi for a shorter-distance one; 80–91 % maxHR | book ch. 1, ch. 7 | the method's rule, on the engine's band |
| The goal race | the athlete's own race | book ch. 10, ch. 12 | the athlete's own data |
| Marathon pace | the engine's normalisation; no band | — | the engine's own |
| Long run | 20–33 % slower than the 10K pace; 17–29 % slower than the 15K–HM pace; 74–84 % maxHR | book ch. 1, ch. 7; App. C | the method's rule |
| Progression long run | the long run's finish is LT pace for the last ~3 miles | book ch. 7; ch. 11–13 | the method's rule; the plan says which long run it is |
| General aerobic | 70–81 % maxHR; **no pace rule** | book ch. 7, ch. 8 | the method's band; the **athlete's own** pace |
| Recovery | +120 s/mi (+75 s/km) off the 15K–HM pace; below 76 % maxHR | book ch. 2, ch. 7 | the method's rule |
| VO₂max | 3K–5K race pace; 94–98 % maxHR | book ch. 1, ch. 7; App. C | the method's rule — a plan session, not a rung |
| Speed · strides · hills | 150–300 m at 800 m–mile pace; ~100 m strides | book ch. 7 | the method's rule — a plan session, not a rung |
| Tune-up race | shorter than the goal distance | book ch. 7 | a race like the goal race |
| Equivalent race paces | the engine's inversion of Daniels' relations | Daniels 2014; Daniels & Gilbert 1979 | the engine's own |
| Race predictions | Riegel, exponent 1.06 | Riegel | the engine's own, named |

**And on the physiological-cost curve (§8), the rules are these:**

| on the curve | the rule | citation | the number's owner |
|---|---|---|---|
| The curve | `%max(t)` — the share of VO₂max a duration holds | Daniels 2014; Daniels & Gilbert 1979 | the engine's own |
| The athlete's races | each race's own time and the share of their VDOT its pace cost | — | the athlete's own data |
| The threshold session | 20–40 minutes at the hour-pace band | book ch. 1, ch. 7 | the method's rule |
| The long run | its duration range, from the start pace to the finish pace | book ch. 1, ch. 7 | the method's rule |
| The recovery run | 3–5 miles at the 15K–HM pace + 75 s/km | book ch. 2, ch. 7 | the method's rule |
| The 45-minute time trial | 45 minutes, on the curve | book ch. 1, ch. 7 | the test's place in time |

**The word count checked in §12** is the prototype's rendered text (1 016 words on the product
surfaces, 8 187 with the page, the script and `athlete.js`), plus the same check on the fifteen
proposed strings in §11 and on this note's prose.

---

## 14. The annotation layer — what moved off the product surface

**The rule, from the user, 24 November 2026:** *"overall i think great visuals. I am seeing that
we are adding text where it is not required."* **Explanation is not product content.** The
product surface carries the **picture**, its **labels**, the **numbers**, the **period**, the
**source** of a value where the source *is* the fact, and — only where something is missing —
the **reason something is missing**. Everything that explains the *design* rather than the *data*
is a **note**, off by default in the clean view.

**The test.** If a line would still make sense on a card that shows a different measurement, it
is explanation and it goes. If it is the period, or the reason a value is absent, it stays.

### 14.1 What stays on the surface, and why

| the line | why it stays |
|---|---|
| the ladder's rungs, their values, their heart-rate bands and their **rule labels** | the rule is the source of the number |
| "From your 10K in 41:36 on 12 Oct 2026 · entered 13 Oct 2026 · the race time was changed on 3 Nov 2026" | the source, and the dates |
| "10K 41:36 · 15K 1:03:56 · half 1:31:47 · marathon 3:11:22 — the equivalents of that race, from VDOT 49.7" | the source of the derived paces |
| "The threshold band is the pace you could race for about an hour; the book places it at your 15K-to-10-mile race pace (ch. 1, ch. 7)" | the source of the band's numbers |
| "A 45-minute time trial is the book's own route to that band (ch. 1, ch. 7)" | the source route for the band |
| "the general-aerobic day is not on the curve: the method fixes no pace and no duration for it" | **the reason a mark is absent** |
| "No race time entered — the race paces and the training paces have no value…" | the reason the values are absent |
| "the general aerobic pace has no value — no runs at that heart rate yet. The band still stands: 128–148 bpm" | the reason the pace is absent |
| "No plan adopted — the method's sessions are not on the curve…" | the reason the sessions are absent |
| the small caption under each chart | the one line that belongs with the picture |

### 14.2 What moved, and where it went

| the line | it was on | it is now |
|---|---|---|
| "the heart-rate band and the pace are two different numbers; they are not converted into one another" | a header above the ladder | a **small caption under the chart**, in the chart's own voice, and a note |
| "the method's heart-rate bands and this curve's %maxVO₂ axis are two different numbers; they are not converted into one another" | the desktop curve's assumptions list | the **small caption under the chart**, and a note |
| "the curve is the share of VO₂max each duration holds; each of your races sits at the share your own pace cost" | the curve card's caption | a note |
| "the method's sessions are placed at their prescribed durations and paces" | the curve card's caption | a note — the legend keeps "the method's sessions" |
| "the 15K pace, 4:16, is the fast edge of that band" | the ladder card's caption | a note |
| "a race time is entered by the athlete; Stride does not estimate one from the training" | the empty ladder's fields card | a note |
| "a model with its assumption named — not a forecast" | the desktop ladder's prediction card | a note |
| "Pfitzinger's bands are percentages of maximal heart rate; this curve's axis is percentages of VO₂max" | the desktop curve's assumptions list | the chart's small caption, and a note |
| the whole of the prototype's design notes, the states' comparison, the rules table and the chain | the page | **the notes**, under the toggle |

In the prototype the annotation layer is a **toggle**: a checkbox in the toolbar, **off by
default**, and the clean view shows only the product surfaces. The notes carry the whole
explanation, and **every line that left a surface is in them** — nothing was deleted.

---

## 15. Open questions

1. **Which chain ships: (a) the book's tables, (b) the model's, or (c) the author's LT framing?**
   This design is (b) with the method's rules applied to it, and the card names it. The spine's
   §5.5 argues (a) is the most citable and (c) is the author's own current framing; the engine's
   advantage over (a) is that it is smooth between the table's 30-second steps and needs no
   transcription of two appendix images.
2. **How old is "old"?** The prototype states the race's age at 90 days. Any threshold is a
   product choice; the fact is the same either way.
3. **Which race pace anchors the recovery rule?** The book says "15K-to-half-marathon race pace";
   the app takes the slower of the two (the half marathon). The other end of the range is the
   easier recovery run and the book's intent is an easy run.
4. **Does the general-aerobic rung's pace come from the history or from the athlete's hand?** The
   card supports both and says which. The athlete's history is the default because it is the
   athlete's own data; typing one is legitimate because the method leaves the pace to them.
5. **What is a "run at that heart rate"?** The app needs a rule for turning stored samples into
   "your pace at 128–148 bpm" — a median, a spread, a minimum sample size. That is a new
   derivation and it belongs in its own note.
6. **Does the zone lane get a pace-zone model?** The ladder's heart-rate axis uses the athlete's
   **heart-rate** zones, which exist and are dated; no pace-zone model is needed, and none is
   proposed.
7. **The two bases' ladders.** The 80/20 ladder is banded against the athlete's threshold and
   avoids the middle; its rungs are a different set. The app holds one base at a time, so the
   second ladder is a separate card the athlete switches to, not a second ladder on one card.
8. **How wide is the threshold band?** The book anchors LT pace between the 15K and 10-mile race
   paces — for the canonical athlete those differ by **one second per kilometre** (4:16 and 4:17) —
   so the book's own band is nearly a point, and the ladder draws the wider 15K-to-half-marathon
   span in which the "hour pace" actually falls. Whether the card shows the narrow book band, the
   wider span, or both (the narrow one marked inside the wide one) is a product choice; the note
   and the caption say which is which either way.
9. **Who owns the five ways to find LT pace?** The lactate test, the 45–60-minute race, the
   45-minute time trial, the training-software estimate and the heart-rate estimate (book ch. 1,
   ch. 7) are the method's own. The threshold-setup thread designs the athlete-facing version;
   the ladder names only the route it used.
10. **Does the curve need the 10-mile equivalent?** The curve's session bands use the hour pace
    and the long run's two ends; a narrower 15K-to-10-mile band would move the threshold session's
    edge by a second or two. The choice follows the ladder's band.
