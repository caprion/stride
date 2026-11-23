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
 */

import type { CatalogueKey } from '../i18n'

/**
 * The side of the split an area belongs to.
 *
 * **The work** is the recent training — weeks. **The athlete profile** is
 * the long pattern — months and seasons. It is a split of Progress and
 * not a split of the app: both sides are on Progress, and whether the
 * athlete profile later becomes a destination of its own is a separate
 * decision that this model does not foreclose.
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
  { id: 'profile', titleKey: 'progress.wall.profile.heading', to: '/progress/shape' },
  { id: 'facets', titleKey: 'progress.profile.heading', to: '/progress/facets' },
  { id: 'challenges', titleKey: 'challenge.heading', to: '/progress/challenges' },
  { id: 'adherence', titleKey: 'progress.adherence.heading', to: '/progress/adherence' },
  { id: 'weekday', titleKey: 'progress.wall.weekday.heading', to: '/progress/weekday' },
  { id: 'daytime', titleKey: 'progress.wall.daytime.heading', to: '/progress/daytime' },
  { id: 'distances', titleKey: 'progress.wall.distances.heading', to: '/progress/distances' },
  { id: 'year', titleKey: 'progress.wall.year.heading', to: '/year' },
  { id: 'heatmap', titleKey: 'progress.wall.heatmap.heading', to: '/progress/heatmap' },
  { id: 'months', titleKey: 'progress.wall.months.heading', to: '/progress/months' },
  { id: 'years', titleKey: 'progress.wall.years.heading', to: '/progress/year' },
  { id: 'streaks', titleKey: 'progress.wall.streaks.heading', to: '/progress/habit' },
  { id: 'eddington', titleKey: 'progress.eddington.heading', to: '/progress/eddington' },
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
    titleKey: 'progress.area.work',
    side: 'work',
    cards: ['load', 'volume', 'consistency'],
  },
  {
    id: 'time',
    titleKey: 'progress.area.time',
    side: 'work',
    cards: ['zones', 'split'],
  },
  {
    id: 'comparison',
    titleKey: 'progress.area.comparison',
    side: 'work',
    cards: ['week', 'window'],
  },
  {
    id: 'pace',
    titleKey: 'progress.area.pace',
    side: 'work',
    cards: ['pace', 'bests'],
  },
  {
    id: 'shape',
    titleKey: 'progress.area.shape',
    side: 'profile',
    cards: ['profile', 'facets'],
  },
  {
    id: 'goals',
    titleKey: 'progress.area.goals',
    side: 'profile',
    cards: ['challenges', 'adherence'],
  },
  {
    id: 'schedule',
    titleKey: 'progress.area.schedule',
    side: 'profile',
    cards: ['weekday', 'daytime', 'distances'],
  },
  {
    id: 'seasons',
    titleKey: 'progress.area.seasons',
    side: 'profile',
    cards: ['year', 'heatmap', 'months', 'years'],
  },
  {
    id: 'habit',
    titleKey: 'progress.area.habit',
    side: 'profile',
    cards: ['streaks', 'eddington'],
  },
  {
    id: 'elsewhere',
    titleKey: 'progress.area.elsewhere',
    side: 'elsewhere',
    cards: ['activities', 'body'],
  },
] as const

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
