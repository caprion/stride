# Stride v2 — the frozen route table

FROZEN. Part of the contract freeze (Slice 0). Changing a route is a plan
change and must travel with a PLAN.md edit in the same pull request.

The route table is inherited from the old core where its behaviour is right,
then shaped for v2: every route below exists on **both** hosts, and the phone
reaches them in-process through the FFI — the desktop reaches the same router
over loopback HTTP. Neither can disagree about a number.

Source: `PLAN.md` §6.2. The old implementation's routes live in
`crates/stride-api/src/routes.rs`; the historical OpenAPI surface is captured
in `fixtures/stride-api/openapi_paths.json` (the pre-cut route set, for
reference only).

## Data

| Route | Notes |
|---|---|
| `/activities` | the feed, windowed by date |
| `/activities/{id}` | one run |
| `/activities/{id}/fit` | the run's **original FIT file** — the source of truth, byte for byte as Garmin served it | the athlete's own file, downloaded to their own machine; the app never uploads it anywhere |
| `/activities/{id}/film` | the run's **film** — the run's own series, playable: the samples with their pace, their heart rate and **the zone each one sits in**, the ground, the splits, and the zone model the zones were measured against (see "The run film" below) |
| `/physiology/{daily,sleep,hrv,vitals,latest,sources}` | Body inputs: the vitals and their directions (no score) |
| `/fitness/{fingerprint,trends,zones}` | `fingerprint` answers **one run's fingerprint** — the observations of the run's mechanics, and the shares of the moving time in the athlete's own zones — and the **run detail shows them** (see "The fingerprint and the run's mechanics" below). `trends` is the measured trend of pace and heart rate. `zones` answers the per-run time in zone from the athlete's **stored zone model**, and names the model with the day it was read (see "The zones and the zone model" below) |
| `/stats/{summary,weekly,pace-trends,elevation,personal-records}` | Progress |

## Plan & insight

| Route | Notes |
|---|---|
| `/plan/{current,today,week,blocks,sources,adherence,results,import}` | the plan engine |
| `/insights/{today,progress,load,chips,rolling,streaks,pace,challenges}` | Today and the feed chips; `progress` is the Progress screen's answer, `load` is the training load over the same window (see "The training load" below), `rolling` is the rolling view (see "The rolling view" below), `streaks` is the streak of days and weeks (see "The streaks" below), `pace` is the pace page's answer (see "The pace page" below) and `challenges` is the athlete's own challenges, week by week (see "The challenges" below) |

| `/insights/{today,progress,load,chips,rolling,streaks,year,challenges}` | Today and the feed chips; `progress` is the Progress screen's answer, `load` is the training load over the same window (see "The training load" below), `rolling` is the rolling view (see "The rolling view" below), `streaks` is the streak of days and weeks (see "The streaks" below) and `year` is the year: the months, the calendar of the days and what changed (see "The year" below) |

| `/insights/{today,progress,load,heatmap,chips,pace,challenges}` | Today and the feed chips; `progress` is the Progress screen's answer and `load` is the training load over the same window; `heatmap` is the grid over the stored positions, computed on the device (see "The heatmap" below); `pace` is the pace page's answer (see "The pace page" below) |
| `/compare` | the metric diff |
| `/compare/candidates` | the runs you can compare against (the Compare screen's "choose a different run" action) |

## App

| Route | Notes |
|---|---|
| `/health` · `/health/daily` | liveness and the sync window |
| `/sync/{status,run,runs,history,settings}` | incremental sync |
| `/auth/{status,garmin/personal-key,garmin/personal-key/test,garmin/fit-import,garmin/logout}` | the one credential |
| `/calendar` | the local month calendar (activities per day) — **not** Garmin's scheduled-workout calendar |
| `/settings` | the weekly goal, the pause and the storage — the inputs that change what the app says |
| `/settings/export` | write a backup of the database |
| `/settings/delete` | delete everything, with the typed confirmation the request carries |
| `/settings/import` | read an export back into the database (the rig's own) |
| `/activities/{id}/note` | the athlete's note on a run — the only free text the product stores |
| `/activities/{id}/update` | the run's **update**: the model's own reading of the run, attributed to the writer and the reviewer, with the checks it passed and the hash of the payload it was written from (see "The run update" below) |
| `/activities/{id}/update/payload` | the **payload** of a run: the facts a model may read, and nothing else (see "The run update" below) |
| `/activities/{id}/update/request` | the **prompt** the rig sends to a model — the writer, the reviewer or the repair — with the payload it carries |
| `/update/settings` | the update's own settings: which model writes, which model reviews, and whether anything may leave the machine at all |

The last five rows arrived with the settings slice (PLAN.md §8, slice 8), and they are
the first change to the frozen table since the freeze. They travel with the plan edit
in the same pull request, as the freeze requires.

The four rows after them arrived with the run update (the plan's amendment of
2026-11-19). They travel with the plan edit in the same pull request too.

## Cut from the old table

`/network/*` · `/tiles/*` · `/gear` · `/auth/garmin/{login,mfa,developer-key,callback}` ·
`/auth/{pair-code,pair,devices}` · `/stats/monthly` · `/stats/hr-zones`

Nothing may reintroduce these. A CI grep asserts that `pair-code`, `pairing`
and `qr` never appear in the route table.

## Discrepancies with `docs/plan.html` §6.2 (recorded, not silently chosen)

1. **`/compare/candidates`** appears in plan.html §6.2 in *both* the "Plan &
   insight" list and the "Cut from the old table" list. This file keeps it:
   the Compare screen's "choose a different run" action needs it, and the
   in-list is the more recent intent.
2. **`/fitness/fingerprint`** was dropped from the `{…}` brace list by the
   route-table fix, but the same commit's note says it "stays (the data is
   already synced) but has no screen". The route stayed, and it stayed
   endpoint-only — until the fingerprint became a set of **observations** on
   the run detail. The route now answers **one run's** fingerprint, named by
   the query, and the run detail shows it: the "endpoint-only" note is
   superseded by the fingerprint's own section below.
3. **`/physiology/readiness`** is renamed **`/physiology/vitals`** (resolved by
   the plan after Slice 0 flagged it): v2 has no readiness score, so the route
   returns the vitals and their directions. The table above already uses the
   new name.

## The shape of an answer

**The answer is the body, and the status is the status.**

- The router answers with the response **body** — the DTO, and nothing around
  it. Not an envelope, not a wrapper, not a `{"json": "..."}` inside a JSON.
- **A failure is a failure, not a body.** The transport reports it as a
  failure: the HTTP layer puts the status in the status line, and the phone's
  bridge rejects the call. A body that says `{"error": …}` with a 200 is
  never the shape.
- The two hosts therefore hand the app **the same answer**: the desktop's HTTP
  layer puts the body on the wire, and the phone's plugin hands the body
  across the bridge. A host that answered with an envelope would make the
  app unwrap it — and an unwrap is where a second implementation grows.
- **The same answer, not necessarily the same bytes.** The phone's JSON is
  parsed and re-serialised by the platform's parser on the way through the
  bridge, so a number's *spelling* can change in transit (`162.0` may arrive
  as `162`, and a `/` may arrive escaped). The **fields, their names, their
  nullability and their values** are the contract; the spelling of a number
  is not — and that is exactly why the app never parses a number to show it.
  The words on screen are the core's rendered text, which is why the two hosts
  cannot disagree about what a value *says*, whatever the transport does to
  its digits.

