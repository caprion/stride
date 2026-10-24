#!/usr/bin/env node
/** Screenshot one palette candidate. Usage: node palette_shot.mjs <name> <outDir> */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'
import { mkdir } from 'node:fs/promises'

const name = process.argv[2]
const out = process.argv[3]
await mkdir(out, { recursive: true })

const browser = await chromium.launch()
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: 2 })
const page = await context.newPage()

for (const [label, route, scroll] of [
  ['dashboard', '/', 0],
  ['activity', '/activities/162', 0],
  ['plan', '/plan', 120],
]) {
  await page.goto(`http://127.0.0.1:8000${route}`, { waitUntil: 'networkidle' })
  await page.waitForTimeout(3200)
  if (scroll) {
    await page.evaluate((y) => window.scrollTo(0, y), scroll)
    await page.waitForTimeout(700)
  }
  await page.screenshot({ path: `${out}/${name}-${label}.png` })
}

await browser.close()
console.log(`  captured ${name}`)
