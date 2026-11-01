/**
 * The units port, against the core's own answers.
 *
 * Every expectation in this file was read out of
 * `crates/stride-core/src/units.rs` — the same file the app's copy is
 * taken from. If the two ever disagree, one of these turns red.
 */

import { describe, expect, it } from 'vitest'
import {
  format_distance_km,
  format_distance_km_short,
  format_duration,
  format_whole,
  ordinal,
  round_half_even,
  round_pace_s_per_km,
  round_time_s,
  truncate_heart_rate,
} from '../src/lib/units'

describe('the duration', () => {
  it('reads the way a person says it', () => {
    expect(format_duration(22_200)).toBe('6 h 10 m')
    expect(format_duration(2_520)).toBe('42 m')
    expect(format_duration(3_600)).toBe('1 h')
    expect(format_duration(3_660)).toBe('1 h 1 m')
    expect(format_duration(4_020)).toBe('1 h 7 m')
  })

  it('rounds a tie to the even minute — 30 seconds is "0 m", not "1 m"', () => {
    // The core's answer: 30 / 60 = 0.5 minutes, and a tie goes to the even
    // digit, so the duration is zero minutes.
    expect(format_duration(30)).toBe('0 m')
    expect(format_duration(90)).toBe('2 m')
    expect(format_duration(150)).toBe('2 m')
    expect(format_duration(42)).toBe('1 m')
    expect(format_duration(0)).toBe('0 m')
  })
})

describe('a whole number', () => {
  it('rounds a tie to the even digit', () => {
    expect(format_whole(0.5)).toBe('0')
    expect(format_whole(1.5)).toBe('2')
    expect(format_whole(2.5)).toBe('2')
    expect(format_whole(3.5)).toBe('4')
    expect(format_whole(14)).toBe('14')
    expect(format_whole(-14)).toBe('-14')
  })

  it('shows one zero, never a negative zero', () => {
    expect(format_whole(0)).toBe('0')
    expect(format_whole(-0.4)).toBe('0')
  })
})

describe('a heart rate', () => {
  it('is truncated, not rounded', () => {
    expect(truncate_heart_rate(148.7)).toBe(148)
    expect(truncate_heart_rate(149.5)).toBe(149)
    expect(truncate_heart_rate(63)).toBe(63)
  })
})

describe('the rounding', () => {
  it('breaks a tie to the even digit, over the exact value of the double', () => {
    expect(round_half_even(2.5, 0)).toBe(2)
    expect(round_half_even(3.5, 0)).toBe(4)
    expect(round_half_even(0.125, 2)).toBe(0.12)
    expect(round_half_even(12.35, 2)).toBe(12.35)
    expect(round_half_even(1.005, 2)).toBe(1)
    expect(round_half_even(2.675, 2)).toBe(2.67)
    expect(round_half_even(30, 0)).toBe(30)
  })

  it('rounds the times to one decimal and the paces to two', () => {
    // 4_020.35 is not exactly 4_020.35 — the double is a hair below it, so
    // it rounds down. The core says the same.
    expect(round_time_s(4_020.25)).toBe(4_020.2)
    expect(round_time_s(4_020.35)).toBe(4_020.3)
    expect(round_time_s(4_020.05)).toBe(4_020.1)
    expect(round_pace_s_per_km(317.125)).toBe(317.12)
    expect(round_pace_s_per_km(317.135)).toBe(317.13)
  })
})

describe('a distance', () => {
  it('drops the decimals a person would leave out', () => {
    expect(format_distance_km_short(12_400)).toBe('12.4')
    expect(format_distance_km_short(5_000)).toBe('5')
    expect(format_distance_km_short(8_400)).toBe('8.4')
    expect(format_distance_km_short(10_000)).toBe('10')
    expect(format_distance_km_short(16_100)).toBe('16.1')
  })

  it('keeps them when they are asked for', () => {
    expect(format_distance_km(5_000, 1)).toBe('5.0')
    expect(format_distance_km(12_400, 1)).toBe('12.4')
  })
})

describe('an ordinal', () => {
  it('is the English one', () => {
    expect(ordinal(1)).toBe('1st')
    expect(ordinal(2)).toBe('2nd')
    expect(ordinal(3)).toBe('3rd')
    expect(ordinal(4)).toBe('4th')
    expect(ordinal(11)).toBe('11th')
    expect(ordinal(21)).toBe('21st')
  })
})
