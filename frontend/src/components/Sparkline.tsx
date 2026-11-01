/**
 * A sparkline.
 *
 * The band the direction is measured against is drawn on the chart, so the
 * athlete can see where the line is — and disagree with it.
 */

export function Sparkline({
  values,
  band,
  height = 34,
  label,
}: {
  values: number[]
  band?: { lower: number; upper: number } | null
  height?: number
  label?: string
}) {
  const width = 120
  const points = values.filter((value) => Number.isFinite(value))
  if (points.length < 2) {
    return <div style={{ height }} aria-hidden="true" />
  }

  const lows = band ? [band.lower, band.upper, ...points] : points
  const min = Math.min(...lows)
  const max = Math.max(...lows)
  const span = max - min || 1
  const x = (index: number) => (index / (points.length - 1)) * width
  const y = (value: number) => height - ((value - min) / span) * (height - 4) - 2

  const line = points
    .map((value, index) => `${x(index).toFixed(2)},${y(value).toFixed(2)}`)
    .join(' ')
  const lastX = x(points.length - 1)
  const lastY = y(points[points.length - 1] as number)

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      className="w-full"
      style={{ height }}
      role="img"
      aria-label={label ?? 'the last nights'}
    >
      {band ? (
        <rect
          x="0"
          y={y(band.upper)}
          width={width}
          height={Math.max(1, y(band.lower) - y(band.upper))}
          fill="#E4EBF0"
        />
      ) : null}
      <polyline points={line} fill="none" stroke="#1E5E96" strokeWidth="1.5" />
      <circle cx={lastX} cy={lastY} r="2" fill="#1E5E96" />
    </svg>
  )
}
