#!/usr/bin/env node
/**
 * The token gate.
 *
 * Three checks, and they are the same three the fixtures' parity test
 * makes:
 *
 *   1. **the generated files are in step with the tokens.** The CSS the
 *      app is painted with and the phone's own chrome are generated from
 *      `tokens.ts`; this regenerates them and compares. A token that
 *      changed and a generated file that did not is a failure;
 *   2. **every colour the app's components use is a token.** A scan of the
 *      frontend's source for raw hex values, naming the offenders. The
 *      token file and the generated CSS are the only places a hex may
 *      live;
 *   3. **the browser's own chrome colour is a token.** The theme colour in
 *      `index.html` must be the default palette's own value.
 *
 *   node scripts/check-tokens.mjs
 */

import { readFileSync, readdirSync, statSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { androidXml, cssText } from './tokens.mjs'
import { DEFAULT_PALETTE, palettes } from '../tokens.ts'

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..')
const frontend = join(root, 'frontend')

const failures = []

function fail(message) {
  failures.push(message)
}

/** Every file under a directory, recursively. */
function filesUnder(directory) {
  const found = []
  for (const name of readdirSync(directory)) {
    const path = join(directory, name)
    if (statSync(path).isDirectory()) found.push(...filesUnder(path))
    else found.push(path)
  }
  return found
}

// ------------------------------------------------- 1 · the generated files

/** The generated files, and what they must say. */
const generated = [
  ['frontend/src/tokens.css', cssText()],
  ['android/app/src/main/res/values/colors.xml', androidXml(palettes[DEFAULT_PALETTE])],
  ['android/app/src/main/res/values-night/colors.xml', androidXml(palettes.night)],
]

for (const [path, expected] of generated) {
  let actual = null
  try {
    actual = readFileSync(join(root, path), 'utf8')
  } catch {
    fail(`${path} is missing — run \`npm --prefix frontend run tokens:generate\``)
    continue
  }
  if (actual !== expected) {
    fail(
      `${path} is not in step with the tokens — run \`npm --prefix frontend run tokens:generate\``,
    )
  }
}

// ------------------------------------------------ 2 · no raw colours

/**
 * A raw colour: a hex value.
 *
 * Three, four, six or eight digits — the shapes a colour is written in.
 */
const HEX = /#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{4}|[0-9a-fA-F]{3})\b/g

/** The files a hex may live in: the tokens, and what they generate. */
const allowed = new Set([
  join(frontend, 'src/tokens.css'),
  join(frontend, 'tokens.ts'),
])

const sources = [
  ...filesUnder(join(frontend, 'src')).filter((path) => /\.(ts|tsx|css)$/.test(path)),
  join(frontend, 'index.html'),
]

for (const path of sources) {
  if (allowed.has(path)) continue
  const text = readFileSync(path, 'utf8')
  text.split('\n').forEach((line, index) => {
    // The one hex the app's own markup may carry is the browser's theme
    // colour, and it is checked against the tokens below.
    const scanned = path.endsWith('index.html')
      ? line.replace(/<meta\s+name="theme-color"[^>]*>/i, '')
      : line
    for (const match of scanned.matchAll(HEX)) {
      fail(
        `${relative(root, path)}:${index + 1} names a raw colour (${match[0]}) — name a token`,
      )
    }
  })
}

// ------------------------------------------- 3 · the browser's own chrome

const indexHtml = readFileSync(join(frontend, 'index.html'), 'utf8')
const theme = indexHtml.match(/<meta\s+name="theme-color"\s+content="([^"]+)"/i)
if (!theme) {
  fail('frontend/index.html has no theme colour — the browser chrome must name a token')
} else if (theme[1].toUpperCase() !== palettes[DEFAULT_PALETTE].brand[700].toUpperCase()) {
  fail(
    `frontend/index.html's theme colour is ${theme[1]}, and the default palette's is ` +
      `${palettes[DEFAULT_PALETTE].brand[700]}`,
  )
}

// The root element carries the palette, so the swap is one attribute from
// the first byte of the page — and the native shell can read the same one.
const rootElement = indexHtml.match(/<html[^>]*\sdata-palette="([^"]+)"/i)
if (!rootElement) {
  fail('frontend/index.html has no palette on the root element')
} else if (!(rootElement[1] in palettes)) {
  fail(`frontend/index.html's palette is ${rootElement[1]}, and there is no such palette`)
}

// --------------------------------------------------------------- the report

if (failures.length > 0) {
  console.error(`check-tokens: ${failures.length} problem(s)\n`)
  for (const failure of failures) console.error('  ✗ ' + failure)
  console.error('')
  process.exit(1)
}

console.log(
  `check-tokens: ${generated.length} generated file(s) in step with the tokens, ` +
    'no raw colour in the app\'s source, the browser chrome named.',
)
