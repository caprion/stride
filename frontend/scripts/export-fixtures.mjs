#!/usr/bin/env node
/**
 * The fixture set, written from the router's own answers.
 *
 * **The fixture is the router's answer, not a mock someone wrote.** The
 * api's parity test seeds a store from the seed that travels with the
 * fixture, calls the real router and compares the answers value for
 * value. It can also write the router's answers instead of comparing them:
 *
 *     STRIDE_FIXTURES_OUT=… STRIDE_COMPARE_OUT=… cargo test -p stride-api --test fixture_parity
 *
 * This script takes those two dumps and writes the fixture files the app
 * renders from.
 *
 * **The seed is the app's, and the answers are the router's.** The seed —
 * the runs, the nights and the plan sessions — is what a store has to
 * hold for the router to answer the same way, and it is written from the
 * app's own fixture module. The answers are the router's, copied out of
 * its dump without being re-serialised: the router writes `62.0` where a
 * JavaScript `JSON.stringify` writes `62`, and the api's test compares
 * the answers value for value, so the answer's own text is what travels.
 *
 * The two halves are written together, and the script refuses to write a
 * dump that was taken from a different seed: the answers would be about
 * another store.
 *
 *   npm --prefix frontend run fixtures:export -- --seed
 *   npm --prefix frontend run fixtures:export -- <router-answers.json> <router-compare.json>
 */

import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { buildSync } from 'esbuild'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')

const answersPath = join(root, 'src', 'fixtures', 'answers.json')
const comparePath = join(root, 'src', 'fixtures', 'compare-answers.json')

/**
 * The app's own fixture module, bundled.
 *
 * The seed is written from it, so the seed and the screens cannot drift
 * apart: there is one set of runs, nights and plan sessions, and both
 * the file and the screens read it.
 */
const bundled = buildSync({
  entryPoints: [join(root, 'src', 'fixtures', 'index.ts')],
  bundle: true,
  format: 'esm',
  platform: 'node',
  target: 'node20',
  write: false,
})
const code = bundled.outputFiles[0]?.text ?? ''
const fixtures = await import(
  `data:text/javascript;base64,${Buffer.from(code).toString('base64')}`
)

/** The seed: the runs, the nights and the plan sessions. */
const seed = {
  activities: fixtures.activityFixtures,
  nights: fixtures.nightsFixture,
  plan_sessions: fixtures.planSessionsFixture,
}

/**
 * The answers' own text, taken out of the dump.
 *
 * Not re-serialised: the router writes `62.0` where a JavaScript
 * `JSON.stringify` writes `62`, and the api's test compares the answers
 * value for value — a whole number and a float are not the same value to
 * it. The dump's text is the answer; the app reads it.
 */
function answersText(dump) {
  const key = dump.indexOf('"answers"')
  if (key < 0) throw new Error('the dump carries no answers')
  const open = dump.indexOf('{', key)
  let depth = 0
  let inString = false
  let escaped = false
  for (let index = open; index < dump.length; index += 1) {
    const character = dump[index]
    if (inString) {
      if (escaped) escaped = false
      else if (character === '\\') escaped = true
      else if (character === '"') inString = false
      continue
    }
    if (character === '"') {
      inString = true
      continue
    }
    if (character === '{') depth += 1
    else if (character === '}') {
      depth -= 1
      if (depth === 0) return dump.slice(open, index + 1)
    }
  }
  throw new Error("the dump's answers are not closed")
}

/**
 * The value, with every object's keys in one order.
 *
 * The router's dump is written by Rust, whose JSON object keys come out
 * in alphabetical order; the app's seed has the order the app wrote it
 * in. The two are the same seed, so they are compared by their values
 * and not by their spelling.
 */
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical)
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, canonical(value[key])]),
    )
  }
  return value
}

/** The fixture file: the answers and the seed they come from. */
function fixtureFile(answers) {
  return `{
  "version": 1,
  "answers": ${answers},
  "seed": ${JSON.stringify(seed, null, 2)}
}
`
}

const args = process.argv.slice(2)

if (args[0] === '--seed') {
  // The seed alone: the api's test reads it, seeds the store with it and
  // answers from it. The answers stay as they are — the router's own
  // text — until the router has been asked again.
  const current = readFileSync(answersPath, 'utf8')
  writeFileSync(answersPath, fixtureFile(answersText(current)), 'utf8')
  console.log('fixtures: the seed is written; ask the router again for the answers')
  process.exit(0)
}

const [answersIn, compareIn] = args
if (!answersIn || !compareIn) {
  console.error(
    'usage: npm run fixtures:export -- <router-answers.json> <router-compare.json>\n' +
      'the two files are the api test\'s dumps (STRIDE_FIXTURES_OUT, STRIDE_COMPARE_OUT)\n' +
      'or: npm run fixtures:export -- --seed   (write the seed alone)',
  )
  process.exit(1)
}

/** The router's answers, by the route that answers them — its own text. */
const routerAnswersText = readFileSync(answersIn, 'utf8')

/** The router's compare answer for every pair — its own text. */
const routerCompareText = readFileSync(compareIn, 'utf8')

// The dump was taken from a seed: it is the seed the file held then. If
// the app's seed has moved on, the dump's answers are about another store
// and writing them would be writing a lie.
const dumped = JSON.parse(routerAnswersText)
if (JSON.stringify(canonical(dumped.seed)) !== JSON.stringify(canonical(seed))) {
  console.error(
    'fixtures: the dump was taken from a different seed — the app\'s runs, nights or ' +
      'plan sessions have changed since it was taken. Write the seed, ask the router ' +
      'again, and write the answers.',
  )
  process.exit(1)
}

writeFileSync(answersPath, fixtureFile(answersText(routerAnswersText)), 'utf8')
console.log(`fixtures: the router's answers written to ${answersPath}`)

writeFileSync(comparePath, routerCompareText, 'utf8')
console.log(
  `fixtures: ${Object.keys(JSON.parse(routerCompareText)).length} compare answers written to ${comparePath}`,
)
