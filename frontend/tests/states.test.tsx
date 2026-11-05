/**
 * The states.
 *
 * The plan says the screen inventory is complete and what was missing was
 * the states. These are the states, as assertions:
 *
 *   - **empty** — no runs, no wellness, no plan: the screen says what is
 *     missing and what to do, and invents no numbers;
 *   - **not enough data** — the honest form of the empty state;
 *   - **stale** — the screen says when the data was last synced;
 *   - **the plan's states** — a baseline, an imported plan and no plan at
 *     all, each named as the source it is;
 *   - **the Activity page** — the note, and the plan line;
 *   - **the Runs page** — the FIT import, and the answer when the file is
 *     a duplicate;
 *   - **the Plan week** — the seven days, the source, and the three
 *     counts: never a percentage, and never a "missed";
 *   - **the Compare page** — the metric diff, the basis, and the swap.
 */

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it } from 'vitest'
import { renderApp } from './helpers'

afterEach(() => {
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('Today, with nothing in it', () => {
  it('says what is missing, and invents no numbers', async () => {
    window.history.pushState({}, '', '/?fixtures=empty')
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-empty"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain("Connect your Garmin key to see today's numbers")
    expect(container.textContent).toContain('Connect Garmin')
    // No vitals, no session card and no last run: nothing is made up.
    expect(container.querySelector('[data-testid="today-vitals"]')).toBeNull()
    expect(container.querySelector('[data-testid="today-session"]')).toBeNull()
    expect(container.querySelector('[data-testid="today-last-run"]')).toBeNull()
    // And the connect card is not a row of "not enough data": the three
    // null-vitalled rows the router sends are hidden behind the card.
    expect(container.textContent).not.toContain('not enough data for a comparison yet')
  })

  it('says what is missing when the app has synced but the wellness has not come', async () => {
    window.history.pushState({}, '', '/?fixtures=nowellness')
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-no-vitals"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('No wellness data yet')
    expect(container.textContent).toContain('appear after they arrive in a sync')
    expect(
      container.querySelector<HTMLDetailsElement>('[data-testid="today-no-vitals"]')?.open,
    ).toBe(false)
    // The wellness is missing, and only the wellness: the runs and the
    // plan are the router's, and they are still there.
    expect(container.querySelector('[data-testid="today-session"]')).toBeTruthy()
    expect(container.querySelector('[data-testid="today-last-run"]')).toBeTruthy()
  })
})

describe('Today, with the data gone stale', () => {
  it('says when it was last synced', async () => {
    window.history.pushState({}, '', '/?fixtures=stale')
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-vitals"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('last synced 3 days ago')
    // And the numbers say which day they are from.
    expect(
      container.querySelector('[data-testid="today-reading-date"]')?.textContent,
    ).toContain('From')
  })
})

describe('Today, with the sync broken', () => {
  it('says the reason, offers a retry, and greys the numbers', async () => {
    window.history.pushState({}, '', '/?fixtures=failed')
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-sync-failed"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('The last sync did not complete')
    expect(container.textContent).toContain('Garmin refused the token')
    expect(container.textContent).toContain('Try again')
    // The numbers are there, but they are the last good ones.
    expect(container.querySelector('[data-testid="today-vitals"]')?.className).toContain(
      'opacity-60',
    )
  })
})

describe('Body, with too little history', () => {
  it('says that there is not enough data, and shows the value alone', async () => {
    window.history.pushState({}, '', '/?fixtures=notenough')
    const { container } = renderApp('/body')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="body-signals"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('not enough data for a comparison yet')
    // The value is there; the direction is not, because there is none.
    expect(container.textContent).toContain('6 h 10 m')
    expect(container.querySelector('[data-direction]')).toBeNull()
  })
})

describe('Body, with nothing in it', () => {
  it('says what is missing', async () => {
    window.history.pushState({}, '', '/?fixtures=empty')
    const { container } = renderApp('/body')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="body-empty"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('No wellness data yet')
  })
})

describe('Progress, with too little history', () => {
  it('says so plainly, and draws no percentage', async () => {
    window.history.pushState({}, '', '/?fixtures=notenough')
    const { container } = renderApp('/progress')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="progress-headline"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('not enough data yet')
    expect(
      container.querySelector('[data-testid="progress-needs-more"]')?.textContent,
    ).toContain('Needs 4 more weeks')
    expect(container.textContent).not.toContain('%')
  })
})

describe('Progress, with no plan', () => {
  it('omits the adherence card entirely', async () => {
    window.history.pushState({}, '', '/?fixtures=noplan')
    const { container } = renderApp('/progress')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="progress-headline"]')).toBeTruthy(),
    )

    expect(container.textContent).not.toContain('sessions run')
  })
})

