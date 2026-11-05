/**
 * The fixture set, and the file the api's test reads.
 *
 * The app's fixture set is a TypeScript module: the screens render from
 * it, and the tests assert against it. The api's test has to read the
 * same answers and compare them with the router's, byte for byte.
 *
 * So the answers are written down in a machine-readable file —
 * `src/fixtures/answers.json` — generated from the module, and this test
 * holds that the file and the module agree. The file cannot drift from
 * the screens, and the screens cannot drift from the file.
 */

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import {
  activityFixtures,
  fixtureResponse,
  nightsFixture,
  planSessionsFixture,
} from '../src/fixtures'

const here = dirname(fileURLToPath(import.meta.url))
const exportedPath = join(here, '..', 'src', 'fixtures', 'answers.json')

interface Exported {
  version: number
  answers: Record<string, unknown>
  seed: {
    activities: unknown
    nights: unknown
    plan_sessions: unknown
  }
}

/** The export, as it is on disk. */
function exported(): Exported {
  return JSON.parse(readFileSync(exportedPath, 'utf8')) as Exported
}

/**
 * The answers without their timestamps.
 *
 * The sync status answers "two hours ago" — the wall clock, so the rig
 * shows a fresh sync — and that is the one answer that moves. The
 * timestamps are placeholders here, and the rest of the answer is
 * compared exactly.
 */
function withoutTimestamps(value: unknown): unknown {
  const text = JSON.stringify(value)
  return JSON.parse(text.replace(/\d{4}-\d{2}-\d{2}T[\d:.]+Z/g, '<time>'))
}

describe("the fixture set's export", () => {
  it('carries the answers the app renders', () => {
    const file = exported()

    for (const [route, answer] of Object.entries(file.answers)) {
      const [path, query] = route.split('?')
      // The compare route is asked for a pair; the rest are asked for
      // as they are.
      const params: Record<string, string> =
        route === '/compare'
          ? { a: '102', b: '101' }
          : Object.fromEntries(new URLSearchParams(query))
      const rendered = fixtureResponse(path!, params, '1')
      expect(withoutTimestamps(rendered)).toEqual(withoutTimestamps(answer))
    }
  })

  it('carries the seed the answers come from', () => {
    const file = exported()

    expect(file.seed.activities).toEqual(activityFixtures)
    expect(file.seed.nights).toEqual(nightsFixture)
    expect(file.seed.plan_sessions).toEqual(planSessionsFixture)
  })

  it('is the file the api thread reads', () => {
    // The path is part of the contract with the api's test: the test
    // reads this file, not a copy of it.
    expect(exportedPath.endsWith(join('src', 'fixtures', 'answers.json'))).toBe(true)
    expect(exported().version).toBe(1)
  })
})

describe('the captured answer', () => {
  it("is the shape the api's parity test reads", () => {
    const captured = JSON.parse(
      readFileSync(join(here, '..', 'src', 'fixtures', 'captured.json'), 'utf8'),
    ) as { route: string; body: Record<string, unknown> }

    expect(typeof captured.route).toBe('string')
    expect(captured.body).toBeTruthy()
  })

  it("carries the delta pull's fields, with the router's nullability", () => {
    // The four fields the router added, and the nullability it answers
    // them with: the cursor is a string or nothing, the three counts are
    // numbers or nothing.
    const captured = JSON.parse(
      readFileSync(join(here, '..', 'src', 'fixtures', 'captured.json'), 'utf8'),
    ) as { body: Record<string, unknown> }

    expect(captured.body.cursor === null || typeof captured.body.cursor === 'string').toBe(true)
    for (const field of ['pulled', 'skipped', 'deleted']) {
      expect(captured.body[field] === null || typeof captured.body[field] === 'number').toBe(
        true,
      )
    }
  })

  it("is the answer the router gives for the store the api's test seeds", () => {
    // The api's seeded harness holds one run and no cursor, so the
    // router answers these numbers — and the capture has to be the same
    // answer, byte for byte, or the parity test fails.
    const captured = JSON.parse(
      readFileSync(join(here, '..', 'src', 'fixtures', 'captured.json'), 'utf8'),
    ) as { body: Record<string, unknown> }

    expect(captured.body.activity_count).toBe(1)
    expect(captured.body.cursor).toBeNull()
    expect(captured.body.pulled).toBe(0)
    expect(captured.body.skipped).toBe(0)
    expect(captured.body.deleted).toBe(0)
  })
})