## The path the app asks for

- The app asks for `/api/v1/...` — the same path the desktop's HTTP layer
  passes, query string and all.
- **The router matches on the path without the query string.** `/api/v1/activities?limit=10`
  is the route `/api/v1/activities`, with `limit=10` as its parameters. A
  decision made on the raw path — or on a path with a different prefix — is
  a bug, and the phone's bridge exists precisely because the old one made it.

## The shapes of the answers

The route table names the routes; the **shapes** name the fields. They are
written down here because a shape that lives in one file is a shape that
drifts: the app and the router must agree on the fields, and the agreement
needs a source on both sides.

Two rules hold for every shape below.

- **A domain value crosses as a rendered value**: `{"text": "8.4 km", "raw":
  8400}` — the core's words and the number a chart draws (core-contract §8).
  There is no `format:` hint: a hint tells the app how to render, which is
  the hole §8 closes. A value that is not there is `null`, never a `text` of
  `"0"`.
- **A sentence crosses as a sentence**: the direction word, the arithmetic,
  the travel, the Rest line, the pattern sentences and the statements are
  the core's, already written. There is no template in a body.

| route | the fields |
|---|---|
| `/activities` | `activities: ActivitySummary[]`, `next_cursor: string \| null`, `generated_at: string` |
| `/activities/{id}` | `ActivitySummary` and `splits: {lap_index, distance, duration, pace, heart_rate}[]`, `elevation: {distance, elevation}[]`, `samples: {offset_s, pace, heart_rate}[]`, `plan_line: {text} \| null`, `note: string \| null` |
| `/activities/{id}/film` | `activity_id`, `local_date`, `basis: string \| null`, `model: ZoneModel \| null`, `zones: {label, range, from_bpm, to_bpm}[]`, `outside_label: string \| null`, `samples: {elapsed, distance, pace, heart_rate, zone}[]`, `ground: {offset_s, elevation}[]`, `splits: {lap_index, offset_s, distance, duration, pace, heart_rate}[]`, `unavailable: string \| null`, `zones_unavailable: string \| null`, `generated_at` |
| `/physiology/vitals` | `date`, `generated_at`, `rest: {label, line}`, `vitals: {key, label, value_text, direction_text, deviation_text, travel_text, status_text, band}[]`, `history: {sleep, hrv, resting_hr}` (each point: `date`, nullable `value`, nullable core-rendered `text` with units), `patterns: {id, text, sample_size}[]` |
| `/plan/week` | `block_name`, `source`, `week_start`, `week_end`, `today`, `target`, `sessions: {date, session_type, title, sentence, target, actual, status, source}[]`, `adherence: {done, planned} \| null`, `counts: {done, planned, not_done}` |
| `/settings` | `goal: {value: {text, raw} \| null, km: number \| null}`, `pause: {start_date, end_date, reason} \| null`, `storage: {size, activity_count, wellness_day_count, plan_session_count, note_count}`, `generated_at` |
| `/settings/export` | `exported: boolean`, `size: {text, raw} \| null`, `generated_at` |
| `/settings/delete` | `deleted: boolean`, `generated_at` |
| `/settings/import` | `activities`, `wellness`, `plan_blocks`, `plan_sessions`, `notes`, `rows`, `generated_at` |
| `/activities/{id}/note` | `note: string \| null`, `generated_at` |
| `/insights/today` | `generated_at`, `date`, `planned`, `options`, `last_run`, `distance_comparison: string \| null`, `week: {distance, sessions, days: {date, sessions}[], previous: {from, to, distance, sessions}, difference: RenderedValue \| null}` |

