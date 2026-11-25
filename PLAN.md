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

### Note — the four insight routes, and the decode cache they read through (2026-11-24)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **Four routes were added**, and the phone's
copy of the route table travels with them.

* **The four routes.** `/insights/paces` is the **ladder** — the training
  paces, each with the rule it comes from, its citation, its heart-rate
  band and the anchor its numbers hang on. `/insights/pacing-curve` is the
  **cost curve** — the share of VO₂max against the time it is held for,
  with the athlete's own races as points, the method's own sessions as
  bands, the field test's marker and the pace equivalents.
  `/insights/threshold` is the **four candidates**, the choice in force
  and the dated history. `/insights/stimulus` is the **ledger** — the
  weeks × bands matrix, the last occurrence of each kind, the coverage,
  the longest run, the repeated work, and the two bases with the overlap.
* **One window for all of them.** Each takes the Progress window's own
  `weeks` parameter and **names the window it answers for**. The ladder's
  own numbers do not depend on the window — the paces come from the
  athlete's own race, and a race is a fact about a day — but the answer
  still names the window it is shown over.
* **The source line is hoisted.** Every answer carries **one** line for
  the screen to name — the zone model's own line, with its read day — and
  the lines it was built from travel behind a click, in `sources`. The
  lines are the core's own words; the api asks the core's catalogue for
  them and passes them on.
* **The decode cache.** Every route that reads a run's file decoded it
  again on every read, and the ledger reads about sixty files. **The
  decode now happens in one place** — `routes/series.rs::decoded` — and
  the decoded file is held in a **bounded cache** on the `Api`: the run's
  id and the file's own SHA-256 are the key, the cache holds sixty-four
  decoded runs by default, and the oldest is thrown away when it is full.
  A file that is re-imported has a new digest, so the old decode is never
  served for it. **Only the decode is cached**: every derived series is
  still recomputed on read, from the decoded file.
* **The read is bounded.** The ledger reads the newest **sixty-four**
  runs — which is what the cache holds — so the second read of the same
  window costs no decodes at all. A store that holds more runs than that
  says so in `more_runs`, and the rows cover the runs that were read.
* **What the ledger cannot read yet.** The **repeated-work row** is a no
  value: the watch's own workout steps are not read from the FIT file
  yet, and the corpus's lap field numbering is the project's own
  convention, so the step name's field was not guessed at. The row names
  the reason, and the reason is the core's own.
* **The phone's table.** The bridge carries a copy of the frozen route
  table, and a route the router serves and the copy does not name is a
  **404 on the phone**. The copy names the four new routes, and the
  bridge's test says so.
* **The app reads them now.** The frontend step drew the ladder, the cost
  curve, the threshold and the ledger, and the app reads the four routes.
  The contract's list of the routes the app does not read was corrected
  with that step.

### Note — corrections to the plan of record (2026-11-25)

These are **corrections, not proposals**. Each one is a decision the user has already made, and
the file was carrying an older answer. Nothing here changes the product's behaviour; it makes the
plan of record say what is true.

* **Five destinations, not four** (D8). The code ships five: Today · Runs · Progress · Body ·
  **Profile**. The profile keeps a door of its own, and the surface cut sizes both readings to
  the same panel count, so this is a placement decision and not a size decision. §1 and §9 said
  four and now say five. The older amendments that say *"the four destinations stay the four"*
  are corrected by this note; their text is kept as history.
* **Seven string types, not five** — the contract's, in `docs/contract/voice.md`. The four
  registers are `vital` · `direction` · `option` · `pattern`; the three supporting types are
  `statement` · `status` · `instruction`. §0 listed five and left out `statement`, which
  carries the sentences that state a fact about a comparison, and `instruction`, which is the
  product's own instruction to a model and the only kind the app never shows.
* **The ban list is the contract's twenty-three, word for word**
  (`crates/stride-core/src/strings.rs`, `BANNED_PHRASES`). §0 carried sixteen and was missing
  seven: `good`, `keep it easy`, `red`, `readiness score`, `not recovered`, `you are ready` and
  `you are not ready`. The core's own comment says why there is one list: *"two lists is how a
  lint ends up passing a string the contract bans."*
* **§2's scope is corrected in both directions.** The **heatmap** and the **monthly table** are
  no longer out — the heatmap is computed on the device (its own amendment) and the year carries
  the months (the year-as-film amendment). The **athlete profile** is in, and so are the things
  this week's build brought: the **training paces** and the **cost curve**, the **threshold**
  and the athlete's own races, the **stimulus ledger**, and the **plan programmes**.

### Note — the adoption, and the plan that is in force (2026-11-24)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **Two routes were added** and one was
extended, and the phone's copy of the route table travels with them.

* **The two routes.** `POST /plan/adopt` is the athlete **adopting a
  programme**: the body names the programme, the tier, the plan's first
  day, the goal race's distance and the day it is run, and the threshold
  to pace with — a candidate, or a number the athlete brought. `GET
  /plan/adopt` reads back the plan that is in force: its programme, its
  tier, its dates, what it was paced with and the threshold it was paced
  with, and the tier's entry requirement **beside the athlete's own
  weekly mileage**.
* **The paces are worked out once, at the adoption** (B4), from the
  threshold **in force on the day of the adoption**, and they are stored
  with the plan. A later threshold does not re-pace a running plan, and a
  later correction to the catalogue cannot either: the sessions are the
  dated copy (B1).
* **The refusals**, each with the core's own reason: a programme or a
  tier that is not in the catalogue, a start date that leaves no room for
  the twelve weeks and the race week, a goal race the programme does not
  print, and a plan that is already in force — **one plan at a time**
  (D13/D14).
* **What is not a refusal:** the athlete's own weekly mileage being below
  the tier's entry requirement. The requirement is **stated beside** their
  mileage (B2) and the athlete chooses; the app never suggests a tier and
  never blocks one.
* **A plan may be adopted before a race is typed.** Its sessions carry
  their kinds and their distances and **no paces**, with the reason, and
  the plan records that it has no pacing yet. The paces are worked out
  **once**, when there is something to work them out from.
* **`/plan/week` now carries the adopted week's own sessions**: their
  kinds, their distances, their paces where they have one, and the plan's
  own source. The adopted week is `null` when no plan is in force, or when
  the week asked for is not in it.
* **The phone does not need a screen for this.** The route is in the
  bridge's table and in its test; the pace page is the precedent — a
  route with no screen. **The app reads the two routes now**: the
  adoption is the app's own page, `/plan/bases`, and the plan in force is
  read there and on the plan week.

### Note — the athlete's own facts, and the programmes they can adopt (2026-11-24)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **Three routes were added**, and the phone's
copy of the route table travels with them.

* **The athlete's own facts.** `/athlete/races` reads the races the
  athlete typed and writes new ones; `/athlete/threshold` makes the
  choice of a threshold. **They are never synced**: they are entered in
  the app and stored locally, in the encrypted database, and they are the
  only things the athlete enters about themselves. Nothing here is
  inferred from the training — a race the athlete did not type is not a
  race, and a threshold the athlete did not choose is not one.
* **The same discipline the wellness observations have.** Every write
  carries **the value, the day it was entered, the day it last changed,
  and the source**.
* **The choice names its candidate, and the value comes from it.** The
  body names which candidate the athlete picked — the watch's own zone
  model, a race, a field test, or a number they brought — and the base
  the choice was made for. The value is the candidate's own value, so the
  choice and the candidate cannot disagree; a typed choice carries the
  value itself.
* **One threshold is in force at a time.** The earlier choices stay in the
  dated history with their base named, and a plan adopted in March is
  paced by March's number. A choice **with no value is refused**, and so
  is a write that would leave two choices in force.
* **The closed lists.** The race's **label** names the distance, and the
  distance comes from it — the app never invents a distance for a race it
  cannot name. The **candidate**, the **typed number's label** and the
  **base** are closed lists too.
* **`/plan/bases` answers with the programmes the app can adopt**: the
  Pfitzinger 10K and half-marathon templates, with their entry
  requirement, their length and their weeks. A programme is an intent, not
  a baseline, and **the app never suggests one**: the tier's entry
  requirement is carried beside the athlete's own mileage, and the athlete
  picks. **The templates' data has landed**: the catalogue holds the four
  programmes, and the list is no longer empty. An empty catalogue is still
  a no value with a reason, and never a programme that is not there.
* **The app reads two of the three now.** The threshold is the app's own
  screen — Settings → Training preferences — and it is where the choice is
  made; the programmes are the adoption page. **The races are still not read
  by the app**: the race is typed on the rig's own page, and the write goes
  through the api's own route.

### Note — the storage for the adopted plan (2026-11-24)

This note travels with a change to `docs/contract/schema.sql`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **The schema version moves from 6 to 7**, and
two tables are added. No route changed in this step, and no shape changed.

* **The adopted plan** (`adopted_plans`). An adopted plan is a **dated
  copy** of a programme from the catalogue (B1), so the row keeps the
  programme's id **and the version** it was adopted from, the tier the
  athlete chose, the plan's first day, the goal race's own distance and
  the day it was run, the day the plan was adopted, and the record of
  **what it was paced with**: the threshold's value and the day it was
  read, the source it came from, and the base it was made for. A plan
  adopted before a race was typed carries its sessions **without paces**,
  and says why: `paced` is 0 and the reason is in the row.
