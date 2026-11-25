/**
 * The athlete's own heart rates.
 *
 * **One small fact, entered once, and the app takes care of it.** The
 * rules the test holds to:
 *
 *   - the declaration in force shows **the value, the source and the
 *     dates**;
 *   - the **bands** and the **watch's own floors** are shown side by side,
 *     with the **difference** stated — a fact, and never a correction;
 *   - a declaration with no bands says **why**, in the core's own words;
 *   - the app **writes** what the athlete typed, and never invents a
 *     number.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import type { HeartRatesInsight } from '../src/lib/types'
import { renderApp } from './helpers'

afterEach(() => vi.restoreAllMocks())

/** The declaration in force, with the bands and the watch's own floors. */
const declared: HeartRatesInsight = {
  generated_at: '2026-11-24T06:30:00Z',
  declaration: {
    id: 1,
    max: {
      label: 'your maximum heart rate',
      value_bpm: 186,
      value: { text: '186 bpm', raw: 186 },
      source: 'test',
      source_label: 'from a test',
      measured_on: '2026-11-08',
      measured_on_label: '8 Nov 2026',
    },
    resting: {
      label: 'your resting heart rate',
      value_bpm: 52,
      value: { text: '52 bpm', raw: 52 },
      source: 'doctor_lab',
      source_label: 'from a doctor or a lab',
      measured_on: '2026-11-12',
      measured_on_label: '12 Nov 2026',
    },
    entered_on: '2026-11-24',
    entered_on_label: '24 Nov 2026',
    entered: 'entered on 24 Nov 2026',
    updated_on: '2026-11-24',
    in_force: true,
  },
  bands: {
    label: 'the bands from the heart rates you entered',
    floors: {
      zone1: { text: '119 bpm', raw: 119 },
      zone2: { text: '132 bpm', raw: 132 },
      zone3: { text: '146 bpm', raw: 146 },
      zone4: { text: '159 bpm', raw: 159 },
      zone5: { text: '173 bpm', raw: 173 },
    },
    max_heart_rate: { text: '186 bpm', raw: 186 },
    range: 'reserve',
    range_label: 'from your maximum heart rate and your resting heart rate',
    entered_on: '2026-11-24',
    source:
      'your zones · the bands from the heart rates you entered · maximum heart rate 186 bpm · resting heart rate 52 bpm · entered on 24 Nov 2026',
  },
  watch: {
    label: "your watch's zones",
    floors: {
      zone1: { text: '115 bpm', raw: 115 },
      zone2: { text: '128 bpm', raw: 128 },
      zone3: { text: '143 bpm', raw: 143 },
      zone4: { text: '159 bpm', raw: 159 },
      zone5: { text: '165 bpm', raw: 165 },
    },
    max_heart_rate: { text: '183 bpm', raw: 183 },
    read_on: '2026-11-03',
    read_on_label: '3 Nov 2026',
    source:
      "your zones · HR_MAX · max heart rate 183 bpm · taken from your watch on 3 Nov 2026",
  },
  difference: [
    {
      zone: 1,
      declared_bpm: 119,
      watch_bpm: 115,
      difference_bpm: 4,
      difference: { text: '+4 bpm', raw: 4 },
    },
    {
      zone: 2,
      declared_bpm: 132,
      watch_bpm: 128,
      difference_bpm: 4,
      difference: { text: '+4 bpm', raw: 4 },
    },
    {
      zone: 3,
      declared_bpm: 146,
      watch_bpm: 143,
      difference_bpm: 3,
      difference: { text: '+3 bpm', raw: 3 },
    },
    {
      zone: 4,
      declared_bpm: 159,
      watch_bpm: 159,
      difference_bpm: 0,
      difference: { text: '0 bpm', raw: 0 },
    },
    {
      zone: 5,
      declared_bpm: 173,
      watch_bpm: 165,
      difference_bpm: 8,
      difference: { text: '+8 bpm', raw: 8 },
    },
  ],
  history: [],
  sources: [
    { key: 'watch', label: 'from my watch' },
    { key: 'test', label: 'from a test' },
    { key: 'doctor_lab', label: 'from a doctor or a lab' },
    { key: 'partner_watch', label: "from my training partner's watch" },
    { key: 'typed', label: 'typed' },
  ],
  kinds: [
    { key: 'max', label: 'your maximum heart rate' },
    { key: 'resting', label: 'your resting heart rate' },
  ],
  unavailable: null,
}

