# Frontend contract — Stride

Everything below is **frozen**. Build your page against it; do not edit the shared
files (they belong to the orchestrator and another agent may be reading them at
the same moment).

## Files you may create

Only the page files listed in your task prompt, inside `frontend/src/pages/`.

## Files you may NOT edit

`src/types.ts`, `src/lib/*`, `src/components/*`, `src/App.tsx`, `src/main.tsx`,
`src/index.css`, `tailwind.config.js`, anything under `backend/`.

If you genuinely need a change in a shared file, **do not make it** — put a
`// TODO(shared):` comment in your page describing exactly what is needed.

## Stack

React 18 · TypeScript (strict) · Vite · Tailwind · Recharts · react-leaflet ·
TanStack Query · react-router-dom v6 · lucide-react icons.

```bash
cd /home/sumit/stride/frontend
npm run typecheck   # must pass
npm run build       # must pass
```

## API

All calls go through the typed client:

```ts
import { api } from '../lib/api'
import { useQuery } from '@tanstack/react-query'

const { data, isLoading, error } = useQuery({
  queryKey: ['activities', page],
  queryFn: () => api.activities({ page, per_page: 20 }),
})
```

Backend is live at `http://127.0.0.1:8000/api/v1` (Vite proxies `/api` to it).
You can `curl` it to check a response shape.

| Method | Call | Returns |
|---|---|---|
| GET | `api.activities({page,per_page,sport,from,to,q,sort})` | `{items,total,page,per_page,pages}` |
| GET | `api.activity(id)` | activity detail (laps, streams, hr_zones, geo_points) |
| PATCH | `api.patchActivity(id, {name?,description?})` | updated activity |
| GET | `api.summary()` | totals / this_year / this_month / this_week / personal_records / streaks / sport_breakdown / recent |
| GET | `api.weekly(weeks)` | `{items: Bucket[]}` — per ISO week |
| GET | `api.monthly(months)` | `{items: Bucket[]}` |
| GET | `api.paceTrends(weeks)` | `{items: PaceTrend[]}` |
| GET | `api.hrZones(weeks)` | `{zones,total_time_s,peak_hr,weeks}` |
| GET | `api.personalRecords()` | `{items: PersonalRecord[]}` |
| GET | `api.heatmap(year)` | `{year,days:[{date,distance_m,activity_count}]}` |
| GET | `api.calendar(year,month)` | `{year,month,days:[{date,day,activities,distance_m}],total_distance_m,activity_count}` |
| GET | `api.authStatus()` | Garmin connection + network route |
| GET | `api.syncStatus()` / `api.syncRuns()` | sync state |
| POST | `api.runSync(days?)` | trigger a sync |
| GET | `api.networkStatus()` / `api.setRoute('direct'\|'exit_node')` | network route |

Exact TypeScript interfaces for every response are in `src/types.ts`. Read it first.

## Design language — Strava-grade

| Token | Value | Use |
|---|---|---|
| `brand` | `#fc5200` | primary actions, the active state, key series |
| `ink` | `#121212` | headings, values |
| `ink-soft` `#3c4043` / `ink-muted` `#6d7176` / `ink-faint` `#9aa0a6` | secondary text |
| `surface` `#ffffff` / `surface-soft` `#f7f7f7` / `surface-sunken` `#f0f0f0` | backgrounds |
| `edge` `#e6e6e6` | borders |
| `ramp` | orange → magenta | multi-series chart ramp |

Rules:

- Cards are white, `rounded-card` (10px), `border border-edge`, `shadow-card`.
- Page background is `surface-soft`.
- Numbers are big and confident (`text-[26px] font-semibold`); labels are tiny,
  uppercase and letter-spaced (`text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted`).
- One accent colour per view. Orange is reserved for the single most important thing.
- Dense, not airy. Strava packs information.
- Icons from `lucide-react`, 14–18px, `strokeWidth={2}`.

## Component kit (already built — use it, do not rebuild it)

From `../components/ui`:

`Card`, `CardHeader`, `StatTile`, `Button` (primary/secondary/ghost/danger),
`Badge` (neutral/brand/positive/warn/negative/info), `Skeleton`, `EmptyState`,
`PageHeader`, `Toolbar`, `SegmentedControl`, `Toggle`, `Field`, `Divider`,
`inputClass` (a css string for `<input>`), `cn`.

From `../components/charts`:

`ChartFrame` (card + title + sized body), `VolumeChart`, `PaceTrendChart`,
`StreamChart`, `ZoneBarChart`, `MonthlyBars`.

From `../components/RouteThumb`: `<RouteThumb points={[[lat,lon],...]} className="..." />`
— an SVG route thumbnail. Use it wherever a route preview belongs.

From `../components/ActivityCard`: `<ActivityCard activity={a} />` — the Strava feed
card, already styled. Use it for feed rows rather than rebuilding one.

## Formatting helpers (`../lib/format`)

`formatDistance(m)`, `formatDistanceShort(m)`, `formatDuration(s)`,
`formatDurationLong(s)`, `formatPace(sPerKm)`, `formatPaceWithUnit(sPerKm)`,
`formatElevation(m)`, `formatNumber(v)`, `formatHr(v)`, `formatDate(iso)`,
`formatDateLong(iso)`, `formatDateTime(iso)`, `formatRelative(iso)`,
`formatWeekKey('2026-W43')`, `formatMonthKey('2026-01')`, `sportLabel(sport)`,
`SPORT_COLOR`.

## Data conventions

