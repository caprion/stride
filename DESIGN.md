# Stride — Phase 2 design: plan intelligence, physiology, athlete fingerprint

> The design of record for the second pass. Phase 1 (the log itself) is live;
> this document defines what makes Stride *more than a log*.

---

## 1. The gap

A social running app is a **social activity log**. Its core loop is: post a run, see friends,
compare segments. It has three properties that matter to a serious runner:

1. **No plan.** "Training Log" is a calendar of what you *did*. A social app has no
   concept of intent — there is no "what you were supposed to do today".
2. **No physiology.** Sleep, HRV, resting heart rate, body battery, training
   status — the state of the athlete on the day — are absent.
3. **No causality.** A pace is a number. A social log cannot tell you *why* it was
   that number, whether it is getting better, or what to do tomorrow.

So a social app answers **"What did I do?"** and stops.

**Stride answers "Was that the right thing to do — and what should I do tomorrow?"**

That is the whole thesis. Everything below is in service of it.

---

## 2. The product loop

```
        ┌──────────────────────────────────────────────────────────┐
        │                                                          │
   PLAN ──▶ EXECUTE ──▶ UNDERSTAND ──▶ ADAPT ──▶ (next PLAN)      │
        │         │            │            │                     │
   what was    what        why it       what to do               │
   intended    happened    happened     tomorrow                 │
        └──────────────────────────────────────────────────────────┘
```

A social log has only the middle box. The differentiator is the loop closing.

### Pillar 1 — The plan exists as a first-class object

A plan is a hierarchy, not a document:

```
TrainingBlock   "HM Sub-2:00 Campaign"        Jan 5 – May 24, 2026
└── Phase       "Base"                        weeks 3–8
    └── Week    week 5                        target 37 km
        └── Session  Thu · Tempo 5km @ 5:50   distance + pace band + HR zone
```

**Sources of a plan (all three supported):**

| Source | How | Notes |
|---|---|---|
| **Import a document** | markdown / CSV / JSON | The athlete already writes plans. `~/running/seasons/*/plan.md` parses directly. |
| **Define in-app** | structured editor | For blocks created inside Stride. |
| **Garmin's calendar** | scheduled workouts pushed to the watch | The plan the watch actually shows you. |

> Importing `plan.md` is the fastest path to a working product, because this
> athlete already has a 20-week plan with a `## Weekly Volume Progression`
> table and day-by-day `#### Week N — Phase (dates) — XXkm` sections. The
> importer is a real piece of engineering, not a toy.

### Pillar 2 — Plan vs Actual is the flagship screen

Every planned session receives a verdict, and every verdict receives a cause.

| Verdict | Meaning |
|---|---|
| `hit` | distance met **and** pace band met |
| `missed_pace` | distance met, pace band missed (by how much) |
| `short` | distance short of target |
| `missed` | no activity logged |
| `substituted` | a different session type done instead (intervals when tempo was planned) |
| `bonus` | an unplanned run — extra work, not a failure |

And a cause, attached automatically:

> **Thu tempo** — planned 9 km @ 5:40–5:45, ran 9.0 km @ 6:02 (−17 s/km).
> Night before: sleep 5 h 40 m (baseline 7 h 32 m), HRV 34 ms (baseline
> 48 ms, −29 %), RHR 63 bpm (baseline 59 bpm).
> → **Fatigue-driven fade, not a fitness regression.** Reduce next quality
> session by one rep.

That sentence is the product. Nothing in a social log can produce it.

### Pillar 3 — The physiological picture

Two complementary pictures of the athlete.

**(a) The day — readiness.** A 0–100 composite for each calendar day, with
its drivers exposed rather than hidden:

