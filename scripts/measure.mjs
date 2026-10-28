#!/usr/bin/env node
/**
 * Measure the app's runtime footprint.
 *
 * The bundle size is only half the story: what matters on a phone is the
 * heap it holds and the work the CPU does when a screen is opened. This
 * walks every route in a fresh page and records the JS heap, the time to
 * paint, the long tasks and the bytes transferred.
 *
 *   node scripts/measure.mjs [baseUrl]              per-route table
 *   node scripts/measure.mjs --session [baseUrl]    one session, every screen
 *   node scripts/measure.mjs --json out.json        machine-readable results
 *
 * The renderer on a phone is not one number. It is five layers that are
 * measured separately here, because they are paid for separately and they
 * are fixed by different code:
 *
 *   1. the JS heap          — CDP `Runtime.getHeapUsage` after a collection
 *   2. the DOM             — node count, and the element types that make it
 *   3. decoded images      — naturalWidth x naturalHeight x 4 per image
 *   4. the query cache     — what React Query is still holding for a session
 *   5. the service worker  — what the Cache Storage is holding on disk
 *
 * And the number that matters on a phone is the *session*: the athlete
 * browses every screen in one renderer, so the caches accumulate. The
 * `--session` mode walks the whole app in one context, which is the only
 * way to see that.
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'

const args = process.argv.slice(2)
const MODE = args.includes('--session') ? 'session' : 'routes'
const JSON_OUT = args.includes('--json') ? args[args.indexOf('--json') + 1] : null
const BASE = args.find((a) => a.startsWith('http')) || 'http://127.0.0.1:8000'
// How long to let a screen settle before it is read. The slow endpoints
// (the fingerprint list recomputes every run) land well after the load
// event, and a screen read mid-flight is not a finished screen.
const SETTLE = Number((args.find((a) => a.startsWith('--settle=')) || '--settle=2500').split('=')[1])
// A phone's pixel ratio. The default of 1 is a desktop; 3 is a real phone,
// and it is the ratio that would inflate a tile request.
const SCALE = Number((args.find((a) => a.startsWith('--scale=')) || '--scale=1').split('=')[1])

const ROUTES = [
  ['dashboard', '/'],
  ['plan', '/plan'],
  ['feed', '/feed'],
  ['activity', '/activities/162'],
  ['training', '/training'],
  ['physiology', '/physiology'],
  ['fitness', '/fitness'],
  ['compare', '/compare'],
  ['stats', '/stats'],
  ['settings', '/settings'],
]

/* ------------------------------------------------------------------ utils */

const kb = (n) => String(Math.round(n / 1024))
const mb = (n) => (n / 1024 / 1024).toFixed(2)
const ms = (n) => `${Math.round(n)}ms`

/**
 * Collect long tasks from the moment the page starts.
 *
 * `performance.getEntriesByType('longtask')` is not implemented everywhere,
 * so an observer is installed before any document script runs.
 */
const INIT_SCRIPT = () => {
  window.__longTasks = []
  try {
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        window.__longTasks.push({ start: entry.startTime, duration: entry.duration })
      }
    }).observe({ type: 'longtask', buffered: true })
  } catch {
    /* longtask is not supported here */
  }
}

