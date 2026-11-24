# Can the zone model's history be backfilled?

*Research and proposal, 24 November 2026. No implementation. Nothing under
`crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/` was
touched; no commit was made.*

---

## 0. The answer in one line

**No.** The Personal API exposes the athlete's heart-rate zone configuration
**as it stands now** — one endpoint, one query parameter, no date, no range,
no history. There is nothing to backfill: the model is not dated by Garmin at
all. *But* the API **does** carry the source's own per-run time in zone
(`hrTimeInZone_1..5`) on the activity payload Stride already fetches, for
essentially the whole imported history, at **zero extra requests**. That is
the answer that makes Z's rule honest across the imported history.

Evidence: `documented` (official specification, 28 paths) + `live response`
(bounded read-only probes; 4 token exchanges, 11 data requests; no raw
response retained).

---

## 1. Method and evidence

### 1.1 The specification (primary evidence)

The official Personal API specification was read from Garmin's own Swagger
page, `https://developer.garmin.com/gc-developer-program/personal/swagger-docs/`
(note: the path in the brief, `gc-develop-program`, is a typo — the live path
is `gc-developer-program`; the same correction is already recorded in
`exploration/garmin-temperature-and-activity-fields.md`). The page is a Gatsby
bundle that embeds the OpenAPI document as a JSON object; the document was
extracted and parsed in full.

* **OpenAPI 3.0.1**, title *Garmin Connect Personal API*, version **1.0.0**.
* **28 paths** (the local inventory `docs/garmin-api-capabilities.md` records
  26 data endpoints, so the spec has moved; see §9 for the two paths that are
  new to the inventory — **neither is a zone endpoint**).
* Servers: `https://personal.connectapi.garmin.com`, `…garmin.cn`.

`documented`.

### 1.2 The live reads (bounded, read-only, sequential)

The specification settled the endpoint question, but it does not describe the
athlete's own data (item 4 of the brief), and a specification cannot prove
that the *response* omits a date the schema does not declare. Two bounded
live reads were therefore made, through `cf-garmin env -- <command>` (Key
Vault credential, in process, never printed, never written).

| # | Request | Result |
|---|---|---|
| 1 | `POST /personalapitoken-service/personal-api/personal-api-key/token` | 200; access value used in-process only |
| 2 | `GET /biometric-service/heartRateZones/v2?generateDefaults=false` | 200 |
| 3–4 | `GET /activitylist-service/activities/search/activities` (12-month window, `start=0` and `start=100`) | 200 each |
| 5 | `GET /activity-service/activity/{id}` | 200 |
| 6 | token exchange (probe 2) | 200 |
| 7–10 | activity search, 12-month window (2 pages) and 36-month window (2 pages) | 200 each |
| 11 | token exchange (probe 3) | 200 |
| 12–15 | activity search, 12-month window with and without `activityType=running` | 200 each |

Fifteen HTTP requests in total, four of them key exchanges; eleven data
requests; every one a GET except the token POST. **No authentication,
permission or rate-limit failure occurred**, so no stop was forced.

**What was retained:** field names, JSON types, presence / absent / null
markers, counts, and coarse month buckets. **What was not retained, printed
or written anywhere:** any measurement value (no zone floor, no maximum heart
rate, no zone second, no duration, no distance), any activity id, any
activity name, any coordinate, any device model, any activity timestamp
beyond a month bucket, and any credential. The probe scripts live outside the
repository (`/tmp`), print only names and counts, and hold nothing after they
exit.

**Not done:** Stride's sync was not run; the local database was not written
to (a read-only open of the rig database failed — it is SQLCipher-encrypted
and no SQLCipher binding is installed — and no package was installed to
change that); no account setting was changed; nothing was published.

`live response`.

### 1.3 Third-party corroboration

Garmin's own published reference client lists exactly one zone wrapper:
`get_heart_rate_zones()` → `GET /biometric-service/heartRateZones/v2`, with
**no date argument** (`garmin/connect-personal-api-python`, README
"Endpoint wrappers" table). `documented` (Garmin-published, not the spec).

---

## 2. Does the API expose the zone model's history? — No.

### 2.1 The one zone endpoint

```
GET /biometric-service/heartRateZones/v2
query: generateDefaults : boolean (default true)
200  : [ HeartRateZonesDTO ]
```

* **One path.** It is the only path in the 28 that concerns zone
  *configuration*. There is no `v1`, no `/history`, no `/range`, no
  `/{calendarDate}` variant, and no second zone endpoint.