| Input | Source | Role |
|---|---|---|
| Sleep duration + stages + score | Garmin sleep | Largest single driver |
| Sleep consistency (bedtime variance) | derived | Distinguishes "5 h bad night" from "6 h every night" |
| HRV + status vs 30-day baseline | Garmin HRV | Best single recovery signal |
| Resting HR vs baseline | Garmin RHR | Illness / overreaching flag |
| Stress (all-day + avg) | Garmin stress | Context for the day |
| Body Battery min/avg/max | Garmin | Recovery reserve |
| Acute : chronic load | Garmin training status | Whether the load is sustainable |
| Training Readiness | Garmin composite | Sanity-check our composite against Garmin's |

**(b) The run — fingerprint.** The mechanics of *how* the athlete runs. This
is the "run characteristics" picture, and it is what turns a stream of
activities into a picture of a person.

| Metric | Definition | Why it matters |
|---|---|---|
| **Cadence stability** | CV of lap cadence | High CV = wasted energy |
| **Cadence drift** | cadence change, first third → last third | Rising cadence late = late fatigue |
| **HR drift / aerobic decoupling** | % change in pace:HR ratio (Efficiency Factor) over the run | The best measure of aerobic durability |
| **Efficiency factor** | pace ÷ %HR, first third vs last third | Absolute economy proxy |
| **Pace variability** | CV of lap pace (after removing terrain) | Consistency |
| **Negative split** | % of runs finishing faster than they started | Aerobic durability |
| **Stride length vs cadence** | the speed equation, split by pace band | Are you speeding up by striding or by turnover? |
| **Time in HR zone** | distribution vs the planned zone | Discipline, and load shape |
| **Training effect** | Garmin aerobic + anaerobic TE | Load classification |
| **VO₂max / recovery time / GCT** | Garmin activity details | Recovery load, running economy |

Then **trend** them: is decoupling improving over the block? Is cadence
drifting up? Is long-run HR drift getting worse?

### Pillar 4 — Adapt

The output of the loop. For today:

- "Plan says: 16 km easy @ 6:45–7:15. **Do 12 km instead** — you are 3 days
  into a readiness dip (HRV −14 %, RHR +4 bpm) and you are 6 km short of the
  week's target. Add the 4 km back at the weekend."
- "Plan says: 6 × 1 km @ 5:30. **Drop to 5 × 1 km** — your cadence is 6 spm
  below your 8-week average on this session type, which precedes your usual
  fade."

This is the feature that makes the app worth opening *before* a run, which
A social log is not.

---

## 3. Confirmed Garmin surface

Every endpoint below was probed live from this VM. `403` means the endpoint
exists and requires a user session (the `garminconnect` account path); `404`
means it does not exist on that host.

### Health & physiology

| Endpoint | Probe | Data |
|---|---|---|
| `/wellness-service/wellness/dailySleepData?date=` | 403 | Sleep duration, stages, score |
| `/wellness-service/wellness/bodyBattery/reports/daily?date=` | 403 | Body battery min/avg/max |
| `/usersummary-service/usersummary/daily` | 403 | Daily summary (steps, resting HR, stress) |
| `/usersummary-service/stats/stress/daily/{start}/{end}` | 403 | Stress series |
| `/biometric-service/biometric` | 403 | Training status, readiness, VO₂max |
| `/biometric-service/stats` | 403 | HRV / sleep / body-battery stats |
| `/biometric-service/heartRateZones` | 403 | **The athlete's own HR zone model** |
| `/metrics-service/metrics/endurancescore` | 403 | Garmin endurance score (new) |
| `/metrics-service/metrics/runningtolerance/stats` | 403 | Garmin running tolerance (new) |
| `/sleep-service/stats/sleep/daily/{date}` | — | Sleep via the stats service |
| `/weight-service/...` | — | Weight trend |

### Plan & gear

| Endpoint | Probe | Data |
|---|---|---|
| `/workout-service/workouts?start=&limit=` | 403 | The workout library |
| `/gear-service/gear` | 405 | Shoes — **rotation mileage and retirement** |
| `/device-service/deviceregistration/devices` | 403 | Watch / phone models |

