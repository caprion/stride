# Stride v2 — the plan of record

> The approved amendments below govern current behavior.
> [Current screenshots and demo](docs/media/README.md) show the implemented UI.
> [`docs/plan.html`](docs/plan.html) preserves the original, historical wireframes.

Stride has one Rust core, a responsive React/TypeScript UI, an Android
Kotlin/Capacitor host and a local desktop HTTP host. One information architecture
adapts to phone and desktop widths.

**Status (2026-11-05):** **v1.0.0** promotes the signed **v0.12.0-rc.3**
application to stable without feature changes. Both hosts, Garmin sync, local
history exploration, responsive screens and native recovery workflows are implemented. The original
vertical line and slice lists below are historical delivery records, not work
still waiting to be wired. Physical-phone acceptance and complete provider
history remain open. No standalone PWA is delivered by this release.

### Approved product-reset amendments (2026-11-04)

The user approved a staged, in-place reset retaining the local Rust foundation,
Android-first delivery and original azure/amber identity. The historical design
below remains applicable except where these amendments explicitly supersede it.
This approval is not permission to publish, install, migrate or sync production data.

- Today leads with recorded weekly activity on phone and desktop. Only an
  explicitly imported or typed plan supplies sessions; inferred habits are not
  prescriptions. A session comparison uses its matched actual result, never an
  unrelated last run.
- Independent weekly pace/heart-rate medians remain descriptive. They cannot
  substantiate a matched-effort fitness headline; no such claim is emitted without
  validated paired evidence. Chart windows use calendar weeks and preserve missing
  observations rather than compressing gaps or substituting zero.
- An incomplete listing never authorizes deletion or a successful activity-scan
  timestamp. Wellness failures are surfaced, preserve previous successful coverage,
  and stop subsequent provider requests instead of masquerading as absent readings.
- Initial history targets the last 12 months, arriving incrementally; mobile data
  is permitted and older history is on request. The import now stores dated
  checkpoints independently for activities and wellness. Each job is bounded;
  Android continues unfinished history through one separate WorkManager owner.
  This is implementation progress, not production/device acceptance.
- Azure evolved light is the accepted default, including native setup/data
  screens. Available measurements lead; wholly absent signals stay in collapsed
  disclosure, while historical readings and available activity details remain.
- Phone backup/restore now uses a native user-chosen passphrase independent of
  the original device key and Android document delivery. Separate dated CSV/JSON
  exports are readable, not recovery backups. Isolated recovery tests do not
  establish physical-device document-provider or process-death acceptance.
- Body explores selected dates and 7/28/90/365 calendar-day windows, preserving
  gaps. Today shows recorded-session dates, not inferred rest days. Progress uses
  the core's calendar anchor and offers 52 weeks. Android refreshes the current
  phone offset before requests and sync without reopening the store.
- Run-detail routes are decoded from every valid position in the stored FIT,
  bounded by the response sample limit across the full route with endpoints
  retained. The import summary's first/last GPS metadata is not the route.
  The GPS outline renders locally without tile requests. An explicit Street map
  action enables OpenStreetMap tiles with attribution and disclosure of the
  viewed area, pan/zoom and automatic local-outline fallback on tile failure.

### Runner-first interaction refinement (2026-11-05)

The user accepted Stitch's compact Azure style as a design reference, not its
invented measurements, diagnostics or unverified capability claims. Apply the
shared responsive UI to the signed Android testing APK first; assess PWA
delivery separately afterwards. The same destinations remain available at phone
and desktop widths.

Body puts selected-date sleep stages inline, with dated charts and explicit
missing observations. Today and run detail reuse that presentation, with the
run's night matched to its date. Stages do not establish sleep quality; do not
invent a score or chronological timeline. Pace and heart rate share an elapsed
time cursor; distance-based elevation and untimed route geometry remain separate.

Runs supports an explicit two-run comparison tray. Date filters now search stored
history; text search remains scoped to loaded pages. Progress describes saved
activity history, marks the partial current week
and distinguishes separate weekly medians from matched-effort evidence. Average
session counts are scoped to recorded weeks. Data & connection leads with
local storage and actual Garmin status; diagnostics are secondary and connection
failures are not interpreted as disconnection. Saving and loading failures remain
visible and do not claim success or replace saved records with invented emptiness.

The user initially authorized a signed GitHub pre-release for physical-phone
testing, then explicitly authorized stable promotion as v1.0.0 on 2026-11-05.
That promotion does not establish physical-device acceptance or complete
provider history.

The rc.3 interaction pass adds shared date/history links, monthly run browsing,
bounded local route previews and preserved history return paths. Progress totals
follow the selected period, and its weekly bars open saved runs. Compare uses an
aligned table, paired desktop charts and compact A/B switches for heart rate or
pace. Settings separates data, preferences, imports and About; desktop backup
opens the host-owned tools rather than an unconfigured export destination.
Desktop/mobile browser acceptance does not establish physical-phone acceptance
or pixel-identical reproduction of the synthetic Stitch prototype.

The initial target is a 365-day lookback, approximately twelve months, inclusive
of today. A foreground pull checks the latest fourteen activity dates and up to
three wellness dates; continuation visits older checkpointed dates. The target
can be extended explicitly to 3650 days without resetting completed history.
Default per-attempt bounds are 20 FIT download attempts, 14 historical wellness
dates, 80 actual HTTP attempts, 32 MiB response-body bytes and 45 seconds. Byte
and elapsed-time limits stop the next request; an already bounded in-flight
response can exceed those soft limits. They are not wire-byte measurements.

Successful no-data days count as checked dates, not measurements or a provider
retention guarantee. Wellness data and its cursor commit together; missing
provider fields preserve existing observations. Activity reconciliation requires
an entirely enumerated dated window. Capped windows shrink, and an unenumerable
single day produces an explicit failure instead of an endless automatic loop.
Partial provider failures remain failures even when some data landed.

The Android `history_days` preference is new. The former `sync_days` value
bounded a single recent pull and is preserved, not reinterpreted or overwritten.
Automatic history continuation requires enabled network and auto-sync, allows
mobile data and waits for sufficient battery and an unlocked phone. WorkManager
backoff paces continuations, so completion is inexact and can span days. A failed
key stops automatic retries; rate limits and connection failures defer instead.
No publish, install or production-data pull is part of verification.

Optional explainable nudges are accepted product direction, but no new nudge
contract or wording is introduced by this slice; existing voice checks remain.

### Note — the phone's route table and the contract's list of unread routes (2026-11-24)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **No route changed.** What changed is the
phone's copy of the route table, and one sentence of prose.

* **The phone's bridge was eleven routes short.** The bridge refuses a path
  that its table does not name, so the phone **404'd** on the Progress
  wall's load card, the rolling window, the pace panel, challenges and
  streaks, on the run's film, and on the run update — the model's own
  reading of a run. The bridge's table now names every route the router
  serves, in the router's own relative order.
* **A copy that drifts is now checked.** `scripts/check-contract-greps.sh`
  compares the router's patterns with the bridge's table, both read out of
  the code, so a route cannot go missing from the phone again.
* **The contract's list of the routes the app does not read was wrong in
  both directions.** The app reads `/plan/import` and four of the five
  `/auth/*` routes, and reads neither `/fitness/trends` nor `/health/daily`.
  The list is corrected. The table of routes itself is untouched.
* **One limitation is recorded rather than papered over.**
  `/activities/{id}/fit` answers with the file's bytes and not with a DTO.
  The bridge carries JSON, so the file travels base64 and the phone does not
  offer the file for download; the export control is the desktop's. A native
  delivery path on the phone is a separate piece of work, not a route.

### Note — one window for the whole wall, and one comparison (2026-11-24)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **No route changed, and no shape changed.**
What changed is the window the rolling view draws, the period it is
compared against, and the length of the window the answer names.

* **The window is the last *N* calendar weeks, through today.** From the
  Monday of the earliest week to the day the answer is about, with the
  current week in it and partial. This is the window the Progress totals
  and the pace page already draw; the rolling view is the odd one out, and
  it now draws the same one.
* **The comparison is the same span, seven days earlier.** The week that
  came in is weighed against the week that fell out, so a partial week is
  compared with a partial week. It is one rule for every window-shaped
  answer, and it replaces the one-day shift the rolling view used.
* **The window has two ends now.** Its length is not a constant: the last
  twelve calendar weeks are seventy-eight days on a Monday and eighty-four
  on a Sunday, so a window that stored a number of days could not say which
  days it covered. `window.days` says how many days the window holds.
* **What the athlete reads.** The Progress totals do not change: they
  already used the calendar weeks. The **profile's volume a week and its
  consistency do**, because the rate is now measured against the window's
  own length — eighty days for a twelve-week window through a Wednesday,
  not eighty-four. The **year's "what changed"** compares the same span
  seven days earlier instead of the days before it.
* **The numbers, for the record.** A twelve-week window through Wednesday
  the 13th of May 2026 is the 23rd of February to the 13th of May: eighty
  days. The period it is compared against is the 16th of February to the
  6th of May.

### Note — the zone model in force on the day of the run (2026-11-24)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **The zone model's shape changed on one
route** (`/insights/rolling`), and the rule behind every zone-time answer
changed with it.

* **The rule.** Every run is measured against the **zone model in force on
  the day of the run** — the last model that was read on or before the end
  of that day. It is the configuration the athlete was wearing when they
  ran it. The answers that carry it: the run detail's zone table, the
  run's film, the fingerprint's distribution and efficiency factor, the
  window's split, and the year's mix.
* **A run that predates the first stored model is not calibrated.** It has
  no model in force, the answer says so, and it never borrows a later
  model. **Every zone number therefore changes, retroactively**, for an
  athlete who has changed their watch configuration, and a run from before
  the first stored model has **no zone times at all**.
* **The source line is hoisted.** The window's answer names **every**
  model it used — `zones.models`, oldest first — and carries **one**
  source line for the screen to name, `zones.source`. When the window was
  measured against more than one model, the line names them by the days
  they were read. The single-run answers keep their own line, and a run
  measured on a different model still says which one.
* **The load model does not move.** The training load weighs a whole
  window of days against one model, and it moves later, in its own change,
  with its own announcement.
* **One seam.** The api reads the stored models in one place
  (`crates/stride-api/src/zonemodel.rs`), so the rule is in one place and
  the store's dated history is read as a history.

### Note — the storage for the races, the threshold and the source's own facts (2026-11-24)

This note travels with a change to `docs/contract/schema.sql`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **The schema version moves from 5 to 6**, and
three tables are added. No route changed, and no shape changed.

* **The athlete's races** (`race_observations`). The distance, the finish
  time, the day of the race, the day it was entered, the day it was last
  changed, and the **label** the athlete gave it — a closed list: `5K`,
  `10K`, `15K`, `10-mile`, `half marathon`, `marathon`. The distance is in
  metres, the core contract's own unit; the label is what the athlete says.
  The **anchor** — the race the paces come from — is a flag, and **at most
  one row carries it**: entering another race never moves it by itself.
