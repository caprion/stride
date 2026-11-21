#!/usr/bin/env node
/**
 * The wall, at the two widths that matter: the desktop's 1440 and the
 * phone's 390. A development aid, not a gate.
 *
 *   node scripts/wall-shots.mjs
 */

import { spawn, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright-core'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const port = Number(process.env.STRIDE_SHOTS_PORT ?? 4175)
const base = `http://127.0.0.1:${port}`
const out = process.env.STRIDE_SHOTS_OUT ?? '/tmp/stride-shots'

const WIDTHS = [
  { width: 1440, height: 3600, name: 'desktop-1440' },
  { width: 390, height: 844, name: 'phone-390' },
]

function ensureBuild() {
  const result = spawnSync('npx', ['vite', 'build'], { cwd: root, stdio: 'inherit' })
  if (result.status !== 0) process.exit(1)
}

async function startServer() {
  const vite = join(root, 'node_modules/.bin/vite')
  const child = spawn(
    vite,
    ['preview', '--port', String(port), '--strictPort', '--host', '127.0.0.1'],
    { cwd: root, stdio: ['ignore', 'pipe', 'pipe'], detached: true },
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

ensureBuild()
if (!existsSync(out)) mkdirSync(out, { recursive: true })
const server = await startServer()
const browser = await chromium.launch()
try {
  for (const size of WIDTHS) {
    const context = await browser.newContext({
      viewport: { width: size.width, height: size.height },
      deviceScaleFactor: 1,
    })
    const page = await context.newPage()
    await page.goto(`${base}/progress?fixtures=1`, { waitUntil: 'networkidle' })
    await page.waitForTimeout(400)
    await page.screenshot({
      path: join(out, `progress-wall-${size.name}.png`),
      fullPage: true,
    })
    console.log(`progress-wall-${size.name}.png`)
    await context.close()
  }
} finally {
  await browser.close()
  server.stop()
}
