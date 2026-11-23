/**
 * The task areas: **how Progress is split**.
 *
 * The wall is twenty-four cards, and every card is the same size and the
 * same weight. The reader has to work out for themselves which cards
 * belong together, and the wall's order is the order the panels happened
 * to be written — an order a reader cannot use.
 *
 * So the cards are grouped into **task areas**, and the areas are the
 * order a runner asks in: what is happening now, then how it compares,
 * then the shape of the last few months, then the habits and the
 * seasons.
 *
 * This module is the **one place** the split is written down. The wall
 * draws the areas from here, the phone's index reads them from here, and
 * the tests read them from here too — so the two screens cannot drift
 * apart without a test saying so.
 *
 * The rules:
 *
 *   - **Every card is in exactly one area.** All twenty-four are here,
 *     and a test says that none is missing and none is twice.
 *   - **An area is a heading and nothing more.** It does not judge, does
 *     not summarise, and adds no number of its own.
 *   - **A card opens the panel that answers it**, and the route is here
 *     with the card, so the two screens cannot disagree about where a
 *     card leads.
 *
 * **And the athlete profile is a destination of its own.** The areas are
 * split between two pages: the work on Progress, the long pattern on the
 * athlete profile. The split is written down here — the area's
 * `destination`, and the panels each destination owns — so that the two
 * walls, the two indexes and the router cannot drift apart.
 */

import type { CatalogueKey } from '../i18n'

/**
 * The side of the split an area belongs to.
 *
 * **The work** is the recent training — weeks. **The athlete profile** is
 * the long pattern — months and seasons. The side says which of the two
 * questions an area answers; the `destination` says which page draws it,
 * and the athlete profile is a destination of its own.
 */
export type AreaSide = 'work' | 'profile' | 'elsewhere'

/** A card of the wall. */
export interface WallCard {
  /** The card's own id — the wall's test id without its prefix. */
  id: string
  /** The catalogue key that names the card. */
  titleKey: CatalogueKey
  /** Where the card's "open →" goes. */
  to: string
}

/**
 * The twenty-four cards of the wall.
 *
 * Each one is a panel the router already answers, and each one is on the
 * wall with the same test id as before: the split moves the cards
 * around, and it does not change what a card is or where it leads.
 */
export const WALL_CARDS: readonly WallCard[] = [
  { id: 'load', titleKey: 'progress.load.heading', to: '/progress/load' },
  { id: 'volume', titleKey: 'progress.volume.heading', to: '/progress/volume' },
  { id: 'consistency', titleKey: 'progress.consistency.heading', to: '/progress/consistency' },
  { id: 'zones', titleKey: 'progress.wall.zones.heading', to: '/progress/zone-time' },
  { id: 'split', titleKey: 'progress.wall.split.heading', to: '/progress/zones' },
  { id: 'week', titleKey: 'progress.wall.week.heading', to: '/progress/week' },
  { id: 'window', titleKey: 'progress.rolling.heading', to: '/progress/window' },
  { id: 'pace', titleKey: 'progress.wall.pace.heading', to: '/pace' },
  { id: 'bests', titleKey: 'progress.wall.bests.heading', to: '/progress/bests' },
  { id: 'profile', titleKey: 'progress.wall.profile.heading', to: '/profile/shape' },
  { id: 'facets', titleKey: 'progress.profile.heading', to: '/profile/facets' },
  { id: 'challenges', titleKey: 'challenge.heading', to: '/profile/challenges' },
  { id: 'adherence', titleKey: 'progress.adherence.heading', to: '/profile/adherence' },
  { id: 'weekday', titleKey: 'progress.wall.weekday.heading', to: '/profile/weekday' },
  { id: 'daytime', titleKey: 'progress.wall.daytime.heading', to: '/profile/daytime' },
  { id: 'distances', titleKey: 'progress.wall.distances.heading', to: '/profile/distances' },
  { id: 'year', titleKey: 'progress.wall.year.heading', to: '/year' },
  { id: 'heatmap', titleKey: 'progress.wall.heatmap.heading', to: '/profile/heatmap' },
  { id: 'months', titleKey: 'progress.wall.months.heading', to: '/profile/months' },
  { id: 'years', titleKey: 'progress.wall.years.heading', to: '/profile/year' },
  { id: 'streaks', titleKey: 'progress.wall.streaks.heading', to: '/profile/habit' },
  { id: 'eddington', titleKey: 'progress.eddington.heading', to: '/profile/eddington' },
  { id: 'activities', titleKey: 'progress.wall.activities.heading', to: '/runs' },
  { id: 'body', titleKey: 'progress.wall.body.heading', to: '/body' },
] as const

/** The id of a card of the wall. */
export type WallCardId = (typeof WALL_CARDS)[number]['id']

/** A group of cards: one task, and the cards that answer it. */
export interface TaskArea {
  /** The area's own id, for the test and the key. */
  id: string
  /** The catalogue key that names the area. */
  titleKey: CatalogueKey
  /** Which side of the split the area is on. */
  side: AreaSide
  /**
   * Which destination the area is drawn on.
   *
   * The athlete profile is a destination of its own, so the areas are
   * split between two pages: the work on Progress, the long pattern on
   * the athlete profile.
   */
  destination: 'progress' | 'profile'
  /** The cards of the area, in the order they are read. */
  cards: readonly WallCardId[]
}

/**
 * The areas, in the order the wall draws them.
 *
 * The order is the order a runner asks in: what is happening now, then
 * how it compares, then the shape of the last few months, then the
 * habits and the seasons.
 */
