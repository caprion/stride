# The cleanup plan — what is dead, what is duplicated, and what to do about it

*24 November 2026. **A plan, not a change.** Nothing under `crates/`,
`frontend/`, `android/`, `desktop/`, `PLAN.md`, `docs/` or the contract was
touched. This file is the only thing created.*

Read with: `exploration/decisions.md` (**binding** — D1…D10),
`exploration/surface-inventory.md` (the surface cut),
`exploration/stimulus-ledger.md` (the ledger), `PLAN.md` (the plan of
record), `docs/contract/` (the frozen contracts), `CONTRIBUTING.md` (the
gates).

> **What changed since the first draft of this plan.** The parent's
> decisions are now binding, so three of the prices here are settled rather
> than recommended, and two new items have been added:
>
> * **§2.2** — the window is **D1/D2**: the last *N* calendar weeks through
>   today, compared with the same span seven days earlier. The rolling
>   route is migrated; its one-day shift is retired.
> * **§2.3** — the zone model is **D3, staged**: stage 1 (the zone-time
>   answers, per-run dated model) is in this cleanup pass; stage 2 (the load
>   model) is later and separate.
> * **§2.4** — the workout-step decode is **D4** and is a cleanup step with
>   its own test data.
> * **§2.5** — the ledger's shares name their denominator (**D5**).
> * **§2.7** — the activity search response's discarded fields are kept
>   (**D10**), with the schema, the sync mapping, the export/backup impact
>   and the test data priced.
> * **§2.8** — temperature is priced at **zero** (**D6/D9**).
> * **D8 is decided**: five destinations; zone bands in the chart *and* the
>   table; altitude on its own scale, default off. **No step in this plan
>   branches on any of the three.**
> * **§2.9** — **D11** (the picture first, the sentence second) is now a
>   first-class requirement of the cleanup: **four of the eleven surviving
>   cards are sentence-only today** and must be redrawn or merged, and
>   **four of the pictures need an answer the api does not carry.** §2.9
>   names them, prices them, and adds two steps (**V1**, **F6**).

---

## 0. How this was established, and what you can check yourself

Every claim below names a path and a command. The four that found the most:

```bash
# which api functions does the app actually call?
cd frontend/src && for f in activities activity activityFit runFilm vitals \
  physiologyDaily physiologySleep physiologyHrv physiologyLatest physiologySources \
  fitnessTrends fitnessFingerprint fitnessZones statsSummary statsWeekly \
  statsPaceTrends statsElevation statsPersonalRecords planCurrent planToday planWeek \
  planBlocks planSources planAdherence planResults planImport insightsToday \
  insightsProgress insightsPace insightsLoad insightsRolling insightsRollingDays \
  insightsStreaks insightsChallenges insightsYear insightsHeatmap insightsChips \
  compare compareCandidates health healthDaily syncStatus syncRun syncRuns \
  syncHistory syncSettings authStatus setGarminPersonalKey testGarminPersonalKey \
  openGarminKeySetup importFit garminLogout calendar settings saveSettings \
  exportBackup deleteEverything saveNote activityUpdate activityUpdatePayload \
  updateSettings saveUpdateSettings; do
  n=$(grep -rn "api\.$f\b" --include=*.ts --include=*.tsx . | grep -v "lib/api.ts" | wc -l)
  [ "$n" = "0" ] && echo "never called: $f"
done

# which catalogue entries does the app never use?
cd frontend/src && grep -oP "^\s*'[a-z0-9._-]+':" i18n/catalogue.ts | tr -d " ':" \
  | while read k; do
    grep -rq "['\"]$k['\"]" --include=*.ts --include=*.tsx . || echo "unused: $k"
  done

# which catalogue entries does the core/api never use?
cd /home/sumit/stride-v2 && grep -oP '^\s*id: "\K[a-z0-9._-]+' crates/stride-core/src/strings.rs \
  | while read k; do
    grep -rq "\"$k\"" crates/stride-core/src crates/stride-api/src --include=*.rs || echo "unreferenced: $k"
  done

# does the Android bridge know every route the router serves?
grep -oP 'pattern: "\K[^"]+' crates/stride-api/src/router.rs | sort -u > /tmp/rust.txt
grep -oP '^\s+"\K[^"]+' android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt \
  | sort -u > /tmp/kt.txt
comm -23 /tmp/rust.txt /tmp/kt.txt
```

**One thing I am not sure of, and how to settle it.** Whether
`/plan/current`, `/plan/today`, `/plan/blocks`, `/plan/sources`,
`/plan/adherence`, `/plan/results`, `/stats/*`, `/physiology/{daily,hrv,
latest,sources}`, `/insights/chips` and `/sync/{runs,history,settings}` are
*dead* or *deliberate endpoints with no UI*. The contract says the latter, in
so many words — "The routes the app does not type … are the router's own, and
they are written down in `crates/stride-api/src/dto.rs`". **I have treated
them as deliberate, not dead**, and this plan does not propose deleting them.
The two the contract does *not* bless this way are `/fitness/trends` and
`/health/daily` (§1.5, **Drift 3**).

---

## 1. What is actually dead or duplicated today

### 1.1 Routes

#### The routes the app never reaches

Twenty-two of the router's sixty routes are never called by the app. Each is
a function in `frontend/src/lib/api.ts` with no caller, and the api client
is the app's only way to reach the router.

| route | the client function | who calls it |
|---|---|---|
| `/physiology/daily` | `api.physiologyDaily` | nobody |
| `/physiology/hrv` | `api.physiologyHrv` | nobody |
| `/physiology/latest` | `api.physiologyLatest` | nobody |
| `/physiology/sources` | `api.physiologySources` | nobody |
| `/fitness/trends` | `api.fitnessTrends` | nobody |
| `/stats/summary` | `api.statsSummary` | nobody |
| `/stats/weekly` | `api.statsWeekly` | nobody |
| `/stats/pace-trends` | `api.statsPaceTrends` | nobody |
| `/stats/elevation` | `api.statsElevation` | nobody |
| `/stats/personal-records` | `api.statsPersonalRecords` | nobody |
| `/plan/current` | `api.planCurrent` | nobody |
| `/plan/today` | `api.planToday` | nobody |
| `/plan/blocks` | `api.planBlocks` | nobody |
| `/plan/sources` | `api.planSources` | nobody |
| `/plan/adherence` | `api.planAdherence` | nobody |
| `/plan/results` | `api.planResults` | nobody |
| `/insights/chips` | `api.insightsChips` | nobody |
| `/health/daily` | `api.healthDaily` | nobody (the fixture set holds `healthDailyFixture`, `frontend/src/fixtures/index.ts:1684`, and no screen reads it) |
| `/sync/runs` | `api.syncRuns` | nobody |
| `/sync/history` | `api.syncHistory` | nobody |
| `/sync/settings` | `api.syncSettings` | nobody |
| `/auth/garmin/logout` | `api.garminLogout` | nobody |

**How I know.** `rg "api\.<name>" frontend/src` outside
`frontend/src/lib/api.ts` returns zero for each. Cross-check: the desktop
(`desktop/src/api.rs`, `desktop/src/server.rs`, `desktop/src/sync.rs`)
hard-codes only `/api/v1/auth/garmin/{personal-key,logout,fit-import}`,
`/api/v1/sync/run`, `/api/v1/sync/status`, `/api/v1/update/settings`,
`/api/v1/activities{,/{id}}` and
`/api/v1/activities/{id}/update{,/payload,/request}`. The Android seam
(`android/app/src/main/rust/stride-ffi/src/stride.udl`) is JSON-in/JSON-out
and calls no path of its own. **So the only consumer of these twenty-two
routes is nobody.**

**Twenty of the twenty-two are blessed by the contract** as endpoint-only
routes. **Two are not** — `/fitness/trends` and `/health/daily` (**Drift 3**, §1.5).

#### The route the UI reaches but the cut removes

`/insights/pace` — `crates/stride-api/src/routes/pace.rs::pace`, wired in
`crates/stride-api/src/router.rs` (`pattern: "/insights/pace"`), read by
`frontend/src/lib/api.ts::api.insightsPace`, called from
`frontend/src/pages/useWall.ts:65`, and drawn by
`frontend/src/pages/PacePage.tsx`, mounted at `/pace` in
`frontend/src/app/App.tsx`.

The **route stays** — **D7** moves the pace panels onto the Progress wall
and they need the same answer. The **page goes**:
`frontend/src/pages/PacePage.tsx` and the `<Route path="/pace">` in
`frontend/src/app/App.tsx`. **Eighteen** catalogue entries are used *only* by
that page and lose their user with it (§1.4).

#### The route the surface cut does not remove

`/insights/rolling` is the answer behind most of the wall's cards. It stays —
but it **changes** (§2.2): under **D1/D2** it moves to the calendar-week
window and the seven-day-shift comparison. And note: **`/insights/rolling` is
one of the eleven routes the Android bridge refuses** (**Drift 1**, §1.5) —
so on the phone the Progress wall is already broken.

### 1.2 Frontend

#### Components that become unused after the cut

| component | file | how I know | after the cut |
|---|---|---|---|
| `ProfileRadar` | `frontend/src/components/ProfileRadar.tsx` | exported and used by `frontend/src/pages/progressWall.tsx:211` (`ProfileRadarBody`), `frontend/src/pages/ProgressPanelPage.tsx:212`, `frontend/src/pages/ProfilePage.tsx:305` | **D7** retires the radar; the seven facets stay. The component and `RADAR_AXES` (`ProfileRadar.tsx:58`) go with it. |
| `RADAR_AXES` | `frontend/src/components/ProfileRadar.tsx:58` | imported by `frontend/src/components/AthleteShape.tsx:40` and `frontend/src/pages/progressPanels.tsx:44` | it is the radar's axis list, and `AthleteShape` uses it only to *name* the six axes. The naming must move somewhere that is not the radar — see step **F4** |
| `PacePage` | `frontend/src/pages/PacePage.tsx` | mounted at `/pace`, `frontend/src/app/App.tsx` | **D7**: retired; its two panels become Progress panels |
| the wall's *Activity heatmap* calendar | `frontend/src/pages/progressWall.tsx` (`progress.wall.heatmap.*` keys) | the card id `heatmap` in `frontend/src/pages/taskAreas.ts:88` | retired; the year calendar survives |
| the wall's *Most recent activities* | `frontend/src/pages/progressWall.tsx` | card id `activities`, `frontend/src/pages/taskAreas.ts:93` | retired; the Runs destination holds it |
| the wall's *Body, last night* | `frontend/src/pages/progressWall.tsx` | card id `body`, `frontend/src/pages/taskAreas.ts:94` | retired; Today and Body hold it |
| the wall's *Yearly stats* | `frontend/src/pages/progressWall.tsx` | card id `years`, `frontend/src/pages/taskAreas.ts:89` | retired; the year page compares the years |
| the wall's *Consistency* | `frontend/src/pages/progressPanels.tsx` (`ConsistencyPanel`) | card id `consistency`, `frontend/src/pages/taskAreas.ts:75` | merged into *The weeks* |
| the wall's *Polarised split* | `frontend/src/pages/progressPanels.tsx` (`ZonesPanel`, `progress.rolling.zones.heading`) | card id `split`, `frontend/src/pages/taskAreas.ts:76` | **D7**: merged into the zone card — "the ledger **is** that card" |
| the wall's *Rolling window* | `frontend/src/pages/progressWall.tsx` | card id `window`, `frontend/src/pages/taskAreas.ts:78` | merged into *This week* |
| the run detail's sleep composition | `frontend/src/pages/ActivityPage.tsx:747` (`<SleepComposition date={data.local_date} beforeRun />`) | the only `SleepComposition` with `beforeRun` | **D7**: the panel moves to Body; the run detail keeps a **one-line link** |
| the phone index's *areas* link lists | `frontend/src/pages/ProgressIndex.tsx` | the `TASK_AREAS`/`WALL_CARDS` link list | the index rows become the wall's card list |

#### The panels that draw the same data twice

1. **The polarised split is the zone distribution, regrouped.**
   `crates/stride-core/src/rolling.rs::polarised_shares` (line 354) takes one
   `ZoneTimes` and folds it into three bands (`PolarisedZone::Low` = Z1+Z2,
   `Moderate` = Z3, `High` = Z4+Z5). The five-zone `distribution` in the same
   answer (`crates/stride-api/src/rolling.rs`, `current.shares()` /
   `current.per_zone()`) is the *same five numbers*, ungrouped. The wall
   draws the first as the card *Polarised split*
   (`frontend/src/pages/progressPanels.tsx:544` `ZonesPanel`) and the second
   as *Time in each zone* (line 626 `ZonesDistributionPanel`). **One fact,
   two drawings, two cards.** **D7** merges them into one card — and the
   ledger *is* that card, so the core keeps both views and the wall keeps
   one card.

2. **Two calendars of the same year.** The wall's *The year*
   (`progress.wall.year.*`) and the wall's *Activity heatmap*
   (`progress.wall.heatmap.*`, a distance calendar) are both grids over the
   same twelve months. The year page's own calendar
   (`frontend/src/pages/YearPage.tsx`, `frontend/src/components/
   YearCalendar.tsx`) is a third. Two survive; the wall's distance calendar
   does not.

3. **The shape is drawn twice.** The radar (`ProfileRadar.tsx`) and the facet
   list (`frontend/src/components/AthleteShape.tsx`) both draw the same six
   facets (`RADAR_AXES` = volume, consistency, variety, density, duration,
   intensity). The core says so itself:
   `crates/stride-core/src/profile.rs` — "**It is not a radar**: a polygon
   over fixed anchors is a claim about a shape, and the anchors are somebody
   else's coaching assumptions."

