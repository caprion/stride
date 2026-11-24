# Plan bases — the spec for the core's plan module

*24 November 2026. **Exploration: a spec and the data behind it.** No code changes under `crates/`, `frontend/`, `android/` or `desktop/`. This prepares step 4b (the plan bases) and it runs alongside step 3 (the API routes).*

## 0. What this is

**Two files:**

| file | what it holds |
|---|---|
| `exploration/plan-templates.json` | the **four shipping programmes**, transcribed cell by cell from `exploration/pfitzinger-spine.md` §2.3 — the 10K tiers 1–2 and the half-marathon tiers 1–2 — with the printed weekly mileage and the reconciliation against it |
| `exploration/plan-bases-spec.md` | this file: how a programme becomes an adopted, dated plan, and what it records |

**What it is built on:**

* `exploration/pfitzinger-spine.md` — the numbers, the run types and the read cell by cell schedules (§2.3). Every cell in the JSON traces to a row of one of the four tables there; the spine's citations travel with the data.
* `exploration/decisions.md` — **D19** (the threshold's four candidates, one choice, dated), **D20** (the plan's pacing names the threshold it was paced with), **D24** (the surface speaks the runner's language).
* `exploration/language.md` — the vocabulary, and §4's exemption for **the plan's own session names**.
* `docs/contract/voice.md` — the ban list and the seven string types.
* `crates/stride-core/src/plan.rs` — today's `PlanSession { date, kind, target_distance_m }`, `PlanSource`, `PlanBlock`; `crates/stride-store/src/schema.rs` — `plan_blocks`, `plan_sessions` (`session_type`, `title`, `details`, `target_distance_m`, `pace_low_s_km`, `pace_high_s_km`), `threshold_observations` (whose `base` column already reads `'pfitzinger' | 'eighty_twenty'`).

**What this file does not do.** It writes no code, adds no route, changes no schema and names no string as final. Where a decision is the parent's to take, it is marked as such. It does not read the 80/20 schedule cell by cell — §10 says what that would need and invents nothing.

**A note on a citation.** The brief for this step cites "D8/D24" for *the app never suggests*. D8 as written in `exploration/decisions.md` is the destinations call (five destinations, zone bands in the chart, altitude on its own scale). The standing call the brief means is the **ban list** (`docs/contract/voice.md`: no advice, no *should*, no *recommended*) together with **D24**. This spec cites those two and leaves D8 alone.

---

## 1. The three objects

### 1.1 The template — a programme, undated

A template is **data**, not code: the shape the JSON file uses.

| field | what it is | example |
|---|---|---|
| `id` | the catalogue's stable key | `pfitzinger-half-marathon-tier-2` |
| `label` | the programme's own name | `half marathon tier 2` |
| `method` | whose method it is — the base's name | `Pfitzinger` |
| `distance_m` | the goal distance | `21097.5` |
| `weeks` | the plan's length, counted down from 11 to 1, then race week | `12` |
| `entry_mileage` | the tier's **entry requirement** — the weekly mileage the athlete is expected to be at before the plan | `{"min_mi": 40, "max_mi": 50, …}` |
| `plan_mileage` | the band the plan itself runs in | `{"min_mi": 46, "max_mi": 63, "printed_km": "73–102 km"}` |
| `goal_race` | the goal race's distance, its printed label and its options (the 10K plan's race is "8K or 10K") | see the JSON |
| `weeks_detail[]` | the weeks, each with its seven days | |
| `source` | where it was read from | `spine §2.3.4`, book ch. 12 |
| `notes[]` | the facts a reader needs | |

### 1.2 The week

