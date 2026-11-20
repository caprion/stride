#!/usr/bin/env node
/**
 * Write the generated files.
 *
 *   node scripts/gen-tokens.mjs
 *
 * The token file is the source; this writes the CSS the app is painted
 * with and the phone's own chrome from it. The gate checks that both are
 * in step, so a token that changed and a generated file that did not is a
 * failure rather than a drift.
 */

import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { androidXml, cssText } from './tokens.mjs'
import { DEFAULT_PALETTE, palettes } from '../tokens.ts'

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..')

/** One generated file: where it goes, and what it says. */
function write(relative, text) {
  const path = join(root, relative)
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, text)
  console.log('wrote ' + relative)
}

write('frontend/src/tokens.css', cssText())
write(
  'android/app/src/main/res/values/colors.xml',
  androidXml(palettes[DEFAULT_PALETTE]),
)
write(
  'android/app/src/main/res/values-night/colors.xml',
  androidXml(palettes.night),
)
