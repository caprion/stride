#!/usr/bin/env node
/**
 * Capture the showcase set.
 *
 *   node scripts/showcase.mjs [baseUrl] [outDir]
 *
 * Produces viewport-sized screenshots of every surface, including the states
 * that need interaction (a comparison with runs selected).
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'
import { mkdir } from 'node:fs/promises'

const BASE = process.argv[2] || 'http://127.0.0.1:8000'
const OUT = process.argv[3] || '/home/sumit/stride/showcase/shots'

await mkdir(OUT, { recursive: true })

const browser = await chromium.launch()
const context = await browser.newContext({
  viewport: { width: 1512, height: 945 },
  deviceScaleFactor: 2,
})
const page = await context.newPage()

const settle = async (ms = 3200) => {
  await page.waitForTimeout(ms)
}

/** Open a route and capture the top of the page. */
async function shot(name, route, { scroll = 0, wait = 3200 } = {}) {
  await page.goto(`${BASE}${route}`, { waitUntil: 'networkidle' })
  await settle(wait)
  if (scroll) {
    await page.evaluate((y) => window.scrollTo(0, y), scroll)
    await page.waitForTimeout(700)
  }
  const file = `${OUT}/${name}.png`
  await page.screenshot({ path: file })
  console.log(`  ✓ ${name.padEnd(14)} ${route}`)
}

console.log('Capturing showcase screenshots...')

await shot('01-dashboard', '/')
await shot('02-plan', '/plan', { scroll: 120 })
await shot('03-activity', '/activities/162')
await shot('04-physiology', '/physiology')
await shot('05-fingerprint', '/fitness')
await shot('06-training', '/training')
await shot('07-feed', '/feed')

// The comparison page needs runs selected before it shows anything.
await page.goto(`${BASE}/compare`, { waitUntil: 'networkidle' })
await settle(2500)
const addButtons = page.locator('button:has-text("Add")')
const count = await addButtons.count()
if (count >= 3) {
  await addButtons.nth(0).click()
  await page.waitForTimeout(600)
  await addButtons.nth(1).click()
  await page.waitForTimeout(600)
  await addButtons.nth(2).click()
}
await settle(3000)
await page.screenshot({ path: `${OUT}/08-compare.png` })
console.log('  ✓ 08-compare      /compare (3 runs selected)')

// The activity page's splits and charts live below the fold.
await page.goto(`${BASE}/activities/162`, { waitUntil: 'networkidle' })
await settle(3000)
await page.evaluate(() => window.scrollTo(0, 1150))
await page.waitForTimeout(900)
await page.screenshot({ path: `${OUT}/09-activity-detail.png` })
console.log('  ✓ 09-activity-detail  /activities/162 (splits)')

await browser.close()
console.log(`\nShowcase shots in ${OUT}`)
