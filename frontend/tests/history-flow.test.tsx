import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from '../src/lib/api'
import { activityDetailFixture, vitalsFixture } from '../src/fixtures'
import { renderApp } from './helpers'

afterEach(() => {
  vi.restoreAllMocks()
  window.history.pushState({}, '', '/?fixtures=1')
})

describe('linked history exploration', () => {
  it('takes a Today day to a server-filtered feed and the same Body date', async () => {
    const activities = vi.spyOn(api, 'activities')
    renderApp('/')
    const week = await screen.findByTestId('today-week')
    const day = week.querySelector('a[href="/runs?from=2026-05-13&to=2026-05-13"]')!
    await userEvent.click(day)
    await waitFor(() =>
      expect(activities).toHaveBeenLastCalledWith({
        limit: 40,
        cursor: undefined,
        since: '2026-05-13',
        until: '2026-05-13',
      }),
    )
    expect(screen.getByRole('link', { name: 'Explore Body' })).toHaveAttribute(
      'href',
      '/body?date=2026-05-13',
    )
    expect(screen.getByRole('button', { name: 'All saved history' })).toBeVisible()
  })

  it('keeps the filtered history when returning from a named activity', async () => {
    renderApp('/runs?from=2026-05-01&to=2026-05-13')
    await userEvent.click(await screen.findByTestId('run-101'))
    expect(
      await screen.findByRole('heading', { level: 1, name: activityDetailFixture.name }),
    ).toBeVisible()
    expect(screen.getByRole('link', { name: 'Back to runs' })).toHaveAttribute(
      'href',
      '/runs?from=2026-05-01&to=2026-05-13',
    )
  })

  it('browses Body dates in the URL and preserves the chosen history window', async () => {
    const vitals = vi.spyOn(api, 'vitals').mockImplementation(async (params) => ({
      ...vitalsFixture,
      date: params?.date ?? vitalsFixture.date,
    }))
    renderApp('/body?date=2026-05-12')
    await userEvent.selectOptions(await screen.findByLabelText('History window'), '90')
    await userEvent.click(screen.getByRole('button', { name: 'Previous week' }))
    await waitFor(() =>
      expect(vitals).toHaveBeenLastCalledWith({ date: '2026-05-05', days: 90 }),
    )
    expect(screen.getByRole('link', { name: 'Runs on this date' })).toHaveAttribute(
      'href',
      '/runs?from=2026-05-05&to=2026-05-05',
    )
  })

  it('shows invalid date order explicitly without requesting a reversed window', async () => {
    const activities = vi.spyOn(api, 'activities')
    renderApp('/runs?from=2026-05-13&to=2026-05-01')
    expect(await screen.findByRole('alert')).toHaveTextContent('through date is before')
    expect(activities).not.toHaveBeenCalled()
  })

  it('keeps filtered history through Compare and switches both charts to pace', async () => {
    const returnTo = '/runs?from=2026-05-01&to=2026-05-13'
    renderApp(`/compare/101?${new URLSearchParams({ with: '102', returnTo })}`)
    await screen.findByTestId('compare-metrics')
    expect(screen.getByRole('link', { name: 'Back to runs' })).toHaveAttribute('href', returnTo)
    await userEvent.selectOptions(screen.getByLabelText('Signal'), 'pace')
    expect(screen.getByText(/^A.*Pace$/)).toBeInTheDocument()
    expect(screen.getByText(/^B.*Pace$/)).toBeInTheDocument()
    await userEvent.click(screen.getByTestId('compare-run-b').querySelector('a')!)
    await screen.findByTestId('activity-detail')
    expect(await screen.findByRole('link', { name: 'Back to runs' })).toHaveAttribute(
      'href',
      returnTo,
    )
  })

  it('requests selected-period totals and links a selected progress week to history', async () => {
    const progress = vi.spyOn(api, 'insightsProgress')
    renderApp('/progress')
    await screen.findByTestId('progress-summary')
    await userEvent.click(screen.getByRole('button', { name: '4 weeks' }))
    await waitFor(() => expect(progress).toHaveBeenLastCalledWith(4))
    fireEvent.change(screen.getByLabelText('Explore a week'), {
      target: { value: '2026-05-04' },
    })
    expect(screen.getByRole('link', { name: 'View runs this week' })).toHaveAttribute(
      'href',
      '/runs?from=2026-05-04&to=2026-05-10',
    )
  })

  it('exposes an empty plan and leads with running before wellness', async () => {
    window.history.pushState({}, '', '/?fixtures=noplan')
    const view = renderApp('/')
    const last = await screen.findByTestId('today-last-run')
    expect(screen.getByRole('link', { name: 'View plan' })).toHaveAttribute('href', '/plan')
    const vitals = view.container.querySelector('[data-testid="today-vitals"]')!
    expect(last.compareDocumentPosition(vitals) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })

  it('routes desktop backup to host tools and separates training preferences', async () => {
    renderApp('/settings')
    const exportBackup = vi.spyOn(api, 'exportBackup')
    expect(
      await screen.findByRole('link', { name: 'Backup, restore and export' }),
    ).toHaveAttribute('href', '/rig')
    expect(screen.queryByTestId('settings-goal-input')).toBeNull()
    await userEvent.click(screen.getByRole('link', { name: 'Training preferences' }))
    expect(await screen.findByTestId('settings-goal-input')).toBeVisible()
    expect(screen.queryByTestId('settings-connection')).toBeNull()
    expect(exportBackup).not.toHaveBeenCalled()
  })
})
