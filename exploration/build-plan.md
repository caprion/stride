# The build — rig v2, end to end

*24 November 2026. The order, the reasons, and what each step must not do.*

## The branches

| branch | what it holds | why |
|---|---|---|
| `v1.0.0` (tag) | the released rig | the shipped app, untouched |
| `rig/existing` | everything shipped so far — the window, the zone model, the cut, the vocabulary | "the existing works as a branch" |
| `rig/v2` | **this build** | the new version of the desktop rig |

`rig/v2` is where all the work below lands, one step at a time, each step
green before the next begins.

## The steps, in order, and why this order

| # | step | why here |
|---|---|---|
| **1** | **the storage** — the athlete's races, the threshold choice, and the source's own activity observations (`hrTimeInZone_1..5`, training effect, effort) | everything else writes or reads these; nothing can be built on a schema that changes underneath it |
| **2** | **the core engines** — VDOT and the race-pace equivalents, the LT band, the ladder's rungs, the physiological-cost curve, the threshold's four candidates, the ledger's two bases | the maths is the product's core, and it belongs in the core where the gates can hold it |
| **3** | **the API** — `/insights/paces`, `/insights/pacing-curve`, `/insights/threshold`, `/insights/stimulus`, the plan's bases and the race times | the surfaces are served by the router, and the route table is frozen — so the routes are added once, deliberately |
| **4** | **the plan bases** — the Pfitzinger 10K and half-marathon programmes as adoptable, dated plans with their pacing | the athlete's plan is the anchor the ledger reads against; it needs the paces from step 2 |
| **5** | **the frontend** — the training-paces card, the cost curve, the threshold card, the ledger's two blocks, the plan's week with its paces | the pictures exist as prototypes; this is them wired to the routes |
| **6** | **the rig** — the desktop host serves the new screens, and the new version's home | the rig is the app's face; the routes exist by then |
| **7** | **the plan of record** — the nine amendments, approved and the stale ones retired | the record should describe what shipped |

## The rules that bind every step

* **Facts, never verdicts.** The ban list (`docs/contract/voice.md`) — no
  advice, no score, no ranking, no *should*, no *missed*.
* **The picture first, the sentence second** (D11).
* **Explanation is not product content** (D22) — it lives in the notes.
* **The surface speaks the runner's language** (D24).
* **Absence is not zero** — a missing value is named, with the reason and
  where it would come from.
* **Every answer names its period, its model and its source.**
* **One wording per idea** — the same thing is said the same way everywhere.
* **The gates**: fmt, clippy `-D warnings`, the workspace tests, check-deps,
  check-corpus (the FIT SHA-256 must not change), the contract greps, the
  secret and privacy scans and their self-tests, the history scan, `tokens`,
  the voice lint, typecheck, unit, viewport. **A gate that did not run is a
  gate that did not pass.**
* **The contract is frozen.** A change needs a `PLAN.md` note in the same
  change.
* **No new dependencies.** Synthetic data only; no Garmin writes; no
  publish.

## What is still waiting on the athlete

* **One structured interval workout**, recorded with a Garmin workout and
  synced. The ledger's repeated-work row is "no value" until it exists.

## The thread sequence

Each step gets its own thread, in this order, and each reports before the
next starts:

1. `the storage and the sync` — schema v6 and the mapping.
2. `the core engines` — the maths.
3. `the api routes` — the surfaces.
4. `the plan bases` — the programmes.
5. `the frontend cards` — the pictures wired.
6. `the rig` — the desktop host and the new home.
7. `the plan of record` — approved and reconciled.

**The parent commits each step** after reading the report and checking the
gates, so `rig/v2` is always a working state.
