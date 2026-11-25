/**
 * The routes.
 *
 * Five destinations, and two contextual routes that are reached from
 * inside the app rather than from the navigation: the Plan week (from
 * Today) and Compare (from a run). Settings is reached from the shell's
 * secondary link, and the first-run page stands alone.
 */

import { useQuery } from '@tanstack/react-query'
import { Navigate, Route, Routes, useLocation, useParams } from 'react-router-dom'
import { AppShell } from '../ui/layout/AppShell'
import { api } from '../lib/api'
import { currentHost } from '../lib/transport'
import { TodayPage } from '../pages/TodayPage'
import { RunsPage } from '../pages/RunsPage'
import { ProgressPage } from '../pages/ProgressPage'
import { ProgressPanelPage } from '../pages/ProgressPanelPage'
import { ProfilePage, ProfilePanelPage } from '../pages/ProfilePage'
import { panelDestination } from '../pages/taskAreas'

import { BodyPage } from '../pages/BodyPage'
import { ComparePage } from '../pages/ComparePage'
import { PlanWeekPage } from '../pages/PlanWeekPage'
import { PlanBasesPage } from '../pages/planBases'
import { ActivityPage } from '../pages/ActivityPage'
import { FilmPage } from '../pages/FilmPage'
import { HeatmapPage } from '../pages/HeatmapPage'
import { YearPage } from '../pages/YearPage'
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
 * The run film.
 *
 * It is reached by an explicit action on the run detail — the same
 * discipline the Street map and the heatmap use — so it is a route of its
 * own and not a fifth destination.
 */
function FilmRoute() {
  const { id } = useParams()
  return <FilmPage id={id ?? ''} />
}

/**
 * The panel route of Progress.
 *
 * **The old links still work.** The athlete profile's panels moved under
 * the athlete profile, and a link someone saved last month still opens
 * the panel it opened then: it lands on the profile's own page, with the
 * place it was opened from carried along, so that the panel's back link
 * says where the reader came from.
 */
function ProgressPanelRoute() {
  const { panel } = useParams()
  const location = useLocation()

  if (panel && panelDestination(panel) === 'profile') {
    // The query travels with the redirect: the reader's own place is
    // kept, and so is everything the app was asked for.
    return (
      <Navigate
        to={{ pathname: `/profile/${panel}`, search: location.search }}
        state={{ from: '/progress' }}
        replace
      />
    )
  }

  return <ProgressPanelPage />
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
      <Route path="/runs/:id/film" element={<FilmRoute />} />
      <Route path="/progress" element={<ProgressPage />} />
      {/*
        The narrow Progress opens one panel at a time, and this is that
        doorway. It is a route and not a destination: the destinations
        stay the destinations, and this is reached from Progress.
      */}
      <Route path="/progress/:panel" element={<ProgressPanelRoute />} />

      {/*
        The athlete profile: a destination of its own, with its own wall,
        its own index and its own panels.
      */}
      <Route path="/profile" element={<ProfilePage />} />
      <Route path="/profile/:panel" element={<ProfilePanelPage />} />

      {/*
        The pace page is gone: the pace is drawn on Progress's own card,
        and on the panel behind it. A link someone saved lands on that
        panel — the same answer, on the screen that owns it.
      */}
      <Route path="/pace" element={<Navigate to="/progress/pace" replace />} />

      {/*
        The year, as a film: the twelve months, the calendar of the days
        and what changed. It is reached from Progress, and it is a page
        and not a destination: the five destinations stay the five.
      */}
      <Route path="/year" element={<YearPage />} />
      <Route path="/body" element={<BodyPage />} />
      <Route path="/plan" element={<PlanWeekPage />} />
      {/*
        Adopting a programme. **A programme is an intent, not a
        baseline**: it is reached from the plan, and the app never
        suggests one.
      */}
      <Route path="/plan/bases" element={<PlanBasesPage />} />
      {/*
        The heatmap is reached contextually, from the Runs screen: it is
        not a destination of its own, and the athlete asks for it.
      */}
      <Route path="/heatmap" element={<HeatmapPage />} />
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
