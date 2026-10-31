/**
 * The voice lint, against deliberately bad catalogues.
 *
 * A scanner that never fails is worse than no scanner, so the bad cases
 * are asserted here.
 */

import { describe, expect, it } from 'vitest'
import { checkCatalogue, checkInlineText } from '../scripts/voice-lint.mjs'
import { DIRECTION_WORDS, catalogue, directions } from '../src/i18n'

const entries = { ...catalogue, ...directions }

describe('the voice lint', () => {
  it('passes the real catalogue', () => {
    expect(checkCatalogue(entries, DIRECTION_WORDS)).toEqual([])
  })

  it('fails on a banned word', () => {
    const failures = checkCatalogue(
      { ...entries, 'x.y': { type: 'status', text: 'You should rest today' } },
      DIRECTION_WORDS,
    )
    expect(failures.join('\n')).toContain('should')
  })

  it('fails on a direction outside the closed set', () => {
    const failures = checkCatalogue(
      { ...entries, 'x.direction': { type: 'direction', text: 'not great' } },
      DIRECTION_WORDS,
    )
    expect(failures.join('\n')).toContain('closed set')
  })

  it('fails when the closed direction set is extended', () => {
    const failures = checkCatalogue(
      { ...entries, 'x.direction': { type: 'direction', text: 'slightly below usual' } },
      DIRECTION_WORDS,
    )
    expect(failures.join('\n')).toContain('closed set')
  })

  it('fails on a pattern without a sample size', () => {
    const failures = checkCatalogue(
      { ...entries, 'x.pattern': { type: 'pattern', text: 'your HRV has been low lately' } },
      DIRECTION_WORDS,
    )
    expect(failures.join('\n')).toContain('sample size')
  })

  it('fails on an option that mentions a vital', () => {
    const failures = checkCatalogue(
      {
        ...entries,
        'x.option': { type: 'option', text: 'a 5 km, because your sleep was low' },
      },
      DIRECTION_WORDS,
    )
    expect(failures.join('\n')).toContain('mentions a vital')
  })

  it('fails on a banned word written inline in a component', () => {
    const failures = checkInlineText('src/pages/Example.tsx', '<p>You should take the day</p>')
    expect(failures.join('\n')).toContain('should')
  })

  it('does not flag a catalogue lookup as inline prose', () => {
    const failures = checkInlineText('src/pages/Example.tsx', "<p>{t('today.rest.label')}</p>")
    expect(failures).toEqual([])
  })
})
