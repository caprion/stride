/**
 * The app's honesty.
 *
 * The findings of an independent product review, as assertions:
 *
 *   - the reading's heading names what it is, and never reads as advice;
 *   - the About card's version is the one the api answered, and there is
 *     no second version string in the app;
 *   - the Compare screen compares the athlete's real runs — the ones the
 *     candidates route names — and the picker opens and asks for the pair
 *     it chose.
 *
 * The charts' axes are checked in the browser (the viewport matrix): a
 * chart has no size in jsdom, so its labels are not in the document there.
 */

import { readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { Sparkline } from '../src/components/Sparkline'
import {
  healthFixture,
  syncStatusFailedFixture,
  syncStatusFreshFixture,
  vitalsFixture,
} from '../src/fixtures'
import { renderApp, renderWith } from './helpers'

const here = dirname(fileURLToPath(import.meta.url))
const src = join(here, '..', 'src')

/** Every file of the app's own source. */
function walk(dir: string): string[] {
  const out: string[] = []
  for (const name of readdirSync(dir)) {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) out.push(...walk(path))
    else if (/\.(ts|tsx)$/.test(name)) out.push(path)
  }
  return out
}

afterEach(() => {
  window.history.pushState({}, '', '/?fixtures=1')
  vi.restoreAllMocks()
})

describe('missing readings', () => {
  it('requests the selected health date and calendar window', async () => {
    const vitals = vi.spyOn(api, 'vitals').mockImplementation(async (params) => ({
      ...vitalsFixture,
      date: params?.date ?? vitalsFixture.date,
    }))
    renderApp('/body')
    await userEvent.selectOptions(await screen.findByLabelText('History window'), '90')
    await waitFor(() => expect(vitals).toHaveBeenLastCalledWith({ date: undefined, days: 90 }))
    fireEvent.change(screen.getByLabelText('Date'), { target: { value: '2026-04-15' } })
    await waitFor(() =>
      expect(vitals).toHaveBeenLastCalledWith({ date: '2026-04-15', days: 90 }),
    )
  })

  it.each(['/', '/body'])(
    'keeps absent signals secondary on %s and restores them when data arrives',
    async (route) => {
      const absent = {
        ...vitalsFixture,
        vitals: vitalsFixture.vitals.filter((vital) => vital.key !== 'hrv'),
        history: {
          ...vitalsFixture.history,
          hrv: vitalsFixture.history.hrv.map((point) => ({ ...point, value: null })),
        },
      }
      const vitals = vi.spyOn(api, 'vitals').mockResolvedValue(absent)
      const view = renderApp(route)
      await screen.findByTestId(route === '/' ? 'today-vitals' : 'body-signals')
      expect(view.container.querySelector('[data-vital=hrv]')).toBeNull()
      const additional = screen.getByTestId('additional-measurements')
      expect(additional).not.toHaveAttribute('open')
      expect(additional).toHaveTextContent('HRV')
      expect(view.container.querySelector('[data-vital=sleep]')).not.toBeNull()
      view.unmount()
      vitals.mockResolvedValue(vitalsFixture)
      const restored = renderApp(route)
      await waitFor(() =>
        expect(restored.container.querySelector('[data-vital=hrv]')).not.toBeNull(),
      )
    },
  )

  it.each(['/', '/body'])(
    'retains recorded history on %s when the latest reading is absent',
    async (route) => {
      vi.spyOn(api, 'vitals').mockResolvedValue({
        ...vitalsFixture,
        vitals: vitalsFixture.vitals.filter((vital) => vital.key !== 'hrv'),
      })
      const { container } = renderApp(route)
      await screen.findByTestId(route === '/' ? 'today-vitals' : 'body-signals')
      expect(container.querySelector('[data-vital=hrv] svg')).not.toBeNull()
      expect(container.querySelector('[data-vital-value=hrv]')).toBeNull()
    },
  )

  it('breaks a sparkline at missing values and retains their horizontal positions', () => {
    const { container } = renderWith(<Sparkline values={[null, 10, 20, null, 30, null]} />)
    const lines = [...container.querySelectorAll('polyline')]
    expect(lines).toHaveLength(2)
    expect(
      lines[0]
        ?.getAttribute('points')
        ?.split(' ')
        .map((point) => point.split(',')[0]),
    ).toEqual(['24', '48'])
    expect(lines[1]?.getAttribute('points')?.split(',')[0]).toBe('96')
  })
})