### Activity detail (already partially used)

`get_activity_details` additionally yields aerobic/anaerobic training effect,
VO₂max, recovery time, ground-contact time, vertical ratio, normalised power
and average power.

> **Note on the developer-key path.** The *Activity API* (OAuth1 consumer key)
> does **not** expose the health endpoints above — those belong to the
> *Health API* product with separate scopes. So the physiology picture requires
> the **account path** (email + password + MFA). The app must degrade honestly:
> with a developer key you get the plan-vs-actual engine and the fingerprint
> (from activity data alone) but not sleep/HRV. The UI must say so.

---

## 4. Data model

Added tables (all created by the backend; the schema is frozen before any
parallel work starts, so no two agents touch the same file).

```
plan_block      id, name, start_date, end_date, goal, status, source, notes
plan_phase      id, block_id, name, start_date, end_date,
                target_weekly_km_min, target_weekly_km_max, target_sessions,
                pace_easy_low, pace_easy_high, pace_tempo_low, pace_tempo_high,
                pace_interval_low, pace_interval_high, pace_long_low, pace_long_high,
                hr_zone_target, notes
plan_week       id, phase_id, week_number, start_date, target_km, key_workout, notes
plan_session    id, week_id, block_id, date, session_type, title, details,
                target_distance_m, pace_low_s_km, pace_high_s_km, hr_zone_target,
                status, source, notes
                session_type ∈ easy|long|tempo|interval|race|recovery|strength|rest

plan_result     id, plan_session_id, activity_id, verdict,
                distance_delta_m, pace_delta_s_km, pace_band_hit, distance_hit,
                adherence_score, cause, cause_confidence, note

daily_physiology date, sleep_s, sleep_score, deep_sleep_s, rem_s, light_s, awake_s,
                hrv_ms, hrv_status, resting_hr, stress_avg, stress_max,
                body_battery_min, body_battery_avg, body_battery_max,
                steps, training_readiness, acute_load, chronic_load, vo2max,
                endurance_score, running_tolerance, source

activity_facts  activity_id, aerobic_te, anaerobic_te, training_effect_label,
                vo2max, recovery_time_min, avg_gct_ms, avg_stride_ratio,
                vertical_ratio, avg_power, normalized_power, avg_grade_adjusted_pace,
                temperature_c, impact_estimate, gear_uuid, device_name

run_fingerprint activity_id, cadence_avg, cadence_cv, cadence_drift_pct,
                hr_avg, hr_max, hr_drift_pct, efficiency_first, efficiency_last,
                decoupling_pct, pace_cv, negative_split_pct, form_score,
                zone_distribution(json), stride_length_cm, notes

readiness       date, score, grade, drivers(json), recommendation, rationale

gear            id, uuid, name, brand, model, status, distance_m, activity_count,
                first_used, last_used, retire_at_m
```

---

## 5. API additions

All under `/api/v1`, all additive — nothing existing changes shape.

### Plan
```
GET    /plan/blocks                     list blocks with phase/week counts
POST   /plan/blocks                     create a block
GET    /plan/blocks/{id}                full block: phases → weeks → sessions
POST   /plan/blocks/{id}/phases         add a phase
POST   /plan/phases/{id}/weeks          add a week
POST   /plan/weeks/{id}/sessions        add a session
POST   /plan/import                     import a plan document (markdown|csv|json)
GET    /plan/current                    the active block + this week's plan
GET    /plan/week?date=                 the week containing a date
GET    /plan/results?from=&to=          plan-vs-actual rows
GET    /plan/adherence?from=&to=        weekly/phase/block adherence summary
GET    /plan/today                      today's plan + verdict + recommendation
```

### Physiology
```
GET    /physiology/daily?from=&to=      daily rows
GET    /physiology/latest               most recent day + drivers
GET    /physiology/sleep?from=&to=      sleep series
GET    /physiology/hrv?from=&to=        HRV series + baseline band
GET    /physiology/readiness?from=&to=  readiness series
GET    /physiology/sources              what is actually available (honest degradation)
```

