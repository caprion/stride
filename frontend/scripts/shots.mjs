#!/usr/bin/env node
/**
 * A look at the screens.
 *
 * A development aid, not a gate: it writes a screenshot of each screen at
 * a width, so a change can be looked at rather than imagined.
 *
 *   node scripts/shots.mjs [width] [fixture]
 */

import { spawn, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright-core'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const port = 4174
const base = `http://127.0.0.1:${port}`

const width = Number(process.argv[2] ?? 390)
const fixture = process.argv[3] ?? '1'
const out = join(root, 'shots')

const PATHS = [
  ['/', 'today'],
  ['/runs', 'runs'],
  ['/runs/101', 'activity'],
  ['/progress', 'progress'],
  ['/body', 'body'],
  ['/plan', 'plan'],
  ['/compare/101', 'compare'],
  ['/settings', 'settings'],
  ['/connect', 'connect'],
]

function ensureBuild() {
  const result = spawnSync('npx', ['vite', 'build'], { cwd: root, stdio: 'inherit' })
  if (result.status !== 0) process.exit(1)
}

async function startServer() {
  const vite = join(root, 'node_modules/.bin/vite')
  const child = spawn(vite, ['preview', '--port', String(port), '--strictPort'], {
    cwd: root,
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: true,
  })
  child.stdout.on('data', () => {})
  child.stderr.on('data', (chunk) => process.stderr.write(chunk))
  const stop = () => {
    try {
      process.kill(-child.pid, 'SIGTERM')
    } catch {
      child.kill('SIGTERM')
    }
  }
  for (let attempt = 0; attempt < 100; attempt += 1) {
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

ensureBuild()
if (!existsSync(out)) mkdirSync(out, { recursive: true })
const server = await startServer()
const browser = await chromium.launch()
try {
  const context = await browser.newContext({
    viewport: { width, height: 900 },
    deviceScaleFactor: 1,
  })
  const page = await context.newPage()
  for (const [path, name] of PATHS) {
    const variant = name === 'connect' ? 'fresh' : fixture
    await page.goto(`${base}${path}?fixtures=${variant}`, { waitUntil: 'networkidle' })
    await page.waitForTimeout(200)
    await page.screenshot({ path: join(out, `${name}-${width}.png`), fullPage: true })
    console.log(`${name}-${width}.png`)
  }
  await context.close()
} finally {
  await browser.close()
  server.stop()
}
