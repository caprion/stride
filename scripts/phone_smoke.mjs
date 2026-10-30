#!/usr/bin/env node
/**
 * The phone smoke test: the four phone surfaces, at 390 px, on synthetic data.
 *
 * The APK cannot always be booted — an emulator without KVM ANRs, and this
 * host has no device attached — but the phone's surfaces are the web
 * bundle, and the bundle can be rendered exactly as the WebView renders it:
 * a 390 px viewport with `window.Capacitor` present, so `isPhone()` is true
 * and the bottom bar is the phone's four tabs.
 *
 * **The data is synthetic.** The bridge is stubbed with the answers in
 * `phone_smoke_fixtures.mjs`; nothing here reads a database, a Garmin key
 * or a health file.
 *
 *   node scripts/phone_smoke.mjs [--out DIR] [--dist DIR]
 *
 * The checks are the ones the phone IA has to pass:
 *
 *   * the bottom bar reads Today · Runs · Progress · Body;
 *   * Today shows one call, one plan line, the guardrail and the week dots;
 *   * Body says "N of 3 signals in your normal range" and shows no 0-100 score;
 *   * Progress shows fitness, consistency, best runs, race range and milestones;
 *   * nothing clips at 390 px.
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'
import { createServer } from 'node:http'
import { readFile, mkdir, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { answerFor } from './phone_smoke_fixtures.mjs'

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..')

function arg(name, fallback) {
  const index = process.argv.indexOf(`--${name}`)
  return index > -1 && process.argv[index + 1] ? process.argv[index + 1] : fallback
}

const DIST = path.resolve(arg('dist', path.join(ROOT, 'frontend/dist')))
const OUT = path.resolve(arg('out', path.join(ROOT, '.run/phone-smoke')))
const WIDTH = 390
const HEIGHT = 844

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.woff2': 'font/woff2',
  '.webmanifest': 'application/manifest+json',
}

/** Serve the built bundle, with the history fallback a SPA needs. */
function serve(dist) {
  const server = createServer(async (req, res) => {
    const url = new URL(req.url ?? '/', 'http://localhost')
    let file = path.join(dist, decodeURIComponent(url.pathname))
    if (!existsSync(file) || url.pathname === '/') {
      file = path.join(dist, 'index.html')
    }
    try {
      const body = await readFile(file)
      res.writeHead(200, { 'Content-Type': MIME[path.extname(file)] ?? 'application/octet-stream' })
      res.end(body)
    } catch (error) {
      res.writeHead(404)
      res.end('not found')
    }
  })
  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => resolve({ server, port: server.address().port }))
  })
}

/** The surfaces the phone has, and what each one has to show. */
const SURFACES = [
  { name: 'today', route: '/', expect: 'Today' },
  { name: 'runs', route: '/feed', expect: 'Runs' },
  { name: 'progress', route: '/progress', expect: 'Progress' },
  { name: 'body', route: '/physiology', expect: 'Body' },
]

const checks = []
function check(surface, name, ok, detail) {
  checks.push({ surface, name, ok: Boolean(ok), detail: detail ?? '' })
  const mark = ok ? '\x1b[38;5;71mPASS\x1b[0m' : '\x1b[38;5;203mFAIL\x1b[0m'
  console.log(`  ${mark}  ${surface.padEnd(9)} ${name}${detail ? ` — ${detail}` : ''}`)
}

const { server, port } = await serve(DIST)
const base = `http://127.0.0.1:${port}`
await mkdir(OUT, { recursive: true })

const browser = await chromium.launch()
const context = await browser.newContext({
  viewport: { width: WIDTH, height: HEIGHT },
  deviceScaleFactor: 2,
  isMobile: true,
  hasTouch: true,
})

/*
  The phone's shell: Capacitor is present, so the app takes the phone
  branch and the bridge answers from the synthetic fixtures.
*/
await context.addInitScript((serialised) => {
  const fixtures = JSON.parse(serialised)
  window.Capacitor = {
    isNativePlatform: () => true,
    isNative: () => true,
    getPlatform: () => 'android',
    Plugins: {
      StrideApi: {
        apiCall: async ({ method, path, body }) => {
          const clean = String(path).split('?')[0]
          const fixture = fixtures[clean]
          if (!fixture) {
            return {
              json: JSON.stringify({
                ok: false,
                status: 404,
                error: { detail: `no fixture for ${clean}` },
              }),
            }
          }
          return {
            json: JSON.stringify({ ok: true, status: 200, body: fixture }),
          }
        },
        syncNow: async () => ({ queued: true }),
        forgetGarminKey: async () => ({ forgotten: true }),
      },
    },
  }
}, JSON.stringify(await import('./phone_smoke_fixtures.mjs').then((m) => m.FIXTURES)))

