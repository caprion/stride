# Brief — does Garmin's API carry per-activity temperature (and what else)?

*Written by the parent thread, 24 November 2026. Follow it exactly.*

## The question

The user reports: **Garmin Connect shows temperature details for a run, and
they may not be in the FIT file.**

That matters because our FIT record messages carry no temperature at all:
`exploration/stimulus-ledger.md` §5 parsed the record-message definitions of
all sixteen files in `fixtures/fit/` (including the three real Garmin files)
and **none carries FIT field 13** — the device recorded no temperature. So if
Garmin's *service* holds a temperature for an activity, it must come from
somewhere else (the device's weather data synced separately, a nearby
weather station, or the service's own model) and it would arrive through the
**API**, not the FIT.

Two things to establish, in order of preference:

1. **Does the Garmin Personal API expose a per-activity ambient temperature?**
   Which endpoint, which field name, which units, and where the value
   comes from (device vs weather station vs service estimate).
2. **What else does the activity detail carry that Stride currently throws
   away** — especially the fields that would help the training analysis:
   training effect, aerobic/anaerobic training effect, "recorded effort" / RPE,
   activity's own aerobic/anaerobic effect, VO2 max, average/max HR, and any
   condition or performance-context fields (heat, altitude, wind).

## Method

**First: the official specification.** `https://developer.garmin.com/gc-develop-program/personal/swagger-docs/`
(the local note `docs/garmin-api-capabilities.md` links to it). Read the
OpenAPI schema for:

* `/activity-service/activity/{activityId}` — the **detail** endpoint
  (Stride has a client method for it but does not call it in normal sync);
* `/activitylist-service/activities/search/activities` — the **summary**
  endpoint Stride does call.

Report the **field names and types** for anything temperature-, condition- or
training-effect-shaped. Say plainly if the spec has no temperature field.

**Second, only if the spec is inconclusive: one bounded live read.**
`docs/garmin-api-capabilities.md` has the rules; follow them exactly:

* use the `cf-garmin env -- <command>` launcher (it is on PATH at
  `~/bin/cf-garmin`) so the credential comes from Key Vault and never touches
  the repo;
* a **short-lived, read-only** process; **one** activity's detail and **one**
  search page, sequential, bounded;
* **retain no raw response** — record only field names, types and whether a
  value is present or absent;
* **do not run Stride's sync**, do not write to the local database, do not
  change account settings, do not publish anything;
* stop on authentication, permission or rate-limit failure and report it;
* if the credential is unavailable, say so and stop — do not widen to other
  endpoints or other data.

**Never** report a measurement value, an activity id, a name, a location or a
coordinate. Field names and presence only.

## Deliverable — `exploration/garmin-temperature-and-activity-fields.md`

* **The answer**: is there a per-activity temperature? Endpoint, field name,
  units, source, and **how often it would be present** (if the spec or the
  live read says anything about coverage — for example, only for activities
  where the device had weather data).
* **The other fields** the activity detail carries that Stride discards
  today, in a table: field name, type, what it would mean, and whether it is
  an *observation* (a fact about the activity) or a *judgement* (Garmin's
  own estimate — note that Stride's contract treats Garmin's training status
  and readiness as judgements and keeps them out of the core).
* **What it would cost**: where a temperature would live (a new table beside
  `activities`, or an observation row — the store already has
  `wellness_observations` with `(day, source, field, item)`, the value, the
  day the response carried and the day it was read), what the schema change
  looks like, what the backfill looks like (12 months of history), and how
  **absence** must be handled (a missing value with a reason — never zero,
  never the day's average substituted in).
* **The product question, answered honestly**: the user has clarified that
  "running too hot or too cold" means **exertion and load, not weather**, so
  the environment axis is out of scope. Say what a per-activity temperature
  would still be good for — the run detail's context, explaining a slow
  session, the run update's facts — and what it must **not** be turned into
  (no heat-adjusted pace, no adjustment of any measurement, no "the heat
  cost you 4%" claim). If the honest answer is "a nice-to-have fact on one
  screen, not an analysis", say that.
* If the answer is "there is no temperature in the API either", say so
  plainly — that closes the question for good.

## Rules

* **Read-only research.** Do not modify anything under `crates/`,
  `frontend/`, `android/`, `desktop/`, `PLAN.md` or the contract. The only
  file you create is `exploration/garmin-temperature-and-activity-fields.md`.
* Do not commit or push.
* No measurement values, no identifiers, no names, no locations, no
  credentials in the document. Field names and presence only.
* If a live read fails, report the failure and the spec-only answer; do not
  retry aggressively.

## Report back (last message, under ~300 words)

* the file you created,
* **the direct answer**: is there a per-activity temperature in the API, or
  not (with endpoint and field name, or with the spec's silence),
* the list of other discarded activity-detail fields worth keeping,
* what a temperature would be good for, in one paragraph, and what it must
  never be turned into,
* anything you could not verify.
