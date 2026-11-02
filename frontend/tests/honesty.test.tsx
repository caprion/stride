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
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { healthFixture } from '../src/fixtures'
import { renderApp } from './helpers'

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

describe("the reading's heading", () => {
  it('names when the readings are from, and is not advice', async () => {
    const { container } = renderApp('/')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="today-state-line"]')).toBeTruthy(),
    )

    const heading = container.querySelector('[data-testid="today-state-line"]')?.textContent ?? ''
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
    expect(container.querySelector('[data-testid="settings-about-version"]')?.textContent).toContain(
      `Version ${healthFixture.version}`,
    )
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
    await user.click(container.querySelector('[data-testid="compare-candidate-106"]') as HTMLElement)

    await waitFor(() => expect(compare).toHaveBeenCalled())
    const asked = compare.mock.calls.at(-1)
    expect(asked?.[0]).toBe(106)
    expect(asked?.[1]).toBe(101)
  })
})
