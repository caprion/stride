/**
 * Stride's client entry point.
 *
 * Two things happen here and nowhere else: the app is mounted, and the
 * service worker is registered.
 */

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import 'leaflet/dist/leaflet.css'
import './index.css'
import App from './App'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)

/**
 * Keep the service worker's cache to the build that is on the phone.
 *
 * The worker caches the shell, the assets and the fonts, and it never
 * forgets. Its cache is named after the app rather than after the build,
 * the assets are content-hashed, and nothing ever deletes an entry — so
 * every release adds a set of files and keeps the previous set alive. On a
 * phone that is disk nobody reclaims, and none of it can ever be used
 * again.
 *
 * The fix is a marker in the cache: it records which assets belong to the
 * build that is running. When the running build changes, the assets of the
 * build it replaced are dropped. The fonts and the shell are not
 * content-hashed, so they stay.
 */
const BUILD_MARKER = '/__stride-build__'

/** The hashed assets the running build is made of. */
function buildAssets(): string[] {
  const paths = new Set<string>()
  for (const element of document.querySelectorAll('script[src], link[href]')) {
    const url =
      element instanceof HTMLScriptElement
        ? element.src
        : element instanceof HTMLLinkElement
          ? element.href
          : ''
    if (!url) continue
    const path = new URL(url, location.origin).pathname
    if (path.startsWith('/assets/')) paths.add(path)
  }
  return [...paths].sort()
}

async function reconcileBuildCache(): Promise<void> {
  if (!('caches' in window)) return
  try {
    for (const name of await caches.keys()) {
      const cache = await caches.open(name)
      await cache.delete(BUILD_MARKER)
      const marker = await cache.match(BUILD_MARKER)
      const current = buildAssets()

      if (marker) {
        const previous = JSON.parse(await marker.text()) as {
          entry: string[]
          assets: string[]
        }
        const sameBuild = previous.entry.join('|') === current.join('|')
        if (!sameBuild) {
          // A new build: the assets of the one it replaced can never be
          // asked for again. The stale entries are deleted by their stored
          // request, not by their path — the responses vary on Origin, and
          // a bare URL does not match an entry that was stored with one.
          const stale = new Set(previous.assets.filter((path) => !current.includes(path)))
          for (const request of await cache.keys()) {
            if (stale.has(new URL(request.url).pathname)) await cache.delete(request)
          }
        }
      }

      const cached = (await cache.keys())
        .map((request) => new URL(request.url).pathname)
        .filter((path) => path.startsWith('/assets/'))
        .sort()
      await cache.put(
        BUILD_MARKER,
        new Response(JSON.stringify({ entry: current, assets: cached })),
      )
    }
  } catch {
    /* a cache that cannot be read is a cache that cannot be reconciled */
  }
}

// Register the service worker so Stride can be installed on a phone and
// start even with no network. It is not a data cache — the API is never
// touched by it.
if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {
      // An unavailable service worker only costs the offline shell.
    })
    // Reconcile what a past build left behind, once the worker is in charge.
    // The worker writes its cache entries as it serves them, so give it a
    // moment to finish before the cache is read.
    navigator.serviceWorker.ready
      .then(() => new Promise((resolve) => setTimeout(resolve, 1500)))
      .then(() => reconcileBuildCache())
      .catch(() => {
        /* nothing to reconcile */
      })
  })
}