* **The plan's own sessions** (`adopted_sessions`). The day, the method's
  own kind, the distance, the pace where the session has one, the **rule**
  the pace came from and the **threshold** it came from, and the **detail**
  the book prints for that day ("3 × 8 min, jog 3"). A session with no
  pace is a session with no pace: the pace's ends are null, never zero.
* **Why the sessions are stored and not derived.** The plan's paces were
  worked out **once**, at the adoption, from the threshold in force that
  day (B4). Derived on read, a later correction to the catalogue — or a
  new threshold — would silently re-pace a plan that is already running.
  The dated copy is what makes "a later threshold does not re-pace a
  running plan" true.
* **One plan is in force at a time.** A plan is in force from its first
  day to its last, and a second plan cannot start while another one is
  running. The store reads the plan in force on a day, and the adoption
  route refuses a second one with the core's own reason.
* **The export and the backup carry every table**, with their counts in the
  manifest, and an export of an older schema is still refused. The plan is
  the athlete's own fact, held locally in an encrypted database.
* **One bump.** The two tables are read and written together, so they ship
  in one step: a database at version 6 takes version 7 and gets both
  tables and their indexes, in one transaction, and a database that already
  ran version 7 changes nothing.

### Note — the desktop rig's own page, and the three decisions it offers (2026-11-25)

The rig is the app's face on the athlete's own machine, and step 6 of the rig-v2 build gave it a
landing surface of its own: `/rig`. **It is the host's face, not the app's.** The app is the
athlete's face; the rig is the machine it runs on, and the page says what that machine is.

* **The four parts.** The **index** — the screens the rig serves, the new ones included: the
  ladder, the cost curve, the ledger, the threshold and the programmes — the **status** (the
  version being served, the database's schema version, what the log holds, and where the first
  run stands), the **storage** (the folder, the database and its size, and the files the keys
  live in), and the **connection** (whether a Garmin key is kept, and when the log was last
  filled from the source).
* **Facts, never verdicts.** Every line is a fact about the machine or about what the athlete
  has already decided. Nothing is scored, ranked or graded, and **the key never reaches the
  page**: it says *whether* a Garmin key is kept and where the file is.
* **The three decisions.** The new screens need three things that cannot be worked out from the
  training — **a race to type, a threshold to choose, a programme to adopt** — and the rig's
  page offers them as the athlete's own choices: **side by side, unranked, and never as a
  recommendation**. The vitals are never linked to them, and nothing on the page presumes any
  of them: a fresh rig leaves all three empty, names each one, and asks for nothing.
* **Where each choice is made.** The **race** is typed on the rig's page — the app has no screen
  for it, and the desktop host already owns the first-run page — and the write goes through
  `stride-api`'s own route, so the race is the app's race and the refusals are the route's own
  words. The **threshold** and the **programme** are made in the app, where the four candidates
  and the four programmes live, and the page says which screen, and what is in force now.
* **The tools moved.** The rig's own tools — the export, the import and the FIT import — are at
  `/rig/tools` now, and the app's **Data & connection** link points there. The page is the rig's
  landing surface; the tools are one part of it.
* **No route changed.** The product's route table is unchanged, and the phone's bridge table is
  unchanged: the rig's page is the host's own, outside the frozen API prefix, and the phone has
  no such page. The new screens are served by the rig the way every other screen is served —
  with the app.
* **The bundle check.** `scripts/check-desktop-bundle.sh` holds `desktop/assets` to the build of
  `frontend/`, so a rig can never carry an app that is one commit behind the source. The build
  is deterministic, and the check writes the bundle the way the sync script writes it, so there
  is one rewrite and not two.

### Note — the plan bases' own card, and the routes the app reads now (2026-11-25)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **No route changed, and no field changed.**
What changed is the wording of two passages and the state of one answer.

* **The mileage row is absent when the athlete has no runs.** The plan
  bases' card carries the level's own entry requirement and the athlete's
  own mileage **side by side, unranked**, and the mileage is a
  **nullable** field: `athlete_mileage` is `null` when the athlete ran in
  none of the last four weeks. There is then no mileage to set the
  requirement beside, so the answer carries no mileage row at all and the
  card shows the requirement alone. **With no runs there is no mileage to
  compare against, and that is a fact and not a failure**: no runs is a
  fact about the athlete's history, and the app states it by leaving the
  row out, never by saying the mileage is missing.
* **The catalogue is not empty.** The four programmes are read cell by
  cell from the book's own tables and they are in the catalogue; the
  passage that said the list was empty until they landed is corrected. An
  empty catalogue is still a **no value with a reason** in `unavailable`,
  never a programme that is not there.
* **The list of the routes the app does not read is corrected again.** The
  app draws the ladder, the cost curve, the threshold, the ledger and the
  two plan routes, and it makes the threshold's choice. The list keeps
  every route it already named, and `/athlete/races` stays on it: the race
  is typed on the rig's own page, and the write goes through the api's own
  route. **No route changed** — the table above the list is the table, and
  what was corrected is the prose about which of its routes the app reads.

### Note — the day's four parts, and the labels that say when (2026-11-24)

This note records a change to a **stated fact** — the cut points of the
day and the words the app reads for them. **No frozen file changed:**
`/insights/rolling` answers with the same four rows and the same fields
(`key`, `label`, `from_hour`, `to_hour`), and the contract's field table
does not name the cut points. What changed is **where the day is cut**,
and **what the label says**.

* **The reason.** The athlete, on their own history: *"another sort of a
  bug is that night time is taken as 23:00 to 6:00. A lot of my runs
  would have started at around 5ish. So everything gets reported to night
  time. We should move night to 21:00 to 4:00 am. And label it that
  way."* The athlete is an early-morning runner, and a night that began
  at 23:00 reported their five-o'clock runs as night runs.
* **The four parts, and the hours.**

  | part | was | is |
  |---|---|---|
  | morning | 06:00 | **04:00** |
  | afternoon | 12:00 | 12:00 |
  | evening | 17:00 | 17:00 |
  | night | 23:00 | **21:00** |

  The night **wraps around midnight**, so it runs from 21:00 to 04:00 —
  seven hours — and the morning from 04:00 to 12:00 — eight. The
  partition follows from the four constants, and the hour a run **started
  in** is still the only thing that places it.
* **The label carries the window.** "Morning (04:00–12:00)",
  "Afternoon (12:00–17:00)", "Evening (17:00–21:00)" and
  "Night (21:00–04:00)". The hours are **rendered from the cut points**
  and are never typed into the wording, so the name of a part and the
  part itself cannot drift apart.
* **The calendar day does not move.** A run at 05:10 still belongs to
  that calendar day — for the load, the ledger and the plan — and it is a
  *morning* run in the day's four parts. **Only the four parts changed.**
  Sleep and wellness are read by the calendar day, and they are untouched.

### Note — the ledger covers the window, and names the right reason (2026-11-25)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **One field was added** (`overlap_missing`),
and the ledger's read and its reasons changed.

It supersedes the Note of 24 November, *"the four insight routes, and the
decode cache they read through"*, on one point: **the read is bounded by
the window, and not by the cache.**

* **The reason it was found.** The athlete's own **twelve months** finally
  arrived — 105 runs, from 25 November 2025 to 25 November 2026 — and the
  ledger answered for **64** of them, because the ledger read the newest
  sixty-four runs: the decode cache's own capacity. The cap was an
  implementation fact, and it was deciding what the answer covered.
* **The cache is a cache.** The ledger now reads **every run the window
  holds**. The runs the cache holds are served from it, and the runs it
  cannot hold are decoded on the read and decoded again on the next one —
  `routes/series.rs::decoded_uncached` is the same decode without the
  keeping. **The cache may make the second read cheap; it may never decide
  what the first read includes.** The cache's own bound is unchanged and
  still explicit (`decode_cache_capacity`, sixty-four by default).
* **The coverage line describes the window.** It counts the runs the
  window holds, the runs that carried heart rate, the runs the app could
  measure and the runs that carried the watch's own zone minutes.
  `more_runs` now means exactly one thing: **the store holds runs in this
  window that the answer does not cover** — and it is false whenever the
  store covers the window.
* **The last occurrence keeps its rule.** The last date a kind occurred is
  searched over the athlete's **whole retained history**, not over the
  window alone. The search reads the older runs newest first and **stops
  as soon as every kind has been found** — an older run cannot change the
  answer — and a run with no model in force is not read at all, because it
  holds no band.
* **Four reasons, not one.** The app's own bands can be missing for four
  different reasons, and the route says which one it is, in the core's own
  words: the runs carry no heart rate; no zone model has been read at all;
  **a model was read, but none of it was in force for these runs**; a model
  was in force and the run carries no heart rate. The third is the case the
  two bases exist for — *your watch's own zone minutes are the only zone
  time before your watch's configuration was read* — and the card says so
  in the runner's language, not in a failure's language.
* **A run with no heart rate carries no measurement.** A run whose model
  was in force but which recorded no heart rate is **not measured**: the
  time in a band would be a zero where there is no measurement at all.
* **An empty overlap is a fact to state.** The overlap — the period where
  the app can check its own zone arithmetic against the watch's own record
  — is a **no value with a reason** now, in `overlap_missing`, in the same
  shape as the repeated-work row's. An empty overlap is stated, not hidden.

### Note — the athlete's declared heart rates are the bands (2026-11-25)

