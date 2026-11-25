# Brief — step 4c: the adopted plan (storage v7) and the adoption route

*Written by the parent thread, 24 November 2026. **Implementation.** The other
half of step 4: `crates/stride-core/src/plan_bases.rs` exists and adopts a
programme in memory; now the adoption must be **stored** and **written**.*

## What to build

### 1. The storage — schema version 7

An **adopted plan is a dated copy** (B1), so the plan's weeks are stored with
it — a later change to the catalogue must never alter a plan that is already
running.

* `adopted_plans` — the programme's id **and its version**, the tier, the
  **start date**, the goal race's distance and date, the **threshold the plan
  was paced with** (value, date, source), the base, the day it was adopted,
  and whether it has pacing at all (a plan adopted before a race was typed
  carries its sessions without paces, and says why).
* `adopted_sessions` — the plan's **dated sessions**: the date, the kind, the
  distance, the pace (where it has one), the pacing's source (which rule, and
  which threshold), and the **detail** the book prints for that session
  ("3 × 8 min, jog 3"; "6 × 12 s uphill + 6 × 100 m strides").

Follow the repository's migration story exactly — read
`crates/stride-store/src/schema.rs`, `crates/stride-store/src/legacy.rs` and
`docs/contract/schema.sql` first. The migration must be **idempotent**, an
older database must take it in one transaction, and an export of an older
schema must still be refused. The **export and the backup** carry the new
tables with their counts in the manifest, and the **privacy** rules apply:
these are the athlete's own facts, held locally.

**Why store the sessions rather than derive them from the catalogue:** the
plan's paces were worked out **once**, at adoption, from the threshold in
force that day (B4). If the sessions were derived, a catalogue revision or a
new threshold would silently re-pace a running plan. Storing the dated copy
is what makes "a later threshold does not re-pace a running plan" true.

### 2. The routes

| route | what it does |
|---|---|
| `POST /plan/adopt` | the athlete adopts a programme: the programme's id, the **tier**, the **start date**, the goal race's distance and date, and which threshold to pace with (a candidate, or a typed value or pair) |
| `GET /plan/adopt` | what is adopted: the plan's programme, tier, dates, its pacing and the threshold it was paced with, and the **entry requirement beside the athlete's own weekly mileage** |
| the existing plan route | extended to return the **adopted** week's sessions — their kinds, distances, paces where they have one, and the plan's source |

**The refusals** (each with the core's own reason): a tier that does not
exist; a start date that leaves no room for the twelve weeks and the race
week; a race date that is not on the plan's shape; a **second plan in force**
at the same time (one plan at a time — D13/D14).

**What is *not* a refusal:** the athlete's weekly mileage being below the
tier's **entry requirement**. The app states the requirement beside their own
mileage (B2) and the athlete chooses — it never suggests, and it never blocks.

**A plan may be adopted before a race is typed** — its sessions then carry
their kinds and distances and **no paces**, with the reason, and the plan
records that it has no pacing yet. The paces are worked out **once**, when
there is something to work them out from.

### 3. The phone's bridge and the contract

The route table is frozen: a new route must be added to
`android/.../bridge/PhoneRoutes.kt`'s table **and** its test (the parity
check compares the two), and `docs/contract/api-paths.md` must gain the row
— with a **`PLAN.md` note** in the same change, because a frozen file changes
only with a plan note. The phone does not need screens for this; the pace
page is the precedent — a route with no screen.

## Rules

* **The core renders the sentences**; the API carries rendered values.
* **The ban list** and the closed direction set: no verdict, no advice, no
  score, no ranking, no *should*, no *missed*.
* **The surface speaks the runner's language** (D24).
* **Absence is not zero**: a session with no pace is a session with no pace,
  with the reason.
* **Every answer names its period, its model and its source** — the plan's
  source, the threshold's value and date.
* **No new dependencies**; synthetic data only; no Garmin writes; no publish.
* **The gates**, per commit: fmt, clippy `-D warnings`, `cargo test
  --workspace`, check-deps, the contract greps, the secret and privacy scans
  and their self-tests, `tokens:check`, `lint:voice`, typecheck, unit,
  viewport.
* **Commit your work** on `rig/v2`, one commit per piece.

## Report back (last message, under ~400 words)

* the migration and the two tables;
* the routes, their bodies and their refusals;
* an adopted plan end to end: the programme, the tier, the start date, the
  race, the threshold it was paced with, and **week 3's seven sessions** with
  their paces;
* the export and backup changes;
* the bridge and contract changes, and the `PLAN.md` note;
* the commits and the gates each passed.