### Fitness / fingerprint
```
GET    /fitness/fingerprint?from=&to=   per-activity fingerprint rows
GET    /fitness/activity/{id}           one activity's full fingerprint
GET    /fitness/trends?metric=          cadence/decoupling/EF trends
GET    /fitness/zones?from=&to=         time in HR zone
GET    /gear                            shoe rotation
```

---

## 6. Workstreams

Four agents, disjoint files, one frozen contract.

| # | Workstream | Files owned | Model |
|---|---|---|---|
| A | **Garmin health & workout ingest** | `backend/app/ingest/garmin_health.py`, `backend/app/ingest/garmin_workouts.py`, `backend/app/services/health_sync.py` | codex `gpt-6-luna` |
| B | **Plan model, importer, plan-vs-actual engine** | `backend/app/ingest/plan_importer.py`, `backend/app/services/plan_engine.py`, `backend/app/api/plan.py` | claude-code `claude-fable-5-1` |
| C | **Fingerprint + readiness engines** | `backend/app/services/fingerprint.py`, `backend/app/services/readiness.py`, `backend/app/api/physiology.py`, `backend/app/api/fitness.py` | claude-code `claude-fable-5-1` |
| D | **Frontend: Plan / Physiology / Fitness surfaces** | `frontend/src/pages/PlanPage.tsx`, `frontend/src/pages/PhysiologyPage.tsx`, `frontend/src/pages/FitnessPage.tsx` | codex `gpt-6-luna` |

Then integration, a "Today" dashboard that leads with the plan, and a final
visual + regression pass.

---

## 7. Honesty rules

The app must never invent physiology it does not have.

- If the athlete connected with a **developer key only**, the health endpoints
  are unreachable. The Physiology surface shows an explicit, useful explanation
  and offers the account path — it does **not** show empty charts that look
  like a bug.
- Plan-vs-actual must distinguish *"no plan defined"* from *"planned and
  missed"*. A blank plan is not 100 % adherence; it is no plan.
- A verdict's `cause` carries a confidence. "Sleep was 5 h 40 m and HRV −29 %"
  is high confidence; "pace was off, possibly fatigue" is low.
- Demo data is flagged everywhere, and the plan engine does not attempt to
  infer a plan for demo activities.

---

## 8. Success criteria

1. Importing `~/running/seasons/2026-spring-hm-sub2/plan.md` produces a block
   with 20 weeks and ~60 day-level sessions, with pace bands parsed from the
   phase pace table.
2. With real Garmin data, the Plan screen shows per-session verdicts with
   causes, and a weekly adherence curve of planned vs actual km.
3. The Physiology screen shows a readiness score with visible drivers and
   30-day baseline bands.
4. The Fingerprint screen shows cadence stability, HR drift / decoupling and
   stride-length-vs-cadence trends.
5. `GET /plan/today` returns one actionable sentence for today.
6. Every new endpoint has a test; the existing 19 still pass; the frontend
   typechecks; all routes render without console errors.

---

## 9. What shipped

All four workstreams landed, with the schema and API contract frozen before any
parallel work began so no two agents touched the same file.

| Workstream | Result |
|---|---|
| **A** — Garmin health & workout ingest | `garmin_health.py`, `garmin_workouts.py`, `health_sync.py` (635 lines). Sleep, HRV, body battery, stress, resting HR, training status, HR-zone model, endurance score, running tolerance, gear, scheduled workouts, workout→session profiles. Every method degrades to `{}` without an account session. |
| **B** — Plan model, importer, engine | The real 20-week plan imports as **8 phases, 20 weeks, 83 sessions**, pace bands on 67. Verdicts: `hit` · `missed_pace` · `short` · `missed` · `substituted` · `bonus`, scored 50 % distance / 40 % pace / 10 % type, with causes drawn from sleep, HRV, resting HR and cadence stability. |
| **C** — Fingerprint & readiness | Cadence CV and drift, HR drift, aerobic decoupling, Minetti efficiency, pace CV, stride length, form score. Readiness is a 0–100 composite with named, renormalised drivers. |
| **D** — New surfaces | `/plan`, `/physiology`, `/fitness`; the dashboard now leads with **Today** (plan → readiness → one recommendation). |

