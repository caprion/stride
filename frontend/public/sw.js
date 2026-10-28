/**
 * Stride's service worker.
 *
 * Two jobs, both narrow:
 *
 *  1. Make the app **installable**. A phone that opens Stride from the home
 *     screen gets a full-screen app with no browser chrome — which is the
 *     difference between "a website" and "an app" in the athlete's hand.
 *  2. Let the shell start when the network is gone, so a run recorded
 *     somewhere with no signal does not open a blank page.
 *
 * What it must never do is serve stale **data**. The API is never cached —
 * every request to /api/ goes straight to the network. Health data is not
 * something to serve from a cache, and the app already says it never leaves
 * the machine.
 */

const VERSION = 'stride-v1'
const SHELL = '/index.html'

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(VERSION)
      .then((cache) => cache.add(SHELL))
      .then(() => self.skipWaiting())
      .catch(() => self.skipWaiting()),
  )
})

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((key) => key !== VERSION).map((key) => caches.delete(key))))
      .then(() => self.clients.claim()),
  )
})

self.addEventListener('fetch', (event) => {
  const request = event.request
  if (request.method !== 'GET') return

  const url = new URL(request.url)
  if (url.origin !== self.location.origin) return

  // The API is never cached. Not a response, not a placeholder.
  if (url.pathname.startsWith('/api/')) return

  // Navigations: network first, so a deployed change is picked up
  // immediately, with the cached shell as the offline fallback.
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const copy = response.clone()
          caches.open(VERSION).then((cache) => cache.put(SHELL, copy))
          return response
        })
        .catch(() => caches.match(SHELL).then((hit) => hit || Response.error())),
    )
    return
  }

  // Static assets are content-hashed, so a cached copy is always correct.
  if (url.pathname.startsWith('/assets/') || url.pathname.startsWith('/fonts/')) {
    event.respondWith(
      caches.match(request).then(
        (hit) =>
          hit ||
          fetch(request).then((response) => {
            if (response.ok) {
              const copy = response.clone()
              caches.open(VERSION).then((cache) => cache.put(request, copy))
            }
            return response
          }),
      ),
    )
  }
})