/** Everything the page knows about itself, read from inside the renderer. */
async function readPageState(page) {
  return page.evaluate(async () => {
    // --- DOM -----------------------------------------------------------
    const all = document.querySelectorAll('*')
    const byType = {}
    for (const node of all) {
      const tag = node.tagName.toLowerCase()
      byType[tag] = (byType[tag] || 0) + 1
    }

    // --- decoded images ------------------------------------------------
    // 4 bytes per pixel is the RGBA cost of one decoded bitmap. The GPU
    // keeps a second copy, so the real cost is roughly double this.
    let decodedBytes = 0
    let tileBytes = 0
    let tileCount = 0
    let imageCount = 0
    const images = []
    for (const img of document.querySelectorAll('img')) {
      const w = img.naturalWidth || 0
      const h = img.naturalHeight || 0
      const bytes = w * h * 4
      imageCount += 1
      decodedBytes += bytes
      const isTile = img.classList.contains('leaflet-tile')
      if (isTile) {
        tileBytes += bytes
        tileCount += 1
      }
      images.push({
        src: (img.currentSrc || img.src || '').slice(-90),
        w,
        h,
        bytes,
        tile: isTile,
      })
    }

    // --- network images ------------------------------------------------
    const resources = performance.getEntriesByType('resource')
    const netImages = resources.filter((r) => r.initiatorType === 'img' || /\.png|\.webp|\.jpg|\.jpeg|\.svg/i.test(r.name))
    const netTiles = resources.filter((r) => r.name.includes('/api/v1/tiles/'))

    // --- React Query cache ---------------------------------------------
    const client = window.__strideQueryClient
    const queries = []
    if (client) {
      for (const query of client.getQueryCache().getAll()) {
        const data = query.state.data
        let size = 0
        try {
          size = data === undefined ? 0 : JSON.stringify(data).length
        } catch {
          size = 0
        }
        queries.push({
          key: JSON.stringify(query.queryKey),
          size,
          state: query.state.status,
          observers: query.getObserversCount(),
        })
      }
    }

    // --- service worker cache ------------------------------------------
    let sw = null
    if ('caches' in window) {
      const names = await caches.keys()
      const caches_ = []
      let total = 0
      for (const name of names) {
        const cache = await caches.open(name)
        const keys = await cache.keys()
        let bytes = 0
        for (const request of keys) {
          const hit = await cache.match(request)
          if (!hit) continue
          const header = hit.headers.get('content-length')
          bytes += header ? Number(header) : (await hit.clone().blob()).size
        }
        caches_.push({ name, entries: keys.length, bytes })
        total += bytes
      }
      sw = { caches: caches_, total }
    }

    // --- long tasks ------------------------------------------------------
    const longTasks = window.__longTasks || []

    // A screen that is still fetching is not a finished screen: its heap
    // and its DOM are a snapshot mid-flight. The harness waits for the
    // network to go quiet, and this counts what is still pending.
    const pending = document.querySelectorAll('.animate-pulse').length

    return {
      dom: { nodes: all.length, byType },
      images: {
        count: imageCount,
        decodedBytes,
        tileCount,
        tileBytes,
        largest: images.sort((a, b) => b.bytes - a.bytes).slice(0, 5),
      },
      network: {
        images: netImages.length,
        imageBytes: netImages.reduce((sum, r) => sum + (r.transferSize || r.encodedBodySize || 0), 0),
        tiles: netTiles.length,
        tileBytes: netTiles.reduce((sum, r) => sum + (r.transferSize || r.encodedBodySize || 0), 0),
        apiBytes: resources
          .filter((r) => r.name.includes('/api/'))
          .reduce((sum, r) => sum + (r.transferSize || r.encodedBodySize || 0), 0),
        totalBytes: resources.reduce((sum, r) => sum + (r.transferSize || r.encodedBodySize || 0), 0),
        top: resources
          .map((r) => ({
            name: r.name.replace(location.origin, ''),
            size: r.transferSize || r.encodedBodySize || 0,
            decoded: r.decodedBodySize || 0,
          }))
          .sort((a, b) => b.size - a.size)
          .slice(0, 8),
      },
      queries: {
        count: queries.length,
        bytes: queries.reduce((sum, q) => sum + q.size, 0),
        detail: queries.sort((a, b) => b.size - a.size),
      },
      sw,
      pending,
      longTasks: {
        count: longTasks.length,
        total: longTasks.reduce((sum, t) => sum + t.duration, 0),
        max: longTasks.reduce((max, t) => Math.max(max, t.duration), 0),
      },
    }
  })
}

/**
 * Track the requests in flight.
 *
 * A screen that is still fetching is not a finished screen: its heap and
 * its DOM are a snapshot mid-flight. Waiting for the network to go quiet
 * — and then to stay quiet — is what makes the reading mean something.
 */
function trackRequests(page) {
  const state = { pending: 0, last: Date.now() }
  const bump = (delta) => {
    state.pending += delta
    state.last = Date.now()
  }
  page.on('request', () => bump(1))
  page.on('requestfinished', () => bump(-1))
  page.on('requestfailed', () => bump(-1))
  return state
}

/** Wait until nothing has been in flight for `quiet` milliseconds. */
async function waitQuiet(page, state, quiet = 1200, timeout = 30_000) {
  const start = Date.now()
  while (Date.now() - start < timeout) {
    await page.waitForTimeout(200)
    if (state.pending <= 0 && Date.now() - state.last > quiet) return true
  }
  return false
}

