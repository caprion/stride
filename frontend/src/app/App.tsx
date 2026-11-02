/**
 * The routes.
 *
 * Four destinations, and two contextual routes that are reached from
 * inside the app rather than from the navigation: the Plan week (from
 * Today) and Compare (from a run). Settings is reached from the shell's
 * secondary link, and the first-run page stands alone.
 */

import { useQuery } from '@tanstack/react-query'
import { Navigate, Route, Routes, useParams } from 'react-router-dom'
import { AppShell } from '../ui/layout/AppShell'
import { api } from '../lib/api'
import { currentHost } from '../lib/transport'
import { TodayPage } from '../pages/TodayPage'
import { RunsPage } from '../pages/RunsPage'
import { ProgressPage } from '../pages/ProgressPage'
import { BodyPage } from '../pages/BodyPage'
import { ComparePage } from '../pages/ComparePage'
import { PlanWeekPage } from '../pages/PlanWeekPage'
import { ActivityPage } from '../pages/ActivityPage'
import { SettingsPage } from '../pages/SettingsPage'
import { ConnectPage } from '../pages/ConnectPage'

function ActivityRoute() {
  const { id } = useParams()
  return <ActivityPage id={id ?? ''} />
}

function CompareRoute() {
  const { id } = useParams()
  return <ComparePage id={id ?? ''} />
}

/**
 * The first-run page, and when it exists.
 *
 * **It is the desktop's page.** The phone's first run is a native screen:
 * the key never enters a WebView, so the web page is not there — and the
 * phone's bridge refuses the key routes besides.
 *
 * And it **disappears once the app is connected**: the app opens on
 * Today instead, and the first-run page is not a page inside the app.
 */
function ConnectRoute() {
  const connection = useQuery({ queryKey: ['auth-status'], queryFn: api.authStatus })

  if (currentHost() === 'native') return <Navigate to="/" replace />
  if (connection.data?.connected) return <Navigate to="/" replace />

  return <ConnectPage />
}

/** The screens inside the app. */
function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<TodayPage />} />
      <Route path="/runs" element={<RunsPage />} />
      <Route path="/runs/:id" element={<ActivityRoute />} />
      <Route path="/progress" element={<ProgressPage />} />
      <Route path="/body" element={<BodyPage />} />
      <Route path="/plan" element={<PlanWeekPage />} />
      <Route path="/compare/:id" element={<CompareRoute />} />
      <Route path="/settings" element={<SettingsPage />} />
      <Route path="*" element={<TodayPage />} />
    </Routes>
  )
}

export function App() {
  return (
    <Routes>
      {/*
        The first-run page stands alone.

        It is outside the shell on purpose: no chrome, no tabs, no
        destinations — it is a page before the app, not a page in it.
      */}
      <Route path="/connect" element={<ConnectRoute />} />
      <Route
        path="*"
        element={
          <AppShell>
            <AppRoutes />
          </AppShell>
        }
      />
    </Routes>
  )
}