* **The athlete's threshold** (`threshold_observations`). One dated fact per
  **choice**: the heart rate, the pace band's two ends, or **both** — a
  typed value from a lab test carries the pair — with the **source** (the
  watch's zone model, a race, a field test, or typed), the **rule** that
  computed it where one applied, the **base** it was made for, the day the
  choice was made and the day the row was last changed. **At most one row
  carries `in_force`**: one threshold at a time, the earlier choices stay in
  the dated history with their base named, and a plan adopted in March is
  paced by March's number. The candidates are computed and never stored.
* **The source's own facts about an activity** (`activity_observations`).
  One row per (activity, source, field, item), in the shape
  `wellness_observations` already uses: the value, the day the response
  carried it, the day it was read, and the source. The watch's own zone
  minutes (`hrTimeInZone_1..5`, over the **elapsed** time) are the fourth
  path for the imported history, and they ride on the **activity-search
  payload the sync already fetches** — **no extra request**, and the pull's
  request budget does not grow. A field the source never sent has **no
  row**; a field it sent as null has a row whose value is null; **never a
  zero for "no time"**.
* **What stays out.** Garmin's estimates — the VO₂ max, the training stress
  score, the activity training load, the lactate threshold, the intensity
  factor, the stress and the body battery — are judgements, and the contract
  excludes them. The races and the threshold are **the athlete's own** and
  are **never synced**: they are typed into the app.
* **The export and the backup carry every table**, with their counts in the
  manifest. These are the athlete's own facts, held locally in an encrypted
  database; the export is the athlete's own file and stays with the athlete.
* **One bump, and where that differs from the amendment above.** The
  amendment "the two bases of zone time" says each new table is its own
  bump. The storage step's brief asks for **schema version 6** with the
  three tables in it, and the three tables are read and written together, so
  they ship in **one** step: a database at version 5 takes version 6 and
  gets all three tables and their indexes, in one transaction, and a
  database that already ran version 6 changes nothing.

### Proposed amendment — the zone model is a dated fact (2026-11-15)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The plan's architecture says every derived series — samples, laps, **zones**,
fingerprint features — is recomputed on read from the FIT and never stored.
For zones that premise is wrong. The zone model is **not in the FIT**: it is
the watch's own configuration, published by Garmin at
`/biometric-service/heartRateZones/v2`, and it is a **dated fact about the
athlete's settings, not a series derived from a run**. Recomputing bands per
run from *that run's observed maximum* gives two runs two different scales,
which is how an easy run and a threshold session come out looking alike; and
an insight that cannot say where its input came from and when it was read is
not inspectable.

The rule this serves: a zone model is read once, stored with the day it was
read, and reported with its source. One row is written per model change (a
dated history, not one current row), so a number always names the model it
used. A read that fails keeps the previous model and its date; absence is not
zero and not "no model". A run with no stored model is **not calibrated** —
the result is "no value", never band 1 and never zero. Garmin reports one
model (one `DEFAULT` sport); no per-sport models are invented.

This travels with the frozen schema change (the `hr_zone_models` table) in
`docs/contract/schema.sql`.

### Proposed amendment — the zone model on the contract (2026-11-16)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The model is stored and read by the sync, and nothing reads it: the route
that answers "how long was the athlete in each of their zones" rebuilt five
bands from **each run's own observed maximum heart rate** — a tenth of it,
from half of it up to all of it. Two runs were therefore measured on two
different scales, an easy run and a threshold session could land in the same
bands, and the bands could not be named, dated or audited: they were not the
athlete's configuration at all.

The rule this serves: **the zone model is a dated fact about the athlete's
configuration, and every intensity statement is made against it.** A number
that cannot say where its input came from and when it was read is not
inspectable, and the product's whole posture is that a number is reported
with its source.

So `/fitness/zones` changes its meaning, and keeps its name and its place:
it answers the per-run question from the **stored model**, and the payload
**names the model** — its sport, its training method, the maximum heart rate
Garmin reported, its five floors and the day it was read. The old
computation is gone, not left beside the new one.

The rules the payload follows:

- **No value is the absence of the whole result.** A run with no stored
  model has no bands and no time outside them, and says why in the core's
  words. It is never band 1 and never a zero. A stored model whose floors
  are not strictly increasing cannot be used, and says so the same way.
- **The boundary rule is one sentence, and it is this one: a reading exactly
  on a floor belongs to the lower zone** — the floors are exclusive lower
  bounds, so the interval of zone *n* is `(floor_n, floor_{n+1}]`, and a
  reading at or below the first floor is outside the zones.
- **The samples are the run's own**, decoded from its FIT file on read, and
  only the moving ones — the same filter the route has always used, so the
  numbers stay comparable with what it answered before.
- **The run-detail surface is not in this change.** Showing the shares to
  the athlete with the model's date beside them is the next item, and it
  reads this payload.

This travels with the route and field tables in `docs/contract/api-paths.md`.

### Proposed amendment — the Eddington number on Progress (2026-11-16)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

A running log has one question it can answer exactly, and the Progress
screen does not answer it yet: *how consistently has this been done?* The
Eddington number is the largest *E* such that the athlete covered at least
*E* units of distance on at least *E* days inside a period. It counts the
habit, not the peak, and it needs no new input: the daily distance is
already stored.

The rule this serves: **a number is only a fact against a stated period.**
So the number travels over the window the screen is already showing — the
same calendar weeks the volume chart draws — and the card names that window.
It is counted in **kilometres**, the contract's unit; the miles variant of
the engine stays unused, because a unit preference is a later decision and
not this one. A window with no distance has **no number at all**: that is
"no value", never a zero.

The number is not a score, a grade, a verdict or a traffic light. It says
what happened — how many days reached how many kilometres, over which
weeks — and the athlete reads what it means. The days still needed for the
next number and the longest distance in a day travel beside it, so the
number has a reference point and a direction of travel without being told
what to do about either.

This travels with the frozen route change (the `eddington` field of
`/insights/progress`) in `docs/contract/api-paths.md`.

### Proposed amendment — the ledger's weeks, and the band each one held (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The wall's *Where the time went* card was one bar: how much of the window's
time fell in each of the athlete's own zones, and the share of the time in
the zones that was. That is the **whole window in one number per band**,
and it cannot say *when* the training was hard. A five-week hole in the
hard band and a window that was hard in one week look the same.

The rule this serves: **absence is drawn, not described** — and a hole in
the training has to be a hole in a picture, with the week it fell in named.

What changes:

- **`/insights/rolling` carries `zones.weeks`**: one row per calendar week
  of the window that carries a run, oldest first, with the time in each of
  the three bands that week. The weeks come from the **same**
  time-in-zone computation as the bands, so a week cannot disagree with
  the window it belongs to.
- **A band with no time in a week is no value, never a zero.** The field
  is absent for that band in that week, so the card can draw the absence
  as white space: a week with no hard running is a hole in the high row.
- The card draws the **weeks × bands matrix**, each row scaled to its own
  busiest week, so a small row is still readable.

This is a **contract change**: `docs/contract/api-paths.md` names the
`/insights/rolling` shape and the zones, and both move with it.

**What this does not settle.** The ledger's two *shape* rows — the longest
run and the repeated work — are not in this change. The repeated work is
the watch's own workout steps, and the corpus carries none: the decode
cannot be proven end to end until the step names are decoded, and the
frozen corpus is not the place to make them.

### Proposed amendment — one chart on the run detail, over the run's own time (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The run detail drew **three charts of one run**: the heart rate with the
athlete's own zone bands behind it, the pace below it, and the altitude on
its own axis. Three charts, three axes, two cursors, and a reader who had
to hold three pictures of one run in their head to answer "how hard was
that, and where".

The rule this serves: **the surface carries the picture, its labels, the
numbers, the period, and the reason something is missing** — and one run
is one picture. Every observation the core makes about a run is a
difference of **times**: the pace drift and the heart-rate drift are
"second half minus first half", the decoupling is the heart rate's rise
against the pace's, and the time in a zone is seconds of moving time. A
stopped stretch covers no distance but is real elapsed time, and the
product has a whole bucket for it — on a distance axis it would vanish.

What changes:

- **The samples carry the distance, the altitude and the movement kind**
  beside the pace and the heart rate. One series, one axis: the seconds
  from the start.
- **The separate elevation array goes.** It was the same altitude on a
  distance axis, and the run detail now reads the altitude off the
  samples, over the time it was covered.
- **The run detail draws one chart**: the pace and the heart rate over the
  elapsed time, the athlete's own zone bands behind the heart rate, and
  **one cursor** over both. The altitude keeps **its own scale and its own
  metres label**, and it is **off by default** — the ground is scenery,
  not an effort signal.
- **The effort table keeps what it carries**: the zone numbers, the *not
  moving* bucket and the model's source line. The chart shows a shape; the
  table shows the bands, and the two are read together.
- **The movement kind is the classification the zones already measure.**
  The ingest classifies and the core reads, so the chart's stopped
  stretches and the table's *not moving* bucket cannot disagree.

This is a **contract change**: `docs/contract/api-paths.md` names the
sample's fields and the run detail's elevation array, and both move with
it.

### Proposed amendment — a stopped stretch is not time in a zone (2026-11-23)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The zone times were computed over a sample list with the **stopped samples
removed**. That looks harmless and is not: a sample owns the span until the
next one, so removing the stopped samples does not remove the time — it
merges the stopped span into the neighbouring running one, and the run then
claims more time in a zone than it spent moving.

On the rig's own runs it was not a rounding matter. A 56-minute easy run
answered **62 minutes of zone time**; a tempo run answered 58 minutes
against 54. The zone times and the run's own duration were two numbers on
one screen that did not agree, and the athlete had no way to see why. The
same arithmetic ran through the **pace bands** and through the **polarised
split**, so the share of the training in the hard zones was a share of an
inflated total.

The rule this serves: **a run cannot claim more time than it spent.** The
zones and the outside answer about the time the athlete was *moving*, so
they are measured over the moving time and over nothing else.

What changes:

- The **stopped samples stay in the list**, each carrying whether the
  athlete was moving. A span owned by a stopped sample is the time they
  stood still, and it goes to its own bucket.
- The core's `TimeInZone` gains **`stopped_s`**, and the zones and the
  outside add up to the **movement** the zones were measured over. The
  stopped time sits beside it, and the three together are the run's
  recorded elapsed time.
- The route answers with **`stopped`**, and the run detail and the ribbon
  show it — "Not moving · 3 m" — so the reader can see where the time
  went instead of finding the arithmetic does not close.
- The pace bands skip the stopped spans the same way, so the two cannot
  disagree about which samples count.

**What this does not settle.** The watch's own timer and the record
stream's movement are two measurements, and on the rig's runs they differ
by one or two per cent: the watch pauses its timer in places the record
stream does not. The zones are measured over the **record stream's**
movement, and the run's duration is the **watch's** timer. Reconciling the
two needs the watch's timer events, which the ingest does not decode; that
is a later item, and until it is done the two numbers are the two
measurements they are.

### Proposed amendment — the zone model's date on the run detail (2026-11-17)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

### Proposed amendment — the fingerprint as observations on the run detail (2026-11-17)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The bands are the product's most interpretable intensity statement, and
since the zone model became a dated fact they are measured against the
athlete's own configuration. What they did not say is **where that model
came from and when it was read**: the payload carried all of it — the
sport, the training method, the maximum heart rate Garmin reported, the
five floors and the day it was read — and the screen showed none of it.

The rule this serves: **a number is reported with its source.** A band that
cannot name the model it was measured against is not inspectable, and the
product's whole posture is that a number is reported with its source. So
the run detail's Effort panel carries a **source line** under the bands,
and the honest "no value" line when there is no usable model.

The rules the surface follows:

- **The source line is a source line.** It sits under the bands, in the
  panel's own weight and spacing, and it is not a card and not a headline.
- **The date is the api's.** "10 May 2026" — the api's own date
  rendering, and the same form the run detail already shows for the run's
  own date. The core's canonical form (`2026-05-12`) stays inside the core:
  the core's source line is handed the rendered string, and a third form
  would be a form nobody owns.
- **The wording is the core's.** The line is a sentence about the data, so
  the core's catalogue writes it and the app renders it as it arrives. The
  app writes no sentence of its own.
- **"No value" is the absence of the whole result.** A run with no stored
  model has no bands and no time outside them, and says why in the core's
  own words. It is never band 1 and never a zero.
- **The model is the current one.** The route answers with the model read
  most recently, and that stays: a run is scored against the athlete's
  present configuration. Which model was in force **on the run's date** is
  a product decision that is not yet made — the store keeps a dated
  history, so it is possible, and it is not this change's to make.

This travels with the two fields of the `model` — `read_on` and `source` —
in `docs/contract/api-paths.md`.

### Proposed amendment — the training load on Progress (2026-11-17)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

Every other dashboard calls the training load "high risk" or "optimal" or
"accumulated fatigue" — verdicts about a person, produced by a model whose
assumptions are nowhere on the screen. What is worth keeping is the
**scenario**: with a complete rest day, the ratio is expected back inside
its usual band by about a day, and the form turns positive after about
three. A scenario says what it assumes and lets the reader decide.

The rule this serves: **a number is only a fact against a stated period,
and a model that produces numbers states its assumptions where the numbers
are read.** So the load travels as values and a scenario — never as a
verdict — and the assumption travels with it: the coefficient it used
(`k = 1.92`), where the maximum heart rate came from (the athlete's stored
zone model, and the day it was read) and the two time constants.

So:

* **The load lives on Progress**, beside the Eddington card, under the
  range selector, so the number is never read apart from the control that
  changes it. Progress is the screen about a window of history, and the
  load is a series over weeks.
* **It is a derived series, and not an "HR-zone page".** §2 defers the
  HR-zone pages; what is deferred is a page of the athlete's zone
  distribution — the zones are already on the contract (`/fitness/zones`)
  and on the run detail. A derived series computed on read from the stored
  facts is neither of those things, and this amendment says so explicitly
  because the plan's §3 lists the derived series (samples, laps, zones,
  fingerprint features) as things recomputed on read.
