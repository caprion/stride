/**
 * The athlete profile: **a destination of its own**.
 *
 * **The long pattern.** Progress is the recent work — weeks. The profile
 * is the shape of the training over months and seasons: the shape
 * itself, the goals the athlete set, the schedule the runs follow, the
 * seasons, and the habit.
 *
 * **Two arrangements of one screen**, the same two Progress has: on the
 * wide screens it is the **wall** of the profile's areas, and on a phone
 * it is the **index** — the picture, the questions the long pattern
 * answers, and the areas' links under them.
 *
 * The answers are the ones Progress draws on: the same two hooks, so
 * that a number on the profile and the same number on Progress are one
 * number, and the range on the screen is the window they cover.
 *
 * **And the panels of the profile open under the profile.** A panel the
 * athlete profile owns is at `/profile/<panel>`, and the old
 * `/progress/<panel>` links still open it — a link someone saved last
 * month is still a link someone follows. The panel's back link says
 * where the reader came from.
 */

import { useState, type ReactNode } from 'react'
import { Link, Navigate, useLocation, useNavigate, useParams } from 'react-router-dom'
import { useStaleness } from '../lib/staleness'
import { t } from '../i18n'
import { PageHeader } from '../ui/layout/PageHeader'
import { StreakCard } from '../components/StreakCard'
import { ProfileIndex } from './ProgressIndex'
import { RangeSelector } from './progressRange'
import { ProgressWall } from './progressWall'
import { useProgress } from './useProgress'
import { useWall } from './useWall'
import { runsPath, shiftDate } from '../lib/history'
import { formatCount } from '../lib/format'
import {
  AdherencePanel,
  ChallengePanel,
  EddingtonPanel,
  ProfilePanel,
  ShapePanel,
} from './progressPanels'
import {
  DaytimeTable,
  DistanceCalendar,
  DistanceLegend,
  DistancesTable,
  MonthsChart,
  ProfileRadarBody,
  WeekdayTable,
  YearsChart,
} from './progressWall'
import { WALL_PANELS, panelDestination } from './taskAreas'

/**
 * The panels of the athlete profile.
 *
 * The model's own: the panels the profile owns, and not the work's.
 */
const PROFILE_PANELS = WALL_PANELS.filter((panel) => panelDestination(panel) === 'profile')

/**
 * A panel that is not one of the app's own cards: the wall's own bodies,
 * in a card of their own.
 *
 * The panel route is the doorway into the panel that answers a card of
 * the wall, and it is the same answer in the same card.
 */
function PanelCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="card px-4 py-3">
      <h2 className="mb-2 text-[12px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
        {title}
      </h2>
      {children}
    </section>
  )
}

/**
 * The athlete profile.
 *
 * The wall on the wide screens, the index on the phone — and the range
 * above both, because every answer on this screen follows it.
 */
