/**
 * The client entry point.
 *
 * The scaffold's placeholder: the real entry mounts the router.
 */

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { App } from './app/App'

const container = document.getElementById('root')
if (!container) {
  throw new Error('the app has no root element')
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
