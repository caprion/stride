/**
 * The challenge grid.
 *
 * **A week is met or it is not**, and the week that did not reach the
 * target is drawn quietly rather than crossed out: nothing in this
 * product is red, and a row of red crosses beside a goal the athlete set
 * themselves is the opposite of what the grid is for.
 */

import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { challengesFixture } from '../src/fixtures'
import { renderApp } from './helpers'

describe('the challenge grid', () => {
  it('draws a mark for every week of every challenge', async () => {
    renderApp('/progress/challenges')
    const grid = await screen.findByTestId('challenge-grid')

    const cells = grid.querySelectorAll('[data-testid^="challenge-cell-"]')
    expect(cells).toHaveLength(3)
  })

  it('says which weeks were met', async () => {
    renderApp('/progress/challenges')
    const grid = await screen.findByTestId('challenge-grid')

    const met = grid.querySelector('[data-testid="challenge-cell-week_distance:40000-2026-04-27"]')
    expect(met?.getAttribute('data-met')).toBe('true')
    const missed = grid.querySelector(
      '[data-testid="challenge-cell-week_distance:40000-2026-05-11"]',
    )
    expect(missed?.getAttribute('data-met')).toBe('false')
  })

  it('counts the weeks, and does not call them a failure', async () => {
    renderApp('/progress/challenges')
    const grid = await screen.findByTestId('challenge-grid')

    // The count is there...
    expect(grid.textContent).toContain('2 of 3 weeks')
    // ...and nothing calls a week a failure.
    const text = (grid.textContent ?? '').toLowerCase()
    for (const word of ['fail', 'missed', 'bad', 'poor']) {
      expect(text).not.toContain(word)
    }
  })

  it('is on the wall, with the habits beside it', async () => {
    renderApp('/profile')
    const card = await screen.findByTestId('progress-wall-goals')
    expect(card).toBeTruthy()
  })

  it('says when there is nothing to draw', async () => {
    vi.spyOn(api, 'insightsChallenges').mockResolvedValue({
      ...challengesFixture,
      challenges: [],
    })
    renderApp('/progress/challenges')
    const empty = await screen.findByTestId('challenge-grid-empty')
    expect(empty.textContent).toContain('No challenges yet')
  })
})