4. **The pace lives in three places.** The run detail's pace chart
   (`frontend/src/pages/ActivityPage.tsx`), the `/pace` page's panels
   (`frontend/src/pages/PacePage.tsx`), and the profile's pace facet. **D7**
   leaves two (the run's own pace, the trend).

5. **Last night is drawn three times.** Today (`frontend/src/pages/
   TodayPage.tsx:588` `vital.rest.heading`, line 611 `<SleepComposition>`),
   the run detail's sleep (`ActivityPage.tsx:747`), and the wall's *Body,
   last night* card. Body owns it after the cut.

#### Where the same string is written twice

| the string | the core's copy | the app's copy | the verdict |
|---|---|---|---|
| `Last night` | `crates/stride-core/src/strings.rs:217` — `vital.rest.heading` | `frontend/src/i18n/catalogue.ts:233` — `vital.rest.heading` | **same key, same text, two files.** The core's copy is referenced by nothing (`rg "vital.rest.heading" crates/` → 0). The app's is read by `TodayPage.tsx:588`. |
| `Version {version}` | `crates/stride-core/src/strings.rs:1227` — `status.version` | `frontend/src/i18n/catalogue.ts:249` — `status.version` | **same key, same text, two files.** The core's copy is referenced by nothing; the app's is read by `SettingsPage.tsx`. |
| `Sleep` | `crates/stride-core/src/strings.rs:187` — `vital.sleep.label` | `frontend/src/i18n/catalogue.ts:238` — `vital.sleep` | same text, different keys, **both unreferenced** by the core/api; the app's is read by `VitalReadings.tsx`. |
| `HRV` | `crates/stride-core/src/strings.rs:193` — `vital.hrv.label` | `frontend/src/i18n/catalogue.ts:239` — `vital.hrv` | same text, different keys; the core's is unreferenced. |
| `Resting HR` | `crates/stride-core/src/strings.rs:199` — `vital.resting_hr.label` | `frontend/src/i18n/catalogue.ts:240` — `vital.restingHr` | same text, different keys; the core's is unreferenced. |
| `Rest` | `crates/stride-core/src/strings.rs:205` — `vital.rest.label` | — | the core's is unreferenced by the core/api. |

**Nothing checks this.** `frontend/scripts/lint-voice.mjs` checks the app's
catalogue and `crates/stride-core/strings.json` **separately**
(`checkCatalogue` then `checkExport`); neither knows about the other. Two
catalogues that both define `vital.rest.heading` cannot drift apart *within*
one file, but they can disagree *between* them and no check would say so.

### 1.3 Core

#### Modules whose output no route reads any more

**None.** Every module in `crates/stride-core/src/` has a reader:

| module | its reader |
|---|---|
| `band` | `crates/stride-core/src/direction.rs` |
| `challenges` | `crates/stride-api/src/routes/challenges.rs` |
| `date` | everywhere |
| `direction` | `crates/stride-api/src/wellness.rs` |
| `eddington` | `crates/stride-api/src/routes/insight.rs` (`eddington_of`) |
| `fingerprint` | `crates/stride-api/src/fingerprint.rs` |
| `heatmap` | `crates/stride-api/src/routes/heatmap.rs` |
| `load` | `crates/stride-api/src/load.rs` |
| `pace` | `crates/stride-api/src/pace.rs` |
| `pattern` | `crates/stride-api/src/wellness.rs` |
| `plan` | `crates/stride-api/src/plan.rs`, `routes/plan.rs` |
| `profile` | `crates/stride-api/src/rolling.rs` (`profile_facets`) |
| `rest` | `crates/stride-api/src/wellness.rs` |
| `rolling` | `crates/stride-api/src/rolling.rs` |
| `statement` | `crates/stride-api/src/routes/insight.rs` |
| `story` | `crates/stride-api/src/story.rs` |
| `streaks` | `crates/stride-api/src/rolling.rs` (`streak_answer`) |
| `strings` | everywhere |
| `types` | the core |
| `units` | everywhere |
| `update` | `crates/stride-api/src/update.rs` |
| `year` | `crates/stride-api/src/year.rs` |
| `zones` | everywhere |

The surface cut **removes no core module**. It removes *card*, *panel* and
*page* — never an engine. **The cut is in which answers reach a screen, not
in how an answer is computed.**

#### The function that exists in two modules with different answers

**The two walking thresholds.** This is the known case, and it is real.
**The parent's decision** is that they become **one rule with two views of
it**, and that the rule **lives in the core** — "the ingest classifies, the
core reads".

| | sample level | lap level |
|---|---|---|
| file | `crates/stride-ingest/src/segment.rs:104` `classify_sample` | `crates/stride-core/src/fingerprint.rs:525` `was_walked` |
| what it decides | run / walk / stand, per sample | whether a *lap* was run or walked |
| the cadence threshold | **< 60 spm** → walk (`segment.rs:111`) | **< 125 spm** → walked (`fingerprint.rs:79`, `WALKING_CADENCE_SPM`) |
| the speed threshold | **< 1.55 m/s** → walk (`segment.rs:116`) | **< 2.2 m/s** → walked (`fingerprint.rs:85`, `WALKING_SPEED_MPS`) |
| the combination | cadence **and** speed (`segment.rs:111-114`) | cadence **and** speed (`fingerprint.rs:529-536`) |
| what is excluded from the zones | the *stand* samples only (`crates/stride-api/src/routes/series.rs:55`) | — |
| what is excluded from the pace bands | everything that is not `Run` (`series.rs:79`) | — |
| what is excluded from the fingerprint | — | the walked laps (`fingerprint.rs:319`, `running_only`) and the fragments under 300 m (`measurable`, `MIN_LAP_DISTANCE_M`) |

**The two disagree.** A lap at 100 spm and 2.0 m/s is **walked** at the lap
level and **run** at the sample level. A stretch at 1.8 m/s and no cadence is
**run** at the sample level (the `speed < 1.55` rule does not fire, and the
cadence rule needs a cadence) and **walked** at the lap level.

**The rule, in one place.** The dependency DAG
(`scripts/check-deps.sh`: `core ← ingest`) means the core cannot call into
the ingest. **So the rule moves down, into the core**, and the ingest calls
it:

```text
crates/stride-core/src/movement.rs        (new module)
    pub enum Movement { Run, Walk, Stand }
    pub fn movement_of(speed: Option<f64>, cadence: Option<f64>) -> Movement
    pub const WALKING_CADENCE_SPM: f64     // the ONE cadence threshold
    pub const WALKING_SPEED_MPS: f64       // the ONE speed threshold
    pub const STAND_SPEED_MPS: f64
    pub const WALK_SPEED_MPS: f64
```

* `crates/stride-ingest/src/segment.rs` — `classify_sample` **delegates** to
  `stride_core::movement::movement_of` and keeps only the *stream* logic
  (which sample owns which span). **The ingest classifies.**
* `crates/stride-core/src/fingerprint.rs` — `was_walked` **delegates** to the
  same function and keeps only the *lap* logic (which laps are long enough
  to measure). `WALKING_CADENCE_SPM` and `WALKING_SPEED_MPS` are **deleted**
  from `fingerprint.rs` and re-exported from the new module. **The core
  reads.**
* `crates/stride-core/src/update.rs:543` — the third copy
  (`.filter(|split| split.distance_m >= MIN_LAP_DISTANCE_M)`) borrows the
  fingerprint's fragment threshold; it comes along.

**The two views.** The one rule has two views because the *grain* differs:
the sample view is "was the athlete moving in this instant", and the lap view
is "was this lap run or walked", computed from the lap's own means. **One
threshold set, two granularities** — which is exactly what the parent asked
for.

**What it costs.** The two thresholds are not the same number, so this is a
**behaviour change**: choosing one cadence threshold and one speed threshold
changes which samples are "moving" (`series.rs:55`, `series.rs:79`) and which
laps the fingerprint measures (`fingerprint.rs:319`). Every zone time, every
pace band, the film ribbon and the run detail's *not moving* bucket move.
**See §4.1 — this is one of the two the user must be told about.**

#### The api has three copies of the zone-model reader

| copy | file | reads |
|---|---|---|
| `stored_model` | `crates/stride-api/src/rolling.rs:266` | `current_hr_zone_model()` → the most recent model |
| `stored_model` | `crates/stride-api/src/fingerprint.rs:185` | the same, into a different struct |
| `zone_model` | `crates/stride-api/src/load.rs:160` | the same, into a third struct |

All three call `crates/stride-store/src/repos/zones.rs:54
current_hr_zone_model()`, which is `ORDER BY read_at DESC, id DESC LIMIT 1` —
**the model read most recently**. §2.3 changes that rule, and it has to
change in three places unless the three are collapsed into one first.

#### A rule the code claims is frozen and is not

`crates/stride-ingest/src/segment.rs:5` says: "The thresholds below are the
**frozen rules** from `docs/contract/core-contract.md`." **The core contract
has no such clause.** `rg "0\.30|1\.55|2\.2|125|segment|classif"
docs/contract/core-contract.md` returns one hit — line 27, about the halving
rule — and no clause stating the thresholds. The thresholds live only in
code. See **Drift 4**.

### 1.4 Strings

#### Dead entries in the app's catalogue — 38 of 533

`frontend/src/i18n/catalogue.ts`. Each is referenced by nothing outside the
catalogue file itself (`rg "'<key>'" frontend/src` → 0 hits):

```text
activity.effort.minutes        activity.note.cleared          activity.note.saved
body.sleep.duration            body.sleep.stages              challenge.streak
common.saving                  compare.chart.toggle           connect.blank
connect.unreachable            film.alt                       pace.open
plan.days.target               plan.source.unclassified       plan.source.why
plan.week.heading              progress.best.10k              progress.best.5k
progress.best.longest          progress.index.shape.question  progress.rolling.daytime.heading
progress.rolling.distances.heading  progress.rolling.history  progress.rolling.months.heading
progress.rolling.weekday.heading    progress.rolling.years.heading
progress.wall.split.note       runs.filter.all                runs.filter.other
runs.filter.runs               settings.files                 settings.storage.export
settings.storage.exported      today.state.stale.minutes      today.streak.longest
today.vitals.heading           year.intro                     year.open
```

**None of these is checked by anything.** `npm run lint:voice` checks the
*text* of the entries, never whether an entry has a user. A catalogue that
grows entries nobody reads is exactly the kind of thing a "no unused
catalogue entry" check would catch.

#### Dead entries in the core's catalogue — 9 of 200

`crates/stride-core/src/strings.rs`, referenced by no core or api code:

```text
option.other_sessions          statement.plan.status.done     statement.plan.status.not_done
statement.plan.status.planned  vital.hrv.label                vital.rest.heading
vital.resting_hr.label         vital.sleep.label              vital.sleep.value
```

`statement.plan.status.{done,not_done,planned}` are referenced only by
`crates/stride-core/tests/plan.rs:234-236` — a test that asserts the
catalogue *contains* them. The test keeps a dead string alive.

`vital.rest.heading` and `status.version` are the two **key collisions**
with the app's catalogue (§1.2). Deleting the core's copies removes the
collision and the dead entry in one move.

#### The catalogue entries that lose their user with the cut

| the cut | the entries that lose their user |
|---|---|
| `/pace` retired (**D7**) | `pace.heading`, `pace.open`*, `pace.question`, `pace.summary`, `pace.window`, `pace.weeks.heading`, `pace.weeks.note`, `pace.weeks.legend.distance`, `pace.weeks.legend.longest`, `pace.weeks.legend.pace`, `pace.paceAtHr.heading`, `pace.paceAtHr.note`, `pace.bands.heading`, `pace.bands.note`, `pace.bands.rest`, `pace.bands.total`, `pace.empty.title`, `pace.empty.body` — **18 entries, all used only by `frontend/src/pages/PacePage.tsx`.** (`pace.open` is already unused; `pace.empty.*` survive only if the panels keep an empty state.) |
| the radar retired (**D7**) | `progress.wall.profile.heading`, `progress.wall.profile.alt`, `progress.wall.profile.axis`, `progress.wall.profile.previous`, `progress.wall.profile.window`, `progress.wall.profile.note`, `progress.index.shape.heading`, `progress.index.shape.open` — 8 entries used only by the radar's two surfaces. (`progress.index.shape.question` is already unused.) |
| the wall's *Activity heatmap* calendar retired | `progress.wall.heatmap.heading`, `progress.wall.heatmap.legend`, `progress.wall.heatmap.none`, `progress.wall.heatmap.l1`…`l4` — 7 entries, used only by `frontend/src/pages/progressWall.tsx`. |
| the wall's *Most recent activities* retired | `progress.wall.activities.heading` (and `progress.wall.runs.{day,week,month}` if the wall's marks go with it) |
| the wall's *Body, last night* retired | `progress.wall.body.heading`, `progress.wall.body.note` |
| the wall's *Yearly stats* retired | `progress.wall.years.heading` |
| the wall's *Consistency* merged | `progress.consistency.heading` (its `progress.consistency.unit` survives — the merged card keeps the unit) |
| the wall's *Polarised split* merged (**D7**) | `progress.wall.split.heading`, `progress.wall.split.note` (the latter is already unused) |
| the wall's *Rolling window* merged | `progress.rolling.heading` (the rest of `progress.rolling.*` survives — the merged *This week* card names two windows) |
| the phone's *areas* link lists retired | `progress.area.*` — 10 entries in `frontend/src/pages/taskAreas.ts` (they survive only if the areas survive as headings; the cut makes the index a list of cards, so they go) |

**The lint and the CI care.** `npm run lint:voice` reads
`frontend/src/i18n/catalogue.ts` and `crates/stride-core/strings.json` and
fails on a banned phrase, a direction outside the closed set, a pattern
without its sample size, an option that mentions a vital, and a ban list
that has drifted from `docs/contract/voice.md`. **A removed entry is a
silent change** — nothing fails if you leave a dead entry behind, and
nothing fails if you remove one that was still used. That is why the
removal step needs a "no entry has no user" check *added*, not just the
removal itself.

**`crates/stride-core/strings.json` is generated.** `crates/stride-core/
src/strings.rs:1984` asserts `catalogue_json() == include_str!("../
strings.json")`. **Any change to the core's catalogue must be followed by
`cargo run -p stride-core --example export_catalogue`** or the build fails.
It is not hand-edited.

#### The entries the new features need

| the feature | the entries | where they live |
|---|---|---|
| the ledger (**D4/D5**) | `stimulus.zone.time`, `stimulus.zone.last`, `stimulus.zone.last_gap`, `stimulus.longest`, `stimulus.longest.before`, `stimulus.repeated`, `stimulus.repeated.last`, `stimulus.changed`, `stimulus.changed.counts`, `stimulus.no_runs`, `stimulus.no_steps`, the coverage line, and **the denominator line** (**D5**: each share names its denominator) | the **core's** catalogue — they are sentences about the data, and `docs/contract/core-contract.md` §8 gives the core the ownership of the sentences |
| the zone model, stage 1 (**D3**) | the answer names the model and the day it was read — `zones.model.source` **already exists** (`crates/stride-core/src/zones.rs`) and `zones.model.source.no_max` is its no-maximum variant. **A new string is needed only for the window answer**, which can no longer name one model: something like `zones.models.source` ("the models the window was measured against"). | the core's catalogue |
| the consolidated chart (**D7**) | the axis labels and the altitude's own metres label. **The app's catalogue** — they are chrome. | `frontend/src/i18n/catalogue.ts` |

### 1.5 Drift between the documents and the code

Facts with paths. **The ones marked ⚠ need your approval to change a
document**; the rest are the code being wrong and the document being right.

---

#### Drift 1 · The Android bridge's frozen route table is eleven routes short ⚠

**`android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt:158-215`**
carries a hand-copied copy of the frozen route table. It is missing eleven
routes the router serves:

```text
/activities/{id}/fit          /activities/{id}/film        /activities/{id}/update
/activities/{id}/update/payload   /activities/{id}/update/request   /update/settings
/insights/load   /insights/rolling   /insights/pace   /insights/challenges   /insights/streaks
```

`StrideApiPlugin.kt:134-135` **rejects** any route the table does not name,
with a 404. So on the phone today:

* the Progress wall's cards that read `/insights/rolling` (the zones split,
  the distance breakdown, the facets, the weekday and daytime splits) fail;
* the load card (`/insights/load`) fails;
* the streaks card (`/insights/streaks`) fails;
* the challenges card (`/insights/challenges`) fails;
* the `/pace` page (`/insights/pace`) fails;
* the run film's own series (`/activities/{id}/film`) fails;
* the run update (`/activities/{id}/update`, `/payload`) and the update
  settings (`/update/settings`) fail.

**This is a live bug, not a cleanup.** It is the first item in the plan
because **D7** widens it: the cut moves more of the product's answers onto
the wall, and the wall reads `/insights/rolling`.

**The table is deliberately a copy** — the file says so: "The table is
frozen, so it is copied here and not derived: a route that appeared or
vanished because a core crate changed would be a change to a contract, and
contracts are not changed by the shell." So the fix is **to add the eleven
routes to the copy**, not to derive it. **The copy must not be shortened by
this cleanup** — see the do-not-touch list.

