# Exploration — race-anchored and level-anchored plans for Stride

> **Status: exploration only.** Nothing here is implemented, nothing is
> committed, nothing is merged. This file is untracked in the working tree so
> it can be read, argued with and thrown away.
>
> It answers three questions: (1) what do RunStrong, McMillan and the other
> plan products actually do, structurally? (2) what shape would the same
> idea take inside Stride, under Stride's voice? (3) **where in this
> repository** would each piece land?

---

## 1. What I scanned

| Source | URL | What it is | What it asks for | How a week is expressed |
|---|---|---|---|---|
| **RunStrong** (GeeksonFeet) | `runstrong.geeksonfeet.com/home`, `/races` | Race-first planner. Free *Standard* plans (ready-made per distance) vs paid *PRO* *personalized* plans that adapt to goal race, timeline and current fitness. Race Finder + race calendar; watch sync to Garmin/COROS/Suunto; strength builder; calendar export; completed runs "scored against the plan"; edit mode to rearrange workouts. | Goal race (from a race list or a date), distance, current fitness, timeline | Standard plans = ready-made structures per distance; PRO = generated around the race date, with phases and per-session targets, plus an "execution score" for a finished run |
| **McMillan Running** (Coach Greg McMillan) | `mcmillanrunning.com/training-plans`, delivered on TrainingPeaks (`finalsurge.com/trainingplans/McMillanPlans`) | A **library** organised by distance (800 m → 100 mile), race-specific (Boston, Berlin, Chicago, London, NYC, Tokyo, Houston, Honolulu, Sydney, California), specialty (10-day cycle, hilly, downhill, BQ-pace targets), ultra/trail, masters, high school, and **preparatory** blocks (Rebuild, Base, Hill, Stamina, Speed, Prehab). Paces come from the McMillan Running Calculator. Coach notes + prehab/strength built in. | Goal distance (+ optional goal time/BQ), **runner level**, **runner type** | **Levels** — TrainingPeaks describes 5 levels (L1 new/returning, 3 days/week; L2 finishing faster, 3–5 days + 1 hard session; L3 4–6 days, long run 60–75 min, 1–2 hard; L4 4–7 days, long run 90 min; L5 5–7 days, long run 120 min, 2 hard). The current site presents Beginner / Intermediate / Advanced / Elite. **Runner types** — Combo (default), Endurance Monster, Speedster. Sessions are **pace/zone-based** (T1–T5 from the calculator) rather than distance-only |
| **Runna** (Strava) | `runna.com`, `support.runna.com` | The most explicit *plan-type taxonomy* of the set: Race, Distance, Improvement, General (Run Faster / Run Further / Run to Maintain / Train Your Way), Beginner & Return (New to Running, Return to Running, Path to parkrun), Functional, Triathlon, Other (post-race, post-injury, post-partum). Custom plan: 6–26 weeks, any distance 5–50 km | Distance **or** specific race (search or manual: name, distance, date), duration, experience level, recent PB, days you want to run, long-run day | Session vocabulary: easy, long, hills, intervals, recovery, strides, tempo, time trial. **Distance-based** by default, effort/RPE for beginner plans. Deload weeks every 3–4; B-race handling; holiday/"not feeling 100%" adjustments; PDF export; strength and mobility as separate tracks |
| **YearRoundRunning generator** | `yearroundrunning.com/running-plan-generator` | The smallest useful generator: race + goal → pick available training days + long-run day + strength → balanced weekly plan (easy / workout / long / recovery) | Race distance & goal, available days, long-run day, strength on/off | Distances per session, one quality session a week, one long run a week |
| **Hal Higdon** | `halhigdon.com/training-programs` | The static-table ancestor: hand-written week tables per level and distance | Level + distance | A literal calendar table; long run progresses by mile; the "race week" table is written out by hand |
| **None to Run** | `nonetorun.com/training-plans` | C25K-style ladder with an explicit "graduates to" chain (Can run 25 min → 5K → 10K → half) | Current capability (how long you can run) | Weeks are run/walk intervals — **effort-based, no pace targets** |
| **Garmin** | TrainingPeaks/Garmin workout schedules | The export target every one of these supports | — | A CSV of scheduled workouts with dates, names and structured steps — **already readable by Stride** (see §3) |

### 1.1 The anatomy they share

Stripped of branding, every one of these products is the same machine:

1. **An anchor** — a race (date + distance) *or* a duration *or* a distance
   target *or* a level ladder.
