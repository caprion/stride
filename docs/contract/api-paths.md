# Stride v2 — the frozen route table

FROZEN. Part of the contract freeze (Slice 0). Changing a route is a plan
change and must travel with a PLAN.md edit in the same pull request.

The route table is inherited from the old core where its behaviour is right,
then shaped for v2: every route below exists on **both** hosts, and the phone
reaches them in-process through the FFI — the desktop reaches the same router
over loopback HTTP. Neither can disagree about a number.

Source: `PLAN.md` §6.2. The old implementation's routes live in
`crates/stride-api/src/routes.rs`; the historical OpenAPI surface is captured
in `fixtures/stride-api/openapi_paths.json` (the pre-cut route set, for
reference only).

## Data

| Route | Notes |
|---|---|
| `/activities` | the feed, windowed by date |
| `/activities/{id}` | one run |
| `/physiology/{daily,sleep,hrv,vitals,latest,sources}` | Body inputs: the vitals and their directions (no score) |
| `/fitness/{fingerprint,trends,zones}` | `fingerprint` is **endpoint-only**: the data is already synced, but it has **no screen**, and the OpenAPI document must say so |
| `/stats/{summary,weekly,pace-trends,elevation,personal-records}` | Progress |

## Plan & insight

| Route | Notes |
|---|---|
| `/plan/{current,today,week,blocks,sources,adherence,results,import}` | the plan engine |
| `/insights/{today,progress,chips}` | Today and the feed chips |
| `/compare` | the metric diff |
| `/compare/candidates` | the runs you can compare against (the Compare screen's "choose a different run" action) |

## App

| Route | Notes |
|---|---|
| `/health` · `/health/daily` | liveness and the sync window |
| `/sync/{status,run,runs,history,settings}` | incremental sync |
| `/auth/{status,garmin/personal-key,garmin/personal-key/test,garmin/fit-import,garmin/logout}` | the one credential |
| `/calendar` | the local month calendar (activities per day) — **not** Garmin's scheduled-workout calendar |

## Cut from the old table

`/network/*` · `/tiles/*` · `/gear` · `/auth/garmin/{login,mfa,developer-key,callback}` ·
`/auth/{pair-code,pair,devices}` · `/stats/monthly` · `/stats/hr-zones`

Nothing may reintroduce these. A CI grep asserts that `pair-code`, `pairing`
and `qr` never appear in the route table.

## Discrepancies with `docs/plan.html` §6.2 (recorded, not silently chosen)

1. **`/compare/candidates`** appears in plan.html §6.2 in *both* the "Plan &
   insight" list and the "Cut from the old table" list. This file keeps it:
   the Compare screen's "choose a different run" action needs it, and the
   in-list is the more recent intent.
2. **`/fitness/fingerprint`** was dropped from the `{…}` brace list by the
   route-table fix, but the same commit's note says it "stays (the data is
   already synced) but has no screen". This file keeps it, endpoint-only, per
   that note.
3. **`/physiology/readiness`** is renamed **`/physiology/vitals`** (resolved by
   the plan after Slice 0 flagged it): v2 has no readiness score, so the route
   returns the vitals and their directions. The table above already uses the
   new name.

## The shape of an answer

**The answer is the body, and the status is the status.**

- The router answers with the response **body** — the DTO, and nothing around
  it. Not an envelope, not a wrapper, not a `{"json": "..."}` inside a JSON.
- **A failure is a failure, not a body.** The transport reports it as a
  failure: the HTTP layer puts the status in the status line, and the phone's
  bridge rejects the call. A body that says `{"error": …}` with a 200 is
  never the shape.
- The two hosts therefore hand the app **the same bytes**: the desktop's HTTP
  layer puts the body on the wire, and the phone's plugin hands the body
  across the bridge. A host that answered with an envelope would make the
  app unwrap it — and an unwrap is where a second implementation grows.

## The path the app asks for

- The app asks for `/api/v1/...` — the same path the desktop's HTTP layer
  passes, query string and all.
- **The router matches on the path without the query string.** `/api/v1/activities?limit=10`
  is the route `/api/v1/activities`, with `limit=10` as its parameters. A
  decision made on the raw path — or on a path with a different prefix — is
  a bug, and the phone's bridge exists precisely because the old one made it.
