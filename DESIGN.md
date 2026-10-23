# Stride — Phase 2 design: plan intelligence, physiology, athlete fingerprint

> The design of record for the second pass. Phase 1 (the Strava-grade log) is
> live; this document defines what makes Stride *not* Strava.

---

## 1. The gap

Strava is a **social activity log**. Its core loop is: post a run, see friends,
compare segments. It has three properties that matter to a serious runner:

1. **No plan.** "Training Log" is a calendar of what you *did*. Strava has no
   concept of intent — there is no "what you were supposed to do today".
2. **No physiology.** Sleep, HRV, resting heart rate, body battery, training
   status — the state of the athlete on the day — are absent.
3. **No causality.** A pace is a number. Strava cannot tell you *why* it was
   that number, whether it is getting better, or what to do tomorrow.

So Strava answers **"What did I do?"** and stops.

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

Strava has only the middle box. The differentiator is the loop closing.

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

That sentence is the product. Nothing in Strava can produce it.

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
Strava is not.

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