| `/activities/{id}/update` | `activity_id`, `update: RunUpdate \| null` (each: `writer`, `reviewer`, `payload_hash`, `draft`, `checks: {check, findings: string[]}[]`, `verdict`, `created_at`), `generated_at` |
| `/activities/{id}/update/payload` | `activity_id`, `payload_hash: string \| null`, `payload: UpdatePayload \| null` (each: `model`, `read_on`, `facts: {label, value}[]`, `splits: {lap_index, distance, pace, heart_rate}[]`, `zones: {label, time, share}[]`, `observations: {label, value, direction, signed, quantity}[]`), `text`, `unavailable: string \| null`, `generated_at` |
| `/activities/{id}/update/request` | `role`, `prompt`, `payload_hash`, `payload`, `findings: string[]`, `generated_at` |
| `/update/settings` | `enabled`, `allow_remote`, `writer_model`, `writer_family`, `writer_base_url`, `reviewer_model`, `reviewer_family`, `reviewer_base_url`, `key_env`, `updated_at`, `generated_at` |
| `/insights/today` | `generated_at`, `date`, `planned`, `options`, `last_run`, `distance_comparison: string \| null`, `week: {distance, sessions, days: {date, sessions}[]}` |
| `/insights/progress` | `generated_at`, `date` (the core's local calendar date), `period: {from, to, distance, duration, sessions, recorded_weeks}`, `headline`, `headline_note`, `weekly_volume`, `typical_week`, `pace_at_hr`, `bests`, `consistency`, `sessions_per_week`, `adherence`, `eddington: {number, days_needed_for_next, longest_day} \| null` |
| `/insights/load` | `generated_at`, `date`, `window: {from, from_label, to, to_label, weeks, warmup_days}`, `load: LoadSeries \| null`, `unavailable: string \| null`, `scenario: LoadScenario \| null` |
| `/insights/rolling` | `generated_at`, `date`, `window: {from, from_label, to, to_label, days}`, `comparison: {from, from_label, to, to_label, days}`, `distance: {current, previous, difference}`, `duration: {current, previous, difference}`, `sessions: {current, previous, difference}`, `zones: {model, shares, distribution, unavailable}`, `distances: {key, label, sessions, distance, elevation, duration}[]`, `profile: {key, label, value, total, previous, movement}[]`, `weekday: {weekday, sessions, distance, duration, share}[]`, `daytime: {key, label, from_hour, to_hour, sessions, distance, duration}[]`, `months: {key, label, sessions, distance, duration, elevation}[]`, `years: {key, label, sessions, distance, duration, elevation}[]` |
| `/insights/streaks` | `generated_at`, `date`, `days: {current, longest}`, `weeks: {current, longest}` |
| `/insights/pace` | `generated_at`, `date`, `window: {from, from_label, to, to_label, weeks}`, `weeks: {week_start, distance, sessions, longest, pace, heart_rate}[]`, `bands: {width_s_per_km, total, distance, rows: {from, to, label, time, distance, share}[], rest} \| null`, `unavailable: string \| null` |

| `/insights/year` | `generated_at`, `date`, `window: {from, from_label, to, to_label, days}`, `comparison: {from, from_label, to, to_label, days}`, `months: {key, label, sessions, distance, duration, elevation, longest, form, weeks: {start, start_label, distance, sessions}[], mix: {zone, share}[], high}[]`, `days: {date, sessions, distance, high}[]`, `changed: string[]` |

| `/insights/streaks` | `generated_at`, `date`, `days: {current, longest}`, `weeks: {current, longest}`, `months: {current, longest}` |
| `/insights/challenges` | `generated_at`, `date`, `window: {from, from_label, to, to_label, weeks}`, `challenges: {key, kind, label, target, streak, weeks: {week_start, label, met, measured}[]}[]`, `unavailable: string \| null` |

| `/insights/heatmap` | `generated_at`, `date`, `runs`, `runs_outside`, `more_runs`, `cell_size: RenderedValue \| null`, `cells: {x, y, south, west, north, east, runs}[]`, `extent: {south, west, north, east} \| null`, `unavailable: string \| null` |
| `/auth/status` | `connected`, `method`, `key_source`, `key_last4`, `last_sync_at`, `generated_at` |
| `/auth/garmin/fit-import` | `imported`, `duplicate`, `activity_id`, `date`, `distance`, `generated_at` |
| `/plan/import` | `imported`, `block_id`, `sessions`, `generated_at` |
| `/compare` | `basis_text`, `metrics: {key, label_key, a, b, delta}[]`, `a: CompareRun`, `b: CompareRun`, `series: {a: {offset_s, pace, heart_rate}[], b: …}` |
| `/compare/candidates` | a list of `{id, label, date, distance}` |
| `/sync/status` | `last_sync_at: string \| null`, `last_error: string \| null`, `activity_count: number`, `cursor`, `pulled`, `skipped`, `deleted`, `more_pending: boolean`, `history: {activities, wellness} \| null` |
| `/sync/run` | `activities`, `wellness_days`, `skipped`, `deleted`, `complete: boolean`, `more_pending: boolean`, `failure: {code, detail, status, retry_after_seconds} \| null`, `finished_at`, `generated_at` |
| `/health/daily` | `date`, `generated_at`, `stale: boolean` |
| `/fitness/zones` | `basis: string \| null`, `model: ZoneModel \| null`, `zones: ZoneBand[]`, `outside: RenderedValue \| null`, `stopped: RenderedValue \| null`, `unavailable: string \| null` |
| `/fitness/fingerprint` | `activity_id`, `local_date`, `basis: string \| null`, `laps`, `observations: FingerprintObservation[]` (each: `key`, `label`, `value: RenderedValue \| null`, `reason: string \| null`), `zones: FingerprintZones` (`model: ZoneModel \| null`, `shares: ZoneShare[]` (each: `label`, `share: RenderedValue`, `time: RenderedValue \| null`), `outside: RenderedValue \| null`, `unavailable: string \| null`, `note: string \| null`), `unavailable: string \| null`, `generated_at` |

The app's own declarations are in `frontend/src/lib/types.ts`, and
`crates/stride-api/tests/frontend_shapes.rs` holds the router to them: it
reads the app's file and compares the **shapes**, field by field, so a
renamed field, a changed nullability or a `format:` hint fails the build.
**The written contract and the app's declarations are checked against each
other**: a contract that is not checked is a document, not a gate.

### The shapes the app does not read

The routes the app does not type — `/physiology/{daily,hrv,latest,sources}`,
`/stats/*`, `/plan/{current,today,blocks,sources,adherence,results,import}`,
`/insights/chips`, `/sync/{runs,history,settings}` and `/auth/*` —
are the router's own, and they are written down in
`crates/stride-api/src/dto.rs`. They follow the same two rules: a domain
value is a rendered value, and a sentence is a sentence.

Each non-null sync history resource has `target_from`, nullable `checked_from`,
nullable `checked_to` and nullable `next_date`. These describe one contiguous
checked range and the next older date, not guaranteed provider availability.
`complete` is true only when the requested history is exhausted without failure.
A normal bounded pause has `more_pending: true` and `failure: null`; clients must
surface a non-null partial failure even when HTTP succeeded and records landed.
The API rejects a concurrent owner with `sync_busy` (409). A provider rate limit
or an unexpired persisted retry delay returns `rate_limited` (429), not a refused
key. The persisted delay prevents another provider request after process restart.
See core-contract section 7 for recent refresh, historical continuation and retry
ownership.

Vitals accept an optional ISO `date` and `days` between 1 and 366 (default 28).
History contains one entry per calendar day ending on that date, including null
entries for missing observations. Changing the chart window does not change the
baseline's own calculation window. Weekly markers count recorded sessions, not
proof that a day was completely checked or that an empty day was a rest day.
Each marker also carries its core-rendered recorded `distance`.

The feed accepts inclusive ISO `since` and `until` dates, with the same stable
date/id cursor within the selected window. Reversed bounds are refused. Date
filters search stored history, not just already loaded pages; client-side text
search remains scoped to loaded results. The feed and Today include local FIT
route previews, uniformly bounded to 64 points with endpoints retained. They
do not request street tiles or decode every historical route for Today.

Progress accepts `weeks` from 1 to 104 (default 12). Its `period` totals include
the current partial week and the preceding calendar weeks in the selected
window. `recorded_weeks` counts weeks with saved sessions, not source coverage.
Measurements remain core-rendered; no frontend unit formatter is introduced.
The calendar accepts `year` and `month` and is typed by the frontend for monthly
history navigation. It describes saved activities, not Garmin's future plan.

**The Eddington number travels in the same answer.** The number is the
largest *E* such that at least *E* days in the selected window covered at
least *E* **kilometres** — consistency over peaks. It is counted in
kilometres, the contract's unit; the core's miles variant is not used by
this route.

* `eddington` is **`null`** when the selected window holds no distance. A
  window with no distance has no number: that is "no value", never a zero.
* When it is there it carries `number`, `days_needed_for_next` (the days
  still needed to reach *E+1*, at *E+1* kilometres) and `longest_day` (the
  longest distance in a day in the window, as the core renders it — the
  conversion happens at the display boundary, not inside the arithmetic).
* The window is the one the answer's `period` names: the same calendar
  weeks the volume chart draws. The card and the chart therefore cannot
  disagree about the period they describe.

The number is a fact, not a score: it says how many days reached how many
kilometres, over which weeks, and the reader decides what it means.

### The training load

`/insights/load?weeks=N` answers with the **training load** over a window:
the daily series, the two exponentially weighted series, the form, the
ratio, the week's impulse, monotony and strain, and the recovery scenario
with the assumption it rests on.

It is a route of its own, and not another field on `/insights/progress`:
the load needs its own window and its own "no value" reasons, and the
progress payload is already wide.

**The numbers are the core's.** The route reads them through the api's seam
(`crates/stride-api/src/load.rs`), which hands the core the athlete's
stored zone model, the measured resting heart rate of the daily summary and
the stored activities. The daily loads, the acute and chronic loads, the
form, the ratio, the week and the scenario all come from
`stride-core`'s load engine; the route writes them as they are.

#### The window and the warm-up

The window is the Progress window: the same `weeks` parameter (1 to 104,
default 12) and the same calendar weeks the volume chart draws, the
current partial week included.

The load's own constant is longer than most windows — the chronic series is
weighted over **forty-two** days — so the series is **warmed** with
`window.warmup_days` (210) days of history read *before* the window. The
warm-up's days are weighed and are not part of the window's series; without
them the numbers at the start of a window would be the numbers of a series
that began there. The warm-up is the reference implementation's own
(`TrainingLoadChart::EMA_WARMUP_DAYS`).

#### The fields

| field | what it is |
|---|---|
| `window.from` · `window.to` | the first and last day of the window, in the athlete's own calendar |
| `window.from_label` · `window.to_label` | the same days, **as the api writes a date**: "4 May 2026" |
| `window.weeks` | how many calendar weeks the window covers |
| `window.warmup_days` | how many days of history the series was warmed with |
| `load.days[]` | one day of the window: `date`, `label`, `load` (a rendered value) and `reason` |
| `load.states[]` | the state of the two series at each of those days: `acute`, `chronic`, `form`, `ratio` |
| `load.week` | the seven-day values: `impulse`, `monotony`, `strain` — `null` when the week is not complete |
| `load.assumptions` | the assumption the numbers rest on: the coefficient, the maximum heart rate, the threshold, the day the zone model was read, the two windows and `statement` |
| `scenario` | the recovery scenario: `days_to_ratio_band`, `days_to_positive_form`, `band`, `statement`, `assumption`, `projection` |
| `unavailable` | why there is no load, or `null` when there is one |

The load is **dimensionless** — there is no unit to write — so the number is
the value, and every one of them crosses as a rendered value with the core's
own precision: whole numbers for the day's load, the impulse and the
strain, one decimal for the acute load, the chronic load and the form, two
for the ratio and the monotony.

#### No value

**No value is the absence of the value**, and it is never a zero:

- no zone model has been read → `load: null`, `scenario: null`, and the
  reason in `unavailable`. Never a guessed maximum heart rate, never an age
  formula, never a zero;
- the stored model carries no maximum heart rate, or its floors do not
  increase → the same shape, and the reason;
- the window holds no recorded day → `load: null`, `scenario: null`, and
  the reason.

A day inside the window that holds nothing which could be weighed carries
`load: null` and its own `reason` — a day is not a zero because nothing on
it could be weighed. A week that is not complete carries no `week` at all:
no impulse, no monotony and no strain.

The reasons are the **core's own** — the api maps the core's reasons onto
stable keys and never writes a reason of its own.

#### The voice

The load is the product's most misused number, so the route carries **no
verdict**: no band name, no "high risk", no "optimal", no traffic light and
nothing that tells the athlete what to do. The band of the ratio is named as
a **range** — `0.8 to 1.3` — and the scenario says what it assumes: a
complete rest day, the Banister coefficient, the zone model the maximum
heart rate came from and the day it was read, and the two windows.

### The rolling view

`/insights/rolling?weeks=N` answers with what a **window of history** holds:
the window against the window before it, the split of the training across
the athlete's own zones, what usually happened on which day of the week and
when in the day, and what each month and each year of the history holds.

It is a route of its own, and not another field on `/insights/progress`:
the rolling view has its own window, its own "no value" reasons and its own
tables, and the progress payload is already wide.

**The numbers are the core's.** The route reads them through the api's seam
(`crates/stride-api/src/rolling.rs`), which hands the core the stored runs
and the stored zone model. The window's totals, the split of the zones, the
weekday and daytime splits and the monthly and yearly tables all come from
`stride-core`'s rolling view; the route writes them as they are. **Nothing
is stored**: every one of them is recomputed on read, like every other
derived value.

#### The window

The window is **the last `N` days ending on the day the answer is about**,
and the comparison period is **the same `N` days ending a day earlier**.
That is the reference implementation's own comparison — its rolling window
is a window that moves one day at a time, and its zone split is compared
against "the same window as of the previous day" — and it is stated here
rather than left to be guessed at.

`weeks` is the Progress route's own parameter (1 to 104, default 12) and
the window is `weeks × 7` days, so the card on Progress follows the range
selector the screen already has.

**`days` is the same window, counted in days** (1 to 728), and it is what
the athlete profile's ladder asks for: thirty, sixty and ninety days. A
month is not four weeks, and the router answers the days exactly. When
both parameters are given, **`days` is the one that counts**.

The window applies to **the comparison, the split of the zones, the
distance breakdown and the profile**. The weekday and daytime splits and
the monthly and yearly tables cover the **whole saved history** — a
weekday is a fact about every Monday the athlete has run, not about the
last twelve weeks — and the cards say so.

#### The distribution, the breakdown and the profile

Three more answers ride on the same window.

* **`zones.distribution`** is the **distribution** of the training across
  the athlete's own zones: the time in each of the five, and the share of
  the time in the zones that was. It comes from the same time-in-zone
  computation as the bands, so the two cannot disagree. The ranges are the
  model's own floors.
* **`distances`** is the **distance breakdown**: the runs by how far they
  went. The bucket size is the reference implementation's own — it is
  taken from the longest run in the window, so that the longest run lands
  in the last bucket and there are about eleven of them. A window with no
  run has no buckets.
* **`profile`** is the athlete's shape over the window, as **seven values
  and their movements**: volume, consistency, intensity, duration,
  density, sport variety and the days of the week. It is **not a radar**:
  a polygon over fixed anchors is a claim about a shape, and the anchors
  are somebody else's coaching assumptions. Each facet is the athlete's
  own number, and each says which way it moved against the window before
  it. A window with no run has **no facets at all** — every value is no
  value, never a zero — and a facet with no value in the window before it
  has **no movement** to report.

  **Each facet carries two numbers.** `total` is what the window holds:
  the distance of the window, the days it carries a run — the number a
  reader asks for when they ask "how far did I run in three months?".
  `value` is the **comparable** number: the volume is a distance **a
  week** and the consistency the **share of the window's days** that
  carry a run, so that a thirty-day window and a ninety-day window can be
  read against each other at all. The intensity (a median), the duration
  (a median) and the density (an average per active day) do not grow with
  the window, so their `value` **is** their own number; the variety is a
  count of the athlete's sports and is a count in every window. The
  division is the core's, so every consumer gets a comparable number and
  nobody has to remember to divide.

### The pace page

`/insights/pace?weeks=N` answers the pace page's three panels in one
answer: the training week by week, the pace at a heart rate, and where the
time went by pace.

It is a route of its own, and not another field on `/insights/progress`:
the bands need their own window and their own "no value" reason, and the
progress payload is already wide. The window is the Progress window — the
same `weeks` parameter (1 to 104, default 12) and the same calendar weeks
the volume chart draws, the current partial week included — and the answer
**names** it.

* **`weeks`** is the training series: the week's distance, how many runs it
  holds, **the long run inside it** (its distance and its pace), and the
  week's own **separate medians** — the pace of the week's runs and the
  heart rate of the week's runs. The two medians are descriptive and are
  never a matched-effort comparison, and the route does not pair them into
  one.
* **`bands`** is the distribution of the window's moving time by the pace it
  was held at: the bands the window's time fills, thirty seconds a
  kilometre wide, fastest first. Each band carries its edges, its name in
  the core's words ("4:30–5:00 /km"), the time in it, the distance in it
  and its share of the window's time.
* **The bands are a property of the window.** The same runs over twelve
  weeks and over a year are two different answers, and the answer names
  the window it carries.
* **The samples are the runs' own**, decoded from their FIT files on read,
  and only the moving ones — the same filter the zones route has always
  used, so the numbers stay comparable with what it answered before. A
  pace exactly on a band's edge belongs to the band below it.
* **No value is the absence of the whole result.** A window with no moving
  samples has **no bands**: `bands` is `null` and `unavailable` says why,
  in the core's own words. It is never a band of zero.

### The heatmap

`/insights/heatmap` answers with **a grid over the stored positions**: how
many runs went through each cell. It is computed **on the device** — the
route fetches nothing, asks nobody and sends nothing, and there is no tile
server, no geocoder and no third-party service in the path that draws it.

The grid is a **derived series**, like the samples, the laps and the zones:
it is recomputed on read from the stored FIT files and **never stored**.
There is no grid, no tile and no heatmap blob in the database.

#### The grid

The grid is the slippy-tile grid at **zoom 14** — the reference
implementation's own resolution. A cell is about **two kilometres** across
at the latitudes a runner lives at, and the coarseness is the point: a cell
is a part of town and not a street, and a coarse grid is both faster to
compute and less identifying than a fine one.

The cells are in the Web Mercator projection, so a cell is a rectangle in
the athlete's own coordinates and a square on the ground. A position that
falls exactly on a cell edge belongs to the cell the edge opens — the
higher one, east or north.

#### A cell's value is a count, never a route

| rule | what it means |
|---|---|
| the count is of **runs** | a cell holds how many runs went through it, not how many positions and not how long |
| a run counts **once** in a cell | a run that loops round a block crosses the same cell many times, and the cell counts the run |
| a cell carries **no time** | there is no per-cell time, so no cell can be placed in a day |
| a cell carries **no order** | the answer is a set of rectangles, not a sequence, so no path can be read back out of it |
| a cell carries **no connection** | the cells are not joined to each other, so two neighbouring cells do not say a route ran between them |
| a cell is a **place, not a person** | nothing names a street, an address or a neighbourhood, and nothing here is a conclusion about the athlete |

#### The fields

| field | what it is |
|---|---|
| `window.from` · `window.to` | the first and last day of the window, in the athlete's own calendar |
| `window.from_label` · `window.to_label` | the same days, **as the api writes a date**: "18 Feb 2026" |
| `window.days` | how many days the window covers |
| `comparison` | the same window, a day earlier: the period the window is compared against |
| `distance` · `duration` | the value in the window, the value in the period before it, and the difference between them |
| `sessions` | the runs in the window, the runs in the period before it, and the difference |
| `zones.model` | the zone model the split was measured against: the day it was read and the model's own source line |
| `zones.shares` | the three bands — low (Z1–Z2), moderate (Z3), high (Z4–Z5) — with the share of the time in the zones, the share in the window before it, and which way it moved |
| `zones.distribution` | the five zones one by one: the label, the range the model gives the zone, the time in it and its share of the time in the zones |
| `zones.unavailable` | why there is no split, in the core's own words |
| `distances` | the distance breakdown: the buckets, the runs in them, their distance, their climb and their time |
| `profile` | the seven facets of the athlete's shape: the label, the value, the value in the window before it and which way it moved |
| `weekday[]` | one row per day of the week: the runs, the distance, the moving time and the day's share of the moving time |
| `daytime[]` | one row per time of day: the runs, the distance and the moving time, with the hours the bucket starts and ends at |
| `months[]` · `years[]` | one row per month and per year of the history: the runs, the distance, the moving time and the elevation gain |

The difference between two windows is the **core's** rendering of it, sign
and all: "+5 km", "−20 m". The sign is the number's own, and nothing in
the answer says which way is the better way.

#### The zones

The split of the training is measured against the athlete's **stored zone
model**, and the model travels with it: the day it was read and its own
source line. The time in the zones comes from the runs' own FIT files,
decoded on read, and only the moving samples — the same filter
`/fitness/zones` has always used, so the two cannot disagree.

The three bands are named for what they are — **low (Z1–Z2), moderate (Z3),
high (Z4–Z5)** — and there is **no target range beside them**: a range
that says how the split ought to be is a prescription, not a fact about
what happened, and the product does not prescribe.

#### No value

**No value is the absence of the whole result**, and it is never a zero:

- no zone model has been read → `zones.shares: null` and the reason in
  `zones.unavailable`;
- the stored model's floors do not increase → the model is named, and
  there is no value: `zones.shares: null` and the reason;
- the runs in the window carry no heart-rate samples → `zones.shares: null`
  and the reason.

The window's **totals** are values: a window with no runs holds 0 km, and
that is a fact about the window rather than a missing measurement. The
**shares** and the **streaks** are the values that are absent when they
cannot be computed.

A month with no runs is **not a row** in the monthly table, and a year with
one run in it is a year with one run in it: the table holds the periods the
athlete ran in, and no row of zeros.

#### The voice

The rolling view invites conclusions, and the product draws none. The
answer states the two windows' numbers side by side and the difference
between them, and says which way each share moved against the window
before it — "up", "down", "steady" — and stops there. There is no verdict,
no grade, no traffic light and nothing that tells the athlete what to do.

### The challenges

`/insights/challenges?weeks=N` answers with **the athlete's own
challenges, measured week by week**. A challenge is something the athlete
set themselves — a run of at least so many kilometres, or so many
kilometres across the week — and the grid says, week by week, whether the
week reached it.

**The weekly goal is a challenge already.** It is the same shape, so it is
in the grid from the first day without anything being set up; the
athlete's own challenges are written through `/settings` and sit beside
it.

**A week is met or it is not.** The answer is a fact: the week's distance
was what it was, and the longest run in it was what it was. **A week that
did not reach the target is not a failure** — it is a week, and the reader
draws the conclusion. Nothing in this answer is coloured as a problem,
because nothing in it is a problem.

The challenges are written as rows of the `notes` table, with the kinds
`challenge_week` and `challenge_run`; the target is in `value`, in
kilometres, and the athlete's own words for it, when they gave any, in
`text`. The weekly goal is a row of its own kind, `goal`, and is not one
of the challenge rows.

### The streaks

`/insights/streaks` answers one question about the whole history: **how
long is the streak**. It carries the current run of days, the run of weeks,
the run of months, and the longest one of each, with the days they cover.

**The arithmetic is the reference implementation's, exactly.** The days are
the distinct local days that carry a run; the walk goes from the newest day
backwards; the current streak survives one quiet day (the last run may
have been yesterday), and a streak that ended longer ago is not current.
The week streak counts weeks with a run in them, and the weeks are the
athlete's own: Monday through Sunday. The month streak counts **months**
with a run in them, and a month with no run in it breaks it — the same
rule with a month's span, so the month itself may still come.

| field | what it is |
|---|---|
| `days.current` · `weeks.current` · `months.current` | the streak that is still going: the count and the day it started |
| `days.longest` · `weeks.longest` · `months.longest` | the longest one in the history: the count and the days it covers |

**A streak of zero days is no streak**: `current` is `null` when the last
run was more than a day ago, and the whole answer is empty when there are
no runs at all. A number that could be read as "zero" is not a number
here.

The start of a **week** streak is the newest run day in the streak's oldest
week — the reference's own value — and the longest week's dates run from the
oldest day of that walk to the newest. A **month** streak follows the same
rule with the month: the start is the newest run day in the streak's oldest
month, and the answer names the **month** — `2026-03` and "March 2026" —
because the month is the unit of a month streak, not the day inside it.

The voice is the plan's: a streak is a count and a date. There is no "you
are consistent" anywhere near it.

| `runs` | how many runs carried a position and were counted |
| `more_runs` | whether the store holds runs this answer did not read: the answer is bounded, and says so rather than letting a partial map read as the whole history |
| `cell_size` | the size of one cell on the ground, as the core renders it: "2 km". `null` when there is no cell |
| `cells` | the cells, north to south and west to east: `x` and `y` are the cell's place in the grid, `south` · `west` · `north` · `east` are its corners, and `runs` is its count |
| `extent` | the box the cells cover, or `null` when there is none |
| `unavailable` | why there is no heatmap, or `null` when there is one |

#### No value

**No value is the absence of the whole result**: a store whose runs carry no
position answers with `cells: []`, `extent: null`, `cell_size: null`,
`runs: 0` and the reason in `unavailable` — never an empty grid of cells
that says "nowhere".

### The year

`/insights/year` answers with **a year of the athlete's own history**: the
twelve calendar months, each with its own character; the **calendar of the
days**, coloured by the zones the days were run in; and **what changed** —
the diff against the athlete's own past.

It is a route of its own, and not another field on `/insights/rolling`: the
year has its own window (the twelve calendar months, and the period before
them), its own "no value" reasons and its own sentences, and the rolling
payload is already wide.

**The numbers are the core's.** The route reads them through the api's seam
(`crates/stride-api/src/year.rs`), which hands the core the stored runs,
their zone times and the load's form. The days, the months, the window's
totals and the sentences of the diff all come from `stride-core`'s year.
**Nothing is stored**: every one of them is recomputed on read, like every
other derived value.

#### The window

The window is **the last twelve calendar months**, ending on the day the
answer is about, and the period it is compared against is **the same number
of days, ending a day earlier** — the rolling view's own comparison. The
current month is the month so far. The answer names both windows, so a
number is never read apart from the period it describes.

#### The fields

| field | what it is |
|---|---|
| `window` · `comparison` | the two windows: their first and last day, the api's own rendering of those days, and how many days each covers |
| `months[]` | one row per calendar month, oldest first: the runs, the distance, the moving time, the elevation gain, the longest run, the form at the month's end, the weeks the month touches, the mix of the zones and the share of the high zones |
| `months[].weeks[]` | one row per calendar week the month touches: the week's first day, the distance of the week's days that fall in the month, and the runs in them |
| `months[].mix[]` | one row per zone: the zone's label and its share of the time in the zones |
| `days[]` | one row per day of the window, oldest first: the day, the runs on it, its distance and the share of its time in the zones that was in the high zones |
| `changed[]` | what changed, in the core's own sentences |

#### The calendar colours days, never places

**The calendar is about *when* and *how hard*.** The days carry the zone
mix of that day's runs and nothing else: no position, no place name and
nothing that says where the athlete ran. Where they ran is the **heatmap's**
surface, and it has its own screen, its own disclosure and its own rules.
The two answers are deliberately different things, and the year's calendar
is not a map.

#### The zone mix

The mix of a day — and of a month — is the **same time-in-zone
computation** the bands use: the athlete's **stored model**, the runs' own
FIT files decoded on read, and only the moving samples. A day's mix is the
time of the runs that could be measured; a day whose runs carry none has
**no mix at all**.

#### No value

**No value is the absence of the whole result**:

- a day with no run has **no distance**;
- a day whose runs carry no zone times — no file, no stored model, no
  heart-rate samples — has **no mix**, and no share of the high zones;
- a month with no runs has **no distance, no longest run, no form and no
  mix**, and its weeks hold no distance;
- a diff with nothing to compare has **no line** for that value.

The window's **totals** are values: a window with no run holds 0 km, and
that is a fact about the window rather than a missing measurement.

#### The voice

The year invites conclusions — that it was a good year, that the training
was unbalanced, that the athlete peaked — and the product draws none. Each
month is a fact about a month, the year says what happened and what
changed, and the sentences of the diff are the diff's own numbers. There is
no verdict, no score and nothing that tells the athlete what to do.

### The week on Today

`/insights/today` answers with the plan's session for the day, the plan's
other sessions, the last run and **the week to date** — and, beside the
week, **the same elapsed days of the previous calendar week**.

The comparison is **like for like**. The week to date runs from the Monday
of the athlete's calendar week to today; the comparison runs from the
Monday of the week before to the same weekday a week back. The two windows
are the same length, so the difference weighs the same number of days
against the same number of days. A week to date weighed against a **whole**
week would weigh a part against the whole, and the number would be about
the calendar rather than about the running.

| field | what it is |
|---|---|
| `week.distance` | the distance of the week to date, as the core renders it |
| `week.sessions` | how many runs there were in it |
| `week.days` | the seven days of the calendar week, each with the sessions recorded on it — recorded sessions, never an inferred rest day |
| `week.previous` | the same elapsed days of the previous calendar week: the days it covers, the distance and the sessions |
| `week.difference` | the week's distance against the previous one's, as the core renders the difference: the sign is the number's own, and nothing here says which way is the better way |

**A week with no runs in it is zero kilometres** — the distance that was
covered, which is a fact — and the difference against the same days last
week says so. It is never a made-up number, and it is never the whole
previous week.

The voice is the plan's: a distance, a difference, and the days they
cover. There is no verdict about any of it.

### The settings, the goal and the pause

Four inputs change what the app says, and three of them live in the `notes` table
(the schema is frozen, and it holds them already).

- **The goal** is a weekly distance, in kilometres — the schema's own unit. **A
  blank goal is no goal**: the answer carries `null`, never a zero that reads
  as a failure. The rendered value's `raw` is in metres, like every other
  distance the app draws, and `km` is the number the field holds.
- **The pause** is a start, an end and one of the athlete's own three reasons —
  `travel`, `sick` or `injured`. A reason outside that set is refused.
- **While the plan is paused**, the adherence is suspended: `/plan/adherence`
  carries `suspended: true` and the reason, and the count is not a verdict
  about a week the athlete was not running.
- **The note** is the only free text the product stores. A run carries one; a
  new note replaces it.
- **The storage** is what is on the device: the size of the database and the
  number of runs, wellness days, planned sessions and notes in it. The size is
  rendered by the **host** — the core has no unit for bytes, and a file's size
  is not one of its values.
- **The delete** refuses the call unless the request carries the word that
  confirms it, and it deletes the data in one transaction. The sync cursors go
  with it: they point at runs that are no longer there.

### The zones and the zone model

`/fitness/zones?id=…` answers one question about one run: **how long was
the athlete in each of their zones**. The zones are the athlete's own — the
watch's configuration, read from Garmin at
`/biometric-service/heartRateZones/v2` and stored with the day it was read.
The route does not rebuild bands from the run's own maximum heart rate: two
runs measured on two scales are not comparable, and a band that is not the
athlete's configuration cannot be named, dated or audited.

The route keeps its name and its place in the table; its **meaning** changed,
and the old computation is gone rather than left beside the new one.

#### The fields

| field | what it is |
|---|---|
| `basis` | the basis the times were computed on: `"moving"` — the spans of the run's FIT file whose sample says the athlete was moving |
| `model` | the zone model the times were computed against, or `null` when no model has been read |
| `model.sport` | the sport Garmin reported the model for: `"DEFAULT"` |
| `model.training_method` | the training method the model was set with: `"HR_MAX"`, `"LACTATE_THRESHOLD"`, `"HR_RESERVE"` or `"CUSTOM"` |
| `model.max_heart_rate` | the maximum heart rate the model was built on, when Garmin reported one: a rendered value |
| `model.floors.zone1` … `zone5` | the five floors, in beats per minute, as rendered values |
| `model.read_at` | when the model was read from Garmin: an ISO-8601 UTC instant |
| `model.read_on` | the **day** the model was read, in the athlete's own calendar, in the api's own date rendering: "10 May 2026" |
| `model.source` | the model's own source line, in the core's words: the method it was set with, the maximum heart rate it was built on and the day it was read |
| `zones` | the five bands, each with the time spent in it: `label`, `range`, `from_bpm`, `to_bpm`, `time` |
| `zones[].time` | the time in the band, as a rendered value; the number beside the words is the time in **seconds** |
| `outside` | the time outside the zones — no heart-rate reading, or a reading at or below the first floor — as a rendered value |
| `stopped` | the time the athlete was **not moving**: the stretches they stood still, as a rendered value. The zones and the outside add up to the movement they were measured over, and the stopped time sits beside it |
| `unavailable` | why there is no value, in the core's words |

The model's floors are the **exclusive lower bounds** of the zones: the
interval of zone *n* is `(floor_n, floor_{n+1}]`, and a reading at or below
the first floor is outside the zones. **A reading exactly on a floor belongs
to the lower zone** — the reading at `floor_n` is in zone *n−1*, and only a
reading strictly above `floor_n` reaches zone *n*. The band drawn on a chart
runs from `floor_n` to `floor_{n+1}`; the top band runs from `floor_5` to the
maximum heart rate the model was built on, and has no upper edge when the
model carries no maximum heart rate.

#### The date of the model

The day the model was read is rendered by the **api**, in the api's own
form — "10 May 2026", the form the run detail already uses for the run's
own date. The core's canonical form (`2026-05-12`) stays inside the core:
the core's source line is handed the rendered string and never formats a
date for a screen, and a third form would be a form nobody owns.

#### No value

**No value is the absence of the whole result**, and it is never band 1 and
never a zero:

- no model has been read → `model: null`, `zones: []`, `outside: null`, and
  the reason in the core's words;
- the stored model's floors are not strictly increasing → the model is named,
  and there is no value: `zones: []`, `outside: null`, and the reason;
- the run has no samples to read → no value, and the reason.

The reason is the core's own sentence, asked for out of the core's catalogue
and passed on. The api never writes a sentence of its own.

#### The samples

The times are computed from the run's own FIT file, decoded on read, and from
the **moving** samples only — the standing samples are filtered out, exactly
as they always were, so the numbers stay comparable with what the route
answered before the model came in. The derived series are recomputed on read
and never stored; the zone **model** is the exception, because it is not in
the FIT: it is the watch's configuration, and it is stored, dated.

### The fingerprint and the run's mechanics

`/fitness/fingerprint?id=…` answers one question about one run: **what did
this run's laps say about this run**. The observations are the run's own
mechanics — the cadence and how it varied, the pace's variation, the drift
between the first third of the run and the last, the half-split drift of the
heart rate and of the pace, the efficiency factor, the aerobic decoupling,
and the shares of the moving time in the athlete's own zones.

The run is named by the query, like the zones. The **basis** is named too —
the run's laps and its moving samples — and the answer says how many laps
the analysis used: the **running** ones.

#### There is no score

The answer carries **observations**, and no composite. The reference
implementation ended with a `form_score`: a weighted mean of three bounded
component scores, with the weights renormalised around whichever component
was missing. That number is a grade, and the product does not ship grades.
What is worth keeping is the observations the score was built from, each of
which says what it measured.

#### The fields

| field | what it is |
|---|---|
| `activity_id` | the run's id |
| `local_date` | the athlete's local date |
| `basis` | the basis the numbers were computed on: `"laps and moving samples"` |
| `laps` | how many laps the analysis used: the running ones |
| `observations` | the observations, in the order the app shows them |
| `observations[].key` | the observation's name: `cadence`, `cadence_variability`, `efficiency`, `pace_variability`, `cadence_drift`, `heart_rate_drift`, `pace_drift`, `decoupling`, `stride_length` |
| `observations[].label` | the label, in the core's own words: "Cadence" |
| `observations[].value` | the value, as the core renders it: "163.0 spm" |
| `observations[].reason` | why there is no value, in the core's words |
| `zones` | the zone distribution: the shares of the moving time in the athlete's own zones |
| `zones.model` | the stored zone model the shares were measured against, named and dated |
| `zones.shares` | one per zone: `label` ("Z1"), `share` (the share of the time **in the zones**) and `time` |
| `zones.outside` | the time outside the zones: no heart-rate reading, or a reading at or below the first floor |
| `zones.note` | where the model came from, in the core's words: "your zone model, read on 2026-05-10" |
| `unavailable` | why there is no value at all, in the core's words |
| `generated_at` | when the answer was produced |

#### The observations

| observation | what it is |
|---|---|
| `cadence` | the mean cadence of the running laps, in steps per minute |
| `cadence_variability` | the **coefficient of variation** of the cadence: the sample standard deviation over the mean, as a percentage, from three laps or more |
| `efficiency` | the **efficiency factor**: the speed over the heart rate as a fraction of a maximum heart rate — speed and not pace, because the pace-to-heart-rate ratio is the reciprocal and would make a slower run look more efficient |
| `pace_variability` | the coefficient of variation of the paces |
| `cadence_drift` | the change in the cadence between the **first third** and the **last third** of the running laps, as a percentage |
| `heart_rate_drift` | the **half-split** drift: the difference between the mean heart rate of the first half of the running laps and the second |
| `pace_drift` | the half-split drift of the pace, in seconds per kilometre. A negative number is a negative split |
| `decoupling` | how far the efficiency factor fell between the first third and the last, as a percentage |
| `stride_length` | the mean of the laps' own stride lengths, in centimetres. A stride longer than three metres is rejected as implausible |

The **sign of a difference is the direction**: `+5 bpm` is a heart rate that
rose, `−4 s/km` is a pace that quickened. The pace variation leaves out a
last lap that holds less than a fifteenth of the run's distance: a fragment
of a lap says more about where the watch stopped than about how the run was
run.

#### No value

**No value is the absence of the value**, and it is never a zero:

- the run has no file → there are no observations at all, and the reason in
  the core's words;
- the run's laps carry no cadence → the cadence, its variation and its drift
  say so;
- the run is too short to split → the drifts say so: fewer than three laps
  for the thirds, fewer than two for the halves;
- the file carries no stride length → the stride length says so;
- there is no maximum heart rate to measure the efficiency factor against →
  the efficiency factor and the decoupling say so;
- no zone model has been read → the zone distribution is "no value", and
  says why.

The reason is the core's own sentence, asked for out of the core's catalogue
and passed on. The api never writes a sentence of its own.

#### Where the inputs come from

The observations are computed from the run's **own laps**, and the zone
shares from the run's **moving samples** — the same filter the zones route
uses, so the two answers cannot disagree. The derived series are recomputed
on read from the FIT file and are never stored.

The maximum heart rate the efficiency factor is measured against is the one
from the athlete's **stored zone model**: their watch's own configuration,
read once and stored with the day it was read. It is never the run's own
observed maximum — two runs, two scales — and never an age formula.

### The run film

`/activities/{id}/film` answers one question about one run: **what this run
looked like while it was happening**. The film is the run's own series,
drawn as a timeline and played back: the pace, the heart rate and the
ground on one axis, the athlete's own zones behind them, and the route
beside it with a camera that follows the runner.

It is a route of its own because it is a **surface**, not a metric: the
answer is one run's whole timeline, and no other route answers that. It
sits with the run's other sub-routes — `/activities/{id}/fit`,
`/activities/{id}/note` — and it is named by the run it belongs to, so the
run's own facts and the film's own series are two halves of one page.

#### The name of the route

`/activities/{id}/film` and not `/fitness/film?id=…`. The `/fitness/*`
routes answer questions about the athlete's **fitness** — the zones, the
fingerprint, the trends — and they are named by a query because they are
about the athlete. The film is about a **run**: it is that run's samples,
that run's ground and that run's splits. The zone model travels with it,
named and dated, exactly as it travels with `/fitness/zones`.

#### The fields

| field | what it is |
|---|---|
| `activity_id` | the run's id |
| `local_date` | the athlete's local date of the run |
| `basis` | the basis the series was read on: `"samples"` — every sample of the run's file, standing ones included |
| `model` | the zone model the zones were measured against, named and dated; `null` when no model has been read |
| `zones` | the five bands of the athlete's zones: `label` ("Z1"), `range` ("106-121 bpm"), `from_bpm` and `to_bpm` |
| `outside_label` | the name of the space **outside** the zones, in the core's words: "outside your zones" |
| `samples` | the run's samples: `elapsed`, `distance`, `pace`, `heart_rate` and `zone` |
| `samples[].elapsed` | the time from the start of the run, as the core renders it: "42 m". The number beside the words is the time in **seconds** |
| `samples[].distance` | the distance from the start, as the core renders it: "3.2 km". The number beside the words is the distance in **metres** |
| `samples[].zone` | the zone the sample sits in: the index of the band in `zones`, counted from zero. `null` when the reading is outside the zones |
| `ground` | the run's ground: `offset_s` (the seconds from the start) and `elevation` |
| `splits` | the run's splits: `lap_index` (the lap's number — a lap, not a kilometre), `offset_s` (the seconds from the start **to the beginning of the split**), `distance`, `duration`, `pace` and `heart_rate` |
| `unavailable` | why there is no film at all, in the core's words |
| `zones_unavailable` | why the zones have no value, in the core's words. The film itself may still be there: the pace, the heart rate and the ground do not need a zone model |
| `generated_at` | when the answer was produced |

