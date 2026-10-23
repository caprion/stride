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
