/**
 * The charts of the training load.
 *
 * **The numbers were already there.** The router's answer carries the
 * day-by-day series — the acute load, the chronic load and the form at
 * every day of the window — and the seven days the recovery scenario
 * projects forward. The panel showed the *last* value of each and
 * nothing else, so the shape of the training was on the wire and not on
 * the screen. These three charts draw what the answer already carried.
 *
 * They are the charts, and not the screens:
 *
 * - **the load over the window** — the two weighted series and the form,
 *   on one axis. The form is the chronic load minus the acute one, so it
 *   is the same quantity and belongs on the same scale: a second axis
 *   would let two lines cross and mean nothing;
 * - **the load of each day** — the days' own loads as bars, so the shape
 *   above has a texture under it;
 * - **the projection** — the form the scenario expects, with the zero the
 *   scenario talks about.
 *
 * ## What these charts are not
 *
 * There is no band name, no status and no colour that means a judgement.
 * The zero line is a line: the form is positive above it, and the core's
 * own sentence says when the form is expected to turn. The band of the
 * ratio is drawn nowhere here, because it is a band on a *ratio* and these
 * charts do not draw the ratio.
 *
 * The days keep their places and the scale always holds the zero: both
 * of those rules are in `lib/loadSeries`, and both are tested there.
 */

import {
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { t } from '../i18n'
import type { CatalogueKey } from '../i18n'
import { axisTick, EdgeTick } from '../components/ChartAxis'
import { axisTicks } from '../lib/chart'
import {
  domainWithZero,
  loadDayBars,
  loadProjectionPoints,
  loadSeriesPoints,
} from '../lib/loadSeries'
import type { LoadInsight } from '../lib/types'

/**
 * The colours of the three lines.
 *
 * **Two loads and their difference.** The chronic load and the acute load
 * are the app's own blue and amber — the two colours the other charts
 * draw their two series in — and the form is the quiet grey, because it
 * is the difference between the other two and not a third thing.
 */
const LINE_COLOURS: Record<'acute' | 'chronic' | 'form', string> = {
  chronic: 'var(--chart-line)',
  acute: 'var(--chart-reference)',
  form: 'var(--ink-muted)',
}

/** The order the legend lists the three lines in. */
const LINE_ORDER: ('chronic' | 'acute' | 'form')[] = ['chronic', 'acute', 'form']

/** The labels of the three lines, in the app's own words. */
const LINE_LABELS: Record<'acute' | 'chronic' | 'form', CatalogueKey> = {
  chronic: 'progress.load.chronic',
  acute: 'progress.load.acute',
  form: 'progress.load.form',
}

/**
 * Which of the three texts a tooltip shows.
 *
 * The tooltip shows the core's own words for the number, so a value is
 * never printed by the app: "40.0", "-11.7".
 */
const TOOLTIP_TEXT: Record<string, 'acuteText' | 'chronicText' | 'formText'> = {
  acute: 'acuteText',
  chronic: 'chronicText',
  form: 'formText',
}

/** The legend of the three lines. */
function LineLegend() {
  return (
    <div className="mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1">
      {LINE_ORDER.map((key) => (
        <span
          key={key}
          className="inline-flex items-center gap-1 text-[11.5px] text-ink-muted"
          data-testid={`progress-load-legend-${key}`}
        >
          <i
            className="inline-block h-2.5 w-2.5 rounded-[3px]"
            style={{ background: LINE_COLOURS[key] }}
          />
          {t(LINE_LABELS[key])}
        </span>
      ))}
    </div>
  )
}

/**
 * The load over the window: the two weighted series and the form, day by
 * day.
 *
 * The window is the one the screen is showing, and the series is the
 * window's own: the router warms it with the days before the window and
 * does not show them, so the first day on the chart is already a day of
 * a warmed series and not the first day of one that began there.
 */
export function LoadSeriesChart({ load }: { load: LoadInsight }) {
  const series = load.load
  const states = series?.states ?? []
  // A line needs two points to be a line. One point is a value, and the
  // panel shows it as a value.
  if (!series || states.length < 2) return null

  const data = loadSeriesPoints(states)
  const ticks = axisTicks(
    [
      ...data.map((point) => ({ value: point.acute, text: point.acuteText })),
      ...data.map((point) => ({ value: point.chronic, text: point.chronicText })),
      ...data.map((point) => ({ value: point.form, text: point.formText })),
    ],
    4,
  )

  return (
    <figure className="m-0" data-testid="progress-load-series">
      <figcaption className="mb-1 caps-sm">{t('progress.load.series.heading')}</figcaption>
      <div style={{ width: '100%', height: 170 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: -18 }}>
            <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
            <XAxis
              dataKey="label"
              interval={0}
              tickLine={false}
              tick={EdgeTick}
            />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks, { dimensionless: true })}
              width={64}
              tickLine={false}
              axisLine={false}
              domain={domainWithZero([
                ...data.map((point) => point.acute),
                ...data.map((point) => point.chronic),
                ...data.map((point) => point.form),
              ])}
            />
            <Tooltip
              formatter={(_value, _name, item) => {
                const key = TOOLTIP_TEXT[String(item?.dataKey ?? '')]
                const payload = item?.payload as Record<string, string> | undefined
                return (key && payload?.[key]) || ''
              }}
            />
            {/* The form's zero: the line the scenario's sentence talks
                about. It is a line, not a band and not a status. */}
            <ReferenceLine y={0} stroke="var(--chart-axis)" strokeDasharray="4 3" />
            {LINE_ORDER.map((key) => (
              <Line
                key={key}
                type="monotone"
                name={t(LINE_LABELS[key])}
                dataKey={key}
                stroke={LINE_COLOURS[key]}
                strokeWidth={2}
                dot={false}
                isAnimationActive={false}
              />
            ))}
          </LineChart>
        </ResponsiveContainer>
      </div>
      <LineLegend />
      <p className="mt-1 text-[12px] text-ink-faint" data-testid="progress-load-series-note">
        {t('progress.load.series.note')}
      </p>
    </figure>
  )
}

