/**
 * Chart kit — shared Recharts wrappers.
 *
 * Every chart in the app uses these so colours, grid lines, tooltip styling and
 * axis formatting stay consistent across pages.
 */

import type { ReactNode } from 'react'
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { Card, CardHeader, cn } from './ui'
import {
  formatDistanceShort,
  formatDuration,
  formatPace,
  formatWeekKey,
} from '../lib/format'

const AXIS = { fontSize: 11, fill: '#6d7176' }
const GRID = '#ececec'
const RAMP = ['#fc5200', '#fd6f31', '#f58a58', '#e9a08a', '#d977a6', '#c14fb0']

export function ChartFrame({
  title,
  subtitle,
  action,
  height = 240,
  children,
  className,
}: {
  title?: ReactNode
  subtitle?: ReactNode
  action?: ReactNode
  height?: number
  children: ReactNode
  className?: string
}) {
  return (
    <Card className={cn('overflow-hidden', className)}>
      {title ? <CardHeader title={title} subtitle={subtitle} action={action} /> : null}
      <div className="px-2 pb-3" style={{ height }}>
        {children}
      </div>
    </Card>
  )
}

function TooltipBox({
  active,
  payload,
  label,
  formatter,
}: {
  active?: boolean
  payload?: any[]
  label?: any
  formatter?: (value: number, name: string) => string
}) {
  if (!active || !payload?.length) return null
  return (
    <div className="rounded-md border border-edge bg-surface px-3 py-2 text-[12px] shadow-lift">
      {label !== undefined ? (
        <div className="mb-1 font-semibold text-ink">{String(label)}</div>
      ) : null}
      {payload.map((entry, index) => (
        <div key={index} className="flex items-center gap-2 text-ink-soft">
          <span
            className="inline-block h-2 w-2 rounded-full"
            style={{ background: entry.color ?? entry.fill }}
          />
          <span className="capitalize">{entry.name}</span>
          <span className="ml-auto font-semibold text-ink">
            {formatter ? formatter(entry.value, entry.name) : String(entry.value)}
          </span>
        </div>
      ))}
    </div>
  )
}

/* --------------------------------------------------------- weekly volume */