/** The same answer, with no declaration in force. */
const empty: HeartRatesInsight = {
  ...declared,
  declaration: null,
  bands: null,
  difference: null,
  history: [],
  unavailable: 'no heart rates entered',
}

/** The card, in the training section of the settings. */
async function heartRatesScreen(answer: HeartRatesInsight = declared) {
  vi.spyOn(api, 'athleteHeartRates').mockResolvedValue(answer)
  renderApp('/settings?section=training')
  return screen.findByTestId('heart-rates-card')
}

describe('the heart rates card', () => {
  it('shows the declaration in force, each value with its source and its dates', async () => {
    await heartRatesScreen()

    const max = await screen.findByTestId('heart-rates-max')
    expect(max.textContent).toContain('your maximum heart rate')
    expect(max.textContent).toContain('186 bpm')
    expect(max.textContent).toContain('from a test')
    expect(max.textContent).toContain('8 Nov 2026')

    const resting = await screen.findByTestId('heart-rates-resting')
    expect(resting.textContent).toContain('your resting heart rate')
    expect(resting.textContent).toContain('52 bpm')
    expect(resting.textContent).toContain('from a doctor or a lab')

    // And the day the declaration was made, in the core's own words.
    expect((await screen.findByTestId('heart-rates-entered')).textContent).toContain(
      'entered on 24 Nov 2026',
    )
  })

  it('shows the bands and the watch’s own floors, with the difference', async () => {
    await heartRatesScreen()

    // The bands: the declared ones, and the range they were derived on.
    const bands = await screen.findByTestId('heart-rates-bands')
    expect(bands.textContent).toContain('the bands from the heart rates you entered')
    expect(bands.textContent).toContain('from your maximum heart rate and your resting heart rate')
    // The watch's own floors, beside them.
    expect(bands.textContent).toContain("your watch's zones")
    // And the difference, boundary by boundary: a fact, not a correction.
    const first = await screen.findByTestId('heart-rates-band-1')
    expect(first.textContent).toContain('119 bpm')
    expect(first.textContent).toContain('115 bpm')
    expect(first.textContent).toContain('+4 bpm')
  })

  it('says why there are no bands when there are none', async () => {
    await heartRatesScreen(empty)

    const none = await screen.findByTestId('heart-rates-none')
    expect(none.textContent).toContain('no heart rates entered')
  })

  it('writes what the athlete typed, and nothing else', async () => {
    const declare = vi
      .spyOn(api, 'declareHeartRates')
      .mockResolvedValue({ declaration: declared.declaration!, generated_at: '2026-11-24T06:30:00Z' })
    await heartRatesScreen()

    await userEvent.type(await screen.findByTestId('heart-rates-max-input'), '186')
    await userEvent.selectOptions(await screen.findByTestId('heart-rates-max-source'), 'test')
    await userEvent.type(await screen.findByTestId('heart-rates-resting-input'), '52')
    await userEvent.selectOptions(await screen.findByTestId('heart-rates-resting-source'), 'doctor_lab')
    await userEvent.click(await screen.findByTestId('heart-rates-save'))

    await waitFor(() => expect(declare).toHaveBeenCalled())
    const body = declare.mock.calls[0]![0]
    expect(body.max_heart_rate_bpm).toBe(186)
    expect(body.max_source).toBe('test')
    expect(body.resting_heart_rate_bpm).toBe(52)
    expect(body.resting_source).toBe('doctor_lab')
  })

  it('does not write a declaration with no heart rate in it', async () => {
    const declare = vi.spyOn(api, 'declareHeartRates')
    await heartRatesScreen()

    await userEvent.click(await screen.findByTestId('heart-rates-save'))

    await waitFor(() => expect(screen.getByTestId('heart-rates-card')).toBeTruthy())
    expect(declare).not.toHaveBeenCalled()
  })
})
