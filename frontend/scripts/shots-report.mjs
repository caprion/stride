#!/usr/bin/env node
/**
 * The screenshots for the report: the shell and one content page, at the
 * desktop width and at the phone width.
 *
 *   node scripts/shots-report.mjs <outDir> [port]
 *
 * A development aid, not a gate.
 */

import { spawn, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright-core'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const out = process.argv[2] ?? '/tmp/stride-shots'
const port = Number(process.argv[3] ?? 4188)
const base = `http://127.0.0.1:${port}`

const PATHS = [
  ['/', 'today'],
  ['/runs', 'runs'],
]

const VIEWPORTS = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
]

function ensureBuild() {
  const result = spawnSync('npx', ['vite', 'build'], { cwd: root, stdio: 'inherit' })
  if (result.status !== 0) process.exit(1)
}

async function startServer() {
  const vite = join(root, 'node_modules/.bin/vite')
  const child = spawn(vite, ['preview', '--port', String(port), '--strictPort', '--host', '127.0.0.1'], {
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
  for (const viewport of VIEWPORTS) {
    const context = await browser.newContext({
      viewport: { width: viewport.width, height: viewport.height },
      deviceScaleFactor: 1,
    })
    const page = await context.newPage()
    for (const [path, name] of PATHS) {
      await page.goto(`${base}${path}?fixtures=route`, { waitUntil: 'networkidle' })
      await page.waitForTimeout(300)
      await page.screenshot({
        path: join(out, `${name}-${viewport.name}.png`),
        fullPage: true,
      })
      console.log(`${name}-${viewport.name}.png`)
    }
    await context.close()
  }
} finally {
  await browser.close()
  server.stop()
}
