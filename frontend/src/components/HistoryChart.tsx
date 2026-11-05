import {
  CartesianGrid,
  Line,
  LineChart,
  ReferenceArea,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { t } from '../i18n'
import { axisTicks } from '../lib/chart'
import { formatDateShort } from '../lib/format'
import type { VitalBand, VitalHistoryPoint } from '../lib/types'
import { axisTick } from './ChartAxis'

export function HistoryChart({
  history,
  band,
  selected,
  onDate,
}: {
  history: VitalHistoryPoint[]
  band: VitalBand | null
  selected: string
  onDate?: (date: string) => void
}) {
  const ticks = axisTicks(
    history.map((point) => ({ value: point.value, text: point.text ?? null })),
  )
  return (
    <div className="mt-3" role="group" aria-label={t('body.history.axis')}>
      <div className="h-[160px] w-full">
        <ResponsiveContainer
          width="100%"
          height="100%"
          initialDimension={{ width: 300, height: 160 }}
        >
          <LineChart
            data={history}
            margin={{ top: 8, right: 12, bottom: 0, left: -12 }}
            onClick={(event) => {
              const date = event?.activeLabel
              if (typeof date === 'string') onDate?.(date)
            }}
          >
            <CartesianGrid stroke="#E4EBF0" vertical={false} />
            <XAxis
              dataKey="date"
              tickFormatter={formatDateShort}
              tick={{ fontSize: 10, fill: '#61788D' }}
              minTickGap={30}
            />
            <YAxis
              ticks={ticks.map((tick) => tick.value)}
              tick={axisTick(ticks)}
              width={76}
              domain={['auto', 'auto']}
              tickLine={false}
              axisLine={false}
            />
            {band ? (
              <ReferenceArea
                y1={band.lower}
                y2={band.upper}
                fill="#D6E6F3"
                fillOpacity={0.45}
                ifOverflow="extendDomain"
              />
            ) : null}
            <ReferenceLine x={selected} stroke="#D99A0B" />
            <Tooltip
              labelFormatter={(date) => formatDateShort(String(date))}
              formatter={(_value, _name, item) =>
                (item.payload as VitalHistoryPoint).text ?? t('body.history.pick')
              }
            />
            <Line
              dataKey="value"
              stroke="#124A7C"
              strokeWidth={2}
              dot={history.length <= 31}
              connectNulls={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}