This note travels with a change to `docs/contract/schema.sql` and to
`docs/contract/api-paths.md`, which the contract freeze says is a plan
change: a frozen file changes only with a plan note in the same change.
**The schema version moves from 7 to 8**, one table is added, two routes
answer a new question, and the rule behind every zone-time answer changes.

* **The athlete's declaration.** One small fact, entered once: the
  **maximum heart rate**, the **resting heart rate**, or **both** — each
  with where it came from (a closed list of five: *from my watch · from a
  test · from a doctor or a lab · from my training partner's watch ·
  typed*), the day it was measured (optional) and the day it was entered.
  A pair entered together is **one declaration with two rows**; a later
  pair is a **new declaration**, and the earlier one stays in the history
  with its own dates and its own sources. **One declaration is in force at
  a time** — the most recent — and the earlier ones stay.
* **The declared pair is the bands** (D27). The **rule**: Garmin's
  published five-zone ladder — 50 %, 60 %, 70 %, 80 %, 90 % — applied to
  the range the declaration supports:
  * **both numbers** → the **heart-rate reserve** (maximum − resting):
    `resting + p × (maximum − resting)`;
  * **a maximum alone** → the **share of the maximum**: `p × maximum`;
  * **a resting rate alone** → **no bands at all**, and the answer says
    why, in the core's own words.

  Each floor is rounded to the nearest beat per minute, halves up, so two
  runs of the app on the same number give the same bands. The bands are
  **never stored**: they are recomputed on read from the declaration in
  force.
* **The citation.** Garmin publishes the ladder — the *Forerunner 245
  Owner's Manual*, "About Heart Rate Zones" → "Heart Rate Zone
  Calculations" — and publishes the two ranges it can be expressed on: the
  maximum heart rate, and the heart-rate reserve (maximum minus resting).
  The app applies **the same published percentages** to the range Garmin
  defines, so there is **one rule with two ranges**, not two rules.
* **Two facts, never blended.** The **watch's zones** (the floors
  configured in the watch, read once and dated) and the **bands from the
  heart rates the athlete entered** are two dated facts. Every answer
  **names the basis it used**, the two are **never mixed inside one
  answer**, and the difference between them is stated as a fact — boundary
  by boundary — and never as a correction.
* **The change to Z's rule.** The rule above — *every run is measured
  with the model in force on the day of the run* — **changes**: where a
  declaration exists, **the declared bands are applied to every run**, so
  the series is comparable across the whole window; the watch's own model
  is used where one was in force and **no declaration exists**. The seam
  is in one place (`crates/stride-api/src/zonemodel.rs`), and the zone
  computation takes the floors it is given and does not care where they
  came from.
* **What does not move.** The **watch's own zone minutes** (the source's
  own record of each run), the **watch's configured floors** (the watch's
  own fact), the **load model** (it keeps the watch's dated model, and
  moving it is its own decision with its own announcement), the **plan's
  session bands** and the **threshold** are all unchanged.
* **The app.** Settings › Training › **Your heart rates**, beside the
  threshold: the two values, each with its source and its dates, the bands
  they produced, the watch's own floors beside them, and the history. And
  the **rig's first-run page** — *"three things only you can give it"*
  becomes **"four things only you can give it"**: a race to type, a
  threshold to choose, a programme to adopt, **your heart rates**. Side by
  side, unranked, every one of them can stay empty, and **no vital is
  linked to any of them**.

### Note — the week against the plan (2026-11-25)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **No route is added** — `/plan/week` answers
a new question in a new block — but the shape of an answer is part of the
frozen table, and the table says what the answer carries.

* **The question.** The rig's review said the Progress wall mixes load
  ratios, streaks, sleep stages and weekday comparisons **without answering
  whether the week's training fits a goal**. The app now answers it: **how
  is this week, against the plan?**
* **The answer is a fact.** Each planned session is *done* or *not done* —
  the plan engine's own statuses — and for the sessions that happened, the
  distance run is read against the distance planned and the pace against the
  plan's pace. **No grade, no percentage, no traffic light**, and the word
  "missed" is not in the app.
* **The paces are compared only when both of them exist.** The general
  aerobic day carries a heart-rate band and no pace, so that session is
  compared on its distance or on nothing; a run is never compared against a
  pace that is not there.
* **The comparison is like for like.** The days that have gone are compared
  with the days that have gone — the plan's first three days against the
  athlete's first three days, never the plan's whole week against a part of
  one.
* **One plan.** The athlete's own plan is compared with the week that
  happened. The app never compares two plans, never says which plan is
  better, and never offers a programme the athlete did not adopt.
* **The card is merged into *This week*.** The wall is eleven cards and the
  instruction was to reduce the surface: the two questions are the same
  question asked of two other things, so the answer lives inside the card
  that already answers it. The panel behind the card, the Plan page and
  Today carry the same answer in full.
* **The core owns it.** `crates/stride-core/src/plan_week.rs` reads the
  athlete's own adopted plan and the week that happened, and writes every
  sentence; the api carries the answer and the app draws it.

### Note — the hours are the machine's own, and the athlete may state theirs (2026-11-24)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **One field was added** (`calendar`), and the
hours the product reads moved from UTC to the machine's own zone.

* **The reason.** The athlete's own schedule card on the desktop rig read
  *"Night 23 -> 6 · 93 sessions · 884.6 km"*, and *"Evening 17 -> 23 · 0
  sessions"*. Ninety-three night sessions and no evening ones is not a
  boundary that was wrong: **the hours were in UTC**. The rig never told the
  router which zone it was in, so every "local hour" was the UTC hour — and
  a run at **05:00 IST** is **23:30 UTC**, which falls in the night bucket
  under any boundary at all.
* **The rig reads the machine's zone.** The offset is read from the
  operating system when the rig starts and **again before every pull** —
  the way the phone already refreshes its calendar before a sync — so a
  machine that changes zone (a flight, a daylight-saving change) does not
  leave the hours behind. **The athlete's own day never moved**: it comes
  from Garmin's own local date, so the load, the ledger and the plan have
  always bucketed the run correctly. Only the *part of the day* was
  computed in the wrong zone, and that is why the athlete's own history
  looked impossible.
* **The athlete may state their own offset.** Settings › Training carries a
  zone field beside the heart rates: the machine's own value is the
  default, the athlete's own is in force over it, and **the screen says
  which of the two is in force** — "Your hours are read in this machine's
  time" or "in the time you stated". It is a fact about the machine and
  never a verdict on the athlete.
* **The day's four parts are the athlete's own hours.** The cut points
  are the morning at **04:00**, the afternoon at 12:00, the evening at 17:00
  and the night at **21:00**; the night wraps around midnight, so it runs
  from 21:00 to 04:00. The label carries the window it covers — "Morning
  (04:00–12:00)", "Night (21:00–04:00)" — and the hours are rendered from
  the cut points, never typed into the wording. **The calendar day does not
  move**: a run at 05:10 belongs to that calendar day, and it is a morning
  run in the day's four parts.

### Note — the run's own zones say when they begin to apply (2026-11-25)

This note travels with a change to `docs/contract/api-paths.md`, which the
contract freeze says is a plan change: a frozen file changes only with a
plan note in the same change. **One field was added** to the run's zones
answer, and one reason was brought into the register the other reasons
use.

* **The run's own page says when the zones begin to apply.** The screen's
  source line already said it — *"the watch's configuration as it stood on
  25 Nov 2026 · the runs from that day on"* — but a reader had to visit
  another screen to learn it. The run's own zones answer now carries
  **`apply_from`**, in the core's own words: **"your watch's zones apply
  to the runs from 25 Nov 2026"**. The field is **absent** when no model
  has ever been read: there is no such day, and the reason the answer
  carries stands as it is.
* **The reasons are one register.** `zones.no_model` — no zone model has
  been read at all — read *"No zones yet — your watch's heart-rate zones
  have not come through"*, which was the old vocabulary. It now reads
  like its siblings: **"not measured — your watch's zones were not read,
  and the watch's own zone minutes are the only zone time here"**. The
  ledger's own test asserted the old sentence, and that one expectation
  was changed with it; **the ledger's behaviour did not change**.
* **The ban list and the closed direction set are untouched.** No string
  was added to either, and no string was reworded in a way that reads as a
  verdict: a reason still says what is missing and never what to do
  about it.

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

**Retired (2026-11-25).** Superseded by *"the fingerprint as observations on the run detail"*
below, and by the Note of 24 November, *"the zone model in force on the day of the run"*, which
is the rule this heading anticipated. Kept as history.

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

### Proposed amendment — the fingerprint as observations on the run detail (2026-11-17)

**Retired in part (2026-11-25).** The bullet *"the model is the current one"* is retired: every
run is measured against **the zone model in force on the day of the run**, which is the Note of
24 November, *"the zone model in force on the day of the run"*, and the load model's own move to
the dated model is stage 2 of that migration (D3). The rest of this amendment stands.

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

**Retired in part (2026-11-25).** The bullet on the window's basis is retired: the window is the
last *N* **calendar weeks** through today, and the period it is weighed against is **the same
span ending seven days earlier** — not the same number of days ending yesterday (D1, D2). See the
Note of 24 November, *"one window for the whole wall, and one comparison"*. The rest of this
amendment stands.

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