const page = await context.newPage()
const consoleErrors = []
page.on('console', (message) => {
  if (message.type() === 'error') consoleErrors.push(message.text())
})
page.on('pageerror', (error) => consoleErrors.push(String(error.message)))

/** The overflow at the phone's width: elements that stick out of it. */
const OVERFLOW = () =>
  page.evaluate((width) => {
    const offenders = []
    for (const element of Array.from(document.querySelectorAll('body *'))) {
      const style = getComputedStyle(element)
      if (style.display === 'none' || style.visibility === 'hidden') continue
      const rect = element.getBoundingClientRect()
      if (rect.width === 0 || rect.height === 0) continue
      if (rect.right > width + 1 || rect.left < -1) {
        // Ignore anything inside a container that scrolls on purpose.
        let parent = element.parentElement
        let scrollable = false
        while (parent && parent !== document.body) {
          const parentStyle = getComputedStyle(parent)
          if (
            parentStyle.overflowX === 'auto' ||
            parentStyle.overflowX === 'scroll' ||
            parentStyle.overflowX === 'hidden'
          ) {
            scrollable = true
            break
          }
          parent = parent.parentElement
        }
        if (scrollable) continue
        offenders.push({
          tag: element.tagName.toLowerCase(),
          className: String(element.className).slice(0, 60),
          left: Math.round(rect.left),
          right: Math.round(rect.right),
          text: (element.textContent ?? '').trim().slice(0, 60),
        })
      }
    }
    return {
      documentWidth: document.documentElement.scrollWidth,
      bodyWidth: document.body.scrollWidth,
      viewport: window.innerWidth,
      offenders: offenders.slice(0, 12),
    }
  }, WIDTH)

