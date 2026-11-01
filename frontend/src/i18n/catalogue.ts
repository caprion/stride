/**
 * The app's string catalogue — the chrome.
 *
 * Every user-facing string the app writes itself lives here, and every one
 * of them is typed. The types are the voice's own registers:
 *
 *   vital     — a measurement's label
 *   direction — a word from the closed set
 *   option    — a session or alternative from the athlete's own plan
 *   pattern   — a fact about the past, always with its sample size
 *   status    — connection, sync, staleness and empty-state lines
 *
 * Since core-contract §8 the *domain* sentences are not here: the
 * direction word, the arithmetic, the travel, the Rest line and the
 * pattern sentences are written by `stride-core` and travel to the app
 * already rendered. What is left is the chrome around them — headings,
 * buttons, labels, empty states — and it is what this file holds.
 *
 * The lint (`npm run lint:voice`) reads this file *and* the core's export
 * (`crates/stride-core/strings.json`), and fails the build on a banned
 * word, on a `direction` string outside the closed set, on a `pattern`
 * string without a sample size, on an `option` string that mentions a
 * vital, and on a ban list that has drifted from the contract's.
 *
 * Nothing in the app may write a user-facing string inline. If a string is
 * not in this file, it does not exist yet.
 */

/** The five registers a string can belong to. */
export const STRING_TYPES = ['vital', 'direction', 'option', 'pattern', 'status'] as const
export type StringType = (typeof STRING_TYPES)[number]

/**
 * The closed direction set. Three words, no synonyms, no extensions.
 * See `docs/contract/voice.md`.
 *
 * The set is frozen by the contract and the core's export carries it too;
 * the lint checks that the two agree. The app knows the words because it
 * colours them — the meaning is always in the word itself, never in the
 * colour.
 */
export const DIRECTION_WORDS = ['better than usual', 'about usual', 'could be better'] as const
export type DirectionWord = (typeof DIRECTION_WORDS)[number]

interface Entry {
  readonly type: StringType
  readonly text: string
}

/**
 * Everything else. Placeholders are written `{name}` and filled in by
 * `t()`.
 */