describe('the Runs page', () => {
  it('says that there is nothing yet, and offers the two actions', async () => {
    window.history.pushState({}, '', '/?fixtures=empty')
    const { container } = renderApp('/runs')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="runs-empty"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('No activities yet')
    expect(container.textContent).toContain('Connect Garmin')
    expect(container.querySelector('[data-testid="fit-import"]')).toBeTruthy()
  })

  it('offers the import as a drop target on the desktop', async () => {
    const { container } = renderApp('/runs')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="fit-import"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="fit-drop"]')?.textContent).toContain(
      'Drop a FIT file here',
    )
  })

  it('says, once, that a file is already in the log', async () => {
    window.history.pushState({}, '', '/?fixtures=duplicate')
    const user = userEvent.setup()
    const { container } = renderApp('/runs')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="fit-import"]')).toBeTruthy(),
    )

    const input = container.querySelector('[data-testid="fit-file"]') as HTMLInputElement
    const file = new File(['not a fit file'], 'run.fit', { type: 'application/octet-stream' })
    await user.upload(input, file)

    await waitFor(() =>
      expect(container.querySelector('[data-testid="fit-answer"]')).toBeTruthy(),
    )
    const answer = container.querySelector('[data-testid="fit-answer"]')?.textContent ?? ''
    expect(answer).toContain('Already have this run')
    expect(answer).toContain('9 May')
    expect(answer).toContain('10.2 km')
  })
})

describe('the Activity page', () => {
  it('shows the story, and the why only when the core says one', async () => {
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-detail"]')).toBeTruthy(),
    )

    // The sentence is the core's and is never empty: the feed's line, on
    // the run's own page.
    expect(container.textContent).toContain('12.4 km in 1 h 7 m, at an average of 148 bpm')
    // The why is the one extra fact the detail knows — the run's own
    // shape — and the seeded run has no splits to stand out, so the
    // router answers null and the screen says nothing more.
    expect(container.querySelector('[data-testid="activity-why"]')).toBeNull()
  })

  it('shows the note, and no plan line when there is none', async () => {
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-detail"]')).toBeTruthy(),
    )

    // The plan line, when the plan has a session on the run's day. This
    // one has not, so the router answers null and the screen is quiet.
    expect(container.querySelector('[data-testid="activity-plan-line"]')).toBeNull()
    // The note is the athlete's own words, and it is edited here.
    expect(container.querySelector('[data-testid="activity-note"]')).toBeTruthy()
    expect(container.textContent).toContain('No note on this run')
  })

  it('saves a note on the run', async () => {
    const user = userEvent.setup()
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-note"]')).toBeTruthy(),
    )

    await user.click(
      container.querySelector('[data-testid="activity-note-edit"]') as HTMLElement,
    )
    await user.type(
      container.querySelector('[data-testid="activity-note-input"]') as HTMLElement,
      'Felt easy all the way.',
    )
    await user.click(
      container.querySelector('[data-testid="activity-note-save"]') as HTMLElement,
    )

    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-note-text"]')).toBeTruthy(),
    )
    expect(
      container.querySelector('[data-testid="activity-note-text"]')?.textContent,
    ).toContain('Felt easy all the way.')
  })
})

describe('the Plan week', () => {
  it('shows the seven days, the source and the three counts', async () => {
    const { container } = renderApp('/plan')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="plan-week"]')).toBeTruthy(),
    )

    const days = container.querySelectorAll('[data-testid^="plan-day-2"]')
    expect(days.length).toBe(7)

    // The source is named as the source it is.
    expect(container.querySelector('[data-testid="plan-source"]')?.textContent).toContain(
      'Prescribed (imported)',
    )

    // The counts, and never a percentage that hides them.
    // The counts are the router's: the seeded plan has no run on its days
    // yet, and the three counts say so.
    expect(container.querySelector('[data-testid="plan-count-done"]')?.textContent).toContain(
      '0 done',
    )
    expect(
      container.querySelector('[data-testid="plan-count-planned"]')?.textContent,
    ).toContain('5 planned')
    expect(
      container.querySelector('[data-testid="plan-count-not-done"]')?.textContent,
    ).toContain('2 not done')
    expect(container.textContent).not.toContain('%')
  })

  it('never says "missed"', async () => {
    const { container } = renderApp('/plan')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="plan-week"]')).toBeTruthy(),
    )

    expect(container.textContent?.toLowerCase()).not.toContain('missed')
  })

  it('does not imply an active baseline plan when no sessions were inferred', async () => {
    window.history.pushState({}, '', '/?fixtures=baseline')
    const { container } = renderApp('/plan')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="plan-none"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="plan-source"]')).toBeNull()
    expect(container.textContent).not.toContain('0 of 0')
  })

  it('says when there is no plan at all', async () => {
    window.history.pushState({}, '', '/?fixtures=noplan')
    const { container } = renderApp('/plan')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="plan-none"]')).toBeTruthy(),
    )

    expect(container.textContent).toContain('No plan is active')
    expect(container.textContent).toContain('imported, or inferred from your own history')
  })
})

describe('the Compare page', () => {
  it('names the basis and shows the metric diff', async () => {
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-metrics"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="compare-basis"]')?.textContent).toContain(
      'Different distances — read with care',
    )
    expect(container.querySelector('[data-testid="compare-row-pace"]')?.textContent).toContain(
      'Pace',
    )
  })

  it('flips every delta when A and B are swapped', async () => {
    const user = userEvent.setup()
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-row-pace"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="compare-row-pace"]')?.textContent).toContain(
      '−1 s/km',
    )

    await user.click(screen.getByTestId('compare-swap'))

    await waitFor(() =>
      expect(
        container.querySelector('[data-testid="compare-row-pace"]')?.textContent,
      ).toContain('+1 s/km'),
    )
  })
})
