# Garmin's API: per-activity temperature, and what else the activity detail carries

*24 November 2026. **Research note.** Nothing under `crates/`, `frontend/`, `android/` or
`desktop/` has been touched; nothing has been committed. The only file this note creates is
itself.*

Read with: [`exploration/stimulus-ledger.md`](stimulus-ledger.md) §5 (the FIT temperature
feasibility note), [`docs/garmin-api-capabilities.md`](../docs/garmin-api-capabilities.md) (the
upstream inventory), and [`docs/contract/core-contract.md`](../docs/contract/core-contract.md)
§2 ("no value" — `null` is the only representation of an absent measurement).

---

## 0. The answer, in one paragraph

**Yes, the API has per-activity temperature fields — and on this athlete's device they are
absent, always.** The official Personal API specification carries three temperature fields on the
activity summary (`averageTemperature`, `maxTemperature`, `minTemperature`, all `number`, all "in
degrees Celsius"), three more on each split summary, and two (`minTemperature`, `maxTemperature`)
on the activity **search** summary Stride already calls. The live read of the same endpoints found
**no temperature key at all** in any of 100 activity summaries over the last twelve months, in any
of 336 split objects, or in one activity's detail payload — while the fields that *are* present
(aerobic and anaerobic training effect, average and maximum heart rate, time in heart-rate zones,
elevation) came back populated. The fields are **absent, not null**: Garmin omits them when the
value does not exist. This is consistent with `exploration/stimulus-ledger.md` §5 — the device that
recorded these runs writes no temperature into the FIT file either (FIT record field 13 is absent
from all sixteen files in `fixtures/fit/`). The specification is silent on where the value comes
from, and the API carries **no weather fields at all** — no humidity, no wind speed, no "feels
like", no air pressure — so the API's temperature is, on this evidence, a device-side reading, while
the temperature
the user saw in Garmin Connect most likely comes from a source this API does not expose at all.
**The environment axis stays out of scope** (the user has said "running too hot or too cold" means exertion and load, not
weather); what survives is a device-conditional, three-number fact for the run detail — and, for
this athlete, a fact that would never appear at all.

---

## 1. Method and evidence

### 1.1 The specification (primary evidence)

The official Personal API specification was read from Garmin's own Swagger page
(`https://developer.garmin.com/gc-developer-program/personal/swagger-docs/`), whose page embeds an
OpenAPI 3.0.1 document titled **"Garmin Connect Personal API", version 1.0.0**, with **28 documented
paths** and **112 component schemas** (this is the same document the local inventory in
`docs/garmin-api-capabilities.md` points at; the path in the brief, `gc-develop-program`, is a
typo — the live path is `gc-developer-program`).

The two endpoints the brief names:

| endpoint | method | 200 response schema |
|---|---|---|
| `/activity-service/activity/{activityId}` | GET | `ActivitySummaryDTO` |
| `/activitylist-service/activities/search/activities` | GET | array of `SummarizedActivityDTO` |

`ActivitySummaryDTO` wraps `summaryDTO` (schema `SummaryDTO`, **188 properties**),
`splitSummaries` (array of `SplitSummaryDTO`), `metadataDTO` (schema `MetadataDTO`),
`activityTypeDTO`, `eventTypeDTO`, `timeZoneUnitDTO`, `deviceDisplayUnitsDTO`, and the
privacy/location/identity fields.

### 1.2 The live read (bounded, read-only)

Per `docs/garmin-api-capabilities.md` §"Refresh this inventory rather than rediscovering it": the
credential came from the Key Vault through `cf-garmin env -- <command>`; nothing was written to
disk; no raw response was retained; Stride's sync was not run; the local database was not touched;
no account setting was changed; nothing was published.

Requests made, sequentially, in short-lived processes:

1. the key exchange (one POST, the bearer never printed);
2. **one search page** — `start=0&limit=5` → HTTP 200, five items;
3. **one activity detail** — HTTP 200, one activity;
4. **one further search page** — `start=0&limit=100&startDate=<12 months ago>&endDate=<today>`
   → HTTP 200, one hundred items.

Request 4 is **one request more than the brief's "one search page"**, and it is disclosed here
deliberately: the deliverable asks "how often would it be present", and five activities cannot
answer that. It is the same endpoint, the same read-only data, one bounded call, no widening to
any other endpoint or any other kind of data. If a stricter reading is preferred, drop request 4
and the coverage answer falls back to "0 of 5 recent activities".

**What was retained:** field names, JSON types, and present / absent / null markers, plus counts
over the returned items. **What was not retained, printed or written anywhere:** any measurement
value, activity id, activity name, location name, coordinate, timestamp of an activity, device
model, or raw payload. The probe scripts live outside the repository (`/tmp`), print only
presence, and hold nothing after they exit.

**Failures:** none. No authentication, permission or rate-limit failure occurred, so no stop was
forced.

---

## 2. What the specification says about temperature

The specification has temperature in exactly **three schemas** and nowhere else:

| schema | field | type | the specification's own description |
|---|---|---|---|
| `SummaryDTO` (activity summary) | `averageTemperature` | `number` (double) | "The average temperature for the activity, in degrees Celsius" |
| `SummaryDTO` | `maxTemperature` | `number` (double) | "The maximum temperature for the activity, in degrees Celsius" |
| `SummaryDTO` | `minTemperature` | `number` (double) | "The minimum temperature for the activity, in degrees Celsius" |
| `SplitSummaryDTO` (per split/lap) | `averageTemperature` | `number` (double) | "The average temperature for the split, in degrees Celsius" |
| `SplitSummaryDTO` | `maxTemperature` | `number` (double) | "The maximum temperature for the split, in degrees Celsius" |
| `SplitSummaryDTO` | `minTemperature` | `number` (double) | "The minimum temperature for the split, in degrees Celsius" |
| `SummarizedActivityDTO` (search summary) | `maxTemperature` | `number` (double) | "The maximum temperature for the activity, in degrees Celsius" |
| `SummarizedActivityDTO` | `minTemperature` | `number` (double) | "The minimum temperature for the activity, in degrees Celsius" |

Two things the specification says nowhere, and which matter:

* **There is no `averageTemperature` on the search summary.** The search endpoint gives the
  minimum and maximum only; the average is on the detail endpoint (and per split).
* **There is no weather.** Across all 112 schemas, the only environment-shaped fields are the
  eight temperature fields above, plus `MetadataDTO.hasRunPowerWindData` and
  `MetadataDTO.runPowerWindDataEnabled` — and those are **flags, not readings**: they say whether
  wind data was used to *compute running power*, and are documented as null when wind is not
  enabled on the device. **No humidity, no wind speed or direction, no "feels like", no air
  pressure, no UV, no precipitation, no daily weather forecast** exists anywhere in this API.
  The daily summary's `averageMonitoringEnvironmentAltitude` is not weather either — its own
  description says it is the day's "Pulse Ox Acclimation".

**Where the value comes from: the specification does not say.** It states the unit (degrees
Celsius) and nothing else — not the device, not a weather station, not a service estimate. The
shape of the schema is evidence: the temperature sits among the device's own measurements
(heart rate, cadence, elevation, respiration) rather than in a weather block, and the API carries
no weather to attach it to. The live read is consistent with the **device-thermometer** reading:
the field is absent for exactly the activities whose FIT files carry no temperature. This is a
well-supported inference, not a documented fact, and §7 says so plainly.

---

## 3. What the live read found

### 3.1 Coverage

| read | result |
|---|---|
| search page, 12-month range, `limit=100` → 100 items | **`minTemperature` key present in 0 of 100**; `maxTemperature` in 0 of 100; `averageTemperature` in 0 of 100 |
| 336 split objects across those 100 items | **0 of 336 carry any temperature key** |
| one activity detail → `summaryDTO` | `averageTemperature`, `maxTemperature`, `minTemperature` **all absent** (not null) |
| one activity detail → its 5 split summaries | all three temperature fields **absent** in every split |
| one activity detail → `metadataDTO` | `hasRunPowerWindData`: null; `runPowerWindDataEnabled`: null; `sensors`: null; no temperature-shaped key anywhere in the payload |

**Absence, not zero and not null.** Garmin omits the key entirely when there is no value. That is
the same convention `wellness_observations` already encodes — "a field the source never sent has
no row; a field it sent as null has a row whose value is null" — and it is the right reading of
this read.

### 3.2 What *was* there, in the same payloads

For contrast, the same read found these populated, which is what tells us the absence of
temperature is a fact about the data and not a broken request:

| field | present in the 12-month search page (of 100) |
|---|---|
| `aerobicTrainingEffect` | 100 |
| `anaerobicTrainingEffect` | 100 |
| `aerobicTrainingEffectMessage` | 100 |
| `averageHR`, `maxHR` | 99 |
| `hrTimeInZone_1` … `hrTimeInZone_5` | 99 |
| `elevationGain`, `minElevation` | 100 |
| `avgElevation` | 80 |
| `moderateIntensityMinutes` | 100 |

and, in the one activity's detail (`summaryDTO`): `averageHR`, `maxHR`, `minHR`,
`trainingEffect`, `anaerobicTrainingEffect`, `aerobicTrainingEffectMessage`,
`anaerobicTrainingEffectMessage`, `directWorkoutRpe`, `directWorkoutFeel`,
`moderateIntensityMinutes`, `vigorousIntensityMinutes`, `steps`, `elevationGain`, `elevationLoss`,
`minElevation`, `maxElevation`, `avgElevation`, `strideLength`, `minActivityLapDuration`.

And these were **absent** from both the search page and the detail: `trainingStressScore`,
`activityTrainingLoad`, `vO2MaxValue`, `lactateThresholdBpm`, `lactateThresholdSpeed`,
`intensityFactor`, `trainingEffectLabel`, `perceivedWorkoutEffort`, `avgStress` / `maxStress` /
`startStress` / `endStress`, `differenceBodyBattery`, `recoveryHeartRate`,
`minRespirationRate` / `maxRespirationRate` / `avgRespirationRate`, `flow` / `grit`, `groundContactTime`,
`verticalOscillation`, `targetDuration` / `targetDistance` / `targetAverageSpeed` (no workout
attached), `directWorkoutComplianceScore`.

The pattern is clean and it is the same pattern the local inventory records for other endpoints:
**the API's optional activity fields are populated where the device or the athlete supplied them
and omitted where they did not.** Temperature is one of the omitted ones, for every activity read.

---

## 4. What the activity detail carries that Stride throws away today

Stride's sync calls the search endpoint and keeps a **projection**: `new_activity()` in
`crates/stride-api/src/routes/app.rs` reads `activityName` and `startTimeLocal` from the search
item and takes every other number from the **decoded FIT**. So the API's own summary is discarded
in full — including the split summaries the search response already carries, and including the
detail endpoint, whose client method exists but is never called in normal sync.

The table below is the discard list, ordered by usefulness to the training analysis. **Observation**
means a fact about the activity that somebody measured or recorded. **Judgement** means Garmin's
own estimate — and the contract's rule is that Garmin's training status and readiness stay out of
the core, so the judgement rows are storage candidates, not core candidates. **Self-report** is a
fact the *athlete* entered; it is an observation about the athlete, but with a provenance the
product must not blur with Garmin's.

| field | type | what it would mean | observation / judgement |
|---|---|---|---|
| `averageHR`, `maxHR`, `minHR` | number (bpm) | the session's heart-rate summary from the service, alongside the FIT's own | observation (device-measured) |
| `hrTimeInZone_1` … `_5` | number (s) | time in each heart-rate band, computed by Garmin against **its own** zone model | derived observation — the same thing Stride computes itself, from its own stored model |
| `elevationGain`, `elevationLoss`, `minElevation`, `maxElevation`, `avgElevation` | number (m) | the service's elevation figures, next to Stride's rebased climb from the FIT | observation (device-measured) |
| `isElevationCorrected` / `metadataDTO.elevationCorrected` | boolean | whether the elevation was corrected by the service | observation (about the record) |
| `metadataDTO.isOriginal`, `metadataDTO.trimmed` | boolean | whether the recording is unedited, and whether it was trimmed | observation (provenance) |
| `steps`, `moderateIntensityMinutes`, `vigorousIntensityMinutes` | integer | step count and minutes by Garmin's own moderate/vigorous definition | observation (steps) / derived (the minutes, by Garmin's bands) |
| `directWorkoutRpe`, `directWorkoutFeel` | integer | the effort the athlete entered after the session | **self-report** — present in the detail read |
| `perceivedWorkoutEffort` | integer | the effort recorded against a workout or a split | **self-report** (absent in the read) |
| `directWorkoutComplianceScore` | integer | how far the session matched a planned workout | **self-report** (absent in the read) |
| `averageTemperature`, `maxTemperature`, `minTemperature` | number (°C) | the ambient temperature recorded with the session | observation **if present**; absent on every activity read |
| split-level `averageTemperature` / `maxTemperature` / `minTemperature` | number (°C) | three numbers per lap — a coarse temperature trace | observation if present; absent from every split read |
| `trainingEffect` | number | Garmin's overall training effect for the session | **judgement** |
| `aerobicTrainingEffect`, `anaerobicTrainingEffect` | number | Garmin's aerobic and anaerobic training effect | **judgement** |
| `aerobicTrainingEffectMessage`, `anaerobicTrainingEffectMessage` | string | Garmin's own classification ("improving", "maintaining", …) | **judgement** — a label with a verdict in it |
| `trainingEffectLabel` | string | the same classification as a label | **judgement** (absent in the read) |
| `trainingStressScore` | number | Garmin's scalar training load | **judgement** (absent in the read) |
| `activityTrainingLoad` | number | Garmin's newer training-load scalar | **judgement** (absent in the read) |
| `intensityFactor` | number (%) | the session's intensity relative to threshold | **judgement** (absent in the read) |
| `vO2MaxValue` | number | Garmin's VO₂ max estimate for the athlete at that moment | **judgement** (absent in the read) |
| `lactateThresholdBpm` / `lactateThresholdHeartRate`, `getlactateThresholdSpeed` / `lactateThresholdSpeed` | number | Garmin's threshold estimates | **judgement** (absent in the read) |
| `recoveryHeartRate` | integer (bpm) | Garmin's post-session recovery figure | **judgement** (absent in the read) |
| `flow`, `grit`, `avgFlow`, `avgGrit` | number | Garmin's proprietary run-quality indices | **judgement** (absent in the read) |
| `avgStress`, `startStress`, `endStress`, `differenceStress`, `maxStress` | number | the session's stress index | **judgement** (absent in the read) |
| `differenceBodyBattery` | integer | the body-battery change across the session | **judgement** (absent in the read) |
| `avgRespirationRate`, `minRespirationRate`, `maxRespirationRate` | number (breaths/min) | the session's respiration | observation (device-measured; absent in the read) |
| `metadataDTO.hasRunPowerWindData`, `metadataDTO.runPowerWindDataEnabled` | boolean | **flags only** — whether wind was used for running power | observation (a flag; null in the read). **Not a wind reading.** |
| `metadataDTO.sensors`, `metadataDTO.calendarEventInfo` | object / object | attached devices and calendar context | observation (null in the read) |
| `/activity-service/activity/{activityId}/workouts` | array of `ActivityWorkoutDTO` | the workout steps performed during the session | observation (the intent behind the session) |