### Integration decisions taken by the orchestrator

- **The plan API was rewritten** to own the wire format. The engine produces
  plain dicts; the API shapes them into the contract the frontend is written
  against. Neither layer has to know about the other.
- **Plans are anchored to the activity history.** A plan written for a past
  season never overlaps the runs it governs, and every session then reads as
  "missed". Importing shifts the block so its final week ends with the most
  recent activity.
- **`has_plan` and `has_session` are separate.** A plan that schedules nothing
  for today is a rest day, not a missing plan.
- **A clean verdict carries no cause.** "Short sleep" attached to a session that
  was hit exactly is noise.
- **The efficiency factor was corrected** to Minetti's `speed / HR`. The
  pace:HR ratio is its reciprocal, and using it would have made a *slower* run
  look more efficient — inverting every downstream reading.

### Verification

- **74 backend tests pass** (19 from Phase 1, 55 new).
- Frontend typechecks with zero errors; production build succeeds.
- All nine routes render in headless Chromium with no console errors, page
  errors or failed requests.
- All 35 API endpoints return 200.

### Bugs the new test suite caught

1. `Long 14km easy` classified as `easy` — the easy check ran before the
   distance check.
2. The plan importer raised `IndexError` / `TypeError` on a malformed table
   instead of warning and continuing.
3. Garmin's aggregate sleep-stage fields were ignored when no per-stage array
   was present.
4. A workout step's *distance* target was read as a *pace* target.
5. The workout client had no non-probing `sources()` status method.


---

## 10. The Personal API — the path that actually works

The Phase-2 design assumed a Garmin developer key would be enough. It is not.
The **Garmin Connect Personal API** is, and the key was in Azure Key Vault the
whole time (`claw-agent` → `garmin-api`).

### What it is

A first-party API for your own account. Generate a key at
`connect.garmin.com/app/settings/personalApiToken`; no developer-program
registration, no consumer secret, no OAuth browser dance. It is a single
credential that exchanges for a 24-hour bearer token.

The key is 133 characters in four dot-separated segments, prefixed `GCDC_`:
a consumer key, a UUID, a secret and a checksum. It is *not* an OAuth1 consumer
key — every pairing tried against `/oauth-service/oauth/request_token` returns
`401`. The giveaway is the token endpoint, which takes the whole string:

```
POST https://personal.connectapi.garmin.com/personalapitoken-service/personal-api/personal-api-key/token
Authorization: ApiKey GCDC_…
```

It returns a JWT whose scopes are the whole point:

| Scope | Unlocks |
|---|---|
| `CONNECT_READ` | activities, workouts, calendar |
| `PERSONAL_API_FITNESS_READ` | activity summaries and measurements |
| `PERSONAL_API_WELLNESS_READ` | **sleep, resting HR, stress, body battery, HRV** |
| `PERSONAL_API_USER_READ` | profile, settings, HR zones |
| `PERSONAL_API_NUTRITION_READ` | calories and nutrition |

The wellness scope is what makes the physiology and plan-cause engines possible
without a password.

### Endpoints verified live