2. **A level** — a named rung (McMillan's 1–5, Runna's beginner→elite,
   Hal Higdon's novice/intermediate/advanced). Level controls **how much**,
   never **what**.
3. **Phases (mesocycles)** — base → build → peak → taper → race week, usually
   3–4 weeks per block with a cutback week every 3rd or 4th.
4. **A weekly template** — a fixed set of *session roles* placed on chosen
   weekdays: long run, easy/recovery, one or two quality sessions
   (tempo / intervals / hills / strides), optional strength or cross-training.
5. **A target per session** — expressed as **distance** (Runna, most free
   generators), **pace band / zone** (McMillan), **effort** (beginner plans),
   or **workout structure** ("3 × 8 min", Garmin-style steps).
6. **A rule for the ends** — taper, race week, and usually a *recovery* block
   after.
7. **An after-the-fact layer** — scoring the run against the plan (RunStrong),
   watch sync, PDF/ICS export, holidays, B-races, and "not feeling 100%"
   auto-adjustments (Runna).

Two things they all agree on that are worth copying:
**levels change volume, not session types**, and **the anchor is the race
date, not the plan**.

Two things worth *not* copying blindly: auto-adjusting a live plan from
observed performance (that is coaching), and hiding the target's provenance
(RunStrong's "execution score" is a verdict).

---

## 2. Where Stride stands today

Stride can **read** a plan. It cannot **build** one, and it has no concept of
a race, a level, a phase or a week number.

| Thing | Where it lives today |
|---|---|
| Plan engine (baseline inference, week, adherence, results, 10 % distance rule) | `crates/stride-core/src/plan.rs` |
| Plan **sources** — `baseline` / `prescribed` / `typed` | `crates/stride-core/src/plan.rs:52` |
| Session **kinds** — easy / long / quality / rest / other | `crates/stride-core/src/plan.rs:90` |
| A session is a date, a kind, a **target distance** and a label ("10 km · easy") | `crates/stride-core/src/plan.rs:124`, `:291` |
| A block = source + name + dated sessions | `crates/stride-core/src/plan.rs:154` |
| Baseline inferred from ≥8 weeks / ≥12 runs, median weekday distances | `crates/stride-core/src/plan.rs:588` |
| Adherence / done / planned / **not done** (never for a baseline) | `crates/stride-core/src/plan.rs:360`, `:383` |
| Distance comparison shown only beyond 10 % | `crates/stride-core/src/plan.rs:519` |
| Import readers — markdown, CSV, JSON, **Garmin workout schedule CSV** | `crates/stride-api/src/planfile.rs` |
| Closed import vocabulary (easy/long/rest/tempo/interval/race/strength/quality; else *unclassified*) | `crates/stride-api/src/planfile.rs:123` |
| Stored word → core kind (unknown words keep their own name) | `crates/stride-api/src/plan.rs:217` |
| Stored source → core source | `crates/stride-api/src/plan.rs:231` |
| Plan routes (`/plan/current|today|week|blocks|sources|adherence|results|import`) | `crates/stride-api/src/router.rs:390-430`, `crates/stride-api/src/routes/plan.rs` |
| Import route | `crates/stride-api/src/routes/plan.rs:313` |
| Tables `plan_blocks`, `plan_sessions` (incl. unused `pace_low_s_km`, `pace_high_s_km`) | `crates/stride-store/src/schema.rs:85-108` |
| **One active block per source**, enforced on write | `crates/stride-store/src/repos/plan.rs` (`upsert_plan_block`) |
| Block identity = source + name + start_date | `crates/stride-store/src/repos/plan.rs` |
| Plan week screen (source banner, week strip, day detail, counts, sources) | `frontend/src/pages/PlanWeekPage.tsx` |
| Import button | `frontend/src/components/PlanImport.tsx` |
| Today shows the planned session as an *option* | `frontend/src/pages/TodayPage.tsx:330` |
| Plan API paths in the app | `frontend/src/lib/api.ts:82-89`, `:260-267` |
| Plan strings (typed, linted) | `frontend/src/i18n/catalogue.ts:389-405`, `:603-617` |
| Frozen contract for the plan routes | `docs/contract/api-paths.md:31`, `:133`, `:144` |
| Voice rules and the ban list | `docs/contract/voice.md`, `PLAN.md` §0 |
| Zone model — **heart rate only**, five bands, dated, from the watch | `crates/stride-core/src/zones.rs` |
| Tests that would guard a generator | `crates/stride-core/tests/plan.rs`, `crates/stride-api/tests/plan.rs`, `crates/stride-store/tests/plan.rs`, `frontend/tests/voice.test.ts` |

Gaps against the anatomy in §1.1:

- **No anchor.** There is no race entity, no goal date, no goal distance.
  `plan_blocks.goal` exists but is a free-text 255-char column and is never
  written by the import path.
- **No level.** The baseline infers *what the athlete already does*; it never
  offers a rung above or below it.
- **No phases / week numbers.** A block is a flat list of dated sessions.
- **No pace or zone target.** `plan_sessions.pace_low_s_km` /
  `pace_high_s_km` exist in the schema and are carried by the store, but the
  import never fills them and the core's `PlanSession` cannot hold them, so
  the label can only ever be a distance + a kind. There is no pace zone
  model in Stride at all (zones are heart-rate bands), so a pace target has
  no athlete-owned basis to come from yet.
- **No week-level structure.** No weekly total, no cutback week, no taper.
- **The source vocabulary is three words wide.** `baseline | prescribed |
  typed` — and `typed` means "written in by hand", so a *generated* plan has
  no honest home today.
- **One active block per source** — good, but a generated plan plus an
  imported plan would fight over the same slot unless it gets its own source.

---

## 3. The shape

### 3.1 The one idea

> **A plan is a shape the athlete chose, measured against their own history —
> never a verdict, never an instruction, never a live coach.**

Everything below follows from that sentence. The generator is *not* a coach:
it produces a block once, deterministically, from a **named, versioned
template** plus the athlete's own stated inputs, and then Stride treats that
block exactly as it treats an imported plan today — done / planned / not done,
with the 10 % distance comparison, with the source always named.

### 3.2 The three inputs, and where they come from

| Input | Source | Why it is safe under the voice |
|---|---|---|
| **Goal** — distance and date | The athlete's own choice (typed, or picked from a race list the app ships) | It is a fact about the plan, not a judgement |
| **Length** — weeks, or "until the race" | Derived from the anchor date | Deterministic |
| **Level** — how many days a week, how long the long run | **Recommended from the athlete's own baseline** (`baseline_from_history_at` already computes a usual week), but **chosen by the athlete** | A baseline is a description of what they do; the level is a choice about what they want to do. No "you are ready for level 3" |
| Weekdays, long-run day | The athlete's choice | — |

No level is ever *inferred silently*. The app may say "your usual week is 3
runs — here are the plans for 3, 4 and 5 runs a week", which is an **option
list, unranked**, exactly the `option` register.

### 3.3 The domain, in Stride's shape

```
PlanTemplate            a named, versioned table of plan shapes
  id                    "half-marathon-16w"
  version               1
  distance_family       5k | 10k | half | marathon
  week_templates[]      one per phase-week, as a table (role → day offset, distance factor, quality?)

PlanRequest             the athlete's inputs
  goal_distance_m       Option<f64>
  goal_date             Date
  level                 1..=5                 (McMillan-shaped)
  runs_per_week         3..=7
  long_run_weekday      0..=6
  start_date            Date
  include_strength      bool

build_plan(request, template, athlete_facts) -> PlanBlock
```

Three properties make this safe to put in the core:

1. **Pure and clock-free.** `build_plan` takes the anchor date as an argument,
   like every other function in `stride-core/src/plan.rs` ("there is no clock
   here"). It is deterministic: same inputs → same block, always.
2. **Templates are data, not code.** A JSON/Rust table per distance family,
   versioned in the repository, reviewable in a diff. A plan product is a
   table; the code is the printer.
3. **The athlete's history only ever *scales* what the athlete asked for.**
   The baseline may pick the starting long-run length so week 1 is not a
   shock; it never invents a level, and it never drops a session.

### 3.4 The session model — the real change

Today a session is `date + kind + target_distance_m`. A generated plan wants
more, and the honest way to say more is to widen the core type rather than
smuggle it into `details`:

```rust
pub struct PlanSession {
    pub date: Date,
    pub kind: SessionKind,
    pub target_distance_m: Option<f64>,
    pub pace_low_s_km:  Option<f64>,   // already in the store schema — unused today
    pub pace_high_s_km: Option<f64>,
    pub structure: Option<String>,     // "3 × 8 min @ threshold" — the athlete's wording
    pub label: String,                 // rendered by the core, as today
}
```

and the block gains the plan-level facts the generator needs to explain
itself:

```rust
pub struct PlanBlock {
    pub source: PlanSource,   // + a fourth source: Built
    pub name: String,         // "Half marathon · 16 weeks · 4 runs a week"
    pub sessions: Vec<PlanSession>,
    pub goal: Option<PlanGoal>,   // distance, date, race name
    pub phase_of: ...             // week index → phase (base/build/peak/taper/race)
}
```

The **fourth source** is the sharpest point of the whole design:

| Source | Means | Can say "not done"? | Currently |
|---|---|---|---|
| `baseline` | inferred from history | no | `plan.rs:52` |
| `prescribed` | imported from a file | yes | |
| `typed` | written in by hand | yes | |
| **`built`** | **generated in the app from a named template and the athlete's inputs** | yes (it *is* an intent) | **new** |

`built` needs: a store constant (`stride-store/src/model.rs:31`), a core
variant (`plan.rs:52`), the API mapping (`stride-api/src/plan.rs:230`), a
banner string in `crates/stride-core/strings.json`, a catalogue entry
(`frontend/src/i18n/catalogue.ts:389`), and the `one active block per source`
invariant in `repos/plan.rs` continues to hold — a built plan would not
displace an imported one.

### 3.5 The phases, and what they are allowed to say

| Phase | Weeks (typical, 16-week half) | What it does | Stride wording |
|---|---|---|---|
| **base** | 1–4 | volume only, all easy, long run grows | *Week 3 of 16 · base* |
| **build** | 5–9 | volume + one quality session | *Week 7 of 16 · build* |
| **peak** | 10–13 | race-specific quality, longest long runs | *Week 11 of 16 · peak* |
| **taper** | 14–16 | volume down, sharpness kept | *Week 15 of 16 · taper* |
| **race** | the race week | short, with the race on the day | *Race week · 12 Apr 2027* |

Cutback weeks (every 3rd–4th) fall inside the phase as a **week type**, not
as a phase, and are expressed as the plan's own fact: *Week 8 of 16 · build ·
cutback*.

Note what the phase name is **not**: it is not a verdict on the athlete
("you are peaking"), and it is not advice ("time to taper"). It is the name
of the template's block, exactly as "10 km · easy" is the name of a session.

### 3.6 The voice — the part that decides whether this ships

The ban list in `docs/contract/voice.md` forbids `should`, `must`, `take the
day`, `rest day` as a command, `missed`, `good`, `bad`, `ready to`,
`recommended`, `red`, `readiness score`. A plan generator is exactly the
thing that would want to say all of those. The rules that keep it honest:

- The **session** is an `option`, unranked, never conditioned on a vital.
  "Today's option — 16 km long" — never "you should do 16 km".
- **Never an imperative.** The label states the intent; the athlete reads it.
- **No verdict about the plan.** "not done" is a fact (the day passed, there
  was no run); "missed" is not; a *score* against the plan (RunStrong's
  "execution score") is not — Stride keeps the 10 % neutral distance
  comparison it already has (`plan.rs:519`) and nothing else.
- **The source is always named**, and now there are four names. A built plan
  says what it was built from: *"Built · half marathon · 16 weeks · 4 runs a
  week · from the v1 half-marathon template"* — provenance, not a claim.
- **The generator never adapts on its own.** No "if you missed Tuesday, we
  shift the week". The plan is a fixed record; the athlete may edit it (see
  §3.8).
- **No health advice, ever.** A plan that ramps a marathon long run does not
  get a disclaimer-laden paragraph about injury; that is a coaching
  conversation, and the app is a log.

### 3.7 The screen — where it lives

Not a new destination. Stride has four destinations and the plan is
contextual; the builder belongs inside the **Plan** screen:

```
Plan
├─ source banner            (now four sources, including "built")
├─ week strip + day detail  (unchanged)
├─ counts                   (unchanged: done / planned / not done)
└─ Sources
   ├─ Import a plan         (existing)
   ├─ Build a plan          (new)
   │   ├─ Goal: distance · date · name (optional)
   │   ├─ Length: until the race (N weeks) · or N weeks
   │   ├─ Level: 3 / 4 / 5 runs a week   ← the athlete's own week, first
   │   ├─ Long run on: [day]
   │   ├─ Strength and cross-training: on / off
   │   └─ Preview → Save
   └─ (the built block appears as a source, with its parameters)
```

The preview is the whole plan, as a list — it must be readable before it is
saved, because a saved block is a commitment to a shape.

### 3.8 What is deliberately *not* in the first shape

| Not now | Why |
|---|---|
| Auto-adjusting the plan from how the athlete performed | That is coaching, and it makes the plan unreadable as a record |
| A pace-band target | There is no pace zone model in Stride; `pace_low/high` exist in the store but no athlete-owned basis fills them yet. A pace target derived from a *guess* is the "guessing easy" bug the import path already refuses (`planfile.rs`) |
| Watch push / .ics export | Worth it, but it is a second contract. Note that the *Garmin workout-schedule CSV* reader already exists (`planfile.rs`, `Format::GarminCsv`) — a generated plan can be exported in exactly that format later, and read back in |
| A race finder with a live race database | The anchor is a date + distance. A race *name* is a label. Pulling a race list means a network dependency in a local-first app |
| "Not feeling 100 %" adjustments, holidays, B-races | All of them are real product features upstream; each is a rule set with editorial judgement baked in. Later, and one at a time |
| Strength/cross-training as first-class sessions | The kind vocabulary can carry `strength` (it is in the import vocabulary), but the store's `session_type` column is 24 chars and the core's `SessionKind` has no strength variant — a small, separable change |

### 3.9 The screen, in the app's own HTML

A working mock lives next to this file: **`exploration/plan-builder-mock.html`**
— three phone-width screens in Stride's own palette (azure/amber, 9 px cards,
the compact tab bar), with the builder as a three-step sheet that is actually
clickable.

What the mock settles, in order:

| Decision | Why |
|---|---|
| **The builder is a sheet inside the Plan screen**, not a destination | The destination set is four and it is *the* information architecture (`frontend/src/app/destinations.ts`). A fifth destination for "plans" would be a bigger change to the app than the feature deserves |
| **Three steps, then a preview** — goal → shape → preview | The preview is the plan, in full, before it is saved. A saved block is a commitment to a shape; the athlete must be able to read the shape |
| **The level chips are days-a-week, and the athlete's own usual week is on the same chip row** | It is the same unit McMillan uses (days/week + long run) and the unit Stride's baseline already computes. The usual week is *named*, never marked "recommended" — the ban list forbids that word, and the `option` register requires unranked choices |
| **Distance targets only, in the first slice** | There is no pace model of the athlete's own in Stride (`crates/stride-core/src/zones.rs` is heart rate). A pace target would have to be guessed, and the import path already refuses to guess a kind it does not know (`planfile.rs`) |
| **Phase and week are shown as part of the plan's own vocabulary** — "Week 6 of 16 · build" | A phase name is a template block name, exactly as "10 km · easy" is a session name. It is not a verdict on the athlete and not an instruction |
| **The race is a date + a distance + an optional free-text name** | No race database, no network, no scrape of a race list into a local-first app |
| **The plan screen keeps its shape** — same banner, same week strip, same day detail, same counts, same sources card | The one change is a fourth source in the banner and one more row in the sources card |
| **Today changes nothing** | The built plan's session is an `option` like any imported session; the other sessions are listed unranked; the distance comparison stays neutral |

Two things the mock deliberately does **not** have, because the voice does not
allow them: a plan score ("execution score", RunStrong) and any sentence that
links a session to a vital ("your sleep was low, so run easy").

---

## 4. Pulling plans in from other services

### 4.1 What is actually available to pull

| Service | Plan data over an API? | What the athlete can hand over instead |
|---|---|---|
| **TrainingPeaks** (delivers the McMillan plans) | **Yes** — OAuth2, the athlete's own account and token; plan, calendar and workout reads | CSV, PDF, .ics export; copy-paste of a week |
| **Google Calendar / generic calendars** | Yes (ICS is the common denominator; the app is a calendar, not a plan service) | .ics file |
| **Garmin** | **No.** The Personal API has **no scheduled-workout calendar** — `docs/garmin-api-capabilities.md` says it plainly: "completed workouts are not future schedules" | Garmin Connect's **workout schedule CSV** — which Stride already reads (`Format::GarminCsv`) |
| **RunStrong** | No public plan API | Calendar export, copy-paste of the week |
| **Runna** (Strava) | No public plan API | Calendar/PDF export, copy-paste |
| **McMillan's own site, Hal Higdon, YearRoundRunning** | No | A plan page, copied or exported as PDF/CSV |

So the honest answer to "make it easy to pull from various websites" is:
**file-first, connector-second, never scrape.** Three of these services have
no plan API at all, and reverse-engineering a private endpoint would put the
athlete's credentials into a local-first app that has never needed them —
and would break without notice.

### 4.2 The four ways in (the mock)

`exploration/plan-import-mock.html` mocks three screens: the ways in, the
paste reader, and the report of what was read.

1. **Paste a week.** The universal fallback, because every one of these sites
   renders a week as text. The reader is forgiving about *shape* (a week start
   asked once, "Tue 24 Feb" or "Tuesday" both fine) and strict about *meaning*
   (a session it cannot name keeps its own words).
2. **Import a file.** `.csv` · `.ics` · `.json` · `.txt`. **ICS is the closest
   thing to a common denominator** — Runna, RunStrong and TrainingPeaks all
   export a calendar — and it is a small, well-specified format: `DTSTART`
   gives the day, `SUMMARY` the session, `DESCRIPTION` the structure. The
   current readers cover csv/json/markdown/Garmin-csv; **ICS is the missing
   one** (`crates/stride-api/src/planfile.rs:44-49`).
3. **From a training service.** One connector worth writing: **TrainingPeaks**,
   because it is where paid plans (McMillan's) actually live and it has a
   documented API. Its credential handling must copy Garmin's rules exactly —
   the athlete's own token, in a private file, never in the WebView, never in
   a URL (`frontend/src/i18n/catalogue.ts:560` is the Garmin key's address;
   the pattern is established).
4. **From a link.** A public `.ics`/`.csv` URL the athlete names. This is the
   only network egress the plan feature would ever want, and `PLAN.md` defers
   "network egress routing" today — so it would be **explicit, one-shot,
   user-named and logged**, never a background crawler.

### 4.3 What has to change in Stride for this to be honest

| Gap | Why it matters | Fix |
|---|---|---|
| **A session cannot hold a structured workout** | "6 × 400 m", "Tempo 25 min", "12 → 16 km" are the *content* of most plans; Stride's `PlanSession` has a distance and a kind, so imported plans arrive **lossy** | Add `structure` (the athlete's own wording) and later `pace_low/high` — the store columns already exist (`crates/stride-store/src/schema.rs:97-104`) but nothing fills them |
| **No provenance beyond `prescribed`** | Every import lands as `prescribed`; a second import archives the first (one active block per source, `crates/stride-store/src/repos/plan.rs`), so an athlete with a TrainingPeaks plan *and* a Runna calendar cannot hold both | Add an **`origin`** (where it came from) beside `source` (who owns the intent): "TrainingPeaks export, 12 Feb 2026" |
| **The CSV reader guesses column names** | It has aliases (`planfile.rs:287-320`) but no way for the athlete to say "this column is the date" | A **column-mapping** screen with per-service presets; the athlete's mapping wins over the guess |
| **The vocabulary is closed** | Deliberately — but a third-party plan uses words Stride does not have (hill reps, fartlek, progression, walk-run) | Keep the closure; **map known words per service**; everything else stays *unclassified* and keeps its wording. The import reports the count: "112 sessions read · 9 with words Stride does not know" |
| **Distances only** | A plan with pace targets (McMillan's world) loses its targets | Needs an athlete-owned **pace model** first — not a guess. Until then, the pace band is preserved as text |

### 4.4 The strategic point

The built plan (§3) removes most of the reason to import. An athlete with a
race in eight weeks does not need to buy McMillan's plan and export it — they
can build one from their own history in Stride. **Import is for the athlete
who already pays for a plan elsewhere**, which is a much smaller, better
defined audience — and the two services worth supporting first are
**TrainingPeaks** (paid plans, real API) and **ICS** (everything else).

That also keeps the dependency honest: Stride stays local-first, offline-capable,
and account-free, and the connector is an *addition* the athlete opts into.

---

## 5. What it would touch (the frozen bits)

The contract in `docs/contract/` is **frozen** and does not change silently —
a change to any of these files is a plan change and must move in the same
pull request (`docs/contract/README.md:16`).

| File | What would have to change |
|---|---|
| `docs/contract/api-paths.md:31` | The plan route group gains a builder route, e.g. `POST /plan/build` (and possibly `GET /plan/templates`) |
| `docs/contract/api-paths.md:133` | `/plan/week` sessions may carry `phase`, `week`, `pace` — the DTO is frozen and shaped as it is |
| `docs/contract/schema.sql` | `plan_blocks` gains the goal/template columns; `plan_sessions` gains `structure` and, if used, the pace columns become meaningful; the `session_type` comment gains the new kinds |
| `docs/contract/voice.md` | The new source word (`built`) and the plan-builder copy must pass the ban list; the "one source" language becomes "four sources" |
| `PLAN.md` §0 and the plan-engine section (`PLAN.md:555-580`) | The new source, the templates, and the promise that the core owns the plan |

The **core** is where the work actually is, and the core already has the
shape of it: `stride-core/src/plan.rs` owns the baseline, the week, the
adherence and the comparison. A builder belongs there, next to
`baseline_from_history_at` (`plan.rs:588`), because the same rules apply —
deterministic, argument-injected, no clock.

---

## 6. A delivery sketch (if it is ever approved)

| Step | What | Gate |
|---|---|---|
| 0 | **Decide the source word** (`built`?) and the level ladder (1–5? 3–5?) — a product decision, not an engineering one | approval |
| 1 | Core: `PlanTemplate`, `PlanRequest`, `build_plan` + the template tables for one distance family (half marathon, 16 weeks, level 3) | core tests: same inputs → same block; the block's dates land where the template says; no session on a day the athlete did not pick |
| 2 | Core: widen `PlanSession` (structure, pace) and `PlanBlock` (goal, phase); render the label; add the fourth source | voice lint + `crates/stride-core/tests/plan.rs` |
| 3 | Store: the fourth source, the goal columns, the new session columns; keep "one active block per source" | `crates/stride-store/tests/plan.rs`, `schema_contract.rs` |
| 4 | API: `POST /plan/build` (preview returns the block, save writes it) | `crates/stride-api/tests/plan.rs`, `route_parity.rs` |
| 5 | Frontend: the builder in the Plan screen, the preview list, the new source banner | `frontend/tests/screens.test.tsx`, `voice.test.ts` |
| 6 | Only then: export (Garmin CSV / .ics), editing a session, strength tracks | per-feature decisions |

Everything above is *additive* to the existing plan engine — the baseline,
the import, the week, the counts and the comparison all keep working
unchanged, and a built plan is measured by exactly the same code as an
imported one.

---

## 7. Open questions

1. **Is a built plan a "prescribed" plan?** Stride's sources say where a plan
   came from. A built plan comes from the app, from a template the athlete
   picked. That is neither `baseline` (inferred from history) nor `typed`
   (written by hand) nor `prescribed` (imported from a file) — so it wants a
   fourth word. Which word?
2. **Levels: McMillan's 5, or Runna's 4, or Stride's own?** The important
   part is the *unit*: McMillan uses days/week + long-run minutes; Runna uses
   days/week + experience; Stride has a baseline that already knows the
   athlete's usual days/week. Suggestion: **3–5 runs a week**, with the
   athlete's own usual week named first and never marked as a recommendation.
3. **Distance-based or pace-based targets?** Distance-based fits Stride today
   (the core's label is "10 km · easy"; there is no pace zone model).
   Pace-based is what a coached plan really needs, and it needs a
   *pace* model the athlete owns — a decision, not a patch.
4. **Who owns the template tables?** They are content. In the repo as JSON
   (reviewable, diffable, versioned) or shipped by the app? In the repo,
   versioned with the app, and every built plan records the template
   version it came from.
5. **Does the race need a name?** "half marathon" is enough for the plan;
   "Mumbai Half, 12 Apr 2027" is nicer. A free-text label on the goal, with
   no race database, no network.
6. **How much of RunStrong's after-the-fact layer do we want?** Stride's
   answer today is the neutral 10 % distance comparison. That is probably
   enough. Scoring the session against the plan is a verdict, and verdicts
   are what this app does not do.

---

## 8. Should we?

**Yes to a first slice — the plan *builder*. No to the rest of the product.**

The reasoning:

**For.**
- Stride already has the plan engine, the store, the week view, the counts
  and the neutral distance comparison. A built plan is *measured by exactly
  the same code* as an imported one. The feature is mostly a **content
  problem** (template tables) and a **one-route problem** (`POST /plan/build`),
  not a new product surface.
- The athlete's own history is already the strongest input the generator
  could have: the baseline knows the usual week, and the app knows the long
  run, the easy pace and the volume from the recorded runs.
- It fills the app's biggest hole: right now a plan can only arrive from a
  file or from history. A runner with a race in eight weeks has nothing to
  put in the Plan screen.
- The voice problem is **solvable by construction**, because the generator
  produces a *record of an intent*, not advice. Every banned construction
  ("you should", "rest day", "missed", "recommended") is a property of
  coaching language, and a coaching language is not in the plan.

**Against (and therefore not now).**
- Auto-adjusting plans from performance is coaching, and coaching is a
  product Stride has said no to.
- Pace targets need a pace model of the athlete's own; there is none.
- Watch push, calendar export, B-races, holidays and "not feeling 100%"
  adjustments are each a rule set with editorial judgement baked in.
- The contract in `docs/contract/` is frozen; the first slice has to move
  with it, in the same pull request, and that is a real cost.

**The slice worth doing.** One distance family (half marathon), one template
version, levels as days-a-week (3/4/5), distance targets only, a three-step
preview, saved as a fourth source. Six weeks of engineering, most of it in
`stride-core/src/plan/build.rs` plus one route and one screen. If the voice
lint and the existing plan tests stay green and the preview reads as a fact
rather than an instruction, the feature is honest. If it starts talking like
a coach, it is the wrong feature for this app and should be dropped.

**The two decisions that gate it** (both product, not engineering): the name
of the fourth source, and whether the level unit is days-a-week. Everything
else follows.

---

## 9. Reference index — where the pieces live

| Piece | File | Anchor |
|---|---|---|
| Plan engine | `crates/stride-core/src/plan.rs` | whole file |
| Plan sources | `crates/stride-core/src/plan.rs` | `:52` |
| Session kinds | `crates/stride-core/src/plan.rs` | `:90` |
| Session + label | `crates/stride-core/src/plan.rs` | `:124`, `:291` |
| Block | `crates/stride-core/src/plan.rs` | `:154` |
| Baseline inference (the level source) | `crates/stride-core/src/plan.rs` | `:588` |
| Adherence / not-done rules | `crates/stride-core/src/plan.rs` | `:360`, `:383` |
| Distance comparison (10 %) | `crates/stride-core/src/plan.rs` | `:519` |
| HR zone model (no pace model exists) | `crates/stride-core/src/zones.rs` | `:37` |
| Plan file readers (csv / json / markdown / Garmin csv — **no ICS yet**) | `crates/stride-api/src/planfile.rs` | `:44-49`, `:161` |
| Import vocabulary (closed) | `crates/stride-api/src/planfile.rs` | `:123` |
| Garmin workout-schedule reader (export target later) | `crates/stride-api/src/planfile.rs` | `Format::GarminCsv` |
| Stored word → core kind | `crates/stride-api/src/plan.rs` | `:217` |
| Stored source → core source | `crates/stride-api/src/plan.rs` | `:231` |
| Plan routes | `crates/stride-api/src/routes/plan.rs` | `:51-330` |
| Route table | `crates/stride-api/src/router.rs` | `:390-430` |
| Plan tables | `crates/stride-store/src/schema.rs` | `:85-108` |
| One active block per source | `crates/stride-store/src/repos/plan.rs` | `upsert_plan_block` |
| Block / session models | `crates/stride-store/src/model.rs` | `:326-412` |
| Source constants | `crates/stride-store/src/model.rs` | `:31`, `:41` |
| Plan week screen | `frontend/src/pages/PlanWeekPage.tsx` | whole file |
| Import button | `frontend/src/components/PlanImport.tsx` | whole file |
| Today: the planned session as an option | `frontend/src/pages/TodayPage.tsx` | `:330` |
| Plan API paths (app side) | `frontend/src/lib/api.ts` | `:82-89`, `:260-267` |
| Plan strings (typed + linted) | `frontend/src/i18n/catalogue.ts` | `:389-405`, `:603-617` |
| Voice lint (ban list, typed strings) | `frontend/tests/voice.test.ts`, `frontend/scripts/voice-lint.mjs` | — |
| Frozen: plan route group | `docs/contract/api-paths.md` | `:31` |
| Garmin: no scheduled-workout calendar exists | `docs/garmin-api-capabilities.md` | "Plans: completed workouts are not future schedules" |
| Frozen: `/plan/week` DTO | `docs/contract/api-paths.md` | `:133` |
| Frozen: `/plan/import` DTO | `docs/contract/api-paths.md` | `:144` |
| Frozen: voice + ban list | `docs/contract/voice.md` | whole file |
| Frozen: schema | `docs/contract/schema.sql` | — |
| Plan of record: voice + plan engine | `PLAN.md` | §0; `:555-580` |
| Core plan tests | `crates/stride-core/tests/plan.rs` | — |
| API plan tests | `crates/stride-api/tests/plan.rs` | — |
| Store plan tests | `crates/stride-store/tests/plan.rs` | — |
| HTML mock: the plan builder (3 screens, clickable) | `exploration/plan-builder-mock.html` | — |
| HTML mock: pulling plans in (3 screens) | `exploration/plan-import-mock.html` | — |
