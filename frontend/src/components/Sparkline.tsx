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
  values: (number | null)[]
  band?: { lower: number; upper: number } | null
  height?: number
  label?: string
}) {
  const width = 120
  const points = values.filter(
    (value): value is number => value !== null && Number.isFinite(value),
  )
  if (points.length === 0) {
    return <div style={{ height }} aria-hidden="true" />
  }

  const lows = band ? [band.lower, band.upper, ...points] : points
  const min = Math.min(...lows)
  const max = Math.max(...lows)
  const span = max - min || 1
  const x = (index: number) => (index / Math.max(1, values.length - 1)) * width
  const y = (value: number) => height - ((value - min) / span) * (height - 4) - 2

  let segment: { x: number; y: number }[] = []
  const segments = [segment]
  values.forEach((value, index) => {
    if (value === null || !Number.isFinite(value)) {
      if (segment.length > 0) {
        segment = []
        segments.push(segment)
      }
    } else {
      segment.push({ x: x(index), y: y(value) })
    }
  })

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
      {segments.map((segment, index) => {
        const last = segment[segment.length - 1]
        if (!last) return null
        return (
          <g key={index}>
            <polyline
              points={segment.map((point) => `${point.x},${point.y}`).join(' ')}
              fill="none"
              stroke="#1E5E96"
              strokeWidth="1.5"
            />
            <circle cx={last.x} cy={last.y} r="2" fill="#1E5E96" />
          </g>
        )
      })}
    </svg>
  )
}
