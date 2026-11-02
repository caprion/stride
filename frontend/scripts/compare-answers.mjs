#!/usr/bin/env node
/**
 * The fixture set's compare answers, for every pair it can show.
 *
 * The Compare screen asks the api for a pair of the athlete's runs, so the
 * fixture set has to answer for every pair it can be asked about — the
 * picker is only real if choosing a run changes the table.
 *
 * This script writes those answers out, and the fixture file holds them
 * as data: the app's fixture is a set of answers, not a rule that
 * computes them. The api's test then holds that the answers are the
 * router's, byte for byte, against the store the fixture set describes.
 *
 *   npm --prefix frontend run fixtures:compare
 *
 * The rendering here follows the core's rules — the distance to one
 * decimal, the duration to the nearest minute, the pace to a whole
 * second, the heart rate truncated — and the sign is the core's: a
 * typographic minus, never a hyphen.
 */

import { writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { buildSync } from 'esbuild'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')
const sourcePath = join(root, 'src', 'fixtures', 'index.ts')

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

/** The core's rounding: half to even. */
function roundHalfEven(value, decimals = 0) {
  const factor = 10 ** decimals
  const scaled = value * factor
  const floor = Math.floor(scaled)
  const diff = scaled - floor
  const rounded =
    Math.abs(diff - 0.5) < Number.EPSILON * Math.max(1, Math.abs(scaled))
      ? floor % 2 === 0
        ? floor
        : floor + 1
      : Math.round(scaled)
  return rounded / factor
}

/** The core's distance: one decimal, and the decimals a person drops. */
function distance(metres) {
  const text = roundHalfEven(metres / 1000, 1).toFixed(1)
  return `${text.endsWith('.0') ? text.slice(0, -2) : text} km`
}

/** The core's duration: to the nearest minute. */
function duration(seconds) {
  const minutes = roundHalfEven(seconds / 60, 0)
  if (minutes === 0) return '0 m'
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  if (hours === 0) return `${rest} m`
  if (rest === 0) return `${hours} h`
  return `${hours} h ${rest} m`
}

/** The core's pace: a clock. */
function pace(secondsPerKm) {
  const total = roundHalfEven(secondsPerKm, 0)
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')} /km`
}

/**
 * The core's pace difference: whole seconds, not a clock.
 *
 * A difference of clock times reads as nothing, so the core writes the
 * seconds: "1 s/km".
 */
function paceDifference(secondsPerKm) {
  return `${roundHalfEven(secondsPerKm, 0)} s/km`
}

/** The core's heart rate: truncated. */
function heartRate(bpm) {
  return `${Math.trunc(bpm)} bpm`
}

/** The core's difference: the sign, then the unit's own rendering. */
function difference(value, render) {
  const sign = value > 0 ? '+' : value < 0 ? '−' : ''
  return `${sign}${render(Math.abs(value))}`
}

/** One metric row: the two values and the difference between them. */
function metric(key, label, a, b, render, differenceRender = render) {
  const delta = b.raw - a.raw
  return {
    key,
    label_key: label,
    a: { text: render(a.raw), raw: a.raw },
    b: { text: render(b.raw), raw: b.raw },
    delta: { text: difference(delta, differenceRender), raw: delta },
  }
}

/** The days between two runs. */
function daysApart(a, b) {
  return Math.abs(
    (new Date(`${b.local_date}T00:00:00Z`).getTime() -
      new Date(`${a.local_date}T00:00:00Z`).getTime()) /
      86_400_000,
  )
}

/** The core's basis: how far apart, in the core's words. */
function basisText(a, b) {
  const sameDistance = Math.abs(b.distance.raw - a.distance.raw) <= a.distance.raw * 0.1
  if (!sameDistance) return 'Different distances — read with care'
  const days = daysApart(a, b)
  if (days === 0) return 'Same distance, on the same day'
  if (days < 14) return `Same distance, ${days} days apart`
  return `Same distance, ${Math.round(days / 7)} weeks apart`
}

/** The pace of a run, in seconds per kilometre. */
function paceOf(run) {
  return run.duration.raw / (run.distance.raw / 1000)
}

/**
 * The effort series of each run.
 *
 * The two captured series are the ones the fixture set has — the 9 May
 * run and the 10 May one. The other runs have no samples in the store,
 * so their series is empty, the way the router would answer it.
 */
const captured = { a: fixtures.SERIES_A, b: fixtures.SERIES_B }
const seriesOf = (run) => {
  if (run.id === 102) return captured.a
  if (run.id === 101) return captured.b
  return []
}

const runs = fixtures.activityFixtures
const answers = {}

for (const a of runs) {
  for (const b of runs) {
    if (a.id === b.id) continue
    answers[`${a.id}:${b.id}`] = {
      basis_text: basisText(a, b),
      metrics: [
        metric('distance', 'vital.distance', a.distance, b.distance, distance),
        metric('time', 'vital.time', a.duration, b.duration, duration),
        metric(
          'pace',
          'vital.pace',
          { raw: paceOf(a) },
          { raw: paceOf(b) },
          pace,
          paceDifference,
        ),
        metric('hr', 'vital.heartRate', a.heart_rate, b.heart_rate, heartRate),
      ],
      a: {
        id: a.id,
        label: a.name,
        date: a.local_date,
        distance: a.distance,
        story: a.story,
      },
      b: {
        id: b.id,
        label: b.name,
        date: b.local_date,
        distance: b.distance,
        story: b.story,
      },
    }
  }
}

const outPath = join(root, 'src', 'fixtures', 'compare-answers.json')
writeFileSync(outPath, `${JSON.stringify(answers, null, 2)}\n`, 'utf8')
console.log(`fixtures: ${Object.keys(answers).length} compare answers written to ${outPath}`)

/**
 * The captured answer for the pair the app's demo shows.
 *
 * One answer, exactly as the router answered it — the api's test reads
 * this file, seeds the matching store, calls the real router and
 * compares the two byte for byte. So the fixture stops being a mock
 * someone wrote and becomes an answer the test keeps true.
 */
const demoPair = '102:101'
const demo = answers[demoPair]
const capturedPath = join(root, 'src', 'fixtures', 'captured.json')
writeFileSync(
  capturedPath,
  `${JSON.stringify(
    {
      route: '/compare',
      query: { a: '102', b: '101' },
      answer: { ...demo, series: { a: captured.a, b: captured.b } },
    },
    null,
    2,
  )}\n`,
  'utf8',
)
console.log(`fixtures: the captured answer written to ${capturedPath}`)
