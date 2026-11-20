/**
 * The tokens.
 *
 * **One source, and the gate can tell.** These tests hold the shape of the
 * token file and the parity between the tokens and the phone's own chrome:
 * the Android colours are generated from the tokens, and a generated file
 * that has fallen behind is a failure, not a drift.
 *
 * The gate's own check (`scripts/check-tokens.mjs`) makes the same parity
 * claim over the whole app — that no component names a raw colour — and
 * this file holds the parts a grep cannot see.
 */

import { existsSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  DEFAULT_PALETTE,
  launcherBackground,
  palettes,
  type PaletteName,
} from '../tokens'

/**
 * The repository root: the directory that holds the phone and the app.
 *
 * The test may be run from either, so it looks for the phone rather than
 * assuming where it was started from.
 */
function repoRoot(): string {
  let directory = process.cwd()
  for (let depth = 0; depth < 6; depth += 1) {
    if (existsSync(join(directory, 'android', 'app'))) return directory
    const parent = dirname(directory)
    if (parent === directory) break
    directory = parent
  }
  throw new Error('the repository root was not found')
}

/** The phone's own chrome, read from the generated file. */
function androidColours(relative: string): Record<string, string> {
  const xml = readFileSync(join(repoRoot(), relative), 'utf8')
  const colours: Record<string, string> = {}
  for (const match of xml.matchAll(/<color name="([^"]+)">([^<]+)<\/color>/g)) {
    colours[match[1] as string] = match[2] as string
  }
  return colours
}

/** Every colour a palette holds, as a flat list of its values. */
function coloursOf(name: PaletteName): string[] {
  const palette = palettes[name]
  return [
    ...Object.values(palette.brand),
    ...Object.values(palette.accent),
    ...Object.values(palette.ink),
    ...Object.values(palette.surface),
    palette.edge,
    palette.better,
    palette.usual,
    palette.could,
    ...palette.ramp,
    ...Object.values(palette.chart),
  ]
}

describe('the tokens', () => {
  it('holds every palette the app can be drawn in', () => {
    expect(Object.keys(palettes)).toEqual(['stride', 'dreeve', 'night'])
    expect(DEFAULT_PALETTE).toBe('stride')
  })

  it('answers every role in every palette', () => {
    const reference = Object.keys(palettes.stride).sort()
    for (const [name, palette] of Object.entries(palettes)) {
      expect(Object.keys(palette).sort(), name).toEqual(reference)
    }
  })

  it('holds colours and nothing else', () => {
    for (const name of Object.keys(palettes) as PaletteName[]) {
      for (const colour of coloursOf(name)) {
        expect(colour, `${name}: ${colour}`).toMatch(/^#[0-9A-F]{6}$/)
      }
    }
  })

  it('draws a chart in the palette it belongs to', () => {
    for (const [name, palette] of Object.entries(palettes)) {
      // The chart's colours are derived from the ramps, so a palette can
      // never come out half painted.
      expect(palette.chart.band, name).toBe(palette.brand[100])
      expect(palette.chart.line, name).toBe(palette.brand[500])
      expect(palette.chart.lineAlt, name).toBe(palette.brand[200])
      expect(palette.chart.reference, name).toBe(palette.accent[500])
    }
  })

  it('never says a verdict in a colour', () => {
    for (const [name, palette] of Object.entries(palettes)) {
      // The three directions and nothing else: there is no "good", no
      // "bad" and no score in the app, so there is none in its colours.
      expect(Object.keys(palette).some((key) => ['good', 'bad', 'score'].includes(key)), name).toBe(
        false,
      )
      expect(palette.better).toMatch(/^#[0-9A-F]{6}$/)
      expect(palette.usual).toMatch(/^#[0-9A-F]{6}$/)
      expect(palette.could).toMatch(/^#[0-9A-F]{6}$/)
    }
  })
})

describe("the phone's own chrome", () => {
  it('is generated from the tokens', () => {
    const colours = androidColours('android/app/src/main/res/values/colors.xml')
    const palette = palettes[DEFAULT_PALETTE]
    expect(colours.stride_surface).toBe(palette.surface.soft)
    expect(colours.stride_on_surface).toBe(palette.ink.DEFAULT)
    expect(colours.stride_muted).toBe(palette.ink.muted)
    expect(colours.stride_brand).toBe(palette.brand[600])
    expect(colours.stride_launcher_background).toBe(launcherBackground)
  })

  it('follows the phone into its dark mode', () => {
    const colours = androidColours('android/app/src/main/res/values-night/colors.xml')
    const palette = palettes.night
    expect(colours.stride_surface).toBe(palette.surface.soft)
    expect(colours.stride_on_surface).toBe(palette.ink.DEFAULT)
    expect(colours.stride_muted).toBe(palette.ink.muted)
    expect(colours.stride_brand).toBe(palette.brand[600])
  })
})