export function ProfilePage() {
  const [range, setRange] = useState<number>(12)
  const navigate = useNavigate()
  const answers = useProgress(range)
  const wall = useWall(range)
  const stale = useStaleness()
  const { progress, data } = answers

  if (progress.isError && !data) {
    return (
      <div>
        <PageHeader title={t('progress.profile.page')} />
        <section className="card py-4" role="alert">
          <p>{t('common.error')}</p>
          <button
            type="button"
            className="tap mt-2 rounded-md border border-edge px-3"
            onClick={() => {
              void progress.refetch()
            }}
          >
            {t('today.state.retry')}
          </button>
        </section>
      </div>
    )
  }

  if (!data) {
    return (
      <div>
        <PageHeader title={t('progress.profile.page')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  // Nothing to draw yet: the screen says so, and invents no numbers.
  if (data.weekly_volume.length === 0) {
    return (
      <div>
        <PageHeader title={t('progress.profile.page')} />
        <div className="card py-8 text-center" data-testid="profile-empty">
          <p className="text-[15px] font-semibold text-ink">{t('progress.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('progress.empty.body')}</p>
        </div>
      </div>
    )
  }

  return (
    <div>
      <PageHeader
        title={t('progress.profile.page')}
        meta={
          <>
            {t('progress.profile.lede')}
            {stale ? <span className="ml-2 text-ink-faint">{stale}</span> : null}
          </>
        }
      />

      {/* --------------------------------------------- the range selector */}
      {/*
        The window's control, above both arrangements: the range is the
        screen's, and every answer under it follows it.
      */}
      <RangeSelector range={range} onChange={setRange} testId="profile-range" />

      {/* ------------------------------------------------------- the wall */}
      {/*
        The wide arrangement: the wall of the profile's areas, in the
        model's cards. The narrow layout replaces this with the index
        below — one arrangement or the other, never both, and the band
        that decides is the container's.
      */}
      <div className="progress-dashboard" data-testid="profile-dashboard">
        <ProgressWall
          answers={answers}
          wall={wall}
          onWeekClick={(weekStart) => {
            navigate(runsPath(weekStart, shiftDate(weekStart, 6)))
          }}
          destination="profile"
        />
        <p className="mt-4 text-[12.5px] text-ink-faint">
          {formatCount(data.weekly_volume.length)} weeks ·{' '}
          {t('progress.sessionsPerWeek', {
            n: data.sessions_per_week?.text ?? formatCount(null),
          })}
        </p>
        <p className="mt-1 text-[12px] text-ink-muted">{t('progress.averageNote')}</p>
      </div>

      {/* ------------------------------------------------------ the index */}
      {/*
        The narrow arrangement: the picture, the questions the long
        pattern answers, and the areas' links under them.
      */}
      <div className="progress-index" data-testid="profile-index-layout">
        <ProfileIndex
          answers={answers}
          shape={{
            rolling: wall.profile.data,
            months: wall.profileScope.months,
            onScope: wall.setProfileMonths,
          }}
        />
      </div>
    </div>
  )
}

/**
 * One panel of the athlete profile, on its own.
 *
 * The panel route is the doorway into the panel that answers a card of
 * the wall: the **existing panel component**, drawn once, at whatever
 * width the phone is, with the window it covers named on it.
 *
 * **The back link says where the reader came from.** A panel opened from
 * the athlete profile goes back to the athlete profile, and a panel
 * opened from Progress — the old `/progress/<panel>` links, and the
 * questions on Progress that answer a panel of the profile — goes back
 * to Progress, and says so in its own words.
 */
export function ProfilePanelPage() {
  const { panel } = useParams()
  const location = useLocation()
  const [range, setRange] = useState(12)
  const answers = useProgress(range)
  const wall = useWall(range)
  const stale = useStaleness()

  /** Where the reader came from, and the words that go with it. */
  const from = (location.state as { from?: string } | null)?.from
  const fromProgress = from === '/progress'
  const backTo = fromProgress ? '/progress' : '/profile'
  const backLabel = fromProgress ? t('progress.back') : t('progress.profile.back')

  // A name that is not a panel of the profile is not a page: back to
  // where the reader came from.
  if (!PROFILE_PANELS.includes(panel as (typeof PROFILE_PANELS)[number])) {
    return <Navigate to={{ pathname: backTo, search: location.search }} replace />
  }

  const { progress, rolling, streaks, challenges } = answers
  const { year, profile, profileScope, setProfileMonths } = wall

  if (progress.isError && !progress.data) {
    return (
      <div>
        <PageHeader title={t('progress.profile.page')} />
        <section className="card py-4" role="alert">
          <p>{t('common.error')}</p>
        </section>
      </div>
    )
  }

  if (!progress.data) {
    return (
      <div>
        <PageHeader title={t('progress.profile.page')} />
        <p className="text-[14px] text-ink-muted">{t('common.loading')}</p>
      </div>
    )
  }

  // Nothing recorded: the panel says so, and invents no numbers.
  if (progress.data.weekly_volume.length === 0) {
    return (
      <div>
        <PageHeader title={t('progress.profile.page')} />
        <div className="card py-8 text-center" data-testid="profile-empty">
          <p className="text-[15px] font-semibold text-ink">{t('progress.empty.title')}</p>
          <p className="mt-1 text-[13px] text-ink-muted">{t('progress.empty.body')}</p>
        </div>
      </div>
    )
  }

  return (
    <div>
      <Link
        to={backTo}
        className="tap-inline mb-2 text-[13px] text-brand-700 underline"
        data-testid="profile-panel-back"
      >
        {backLabel}
      </Link>
      <PageHeader
        title={t('progress.profile.page')}
        meta={
          <>
            {t('progress.profile.lede')}
            {stale ? <span className="ml-2 text-ink-faint">{stale}</span> : null}
          </>
        }
      />
      <RangeSelector range={range} onChange={setRange} testId="profile-panel-range" />

      {/*
        The panel is one card, and it is capped: a single panel does not
        need the whole of a wide screen.
      */}
      <div className="mt-3 max-w-[680px] space-y-3" data-testid={`profile-panel-${panel}`}>
        {panel === 'shape' ? (
          <ShapePanel
            rolling={profile.data}
            error={profile.isError}
            months={profileScope.months}
            onScope={setProfileMonths}
          />
        ) : null}
        {panel === 'profile' ? (
          <PanelCard title={t('progress.wall.profile.heading')}>
            <ProfileRadarBody
              rolling={profile.data}
              months={profileScope.months}
              onScope={setProfileMonths}
            />
          </PanelCard>
        ) : null}
        {panel === 'facets' ? <ProfilePanel rolling={rolling.data} /> : null}
        {panel === 'challenges' ? (
          <ChallengePanel challenges={challenges.data} />
        ) : null}
        {panel === 'adherence' ? (
          <AdherencePanel adherence={progress.data.adherence} />
        ) : null}
        {panel === 'weekday' ? (
          <PanelCard title={t('progress.wall.weekday.heading')}>
            <WeekdayTable rolling={rolling.data} />
          </PanelCard>
        ) : null}
        {panel === 'daytime' ? (
          <PanelCard title={t('progress.wall.daytime.heading')}>
            <DaytimeTable rolling={rolling.data} />
          </PanelCard>
        ) : null}
        {panel === 'distances' ? (
          <PanelCard title={t('progress.wall.distances.heading')}>
            <DistancesTable rolling={rolling.data} />
          </PanelCard>
        ) : null}
        {panel === 'heatmap' ? (
          <PanelCard title={t('progress.wall.heatmap.heading')}>
            <DistanceCalendar days={year.data?.days ?? []} />
            <DistanceLegend />
          </PanelCard>
        ) : null}
        {panel === 'months' ? (
          <PanelCard title={t('progress.wall.months.heading')}>
            <MonthsChart rolling={rolling.data} />
          </PanelCard>
        ) : null}
        {panel === 'year' ? (
          <PanelCard title={t('progress.wall.years.heading')}>
            <YearsChart rolling={rolling.data} />
          </PanelCard>
        ) : null}
        {panel === 'habit' ? (
          <>
            {streaks.data ? (
              <StreakCard
                streaks={streaks.data}
                className="card py-3"
                testId="progress-streaks"
              />
            ) : null}
            <EddingtonPanel data={progress.data} />
          </>
        ) : null}
        {panel === 'eddington' ? <EddingtonPanel data={progress.data} /> : null}
      </div>
    </div>
  )
}
