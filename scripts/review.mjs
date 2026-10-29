#!/usr/bin/env node
/**
 * The review: every screen, every host, one place to look.
 *
 * The desktop and the phone are the same app with different surfaces, so a
 * review that only looks at one of them misses half of what was built. This
 * captures every route at a desktop size and at a phone size, and writes a
 * contact sheet that links them all.
 *
 *   node scripts/review.mjs [baseUrl] [outDir]
 *
 * The data is real - the athlete's own history - because a review against
 * invented data tells you nothing about density, legibility or the empty
 * states.
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'

const BASE = process.argv[2] || 'http://127.0.0.1:8000'
const OUT = process.argv[3] || '/home/sumit/stride/showcase/review'

const DESKTOP = { width: 1440, height: 1000 }
const PHONE = { width: 390, height: 844 }

// Every surface the app has, and which of them the phone keeps.
const SURFACES = [
  ['today', '/', 'Today', 'both'],
  ['plan', '/plan', 'Plan', 'both'],
  ['runs', '/feed', 'Runs', 'both'],
  ['body', '/physiology', 'Body', 'both'],
  ['form', '/fitness', 'Form', 'both'],
  ['stats', '/stats', 'Stats', 'both'],
  ['activity', '/activities/162', 'An activity', 'both'],
  ['compare', '/compare', 'Compare', 'desktop'],
  ['calendar', '/training', 'Training calendar', 'desktop'],
  ['settings', '/settings', 'Settings', 'both'],
]

await mkdir(OUT, { recursive: true })

const browser = await chromium.launch()
const problems = []
const captured = []

for (const [host, viewport, label] of [
  ['desktop', DESKTOP, 'Desktop'],
  ['phone', { ...PHONE, isMobile: true, hasTouch: true }, 'Phone'],
]) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 })
  const page = await context.newPage()
  page.on('console', (msg) => {
    if (msg.type() === 'error') problems.push(`[${host}] ${msg.text()}`)
  })
  page.on('pageerror', (err) => problems.push(`[${host}] ${err.message}`))

  for (const [name, route, title, where] of SURFACES) {
    if (where !== 'both' && where !== host) continue
    await page.goto(BASE + route, { waitUntil: 'networkidle', timeout: 45_000 })
    await page.waitForTimeout(900)
    const heading = await page.locator('h1, h2').first().innerText().catch(() => '')
    const file = `${host}-${name}.png`
    await page.screenshot({ path: path.join(OUT, file), fullPage: host === 'phone' })
    captured.push({ host, name, route, title, heading, file })
    console.log(`  ${host.padEnd(8)} ${name.padEnd(10)} ${heading.slice(0, 48)}`)
  }
  await context.close()
}

await browser.close()

const rows = captured
  .map(
    (c) => `      <figure>
        <a href="${c.file}"><img src="${c.file}" alt="${c.title}" loading="lazy" /></a>
        <figcaption><strong>${c.title}</strong><span>${c.host} &middot; ${c.route}</span><span>${c.heading}</span></figcaption>
      </figure>`,
  )
  .join('\n')

const html = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Stride &mdash; the review</title>
    <style>
      :root { color-scheme: light; }
      body { margin: 0; padding: 32px; background: #f4f7fa; color: #0b1b27;
             font: 15px/1.5 ui-sans-serif, system-ui, sans-serif; }
      h1 { font-size: 22px; margin: 0 0 4px; }
      p.lede { margin: 0 0 28px; color: #61788d; }
      .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 20px; }
      figure { margin: 0; background: #fff; border: 1px solid #dee7ee; border-radius: 10px; overflow: hidden; }
      figure img { width: 100%; display: block; background: #fff; }
      figcaption { padding: 10px 14px; border-top: 1px solid #dee7ee; display: grid; gap: 2px; }
      figcaption strong { font-size: 14px; }
      figcaption span { font-size: 12px; color: #61788d; }
      .problems { margin-top: 28px; padding: 14px 18px; border-radius: 10px;
                  background: #fff; border: 1px solid #dee7ee; }
      .problems h2 { font-size: 15px; margin: 0 0 8px; }
      .problems li { font-size: 13px; color: #c0392b; }
    </style>
  </head>
  <body>
    <h1>Stride &mdash; every screen</h1>
    <p class="lede">The desktop and the phone are the same app with different surfaces.
       ${captured.length} screens, captured against the real data.</p>
    <div class="grid">
${rows}
    </div>
    <div class="problems">
      <h2>Console</h2>
      ${problems.length ? `<ul>${problems.map((p) => `<li>${p}</li>`).join('')}</ul>` : '<p>No console errors on any screen.</p>'}
    </div>
  </body>
</html>
`

await writeFile(path.join(OUT, 'index.html'), html)
console.log(`\n  ${captured.length} screens -> ${OUT}/index.html`)
console.log(problems.length ? `  ${problems.length} console problems` : '  no console errors')