describe('screen loading failures', () => {
  it.each([
    ['/', 'insightsToday'],
    ['/body', 'vitals'],
    ['/progress', 'insightsProgress'],
  ] as const)(
    'shows a retry rather than empty data or endless loading on %s',
    async (route, method) => {
      vi.spyOn(api, method).mockRejectedValue(new Error('synthetic unavailable store'))
      renderApp(route)
      expect(await screen.findByRole('alert')).toHaveTextContent('This did not load')
      expect(screen.getByRole('button', { name: 'Try again' })).toBeVisible()
    },
  )

  describe('manual sync', () => {
    it('labels pending history separately from a failed pull', async () => {
      vi.spyOn(api, 'syncStatus').mockResolvedValue({
        ...syncStatusFreshFixture,
        activity_count: 6,
        more_pending: true,
      })
      renderApp('/')
      expect(await screen.findByTestId('today-history-pending')).toHaveTextContent(
        'Older history is still being checked',
      )
      expect(screen.queryByTestId('today-sync-failed')).not.toBeInTheDocument()
    })

    it('refreshes the last sync state after a successful retry', async () => {
      vi.spyOn(api, 'syncStatus')
        .mockResolvedValueOnce(syncStatusFailedFixture)
        .mockResolvedValue(syncStatusFreshFixture)
      vi.spyOn(api, 'syncRun').mockResolvedValue({})
      renderApp('/')
      await screen.findByTestId('today-sync-failed')
      await userEvent.click(screen.getByRole('button', { name: 'Try again' }))
      await waitFor(() => expect(screen.queryByTestId('today-sync-failed')).toBeNull())
    })
  })
})

describe("the reading's heading", () => {
  it('names when the readings are from, and is not advice', async () => {
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-state-line"]')).toBeTruthy(),
    )

    const heading =
      container.querySelector('[data-testid="today-state-line"]')?.textContent ?? ''
    expect(heading).toContain('Last night')
    // The heading is not "Rest": a card headed with the single word can
    // be heard as an instruction. The core's own label for the summary
    // row stays below it, where it is the name of a row.
    expect(heading.startsWith('Rest')).toBe(false)
    expect(heading).toContain('Rest')
  })
})

describe('the version', () => {
  it('is the one the api answered', async () => {
    const { container } = renderApp('/settings')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="settings-about-version"]')).toBeTruthy(),
    )

    // The api's answer is the only source: the card shows what /health
    // said, and the app holds no version of its own.
    expect(
      container.querySelector('[data-testid="settings-about-version"]')?.textContent,
    ).toContain(`Version ${healthFixture.version}`)
  })

  it('is the only version string in the app', () => {
    // The fixture is the api's answer, so it holds the version. Nothing
    // else may: a second version string is a second truth about which
    // build is running.
    const offenders: string[] = []
    for (const file of walk(src)) {
      if (file.includes(join('', 'fixtures'))) continue
      const text = readFileSync(file, 'utf8')
      if (/\b\d+\.\d+\.\d+\b/.test(text)) {
        offenders.push(`${relative(src, file)}: ${text.match(/\b\d+\.\d+\.\d+\b/)?.[0]}`)
      }
    }
    expect(offenders).toEqual([])
  })
})

describe('the Compare screen', () => {
  it('compares the runs the candidates route names, not a hardcoded pair', async () => {
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-metrics"]')).toBeTruthy(),
    )

    // The pair the table renders is the one the app chose out of the
    // candidates: the 9 May run against the 10 May one.
    const runs = container.querySelector('[data-testid="compare-runs"]')?.textContent ?? ''
    expect(runs).toContain('9 May 2026')
    expect(runs).toContain('10 May 2026')
    // And the file names no run id of its own.
    const source = readFileSync(join(src, 'pages', 'ComparePage.tsx'), 'utf8')
    expect(source).not.toMatch(/'1\d\d'/)
  })

  it('opens the picker, and the chosen run is the one it asks for', async () => {
    const user = userEvent.setup()
    const compare = vi.spyOn(api, 'compare')
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-metrics"]')).toBeTruthy(),
    )

    await user.click(screen.getByTestId('compare-choose'))

    const picker = container.querySelector('[data-testid="compare-picker"]')
    expect(picker).toBeTruthy()
    // The real runs, and not two of them: the athlete has six.
    const options = picker?.querySelectorAll('[data-testid^="compare-candidate-"]') ?? []
    expect(options.length).toBe(5)

    // The 3 May run is in the list; choosing it asks the api for that
    // pair — the earlier run is A, and the app does not decide that.
    await user.click(
      container.querySelector('[data-testid="compare-candidate-106"]') as HTMLElement,
    )

    await waitFor(() => expect(compare).toHaveBeenCalled())
    const asked = compare.mock.calls.at(-1)
    expect(asked?.[0]).toBe(106)
    expect(asked?.[1]).toBe(101)
  })
})
