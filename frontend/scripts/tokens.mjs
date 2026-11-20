/**
 * The generators.
 *
 * Two files are generated from the token file, and neither is written by
 * hand:
 *
 *   * `frontend/src/tokens.css` — the colours the app is painted with, one
 *     block per palette;
 *   * `android/app/src/main/res/values/colors.xml` — the phone's own
 *     chrome, and the night variant of it.
 *
 * The gate checks both against the tokens — the same way the app's
 * fixtures are checked against the router's answers. A token that changed
 * and a generated file that did not is a failure, not a drift.
 */

import { androidColors, DEFAULT_PALETTE, launcherBackground, palettes } from '../tokens.ts'

/** The ten steps of a ramp. */
const STEPS = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900]

/** The inks, in the order they are written in. */
const INKS = ['DEFAULT', 'soft', 'muted', 'faint']

/** The surfaces, in the order they are drawn. */
const SURFACES = ['DEFAULT', 'soft', 'raised', 'sunken']

/**
 * A token's value, by its path.
 *
 * The paths are the ones the token file uses: `surface.soft`, `ink.DEFAULT`,
 * `brand.600`.
 */
export function valueAt(palette, path) {
  return path.split('.').reduce((node, key) => node[key], palette)
}

/**
 * The CSS custom properties of one palette.
 *
 * Every role the app has is here, so a palette can never be half
 * painted: the names are the same in every block, and only the values
 * change.
 */
export function paletteVariables(palette) {
  const variables = {}
  for (const step of STEPS) variables[`--brand-${step}`] = palette.brand[step]
  for (const step of STEPS) variables[`--accent-${step}`] = palette.accent[step]
  for (const ink of INKS) {
    variables[ink === 'DEFAULT' ? '--ink' : `--ink-${ink}`] = palette.ink[ink]
  }
  for (const surface of SURFACES) {
    variables[surface === 'DEFAULT' ? '--surface' : `--surface-${surface}`] =
      palette.surface[surface]
  }
  variables['--edge'] = palette.edge
  variables['--better'] = palette.better
  variables['--usual'] = palette.usual
  variables['--could'] = palette.could
  palette.ramp.forEach((colour, index) => {
    variables[`--ramp-${index + 1}`] = colour
  })
  variables['--chart-grid'] = palette.chart.grid
  variables['--chart-axis'] = palette.chart.axis
  variables['--chart-band'] = palette.chart.band
  variables['--chart-line'] = palette.chart.line
  variables['--chart-line-alt'] = palette.chart.lineAlt
  variables['--chart-reference'] = palette.chart.reference
  return variables
}

/** The variables of one palette, as CSS text. */
function block(variables, selector, extra = '') {
  const lines = Object.entries(variables).map(([name, value]) => `  ${name}: ${value};`)
  return `${selector} {\n${lines.join('\n')}${extra}\n}`
}

/**
 * The CSS the app is painted with.
 *
 * The default palette is on `:root`, and every palette has a block of its
 * own. The blocks are keyed on the attribute the app sets on the root
 * element — `data-palette` — so the swap is one attribute and nothing
 * else.
 */
export function cssText() {
  const names = Object.keys(palettes)
  const sections = [
    '/*',
    ' * The colours the app is painted with.',
    ' *',
    ' * GENERATED from `frontend/tokens.ts` — do not edit.',
    ' * Run `npm --prefix frontend run tokens:generate` after a token changes.',
    ' *',
    ' * The palette is one attribute on the root element: `data-palette`.',
    ' */',
    '',
    `:root {`,
    `  color-scheme: light;`,
    ...Object.entries(paletteVariables(palettes[DEFAULT_PALETTE])).map(
      ([name, value]) => `  ${name}: ${value};`,
    ),
    '}',
    '',
  ]
  for (const name of names) {
    if (name === DEFAULT_PALETTE) continue
    const extra = name === 'night' ? '\n  color-scheme: dark;' : ''
    sections.push(block(paletteVariables(palettes[name]), `html[data-palette='${name}']`, extra))
    sections.push('')
  }
  return sections.join('\n')
}

/**
 * The phone's own chrome.
 *
 * Five colours: the surface the screens are drawn on, the ink they are
 * written in, the muted ink, the one accent colour a link may take, and
 * the launcher's backdrop.
 */
export function androidXml(palette) {
  const colours = [
    ['stride_surface', valueAt(palette, androidColors.surface)],
    ['stride_on_surface', valueAt(palette, androidColors.onSurface)],
    ['stride_muted', valueAt(palette, androidColors.muted)],
    ['stride_brand', valueAt(palette, androidColors.brand)],
    ['stride_launcher_background', launcherBackground],
  ]
  const lines = colours.map(([name, value]) => `    <color name="${name}">${value}</color>`)
  return [
    "<?xml version='1.0' encoding='utf-8'?>",
    '<!--',
    '    GENERATED from `frontend/tokens.ts` — do not edit.',
    '    Run `npm --prefix frontend run tokens:generate` after a token changes.',
    '-->',
    '<resources>',
    '    <!-- The app\'s own colours: the surface the screens are drawn on, and',
    '         the one accent colour a link may take. -->',
    ...lines,
    '</resources>',
    '',
  ].join('\n')
}

/** The paths of the files the generators own. */
export const generatedFiles = {
  css: 'frontend/src/tokens.css',
  android: 'android/app/src/main/res/values/colors.xml',
  androidNight: 'android/app/src/main/res/values-night/colors.xml',
}
