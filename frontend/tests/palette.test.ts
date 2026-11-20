/**
 * The palette.
 *
 * The swap is one attribute on the root element, and the athlete's choice
 * is kept on the device. Nothing here talks to a server: the palette is a
 * property of this screen, not of the training.
 */

import { beforeEach, describe, expect, it } from 'vitest'
import { applyPalette, chosenPalette, isPalette, setPalette } from '../src/lib/palette'
import { DEFAULT_PALETTE } from '../tokens'

describe('the palette', () => {
  beforeEach(() => {
    window.localStorage.clear()
    document.documentElement.removeAttribute('data-palette')
  })

  it('names only the palettes there are', () => {
    expect(isPalette('dreeve')).toBe(true)
    expect(isPalette('stride')).toBe(true)
    expect(isPalette('night')).toBe(true)
    expect(isPalette('vivid')).toBe(false)
    expect(isPalette('')).toBe(false)
    expect(isPalette(null)).toBe(false)
  })

  it('paints the app in the palette the athlete chose', () => {
    setPalette('dreeve')
    expect(document.documentElement.dataset.palette).toBe('dreeve')
    expect(chosenPalette()).toBe('dreeve')
  })

  it('falls back to the default when nothing was chosen', () => {
    expect(chosenPalette()).toBe(DEFAULT_PALETTE)
    applyPalette()
    expect(document.documentElement.dataset.palette).toBe(DEFAULT_PALETTE)
  })

  it('falls back to the default when the stored name is not a palette', () => {
    window.localStorage.setItem('stride.palette', 'a palette from another build')
    expect(chosenPalette()).toBe(DEFAULT_PALETTE)
  })
})
