/**
 * The heatmap — a grid over the stored positions.
 *
 * **Made on this device.** The grid is computed from the runs the store
 * holds, decoded from their own FIT files, and nothing is fetched for it
 * and nothing is sent with it. The disclosure under the heading says so,
 * in the app's own words, and it is true.
 *
 * **An explicit action opens it.** The heatmap is not on the run list: the
 * Runs screen has a button, and the athlete asks for this the way the
 * athlete asks for the Street map.
 *
 * **It is a place, not a person.** The screen shows a grid of cells and how
 * many runs went through each of them. It names no street, no address and
 * no neighbourhood, it draws no conclusion about the athlete's life, and
 * it never shows a route: a cell is a count of runs, and nothing else.
 */

import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { formatCount, renderedText } from '../lib/format'
import { t } from '../i18n'
import { HeatmapGrid } from '../components/HeatmapGrid'
import { QueryError } from '../components/QueryError'
import { PageHeader } from '../ui/layout/PageHeader'

export function HeatmapPage() {
  const heatmap = useQuery({ queryKey: ['heatmap'], queryFn: api.insightsHeatmap })
  const data = heatmap.data
  const empty = Boolean(data?.unavailable)
  const busiest = data?.cells.reduce((most, cell) => Math.max(most, cell.runs), 0) ?? 0


  return (
    <div>
      <PageHeader
        title={t('heatmap.heading')}
        meta={
          <Link to="/runs" className="tap-inline underline">
            {t('activity.back')}
          </Link>
        }
      />

      {/* The disclosure, always: the aggregation is local, and nothing is
          sent anywhere. */}
      <p className="mb-4 text-[12px] text-ink-muted" data-testid="heatmap-disclosure">
        {t('heatmap.disclosure')}
      </p>

      {heatmap.isError ? (
        <QueryError
          retry={() => {
            void heatmap.refetch()
          }}
        />
      ) : null}

      {heatmap.isPending ? <p className="text-[14px] text-ink-muted">{t('common.loading')}</p> : null}

      {empty ? (
        <div className="card py-8 text-center" data-testid="heatmap-empty">
          <p className="text-[15px] font-semibold text-ink">{t('heatmap.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('heatmap.empty.body')}</p>
        </div>
      ) : null}

      {data && !empty && data.extent ? (
        <div className="heatmap-layout">
          <div className="card overflow-hidden p-2">
            <div className="aspect-square w-full">
              <HeatmapGrid cells={data.cells} extent={data.extent} />
            </div>
          </div>
          <div>
            <p className="text-[12.5px] text-ink-muted" data-testid="heatmap-summary">
              {t('heatmap.summary', {
                runs: formatCount(data.runs),
                cells: formatCount(data.cells.length),
              })}
            </p>
            {data.cell_size ? (
              <p className="mt-1 text-[12px] text-ink-muted" data-testid="heatmap-cell-size">
                {t('heatmap.cellSize', { size: renderedText(data.cell_size) })}
              </p>
            ) : null}
            <ul className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-[12px] text-ink-muted">
              <li className="flex items-center gap-2">
                <span
                  className="inline-block size-3 rounded-xs"
                  style={{ backgroundColor: 'var(--brand-100)' }}
                  aria-hidden="true"
                />
                {t('heatmap.legend.one')}
              </li>
              {busiest > 1 ? (
                <li className="flex items-center gap-2">
                  <span
                    className="inline-block size-3 rounded-xs"
                    style={{ backgroundColor: 'var(--brand-600)' }}
                    aria-hidden="true"
                  />
                  {t('heatmap.legend.many', { n: formatCount(busiest) })}
                </li>
              ) : null}
            </ul>
            {data.runs_outside > 0 ? (
              <p className="mt-1 text-[12px] text-ink-muted" data-testid="heatmap-outside">
                {t('heatmap.outside', { n: formatCount(data.runs_outside) })}
              </p>
            ) : null}
            {data.more_runs ? (
              <p className="mt-3 text-[12px] text-ink-muted" data-testid="heatmap-more-runs">
                {t('heatmap.moreRuns')}
              </p>
            ) : null}
          </div>
        </div>
      ) : null}
    </div>
  )
}
