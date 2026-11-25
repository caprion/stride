/**
 * The threshold: **the four candidates, the choice, the dated history**.
 *
 * The app computes the candidates; the athlete chooses one. The rules
 * the test holds to:
 *
 *   - each candidate shows **the value, the source and the date**;
 *   - **the note that says what a candidate cannot know is not on the
 *     card** — that line is the explanation, and the explanation is not
 *     product content;
 *   - the choice in force is named, and the choices that came before it
 *     stay in the dated history;
 *   - a candidate with no value says why, and shows no number.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { thresholdFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The threshold screen, in the training section of the settings. */
async function thresholdScreen() {
  renderApp('/settings?section=training')
  return screen.findByTestId('threshold-card')
}

describe('the threshold card', () => {
  it('shows the four candidates, each with its value and its date', async () => {
    await thresholdScreen()

    // The watch's own zone model, the race, the field test and the
    // number the athlete brought.
    await waitFor(() =>
      expect(screen.getByTestId('threshold-candidates')).toBeTruthy(),
    )
    expect(screen.getByTestId('threshold-value-watch_zone_model')?.textContent).toContain('163 bpm')
    expect(screen.getByTestId('threshold-value-race')?.textContent).toContain('/km')
    // The field test has no run to read: it says why, and shows no
    // number.
    expect(screen.getByTestId('threshold-value-field_test')?.textContent).toContain(
      'no run chosen',
    )
    // And the dates are the facts the values came from.
    expect(screen.getByTestId('threshold-date-watch_zone_model')?.textContent).toContain('2026')
  })

  it('keeps the note that says what a candidate cannot know off the card', async () => {
    await thresholdScreen()

    await waitFor(() =>
      expect(screen.getByTestId('threshold-candidates')).toBeTruthy(),
    )
    // The limit on a candidate is the explanation: it is not on the card.
    expect(screen.queryByText(/the app cannot check what was put into the watch/)).toBeNull()
    expect(screen.queryByText(/a race is a race, not a test/)).toBeNull()
  })

  it('names the choice in force, and the choices that came before it', async () => {
    await thresholdScreen()

    const choice = await screen.findByTestId('threshold-choice')
    expect(choice.textContent).toContain('168 bpm')

    // The dated history: the choice, and the day it was made.
    const history = await screen.findByTestId('threshold-history')
    expect(history.textContent).toContain('11 May 2026')
  })

  it('writes the athlete’s choice', async () => {
    const choose = vi.spyOn(api, 'chooseThreshold').mockResolvedValue({
      choice: {
        source: 'watch_zone_model',
        label: "your watch's zones",
        value: '163 bpm',
        heart_rate_bpm: 163,
        pace: null,
        base: 'pfitzinger',
        typed_label: null,
        origin_date: '2026-05-10',
        entered_on: '2026-05-13',
        updated_on: '2026-05-13',
        in_force: true,
      },
      generated_at: '2026-05-13T06:30:00Z',
    })
    await thresholdScreen()

    const button = await screen.findByTestId('threshold-choose-watch_zone_model')
    await userEvent.click(button)

    await waitFor(() => expect(choose).toHaveBeenCalled())
    // The choice names the candidate and the base it was made for.
    expect(choose.mock.calls[0]?.[0]).toMatchObject({
      source: 'watch_zone_model',
      base: 'pfitzinger',
    })
  })

  it('takes a number the athlete brought', async () => {
    const choose = vi.spyOn(api, 'chooseThreshold').mockResolvedValue({
      choice: {
        source: 'typed',
        label: 'entered by you',
        value: '170 bpm',
        heart_rate_bpm: 170,
        pace: null,
        base: 'pfitzinger',
        typed_label: 'my own number',
        origin_date: null,
        entered_on: '2026-05-13',
        updated_on: '2026-05-13',
        in_force: true,
      },
      generated_at: '2026-05-13T06:30:00Z',
    })
    await thresholdScreen()

    await userEvent.type(await screen.findByTestId('threshold-typed-number'), '170')
    await userEvent.click(screen.getByTestId('threshold-choose-typed'))

    await waitFor(() => expect(choose).toHaveBeenCalled())
    expect(choose.mock.calls[0]?.[0]).toMatchObject({
      source: 'typed',
      heart_rate_bpm: 170,
      label: 'my own number',
    })
  })

  it('says when the answer has no value at all', async () => {
    vi.spyOn(api, 'insightsThreshold').mockResolvedValue({
      ...thresholdFixture,
      candidates: [],
      in_force: null,
      history: [],
      unavailable: 'no zone model has been read',
    })
    renderApp('/settings?section=training')

    await screen.findByTestId('threshold-card')
    expect(screen.queryByTestId('threshold-candidates')).toBeNull()
  })
})
