/**
 * The reasons the load can have no value, and the line each one gets.
 *
 * The reasons are the **core's** — the api passes the core's own reasons
 * on — and the line is the app's chrome. A reason the app does not know
 * gets the plain one: no value is still no value.
 *
 * The mapping lives here because two screens now read the load: the
 * Progress card and the cockpit's "Form" row. One mapping, one set of
 * words, and the two cannot drift apart.
 */

import type { CatalogueKey } from '../i18n'

export const LOAD_EMPTY: Record<string, CatalogueKey> = {
  no_zone_model: 'progress.load.empty.no_zone_model',
  no_max_heart_rate: 'progress.load.empty.no_max_heart_rate',
  unusable_zone_model: 'progress.load.empty.unusable_zone_model',
  no_history: 'progress.load.empty.no_history',
}