**Retired in part (2026-11-25).** The bullet on the comparison period is retired: the year is
weighed against **the same span ending seven days earlier**, like every other window-shaped
answer (D1, D2), and not against the same number of days ending a day earlier. See the Note of
24 November, *"one window for the whole wall, and one comparison"*. The rest of this amendment
stands.

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

**Retired (2026-11-25).** Superseded by *"the pace engine: the chain, the ladder, and the cost
curve"* below: the `/pace` page was retired into the Progress panels by the surface cut (D7),
and the ladder and the cost curve are what the page became. The heading is kept as history.

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

**What changed since the draft (2026-11-25).** The plan's own sessions on
Today now carry **the pace the plan was worked out with**, and the pacing is
named **once**, above the sessions — the threshold and the day it was read —
because a pace with no threshold beside it is a number with no owner. A
session with no pace says why and shows none.

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

**The files it touches.** `crates/stride-core/src/stimulus.rs` — the ledger,
beside the other window-shaped analyses: `ledger()`, `Ledger`, `KindRow`,
`LastOccurrence`, `Coverage`, `AppBasis`, `SourceBasis`, `Overlap`, `Bases`,
`LongestRun`, `RepeatedWork` · the api's read path,
`crates/stride-api/src/routes/stimulus.rs`, which decodes each run's stored FIT
on read through the shared decode cache · `crates/stride-ingest/src/fit.rs`,
for the lap message's definition 24 and the `workout_step` message · the
Progress wall's card and one line on the run detail · **a contract change**:
the route `/insights/stimulus` and its field table in
`docs/contract/api-paths.md` · the catalogue's strings, `stimulus.rule.*`,
`stimulus.steps.no_data`, `stimulus.no_runs`. Nothing is stored — every value
is recomputed on read, like every other derived value.

**What changed since the draft (2026-11-25).** The module and the route
exist: the two bases and the overlap are in the same answer. The core's
`WorkoutStep` carries the FIT's step codes, but **the ingest does not read
the step field yet** — the corpus's lap field numbering is the project's own
convention, so the field was not guessed at — and the row answers **no value
with its reason**, `stimulus.steps.no_data`. It stays blocked on **a real
workout file from the athlete's watch**: no file in the corpus carries a
workout step, and until one is recorded and synced the row has little to say,
which is the correct answer and not a defect. The window the ledger reads is
`rolling::Window::calendar_weeks(weeks, today)` — the Progress window, the
same one the totals and the pace page draw — and the read is **bounded**: the
newest **sixty-four** runs, which is what the decode cache holds, with a
store that holds more saying so in `more_runs`. **The card ships**: the
ledger is the Progress wall's **Where the time went** panel, and it draws
the two blocks — the app's own zone time and the watch's own zone minutes —
with the overlap under them, and a band a week held none is white space.

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
  its own route** (`/insights/pacing-curve`), linked from the pace card — not a
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

**The files it touches.** `crates/stride-core/src/vo2.rs` — the effort model:
`effort_from_race()`, `equivalents()`, `cost_curve()`, `riegel_time()`,
`pace_for_vdot()`, `time_for_vdot()` · `crates/stride-core/src/training_paces.rs`
— the ladder: `ladder()`, `Ladder`, `Rung`, `RungKind`, `PaceBand`,
`HeartRateBand`, `Anchor`, `MissingInput`, and the `RULE_*` constants that
carry each rung's rule and its citation · `crates/stride-core/src/units.rs`,
which gained the renderer that reads a finish time as a clock · **a schema
change**: the `race_observations` table in `docs/contract/schema.sql` — the
value, the day entered, the day last changed, the day of the race, the
athlete's own label and the anchor flag · **a contract change**: the routes
`/insights/paces` and `/insights/pacing-curve` and their field tables in
`docs/contract/api-paths.md` · the catalogue's strings, `pace.chain`,
`pace.source.*`, `pace.rule.*`, `pace.missing.*`.

**What changed since the draft (2026-11-25).** The engine's names are **`vo2.rs`
and `training_paces.rs`** — two modules, not one `pace_engine.rs`: the effort
model and the ladder are separate, and the ladder reads the effort model. The
cost curve's route is **`/insights/pacing-curve`**, not `/insights/pace-curve`.
The race is stored with the **label the athlete gave it** — a closed list of
six distances — and the distance is the label's own; the anchor is a flag, and
**entering another race never moves it**. The ladder takes the athlete's
maximum and resting heart rates as **inputs it may not have**, so a rung with
no band says so and the other rungs still answer. **The ladder and the curve
are screens now**: the ladder is the Progress wall's **The pace** card, the
curve is its own panel behind it (`/progress/curve`), and the weekly
pace-at-heart-rate chart stays on the pace panel.

**What this does not settle.** Which chain ships — the engine's own with the
method's rules applied to it is the proposal here, and the book's own table
chain and the author's own framing stay on the table; how old a race may be
before the card states its age (90 days is the prototype's); which race pace
anchors the recovery rule; whether the general-aerobic pace comes from the
history or from the athlete's hand; **what rule turns stored samples into
"your pace at 128–148 bpm"** — a median, a spread, a minimum sample size,
which is a new derivation and belongs in its own note; how wide the threshold
band is drawn; and whether the curve needs the 10-mile equivalent.

**A conflict, now closed.** The heading *"Proposed amendment — the pace page
(2026-11-20)"* above described a page reached from Progress, carrying the
training chart, the pace at a heart rate and the pace bands. The surface cut
(D7) retired the `/pace` page into the Progress panels. That heading is
**retired** — it is kept as history, and it says so — and the ladder and the
curve are its successors. The **route** the page read went with it: see
*"the pace page's route is retired"* below.

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

**The files it touches.** `crates/stride-core/src/threshold.rs` — `candidates()`,
`Candidate`, `ThresholdSource`, `ThresholdBase`, `TypedSource`,
`field_test_estimate()`, and the `RULE_WATCH`, `RULE_RACE` and
`RULE_FIELD_TEST` rules · **a schema change**: the `threshold_observations`
table in `docs/contract/schema.sql` — the value or the band's two ends, the
source, the rule, the base, the day the choice was made, the day it last
changed and the `in_force` flag · **a contract change**: the route
`/insights/threshold`, and — when the write route lands — the athlete's races
and the threshold choice in `docs/contract/api-paths.md` · the catalogue's
strings, `threshold.rule.*`, `threshold.limit.*` — the limits, which are
annotations and not surface lines — `threshold.missing.*`, `threshold.in_use`,
`threshold.history`.

**What changed since the draft (2026-11-25).** The module and the read route
exist: `crates/stride-core/src/threshold.rs` and `/insights/threshold`, which
answers with the four candidates, the choice in force and the dated history.
The field-test candidate computes **the book's 45-minute trial** and reads the
last thirty minutes of it (`FIELD_TEST_MINUTES`, `FIELD_TEST_WINDOW_S`). The
candidates are **computed and never stored**; the choice is stored, and the
core keeps the **base** it was made for beside it — `ThresholdBase`. The write
routes landed with the step: **`/athlete/threshold`** makes the choice, and
**`/athlete/races`** reads and writes the races — and they are in the frozen
route table, with the Note of 24 November, *"the athlete's own facts, and the
programmes they can adopt"*. A choice with no value is refused, and so is a
second choice that would leave two in force. **The card ships**: the
threshold is the app's own screen, Settings → Training preferences, where
the four candidates stand side by side in two lanes, the athlete picks one,
and the choice in force and the dated history are named with their dates.
The note that says what a candidate cannot know is **not on the card** — that
is the explanation, and the explanation is not product content.

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

**The files it touches.** **a schema change**: the `activity_observations` table in
`docs/contract/schema.sql` — one row per (activity, source, field, item), with
the value, the day the response carried it, the day it was read and the
source; the watch's own zone minutes (`hrTimeInZone_1..5`, over the **elapsed**
time) are in it · the sync's search listing, which captures the source's own
fields from the payload it already holds, so **no extra request** — a test
compares the request counts · `crates/stride-core/src/stimulus.rs`, where
`AppBasis`, `SourceBasis`, `SourceZoneRow` and `Overlap` hold the two blocks
and the check · the api's read path,
`crates/stride-api/src/routes/stimulus.rs` · **a contract change**: the fields
in `docs/contract/api-paths.md`.

**What changed since the draft (2026-11-25).** The table is
**`activity_observations`** and it holds more than the zone minutes: the
source's own facts about a run, in the shape `wellness_observations` already
uses. The sync captures them from the **search payload it already holds**, and
the test that compares the request counts is the proof that the pull's budget
did not grow. The api's read seam is the ledger's own route rather than the
zone model's, and it reads the source's zone minutes **by their source's own
field name**. Garmin's estimates — the VO₂ max, the training stress score,
the activity training load, the lactate threshold, the intensity factor, the
stress and the body battery — are **not stored**. **The card draws the two
blocks**: the app's own zone time and the watch's own zone minutes stand
under their own headings, each with its own basis, and the overlap reads
underneath them — the app checking itself against the watch's own record,
and not a verdict on either.

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

**What changed since the draft (2026-11-25).** Nothing in the rule, and the
build is an instance of it: the ladder, the cost curve, the four candidates in
two lanes and the weeks matrix are all pictures with a caption, and the app's
own screens come with the frontend step — the rule binds them when they are
drawn.

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