* **One parameter, and it is not a date.** `generateDefaults` decides whether
  Garmin may invent zones for an unconfigured account. Stride already sends
  `generateDefaults=false` (`crates/stride-ingest/src/garmin.rs`,
  `heart_rate_zones()`), which is right: an invented model must not be passed
  off as the athlete's.
* **The response is an array, one entry per sport.** Garmin's enum is
  `DEFAULT | RUNNING | CYCLING | SWIMMING | ROWING`; Stride's
  `zone_model_of()` picks the `DEFAULT` entry (`crates/stride-api/src/routes/app.rs`).

### 2.2 What each entry carries

`HeartRateZonesDTO` — eleven fields, and **not one of them is a date**:

| field | type | meaning |
|---|---|---|
| `sport` | string (enum) | the sport the model is configured for |
| `trainingMethod` | string (enum) | `HR_MAX | LACTATE_THRESHOLD | HR_RESERVE | CUSTOM` |
| `zone1Floor` … `zone5Floor` | integer | the five lower boundaries, bpm |
| `maxHeartRateUsed` | integer | the maximum heart rate the model was built on |
| `restingHeartRateUsed` | integer | resting HR used in the calculation |
| `lactateThresholdHeartRateUsed` | integer | lactate threshold used |
| `restingHrAutoUpdateUsed` | boolean | whether resting HR auto-updates |

The live read confirms the schema exactly: **one** entry, `sport=DEFAULT`,
`trainingMethod=HR_MAX`, eleven fields with those names and types, and **no
field whose name denotes a date, an effective-from, or a history**. (The only
name that superficially looks date-ish is `restingHrAutoUpdateUsed`, a
boolean.) `live response`.

**Therefore: the zone model has no effective date of its own.** The only date
Stride can ever attach to it is the day it was *read* — which is exactly what
`hr_zone_models.read_at` is. `documented` + `live response`.

### 2.3 The consequence, stated precisely

`hr_zone_models` is a history of **reads**, and `write_hr_zone_model_if_new`
writes a row only when the model *differs* from the stored one. So the table
holds **one row per configuration change, dated by the day Stride noticed
it**. Its oldest row is the athlete's first sync. "The model in force on the
day of the run" can therefore only ever be true for runs on or after that
first sync — and the imported history is all *before* it. `stored` (code
reading) + `inferred` (the rule that follows).

A second, smaller honesty caveat follows from the same fact: a change made
between two syncs is dated by the **later** read, so a run made under the new
configuration is measured against the previous model until the next sync
observes the change. The API cannot date a change (there is no effective-from
field), so this cannot be fixed from the source — only **said**, and the
answer already names the model and the day it was read.

### 2.4 Everything else in the spec that touches "zones"

| path | what it is | zone history? |
|---|---|---|
| `/biometric-service/heartRateZones/v2` | the athlete's zone **configuration** | no — current state only |
| `/wellnessactivity-service/activity/summary/range/{fromDate}/{untilDate}` | per-activity wellness summaries with `timeInZoneList` (`zoneNumber`, `millisInZone`, `zoneLowBoundary`) — a **per-activity** witness, not a configuration history | no. The local inventory records it as *live 200, empty result* for this athlete (2026-11-04) |
| `/activitylist-service/activities/search/activities` → `SummarizedActivityDTO` | `hrTimeInZone_1..5`, `powerTimeInZone_1..7` | no — per-activity zone **times**, §6 |
| `/activity-service/activity/{id}` → `SummaryDTO` | no zone fields at all | no |

---

## 3. If not: the three options, their costs, and a recommendation

The brief's three options, with what each costs against this athlete's real
data (§4).

### Option A — the current rule: no model in force, no zone times

* **What it says.** Nothing false. A run before the first stored model is
  uncalibrated, and the answer says "no model in force".
* **What it costs.** The ledger's intensity rows are empty for the whole
  imported history — the wedge's own point. For this athlete that is **107 of
  107 imported runs (100%)** if the first model is read today, or **~98 of
  107 (~92%)** if a model already dated 3 Nov 2026 exists (the brief's own
  example). The product looks broken on the first launch, on the runs the
  athlete most wants to see.
* **What it does not cost.** No requests, no storage, no schema change, no
  new labels. It is the cheapest option and it is already implemented.

### Option B — measure earlier runs against the earliest stored model, and say so