export const catalogue = {
  // ------------------------------------------------------------- shell
  'nav.today': { type: 'status', text: 'Today' },
  'nav.runs': { type: 'status', text: 'Runs' },
  'nav.progress': { type: 'status', text: 'Progress' },
  'nav.body': { type: 'status', text: 'Body' },
  'nav.settings': { type: 'status', text: 'Settings' },
  'nav.label': { type: 'status', text: 'Destinations' },

  // ------------------------------------------------------------- today
  'today.heading': { type: 'status', text: 'Today' },
  'today.vitals.heading': { type: 'status', text: 'Vitals' },
  'today.session.source.imported': { type: 'status', text: 'from your imported plan' },
  'today.session.source.baseline': { type: 'status', text: 'from your inferred baseline' },
  'today.session.source.typed': { type: 'status', text: 'from your plan' },
  'today.options.heading': { type: 'status', text: 'Also in your plan' },
  'today.lastRun.label': { type: 'status', text: 'Last run' },
  'today.week.heading': { type: 'status', text: 'This week' },
  'today.state.noData': {
    type: 'status',
    text: "Connect your Garmin key to see today's numbers",
  },
  'today.state.noDataAction': { type: 'status', text: 'Connect Garmin' },
  'today.state.stale.days': { type: 'status', text: 'last synced {n} days ago' },
  'today.state.stale.hours': { type: 'status', text: 'last synced {n} hours ago' },
  'today.state.stale.minutes': { type: 'status', text: 'last synced {n} minutes ago' },
  'today.state.syncIncomplete': { type: 'status', text: 'The last sync did not complete' },
  'today.state.retry': { type: 'status', text: 'Try again' },
  'today.state.noPlan': { type: 'status', text: 'No plan is active for today' },

  // ------------------------------------------------------ the vitals
  'vital.distance': { type: 'vital', text: 'Distance' },
  'vital.time': { type: 'vital', text: 'Time' },
  'vital.pace': { type: 'vital', text: 'Pace' },
  'vital.heartRate': { type: 'vital', text: 'Heart rate' },

  // ------------------------------------------------------------- runs
  'runs.heading': { type: 'status', text: 'Runs' },
  'runs.group.thisWeek': { type: 'status', text: 'This week' },
  'runs.group.date': { type: 'status', text: '{date}' },
  'runs.group.total': { type: 'status', text: '{distance} · {time}' },
  'runs.empty.title': { type: 'status', text: 'No activities yet' },
  'runs.empty.body': { type: 'status', text: 'Activities appear here after the first sync' },
  'runs.import': { type: 'status', text: 'Import a FIT file' },
  'runs.importing': { type: 'status', text: 'Importing…' },
  'runs.duplicate': { type: 'status', text: 'Already have this run ({date}, {distance})' },
  'runs.compareWith': { type: 'status', text: 'Compare with…' },
  'runs.addNote': { type: 'status', text: 'Add a note' },
  'runs.loadMore': { type: 'status', text: 'Load more' },
  'runs.filter.all': { type: 'status', text: 'All' },
  'runs.filter.runs': { type: 'status', text: 'Runs' },
  'runs.filter.other': { type: 'status', text: 'Other' },

  // --------------------------------------------------------- progress
  'progress.heading': { type: 'status', text: 'Progress' },
  'progress.volume.heading': { type: 'status', text: 'Weekly volume' },
  'progress.paceAtHr.heading': { type: 'status', text: 'Pace at heart rate' },
  'progress.bests.heading': { type: 'status', text: 'Bests & milestones' },
  'progress.consistency.heading': { type: 'status', text: 'Consistency' },
  'progress.adherence.heading': { type: 'status', text: 'Adherence' },
  'progress.adherence.count': { type: 'status', text: '{done} of {planned} sessions run' },
  'progress.range.label': { type: 'status', text: 'Range' },
  'progress.range.weeks': { type: 'status', text: '{n} weeks' },
  'progress.typicalWeek': { type: 'status', text: 'your typical week' },
  'progress.sessionsPerWeek': { type: 'status', text: '{n} sessions a week' },
  'progress.best.5k': { type: 'status', text: 'Fastest 5 km' },
  'progress.best.10k': { type: 'status', text: 'Fastest 10 km' },
  'progress.best.longest': { type: 'status', text: 'Longest run' },

  // ------------------------------------------------------------- body
  'body.heading': { type: 'status', text: 'Body' },
  'body.howToRead.heading': { type: 'status', text: 'How to read this' },
  'body.howToRead.band': {
    type: 'status',
    text: 'The band is the middle of your last {n} nights',
  },
  'body.howToRead.inside': {
    type: 'status',
    text: 'Inside the band reads as about usual',
  },
  'body.patterns.heading': { type: 'status', text: 'Patterns' },
  'body.sleep.duration': { type: 'status', text: 'duration' },
  'body.sleep.stages': { type: 'status', text: 'stages' },
  'body.history.label': { type: 'status', text: 'Last {n} nights' },

  // ---------------------------------------------------------- compare
  'compare.heading': { type: 'status', text: 'Compare' },
  'compare.column.metric': { type: 'status', text: 'Metric' },
  'compare.column.a': { type: 'status', text: 'A' },
  'compare.column.b': { type: 'status', text: 'B' },
  'compare.column.delta': { type: 'status', text: 'Difference' },
  'compare.swap': { type: 'status', text: 'Swap A and B' },
  'compare.chooseOther': { type: 'status', text: 'Choose a different run' },
  'compare.share': { type: 'status', text: 'Share' },
  'compare.chart.toggle': { type: 'status', text: 'Show' },
  'compare.run.label': { type: 'status', text: 'Run' },

  // ------------------------------------------------------------- plan
  'plan.heading': { type: 'status', text: 'Plan' },
  'plan.source.baseline': { type: 'status', text: 'Baseline (inferred from your history)' },
  'plan.source.prescribed': { type: 'status', text: 'Prescribed (imported)' },
  'plan.source.typed': { type: 'status', text: 'Typed' },
  'plan.source.unclassified': { type: 'status', text: 'Unclassified' },
  'plan.week.heading': { type: 'status', text: 'This week' },
  'plan.day.today': { type: 'status', text: 'Today' },
  'plan.status.done': { type: 'status', text: 'done' },
  'plan.status.planned': { type: 'status', text: 'planned' },
  'plan.status.notDone': { type: 'status', text: 'not done' },
  'plan.adherence.heading': { type: 'status', text: 'Adherence' },
  'plan.adherence.count': { type: 'status', text: '{done} of {planned} sessions run' },
  'plan.sources.heading': { type: 'status', text: 'Sources' },
  'plan.import': { type: 'status', text: 'Import a plan' },
  'plan.days.target': { type: 'status', text: '{distance}' },
  'plan.days.actual': { type: 'status', text: '{distance} run' },

  // --------------------------------------------------------- activity
  'activity.heading': { type: 'status', text: 'Activity' },
  'activity.story.heading': { type: 'status', text: 'What it meant' },
  'activity.effort.heading': { type: 'status', text: 'Effort' },
  'activity.splits.heading': { type: 'status', text: 'Splits' },
  'activity.elevation.heading': { type: 'status', text: 'Elevation' },
  'activity.route.heading': { type: 'status', text: 'Route' },
  'activity.plan.heading': { type: 'status', text: 'Against the plan' },
  'activity.plan.line': { type: 'status', text: '{actual} run · {target} planned' },
  'activity.export': { type: 'status', text: 'Export FIT' },
  'activity.showSplits': { type: 'status', text: 'Show splits' },
  'activity.hideSplits': { type: 'status', text: 'Hide splits' },
  'activity.showElevation': { type: 'status', text: 'Show elevation' },

  // ----------------------------------------------------------- common
  'common.loading': { type: 'status', text: 'Loading…' },
  'common.error': { type: 'status', text: 'This did not load' },
  'common.close': { type: 'status', text: 'Close' },
} as const satisfies Record<string, Entry>

export type CatalogueEntry = (typeof catalogue)[keyof typeof catalogue]
export type CatalogueKey = keyof typeof catalogue

/** Every entry, in one map. */
export const allEntries = catalogue
export type AllKeys = keyof typeof allEntries