**What changed since the draft (2026-11-25).** Nothing in the rule, and the
build applied it: the candidates' limits are already **annotations**
(`threshold.limit.*`), the two-axis reasoning is a caption inside the frame,
and every answer carries **one** source line for the screen to name, with the
lines it was built from behind a click.

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

**What changed since the draft (2026-11-25).** Nothing in the rule, and the
build follows it: the zone model is **"your zones"**, the two bases are named
by their basis in the runner's words, and the catalogue is
`crates/stride-core/strings.json` and `frontend/src/i18n/catalogue.ts`.

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
- **The 80/20 base is a spec, and not yet a second base.** Its schedule has
  **not** been read cell by cell, and a published 80/20 week is written in
  **minutes** where the book's is written in distances — so there is no
  80/20 template in the catalogue and the app holds one base at a time. Its
  threshold would be a **lactate-threshold heart rate**, and that is a fifth
  candidate source the app does not have. The amendment *"the plan bases: a
  versioned catalogue the athlete adopts"* below records what a second base
  would need before it could be one.
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

**What changed since the draft (2026-11-25).** The four programmes are now
**data** — `exploration/plan-templates.json`, transcribed cell by cell from the
spine — and the spec is `exploration/plan-bases-spec.md`. The spec settles
five things, and the amendment below carries them: the catalogue is
**versioned** and an adopted plan is a dated copy; the **tier** is the
athlete's choice and the entry requirement is stated beside their own
mileage; the **race date** anchors the twelve weeks; a session's paces are
computed **once, at adoption**; and a race day's printed miles are the day's
mileage, not the race's distance. The core holds the **shape** of a
programme — `PlanBase`, `bases()` — and the route `/plan/bases` answers with
the programmes the app can adopt. **The catalogue is no longer empty**: the
templates are in the crate — `crates/stride-core/src/plan_bases.json`, read
with `include_str!` and parsed when it is first needed — and the four
programmes are in the answer. **The adoption is built**: `POST /plan/adopt`
adopts a programme and `GET /plan/adopt` reads back the plan in force, the
store's own tables are schema **version 7**, and the spec settles one thing
this amendment had proposed: an adopted programme is a **prescription** with a
**base** beside it, not a fourth `PlanSource` value. **The plan's own week
reads the dated copy**: the adopted week's sessions carry the runner's wording
for the kind, the method's own name beside it, the distance and the pace, and
the card says what the plan was paced with — the threshold and the day it was
read, or the plan's own rule and the race it was read from. The card does not
say whether the athlete is keeping up, what the level suits them, or what to
do next.

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

**What changed since the draft (2026-11-25).** Nothing in the rule, and the
build is consistent with it: the four programmes are the 10K and the half
marathon, at the levels the spine read cell by cell, and the entry
requirement is **stated** beside the athlete's own mileage rather than
enforced.

**What this does not settle.** Whether an athlete below a tier's entry
requirement is offered the base-training schedule as a note — the spine
extracted the book's three base schedules and does not build the templates —
and whether the app states the audience anywhere on the surface, which would
be a statement about the product rather than a fact about the athlete.

### Proposed amendment — the storage: three tables of dated facts, and the plan's own rows (2026-11-25)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The build added the storage the new answers read: **schema version 6** with
three tables, and **version 7** with the adopted plan's own two. The Notes of
24 November record what shipped and the contract changes they travelled with;
this amendment records the **rules** that storage obeys, which are the user's
to approve.

What the three tables hold, and the rule each one follows:

* **`race_observations` — the athlete's own race.** The distance, the finish
  time, the day of the race, the day it was entered, the day it was last
  changed, the **label** the athlete gave it, and the **anchor** flag. The
  rule it serves: **a race time is an observation, not a verdict.** The
  athlete types it; the app never invents one, never infers one from the
  training, and never moves the anchor by itself. **At most one row carries
  the anchor**, and the distance is the label's own — the app never invents a
  distance for a race it cannot name.
* **`threshold_observations` — the athlete's own threshold choice.** The value
  or the band's two ends, the **source** (the watch's zone model, a race, a
  field test, or typed), the **rule** that computed it where one applied, the
  **base** it was made for, the day the choice was made, the day the row was
  last changed, and the `in_force` flag. The rule it serves: **one threshold
  at a time, dated.** The earlier choices stay in the dated history with their
  base named, and a plan adopted in March is paced by March's number. The
  **candidates are computed and never stored**; only the choice is kept.
* **`activity_observations` — the source's own facts about a run.** One row
  per (activity, source, field, item), in the shape `wellness_observations`
  already uses: the value, the day the response carried it, the day it was
  read, and the source. The rule it serves: **absence is not zero.** A field
  the source never sent has **no row**; a field it sent as null has a row
  whose value is null; never a zero for "no time".

And three rules that hold across all three:

* **The athlete's own facts are never synced.** The races and the threshold
  choice are typed into the app and stored locally, in the encrypted
  database. The sync never reads them and never writes them.
* **The source's own facts come from a payload the sync already holds.** The
  watch's own zone minutes, the training effect and the athlete's own
  recorded effort ride on the **activity-search response** the sync already
  fetches to discover the runs — **no extra request**, and a test compares
  the request counts so the pull's budget cannot grow by accident.
* **Garmin's estimates are not stored.** The VO₂ max, the training stress
  score, the activity training load, the lactate threshold, the intensity
  factor, the stress and the body battery are **judgements**, and the
  contract excludes them.

**And the plan's own two tables, at version 7**, with the rule each one
follows:

* **`adopted_plans` — the dated copy of a programme.** The programme's id
  **and the version** it was adopted from, the tier the athlete chose, the
  plan's first day, the goal race's own distance and the day it is run, the
  day the plan was adopted, and the record of **what it was paced with**: the
  threshold's value and the day it was read, the source it came from, and the
  base it was made for. A plan adopted before a race was typed carries its
  sessions without paces, and says why: `paced` is 0 and the reason is in
  the row. The rule it serves: **a running plan keeps the catalogue it was
  adopted from.** A later correction to the catalogue cannot alter it, and a
  later threshold does not re-pace it either — the paces were worked out
  **once**, at the adoption.
* **`adopted_sessions` — the plan's own dated sessions.** The day, the
  method's own kind, the distance, the pace where the session has one, the
  **rule** the pace came from and the **threshold** it came from, and the
  **detail** the book prints for that day ("3 × 8 min, jog 3"). The rule it
  serves: **a session with no pace is a session with no pace.** The pace's two
  ends are null, never zero, and the rule and the threshold beside it say
  where the pace came from.
* **One plan is in force at a time.** A plan is in force from its first day to
  its last, a second plan cannot start while another one is running, and the
  write archives the plan that was in force **in the same transaction** as
  the new one — so a database can never hold two plans in force.

**Why the sessions are stored and not derived.** The paces were worked out
once, at the adoption, from the threshold in force that day. Derived on read,
a later correction to the catalogue — or a new threshold — would silently
re-pace a plan that is already running. The dated copy is what makes "a later
threshold does not re-pace a running plan" true.

**What it may and may not say.** Nothing: the storage holds facts and their
provenance, and the sentences that name them are the catalogue's. A stored
value never carries a verdict, a freshness rule, or a warning that it is old
— a date is a date.

**The files it touches.** `crates/stride-store/src/schema.rs` and
`docs/contract/schema.sql` — version 5 → 7, the five tables and their
indexes, in one transaction each · `crates/stride-store/src/repos/races.rs`,
`threshold.rs`, `activity_observations.rs` and `adopted.rs` · the sync's
search listing in `crates/stride-ingest` · the export's table list,
`crates/stride-store/src/export.rs`, which copies every table and counts it
in the manifest.

**What changed since the draft (2026-11-25).** The build settled the order of
the bumps: the three tables ship in **one** step, version 6, because they are
read and written together — so the one-bump-each rule this amendment was
drafted against is superseded by the Note of 24 November — and the adopted
plan's own two tables ship in the **next** step, version 7, because a plan
and the sessions that belong to it are written in one transaction. The **write
routes** landed with the first step — `/athlete/races` and
`/athlete/threshold` — and the Note of 24 November, *"the athlete's own
facts, and the programmes they can adopt"*, is their record; the **adoption**
landed with the second, and the Note of 24 November, *"the storage for the
adopted plan"*, is its record. The export carries **every** table with its
counts in the manifest, and an export of an older schema is still refused.

### Proposed amendment — the decode cache, and what it does not cache (2026-11-25)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

Every route that reads a run's file decoded it again on every read, and the
ledger reads about sixty files. The build put the decode in one place and
gave it a bounded cache.

The rule this serves: **a derived value is recomputed on read and never
stored** — and the cache does not weaken it, because the cache holds the
**decoded file**, not a derived value.

What changes:

