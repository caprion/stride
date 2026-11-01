/**
 * The chrome.
 *
 * The app's formatting file holds the chrome and nothing else: the dates,
 * the counts, the sport's own name, and the em dash that stands for a
 * value that is not there. A domain value is not formatted here — it
 * arrives rendered (`src/lib/types.ts`).
 */

import { describe, expect, it } from 'vitest'
import {
  ABSENT,
  formatCount,
  formatDateMedium,
  formatDateShort,
  renderedText,
  sessionTypeLabel,
  sportLabel,
} from '../src/lib/format'

describe('the chrome', () => {
  it('reads a date the way the athlete reads it', () => {
    expect(formatDateShort('2026-05-10')).toBe('Sun 10 May')
    expect(formatDateMedium('2026-05-10')).toBe('10 May 2026')
  })

  it('says nothing about a date that is not there', () => {
    expect(formatDateShort(null)).toBe(ABSENT)
    expect(formatDateShort('not a date')).toBe(ABSENT)
    expect(formatDateMedium(undefined)).toBe(ABSENT)
  })

  it('counts without rounding — a count is already whole', () => {
    expect(formatCount(8)).toBe('8')
    expect(formatCount(0)).toBe('0')
    expect(formatCount(null)).toBe(ABSENT)
  })

  it('names the sport the way the athlete names it', () => {
    expect(sportLabel('running')).toBe('Run')
    expect(sportLabel('trail_running')).toBe('Trail run')
    expect(sportLabel(null)).toBe('Run')
  })

  it('reads the plan\'s own vocabulary', () => {
    expect(sessionTypeLabel('easy')).toBe('easy')
    expect(sessionTypeLabel('long_run')).toBe('long run')
    expect(sessionTypeLabel('unknown')).toBe('Unknown')
  })

  it('shows a value that is not there as the em dash — never "0"', () => {
    expect(renderedText(null)).toBe(ABSENT)
    expect(renderedText(undefined)).toBe(ABSENT)
    expect(renderedText({ text: '8.4 km', raw: 8_400 })).toBe('8.4 km')
    // A raw of zero with a text is still the text: the app does not
    // second-guess the core's answer.
    expect(renderedText({ text: '0 km', raw: 0 })).toBe('0 km')
  })
})