/** The renderer's own accounting: heap, DOM counters and scripting time. */
async function readRendererState(cdp) {
  await cdp.send('HeapProfiler.enable')
  await cdp.send('HeapProfiler.collectGarbage')
  const { usedSize, totalSize } = await cdp.send('Runtime.getHeapUsage')
  const perf = await cdp.send('Performance.getMetrics')
  const counters = await cdp.send('Memory.getDOMCounters')
  const byName = {}
  for (const metric of perf.metrics) byName[metric.name] = metric.value
  return {
    heap: usedSize,
    heapTotal: totalSize,
    scriptDuration: (byName.ScriptDuration || 0) * 1000,
    taskDuration: (byName.TaskDuration || 0) * 1000,
    layoutDuration: (byName.LayoutDuration || 0) * 1000,
    styleDuration: (byName.RecalcStyleDuration || 0) * 1000,
    nodes: counters.nodes,
    listeners: counters.jsEventListeners,
    documents: counters.documents,
  }
}

/** Open one route in a fresh context and take the full reading. */
async function measureRoute(browser, name, route) {
  // A fresh context per route: the heap then reflects the cost of opening
  // that screen, not everything that was opened before it.
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: SCALE,
  })
  await context.addInitScript(INIT_SCRIPT)
  const page = await context.newPage()

  // The performance domain has to be on before the first byte, or the
  // scripting time is not counted at all.
  const cdp = await context.newCDPSession(page)
  await cdp.send('Performance.enable')
  await cdp.send('HeapProfiler.enable')

  const requests = trackRequests(page)
  const bytes = { total: 0, js: 0, css: 0, other: 0 }
  page.on('response', async (res) => {
    try {
      const headers = res.headers()
      const len = Number(headers['content-length'] || 0)
      bytes.total += len
      const type = headers['content-type'] || ''
      if (type.includes('javascript')) bytes.js += len
      else if (type.includes('css')) bytes.css += len
      else bytes.other += len
    } catch {
      /* a response may be gone by the time we look */
    }
  })

  const started = Date.now()
  await page.goto(BASE + route, { waitUntil: 'networkidle', timeout: 45_000 })
  const loaded = Date.now() - started

  // Let the charts settle — the heap grows as they mount, and the slow
  // endpoints (the fingerprint list recomputes every run) land late.
  await waitQuiet(page, requests)
  await page.waitForTimeout(SETTLE)

  const renderer = await readRendererState(cdp)
  const pageState = await readPageState(page)
  await cdp.detach()

  const result = {
    name,
    route,
    loaded,
    ...renderer,
    dom: pageState.dom,
    images: pageState.images,
    network: pageState.network,
    queries: pageState.queries,
    sw: pageState.sw,
    longTasks: pageState.longTasks,
    pending: pageState.pending,
    bytes,
  }
  await context.close()
  return result
}

/* --------------------------------------------------------------- session */

/**
 * Walk every screen in one context — the session, not the screen.
 *
 * Navigation is in-app (a real link click, falling back to the history
 * API), because a full page load would throw away the very caches whose
 * accumulation we are measuring.
 */
async function measureSession(browser) {
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: SCALE,
  })
  await context.addInitScript(INIT_SCRIPT)
  const page = await context.newPage()

  const cdp = await context.newCDPSession(page)
  await cdp.send('Performance.enable')
  await cdp.send('HeapProfiler.enable')

  const requests = trackRequests(page)
  const bytes = { total: 0, js: 0, css: 0, other: 0 }
  page.on('response', async (res) => {
    try {
      const headers = res.headers()
      const len = Number(headers['content-length'] || 0)
      bytes.total += len
      const type = headers['content-type'] || ''
      if (type.includes('javascript')) bytes.js += len
      else if (type.includes('css')) bytes.css += len
      else bytes.other += len
    } catch {
      /* a response may be gone by the time we look */
    }
  })

  await page.goto(BASE + '/', { waitUntil: 'networkidle', timeout: 45_000 })
  await waitQuiet(page, requests)
  await page.waitForTimeout(SETTLE)

  const steps = []
  for (const [name, route] of ROUTES.slice(1)) {
    await page.evaluate((href) => {
      const link = document.querySelector(`a[href="${href}"]`)
      if (link) {
        link.click()
        return
      }
      window.history.pushState({}, '', href)
      window.dispatchEvent(new PopStateEvent('popstate'))
    }, route)
    await waitQuiet(page, requests)
    await page.waitForTimeout(SETTLE)

    const renderer = await readRendererState(cdp)
    const state = await readPageState(page)
    steps.push({
      name,
      route,
      heap: renderer.heap,
      nodes: renderer.nodes,
      listeners: renderer.listeners,
      scriptDuration: renderer.scriptDuration,
      dom: state.dom.nodes,
      decoded: state.images.decodedBytes,
      tiles: state.images.tileCount,
      queryCount: state.queries.count,
      queryBytes: state.queries.bytes,
      swBytes: state.sw ? state.sw.total : 0,
      longTasks: state.longTasks.count,
      pending: state.pending,
    })
  }

  const finalRenderer = await readRendererState(cdp)
  const finalState = await readPageState(page)

  const result = {
    base: BASE,
    steps,
    final: {
      ...finalRenderer,
      dom: finalState.dom,
      images: finalState.images,
      queries: finalState.queries,
      sw: finalState.sw,
      longTasks: finalState.longTasks,
      network: finalState.network,
    },
    bytes,
  }
  await context.close()
  return result
}

