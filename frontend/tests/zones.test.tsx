/**
 * The zone model on the run detail.
 *
 * The bands are the athlete's own: they are measured against the model the
 * watch was configured with, and the screen says **which** model and **the
 * day it was read**. A number that cannot say where its input came from is
 * not inspectable, and the bands are the most interpretable intensity
 * statement the product makes.
 *
 * The line is the core's sentence and the date is the api's own rendering,
 * so the assertions below are on the words as they arrive — the app writes
 * none of them.
 */

import { waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The source line under the bands, when there is one. */
async function sourceLine(route = '/runs/101') {
  const { container } = renderApp(route)
  await waitFor(() =>
    expect(container.querySelector('[data-testid="activity-effort"]')).toBeTruthy(),
  )
  return {
    container,
    line: container.querySelector('[data-testid="activity-zones-source"]')?.textContent ?? '',
    unavailable:
      container.querySelector('[data-testid="activity-zones-unavailable"]')?.textContent ?? '',
  }
}

describe('the zone model on the run detail', () => {
  it('names the model and the day it was read', async () => {
    const { line } = await sourceLine()

    // The method the watch was set with, the maximum heart rate the model
    // was built on, and the day it was read.
    expect(line).toContain('HR_MAX')
    expect(line).toContain('178 bpm')
    expect(line).toContain('10 May 2026')
    // The date is the api's own rendering — "10 May 2026" — and not the
    // core's canonical form, which stays inside the core.
    expect(line).not.toContain('2026-05-10')
  })

  it('says there is no value when there is no model', async () => {
    // A store with no zone model: the run is not calibrated.
    window.history.pushState({}, '', '/runs/101?fixtures=uncalibrated')
    const { line, unavailable } = await sourceLine('/runs/101')

    // No model, so no source line: there is nothing to name.
    expect(line).toBe('')
    // And the honest reason, in the core's own words.
    expect(unavailable).toContain("No zones yet — your watch's heart-rate zones have not come through")
    window.history.pushState({}, '', '/?fixtures=1')
  })
})