**The three findings that matter for the analysis.**

1. **Aerobic and anaerobic training effect are present and populated** (100 of 100 activities in
   the 12-month page), and so are the time-in-zone figures. They are Garmin judgements and stay
   out of the core — but they are *available* as attributed observations, and they are the
   nearest thing the API offers to "how hard was this, by Garmin's reckoning".
2. **The athlete's own effort is present in the detail** (`directWorkoutRpe`, `directWorkoutFeel`)
   and absent in the search summary. It is the one effort figure that is *not* Garmin's estimate.
   It lives on the detail endpoint, so it costs one call per activity.
3. **VO₂ max, training load, training stress score and lactate threshold are absent** for these
   activities — they are in the specification but the service did not supply them here. They
   cannot be promised on the strength of a schema alone; the same rule the local inventory states
   for the "later: device-conditional detail" row.

---

## 5. What a temperature would cost

### 5.1 Where it would live

The temperature is a **per-activity** fact. The store's existing observation table,
`wellness_observations`, is a **per-day** table keyed `(date, source, field, ordinal)` with no
activity reference — a day can hold several activities, so it cannot hold an activity's own
temperature without a second key. Three options, in order of preference:

**(a) A new table beside `activities` — recommended.**

```sql
-- schema version 5 -> 6
CREATE TABLE activity_observations (
    id           INTEGER NOT NULL PRIMARY KEY,
    activity_id  INTEGER NOT NULL,
    source       VARCHAR(24) NOT NULL,        -- 'garmin_personal'
    endpoint     VARCHAR(96) NOT NULL,        -- the API path the value was read from
    field        VARCHAR(48) NOT NULL,        -- the source's own field name
    ordinal      INTEGER NOT NULL DEFAULT 0,  -- which item of a repeated field
    value        REAL,                        -- the number, as reported; NULL when the source sent null
    text_value   TEXT,                        -- a value that is not a number
    unit         VARCHAR(16),                 -- 'C' for degrees Celsius
    observed_at  DATETIME,                    -- the instant the item is about
    retrieved_at DATETIME NOT NULL,           -- when the value was read, ISO-8601 UTC
    FOREIGN KEY (activity_id) REFERENCES activities (id) ON DELETE CASCADE,
    CONSTRAINT uq_activity_observations UNIQUE (activity_id, source, field, ordinal)
);
CREATE INDEX ix_activity_observations_activity ON activity_observations (activity_id);
```

