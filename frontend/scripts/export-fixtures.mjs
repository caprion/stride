#!/usr/bin/env node
/**
 * The fixture set, as a file the api's test can read.
 *
 * The app's fixture set is a TypeScript module — the screens render from
 * it, and the unit tests assert against it. The api's test has to read
 * the same answers, byte for byte, and compare them with the router's.
 *
 * So the answers are written down, once, in a machine-readable file:
 * `src/fixtures/answers.json`. It is generated from the module — never
 * hand-edited — and a unit test holds that the two agree, so the file
 * cannot drift from the screens.
 *
 *   npm --prefix frontend run fixtures:export
 *
 * The file carries the answers the app renders, and the seed they come
 * from: the runs, the nights and the plan sessions. The api's test seeds
 * a store from the seed and asks the router the same questions.
 */

import { writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { buildSync } from 'esbuild'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const sourcePath = join(root, 'src', 'fixtures', 'index.ts')
const outPath = join(root, 'src', 'fixtures', 'answers.json')

/**
 * The routes the fixture set answers, and the parameters each takes.
 *
 * The compare route is asked for a pair — the two runs the screen
 * compares — and the answer is for that pair.
 */
const ROUTES = [
  { path: '/compare', params: { a: '102', b: '101' } },
  '/activities',
  '/activities/101',
  '/physiology/vitals',
  '/plan/week',
  '/insights/today',
  '/insights/progress',
  '/compare/candidates',
  '/sync/status',
  '/health',
  '/health/daily',
  '/auth/status',
  '/settings',
]

// The fixture set is bundled, so its own imports — the generated compare
// answers — come with it.
const bundled = buildSync({
  entryPoints: [sourcePath],
  bundle: true,
  format: 'esm',
  platform: 'node',
  target: 'node20',
  write: false,
})
const code = bundled.outputFiles[0]?.text ?? ''
const dataUrl = `data:text/javascript;base64,${Buffer.from(code).toString('base64')}`
const fixtures = await import(dataUrl)

const answers = {}
for (const route of ROUTES) {
  const path = typeof route === 'string' ? route : route.path
  const params = typeof route === 'string' ? {} : (route.params ?? {})
  const answer = fixtures.fixtureResponse(path, params, '1')
  if (answer === undefined) {
    console.error(`the fixture set has no answer for ${path}`)
    process.exit(1)
  }
  answers[path] = answer
}

const exported = {
  /**
   * The version of this file's shape. A change here is a change the api's
   * test has to know about.
   */
  version: 1,
  /** The answers, by the route that answers them. */
  answers,
  /**
   * The seed the answers come from.
   *
   * The runs, the nights and the plan sessions: what a store has to hold
   * for the router to answer the same way.
   */
  seed: {
    activities: fixtures.activityFixtures,
    nights: fixtures.nightsFixture,
    plan_sessions: fixtures.planSessionsFixture,
  },
}

writeFileSync(outPath, `${JSON.stringify(exported, null, 2)}\n`, 'utf8')
console.log(`fixtures: ${Object.keys(answers).length} answers written to ${outPath}`)