- **One decode per run, in one place.**
  `crates/stride-api/src/routes/series.rs::decoded` is the only reader of a
  stored FIT file, and it keeps the decoded file in the api's **bounded
  cache**: the key is the run's id **and the file's own SHA-256**, so a file
  that is re-imported has a new digest and the old decode is never served for
  it. The cache holds **sixty-four** decoded runs by default
  (`decode_cache_capacity` on the api's configuration) and the oldest is
  thrown away when it is full.
- **Only the decode is cached.** Every derived series — the samples, the
  laps, the zones, the fingerprint's features, the ledger's rows, the
  ladder's rungs — is still **recomputed on read**, from the decoded file. A
  changed engine needs no re-sync and a stored number can never rot: that is
  the whole point of the rule, and the cache is not an exception to it.
- **The read is bounded, and says so.** The ledger reads the newest
  **sixty-four** runs — the cache's own capacity — so the second read of the
  same window costs no decodes at all. A store that holds more runs than
  that says so in `more_runs`, and the rows cover the runs that were read.

**What it may and may not say.** The cache is an implementation fact and
never appears on a surface. The bound is not a truncation the athlete has to
be told about in words: it is a fact about the answer's coverage, and the
answer carries it in `more_runs` for the surfaces that draw a coverage line.

**The files it touches.** `crates/stride-api/src/routes/series.rs` — the one
decode · `crates/stride-api/src/state.rs` — the `DecodeCache`, its capacity
and its statistics · `crates/stride-api/src/routes/stimulus.rs` — the bound,
`MAX_LEDGER_RUNS`.

**What changed since the draft (2026-11-25).** The names are the ones the
build uses: the one decode is `routes/series.rs::decoded`, the cache is the
`DecodeCache` in `state.rs` with `decode_cache_capacity` on the api's own
configuration, and the ledger's bound is `MAX_LEDGER_RUNS` in
`routes/stimulus.rs` — **sixty-four**, the cache's own capacity. The cache
keys on the run's id **and** the file's SHA-256, so a re-imported file is
never served from the old decode, and it keeps the decoded file only: every
derived series is still recomputed on read.

**What this does not settle.** Whether the capacity should follow the window
rather than sit at sixty-four, and what the ledger does with a window that
holds more runs than the bound — today it reads the newest sixty-four and
says so in `more_runs`.

### Proposed amendment — the plan bases: a versioned catalogue the athlete adopts (2026-11-25)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The four programmes are **data** — `exploration/plan-templates.json`,
transcribed cell by cell from the spine — and the spec is
`exploration/plan-bases-spec.md`. The catalogue, the adoption and the dated
copy are built; this amendment records the rules they follow, so that the
user approves them.

The five decisions the catalogue and the adoption follow:

- **B1 — The template is a versioned catalogue; an adopted plan is a dated
  copy of one of them.** The catalogue may be corrected later; a plan that is
  running keeps the template's id and version it was adopted from, so a later
  correction cannot alter it. The adopted plan's own **snapshot** of the entry
  requirement travels with it, for the same reason.
- **B2 — The tier is the athlete's choice, and the app states the entry
  requirement beside the athlete's own mileage and never suggests one.**
  Three facts side by side, unranked: the level's requirement as the book
  prints it, the plan's own mileage band, and the athlete's own last four
  weeks. No comparison of the athlete to the book, no colour on a number, no
  advice.
- **B3 — The race date anchors the plan; the race week is the twelfth week
  and the goal race is the book's day for that template.** The start date and
  the race date must agree, or **adoption is refused** with the difference
  stated — a refusal is a fact with a reason, the precedent being the plan
  import. A race on another day shifts the race week alone, and the shift is
  stated as a fact.
- **B4 — A session's pace is computed once, at adoption, from the threshold
  in force then; the plan stores the bands it was given and the threshold's
  value and date.** **A later threshold choice does not re-pace a plan that
  is running** — a threshold chosen on 3 December does not change the pace on
  a session dated 10 November, and the card still names the threshold the
  plan was paced with, with its date. Re-pacing is an **explicit act**: the
  athlete asks, the bands are rewritten, and the new threshold's value and
  date are recorded with them. With no threshold chosen, the method's rule on
  the plan's anchor race paces the plan, and the card says which of the two
  paced it.
- **B5 — A race day's printed miles are the day's mileage in the weekly
  total, not the race's distance.** The session's target distance is the
  race's own distance; the printed number stays in the volume check.

The three objects, and what the core would own:

- **The template** — the programme, undated: its id, its label, its method,
  its goal distance, its length, its **entry requirement**, its mileage band,
  its goal race and its weeks, each week carrying the seven days, each day
  carrying the method's kind, the distance in miles and in kilometres, and
  the cell's own text.
- **The week** — the weeks counted down from 11 to 1, then the race week,
  then the weeks after the race. The **printed** weekly mileage, and the
  reconciliation against the sum of the seven days. **All fifty-four
  week-rows of the four programmes reconcile**, and the check stays in the
  data: **no cell is ever adjusted to make a week sum** — a week that does not
  reconcile is reported, with the gap, and the cells stand as the book prints
  them.
- **The session** — the day. The app's own kind is what the surface prints,
  and the **method's kind travels as data**, because the ledger compares the
  plan's kind against the watch's evidence. Two names, one day, never both
  leading: the runner's wording is the label, the method's name is the title
  beside it, and the cell's own parenthetical is the sentence under it.

**What the plan card shows, and what it may not say.** The base, the level,
the week's sessions with their distances, their details and their pace bands
where there is one, the pacing's threshold with its date, the entry
requirement beside the athlete's own mileage, the week's own countdown, and
the week's volume. It may **not** say whether the athlete is keeping up,
whether the level suits them, what to do next, or anything about a threshold
they have not chosen. The plan's cells are **intent** and the watch's
evidence is **evidence**, and the two are never merged into one number.

**The 80/20 base is a spec, and not yet a second base.** Its schedule has
**not** been read cell by cell, and it is published by level rather than by
the book's shape: a published week is in **minutes**, not distances, so a
template in the same shape needs a **duration on the session**, which the
core's `PlanSession` has no field for today. Its bands are set from the
**threshold** — seven zones, two of them deliberately avoided — where the
app's zones come from the watch; its entry requirement is in **hours**, not
miles; **its threshold candidate is an LTHR** — a lactate-threshold heart
rate — which would be a fifth source with a rule of its own, and the four
candidates the app computes do not carry it; and its recovery weeks fall
every third week, which the Pfitzinger templates do not have. **None of these
weeks is invented**: there is no 80/20 template in the catalogue, and the app
holds one base at a time.

**The files it touches.** `crates/stride-core/src/plan_bases.rs` — the
catalogue, the adoption, the pacing assignment and the card's rendering ·
`crates/stride-core/src/plan_bases.json` — the catalogue's own data, read
with `include_str!` and parsed when it is first needed ·
`crates/stride-core/src/plan.rs` — `PlanBase` and `bases()`, where the
programmes are held · `crates/stride-store/src/schema.rs` and
`docs/contract/schema.sql` — the schema's bump to **version 7**: the
`adopted_plans` and `adopted_sessions` tables carry the dated copy, the
template's id and version, the tier, the pacing's threshold and the
sessions' own kinds, paces and details · `crates/stride-api/src/planbases.rs`
and `crates/stride-api/src/adoption.rs` — the catalogue's answer and the
adoption's seam · `crates/stride-api/src/routes/plan.rs` — the catalogue route
`/plan/bases` and the adoption route `/plan/adopt`, which needed a note in
the same change because the route table is frozen · the catalogue's strings.

**What changed since the draft (2026-11-25).** The core holds the shape of a
programme — `PlanBase`, with the template's id, its label, its method, its
goal distance, its **entry requirement** and its length — and the route
**`/plan/bases`** answers with the programmes the app can adopt. **The
catalogue is no longer empty**: the templates' data is in the crate —
`crates/stride-core/src/plan_bases.json`, read with `include_str!` — and the
four programmes are in the answer; an empty catalogue is still a **no value
with a reason**, never a programme that is not there. The data itself was
written in `exploration/plan-templates.json`, and the spine's transcription
was corrected and re-read by row-index alignment, so **all fifty-four
week-rows reconcile** — and the rule behind that check stays: *no cell is
ever adjusted to make a week sum*. **The adoption is built**: `POST
/plan/adopt` adopts a programme and `GET /plan/adopt` reads back the plan in
force, the paces are worked out **once**, at the adoption, and the **dated
copy** is stored — the plan and its sessions go in one transaction, and the
store's tables are version 7. The frozen route table took the adoption write
route in the same change as the catalogue's read route, and the plan's week
is read from the dated copy, not from the catalogue.

**What this does not settle.** Whether the session gains a duration field now,
for the 80/20 base that needs it, or only when that base lands; whether the
method's kind is a column of its own or is folded into the session's title
and details — the ledger needs it as data, and the surface can carry it as a
title; and whether the 10K template, whose race is already on the Saturday,
needs a shift rule of its own for a Sunday race.

---

### Proposed amendment — the step-5 cards, and the three answers that changed (2026-11-25)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

**The pace card on Progress is now the ladder.** The card that answered
"the pace" with the training week by week, the pace-at-heart-rate cloud and
the pace bands answers with the **ladder** instead: the anchors, the rungs
with their values, their bands and their rules, drawn on two axes — the
heart rate on the left, the pace on the right — and captioned with the race
the paces came from and the chain that produced the number.

- **The card is the picture** (D11). The rungs are drawn; the values, the
  bands and the labels are on them; the caption names the race and the
  chain. The screen **hoists one source line** — the zone model and the day
  it was read — and the lines behind it travel behind a click.
- **The explanation is a details affordance and nowhere else** (D22). A
  rung's rule opens beside it, with its citation, and the rule's wording is
  the core's own. No note layer, no paragraph under the card.
- **The colours say whose number each one is**: the method's amber and the
  athlete's own azure. **No colour marks a problem**, and there is no score,
  no grade and no traffic light.
- **Absence is drawn.** A rung with no pace is a hollow marker with the
  reason written where the pace would have been; a ladder with no race draws
  its axes and says what it needs. **A missing value is never a zero.**
- **The pace page's own panels go with the route they read.** The
  pace-at-heart-rate *chart* stays — it is the weekly median the Progress
  answer already carries — and the cloud and the bands, which only the
  `/insights/pace` route answered, are retired with it.

**The cost curve is the drill-through, on its own panel.** The curve is a
**second reading** of the same athlete and not a second ladder: no rungs and
no pace bands repeated. It is reached from the pace card and drawn on its
own panel, `/progress/curve`.

- **The method's heart-rate bands are never drawn on the %maxVO₂ axis.**
  Converting one into the other needs the athlete's lactate relation, which
  the app does not have; the card says so in one line under the drawing.
- **The curve never judges.** Nothing on it says a session is too hard, and
  nothing ranks the athlete's races against the method's sessions.

**The one answer that changed, and why it needs this note.** The frozen
route table is the contract, and a change to an answer's shape needs a note
in the same change. Three answers changed in this step, and each change
removes work from the surface rather than adding to it:

- **`/insights/pacing-curve`'s session rows now carry the share of the
  maximum their own prescribed pace costs** — `from_percent_max` and
  `to_percent_max` — because the curve's vertical axis is that share, and
  **the app does not compute it**: the core's own relation does, and the
  answer carries the number. Without it the surface would have had to hold a
  second implementation of the engine's arithmetic.
- **`/insights/threshold`'s candidates no longer carry the note** that says
  what a candidate cannot know. That line is the **explanation**, and the
  explanation is not product content (D22): the card shows the value, the
  source and the date, and the core keeps the wording (`threshold.limit.*`)
  for the notes to read. The note was also free text in the answer, and the
  answer is captured in the app's fixture set.
- **`/plan/bases` carries the entry requirement in the programme's own
  words and the athlete's own mileage beside it** — `entry_line`, "this
  level asks for 40.2-56.3 km a week", and `athlete_mileage`, "your last 4
  weeks: 60.8, 61.1, 50.8, 6 km". The two are **side by side and
  unranked** (B2), and both are the core's own rendering: the app states
  them and never compares them.

**The files it touches.** `crates/stride-api/src/dto.rs`,
`crates/stride-api/src/routes/pacing_curve.rs` and
`crates/stride-api/src/routes/threshold.rs` — the changed fields ·
`crates/stride-core/src/plan_bases.rs` — the entry requirement's line, in
the core's own words · `crates/stride-api/src/planbases.rs` — the bases'
answer · `frontend/src/pages/ladder.tsx` and
`frontend/src/pages/curve.tsx` — the two drawings · `frontend/src/pages/
progressWall.tsx`, `frontend/src/pages/ProgressPanelPage.tsx` and
`frontend/src/pages/useWall.ts` — the card, the panels and the answers ·
`frontend/src/lib/types.ts`, `frontend/src/lib/api.ts` and
`frontend/src/i18n/catalogue.ts` — the shapes, the calls and the chrome ·
`docs/contract/api-paths.md` — the changed fields.

**What changed since the draft (2026-11-25).** The three answers changed as
described, and the cards are drawn: the ladder is the Progress wall's **The
pace** card, the cost curve is its own panel behind it, and the new screens
— the ladder, the cost curve, the ledger's two blocks and the programmes —
are in the **viewport matrix** at the six widths like every other screen, so
a regression in their layout cannot merge green.

---

### Proposed amendment — the pace page's route is retired (2026-11-25)

**Retired (2026-11-25).** The route is gone: `/insights/pace` is in neither
the router nor the frozen table nor the phone's bridge table. The retirement
is a fact of the build and not a proposal awaiting an approval, so the
heading is **kept as history** and what it says is what happened. The app's
own `/pace` path now **redirects** to the panel that replaced it,
`/progress/pace`, so an old link lands on the ladder.

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

**`/insights/pace` is retired.** The pace page was a page of its own, and
the cut that removed it moved its two panels — the pace-at-heart-rate
cloud and the pace bands — onto Progress's **The pace** card. That card is
now the **ladder**: the anchors, the rungs with their values, their bands
and their rules, with the physiological-cost curve behind it on its own
panel. Nothing reads the pace bands any more, so the route that answered
them goes with them.

- **The weekly pace-at-heart-rate chart stays.** It is the weekly median
  the Progress answer already carries — `/insights/progress`'s
  `pace_at_hr` — and it is drawn on the pace panel, not on the card.
- **The route table is the contract, so the retirement is written down
  here and in the same change.** The frozen table
  (`docs/contract/api-paths.md`) loses the route, the **phone's bridge
  table** — a copy of it — loses it too, and the api's parity test holds
  the two to the contract, route for route.
- **The core keeps the engine.** `crates/stride-core/src/pace.rs` is the
  core's own, and the core is a library: a route going away is not the
  core's business. The api's seam for it — the module that read the runs'
  files for the bands — goes with the route.

**The files it touches.** `crates/stride-api/src/router.rs` and
`crates/stride-api/src/routes.rs` — the route and its module ·
`crates/stride-api/src/routes/pace.rs` and `crates/stride-api/src/pace.rs`
— removed · `crates/stride-api/src/dto.rs` — the answer's own shapes ·
`crates/stride-api/tests/{pace.rs,fixture_parity.rs,frontend_shapes.rs}` —
the tests that read it · `android/app/src/main/java/app/stride/running/
bridge/PhoneRoutes.kt` and its test — the phone's copy of the table ·
`frontend/src/lib/api.ts`, `frontend/src/lib/types.ts` and
`frontend/src/fixtures/index.ts` — the client, the declarations and the
fixture · `docs/contract/api-paths.md` and `docs/product-brief.md` — the
table and the brief that quotes it.

---

### Proposed amendment — the rig's own surface: the host's face, and the three first-run decisions (2026-11-25)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The app is the athlete's face; the rig is the machine it runs on, and the rig
had no face of its own — the host's tools page was all there was, and it said
nothing about the machine it ran on. Step 6 of the build gave it one:
**`/rig`**.

The rule this serves: **the host may state facts about itself, and it may
offer the athlete the choices only the athlete can make — but it never
advises, never ranks and never grades, and it never turns a fact about the
machine into a verdict about the athlete.**

What changes:

- **The four parts.** The **index** — the screens the rig serves, the new ones
  included: the ladder, the cost curve, the ledger, the threshold and the
  programmes · the **status** — the version being served, the database's
  schema version, what the log holds, and where the first run stands · the
  **storage** — the folder, the database and its size, and the files the keys
  live in · the **connection** — whether a Garmin key is kept, and when the log
  was last filled from the source.
- **Facts, never verdicts.** Every line is a fact about the machine or about
  what the athlete has already decided. Nothing is scored, ranked or graded,
  and **the key never reaches the page**: it says *whether* a Garmin key is
  kept and where the file is.
- **The three decisions.** The new screens need three things that cannot be
  worked out from the training — **a race to type, a threshold to choose, a
  programme to adopt** — and the page offers them as the athlete's own
  choices: **side by side, unranked, and never as a recommendation**. The
  vitals are never linked to them, and nothing on the page presumes any of
  them: a fresh rig leaves all three empty, names each one, and asks for
  nothing.
- **Where each choice is made.** The **race** is typed on the rig's page — the
  app has no screen for it — and the write goes through `stride-api`'s own
  route, so the race is the app's race and the refusals are the route's own
  words. The **threshold** and the **programme** are made in the app, where
  the four candidates and the four programmes live, and the page says which
  screen, and what is in force now.
- **The tools moved.** The rig's own tools — the export, the import and the
  FIT import — are at `/rig/tools` now, and the app's **Data & connection**
  link points there. The page is the rig's landing surface; the tools are one
  part of it.
- **No route changed.** The product's route table is unchanged, and the
  phone's bridge table is unchanged: the rig's page is the host's own,
  outside the frozen API prefix, and the phone has no such page. The new
  screens are served by the rig the way every other screen is served — with
  the app.

**What it may and may not say.** It may say what the machine is, what it
holds and what the athlete has already decided, and it may ask for the three
things only the athlete can give. It may **not** say which choice to make
first, may not grade what has been chosen, may not link a choice to a vital,
and may not say that the rig is ready, healthy or anything else about itself
that is not a fact about it.

**The files it touches.** `desktop/src/rig.rs` — the page, its four parts and
the three decisions · `desktop/src/server.rs` — the host's own paths, `/rig`,
`/rig/tools`, `/rig/race` and the tools' own · `desktop/pages/tools.html` —
the host's tools page · `desktop/tests/rig_page.rs` — the page's own tests ·
`frontend/src/pages/SettingsPage.tsx` — the app's link to the host's tools.

**What changed since the draft (2026-11-25).** The page is built and served:
`/rig` answers, and the host's tools are at `/rig/tools`. The race is written
through the api's own route, and the page's own test holds the key to the
page — it says *whether* a key is kept and where the file is, and the key's
value has no path onto it. **The page is outside the catalogue**: see the
amendment *"the host's own surfaces: outside the catalogue and the lint"*
below.

**What this does not settle.** Whether the three decisions are listed in a
fixed order or in the order the athlete last touched them; whether the
first-run page and this page are one surface or two; and what the page says
once a plan is running — the plan in force is named, and nothing more is
proposed here.

---

### Proposed amendment — the rig carries the app: the bundle check (2026-11-25)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The desktop embeds the built web bundle, so that a clean checkout can build
the rig with Rust alone. The rule has always been that the bundle is rebuilt
and replaced **in the same change** as the app it carries — and **nothing
held the tree to it**: the bundle in the tree was one commit behind, and a rig
built from a clean checkout would have served an app that was not the one in
the source.

The rule this serves: **a rig that serves an app one commit behind the
source is not the app the plan describes.**

What changes:

- **The check builds the app and compares.**
  `scripts/check-desktop-bundle.sh` builds `frontend/`, writes the bundle
  **the way the sync script writes it** — the same rewrite, in the same place,
  so there is one rewrite and not two — and compares it with the one in
  `desktop/assets`. The build is deterministic: the same source produces the
  same files, with the same names, so the comparison is a comparison and not a
  guess.
- **It is a gate.** `scripts/gate.sh` runs it with the other checks, so CI
  fails on a bundle that is out of step. A gate that did not run is a gate
  that did not pass.
- **The rule it does not change.** The bundle is still **replaced in the same
  change** as the app it carries; the check does not rewrite the tree for
  you. It says that the tree is out of step, and it names the one command
  that writes it back.

**What it may and may not say.** The check is an implementation fact and
never appears on a surface. It fails with the files that differ, and it
names the one command that fixes it.

**The files it touches.** `scripts/check-desktop-bundle.sh` — the check ·
`scripts/sync-desktop-bundle.sh` — the one rewrite the check shares with the
sync · `scripts/gate.sh` — the gate · `desktop/assets/` — the bundle the rig
embeds.

**What changed since the draft (2026-11-25).** The check is written and it is
in the gate, and the bundle was rebuilt and replaced in the same change as the
app it carries.

**What this does not settle.** Whether the check should also hold the app's
fixtures to the router's answers — the api's own parity test does that — and
whether the desktop's own pages should be covered by a check of their own.

---

### Proposed amendment — the host's own surfaces: outside the catalogue and the lint (2026-11-25)

**Proposal, not approved.** The approvals below are the user's to give. This
section records the change and the rule it serves; it is not permission to
ship, publish, migrate or sync production data.

The rig's own pages — `/rig`, the host's tools page, the first-run page — are
written in the host's own words, and they are **not in the catalogue**: the
core's string catalogue holds the app's strings, and the voice lint reads the
catalogue and the app's own components. So the host's surfaces are **outside
the lint**, and a word the contract bans can sit on one of them without the
build noticing.

The rule this serves: **one wording per idea** — and, under it, **the voice
is the product's central design constraint, so it is machine-enforced and not
merely reviewed.** A surface the lint cannot reach is a surface where the
voice is only reviewed, and review is what the lint exists to replace.

What changed: the host's tools page was titled **"Backup and restore"** while
the app said **"Backup, restore and export"** — the same idea, two wordings,
on two surfaces, and the host's was the narrower one. **The host's wording now
matches the app's**: the tools page, the rig's own index and the rig's own
link to it all carry the app's phrase.

**The gap, and what is recommended.** The known gap is that the host's
surfaces are outside both the catalogue and the lint. **The recommendation is
that they come under it**: the host's own sentences should be **in the
catalogue**, and the lint should read the host's files the way it reads the
app's components. That is a change to the lint and to the catalogue's shape,
and it is **not this change** — the wording is fixed here, the mechanism is
proposed here, and neither is shipped until the user says so.

**The files it touches.** `desktop/pages/tools.html` and `desktop/src/rig.rs`
— the host's own wording · `desktop/tests/rig_page.rs` — the host's own test.

**What changed since the draft (2026-11-25).** The wording is fixed and the
gap is recorded. The lint still reads the app's catalogue and the app's own
components, and the host's pages are still outside it.

**What this does not settle.** Whether the host's strings go into the
catalogue itself, or into a second catalogue the lint reads beside it; and
whether the host's pages are checked by the same lint or by a second one with
the same list.

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

Every user-facing string lives in one catalogue and is typed — the four registers (`vital`, `direction`, `option`, `pattern`)
and the three supporting types (`statement`, `status`, `instruction`), **seven in all**, and the contract's list is the
authority (`docs/contract/voice.md`).
The lint fails the build on banned words (`should`, `must`, `take the day`, `rest day`, `rest up`, `rest today`, `go easy`,
`missed`, `bad`, `poor`, `fail`, `behind`, `good`, `keep it easy`, `ready to`, `recommended`, `better to`, `need to`, `red`,
`readiness score`, `not recovered`, `you are ready`, `you are not ready`), requires a `direction` string to come
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
- **Five destinations, at every width:** Today · Runs · Progress · Body · Profile. Compare and the Plan week are
  reached **contextually**, never as extra tabs.
- **The rule that kills drift:** *if a screen cannot exist on the phone, it does not exist.* The desktop is the
  same five destinations with more room; it never grows features of its own.

## 2. Scope

**In:** Connect (Garmin key) · Sync (incremental) · Today · Runs · Activity detail · Run film · Progress · Body ·
Profile · Plan week · Compare · FIT import (any watch) · Settings (goal, pause, note, connect, about) · the
**heatmap**, computed on the device · the **year as a film**, with its months · the **training paces** and the
**cost curve** · the **threshold**, the athlete's own races and the choice in force · the **stimulus ledger** ·
the **plan programmes** — the 10K and the half marathon, as a catalogue the athlete adopts.

**Out (deferred, with reasons):** Garmin account login/MFA and the scheduled-workout calendar · MCP/agent
harness · running-form fingerprint **page** (the fingerprint itself is a section on the run detail) · gear ·
tiles proxy · an HR-zone **page** (the zone time lives on the wall and in the ledger's card) · network egress
routing · phone plan editor · desktop-only screens · the **method lens** — designed, then deferred (D13) ·
the **5K, 8K and 15K plan programmes** — cut (D15) · the **80/20 base as a second base** — decided (D14), but its
schedule has not been read cell by cell, so it is a spec and not a plan yet.

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
(loopback + CIDR allowlist) · five destinations
everywhere with Compare and Plan contextual · Garmin account flow out · MCP out · "issues welcome, features are
mine" · **voice: vitals, direction (closed set), options — no verdicts** · **distance comparison only beyond a 10% gap** ·
**the picture first, and the sentence second** · **explanation is not product content — the notes carry it** ·
**one plan at a time: Pfitzinger first, 80/20 as the second base when its schedule has been read** ·
**the athlete's own facts are typed, never synced**.

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

---

## The approval

**What the approval would cover.** Approving this set approves **sixteen
amendments** and **three corrections**, and it approves the **rules and the
refusals** — **not a release**: it is not permission to ship, publish,
migrate or sync production data.

**The amendments, by title.** The **stimulus ledger** and the card it lives
on · the **pace engine** — the chain, the ladder and the cost curve · the
**threshold** — four candidates, one choice, a dated fact · the **two bases
of zone time**, with the overlap as the app's check against itself · the
**picture first** rule · **explanation is not product content** · **the
surface speaks the runner's language** · the **plan bases** — Pfitzinger
first, one at a time, the 80/20 base a spec and not yet a second one · the
**audience** — the intermediate runner · the **storage** — the dated facts and
the plan's own rows · the **decode cache** — one decode per run, bounded at
sixty-four · the **versioned plan catalogue** the athlete adopts · the
**step-5 cards** and the three answers that changed · the **rig's own
surface** — the host's face and the three first-run decisions · the **bundle
check** — the rig carries the app · and the **host's own surfaces** — outside
the catalogue and the lint. The amendments of the earlier weeks — the zone
model, the Eddington number, the ledger's weeks, the run detail's one chart,
the stopped stretch, the fingerprint, the training load, the rolling view,
the wellness observations, the zone distribution, the year, the run film, the
heatmap, the run update and the cockpit Today — stand in the file as they
stand, and this set does not change them.

**The rules and the refusals.** One zone model in force on the day of the
run · the athlete's own facts typed and never synced · the candidates
computed and never stored · one threshold and one plan in force at a time ·
the tier **stated beside the athlete's own mileage and never suggested** ·
absence drawn, never a zero · the cache holds the **decode** and nothing
derived · and the refusals, each with the core's own reason: a programme or a
tier not in the catalogue, a start date that leaves no room for the twelve
weeks and the race week, a goal race the programme does not print, a plan
already in force, and a choice with no value.

**The corrections** are not proposals — the user has already decided them:
**five destinations** rather than four · the contract's **seven string
types** and its **twenty-three banned words** · **§2's scope**, in both
directions.

**The two open items.** The ledger's **repeated-work row** is a **no value
with the core's own reason**: it needs a real workout file from the
athlete's watch — no file in the corpus carries a workout step, and the
ingest does not yet decode the step-name field. The **80/20 base** is a
**spec, not a second base**: its schedule has not been read cell by cell, its
cells are minutes rather than miles, and its threshold candidate is an LTHR.

**The retirements are marked, not deleted** — the pace page's heading and its
route, the zone model's date on the run detail, and the parts retired in the
fingerprint, the rolling view and the year — and everything above stays
**proposal, not approved** until the user says so.