This is the same shape as `wellness_observations`, at the activity grain, and it takes the
training-effect and effort rows with it without a second mechanism. The three temperature numbers
are three rows with the source's own field names and the unit `C`.

**(b) Three nullable columns on `activities`** (`temperature_min_c`, `temperature_max_c`,
`temperature_average_c`, all `REAL`, `NULL` = no value). Cheapest of all — no new table, no
migration of existing rows — but it loses provenance (which endpoint, read when) and it does not
extend to the training-effect fields, which are the more interesting half of §4.

**(c) Reuse `wellness_observations` as it stands.** Not possible without a schema change: there
is no activity reference in it, and a per-day key cannot express "this run's temperature".

### 5.2 The backfill (twelve months)

* **The cheap route is the search endpoint.** The search summary already carries
  `minTemperature` and `maxTemperature`; one page returns up to 100 activities and Stride's
  transport already caps a traversal at 300. Twelve months of this athlete's activity count came
  back on **one** page, so the whole year's minimum/maximum pair is a handful of requests, not
  one call per activity.
* **The average costs one detail call per activity** — the search summary has no
  `averageTemperature`. For ~100 activities that is ~100 requests, which is a real budget item and
  would have to be rate-limited, resumable and dated (the shape the local inventory says the sync
  still lacks).