| Endpoint | Query | Returns |
|---|---|---|
| `/activitylist-service/activities/search/activities` | `start`, `limit`, `startDate`, `endDate` | activity summaries |
| `/activity-service/activity/{id}` | — | `summaryDTO` (cadence, stride, training effect, HR) + `splitSummaries` |
| `/activity-service/activity/{id}/workouts` | — | associated workouts |
| `/download-service/files/activity/{id}` | — | the original **FIT** as a zip |
| `/sleep-service/sleep/dailySleep` | `date` | duration + deep/REM/light/awake split |
| `/wellness-service/wellness/dailyHeartRate` | `date` | resting HR and the day's curve |
| `/wellness-service/wellness/bodyBattery/reports/daily` | `startDate`, `endDate` | body battery series |
| `/usersummary-service/usersummary/daily` | `calendarDate` | steps, RHR, stress, body battery, intensity minutes — one call per day |
| `/usersummary-service/stats/stress/daily/{from}/{to}` | — | stress series |
| `/usersummary-service/stats/steps/daily/{from}/{to}` | — | steps series |
| `/biometric-service/heartRateZones/v2` | — | the athlete's HR zone model |
| `/hrv-service/hrv/{date}`, `/hrv-service/hrv/daily/{from}/{to}` | — | HRV |

Two quirks worth recording: the parameter is `date`, not `calendarDate`, and the
stats endpoints return `400` for a window that reaches outside the retained
history — so the window is clamped.

### What the FIT taught us

Downloading the original file and decoding it against the API's own summary
exposed three Garmin behaviours that would have silently corrupted the data:

1. **Positions are semicircles.** `position_lat` of 156,225,859 is 13.09° —
   the file stores `2**31` semicircles per 180°.
2. **Running cadence is stored halved.** The watch displays 154.8 spm; the file
   says 77.4. Some devices record *stride cycles* (two footfalls), so the file
   value is doubled to get steps per minute. A real running cadence is never
   below 100, so the correction cannot fire on genuine data.
3. **Lap speed is not always present.** When the device omits it, pace follows
   from distance and duration.

The API's `summaryDTO` wins for distance, pace, heart rate and cadence; the FIT
wins for the GPS track, per-second streams and kilometre splits.

### The end state

The app now connects itself from the Key Vault at startup, syncs real runs with
full fidelity, and builds the physiological picture from the same credentials —
no password anywhere in the system.


---

## 11. Segmenting a "run" — and the case against average heart rate

### The problem

A recorded activity is not a run. It contains a walking warm-up, a stand at a
crossing, a walk break, and a walking cool-down. Two things follow:

1. **Average heart rate is a misleading headline.** Averaging heart rate over a
   session that includes walking and standing produces a number that describes
   nobody — it is pulled toward the warm-up and the rest stops rather than
   describing the running.
2. **Cadence, pace and the splits are contaminated the same way.** A
   kilometre lap that includes a minute of walking reads as 9:14/km, when the
   athlete was running at 7:00/km for the part of it they were running.

### The segmentation

Every FIT file is a per-second (or per-record) stream, and each record can be
classified from its own speed and cadence:

| Class | Rule |
|---|---|
| `stand` | speed < 0.30 m/s, or no cadence at all |
| `walk` | cadence < 60 (stored) *and* speed < 2.2 m/s, or speed < 1.55 m/s |
| `run` | everything else |

The cadence threshold is deliberately conservative. Running cadence is stored
**halved** on many devices (a displayed 160 spm is stored as 80), so the
threshold is applied to the stored value. A run at 2.3 m/s with a stored
cadence of 55 is a genuine short-strided runner, not a walker, and both
conditions must hold before a sample is called walking.

### Garmin's own splits are not the answer

Garmin's activity API returns auto-detected splits with a type
(`RWD_WALK`, `RWD_RUN`, `RWD_STAND`). For one real activity they said:

| Split | Type | Distance | Elapsed |
|---|---|---|---|
| 1 | `RWD_WALK` | 95.7 m | 90.9 s |
| 2 | `RWD_RUN` | 9331.8 m | 4242 s |
| 3 | `RWD_STAND` | 26.8 m | 76 s |

But the record stream disagrees with them. The same activity, read from the
records, is:

```
run 0-2116 s · stand 2116-2212 s · run 2212-4302 s · walk 4302-4337 s · walk 4337-4343 s
```