* **It has a route of its own** — `/insights/load?weeks=N`, the same
  parameter and the same bound as the Progress route — and not another
  field on `/insights/progress`: the load needs its own window and its own
  "no value" reasons, and the progress payload is already wide.
* **The window and the constant.** The chronic series is weighted over
  forty-two days, which is longer than most windows, so the series is
  warmed with 210 days of history read before the window — the reference
  implementation's own warm-up. Without it the numbers at the start of a
  window would be the numbers of a series that began there. The window's
  days are the window's days; the warm-up's are weighed and are not shown.
* **No value is the absence of the value.** A store with no zone model
  answers with the reason and no numbers: never a zero, never a guessed
  maximum heart rate. A day with nothing that could be weighed carries its
  own reason, and a week that is not complete carries no impulse, no
  monotony and no strain.
* **No age, no date of birth, no sex, no athlete table, no FTP and no
  power.** The inputs are facts: the maximum heart rate and the threshold
  from the stored zone model, the resting heart rate from the measured
  daily summary.

The voice is the plan's: values and directions, and **no verdict** — no band
name, no risk, no traffic light and nothing that tells the athlete what to
do. The band of the ratio is named as a **range** ("0.8 to 1.3").

This travels with the route and the field tables in
`docs/contract/api-paths.md`.

The old application kept a table of **running mechanics** per run: the
cadence and its variability, the pace's variability, the aerobic decoupling,
the negative split, the stride length, the zone distribution — and a
`form_score` at the end of them. The legacy importer dropped that table with
the reason *"derived from the FIT"*, which is right in principle: the
derived series are recomputed on read and never stored. The work was
therefore to **derive it properly, in the core, from the series Stride
already has** — and that is what this amendment records.

The rule this serves: **a run's mechanics are observations, not a grade.**
The old `form_score` was a weighted mean of three bounded component scores
(cadence 30 %, decoupling 45 %, pace 25 %), with the weights renormalised
around whichever component was missing and a hundred-point scale on top.
That is a grade: it says the run was worth a number out of a hundred, and
the thresholds that made it so were the implementation's own. The product
reports values and lets the reader decide, so **the composite is not
ported**. The observations it was built from are, and each of them says
what it measured: the cadence, the variability of the cadence and of the
pace, the drift between the first third of the run and the last, the
half-split drift of the heart rate and of the pace, the efficiency factor,
the decoupling, the stride length.

Two of the reference's inputs are **not** ported either, and both for the
same reason the zones were rebuilt:

- the **reference maximum heart rate** — the old application measured the
  efficiency factor against the largest maximum heart rate it had ever seen
  on that athlete, and fell back to 190. Two numbers measured against two
  scales are not comparable, and 190 is a guess. The efficiency factor is
  now measured against the maximum heart rate of the athlete's **stored
  zone model**: their watch's own configuration, read once and stored with
  the day it was read. A model with no maximum heart rate has no
  efficiency factor, and says so;
- the **zone model** — the distribution is the **stored** model's, and it
  is reported as the **shares of the moving time** in the athlete's own
  zones, with the model named and dated. The old application rebuilt five
  bands from a reference maximum, which is what Epic A removed.

And the old **negative split** — a flag that was `100` for a negative split
and `0` otherwise — is not ported as a flag: a zero is not a measurement.
The pace drift is reported as the **signed difference** between the two
halves, so a negative number *is* the negative split, and the sign is the
direction.

The surface is a **detail section on the run detail**, below the other
sections: the values, and the direction each one moved — the sign of the
difference. It is not a headline, and it is not a page: PLAN.md §2 keeps
the running-form fingerprint **page** out of scope, and this amendment does
not bring one back. A metric the run cannot answer says why, in the core's
own words, and never shows a zero.

The route changes its meaning with it: `/fitness/fingerprint` answered a
list of runs with their distance and their pace, and it now answers **one
run's** fingerprint, named by the query, with the run detail showing it.
The route keeps its name and its place in the frozen table; the old answer
is gone rather than left beside the new one.

This travels with the route's field table in `docs/contract/api-paths.md`.

### Proposed amendment — the rolling view on Progress, the streak on Today (2026-11-18)

### Proposed amendment — the source's wellness observations are kept (2026-11-17)

**Proposal, not approved.** The approvals below are the user's to give.
This section records the change and the rule it serves; it is not permission
to ship, publish, migrate or sync production data.

A running log has six questions about its own history that it can answer
exactly, and this product answered none of them: how long the streak is,
how the last *N* days compare with the *N* before, how the training is
distributed across the athlete's own zones, which way that distribution
moved, which days and which times of day the athlete runs, and what a month
or a year looks like. Every one of them needs **no new input** — they are
derived from the runs and the zone model that are already stored — and
every one of them is *depth*, not breadth.

The rule this serves: **the product shows the values, says which way each
one moved against the athlete's own usual, and lets the reader decide.** So:

* **The rolling view lives on Progress**, under the range selector, because
  Progress is the screen about a window of history. It answers with the
  window and the period before it, the two numbers side by side and the
  difference between them, and the card **names the window it describes**.
* **The streak lives on Today**, because it is the "how is the habit going"
  number, and Today is where the athlete looks first.
* **The window's basis is stated, not guessed at.** The window is the last
  *N* days ending today, and the comparison period is the same *N* days
  ending yesterday — the reference implementation's own comparison. The
  parameter is the Progress route's own `weeks`, so the card follows the
  range selector the screen already has.
* **The zones are the athlete's own**, measured against the stored model
  and named with the day it was read. The three bands are **low (Z1–2),
  moderate (Z3), high (Z4–Z5)** — what they are, not what they ought to
  be. **No target range travels with them**: a range that says how the
  split ought to be is a prescription, and this product does not prescribe.
* **Nothing is stored.** Every one of these numbers is recomputed on read,
  like every other derived value.
* **Every value that cannot be computed is "no value", never a zero.** A
  streak of zero days is no streak; a window with no time in the zones has
  no shares; a month with no runs is not a row in the monthly table.

The voice is the hard part of this item, and the string lint is what holds
it: these panels invite conclusions — "you are in a good streak", "your
training is imbalanced", "you are peaking" — and the product draws none. A
streak is a count and a date. A share says which way it moved and stops
there. A difference is a difference.

This travels with the two routes and their field tables in
`docs/contract/api-paths.md`.

### Proposed amendment — the zone distribution, the distance breakdown and the training profile (2026-11-18)

**Proposal, not approved.** The approvals below are the user's to give.
This section records the change and the rule it serves; it is not permission
to ship, publish, migrate or sync production data.

