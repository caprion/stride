/**
 * The app renders chrome, and the core renders the domain.
 *
 * `docs/contract/core-contract.md` §8: "The frontend renders app chrome
 * only. It does not re-implement the domain's wording, and it does not
 * re-implement the domain's rounding."
 *
 * The frontend once shipped a port of `units.rs` and rendered values with
 * it. The port was exact, and it still drifted the moment the core's rule
 * changed — nothing failed, because a copy of a rule is not a rule.
 *
 * So this is a test, not a convention: the app's own source is scanned for
 * the shapes a second implementation of the domain would take.
 */

import { readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const here = dirname(fileURLToPath(import.meta.url))
const src = join(here, '..', 'src')

function walk(dir: string): string[] {
  const out: string[] = []
  for (const name of readdirSync(dir)) {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) out.push(...walk(path))
    else if (/\.(ts|tsx)$/.test(name)) out.push(path)
  }
  return out
}

const files = walk(src)

describe('the app renders chrome only', () => {
  it('has no port of the core\'s units', () => {
    const names = files.map((file) => relative(src, file))
    expect(names).not.toContain(join('lib', 'units.ts'))
  })

  it('has no formatter for a domain value', () => {
    // A function that turns a number of metres, seconds, beats per minute
    // or seconds per kilometre into words is the core's business. The
    // names are listed so that the day one of them comes back, the test
    // says so.
    const domainFormatters = [
      'formatDistance',
      'formatDuration',
      'formatPace',
      'formatHeartRate',
      'formatElevation',
      'formatKm',
      'formatMetres',
      'formatSeconds',
      'formatUnits',
      'renderDistance',
      'renderDuration',
      'renderPace',
    ]
    const offenders: string[] = []
    for (const file of files) {
      const text = readFileSync(file, 'utf8')
      for (const name of domainFormatters) {
        if (new RegExp(`\\b${name}\\b`).test(text)) {
          offenders.push(`${relative(src, file)}: ${name}`)
        }
      }
    }
    expect(offenders).toEqual([])
  })

  it('rounds no domain value of its own', () => {
    // The core rounds. The app draws a chart's scale, which is a chart's
    // business — so a rounding that feeds a chart's axis is fine. A
    // rounding of a distance, a duration, a pace, a heart rate or an
    // elevation, anywhere else, is the core's rule, and a second one is a
    // second answer to the same question.
    const domainValue =
      /(distance|duration|moving|elapsed|pace|heart_rate|avg_hr|max_hr|elevation|climb|metres|_m\b|_s\b|bpm|ms\b)/i
    // A chart's own geometry: the axis, the pixel, the point.
    const chartGeometry = /^\s*(minute|km|m|x|y|px|width|height)\s*:/i

    const offenders: string[] = []
    for (const file of files) {
      // The fixtures synthesise a chart's numbers; they are not a rule.
      if (file.includes(`${join('', 'fixtures')}`)) continue
      const lines = readFileSync(file, 'utf8').split('\n')
      lines.forEach((line, index) => {
        if (!/Math\.round\s*\(|\.toFixed\s*\(/.test(line)) return
        if (chartGeometry.test(line)) return
        // A rounding often reads its value on the line above.
        const window = lines.slice(Math.max(0, index - 2), index + 1).join(' ')
        if (!domainValue.test(window)) return
        offenders.push(`${relative(src, file)}:${index + 1}: ${line.trim()}`)
      })
    }
    expect(offenders).toEqual([])
  })

  it('carries no rendering hint in its types', () => {
    // A `format:` hint tells the frontend *how* to render — the hole §8
    // closes.
    const types = readFileSync(join(src, 'lib', 'types.ts'), 'utf8')
    expect(types).not.toMatch(/\bformat\??\s*:/)
  })

  it('reads a domain value only as text and as a number', () => {
    const types = readFileSync(join(src, 'lib', 'types.ts'), 'utf8')
    expect(types).toContain('export interface RenderedValue')
    expect(types).toContain('text: string')
    expect(types).toContain('raw: number | null')
  })
})