- `distance_m` is metres, `moving_time_s` is seconds, pace is **seconds per km**.
- `local_date` is `YYYY-MM-DD`; `start_time` is a full ISO timestamp.
- Demo activities carry `source === 'demo'` — label them with a `<Badge tone="info">Demo</Badge>`.
- A `null` metric must render as an em dash, never `0` and never `NaN`.

## Quality bar

- Typecheck and build must pass with **zero** errors.
- Handle loading (skeletons) and empty (EmptyState) states for every list.
- No `any` in exported signatures. No `console.log` left behind.
- Real numbers, real labels — no lorem ipsum, no "Stat 1".
- If a chart has no data for the selected range, say so rather than rendering empty axes.

---

# Phase 2 contract — plan intelligence, physiology, fingerprint

Frozen. Same rules as above: **only create the page files named in your task**.
Everything shared (`src/types.ts`, `src/types_phase2.ts`, `src/lib/*`,
`src/components/*`) is owned by the orchestrator and is already written.

## New TypeScript types

All in **`src/types_phase2.ts`** — read it first. Key shapes:

| Type | Purpose |
|---|---|
| `PlanBlock`, `PlanPhase`, `PlanWeek`, `PlanSession` | The plan hierarchy |
| `PlanResult` | One planned session vs what happened (`verdict`, deltas, `cause`) |
| `PlanSessionWithResult` | A session plus its result plus the matched `activity` |
| `PlanWeekDetail` | A week: `sessions[]`, `planned_km`, `actual_km`, `adherence_pct` |
| `PlanBlockDetail` | A block: `phases[]` each with `weeks[]` |
| `PlanAdherenceRow`, `AdherenceSummary` | Weekly roll-up + `by_verdict` histogram |
| `PlanToday` | `{session, week, readiness, recommendation, rationale, has_plan}` |
| `DailyPhysiology`, `Readiness`, `ReadinessDriver` | The day's physiological state |
| `SleepPoint`, `HrvPoint` | Series with baseline bands |
| `PhysiologySources` | What data is actually available, and why not |
| `Fingerprint`, `FitnessTrend`, `ZoneShare`, `GearItem` | Mechanics + trends + shoes |

## New API calls (already added to `src/lib/api.ts`)

```ts
api.planBlocks()                  // {items: PlanBlock[]}
api.planBlock(id)                 // PlanBlockDetail
api.planCurrent()                 // {block, week}
api.planWeek(date)                // PlanWeekDetail
api.planResults(from?, to?)       // {items: PlanResult[]}
api.planAdherence(from?, to?)     // AdherenceSummary
api.planToday()                   // PlanToday
api.importPlan({format, content, name?})  // ImportPlanResult

api.physiologyDaily(from?, to?)   // {items: DailyPhysiology[]}
api.physiologyLatest()            // {day, readiness}
api.physiologySleep(from?, to?)   // {items: SleepPoint[]}
api.physiologyHrv(from?, to?)     // {items: HrvPoint[], baseline}
api.physiologyReadiness(from?,to?)// {items: Readiness[]}
api.physiologySources()           // PhysiologySources

api.fitnessFingerprint(from?, to?)// {items: Fingerprint[]}
api.fitnessActivity(id)           // Fingerprint & {zones}
api.fitnessTrends(metric?, from?, to?) // FitnessTrend
api.fitnessZones(from?, to?)      // {zones, total_time_s}
api.gearList()                    // {items: GearItem[]}
```

## Semantic rules the UI must respect

1. **`has_plan: false` is not 0 % adherence.** A missing plan is "no plan
   defined" — never render it as total failure.
2. **Verdict colours** — use these consistently across every surface:
   `hit` → positive/green · `missed_pace` → warn/amber · `short` → warn/amber ·
   `missed` → negative/red · `substituted` → info/blue · `bonus` → brand/orange.
3. **`cause_confidence`** is 0–1. Render a high-confidence cause as a plain
   sentence; render a low-confidence one as a question ("might be fatigue?").
4. **`PhysiologySources.available === false`** means the health endpoints are
   unreachable. Show the explanation and the remedy — do not render empty charts
   that look broken.
5. **Baseline vs current.** `HrvPoint.baseline` is the trailing 30-day mean;
   show it as a band, not a line.
6. **`decoupling_pct`** is "lower is better" — invert the colour scale.
   `cadence_cv`, `pace_cv`, `cadence_drift_pct`, `hr_drift_pct` are also
   lower-is-better. `efficiency_first`, `cadence_avg`, `form_score` are
   higher-is-better. `FitnessTrend.better` already tells you which.
7. Never render `null` metrics as `0`. Use an em dash.

## Existing kit you should reuse

`Card`, `CardHeader`, `StatTile`, `Button`, `Badge`, `Skeleton`, `EmptyState`,
`PageHeader`, `Toolbar`, `SegmentedControl`, `Toggle`, `Field`, `Divider`,
`inputClass`, `cn` from `../components/ui`.

`ChartFrame`, `VolumeChart`, `PaceTrendChart`, `StreamChart`, `ZoneBarChart`,
`MonthlyBars` from `../components/charts`.

`RouteThumb`, `ActivityCard` from `../components/RouteThumb` / `../components/ActivityCard`.

New helpers you may want (already in `../lib/format`): `formatDistanceShort`,
`formatDurationLong`, `formatPace`, `formatDate`, `formatDateLong`,
`formatRelative`, `formatWeekKey`, `sportLabel`, `SPORT_COLOR`.

## Quality bar

Same as Phase 1: zero type errors, no `any` in exported signatures, no
`console.log`, loading + empty states everywhere, real copy.
