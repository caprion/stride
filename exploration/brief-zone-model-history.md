# Brief — can the zone model's history be backfilled? (research + implementation proposal)

*Written by the parent thread, 24 November 2026. Research first; a proposal
second; **no implementation** until the parent says so.*

## The problem, precisely

Cleanup Pass 1 landed Z's stage 1: **every run is measured with the zone
model in force on the day of the run.** The seam is
`crates/stride-api/src/zonemodel.rs` (`ZoneModels::read`, `in_force(date)`).

The consequence, flagged by the implementation thread:

> a run predating the first stored model has **no** zone times — for an
> athlete whose first sync is recent, older history loses its zones.

And that is exactly our athlete: **Stride syncs up to twelve months of
activities**, but the **zone model is read from now on** — the first read is
the first sync's zone fetch. So on day one, **most of the imported history
would have no zone times**, which means the ledger's intensity rows — the
whole point of the wedge — would be empty for the part of the window the
athlete cares about.

**So the question is: does the Garmin Personal API expose the athlete's
historical zone models?**

## What to establish

1. **Does the Personal API return zone models with a date range?** The
   current read is `GET /biometric-service/heartRateZones/v2` (used with
   `generateDefaults=false`). Check the official Swagger for a date
   parameter, a variant path, or a separate endpoint that lists **zone models
   over time** — Garmin's watch configuration changed over the years, and
   some Garmin surfaces do expose a history of it.
2. **If it does:** what does each entry carry (the sport, the method, the five
   floors, the date it took effect), and how far back does it go?
3. **If it does not:** then the fallback question must be answered instead —
   and it is a **product** question, not a technical one:
   * **Option A (current rule):** a run before the first stored model has
     **no** zone times. Honest, but it empties the ledger for most of the
     imported history.
   * **Option B:** earlier runs are measured against the **earliest stored
     model**, and the answer **says so** — "measured against the first model
     read on 3 Nov 2026" — rather than claiming a model that did not exist
     on the run's day.
   * **Option C:** the zone-time answers are **withheld** for the pre-model
     period and the app says the history is outside its zone model's reach.

   Give the parent the three, with what each costs, and a recommendation.
4. **What the athlete's own data looks like** — how many runs in a 12-month
   import would be affected under each option, given the real sync window.

## Method

**The specification first** —
`https://developer.garmin.com/gc-develop-program/personal/swagger-docs/`, per
`docs/garmin-api-capabilities.md`. Then, **only if the spec is
inconclusive**, a bounded live read: the `cf-garmin env -- <command>`
launcher (Key Vault credential, never in the repo), a short-lived read-only
process, **sequential and bounded**, **no raw response retained** — field
names, types and presence only; **no measurement values, no identifiers, no
names, no coordinates, no credentials** in any file or message.

Do **not** run Stride's sync, do not write to the local database, do not
change account settings, do not widen to other endpoints, and stop on
authentication, permission or rate-limit failure. Record the endpoint, the
parameters, and the evidence level (`documented`, `live response`).

## Deliverable — `exploration/zone-model-history.md`

* **The answer**: does the API expose the zone model's history? Endpoint,
  parameters, what each entry carries, how far back it goes.
* If not: **the three options**, their costs, and your recommendation.
* **What a backfill would cost**: the request budget (the sync's own bounds
  are in `crates/stride-api/src/state.rs`), where the models would be stored
  (`hr_zone_models` already holds dated models), and how the existing sync
  would change — including the request budget it would consume, since the
  sync's budget is deliberately small.
* **A proposal, not code**: the smallest change that would make Z's rule
  honest across the imported history.
* Also note: the Garmin Personal API's **activity detail** response carries
  `hrTimeInZone_1..5` — the source's own time in zone, per activity. That is
  a **second, independent witness** to a run's zone time, and it may be a
  better answer for historical runs than any model we hold. Say what it can
  and cannot do (is it the watch's own zones at the time? does it agree with
  a stored model? what does it mean for runs with no model in force?).

## Rules

* **Research and a proposal only.** Create only
  `exploration/zone-model-history.md`. Do not modify anything under
  `crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/`. No
  commit.
* Every claim carries an evidence level. No invented fields, no invented
  history.

## Report back (last message, under ~350 words)

* the file created,
* **the direct answer**: can the zone model's history be read from the API —
  yes with endpoint and parameters, or no,
* if no: the three options, and your recommendation in one paragraph,
* what `hrTimeInZone_1..5` can and cannot do for historical runs,
* what you could not verify.