/* ------------------------------------------------------------------ main */

const browser = await chromium.launch()
let report

if (MODE === 'session') {
  const session = await measureSession(browser)
  await browser.close()

  console.log('')
  console.log('  session — every screen, one renderer')
  console.log('  ' + '-'.repeat(100))
  console.log(
    '  step         heap    DOM  nodes  listeners   query cache   decoded   scripting  long tasks',
  )
  for (const step of session.steps) {
    console.log(
      `  ${step.name.padEnd(12)} ${mb(step.heap).padStart(5)}MB ` +
        `${String(step.dom).padStart(5)} ${String(step.nodes).padStart(6)} ` +
        `${String(step.listeners).padStart(9)}   ` +
        `${String(step.queryCount).padStart(3)}q ${kb(step.queryBytes).padStart(6)}KB ` +
        `${kb(step.decoded).padStart(7)}KB ${ms(step.scriptDuration).padStart(9)} ` +
        `${String(step.longTasks).padStart(9)}` +
        `${step.pending ? `  (${step.pending} still loading)` : ''}`,
    )
  }
  const f = session.final
  console.log('')
  console.log(`  final JS heap        ${mb(f.heap)} MB`)
  console.log(`  final DOM nodes      ${f.dom.nodes}`)
  console.log(`  event listeners      ${f.listeners}`)
  console.log(`  query cache          ${f.queries.count} queries, ${kb(f.queries.bytes)} KB of data`)
  console.log(`  decoded images       ${kb(f.images.decodedBytes)} KB (${f.images.count} images, ${f.images.tileCount} tiles)`)
  console.log(`  service worker cache ${f.sw ? kb(f.sw.total) + ' KB' : 'none'}`)
  console.log(`  scripting            ${Math.round(f.scriptDuration)} ms`)
  console.log('')
  console.log('  largest cached queries:')
  for (const q of f.queries.detail.slice(0, 10)) {
    console.log(`    ${kb(q.size).padStart(7)}KB  ${q.key}`)
  }
  report = session
} else {
  const rows = []
  for (const [name, route] of ROUTES) {
    rows.push(await measureRoute(browser, name, route))
  }
  await browser.close()

  console.log('')
  console.log('  route        heap    DOM   nodes   decoded   queries      API   scripting  long tasks')
  console.log('  ' + '-'.repeat(94))
  for (const r of rows) {
    console.log(
      `  ${r.name.padEnd(12)} ${mb(r.heap).padStart(5)}MB ${String(r.dom.nodes).padStart(5)} ` +
        `${String(r.nodes).padStart(6)} ${kb(r.images.decodedBytes).padStart(7)}KB ` +
        `${String(r.queries.count).padStart(3)}q ${kb(r.queries.bytes).padStart(6)}KB ` +
        `${kb(r.network.apiBytes).padStart(6)}KB ${ms(r.scriptDuration).padStart(8)} ` +
        `${String(r.longTasks.count).padStart(5)}` +
        `${r.pending ? `  (${r.pending} still loading)` : ''}`,
    )
  }
  const worst = rows.reduce((a, b) => (a.heap > b.heap ? a : b))
  const peakTransfer = rows.reduce((a, b) => (a.bytes.total > b.bytes.total ? a : b))
  console.log('')
  console.log(`  heaviest screen: ${worst.name} — ${mb(worst.heap)} MB of JS heap`)
  console.log(`  largest transfer: ${peakTransfer.name} — ${kb(peakTransfer.bytes.total)} KB`)
  console.log(`  total JS heap across all ten screens: ${mb(rows.reduce((s, r) => s + r.heap, 0))} MB`)
  console.log('')
  report = { base: BASE, routes: rows }
}

if (JSON_OUT) {
  const { writeFileSync } = await import('node:fs')
  writeFileSync(JSON_OUT, JSON.stringify(report, null, 2))
  console.log(`  wrote ${JSON_OUT}`)
  console.log('')
}