| field | what it is |
|---|---|
| `week` | the week's number: `11` … `1`, `0` for race week, negative for the weeks after the race |
| `weeks_to_go` | `11` … `1`, `0` for race week, absent for the weeks after the race |
| `label` | "11 weeks to go", "race week", "post-race recovery week", "transition week" |
| `in_plan` | `true` for the twelve plan weeks, `false` for the weeks after the race |
| `recovery_week` | present and `true` only where the book prints one (week 8, in all four tiers) |
| `printed_volume_mi` / `_km` | the weekly mileage **as the book prints it** — `null` where the book prints `?` or `—` |
| `volume_check` | the reconciliation: `cells_mi` (the sum of the printed daily miles), `reconciles` (`true`/`false`/`null`), and a note when it does not |
| `days[]` | the seven days, Monday first |
| `source` | the spine row this week came from |

### 1.3 The session (the day)

| field | what it is |
|---|---|
| `day` | `mon` … `sun` |
| `kind` | the **method's** kind — one of the thirteen in §2 |
| `kind_label` | the book's own short form as the spine's legend prints it: `Rest`, `GA`, `GA+s`, `End`, `Prog`, `LT`, `VO₂`, `SR`, `Rec`, `RP`, `RP+s` |
| `distance_mi` / `distance_km` | the day's distance; **absent** on a rest day and on a cell whose number could not be read — never zero |
| `detail` | the spine's own parenthetical, verbatim: `"3 × 8 min, jog 3"`, `"6×12 s uphill + 6×100 m strides"`, `"2 sets of 4 × 200 m @ 800 m–mile race pace, jog 200 m, 4 min between sets"` |
| `cell` | the whole cell as the spine prints it — the traceability anchor |
| `race_distance_m` / `race_distance_label` | on a race day: the race's own distance (the 5K, the 8K–10K, the half marathon) |
| `distance_basis` | on a race day: that the printed number is the miles the schedule counts for the day in the weekly total, and not the race's distance |
| `lt_last_mi` / `lt_last_km` | on a progression long run: how many of the last miles are at LT pace |
| `readability` | present only where a cell could not be read: what the spine prints and what is missing. **Absent throughout the four programmes** — every cell is now readable |

**Miles and kilometres both**, everywhere a distance appears, so the app can print in the athlete's own unit. Kilometres are derived from the printed miles at 1 mile = 1.609344 km, to 0.1 km; the spine's own printed kilometre bands are kept beside them (`printed_km`).

---

## 2. The kinds of run

The method's names and the runner's names are two different vocabularies, and the app keeps them apart.

* The **method's** name — `GA+s`, `endurance`, `LT`, `VO₂max` — is the book's, and it may be shown: `exploration/language.md` §4 exempts **the plan's own session names** ("Endurance, Intervals, Long run") from the plain-form rule.
* The **runner's** wording is what the surface leads with.

| the method's kind | the book's legend | the app's session kind | the runner's wording on a surface | the method's name, which may ride beside it |
|---|---|---|---|---|
| `rest_cross` | Rest | `rest` | rest or cross-training | — |
| `ga` | GA | `easy` | easy | general aerobic |
| `ga_speed` | GA+s | `easy` | easy, with strides | general aerobic + speed |
| `endurance` | End | `long` | long run | endurance |
| `progression_long_run` | Prog | `long` | long run, finishing fast | progression long run |
| `lt` | LT | `quality` | tempo | lactate threshold |
| `vo2max` | VO₂ | `quality` | intervals | VO₂max |
| `speed_repetitions` | SR | `quality` | speed repetitions | speed repetitions |
| `recovery` | Rec | `easy` | recovery | recovery |
| `race_pace` | RP | `quality` | race pace | race pace |
| `race_pace_speed` | RP+s | `quality` | race pace, with strides | race pace |
| `tune_up_race` | tune-up race | `race` | race | tune-up race (5K / 8K–10K) |
| `goal_race` | goal race | `race` | race | the goal race |

**Why the app's kind is a bucket.** The core's `SessionKind` today is `Easy | Long | Quality | Rest | Other(String)` and the store's `session_type` is a short word; the ledger compares the plan's kind against the watch's evidence, so the **method's kind must travel as data, not as a label**. The app's kind is what the surface prints; the method's kind is what the ledger reads. The plan's own `title` and `details` columns carry the method's name and the cell's detail.

