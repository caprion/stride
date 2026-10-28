#!/usr/bin/env node
/**
 * Measure the map: tiles fetched, tiles decoded, and what is left behind.
 *
 * The map is the one part of the app that decodes images at full
 * resolution. A 256x256 tile is 256 KB of pixels once decoded, and the
 * browser keeps a second copy for the GPU — so every tile on screen is
 * half a megabyte of renderer memory that the JS heap never shows.
 *
 * This asks four questions:
 *
 *   1. How many tiles does the map fetch, and how big are they?
 *   2. Does the device pixel ratio inflate the request? (It should not:
 *      Leaflet only asks for retina tiles when told to.)
 *   3. What happens when the athlete zooms in — does the tile layer hold
 *      a ring of tiles the athlete never looks at?
 *   4. When the route unmounts, is the map released?
 *
 *   node scripts/measure-map.mjs [baseUrl]
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'

const BASE = process.argv[2] || 'http://127.0.0.1:8000'
const ACTIVITY = '/activities/162'

const kb = (n) => String(Math.round(n / 1024))

/** Everything about the tiles the page currently holds. */
async function tileState(page) {
  return page.evaluate(() => {
    const tiles = [...document.querySelectorAll('img.leaflet-tile')]
    const decoded = tiles.reduce((sum, t) => sum + (t.naturalWidth || 0) * (t.naturalHeight || 0) * 4, 0)
    const resources = performance.getEntriesByType('resource')
    const tileRequests = resources.filter((r) => r.name.includes('/api/v1/tiles/'))
    return {
      tilesInDom: tiles.length,
      decodedBytes: decoded,
      tileRequests: tileRequests.length,
      tileBytes: tileRequests.reduce((sum, r) => sum + (r.transferSize || 0), 0),
      devicePixelRatio: window.devicePixelRatio,
      mapNodes: document.querySelectorAll('.leaflet-container, .leaflet-pane, .leaflet-tile-pane').length,
    }
  })
}

async function open(browser, scale) {
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: scale,
  })
  const page = await context.newPage()
  await page.goto(BASE + ACTIVITY, { waitUntil: 'networkidle', timeout: 45_000 })
  await page.waitForTimeout(2500)
  return { context, page }
}

const browser = await chromium.launch()

console.log('')
console.log('  map — tiles, decoded pixels, and what is left behind')
console.log('  ' + '-'.repeat(78))

for (const scale of [1, 3]) {
  const { context, page } = await open(browser, scale)
  const initial = await tileState(page)
  console.log(
    `  scale ${scale}x  dpr ${initial.devicePixelRatio}  tiles ${initial.tileRequests} fetched, ` +
      `${initial.tilesInDom} in the DOM, ${kb(initial.decodedBytes)}KB decoded, ` +
      `${kb(initial.tileBytes)}KB transferred`,
  )

  // Zoom in twice with the map's own control: the tile layer keeps a ring
  // of tiles around the view, and that ring is what costs memory.
  const zoomIn = await page.$('.leaflet-control-zoom-in')
  if (zoomIn) {
    for (let step = 0; step < 2; step += 1) {
      await zoomIn.click()
      await page.waitForTimeout(1800)
    }
  }
  const zoomed = await tileState(page)
  console.log(
    `            after 2 zooms: ${zoomed.tileRequests} fetched, ` +
      `${zoomed.tilesInDom} in the DOM, ${kb(zoomed.decodedBytes)}KB decoded`,
  )

  // Leave the route: the map should go with it.
  await page.goto(BASE + '/', { waitUntil: 'networkidle' })
  await page.waitForTimeout(1500)
  const left = await tileState(page)
  const counters = await page.evaluate(() => ({
    leafletNodes: document.querySelectorAll('[class*="leaflet"]').length,
    tileNodes: document.querySelectorAll('img.leaflet-tile').length,
  }))
  console.log(
    `            after leaving: ${left.tileRequests} fetched, ${counters.tileNodes} tiles in the DOM, ` +
      `${counters.leafletNodes} leaflet nodes left`,
  )
  console.log('')
  await context.close()
}

await browser.close()
console.log('')
