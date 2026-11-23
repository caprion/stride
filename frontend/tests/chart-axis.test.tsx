/**
 * A chart's axis.
 *
 * The labels are the core's own rendering of the number — "148 bpm",
 * "5:24 /km" — so every label carries its unit, and the app never
 * prints a domain number on its own.
 *
 * **The load is the one quantity with no unit.** The core renders it as
 * the number itself, because a load is not measured in anything. Such a
 * tick says so with `data-dimensionless`, so the gate that checks every
 * axis for a unit can tell a bare number the *core* rendered from a bare
 * number the *app* printed. The second is the mistake; the first is the
 * model, and inventing a unit for it would be the app writing the
 * model's wording.
 */

import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { AxisTick, axisTick } from '../src/components/ChartAxis'

describe('a chart’s axis', () => {
  it('labels a tick with the core’s own words', () => {
    const { container } = render(
      <svg>
        <AxisTick
          x={10}
          y={20}
          payload={{ value: 148 }}
          labels={new Map([[148, '148 bpm']])}
        />
      </svg>,
    )

    expect(container.querySelector('text')?.textContent).toBe('148 bpm')
    // A quantity with a unit does not claim to be dimensionless.
    expect(container.querySelector('text')?.getAttribute('data-dimensionless')).toBeNull()
  })

  it('says so when the quantity has no unit', () => {
    const { container } = render(
      <svg>
        <AxisTick
          payload={{ value: 40 }}
          labels={new Map([[40, '40.0']])}
          dimensionless
        />
      </svg>,
    )

    const text = container.querySelector('text')
    // The core's words, unchanged.
    expect(text?.textContent).toBe('40.0')
    // And the tick says that the number is the whole of it.
    expect(text?.getAttribute('data-dimensionless')).toBe('true')
  })

  it('marks the axis the chart asks to be marked', () => {
    const element = axisTick([{ value: 40, text: '40.0' }], { dimensionless: true })

    expect(element.props.dimensionless).toBe(true)
    // And the axis that has a unit is not marked.
    expect(axisTick([{ value: 40, text: '40.0' }]).props.dimensionless).toBe(false)
  })

  it('leaves a value that is not there unlabelled', () => {
    const { container } = render(
      <svg>
        <AxisTick payload={{ value: 999 }} labels={new Map([[148, '148 bpm']])} />
      </svg>,
    )

    expect(container.querySelector('text')?.textContent).toBe('')
  })
})
