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
  'nav.foot.one': {
    type: 'statement',
    text: 'Real data, read from your own Garmin on this device.',
  },
  'nav.foot.two': { type: 'statement', text: 'Every number here is your own.' },
  'common.local': { type: 'status', text: 'Stored on this device' },
  'common.saved': { type: 'statement', text: 'Your history is stored locally on this device.' },
  'history.previousWeek': { type: 'status', text: 'A week earlier' },
  'history.nextWeek': { type: 'status', text: 'A week later' },
  'history.previousMonth': { type: 'status', text: 'A month earlier' },
  'history.nextMonth': { type: 'status', text: 'A month later' },
  'history.invalidRange': {
    type: 'status',
    text: 'The through date is before the from date.',
  },
  'today.body.open': { type: 'status', text: 'Explore Body' },
  'today.plan.open': { type: 'status', text: 'View plan' },
  'body.history.runs': { type: 'status', text: 'Runs on this date' },
  'runs.allHistory': { type: 'status', text: 'All saved history' },
  'runs.calendar': { type: 'status', text: 'Browse calendar' },
  'runs.calendar.hide': { type: 'status', text: 'Hide calendar' },
  'runs.calendar.note': {
    type: 'status',
    text: 'Dots mark saved runs, not complete source coverage.',
  },
  'progress.summary.sessions': { type: 'status', text: 'Recorded sessions' },
  'progress.summary.weeks': { type: 'status', text: 'Weeks with activity' },
  'progress.exploreWeek': { type: 'status', text: 'Explore a week' },
  'progress.viewRuns': { type: 'status', text: 'View runs this week' },
  'activity.zones.basis': {
    type: 'status',
    text: 'Bands from your own zones.',
  },
  'activity.zones.stopped': { type: 'status', text: 'Not moving' },
  'activity.zones.missing': { type: 'status', text: 'Time outside your zones' },
  'settings.preferences': { type: 'status', text: 'Training preferences' },
  'settings.sections': { type: 'status', text: 'Settings sections' },
  'settings.tab.data': { type: 'status', text: 'Data' },
  'common.saveFailed': {
    type: 'status',
    text: 'This was not saved. Your changes are still here.',
  },
  'common.invalidGoal': {
    type: 'status',
    text: 'Enter a positive distance, or leave the goal blank.',
  },

  // ------------------------------------------------------------- today
  'today.heading': { type: 'status', text: 'Today' },
  'today.session.source.imported': { type: 'status', text: 'from your imported plan' },
  'today.session.source.baseline': { type: 'status', text: 'from your usual week' },
  'today.session.source.typed': { type: 'status', text: 'from your typed plan' },
  'today.options.heading': { type: 'status', text: 'Also in your plan' },
  'today.lastRun.label': { type: 'status', text: 'Last run' },
  /**
   * The streak.
   *
   * A streak is a **count and a date**: how many days in a row, since
   * when, and how the longest one compares. There is no "you are
   * consistent" here — the number is the number, and the reader decides
   * what it means.
   */
  'today.streak.heading': { type: 'status', text: 'Streak' },
  'today.streak.days': { type: 'status', text: 'Days in a row' },
  'today.streak.daysLongest': { type: 'status', text: 'Longest days in a row' },
  'today.streak.weeks': { type: 'status', text: 'Weeks in a row' },
  'today.streak.weeksLongest': { type: 'status', text: 'Longest weeks in a row' },
  'today.streak.since': { type: 'status', text: 'since {date}' },
  'today.streak.range': { type: 'status', text: '{from} – {to}' },
  'today.streak.none': { type: 'status', text: 'no streak' },
  'today.week.heading': { type: 'status', text: 'This week' },
  'today.week.recordedDays': { type: 'status', text: 'Recorded sessions this week' },
  'today.week.sessions': { type: 'status', text: '{n} sessions' },
  /**
   * The cockpit.
   *
   * One hero, three rows, and what is on the table. The rows are
   * questions with the answer in them: this week's distance against the
   * same days last week, the form with its ratio and its band, and the
   * habit with the streak, the months and the Eddington number.
   *
   * Nothing here is a verdict, a score or a command: a difference is a
   * difference, a form is a form, and a streak is a count.
   */
  'today.cockpit.hero': { type: 'status', text: 'The latest run' },
  'today.cockpit.standing': { type: 'status', text: 'Where you stand' },
  'today.cockpit.table': { type: 'status', text: "What's on the table" },
  'today.hero.open': { type: 'status', text: 'Open the run' },
  'today.hero.zones': { type: 'status', text: 'Time in zones' },
  'today.week.against': { type: 'status', text: '{difference} against {from} – {to}' },
  'today.form.label': { type: 'status', text: 'The difference' },
  'today.form.ratio': { type: 'status', text: '{ratio} times the last 6 weeks' },
  'today.form.band': { type: 'status', text: 'the usual range is {band}' },
  'today.form.asOf': { type: 'status', text: 'as of {date}' },
  'today.habit.label': { type: 'status', text: 'The habit' },
  'today.habit.day': { type: 'status', text: '{n} day in a row' },
  'today.habit.days': { type: 'status', text: '{n} days in a row' },
  'today.habit.month': { type: 'status', text: '{n} month in a row' },
  'today.habit.months': { type: 'status', text: '{n} months in a row' },
  'today.habit.months.none': { type: 'status', text: 'no months in a row' },
  'today.habit.eddington': {
    type: 'status',
    text: '{number} km on at least {number} days, over {from} – {to}',
  },
  'today.habit.eddington.empty': {
    type: 'status',
    text: 'No recorded distance in this period, so there is no number.',
  },
  'today.state.noData': {
    type: 'status',
    text: "Connect your Garmin key to see today's numbers",
  },
  'today.state.noDataAction': { type: 'status', text: 'Connect Garmin' },
  'today.state.noRecords': {
    type: 'status',
    text: 'No saved observations yet. Refresh Garmin to check for recent records.',
  },
  'today.state.connectFailed': { type: 'status', text: 'The setup screen did not open' },
  'today.state.stale.days': { type: 'status', text: 'last synced {n} days ago' },
  'today.state.stale.hours': { type: 'status', text: 'last synced {n} hours ago' },
  'today.state.syncIncomplete': { type: 'status', text: 'The last sync did not complete' },
  'today.state.historyPending': {
    type: 'status',
    text: 'More history is available to check. Saved records remain available.',
  },
  'sync.refresh': { type: 'status', text: 'Refresh Garmin' },
  'sync.refreshing': { type: 'status', text: 'Refreshing...' },
  'sync.older': { type: 'status', text: 'Check older history' },
  'sync.coverage': { type: 'status', text: 'History checked' },
  'sync.coverage.range': { type: 'status', text: '{from} to {to}' },
  'sync.coverage.unknown': { type: 'status', text: 'Not checked yet' },
  'sync.activities': { type: 'status', text: 'Runs' },
  'sync.wellness': { type: 'status', text: 'Body' },
  'sync.failed': {
    type: 'status',
    text: 'The refresh did not finish. Saved records are unchanged.',
  },
  'sync.requested': {
    type: 'status',
    text: 'Refresh requested. The connection status updates when it finishes.',
  },
  'today.state.retry': { type: 'status', text: 'Try again' },
  'today.state.noPlan': { type: 'status', text: 'No plan is active for today' },

  // ------------------------------------------------------ the vitals
  /**
   * The labels of the readings.
   *
   * The first one is the heading of the card on Today: it names **when
   * the numbers are from** — the sleep of last night, the night's
   * heart-rate variability, the night's resting heart rate — and it
   * interprets nothing. A card headed with the single word "Rest" reads
   * as an instruction; this one does not. The core's own catalogue
   * carries the same words.
   */
  'vital.rest.heading': { type: 'vital', text: 'Last night' },
  'vital.distance': { type: 'vital', text: 'Distance' },
  'vital.time': { type: 'vital', text: 'Time you were running' },
  'vital.pace': { type: 'vital', text: 'Pace' },
  'vital.heartRate': { type: 'vital', text: 'Heart rate' },
  'vital.sleep': { type: 'vital', text: 'Sleep' },
  'vital.hrv': { type: 'vital', text: 'HRV' },
  'vital.restingHr': { type: 'vital', text: 'Resting HR' },

  /**
   * The version, as the app words it.
   *
   * The number is the api's — the binary's — and the app holds no version
   * of its own: there is one version string in the app, and it is the
   * one the api answered.
   */
  'status.version': { type: 'status', text: 'Version {version}' },

  // ------------------------------------------------------------- runs
  'runs.heading': { type: 'status', text: 'Runs' },
  'runs.group.thisWeek': { type: 'status', text: 'This week' },
  'runs.group.date': { type: 'status', text: '{date}' },
  'runs.group.total': { type: 'status', text: '{distance} · {time}' },
  'runs.empty.title': { type: 'status', text: 'No runs yet' },
  'runs.empty.body': { type: 'status', text: 'Runs appear here after the first sync' },
  'runs.import': { type: 'status', text: 'Import a FIT file' },
  'runs.importing': { type: 'status', text: 'Importing…' },
  'runs.duplicate': { type: 'status', text: 'Already have this run ({date}, {distance})' },
  'runs.compareWith': { type: 'status', text: 'Compare with…' },
  'runs.addNote': { type: 'status', text: 'Add a note' },
  'runs.loadMore': { type: 'status', text: 'Load more' },
  'runs.search': { type: 'status', text: 'Search saved runs' },
  'runs.from': { type: 'status', text: 'From date' },
  'runs.to': { type: 'status', text: 'Through date' },
  'runs.noMatches': { type: 'status', text: 'No saved runs match these filters.' },
  'runs.select': { type: 'status', text: 'Select {date} for comparison' },
  'runs.selected': { type: 'status', text: '{n} of 2 runs selected' },
  'runs.compareSelected': { type: 'status', text: 'Compare selected runs' },
  'runs.clearSelection': { type: 'status', text: 'Clear selection' },
  'runs.filterScope': {
    type: 'status',
    text: 'Dates search saved history. Text searches loaded results; load more to search further back.',
  },

  // --------------------------------------------------------- progress
  'progress.heading': { type: 'status', text: 'Progress' },
  /**
   * The wall: the desktop's Progress.
   *
   * The names of the panels are the mock's own — the year, the pace, the
   * training load — and the sentences under them are the app's: what the
   * panel is, and the window it covers. Nothing here is a number: every
   * number on the wall is the router's answer.
   */
  'progress.wall.open': { type: 'status', text: 'open →' },
  /*
    The ways into the runs behind a picture.

    A mark on a chart is a shape and a shape has no words in it, so the
    link that carries it says what it opens: the date, and the runs
    behind it. A reader who cannot see the mark — or who reads the
    screen aloud — still knows where the link goes.
  */
  'progress.wall.runs.day': { type: 'status', text: 'The runs of {date}' },
  'progress.wall.runs.week': { type: 'status', text: 'The runs of the week starting {date}' },
  'progress.wall.runs.month': { type: 'status', text: 'The runs of {month}' },
  'progress.wall.window.weeks': { type: 'status', text: 'The weeks of the period' },
  /*
    The eleven cards of the wall.

    **A card is a question.** The name is the question and the picture
    under it is the answer, so the names are short and they name the
    thing: this week, the load, the weeks, where the time went, the
    pace, the bests — and on the athlete profile, the shape, the months,
    the schedule, the habit and the goals.
  */
  'progress.wall.thisWeek.heading': { type: 'status', text: 'This week' },
  'progress.wall.load.heading': { type: 'status', text: 'Training load' },
  'progress.wall.weeks.heading': { type: 'status', text: 'The weeks' },
  'progress.wall.time.heading': { type: 'status', text: 'Where the time went' },
  'progress.wall.shape.heading': { type: 'status', text: 'Your shape' },
  'progress.wall.schedule.heading': { type: 'status', text: 'The schedule' },
  'progress.wall.habit.heading': { type: 'status', text: 'The habit' },
  'progress.wall.goals.heading': { type: 'status', text: 'The goals' },
  /*
    The pictures' own words.

    **A drawing is read by eye, and its labels are read aloud.** These
    are the names the pictures carry: the key of a strip, the note under
    a bar, the sentence that says a window holds nothing.
  */
  'progress.wall.week.picture': {
    type: 'status',
    text: 'This week, and the same days, a week earlier.',
  },
  'progress.wall.week.now': { type: 'status', text: 'this week' },
  'progress.wall.week.previous': { type: 'status', text: 'the same days, a week earlier' },
  'progress.wall.week.difference': {
    type: 'status',
    text: '{difference} against the same days, a week earlier',
  },
  'progress.wall.time.picture': { type: 'status', text: 'The time in each of the five zones.' },
  'progress.wall.bests.empty': { type: 'status', text: 'No runs recorded.' },
  'progress.wall.shape.empty': { type: 'status', text: 'No runs recorded.' },
  'progress.wall.schedule.timeOfDay': { type: 'status', text: 'the time of day' },
  'progress.wall.habit.picture': {
    type: 'status',
    text: 'A mark for every day of the period, filled where the day held a run.',
  },
  'progress.wall.habit.streak': { type: 'status', text: '{count} days in a row' },
  'progress.wall.habit.eddington': { type: 'status', text: '{n} km on at least {n} days' },
  'progress.wall.pace.heading': { type: 'status', text: 'The pace' },
  'progress.wall.pace.weeks': { type: 'status', text: '{n} weeks' },
  'progress.wall.pace.note': {
    type: 'status',
    text: 'The training week by week, and where the time went by pace.',
  },
  'progress.wall.week.distance': { type: 'status', text: 'Distance this week' },
  'progress.wall.week.sessions': { type: 'status', text: 'Recorded sessions' },
  'progress.wall.week.before': { type: 'status', text: 'A week earlier' },
  'progress.wall.week.window': {
    type: 'status',
    text: '{from} – {to}, against {beforeFrom} – {beforeTo}.',
  },
  /*
    The calendar of the year, coloured by the distance of the day.

    **One calendar, and it is the wall's own.** The other calendar — the
    one the year page draws — is a different thing, and the two are no
    longer called the same thing.
  */
  'progress.wall.calendar.legend': {
    type: 'status',
    text: 'The days of the year, by distance.',
  },
  'progress.wall.calendar.none': { type: 'status', text: 'no run' },
  'progress.wall.calendar.l1': { type: 'status', text: 'under 5 km' },
  'progress.wall.calendar.l2': { type: 'status', text: '5–10 km' },
  'progress.wall.calendar.l3': { type: 'status', text: '10–15 km' },
  'progress.wall.calendar.l4': { type: 'status', text: 'over 15 km' },
  'progress.wall.streaks.days': { type: 'status', text: 'Days' },
  'progress.wall.streaks.weeks': { type: 'status', text: 'Weeks' },
  'progress.wall.streaks.months': { type: 'status', text: 'Months' },
  'progress.wall.streaks.longest': {
    type: 'status',
    text: 'The longest: {days} days · {months} months.',
  },
  /*
    The task areas: the groups the wall's cards are drawn in, and the
    groups the phone's index carries. A heading and nothing else — no
    verdict, no summary, no number of its own.
  */
  /*
    The nav label of the athlete profile.

    **"My running" and not "Profile".** The tab bar holds Progress and
    the athlete profile side by side, and the two labels read alike: a
    runner scanning five tabs cannot tell "Progress" (the recent work)
    from "Profile" (the long pattern). "My running" names the athlete's
    own history and cannot be confused with either. It is one line
    here: "Profile" reads fine in the rail and the top bar.
  */
  'progress.profile.scope.30': { type: 'status', text: '30 days' },
  'progress.profile.scope.60': { type: 'status', text: '60 days' },
  'progress.profile.scope.90': { type: 'status', text: '90 days' },

  'nav.profile': { type: 'status', text: 'My running' },
  'progress.profile.page': { type: 'status', text: 'The athlete profile' },
  'progress.profile.lede': { type: 'status', text: 'What does my training look like?' },
  'progress.profile.back': { type: 'status', text: 'Back to the athlete profile' },

  /*
    The phone's athlete profile: the picture, the questions, and the
    areas' links. The questions are the profile's own — the goals, the
    schedule, the seasons and the number that counts the history — and
    every number in them is the core's own rendering.
  */
  'progress.profile.index.intro': {
    type: 'status',
    text: 'Four questions, and your answer to each.',
  },
  'progress.profile.index.goals.empty': { type: 'status', text: 'No challenges yet' },

  'progress.wall.day': { type: 'status', text: 'Day' },
  'progress.wall.timeOfDay': { type: 'status', text: 'Time of day' },
  'progress.wall.distances.sessions': { type: 'status', text: 'Sessions' },
  'progress.wall.distances.total': { type: 'status', text: 'Total' },
  'progress.wall.months.heading': { type: 'status', text: 'The months' },
  'progress.wall.bests.heading': { type: 'status', text: 'Bests & milestones' },
  'progress.volume.heading': { type: 'status', text: 'Weekly volume' },
  'progress.paceAtHr.heading': { type: 'status', text: 'Weekly median pace' },
  'progress.bests.heading': { type: 'status', text: 'Bests & milestones' },
  'progress.consistency.heading': { type: 'status', text: 'Consistency' },
  'progress.consistency.unit': { type: 'status', text: 'sessions a week' },
  'progress.adherence.heading': { type: 'status', text: 'Sessions run' },
  'progress.adherence.count': { type: 'status', text: '{done} of {planned} sessions run' },
  /**
   * The Eddington number: consistency over peaks.
   *
   * The number is a count, and the note says what it counts: the number
   * is how many days reached that many kilometres, and the days to the
   * next number are days at one kilometre more. The note carries no
   * number of its own, so it stays true when the window changes.
   */
  'progress.eddington.heading': { type: 'status', text: 'A distance, and the days you reach it' },
  'progress.eddington.note': {
    type: 'status',
    text: 'The number is a distance, and the days are the days you reached that distance or more. The days to the next one are days at one kilometre more.',
  },
  'progress.eddington.daysToNext': { type: 'status', text: 'Days to the next distance' },
  'progress.eddington.longestDay': { type: 'status', text: 'Longest distance in a day' },
  'progress.eddington.empty': {
    type: 'status',
    text: 'No recorded distance in this period, so there is no number.',
  },
  /**
   * The training load.
   *
   * The card is **values and a scenario**: the two weighted series, the
   * form, the ratio, the week's three numbers, and what a run of rest
   * days is expected to do with them. There is no score, no grade, no
   * verdict and no instruction, and the band of the ratio is named as a
   * range — "0.8 to 1.3" — which the reader may disagree with.
   *
   * The sentences themselves are the core's: they arrive written, and the
   * app shows them as they stand.
   */
  'progress.load.heading': { type: 'status', text: 'Training load' },
  'progress.load.asOf': { type: 'status', text: 'as of {date}' },
  'progress.load.acute': { type: 'status', text: 'The last 7 days' },
  'progress.load.chronic': { type: 'status', text: 'The last 6 weeks' },
  'progress.load.form': { type: 'status', text: 'The difference' },
  'progress.load.ratio': { type: 'status', text: 'Times the last 6 weeks' },
  'progress.load.band': { type: 'status', text: 'The usual range is {band}' },
  'progress.load.week.heading': { type: 'status', text: 'The last 7 days' },
  'progress.load.impulse': { type: 'status', text: 'The load of the week' },
  'progress.load.monotony': { type: 'status', text: 'How even the week was' },
  'progress.load.strain': { type: 'status', text: 'The week as one effort' },
  'progress.load.window': { type: 'status', text: 'The period: {from} – {to}' },
  /*
    The charts of the load: the series over the window, the load of each
    day, and the form the scenario projects. The charts draw what the
    answer already carried — the shape of the training was on the wire
    and not on the screen.
  */
  'progress.load.series.heading': { type: 'status', text: 'The load over the period' },
  'progress.load.series.note': {
    type: 'statement',
    text: 'The load of the last 7 days, the load of the last 6 weeks, and the difference. The difference is positive above the line.',
  },
  'progress.load.days.heading': { type: 'status', text: 'The load of each day' },
  'progress.load.days.note': {
    type: 'statement',
    text: 'A day with no training has a load of zero, and a day with nothing to weigh has no load.',
  },
  'progress.load.projection.heading': { type: 'status', text: 'The difference, projected' },
  'progress.load.projection.note': {
    type: 'status',
    text: 'The next {n} days, with no training.',
  },
  'progress.load.empty.no_zone_model': {
    type: 'status',
    text: "Your watch's zones have not come through, so there is no load to show.",
  },
  'progress.load.empty.no_max_heart_rate': {
    type: 'status',
    text: "Your watch's zones carry no maximum heart rate, so there is no load to show.",
  },
  'progress.load.empty.unusable_zone_model': {
    type: 'status',
    text: "Your watch's zones do not start in order, so there is no load to show.",
  },
  'progress.load.empty.no_history': {
    type: 'status',
    text: 'No recorded days in this period, so there is no load to show.',
  },
  'progress.load.empty.other': {
    type: 'status',
    text: 'There is no load to show.',
  },
  'progress.range.label': { type: 'status', text: 'Range' },
  'progress.range.weeks': { type: 'status', text: '{n} weeks' },
  'progress.typicalWeek': { type: 'status', text: 'your typical week' },
  'progress.sessionsPerWeek': { type: 'status', text: '{n} sessions a week' },
  'progress.best.5k': { type: 'status', text: 'Fastest 5 km' },
  'progress.best.10k': { type: 'status', text: 'Fastest 10 km' },
  'progress.best.longest': { type: 'status', text: 'Longest run' },
  'progress.question': { type: 'status', text: 'Am I training consistently?' },
  'progress.partial': { type: 'status', text: 'Striped bar: this week is still in progress.' },
  'progress.scope': {
    type: 'status',
    text: 'Saved running history; gaps are not confirmed rest weeks. History may still be arriving.',
  },
  'progress.paceNote': {
    type: 'status',
    text: 'Weekly pace and heart rate are separate medians, not a same-effort fitness comparison.',
  },
  'progress.paceHr': { type: 'status', text: '{pace} at {hr} (separate weekly medians)' },
  'progress.averageNote': {
    type: 'status',
    text: 'Average across weeks with recorded sessions, not all calendar weeks.',
  },
  /**
   * The rolling view.
   *
   * The card is a **comparison**: the window's numbers, the numbers of the
   * window before it, and the difference between them. There is no
   * verdict, no grade and nothing that says which way is the better way —
   * the reader decides.
   *
   * The window is named on the card, so a number is never read apart from
   * the period it describes.
   */
  'progress.rolling.heading': { type: 'status', text: 'Against a week earlier' },
  'progress.rolling.window': { type: 'status', text: 'The last {days} days' },
  'progress.rolling.range': { type: 'status', text: '{from} – {to}' },
  'progress.rolling.before': { type: 'status', text: 'The same {days} days, a week earlier' },
  'progress.rolling.difference': { type: 'status', text: 'Difference' },
  'progress.rolling.sessions': { type: 'status', text: 'Runs' },
  'progress.rolling.zones.heading': { type: 'status', text: 'Time in zones' },
  'progress.rolling.zones.distribution': { type: 'status', text: 'By zone' },
  'progress.rolling.zones.models': {
    type: 'status',
    text: 'The zones this period was measured against',
  },
  'progress.profile.heading': { type: 'status', text: 'Training profile' },
  'progress.profile.facets.note': { type: 'status', text: 'What the period holds.' },

  /**
   * The athlete's shape: the same six measures of the training, over three
   * periods, drawn on one chart.
   *
   * The chart is the athlete's own — every axis is scaled to the largest
   * of the three periods' values, so the three shapes are read against
   * each other and not against anybody's idea of a good week.
   */
  'progress.shape.heading': { type: 'status', text: 'Your shape' },
  'progress.shape.note': {
    type: 'status',
    text: 'Six measures of your training, over three periods, on one chart.',
  },
  'progress.shape.scale': {
    type: 'status',
    text: 'Each axis is scaled to the largest of the three periods.',
  },
  'progress.shape.rates': {
    type: 'status',
    text: 'The volume is a distance a week and the consistency the share of the days, so the periods can be read against each other.',
  },
  'progress.shape.alt': {
    type: 'status',
    text: 'Six axes, with the shape of each of the three periods drawn on them.',
  },
  'progress.shape.empty': {
    type: 'status',
    text: 'This period holds no runs, so it has no shape.',
  },
  'progress.shape.axis': { type: 'status', text: 'Measure' },
  'progress.shape.window': { type: 'status', text: 'The last {days} days' },
  'progress.shape.previous': { type: 'status', text: 'the same {days} days, a week earlier' },
  'progress.shape.legend': { type: 'status', text: 'last {days} days' },
  'progress.shape.movement': { type: 'status', text: 'Movement' },
  'progress.rolling.trend.up': { type: 'status', text: 'up on the same {days} days, a week earlier' },
  'progress.rolling.trend.down': {
    type: 'status',
    text: 'down on the same {days} days, a week earlier',
  },
  'progress.rolling.trend.steady': {
    type: 'status',
    text: 'the same as the {days} days, a week earlier',
  },
  'progress.rolling.trend.up.short': { type: 'status', text: 'up' },
  'progress.rolling.trend.down.short': { type: 'status', text: 'down' },
  'progress.rolling.trend.steady.short': { type: 'status', text: 'the same' },
  'progress.rolling.noRows': { type: 'status', text: 'Nothing recorded yet.' },

  /**
   * The narrow Progress: an index of the questions.
   *
   * Six rows, each one a question with the athlete's answer in it. The
   * wording is the app's own: a question, the answer in the core's own
   * rendering, and the window the answer covers.
   */
  'challenge.heading': { type: 'status', text: 'Your challenges' },
  'challenge.challenges': { type: 'status', text: 'The challenges' },
  'challenge.met': { type: 'status', text: '{n} of {of} weeks' },
  'challenge.inARow': { type: 'status', text: '{n} in a row' },
  'challenge.empty': {
    type: 'status',
    text: 'No challenges yet. Set a weekly goal, or a run to keep doing.',
  },
  'challenge.note': {
    type: 'status',
    text: 'A week is met or it is not. The pattern across the row is yours to read.',
  },
  'challenge.add': { type: 'status', text: 'Add a challenge' },
  'challenge.label': { type: 'status', text: 'Your words for it' },
  'challenge.kind.week': { type: 'status', text: 'Across the week' },
  'challenge.kind.run': { type: 'status', text: 'A single run' },
  'challenge.target': { type: 'status', text: 'Kilometres' },
  'challenge.remove': { type: 'status', text: 'Remove' },
  'progress.index.intro': { type: 'status', text: 'Six questions, and your answer to each.' },
  'progress.index.footnote': {
    type: 'status',
    text: 'Each question opens on its own. The wider screens show the panels side by side.',
  },
  'progress.index.run': { type: 'status', text: '{n} run' },
  'progress.index.runs': { type: 'status', text: '{n} runs' },
  'progress.index.day': { type: 'status', text: '{n} day' },
  'progress.index.days': { type: 'status', text: '{n} days' },
  'progress.index.week': { type: 'status', text: '{n} week' },
  'progress.index.weeks': { type: 'status', text: '{n} weeks' },
  'progress.index.eddington': { type: 'status', text: '{n} km on at least {n} days' },
  'progress.index.pace.week': { type: 'status', text: 'The week of {date}' },
  'progress.index.pace.weeks': { type: 'status', text: 'The last {n} weeks' },
  'progress.index.habit.window': {
    type: 'status',
    text: 'The distance and the days, over {from} – {to}',
  },
  'progress.back': { type: 'status', text: 'Back to Progress' },

  // ------------------------------------------------------------- pace
  //
  // The pace, as Progress draws it: the pace at a heart rate, and where
  // the time went by pace. The pace page is gone — these two panels are
  // on The pace card and on the panel behind it.
  //
  // The sentences about the data are the core's, and they arrive already
  // written; what is here is the chrome around them.
  'pace.paceAtHr.heading': { type: 'status', text: 'The pace at a heart rate' },
  'pace.paceAtHr.note': {
    type: 'status',
    text: "One point a week: the pace you held at that week's median heart rate.",
  },
  'pace.bands.heading': { type: 'status', text: 'Where the time went, by pace' },
  'pace.bands.note': {
    type: 'status',
    text: 'Every sample of every run, bucketed by the pace it was held at.',
  },
  'pace.bands.rest': {
    type: 'status',
    text: 'The smaller bands',
  },
  'pace.bands.total': { type: 'status', text: 'The time in the bands' },
  'pace.empty.title': { type: 'status', text: 'Nothing to show yet' },
  'pace.empty.body': {
    type: 'status',
    text: 'No run in this period. The pace is read from the runs that are saved.',
  },

  /*
   * The year, as a film.
   *
   * The chrome around the year's own numbers: the headings, the labels
   * and the empty states. The sentences of the diff are the core's, and
   * they arrive already written.
   */
  'year.heading': { type: 'status', text: 'The year' },
  'year.back': { type: 'status', text: 'Back to Progress' },
  'year.window': { type: 'status', text: 'The period: {from} – {to}' },
  'year.comparison': { type: 'status', text: 'against the same {days} days, a week earlier' },
  'year.months.heading': { type: 'status', text: 'The months' },
  'year.month.runs': { type: 'status', text: 'Runs' },
  'year.month.longest': { type: 'status', text: 'Longest run' },
  'year.month.form': { type: 'status', text: 'The difference at the end' },
  'year.month.weeks': { type: 'status', text: 'The weeks of the month' },
  'year.month.mix': { type: 'status', text: 'Where the time went' },
  'year.month.high': { type: 'status', text: '{share} in the high zones' },
  'year.week.title': { type: 'status', text: '{week} · {distance}' },
  'year.zone.title': { type: 'status', text: '{zone} · {share}' },
  'year.day.title': { type: 'status', text: '{date} · {distance} · {high} in the high zones' },
  'year.day.none': { type: 'status', text: 'no run' },
  'year.day.noMix': { type: 'status', text: 'no mix' },
  'year.calendar.heading': { type: 'status', text: 'The year as a calendar' },
  'year.calendar.note': {
    type: 'statement',
    text: 'Each day is coloured by how much of that day was in the high zones. The calendar says when and how hard, and never where.',
  },
  'year.calendar.alt': {
    type: 'status',
    text: 'The days of the year, coloured by the share of each day in the high zones',
  },
  'year.calendar.legend': { type: 'status', text: 'Share of the day in the high zones' },
  'year.calendar.legend.noMix': { type: 'status', text: 'no mix' },
  'year.calendar.legend.low': { type: 'status', text: 'under 15%' },
  'year.calendar.legend.mid': { type: 'status', text: '15% to 35%' },
  'year.calendar.legend.high': { type: 'status', text: '35% to 60%' },
  'year.calendar.legend.top': { type: 'status', text: '60% and over' },
  'year.empty': { type: 'status', text: 'Nothing recorded in this year yet.' },
  'year.changed.heading': { type: 'status', text: 'What changed' },
  'year.changed.note': {
    type: 'statement',
    text: 'A diff against your own past, in facts. The sentence says what the records say.',
  },

  // ------------------------------------------------------------- body
  'body.heading': { type: 'status', text: 'Body' },
  'body.question': { type: 'status', text: 'How was my sleep?' },
  'body.sleep.heading': { type: 'status', text: 'Sleep on {date}' },
  'body.sleep.deep': { type: 'vital', text: 'Deep' },
  'body.sleep.rem': { type: 'vital', text: 'REM' },
  'body.sleep.light': { type: 'vital', text: 'Light' },
  'body.sleep.awake': { type: 'vital', text: 'Awake' },
  'body.sleep.composition': { type: 'status', text: 'Recorded sleep-stage durations' },
  'body.sleep.note': {
    type: 'status',
    text: 'Wearable estimates, not a timeline or a sleep-quality score. Duration alone does not describe sleep quality.',
  },
  'body.sleep.noStages': {
    type: 'status',
    text: 'No sleep-stage breakdown saved for this date.',
  },
  'body.sleep.missing': { type: 'status', text: 'Not recorded' },
  'body.sleep.run': { type: 'status', text: 'Sleep before this run' },
  'body.history.pick': { type: 'status', text: 'Choose a date to inspect its readings' },
  'body.history.axis': {
    type: 'status',
    text: 'Recorded readings with gaps where data is missing',
  },
  'body.additional.heading': { type: 'status', text: 'More measurements' },
  'body.history.controls': { type: 'status', text: 'Explore recorded history' },
  'body.history.date': { type: 'status', text: 'Date' },
  'body.history.range': { type: 'status', text: 'History period' },
  'body.history.days': { type: 'status', text: '{n} days' },
  'body.history.latest': { type: 'status', text: 'Latest' },
  'body.additional.body': {
    type: 'status',
    text: 'Other readings appear after they arrive in a sync. A missing reading does not establish that the source cannot supply it.',
  },
  'body.howToRead.heading': { type: 'status', text: 'How to read this' },
  'body.howToRead.band': {
    type: 'status',
    text: 'The range is the recent usual one for the selected date, not a medical reference range',
  },
  'body.howToRead.inside': {
    type: 'status',
    text: 'Inside the range reads as about usual',
  },
  'body.patterns.heading': { type: 'status', text: 'Patterns' },
  'body.history.label': { type: 'status', text: 'Last {n} nights' },

  // ---------------------------------------------------------- compare
  'compare.heading': { type: 'status', text: 'Compare' },
  'compare.column.metric': { type: 'status', text: 'Metric' },
  'compare.column.a': { type: 'status', text: 'A' },
  'compare.column.b': { type: 'status', text: 'B' },
  'compare.column.delta': { type: 'status', text: 'Difference' },
  'compare.swap': { type: 'status', text: 'Swap A and B' },
  'compare.chooseOther': { type: 'status', text: 'Choose a different run' },
  /**
   * The share: the comparison as a file the athlete keeps.
   *
   * The file is the app's own — the core's words — and it goes to the
   * athlete's machine and nowhere else.
   */
  'compare.share': { type: 'status', text: 'Share the comparison' },
  'compare.share.file': { type: 'status', text: 'stride-comparison' },
  'compare.chart.signal': { type: 'status', text: 'Signal' },
  'compare.run.label': { type: 'status', text: 'Run' },

  // ------------------------------------------------------------- plan
  'plan.heading': { type: 'status', text: 'Plan' },
  'plan.source.baseline': { type: 'status', text: 'Your usual week (inferred from your history)' },
  'plan.source.prescribed': { type: 'status', text: 'Your imported plan' },
  'plan.source.typed': { type: 'status', text: 'Your typed plan' },
  'plan.day.today': { type: 'status', text: 'Today' },
  'plan.status.done': { type: 'status', text: 'done' },
  'plan.status.planned': { type: 'status', text: 'planned' },
  'plan.status.notDone': { type: 'status', text: 'not done' },
  'plan.adherence.heading': { type: 'status', text: 'Sessions run' },
  'plan.adherence.count': { type: 'status', text: '{done} of {planned} sessions run' },
  'plan.sources.heading': { type: 'status', text: 'Sources' },
  'plan.import': { type: 'status', text: 'Import a plan' },
  'plan.import.done': { type: 'status', text: 'The plan is imported' },
  'plan.days.actual': { type: 'status', text: '{distance} run' },

  // --------------------------------------------------------- activity
  'activity.heading': { type: 'status', text: 'Activity' },
  /** The action that opens the run's film. */
  'activity.film': { type: 'status', text: 'Play the run' },
  'activity.story.heading': { type: 'status', text: 'What it meant' },
  'activity.effort.heading': { type: 'status', text: 'Effort' },
  'activity.splits.heading': { type: 'status', text: 'Splits' },
  'activity.form.heading': { type: 'status', text: 'The difference' },
  'activity.elevation.heading': { type: 'status', text: 'Elevation' },
  'activity.route.heading': { type: 'status', text: 'Route' },
  /*
    The run's update: the model's own reading of the run.

    It is **not** the product's voice, and the app says so: the writer
    and the reviewer are named, the checks are listed under it, and the
    athlete can dismiss it. The app's own facts are above it, and they
    are the app's.
  */
  'activity.update.heading': { type: 'status', text: "The run's update" },
  'activity.update.attribution': {
    type: 'status',
    text: 'Written by {writer}, checked by {reviewer}',
  },
  'activity.update.attribution.unchanged': {
    type: 'status',
    text: 'Written by {writer}; the checks found nothing to change',
  },
  'activity.update.checks.heading': { type: 'status', text: 'The checks' },
  'activity.update.check.groundedness': {
    type: 'status',
    text: 'Every number is in the payload',
  },
  'activity.update.check.direction': {
    type: 'status',
    text: 'Every direction agrees with the payload',
  },
  'activity.update.check.shape': {
    type: 'status',
    text: 'It is four to six sentences, plain prose, and it addresses you',
  },
  'activity.update.check.register': {
    type: 'status',
    text: 'No word the app does not use',
  },
  'activity.update.dismiss': { type: 'status', text: 'Dismiss' },
  'activity.update.dismissed': {
    type: 'status',
    text: 'The update was dismissed. It is still stored.',
  },
  'activity.update.showAgain': { type: 'status', text: 'Show it again' },
  'activity.update.sent.heading': { type: 'status', text: 'What the model was given' },
  /**
   * The route, for a screen reader.
   *
   * The drawing is the route and the two dots are where it started and
   * where it ended; this says so in one line, because a picture of a
   * line with two dots says nothing to anyone who cannot see it.
   */
  'activity.route.alt': {
    type: 'status',
    text: 'The route of the run, from the start to the finish',
  },
  'activity.plan.heading': { type: 'status', text: 'Against the plan' },
  'activity.export': { type: 'status', text: 'Export FIT' },
  'activity.exported': { type: 'status', text: 'The file is downloaded' },
  'activity.exportFailed': { type: 'status', text: 'The file could not be fetched' },
  'activity.showSplits': { type: 'status', text: 'Show splits' },
  'activity.hideSplits': { type: 'status', text: 'Hide splits' },
  'activity.showElevation': { type: 'status', text: 'Show elevation' },
  'activity.hideElevation': { type: 'status', text: 'Hide elevation' },
  'activity.back': { type: 'status', text: 'Back to runs' },
  'activity.route.street': { type: 'status', text: 'Street map' },
  'activity.route.offline': { type: 'status', text: 'GPS outline' },
  'activity.route.disclosure': {
    type: 'status',
    text: 'Street tiles come from OpenStreetMap and reveal the area being viewed. GPS outline stays on this device.',
  },
  'activity.route.failed': {
    type: 'status',
    text: 'Street tiles did not load. The saved GPS outline is shown instead.',
  },
  'activity.route.startEnd': { type: 'status', text: 'Filled marker: start. Ring: finish.' },
  'activity.effort.cursor': {
    type: 'status',
    text: 'Pace and heart rate share the same cursor along the whole run, and the elevation has its own scale.',
  },

  // --------------------------------------------------------- the run film
  //
  // The film is one run, playable: the pace, the heart rate and the
  // ground on one axis, the athlete's own zones behind them, and the
  // route beside it with a camera that follows the runner.
  //
  // The film's **sentences** are the core's — the reasons a run has no
  // film, the name of the space outside the zones — and they arrive
  // already written. What is here is the chrome around them.
  'film.heading': { type: 'status', text: 'Run film' },
  'film.back': { type: 'status', text: 'Back to the run' },
  'film.play': { type: 'status', text: 'Play' },
  'film.pause': { type: 'status', text: 'Pause' },
  'film.scrub': { type: 'status', text: 'Move through the run' },
  'film.speed': { type: 'status', text: 'The run plays at sixty times its own time.' },
  'film.chart.alt': {
    type: 'status',
    text: "The run over time: the pace, the heart rate, the ground, and your own zones",
  },
  'film.elapsed': { type: 'status', text: 'The whole run' },
  'film.ground': { type: 'status', text: 'Ground' },
  'film.now': { type: 'status', text: 'Right now' },
  'film.axis.pace': { type: 'status', text: 'Pace — faster is higher' },
  'film.axis.heartRate': { type: 'status', text: 'Heart rate' },
  'film.zones.heading': { type: 'status', text: 'The zones you were in' },
  'film.route.heading': { type: 'status', text: 'The route' },
  'film.route.coloured': { type: 'status', text: 'The track is coloured by the pace' },
  'film.splits.heading': { type: 'status', text: 'Splits' },
  'film.splits.column': { type: 'status', text: 'Split' },
  'film.splits.jump': { type: 'status', text: 'Play from this split' },
  'film.empty.title': { type: 'status', text: 'There is no film for this run' },
  'film.empty.body': {
    type: 'status',
    text: "The film is read from the run's own file.",
  },

  // ----------------------------------------------------------- common
  'common.loading': { type: 'status', text: 'Loading…' },
  'common.error': { type: 'status', text: 'This did not load' },
  'common.close': { type: 'status', text: 'Close' },
  /**
   * The promise, and the fact that the plan is paused.
   *
   * The promise is a `statement`: a whole sentence with no hole in it.
   */
  'common.promise': {
    type: 'statement',
    text: 'Your history is stored locally on this device.',
  },
  'common.paused.until': { type: 'status', text: 'paused until {date}' },
  'common.paused.untilReason': { type: 'status', text: 'paused until {date} · {reason}' },

  // ---------------------------------------------------------- the heatmap
  //
  // The heatmap is a grid over the positions the stored runs carry, and it
  // is made on this device. The disclosure says so, in the app's own words:
  // the aggregation is local, and nothing is sent anywhere.
  'heatmap.heading': { type: 'status', text: 'Where you run' },
  'heatmap.action': { type: 'status', text: 'Where you run' },
  'heatmap.disclosure': {
    type: 'status',
    text: 'The map is made on this device from the stored runs. Nothing is sent anywhere.',
  },
  'heatmap.summary': { type: 'status', text: '{runs} runs in {cells} places' },
  'heatmap.outside': {
    type: 'status',
    text: '{n} runs are in other places, outside this picture.',
  },
  'heatmap.cellSize': { type: 'status', text: 'Each square is about {size} across' },
  'heatmap.legend.one': { type: 'status', text: '1 run' },
  'heatmap.legend.many': { type: 'status', text: '{n} runs' },
  'heatmap.empty.title': { type: 'status', text: 'No run has a saved route yet' },
  'heatmap.empty.body': {
    type: 'status',
    text: 'The map is made from the routes in the stored runs.',
  },
  'heatmap.moreRuns': {
    type: 'status',
    text: 'The map covers the most recent runs. Older runs are not on it.',
  },
  'heatmap.alt': {
    type: 'status',
    text: 'A grid of squares, shaded by how many runs went through each square',
  },
  // --------------------------------------------------------- settings
  'settings.heading': { type: 'status', text: 'Data & connection' },
  'settings.connection.unknown': {
    type: 'status',
    text: 'Connection status could not be checked. Saved history is still available.',
  },
  'settings.connection.manage': { type: 'status', text: 'Connection details' },
  'settings.connection.connect': { type: 'status', text: 'Connect to Garmin' },

  /*
    The run update's own settings: which model writes, which model
    reviews, and whether anything may leave the machine at all.

    The **key is not here**: the settings name the environment variable
    it is read from, and the app says what would be sent.
  */
  'settings.update.heading': { type: 'status', text: 'Run updates' },
  'settings.update.state': { type: 'status', text: 'The update' },
  'settings.update.on': { type: 'status', text: 'On' },
  'settings.update.off': { type: 'status', text: 'Off' },
  'settings.update.writer': { type: 'status', text: 'The writer' },
  'settings.update.reviewer': { type: 'status', text: 'The reviewer' },
  'settings.update.model': { type: 'status', text: 'Model' },
  'settings.update.family': { type: 'status', text: 'Family' },
  'settings.update.baseUrl': { type: 'status', text: 'Base URL' },
  'settings.update.allowRemote': {
    type: 'status',
    text: 'Allow anything to leave this machine',
  },
  'settings.update.keyEnv': { type: 'status', text: 'The key is read from' },
  'settings.update.sent': { type: 'status', text: 'What would be sent' },
  'settings.update.sent.body': {
    type: 'statement',
    text: "The run's own facts: the distance, the duration, the pace, the heart rate, the splits, the time in your own zones and the run's own mechanics. Never your positions, never your note, never your name.",
  },
  'settings.update.save': { type: 'status', text: 'Save the settings' },
  'settings.update.saved': { type: 'status', text: 'The settings are saved' },
  'settings.update.sameFamily': {
    type: 'status',
    text: "The reviewer's family is the writer's family.",
  },

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
  'settings.storage.yourData': { type: 'status', text: 'Backup, restore and export' },
  'settings.storage.size': { type: 'status', text: 'Database size' },
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
  'settings.about.licence': { type: 'status', text: 'Licence' },
  'settings.about.licenceValue': { type: 'status', text: 'AGPL-3.0' },
  'settings.about.source': { type: 'status', text: 'Source' },
  'settings.about.sourceValue': { type: 'status', text: 'github.com/caprion/stride' },
  'settings.about.sourceUrl': {
    type: 'status',
    text: 'https://github.com/caprion/stride',
  },

  // ---------------------------------------------------------- connect
  'connect.promise': {
    type: 'statement',
    text: 'Your history is stored locally on this device.',
  },
  'connect.key.label': { type: 'status', text: 'Garmin Personal API key' },
  'connect.key.note': {
    type: 'statement',
    text: 'The Garmin connection is kept on this device and is not included in history backups.',
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
  'connect.test': { type: 'status', text: 'Test the key' },
  'connect.testOk': { type: 'status', text: 'Garmin accepted the key' },
  'connect.testRefused': { type: 'status', text: 'Garmin refused the key' },

  // -------------------------------------------------- the empty states
  'today.state.noVitals': { type: 'status', text: 'No wellness data yet' },
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

  /* ------------------------------------------- the ladder, the pace card */
  /*
    The ladder's own chrome: the two axes, the legend that says whose
    number each one is, and the details affordance that opens a rule.

    **The chart's own words are the app's chrome** — the axes, the
    legend and the affordances. The numbers, the rules and the reasons
    are the core's, and they arrive already rendered.
  */
  'common.window.weeks': { type: 'status', text: 'the last {n} weeks' },
  'pace.card.hr': { type: 'status', text: 'your heart rate' },
  'pace.card.pace': { type: 'status', text: 'your pace' },
  'pace.card.rule': { type: 'status', text: 'the rule' },
  'pace.card.sources': { type: 'status', text: 'the sources' },
  'pace.card.anchors': { type: 'status', text: 'the race paces' },
  'pace.card.empty': { type: 'status', text: 'No race time entered' },
  'pace.legend.method': { type: 'status', text: "the method's rule" },
  'pace.legend.athlete': { type: 'status', text: 'your own race' },
  'pace.card.note': {
    type: 'status',
    text: 'The heart-rate band and the pace are two different numbers, and they are not converted into one another.',
  },

  /* --------------------------------------------------- the effort curve */
  'progress.wall.curve.heading': { type: 'status', text: 'The effort curve' },
  'curve.card.races': { type: 'status', text: 'your races' },
  'curve.card.sessions': { type: 'status', text: "the method's sessions" },
  'curve.card.fieldTest': { type: 'status', text: 'the time trial' },
  'curve.card.equivalents': { type: 'status', text: 'the race paces' },
  'curve.card.curve': { type: 'status', text: 'the curve' },
  'curve.card.percent': { type: 'status', text: 'the share of your maximum' },
  'curve.card.minutes': { type: 'status', text: 'minutes' },
  'curve.card.note': {
    type: 'status',
    text: "The method's heart-rate bands and this curve's axis are two different numbers, and they are not converted into one another.",
  },

  /* -------------------------------------------------------- the ledger */
  'ledger.block.app': { type: 'status', text: 'your zones' },
  'ledger.block.app.note': { type: 'status', text: 'the time you were running' },
  'ledger.block.source': { type: 'status', text: "the watch's own zones" },
  'ledger.block.source.note': { type: 'status', text: 'the whole run' },
  'ledger.overlap': { type: 'status', text: 'where both bases exist' },
  'ledger.overlap.gap': { type: 'status', text: 'the distance between the two' },
  'ledger.coverage': { type: 'status', text: 'the runs the period holds' },
  'ledger.longest': { type: 'status', text: 'the longest run' },
  'ledger.repeated': { type: 'status', text: 'the repeated work' },
  'ledger.more': { type: 'status', text: 'The store holds more runs than this answer read' },

  /* ------------------------------------------------------ the threshold */
  'settings.threshold.heading': { type: 'status', text: 'Your threshold' },
  'threshold.card.candidates': { type: 'status', text: 'the four candidates' },
  'threshold.card.choose': { type: 'status', text: 'use this one' },
  'threshold.card.inUse': { type: 'status', text: 'in use' },
  'threshold.card.history': { type: 'status', text: 'the choices you have made' },
  'threshold.card.typed': { type: 'status', text: 'a number you brought' },
  'threshold.card.typedField': { type: 'status', text: 'your number, in bpm' },
  'threshold.card.typedSource': { type: 'status', text: 'where it came from' },
  'threshold.card.base': { type: 'status', text: 'the base the choice is made for' },
  'threshold.card.saved': { type: 'status', text: 'Your threshold is saved' },
  'threshold.card.refused': { type: 'status', text: 'That choice could not be saved' },

  /* ----------------------------------------------------- the heart rates */
  'settings.heartRates.heading': { type: 'status', text: 'Your heart rates' },
  'heartRates.card.bands': { type: 'status', text: 'the bands' },
  'heartRates.card.band': { type: 'status', text: 'band' },
  'heartRates.card.declared': { type: 'status', text: 'yours' },
  'heartRates.card.watch': { type: 'status', text: "your watch's" },
  'heartRates.card.difference': { type: 'status', text: 'difference' },
  'heartRates.card.source': { type: 'status', text: 'where it came from' },
  'heartRates.card.measured': { type: 'status', text: 'when it was measured' },
  'heartRates.card.save': { type: 'status', text: 'Save your heart rates' },
  'heartRates.card.saved': { type: 'status', text: 'Your heart rates are saved' },
  'heartRates.card.refused': { type: 'status', text: 'Those heart rates could not be saved' },
  'heartRates.card.history': { type: 'status', text: 'the heart rates you have entered' },
  'heartRates.card.inUse': { type: 'status', text: 'in use' },
  'heartRates.card.none': { type: 'status', text: 'No heart rates entered' },

  /* ------------------------------------------------------- the plan's bases */
  'plan.bases.heading': { type: 'status', text: 'Adopting a programme' },
  'plan.bases.pick': { type: 'status', text: 'adopt this one' },
  'plan.bases.weeks': { type: 'status', text: '{n} weeks' },
  'plan.bases.start': { type: 'status', text: 'the plan starts on' },
  'plan.bases.race': { type: 'status', text: 'the race is on' },
  'plan.bases.mileage': { type: 'status', text: 'your own mileage' },
  'plan.bases.entry': { type: 'status', text: 'the level asks for' },
  'plan.bases.adopted': { type: 'status', text: 'The plan is adopted' },
  'plan.bases.refused': { type: 'status', text: 'That programme could not be adopted' },

  /* --------------------------------------------- the week, and the day */
  'plan.week.paces': { type: 'status', text: 'the paces of the week' },
  'plan.week.pacedWith': { type: 'status', text: 'the plan was paced with' },
  'plan.week.entry': { type: 'status', text: 'the level asks for' },
  'today.options.pacing': { type: 'status', text: "the plan's pacing" },
} as const satisfies Record<string, Entry>

export type CatalogueEntry = (typeof catalogue)[keyof typeof catalogue]
export type CatalogueKey = keyof typeof catalogue

/** Every entry, in one map. */
export const allEntries = catalogue
export type AllKeys = keyof typeof allEntries
