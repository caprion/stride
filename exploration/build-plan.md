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

---

## The build, as it happens

| step | state | note |
|---|---|---|
| 1 · the storage | **done** | schema v6 — the races, the threshold choice, the activity observations; the sync captures the source's own fields with no extra request; five commits, green |
| 2 · the core engines | **done** | `vo2.rs`, `training_paces.rs`, `threshold.rs`, `stimulus.rs`; four commits, green. VDOT 49.6 (the prototype's data file said 49.7 — a rounding difference, corrected there) |
| 3 · the API routes | running | the ladder, the curve, the threshold, the ledger, the athlete's races and threshold choice, and the decode cache |
| 4a · the plan programmes as data | **done**, and it found a fault | the four programmes transcribed cell by cell — and the transcription found that the **spine's own HM tier 2 table does not reconcile with its printed volumes**, and that two of its weeks are identical. The spine is being re-read from the book |
| 4b · the core's plan module | waiting on 4a's correction | the templates, the adoption, the dated paces |
| 5 · the frontend cards | queued | the pictures wired to the routes |
| 6 · the rig | queued | the desktop host and the new home |
| 7 · the plan of record | queued | the nine amendments, approved, and the stale ones retired |

**The transcription fault is fixed.** The schedules print as two images per
plan with different row heights; the first pass paired them by pixel
position and silently shifted rows. They are now aligned by row index and
checked by sum — **54 week-rows, 0 mismatching**. The "identical cells,
different totals" was the transcription's error, not the book's: weeks 4 and
2 of HM tier 2 are the same week twice, both 52 miles.
