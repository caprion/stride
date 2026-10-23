#!/usr/bin/env node
/**
 * Visual smoke test — renders every page of Stride in headless Chromium and
 * writes a PNG per route plus a console-error report.
 *
 *   node scripts/screenshot.mjs [baseUrl] [outDir]
 *
 * Requires the app to be running (./scripts/run.sh) and playwright's chromium
 * to be installed (npx playwright install chromium).
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'
import { mkdir } from 'node:fs/promises'
import path from 'node:path'

const BASE = process.argv[2] || 'http://127.0.0.1:8000'
const OUT = process.argv[3] || '/home/sumit/stride/.run/screenshots'

const ROUTES = [
  ['dashboard', '/'],
  ['feed', '/feed'],
  ['training', '/training'],
  ['stats', '/stats'],
  ['settings', '/settings'],
  ['activity', '/activities/161'],
]

const VIEWPORT = { width: 1440, height: 1000 }

await mkdir(OUT, { recursive: true })

const browser = await chromium.launch()
const context = await browser.newContext({ viewport: VIEWPORT, deviceScaleFactor: 1 })
const page = await context.newPage()

const problems = []
page.on('console', (msg) => {
  if (msg.type() === 'error') problems.push(`[console] ${msg.text()}`)
})
page.on('pageerror', (err) => problems.push(`[pageerror] ${err.message}`))
page.on('requestfailed', (req) => {
  const url = req.url()
  // Tile servers are best-effort; a failed tile is not an app bug.
  if (url.includes('tile.openstreetmap.org')) return
  problems.push(`[requestfailed] ${url} — ${req.failure()?.errorText}`)
})

for (const [name, route] of ROUTES) {
  const url = `${BASE}${route}`
  try {
    await page.goto(url, { waitUntil: 'networkidle', timeout: 45_000 })
  } catch (err) {
    console.log(`✗ ${name.padEnd(10)} ${route} — navigation failed: ${err.message.split('\n')[0]}`)
    problems.push(`[nav] ${route}: ${err.message.split('\n')[0]}`)
    continue
  }
  await page.waitForTimeout(1200)
  const file = path.join(OUT, `${name}.png`)
  await page.screenshot({ path: file, fullPage: true })
  const title = await page.title()
  const h1 = await page.locator('h1').first().textContent().catch(() => null)
  console.log(`✓ ${name.padEnd(10)} ${route}  →  ${file}  |  h1="${(h1 ?? '').trim().slice(0, 48)}"`)
}

// Mobile pass on the two most important surfaces.
const mobile = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 1 })
const mpage = await mobile.newPage()
for (const [name, route] of [['dashboard-mobile', '/'], ['feed-mobile', '/feed']]) {
  try {
    await mpage.goto(`${BASE}${route}`, { waitUntil: 'networkidle', timeout: 45_000 })
    await mpage.waitForTimeout(900)
    await mpage.screenshot({ path: path.join(OUT, `${name}.png`), fullPage: true })
    console.log(`✓ ${name.padEnd(10)} ${route}`)
  } catch (err) {
    problems.push(`[nav] ${route}: ${err.message.split('\n')[0]}`)
  }
}

await browser.close()

if (problems.length) {
  console.log(`\n${problems.length} problem(s):`)
  for (const p of [...new Set(problems)]) console.log('  ' + p)
  process.exitCode = 1
} else {
  console.log('\nNo console errors, page errors or failed requests.')
}
