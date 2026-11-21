/**
 * The Tailwind theme.
 *
 * **Generated from `tokens.ts` — this file holds no colour, no size and no
 * shadow of its own.** Every value below is read from the token file, so
 * there is one source for the app: the tokens.
 *
 * The colours are CSS variables rather than literal values, and that is
 * what makes the palette a one-line change: `data-palette="dreeve"` on the
 * root element swaps the variables, and every class in the app follows
 * without being rewritten.
 */

import { fonts, motion, palettes, radii, shadows, spacing, typeScale } from './tokens.ts'

/** The ten steps of a ramp. */
const STEPS = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900] as const

/** A ramp as Tailwind wants it: every step a CSS variable. */
function ramp(name: string): Record<string, string> {
  return Object.fromEntries(STEPS.map((step) => [step, `var(--${name}-${step})`]))
}

/** The inks: the four weights of text. */
const ink = {
  DEFAULT: 'var(--ink)',
  soft: 'var(--ink-soft)',
  muted: 'var(--ink-muted)',
  faint: 'var(--ink-faint)',
}

/** The surfaces: the four planes the app is drawn on. */
const surface = {
  DEFAULT: 'var(--surface)',
  soft: 'var(--surface-soft)',
  raised: 'var(--surface-raised)',
  sunken: 'var(--surface-sunken)',
}

/** The chart's own colours: the grid, the axis, the band, the series. */
const chart = {
  grid: 'var(--chart-grid)',
  axis: 'var(--chart-axis)',
  band: 'var(--chart-band)',
  line: 'var(--chart-line)',
  lineAlt: 'var(--chart-line-alt)',
  reference: 'var(--chart-reference)',
}

/** The type scale: the size, and the line height that goes with it. */
const fontSize = Object.fromEntries(
  Object.entries(typeScale).map(([name, step]) => [name, [step.size, { lineHeight: step.lineHeight }]]),
)

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Brand — the deep azure in `stride`, the orange-red in `dreeve`.
        brand: ramp('brand'),
        // Accent — the warm ramp: the athlete's own data.
        accent: ramp('accent'),
        // Ink — a blue-black in `stride`, a pale grey in `night`.
        ink,
        surface,
        // The rail: the wide arrangement's own ground and ink.
        rail: {
          ground: 'var(--rail-ground)',
          ink: 'var(--rail-ink)',
          'ink-strong': 'var(--rail-ink-strong)',
          active: 'var(--rail-active)',
          hover: 'var(--rail-hover)',
          foot: 'var(--rail-foot)',
        },
        edge: 'var(--edge)',
        // The only "positive"/"negative" tones in the app are for the
        // direction words, and they are never red: nothing is a failure.
        better: 'var(--better)',
        usual: 'var(--usual)',
        could: 'var(--could)',
        // The series ramp: six steps, for a chart that draws more than one.
        ramp: palettes.stride.ramp.map((_, index) => `var(--ramp-${index + 1})`),
        chart,
      },
      fontFamily: {
        sans: [...fonts.sans],
        mono: [...fonts.mono],
      },
      fontSize,
      spacing: { ...spacing },
      boxShadow: { ...shadows },
      borderRadius: { ...radii },
      keyframes: motion.keyframes,
      animation: motion.animation,
    },
  },
  plugins: [],
}
