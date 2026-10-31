#!/usr/bin/env node
/**
 * The voice lint.
 *
 * It reads the string catalogue and fails the build on:
 *
 *   - a banned word or construction (the contract's ban list),
 *   - a `direction` string outside the closed set,
 *   - a `pattern` string with no sample size,
 *   - an `option` string that mentions a vital,
 *   - a banned word written inline in a component instead of in the
 *     catalogue.
 *
 * Run it with `npm run lint:voice`.
 */

import { readFileSync, readdirSync, statSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { transformSync } from 'esbuild'
import { checkCatalogue, checkInlineText } from './voice-lint.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const cataloguePath = join(root, 'src/i18n/catalogue.ts')

// ---------------------------------------------------------------- catalogue

const source = readFileSync(cataloguePath, 'utf8')
const compiled = transformSync(source, { loader: 'ts', format: 'esm', target: 'node20' })
const dataUrl = `data:text/javascript;base64,${Buffer.from(compiled.code).toString('base64')}`
const catalogueModule = await import(dataUrl)

const { catalogue, directions, DIRECTION_WORDS } = catalogueModule
const allEntries = { ...catalogue, ...directions }

const failures = checkCatalogue(allEntries, DIRECTION_WORDS)

// ------------------------------------- inline strings in the components

function walk(dir) {
  const out = []
  for (const name of readdirSync(dir)) {
    const path = join(dir, name)
    const info = statSync(path)
    if (info.isDirectory()) out.push(...walk(path))
    else if (/\.tsx$/.test(name)) out.push(path)
  }
  return out
}

for (const file of walk(join(root, 'src'))) {
  failures.push(...checkInlineText(relative(root, file), readFileSync(file, 'utf8')))
}

// ---------------------------------------------------------------- report

const count = Object.keys(allEntries).length
if (failures.length > 0) {
  console.error(`voice lint: ${failures.length} failure(s) in ${count} strings\n`)
  for (const failure of failures) console.error(`  ✗ ${failure}`)
  console.error('')
  process.exit(1)
}

console.log(`voice lint: ${count} strings, no banned words, closed direction set intact.`)