for (const surface of SURFACES) {
  console.log(`\n\x1b[1m${surface.name}\x1b[0m (${surface.route})`)
  await page.goto(`${base}${surface.route}`, { waitUntil: 'networkidle', timeout: 45_000 })
  await page.waitForTimeout(1_200)

  const shot = path.join(OUT, `phone-${surface.name}.png`)
  await page.screenshot({ path: shot, fullPage: true })
  // The screen as the phone shows it: the bottom bar is fixed, so a
  // full-page capture puts it in the middle of the picture.
  await page.screenshot({ path: path.join(OUT, `phone-${surface.name}-top.png`) })

  const text = await page.evaluate(() => document.body.innerText)

  // ---------------------------------------------------- the bottom bar
  // Only the bottom bar: the desktop rail is in the DOM on the phone too,
  // and hidden.
  const tabs = await page.evaluate(() =>
    Array.from(document.querySelectorAll('nav.fixed a')).map((a) => a.innerText.trim()),
  )
  const wanted = ['Today', 'Runs', 'Progress', 'Body']
  check(
    surface.name,
    'the bottom bar reads Today · Runs · Progress · Body',
    JSON.stringify(tabs) === JSON.stringify(wanted),
    tabs.length ? tabs.join(' · ') : 'no tabs found',
  )

  // --------------------------------------------------------- the content
  if (surface.name === 'today') {
    const headlines = await page.evaluate(() =>
      Array.from(document.querySelectorAll('p')).filter((p) =>
        /^(Run|Today|Done|Take it easy|Comeback)/.test(p.innerText.trim()),
      ).map((p) => p.innerText.trim()),
    )
    check('today', 'exactly one call', headlines.length === 1, headlines.join(' | ') || 'none')

    const planLines = await page.evaluate(() =>
      Array.from(document.querySelectorAll('p'))
        .map((p) => p.innerText.trim())
        .filter((t) => t.startsWith('Your plan says')),
    )
    check(
      'today',
      'one plan line when a plan session exists',
      planLines.length === 1,
      planLines.join(' | ') || 'no plan line',
    )

    const guardrails = await page.evaluate(() =>
      Array.from(document.querySelectorAll('p'))
        .map((p) => p.innerText.trim())
        .filter((t) => t.startsWith('Your longest run in 30 days')),
    )
    check('today', 'the guardrail copy is there', guardrails.length === 1, guardrails.join(' | ') || 'none')

    const dots = await page.evaluate(() => {
      const strip = Array.from(document.querySelectorAll('div')).find(
        (d) => d.className.includes('border-t') && d.querySelectorAll('span[title]').length >= 7,
      )
      return strip ? strip.querySelectorAll('span[title]').length : 0
    })
    check('today', 'seven week dots', dots === 7, `${dots} dots`)

    // The desktop's plan card is not on the phone: the phone's call carries
    // the plan line, and the card behind it is the desktop's.
    check(
      'today',
      "the desktop's plan card is not on the phone",
      !text.includes('On your plan'),
      '',
    )
  }

  if (surface.name === 'runs') {
    // The feed lists the runs, and each one says what it meant.
    const listed = await page.evaluate(() =>
      Array.from(document.querySelectorAll('a[href^="/activities/"]')).map((a) =>
        a.innerText.trim().split('\n')[0],
      ),
    )
    check(
      'runs',
      'lists the runs',
      listed.length >= 3,
      `${listed.length} run(s): ${listed.slice(0, 3).join(', ')}`,
    )
    check(
      'runs',
      'says what a run meant',
      text.includes('Longest in 30 days'),
      'the chip is on the card',
    )
  }

  if (surface.name === 'body') {
    const headline = await page.evaluate(() => {
      const strong = document.querySelector('strong')
      return strong ? strong.innerText.trim() : ''
    })
    check(
      'body',
      'says "N of 3 signals in your normal range"',
      /^\d of 3 signals in your normal range$/.test(headline),
      headline || 'no headline',
    )
    // The readiness score is 74 in the fixture. It may not be shown as a
    // number on the phone: the headline is the athlete's normal.
    const showsScore = /(^|\D)74(\D|$)/.test(text)
    check('body', 'no 0-100 readiness score on the phone', !showsScore, showsScore ? '74 is on the page' : '')
  }

  if (surface.name === 'progress') {
    const wanted2 = ['Fitness', 'Consistency', 'Best runs', 'Race range today', 'Milestones']
    // The section labels are set in small caps, so the rendered text is
    // upper case: the comparison is case-insensitive.
    const lower = text.toLowerCase()
    for (const section of wanted2) {
      const found = lower.includes(section.toLowerCase())
      check('progress', `shows ${section}`, found, found ? '' : 'missing')
    }
  }

  // ------------------------------------------------------------- overflow
  const overflow = await OVERFLOW()
  check(
    surface.name,
    'nothing clips at 390px',
    overflow.documentWidth <= WIDTH + 1 && overflow.offenders.length === 0,
    overflow.documentWidth > WIDTH + 1
      ? `the document is ${overflow.documentWidth}px wide`
      : overflow.offenders.length
        ? `${overflow.offenders.length} element(s) stick out`
        : '',
  )
  if (overflow.offenders.length) {
    for (const offender of overflow.offenders) {
      console.log(`        ${offender.tag}.${offender.className} ${offender.left}→${offender.right} "${offender.text}"`)
    }
  }

  /*
    A chart's own labels are inside an SVG, so the element-overflow check
    above cannot see them: an axis label that runs past the chart's own
    box is cut off by the SVG, not by the page.
  */
  const clippedLabels = await page.evaluate(() => {
    const clipped = []
    for (const svg of Array.from(document.querySelectorAll('svg'))) {
      const box = svg.getBoundingClientRect()
      if (box.width === 0) continue
      for (const label of Array.from(svg.querySelectorAll('text'))) {
        const rect = label.getBoundingClientRect()
        if (rect.width === 0) continue
        if (rect.right > box.right + 1) {
          clipped.push({
            text: (label.textContent ?? '').trim(),
            right: Math.round(rect.right),
            limit: Math.round(box.right),
          })
        }
      }
    }
    return clipped.slice(0, 10)
  })
  check(
    surface.name,
    'no chart label is cut off',
    clippedLabels.length === 0,
    clippedLabels.map((c) => `"${c.text}" ${c.right}>${c.limit}`).join(', '),
  )
}

check('app', 'no console errors', consoleErrors.length === 0, consoleErrors.slice(0, 3).join(' | '))

// The phone's cut: a route the phone does not have lands on Today.
await page.goto(`${base}/plan`, { waitUntil: 'networkidle', timeout: 45_000 })
await page.waitForTimeout(900)
const landed = await page.evaluate(() => window.location.pathname)
check('phone', 'a route the phone does not have lands on Today', landed === '/', landed)

await browser.close()
server.close()

await writeFile(
  path.join(OUT, 'report.json'),
  JSON.stringify({ width: WIDTH, checks, consoleErrors }, null, 2),
)

const failed = checks.filter((c) => !c.ok)
console.log(
  `\n${checks.length - failed.length}/${checks.length} checks passed` +
    (failed.length ? `, ${failed.length} failed` : ''),
)
console.log(`screenshots in ${OUT}`)
process.exit(failed.length ? 1 : 0)