The reference dashboard's panel catalogue was read against this port, and
three of its panels had no home here: the **heart-rate zone distribution**
(the histogram and each zone's share), the **distance breakdown** (the runs
by how far they went) and the **athlete profile** (the radar's content).

The rule this serves is the one the whole port runs on: **a number is only
a fact against a stated period, and a shape is not a fact.** So:

* **The distribution** is the same time-in-zone computation the bands
  already use, one zone at a time: the time in each of the five and the
  share of the time in the zones that was. It comes free from what the
  zones route already measures, and the model is named with it.
* **The distance breakdown** is one aggregation over the stored distances,
  with the reference implementation's own bucket rule: the bucket size
  comes from the longest run in the window, so the longest run lands in
  the last bucket.
* **The profile is seven values and their movements, and not a radar.** A
  polygon over fixed anchors — "ten hours a week is a very active
  amateur", "five days a week is excellent consistency" — is a claim
  about a shape, and the anchors are somebody else's coaching
  assumptions. The facets here are the athlete's own numbers: the
  volume, the consistency, the intensity, the duration, the density, the
  sport variety and the days of the week. Each says which way it moved
  against the window before it, and the reader draws the shape.
* **No value is the absence of the whole result.** A window with no run
  has no profile at all — every facet is no value, never a zero — and a
  facet with nothing to compare against has no movement to report.

This travels with the fields of `/insights/rolling` in

### Proposed amendment — the year, as a film (2026-11-20)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

A running log can answer three questions about its own year, and all three
are facts about what happened: what each month held, what each day held,
and what changed against the athlete's own past. This item adds them as
one surface — **the year, as a film** — and it adds them on the terms the
plan already sets.

The rule this serves is the one the whole product runs on: **a number is
only a fact against a stated period, and the calendar is about the days
and never about the places.**

So:

* **The window is the last twelve calendar months**, ending on the day the
  answer is about, and the period it is compared against is the same
  number of days ending a day earlier — the rolling view's own
  comparison. The answer names both windows, so a number is never read
  apart from the period it describes.
* **The months are the core's.** Each one holds the distance, the runs,
  the longest run, the shape of the weeks it touches, the mix of the
  zones it was trained in and the form at its end. **Nothing is stored**:
  every one of them is recomputed on read, like every other derived
  value.
* **The calendar's colours are days, never places.** A day is coloured by
  the share of its time in the zones that was in the high zones — the same
  time-in-zone computation the run detail uses, against the athlete's own
  stored model. **No position and no place name travels in the answer.**
  Where the athlete ran is the heatmap's surface, and it has its own
  screen, its own disclosure and its own rules.
* **The diff is the core's own sentences.** The distance, the sessions,
  the longest run, the pace and the form, said as the numbers of two
  periods side by side. The app writes no sentence of its own.
* **No value is the absence of the value.** A day with no run has no
  distance; a day whose runs carry no zone times has no mix and no
  colour; a month with no runs holds no numbers; and a diff with nothing
  to compare has no line for that value.
* **The voice is the plan's.** Each month is a fact about a month, and the
  year says what happened and what changed. It does not say whether the
  year was good, and it does not tell the athlete what to do.

**The surface is a page reached from Progress**, the way Compare and the
Plan week are reached from inside the app — not a fifth destination. The
four destinations stay the four, and the same page is there at every
width.

This travels with the route and the field table of `/insights/year` in
`docs/contract/api-paths.md`.

### Proposed amendment — the run film, with the flyby (2026-11-20)

**Proposal, not approved.** The approvals below are the user's to give. This section records the change and the rule it serves; it is not permission to ship, publish, migrate or sync production data.

The mock's surface the athlete liked most is the one the product is
missing: **one run, playable**. The pace, the heart rate and the ground on
one axis, the athlete's own zones behind them, and the route beside it
with a camera that follows the runner. The readout says where you are:
elapsed, distance, pace, heart rate, ground, and the zone you are in.

The rule this serves: **a run's own facts, read from the file that was
stored with it, and named with the source they came from.** The film is
the run read back as a timeline, and every number on it is the run's own.

So:

* **The film is a surface, not a destination.** It is reached by an
  **explicit action** on the run detail — the same discipline the Street
  map and the heatmap use — and it is a route under `/runs`
  (`/runs/{id}/film`) with a back link. The run detail keeps its own
  panels; the film is the deep dive. The four destinations stay the four.
* **The series is the run's own file, decoded on read.** The samples, the
  ground and the splits come from the FIT file that was stored with the
  run, and nothing is stored: the derived series are recomputed on every
  read, like every other derived value.
* **The zone of a sample is the core's.** Each sample carries the zone it
  sits in, measured against the athlete's **stored** zone model through the
  core's own classifier — the one the time-in-zone computation uses — so
  the ribbon behind the film's timeline and the bands on the run detail
  cannot disagree. The model travels with the film, **named and dated**.
* **The route is the app's own map.** The flyby is the Street map with a
  marker that moves and a track that is coloured by the pace: the same
  tile layer, the same disclosure, no second map library. The route is
  drawn once and the marker moves, so the film stays smooth. And the
  **street tiles are behind the same explicit action** they are behind on
  the run detail — the athlete asks for them, and the app says what they
  reveal. Without them the flyby is the athlete's own route on this
  device, and nothing leaves it.
* **The scales are the chart's own.** A run carries a few readings far
  outside the rest — a walk break's pace of nineteen minutes a kilometre —
  and a scale that reaches them squashes the series into the top of the
  chart. So the pace's and the heart rate's scales cover the middle of the
  readings, and the readings outside are drawn past the edge of the
  drawing. Nothing is thrown away and nothing is rounded: the values are
  the run's own, and the words beside them are the core's.
* **No value is the absence of the whole result.** A run with no file has
  no series at all, and says why in the core's words. It is never a zero,
  never a band, and never an empty chart that looks like a run of nothing.
* **The film's own words are the product's.** The core's catalogue holds
  the sentences; the app writes none of its own.

This travels with the route and the field tables in `docs/contract/api-paths.md`.

### Proposed amendment — the heatmap, computed on the device (2026-11-17)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

§2 deferred the heatmap with the tiles proxy and the other out-of-scope
extras. This amendment brings **one** of them back, in one form: a grid
over the positions the athlete's own runs carry, computed on the device.
The tiles proxy stays out.

The rule this serves: **nothing leaves the device, and a heatmap is a
picture of a habit — so it is a grid, never a place name.**

A heatmap is the surface where a running log says the most about where a
person lives. The product's answer is the one it already uses for the
Street map: **local aggregation, an explicit action, and a disclosure of
what leaves.** This item has none of the Street map's outbound risk —
nothing is fetched, nothing is sent — and that is the point of it.

So:

* **The grid is computed on the device.** The route reads the stored runs
  and decodes their stored FIT files — the same files the run detail's
  route decodes — and answers with a count of runs per cell. There is no
  tile request, no geocoding, no reverse geocoding, no third-party
  service and no outbound anything.
* **The grid is coarse on purpose.** It is the reference implementation's
  own resolution: zoom 14 of the slippy-tile grid, a cell about two
  kilometres across at the latitudes a runner lives at. A coarse grid is
  both faster to compute and less identifying than a fine one, and this
  surface is *about* a habit.
* **A cell's value is a count, never a route.** A cell holds how many runs
  went through it; a run counts once in a cell however many times it
  crossed it; a cell carries no time, no order and no connection to
  another cell. The answer is a set of rectangles with a number each, and
  no path can be read back out of it.
* **It is a place, not a person.** No street names, no addresses, no
  reverse geocoding and no place labels: a grid of cells over a map the
  athlete can draw themselves is the whole design. The surface says where
  the runs went and how often, and it draws no conclusion about the
  athlete's life.
* **It is behind an explicit action.** The heatmap is not on the run list;
  it is something the athlete asks for, the way the Street map is asked
  for.
* **It discloses what it is.** The surface says, in the app's own words,
  that the aggregation is local: it is computed on this device from the
  stored runs, and nothing is sent anywhere. The disclosure is true, and
  it says nothing about a service that is not involved.
* **Nothing is stored.** The grid is a derived series like the samples, the
  laps and the zones: recomputed on read, never written down. There is no
  grid, no tile and no heatmap blob in the database, and no schema
  change.

The heatmap is not a fifth destination and not a new tab: it is a
contextual surface reached from the Runs screen, the way Compare and the
Plan week are reached. The four destinations stay the four.

This travels with the route and the field tables in
`docs/contract/api-paths.md`.

The capability inventory's own verdict on the wellness is the heading of this
item: *"First: ownership — preserve source wellness observations, with
retrieval time and schema/provenance, inside encrypted storage and backup.
Sleep, HRV and summary fields are discarded today. FIT retention alone does
not protect wellness history. This needs a reviewed schema migration, not an
unversioned side file."*

The rule this serves: **a reading the source served is kept, with the day it
was read and the day it was for.** The product kept a projection of the
wellness — sleep seconds, the stages, the HRV, the resting heart rate, the
average stress — and dropped the rest: the sleep window and the window's
confirmation, the unmeasurable time, the naps, the stress durations per band
and the stress qualifier, the whole body battery, the seven-day average
resting heart rate, the respiration, the day's minimum and maximum heart
rate, and the source's own range of the day. A year of sleep and recovery
cannot be looked at later from a projection, and a number that cannot say
where its input came from and when it was read is not inspectable.

So the schema grows a table, and the shape is decided by the missingness:

* **One row per (day, source, field, item).** A field the source never sent
  has **no row at all**; a field the source sent as null has a row whose
  value is null. A wide table cannot tell those two apart, and this one has
  to: *"no reading"* and *"the reading was null"* are different facts, and a
  store that cannot say which one it is holding is not inspectable.
* **The field name is the source's own.** Nothing is renamed into the
  product's vocabulary, and nothing is derived: every value is one the
  source reported, in the source's own unit, which is recorded beside it.
* **A later read never deletes a row.** It replaces the value it carried and
  keeps the reading time of the read that carried it, so a failed read
  leaves the previous value and its reading time in place. A reading without
  its retrieval time cannot be audited.
* **The device's verdicts are not observations.** The sleep scores, the HRV
  status, the VO2 max and the like are the device's opinion of the athlete,
  and the plan drops a device score everywhere else. They are not kept.

The ingest follows the same rule: a payload is filed under the day **it**
carries, not under the day that was asked for, and a failed read is a
failure — the values already stored stay, and the pull says it failed. The
reports that carry the rest — the stress durations and the body battery —
are separate endpoints, and they are read over the window the pull covers:
two requests for a window, inside the sync's existing request budget.

**The surface is optional, and this change does not add one.** A surface is
where it can be honest — on Body, the source's own observation, and never a
second score — but the point of this item is the retention and the
provenance, and a reviewable, inspectable store is not optional. What the
product already shows keeps working exactly as it did: `wellness_daily`
stays the product's projection of the three vitals, and the source's own
readings live beside it.

This travels with the schema change (the `wellness_observations` table) in
`docs/contract/schema.sql`.

### Proposed amendment — the pace page (2026-11-20)

### Proposed amendment — the run update, the model's reading of a run (2026-11-19)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

A running log answers one question about its own history that nothing
answered: **how has the pace behaved?** The training chart every plan
draws — the week's distance with the long run inside it and the pace on the
other axis — was not in the app, and neither was the distribution of the
time by pace.

The rule this serves is the one the whole port runs on: **a number is only
a fact against a stated period, and a shape is not a fact.** So:

* **The pace page is a page, and it is reached from Progress.** It is not a
  sixth destination: the navigation stays Today · Runs · Progress · Body,
  and the page is reached from the Progress screen, the way the heatmap is
  reached from Runs. The four destinations stay the four.
* **It has three panels, and each says what happened.** The training, week
  by week — the week's distance as bars, **the long run inside each bar**,
  and the week's pace on the other axis; the pace at a heart rate — one
  point a week, the pace held at that week's median heart rate; and where
  the time went, by pace — the bands.
* **The bands are the core's**, beside the other derived values: the
  distribution of a window's moving time by the pace it was held at,
  measured over the runs' own **moving samples**, the same filter the
  time-in-zone computation has always used. They are thirty seconds a
  kilometre wide, and a pace exactly on an edge belongs to the band below
  it.
* **The bands are a property of the window**, and the answer **names** the
  window: the same bands over twelve weeks and over a year are two
  different answers. The window is the Progress window — the same `weeks`
  parameter and the same calendar weeks.
* **The bands are the bands the window's own time fills.** They are not
  cut at a fixed pace and gathered into two rows: a range cut at somebody
  else's paces is a claim about the athlete, and it hides most of the time
  in one row. Every band the window fills is there, and the times in them
  add up to the window's moving time.
* **The pace and the heart rate are separate medians**, and the page says
  so where it draws them. They are descriptive; they are not a
  same-effort fitness comparison, and the page makes no claim from them.
* **No value is the absence of the whole result.** A window with no moving
  samples has **no bands**: the answer carries the reason and no numbers,
  never a band of zero.
* **The voice is the plan's.** The pace trend is a line and the bands are a
  distribution: there is no target, no "too slow", no verdict and nothing
  that tells the athlete what to do.

This travels with the route and the field tables in
`docs/contract/api-paths.md`.



### Proposed amendment — the cockpit Today (2026-11-21)

**Proposal, not approved.** The approvals below are the user's to give.
This section records the change and the rule it serves; it is not permission
to ship, publish, migrate or sync production data.

Today was a wall of cards: the week, the last run, the streak, the plan,
and the readings, each in its own box. The **cockpit** replaces it with
three bands — **the hero**, **where you stand** and **what's on the table**
— and the rule it serves is the one the screen system already runs on: *the
phone is not a smaller desktop*. One hero, a few rows, and everything else
one tap away.

* **The hero is the latest run** — its name, its distance, its time, its
  pace, its heart rate and its elevation, the **zone ribbon** drawn from
  the athlete's own zones, the route the way the run detail draws it (the
  GPS outline, and the Street map behind the app's own disclosure), and a
  way into the run. The ribbon is a drawing of the **totals** the zones
  route already answers (`/fitness/zones`): the per-sample series over
  time belongs to the run film, and it is not built twice.
* **Where you stand is three rows, not three cards**: *this week* (the
  distance, and the difference against the **same elapsed days** of the
  previous calendar week), *form* (the load's own state, with the ratio and
  the band), and *the habit* (the streak, the months and the Eddington
  number). Each row is a question with the answer in it, and each answer
  names the period it describes.
* **What's on the table** is the plan's own sessions — unranked, and never
  linked to a vital — and the night's readings, in the words the app
  already uses for them.
* **The comparison is like for like.** The week to date is weighed against
  the same days of the week before, not against a whole week: the two
  windows are the same length, and a part weighed against the whole is a
  fact about the calendar, not about the running.
* **The months are a streak, and the streak is the core's.** A run of
  calendar months with a run in each is the same rule the day and week
  streaks already follow, and it lives in the core beside them.
* **Nothing on the screen becomes a verdict, a score or a command.** A
  difference is a difference, a form is a form, a band is a range, and a
  streak is a count and a date. The phone has no hero map — the map is the
  wide widths' business — and the scroll is the gesture.

This travels with the two answers' field tables in
`docs/contract/api-paths.md`.

Every run gets a short **update**: four to six sentences saying what the run
was and how it went. A language model writes it from the run's own numbers,
a **second** model reviews the draft before it reaches the athlete, and
three mechanical checks stand between the two models and the screen.

The update is **not the product's voice.** It is the model's reading, it is
attributed — the writer and the reviewer are named — and it lives beside the
app's own facts and not inside them. The athlete can dismiss it. The app's
numbers stay the app's: computed in Rust, from the stored series, with their
sources named.

The rule this serves: **the product is the athlete's alone, and a reading
that cannot be inspected is a sentence.** Everything the run already is —
the split table, the zone times, the fingerprint — is computed in Rust from
the stored series and stored locally. This item adds the *reading* and it
must not weaken anything the product already guarantees.

The rules the update follows:

* **The model never computes a number.** It is handed facts that are already
  computed and may only arrange them into sentences. Every number in the
  draft appears in the payload it was given.
* **Two models, two jobs.** The writer writes; the reviewer checks the draft
  *against the payload* — the numbers, the directions and the register — and
  answers ship, repair or discard. A repair is a second pass with the
  reviewer's findings; a discard keeps no update.
* **The reviewer's family must differ from the writer's.** A same-family
  reviewer repeats the same misreadings: in an earlier bake-off one model
  read a pace drift of −43 s/km as "pace improved by 43 s/km" — a sign error
  that a mechanical check catches and that a reviewer of another family
  would also catch.
* **Nothing leaves the device unless the athlete says so.** The feature is
  off by default, and the payload is **metrics only**: never the positions,
  never the athlete's note, never their name. The app shows exactly what
  would be sent before the first send.
* **A local model is a first-class choice.** The client speaks the
  OpenAI-compatible chat shape, which every provider here speaks and which a
  local server speaks on `127.0.0.1`. "Local" is then only a different base
  URL.
* **The models are settings.** Nothing is hard-coded: the writer, the
  reviewer and the two base URLs are the athlete's own choice, and the
  defaults may change when the evaluation of the candidates lands.

Where the parts live:

* the **checks** are in the core, beside the other derived values, and the
  forbidden list is the core's own ban list — so the app's lint and the
  reviewer's check read one list and cannot drift apart;
* the **update** is in the store, with the writer, the reviewer, the hash of
  the payload, the draft, the checks and their findings, the verdict and the
  day it was made — and it travels with the export, the import and the
  delete, like every other table;
* the **payload, the prompt and the stored update** are served by the api,
  and the **rig** owns the calling: the model client, the settings and the
  orchestration, after a pull that brought a new run;
* the **app** shows the update on the run detail, below its own facts,
  attributed, with the checks listed under it, and it can be dismissed.

This travels with the two route families and their field tables in
`docs/contract/api-paths.md`, and with the schema change (the `run_updates`
and `update_settings` tables) in `docs/contract/schema.sql`.

*The nine amendments that follow record the week of 24 November 2026 — the direction the wedge opened: the stimulus ledger, the pace engine, the threshold, the two bases of zone time, the plan's bases, the audience, and the three rules that now govern every surface. Each binds the decisions named in `exploration/decisions.md` (D1–D25) and is drawn from the documents in `exploration/`, which are cited by path rather than copied. Each is a proposal until the user approves it, and each names the rule it serves, what it may and may not say, the files it touches, and what is still open.*

### Proposed amendment — the stimulus ledger, the rows it holds, and the card it lives on (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The wall's *Where the time went* card is one bar: how much of the window's
moving time fell in each of the athlete's own five heart-rate bands, and the
share of that time in the bands grouped three ways. That is the whole window
in one number per band, and it cannot say *when* the training was hard, nor
what the window lacks. A five-week hole in the hard band and a window that
was hard in one week look the same.

The rule this serves: **absence is drawn, not described** — and a hole in the
training has to be a hole in a picture, with the week it fell in named. The
design is `exploration/stimulus-ledger.md`; the decisions it binds are D1,
D3, D4, D5, D7, D11 and D23.

What changes:

- **The card carries two families, and they are never added together.** Three
  **kinds** partition the window's moving time — **Z1–Z2 low · Z3 moderate ·
  Z4–Z5 high** — and two **values** overlap it: the **longest run** and the
  **repeated work**. The kinds' shares are over the record stream's movement,
  so the bands and the outside add up to 100 %; the longest run's share is
  over the window's stored moving time, the watch's own timer. Each row names
  its own denominator, and no total on the card adds the two families
  together.
- **Each row carries four things and no more**: the time, the number of runs,
  the **last date** the kind occurred, and the change against the period
  before — the same span, seven days earlier. The shares are read from the
  distribution printed once above the rows; no number is printed twice.
- **The last date is searched over the athlete's whole retained history**, at
  or before the end of the window, not inside the window alone — because a
  kind absent for longer than the window would otherwise show nothing at all.
  When the date falls outside the window the card says so ("35 days before the
  end of this period").
- **A measured zero is not a missing value.** A window with one easy run
  answers **0 m** of Z4–Z5 and no date — a measurement. A window with no
  stored zone model answers **no value**, with the reason. A run dated before
  the first stored model is a run the app cannot measure: its cell is empty,
  and the coverage line says why.
- **The weeks × bands matrix** is the 24 November amendment's, and this
  amendment sits on top of it: one row per calendar week, coloured by what
  that week held, each row scaled to its own busiest week so a small row is
  still readable. A band with no time in a week is **no value**, and the card
  draws it as white space.
- **The source line is hoisted once per screen** — the model, its method, its
  maximum heart rate and the day it was read, at the top of the screen, once —
  and an answer that used a different model says so in place.
- **One line on the run detail**: the run's own band, its time in that band,
  and the last date the band occurred. The run detail's own zone table and
  the ledger must not disagree, which is why the ledger ships **after stage 1**
  of the zone-model migration (D3).

**The repeated-work row depends on the watch's own workout steps.** A run is
repeated work when two or more of its own laps carry the step name
`interval` — the FIT lap message's definition 24, `wkt_step_name`. The
product adds no threshold and infers no structure: the watch wrote the step
names. The states are: steps present → the row answers, measured by the
watch's own structure; **no run in the window carries steps → no value**, in
the core's own words ("not recorded — the runs in this period carry no
workout steps"), never "0 m of repeated work"; steps present but none of them
`interval` → a measured fact about the file, not a missing value.

**The lap-shape heuristic is rejected, and the reason is kept.** "Three
consecutive laps of at least a kilometre within ±10 % of each other" answers
"repeated work" for a plain run with three equal auto-laps — and auto-laps
are equal by construction, so a shape rule cannot tell a workout from a lapped
run. The row is therefore **blocked on a file the corpus does not carry**: the
watch's own path cannot be built or tested until a structured workout is
recorded and synced. Until then the row has little to say, and that is the
correct answer, not a defect.

**What the card may and may not say.** The absence is a **date**, never a
verdict: the card says when the kind last occurred and does not say what was
not done, so it never needs the word the ban list reserves for that. It may
not say the training was unbalanced, that the athlete needs threshold work, or
that a week without hard running is a problem. And because heart-rate time in
zone under-counts hard running and over-counts easy running (Seiler 2010), an
absence row is corroborated by the run's own lap structure where the file
carries one — and where the lap data is missing, the card says the lap data
is missing.

**The files it touches.** `crates/stride-core/src/stimulus.rs` (new, beside
the other window-shaped analyses) · the api's read path, which decodes each
run's stored FIT on read (the precedent is `stride_api::pace::pace_runs`) ·
`crates/stride-ingest/src/fit.rs`, for the lap message's definition 24 and
the `workout_step` message · the Progress wall's card and one line on the run
detail · **a contract change**: the route `/insights/stimulus` and its field
table in `docs/contract/api-paths.md`. Nothing is stored — every value is
recomputed on read, like every other derived value.

**What this does not settle.** Whether the repeated-work row ships at all
before a structured workout file exists, or ships with its "no value" state
until one does; whether the card carries both the one-week-shift change and
the full-length comparison, or only the first; the card's density on the
phone — the merged card is long, and the phone may fold the change lines
away; which of the ledger's fields travel to the run detail; and whether the
card marks the window's partial current week.

### Proposed amendment — the pace engine: the chain, the ladder, and the cost curve (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The product has no race store, no VDOT, no race prediction and no pace of its
own, and the pace page the plan describes was retired into the Progress panels
by the surface cut (D7). So the one thing every training plan is written in —
the pace — is not in the product at all, and a Pfitzinger plan would arrive as
a set of distances with nothing to run them at. The design is
`exploration/pace-engine.md`; the decisions are D13, D14, D16, D17, D19, D20
and D24.

What changes:

- **The chain.** `race time (typed, dated) → VDOT → the race-pace equivalents
  → LT pace → the rungs`, with Riegel for a prediction. The **VDOT is the
  engine's own** — Daniels' oxygen-cost and percent-of-VO₂max curves,
  computed rather than looked up, so a table update cannot move a number
  under the athlete's feet — and the **rules applied to it are the method's**,
  quoted and cited. The chain replaces the book's lookup tables: the app
  computes the 10K, the 15K and the half-marathon pace from the athlete's own
  race time, and the card names which chain produced the number.
- **A race time is an observation, not a verdict.** The athlete types it; the
  store keeps the value, the day it was entered, the day it was last changed
  and the day of the race. The engine never invents one, never infers one from
  the training, and never derives a maximum heart rate from an age — there is
  no birthdate in Stride, and none is added.
- **The ladder is the picture, and it has two axes.** The rows are the
  method's run types, ordered by pace; the left axis is the **heart rate**
  (the method's band for that rung, drawn against the athlete's own zone
  floors) and the right axis is the **pace** (the method's rule, the athlete's
  own data, or the engine's own). Amber is the method's rule, azure is the
  athlete's own data, grey is the engine's own derivation. **No green and no
  colour that marks a problem** — the colours say *whose* number it is, not
  how it compares.
- **Every rung is a band and a pace, kept apart.** The threshold rung is a
  **band** (the hour pace, its fast edge the 15K equivalent), the long run is
  one rung with two ends, and the general-aerobic rung has **no pace from the
  method at all** — the athlete's own pace at that heart rate, or the athlete's
  own hand. Where there is no history at that heart rate the pace is a **no
  value with a reason**, and the band still stands.
- **Absence is drawn.** No race time → an empty ladder: the axes drawn, no
  rungs, and the caption names the missing input. A rung with no pace → a
  hollow marker on the pace axis, with the band still drawn on the heart-rate
  axis. Never a card of zeroes, never an empty chart that looks like a run of
  nothing.
- **The physiological-cost curve is a second reading of the same athlete, on
  its own route** (`/insights/pace-curve`), linked from the pace card — not a
  second ladder, and not crowded onto the ladder. It carries the curve, the
  athlete's own races, the goal race, the method's sessions, and the
  45-minute time trial's place in time.
- **Every derived number names its source**: the race it came from, the day it
  was entered, the day it was last changed, and which chain produced it.

**The rule this serves: the method's heart-rate bands are never drawn on a
%maxVO₂ axis.** Pfitzinger's bands are percentages of maximal heart rate; the
curve's axis is percentages of VO₂max. Converting one into the other needs
the athlete's lactate relation, which Stride does not have and will not
estimate. So the bands are **quoted in the caption and the legend** and are
never drawn on the curve, and where the athlete's own history allows a pace to
be named for a session, that pace is the athlete's own and is labelled as
such. One small caption inside each chart's frame says so in the chart's own
voice.

**What it may and may not say.** It may name the chain, the rule, the
citation, the race and the dates. It may not present the athlete's own pace as
the method's; it may not present the method's rule applied to the engine's
anchor without saying which is which; it may not state a readiness or a
fitness verdict from the curve; it may not rank the athlete's races against
the method's sessions; and it may not convert a pace into a heart rate or the
reverse.

**The files it touches.** `crates/stride-core/src/pace_engine.rs` (new) ·
`crates/stride-core/src/units.rs`, which gains one renderer — a race's finish
time is read as a clock ("41:36"), and the duration renderer would give
"1 h 32 m" · **a schema change**: a `race_observations` table in
`docs/contract/schema.sql`, holding the value, the day entered, the day last
changed, the day of the race and the anchor flag · **a contract change**: the
routes `/insights/paces` and `/insights/pace-curve` and their field tables in
`docs/contract/api-paths.md` · the catalogue's strings, which the lint checks
against the ban list.

**What this does not settle.** Which chain ships — the engine's own with the
method's rules applied to it is the proposal here, and the book's own table
chain and the author's own framing stay on the table; how old a race may be
before the card states its age (90 days is the prototype's); which race pace
anchors the recovery rule; whether the general-aerobic pace comes from the
history or from the athlete's hand; **what rule turns stored samples into
"your pace at 128–148 bpm"** — a median, a spread, a minimum sample size,
which is a new derivation and belongs in its own note; how wide the threshold
band is drawn; and whether the curve needs the 10-mile equivalent.

**A conflict to reconcile.** The heading *"Proposed amendment — the pace page
(2026-11-20)"* above describes a page reached from Progress, carrying the
training chart, the pace at a heart rate and the pace bands. The surface cut
(D7) retired the `/pace` page into the Progress panels. This amendment
supersedes that heading: the ladder and the curve are its successors, and the
heading should be retired rather than left standing beside its replacement.

### Proposed amendment — the threshold: four candidates, one choice, a dated fact (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The word "threshold" already lives in the product, and it lives there as the
**zone model's fifth-zone floor** — the load model reads it as the TRIMP
threshold. That is the watch's configuration, not the athlete's own number,
and the athlete has no way to state theirs. The design is
`exploration/threshold-setup.md`; the decisions are D19, D20 and D24.

**The line this design holds:** the app computes candidates, the athlete
chooses one, and the choice is stored as a dated fact with its source.

What changes:

- **Four candidates, and they are four sources rather than four rules.** The
  **watch's own zone model** (its fifth zone's floor — no typing at all) ·
  **from a race** (a band of paces, by the book's rule, "the pace you could
  race for about an hour") · **from a field test** (the book's 45-minute time
  trial, computed from the stored FIT) · **typed by the athlete** (from a lab
  or a coach; a **pair** is allowed — a pace band and/or a heart rate). Each
  candidate is computed by **one** rule and names it, and for the Pfitzinger
  base that rule is the book's. Friel's 30-minute test, Anderson's estimate and
  the 80/20 family's 20-minute test stay **named as published alternatives**:
  the card may say that other rules exist, and it does not compute four
  estimates and choose among them. The book's own fifth method (a heart-rate
  estimate, 82–90 % of maximum) is **not a fifth card** — it is the same kind
  of thing as the watch's own number and sits beside it.
- **A candidate carries the value, the source and the date.** Nothing else. A
  pace candidate is a **band**, drawn as a band with its two ends labelled and
  never collapsed to a midpoint, because a midpoint is a number the method
  does not state. The candidate's limitation is an annotation under the card,
  not a line on it.
- **Two lanes, and no conversion.** The candidates are also drawn as a
  heart-rate lane and a pace lane, each candidate a mark on its own lane. Two
  marks in one lane are two numbers about the same thing and the distance
  between them is drawn; two marks in different lanes are two kinds of number,
  and **no lane converts into the other**.
- **The athlete chooses; the app never chooses.** The candidates are listed
  side by side, in a fixed order that is not a ranking, with no chip naming
  one as the one to take and no colour that judges. The pick writes a choice
  row and marks that card as the one in use, and **the others stay exactly
  where they are** — choosing one is not deleting the others.
- **One threshold at a time, dated, and tied to the base it was made for.** A
  later change writes another row: the previous row stops being in force, is
  not deleted and is not edited. The history reads as a list of facts. The app
  **never warns** that a threshold needs retesting or that anything is stale
  — a date is a date.
- **A disagreement is shown, not resolved.** If the watch's fifth-zone floor
  says 165 and the typed number says 168, both marks stand on the lane at the
  two values; the app does not average them, does not pick between them, and
  does not call the distance between them a problem.

**The rule this serves: the watch's zone model and the athlete's threshold
are two different numbers, and the app never merges them.** The athlete's
threshold is read in **two places only** — the **plan's pacing** and the
**80/20 base's bands**. Everywhere else the app keeps the watch's own
numbers: not the ladder, which keeps the method's rule on the race time; not
the ledger, whose rows answer against the model in force on each run's day;
and not the load model, which keeps the watch's model as its reference, so
the load history does not move because the athlete chose a different number,
and the load model's own inputs are examined separately, in stage 2 of the
zone-model migration (D3).

**What it may and may not say.** It may name the value, the source, the rule
and the date. It may not say which candidate is right or which to take, and
may not say that any of them is old; it may not score, rank, recommend, or
colour a candidate as a problem; it may not compute a threshold or a maximum
heart rate from an age — there is no birthdate in Stride and none is added;
it may not convert a pace into a heart rate or the reverse; and it may not
collapse a pace band to a point. **A base may be adopted with no threshold
chosen**: then the plan's pacing falls back to the method's rule on the
plan's anchor race, the ladder shows the method's rung, and the threshold
card names the four ways a threshold could arrive — without a prompt to
choose and without a warning colour.

**The files it touches.** `crates/stride-core/src/threshold.rs` (new) ·
**a schema change**: a `threshold_observations` table in
`docs/contract/schema.sql`, holding the value or the band's two ends, the
source, the rule, the origin date, the base it was made for, the day the
choice was made and the in-force flag · **a contract change**: the
candidates route and the choice route in `docs/contract/api-paths.md` · the
catalogue's strings.

**What this does not settle.** Whether the setup card may be re-entered for a
second base, and what the second entry looks like beside the first; whether
the field-test candidate's window is the book's "last 30 minutes" of a
45-minute trial exactly, and what the app does when the pointed-at run is
shorter than the trial; and where the threshold's detail lives — a sheet on
the screen it was opened from, or a panel of its own.

**Three of the design note's open questions are closed by D20**, and this
amendment adopts them: the ladder's rung and the plan's session pace are
**never drawn together**, because putting them on one axis would imply they
are the same number; a typed value carries **two dates** — the day the
measurement was taken and the day it was entered — and the sentence names the
entered day while the detail shows both; and the "other rules exist" line
has its wording: *"One rule is used here: your pace and heart rate over the
last 30 minutes. Other rules for this test are published; they are not
computed here."*

### Proposed amendment — the two bases of zone time (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The zone model has no history in the source: Garmin serves the zone
configuration as it stands now — one record, no date, no effective-from. So
the rule that every run is measured with the model in force on the day of the
run leaves **every imported run** with no zone time, and the ledger's
intensity rows would be empty over exactly the athlete's history. The
measurement is in `exploration/zone-model-history.md`; the design is
`exploration/two-bases.md`; the decisions are D3, D10, D11, D18 and D23.

What changes — **two blocks on the ledger card, each naming its basis, and
never added together**:

| block | what it is | basis | where it comes from |
|---|---|---|---|
| **your zones** | the app's own zone time, in the athlete's own bands — Z1–Z2 low · Z3 moderate · Z4–Z5 high | **the moving time** — the time the athlete was running | the app's computation over the stored dated model, run by run |
| **the watch's own zones** | the watch's own record of each run — its zone 1 … its zone 5, five rows and no roll-up | **the elapsed time** — the whole run, stops included | `hrTimeInZone_1..5` on the activity-search payload the sync already fetches — **no extra request** |

- **One block per period.** The first answers the runs with a model in force;
  the second answers the runs without one. Their coverage does not overlap, and
  where the app holds the model the watch's minutes appear **only inside the
  check** — so no week ever carries two zone times that could be added.
- **The overlap is the third element, and it is a check.** Over the weeks
  where both exist, the app compares its own arithmetic with the watch's own
  record: the two numbers per week and in total, the difference as arithmetic
  ("3 minutes apart — one counts the time you were running, the other the
  whole run"), and what could account for it — the two spans, the boundaries
  the app does not hold, and the watch's own configuration of that day. **It
  is the app checking itself; the watch is the witness, not the defendant.**
- **The check may not say** which is right; that the agreement is accurate,
  consistent or inconsistent; that a difference is an error; that the model
  has not changed; or anything at all about the runs outside the overlap. It
  may not score the agreement, and it may not extend its result past the
  period it covers.
- **The names differ deliberately.** The first block's rows are the athlete's
  own bands, which the app holds the boundaries for; the second block's rows
  are the watch's own numbers, and the line under them says so — the
  boundaries of those zones are not recorded — because a reader who sees
  "watch zone 3 — 4 h 12 m" must not be left to assume the app knows what
  zone 3 was.
- **The picture separates them.** The app's own zone time is drawn as a solid
  fill, the watch's own minutes as a hatched fill, and the overlap as a
  marker strip over the weeks where both exist; a week the app cannot measure
  is an **empty cell**, a week with no runs a **blank** cell, and a measured
  zero a **light** cell. The two fills cannot be mistaken for two parts of one
  bar.

**What it may and may not say.** The card may state the two numbers, the
difference, the coverage, the basis of each block and the reason a value is
absent. It may not sum the two bases, put a row of one block in a column of
the other, declare a winner, grade the agreement, offer advice, or turn the
period the app cannot measure into a failure.

**The files it touches.** **a schema change**: the watch's zone minutes belong
in the `activity_observations` table — one row per activity per zone, with
the source, the endpoint it was read from, the source's own field name, the
value, the unit, the day the response carried it and the day it was read — in
`docs/contract/schema.sql`, with the matching edit to
`crates/stride-store/src/schema.rs`, which the schema contract test compares
statement for statement, and a schema version bump — each of the new tables is
its own bump, and the order among them is the implementation's to set · the
sync's search listing, which carries the values on the payload it already
reads, so **no extra HTTP request** and no widening of the pull's request
budget · the api's read seam in
`crates/stride-api/src/zonemodel.rs`, which gains a third case: no model in
force and the run carries the source's own zone time → answer with the
source's numbers, under the second basis, with the zone edges rendered as
unknown · **a contract change**: the fields in `docs/contract/api-paths.md`.

**What this does not settle.** Whether the check survives if the strict
reading is chosen instead — nothing from the watch before the first model, in
which case the check would cover the model period only; whether the first
block keeps a last-date row while its coverage is partial; whether the
watch's rows carry their own last dates; what the run detail does with the
same two-base question for a run before the first model; and the load model,
which is untouched — it keeps the model's own constants, and the second block
has no part in it.

### Proposed amendment — the picture first, and the sentence second (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The user's directive, 24 November 2026: **less verbose, more visual**. The
decision is D11; the sweep it drives is `exploration/cards-as-pictures.md`.

What changes:

- **A card is read as a picture first** — the shape, the band, the gap, the
  streak — and the sentence is the **caption** under it.
- **Absence is drawn, not described.** The twelve-week strip, coloured by what
  each week held, makes a five-week hole in the hard band visible as a hole
  before any sentence is read; the sentence then names the date. A band with
  no time in a week is **no value** — the cell is empty — and never a zero in
  a table.
- **The chart must be readable on its own**, and the caption must not repeat
  what the picture already says.
- **Two bars are not two facts.** The five zone shares and the polarised
  roll-up are **one bar, printed once**, and the ledger's rows read the share
  from it.
- **A surviving card that is only a sentence is redrawn as a picture, or
  merged.** This applies to the ledger card, the method-lens card (deferred,
  D13), the consolidated chart, and every card that survives the cut.
- **The accessible fallback is still a number**, and a number needs its period
  and its reason stated in words — which is what the caption is.

**What this does not change.** The numbers, the ban list, and the rule that
the app writes no sentence of its own: the core's catalogue owns the words. It
does not make the picture a decoration, and it does not remove the caption.

**The files it touches.** The frontend's surfaces and the chart code. The
ledger's own route and field table travel with the ledger's amendment. **No
contract change** — the shape of a card is a design decision, not an answer's
shape.

### Proposed amendment — explanation is not product content (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The user, after walking the end-to-end: *"overall i think great visuals. I am
seeing that we are adding text where it is not required."* The decision is
D22; the note is `exploration/rule-explanation-is-not-product.md`.

The rule, in one line: **the product surface carries the picture, its labels,
the numbers, the period, and — only where something is missing — the reason
something is missing. Nothing else.**

- **Every line that explains the design rather than the data moves to the
  annotation layer** — the notes, off by default in the clean view. That
  includes a header that explains the drawing; the reasoning behind two axes,
  which belongs as a small caption inside the chart's frame; a candidate's
  limitation; and any basis line that only explains the drawing.
- **The test.** If the line would still make sense on a card showing a
  different measurement, it is explanation and it goes. If it is the
  **period**, or the **source** where the source is the fact, or the **reason
  a value is absent**, it stays.
- **Explanation is moved, never deleted.** Every line that leaves a product
  surface appears as a note — or as a caption where it belongs with the
  picture.

**The files it touches.** The frontend's surfaces and the notes toggle; the
catalogue's strings, where a sentence that moved out of a surface may be
re-typed as a note. **No contract change** — `docs/contract/voice.md` freezes
the string types, the closed direction set and the ban list, and none of them
moves. This rule is a rule about what sits on a surface, and it is enforced by
review rather than by the lint.

**What this does not settle.** Whether a note may be edited by the athlete,
and whether the notes toggle is one switch for the app or a per-surface
control.

### Proposed amendment — the surface speaks the runner's language (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The user, on the language of the two-basis design: *"The language again — the
watch's own zone minutes, elapsed time. We need to work on everything like
that if we are showing that in the surface."* The decision is D24.

- **The notes may use the precise terms** — "elapsed time", "moving time", the
  zone model's floors, a field name — because a note is written for someone
  reading *about* the product.
- **The surface is written for a runner looking at their own training**: no
  field names, no model names, no jargon, no internal vocabulary, and no
  phrase that only makes sense to whoever wrote the code. So the two bases are
  named on the card as *the time you were running* and *the whole run, stops
  included*, and the zone model is *your zones*.
- **The test:** would a runner who has never read the code understand this
  line without it being explained? If not, it is too technical, too abstract,
  or it is explanation that belongs in a note.
- **Numbers and their formats are not touched.** The app's own conventions
  (`crates/stride-core/src/units.rs`) are the authority. **Words only.**

**The files it touches.** The catalogue's strings —
`crates/stride-core/strings.json` and `frontend/src/i18n/catalogue.ts`. **No
contract change** — `docs/contract/voice.md` keeps its seven string types and
its ban list; this rule governs the wording, which the lint cannot check, so
it lives in the plan and is enforced by review. If the user wants it
machine-checked — a new list of forbidden field names and model names in the
lint — that would be a contract change, and would need its own note.

**What this does not settle.** Whether a piece of jargon may be defined once
in the notes and then used on a surface, and the app's own word for the zone
model's maximum heart rate, which the surfaces have used inconsistently.

### Proposed amendment — the plan bases: Pfitzinger first, 80/20 second, one at a time (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The plan engine today knows two things about a plan: a **baseline** inferred
from the athlete's own history, and a **prescribed** plan imported or typed.
It has no method, no session vocabulary and no generated programme — and a
baseline is what the athlete usually does, not what they were told to do. The
plan's bases are D13, D14, D15, D16 and D18; the design is
`exploration/pfitzinger-spine.md` and `exploration/race-and-level-plans.md`.

What changes:

- **Two plan bases, one at a time.** **Pfitzinger is the first** — the plan's
  spine. **80/20 is a separate second method**, not a lens and not a variant:
  the two band their easy days differently (Pfitzinger against the athlete's
  maximum heart rate, 80/20 against the athlete's threshold), they run a
  different week (one hard session against two), and they pace from a
  different anchor (the athlete's race distance against the threshold). The
  athlete adopts one; **the app never recommends either**, and the plan card
  names the base it is following.
- **Two programmes ship: the 10K and the half marathon**, the book's tiers 1–2
  of each. The 5K and the 8K are cut, and so are the 15K/10-mile, the
  multi-distance plan, the higher tiers, the three base-training schedules
  and the marathon book's MP-relative rules. The method's **rules** still come
  from the whole book; only the plan **templates** are cut.
- **The plan's shape is the book's, read cell by cell**: twelve weeks, weeks
  to go counted 11 → 1 and then race week, a recovery week at eight weeks to
  go in every tier, one quality session a week whose type moves with the
  phase, the long run on Sunday, two tune-up races, and a recovery week and a
  transition week after the race. The app claims **no** weekly skeleton for a
  tier the spine has not read cell by cell.
- **The tier's entry requirement is stated beside the athlete's own weekly
  mileage, and the athlete picks.** The app never suggests a tier.
- **The plan carries the method's vocabulary** — the session kinds become
  endurance, general aerobic, recovery, progression long run, lactate
  threshold, VO₂max, speed, tune-up race and race — because the plan's kind is
  what the ledger's evidence is measured against.
- **The plan's pacing names the threshold it was paced with** — the value and
  the day it was chosen. A plan adopted in March and still running in July is
  paced by March's number, and the card says so. Sessions from the day of a
  change take the new band, and **sessions before it keep the band they were
  given** — the same discipline the zone model follows.

**What it may and may not say.** The plan card may name the base, the goal
race and its date, the tier's entry requirement, and the threshold it was
paced with. It may **not** say whether the athlete did what they were told:
the plan's cells are **intent** and the watch's zones and lap structure are
**evidence**, and no adherence score, no count of what was not done, no
quality verdict on a session and no measure of how far the training was from
the plan may exist anywhere. The two are never merged into one number.

**The files it touches.** `crates/stride-core/src/plan.rs` — a method value
on `PlanSource`, the book's run types on `SessionKind`, and a pace band and a
structure on the session · the store's `plan_sessions`, which already carries
`pace_low_s_km` and `pace_high_s_km` and has nothing filling them · a module
holding the two shipping templates · **a contract change**: the plan route's
field table in `docs/contract/api-paths.md`.

**What this does not settle.** The plan's **source word** — a generated
Pfitzinger plan is an intent and may say "not done", but it is not a file the
athlete imported, and a fourth source value is proposed; whether the 80/20
base's weekly structure is a template or a rule, and what its recovery week
looks like against Pfitzinger's; and what a plan carries when the athlete
adopts it with no threshold chosen — the pacing then falls back to the
method's rule on the plan's anchor race, and the session's pace is a no value
with a reason when the plan carries no race time either.

### Proposed amendment — the audience: the intermediate runner (2026-11-24)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The user's words: *"Mark this whole app for intermediate runners. For entry
level, Strava/Garmin is enough."* The decision is D14.

What the audience means for what the product offers:

- **The athlete already trains, has a base, and is chasing a specific
  distance.** So the product offers a **plan with a goal race and a date** (the
  10K and the half marathon), a **ledger that reads a window of history
  against that intent**, a **pace engine anchored on a race time the athlete
  has already run**, a **threshold the athlete can state** from their own
  watch, a race, a field test or a lab, and **two plan bases** to choose
  between. Every one of those starts from something the athlete already has.
- **The product offers no onboarding for a first run.** There is no
  "getting started" path, no first-week plan, no beginner's tier and no
  beginner's vocabulary: the session kinds are the method's, the plan's cells
  are the book's, and the ledger's rows assume a reader who already knows
  what a threshold session is.
- **The tier's entry requirement is stated, not enforced.** The plan card
  carries the requirement beside the athlete's own weekly mileage, and the
  athlete picks. The app does not turn a reader away, and does not suggest a
  tier.
- **The absence rows are for a trained reader.** "The last run above Z4 was 35
  days before the end of this period" is a fact about an athlete who already
  knows what a hard week is. The product attaches no interpretation to it —
  and no beginner's reading of it either.

**What this does not become.** Not a coach, not a marketplace, not a plan
recommender and not a scored product: the app still never says what to do, and
the audience does not license a verdict. The method lens — a catalogue of
coaching methods quoted over the athlete's own numbers — was designed and
**deferred** (D13); it stays in `exploration/` as the record of what was
designed.

**The files it touches.** No contract change: it is a constraint on the
plan's tiers, on the vocabulary and on what the product offers, and it
constrains the amendments above.

**What this does not settle.** Whether an athlete below a tier's entry
requirement is offered the base-training schedule as a note — the spine
extracted the book's three base schedules and does not build the templates —
and whether the app states the audience anywhere on the surface, which would
be a statement about the product rather than a fact about the athlete.

---

## 0. The voice — vitals, direction, options; no verdicts

Not a data dump, and not a set of orders. The app shows the vitals, says which way each has moved against the
athlete's own usual, and lists what is on the table. It draws no conclusion. Enforced in CI.

**1 · Vital — the measurement.** Label, value, unit. Nothing added: *Sleep — 6 h 10 m · HRV — 48 ms ·
Resting HR — 63 bpm.* No score, no band colour, no icon standing in for a word.

**2 · Direction — which way it moved.** A direction word from a **closed set** — *better than usual · about usual ·
could be better* — beside the muted arithmetic (*5 above your usual*) and the direction of travel (*3rd night down*,
*steady*). "Usual" is defined once and drawn on every chart: the median of the last 30 nights, band = median ±
1.5 × MAD. Which side is better is a fact about the measurement (more sleep, higher HRV, lower resting HR) — and
because the band is drawn, the athlete can disagree with it.

**3 · Options — what is on the table.** The plan's own sessions and alternatives, listed unranked and **never
conditioned on the vitals**. *Today — 10 km · easy · from your imported plan · Also in your plan — a 5 km on
Friday, a 30-minute one on Sunday.* The app never says which to pick, and never says "because your sleep was low" —
that link would be the judgement.

**4 · Pattern — the history, in the history screen.** What usually happened to this athlete, as a fact about the
past, always with its sample size: *"the 5 other times your sleep was below your usual, you ran 8.4 km or less."* It lives on
**Body**, over the visible history — not on the morning screen, and never next to a suggestion.

The Progress headline is the factual mirror of the evidence, in one of three wordings — *fitter than 8 weeks
ago*, *the same pace as 8 weeks ago*, *slower than 8 weeks ago* — with the evidence in a clause. It never
claims a change it cannot see.

**Forbidden:** commands (*take the day*, *rest day*, *rest up*, *rest today*, *go easy*, *you should*, *it'd be better to*) · verdicts (*good*,
*bad*, *poor*, *missed*, *behind*, a readiness score) · **conclusions** (the app never combines vitals into a
verdict — it reports each direction and the athlete reads the combination) · ranking the options (*recommended*) ·
shame (red, *should have*).

```
TODAY · Tue 12 May
Rest         could be better · 2 of 3 signals below your usual
            (the count names the side the signals are literally on — "above" when a resting heart rate is the one
             on its worse side, "outside" when they disagree; the sentence must be true as written)
Sleep        6 h 10 m   · could be better · 1 h 10 m less than your usual
HRV          48 ms      · could be better · 14 below your usual · 3rd night down
Resting HR   63 bpm     · about usual · 5 above your usual
Today        10 km · easy · from your imported plan
Also in plan a 5 km on Friday · a 30-minute one on Sunday
Last run     8.4 km · 42 m · Sat 10 May
```

Every user-facing string lives in one catalogue and is typed (`vital`, `direction`, `option`, `pattern`, `status`).
The lint fails the build on banned words (`should`, `must`, `take the day`, `rest day`, `rest up`, `rest today`, `go easy`, `missed`, `bad`,
`poor`, `fail`, `behind`, `ready to`, `recommended`, `better to`, `need to`), requires a `direction` string to come
from the closed set, and requires a `pattern` string to carry its sample size.

| old product | v2 |
|---|---|
| "Take the day, or make it very easy." | "Rest — could be better · 2 of 3 signals below your usual", and the plan's options, unlinked |
| "Keep it easy today." | "Sleep — 6 h 10 m · could be better · 1 h 10 m less than your usual" |
| "You missed Tuesday's 10 km." | "Tuesday — 10 km planned · not done" |
| "0.62 km over your target." | "10.6 km — 0.6 km more than the 10 km target" (and only when the gap is over 10%) |
| Readiness 63/100 | no score exists |

## 1. The product in one page

Stride is a private, local-first running log for one athlete. It shows you **your own numbers against your own
normal**, in plain language — and leaves the decision with you.

- **Loop:** Plan → Execute → Understand → Adapt.
- **Promise:** "Your data stays on this phone." One credential — a Garmin Personal API key — in the Android
  Keystore on Android. No Stride account or hosted Stride data store; desktop
  uses a local HTTP server and Garmin pulls require a network.
- **Four destinations, at every width:** Today · Runs · Progress · Body. Compare and the Plan week are
  reached **contextually**, never as extra tabs.
- **The rule that kills drift:** *if a screen cannot exist on the phone, it does not exist.* The desktop is the
  same four destinations with more room; it never grows features of its own.

## 2. Scope

**In:** Connect (Garmin key) · Sync (incremental) · Today · Runs · Activity detail · Run film · Progress · Body ·
Plan week · Compare · FIT import (any watch) · Settings (goal, pause, note, connect, about).

**Out (deferred, with reasons):** Garmin account login/MFA and the scheduled-workout calendar · MCP/agent
harness · running-form fingerprint page · gear · heatmap · tiles proxy · monthly stats table · HR-zone
pages · network egress routing · phone plan editor · desktop-only screens.

## 3. Architecture

```
stride-core    pure domain: plan engine, vitals & direction, story, fitness, compare   (no I/O, no clock)
stride-ingest  Garmin Personal API client · FIT decoder · segmentation
stride-store   SQLCipher schema, migrations, repositories, FIT blobs
stride-api     the one router · DTOs · the FFI surface · the trust gate
stride-testkit frozen golden corpus · parity differ · UI fixtures
desktop        one binary: embedded SPA + router + trust model
android/…/stride-ffi   UniFFI bindings, panic catching, JSON in / JSON out
```

- The dependency graph is a DAG: core ← ingest, core ← store, api → {core, ingest, store}. No cycles.
- **One core, two hosts:** the phone reaches it in-process through the FFI; the desktop reaches it over loopback
  HTTP. Neither can disagree about a number.
- **FIT is the source of truth.** The original file is stored; derived series are recomputable.
- **Views are computed on read** from stored primitives — a changed engine needs no re-sync, and a stored score
  can never rot.
- **The old Rust core is the oracle** during the migration; Python is retired and its outputs frozen as JSON
  fixtures first.

### The engine contract

| engine | contract |
|---|---|
| vitals & direction | never a 0–100 score, never a grade, never a recommendation — a value and a direction word from a closed set, with the band drawn on the chart |
| pattern | the athlete's own history of comparable states; always carries its n; never from fewer than three cases; lives on Body, not Today |
| today's state | recorded weekly activity first, then the plan and available body observations. No commands, verdicts or link between an option and a vital. Today and Body read the same **values and directions** from the core |
| plan | plain fact: *done* · *not done* · *planned*. Never a red "missed". An inferred baseline never yields "not done" |
| story | descriptive, never evaluative; reads the same primitives the charts read; the feed shows the sentence, the detail shows it plus the why |
| fitness | no trend percentage without a real baseline; consistency is stated, never graded |
| compare | the basis is always stated; swapping A and B flips every delta |

## 4. The screen system — one IA, every width

**Bands:** compact `<600` (bottom tabs) · medium `600–1023` (top bar, column capped 680 px) ·
wide `>=1024` (left rail, content capped at 1240 px with page-specific columns).

Bands are decided by **container** width, not viewport — split-screen, unfolded foldables and resized windows
all land correctly without a reload.

**Reflow rules:** destination set constant · no page-level horizontal scroll · tap targets ≥44 px at compact ·
safe-area insets honoured · `dvh` not `vh` · no device sniffing for layout (a build flag only for genuinely
native capabilities).

**Viewport matrix gate** (six widths, every PR): 360×640 · 390×844 · 430×932 · 768×1024 · 1024×768 · 1280×800.
Each asserts: no horizontal scroll, no clipped labels, tap targets, **identical destination set**, charts
legible, safe-area applied.

## 5. Screens

The original specifications remain in `docs/plan.html`; the approved amendments
and [current media](docs/media/README.md) supersede their layout and old status claims.

Today · Runs · Activity · Run film · Progress · Body · Plan week · Compare · Connect · Settings — each with purpose, what
it must never do, content blocks, layout per width, states, and acceptance checks.

**Today** leads with recorded weekly activity and the named latest run, then
the plan and available body observations. Vitals retain their values, closed-set
directions and arithmetic; unavailable signals do not become fabricated values.
The **distance comparison appears only when the gap exceeds 10%**, stated neutrally ("2.4 km more than the 10 km
target"); within 10% it is absent — 10.2 km against a 10 km target is simply not news, and neither more nor less is
good or bad.

**Compare is the canonical form to get right:** metric rows (`metric · A · B · Δ`) at every width; two columns
and side-by-side charts only when there is room; an A/B chart toggle on narrow screens; the basis always named.

## 6. Security and trust

- Garmin key: native field → non-exportable Keystore key → `noBackupFilesDir`. Never in the WebView, the
  database, or a log. The 24 h token is disposable: a 401 triggers exactly one re-exchange and one retry.
- Database: SQLCipher, key from the caller, never logged or API-returned.
  Android wraps it with Keystore; desktop persists its live key in a private file.
  Every query is parameterised.
- **Desktop trust model:** loopback always allowed · explicitly trusted CIDRs via
  `STRIDE_TRUSTED_NETWORKS` · everything else refused with a message naming the variable. No pairing codes, no
  QR, no device tokens — ever.

## 7. Open-source posture

**AGPL-3.0.** Public repository **from day one** (no history scrub later — the old repo's history contains
personal data from the demo-seed incident). `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`,
`SUPPORT.md` ("issues welcome, features are mine").

CI gates from commit #1: cargo test + clippy (deny `unwrap`/`expect`/`panic`) · **corpus integrity** (FIT
SHA-256) · **secret scan** · **privacy scan**. Golden parity, frontend checks and
the responsive viewport matrix are implemented in `scripts/gate.sh`.
The APK/core/signature checks run in the release pipeline; desktop trust-model
tests run with the Rust workspace.

> The old repo's `android.yml` is **not** fork-safe — its APK job needs the signing secret, so every outside
> contributor's build fails, and it grants `contents: write` to pull requests. v2: `contents: read` globally,
> secrets only in the release job, release only from upstream tags.

The current release ships a signed APK with release notes. Desktop can be built
from source with the checked-in shared UI bundle. Demo videos and screenshots
are release assets, not additional application binaries.

## 8. Delivery plan

This is the original implementation sequence. It is not the current backlog;
see the status and approved amendments above for remaining acceptance boundaries.

| Slice | Content | Done when |
|---|---|---|
| 0 | Contract freeze: public repo, corpus extracted + sanitised, fork-safe CI, schema/API frozen | **done** — merged |
| 1 | Thin vertical line: key → sync → one run → today's readout → APK | a real key syncs a real run and the app shows its true numbers |
| 2 | FIT decode + segmentation + store | golden parity on every corpus activity |
| 3 | Vitals & direction engine + pattern engine + string catalogue + voice lint + Body | Today and Body render the same values and directions; the lint forbids every imperative, verdict and ranking word |
| 4 | Plan engine (sources, active block, adherence, results) | adherence matches the old core number-for-number |
| 5 | Runs feed + activity detail | every row carries a story sentence |
| 6 | Progress + fitness + bests | no percentage without a baseline |
| 7 | Compare + FIT import + dedupe | deltas flip on swap; a duplicate import adds no row |
| 8 | Settings, goal, pause, notes, export/import | the desktop export opens on the phone |
| 9 | Responsive pass + viewport matrix gate | all six widths pass; identical IA everywhere |
| 10 | Release: signed APK, binaries, docs, first public tag | a stranger clones, builds and runs it |

Rules: the old core is the oracle until parity is 100% · one slice at a time · every slice ships and is gated ·
Rust owns the domain engine; host tooling can use Python or shell.
Acceptance is a golden assertion wherever possible.

## 9. Defaults applied

AGPL-3.0 · public from day one · old repo kept private as the oracle, sanitised public archive at the end ·
Rust core with a shared React UI and native host boundaries · desktop local host
(loopback + CIDR allowlist) · four destinations
everywhere with Compare and Plan contextual · Garmin account flow out · MCP out · "issues welcome, features are
mine" · **voice: vitals, direction (closed set), options — no verdicts** · **distance comparison only beyond a 10% gap**.

**Still open to change on request:** MIT instead of AGPL · anything from the cut list · the destination names ·
the plan-source set · a native phone UI (a separate project, not v2).

---

## 11. Historical post-v0.9 planning notes

This section records the plan made after the v0.9.0 vertical line. Its slice
table is retained as history, not a claim that those implemented features are
still missing from rc.3. The ownership and honest-state rules remain applicable.

### Two rules this stretch earned

**The plan engine has one owner: the core.** `crates/stride-core/src/plan.rs` holds the baseline inference,
the week, the adherence, the results and the distance comparison. The api reads the plan out of the store
and hands it to the core; the frontend renders what the core rendered. The reason is written down because it
has been learned three times in this project: a client that retried *and* a shell that retried; a frontend
that carried a port of `units.rs`; two FFI surfaces. **A rule that lives in two places is a rule that will
drift.**

**The screen inventory is complete; what was missing was the states.** All nine screens of §5 exist. The
states each must cover:

- **empty** — no runs, no wellness, no plan: the screen says what is missing and what to do, in the app's
  voice, with no invented numbers;
- **not enough data** — the honest form of the empty state, where a comparison is impossible;
- **stale** — the data is older than it should be, and the screen says when it was last synced;
- **the plan's states** — a baseline plan, an imported plan and no plan at all, each named as the source it is;
- **a baseline is not a prescription** — a plan inferred from the athlete's own history never produces a "not
  done"; it is what they usually do, not what they were told to do;
- **an unrecognised session is unclassified**, never labelled "easy" by guesswork.

### The delivery plan from here

| slice | what | done when |
|---|---|---|
| **1** | the vertical line — key → sync → one run → today's readout → the phone | **done** (merged; released as v0.9.0) |
| **2** | the plan engine in the core: the baseline, the week, the adherence, the results, the 10% rule | the baseline infers a week from a synthetic history and says how many weeks it used; a thin history yields nothing |
| **3** | the api's plan, settings, notes, import and export — all on the core's engine | every route in the frozen table answers; the import refuses what it cannot read; the shapes are written into the contract |
| **4** | the app's screens: Settings, Connect, and the states on every screen | the viewport matrix passes at six widths; the lint passes; no screen invents a number |
| **5** | the phone's first run, its notifications, and the key's whole life | the key goes in, the pull lands the runs, the app says so; "forget the key" keeps the history |
| **6** | the rig: the connect page, the export, the import, the stranger's README | a stranger clones it, runs it, gives it a key and sees their own runs |
| **7** | the second review, and the release that carries it | an independent review of the whole product, and a release an Instinct can install |

Slices 2 to 6 run in parallel where they touch different crates. 3 and 4 meet at the shapes — the app's
declarations are the contract, and the api is held to them — and 5 and 6 are independent of everything but the
router.