#### The zone of a sample

The zone of a sample is the **core's**: the athlete's stored model, read
through `stride-core`'s own classifier — the one the time-in-zone
computation uses — so the ribbon behind the film's timeline and the bands
on the run detail cannot disagree about which zone a reading is in.

The boundary rule is the contract's: a reading exactly on a floor belongs
to the **lower** zone, and a reading at or below the first floor is
**outside** the zones, as is a reading that is not there at all. The
samples carry the zone as the index of the band in `zones`, and `null` for
outside — so the ribbon has six bands: the five zones and the space
outside them.

#### The ground is on the run's time axis

The run detail's elevation is **distance-based**, and it stays that way: it
is the profile of the ground, not a series over time. The film's ground is
different — it is the same altitude on the **elapsed** axis, so the pace,
the heart rate and the ground share one cursor, and the playhead crosses
all three at the same moment.

#### No value

**No value is the absence of the whole result**, and it is never a zero:

- the run has no file → there is no series at all, and the reason in the
  core's words;
- the run's file carries no samples → the same shape, and the reason;
- no zone model has been read → the film is still there, the **zones** have
  no value, and the reason says so;
- the stored model's floors are not strictly increasing → the same: the
  model is named, and the zones have no value.

The reason is the core's own sentence, asked for out of the core's
catalogue and passed on. The api never writes a sentence of its own.

