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

/**
 * The five registers a string can belong to.
 *
 * `statement` is the sixth: a factual sentence about a comparison, or
 * about the app itself — "2.4 km more than the 10 km target", "Your data
 * stays on this phone". A statement is a whole sentence: **it never
 * carries a hole**, because the values it talks about are already in it.
 */
export const STRING_TYPES = [
  'vital',
  'direction',
  'option',
  'pattern',
  'statement',
  'status',
] as const
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
  'today.state.connectFailed': { type: 'status', text: "The setup screen did not open" },
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
  'plan.import.done': { type: 'status', text: 'The plan is imported' },
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
  'activity.export': { type: 'status', text: 'Export FIT' },
  'activity.showSplits': { type: 'status', text: 'Show splits' },
  'activity.hideSplits': { type: 'status', text: 'Hide splits' },
  'activity.showElevation': { type: 'status', text: 'Show elevation' },

  // ----------------------------------------------------------- common
  'common.loading': { type: 'status', text: 'Loading…' },
  'common.error': { type: 'status', text: 'This did not load' },
  'common.close': { type: 'status', text: 'Close' },
  /**
   * The promise, and the fact that the plan is paused.
   *
   * The promise is a `statement`: a whole sentence with no hole in it.
   */
  'common.promise': { type: 'statement', text: 'Your data stays on this phone.' },
  'common.paused.until': { type: 'status', text: 'paused until {date}' },
  'common.paused.untilReason': { type: 'status', text: 'paused until {date} · {reason}' },
  // --------------------------------------------------------- settings
  'settings.heading': { type: 'status', text: 'Settings' },

  // The weekly goal: one number, and a blank is no goal.
  'settings.goal.heading': { type: 'status', text: 'Weekly goal' },
  'settings.goal.unit': { type: 'status', text: 'km a week' },
  'settings.goal.none': { type: 'status', text: 'No goal' },
  'settings.goal.empty': { type: 'statement', text: 'A blank goal means no goal.' },
  'settings.goal.save': { type: 'status', text: 'Save the goal' },
  'settings.goal.saved': { type: 'status', text: 'The goal is saved' },
  'settings.goal.cleared': { type: 'status', text: 'The goal is cleared' },

  // The pause: a start, an end and a reason.
  'settings.pause.heading': { type: 'status', text: 'Pause' },
  'settings.pause.start': { type: 'status', text: 'From' },
  'settings.pause.end': { type: 'status', text: 'Until' },
  'settings.pause.reason': { type: 'status', text: 'Reason' },
  'settings.pause.reason.travel': { type: 'status', text: 'travel' },
  'settings.pause.reason.sick': { type: 'status', text: 'sick' },
  'settings.pause.reason.injured': { type: 'status', text: 'injured' },
  'settings.pause.reason.none': { type: 'status', text: 'no reason given' },
  'settings.pause.save': { type: 'status', text: 'Save the pause' },
  'settings.pause.saved': { type: 'status', text: 'The pause is saved' },
  'settings.pause.clear': { type: 'status', text: 'End the pause' },
  'settings.pause.cleared': { type: 'status', text: 'The pause is ended' },

  // The connection: Garmin's state, the key's identifier, the last sync.
  'settings.connection.heading': { type: 'status', text: 'Garmin' },
  'settings.connection.status': { type: 'status', text: 'Status' },
  'settings.connection.connected': { type: 'status', text: 'Connected' },
  'settings.connection.notConnected': { type: 'status', text: 'Not connected' },
  'settings.connection.key': { type: 'status', text: 'Key identifier' },
  'settings.connection.keyNote': {
    type: 'statement',
    text: 'The last four characters of the key, and nothing else.',
  },
  'settings.connection.lastSync': { type: 'status', text: 'Last synced' },
  'settings.connection.never': { type: 'status', text: 'Never synced' },
  'settings.connection.reconnect': { type: 'status', text: 'Re-connect Garmin' },
  'settings.connection.setup': { type: 'status', text: 'Set up Garmin' },

  // The import: a plan file, or a FIT file.
  'settings.import.heading': { type: 'status', text: 'Import' },
  'settings.import.plan': { type: 'status', text: 'Import a plan' },
  'settings.import.fit': { type: 'status', text: 'Import a FIT file' },

  // The storage: the size, a backup, and the one destructive action.
  'settings.storage.heading': { type: 'status', text: 'Storage' },
  'settings.storage.size': { type: 'status', text: 'Database size' },
  'settings.storage.export': { type: 'status', text: 'Export a backup' },
  'settings.storage.exported': { type: 'status', text: 'The backup is written' },
  'settings.storage.exportFailed': { type: 'status', text: 'The backup was not written' },
  'settings.storage.delete': { type: 'status', text: 'Delete all data' },
  'settings.storage.deleted': { type: 'status', text: 'Everything is deleted' },
  'settings.delete.what': {
    type: 'statement',
    text: 'This deletes every run, every wellness day, the plan and every note.',
  },
  'settings.delete.undo': { type: 'statement', text: 'It cannot be undone.' },
  'settings.delete.confirm': { type: 'status', text: 'Type DELETE to confirm' },
  'settings.delete.word': { type: 'status', text: 'DELETE' },
  'settings.delete.action': { type: 'status', text: 'Delete everything' },

  // About: the version, the licence, the promise and the source.
  'settings.about.heading': { type: 'status', text: 'About' },
  'settings.about.version': { type: 'status', text: 'Version' },
  'settings.about.licence': { type: 'status', text: 'Licence' },
  'settings.about.licenceValue': { type: 'status', text: 'AGPL-3.0' },
  'settings.about.source': { type: 'status', text: 'Source' },
  'settings.about.sourceValue': { type: 'status', text: 'github.com/caprion/stride' },
  'settings.about.sourceUrl': {
    type: 'status',
    text: 'https://github.com/caprion/stride',
  },

  // ---------------------------------------------------------- connect
  'connect.promise': { type: 'statement', text: 'Your data stays on this phone.' },
  'connect.key.label': { type: 'status', text: 'Garmin Personal API key' },
  'connect.key.note': {
    type: 'statement',
    text: 'A key is created on Garmin, and it is never written down anywhere else.',
  },
  'connect.key.link': { type: 'status', text: "Garmin's page for the key" },
  'connect.key.linkUrl': {
    type: 'status',
    text: 'https://connect.garmin.com/app/settings/personalApiToken',
  },
  'connect.action': { type: 'status', text: 'Connect' },
  'connect.checking': { type: 'status', text: 'Checking…' },
  'connect.syncing': { type: 'status', text: 'Syncing…' },
  'connect.connected': { type: 'status', text: 'Connected · {n} runs' },
  'connect.notNow': { type: 'status', text: 'Not now' },
  'connect.rejected': { type: 'status', text: 'That key was refused by Garmin' },
  'connect.unreachable': {
    type: 'status',
    text: 'Garmin could not be reached, so the key was not checked',
  },
  'connect.blank': { type: 'status', text: 'A key is needed to connect' },
  'connect.test': { type: 'status', text: 'Test the key' },
  'connect.testOk': { type: 'status', text: 'Garmin accepted the key' },
  'connect.testRefused': { type: 'status', text: 'Garmin refused the key' },

  // -------------------------------------------------- the empty states
  'today.state.noVitals': { type: 'status', text: 'No wellness data yet' },
  'today.state.noVitalsBody': {
    type: 'status',
    text: 'Sleep, HRV and resting heart rate appear after the first sync',
  },
  'today.state.noRuns': { type: 'status', text: 'No runs yet' },
  'today.state.from': { type: 'status', text: 'From {when}' },
  'runs.import.drop': { type: 'status', text: 'Drop a FIT file here' },
  'runs.import.hint': { type: 'status', text: 'A FIT file from any watch' },
  'runs.import.done': { type: 'status', text: 'The file is imported' },
  'runs.import.failed': { type: 'status', text: 'The file could not be read' },
  'activity.note.heading': { type: 'status', text: 'Note' },
  'activity.note.none': { type: 'status', text: 'No note on this run' },
  'activity.note.edit': { type: 'status', text: 'Edit the note' },
  'activity.note.save': { type: 'status', text: 'Save the note' },
  'activity.note.saved': { type: 'status', text: 'The note is saved' },
  'activity.note.cleared': { type: 'status', text: 'The note is cleared' },
  'activity.note.placeholder': { type: 'status', text: 'A note about this run' },
  'progress.needsMore': { type: 'status', text: 'Needs {n} more weeks' },
  'progress.empty.title': { type: 'status', text: 'Nothing to show yet' },
  'progress.empty.body': {
    type: 'status',
    text: 'The charts fill in as runs and wellness days arrive',
  },
  'body.state.empty': { type: 'status', text: 'No wellness data yet' },
  'body.state.emptyBody': {
    type: 'status',
    text: 'Sleep, HRV and resting heart rate appear after the first sync',
  },
  'plan.state.none': { type: 'status', text: 'No plan is active' },
  'plan.state.noneBody': {
    type: 'status',
    text: 'A plan is imported, or inferred from your own history',
  },
  'plan.counts.done': { type: 'status', text: '{n} done' },
  'plan.counts.planned': { type: 'status', text: '{n} planned' },
  'plan.counts.notDone': { type: 'status', text: '{n} not done' },
  'plan.source.why': { type: 'status', text: 'Why this plan' },
  'plan.source.why.baseline': {
    type: 'status',
    text: 'Inferred from the runs you usually do',
  },
  'plan.source.why.prescribed': { type: 'status', text: 'Imported from a plan file' },
  'plan.source.why.typed': { type: 'status', text: 'Written in by hand' },
  'compare.state.none': { type: 'status', text: 'A comparison needs two runs' },
  'compare.state.noneBody': {
    type: 'status',
    text: 'A run on the feed can be compared with another one',
  },
} as const satisfies Record<string, Entry>

export type CatalogueEntry = (typeof catalogue)[keyof typeof catalogue]
export type CatalogueKey = keyof typeof catalogue

/** Every entry, in one map. */
export const allEntries = catalogue
export type AllKeys = keyof typeof allEntries
