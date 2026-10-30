#!/usr/bin/env node
/**
 * The APK's own WebView, driven over the DevTools protocol.
 *
 * The headless harness (`phone_smoke.mjs`) renders the bundle in desktop
 * Chromium. This one renders **the bundle inside the APK** — the WebView of
 * the installed app, reached through the emulator's DevTools socket — so
 * the screenshots are of the shipped app and not of a browser.
 *
 * It talks CDP directly (playwright's `connectOverCDP` wants browser-context
 * management, which a WebView does not have).
 *
 * The data is synthetic: the bridge is replaced in the page with the
 * answers in `phone_smoke_fixtures.mjs`, exactly as the headless harness
 * does. The APK's own plugin is untouched; the page is simply told to ask
 * the fixtures instead.
 *
 *   adb forward tcp:9222 localabstract:webview_devtools_remote_<pid>
 *   node scripts/apk_smoke_weblive.mjs [--out DIR] [--endpoint http://127.0.0.1:9222]
 */
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { FIXTURES } from './phone_smoke_fixtures.mjs'

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..')

function arg(name, fallback) {
  const index = process.argv.indexOf(`--${name}`)
  return index > -1 && process.argv[index + 1] ? process.argv[index + 1] : fallback
}

const ENDPOINT = arg('endpoint', 'http://127.0.0.1:9222')
const OUT = path.resolve(arg('out', path.join(ROOT, '.run/emulator/webview')))
const WIDTH = 390

const SURFACES = [
  { name: 'today', route: '/' },
  { name: 'runs', route: '/feed' },
  { name: 'progress', route: '/progress' },
  { name: 'body', route: '/physiology' },
]

/** The DevTools page target of the app's WebView. */
async function pageTarget() {
  const response = await fetch(`${ENDPOINT}/json/list`)
  const targets = await response.json()
  const page = targets.find((target) => target.type === 'page')
  if (!page) throw new Error('the WebView has no page target')
  return page
}

/** A very small CDP client. */
function connect(url) {
  const socket = new WebSocket(url)
  const pending = new Map()
  let next = 1
  const events = []
  socket.addEventListener('message', (event) => {
    const message = JSON.parse(event.data)
    if (message.id && pending.has(message.id)) {
      const { resolve, reject } = pending.get(message.id)
      pending.delete(message.id)
      if (message.error) reject(new Error(JSON.stringify(message.error)))
      else resolve(message.result)
    } else if (message.method) {
      events.push(message)
    }
  })
  const ready = new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve)
    socket.addEventListener('error', reject)
  })
  return {
    ready,
    events,
    send(method, params = {}) {
      const id = next++
      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject })
        socket.send(JSON.stringify({ id, method, params }))
      })
    },
    close() {
      socket.close()
    },
  }
}

await mkdir(OUT, { recursive: true })

const target = await pageTarget()
const client = connect(target.webSocketDebuggerUrl)
await client.ready

await client.send('Page.enable')
await client.send('Runtime.enable')

/*
  The bridge, replaced with the synthetic answers.

  It has to happen *after* the native bridge has injected itself: Capacitor
  puts the real plugin on `window.Capacitor` when the document starts, so
  an init script would be overwritten. The plugin's own method is replaced
  instead, and the app's query cache is invalidated so the screens ask
  again — the app exposes the cache for the measurement harness
  (`window.__strideQueryClient`).
*/
const patch = `
(() => {
  const fixtures = ${JSON.stringify(FIXTURES)};
  const answer = (path) => {
    const clean = String(path).split('?')[0];
    const body = fixtures[clean];
    return body === undefined
      ? { ok: false, status: 404, error: { detail: 'no fixture for ' + clean } }
      : { ok: true, status: 200, body };
  };
  const plugin = window.Capacitor && window.Capacitor.Plugins && window.Capacitor.Plugins.StrideApi;
  if (!plugin) return 'no plugin';
  plugin.apiCall = async ({ path }) => ({ json: JSON.stringify(answer(path)) });
  plugin.syncNow = async () => ({ queued: true });
  plugin.forgetGarminKey = async () => ({ forgotten: true });
  if (window.__strideQueryClient) window.__strideQueryClient.invalidateQueries();
  return 'patched';
})();
`

const evaluate = async (expression) => {
  const result = await client.send('Runtime.evaluate', {
    expression,
    returnByValue: true,
    awaitPromise: true,
  })
  if (result.exceptionDetails) {
    throw new Error(JSON.stringify(result.exceptionDetails))
  }
  return result.result.value
}