### The run update

Every run gets a short **update**: four to six sentences saying what the run
was and how it went. A language model writes it from the run's own numbers,
a **second** model reviews the draft before it reaches the athlete, and
three mechanical checks stand between the two models and the screen.

The update is **not the product's voice**. It is the model's reading, it is
attributed — the writer and the reviewer are named — and it lives beside the
app's own facts and not inside them. The app's numbers stay the app's:
computed in Rust, from the stored series, with their sources named.

#### The routes

| route | what it answers |
|---|---|
| `/activities/{id}/update` | the stored update of a run, with its checks and its verdict — `null` when the run has none |
| `/activities/{id}/update/payload` | the facts a model may read for that run, and the hash of them |
| `/activities/{id}/update/request` | the prompt for a role — `writer`, `reviewer` or `repair` — with the payload it carries |
| `/update/settings` | the update's own settings |

**The rig owns the calling, not the router.** The rig asks the model to
write, asks the second model to review, and asks this router what to send
it and whether a draft may be kept. The router never calls a model.

#### The payload is metrics only

The payload carries the run's own facts — the distance, the duration, the
pace, the heart rate, the splits, the time in the athlete's own zones, the
run's own mechanics, the writer's model name and the day it was read — and
**nothing else**: never the positions, never the athlete's note, never their
name, never the plan. The payload route is the answer to "what would be
sent", and the app shows it before the first send.

