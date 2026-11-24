/**
 * The wall's cards: **eleven of them**.
 *
 * The wall used to be twenty-four cards, and every card was the same
 * size and the same weight. The reader had to work out for themselves
 * which cards belonged together, and the wall's order was the order the
 * panels happened to be written — an order a reader cannot use.
 *
 * So the cards are cut. The wall is now **eleven cards**, and each one
 * is a question with one answer:
 *
 *   - **six on Progress** — the work: this week, the load, the weeks,
 *     where the time went, the pace, and the bests;
 *   - **five on the athlete profile** — the long pattern: the shape, the
 *     months, the schedule, the habit and the goals.
 *
 * **This module is the one place the list is written down.** The wall
 * draws the cards from here, the phone's index reads them from here, and
 * the tests read them from here too — so the two screens cannot drift
 * apart without a test saying so.
 *
 * The rules:
 *
 *   - **Every card is on one wall.** Six on Progress, five on the
 *     athlete profile, and a test says that none is on both and none is
 *     on neither.
 *   - **A card is a question, and the panel behind it answers it.** The
 *     card's "open →" is the doorway into that panel, and the route is
 *     written here with the card, so the two screens cannot disagree
 *     about where a card leads.
 *   - **The order is the order a runner asks in.** What is happening
 *     now, then what it cost, then what it became.
 *
 * **And the athlete profile is a destination of its own.** The cards are
 * split between two pages: the work on Progress, the long pattern on the
 * athlete profile. The split is written down here — the card's
 * `destination`, and the panels each destination owns — so that the two
 * walls, the two indexes and the router cannot drift apart.
 */

import type { CatalogueKey } from '../i18n'

/**
 * Which destination a card or a panel belongs to.
 *
 * **The work** is the recent training — weeks. **The athlete profile** is
 * the long pattern — months and seasons. The destination says which page
 * draws it, and the athlete profile is a destination of its own.
 */
export type WallDestination = 'progress' | 'profile'

/** A card of the wall. */
export interface WallCard {
  /** The card's own id — the wall's test id without its prefix. */
  id: string
  /** The catalogue key that names the card. */
  titleKey: CatalogueKey
  /** Where the card's "open →" goes. */
  to: string
  /** Which wall the card is drawn on. */
  destination: WallDestination
}

/**
 * The eleven cards of the wall.
 *
 * Each one is a panel the router already answers, and each one is on
 * the wall with the same test id as before: the cut moves the cards
 * around, and it does not change what a card is or where it leads.
 */
export const WALL_CARDS: readonly WallCard[] = [
  /* ------------------------------------------- the work, on Progress */
  { id: 'week', titleKey: 'progress.wall.thisWeek.heading', to: '/progress/week', destination: 'progress' },
  { id: 'load', titleKey: 'progress.wall.load.heading', to: '/progress/load', destination: 'progress' },
  { id: 'weeks', titleKey: 'progress.wall.weeks.heading', to: '/progress/volume', destination: 'progress' },
  { id: 'time', titleKey: 'progress.wall.time.heading', to: '/progress/zone-time', destination: 'progress' },
  { id: 'pace', titleKey: 'progress.wall.pace.heading', to: '/progress/pace', destination: 'progress' },
  { id: 'bests', titleKey: 'progress.wall.bests.heading', to: '/progress/bests', destination: 'progress' },
  /* --------------------------------- the long pattern, on the profile */
  { id: 'shape', titleKey: 'progress.wall.shape.heading', to: '/profile/shape', destination: 'profile' },
  { id: 'months', titleKey: 'progress.wall.months.heading', to: '/profile/months', destination: 'profile' },
  { id: 'schedule', titleKey: 'progress.wall.schedule.heading', to: '/profile/weekday', destination: 'profile' },
  { id: 'habit', titleKey: 'progress.wall.habit.heading', to: '/profile/habit', destination: 'profile' },
  { id: 'goals', titleKey: 'progress.wall.goals.heading', to: '/profile/challenges', destination: 'profile' },
] as const

/** The id of a card of the wall. */
export type WallCardId = (typeof WALL_CARDS)[number]['id']

/**
 * The cards of one wall, in the model's order.
 *
 * The wall draws them in this order, the phone's index draws them in
 * this order, and the tests read them in this order: one list, one
 * order, two widths.
 */
export function cardsOf(destination: WallDestination): WallCard[] {
  return WALL_CARDS.filter((card) => card.destination === destination)
}

/* ------------------------------------------------------------- the panels */

/**
 * The panels of the wall.
 *
 * A panel is the answer behind a card: the card's "open →" is the door
 * to it, and the panel is drawn on its own page. The list is here so
 * that the panels of the two destinations are written down in one place,
 * next to the cards that own them.
 */
export const WALL_PANELS = [
  // the work
  'week',
  'load',
  'pace',
  'zone-time',
  'volume',
  'bests',
  // the athlete profile
  'shape',
  'facets',
  'challenges',
  'weekday',
  'months',
  'habit',
] as const

/** The name of a panel of the wall. */
export type WallPanel = (typeof WALL_PANELS)[number]

/**
 * The panels of each destination.
 *
 * **A panel belongs to the destination of the card that answers it**, and
 * a test says the two never disagree.
 */
export const PANEL_DESTINATIONS: Record<WallPanel, WallDestination> = {
  week: 'progress',
  load: 'progress',
  pace: 'progress',
  'zone-time': 'progress',
  volume: 'progress',
  bests: 'progress',
  shape: 'profile',
  facets: 'profile',
  challenges: 'profile',
  weekday: 'profile',
  months: 'profile',
  habit: 'profile',
}

/** The destination a panel opens under. */
export function panelDestination(panel: string): WallDestination {
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
  // Every id in the model is in the cards: a test says so, and a card
  // that is not there has nowhere to open.
  return card?.to ?? '/progress'
}