The record stream finds a **72-second stand and a 35-second walk break in the
middle of the run** that Garmin's splits lumped into a single `RWD_RUN`, and it
finds the cool-down to be a walk where Garmin said stand. The records are the
truth; Garmin's split types are a coarse convenience.

### What is recomputed over the running parts only

- **Streams** — heart rate, cadence, pace and distance contain only running
  samples.
- **Laps** — each kilometre lap is rebuilt from the running samples inside it.
  An interval between two running samples counts as running time; a gap that
  spans a rest stop does not. Lap 5 of the reference activity went from
  `9:14` to `7:42` for 897 m of running.
- **The fingerprint** — cadence variability, drift, decoupling and the form
  score all describe the running only.

### Drift leads

Average heart rate is demoted to a hint. The headline numbers are:

| Metric | Meaning |
|---|---|
| `hr_drift_bpm` | heart-rate rise between the first and second half of the running |
| `pace_drift_pct` | how much the pace faded (positive = slower) |
| `decoupling_pct` | aerobic decoupling — the efficiency lost across the run |
| `cadence_cv` | cadence stability |
| `form_score` | a composite of the three above |

The UI states the exclusion explicitly — *"Running only · 9.30 km of 9.45 km ·
1:03 walking, 2:01 standing excluded"* — so the athlete can see what was left
out and agree with the call.

### Fewer API calls

If the FIT file is already downloaded and parsed, the record stream cannot have
changed, so the sync skips the download **and** the activity-summary call, and
reuses what is stored. A re-sync of 103 activities went from **133 API calls
and 127 seconds** to **1 call and 4 seconds**.


---

## 12. Comparing runs — and making the extraction general

### The comparison feature

Most apps let you see two activities side by side, but they compare them as
*activities*. The question an athlete actually asks is different: **was that
run really better, or was it just shorter and flatter?**

`/compare` answers it by putting two or more runs on the same axis. The
backend resamples every run onto a **shared distance axis** — 120 points from
the start of the run to the end — so a 5 km and a 21 km run can be laid over
each other and their heart-rate, cadence, pace and elevation curves read
directly against one another.

The matrix underneath compares 15 parameters, and the "best" cell in each row
is marked using the parameter's own direction (`lower`, `higher`, `neutral`):

| | |
|---|---|
| Distance · moving time · pace | the shape of the run |
| Elevation gain | the terrain |
| Average heart rate · HR drift · pace drift | the physiology |
| Aerobic decoupling · efficiency factor | the aerobic durability |
| Average cadence · cadence stability | the mechanics |
| Form score | the composite |
| Running / walking / standing time | what was excluded, and why |

The summary underneath reads the numbers back in plain language — derived from
the data, never hard-coded. Two runs that share a name ("Bengaluru Running")
are told apart by their date, and a percentage is only quoted when the
baseline is large enough for the ratio to mean something.

### Making the extraction general

The segmentation was developed against one real run, so the general case is
protected explicitly:

| Case | Behaviour |
|---|---|
| A walk-only activity | no running samples; the streams are empty, not zero-filled |
| No GPS (treadmill) | no track, everything else intact |
| No cadence sensor | the cadence stream is absent; heart rate is unaffected |
| No speed field | pace is derived from the distance stream, or absent |
| A very short effort | parses without error; the totals are sane |
| A non-running sport | cadence is **not** halved — a cyclist's 90 rpm is 90 rpm |
| An empty record stream | empty segments, empty running set, no exception |

The regression guard is `test_every_activity_in_the_live_history_extracts_without_error`,
which puts **every activity in the athlete's real history** through the
segmenter and asserts that the run / walk / stand split accounts for the whole
activity.

### Tests

119 tests pass. A shared `tests/conftest.py` gives every test a fresh database
and blocks network access by default; the handful of tests that genuinely need
the network opt out with `@pytest.mark.network`.