* **What the backfill would fetch today: nothing.** The 12-month read found no temperature key on
  any of 100 activities. A backfill run against this account would write **zero** temperature
  rows and would still cost the requests.
* **What it would fetch on a device with a thermometer: unknown.** No sample of such a device was
  available, so the coverage of the field *where it exists* is unverified (§7).

### 5.3 How absence must be handled

The store's rule already covers this, and the temperature must obey it exactly:

* **A missing temperature is `null`, never `0`.** Zero degrees is a temperature; a field the
  service never sent is not a cold day.
* **Never substituted.** Not with the day's average, not with the window's mean, not with the
  previous activity's value, not with a weather station, not with a season. A substituted number
  cannot say where its input came from, and the product's whole posture is that a number is
  reported with its source.
* **The absence keeps its reason.** The run detail says why, in the catalogue's own shape — e.g.
  `condition.no_temperature`: "temperature unavailable — the device recorded no temperature for
  this run". The FIT-side note's `condition.no_temperature` is the key to copy; the value is
  absent, the reason is present.
* **The two absences stay distinct.** "The service sent no field" → no row (or a row with a null
  value, depending on the shape chosen). "The service sent null" → a row with a null value. The
  observation table's uniqueness constraint and its provenance columns are what keep them apart.
* **Coverage is a fact and is stated.** "N of M runs in the window carry a temperature reading" is
  a true sentence; a window-average temperature over runs that carry none is not.
