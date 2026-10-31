/**
 * The client entry point.
 *
 * The app is mounted, the query client is created, and the router is
 * installed. Nothing else happens here.
 */

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import './index.css'
import { App } from './app/App'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // The data is read from the device; there is no server to race.
      staleTime: 30_000,
      refetchOnWindowFocus: false,
      // The shell owns the retry, the client does not (core-contract §7:
      // one rule, one owner). A failed read is a fact the screen shows.
      retry: false,
    },
  },
})

const container = document.getElementById('root')
if (!container) {
  throw new Error('the app has no root element')
}

createRoot(container).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>,
)
