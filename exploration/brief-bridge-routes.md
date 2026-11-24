# Brief — B1: the Android bridge is missing 11 routes (bug fix, first)

*Written by the parent thread, 24 November 2026. This is a **bug fix**, not a
design change: the phone currently cannot reach eight features the API
serves.*

## The defect

`crates/stride-api/src/router.rs` serves **63** route patterns.
`android/app/src/main/java/app/stride/running/bridge/PhoneRoutes.kt` names
**52**. The eleven the bridge does not name:

```
/insights/load
/insights/rolling
/insights/pace
/insights/challenges
/insights/streaks
/activities/{id}/fit
/activities/{id}/film
/activities/{id}/update
/activities/{id}/update/payload
/activities/{id}/update/request
/update/settings
```

So on the phone today: the **Progress wall's load card**, the **rolling
window**, the **pace panel**, **challenges**, **streaks**, the **FIT
download**, the **run film** and the **run update** (the model's reading of a
run) all 404. The desktop host has the same routes and does not have this
problem, so the bug is in the bridge's table only.

Verified by diffing the two lists — please re-verify it yourself rather than
trusting the list above.

## What to do

1. **Read the bridge's dispatch** and its existing route table, and add the
   eleven missing patterns in the file's own style and its own order (the
   table appears grouped by area — follow that grouping, do not append a
   block at the end).
2. **Check each route's shape** against the API: the response types, the
   query parameters each one takes, and the bridge's existing handling of
   them. A route that needs a parameter the bridge cannot send is a **finding
   to report**, not something to paper over — e.g. if `/insights/rolling`
   needs a `weeks` parameter, the bridge must be able to pass it, and if it
   cannot, say so.
3. **Check whether the phone's UI actually calls these** — grep the Kotlin
   sources for each path. If the phone calls a path that is *not* in the API
   (the reverse gap), that is a second defect: report it, and fix it only if
   the fix is unambiguous.
4. **Contract drift, in the same change.** `docs/contract/api-paths.md` has a
   list of the routes the app reads, and it is wrong in both directions:
   `/plan/import` and the `/auth/*` routes **are** read, while
   `/fitness/trends` and `/health/daily` are **not**. Correct that list so it
   matches the code after your change, and keep the contract's style. The
   contract is frozen — read `docs/contract/README.md` and the rules for
   changing it before editing, and say in your report what the contract
   requires for a correction of this kind.
5. **Tests.** Add or extend whatever test covers the bridge's route table, so
   the eleven paths cannot go missing again. If the repository has a
   contract-grep script (`scripts/check-contract-greps.sh`) that can assert
   the bridge and the API agree, extend it — a mechanical check is worth more
   than a hand-written test.

## Rules

* **This is a bug fix.** Do not change the API's routes, their behaviour, or
  the core. Do not refactor anything else. Do not touch `PLAN.md`, the
  contract's *rules* (only the route list above, if the contract's own
  process allows it), or anything under `crates/`.
* Keep the change minimal and in the repository's style. No new
  dependencies (`scripts/approved-deps.txt`).
* No secrets, no credentials, no network calls to a live Garmin account, no
  production data. Use synthetic data and the existing test fixtures.
* Run the gates that apply to an Android/Kotlin change and report the
  results: the Android unit tests and lint if they run in this environment,
  `scripts/check-contract-greps.sh`, `scripts/privacy-rules.sh`, and the
  frontend/contract checks if your change touches shared files. If a gate
  cannot run here, say so rather than claiming it passed.
* **Do not commit or push.** Leave the change in the working tree; the parent
  will review it.

## Report back (last message, under ~350 words)

* the files you changed, and the diff in brief,
* the eleven routes now in the bridge's table, and the order you put them in,
* for each route: does the phone already call it, and does it need a query
  parameter the bridge could not previously send (and what you did about it),
* the contract's route list — what you changed and under which rule,
* the tests you added or extended, and the gate results (or why a gate could
  not run).