A run with no file to read has no payload: the answer says why in the
core's own words, and there is no update for it.

#### The three checks

The core owns them, beside the other derived values, and they are
mechanical:

| check | what it asks |
|---|---|
| groundedness | does every number in the draft appear in the payload? |
| direction | does a claim with a sign agree with the payload's signed value? |
| register | does the draft use a word the product forbids? |

The register's word list is the core's own ban list — the one
`docs/contract/voice.md` freezes and the one the app's lint reads out of
the export — plus the words that would make an observation a prediction.
**One list, two readers**: the app's lint and the reviewer's check cannot
drift apart.

#### What may be kept

Only a draft that passes the three checks **and** that the reviewer
shipped. A repair is a second pass with the reviewer's findings; a discard
keeps no update. A draft that does not pass the checks keeps nothing, and
the answer says what the checks found.

The stored update keeps the writer, the reviewer, the **hash of the
payload**, the draft, the checks and their findings, the verdict and the
day it was made. An update you cannot re-check is just a sentence, and the
product's whole claim is that its numbers can be inspected.

#### The settings, and what may leave the machine

The feature is **off by default**, and **nothing leaves the machine** until
the athlete says so. The settings name the writer, the reviewer and the two
base URLs; the **key is not in them** — the settings name the environment
variable it is read from, so the key is never in the database, never in an
export and never in a log line.

The reviewer's family has to differ from the writer's: a reviewer of the
writer's own family repeats the writer's misreadings. The settings refuse
that, and the rig refuses it again before it writes anything.