/**
 * The load of each day of the window.
 *
 * The bars are laid on the calendar, so the days the record does not
 * cover are gaps and not missing days. A day with no load has no bar,
 * and a day whose load is zero has no bar either — the caption says which
 * is which.
 */
export function LoadDaysChart({ load }: { load: LoadInsight }) {
  const series = load.load
  if (!series) return null

  const data = loadDayBars(series.days, load.window.from, load.window.to)
  if (data.length === 0) return null

  const ticks = axisTicks(
    data.map((point) => ({ value: point.load, text: point.loadText })),
    3,
  )

  return (
    <figure className="m-0" data-testid="progress-load-days">
      <figcaption className="mb-1 caps-sm">{t('progress.load.days.heading')}</figcaption>
      <div style={{ width: '100%', height: 130 }}>
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: -18 }}>
            <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
            <XAxis
              dataKey="label"
              interval={0}
              tickLine={false}
              tick={EdgeTick}
            />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks, { dimensionless: true })}
              width={64}
              tickLine={false}
              axisLine={false}
              domain={[0, 'auto']}
            />
            <Tooltip
              formatter={(_value, _name, item) => {
                const payload = item?.payload as { loadText?: string | null } | undefined
                return payload?.loadText ?? ''
              }}
            />
            <Bar
              name={t('progress.load.heading')}
              dataKey="load"
              fill="var(--chart-line)"
              isAnimationActive={false}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>
      <p className="mt-1 text-[12px] text-ink-faint" data-testid="progress-load-days-note">
        {t('progress.load.days.note')}
      </p>
    </figure>
  )
}

/**
 * The form the scenario projects forward.
 *
 * **A projection, not a promise.** It is what the model expects with no
 * training at all, and the assumption it rests on is already on the
 * panel: no training for seven days, the coefficient, and where the
 * maximum heart rate came from.
 *
 * The chart is the form only. The ratio is a different quantity and the
 * band is a band on the ratio; neither is drawn here, and the scenario's
 * own sentence says what the ratio is expected to do.
 */
export function LoadProjectionChart({ load }: { load: LoadInsight }) {
  const projection = load.scenario?.projection ?? []
  if (projection.length === 0) return null

  const data = loadProjectionPoints(projection)
  const ticks = axisTicks(
    data.map((point) => ({ value: point.form, text: point.formText })),
    3,
  )

  return (
    <figure className="m-0" data-testid="progress-load-projection">
      <figcaption className="mb-1 caps-sm">{t('progress.load.projection.heading')}</figcaption>
      <div style={{ width: '100%', height: 120 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: -18 }}>
            <CartesianGrid stroke="var(--chart-grid)" vertical={false} />
            <XAxis
              dataKey="label"
              interval={0}
              tickLine={false}
              tick={EdgeTick}
            />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks, { dimensionless: true })}
              width={64}
              tickLine={false}
              axisLine={false}
              domain={domainWithZero(data.map((point) => point.form))}
            />
            <Tooltip
              formatter={(_value, _name, item) => {
                const payload = item?.payload as { formText?: string | null } | undefined
                return payload?.formText ?? ''
              }}
            />
            <ReferenceLine y={0} stroke="var(--chart-axis)" strokeDasharray="4 3" />
            <Line
              type="monotone"
              name={t('progress.load.form')}
              dataKey="form"
              stroke={LINE_COLOURS.form}
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
      <p className="mt-1 text-[12px] text-ink-faint" data-testid="progress-load-projection-note">
        {t('progress.load.projection.note', { n: projection.length })}
      </p>
    </figure>
  )
}
