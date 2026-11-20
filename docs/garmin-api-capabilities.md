# Garmin Personal API capabilities

Verified 2026-11-04 against Garmin's official Personal API specification and
bounded live reads. Implementation baseline:
`3b568bf9bf68df27dc00e74056d598f133a89fdb`.

This is the canonical upstream capability inventory. Stride's own routes are
specified separately in [the API contract](contract/api-paths.md). An upstream
endpoint, a client method, a stored observation and a working screen are four
different levels of support.

## Evidence and boundaries

- [Official Personal API specification][swagger]: OpenAPI 3.0.1, API 1.0.0,
  26 documented data endpoints, all GET.
- Official API host: `https://personal.connectapi.garmin.com`.
- Fifteen relevant data endpoints were read live. Only HTTP outcomes and field
  names/types were retained. No measurement values, identifiers, names,
  coordinates, workout notes, raw payloads or credentials belong in this document.
- Live reads did not run Stride sync, modify its database, change account
  settings or publish anything. The zone request explicitly used
  `generateDefaults=false`.
- HTTP 200 confirms access to that response, not completeness. HTTP 204 or an
  empty result is not proof that a metric is permanently unsupported. A scalar
  field can still contain a sentinel or an unusable reading.
- Unrelated sensitive data and out-of-scope measurements were not probed.

Authentication exchanges the Personal API key for a short-lived access value.
The key comes from Key Vault in process memory; neither key nor bearer is
printed or saved. The exchange can return plain text, so a probe must not
require an `application/json` response for that request.

## What Stride currently pulls

The normal sync uses activity search and original FIT downloads, plus daily
sleep, daily HRV and daily user summaries. It persists sleep duration/stages,
nightly HRV, resting HR and average stress — the product's own projection of
the three vitals. It also reads the heart-rate zone model
(`/biometric-service/heartRateZones/v2`) and stores it as a dated fact —
the watch's configuration, not a per-run series. The profile endpoint is used
to test a connection, not as a health-data sync.

**The source's own wellness observations are kept, with their provenance.**
The `wellness_observations` table holds one row per (day, source, field,
item): the value, the day the response carried, and the day it was read. A
field the source never sent has no row; a field it sent as null has a row
whose value is null; a later read never deletes a row, so a failed read
leaves the previous value and its reading time in place. The two reports
that carry the rest — the stress durations per band and the body battery —
are read over the window the pull covers.

The FIT original is retained, allowing future reprocessing of fields the
current decoder does not project. The wellness projections retain only
selected scalars, but the source's own readings are no longer discarded:
the original sleep, HRV and daily-summary payloads are not stored verbatim,
yet every field they carried is kept as an observation.

Implementation evidence:

- [Transport and endpoint methods](../crates/stride-ingest/src/garmin.rs),
  constants at lines 21-66 and methods at lines 431-575.
- [Sync, wellness and zone-model mapping](../crates/stride-api/src/routes/app.rs),
  the activity loop, the wellness loop, and `wellness_of` /
  `zone_model_of` in the helpers below.
- [The wellness observations](../crates/stride-api/src/wellness.rs): a Garmin
  payload and the rows it carries.
- [Stored activity, wellness and zone-model fields](../crates/stride-store/src/model.rs),
  `NewActivity`, `WellnessDaily`, `WellnessObservation` /
  `NewWellnessObservation` and `HeartRateZoneModel`.
- [Wellness-observations repository](../crates/stride-store/src/repos/observations.rs),
  `write_wellness_observations` and the reads beside it.
- [Zone-model repository](../crates/stride-store/src/repos/zones.rs),
  `write_hr_zone_model_if_new` and `current_hr_zone_model`.
- [Default sync budget](../crates/stride-api/src/state.rs), lines 208-210.

## Complete upstream inventory

Paths are relative to the official API host. "Unused" means not pulled by
normal v2 sync, not that no code or test mentions the endpoint.

