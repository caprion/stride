/**
 * The voice rules, as code.
 *
 * The lint is a function so it can be tested against deliberately bad
 * catalogues: a scanner that never fails is worse than no scanner.
 *
 * Plain JavaScript on purpose — the CLI runs it directly, and the unit
 * test imports it.
 */

/**
 * The ban list, from `docs/contract/voice.md`.
 *
 * `caseSensitive` marks a term that is only banned in the lower case. The
 * one exception is "rest": the plan's own Today screen names its composite
 * summary "Rest" (a label for a measurement), while "rest" in the lower
 * case is the instruction the ban list is about.
 */
export const BANNED = [
  { term: 'should', pattern: /\bshould\b/i },
  { term: 'must', pattern: /\bmust\b/i },
  { term: 'take the day', pattern: /\btake\s+the\s+day\b/i },
  { term: 'rest', pattern: /\brest\b/, caseSensitive: true },
  { term: 'go easy', pattern: /\bgo\s+easy\b/i },
  { term: 'missed', pattern: /\bmissed\b/i },
  { term: 'bad', pattern: /\bbad\b/i },
  { term: 'poor', pattern: /\bpoor\b/i },
  { term: 'fail', pattern: /\bfail/i },
  { term: 'behind', pattern: /\bbehind\b/i },
  { term: 'ready to', pattern: /\bready\s+to\b/i },
  { term: 'recommended', pattern: /\brecommended\b/i },
  { term: 'better to', pattern: /\bbetter\s+to\b/i },
  { term: 'need to', pattern: /\bneed\s+to\b/i },
]

/** The words that name a vital. An `option` may not mention one. */
export const VITAL_WORDS = ['sleep', 'hrv', 'resting hr', 'resting heart rate', 'stress']

/** A sample size is a number — written out or carried in a placeholder. */
export const SAMPLE_SIZE = /\b\d+\b|\{n\}/
export const SAMPLE_NOUN =
  /\b(times|nights|days|runs|weeks|sessions|months|cases|comparables)\b/

/** The banned words inside a piece of text. */
export function bannedWords(text) {
  const found = []
  for (const { term, pattern, caseSensitive } of BANNED) {
    const matcher = caseSensitive ? new RegExp(pattern.source, 'g') : pattern
    const hit = matcher.test(text)
    matcher.lastIndex = 0
    if (hit) found.push(term)
  }
  return found
}

/**
 * Check a catalogue.
 *
 * `entries` is a map of key → { type, text }; `directionWords` is the
 * closed set.
 */
export function checkCatalogue(entries, directionWords) {
  const failures = []

  for (const [key, entry] of Object.entries(entries)) {
    if (!entry || typeof entry.text !== 'string') {
      failures.push(`catalogue.${key}: has no text`)
      continue
    }
    for (const term of bannedWords(entry.text)) {
      failures.push(`catalogue.${key}: contains the banned word "${term}" — "${entry.text}"`)
    }
  }

  // The closed direction set.
  const directions = Object.entries(entries).filter(([, entry]) => entry.type === 'direction')
  for (const [key, entry] of directions) {
    if (!directionWords.includes(entry.text)) {
      failures.push(`catalogue.${key}: is a direction outside the closed set — "${entry.text}"`)
    }
  }
  const seen = new Set(directions.map(([, entry]) => entry.text))
  for (const word of directionWords) {
    if (!seen.has(word)) {
      failures.push(`catalogue: the closed direction set is missing "${word}"`)
    }
  }
  if (seen.size !== directionWords.length) {
    failures.push('catalogue: the closed direction set has been extended')
  }

  // A pattern carries its sample size.
  for (const [key, entry] of Object.entries(entries)) {
    if (entry.type !== 'pattern') continue
    if (!SAMPLE_SIZE.test(entry.text) || !SAMPLE_NOUN.test(entry.text)) {
      failures.push(`catalogue.${key}: is a pattern without a sample size — "${entry.text}"`)
    }
  }

  // An option never mentions a vital.
  for (const [key, entry] of Object.entries(entries)) {
    if (entry.type !== 'option') continue
    const lowered = entry.text.toLowerCase()
    for (const vital of VITAL_WORDS) {
      if (lowered.includes(vital)) {
        failures.push(
          `catalogue.${key}: is an option that mentions a vital ("${vital}") — "${entry.text}"`,
        )
      }
    }
  }

  return failures
}

/** Check prose written inline in a component. */
export function checkInlineText(file, text) {
  const failures = []
  const jsxText = />([^<>]+)</g
  let match
  while ((match = jsxText.exec(text)) !== null) {
    const literal = match[1].trim()
    if (!/[A-Za-z]{3}/.test(literal)) continue
    // An expression, not prose: `{t('...')}` is a catalogue lookup, and
    // the catalogue is linted on its own.
    if (literal.includes('{') || literal.includes('}')) continue
    for (const term of bannedWords(literal)) {
      failures.push(`${file}: writes the banned word "${term}" inline — "${literal}"`)
    }
  }
  return failures
}
