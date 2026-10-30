#!/usr/bin/env node
/**
 * Render the collage.
 *
 *   node scripts/render_collage.mjs [html] [outDir]
 *
 * The collage is an HTML sheet; the shareable images are that sheet
 * rendered by a browser. It writes three:
 *
 *   stride-collage.png     2× — 3520 px wide, for a deck
 *   stride-collage@1x.png  1× — for chat, email or a slide
 *   stride-collage.jpg     progressive JPEG, for anywhere that resists a
 *                           large PNG
 *
 * The font is loaded from the network, so the render waits for it: a sheet
 * in the fallback font is not the sheet.
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..')
const HTML = path.resolve(process.argv[2] || path.join(ROOT, 'showcase/collage.html'))
const OUT = path.resolve(process.argv[3] || path.join(ROOT, 'showcase'))

await mkdir(OUT, { recursive: true })

const browser = await chromium.launch()
const context = await browser.newContext({
  viewport: { width: 1760, height: 1200 },
  deviceScaleFactor: 2,
})
const page = await context.newPage()

await page.goto(`file://${HTML}`, { waitUntil: 'networkidle', timeout: 60_000 })
// The sheet's own height, so the image is the sheet and nothing else.
await page.evaluate(() => document.fonts.ready)
const height = await page.evaluate(() => document.querySelector('.sheet').scrollHeight + 112)
await page.setViewportSize({ width: 1760, height })
await page.waitForTimeout(800)

const sheet = page.locator('.sheet')
await sheet.screenshot({ path: path.join(OUT, 'stride-collage.png') })
console.log(`  ✓ stride-collage.png     3520 × ${height * 2}`)

const context1x = await browser.newContext({
  viewport: { width: 1760, height },
  deviceScaleFactor: 1,
})
const page1x = await context1x.newPage()
await page1x.goto(`file://${HTML}`, { waitUntil: 'networkidle', timeout: 60_000 })
await page1x.evaluate(() => document.fonts.ready)
await page1x.setViewportSize({ width: 1760, height })
await page1x.waitForTimeout(500)
await page1x.locator('.sheet').screenshot({ path: path.join(OUT, 'stride-collage@1x.png') })
console.log(`  ✓ stride-collage@1x.png  1760 × ${height}`)

const jpeg = await page1x.locator('.sheet').screenshot({ type: 'jpeg', quality: 88 })
await writeFile(path.join(OUT, 'stride-collage.jpg'), jpeg)
console.log('  ✓ stride-collage.jpg')

await browser.close()
console.log(`\nCollage in ${OUT}`)