export function VolumeChart({
  data,
  height = 240,
  title = 'Weekly volume',
  subtitle,
}: {
  data: { week_key?: string; label?: string; distance_m: number; activity_count?: number }[]
  height?: number
  title?: string
  subtitle?: ReactNode
}) {
  const rows = data.map((d, index) => ({
    ...d,
    name: d.week_key ? formatWeekKey(d.week_key) : (d.label ?? ''),
    km: d.distance_m / 1000,
    index,
  }))
  const average = rows.length ? rows.reduce((sum, r) => sum + r.km, 0) / rows.length : 0

  return (
    <ChartFrame title={title} subtitle={subtitle} height={height}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={rows} margin={{ top: 8, right: 12, left: 0, bottom: 0 }}>
          <CartesianGrid stroke={GRID} vertical={false} />
          <XAxis
            dataKey="name"
            tick={AXIS}
            tickLine={false}
            axisLine={{ stroke: GRID }}
            interval="preserveStartEnd"
            minTickGap={18}
          />
          <YAxis tick={AXIS} tickLine={false} axisLine={false} width={56} unit=" km" />
          <Tooltip
            cursor={{ fill: 'rgba(252,82,0,0.06)' }}
            content={
              <TooltipBox formatter={(value) => `${Number(value).toFixed(1)} km`} />
            }
          />
          {average > 0 ? (
            <ReferenceLine
              y={average}
              stroke="#9aa0a6"
              strokeDasharray="4 4"
              label={{
                value: `avg ${average.toFixed(0)} km`,
                position: 'insideTopRight',
                fontSize: 10,
                fill: '#6d7176',
              }}
            />
          ) : null}
          <Bar dataKey="km" name="Distance" radius={[3, 3, 0, 0]} maxBarSize={34}>
            {rows.map((_row, index) => (
              <Cell
                key={index}
                fill={index === rows.length - 1 ? '#fc5200' : '#fcb596'}
                fillOpacity={index === rows.length - 1 ? 1 : 0.85}
              />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </ChartFrame>
  )
}

/* ----------------------------------------------------------- pace trends */

export function PaceTrendChart({
  data,
  height = 240,
}: {
  data: {
    week_key: string
    median_pace_s_per_km: number | null
    best_pace_s_per_km: number | null
    long_run_pace_s_per_km: number | null
  }[]
  height?: number
}) {
  const rows = data.map((d) => ({
    name: formatWeekKey(d.week_key),
    median: d.median_pace_s_per_km ? Number((d.median_pace_s_per_km / 60).toFixed(2)) : null,
    best: d.best_pace_s_per_km ? Number((d.best_pace_s_per_km / 60).toFixed(2)) : null,
    longRun: d.long_run_pace_s_per_km
      ? Number((d.long_run_pace_s_per_km / 60).toFixed(2))
      : null,
  }))
  // Y axis is inverted: lower minutes/km (faster) must sit at the top.
  const values = rows.flatMap((r) => [r.median, r.best, r.longRun]).filter((v): v is number => v != null)
  const min = values.length ? Math.min(...values) : 4
  const max = values.length ? Math.max(...values) : 8
  const pad = Math.max(0.4, (max - min) * 0.25)

  return (
    <ChartFrame
      title="Pace trend"
      subtitle="Median pace per week — lower is faster"
      height={height}
    >
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={rows} margin={{ top: 8, right: 12, left: 0, bottom: 0 }}>
          <CartesianGrid stroke={GRID} vertical={false} />
          <XAxis dataKey="name" tick={AXIS} tickLine={false} axisLine={{ stroke: GRID }} interval="preserveStartEnd" minTickGap={18} />
          <YAxis
            tick={AXIS}
            tickLine={false}
            axisLine={false}
            width={56}
            domain={[Math.max(0, min - pad), max + pad]}
            reversed
            tickFormatter={(v) => {
              const total = Math.round(v * 60)
              return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`
            }}
          />
          <Tooltip
            content={
              <TooltipBox formatter={(value) => `${formatPace(Number(value) * 60)} /km`} />
            }
          />
          <Legend wrapperStyle={{ fontSize: 12, paddingTop: 4 }} iconType="plainline" />
          <Line
            type="monotone"
            dataKey="median"
            name="Median"
            stroke="#fc5200"
            strokeWidth={2.5}
            dot={false}
            activeDot={{ r: 4 }}
            connectNulls
          />
          <Line
            type="monotone"
            dataKey="best"
            name="Best"
            stroke="#c14fb0"
            strokeWidth={1.5}
            dot={false}
            activeDot={{ r: 4 }}
            connectNulls
          />
          <Line
            type="monotone"
            dataKey="longRun"
            name="Long run"
            stroke="#2d9cdb"
            strokeWidth={1.5}
            strokeDasharray="4 3"
            dot={false}
            activeDot={{ r: 4 }}
            connectNulls
          />
        </LineChart>
      </ResponsiveContainer>
    </ChartFrame>
  )
}

/* ------------------------------------------------------ activity streams */

export function StreamChart({
  data,
  dataKey = 'value',
  color = '#fc5200',
  unit = '',
  height = 220,
  title,
  subtitle,
  strokeWidth = 2,
  fillOpacity = 0.12,
  referenceLines,
  tickFormatter,
  valueFormatter,
}: {
  data: { x: number; value: number | null }[]
  dataKey?: string
  color?: string
  unit?: string
  height?: number
  title?: ReactNode
  subtitle?: ReactNode
  strokeWidth?: number
  fillOpacity?: number
  referenceLines?: { value: number; label: string; color?: string }[]
  /** Formats the Y-axis tick values, e.g. 6.32 -> "6:19". */
  tickFormatter?: (value: number) => string
  /** Formats values inside the tooltip. */
  valueFormatter?: (value: number) => string
}) {
  const values = data.map((d) => d.value).filter((v): v is number => v != null)
  const min = values.length ? Math.min(...values) : 0
  const max = values.length ? Math.max(...values) : 1

  return (
    <ChartFrame title={title} subtitle={subtitle} height={height}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data} margin={{ top: 8, right: 12, left: 0, bottom: 0 }}>
          <defs>
            <linearGradient id={`grad-${color.replace('#', '')}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={color} stopOpacity={fillOpacity * 3} />
              <stop offset="100%" stopColor={color} stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid stroke={GRID} vertical={false} />
          <XAxis
            dataKey="x"
            tick={AXIS}
            tickLine={false}
            axisLine={{ stroke: GRID }}
            tickFormatter={(v) => formatDuration(v)}
            minTickGap={40}
          />
          <YAxis
            tick={AXIS}
            tickLine={false}
            axisLine={false}
            width={56}
            domain={[min - (max - min) * 0.1 - 1, max + (max - min) * 0.1 + 1]}
            tickFormatter={
              tickFormatter ?? ((v) => `${Math.round(v)}${unit}`)
            }
          />
          <Tooltip
            content={
              <TooltipBox
                formatter={
                  valueFormatter ??
                  ((value) => `${Number(value).toFixed(unit === ' m' ? 0 : 1)}${unit}`)
                }
              />
            }
          />
          {referenceLines?.map((line, index) => (
            <ReferenceLine
              key={index}
              y={line.value}
              stroke={line.color ?? '#9aa0a6'}
              strokeDasharray="4 4"
              label={{ value: line.label, position: 'right', fontSize: 10, fill: '#6d7176' }}
            />
          ))}
          <Area
            type="monotone"
            dataKey={dataKey}
            stroke={color}
            strokeWidth={strokeWidth}
            fill={`url(#grad-${color.replace('#', '')})`}
            connectNulls
            dot={false}
            activeDot={{ r: 4 }}
          />
        </AreaChart>
      </ResponsiveContainer>
    </ChartFrame>
  )
}

/* ------------------------------------------------------------ hr zones */

export function ZoneBarChart({
  zones,
  height = 220,
}: {
  zones: { zone_no: number; name: string; percent: number; low_bpm?: number | null; high_bpm?: number | null }[]
  height?: number
}) {
  const colors = ['#9aa0a6', '#2d9cdb', '#1a9c5a', '#e6a700', '#d93a2b']
  const rows = zones.map((z, index) => ({
    ...z,
    label: `Z${z.zone_no} ${z.name}`,
    color: colors[index % colors.length],
  }))
  return (
    <ChartFrame title="Heart-rate zones" subtitle="Time in each zone" height={height}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={rows} layout="vertical" margin={{ top: 4, right: 16, left: 4, bottom: 0 }}>
          <CartesianGrid stroke={GRID} horizontal={false} />
          <XAxis type="number" tick={AXIS} tickLine={false} axisLine={false} unit="%" width={52} />
          <YAxis type="category" dataKey="label" tick={AXIS} tickLine={false} axisLine={false} width={116} />
          <Tooltip content={<TooltipBox formatter={(v) => `${Number(v).toFixed(1)}%`} />} />
          <Bar dataKey="percent" name="Time" radius={[0, 4, 4, 0]} maxBarSize={22}>
            {rows.map((row, index) => (
              <Cell key={index} fill={row.color} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </ChartFrame>
  )
}

/* --------------------------------------------------------- monthly bars */

export function MonthlyBars({
  data,
  height = 200,
}: {
  data: { month_key?: string; label?: string; distance_m: number }[]
  height?: number
}) {
  const rows = data.map((d) => ({
    label: d.month_key ?? d.label ?? '',
    km: d.distance_m / 1000,
  }))
  return (
    <ChartFrame title="Monthly volume" height={height}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={rows} margin={{ top: 8, right: 12, left: 0, bottom: 0 }}>
          <CartesianGrid stroke={GRID} vertical={false} />
          <XAxis dataKey="label" tick={AXIS} tickLine={false} axisLine={{ stroke: GRID }} />
          <YAxis tick={AXIS} tickLine={false} axisLine={false} width={56} unit=" km" />
          <Tooltip content={<TooltipBox formatter={(v) => `${Number(v).toFixed(1)} km`} />} />
          <Bar dataKey="km" name="Distance" fill="#fc5200" radius={[3, 3, 0, 0]} maxBarSize={38} />
        </BarChart>
      </ResponsiveContainer>
    </ChartFrame>
  )
}

/** Colour ramp used for multi-series charts. */
export const CHART_RAMP = RAMP

export { formatDistanceShort }
