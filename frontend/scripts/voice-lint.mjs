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
 * The contract's list, word for word:
 *
 *   should   must      take the day   rest day   rest up   rest today   go easy
 *   missed   bad       poor           fail       behind    good        keep it easy
 *   ready to recommended   better to  need to   red        readiness score
 *   not recovered   you are ready   you are not ready
 *
 * `rest` is banned as a *command* — `rest day`, `rest up`, `rest today` —
 * and never as the noun: the "Rest" summary label on Today is the name of
 * a row, not an instruction.
 */
export const BANNED = [
  { term: 'should', pattern: /\bshould\b/i },
  { term: 'must', pattern: /\bmust\b/i },
  { term: 'take the day', pattern: /\btake\s+the\s+day\b/i },
  { term: 'rest day', pattern: /\brest\s+day\b/i },
  { term: 'rest up', pattern: /\brest\s+up\b/i },
  { term: 'rest today', pattern: /\brest\s+today\b/i },
  { term: 'go easy', pattern: /\bgo\s+easy\b/i },
  { term: 'keep it easy', pattern: /\bkeep\s+it\s+easy\b/i },
  { term: 'missed', pattern: /\bmissed\b/i },
  { term: 'bad', pattern: /\bbad\b/i },
  { term: 'poor', pattern: /\bpoor\b/i },
  { term: 'fail', pattern: /\bfail/i },
  { term: 'behind', pattern: /\bbehind\b/i },
  { term: 'good', pattern: /\bgood\b/i },
  { term: 'red', pattern: /\bred\b/i },
  { term: 'readiness score', pattern: /\breadiness\s+score\b/i },
  { term: 'not recovered', pattern: /\bnot\s+recovered\b/i },
  { term: 'you are ready', pattern: /\byou\s+are\s+ready\b/i },
  { term: 'you are not ready', pattern: /\byou\s+are\s+not\s+ready\b/i },
  { term: 'ready to', pattern: /\bready\s+to\b/i },
  { term: 'recommended', pattern: /\brecommended\b/i },
  { term: 'better to', pattern: /\bbetter\s+to\b/i },
  { term: 'need to', pattern: /\bneed\s+to\b/i },
]

/**
 * The words that name a vital.
 *
 * There is no list here on purpose: the core's export carries the vital
 * words — `crates/stride-core/strings.json`, `rules.vital_words` — and an
 * `option` may not mention one. A second copy of the list is how the two
 * drift, and the two have already drifted once.
 */

/** A sample size is a number — written out or carried in a placeholder. */
export const SAMPLE_SIZE = /\b\d+\b|\{n\}/
export const SAMPLE_NOUN =
  /\b(times|nights|days|runs|weeks|sessions|months|cases|comparables)\b/

/** A hole in a sentence: a placeholder waiting for a value. */
export const HOLE = /\{[^{}]*\}/

/** The banned words inside a piece of text. */
export function bannedWords(text) {
  const found = []
  for (const { term, pattern } of BANNED) {
    const matcher = new RegExp(pattern.source, 'gi')
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
 * closed set; `vitalWords` is the core's list of the words that name a
 * vital — read from the export, never written out here.
 */
export function checkCatalogue(entries, directionWords, vitalWords = []) {
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

  // The closed direction set — for the catalogues that carry one. The
  // app's own catalogue no longer does: the direction words arrive
  // rendered, and the core's export is the one that holds them.
  const directions = Object.entries(entries).filter(([, entry]) => entry.type === 'direction')
  if (directions.length > 0) {
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
  }

  // A pattern carries its sample size.
  for (const [key, entry] of Object.entries(entries)) {
    if (entry.type !== 'pattern') continue
    if (!SAMPLE_SIZE.test(entry.text) || !SAMPLE_NOUN.test(entry.text)) {
      failures.push(`catalogue.${key}: is a pattern without a sample size — "${entry.text}"`)
    }
  }

  // A statement is a whole sentence: it never carries a hole.
  //
  // The values a statement talks about are already in it — "2.4 km more
  // than the 10 km target" — and a hole is where a second implementation
  // grows back: a template that some other screen fills in differently.
  for (const [key, entry] of Object.entries(entries)) {
    if (entry.type !== 'statement') continue
    if (HOLE.test(entry.text)) {
      failures.push(`catalogue.${key}: is a statement with a hole in it — "${entry.text}"`)
    }
  }

  // An option never mentions a vital.
  for (const [key, entry] of Object.entries(entries)) {
    if (entry.type !== 'option') continue
    const lowered = entry.text.toLowerCase()
    for (const vital of vitalWords) {
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

/**
 * Check the core's export — `crates/stride-core/strings.json`.
 *
 * The export carries the catalogue's strings *and the rules with them*:
 * the ban list, the closed direction set, the minimum pattern sample and
 * the vital words. It is the artefact the contract names, and it is
 * regenerated, never hand-edited:
 *
 *     cargo run -p stride-core --example export_catalogue
 *
 * So the export is checked against the same rules the app's own catalogue
 * is checked against, and its ban list is checked against the contract's:
 * if the two ever disagree, the contract's list wins and the export is
 * regenerated.
 */
export function checkExport(exported, directionWords) {
  const failures = []
  const rules = exported?.rules ?? {}
  const entries = exported?.entries ?? []

  // The ban list is one list: the contract's, word for word.
  const exportList = rules.banned_phrases ?? []
  const contractList = BANNED.map((entry) => entry.term)
  const missing = contractList.filter((term) => !exportList.includes(term))
  const extra = exportList.filter((term) => !contractList.includes(term))
  if (missing.length > 0) {
    failures.push(`the export's ban list is missing: ${missing.join(', ')}`)
  }
  if (extra.length > 0) {
    failures.push(`the export's ban list has words the contract does not: ${extra.join(', ')}`)
  }

  // The closed direction set is the contract's, and it is not extended.
  const exportSet = rules.direction_set ?? []
  const sameSet =
    exportSet.length === directionWords.length &&
    directionWords.every((word) => exportSet.includes(word))
  if (!sameSet) {
    failures.push(`the export's direction set is not the contract's: ${exportSet.join(' · ')}`)
  }

  // No banned word in any exported string.
  for (const entry of entries) {
    for (const term of bannedWords(entry.template ?? '')) {
      failures.push(`the export's ${entry.id}: contains the banned word "${term}"`)
    }
  }

  // A pattern carries its sample size.
  for (const entry of entries) {
    if (entry.kind !== 'pattern') continue
    if (!SAMPLE_SIZE.test(entry.template ?? '') || !SAMPLE_NOUN.test(entry.template ?? '')) {
      failures.push(`the export's ${entry.id}: is a pattern without a sample size`)
    }
  }

  // An option never mentions a vital — and the list is the export's.
  const vitalWords = rules.vital_words
  if (!Array.isArray(vitalWords) || vitalWords.length === 0) {
    failures.push('the export does not carry the words that name a vital')
  }
  for (const entry of entries) {
    if (entry.kind !== 'option') continue
    const lowered = (entry.template ?? '').toLowerCase()
    for (const vital of vitalWords ?? []) {
      if (lowered.includes(vital)) {
        failures.push(`the export's ${entry.id}: is an option that mentions a vital ("${vital}")`)
      }
    }
  }

  return failures
}
