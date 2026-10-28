#!/usr/bin/env node
/**
 * Measure the app's runtime footprint.
 *
 * The bundle size is only half the story: what matters on a phone is the
 * heap it holds and the work the CPU does when a screen is opened. This
 * walks every route in a fresh page and records the JS heap, the time to
 * paint, the long tasks and the bytes transferred.
 *
 *   node scripts/measure.mjs [baseUrl]
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'

const BASE = process.argv[2] || 'http://127.0.0.1:8000'

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

const browser = await chromium.launch()
const rows = []

for (const [name, route] of ROUTES) {
  // A fresh context per route: the heap then reflects the cost of opening
  // that screen, not everything that was opened before it.
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } })
  const page = await context.newPage()

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

  // Let the charts settle — the heap grows as they mount.
  await page.waitForTimeout(1200)

  // Chrome's performance.memory is bucketed and never drops back, so it
  // cannot tell one screen from another. Ask the debugger for the real
  // figure instead: collect the garbage, then read the heap.
  const cdp = await context.newCDPSession(page)
  await cdp.send('HeapProfiler.enable')
  await cdp.send('HeapProfiler.collectGarbage')
  const { usedSize, totalSize } = await cdp.send('Runtime.getHeapUsage')
  await cdp.detach()

  const metrics = await page.evaluate(() => {
    const nav = performance.getEntriesByType('navigation')[0]
    const paint = performance.getEntriesByType('paint')
    const longTasks = performance.getEntriesByType('longtask') || []
    return {
      heap: performance.memory?.usedJSHeapSize ?? 0,
      heapLimit: performance.memory?.jsHeapSizeLimit ?? 0,
      domContentLoaded: nav ? Math.round(nav.domContentLoadedEventEnd) : 0,
      load: nav ? Math.round(nav.loadEventEnd) : 0,
      firstPaint: paint.find((p) => p.name === 'first-paint')?.startTime ?? 0,
      longestTask: longTasks.reduce((max, t) => Math.max(max, t.duration), 0),
      longTaskCount: longTasks.length,
      nodes: document.querySelectorAll('*').length,
      listeners: 0,
    }
  })

  rows.push({ name, route, ...metrics, heap: usedSize, bytes, loaded })
  await context.close()
}

await browser.close()

const mb = (n) => (n / 1024 / 1024).toFixed(1)
const kb = (n) => (n / 1024).toFixed(0)
const ms = (n) => `${Math.round(n)}ms`

console.log('')
console.log('  route        heap    DOM   nodes   long tasks   transfer   load')
console.log('  ' + '-'.repeat(72))
for (const r of rows) {
  console.log(
    `  ${r.name.padEnd(12)} ${mb(r.heap).padStart(5)}MB ${String(r.nodes).padStart(5)} ` +
      `${String(r.longTaskCount).padStart(5)}   ${ms(r.longestTask).padStart(8)}   ` +
      `${kb(r.bytes.total).padStart(6)}KB  ${ms(r.loaded).padStart(6)}`,
  )
}
const worst = rows.reduce((a, b) => (a.heap > b.heap ? a : b))
const peakTransfer = rows.reduce((a, b) => (a.bytes.total > b.bytes.total ? a : b))
console.log('')
console.log(`  heaviest screen: ${worst.name} — ${mb(worst.heap)} MB of JS heap`)
console.log(`  largest transfer: ${peakTransfer.name} — ${kb(peakTransfer.bytes.total)} KB`)
console.log(`  total JS heap across all ten screens: ${mb(rows.reduce((s, r) => s + r.heap, 0))} MB`)
console.log('')
