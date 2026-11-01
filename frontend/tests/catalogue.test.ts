/**
 * The catalogue's rules cannot drift.
 *
 * Three lists have to agree, and the contract is the one that wins:
 *
 *   docs/contract/voice.md      — the contract's ban list, in words
 *   frontend/scripts/voice-lint.mjs — the lint's transcription of it
 *   crates/stride-core/strings.json — the core's export, regenerated with
 *                                    `cargo run -p stride-core --example export_catalogue`
 *
 * A ban list is one list. Two of them is how a lint ends up passing a
 * string the contract bans, so the three are compared here.
 */

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { BANNED, checkExport } from '../scripts/voice-lint.mjs'
import { DIRECTION_WORDS } from '../src/i18n'

const here = dirname(fileURLToPath(import.meta.url))
const repo = join(here, '..', '..')

const contract = readFileSync(join(repo, 'docs/contract/voice.md'), 'utf8')
const exported = JSON.parse(readFileSync(join(repo, 'crates/stride-core/strings.json'), 'utf8'))

/** The contract's ban list, as the fenced block under "The lint fails…". */
function contractBlock(): string {
  const match = contract.match(
    /The lint fails the build on any of these words or constructions:\s*```([\s\S]*?)```/,
  )
  if (!match || !match[1]) throw new Error('the contract has no ban list block')
  return match[1]
}

/** The lint's terms, in the order the lint holds them. */
const lintTerms = BANNED.map((entry) => entry.term)

describe('the ban list', () => {
  it('is the contract\'s list, word for word, in the export', () => {
    expect([...(exported.rules?.banned_phrases ?? [])].sort()).toEqual([...lintTerms].sort())
  })

  it('names every word the contract names', () => {
    const block = contractBlock().replace(/\s+/g, ' ')
    for (const term of lintTerms) {
      expect(block, `the contract does not name "${term}"`).toContain(term)
    }
  })

  it('names nothing the contract does not', () => {
    // Take the contract's block and remove every term the lint knows,
    // longest first — "readiness score" before "red". Whatever is left
    // is a word the contract bans and the lint does not.
    let rest = contractBlock()
    for (const term of [...lintTerms].sort((a, b) => b.length - a.length)) {
      rest = rest.replace(new RegExp(`\\b${term.replace(/ /g, '\\s+')}\\b`, 'gi'), ' ')
    }
    expect(rest.replace(/[\s`,.]/g, '')).toBe('')
  })
})

describe('the closed direction set', () => {
  it('is the contract\'s set in the export', () => {
    expect([...(exported.rules?.direction_set ?? [])].sort()).toEqual(
      [...DIRECTION_WORDS].sort(),
    )
  })

  it('is not extended by the export', () => {
    expect(exported.rules?.direction_set).toHaveLength(DIRECTION_WORDS.length)
  })
})

describe('the export', () => {
  it('passes the same rules the app\'s catalogue passes', () => {
    expect(checkExport(exported, DIRECTION_WORDS)).toEqual([])
  })

  it('holds the domain sentences the app no longer owns', () => {
    const ids = (exported.entries ?? []).map((entry: { id: string }) => entry.id)
    expect(ids).toContain('direction.travel.up')
    expect(ids).toContain('direction.travel.down')
    expect(ids).toContain('rest.count.below')
    expect(ids).toContain('pattern.usual_streak')
    expect(ids).toContain('pattern.comparable_outcome')
  })

  it('states the travel in the ordinal form, for both directions', () => {
    const travel = (exported.entries ?? []).filter(
      (entry: { id: string }) => entry.id.startsWith('direction.travel.'),
    )
    const templates = travel.map((entry: { template: string }) => entry.template)
    expect(templates).toContain('{ordinal} night down')
    expect(templates).toContain('{ordinal} night up')
    expect(templates).toContain('steady')
  })
})
