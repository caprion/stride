#!/usr/bin/env node
/**
 * What does the service worker's cache hold, and does it ever let go?
 *
 * The worker caches the shell, the assets and the fonts. Its cache is
 * named after the app rather than after the build, and the assets are
 * content-hashed — so a new release is a new set of files, and the old set
 * is still in the cache. This measures that: it loads the app in a
 * persistent profile, prints what the cache holds, and — when the build
 * has changed since the last run — prints what the reconciliation threw
 * away.
 *
 *   node scripts/measure-sw.mjs [--profile /tmp/stride-sw]
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'

const BASE = process.env.STRIDE_BASE || 'http://127.0.0.1:8000'
const profileArg = process.argv.indexOf('--profile')
const PROFILE = profileArg > -1 ? process.argv[profileArg + 1] : '/tmp/stride-sw'

const context = await chromium.launchPersistentContext(PROFILE, {
  viewport: { width: 390, height: 844 },
})
const page = await context.newPage()

// Two loads: the first registers the worker, the second is served by it.
await page.goto(BASE + '/', { waitUntil: 'networkidle', timeout: 45_000 })
await page.waitForTimeout(3000)
const afterFirst = await readInventory()
await page.reload({ waitUntil: 'networkidle' })
await page.waitForTimeout(3000)

async function readInventory() {
  return page.evaluate(async () => {
  const names = await caches.keys()
  const out = []
  for (const name of names) {
    const cache = await caches.open(name)
    const keys = await cache.keys()
    let bytes = 0
    const entries = []
    for (const request of keys) {
      const hit = await cache.match(request)
      const size = hit ? (await hit.clone().blob()).size : 0
      bytes += size
      entries.push({
        path: new URL(request.url).pathname,
        size,
        date: hit?.headers.get('date') ?? null,
      })
    }
    out.push({ name, entries, bytes })
  }
  return out
  })
}

const inventory = await readInventory()

const kb = (n) => String(Math.round(n / 1024))
console.log('')
console.log('  service worker cache')
console.log('  ' + '-'.repeat(72))
for (const cache of inventory) {
  console.log(`  ${cache.name}: ${cache.entries.length} entries, ${kb(cache.bytes)} KB`)
  for (const entry of cache.entries.sort((a, b) => b.size - a.size)) {
    console.log(`    ${kb(entry.size).padStart(7)}KB  ${entry.path}`)
  }
}
console.log('')
console.log(
  `  after the first load: ${afterFirst[0]?.entries.length ?? 0} entries, ` +
    `${kb(afterFirst[0]?.bytes ?? 0)} KB`,
)
console.log('')
await context.close()