* **What it says.** "Measured against the first model read on 3 Nov 2026."
  Honest about the *provenance*; not honest about the *configuration* — a
  model read in November 2026 is applied to runs from November 2025, which is
  a different watch configuration from any that existed on those days (the
  athlete's account history spans ~20 months, §4).
* **What it costs.** No requests (the model is already stored), no schema
  change. The cost is **interpretive**: the zone bands shown for a 2025 run
  would be 2026's bands, and the athlete's own Garmin record for that run
  would disagree with them (it was computed under the zones in force then —
  §6). A number that is provably not the configuration of the day is a number
  that will be wrong in the athlete's eyes, however well it is labelled.
* **The failure mode.** It reads as "the app knows my zones" and then
  disagrees with Garmin about the same run. That is worse than saying nothing.

### Option C — withhold the zone-time answers for the pre-model period

* **What it says.** "This history is outside the zone model's reach."
* **What it costs.** The same emptiness as Option A (**107 of 107** runs, or
  ~98 of 107), plus a sentence in the product that explains a hole. It is
  Option A with better manners and no more data.
* **When it is right.** It is the right answer *for a run with no model and
  no other witness*. It is the wrong answer as a **policy**, because a second
  witness exists (§6) and is present on 106 of the athlete's 107 imported
  runs.

### Recommendation

**Keep Z's rule; do not bend it (not B). Do not withhold what the source
already told us (not C). Extend the rule with the source's own witness, and
withhold only where nothing survives.**

The smallest honest form of that is a **fourth path**, which the brief
anticipated in its last section: for a run whose day has **no model in
force**, use the source's own `hrTimeInZone_1..5` — the zone time Garmin
itself recorded for that run — labelled as the source's record, with the
boundaries of that day marked as unknown to Stride. The rule Z states ("every
run is measured with the zone model in force on the day of the run") stays
true for every run where a model is in force; for the rest, the answer stops
claiming a model and starts quoting the source. Where neither exists (one run
in this athlete's import, which recorded no heart rate), the answer says no
value, exactly as today. Cost: **zero extra requests**, one small storage
change, one label. It fills 106 of 107 runs with the source's own number and
leaves one run honestly empty.

Options A, B and C remain the fallback if the parent rejects the fourth path:
then **A** (with C's wording) is the choice — it is the only one that never
states a model that did not exist on the run's day, and it is already built.

---

## 4. What the athlete's own data looks like

Live, over the athlete's real account (counts only):

| window | activities | runs (`activityType=running`) | with all five `hrTimeInZone_*` | with none |
|---|---|---|---|---|
| 12 months (2025-11-24 → 2026-11-24) | **107** | **107** (the same set) | **106** | **1** |
| 36 months (2023-11-24 → 2026-11-24) | 175 | — | 174 | 1 |

* Every activity in the 12-month window is a run; the `activityType=running`
  filter returns the same 107. `live response`.
* **The account's whole history is ~20 months old** (the oldest activity in the
  36-month window sits in the ~20-month bucket), so a 12-month import covers
  just over half of the athlete's history, and the source's zone times reach
  all the way back to the beginning of it. `live response` + `inferred` (from
  the bucket).
* Month buckets of the 12-month window (months before today → runs):
  `11: 14 · 10: 11 · 9: 11 · 8: 11 · 7: 4 · 6: 2 · 5: 4 · 4: 6 · 3: 8 ·
  2: 11 · 1: 11 · 0: 14`. **Runs older than one month: 93.** All 93 carry
  the source's own zone time. `live response` (buckets) + `inferred`
  (bucket → date arithmetic).

### Runs affected, per option

| option | runs in the 12-month import with **no** zone time | share |
|---|---|---|
| A (today's rule, first model read today) | **107 of 107** | 100% |
| A (model already dated 3 Nov 2026, per the brief's example) | **~98 of 107** (the 93 runs older than a month, plus ~5 in the first days of November) | ~92% |
| B (earliest stored model) | **0** — but every one of them is measured against a model read up to ~2 years after the run | 0% empty, 100% borrowed |
| C (withhold) | **107 of 107** | 100% |
| **D (source's own witness)** | **1 of 107** (the single run with no heart rate recorded) | 0.9% |

---

## 5. What a backfill would cost

### 5.1 Backfilling the *model* — there is nothing to fetch

* **Requests: 0.** The endpoint has no date parameter and no history; a
  "backfill" of the configuration would be the same one request, repeated,
  hoping to catch a change the source cannot date anyway. Stride already does
  that: **one zone read per pull** (`crates/stride-api/src/routes/app.rs`,
  the zone block after the wellness loop), and it writes a row only when the
  model changed.
* **Storage: `hr_zone_models` already holds dated models** (schema v3:
  `sport, training_method, max_heart_rate, zone1_floor … zone5_floor,
  read_at`). Nothing needs to change to hold more of them — but nothing more
  will ever arrive to fill it, because the source has only one current
  configuration.
* **What the sync would change: nothing.** The zone read is one request out
  of a deliberately small budget.

### 5.2 The sync's own budget (the numbers that matter)

`crates/stride-api/src/state.rs`, `Config::default()`:

| bound | value | what it is for |
|---|---|---|
| `max_sync_requests` | **80** | HTTP attempts per job, **including the key exchange and redirects** |
| `max_activities_per_sync` | **20** | FIT downloads per pull — the budget is spent on *runs*, not on list requests |
| `max_wellness_days_per_sync` | 14 | wellness days per pull (≈3 requests each) |
| `max_sync_seconds` | 45 | soft elapsed limit |
| `max_sync_response_bytes` | 32 MiB | response cap |
| `sync_days` | 365 | how far a pull reaches by default |
| `history_months` | 12 | the window the settings say the app keeps |
| `SYNC_PAGE_SIZE` / `SYNC_MAX_ACTIVITIES` | 100 / 300 | search page size and traversal cap (`routes/app.rs`) |

A steady-state pull therefore costs roughly: 1 key exchange + 1–3 search pages
+ up to 20 FIT downloads + 14 × 3 wellness reads + 1 stress + 1 body battery
+ **1 zone read ≈ 70 requests against a ceiling of 80**.

Two consequences:

* **The import takes several pulls.** 107 runs ÷ 20 per pull ≈ **6 pulls** for
  the 12-month window. The zone model is read once per pull — 6 requests
  across the whole import — and, if the configuration does not change, writes
  **one** row.
* **The budget has ~10 requests of slack, not 20.** A naive alternative —
  "fetch each activity's detail to get its zone time" — would add one request
  per imported run: **+20 per pull**, i.e. ~90 requests against a ceiling of
  80. It would break the sync. **The proposal in §7 must not add a per-run
  request**, and it does not: the zone times ride on the search response that
  is already fetched.

---

## 6. `hrTimeInZone_1..5` — the second, independent witness

### 6.1 Where it actually lives (and where it does not)

The brief says the **activity detail** response carries `hrTimeInZone_1..5`.
The specification says otherwise, and the live read confirms it:

* **`SummarizedActivityDTO`** (the **activity search / list** payload) carries
  `hrTimeInZone_1` … `hrTimeInZone_5` — "the number of seconds that the
  user's heart rate was in zone N" — and `powerTimeInZone_1` … `_7`.
  `documented`.
* **`SummaryDTO`** (the **activity detail** payload, `/activity-service/activity/{id}`)
  has **no** time-in-zone field at all: the live detail read was searched
  recursively for any `*TimeInZone*` key and found **NONE**, and the spec's
  188-property `SummaryDTO` agrees. `live response` + `documented`.

This matters, and it is good news: **Stride's sync already fetches the
search payload** — the same pages it walks to discover runs. The zone times
are in the response Stride is already reading.

### 6.2 What it can do

* **It fills the imported history.** 106 of 107 runs in the 12-month window
  carry all five zone times; 174 of 175 over 36 months; the oldest activity
  in the account carries them too. `live response`.
* **It is coherent, not a placeholder.** For all 106 runs, the five zone
  seconds sum to the activity's duration within 2%; none sums to zero. So the
  source's zone times account for the whole activity. `live response`.
* **It costs zero extra requests** — it rides on the search response the sync
  already fetches (§5.2). `inferred` from the sync's own code.
* **It is independent of any model Stride holds.** It is the watch's own
  record, made at the time of the run, under whatever configuration was in
  force then. It does not need Stride to know that configuration to be right.

### 6.3 What it cannot do

* **It does not give the zone boundaries.** It gives seconds per zone; it
  does not say what bpm the edges were. For a run with no model in force,
  Stride can say "28 minutes in zone 3" but cannot say "zone 3 was
  160–175 bpm". The model's *floors* remain unknown for the pre-model
  period, and the band's edges must be rendered as unknown, not borrowed.
* **It does not agree with Stride's basis by construction.** Stride's
  `time_in_zone` (crates/stride-core/src/zones.rs) charges a sample's span
  to the next sample and **skips stopped stretches** — its zones sum to the
  run's *moving* time, with `outside_s` and `stopped_s` beside them. The
  source's zone times sum to the activity's **elapsed** time. For a run with
  pauses the two are different measurements, not two estimates of one thing.
  The answer must therefore say which basis it is on; the two must not be
  summed or charted as if they were one.
* **It is absent when the run recorded no heart rate** — 1 of 107 runs here.
  For that run there is no zone time of any kind, and no option invents one.
* **It does not tell Stride which configuration produced it.** The payload
  carries no model, no floors, no sport-to-model link. So it cannot *prove*
  agreement with a stored model; it can only be compared with one.
* **It gives totals, not a distribution.** There is no per-split zone time in
  either payload, so the source cannot supply a zone curve through the run —
  only the shape of "time in zone" per run.
* **It is only available at list time.** Because the detail payload lacks it,
  the zone seconds must be captured from the **search** response. A run
  imported from a FIT file rather than from a search listing would have no
  source zone time — another reason the capture has to happen in the sync's
  listing loop, where the entry is already in hand.

### 6.4 A useful cross-check, not yet done

For a run whose day *does* have a model in force, Stride's computed zone
times and the source's reported zone times are two answers to the same
question, and their disagreement (beyond the stopped-time difference) would
be evidence of a mis-read model. That comparison needs FIT samples and the
stored model — i.e. the local database — and it was **not** performed here
(the rig database is SQLCipher-encrypted and no SQLCipher binding is
installed; no package was installed). It is the natural regression test for
the proposal in §7, and it is listed under §9.

---

## 7. Proposal (not code): the smallest change that makes Z's rule honest

**The shape of it, in one sentence: store the source's own zone time with the
run, and let the zone answer fall back to it — clearly labelled — when no
model is in force.**

Four parts, smallest first.

### 7.1 Capture (ingestion) — no new request

In the sync's activity-listing loop (`crates/stride-api/src/routes/app.rs`,
the `for entry in &list` arm, and the `new_activity(...)` mapper), read
`hrTimeInZone_1` … `hrTimeInZone_5` from the **search entry that is already
in hand** and carry them into the stored run. Absent or null stays absent —
never zero, never band 1.

### 7.2 Storage

Two shapes; the second is the better fit for this repository.

* **(a) five nullable REAL columns on `activities`.** Simple, but it mixes the
  run's own facts with a source's account of the run, and it has no place to
  say *which* source or *which* basis.
* **(b) a new table `activity_zone_times`** —
  `activity_id, zone1_s … zone5_s, source, basis, read_at`, with
  `basis = 'garmin_elapsed'` (or `'garmin_reported'`) written explicitly.
  It mirrors `wellness_observations`' shape: **one row per (run, source,
  field)** with its provenance, `NULL` where the source sent nothing, and a
  reading time. It keeps "no value" and "zero" apart, and it leaves room for
  a second source later.

Either way the cost is the same set of chores, and they are the repository's
normal ones:

1. a **schema version bump** (currently 5 → 6) in
   `crates/stride-store/src/schema.rs`;
2. the matching edit to **`docs/contract/schema.sql`** — the DDL there is a
   verbatim copy and `crates/stride-store/tests/schema_contract.rs` compares
   the two, so one without the other fails the build (a golden assertion);
3. a row in the **export table list** (`crates/stride-store/src/export.rs`
   already lists `hr_zone_models` and `wellness_observations`), so the
   backup carries the new table and the manifest counts it;
4. **no change to the sync's request budget** (§5.2).

### 7.3 The answer (the seam)

In `crates/stride-api/src/zonemodel.rs` — the one reader — extend the
"no model in force" branch:

* **`ModelFor::Model`** (today): unchanged. The run is measured against the
  model in force on its day, computed from the FIT samples under that model.
  Z's rule stands.
* **`ModelFor::NoneInForce`, and the run has the source's zone time:** answer
  with the **source's** zone times, under a **different basis label** ("the
  source's own record of this run", over the elapsed time), with the zone
  **edges rendered as unknown** rather than borrowed from another day. The
  reason id becomes something like `zones.no_model_in_force` → *reported*,
  not *no value*.
* **`ModelFor::NoneInForce`, and the run has none** (one run in this
  athlete's import): today's answer — no value, with the reason. Option A,
  honestly.

Two rules that must survive the change:

* **Never mix the two bases.** Stride's computed zones sum to *moving* time;
  the source's sum to *elapsed* time. The answer states which one it is
  showing. They are not summed, averaged or charted as one series.
* **Never present the source's zones as a model.** The boundaries are
  unknown for those days, and the label says so.

### 7.4 One small extra, for the same request budget

`zone_model_of()` currently keeps **only** the `DEFAULT` entry and drops the
rest of the array. The endpoint returns one entry **per sport** in a single
request (`DEFAULT | RUNNING | CYCLING | SWIMMING | ROWING`); this athlete's
live payload carried exactly one (`DEFAULT`). Storing every sport's model
costs **no extra request** and lets a run be measured against *its* sport's
model when the athlete configures per-sport zones. The table already has a
`sport` column; the change is in the mapper and the "in force" lookup.

### 7.5 What this proposal does *not* do

* It does not bend Z's rule. No run is ever measured against a model that
  did not exist on its day.
* It does not invent a model for the pre-model period.
* It does not widen the sync's request budget, and it does not add a request
  per run.
* It does not change `PLAN.md`, the API contract, or the zone-time
  computation itself.

---

## 8. Evidence ledger

| claim | evidence level |
|---|---|
| The specification has 28 paths; only one concerns zone configuration | `documented` |
| `GET /biometric-service/heartRateZones/v2` takes one query parameter, `generateDefaults` (boolean, default true) | `documented` |
| `HeartRateZonesDTO` carries sport, training method, five floors, max HR, resting HR, lactate threshold, auto-update flag — **no date** | `documented` |
| The live response is one entry (`DEFAULT`, `HR_MAX`) with exactly those eleven fields and no date-like field | `live response` |
| Garmin's own reference client exposes one zone wrapper, with no date argument | `documented` |
| `hrTimeInZone_1..5` live on `SummarizedActivityDTO` (search), not on `SummaryDTO` (detail) | `documented` + `live response` |
| 107 activities/runs in the 12-month window; 106 carry all five zone times; 1 carries none | `live response` |
| The five zone seconds sum to the activity duration within 2% for all 106; none is zero | `live response` |
| Over 36 months: 175 activities, 174 with zone times; the account history is ~20 months | `live response` + `inferred` |
| Runs older than one month: 93 (month buckets) | `live response` |
| `hr_zone_models` holds one row per configuration **change**, dated by the read | `stored` (code reading) |
| A pull costs ≈70 of its 80-request budget; 20 FIT downloads per pull; 107 runs ≈ 6 pulls | `inferred` from `state.rs` + `routes/app.rs` |
| A per-run detail fetch to obtain zone times would add ~20 requests and break the 80-request ceiling | `inferred` |
| The source's zone times sum to *elapsed* time; Stride's computed zones sum to *moving* time | `inferred` from `crates/stride-core/src/zones.rs` + the live duration check |
| The source's zone times can be captured at zero extra request cost | `inferred` from the sync's listing loop |

---

## 9. What could not be verified

* **Whether the source's zone times agree with a stored model.** That
  comparison needs the FIT samples and the stored model — the local database
  — and the rig database is SQLCipher-encrypted with no SQLCipher binding
  installed. It is the natural regression test for §7 and it is **not** done.
  No claim in this document rests on it.
* **Which zone configuration the source used for each run.** The payload
  carries no model, no floors and no sport link; it can be *compared* with a
  model, never *identified*.
* **Whether the source's zone times ever existed for a manual or
  heart-rate-less activity.** One run in the import has none; its type was
  not probed (type names would have been read to be worth nothing here).
* **How far back the source's zone times go in general.** For this account the
  oldest activity (~20 months) carries them; whether a longer-lived account
  would too was not tested, and Garmin's documentation says nothing about a
  retention period for the field.
* **Whether the athlete already has stored zone models** (which decides
  whether the affected set is 107 or ~98 runs). The rig database could not be
  opened read-only; §4 gives both figures rather than picking one.
* **The specification's new paths.** The spec now carries 28 paths where the
  local inventory records 26. The two that are new to the inventory —
  `/activity-service/activity/activityTypes/simple` and
  `/metrics-service/metrics/maxmet/daily/{startDate}/{endDate}` — are not
  zone endpoints and were **not** probed. The local inventory should be
  refreshed separately; nothing here depends on it.
* **Garmin's own documentation of what the zone endpoint means over time.**
  There is none: the parameter list is one boolean and the response carries
  no date. The absence of a history is established by the specification and
  the live response, not by a Garmin statement that history does not exist.
