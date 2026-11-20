/**
 * The Form card on the run detail.
 *
 * The run's own mechanics, as **observations**: the values, and the
 * direction each one moved. There is no score and no composite — the
 * numbers are what the run measured, and the reader decides what they
 * mean.
 */

import { screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { fingerprintFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

describe('the Form card on the run detail', () => {
  it('shows the observations and the direction each one moved', async () => {
    vi.spyOn(api, 'fitnessFingerprint').mockResolvedValue(fingerprintFixture)
    renderApp('/runs/101')
    const card = await screen.findByTestId('activity-form')

    // The values are the core's own words, with their units.
    expect(card.textContent).toContain('163.0 spm')
    expect(card.textContent).toContain('Aerobic decoupling')
    expect(card.textContent).toContain('3.9 %')
    // The sign of the difference is the direction: the cadence drifted
    // down and the heart rate drifted up.
    expect(card.textContent).toContain('−1.4 %')
    expect(card.textContent).toContain('+5 bpm')
  })

  it('carries no score and no verdict', async () => {
    vi.spyOn(api, 'fitnessFingerprint').mockResolvedValue(fingerprintFixture)
    renderApp('/runs/101')
    const card = await screen.findByTestId('activity-form')

    // No score, no grade, no traffic light: the card reports the run's
    // own measurements and nothing else.
    expect(card.textContent?.toLowerCase()).not.toContain('score')
    expect(card.textContent?.toLowerCase()).not.toContain('form score')
    expect(card.textContent?.toLowerCase()).not.toContain('excellent')
    expect(card.textContent?.toLowerCase()).not.toContain('poor')
  })

  it('says why a metric has no value, in the core’s words', async () => {
    vi.spyOn(api, 'fitnessFingerprint').mockResolvedValue(fingerprintFixture)
    renderApp('/runs/101')
    const card = await screen.findByTestId('activity-form')

    // The stride length is not in the file: the row says so instead of
    // showing a zero.
    expect(card.textContent).toContain("the run's file carries no stride length")
  })

  it('shows the reason when the run has nothing to measure', async () => {
    // The seeded run holds no file, so the router answers the honest one:
    // there is nothing to measure, and the core says why.
    renderApp('/runs/101')
    const card = await screen.findByTestId('activity-form')

    expect(card.querySelector('[data-testid="activity-form-unavailable"]')?.textContent).toBe(
      'the run has no file to read',
    )
    // And no numbers at all.
    expect(card.querySelector('[data-testid="activity-form-observations"]')).toBeNull()
  })
})
