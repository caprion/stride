/**
 * The units, the rounding and the human rendering.
 *
 * This is a port of `crates/stride-core/src/units.rs` — the same names, the
 * same rules, the same answers. Contract §8 gives the rounding to the
 * core: "a second implementation of 'how do we say 30 seconds' is the same
 * disease as a second implementation of the FIT decoder: it will disagree,
 * and it will disagree quietly."
 *
 * So this file is the *only* place in the app that rounds a number, and it
 * is a copy of the core's rules rather than a second set of them. Change a
 * rule here and in `units.rs` in the same commit; `tests/units.test.ts`
 * holds the two to the same answers.
 *
 * The API may send a value already rendered ("6 h 10 m") — when it does,
 * the app shows it as it stands and this file is not consulted. The file is
 * for the numbers the API sends as numbers.
 */

/**
 * The exact decimal value of a double.
 *
 * A double is a binary fraction, so "2.5" is exactly 2.5 but "0.1" is not
 * exactly a tenth. Rust's `{:.N}` formats the *exact* binary value and
 * breaks a tie to the even digit, which is what Python's `round()` does.
 * JavaScript's `toFixed` breaks a tie the other way (2.5 → "3"), so
 * `toFixed` cannot stand in here.
 *
 * The expansion is exact: `digits × 10^exponent` is the double's value,
 * with no rounding at all.
 */
function exactDecimal(value: number): { digits: bigint; exponent: number } {
  const view = new DataView(new ArrayBuffer(8))
  view.setFloat64(0, value)
  const high = view.getUint32(0)
  const low = view.getUint32(4)
  const negative = (high & 0x8000_0000) !== 0
  const biased = (high >>> 20) & 0x7ff

  let mantissa = (BigInt(high & 0x1f_ffff) << 32n) | BigInt(low)
  let exponent: number
  if (biased === 0) {
    // A subnormal: no implicit leading bit.
    exponent = -1074
  } else {
    mantissa |= 1n << 52n
    exponent = biased - 1075
  }

  const sign = negative ? -1n : 1n
  if (exponent >= 0) {
    return { digits: sign * mantissa * (1n << BigInt(exponent)), exponent: 0 }
  }

  // 2^-shift is 5^shift × 10^-shift, so the fraction becomes a decimal one.
  const shift = BigInt(-exponent)
  const divisor = 1n << shift
  const whole = mantissa / divisor
  const fraction = (mantissa % divisor) * 5n ** shift
  return { digits: sign * (whole * 10n ** shift + fraction), exponent: -Number(shift) }
}

/**
 * Round half to even at `decimals` places.
 *
 * The same rule as the core's `round_half_even`, and the same rule as
 * Python's `round(value, digits)`: the *decimal* value of the double is
 * rounded, and an exact tie goes to the even digit.
 */
export function round_half_even(value: number, decimals: number): number {
  if (!Number.isFinite(value)) return value

  const { digits, exponent } = exactDecimal(Math.abs(value))
  const shift = exponent + decimals

  let numerator: bigint
  let denominator: bigint
  if (shift >= 0) {
    numerator = digits * 10n ** BigInt(shift)
    denominator = 1n
  } else {
    numerator = digits
    denominator = 10n ** BigInt(-shift)
  }

  let quotient = numerator / denominator
  const remainder = numerator % denominator
  const twice = remainder * 2n
  if (twice > denominator || (twice === denominator && quotient % 2n !== 0n)) {
    quotient += 1n
  }

  const scaled = Number(quotient)
  const rounded = decimals > 0 ? scaled / 10 ** decimals : scaled
  return value < 0 ? -rounded : rounded
}

/** Contract §3: the times to one decimal. */
export function round_time_s(value: number): number {
  return round_half_even(value, 1)
}

/** Contract §3: a distance to one decimal. */
export function round_distance_m(value: number): number {
  return round_half_even(value, 1)
}

/** Contract §3: a lap's distance to two decimals. */
export function round_lap_distance_m(value: number): number {
  return round_half_even(value, 2)
}

/** Contract §3: a cadence to one decimal. */
export function round_cadence(value: number): number {
  return round_half_even(value, 1)
}

/** Contract §3: a pace to two decimals. */
export function round_pace_s_per_km(value: number): number {
  return round_half_even(value, 2)
}

/** Contract §3: GPS degrees to seven decimals. */
export function round_gps_degrees(value: number): number {
  return round_half_even(value, 7)
}

/** Contract §3: a heart rate is truncated to an integer — not rounded. */
export function truncate_heart_rate(value: number): number {
  return Math.trunc(value)
}

/** Contract §3: the distance sums Progress reports to two decimals. */
export function round_km(value: number): number {
  return round_half_even(value, 2)
}

/**
 * The human rendering of a duration (contract §5).
 *
 * "6 h 10 m", not "6.166 h". Under an hour: "42 m". Minutes are rounded to
 * the nearest minute, and an exact zero minutes is left out — a person says
 * "1 h", not "1 h 00 m".
 */
export function format_duration(seconds: number): string {
  if (!Number.isFinite(seconds)) return ''
  const minutes = Math.trunc(round_half_even(seconds / 60, 0))
  const sign = minutes < 0 ? '-' : ''
  const absolute = Math.abs(minutes)
  const hours = Math.floor(absolute / 60)
  const rest = absolute % 60
  if (hours === 0) return `${sign}${rest} m`
  if (rest === 0) return `${sign}${hours} h`
  return `${sign}${hours} h ${rest} m`
}

/** The human rendering of a distance, in kilometres. */
export function format_distance_km(metres: number, decimals: number): string {
  if (!Number.isFinite(metres)) return ''
  return round_half_even(metres / 1000, decimals).toFixed(decimals)
}

/**
 * The human rendering of a distance, in kilometres, without the decimals a
 * person would leave out: "6 km", "8.4 km".
 */
export function format_distance_km_short(metres: number): string {
  if (!Number.isFinite(metres)) return ''
  const text = round_half_even(metres / 1000, 1).toFixed(1)
  return text.endsWith('.0') ? text.slice(0, -2) : text
}

/** A number in its own scale, with no decimals: "48", "63". */
export function format_whole(value: number): string {
  if (!Number.isFinite(value)) return ''
  const rounded = round_half_even(value, 0)
  if (rounded === 0) {
    // `-0` and `0` are the same number; only one of them may be shown.
    return '0'
  }
  return String(rounded)
}

/** The ordinal of a count: 1st, 2nd, 3rd, 4th, 11th, 21st. */
export function ordinal(count: number): string {
  const suffix =
    count % 100 >= 11 && count % 100 <= 13
      ? 'th'
      : count % 10 === 1
        ? 'st'
        : count % 10 === 2
          ? 'nd'
          : count % 10 === 3
            ? 'rd'
            : 'th'
  return `${count}${suffix}`
}
