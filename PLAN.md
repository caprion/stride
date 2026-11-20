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

**In:** Connect (Garmin key) · Sync (incremental) · Today · Runs · Activity detail · Progress · Body ·
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

Today · Runs · Activity · Progress · Body · Plan week · Compare · Connect · Settings — each with purpose, what
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
