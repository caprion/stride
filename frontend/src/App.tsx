import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './components/AppLayout'
import { AuthGate } from './components/AuthGate'
import { DashboardPage } from './pages/DashboardPage'
import { FeedPage } from './pages/FeedPage'
import { ActivityPage } from './pages/ActivityPage'
import { TrainingPage } from './pages/TrainingPage'
import { StatsPage } from './pages/StatsPage'
import { SettingsPage } from './pages/SettingsPage'
import PlanPage from './pages/PlanPage'
import PhysiologyPage from './pages/PhysiologyPage'
import FitnessPage from './pages/FitnessPage'
import ComparePage from './pages/ComparePage'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 30_000,
    },
  },
})

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AuthGate>
          <AppLayout>
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
          </AppLayout>
        </AuthGate>
      </BrowserRouter>
    </QueryClientProvider>
  )
}
