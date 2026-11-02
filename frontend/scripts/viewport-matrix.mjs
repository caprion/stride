#!/usr/bin/env node
/**
 * The viewport matrix.
 *
 * Six widths, every screen, on a real browser: no horizontal scroll, no
 * clipped labels, tap targets of at least 44 px at compact, the same four
 * destinations everywhere, and the screen's primary action reachable
 * without a horizontal scroll.
 *
 * The browser is real, because jsdom has no layout engine and "no
 * horizontal scroll" is a layout claim.
 *
 *   node scripts/viewport-matrix.mjs
 */

import { spawn, spawnSync } from 'node:child_process'
import { existsSync, readdirSync, statSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright-core'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const port = 4173
const base = `http://127.0.0.1:${port}`

/** The six widths the gate runs at. */
const VIEWPORTS = [
  { name: '360x640 small phone', width: 360, height: 640 },
  { name: '390x844 phone', width: 390, height: 844 },
  { name: '430x932 large phone', width: 430, height: 932 },
  { name: '768x1024 tablet', width: 768, height: 1024 },
  { name: '1024x768 landscape', width: 1024, height: 768 },
  { name: '1280x800 desktop', width: 1280, height: 800 },
]

/** The screens the gate runs at. */
const ROUTES = [
  { path: '/', name: 'Today', primary: '[data-testid="today-session"]' },
  { path: '/runs', name: 'Runs', primary: '[data-testid="run-101"]' },
  { path: '/runs/101', name: 'Activity', primary: '[data-testid="activity-detail"]' },
  { path: '/progress', name: 'Progress', primary: '[data-testid="progress-headline"]' },
  { path: '/body', name: 'Body', primary: '[data-testid="body-signals"]' },
  { path: '/plan', name: 'Plan week', primary: '[data-testid="plan-week"]' },
  { path: '/compare/101', name: 'Compare', primary: '[data-testid="compare-metrics"]' },
]

/** The four destinations — the set that must never change. */
const DESTINATIONS = ['/', '/runs', '/progress', '/body']

// ------------------------------------------------------------------ build

function newestSourceMtime(dir) {
  let newest = 0
  for (const name of readdirSync(dir)) {
    const path = join(dir, name)
    const info = statSync(path)
    if (info.isDirectory()) newest = Math.max(newest, newestSourceMtime(path))
    else newest = Math.max(newest, info.mtimeMs)
  }
  return newest
}

function ensureBuild() {
  const dist = join(root, 'dist/index.html')
  const sources = Math.max(
    newestSourceMtime(join(root, 'src')),
    statSync(join(root, 'index.html')).mtimeMs,
  )
  if (existsSync(dist) && statSync(dist).mtimeMs >= sources) return
  console.log('building the bundle…')
  const result = spawnSync('npx', ['vite', 'build'], {
    cwd: root,
    stdio: 'inherit',
    env: process.env,
  })
  if (result.status !== 0) {
    console.error('the bundle did not build')
    process.exit(1)
  }
}

// ----------------------------------------------------------------- server

async function startServer() {
  const vite = join(root, 'node_modules/.bin/vite')
  const child = spawn(
    vite,
    ['preview', '--port', String(port), '--strictPort', '--host', '127.0.0.1'],
    {
      cwd: root,
      stdio: ['ignore', 'pipe', 'pipe'],
      detached: true,
    },
  )
  child.stdout.on('data', () => {})
  child.stderr.on('data', (chunk) => process.stderr.write(chunk))

  const stop = () => {
    try {
      process.kill(-child.pid, 'SIGTERM')
    } catch {
      child.kill('SIGTERM')
    }
  }

  // Twenty seconds is enough on a warm machine and not enough on a cold CI
  // runner, where the preview server is starting for the first time.
  for (let attempt = 0; attempt < 300; attempt += 1) {
    try {
      const response = await fetch(base)
      if (response.ok) return { stop }
    } catch {
      /* not up yet */
    }
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
  stop()
  throw new Error('the preview server did not start')
}

// ------------------------------------------------------------------ gates

/**
 * The checks, run inside the page.
 *
 * They are written as one function so the browser does exactly one round
 * trip per screen.
 */
const IN_PAGE_CHECKS = `
() => {
  const problems = []
  const doc = document.documentElement

  // 1. No horizontal page scroll.
  if (doc.scrollWidth > doc.clientWidth + 1) {
    problems.push('the page scrolls sideways: ' + doc.scrollWidth + ' > ' + doc.clientWidth)
  }

  // 2. No horizontal scroll inside the content column either.
  const main = document.querySelector('.shell__main')
  if (main && main.scrollWidth > main.clientWidth + 1) {
    problems.push('the content scrolls sideways: ' + main.scrollWidth + ' > ' + main.clientWidth)
  }

  const visible = (element) => {
    const style = getComputedStyle(element)
    if (style.display === 'none' || style.visibility === 'hidden') return false
    const box = element.getBoundingClientRect()
    return box.width > 0 && box.height > 0
  }

  // 3. Nothing sticks out past the right edge of the window.
  for (const element of document.querySelectorAll('body *')) {
    if (!visible(element)) continue
    const box = element.getBoundingClientRect()
    if (box.right > window.innerWidth + 1) {
      problems.push('an element sticks out past the right edge: ' + element.tagName + '.' + element.className)
      break
    }
  }

  // 4. No clipped labels: an element that hides its overflow and still
  //    has more text than fits is a truncated label.
  for (const element of document.querySelectorAll('body *')) {
    if (!visible(element)) continue
    if (element.children.length > 0) continue
    const style = getComputedStyle(element)
    if (style.overflowX !== 'hidden' && style.textOverflow !== 'ellipsis') continue
    if (element.scrollWidth > element.clientWidth + 1) {
      problems.push('a label is clipped: "' + element.textContent.trim().slice(0, 40) + '"')
    }
  }

  // 5. Tap targets.
  const compact = window.innerWidth < 600
  for (const element of document.querySelectorAll('.tap, button, nav a')) {
    if (!visible(element)) continue
    const box = element.getBoundingClientRect()
    const minimum = compact ? 44 : 32
    if (box.height < minimum - 0.5 || box.width < minimum - 0.5) {
      problems.push('a tap target is smaller than ' + minimum + 'px: ' + Math.round(box.width) + 'x' + Math.round(box.height))
    }
  }

  // 6. The destination set.
  const destinations = [...document.querySelectorAll('[data-destination]')]
    .filter(visible)
    .map((element) => element.getAttribute('data-destination'))
  const unique = [...new Set(destinations)].sort()

  return { problems, destinations, unique }
}
`

// ------------------------------------------------------------------- main

async function main() {
  ensureBuild()
  const server = await startServer()

  const executable = chromium.executablePath()
  if (!existsSync(executable)) {
    server.stop()
    console.error(
      'no chromium for playwright-core at ' +
        executable +
        '\ninstall it with: npx playwright install chromium',
    )
    process.exit(1)
  }

  const browser = await chromium.launch()
  const failures = []
  const seenDestinations = new Map()
  let checks = 0

  try {
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        deviceScaleFactor: 1,
      })
      const page = await context.newPage()
      page.on('pageerror', (error) =>
        failures.push(`${viewport.name}: the page threw ${error.message}`),
      )

      for (const route of ROUTES) {
        const url = `${base}${route.path}?fixtures=1`
        await page.goto(url, { waitUntil: 'networkidle' })
        await page.waitForSelector('.shell', { state: 'attached', timeout: 10_000 })
        await page.waitForTimeout(150)

        const result = await page.evaluate(`(${IN_PAGE_CHECKS})()`)
        checks += 1

        const label = `${viewport.name} · ${route.name}`
        for (const problem of result.problems) {
          failures.push(`${label}: ${problem}`)
        }

        // The destination set is identical at every width.
        if (result.unique.join(',') !== DESTINATIONS.slice().sort().join(',')) {
          failures.push(
            `${label}: the destination set is [${result.unique.join(', ')}], expected [${DESTINATIONS.join(', ')}]`,
          )
        }
        seenDestinations.set(viewport.name, result.unique.join(','))

        // The primary action is reachable without a horizontal scroll.
        const primary = await page.$(route.primary)
        if (!primary) {
          failures.push(`${label}: the primary action (${route.primary}) is not on the screen`)
        } else {
          const box = await primary.boundingBox()
          if (!box || box.x + box.width > viewport.width + 1) {
            failures.push(`${label}: the primary action is outside the viewport`)
          }
        }
      }

      await context.close()
    }
  } finally {
    await browser.close()
    server.stop()
  }

  const sets = new Set(seenDestinations.values())
  if (sets.size !== 1) {
    failures.push(`the destination set differs between widths: ${[...sets].join(' | ')}`)
  }

  if (failures.length > 0) {
    console.error(`viewport matrix: ${failures.length} failure(s) over ${checks} screens\n`)
    for (const failure of failures) console.error(`  ✗ ${failure}`)
    console.error('')
    process.exit(1)
  }

  console.log(
    `viewport matrix: ${checks} screens over ${VIEWPORTS.length} widths — no sideways scroll, no clipped labels, tap targets, one destination set.`,
  )
}

await main()