**Two names, one day, never both leading.** The runner's wording is the label; the method's name is the title beside it; the detail (the cell's parenthetical) is the sentence under it. Nothing on a surface says "GA+s 6 mi".

---

## 3. Adopting a template

### 3.1 The athlete's four choices

| the choice | what it is | what the app does with it |
|---|---|---|
| the **programme** | one of the four templates | states its method, its goal distance, its length, its entry requirement and its mileage band |
| the **tier** | which level of that programme | the app states the entry requirement; the athlete chooses (see §3.2) |
| the **start date** | the Monday of the plan's first week | must be consistent with the race date (§4) |
| the **goal race** — its distance and its date | the 8K or the 10K, or the half marathon; the race's day | anchors the twelve weeks; the race's distance must be one the template prints |

### 3.2 The entry requirement, beside the athlete's own mileage

The app **states** and **never suggests**:

* the tier's entry requirement, as the book prints it — 25–35 mi/wk for tier 1, 40–50 mi/wk for tier 2 (spine §2.2), in the athlete's unit;
* the plan's own band — 30–42, 45–57, 31–47, 46–63 mi/wk;
* the athlete's own recent weekly mileage, computed from their runs.

Three facts, side by side, unranked:

> **This level asks for 25–35 miles a week.**
> **This plan runs 30–42 miles a week.**
> **Your last four weeks: 31, 34, 29, 33.**

The app does not say which tier fits, does not mark one as recommended, does not compare the athlete to the book, and does not colour a number. The ban list forbids *should*, *recommended* and *need to*; D24 forbids the internal nouns.

### 3.3 What adoption refuses

A refusal is a fact with a reason, never a verdict (the precedent is the plan import: "a file that cannot be read is refused"). Adoption is refused when:

* the start date and the race date cannot both be true — the race date does not fall in the twelfth week (§4);
* the goal race's distance is not one the template prints (the 10K template's race is the 8K or the 10K; the half-marathon template's race is the half marathon);
* the template's own weeks do not reconcile against their printed weekly mileage — **no template is in that state today**: all 54 week-rows of the four programmes reconcile (§11.1).

---

## 4. The dates

### 4.1 The twelve weeks

The plan is **twelve weeks**: the weeks-to-go counted down from 11 to 1, then race week. The book's schedules are printed that way, and the count-down is the template's own numbering — it is not the app's calendar week.

Given the goal race's date:

```
race_monday   = the Monday of the week containing the goal race
week  0 (race week) starts   race_monday
week  n (n = 11 … 1) starts  race_monday − n × 7 days
plan start    = race_monday − 11 × 7 days      (the Monday of week 11)
```

The plan's **12th week is race week** and the goal race is the **Sunday** of that week (§4.3).

### 4.2 Which week is the race week

The week whose Sunday carries the goal race — the twelfth week of the plan, and the only week whose numbering is `0`. Every other week is numbered by the weeks remaining to the race, so the number on the card is the runner's own countdown.

### 4.3 The race's day

The book fixes the goal race's day: **the half marathon is on Sunday** in the half-marathon plans, and the 10K plan's race is on **Saturday** (the 10K tier 1's race week puts the race on the Saturday, and the tier 2's race week too). The book's own rule for a race on another day: **for a Saturday half marathon, drop one GA run in race week and shift everything a day earlier** (spine §2.2).

So the adopted plan has two shapes, and the athlete's race date picks one:

* the race is on the book's day for that template — the week is laid out as printed;
* the race is on another day — the race week is shifted so the race falls where the athlete raced, and the shift is stated on the card as a fact ("your race is on Saturday; this week's sessions are a day earlier, and one easy run is not in this week").

No other week moves. The taper, the recovery week and the twelve-week shape are unchanged.

### 4.4 The weeks after the race

The templates carry them, and they are part of the plan:

| template | the weeks after the race |
|---|---|
| 10K tier 1 | one post-race recovery week (32 mi) |
| 10K tier 2 | one post-race week (39 mi) |
| half marathon tier 1 | a post-race recovery week (30 mi) and a transition week (38 mi) |
| half marathon tier 2 | a post-race recovery week (37 mi) and a transition week (50 mi) |

They are dated: the week after race week, and the week after that. The plan block's end date is the last of them.

---

## 5. The pace

### 5.1 The rule that gives each session its pace

From the spine §5.2–§5.3 (the rules are the book's; the arithmetic is the core's, from step 2's `training_paces`):

| the session's kind | the rule | the shape of the answer |
|---|---|---|
| `ga`, `ga_speed` | **no pace rule at all** — 70–81 % of max HR (62–75 % of HR reserve) | a heart-rate band; the pace is the athlete's own data at that heart rate, or absent |
| `recovery` | 2 min/mile (75 s/km) slower than the 15K–half-marathon race pace; below 76 % of max HR | one band |
| `endurance` | **two ends**: 10K pace × 1.20–1.33 at the start, 15K–HM pace × 1.17–1.29 at the finish | two windows on one session |
| `progression_long_run` | as the long run, then the **last *k* miles at LT pace** | two windows, the second one LT |
| `lt` | **LT pace** — the pace you could race for about an hour (the 15K-to-10-mile route, or 10K + 10–15 s/mi, or 5K + 20–30 s/mi) | the threshold's band, or the method's rule on the athlete's race |
| `vo2max` | 3K–5K race pace | one band |
| `speed_repetitions` | 800 m–mile race pace | one band |
| `race_pace`, `race_pace_speed` | the goal race's own pace | one band |
| `tune_up_race` | raced near maximal effort | the race's own pace |
| `goal_race` | the goal race's own pace | the race's own pace |
| `rest_cross` | nothing | no pace |

**The session's paces are computed once, at adoption, and stored on the session** (`pace_low_s_km`, `pace_high_s_km` — the columns already exist). A session's structure — "3 × 8 min, jog 3", "2 sets of 4 × 200 m @ 800 m–mile race pace" — is the method's own text and travels as the session's detail.

### 5.2 The threshold it was paced with (D20)

> *"The plan card names the threshold it was paced with (value and date); the ladder names the rule."*

So an adopted plan records, once:

* the **threshold in force at adoption** — its value (a pace band, a heart rate, or both) and **its date**;
* the fallback, when there was no threshold chosen: the **method's rule on the plan's anchor race** (the goal race's own time). A base may be adopted with no threshold chosen (D20.4), and then the card says which of the two paced the plan.

The threshold is read in **two** places only — the plan's pacing and the 80/20 bands (D20). The ladder keeps the method's rule on the race time; the plan's sessions keep the bands they were given. The two are never drawn on one axis (D20.1).

### 5.3 When the threshold changes afterwards

> **A later threshold choice does not re-pace the plan that is running.**

The plan keeps the bands it was given. A threshold chosen on 3 December does not change the pace on a session dated 10 November, and the plan card still names the threshold the plan was paced with, with its date. Re-pacing is an **explicit act** — the athlete asks for it, the plan's bands are rewritten, and the new threshold's value and date are recorded with them. The plan card then shows the threshold it was paced with, and the history shows the earlier one.

This is the same rule the threshold's own history already follows: "a plan adopted in March is paced by March's number" (`crates/stride-store/src/schema.rs`, `threshold_observations`).

---

## 6. What an adopted plan records

| what | where it lives | notes |
|---|---|---|
| the template it came from | `plan_blocks` | a new column: `template_id` + `template_version` — so a later edit to the catalogue cannot alter a plan that is running |
| the base it follows | `plan_blocks` | a new column: `base` — `'pfitzinger' \| 'eighty_twenty'`, matching the column `threshold_observations.base` already has |
| the source | `plan_blocks.source` | **unchanged**: an adopted programme is a **prescription** (`'prescribed'`) — the athlete chose it. The base is a separate fact, not a fourth source |
| the tier | `plan_blocks` | a new column: `tier` |
| the entry requirement | `plan_blocks` | a new column: the snapshot of the requirement **as it was at adoption**, so a later change to the template does not rewrite the athlete's plan |
| the start and end dates | `plan_blocks.start_date` / `.end_date` | the existing columns |
| the goal race | `plan_blocks.goal` | the existing column: the distance and the date |
| the pacing anchor | `plan_blocks` | new columns: the threshold's value and date, or the fallback and the race it came from |
| the sessions | `plan_sessions` | the existing columns, plus a new `plan_kind` (the method's kind) and `plan_detail` (the cell's text) |
| the paces | `plan_sessions.pace_low_s_km` / `.pace_high_s_km` | the existing columns |

**A note on `PlanSource`.** The spine §4.3 suggests giving `PlanSource` a fourth and fifth value for the method (`Pfitzinger | EightyTwenty`). This spec does **not**: the source says *where the plan came from* — a baseline inferred from the athlete's history, or an intent the athlete chose — and the base says *whose method it follows*. An adopted Pfitzinger plan is `source = 'prescribed'` and `base = 'pfitzinger'`; a baseline stays `source = 'baseline'` with no base. The threshold's own table already keeps the two apart (`threshold_observations.base`).

**Absence is not zero.** A session with no pace — a general aerobic run, because the method has no pace rule for it — has **no pace row and no pace value**. It is not zero, and it is not "no pace yet" for a reason the app has to invent: the reason is that the method fixes a heart-rate band and no pace.

---

## 7. What the plan card shows

| the row | what it says | where the words come from |
|---|---|---|
| **the plan you follow** | "the Pfitzinger half marathon plan · level 2" | the template's `label` and `method`; "the base" is **"the plan you follow"** and "a tier" is **"the level of the plan"** (`language.md` §2) |
| **the week's sessions** | each day: its kind in the runner's wording, its distance in the athlete's unit, the detail under it, and the pace band when there is one | §2 and §5 |
| **the pacing's threshold** | "paced with the threshold of 3 Nov 2026 · 4:52–5:04 /km" — or, with no threshold chosen, "paced by the rule on your race of 12 Oct 2026" | D20.1 |
| **the entry requirement** | "this level asks for 25–35 miles a week" beside "your last four weeks: 31, 34, 29, 33" | §3.2 |
| **the week itself** | "5 weeks to go", "recovery week", "race week" | the template's own numbering |
| **the plan's volume** | the week's printed mileage, in the athlete's unit | the template's `volume_check` |

The card does **not** say: whether the athlete is keeping up (the adherence count already exists and is a count, not a score), whether the tier suits them, what to do next, or anything about a threshold they have not chosen.

---

## 8. The tune-up race, the race week, and the weeks after

**The tune-up races.** Two per plan, at **4 and 2 weeks to go**: a **5K** in the 10K templates, an **8K–10K** in the half-marathon templates. On the card a tune-up race is a session of kind `race` with its own distance as the target — the 5K is 5000 m, the 8K–10K is 8000 or 10000 m — and the method's note rides with it: the race is shorter than the goal distance and *"you will not be completely fresh… expect residual fatigue"* (spine §1.1). The app states that as a note, and never as a warning about the athlete's readiness.

**The race week.** The twelfth week: a race-pace rehearsal carrying the goal race's own pace — **1,200 m at 8K–10K race pace** in the 10K plans, **2 miles at half-marathon race pace** in the half-marathon plans — the goal race itself on the book's day, and the rest of the week as recovery runs and a rest day.

**A race day's printed miles.** On a race day the schedule prints a number (8, 9, 10, 11, 16, 17) that is the **miles it counts for that day in the weekly total** — it is not the race's distance. The plan's **target distance** for a race session is the race's own distance; the printed number stays in the volume check and nowhere else. This is the shape of the JSON's `distance_basis`.

**The weeks after the race.** The post-race recovery week, then the transition week (half-marathon templates), as printed. The book also says what follows: *"after the half marathon, you need about 10 days to two weeks to fully recover"* (spine §2.2) — that belongs in the note, not on the card.

---

## 9. The five decisions

> **B1 — The template is a versioned catalogue; an adopted plan is a dated copy of one of them.** The catalogue may be corrected later; a plan that is running keeps the template's id and version it was adopted from, so a later correction cannot alter it.
>
> **B2 — The tier is the athlete's choice, and the app states the entry requirement beside the athlete's own mileage and never suggests one.** Three facts side by side, unranked; no advice, no recommendation, no comparison of the athlete to the book.
>
> **B3 — The race date anchors the plan; the race week is the twelfth week and the goal race is the book's day for that template.** The start date and the race date must agree, or adoption is refused with the difference stated; a race on another day shifts the race week alone, and the shift is stated as a fact.
>
> **B4 — A session's pace is computed once, at adoption, from the threshold in force then; the plan stores the bands it was given and the threshold's value and date.** A later threshold does not re-pace a running plan; re-pacing is an explicit act. With no threshold chosen, the method's rule on the plan's anchor race paces the plan and the card says so.
>
> **B5 — A race day's printed miles are the day's mileage in the weekly total, not the race's distance.** The session's target distance is the race's own distance; the printed number is kept only for the volume check.

---

## 10. The 80/20 base — the same shape, not yet read

The app holds **one plan at a time** (spine §4.3), so the 80/20 base is a **second method**, not a second tier. It would be the same three objects — template, week, session — with `method: "80/20"`, and it would need:

1. **Its schedule read cell by cell.** The published plans are **level-based and time-based** — the Level 1 marathon week is *Mon rest · Tue 35 min · Wed 30 min · Thu 30 min · Fri 32 min · Sat 30 min · Sun long run 77 min* (spine §4.2). A template in the same shape as the Pfitzinger one needs **distances** per cell, and a published week gives **minutes**. The minutes become distances at adoption time from the athlete's own paces — which means the template's cell carries a *duration*, and the core's `PlanSession` has no duration field today (`{ date, kind, target_distance_m }`). **A duration on a session is the first thing the core would need.**
2. **Its own band scale.** 80/20's zones are set from the **threshold** (LTHR): Z1 72–81 %, Z2 81–90 %, X 90–95 %, Z3 95–100 %, Y 100–102 %, Z4 102–105 %, Z5 >105 % of threshold speed — seven zones with two of them (X and Y) deliberately avoided. The app's zones today come from the watch (max HR based) and there is no threshold-relative scale. The threshold's own `base` column already reads `'eighty_twenty'`, so the storage is expecting this.
3. **A threshold that is LTHR.** The 80/20 rule takes a recent race at 5K/10K/HM, a 20-minute time trial, or the talk test, and is **retested every 6–8 weeks**. The core's four candidates (D19) are the watch's zone model, from a race, from the field test, and typed — the 80/20 rule is a fifth source, and it would need a candidate of its own or a typed value tied to the 80/20 base.
4. **Its own entry requirement, in hours rather than miles.** The levels are described by sustainable weekly hours, not by a weekly mileage band, so the card's "this level asks for X" line is a different kind of number from the Pfitzinger tiers'.
5. **Its source's terms.** The 80/20 plans are a **published product** (Fitzgerald's), not a book chapter; reproducing its week-by-week schedule has its own terms, and its bands are **product documentation** while Seiler's rule is stated in blood lactate (spine §6.8). The app's note must say which is which.
6. **Its plan length and its recovery weeks.** "Recovery weeks fall **every third week** in all of our run plans" — a shape the Pfitzinger templates do not have (their recovery week is a printed week 8), and the adopted plan's own week structure would differ.

**None of these weeks is invented here.** The schedule is not read cell by cell, so there is no 80/20 template in the JSON, and the spec above is what would have to be settled before there could be one.

---

## 11. What step 4b will have to touch — and what is frozen

**The core** — a new module beside `plan.rs` (say `crates/stride-core/src/plan_bases.rs`) holding: the template catalogue (the JSON, compiled in or read at build time), the adoption (dates, tiers, the refusals), the pacing assignment, and the plan card's rendering. Every sentence it renders goes in `crates/stride-core/src/strings.rs`; `strings.json` is generated, never hand-edited.

**The store** — a schema change (v7): `plan_blocks` gains `template_id`, `template_version`, `base`, `tier`, the entry-requirement snapshot and the pacing anchor; `plan_sessions` gains `plan_kind` and `plan_detail`. `plan_blocks.source` and `plan_sessions.session_type` stay as they are.

**The API** — `GET /plan/bases` (step 3's route) serves the catalogue: each programme's id, label, method, distance, length, entry requirement, mileage band and goal race; the weeks come with it or on a second route under the same prefix. **Adoption needs a write route**, and the route table is frozen: a new route has to be added to the bridge's table (`PhoneRoutes.kt`), to `PhoneRoutesTest.kt`, and to `docs/contract/api-paths.md` **with a `PLAN.md` note in the same change** (the contract-freeze rule). This is the step 4b thread's to settle, and it is a change to a frozen file — it should be raised with the parent before it is written.

**The frontend** — the plan card's week (step 5) reads the adopted plan's sessions and their paces; nothing else changes.

### 11.1 The check, kept

Every week of all four programmes carries `volume_check`: the printed weekly mileage and the sum of the week's seven days. **All 54 week-rows reconcile** — every `reconciles` is `true`, and no week carries a `source_issue` or a `readability` note, because every cell is now readable.

**The check stays in the data, and the rule behind it stays too.** The spine's own method note (spine §2.3) says it plainly: the schedules print as **two images per plan** — the Monday–Thursday half and the Friday–Sunday-plus-volume half — with **different row heights**, so the halves must be paired by **row index**, not by pixel position; and **a volume that does not sum is a finding, not a licence to adjust a cell**. The earlier pass paired the halves by pixel position and produced a wrong half-marathon tier-2 table; the weekly sum is what caught it.

So the rule for this data, and for step 4b: **no cell is ever adjusted to make a week sum.** A week that does not reconcile is reported, with the gap, and the cells stand as the book prints them.

**How the JSON is checked.** The four tables are parsed out of `exploration/pfitzinger-spine.md` §2.3, and the JSON is then cross-checked against the file by a second, independently written pass: every day's `cell` text compared with the spine's cell, every kind and every mileage re-derived by a different parsing path, every week's sum recomputed from the JSON's own day distances, and every kilometre value recomputed from its miles. The last run: **54 week-rows, 378 days, every cell matching the spine verbatim, every week reconciling, every kilometre value checking out.**

---

## 12. Open questions — the parent's to settle

1. **The frozen route table.** Adoption needs a write route. Does it travel with a `PLAN.md` note in the same change, as the contract-freeze rule says?
2. ~~The half-marathon tier 2's seven rows~~ — **closed**: the spine was re-read by row-index alignment and all four programmes now reconcile (§11.1).
3. ~~The 10K tier 1's week 5 and post-race week~~ — **closed**: week 5 reads 41 miles and the post-race week reads 32 miles, both reconciled (§11.1).
4. **The session's duration.** The 80/20 base needs it; the Pfitzinger templates do not. Does the core's session gain a duration field in the same change, or only when the second base lands?
5. **The `plan_kind` column.** A column, or the method's kind folded into the session's `title` and `details`? The ledger needs it as data; the surface can carry it as a title.
6. **A Saturday race.** The book's shift rule is for a Saturday half marathon. Does the 10K template, whose race is already on the Saturday, need a shift rule of its own for a Sunday race?