* **Per activity, not per window.** The temperature belongs to one run. A window's temperature is
  a different, weaker thing, and is only computed over the runs that carry a reading.

### 5.4 The other cost

The detail endpoint would have to become part of the sync (it is a client method today, never
called), which brings the request budget with it: one detail call per activity is the price of
the athlete's own effort ratings, the average temperature and the training-effect block — all of
which the search summary does not carry. That budget is a sync-design decision, not a schema
decision, and it belongs with the "resumable dated work" gap the local inventory already names.

---

## 6. The product question, answered honestly

The user has clarified that "running too hot or too cold" means **exertion and load, not weather**.
The environment axis is therefore out of scope, and this note does not propose it: no heat load,
no weather comparison between windows, no "the heat cost you 4 %".

**What a per-activity temperature would still be good for:**

* **The run detail's context.** One line on one screen, on the run that has it: the temperature
  the device recorded with the session, named as what it is — a reading, with its unit, on the
  day. It sits beside the other facts of the session (distance, time, elevation, heart rate) as
  one more thing that was true that morning.
* **Explaining a slow session, as context and not as cause.** "This session was slower than your
  usual at the same heart rate" is a fact. "The device recorded a temperature that morning" is
  another fact. Putting them on the same screen lets the athlete draw the connection themselves —
  which is exactly the product's posture: it does not draw conclusions. The sentence must be a fact
  and stay a fact: the screen may carry both numbers; it may not say the second one caused the
  first.
