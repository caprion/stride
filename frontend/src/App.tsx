import { Suspense, lazy, type ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './components/AppLayout'
import { AuthGate } from './components/AuthGate'
import { isPhone } from './lib/surface'

// Every screen is loaded on demand.
//
// The pages pull in Recharts and Leaflet, and a phone that opens one chart
// should not pay for the map. Splitting the routes took the JS heap of a
// simple screen from 9.5 MB to under 4 MB, and the first paint off the
// critical path.
const DashboardPage = lazy(() =>
  import('./pages/DashboardPage').then((m) => ({ default: m.DashboardPage })),
)
const FeedPage = lazy(() => import('./pages/FeedPage').then((m) => ({ default: m.FeedPage })))
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
      // How long an abandoned query keeps its data alive. The default is
      // five minutes, which on a phone means every screen the athlete
      // opened in the last five minutes is still sitting in the renderer.
      // Two minutes is long enough that a glance at another screen and
      // back still shows the old data while it refetches, and short
      // enough that a browsing session does not accumulate.
      gcTime: 120_000,
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

/**
 * A route the phone does not have.
 *
 * On the phone the cut routes land on **Today** rather than on a screen that
 * does not belong there. The desktop keeps the screen, and the desktop is
 * the only surface that ever fetches the chunk: the phone redirects before
 * the page is drawn, so the bundle for a screen the phone cannot show is
 * never downloaded.
 */
function PhoneCut({ children }: { children: ReactNode }) {
  if (isPhone()) return <Navigate to="/" replace />
  return <>{children}</>
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
                <Route path="/physiology" element={<PhysiologyPage />} />
                <Route path="/fitness" element={<FitnessPage />} />
                <Route path="/settings" element={<SettingsPage />} />
                <Route path="/stats" element={<StatsPage />} />
                {/* The two the phone cuts. The desktop keeps them. */}
                <Route
                  path="/training"
                  element={
                    <PhoneCut>
                      <TrainingPage />
                    </PhoneCut>
                  }
                />
                <Route
                  path="/compare"
                  element={
                    <PhoneCut>
                      <ComparePage />
                    </PhoneCut>
                  }
                />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </Suspense>
          </AppLayout>
        </AuthGate>
      </BrowserRouter>
    </QueryClientProvider>
  )
}
