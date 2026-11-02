/**
 * The files the app makes, and the controls that make them.
 *
 * The plan's actions are "share the comparison as an image or HTML file"
 * and "Export FIT". The app makes the file; the host delivers it. So the
 * control is only there where the host can deliver one — the desktop,
 * where a download delivers it — and the phone's native share sheet
 * brings it back.
 */

import { readFileSync, readdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it } from 'vitest'
import { comparisonDocument } from '../src/lib/share'
import { compareAnswer } from '../src/fixtures'
import { renderApp } from './helpers'

const here = dirname(fileURLToPath(import.meta.url))

afterEach(() => {
  delete (window as { __stride_bridge__?: unknown }).__stride_bridge__
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('the comparison as a file', () => {
  it('holds the core\'s words and the catalogue\'s labels', () => {
    const comparison = compareAnswer('102', '101')
    expect(comparison).toBeTruthy()
    const document = comparisonDocument(comparison as NonNullable<typeof comparison>)

    // The basis is the core's sentence.
    expect(document).toContain('Different distances — read with care')
    // The values are the core's rendering.
    expect(document).toContain('10.2 km')
    expect(document).toContain('5:25 /km')
    // The labels are the catalogue's.
    expect(document).toContain('Difference')
    expect(document).toContain('Metric')
  })

  it('is offered where the host can deliver it', async () => {
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-metrics"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="compare-share"]')).toBeTruthy()
  })

  it('is not offered on the phone, which has no share sheet yet', async () => {
    // The phone's bridge: the native host.
    window.__stride_bridge__ = {
      apiCall: async () => JSON.stringify(compareAnswer('102', '101')),
    }
    const { container } = renderApp('/compare/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="compare-metrics"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="compare-share"]')).toBeNull()
  })
})

describe('the FIT file', () => {
  it('is offered on the desktop', async () => {
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-detail"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="activity-export"]')).toBeTruthy()
  })

  it('is not offered on the phone, which has no share sheet yet', async () => {
    window.__stride_bridge__ = {
      apiCall: async () => JSON.stringify(compareAnswer('102', '101')),
    }
    const { container } = renderApp('/runs/101')
    await waitFor(() =>
      expect(container.querySelector('[data-testid="activity-detail"]')).toBeTruthy(),
    )

    expect(container.querySelector('[data-testid="activity-export"]')).toBeNull()
  })
})

describe('the app\'s own source', () => {
  it('names no run id of its own', () => {
    // The Compare screen's pair comes from the candidates route; a run id
    // written into the file would be a pair the app made up.
    const source = readFileSync(join(here, '..', 'src', 'pages', 'ComparePage.tsx'), 'utf8')
    expect(source).not.toMatch(/'1\d\d'/)
  })

  it('offers no control that does nothing', () => {
    // Every button in the app's pages acts: a control that promises a
    // thing and does nothing is worse than an absent one.
    const pages = join(here, '..', 'src', 'pages')
    for (const file of readdirSync(pages)) {
      const source = readFileSync(join(pages, file), 'utf8')
      const buttons = source.split('<button').slice(1)
      for (const button of buttons) {
        const opening = button.slice(0, 600)
        const acts = opening.includes('onClick') || opening.includes('type="submit"')
        expect(acts, `${file}: a button with no action`).toBe(true)
      }
    }
  })
})
