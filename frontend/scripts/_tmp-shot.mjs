#!/usr/bin/env node
/**
 * A one-off: photograph the **running rig**, with the athlete's real data.
 *
 * The repo's own `scripts/shots.mjs` photographs the fixtures; this points
 * the same browser at the live rig on :8181 instead, so what it captures is
 * what the desktop actually shows.
 *
 *   node scripts/shot-live.mjs [width] [path] [name]
 */

import { existsSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright-core'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const base = 'https://cf-openclaw.tail62fa14.ts.net:8181'

const width = Number(process.argv[2] ?? 1280)
const path = process.argv[3] ?? '/progress'
const name = process.argv[4] ?? 'live'
const out = '/tmp/stride-shots'

if (!existsSync(out)) mkdirSync(out, { recursive: true })

const browser = await chromium.launch()
try {
  const context = await browser.newContext({
    viewport: { width, height: 1000 },
    deviceScaleFactor: 1,
  })
  const page = await context.newPage()
  await page.goto(`${base}${path}`, { waitUntil: 'networkidle' })
  await page.waitForTimeout(600)
  const file = join(out, `${name}-${width}.png`)
  await page.screenshot({ path: file, fullPage: true })
  console.log(file)
  await context.close()
} finally {
  await browser.close()
}