export const TASK_AREAS: readonly TaskArea[] = [
  {
    id: 'work',
    destination: 'progress',
    titleKey: 'progress.area.work',
    side: 'work',
    cards: ['load', 'volume', 'consistency'],
  },
  {
    id: 'time',
    destination: 'progress',
    titleKey: 'progress.area.time',
    side: 'work',
    cards: ['zones', 'split'],
  },
  {
    id: 'comparison',
    destination: 'progress',
    titleKey: 'progress.area.comparison',
    side: 'work',
    cards: ['week', 'window'],
  },
  {
    id: 'pace',
    destination: 'progress',
    titleKey: 'progress.area.pace',
    side: 'work',
    cards: ['pace', 'bests'],
  },
  {
    id: 'shape',
    destination: 'profile',
    titleKey: 'progress.area.shape',
    side: 'profile',
    cards: ['profile', 'facets'],
  },
  {
    id: 'goals',
    destination: 'profile',
    titleKey: 'progress.area.goals',
    side: 'profile',
    cards: ['challenges', 'adherence'],
  },
  {
    id: 'schedule',
    destination: 'profile',
    titleKey: 'progress.area.schedule',
    side: 'profile',
    cards: ['weekday', 'daytime', 'distances'],
  },
  {
    id: 'seasons',
    destination: 'profile',
    titleKey: 'progress.area.seasons',
    side: 'profile',
    cards: ['year', 'heatmap', 'months', 'years'],
  },
  {
    id: 'habit',
    destination: 'profile',
    titleKey: 'progress.area.habit',
    side: 'profile',
    cards: ['streaks', 'eddington'],
  },
  {
    id: 'elsewhere',
    destination: 'progress',
    titleKey: 'progress.area.elsewhere',
    side: 'elsewhere',
    cards: ['activities', 'body'],
  },
] as const

/** The areas of one destination, in the model's order. */
export function areasOf(destination: 'progress' | 'profile'): TaskArea[] {
  return TASK_AREAS.filter((area) => area.destination === destination)
}

/* ------------------------------------------------------------- the panels */

/**
 * The panels of the wall.
 *
 * A panel is the answer behind a card: the card's "open →" is the door
 * to it, and the panel is drawn on its own page. The list is here so
 * that the panels of the two destinations are written down in one place,
 * next to the areas that own them.
 */
export const WALL_PANELS = [
  // the work
  'window',
  'load',
  'pace',
  'zones',
  'zone-time',
  'volume',
  'week',
  'consistency',
  'bests',
  // the athlete profile
  'shape',
  'profile',
  'facets',
  'challenges',
  'adherence',
  'weekday',
  'daytime',
  'distances',
  'year',
  'heatmap',
  'months',
  'habit',
  'eddington',
  // somewhere else
  'activities',
] as const

/** The name of a panel of the wall. */
export type WallPanel = (typeof WALL_PANELS)[number]

/**
 * The panels of each destination.
 *
 * **Most of a panel's owner is the card's own area.** A card that opens
 * a panel drags that panel to its own side of the split, and a test says
 * the two never disagree. Two panels have no card of their own — the
 * radar, which is the athlete profile's shape, and the most recent
 * activities, which are the work's door out — and they are written down
 * here.
 */
export const PANEL_DESTINATIONS: Record<WallPanel, 'progress' | 'profile'> = {
  window: 'progress',
  load: 'progress',
  pace: 'progress',
  zones: 'progress',
  'zone-time': 'progress',
  volume: 'progress',
  week: 'progress',
  consistency: 'progress',
  bests: 'progress',
  activities: 'progress',
  shape: 'profile',
  profile: 'profile',
  facets: 'profile',
  challenges: 'profile',
  adherence: 'profile',
  weekday: 'profile',
  daytime: 'profile',
  distances: 'profile',
  year: 'profile',
  heatmap: 'profile',
  months: 'profile',
  habit: 'profile',
  eddington: 'profile',
}

/** The destination a panel opens under. */
export function panelDestination(panel: string): 'progress' | 'profile' {
  return PANEL_DESTINATIONS[panel as WallPanel] ?? 'progress'
}

/**
 * The route a panel opens on.
 *
 * **The panel opens under its own destination**, and the route is the
 * card's own route: the card that answers a panel opens the panel, and
 * the two cannot disagree about where the panel is.
 */
export function panelRoute(panel: WallPanel): string {
  return `/${panelDestination(panel)}/${panel}`
}

/**
 * Where a panel opens, and where the reader came from.
 *
 * **A panel of the athlete profile opened from Progress opens under the
 * profile**, and the place it was opened from travels with it, so that
 * the panel's back link can say where the reader came from.
 */
export function panelOpening(
  panel: WallPanel,
  origin: '/progress' | '/profile',
): { to: string; state?: { from: string } } {
  const destination = panelDestination(panel)
  const to = panelRoute(panel)
  return destination === origin.slice(1) ? { to } : { to, state: { from: origin } }
}

/**
 * The panel a card opens, if it opens one.
 *
 * A card's route is either a panel of the wall (`/profile/weekday`) or
 * a page of its own (`/pace`, `/year`, `/runs`, `/body`).
 */
export function panelOfCard(id: WallCardId): WallPanel | null {
  const card = WALL_CARDS.find((entry) => entry.id === id)
  const match = card ? /^\/(?:progress|profile)\/([a-z-]+)$/.exec(card.to) : null
  const panel = match?.[1] as WallPanel | undefined
  return panel && panel in PANEL_DESTINATIONS ? panel : null
}

/** The route a card opens. */
export function cardRoute(id: WallCardId): string {
  const card = WALL_CARDS.find((entry) => entry.id === id)
  // Every id in the areas is in the cards: a test says so, and a card
  // that is not there has nowhere to open.
  return card?.to ?? '/progress'
}

/** The area a card belongs to. */
export function areaOfCard(id: WallCardId): TaskArea | undefined {
  return TASK_AREAS.find((area) => area.cards.includes(id))
}