const navigate = async (url) => {
  await client.send('Page.navigate', { url })
  await new Promise((resolve) => setTimeout(resolve, 2500))
}

/** Wait until the page has the bottom bar, so the checks do not race it. */
const waitForTabs = async () => {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const ready = await evaluate(
      "document.querySelectorAll('nav.fixed a').length === 4 ? 'yes' : 'no'",
    )
    if (ready === 'yes') return true
    await new Promise((resolve) => setTimeout(resolve, 500))
  }
  return false
}

const checks = []
function check(name, ok, detail) {
  checks.push({ name, ok: Boolean(ok), detail: detail ?? '' })
  const mark = ok ? '\x1b[38;5;71mPASS\x1b[0m' : '\x1b[38;5;203mFAIL\x1b[0m'
  console.log(`  ${mark}  ${name}${detail ? ` — ${detail}` : ''}`)
}

const screenshot = async (file) => {
  const metrics = await client.send('Page.getLayoutMetrics')
  const size = metrics.cssContentSize ?? metrics.contentSize
  const shot = await client.send('Page.captureScreenshot', {
    format: 'png',
    captureBeyondViewport: true,
    clip: { x: 0, y: 0, width: size.width, height: size.height, scale: 1 },
  })
  await writeFile(file, Buffer.from(shot.data, 'base64'))
}

/**
 * The screen, as the phone shows it.
 *
 * The full-page capture repeats the fixed header and the fixed bottom bar
 * (Chromium draws the fixed elements against the expanded viewport), so the
 * phone's own screen is captured separately: one viewport, no stitching.
 */
const screen = async (file) => {
  const shot = await client.send('Page.captureScreenshot', { format: 'png' })
  await writeFile(file, Buffer.from(shot.data, 'base64'))
}

console.log("\n\x1b[1mthe APK's own WebView\x1b[0m")
await navigate(target.url)

const width = await evaluate('window.innerWidth')
check('the viewport is 390px', Math.abs(width - WIDTH) <= 1, `${width}px`)

const agent = await evaluate('navigator.userAgent')
check('it is the phone', /Mobile|Android/.test(agent), agent.slice(0, 60))

for (const surface of SURFACES) {
  await navigate(`https://localhost${surface.route}`)
  await waitForTabs()
  const patched = await evaluate(patch)
  await new Promise((resolve) => setTimeout(resolve, 2000))
  await screenshot(path.join(OUT, `apk-${surface.name}.png`))
  await screen(path.join(OUT, `apk-${surface.name}-screen.png`))
  const text = await evaluate('document.body.innerText')
  const tabs = await evaluate(
    "Array.from(document.querySelectorAll('nav.fixed a')).map((a) => a.innerText.trim())",
  )
  check(
    `${surface.name}: the bottom bar reads Today · Runs · Progress · Body`,
    JSON.stringify(tabs) === JSON.stringify(['Today', 'Runs', 'Progress', 'Body']),
    tabs.join(' · '),
  )
  if (surface.name === 'today') {
    check('today: one call', /km/.test(text), '')
    check('today: the guardrail is there', text.includes('Your longest run in 30 days'), '')
    check('today: the plan line is there', text.includes('Your plan says'), '')
  }
  if (surface.name === 'body') {
    check(
      'body: "N of 3 signals in your normal range"',
      /\d of 3 signals in your normal range/.test(text),
      '',
    )
    check('body: no 0-100 score', !/(^|\D)74(\D|$)/.test(text), '')
  }
  if (surface.name === 'progress') {
    const lower = text.toLowerCase()
    for (const section of ['fitness', 'consistency', 'best runs', 'race range', 'milestones']) {
      check(`progress: shows ${section}`, lower.includes(section), '')
    }
  }
}

const errors = client.events
  .filter((event) => event.method === 'Runtime.consoleAPICalled' && event.params.type === 'error')
  .map((event) => event.params.args.map((arg) => arg.value ?? arg.description).join(' '))
check('no console errors', errors.length === 0, errors.slice(0, 3).join(' | '))

client.close()
await writeFile(path.join(OUT, 'report.json'), JSON.stringify({ width, checks, errors }, null, 2))

const failed = checks.filter((c) => !c.ok)
console.log(`\n${checks.length - failed.length}/${checks.length} checks passed`)
console.log(`screenshots in ${OUT}`)
process.exit(failed.length ? 1 : 0)
