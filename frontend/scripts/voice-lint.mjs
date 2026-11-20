/**
 * The voice rules, as code.
 *
 * The lint is a function so it can be tested against deliberately bad
 * catalogues: a scanner that never fails is worse than no scanner.
 *
 * Plain JavaScript on purpose — the CLI runs it directly, and the unit
 * test imports it.
 *
 * ## One list, two readers
 *
 * **The ban list is the core's.** It lives in `stride-core`, beside the
 * catalogue it holds, and it travels to the app in the export
 * (`crates/stride-core/strings.json`). The app's lint reads it from
 * there, and the reviewer's check reads the same list out of the core
 * itself — so the app's lint and the model's reviewer cannot drift
 * apart, because there is only one list.
 *
 * The list is compared with the contract's own (`docs/contract/voice.md`)
 * by the lint and by the core's test, so the file that freezes the words
 * still holds them.
 */

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

/**
 * The ban list, out of the core's export.
 *
 * The export is the one list: the core holds it beside the catalogue and
 * sends it here with the strings.
 */
export function bannedTerms(exported) {
  return exported?.rules?.banned_phrases ?? []
}

/**
 * The pattern of a banned phrase.
 *
 * **On word boundaries**, the way the core matches it: "resting" is not
 * the command "rest today", and "mustard" is not "must". The core's own
 * matcher is the rule, and the app follows it — that is the whole point
 * of the one list.
 */
export function patternFor(term) {
  const escaped = term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return new RegExp(`\\b${escaped.replace(/\s+/g, '\\s+')}\\b`, 'i')
}

/** The banned words inside a piece of text. */
export function bannedWords(text, terms) {
  const found = []
  for (const term of terms ?? []) {
    if (patternFor(term).test(text)) found.push(term)
  }
  return found
}

/**
 * Check a catalogue.
 *
 * `entries` is a map of key → { type, text }; `directionWords` is the
 * closed set; `vitalWords` is the core's list of the words that name a
 * vital — read from the export, never written out here; and `terms` is
 * the core's ban list, read from the same export.
 */
export function checkCatalogue(entries, directionWords, vitalWords = [], terms = []) {
  const failures = []

  for (const [key, entry] of Object.entries(entries)) {
    if (!entry || typeof entry.text !== 'string') {
      failures.push(`catalogue.${key}: has no text`)
      continue
    }
    for (const term of bannedWords(entry.text, terms)) {
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
export function checkInlineText(file, text, terms = []) {
  const failures = []
  const jsxText = />([^<>]+)</g
  let match
  while ((match = jsxText.exec(text)) !== null) {
    const literal = match[1].trim()
    if (!/[A-Za-z]{3}/.test(literal)) continue
    // An expression, not prose: `{t('...')}` is a catalogue lookup, and
    // the catalogue is linted on its own.
    if (literal.includes('{') || literal.includes('}')) continue
    for (const term of bannedWords(literal, terms)) {
      failures.push(`${file}: writes the banned word "${term}" inline — "${literal}"`)
    }
  }
  return failures
}

/**
 * Check the core's export — `crates/stride-core/strings.json`.
 *
 * The export carries the catalogue's strings *and the rules with them*:
 * the ban list, the closed direction set, the minimum pattern sample
 * and the vital words. It is the artefact the contract names, and it is
 * regenerated, never hand-edited:
 *
 *     cargo run -p stride-core --example export_catalogue
 *
 * So the export is checked against the same rules the app's own catalogue
 * is checked against, and its ban list is checked against the contract's:
 * if the two ever disagree, the contract's list wins and the export is
 * regenerated.
 */
export function checkExport(exported, directionWords, terms = []) {
  const failures = []
  const rules = exported?.rules ?? {}
  const entries = exported?.entries ?? []

  // The ban list is one list: the core's, read out of the export.
  const exportList = rules.banned_phrases ?? []
  if (exportList.length === 0) {
    failures.push('the export carries no ban list')
  }
  const missing = terms.filter((term) => !exportList.includes(term))
  const extra = exportList.filter((term) => !terms.includes(term))
  if (missing.length > 0) {
    failures.push(`the export's ban list is missing: ${missing.join(', ')}`)
  }
  if (extra.length > 0) {
    failures.push(`the export's ban list has words the list does not: ${extra.join(', ')}`)
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
    for (const term of bannedWords(entry.template ?? '', terms)) {
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

/**
 * Check the contract's own list against the core's.
 *
 * The contract (`docs/contract/voice.md`) is where the words are written
 * down, and the core's list is where they are kept. The two are compared
 * here, so the file that freezes the words still holds them.
 */
export function checkContract(contract, terms) {
  const failures = []
  const marker = 'The lint fails the build on any of these words or constructions:'
  const at = contract.indexOf(marker)
  if (at < 0) {
    failures.push('the contract has no ban list')
    return failures
  }
  const rest = contract.slice(at + marker.length)
  const open = rest.indexOf('```')
  const close = open < 0 ? -1 : rest.indexOf('```', open + 3)
  if (open < 0 || close < 0) {
    failures.push('the contract has no ban list block')
    return failures
  }
  const block = rest.slice(open + 3, close)

  for (const term of terms) {
    if (!block.includes(term)) {
      failures.push(`the contract does not name the banned word "${term}"`)
    }
  }
  // And the contract names nothing the list does not.
  let left = block
  for (const term of [...terms].sort((a, b) => b.length - a.length)) {
    left = left.replace(new RegExp(`\\b${term.replace(/ /g, '\\s+')}\\b`, 'gi'), ' ')
  }
  const rest2 = left.replace(/[\s`,.]/g, '')
  if (rest2.length > 0) {
    failures.push(`the contract bans words the list does not: ${rest2}`)
  }
  return failures
}
