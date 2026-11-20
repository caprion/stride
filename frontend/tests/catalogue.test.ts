/**
 * The catalogue's rules cannot drift.
 *
 * The list of banned words is the core's: it lives in `stride-core`,
 * beside the catalogue it holds, and it travels to the app in the export
 * (`crates/stride-core/strings.json`). The app's lint reads it from
 * there, and the reviewer's check — the core's own — reads the same
 * list, so the two cannot drift apart.
 *
 * What is compared here is the export against the contract
 * (`docs/contract/voice.md`), which is where the words are written down.
 */

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { bannedTerms, checkContract, checkExport } from '../scripts/voice-lint.mjs'
import { DIRECTION_WORDS } from '../src/i18n'

const here = dirname(fileURLToPath(import.meta.url))
const repo = join(here, '..', '..')

const contract = readFileSync(join(repo, 'docs/contract/voice.md'), 'utf8')
const exported = JSON.parse(readFileSync(join(repo, 'crates/stride-core/strings.json'), 'utf8'))

/** The core's ban list, read out of the export. */
const terms = bannedTerms(exported)

describe('the ban list', () => {
  it('is the core\'s list, and the export carries it', () => {
    expect(terms.length).toBeGreaterThan(0)
    expect([...(exported.rules?.banned_phrases ?? [])].sort()).toEqual([...terms].sort())
  })

  it('is the contract\'s list, word for word', () => {
    expect(checkContract(contract, terms)).toEqual([])
  })

  it('names every word the contract names', () => {
    const failures = checkContract(contract, terms)
    expect(failures).toEqual([])
  })

  it('names nothing the contract does not', () => {
    // The contract's block and the list are compared in both directions:
    // a word the contract bans and the list does not is a failure, and so
    // is a word the list bans and the contract does not name.
    const failures = checkContract(contract, terms)
    expect(failures).toEqual([])
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
    expect(checkExport(exported, DIRECTION_WORDS, terms)).toEqual([])
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

  it('holds the run update\'s prompt, and the payload\'s labels', () => {
    const ids = (exported.entries ?? []).map((entry: { id: string }) => entry.id)
    expect(ids).toContain('update.prompt.writer')
    expect(ids).toContain('update.prompt.reviewer')
    expect(ids).toContain('update.prompt.repair')
    expect(ids).toContain('update.payload.distance')
    expect(ids).toContain('update.check.groundedness')
  })
})
