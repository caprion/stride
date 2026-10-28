import { Suspense, lazy } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './components/AppLayout'
import { AuthGate } from './components/AuthGate'

// Every screen is loaded on demand.
//
// The pages pull in Recharts and Leaflet, and a phone that opens one chart
// should not pay for the map. Splitting the routes took the JS heap of a
// simple screen from 9.5 MB to under 4 MB, and the first paint off the
// critical path.
const DashboardPage = lazy(() =>
  import('./pages/DashboardPage').then((m) => ({ default: m.DashboardPage })),
)
const FeedPage = lazy(() =>
  import('./pages/FeedPage').then((m) => ({ default: m.FeedPage })),
)
const ActivityPage = lazy(() =>
  import('./pages/ActivityPage').then((m) => ({ default: m.ActivityPage })),
)
const TrainingPage = lazy(() =>
  import('./pages/TrainingPage').then((m) => ({ default: m.TrainingPage })),
)
const StatsPage = lazy(() =>
  import('./pages/StatsPage').then((m) => ({ default: m.StatsPage })),
)
const SettingsPage = lazy(() =>
  import('./pages/SettingsPage').then((m) => ({ default: m.SettingsPage })),
)
const PlanPage = lazy(() => import('./pages/PlanPage'))
const PhysiologyPage = lazy(() => import('./pages/PhysiologyPage'))
const FitnessPage = lazy(() => import('./pages/FitnessPage'))
const ComparePage = lazy(() => import('./pages/ComparePage'))

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 30_000,
    },
  },
})

// The measurement harness (scripts/measure.mjs) needs to see what the query
// cache is holding — the resident queries and the size of their data are a
// large part of what a renderer keeps alive on a phone. One property, no
// runtime dependency, and it is the same client the app already uses.
;(window as unknown as { __strideQueryClient?: QueryClient }).__strideQueryClient = queryClient

function RouteFallback() {
  return (
    <div className="flex min-h-[60vh] items-center justify-center">
      <div className="h-6 w-6 animate-spin rounded-full border-2 border-edge border-t-brand-500" />
    </div>
  )
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AuthGate>
          <AppLayout>
            <Suspense fallback={<RouteFallback />}>
              <Routes>
                <Route path="/" element={<DashboardPage />} />
                <Route path="/plan" element={<PlanPage />} />
                <Route path="/feed" element={<FeedPage />} />
                <Route path="/activities/:id" element={<ActivityPage />} />
                <Route path="/training" element={<TrainingPage />} />
                <Route path="/physiology" element={<PhysiologyPage />} />
                <Route path="/fitness" element={<FitnessPage />} />
                <Route path="/compare" element={<ComparePage />} />
                <Route path="/stats" element={<StatsPage />} />
                <Route path="/settings" element={<SettingsPage />} />
                    <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </Suspense>
          </AppLayout>
        </AuthGate>
      </BrowserRouter>
    </QueryClientProvider>
  )
}