* **The run update's facts.** If the update's model is given the temperature as one of the
  activity's measured facts, it may cite it — but only as a fact it was given, and the update's
  payload-hash check (the `run_updates` table's `payload_hash`) must cover it, so the sentence
  stays re-checkable.

**What a temperature must never be turned into:**

* **no heat-adjusted pace** — no pace adjusted for temperature, in any direction;
* **no adjustment of any measurement** — not pace, not heart rate, not training effect, not
  calories, not the zone split; a context field does not become an input to a model;
* **no "the heat cost you 4 %" claim**, and no equivalent — a percentage attributed to the
  environment is a model wearing a measurement's clothes, and the product does not publish models
  whose assumptions are invisible at the point of the number;
* **no verdict and no readiness** — a temperature is not a status, not a score, not a colour, and
  it may never sit next to one in a way that implies a conclusion;
* **no weather claim at all** — the API carries no humidity, no wind, no "feels like"; any
  sentence that implies weather the API did not supply is unsourced.

**The honest bottom line.** A per-activity temperature is **a nice-to-have fact on one screen,
not an analysis**. And on this athlete's account it is not even that: the field is absent from
every activity read, so the screen would show nothing but the reason. It is worth keeping in the
inventory — the schema exists, the fields are documented, another device may populate them — and
it is **not** worth a sync budget, a schema migration or a screen on its own. If it comes, it
comes as a column on the run detail, with a null and a reason beside it, and as nothing else.

---

## 7. What could not be verified

* **The source of the temperature value.** The specification states the unit and nothing else.
  "Device thermometer" is an inference from the schema's shape and from the correlation with the
  FIT corpus, not a documented fact. A device with a thermometer would settle it.
* **Whether the temperature Garmin Connect shows is the same quantity.** The user saw temperature
  on a run in Connect. The Personal API's temperature is absent for those runs, and the API has
  no weather fields at all — so Connect's display most likely comes from a source this API does
  not expose (Connect's own weather service), but **this was not verified**: the Connect web
  surface was not read, and no such value may be assumed to exist in the API.
* **Coverage where the field does exist.** No activity with a temperature was observed, so the
  field's population rate on a thermometer-equipped device is unknown. The 0-of-100 figure is
  **this athlete's** coverage, not the field's.
* **The judgement fields' population on other activity types.** `trainingStressScore`,
  `activityTrainingLoad`, `vO2MaxValue`, `lactateThresholdBpm`, `intensityFactor`,
  `perceivedWorkoutEffort` and the stress fields were absent for the activities read; whether
  they appear for cycling, swimming or indoor activities was not probed.
* **The `/workouts` endpoint.** Documented, not read in this session.
* **Rate limiting, pagination beyond one page, and retention beyond twelve months.** Not
  exercised.
* **The live key set was read once, on 24 November 2026.** The specification is version 1.0.0
  with 28 paths; a future change to the schema would change this note.

---

## 8. One-line inventory update

`docs/garmin-api-capabilities.md` may add, to the row for
`/activity-service/activity/{activityId}`: *"the detailed summary carries three temperature
fields (`averageTemperature`, `maxTemperature`, `minTemperature`, degrees Celsius) and the search
summary carries two of them (`minTemperature`, `maxTemperature`); a live 12-month read found the
keys **absent** from all 100 activities and from all 336 split objects, and the API carries no
humidity, wind or other weather field at all — the field is device-conditional and the environment
axis stays out of scope."* The row for
`/activitylist-service/activities/search/activities` may add: *"the search summary also carries
the aerobic and anaerobic training effect (100 of 100 activities in the 12-month read) and the
time in heart-rate zones — Garmin's judgements and Garmin's own zone model, kept out of the
core."*

That is a change to a file this note is not allowed to make; it is offered here for the parent to
apply or decline.
