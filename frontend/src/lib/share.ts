/**
 * The comparison, as a file the athlete can keep.
 *
 * The plan's action is "share the comparison as an image or HTML file",
 * and the file is the app's own: the basis, the metric diff and the two
 * runs, in the core's words. It is the athlete's own data and it goes
 * nowhere — the app hands it to the host, and the host hands it to the
 * athlete.
 *
 * Every word in the document is the core's or the catalogue's: the
 * app does not write a sentence of its own into a file the athlete will
 * read.
 */

import type { CompareResponse } from './types'
import { renderedText } from './format'
import { t } from '../i18n'

/** The characters that would break the document. */
function escape(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

/** The comparison as an HTML document. */
export function comparisonDocument(comparison: CompareResponse): string {
  const rows = comparison.metrics
    .map(
      (metric) => `      <tr>
        <td>${escape(t(metric.label_key))}</td>
        <td>${escape(renderedText(metric.a))}</td>
        <td>${escape(renderedText(metric.b))}</td>
        <td>${escape(renderedText(metric.delta))}</td>
      </tr>`,
    )
    .join('\n')

  const run = (key: 'a' | 'b') => {
    const value = comparison[key]
    const label = key === 'a' ? t('compare.column.a') : t('compare.column.b')
    return `    <h2>${escape(label)}</h2>
    <p>${escape(renderedText(value.distance))} · ${escape(value.date)}</p>
${value.story ? `    <p>${escape(value.story)}</p>\n` : ''}`
  }

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>${escape(t('compare.heading'))}</title>
  </head>
  <body>
    <h1>${escape(t('compare.heading'))}</h1>
    <p>${escape(comparison.basis_text)}</p>
    <table>
      <thead>
        <tr>
          <th>${escape(t('compare.column.metric'))}</th>
          <th>${escape(t('compare.column.a'))}</th>
          <th>${escape(t('compare.column.b'))}</th>
          <th>${escape(t('compare.column.delta'))}</th>
        </tr>
      </thead>
      <tbody>
${rows}
      </tbody>
    </table>
${run('a')}
${run('b')}
    <p>${escape(t('common.promise'))}</p>
  </body>
</html>
`
}
