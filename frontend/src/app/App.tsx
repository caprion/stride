/**
 * The routes.
 *
 * Four destinations, and two contextual routes that are reached from
 * inside the app rather than from the navigation: the Plan week (from
 * Today) and Compare (from a run).
 */

import { Route, Routes, useParams } from 'react-router-dom'
import { AppShell } from '../ui/layout/AppShell'
import { TodayPage } from '../pages/TodayPage'
import { RunsPage } from '../pages/RunsPage'
import { ProgressPage } from '../pages/ProgressPage'
import { BodyPage } from '../pages/BodyPage'
import { ComparePage } from '../pages/ComparePage'
import { PlanWeekPage } from '../pages/PlanWeekPage'
import { ActivityPage } from '../pages/ActivityPage'

function ActivityRoute() {
  const { id } = useParams()
  return <ActivityPage id={id ?? ''} />
}

function CompareRoute() {
  const { id } = useParams()
  return <ComparePage id={id ?? ''} />
}

export function App() {
  return (
    <AppShell>
      <Routes>
        <Route path="/" element={<TodayPage />} />
        <Route path="/runs" element={<RunsPage />} />
        <Route path="/runs/:id" element={<ActivityRoute />} />
        <Route path="/progress" element={<ProgressPage />} />
        <Route path="/body" element={<BodyPage />} />
        <Route path="/plan" element={<PlanWeekPage />} />
        <Route path="/compare/:id" element={<CompareRoute />} />
        <Route path="*" element={<TodayPage />} />
      </Routes>
    </AppShell>
  )
}
