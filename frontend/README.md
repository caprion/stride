# Stride v2 — the frontend

The phone's screen, written once for every width.

**Four destinations, always:** Today · Runs · Progress · Body. Compare and
Plan week are contextual — you reach them from a run or from Today, and they
are not part of the destination set.

```
npm --prefix frontend install
npm --prefix frontend run dev          # http://localhost:5173
npm --prefix frontend run build        # the bundle, in frontend/dist
```

## The bands

| band | width | arrangement |
|---|---|---|
| compact | `< 600px` | a bottom tab bar |
| medium | `600–1023px` | a top bar, the column capped at 680px |
| wide | `≥ 1024px` | a left rail, the column capped at 880px |

The band is decided by **container** width, in CSS (`@container shell`), not
by the viewport: split-screen, an unfolded foldable and a resized window all
land correctly without a reload, and no JavaScript reads the viewport for
layout. All three arrangements are in the markup; the container query picks
one. The destination set is therefore identical at every width by
construction, not by a test.

The shell uses `100dvh`, honours the safe-area insets, never scrolls
sideways, and every tap target is at least 44px tall at compact.

## The screens

| route | screen |
|---|---|
| `/` | **Today** — the vitals, each with its value, its direction word and the muted arithmetic; then the Rest summary; then the plan's own session and its other options, unranked and unlinked |
| `/runs` | **Runs** — the feed; every row carries a sentence about what the run meant |
| `/runs/:id` | **Activity** — the run, its story, its splits, its elevation |
| `/progress` | **Progress** — the trajectory, forgivingly; no percentage without a real baseline |
| `/body` | **Body** — the same values and directions as Today, plus the patterns, each with its n |
| `/plan` | **Plan week** — the week, and today's plan: *done · planned · not done* |
| `/compare/:id` | **Compare** — `metric · A · B · Δ`, the basis always named, swapping A and B flipping every delta |

## The voice

Every string lives in `src/i18n/catalogue.ts`, typed and closed:

```ts
export const catalogue = {
  'today.state.count': { type: 'status', text: '{n} of 3 signals below your usual' },
  // …
} as const
```

The lint (`npm --prefix frontend run lint:voice`) fails the build on:

- a banned word or construction — *should · must · take the day · rest ·
  go easy · missed · bad · poor · fail · behind · ready to · recommended ·
  better to · need to*;
- a `direction` string outside the closed set — *better than usual · about
  usual · could be better*;
- a `pattern` string with no sample size;
- an `option` string that mentions a vital (an option is never linked to a
  vital);
- a banned word written inline in a component instead of in the catalogue.

The lint is a function (`scripts/voice-lint.mjs`) and is tested against
deliberately bad catalogues: a scanner that never fails is worse than no
scanner.

## The API client

`src/lib/api.ts` is one typed client over the frozen route table
(`docs/contract/api-paths.md`). One transport, two implementations:

- `src/lib/transport.ts` — the native Capacitor bridge when it is there,
  `fetch` over loopback HTTP otherwise. The same request, the same response
  shape, whichever host the app is on.

```ts
const client = createApiClient(transport)
client.insightsToday()
```

The contract froze the **routes**; it did not freeze the **response
bodies**. The shapes this client speaks are written down in
`src/lib/types.ts`, and they are a proposal for the backend slice to match.
Two names are worth knowing about: `TodayInsight.planned` is the plan's own
session for today, and `TodayInsight.options` are its other options.

## The fixtures

`src/fixtures/index.ts` is a synthetic, typed fixture set — no athlete, no
real numbers, no real dates. With `?fixtures=1` the app answers from the
fixtures instead of the network, so every screen can be rendered, reviewed
and tested without a backend. `?fixtures=over` selects the one variant whose
last run is beyond 10% of the plan's target — the only fixture where the
distance comparison appears.

## The checks

```
npm --prefix frontend run typecheck     # tsc, app and node configs
npm --prefix frontend run lint:voice    # the voice contract
npm --prefix frontend run test:unit     # vitest (happy-dom)
npm --prefix frontend run test:viewport  # the viewport matrix, on a real browser
npm --prefix frontend test               # all four, in that order
npm --prefix frontend run build          # the bundle
```

The **viewport matrix** (`scripts/viewport-matrix.mjs`) runs seven screens at
six widths — 360×640, 390×844, 430×932, 768×1024, 1024×768, 1280×800 — in
Chromium, and asserts: no horizontal page scroll, no content column scroll,
nothing sticking out past the right edge, no clipped labels, tap targets of
at least 44px at compact, the same four destinations at every width, and
the screen's primary action reachable without a sideways scroll.

It needs a browser: `npx playwright install chromium` (the `playwright-core`
in `devDependencies` does not download one by itself).