⚠ **Needs your approval** if the fix is to *derive* the table instead of
copying it (a contract change to `docs/contract/api-paths.md` and a change
to the shell's stated rule). The cheap fix — eleven more strings — needs
nothing.

---

#### Drift 2 · "Four destinations" in three documents; five in the code — **decided (D8.1): five** ⚠

| the document | the line |
|---|---|
| `PLAN.md:577` | "the four destinations stay the four" (the year) |
| `PLAN.md:603` | "The four destinations stay the four." (the run film) |
| `PLAN.md:692` | "The four destinations stay the four." (the heatmap) |
| `PLAN.md:771` | "The four destinations stay the four." (the pace page) |
| `PLAN.md:982` | "**Four destinations, at every width:** Today · Runs · Progress · Body." |
| `PLAN.md:1121` | "four destinations everywhere with Compare and Plan contextual" |
| `README.md:49` | "Today, Runs, Progress and Body remain the four destinations at every width." |
| `CONTRIBUTING.md` | "**One information architecture.** Today · Runs · Progress · Body — at every width." (rule 1, "not negotiable") |

The code ships **five**:

* `frontend/src/app/destinations.ts:36-45` — `DESTINATIONS` holds Today,
  Runs, Progress, **Profile** (`nav.profile`), Body;
* `frontend/src/app/App.tsx` — `<Route path="/profile">` and
  `<Route path="/profile/:panel">`;
* `frontend/src/pages/ProfilePage.tsx` — the athlete profile's wall;
* `frontend/src/pages/taskAreas.ts` — the areas are split across two
  destinations (`destination: 'progress' | 'profile'`), so the split is
  structural, not cosmetic.

**D8.1 settles it: five.** The profile keeps its own door. **The drift is
the defect, not the count.** So the work is a **documentation pass**:
`PLAN.md` (six places: 577, 603, 692, 771, 982, 1121), `README.md:49` and
`CONTRIBUTING.md` (rule 1) are corrected to say five, and the *cut size is
unchanged* (24 wall cards → 11: six on Progress, five on the profile).

**`README.md` is user-facing** and states four destinations today. It is the
one document in this drift that a stranger reads. **It gets its own step
with its own check** (step **P2**), separate from `PLAN.md`, because
`PLAN.md` travels with a contract edit and `README.md` does not.

⚠ **`PLAN.md` is the plan of record** — the amendment is a plan change and
travels with the pull request that carries the cut, as the contract's own
rule demands.

---

#### Drift 3 · The contract's "the app does not read these" list is wrong in both directions ⚠

`docs/contract/api-paths.md`, "The shapes the app does not read":

> "The routes the app does not type — `/physiology/{daily,hrv,latest,sources}`,
> `/stats/*`, `/plan/{current,today,blocks,sources,adherence,results,import}`,
> `/insights/chips`, `/sync/{runs,history,settings}` and `/auth/*` — are the
> router's own, and they are written down in `crates/stride-api/src/dto.rs`."

Two directions of wrong:

* **The list names routes the app *does* read.** `/plan/import` is read by
  `frontend/src/components/PlanImport.tsx` → `api.planImport`. Four of the
  five `/auth/*` routes are read: `api.authStatus` (four call sites),
  `api.setGarminPersonalKey`, `api.testGarminPersonalKey`,
  `api.openGarminKeySetup`, `api.importFit`. Only `/auth/garmin/logout` is
  unread.
* **The list omits two the app does *not* read.** `/fitness/trends` and
  `/health/daily`. `/health/daily` is even held in the app's fixture set
  (`frontend/src/fixtures/index.ts:1684`, `healthDailyFixture`) and read by
  nothing; `/fitness/trends` is not held anywhere.

Also: the same document's "Plan & insight" table carries **three
overlapping rows** for `/insights/{…}` (the sets
`{today,progress,load,chips,rolling,streaks,pace,challenges}`, then
`{today,progress,load,chips,rolling,streaks,year,challenges}`, then
`{today,progress,load,heatmap,chips,pace,challenges}`). They are three
successive amendments stacked rather than overwritten.
`crates/stride-api/tests/route_parity.rs` unions them, so the test passes —
but a reader sees three contradictory route lists where there should be one.

⚠ **Needs your approval**: `docs/contract/api-paths.md` is frozen ("Changing
a route is a plan change and must travel with a PLAN.md edit in the same pull
request"). Correcting the *prose* about which routes the app reads does not
change a route — but the file says it is frozen, so it is your call.

---

#### Drift 4 · The segmentation thresholds claim a contract clause that does not exist

`crates/stride-ingest/src/segment.rs:5`: "The thresholds below are the
**frozen rules** from `docs/contract/core-contract.md`."

`docs/contract/core-contract.md` has no clause stating them. Its sections
are: units · no value · rounding · the direction band · human rendering ·
expected differences · the retry rule · one owner for the sentences · the
FIT file · the story. **The run/walk/stand thresholds are in code only.**

Two fixes, and they are not equivalent:

* **the code is right, the document is behind** → add the clause to
  `docs/contract/core-contract.md` (a contract change ⚠);
* **the document is right** → there is no clause, so the code's comment is
  simply wrong, and it should say "the reference implementation's rules,
  kept" (which is what `crates/stride-core/src/fingerprint.rs:79-85` says of
  its own thresholds).

Either way, **one rule must live somewhere both modules can reach** (§1.3,
step **C1**).

---

#### Drift 5 · `PLAN.md` §2 "Out (deferred)" names four things the code has shipped ⚠

`PLAN.md:989-991`:

> "**Out (deferred, with reasons):** … running-form fingerprint page · gear ·
> **heatmap** · tiles proxy · **monthly stats table** · **HR-zone pages** · …"

The code has all four:

* **running-form fingerprint page** — `/fitness/fingerprint`,
  `crates/stride-api/src/fingerprint.rs`, and the run detail draws it
  (`frontend/src/pages/ActivityPage.tsx`, the *form* card,
  `activity.form.heading`). The plan's own amendment of 2026-11-17
  supersedes the scope line — but the scope line was never updated.
* **heatmap** — `crates/stride-api/src/routes/heatmap.rs`,
  `frontend/src/pages/HeatmapPage.tsx`, `/insights/heatmap`, reached from
  Runs. The plan's amendment of 2026-11-17 supersedes it — and again the
  scope line was never updated.
* **monthly stats table** — `crates/stride-core/src/rolling.rs::monthly_totals`,
  `crates/stride-core/src/year.rs`, and `/insights/rolling`'s `months[]`.
* **HR-zone pages** — `/fitness/zones`, `crates/stride-api/src/routes/
  data.rs`, and the run detail's *effort* card.

The pattern: **the amendments were appended to `PLAN.md` and §2 was never
revised.** Four amendments (`2026-11-17` fingerprint, `2026-11-17` heatmap,
`2026-11-20` year, `2026-11-20` pace) each supersede the scope line.

⚠ **Needs your approval**: `PLAN.md` is the plan of record. It is folded
into the same documentation pass as **Drift 2** (step **P1**).

---

#### Drift 6 · `PLAN.md` §2 "In" omits the athlete profile

`PLAN.md:988-989`:

> "**In:** Connect (Garmin key) · Sync (incremental) · Today · Runs ·
> Activity detail · **Run film** · Progress · Body · Plan week · Compare ·
> FIT import (any watch) · Settings (goal, pause, note, connect, about)."

**The athlete profile is not in the list**, though it is a shipped
destination with its own wall, its own index, its own panels and its own
route. The run film *is* in the list. So the scope line under-counts by one
and over-counts by nothing. **Folded into step P1** with **Drift 5**.

---

#### Drift 7 · The contract's zone-model clause and the decision disagree ⚠

`docs/contract/api-paths.md`, "The zones and the zone model":

> "`/fitness/zones?id=…` answers one question about one run … The route does
> not rebuild bands from the run's own maximum heart rate: two runs measured
> on two scales are not comparable, and a band that is not the athlete's
> configuration cannot be named, dated or audited."

and "The rolling view → The zones":

> "The split of the training is measured against the athlete's **stored zone
> model**, and the model travels with it: the day it was read and its own
> source line."

Neither sentence says **which** stored model when the store holds several —
and it does. `crates/stride-store/src/repos/zones.rs:54` returns the most
recent one; `crates/stride-store/src/repos/zones.rs:93` (`hr_zone_models()`)
returns them all, oldest first, and **nothing reads it**. The schema keeps a
dated history (`crates/stride-store/src/schema.rs:160`) that **no route
uses**.

**D3 settles the rule** (per-run dated model, for the zone-time answers,
staged). **The contract's prose must change with it** — the clause that says
"the stored zone model" must say "the model in force on the day of the run",
and the window answer's single `zones.model` becomes the list of the models
the window's runs were measured against. ⚠ It is a contract change.

---

#### Drift 8 · The contract's `/activities/{id}` shape is the shape the cut changes ⚠

`docs/contract/api-paths.md`, the shapes table:

> "| `/activities/{id}` | `ActivitySummary` and `splits: {…}[]`, `elevation:
> {distance, elevation}[]`, `samples: {offset_s, pace, heart_rate}[]`,
> `plan_line: {text} \| null`, `note: string \| null` |"

The `elevation` array is a **separate series with its own x-axis**
(distance), and `samples` carries no distance, no altitude and no movement
kind. §2.1 changes it, and the contract line changes with it.

---

#### Drift 9 · The contract's `/insights/streaks` shape row is stale

Line with:
`| /insights/streaks | generated_at, date, days: {current, longest}, weeks: {current, longest} |`
and, later:
`| /insights/streaks | generated_at, date, days: {current, longest}, weeks: {current, longest}, months: {current, longest} |`

The router answers the **second** (the DTO is `StreakInsight` with
`months: MonthStreakDetail`, `crates/stride-api/src/routes/rolling.rs`), and
the **first** row is stale. The app declares all three
(`frontend/src/lib/types.ts`, `StreakInsight`). **The document's first row
is wrong.** Folded into step **P3**.

---

#### Drift 10 · The pace page's answer is two different windows under one name — **settled by D1/D2**

`docs/contract/api-paths.md`, "The pace page":

> "The window is the Progress window — the same `weeks` parameter (1 to 104,
> default 12) and the same calendar weeks the volume chart draws, the
> current partial week included."

and "The rolling view → The window":

> "The window is **the last `N` days ending on the day the answer is about**,
> and the comparison period is **the same `N` days ending a day earlier**."

**Both are true, and they are different windows.** `/insights/progress` and
`/insights/pace` use calendar weeks
(`crates/stride-api/src/routes/insight.rs:262`,
`week_start(current).add_days(-7 * (weeks - 1))`); `/insights/rolling` uses a
rolling N-day window (`crates/stride-core/src/rolling.rs:86`
`RollingWindow::new(days, to)`, where `from() = to - (days - 1)`). **A
"12 weeks" Progress card and a "12 weeks" rolling card cover different
spans.**

**D1 settles it**: the Progress wall's window is the last *N* calendar weeks
through today, and the rolling route is **migrated** to it. **D2 settles the
comparison**: every window-shaped answer compares with the same span, **one
calendar week earlier**, and `RollingWindow::previous()`'s one-day shift is
retired. §2.2 prices the migration.

---

## 2. The changes the new features need, as cleanup, not as features

### 2.1 `SamplePoint` gains `distance_m`, `altitude_m` and the movement kind; the `elevation` array goes

**The parent's decision is unchanged** (**D7**): the consolidated chart is
one chart over pace / heart rate / altitude, one shared cursor, **time on the
x-axis**, the athlete's **own zone bands behind the heart-rate line as well
as in the table**, and **altitude on its own scale, default off** with its
own metres label.

**The change.**

* `crates/stride-api/src/dto.rs:157` — `SamplePoint` gains
  `distance_m: Option<RenderedValue>`, `altitude_m: Option<RenderedValue>`,
  `kind: Option<&'static str>` (`"run"` / `"walk"` / `"stand"`).
* `crates/stride-api/src/dto.rs:148` — `ElevationPoint` and
  `ActivityDetail::elevation` (`dto.rs:187`) go.
* `crates/stride-api/src/routes/series.rs:108` `elevation()` goes;
  `series.rs:124` `samples()` gains the three fields from the decoded
  sample (`sample.distance_m`, `sample.altitude_m`, `sample.kind` — the
  `SegmentKind` the ingest already computed,
  `crates/stride-ingest/src/segment.rs:19`).
* `crates/stride-api/src/routes/data.rs:155-156` — the two calls become one.

**Who else reads it.** This is the whole list:

| the reader | what it does with the shape | file |
|---|---|---|
| the run detail | `data.samples` → the HR chart, the pace chart; `data.elevation` → the elevation chart | `frontend/src/pages/ActivityPage.tsx:133,149,155,160-176,557,589` |
| the app's declarations | `ActivityDetail.elevation` / `.samples` | `frontend/src/lib/types.ts:257-258` |
| the contract | the shape row | `docs/contract/api-paths.md` (the shapes table) |
| the shape test | asserts the sample's fields | `crates/stride-api/tests/frontend_shapes.rs:392-400` |
| the shape test | the field list | `crates/stride-api/tests/shapes.rs:96` |
| the fixture set | the captured answers | `frontend/src/fixtures/answers.json` (generated) |
| **Compare** | `CompareSeries { a: Vec<SamplePoint>, b: Vec<SamplePoint> }` | `crates/stride-api/src/dto.rs:1538-1540`; `frontend/src/lib/types.ts:347-354` (`CompareSeriesPoint`); `frontend/src/pages/ComparePage.tsx:67-85` |

**The Compare surprise.** `/compare` reuses `SamplePoint` for its two
series. So the shape change **also changes `/compare`'s wire shape**, and the
app's `CompareSeriesPoint` is a *separate* declaration from `ActivityDetail`'s
inline sample type. **Two options:** (a) give `/compare` its own DTO
(`CompareSeriesPoint`) and leave the shape change to `/activities/{id}` —
one line of blast radius instead of two; or (b) let both change together.
**Recommend (a).**

**The shape change is not a cleanup — it is a contract change.** It travels
with a `PLAN.md` edit (the contract's own rule) and with
`docs/contract/api-paths.md`.

**Why the x-axis is time, not distance** (the surface inventory's §3, which
the parent has accepted): every core observation about a run is a difference
of **times** (`crates/stride-core/src/fingerprint.rs` — the drift is "second
half minus first half"), and a stopped stretch covers no distance but is real
elapsed time, and the product has a whole bucket for it. The elevation
series therefore moves onto the time axis, and the separate distance-keyed
elevation array really does go. **The cost:** the elevation chart loses its
distance scale — which **D8.3** answers: altitude keeps its own scale and
its own metres label, and is **default off**, so the default chart is pace +
heart rate with the athlete's own bands behind the heart-rate line.

### 2.2 One window — the migration (**D1**, **D2**)

**The decision, restated.**

* **window** = the last *N* calendar weeks, from the start of the earliest
  one **through today**: `week_start(today) − 7 × (weeks − 1)` → today.
* **compared period** = the **same span ending seven days earlier** — the
  week that came in against the week that fell out, so the partial week
  compares with the partial week.
* **every** window-shaped answer uses that comparison.

**What already obeys it.**

| the answer | file | its window |
|---|---|---|
| `/insights/progress` | `crates/stride-api/src/routes/insight.rs:262` | `week_start(current).add_days(-7 * (weeks - 1))` → current ✓ |
| `/insights/pace` | `crates/stride-api/src/routes/pace.rs:60` | the same ✓ |
| `/insights/load` | the contract: "the Progress window" | ✓ |
| `/insights/today` | `crates/stride-api/src/routes/insight.rs:88-96` | the week to date against **the same elapsed days of the previous calendar week** — which *is* the same span seven days earlier ✓ |
| the Eddington number | `crates/stride-api/src/routes/insight.rs:432` | over the same window ✓ |

**What does not: the rolling route.** `/insights/rolling` is the odd one
out, and it is the migration.

**The migration, concretely.**

**The core's window type changes shape.** Today
(`crates/stride-core/src/rolling.rs:75-112`):

```rust
pub struct RollingWindow { pub days: i64, pub to: Date }
    pub fn new(days: i64, to: Date) -> Self
    pub fn from(&self) -> Date { self.to.add_days(-(self.days - 1)) }
    pub fn previous(&self) -> RollingWindow { /* the same length, a day earlier */ }
```

A **fixed length** cannot express the calendar-week window: the last twelve
calendar weeks through a Wednesday is **79 days**, not 84, and the length
changes with the day of the week. So the type becomes **two ends**:

```rust
// crates/stride-core/src/rolling.rs
pub struct Window { from: Date, to: Date }
impl Window {
    pub fn calendar_weeks(weeks: i64, today: Date) -> Self   // D1
    pub fn days(days: i64, today: Date) -> Self                // the profile's ladder
    pub fn from(&self) -> Date; pub fn to(&self) -> Date
    pub fn days(&self) -> i64 { /* to - from + 1 */ }
    pub fn contains(&self, date: Date) -> bool
    pub fn previous(&self) -> Window { /* THE SAME SPAN, SEVEN DAYS EARLIER (D2) */ }
}
```

`previous()` is the **only** comparison rule in the core after this, and it is
a **seven-day translation** — not the one-day shift it is today.

**The two constructors, and why there are two.** The Progress wall's window
is calendar weeks (**D1**). The **athlete profile's ladder** — 30, 60, 90
days (`frontend/src/pages/useWall.ts:87-95`, `/insights/rolling?days=N`) —
asks for a number of *days*, and "the last 30 days" is a window the athlete
chose. It survives as a **second constructor of the same type**, and it
adopts the same comparison rule (**D2**): the same span, seven days earlier.
**One type, two constructors, one comparison.** *(If the profile ever joins
Progress — it does not, **D8.1** keeps its own door — the ladder would fold
into calendar weeks and the `days` constructor would be retired. Neither
branch exists in this plan.)*

**The breakage list — this is the expensive part of D1.**

| what | where | the cost |
|---|---|---|
| the window type | `crates/stride-core/src/rolling.rs:75-112` | `days` is no longer a stored constant; `previous()` changes its meaning |
| `totals_in`, `compare_windows` | `crates/stride-core/src/rolling.rs:146,164` | take the new type; the arithmetic is unchanged |
| **`profile::facets`** | `crates/stride-core/src/profile.rs:226-320` | **`days` is the denominator of two facets.** `volume_per_week_m: volume_m * DAYS_PER_WEEK / days` and `active_share: active_days / days * 100`. With a variable-length window the denominator becomes `to − from + 1` — **the profile's volume-per-week and consistency values change.** This is the one place the window's length leaks into a number a reader sees. |
| `year::year_window` | `crates/stride-core/src/year.rs:206-212` | the year's window is "the last twelve calendar months" and its comparison is "the same number of days, ending a day earlier" (`year.rs:25`, `crates/stride-api/src/routes/year.rs:60`). **Under D2's one rule, the year's comparison becomes the same span seven days earlier.** A behaviour change on the year page's *what changed*. |
| `year::days_of`, `totals_of` | `crates/stride-core/src/year.rs:231,407` | take the new type |
| the api's rolling route | `crates/stride-api/src/routes/rolling.rs:70` | `window_days(request)` → the window builder; `window.previous()` at lines 86, 105, 115; `window_of` at 189 |
| the api's year route | `crates/stride-api/src/routes/year.rs:60,100` | the comparison |
| the api's seam | `crates/stride-api/src/rolling.rs:166,184,219` | `zone_split` and `window_comparison` take the new type |
| the DTO | `crates/stride-api/src/dto.rs:2045` | `RollingWindow { from, from_label, to, to_label, days }` — **the shape is unchanged; the number in `days` changes** (79, not 84) |
| the app | `frontend/src/lib/types.ts` (`RollingInsight`) | **no shape change** — `days: number` |
| the app's wall | `frontend/src/pages/useWall.ts:87-95` | the ladder still asks for 30/60/90 days |
| the app's copy | `frontend/src/pages/progressPanels.tsx` | the window's labels are the api's `day_label`s — no new string |
| the core's tests | `crates/stride-core/src/rolling.rs:824-866` and `crates/stride-core/src/year.rs:602-661` | `window.days` assertions change; `year.rs:659` asserts `window.days == 347` |
| the api's tests | `crates/stride-api/tests/rolling.rs`, `year.rs`, `load_surface.rs` | expected numbers change |
| the fixtures | `frontend/src/fixtures/answers.json` | **regenerated** (§4.4) |

**What does *not* break.** The Progress totals, the pace page, the load and
Today already use the calendar-week window and the seven-day shift. **D1 is
mostly a migration of one route and one core type, plus the profile
facets' denominator.**

**The one number the user must be told about:** the **profile's** volume a
week and consistency change, because the window's length is no longer a
round number of days. **The Progress totals do not change** — they already
use the calendar weeks.

### 2.3 The zone model, staged (**D3**)

**The decision, restated.**

* **Stage 1 (this cleanup pass):** every run is measured with **the zone
  model in force on the day of the run**, for the **zone-time** answers —
  the run detail's zone table, the window's zone split, the ledger.
* **Stage 2 (later, separately, with its own announcement):** the **load
  model** (`load.rs` — TRIMP's threshold and max heart rate) moves to the
  dated model per run. It changes load history the product has shown since
  v1, so it is its own change, not a quiet refactor.
* **Always:** every answer names the model it used and the day that model
  was read (`zones::model_source`).

#### Stage 1 — the zone-time answers

**The rule today.** `crates/stride-store/src/repos/zones.rs:54`
`current_hr_zone_model()` → `ORDER BY read_at DESC, id DESC LIMIT 1` —
**the model read most recently**.

**The rule after stage 1.** Each run is measured against the model in force
on its date — the last model read on or before the end of that day.

**The store already has the data.** `crates/stride-store/src/repos/zones.rs:93`
`hr_zone_models()` returns **every** model, oldest first, and **nothing reads
it**. A new method `hr_zone_model_on(date)` — "the last model with
`read_at <= end of that day`" — is a query in the same file. **No schema
change** (`docs/contract/schema.sql` and
`crates/stride-store/src/schema.rs:160` are untouched), **no migration**.

**The readers, and what changes for each.**

| # | the reader | file | the change |
|---|---|---|---|
| 1 | `/fitness/zones` — the run detail's zone table | `crates/stride-api/src/routes/data.rs:695` | the model for **this run's date**. The answer's `model.read_on` and `model.source` already name the model and the day — **the shape does not change; the selection does.** |
| 2 | `/activities/{id}/film` — the film's ribbon | `crates/stride-api/src/routes/film.rs:53` | the model for this run's date. The film already names the model it used (`FilmModel`). |
| 3 | `/fitness/fingerprint` — the run detail's form and its zone distribution | `crates/stride-api/src/fingerprint.rs:188` (`stored_model`) | the model for this run's date. `FingerprintZoneModel.source()` (`crates/stride-core/src/fingerprint.rs:164`) already names it. |
| 4 | **`/insights/rolling` — the window's zone split** | `crates/stride-api/src/rolling.rs:266` (`stored_model`), used at `rolling.rs:186` (`zone_split`) | **the hard one.** The window's five zone times would each come from its own run's model. The answer's **single** `zones.model` can no longer name one model — **the shape changes** (§ below). |
| 5 | `/insights/year` — the year's zone mix | `crates/stride-api/src/year.rs:46` | the same as #4, per day and per month. **It is a zone-time answer, so it is in stage 1.** |
| — | the **ledger** | not written yet | it is in stage 1's scope: the ledger must measure every run against its own dated model, or it would disagree with the run detail. **D3's sequencing consequence: the ledger ships after stage 1.** |

**The shape change for the window answers.** `/insights/rolling`'s
`zones.model` (one model, named) becomes the **list of the models the
window's runs were measured against**, each named with its read day:

```text
zones.model   { read_on, read_on_label, source }          →  the shape today
zones.models  [{ read_on, read_on_label, source }, …]     →  the shape after stage 1
```

`zones::model_source` (`crates/stride-core/src/zones.rs`) already builds the
source line — **one call per model**. The app's *Where the time went* card
names them all. **This is a contract change**
(`docs/contract/api-paths.md`, "The rolling view → The zones"), and it needs
**one new string** in the core's catalogue for the plural case
(§1.4).

**The cost of stage 1, honestly.**

| dimension | the price |
|---|---|
| the store | one new method in `crates/stride-store/src/repos/zones.rs`. **Small.** |
| the api | a new seam, `model_for(api, date)`, replacing the three copies of `stored_model`/`zone_model` (§1.3). **Medium** — and it must be done *before* the rule changes, or the rule is changed three times. |
| the core | **the zone computation needs no change at all**: `zones::time_in_zone(floors, samples)` already takes the floors as an argument. The core's *shape* changes only where it names a model: `crates/stride-core/src/fingerprint.rs:164` (`FingerprintZoneModel.source()`) — which already names the model and the day, so **it is already right**. |
| the answers | `/insights/rolling`'s `zones.model` → `zones.models`; `/insights/year`'s the same. **A contract change.** |
| the product | **every zone number in the product changes**, retroactively, for any athlete who has changed their watch configuration. A run from March measured against November's model is a different number today. |
| the tests | `crates/stride-api/tests/zone_model.rs`, `zones.rs`, `fingerprint.rs` all assume "the most recent model". Each needs a **second** model in the seed to prove the rule — **that is the test that proves stage 1.** |
| the fixtures | `frontend/src/fixtures/answers.json` is the router's captured answer — **regenerated** (§4.4). |

**The verdict: stage 1 is large but it is one rule in one place, and it must
be its own pull request.** It is the second of the two behaviour changes the
user must be told about (§4.1).

#### Stage 2 — the load model (NOT in this cleanup pass)

`crates/stride-api/src/load.rs:161` (`zone_model`) → the core's
`LoadZoneModel` (`crates/stride-core/src/load.rs:199`) → TRIMP's threshold
and the maximum heart rate. **Every day of the window is weighed against one
model today.** Under stage 2, each day is weighed against the model in
force on that day, and the load's **assumption line**
(`crates/stride-core/src/load.rs:381`, `zone_model_read_on`) — which names
**one** model and **one** day — can no longer name one.

**It is not in this plan's steps**, because **D3** says so: it changes load
history the product has shown since v1, and it is "its own change, not a
quiet refactor". **What this plan owes it:**

1. the **seam** (step **C2**) makes it possible — one `model_for(api, date)`
   instead of three copies;
2. the **core's load engine** already takes the model as an input
   (`LoadInputs.zone_model`, `crates/stride-core/src/load.rs:243`), so the
   *shape* of the core's load is not the blocker — the **api's** choice of
   one model per window is;
3. the **assumption line** needs a plural form, like `zones.models` above.

**A note the implementation must carry:** stage 1 changes the zone times and
stage 2 changes the load, and the load's *scenario* and *form* are read on
the wall next to the zone split. **A window that spans two models will
show two sets of facts side by side.** The wall's *This week* and *The load*
cards must each say which model they were measured against, or the reader
sees two numbers that disagree and no reason.

### 2.4 The watch's own workout steps (**D4**) — a cleanup step with its own test data

**The decision, restated.** Repeated work = **the watch's own workout steps
only**: a run is repeated work when **two or more of its laps carry the step
name `interval`**. The lap-shape heuristic is **deleted** — it answers
"repeated work" for a plain run with three equal auto-laps
(`real_garmin_run.fit`) and five other files, because auto-laps are equal by
construction.

**What the ledger's rule was, and why it goes.** The heuristic (laps of at
least 1 km, a longest run of consecutive laps within ±10 %, at least three
laps, at least half the run's moving time) is a **product-invented
threshold** that a plain run satisfies by accident. **D4** replaces it with
a fact the watch recorded.

**The decode, and what it costs.**

| what | where | the cost |
|---|---|---|
| the string machinery | `crates/stride-ingest/src/fit.rs:290-334` | **already exists.** `RawValue::Text` and `RawMessage::text(definition_number)` read a string field by its definition number, and `decode_value`'s `0x07` arm (`fit.rs:631`) already turns it into text. The device's product name is read the same way (`fit.rs:897`, `message.text(27)`). **No new value type.** |
| the lap's field | `crates/stride-ingest/src/fit.rs:846-866` (`MESG_LAP`) | `RawLap` (`fit.rs:736`) gains `wkt_step_name: Option<String>`; the arm gains one `message.text(N)` call. **One line of parsing, one field.** |
| the lap's output | `crates/stride-ingest/src/fit.rs:71-108` (`FitLap`) and `build_laps` (`fit.rs:1395`) | `FitLap` gains `step_name: Option<String>`, carried through `running_laps` (`fit.rs:1455`) with the rest of the lap. |
| the api's seam | `crates/stride-api/src/routes/series.rs::splits` (`series.rs:88`) and `crates/stride-api/src/fingerprint.rs` | the lap's step name must reach the core's `LapReading` (`crates/stride-core/src/fingerprint.rs:124`) — the core reads it, the ingest classifies it. |
| the core | `crates/stride-core/src/fingerprint.rs` | `LapReading` gains `step_name: Option<String>`; the repeated-work rule is **not in the core's fingerprint** — it belongs to the ledger's module (`crates/stride-core/src/stimulus.rs`, not yet written). |

**The one thing to verify before the code is written.** The parent says
**lap definition 24, `wkt_step_name`**. In the FIT profile the field number
for `wkt_step_name` is **24 on the `workout_step` message (global 27)** and
**28 on the `lap` message (global 19)** — and the step a lap belongs to is
established by the **`workout_step` message's timestamp**, not by the lap
message. **The repository cannot settle this**: the corpus carries no such
field (below). **The implementer must check the number against the FIT
profile before writing the decoder**, and the decoder should read **both**
places — a step name carried on the lap, and a `workout_step` message that
covers it.

**The corpus: verified empty of step names.** I read every file in
`fixtures/fit/` for printable strings:

```bash
for f in fixtures/fit/*.fit; do strings -n 4 "$f" | grep -aiE "interval|recovery|warmup|step|repeat|workout"; done
```

**No file carries a workout-step name.** The only readable strings in the
corpus are the device names in the synthetic files ("Forerunner 965",
"Forerunner 265", "Edge 1040", "Venu 3", "COROS DURA") and the file
magic. **The parent's claim holds: no file in `fixtures/fit/` carries the
field.**

**So the test data must be made.** This is the part that is genuinely new
work in this plan:

* **the corpus is frozen** (`scripts/check-corpus.sh` holds the SHA-256s) and
  `CONTRIBUTING.md` says "**Do not add real data**" — a synthetic FIT with
  a step name is not real data, but it is a **new file in a frozen
  directory**, which needs a decision: a new file in `fixtures/fit/` with
  its SHA-256 registered, or a **synthesised file written by a test**.
* **the cheap, honest option:** the golden corpus already has a *writer*?
  No — the golden files are captured answers. **The cheapest route is a
  synthetic FIT built in the test** (a handful of `lap` messages with
  definition 24 = "interval" and 28 = "recovery", plus the records), decoded
  by the real decoder, asserted on. **That keeps `fixtures/fit/` frozen.**
* **the cheaper-still option:** a **unit test on the decoder** — construct
  the bytes of one lap message in the test and assert the step name comes
  out. That needs no file at all, but it proves the *decoder*, not the
  *pipeline*.
* **the belt-and-braces option:** both — a decoder unit test, plus one
  synthetic file in a **new** fixture directory (not `fixtures/fit/`) that
  the golden corpus does not hash.

**Recommendation: the decoder unit test plus one synthetic file in a new
directory** (`fixtures/fit-steps/`, with its own SHA-256 list), and **no
change to `fixtures/fit/`**. The step that decides this is **step S3** and
it is **the one step in this plan that needs a decision about the frozen
corpus**.

**The ledger's states (**D4**), and what they cost in the product.**

| state | the answer |
|---|---|
| steps present | the row answers: the runs whose laps carry `interval`, their time, their share |
| no run in the window carries steps | **no value** — "the runs in the window carry no workout steps" — **never "0 m of repeated work"** |
| steps present but none is `interval` | **a measured fact**, not a missing value: "no run in the window was an interval" |

**The honest consequence, which the ledger's own note already names:** until
the athlete records a structured workout on the watch and syncs it, **this
row has little to say** — "which is the correct answer". **On today's
corpus, and on every run this product has ever seen, the row is *no value*.**
**That is a real cost of D4 and the user should hear it once:** the ledger's
fifth row is empty for everyone until the hardware cooperates.

### 2.5 The ledger's shares name their denominator (**D5**)

**The decision.** The intensity bands' shares are over the **record
stream's movement** (so the bands and the outside add up to 100 %). The
longest run's share is over the **stored moving time** (the watch's timer —
the run's own number). **Shares over different denominators are not added
together, and each row names its denominator.**

**What it costs in the cleanup.**

* **The core already measures both.** `zones::TimeInZone::moving_s()` is the
  record stream's movement; `rolling::WindowTotals::moving_time_s` is the
  watch's timer (`crates/stride-core/src/rolling.rs:145`). **No new
  measurement.**
* **Two denominators in one answer** means the answer carries **two
  denominators**, and the app must not add across them. The app's own
  arithmetic is the risk: the frontend "never parses a number to show it"
  (the contract), but a *sum* of two shares from different denominators
  would be a new piece of arithmetic in the app. **The panel must not do
  it.**
* **A string per denominator.** Each row names its denominator — two
  catalogue entries in the core (one per denominator), and the ledger's
  sentences carry them.
* **The API shape** carries both denominators, so the ledger's answer is
  wider than one number. **A contract change** when the ledger's route
  lands.

**This is a small, cheap and important rule, and it is a *design* constraint
on the ledger's card, not a computation.**

### 2.6 The API change the ledger needs, and where a cache would go

**The change the ledger needs.** The ledger needs **each run's samples and
its laps, decoded on read**. The precedent is
`crates/stride-api/src/pace.rs::pace_runs` — it decodes every run's stored
FIT and builds the core's `PaceRun`s.

**The cost is real.** A 12-week window at 61 runs is **61 file decodes per
read**. Today the same cost is paid by:

* `/insights/pace` — `crates/stride-api/src/routes/pace.rs:78` →
  `crate::pace::pace_runs(api, &runs)` → 61 decodes;
* `/insights/rolling` — `crates/stride-api/src/rolling.rs:298` `zone_samples`
  → one decode **per run in the window and the previous window**;
* `/insights/year` — `crates/stride-api/src/year.rs:91` → one decode per run
  in the year;
* `/insights/heatmap` — `crates/stride-api/src/routes/heatmap.rs:63` → one
  decode per run with a position.

**So this is not a new cost — it is a fifth copy of an existing cost, and the
right cleanup is to pay it once.**

**Where the cache goes: `crates/stride-api/src/routes/series.rs::decoded`.**

That function (`series.rs:22`) is **the single choke point for every
decode**: `data.rs:62`, `data.rs:152`, `data.rs:712`, `film.rs:76`,
`fingerprint.rs:74`, `rolling.rs:299`, `year.rs:91`, `pace.rs:34`,
`update.rs:63`, `heatmap.rs:63`, `routes.rs:303`, `routes/insight.rs:73`,
`routes/insight.rs:395-396`. **One cache there, and every route above gets
it.**

**The design.**

* **Where the cache lives:** `crates/stride-api/src/state.rs`, on the `Api`
  struct (which already holds `garmin: Mutex<…>`, `export: Mutex<…>`,
  `sync: Mutex<()>`). A `decoded: Mutex<HashMap<i64, Arc<DecodedRun>>>`.
* **The key:** the activity id **plus the file's digest**. The store already
  has both: `crates/stride-store/src/repos/activity.rs:553`
  `fit_sha256(activity_id)` and `crates/stride-store/src/hash.rs:31`
  `sha256_hex`. The key `(id, sha256)` means **a re-imported file invalidates
  its own entry** with no explicit invalidation — the blob for an id *can*
  change (`activity.rs:263`, `fit_blob = excluded.fit_blob`).
* **What is cached:** the decoded `stride_ingest::FitActivity`, shared by
  `Arc`, so sixty-one callers hold one decode, not sixty-one copies.
* **What is not cached:** the *answers*. Every derived series is recomputed
  on read — that is the product's rule, and the cache must not become a
  place where a derived value is stored.
* **The bound:** the cache must be bounded. A simple LRU or a size cap with
  a clear is enough; **the workspace denies `unwrap`/`expect`/`panic`**
  (`Cargo.toml`, `[workspace.lints.clippy]`), so the eviction path is code,
  not an assumption.
* **The privacy consequence:** the cache holds the athlete's positions in
  memory for longer than the call. That is the same posture as the store's
  own in-memory pages, but **it is a posture statement to make in the PR**,
  not a decision to make silently.
* **The core is untouched:** the core is pure, and the cache is in the api,
  which is allowed I/O.

**What it buys.** The ledger's 61 decodes become 61 decodes the first time
and zero thereafter; `/insights/pace`, `/insights/rolling` and
`/insights/year` all get faster at the same time. **This is the single
highest-value cleanup in the plan**, because it is the one that makes the
next feature cheap.

### 2.7 The activity search response's discarded fields (**D10**)

**The decision.** The activity **search** endpoint Stride already calls
carries fields it currently discards. Keeping them costs **no extra
requests** — only storage and schema. Each is stored **with its provenance**
(source, the day the response carried it, the day it was read), like
`wellness_observations`. Absence is a missing value with a reason, never
zero.

**What is kept.**

| the field | what it is |
|---|---|
| aerobic and anaerobic **training effect** and their messages | the source's own reading of the run's aerobic and anaerobic work |
| `trainingEffect` | the source's overall training effect |
| `hrTimeInZone_1..5` | **the source's own time in each zone — an independent cross-check on our computed zone time, never a replacement for it** |
| `directWorkoutRpe` | **the athlete's own recorded effort** |
| `directWorkoutFeel` | **the athlete's own recorded feel** |
| moderate and vigorous minutes | the source's own minutes |
| respiration | the run's own respiration |
| steps | the run's own steps |

**What is kept out (the contract excludes them):** VO₂ max, training stress
score, activity training load, lactate threshold, intensity factor, stress,
body battery. **Garmin's estimates are judgements, not observations.** The
precedent is the wellness table's own comment
(`crates/stride-store/src/schema.rs:186-189`): "The device's own verdicts —
the sleep scores, the HRV status, the VO2 max — are judgements, not
observations, and are not stored."

**Where the fields are today, and what is thrown away.**
`crates/stride-api/src/routes/app.rs::new_activity` (line 1384) builds the
`NewActivity` from the search entry and keeps **nine** things — name, sport,
start time, local date, distance, moving time, elapsed time, elevation gain,
average and max heart rate — and **the numbers come from the decoded FIT,
not from the entry**. Everything else the entry carries is discarded
silently.

**The precedent to copy, exactly.** `crates/stride-api/src/wellness.rs` is
the model: a **pure** module (no clock, no store, no I/O) that maps a
Garmin payload to `NewWellnessObservation` rows with their provenance, with
three rules stated at the top of the file:

* the day is the response's own;
* the missingness is kept — a field the payload does not carry has **no row
  at all**; a field it carries as null has a row whose value is null;
* **nothing is derived** — every value is one the source reported, in the
  source's own unit.

**The price, in five parts.**

| # | the part | the files | the cost |
|---|---|---|---|
| 1 | **the schema** | `crates/stride-store/src/schema.rs` — `SCHEMA_VERSION` 5 → **6**; a new `TABLE_STATEMENTS` entry and a `V6_INDEX_STATEMENTS`; a new `Migration { version: 6 }` in `migrations()` (line 341) | a new table `activity_observations`, shaped like `wellness_observations` (`schema.rs:195`): `id, activity_id, source, endpoint, field, ordinal, value, text_value, unit, observed_at, retrieved_at`. **A version bump is a schema change** — and `docs/contract/schema.sql` **must** change with it, because `crates/stride-store/tests/schema_contract.rs:142` compares the store's schema to the contract's file, statement for statement, and the table list at line 105-130 is asserted exactly. |
| 2 | **the store** | `crates/stride-store/src/repos/observations.rs` (the wellness repo) — a sibling for activities; `crates/stride-store/src/model.rs` — `ActivityObservation` / `NewActivityObservation` (the wellness pair is at `model.rs:284` and `model.rs:317`); `crates/stride-store/src/lib.rs:73-74` (the re-exports) | **the wellness repository is the template.** The write must be idempotent, like the wellness one: a later read replaces the value it carried and keeps the reading time of the read that carried it, so a failed read leaves the previous value in place. |
| 3 | **the sync mapping** | `crates/stride-api/src/routes/app.rs::new_activity` (line 1384) and a new pure module beside `crates/stride-api/src/wellness.rs` — say `crates/stride-api/src/activity_observations.rs` | the search entry's JSON is **already in hand**; the mapping is the wellness module's `Writer` pattern, with the same three rules. The rows are written **with the activity**, in the same transaction if the store allows it (`crates/stride-store/src/repos/activity.rs:242-280` is one `INSERT … ON CONFLICT`). **No extra HTTP request** — that is the whole point of **D10**. |
| 4 | **the export and the backup** | `crates/stride-store/src/export.rs` — the frozen table list (the wellness entry is at line 508) | the new table joins the frozen export list, with its `select` / `insert` / `columns`. **The import path must restore it** (`/settings/import`, `crates/stride-api/src/routes/export.rs`). `crates/stride-store/tests/export_import.rs` holds the round trip. **A backup that silently dropped the observations would be a data loss the athlete cannot see.** |
| 5 | **the test data** | `crates/stride-api/tests/sync_delta.rs` (the search entry fixture, line 176); `crates/stride-store/tests/schema_contract.rs` (the table list); `crates/stride-store/tests/export_import.rs`; a new test in `crates/stride-api/tests/` | the search-entry fixture must carry the new fields, in **three states**: present with a value, present and null, and **absent**. The absent case must produce **no row**. And there must be a test that the **excluded judgements are not stored** — a whitelist of field names, asserted against what the mapping wrote. |

**What the app does with it: nothing, yet.** The fields are stored, not
shown. **The value of stage 1 of D10 is that the cross-check exists**:
`hrTimeInZone_1..5` is Garmin's own time in each zone, and **our computed
zone time is computed against the athlete's own dated model** (§2.3). When
the two disagree, the athlete's model was in force on the day — and the
ledger can say so. **That cross-check is a *later* feature; this step only
keeps the fact.**

**One caution.** `hrTimeInZone_1..5` is **Garmin's** zone model, not the
athlete's. It is a **cross-check on the magnitude** and **never a
replacement** — the contract's rule is that the bands are the athlete's own
stored configuration, named and dated, or they are not the athlete's. **The
storage step must not let the two models be confused in a later reader's
mind: the stored field is named `hrTimeInZone_1..5`, the source's own name,
and it is stored as an observation with its endpoint and its reading time.**
**A cross-check is a fact about the source; the zone time is the product's
own answer.**

### 2.8 What is priced at **zero** (**D6**, **D9**)

**No temperature work. No schema change. No sync change. No ingestion.**

The Garmin Personal API **does** expose a per-activity temperature
(`SummaryDTO.averageTemperature` / `maxTemperature` / `minTemperature`, and
the same three per split, on `GET /activity-service/activity/{activityId}`;
`SummarizedActivityDTO.minTemperature` / `maxTemperature` on the search
endpoint). **On this account the keys are absent — not null** — from all 100
activities of a 12-month search page, all 336 split objects and the detail
payload, while training effect, heart rate and elevation were populated.
**The API carries no weather at all.**

So:

* **the environment axis stays dropped** (**D6**);
* **the FIT temperature decoding that `exploration/stimulus-ledger.md` §5
  proposed is not in this plan** — and neither is the API temperature;
* **the corpus measurement stays in the ledger's §5** as the record that
  closed the question, and **this plan adds nothing to it**;
* if the field ever populates, it is a **dated context fact on the run
  detail**, never an adjustment.

**This plan prices the temperature work at zero, on purpose, so that nobody
re-raises it as a pending item.**

**One thing to note, because it is the same shape as **D10**:** the
temperature field would arrive on the **same search response** Stride
already calls. **If it ever populates, it is a row in the §2.7 table** — no
schema change beyond what **D10** already makes, no sync change, no extra
request. **That is the whole argument for doing **D10** first.**

### 2.9 The picture first, the sentence second (**D11**)

**The directive.** A card is read as a **picture** first — the shape, the
band, the gap, the streak. The sentence is the **caption** under it, and the
accessible fallback. **Absence is drawn, not described.** Two bars are not
two facts. **The chart is readable on its own**, and the caption must not
repeat what the picture already says.

**This is now a first-class requirement of the cleanup, not a nicety.** The
audit below is the whole of what it costs.

#### The audit — every card, as it is drawn today

Read off the code: `frontend/src/pages/progressWall.tsx` (the card bodies,
line 1067 onward) and `frontend/src/pages/progressPanels.tsx` (the panels).

| card | its body | today |
|---|---|---|
| `load` | `LoadPanel` — three charts (`progressPanels.tsx`) | **picture + caption** |
| `volume` | `VolumePanel` — the weekly bars | **picture + caption** |
| `consistency` | `ConsistencyPanel` — a bar chart | **picture** |
| `zones` | `ZonesDistributionPanel` — five rows, each with a proportional bar | **picture + caption** |
| `split` | `ZonesPanel` — three rows with bars — **the same five numbers, regrouped** | **picture + caption**, but **a second drawing of one fact** |
| `week` | `WeekBody` (`progressWall.tsx:1129` body) — a number, a difference, a `<dl>`, a window line | **sentence-only** |
| `window` | `RollingPanel` + `WindowWeeks` (`progressWall.tsx:941`) | **picture + caption** |
| `pace` | `BigValue` + `Note` + `PaceBars` | **picture + caption** |
| `bests` | `BestsPanel` — a `<ul>` of label · value · date | **sentence-only** |
| `profile` | `ShapePanel` — the radar | **picture** — **retired by D7** |
| `facets` | `ProfilePanel` — a `<ul>` of label · value · movement | **sentence-only** |
| `challenges` | `ChallengePanel` → `ChallengeGrid` — a week grid | **picture + caption** |
| `adherence` | `AdherencePanel` — one line, "9 of 10 sessions done" | **sentence-only** |
| `weekday` | `WeekdayTable` — a `<table>` | **table, not a picture** |
| `daytime` | `DaytimeTable` — a `<table>` | **table, not a picture** |
| `distances` | `DistancesTable` — a `<table>` | **table, not a picture** |
| `year` | `YearCalendar` | **picture + caption** |
| `heatmap` | `DistanceCalendar` + `DistanceLegend` | **picture** — retired |
| `months` | `MonthsChart` | **picture** |
| `years` | `YearsChart` | **picture** — retired |
| `streaks` | `StreaksBody` — a `<dl>` of three counts + a longest line | **sentence-only** |
| `eddington` | `EddingtonPanel` — a number, two rows, a note | **sentence-only** |
| `activities` | `ActivitiesTable` | **table** — retired |
| `body` | `VitalsBody` — a `<dl>` of readings | **sentence-only** — retired |

**Six cards are sentence-only today; three more are tables.** The cut
retires three of the sentence-only ones (`body`, and `activities` and
`years` which are pictures/tables).

#### The eleven survivors, and what each one needs

| # | the card | after the cut | what it is today | **what D11 requires** |
|---|---|---|---|---|
| 1 | **This week** (Progress) | `week` + `window` | the `window` half is a picture; the `week` half is a sentence | **redraw** — the week strip must carry the *comparison*, so the card's own picture shows the week against the same elapsed days of the previous week. A number under a picture that does not show it is a caption repeating nothing. |
| 2 | **The load** (Progress) | `load` | picture + caption | **nothing** — but the caption must not repeat the chart (**D11(e)**) |
| 3 | **The weeks** (Progress) | `volume` + `consistency` | picture + caption | **nothing** — two panels in one card is fine; **one caption, not two** |
| 4 | **Where the time went** (Progress) | `zones` + `split` | two drawings of one fact | **redraw — one bar, printed once (D11(d)).** The five zone shares *are* the polarised roll-up; the roll-up is **read off** the same bar, not printed beside it. **And the ledger's strip goes here (D11(c))** — the weeks, coloured by the band each week held, so a five-week hole in the hard band is a hole. |
| 5 | **The pace** (Progress) | `pace` | picture + caption | **nothing** — the records move here (see 6) |
| 6 | **Bests & milestones** (Progress) | `bests` | sentence-only | **redraw** — distance against time, the three records as marks. The surface inventory keeps this card ("cutting a milestone list would be a real loss"), so it becomes a small picture rather than a list. |
| 7 | **Your shape** (Profile) | `profile` (the radar, retired) + `facets` | sentence-only (the facets are a `<ul>`) | **redraw** — **seven bars on a common scale**, each labelled, each with its movement. **This is the natural replacement for the radar**: a polygon over fixed anchors is a claim, and seven bars on a common scale are not. |
| 8 | **The months** (Profile) | `year` + `heatmap` + `months` | picture | **nothing** — one calendar survives, coloured by distance |
| 9 | **The schedule** (Profile) | `weekday` + `daytime` + `distances` | three tables | **redraw** — one **7 × 24 strip**: each run a mark on its weekday and hour, sized by distance. That *is* the shape of the athlete's week; three tables of numbers is not a picture. |
| 10 | **The habit** (Profile) | `streaks` + `eddington` | **both sentence-only** | **redraw** — the Eddington number has a canonical picture: the **E × E grid** of days, coloured where the day reached E kilometres. The streak is a **row of day marks** with the current run held. |
| 11 | **The goals** (Profile) | `challenges` + `adherence` | a grid + one sentence | **the grid is the picture**; the adherence must be **drawn into it** — a row of the week's sessions, coloured done / planned / not done — rather than printed as a line beside it. |

**So: four of the eleven survivors are sentence-only or tables today and
must be redrawn (**This week**, **Where the time went**, **Bests &
milestones**, **Your shape**, **The schedule**, **The habit** — six, of which
four are strictly sentence-only and two are tables), and one must have its
two drawings collapsed into one.**

#### What the pictures need, and what the api does not carry

**This is the real price of D11.** A picture needs its data, and four of
the pictures need data the answers do not carry today.

| the card | the data the picture needs | does the answer carry it? |
|---|---|---|
| **This week** | the previous week's distance over the same elapsed days | ✓ `today.week.previous` (`crates/stride-api/src/dto.rs`, `WeekRunning.previous`) |
| **Where the time went** | **the band each week held** — a `weeks[]` array | ✗ **new.** `/insights/rolling` carries `months[]`, `weekday[]`, `daytime[]` — **no per-week band** |
| **The load** | the daily series | ✓ `load.days[]` |
| **The weeks** | the weeks and their sessions | ✓ `progress.weekly_volume[]`, `progress.consistency[]` |
| **The pace** | the weeks, the bands, the pace-at-HR | ✓ `pace.weeks[]`, `pace.bands`, `progress.pace_at_hr[]` |
| **Bests & milestones** | the records | ✓ `progress.bests[]` |
| **Your shape** | the seven facets | ✓ `rolling.profile[]` |
| **The months** | the months and the days | ✓ `rolling.months[]`, `year.days[]` |
| **The schedule** | **each run's weekday, hour and distance** | ✗ **new.** `rolling.weekday[]` and `rolling.daytime[]` are **aggregates**; a 7 × 24 strip needs the runs |
| **The habit** | **the days that reached E kilometres** | ✗ **new.** The Eddington answer carries `number`, `days_needed_for_next` and `longest_day` — **the grid needs the days themselves**. The core has them: `stride_core::eddington::eddington` takes `DailyDistance[]` (`crates/stride-core/src/eddington.rs`) |
| **The goals** | the challenge weeks and the adherence | ✓ `challenges.challenges[]`, `progress.adherence` |

**Three new fields, each a contract change** to
`docs/contract/api-paths.md` (the `/insights/rolling` shape row, and the
`/insights/progress` shape row for the Eddington's days):

1. `/insights/rolling` — a `weeks[]` array: each calendar week of the
   window, and the band it held (the hard band, the moderate band, the low
   band, or nothing). **This is the strip the ledger card draws (D11(c)).**
2. `/insights/rolling` — the runs' weekday and hour, for the schedule strip.
   (Or the strip is drawn from the `activities` feed the app already calls
   — **check** `frontend/src/pages/RunsPage.tsx`; the feed carries the runs
   with their dates, and a strip needs a date and a distance, which it has.
   **If the feed can carry it, no new field is needed for the schedule** —
   the strip is drawn from `/activities`, which the wall already calls for
   the retired *Most recent activities* card. **That is the cheapest route
   and it is worth checking first.**)
3. `/insights/progress` — the Eddington's days, for the E × E grid. **Or**
   the grid is drawn from the same `/activities` feed: a day reaches E km
   when the sum of that day's runs reaches E km, and the feed carries the
   runs. **Again, check the feed first.**

**The honest conclusion: D11 is cheap in drawing and dearer in data, and
the cheapest data route is the `/activities` feed the wall already calls.**
**Step V1 checks that first and only then asks the api for a new field.**

#### The captions

**D11(e): the chart is readable on its own, and the caption must not repeat
what the picture already says.** That means the captions that *are* the
picture's title, or that restate a number the picture shows, go. The ones
in the catalogue today that are exactly that:

| the caption | where | what it says |
|---|---|---|
| `progress.wall.pace.note` | `frontend/src/i18n/catalogue.ts:311` | "The training week by week, and where the time went by pace." — **the card's own title, said again** |
| `progress.wall.body.note` | `catalogue.ts:429` | retired with the card |
| `progress.profile.facets.note` | `catalogue.ts` | a sentence about the facets, beside the facets' own numbers |
| `progress.wall.year.note` | `catalogue.ts:305` | says what the calendar is |
| `progress.rolling.window` | `catalogue.ts` | the window's length, printed under the picture — **this one is the accessible fallback and stays** |

**The rule to keep:** a caption states **the period** and **the reason for
an absence**, and nothing the picture already shows. A number is still a
number, and a number needs its period and its reason stated in words.

#### The run detail's chart (**D11(b)**)

**Settled, and it is a requirement on the new chart, not an option:**

* the **zone bands stay behind the heart-rate line** — the athlete's own
  configuration drawn on their own data (**D8.2**);
* the **table beneath keeps** the numbers, the **not-moving bucket** and
  the **model's source line** — and after **D3** stage 1, the source line
  names the model **in force on that run's day**;
* **time on the x-axis**, one shared cursor, pace and heart rate in the
  effort band, **altitude on its own scale, default off** with its own
  metres label (**D8.3**).

#### The method lens (**D12**)

**Not cleanup, and it adds no step to this plan.** The lens card is a
feature that lands after the ledger. **Two things this plan owes it:**

1. **D11 applies to it** — the lens card is a picture with a caption, and a
   two-clause rule computes the clause that resolves and states the clause
   that does not.
2. **D12's rule 4 is a store addition** — "the athlete's own cap is a dated
   fact (value, day entered, day last changed), stored beside the zone
   model's dated facts". **That is a new table beside `hr_zone_models`**
   (`crates/stride-store/src/schema.rs:160`), and it is the same shape of
   thing the **zone-model seam (step C2)** already has to build. **C2 should
   be written so the dated-model reader can carry a second dated fact
   without being rewritten** — that is the whole of the debt this plan
   leaves.

---

## 3. The order, and the gates

### 3.1 The order of execution

**Twenty-five steps. Each lands as one commit. Each is small enough to
review in one sitting.** Steps are ordered so that **no step breaks a step
after it**, and so that the two behaviour changes (**W** and **Z**) land
last and alone.

The letters are the families: **B** bridge · **P** paper (the documents) ·
**S** strings · **F** frontend · **C** core · **W** window · **Z** zone
model · **K** the workout-step decode · **O** observations (D10) · **X** the
shape change · **L** the ledger.

---

#### Phase 0 — the live bug (one commit, before anything else)

**Step B1 — The Android bridge's route table.** ⚠ *behaviour on the phone
only*

* **files:** `android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt`
* **change:** add the eleven missing routes to `TABLE`.
* **could break:** the bridge's own tests may assert the table's length; the
  phone's Progress wall, the load card, the streaks card, the challenges
  card, the film, the run update and the update settings all start
  answering — **so screens that were silently showing their error state will
  now show data**, and the viewport matrix may see new content at narrow
  widths.
* **the test that proves it:** the bridge's own unit test
  (`android/app/src/test/java/…/PhoneRoutesTest.kt` if present — *check*);
  plus a manual run of the phone shell. **The Rust side needs no change.**
* **gate:** `bash scripts/gate.sh` (the Kotlin is not compiled by the gate —
  `./gradlew assembleDebug` is the phone's own build, per `CONTRIBUTING.md`).

---

#### Phase 1 — the documents (documentation only; **D8.1** and **D5/D6**)

**Step P1 — `PLAN.md`: five destinations, and the scope line.** ⚠ *a plan
change*

* **files:** `PLAN.md`
* **change:**
  * the six "four destinations" places (577, 603, 692, 771, 982, 1121) →
    five, naming Profile;
  * `PLAN.md:988-989` — add the athlete profile to "In";
  * `PLAN.md:989-991` — remove (or mark superseded by the named amendment)
    *running-form fingerprint page*, *heatmap*, *monthly stats table*,
    *HR-zone pages* from "Out";
  * **a new amendment section** recording **D8.1** (five destinations), and
    the cut's size (24 → 11).
* **could break:** nothing in the code. **But** `PLAN.md` is the plan of
  record and the contract's own rule ("Changing a route is a plan change
  and must travel with a PLAN.md edit in the same pull request") means this
  amendment **must travel in the same pull request** as the contract edits
  (steps W2, Z1, X1).
* **the test that proves it:** none — but `bash scripts/gate.sh` must stay
  green, because `PLAN.md` is read by nothing.

**Step P2 — `README.md`: the four destinations.** ⚠ *a user-facing document*

* **files:** `README.md:49` (and any other "four" in the file — **check
  with `rg -n "four|four destinations" README.md`**).
* **change:** "Today, Runs, Progress and Body remain the four destinations
  at every width." → five, naming the athlete profile.
* **why its own step:** `README.md` is the document a stranger reads, and it
  is **not** coupled to the contract's edits. It can land as soon as the
  decision is recorded, **before** the code changes — which is the point: a
  reader of the README should never be told something the code does not do.
* **the test that proves it:** a **reader's check** — `rg -n "four
  destinations|four destinations at every width" README.md PLAN.md
  CONTRIBUTING.md` returns nothing after the amendment; and
  `frontend/src/app/destinations.ts` is the file the README now describes.
* **note:** `CONTRIBUTING.md` (rule 1, "not negotiable") says four too. It
  is a contributor-facing contract about the information architecture, so it
  **must** be corrected in the same pull request as **P1**.

---

#### Phase 2 — the strings (safe, mechanical, and they shrink)

**Step S1 — Add a "no catalogue entry without a user" check.** (the check
first)

* **files:** `frontend/tests/catalogue.test.ts` (or a new
  `frontend/tests/catalogue-usage.test.ts`).
* **change:** read `frontend/src/i18n/catalogue.ts`, and for each key assert
  that some file under `frontend/src` (excluding `i18n/catalogue.ts` and
  `i18n/index.ts`) references it.
* **could break:** **it fails immediately** — 38 entries have no user. That is
  the point: the check and the removal land together, or the check lands
  with the 38 removed.
* **the test that proves it:** itself.
* **note:** the same check for the core's catalogue cannot live here — the
  core's entries are read from Rust. Put that one in
  `crates/stride-core/src/strings.rs`'s own `mod tests` (the file already
  has a test module), or in `crates/stride-core/tests/`.

**Step S2 — Remove the 38 dead app catalogue entries and the 9 dead core
entries; resolve the two key collisions.**

* **files:** `frontend/src/i18n/catalogue.ts`;
  `crates/stride-core/src/strings.rs`; **then regenerate**
  `crates/stride-core/strings.json` with
  `cargo run -p stride-core --example export_catalogue`.
* **the two collisions:** delete the core's `vital.rest.heading` and
  `status.version` (both unreferenced by the core/api; the app owns them).
  Also decide the three *same-text-different-key* pairs — `vital.sleep.label`
  vs `vital.sleep`, `vital.hrv.label` vs `vital.hrv`,
  `vital.resting_hr.label` vs `vital.restingHr` — and `vital.rest.label`
  ("Rest"), which has no app copy.
* **could break:** `crates/stride-core/tests/plan.rs:234-236` asserts the
  catalogue *contains* `statement.plan.status.{done,not_done,planned}`.
  **That test must change with the removal** — and it is the one place where
  "the test keeps a dead string alive".
* **the test that proves it:** `cargo test -p stride-core` (the
  `the_exported_catalogue_is_valid_json_and_matches_the_file` test at
  `strings.rs:1984` fails if the export is not regenerated);
  `npm --prefix frontend run lint:voice` (it reads the export).

---

#### Phase 3 — the frontend cut (**D7**)

**Step F1 — Retire the radar.**

* **files:** `frontend/src/components/ProfileRadar.tsx` (delete),
  `frontend/src/components/AthleteShape.tsx` (the `RADAR_AXES` import),
  `frontend/src/pages/progressPanels.tsx` (the `RADAR_AXES` import, line 44),
  `frontend/src/pages/progressWall.tsx` (the `ProfileRadar` import at line
  39 and `ProfileRadarBody` at line 211), `frontend/src/pages/
  ProgressPanelPage.tsx` (line 46, line 212), `frontend/src/pages/
  ProfilePage.tsx` (line 51, line 305), `frontend/src/pages/taskAreas.ts`
  (the card `profile`, the panel `shape`), `frontend/src/i18n/catalogue.ts`
  (the eight radar entries).
* **the trap:** `RADAR_AXES` is the radar's **axis list**, and
  `AthleteShape.tsx` imports it *only to name the six axes*. **The naming
  must move to a module that is not the radar** — a new
  `frontend/src/pages/shapeAxes.ts` or `frontend/src/lib/shape.ts`.
  **Deleting `ProfileRadar.tsx` without moving `RADAR_AXES` breaks the
  build.**
* **could break:** `frontend/tests/shape.test.tsx`,
  `frontend/tests/shapeWindows.test.tsx`, `frontend/tests/wall.test.tsx`,
  `frontend/tests/progress-index.test.tsx`.
* **the test that proves it:** `npm --prefix frontend run test:unit`; the
  viewport matrix.

**Step F2 — Retire the `/pace` page; move its two panels onto the wall.**

* **files:** `frontend/src/pages/PacePage.tsx` (delete),
  `frontend/src/app/App.tsx` (the `/pace` route and the import),
  `frontend/src/pages/taskAreas.ts` (the card `pace` now points at a
  Progress panel, not at `/pace`), `frontend/src/pages/progressPanels.tsx`
  (the two panels), `frontend/src/pages/useWall.ts` (the `insights-pace`
  query — it stays, the panel needs it), `frontend/src/i18n/catalogue.ts`
  (the eighteen `pace.*` entries that do not survive).
* **could break:** `frontend/tests/pace.test.tsx` (rewritten against the
  panel, not the page); the wall's card list; **the saved-link rule** — a
  saved `/pace` link lands on Today (`frontend/src/app/App.tsx`,
  `<Route path="*" element={<TodayPage />} />`) with no explanation. **Add a
  redirect**, as `ProgressPanelRoute` already does for the old panel links.
* **the test that proves it:** `npm --prefix frontend run test:unit`.

**Step F3 — Merge the wall's cards.** (24 → 11)

* **files:** `frontend/src/pages/taskAreas.ts` (`WALL_CARDS`, `TASK_AREAS`,
  `WALL_PANELS`, `PANEL_DESTINATIONS`), `frontend/src/pages/progressWall.tsx`,
  `frontend/src/pages/progressPanels.tsx`, `frontend/src/pages/
  ProgressIndex.tsx` (the index becomes the card list),
  `frontend/src/i18n/catalogue.ts`.
* **the merges:** `volume`+`consistency` → *The weeks*; `zones`+`split` →
  *Where the time went* (**D7**: the ledger *is* that card); `week`+`window`
  → *This week*; `profile`+`facets` → *Your shape* (the radar is already
  gone); `challenges`+`adherence` → *The goals*; `weekday`+`daytime`+
  `distances` → *The schedule*; `year`+`heatmap`+`months` → *The months*;
  `streaks`+`eddington` → *The habit*; **retired**: `activities`, `body`,
  `years`.
* **five destinations stand** (**D8.1**): six cards on Progress, five on the
  athlete profile.
* **this step merges; it does not redraw.** The merged cards **keep today's
  bodies** — the sentence-only ones are still sentence-only here. **The
  redraws are step F6** (§2.9, **D11**), and the merged card's body is
  replaced there. **Splitting them keeps the cut reviewable**: a merge is a
  rearrangement and a redraw is new drawing code, and `CONTRIBUTING.md`
  says "Do not mix a refactor with a behaviour change".
* **could break:** `frontend/tests/wall.test.tsx` asserts that every card is
  in exactly one area, that the areas cover every card, that each card is
  drawn once, and that each area's cards are drawn in the model's order —
  **those assertions are the test, and they will keep working**. The
  *counts* change, so the test's expectations must change with them.
  `frontend/tests/progress-index.test.tsx` asserts the index's shape.
* **the test that proves it:** `npm --prefix frontend run test:unit`; the
  viewport matrix (`test:viewport`) — the wall is the desktop's widest
  surface.

**Step F4 — Fold Today's three blocks into one *Last night* card, and give
the run detail its one-line sleep link.**

* **files:** `frontend/src/pages/TodayPage.tsx` (lines 585-611),
  `frontend/src/pages/ActivityPage.tsx` (line 747 and the import at line 31),
  `frontend/src/i18n/catalogue.ts` (`today.vitals.heading` is already dead).
* **D7**: the run detail keeps a **one-line link** to the night's sleep —
  the panel moves to Body, the link stays.
* **could break:** `frontend/tests/cockpit.test.tsx`, `frontend/tests/
  states.test.tsx`.
* **the test that proves it:** `npm --prefix frontend run test:unit`.

**Step F5 — Fold Body's *How to read this* into the page's lede.**

* **files:** `frontend/src/pages/BodyPage.tsx`.
* **the test that proves it:** `npm --prefix frontend run test:unit`.

---

#### Phase 4 — the core cleanups

**Step C1 — One movement rule (the two walking thresholds).** ⚠ *a behaviour
change — see §4.1*

* **files:** `crates/stride-core/src/movement.rs` (new),
  `crates/stride-core/src/lib.rs` (the module and the re-exports),
  `crates/stride-core/src/fingerprint.rs` (delete `WALKING_CADENCE_SPM`,
  `WALKING_SPEED_MPS`, `was_walked`; delegate),
  `crates/stride-ingest/src/segment.rs` (delegate `classify_sample` to the
  core), `crates/stride-core/src/update.rs:543` (the third copy).
* **the decision it needs first:** which cadence threshold wins — 60 spm
  (`segment.rs:111`) or 125 spm (`fingerprint.rs:79`)? **They disagree, and
  one of them has to go.** ⚠
* **could break:** **every zone time, every pace band, the film's ribbon,
  the run detail's *not moving* bucket, and every fingerprint observation.**
  `crates/stride-core/tests/fingerprint.rs`, `crates/stride-ingest/tests/
  fit_golden.rs`, `crates/stride-api/tests/zones.rs`,
  `crates/stride-api/tests/fingerprint.rs`, `crates/stride-api/tests/load.rs`.
* **the test that proves it:** `cargo test --workspace`, and the golden
  corpus (`bash scripts/check-corpus.sh` — **the FIT SHA-256s must not
  change; the *answers* may**).

**Step C2 — One zone-model seam in the api.**

* **files:** `crates/stride-api/src/rolling.rs` (`stored_model`),
  `crates/stride-api/src/fingerprint.rs` (`stored_model`),
  `crates/stride-api/src/load.rs` (`zone_model`); a new
  `crates/stride-api/src/zonemodel.rs`.
* **change:** one seam — `model_for(api, date)` — that reads the store's
  model and renders it, used by all three.
* **could break:** `crates/stride-api/tests/zone_model.rs`,
  `crates/stride-api/tests/fingerprint.rs:116`.
* **the test that proves it:** `cargo test -p stride-api`.
* **why it comes before step Z1:** changing the *rule* in three copies is
  three times the work and three times the risk. **This step is also what
  makes D3's stage 2 possible later** (§2.3).

**Step C3 — The decode cache.** (the enabler for the ledger)

* **files:** `crates/stride-api/src/state.rs` (the cache on `Api`),
  `crates/stride-api/src/routes/series.rs` (`decoded`).
* **could break:** memory growth; the `Api` is shared between the desktop's
  HTTP threads and the phone's single process — **the cache needs a lock
  that is not held across a decode** (the workspace denies `unwrap`, so the
  poisoned-lock path is code). The store's `fit_sha256` must be read with
  the blob, ideally in one query.
* **the test that proves it:** a test that calls `/insights/rolling` twice
  and asserts the second is served from the cache (a counter on the cache,
  or a decode counter) — plus `cargo test --workspace` to prove no answer
  changed.

---

#### Phase 5 — **D10**, the observations (storage, not surfaces)

**Step O1 — The schema, version 6.** ⚠ *a schema change — the contract's
file travels with it*

* **files:** `crates/stride-store/src/schema.rs` (`SCHEMA_VERSION` 5 → 6,
  the new `TABLE_STATEMENTS` entry, `V6_INDEX_STATEMENTS`, a new
  `Migration { version: 6 }`), **`docs/contract/schema.sql`** (the new
  table and its index), `crates/stride-store/tests/schema_contract.rs` (the
  table list at line 105-130, and the version assertions).
* **the table:** `activity_observations`, shaped like
  `wellness_observations`: `id, activity_id, source, endpoint, field,
  ordinal, value, text_value, unit, observed_at, retrieved_at`.
* **could break:** `crates/stride-store/tests/schema_contract.rs` compares
  the store's schema to `docs/contract/schema.sql` **statement for
  statement** — a mismatch fails the build, which is the point.
  `crates/stride-store/tests/common/old_schema.rs` builds an older database
  for the migration tests; it needs a version-5 fixture and a
  **version-5 → version-6** migration assertion.
* **the test that proves it:** `cargo test -p stride-store --test
  schema_contract`; the migration test that a version-5 database gains the
  table and keeps its rows.

**Step O2 — The store's repository and the export.**

* **files:** `crates/stride-store/src/repos/observations.rs` (a sibling for
  activities), `crates/stride-store/src/model.rs` (`ActivityObservation`,
  `NewActivityObservation`), `crates/stride-store/src/lib.rs:73-74` (the
  re-exports), `crates/stride-store/src/export.rs` (the new frozen table,
  beside `WELLNESS_OBSERVATIONS` at line 508).
* **could break:** the export/import round trip —
  `crates/stride-store/tests/export_import.rs`. **A backup that silently
  dropped the observations would be a data loss the athlete cannot see.**
* **the test that proves it:** `cargo test -p stride-store --test
  export_import`; `cargo test -p stride-store --test observations`.

**Step O3 — The sync mapping.**

* **files:** a new `crates/stride-api/src/activity_observations.rs` (the
  pure module, copied from `crates/stride-api/src/wellness.rs`'s shape),
  `crates/stride-api/src/routes/app.rs::new_activity` (line 1384),
  `crates/stride-api/tests/sync_delta.rs` (the search-entry fixture).
* **the fields:** aerobic / anaerobic training effect and their messages,
  `trainingEffect`, `hrTimeInZone_1..5`, `directWorkoutRpe`,
  `directWorkoutFeel`, moderate and vigorous minutes, respiration, steps.
* **the exclusions (a whitelist, asserted):** VO₂ max, training stress
  score, activity training load, lactate threshold, intensity factor,
  stress, body battery. **A test asserts none of them is ever written.**
* **could break:** the sync path writes more rows per activity — the
  **sync byte budget** (`crates/stride-api/src/state.rs:220`,
  `max_sync_response_bytes`) and the **time budget**
  (`max_sync_seconds`, line 221) are unchanged, because **no extra request
  is made**; but the **write volume** grows, and the store's per-activity
  write is one transaction (`crates/stride-store/src/repos/activity.rs:242`).
* **the test that proves it:** `cargo test -p stride-api --test sync_delta`;
  the three states (present with a value, present and null, **absent → no
  row**).

---

#### Phase 6 — **D4**, the workout-step decode

**Step K1 — Decode the workout-step name.** ⚠ *a new field, and a decision
about the frozen corpus*

* **files:** `crates/stride-ingest/src/fit.rs` (`RawLap` at line 736, the
  `MESG_LAP` arm at line 846, `FitLap` at line 71, `build_laps` at line
  1395), `crates/stride-api/src/routes/series.rs::splits` (line 88),
  `crates/stride-api/src/fingerprint.rs` (the seam to the core),
  `crates/stride-core/src/fingerprint.rs` (`LapReading` at line 124).
* **the change:** `RawLap` and `FitLap` gain `wkt_step_name: Option<String>`;
  the lap's arm gains one `message.text(N)` call; `LapReading` gains
  `step_name: Option<String>`.
* **the number to verify first:** the parent says **definition 24**. **In
  the FIT profile `wkt_step_name` is 24 on the `workout_step` message
  (global 27) and 28 on the `lap` message (global 19)**, and a step covers a
  lap by **timestamp**. **The repository cannot settle this** — the corpus
  carries no such field (verified below) — so **the implementer must check
  the FIT profile before writing the decoder**, and the decoder should read
  **both** the lap's own field and a `workout_step` message that covers it.
* **could break:** `crates/stride-ingest/tests/fit_golden.rs` — the golden
  JSON files (`fixtures/golden/golden/*.json`) carry an `import` object per
  file, and a new field on `FitLap` **changes that object**. The golden
  files are **frozen** (`scripts/check-corpus.sh` hashes the FIT files; the
  golden JSON is compared by `crates/stride-ingest/tests/fit_golden.rs`).
  **A new optional field that is `None` for every corpus file must not
  change the golden output** — the comparison (`compare`, line 228) walks
  the keys, so a `None` that serialises as `null` **will** add a key. **The
  golden files may need regenerating, or the field must be omitted when
  absent.** *Check this before touching the decoder.*
* **the test that proves it:** a **decoder unit test** that builds the bytes
  of one lap message with the step name and asserts it comes out, plus a
  test that a file without the field decodes with `None`.
* **the corpus:** **verified empty** — `for f in fixtures/fit/*.fit; do
  strings -n 4 "$f" | grep -aiE "interval|recovery|warmup|step|repeat"; done`
  finds nothing in any of the sixteen files. The only readable strings are
  the device names in the synthetic files. **So the test data must be
  made**, and **`fixtures/fit/` stays frozen**: a synthetic file in a **new**
  directory (`fixtures/fit-steps/`) with its own SHA-256 list, or a file
  built inside the test. **This is the one step that needs a decision about
  the frozen corpus.**

---

#### Phase 6b — the pictures (**D11**)

**Step V1 — What the pictures need: check the feed first.** ⚠ *a contract
change **only if** the feed cannot carry it*

* **files:** `frontend/src/pages/progressWall.tsx` (the wall's data),
  `frontend/src/pages/RunsPage.tsx` (the feed the app already calls),
  `crates/stride-api/src/dto.rs` and `crates/stride-api/src/routes/rolling.rs`
  (**only if** the feed cannot carry it), `docs/contract/api-paths.md`
  (**only if**).
* **the work:** three pictures need data the answers do not carry — the
  **ledger's week strip** (the band each week held), the **schedule strip**
  (each run's weekday and hour), and the **Eddington grid** (the days that
  reached E kilometres). §2.9's table says which.
* **the check that comes first:** **can the `/activities` feed carry it?**
  The wall already calls it (for the retired *Most recent activities* card,
  `frontend/src/pages/useWall.ts:71`), and the feed carries each run's date
  and distance. **The schedule strip and the Eddington grid can both be
  drawn from the feed**, because a day reaches E km when the sum of that
  day's runs reaches E km. **The week strip cannot** — the band needs the
  zone time — **so the shape change is one field, not three.**
* **the shape change:** `/insights/rolling` gains a `weeks[]` array — each
  calendar week of the window and the band it held (hard, moderate, low, or
  nothing). **One contract change.**
* **could break:** the rolling answer's shape; the app's `RollingInsight`
  declaration; the fixture set.
* **the test that proves it:** `cargo test -p stride-api --test
  frontend_shapes`; a new test that a window's `weeks[]` says *nothing* for
  a week with no run (absence is a hole in the strip, not a zero).

**Step F6 — Redraw the sentence-only cards.** ⚠ *the visual work — gated on
V1*

* **files:** `frontend/src/pages/progressWall.tsx` (the card bodies),
  `frontend/src/pages/progressPanels.tsx` (the panels),
  `frontend/src/components/` (new drawing components — **the names are the
  design's, not this plan's**), `frontend/src/i18n/catalogue.ts` (the
  captions that go, §2.9's table).
* **the six cards and their drawings:**

  | the card | the drawing |
  |---|---|
  | **This week** | the week strip carries the comparison — this week against the same elapsed days of the previous week |
  | **Where the time went** | **one bar, printed once (D11(d))** — the five zones; the polarised roll-up read off it — **and the ledger's strip: the weeks coloured by the band each week held, so a hole in the hard band is a hole (D11(c))** |
  | **Bests & milestones** | distance against time, the records as marks |
  | **Your shape** | seven bars on a common scale, each labelled, each with its movement — **the radar's honest replacement** |
  | **The schedule** | a 7 × 24 strip of the window's runs, each a mark sized by distance |
  | **The habit** | the Eddington **E × E grid** (the days that reached E km) and the streak's **row of day marks** |

* **the captions that go with them:** the ones that repeat the picture —
  `progress.wall.pace.note` (the card's own title said again), and the
  captions that restate a number the drawing shows. **A caption states the
  period and the reason for an absence, and nothing else** (D11(e)).
* **could break:** `frontend/tests/wall.test.tsx` (the card list and the
  area order), `frontend/tests/progress.test.tsx`,
  `frontend/tests/shape.test.tsx`, `frontend/tests/rolling.test.tsx`;
  **the viewport matrix** — the new drawings are new geometry at six widths,
  and the gate asserts no horizontal scroll, no clipped labels and charts
  legible.
* **the test that proves it:** `npm --prefix frontend run test:unit`;
  `npm --prefix frontend run test:viewport`; `npm run tokens:check` — **every
  new drawing must use the tokens, and the check names any component that
  reached for a raw colour.**
* **the accessibility requirement, which is the point of the captions:** the
  picture is for the eye and the caption is the fallback. **Every drawing
  needs an accessible name and its numbers in text.** **No check enforces
  that today** — `frontend/tests/honesty.test.tsx` holds the app's
  *honesty*, and the charts' labels are checked in the browser by the
  viewport matrix. **This step adds the check**, because **D11's whole claim
  is that the caption is the accessible fallback**, and a fallback that is
  not checked is a fallback that rots.

---

#### Phase 7 — the two behaviour changes, each alone

**Step W1 — The window type.** ⚠ *a behaviour change — see §4.1*

* **files:** `crates/stride-core/src/rolling.rs` (the window type: two ends,
  two constructors, `previous()` = the same span seven days earlier),
  `crates/stride-core/src/profile.rs:226-320` (the facets' denominator),
  `crates/stride-core/src/year.rs:206-212` (the year's window and its
  comparison), `crates/stride-api/src/routes/rolling.rs:70,86,105,115,189`,
  `crates/stride-api/src/routes/year.rs:60,100`,
  `crates/stride-api/src/rolling.rs:166,184,219`,
  `crates/stride-core/src/lib.rs:119-121` (the re-exports).
* **what changes for the reader:** **the Progress totals do not** — they
  already use the calendar weeks. **The profile's volume-a-week and
  consistency do**, because the window's length is no longer a round number
  of days. **The year's *what changed* does**, because its comparison
  moves from "a day earlier" to "a week earlier" (**D2**).
* **could break:** `crates/stride-core/src/rolling.rs:824-866` (the window
  assertions), `crates/stride-core/src/year.rs:602-661` (including
  `window.days == 347` at line 659 and `window.previous().to` at line 661),
  `crates/stride-api/tests/rolling.rs`, `year.rs`, `load_surface.rs`,
  `frontend/tests/rolling.test.tsx`.
* **the test that proves it:** `cargo test -p stride-core`; `cargo test -p
  stride-api --test rolling --test year --test load_surface`; the fixture
  regeneration (§4.4).

**Step W2 — The contract's window prose.** ⚠ *the contract travels with
`PLAN.md`*

* **files:** `docs/contract/api-paths.md` — "The rolling view → The window"
  (the one-day shift becomes the seven-day shift; the calendar-week window
  is named), "The pace page" (unchanged — it already obeys), "The
  training load" (unchanged), and the `/insights/rolling` shape row.
* **could break:** `crates/stride-api/tests/route_parity.rs` reads the
  contract's route list — **the route list is unchanged; only the prose
  moves**, so the test is unaffected.
* **the test that proves it:** `cargo test -p stride-api --test route_parity`.

**Step Z1 — Stage 1 of the zone model: the zone-time answers.** ⚠ *the
second behaviour change — the largest item*

* **files:** `crates/stride-store/src/repos/zones.rs` (a new
  `hr_zone_model_on(date)`), `crates/stride-api/src/zonemodel.rs` (the seam
  from step C2), `crates/stride-api/src/routes/data.rs:695`,
  `crates/stride-api/src/routes/film.rs:53`, `crates/stride-api/src/fingerprint.rs:188`,
  `crates/stride-api/src/rolling.rs:266` (the window's zone split) and its
  answer's shape (`zones.model` → `zones.models`),
  `crates/stride-api/src/year.rs:46` (the year's zone mix),
  `crates/stride-core/src/strings.rs` (the plural source line) **+ regenerate
  the export**, `crates/stride-api/src/dto.rs` (the `zones.models` field),
  `frontend/src/lib/types.ts` (the app's declaration),
  `docs/contract/api-paths.md` (the zones sections) **+ a `PLAN.md` edit**.
* **stage 2 is NOT here** (**D3**): `crates/stride-api/src/load.rs` and
  `crates/stride-core/src/load.rs` keep the most-recent model, and the load's
  assumption line keeps naming one model and one day.
* **could break:** **every zone number in the product**, retroactively.
  `/insights/rolling`'s `zones.model` becomes `zones.models` — **the answer's
  shape changes**, so the app's *Where the time went* card changes and the
  contract changes. The single-run answers keep their shape; only the
  selection changes.
* **the test that proves it:** `cargo test --workspace`, with a **second
  model in the seed** in `crates/stride-api/tests/zone_model.rs` — **that is
  the test that proves the rule, and it cannot exist until the rule
  changes.** The fixture set must be **regenerated** (§4.4).

**Step X1 — `SamplePoint` gains the three fields; the `elevation` array
goes.** ⚠ *a contract change*

* **files:** `crates/stride-api/src/dto.rs` (`SamplePoint`,
  `ElevationPoint`, `ActivityDetail::elevation`, `CompareSeries`),
  `crates/stride-api/src/routes/series.rs` (`elevation()` deleted,
  `samples()` extended), `crates/stride-api/src/routes/data.rs:155-156`,
  `frontend/src/lib/types.ts`, `frontend/src/pages/ActivityPage.tsx`,
  `frontend/src/pages/ComparePage.tsx`, `docs/contract/api-paths.md` (the
  `/activities/{id}` shape row) **+ a `PLAN.md` edit**.
* **recommendation:** give `/compare` its own `CompareSeriesPoint` DTO
  (§2.1) so the blast radius is `/activities/{id}` alone.
* **D8.2 and D8.3 are settled and are not a branch:** the chart keeps the
  athlete's **own bands behind the heart-rate line as well as in the
  table**, and **altitude keeps its own scale, default off**, with its own
  metres label.
* **could break:** the consolidated chart is **new work**, not a cleanup —
  this step changes the *shape*; the chart lands with it or after it.
  `crates/stride-api/tests/frontend_shapes.rs:392` asserts the sample's
  fields; `crates/stride-api/tests/shapes.rs:96` lists them.
* **the test that proves it:** `cargo test -p stride-api --test
  frontend_shapes --test shapes`; `npm --prefix frontend run typecheck`.

---

#### Phase 8 — the ledger (the feature itself — **not** cleanup)

**Step L1 — The ledger's route and core module.** (the feature; listed so
the plan is complete, and marked as *not* cleanup)

* **files:** `crates/stride-core/src/stimulus.rs` (new),
  `crates/stride-core/src/lib.rs`, `crates/stride-api/src/stimulus.rs`
  (new), `crates/stride-api/src/routes/insight.rs` (or a new
  `routes/stimulus.rs`), `crates/stride-api/src/router.rs` (the route),
  `docs/contract/api-paths.md` (the route + the shape) **+ a `PLAN.md`
  edit**, `frontend/src/lib/api.ts`, `frontend/src/lib/types.ts`, the
  panel, and **`android/…/PhoneRoutes.kt`** (the bridge's table, §4.2).
* **depends on:** step S2 (the strings), step C2 (the seam), step C3 (the
  cache), step W1 (the window), step Z1 (the zone model), step K1 (the step
  names), step O1–O3 (the observations, for the cross-check later).
  **D3's sequencing consequence: the ledger ships after stage 1.**
* **the rules it must obey:** **D4** (repeated work = the watch's own steps
  only; the lap-shape heuristic is **deleted**), **D5** (each share names
  its denominator; shares over different denominators are not added).

---

### 3.2 The gates every cleanup step passes

**`bash scripts/gate.sh` is the authoritative list** (`CONTRIBUTING.md`: "If
a gate is added, add it here and it runs everywhere"). It runs, in order:

| # | the gate | what it protects | applies to |
|---|---|---|---|
| 1 | `cargo fmt --check` | the formatting | every Rust step |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | the lints | every Rust step |
| 3 | `cargo test --workspace` | the maths, the golden corpus, the FIT position privacy gate | every Rust step |
| 4 | `bash scripts/check-deps.sh` | **the dependency DAG** and the lint table | every Rust step |
| 5 | `bash scripts/check-corpus.sh` | **the FIT SHA-256s** | every Rust step |
| 6 | `bash scripts/check-contract-greps.sh` | the frozen-contract greps | every Rust step |
| 7 | `bash scripts/scan-secrets.sh` | no credential in the tree | every step |
| 8 | `bash scripts/scan-privacy.sh` (via `scripts/privacy-rules.sh`) | no real coordinates, names, notes, serials | every step |
| 9 | the scanner self-tests | that the scanners still catch their bad fixtures | every step |
| 10 | `bash scripts/scan-history.sh` | every commit on every ref, every added line | every step |
| 11 | `npm --prefix frontend run tokens:check` | the design tokens, one source | every frontend step |
| 12 | `npm --prefix frontend run lint:voice` | **the voice**, the catalogue, the ban list | every frontend step |
| 13 | `npm --prefix frontend run typecheck` | the types | every frontend step |
| 14 | `npm --prefix frontend run test:unit` | the app's unit tests | every frontend step |
| 15 | `npm --prefix frontend run test:viewport` | **the six-width matrix** | every frontend step |
| 16 | `bash scripts/scan-pr-self-test.sh` | that the PR scanner works | every step |
| — | `bash scripts/scan-pr.sh origin/master HEAD` | **the diff scanner — pull requests only** (`.github/workflows/ci.yml:69`) | every PR |

**The lint rules that bite during a cleanup specifically:**

* **`cargo clippy … -D warnings`** with the workspace table
  (`Cargo.toml`, `[workspace.lints.clippy]`): **`unwrap_used` denied,
  `expect_used` denied, `panic` denied, `todo` denied, `unimplemented`
  denied.** A cleanup that adds an early-return or a cache eviction path
  will trip these. Test code opts out with a module-level
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` —
  "the one sanctioned place" (`CONTRIBUTING.md`).
* **`scripts/check-deps.sh`** asserts two things beyond the DAG: **the
  workspace has exactly eight members**, and **no crate may override the
  lint table** (`grep -qE '^\[lints\.(clippy|rust)\]' crates/*/Cargo.toml`).
  It also asserts `stride-testkit` is never a hard dependency.
  **A cleanup that adds a crate fails this gate** — and adding a module to
  an existing crate does not. (Step C1's `crates/stride-core/src/movement.rs`
  is a **module**, not a crate. It must not become one.)
* **`scripts/check-contract-greps.sh`** greps `crates/` for: `pair-code` /
  `pairing` / `qr`; string-built SQL (`(SELECT|INSERT|UPDATE|DELETE)` with
  `{…}` in the literal); **an interpolated identifier**
  (`(SELECT|INSERT|UPDATE|DELETE)[[:space:]].*\{(?-i:[a-z_])[a-zA-Z0-9_]*\}`);
  and **`unsafe` outside the FFI crate** (in `crates/`, `desktop/` and
  `android/` except `android/app/src/main/rust/stride-ffi/`).
  **The new store method in step Z1 writes a parameterised query — the
  interpolation rule is why the `COLUMNS` constant is written in UPPER
  CASE.**
* **`scripts/privacy-rules.sh`** (shared by `scan-privacy.sh` and
  `scan-pr.sh`): the only coordinates allowed are within 1° of the
  synthetic anchor (−35.0, −140.0); a labelled coordinate, an unlabelled
  pair, a degree conversion, a Garmin profile field with a personal value,
  a non-synthetic serial, an email, a phone number, and **a free-text note**
  in `fixtures/`, `crates/*/tests/`, `desktop/tests/`, `*.json`, `*.csv`,
  `*.toml` are all violations. **The app's own source is exempt from the
  note rule** (`frontend/src/*` → `note_is_data=0`).
  **Step K1's synthetic FIT carries a *step name* — "interval", "recovery".
  Those are the watch's own words and not a note, but the rule scans
  `*.json` and `fixtures/` for `note|comment|remark` keys, so the
  synthetic file's field names must not look like a note.**
* **`npm run lint:voice`** reads `frontend/src/i18n/catalogue.ts` **and**
  `crates/stride-core/strings.json`; it fails on a banned phrase, a
  `direction` string outside the closed set, a `pattern` string without a
  sample size, an `option` string that mentions a vital, an inline
  user-facing string in any `.tsx`, and a ban list that has drifted from
  `docs/contract/voice.md`. **`checkInlineText` is the one that a cleanup
  trips**: any string literal that reads as user-facing text in a `.tsx`
  file fails. **So a merged card's heading must come from the catalogue,
  not from a literal.**
* **`npm run tokens:check`** — the design tokens have one source
  (`frontend/src/lib/tokens.ts`), and the check names any component that has
  reached for a raw colour. **The consolidated chart is a new drawing: it
  must use the tokens** — including the altitude band, which has its own
  scale and its own label (**D8.3**). **And step F6 adds six more drawings
  (D11): the week strip, the zone bar, the records' marks, the seven bars,
  the 7 × 24 strip and the E × E grid. Every one of them must use the
  tokens, and every one of them must have an accessible name and its
  numbers in text — the picture is for the eye and the caption is the
  fallback.** **There is no check for that today** —
  `frontend/tests/honesty.test.tsx` holds the app's *honesty* (the missing
  readings, the heading, the version, the Compare selection), and the
  charts' labels are checked in the browser by the viewport matrix
  ("a chart has no size in jsdom, so its labels are not in the document
  there"). **F6 adds the missing check** — every new drawing's numbers
  present in text, beside the drawing — because **D11's whole claim is that
  the caption is the accessible fallback**, and a fallback that is not
  checked is a fallback that rots.
* **`npm run test:viewport`** — the six widths (360×640, 390×844, 430×932,
  768×1024, 1024×768, 1280×800) assert no horizontal scroll, no clipped
  labels, tap targets, **an identical destination set**, charts legible and
  safe areas. **A merged wall card changes the desktop's widest surface, and
  the phone's index changes its narrowest.** Steps F2–F4 are the ones this
  gate catches — **and F6, which adds new geometry to six cards.**
* **`cargo test --workspace`** includes the four that a cleanup must not
  break: `route_parity` (the contract's table is the router's table),
  `frontend_shapes` (the app's declarations are the answers' shapes),
  `fixture_parity` (the app's fixture is the router's answer, value for
  value), and `purity` (the core has no I/O, no clock, no randomness).
  **Step O1 adds a fifth to watch: `schema_contract`** (the store's schema
  is the contract's schema).

**`CONTRIBUTING.md`'s own rules, which are gates in everything but name:**

* "**One slice or one coherent change per pull request. Do not mix a refactor
  with a behaviour change.**" — **steps C1, W1 and Z1 are behaviour changes
  and must be alone.** Steps B1, P1–P2, S1–S2, F1–F5, C2–C3, O1–O3, K1 and
  X1 are refactors or additions and must not be mixed with them.
* "If it changes a number, a string, a route shape or the schema, say so
  explicitly — those are the four frozen contracts." **Steps W1, Z1, X1 and
  O1 touch all four.**
* "**Do not edit an expectation to make a test pass.** If a number is wrong,
  the *code* is wrong — unless the change is a deliberate contract change,
  which needs the plan updated in the same pull request."
* "**Do not delete a fixture** because it is inconvenient."
* "The dependency DAG, the string lint, the privacy scan and the clippy
  lints (no unwrap, no expect, no panic, no todo)" — **all four are named
  above.**

### 3.3 The do-not-touch list

**What must not change during a cleanup.** Each item, and why.

| the thing | where | why |
|---|---|---|
| **The contract's voice rules** | `docs/contract/voice.md`; `crates/stride-core/src/strings.rs:49` (`BANNED_PHRASES`, 23 phrases); `crates/stride-core/src/strings.rs:37` (`DIRECTION_SET`, 3 words); `frontend/src/i18n/catalogue.ts` (`DIRECTION_WORDS`) | the closed sets are frozen; the lint is the product's central design constraint and it is machine-enforced. A cleanup may **add** entries; it may not **relax** a rule. |
| **The schema** — *except the one approved addition in step O1* | `docs/contract/schema.sql`; `crates/stride-store/src/schema.rs` | frozen. **The zone-model change (§2.3) needs no schema change** — `hr_zone_models` already keeps the dated history (`schema.rs:160`) and `hr_zone_models()` already returns it (`repos/zones.rs:93`). **Do not add a column for it.** The **one** approved exception is **D10**'s `activity_observations` table, and it is a version bump with the contract's file travelling beside it. |
| **The sync behaviour** | `crates/stride-api/src/sync.rs`, `crates/stride-store/src/sync.rs`, `crates/stride-store/src/repos/sync.rs`, `crates/stride-api/src/routes/app.rs` | the retry rule and its owner are in `docs/contract/core-contract.md` §7. **Step O3 writes *more rows* per activity; it must not change *when* or *how* a pull happens.** A cleanup that touches sync is not a cleanup. |
| **The FIT retention** | `crates/stride-store/src/schema.rs` (`activities.fit_blob`, `fit_sha256`); `crates/stride-store/src/export.rs:494`; `crates/stride-ingest/src/fit.rs` (`MAX_FIT_INPUT_BYTES`) | the original file is the source of truth, and it is kept. **The decode cache (§2.6) must not become a second copy of the file on disk** — it is in memory, and it is a cache. |
| **The privacy posture** | `scripts/privacy-rules.sh`; `crates/stride-store/src/key.rs`; `crates/stride-api/src/state.rs` (`DatabaseKey`); `android/app/src/main/java/app/stride/running/bridge/KeyGuard.kt` | one credential, in the Keystore, never in the WebView. **The decode cache holds positions in memory longer than the call** — that is a posture statement to make in the PR, not a decision to make silently. **Step K1's synthetic FIT adds a file to the tree** — the privacy scan and the corpus SHA list both apply to it. |
| **The FFI seam's shape** | `android/app/src/main/rust/stride-ffi/src/stride.udl` (six functions, all JSON strings); `crates/stride-api/src/ffi.rs` | the seam is JSON-in/JSON-out. **It does not need to change when the route shapes change** — the phone and the desktop both read the same envelope. |
| **The Android bridge's route table** | `android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt` | **it may only grow** (step B1, and the ledger's route in step L1). It is a copy of a frozen contract, and the file says so: "contracts are not changed by the shell." **A cleanup that removes a route from the router must not silently remove it from the bridge — the contract's rule is that the route table travels with a `PLAN.md` edit, and the bridge's copy travels with it.** |
| **The route table** | `crates/stride-api/src/router.rs` (`ROUTES`, `RAW_ROUTES`); `docs/contract/api-paths.md` | frozen. `crates/stride-api/tests/route_parity.rs` holds the router to the contract, route for route, in both directions. **A cleanup adds no route and removes none** — the ledger's route (step L1) is the only addition, and it is a feature, not a cleanup. |
| **The store's `hr_zone_models` rows** | `crates/stride-store/src/repos/zones.rs:71` `write_hr_zone_model_if_new` | "A read that returns the same model writes nothing; a read that fails writes nothing. Absence is not zero and not 'no model' — the previous model, and its date, stay in place." **Step Z1 reads this history; it must not write to it.** |
| **The empty states and the "no value" reasons** | `crates/stride-core/src/fingerprint.rs:200-240` (`NoValue` → `zones.no_model`, `zones.unusable_model`, …); `crates/stride-core/src/zones.rs` | "Absence is not zero. A cut that removes the reason turns an absent number into a zero, and the zero is a lie." **Every merge in step F3 must carry its empty state with it**, and **D4**'s three states (steps present / no steps / steps but none `interval`) are exactly this rule. |
| **The plan's session on Today, unranked** | `frontend/src/pages/TodayPage.tsx`; `crates/stride-core/src/plan.rs` | the contract's rule: an `option` string may not mention a vital. The plan is the only source of a choice. |
| **The corpus** | `fixtures/fit/`; `scripts/check-corpus.sh` | SHA-256 frozen. **Do not edit an expectation; do not delete a fixture.** A cleanup that changes a number changes the *code*, and the expectation moves with the contract change it belongs to. **Step K1's synthetic file belongs in a *new* directory, not here.** |

---

## 4. Risks and traps

### 4.1 The cleanups that are behaviour changes disguised as tidying

**Three. Two of them the user must be told about; the third is small but
real.**

---

#### ⚠ **W — the window migration** (§2.2, **D1**/**D2**)

**What it looks like:** a tidy-up. "One window definition" reads like a
naming convention.

**What it is:** **the profile's volume-a-week and consistency change**, and
**the year's *what changed* compares a different period than it did**.

**How to tell the user apart:** point at the profile's facets.
`crates/stride-core/src/profile.rs:311` —
`volume_per_week_m: volume_m * DAYS_PER_WEEK / days` — and
`active_share: active_days / days * 100`. The `days` is the window's
length, and **the window's length is no longer a round number of days**. A
twelve-week window through a Wednesday is 79 days, so the volume-a-week is
divided by 79/7, not by 12.

**The tell in the diff:** a change to `crates/stride-core/src/rolling.rs`'s
window type, to `profile::facets`'s denominator, or to
`year::year_window`'s comparison.

**The tell in the tests:** `crates/stride-core/src/profile.rs`'s own tests,
`crates/stride-core/src/year.rs:659` (`window.days == 347`) and
`crates/stride-api/tests/rolling.rs` change their expected numbers.

**What does *not* change:** the Progress totals, the pace page, the load and
Today — they already use the calendar weeks and the seven-day shift. **This
is why the migration is safe to ship: the numbers the athlete reads most
often do not move.**

---

#### ⚠ **Z — the zone model in force on the day of the run** (§2.3, **D3**
stage 1)

**What it looks like:** a tidying of three duplicated readers.

**What it is:** **every zone number in the product changes, retroactively**,
for any athlete who has changed their watch configuration. A run from March
measured against November's model is a different number today. **And the
window answer's shape changes** — one named model becomes a list.

**How to tell the user apart:** the answer's *own source line* changes.
`zones.model.source` — "your zone model · HR_MAX · max heart rate 183 bpm ·
read on 3 Nov 2026" — names **one** model. After stage 1, a 12-week window
measured across two models **cannot name one**, and `zones.models` has to
appear.

**The tell in the diff:** a change to `current_hr_zone_model()`'s callers,
or a new `hr_zone_model_on(date)` in
`crates/stride-store/src/repos/zones.rs`, or `zones.model` → `zones.models`
in `crates/stride-api/src/dto.rs`.

**The tell in the tests:** `crates/stride-api/tests/zone_model.rs` must grow
a **second** model in the seed to prove the rule; that is the giveaway — the
test that proves it cannot exist until the rule changes.

**The tell in the fixtures:** `frontend/src/fixtures/answers.json` changes.
`crates/stride-api/tests/fixture_parity.rs` holds it to the router, so the
fixture is regenerated, not hand-edited.

**Stage 2 is separate on purpose** (**D3**): the load's TRIMP threshold and
maximum heart rate move later, in their own change, with their own
announcement — because they change load history the product has shown since
v1.

---

#### **T — retiring a card that carried a fact nobody else carries**

**What it looks like:** a merge.

**What it is:** three cards carry facts that no other card carries: the
wall's *Activity heatmap* (the distance calendar), the wall's *Most recent
activities* (the recent list), and the wall's *Body, last night* (the last
night's vitals). **Merging them away is fine only if the fact moves.** If
the wall's distance calendar is simply dropped and the year page's calendar
does not carry the distance colouring, **a fact disappears from the product**
and no test fails — because the card *was* the fact.

**How to tell the user apart:** the surface inventory is explicit that "one
calendar survives, coloured by distance" (§1, the wall's `year`/`heatmap`
rows). **If the implementation drops the distance colouring, that is a
behaviour change and it must be said.**

**The tell in the diff:** a calendar that loses a channel, a list that loses
a row, a vitals card that loses a signal.

**The tell in the tests:** `frontend/tests/wall.test.tsx` counts the cards;
if the count falls and no card took the fact, nothing fails.

**A fourth, smaller one, from **D4**:** the ledger's *repeated work* row
is **no value** for every run anyone has recorded so far, because no watch
has written a workout-step name into the corpus. **That is the correct
answer** — but the user should hear once that the fifth row of the ledger
is empty until the hardware cooperates.

---

#### **V — the redraws, and the captions that go with them** (§2.9, **D11**)

**What it looks like:** a visual refresh. Six cards get a drawing.

**What it is:** **a card's text changes.** Every caption that restates what
the picture shows is removed, and every card that is a sentence becomes a
picture. The athlete reads a different screen. **And one answer changes
shape** — `/insights/rolling` gains `weeks[]` — which is a contract change.

**How to tell the user apart:** the captions are the tell. `progress.wall.
pace.note` is "The training week by week, and where the time went by pace."
— **the card's own title, said again** — and it goes. **If the caption
survives, the picture did not earn its place.**

**The tell in the diff:** a `catalogue.ts` entry removed for being a
duplicate of the card's title, and a new field on `RollingInsight`.

**The tell in the tests:** `frontend/tests/wall.test.tsx` (the card bodies),
`frontend/tests/progress.test.tsx` (the captions), and the viewport matrix
— **the new drawings are new geometry at six widths**, and the gate asserts
no horizontal scroll and no clipped labels.

**The trap that is not a behaviour change but is still a trap:** **the
drawings must not invent data.** The strip is drawn from the answer; it is
never computed in the app. **And the schedule strip and the Eddington grid
are drawn from the `/activities` feed** — *if* step V1 confirms the feed
carries it — **not from a new field.** The cheapest route is the feed, and
the plan checks it first.

**One more, from **D11(c)**:** *absence is drawn, not described.* A week
with no run in the hard band is **a hole in the strip**, and the sentence
then names the date. **A hole must not be drawn as a zero-width bar that
reads as "no time", and it must not be drawn as a missing cell that reads
as "not known".** It is a hole: the week's column is simply not coloured.
**That distinction is the whole of the rule, and it is worth one line in
the implementation.**

### 4.2 The cleanups that would break the Android or desktop hosts

**The seam is safe; the *bundle* is not.** This is the whole of it.

`android/app/src/main/rust/stride-ffi/src/stride.udl` exposes six functions
and **every one answers with a JSON string** — the same envelope
(`{"ok":…,"status":…,"body":…}`) the desktop's router answers with. **So a
change to a route's *shape* does not break the FFI seam.** The Rust core
answers the same bytes on both hosts.

**The break is in the bundle.** The phone runs the **same web app** as the
desktop, bundled:

* `android/sync-web.sh` copies `frontend/dist` into
  `android/app/src/main/assets/public` (keeping `stride-bridge.js`, the
  shell's half of the seam). **The phone's WebView loads the bundled app** —
  so a frontend change that is not re-synced is a phone running an older
  app against a newer core.
* `scripts/sync-desktop-bundle.sh` copies `frontend/dist` into
  `desktop/assets`, rewriting the React `displayName` to the placeholder
  `test` so the generated library text stays out of the privacy scanner.
  `desktop/ASSET-BUNDLE.md` says the rule: "**when the frontend changes,
  rebuild it and replace these files in the same change**".

**So: any step that touches `frontend/src/` or the route shapes must
rebuild and re-sync both bundles in the same pull request.** The steps that
touch the frontend: S2, F1–F5, Z1, X1, L1. The steps that change a route
shape (and therefore need the *new* app against the *new* core): **X1 and
L1**.

**The specific hazards.**

| the hazard | where | the trap |
|---|---|---|
| **The bundle is stale** | `desktop/assets/`, `android/app/src/main/assets/public/` | a phone that runs the old app against the new core. `SamplePoint` gaining three fields is additive and safe; **removing `elevation` is not** — the old app reads `data.elevation` (`ActivityPage.tsx:160`) and would draw nothing, silently. |
| **The bridge refuses the route** | `android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt` | step B1. **The ledger's route (`/insights/stimulus`) must be added to the bridge's table in the same pull request**, or the phone gets a 404 for the ledger while the desktop works. |
| **The bridge's key routes** | `PhoneRoutes.decide` | `PERSONAL_KEY`, `PERSONAL_KEY_TEST`, `FIT_IMPORT` and `GARMIN_LOGOUT` are refused on the phone whatever the method is. **A cleanup must not add a route that carries the key to the phone's table.** |
| **The desktop's own paths** | `desktop/src/api.rs:46` (`API_PREFIX = "/api/v1"`), `desktop/src/api.rs:184,198`, `desktop/src/http.rs:315`, `desktop/src/server.rs:140,357,400,525` | the desktop hard-codes a handful of paths for its own behaviour (the key's 200 handling, the logout, the fit-import's body limit, the sync run). **A route that changes shape may need a line in `desktop/src/http.rs`'s body-size table.** |
| **The desktop's SPA fallback** | `desktop/src/spa.rs` | the desktop serves the app's own routes. **Removing `/pace` from the app means the desktop's SPA must not 404 on a saved `/pace` link** — it falls through to `index.html` and the app's `*` route sends it to Today (`frontend/src/app/App.tsx`, `<Route path="*" element={<TodayPage />} />`). **A saved `/pace` link lands on Today with no explanation.** Add a redirect, as `ProgressPanelRoute` already does for the old panel links. |
| **The Android keystore path** | `android/app/src/main/java/app/stride/running/bridge/KeyGuard.kt` | untouched by every step here. |
| **The APK's own checks** | `scripts/check_apk.py`, `android/scripts/check-apk-selftest.sh` | the release workflow runs them; a cleanup that changes the bundle must still pass them. |

**The one that is easy to miss:** the phone's bridge table is **already
eleven routes short** (§1.5, Drift 1). Every step that adds a route to the router
must add it to the bridge in the same pull request — and step B1 fixes the
eleven that are missing *today*.

### 4.3 The generated and frozen things

**Regenerate; never hand-edit.**

| the thing | how it is made | how it is checked | what breaks if you hand-edit it |
|---|---|---|---|
| **`crates/stride-core/strings.json`** | `cargo run -p stride-core --example export_catalogue` (`crates/stride-core/examples/export_catalogue.rs`) | `crates/stride-core/src/strings.rs:1984` — `assert_eq!(exported, include_str!("../strings.json"))`; and `frontend/scripts/lint-voice.mjs` reads it for the ban list | **the build fails.** The test compares the export to the file, byte for byte. |
| **`frontend/src/fixtures/answers.json`** and **`compare-answers.json`** | `STRIDE_FIXTURES_OUT=… STRIDE_COMPARE_OUT=… cargo test -p stride-api --test fixture_parity` → `npm run fixtures:export` (`frontend/scripts/export-fixtures.mjs`) | `crates/stride-api/tests/fixture_parity.rs` — the app's whole fixture set is held to the router **value for value** | **the build fails.** The test seeds a store from the fixture's own seed, calls the real router and compares the answers. |
| **`frontend/src/lib/tokens.ts` → `frontend/src/tokens.css` and the phone's chrome** | `npm run tokens:generate` (`frontend/scripts/gen-tokens.mjs`) | `npm run tokens:check` (`frontend/scripts/check-tokens.mjs`), in the gate | the check names any component that reached for a raw colour. **The consolidated chart is a new drawing: it must use the tokens** — including the altitude band (**D8.3**). |
| **`desktop/assets/`** | `scripts/sync-desktop-bundle.sh` | `desktop/ASSET-BUNDLE.md` — "when the frontend changes, rebuild it and replace these files in the same change" | the released rig and the app are not the same app. |
| **`android/app/src/main/assets/public/`** | `android/sync-web.sh` | the APK's own checks in the release workflow | the phone runs an older app against the newer core. |

**Frozen, and hand-edited on purpose — but checked:**

* **`frontend/src/i18n/catalogue.ts`** — hand-written, but `npm run lint:voice`
  checks every entry's text, and `frontend/tests/catalogue.test.ts` checks
  the export against `docs/contract/voice.md`. **It is checked by CI; it is
  not generated.**
* **`docs/contract/api-paths.md`** — hand-written, and
  `crates/stride-api/tests/route_parity.rs` walks its route list and holds
  the router to it. **A change to it travels with a `PLAN.md` edit in the
  same pull request** (the file's own header).
* **`docs/contract/schema.sql`** — hand-written, and
  `crates/stride-store/tests/schema_contract.rs` compares the store's
  schema to it, **statement for statement**. **Step O1 changes it, and the
  store's schema in the same pull request.**
* **`fixtures/fit/`** — frozen by SHA-256 (`scripts/check-corpus.sh`).
  **Step K1's synthetic file belongs in a new directory.**
* **`fixtures/golden/golden/*.json`** — the captured answers, compared by
  `crates/stride-ingest/tests/fit_golden.rs`. **Step K1's new optional
  field must not change them** — check the comparison before touching the
  decoder (§3.1, step K1).

**And the one that is neither generated nor frozen, but is the most
dangerous:**

* **`android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt`'s
  route table** — a hand-copied list with **no automated check that it
  agrees with the router**. `scripts/gate.sh` does not build the Android
  app; `scripts/check-contract-greps.sh` greps it for `unsafe` and for the
  pairing/QR names, but **nothing compares the two route tables.**
  **That is why it drifted by eleven routes.** The cleanup should add the
  comparison — a test that walks the contract's list and holds the bridge's
  copy to it, the way `route_parity.rs` holds the router. **That is step
  B2, and it is not optional: step B1 fixes today's drift, and step B2
  stops it coming back.**

---

## 5. The decisions, and what each one costs

**All settled. No step in this plan branches on any of them.**

| the decision | what it fixes | the step | the cost |
|---|---|---|---|
| **D1/D2** — one window, one comparison | the rolling route's N-day window and its one-day shift are the odd ones out | **W1**, **W2** | the core's window type gains two ends and two constructors; `profile::facets`'s denominator becomes the window's length; the year's comparison moves to a week. **The Progress totals do not change.** |
| **D3** — the zone model, staged | the zone-time answers measure every run against its own dated model; the load model moves later and separately | **C2**, **Z1** | a new store method; one seam replacing three copies; the window answer's `zones.model` becomes `zones.models`; **every zone number changes, retroactively**; a contract change. |
| **D4** — repeated work = the watch's own steps | the lap-shape heuristic is deleted | **K1** | one field on the lap, carried to the core; **the test data must be made**, because the corpus carries no step name; the ledger's fifth row is *no value* until a structured workout is recorded. |
| **D5** — shares name their denominator | the intensity bands and the longest run are over different denominators | **L1** | two denominators in one answer; two catalogue entries; **the panel must not add across them.** |
| **D6/D9** — temperature dropped | the API has the field, this account does not | — | **priced at zero.** No schema change, no sync change, no ingestion. If it ever populates it is a row in the **D10** table — which is the argument for doing **D10** first. |
| **D7** — the surface cut | 24 cards → 11; 13 panels → 10; the `/pace` page retired; the radar retired; the polarised split merged | **F1**–**F5** | the wall, the panels, the index, the run detail and eighteen + eight + seven catalogue entries. |
| **D8.1** — five destinations | the four-vs-five drift | **P1**, **P2** | **`PLAN.md`'s six places and `README.md`'s one are corrected; `CONTRIBUTING.md`'s rule 1 too.** The cut size is unchanged. **The profile's five cards stay on the profile.** |
| **D8.2** — the bands in the chart and the table | the chart keeps the athlete's own bands behind the heart-rate line | **X1** | the chart draws them; the table keeps the numbers, the *not moving* bucket and the model's source line. |
| **D8.3** — altitude on its own scale, default off | altitude never shares the effort band's scale | **X1** | its own metres label; the default chart is pace + heart rate. |
| **D10** — the search response's discarded fields | ten fields arrive on a response Stride already calls | **O1**, **O2**, **O3** | a schema version bump and a new table; a pure mapping module copied from `wellness.rs`; a new frozen export table; three test states plus an exclusion whitelist. **No extra request.** |
| **D11** — the picture first, the sentence second | six of the eleven surviving cards are sentence-only or tables today; the polarised roll-up is a second drawing of one fact; the captions repeat the pictures | **V1**, **F6** | **six redraws** (This week · Where the time went · Bests & milestones · Your shape · The schedule · The habit); **one bar printed once**; the captions that restate the picture go; **one new field** (`/insights/rolling`'s `weeks[]`) — and the schedule strip and the Eddington grid come from the `/activities` feed the wall already calls, if V1 confirms it. |
| **D12** — the method lens | the lens card is a feature, not a cleanup | — | **no step.** Two debts this plan leaves: **D11 applies to the lens card**, and **D12's rule 4 (the athlete's own cap, a dated fact) is a dated fact beside the zone model's** — so **step C2's dated-model reader should be written to carry a second dated fact without being rewritten.** |

---

*Nothing was changed under `crates/`, `frontend/`, `android/`, `desktop/`,
`PLAN.md`, `docs/` or the contract. This file is the only thing created, and
it is not committed.*

---

## Addendum — the parent, 24 November 2026

Two items found while the B1 bridge fix was landing, for the cleanup to carry:

1. **Android lint is red on pre-existing errors.** `:app:lintDebug` fails on
   four `strings.xml` problems (unused resources, an ellipsis) that are not
   in any file the bridge fix touched. A gate that is already red cannot
   catch a new red, so this belongs at the top of the cleanup: fix the four,
   make `:app:lintDebug` green, and keep it in the gate list.
2. **The generated `colors.xml` breaks `aapt2`.** A `--` inside an XML
   comment is rejected by `aapt2`, so the Android build needs a temporary
   local patch of a generated file to compile. It was restored byte-for-byte
   and `tokens:check` is green, so nothing is broken — but the generator
   should not emit a comment `aapt2` refuses, or every local build needs
   the same patch.

3. **The README's string count is wrong.** `README.md` says "six string types
   in all"; `docs/contract/voice.md` says **seven** — the `instruction` type
   (the run update's prompt) is missing from the README's list. The README
   also still says "four destinations" where the code ships five (P1–P2).
   Both are README corrections, in the documentation step.
