/**
 * The wall's pictures open the runs behind them.
 *
 * A mark on a chart that carries nothing but a shape is a picture of a
 * picture: the reader sees the week, the day or the month, and there is
 * no way into the runs that made it. These tests hold the other half of
 * that:
 *
 *   - a mark that has runs behind it is a **link**, and it opens exactly
 *     the runs of the period it stands for;
 *   - the link **says what it opens**, so a reader does not have to
 *     hover to find out;
 *   - a mark with nothing behind it is **not** a link: absence is not a
 *     door.
 *
 * The words are the catalogue's own, and the links are built with the
 * app's own `runsPath`, so the runs list is asked for the range the mark
 * stands for.
 */

import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { yearFixture } from '../src/fixtures'
import { renderApp } from './helpers'

/** The days of the year that hold runs. */
const daysWithRuns = yearFixture.days.filter((day) => day.sessions > 0)

describe('the ways into the runs behind the wall', () => {
  it('a week on the pace card opens that week’s runs', async () => {
    renderApp('/progress')
    const link = await screen.findByTestId('progress-wall-pace-2026-04-27')

    // The week is the Monday and the six days after it — the same seven
    // days the weekly volume chart uses.
    expect(link).toHaveAttribute('href', '/runs?from=2026-04-27&to=2026-05-03')
    // And the link says what it opens.
    expect(link).toHaveAccessibleName('The runs of the week starting Mon 27 Apr')
  })

  it('a week on the rolling window opens that week’s runs', async () => {
    renderApp('/progress')
    const link = await screen.findByTestId('progress-wall-window-2026-04-27')

    expect(link).toHaveAttribute('href', '/runs?from=2026-04-27&to=2026-05-03')
    expect(link).toHaveAccessibleName('The runs of the week starting Mon 27 Apr')
  })

  it('a day on the year card opens that day’s runs', async () => {
    renderApp('/year')
    const link = await screen.findByTestId('progress-wall-year-2026-05-10')

    // A day is one day.
    expect(link).toHaveAttribute('href', '/runs?from=2026-05-10&to=2026-05-10')
    expect(link).toHaveAccessibleName('The runs of Sun 10 May')
  })

  it('a day on the activity heatmap opens that day’s runs', async () => {
    renderApp('/progress/heatmap')
    const link = await screen.findByTestId('progress-wall-heatmap-2026-05-10')

    expect(link).toHaveAttribute('href', '/runs?from=2026-05-10&to=2026-05-10')
    expect(link).toHaveAccessibleName('The runs of Sun 10 May')
  })

  it('a month on the months chart opens that month’s runs', async () => {
    renderApp('/progress/months')
    const link = await screen.findByTestId('progress-wall-months-2026-05')

    // A month is the first of the month to the last.
    expect(link).toHaveAttribute('href', '/runs?from=2026-05-01&to=2026-05-31')
    expect(link).toHaveAccessibleName('The runs of May 2026')
  })

  it('a day with no runs is not a link', async () => {
    renderApp('/year')
    const grid = await screen.findByTestId('year-calendar')

    // The days that hold runs are links, and only those days are.
    expect(grid.querySelectorAll('a')).toHaveLength(daysWithRuns.length)

    // A day with nothing in it is still drawn, and it is a quiet mark:
    // absence is not a door.
    const quiet = grid.querySelector('[data-date="2025-06-01"]')
    expect(quiet).toBeTruthy()
    expect(quiet?.closest('a')).toBeNull()
  })
})