| GET path | Verification | Current Stride use / limitation |
|---|---|---|
| `/activitylist-service/activities/search/activities` | Live 200 | Used for sync discovery; additional training metadata is not retained as source JSON. |
| `/activity-service/activity/{activityId}` | Live 200 | Client method exists, not used by normal sync; detailed summary includes optional training effect and recorded effort fields. |
| `/activity-service/activity/{activityId}/workouts` | Live 200 | Unused. Workout steps associated with an already recorded activity, not the future workout calendar. |
| `/download-service/files/activity/{activityId}` | Documented; not re-downloaded in this audit | Used by sync. Preserve original FIT bytes; this audit did not repeat downloads. |
| `/biometric-service/heartRateZones/v2` | Live 200, defaults disabled | Consumed by sync; stored as a dated fact in `hr_zone_models` (the watch's configuration, one row per model change). The per-run time in zone is computed from that stored model, and the answer names the model and the day it was read. |
| `/sleep-service/sleep/dailySleep` | Live 200 | Used. The duration and the stages are the product's projection; the window, the window's confirmation, the unmeasurable time, the naps and the respiration are kept as observations, with the day the response carried and the day it was read. |
| `/sleep-service/sleep/dailySleepSummariesByDate` | Live 200 with correct range parameters | Client helper exists but sends `date`; documented parameters are `startDate` and `endDate`. Not ready to wire into sync unchanged. |
| `/hrv-service/hrv/{calendarDate}` | Live 204 in sampled window | Used. The projection extracts `hrvSummary.lastNightAvg`; the weekly average, the five-minute high and the baseline are kept as observations. A no-content response must stay missing, not become zero. |
| `/hrv-service/hrv/daily/{fromDate}/{toDate}` | Live 204 in sampled window | Client helper exists, unused. Range support is documented; data availability remains conditional. |
| `/usersummary-service/usersummary/daily` | Live 200 | Used. The projection keeps the resting HR and the average stress; the rest of the payload — the stress durations per band, the stress qualifier, the body battery, the seven-day average resting heart rate, the day's minimum and maximum heart rate, the respiration, the steps and the source's own range of the day — is kept as observations. |
| `/usersummary-service/v2/dailysummaries` | Live 200 | Unused range endpoint. Response wraps days in `data`; nested schema differs from the single-day endpoint. |
| `/wellness-service/wellness/dailyHeartRate` | Live 200 | Client method exists, unused. The daily summary carries the same readings — the day's minimum and maximum heart rate, and the seven-day average resting heart rate — and the sync's request budget does not stretch to a fourth read per day. Dated heart-rate samples and resting HR, not an ECG. |
| `/wellness-service/wellness/bodyBattery/reports/daily` | Live 200 | Read once per pull, over the window the pull covers: the day's charged and drained, and the curve, are kept as observations. Garmin's estimate, not Stride's recovery verdict — and never combined into one. |
| `/usersummary-service/stats/stress/daily/{startDate}/{endDate}` | Live 200 | Read once per pull, over the window the pull covers: the durations per band, the proportions and the qualifier are kept as observations. Daily aggregates, not a continuous intraday stress stream. |
| `/usersummary-service/stats/steps/daily/{startDate}/{endDate}` | Live 200 | Client method exists, unused; later context, not a reason to expand the initial product. |
| `/wellnessactivity-service/activity/summary/range/{fromDate}/{untilDate}` | Live 200, empty result | Unused wellness-activity summaries. Empty does not establish support for a particular activity type or device. |
| `/userprofile-service/userprofile/settings` | Documented; not probed | Used for connection testing. Avoid copying personal profile data into diagnostics. |
| `/fitnessage-service/fitnessage/v2/{calendarDate}` | Documented only | Unused. Optional proprietary estimate, not a clinical metric or initial-release requirement. |
| `/wellness-service/wellness/daily/spo2acclimation/{calendarDate}` | Documented only | Unused; device-dependent and outside the initial running/sleep/HRV/resting-HR scope. |
| `/usersummary-service/stats/hydration/daily/{startDate}/{endDate}` | Documented only | Unused; later scope. |
| `/weight-service/weight/all/{startDate}/{endDate}` | Documented only | Unused; later scope. |
| `/weight-service/weight/range/{startDate}/{endDate}` | Documented only | Unused; later scope. |
| `/goal-service/goal/user/effective/weightgoal/{startDate}/{endDate}` | Documented only | Weight goals, not weekly running-distance goals or workout scheduling. |
| `/nutrition-service/food/logs/{date}` | Documented only; deliberately not probed | Outside agreed scope. |
| `/periodichealth-service/menstrualcycle/summary/{calendarDate}` | Documented only; deliberately not probed | Outside agreed scope; do not collect by default. |
| `/periodichealth-service/menstrualcycle/dayview/{calendarDate}` | Documented only; deliberately not probed | Outside agreed scope; do not collect by default. |

## Important unused value

These are implementation proposals, not features shipped by this audit.

| Priority | Opportunity | Why it matters / prerequisite |
|---|---|---|
| First: correctness | Use configured sport-specific HR zones, or explicitly labelled Garmin-reported time in zones. | Zone floors/method were live-accessible. Activity search also returned `hrTimeInZone_1` through `_5`. Stride now reads the configured model and stores it as a dated fact, and computes the per-run time in zone from that model rather than from each run's observed maximum. Preserve model/source and capture date; today's settings cannot be passed off as historical settings. |
| First: ownership | Preserve source wellness observations, with retrieval time and schema/provenance, inside encrypted storage and backup. | **Done.** The `wellness_observations` table (schema version 4) keeps one row per (day, source, field, item): the value, the day the response carried, and the day it was read. A field the source never sent has no row; a field it sent as null has a row whose value is null; a later read never deletes a row, so a failed read keeps the previous value and its reading time. The export carries the table and the manifest counts it. |
| First: existing scope | Preserve sleep start/end, confirmation, unmeasurable time and naps when supplied; preserve HRV baseline/weekly fields when supplied. | **Done**, as observations: the sleep window, the window's confirmation, the unmeasurable time, the naps and the HRV's weekly and baseline fields are kept with the day the response carried and the day it was read. The missingness is kept, and no sleep debt is inferred from absent sleep-need data. |
| First: reliable sync | Add dated range pulls with explicit budgets, gaps and retry state. | Sleep range and v2 summary range worked live. Correct the existing sleep-helper parameter mismatch first; map each response's actual date and units. Do not silently turn API failures into missing measurements. |
| Next: activity context | Retain training-effect fields, effort/feel and attached completed-workout steps where present. | Live responses exposed these fields. They can explain a completed session or compare historical intent with execution. Garmin-authored estimates and user-recorded effort need distinct provenance; steps have typed units/repeat semantics. |
| Later: optional recovery context | Expose stress and Body Battery as opt-in Garmin data, not a second readiness score. | Much of the daily context already arrives in the summary payload; a Body Battery curve needs the separate report. Average stress is already stored, so this is not wholly new ingestion. |
| Later: device-conditional detail | Running power, VO2 max, training load and threshold fields. | Listed in official activity schemas, but not verified populated by this probe. Preserve where present before promising charts; no new major screen or medical interpretation follows merely from field availability. |

Cadence and run/walk segmentation are already decoded from FIT. Restoring
their useful presentation is not an upstream API expansion.

### Existing incremental sync is not complete backfill

Stride already skips unchanged FIT downloads and Android already schedules
background pulls. The remaining orchestration gap is durable per-resource
history coverage and resumable dated work, not the absence of a scheduler.
The activity scan starts at page zero on each call; its saved `newest` field
is not a persisted page/date backfill position. Wellness refreshes recent
days rather than progressively advancing through older missing windows.

**Source-level data-loss risk:** the activity listing loop can exit after
spending its download budget, without setting `stopped`. The subsequent
`stopped.is_none()` deletion guard can then treat unseen stored activities
as absent upstream. Fix and regression-test this completion distinction
before expanding backfill. Neither a partial scan nor a provider pagination
cap proves deletion; local history must survive by default.

See `routes/app.rs` lines 164-289 and 795-839, and the existing Android
`sync/SyncScheduler.kt` and `sync/GarminSyncWorker.kt`. This audit did not
run a destructive reproduction against live data.

## Plans: completed workouts are not future schedules

The 26-route specification has no scheduled-workout calendar, future plan,
workout publishing or device push endpoint. It also has no standalone
training-readiness or training-status endpoint. Do not borrow routes from an
unofficial Connect library and describe them as Personal API support.

The recorded-activity `/workouts` response did expose workout steps. That
can enrich the activity's historical context; it cannot populate next week's
calendar. A reference such as `metadataDTO.calendarEventInfo` is not a
future-calendar API either.

Stride's `/plan/week` reads its local plan engine: imported/prescribed,
typed, or inferred baseline. See
[the adapter](../crates/stride-api/src/plan.rs), lines 38-42 and 97-146.
The first experience should show actual weekly activity plus an optional
user-set goal. An explicit local plan may add planned sessions. A usual
week inferred from history must be labelled a baseline, not a prescription.

## Request bounds are not retention guarantees

| Layer | Verified fact | What it does not establish |
|---|---|---|
| Official sleep range | At most 31 days per request, using `startDate` / `endDate`. | How many years the account can recover or whether every day has data. |
| Official v2 daily-summary range | At most one year per request. | A one-year retention policy or permission to request all data eagerly. |
| Stride transport | Caps activity search traversal at 300 and date ranges at 365 days. | An independently verified universal Garmin history ceiling. The current specification's search parameters do not state that 300 is a provider-wide lifetime cap. |
| Stride default sync | 30-day lookback, at most 20 activity downloads and 14 wellness days per sync. | Full historical import, or recovery of an old gap by repeatedly fetching only the newest 14 days. Host configuration can override these defaults. |

No long-history, pagination-ceiling or maximum-date experiment was performed.
Until tested separately, document those limits as unknown and keep backup,
FIT import and durable wellness preservation independent of a future resync.

## Refresh this inventory rather than rediscovering it

1. Read this page and [the local API contract](contract/api-paths.md) first.
   Record whether a question is about upstream access, ingestion, storage,
   analytics or presentation.
2. Recheck [the official specification][swagger] after upstream changes.
   Its Swagger component currently embeds an OpenAPI JSON object; do not
   treat a bundled JavaScript filename as a permanent API URL.
3. Use a short-lived read-only process for any needed live check. Load the
   Key Vault credential inside that process or through the documented
   `cf-garmin env -- <command>` launcher. Retain no raw response. Keep
   requests bounded and sequential; stop on authentication, permission or
   rate-limit failures instead of looping. Do not automatically widen the
   set of sensitive endpoints.
4. Update the date, source commit, relevant rows and evidence level here.
   Use `documented`, `live response`, `ingested`, `stored` and `displayed`
   precisely. A mock screen is not integration evidence.
5. Any implementation needs synthetic fixtures, missing-data and failure
   cases, source/schema versioning and the normal review gates. If it changes
   scope or contracts, update `PLAN.md` and the contract in the same change.
   Link here from proposals rather than copying the whole matrix.

[swagger]: https://developer.garmin.com/gc-developer-program/personal/swagger-docs/
