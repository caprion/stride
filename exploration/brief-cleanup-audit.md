# Brief — the cleanup audit (read-only, before any new code)

*Written by the parent thread, 24 November 2026. Follow it exactly.*

## Why this thread exists

The user wants the codebase **cleaned up before any new feature code is
written**. Two design threads have already produced the shape of what comes
next:

* `exploration/surface-inventory.md` — the surface cut (24 Progress cards →
  11, three run-detail charts → one, the `/pace` page retired, and a list of
  panels that are duplicates of each other).
* `exploration/stimulus-ledger.md` — the ledger card and the core module that
  would implement it.

Your job is **not** to write the cleanup. It is to produce the **ordered,
priced cleanup plan** the parent can approve, with every file path named, so
the cleanup is a plan and not a pile of good intentions.

## What to produce — `exploration/cleanup-plan.md`

### 1. What is actually dead or duplicated today

Read the code, not the docs. For each item: the file path, the symbol, and
**how you know** it is unused (no route, no import, no reference).

* **Routes** — `crates/stride-api/src/router.rs`, `crates/stride-api/src/routes/`.
  Which routes are unreachable from the UI, and which the UI reaches but the
  surface cut removes (the `/pace` page and its route are the known case).
* **Frontend** — `frontend/src/pages/`, `frontend/src/components/`,
  `frontend/src/app/`. Which components become unused after the cut; which
  panels draw the same data twice; where the same string is written twice
  (the catalogue vs the component).
* **Core** — `crates/stride-core/src/`. Any module whose output no route
  reads any more; any function that exists in two modules with different
  answers (the **two walking thresholds** — `crates/stride-ingest/src/segment.rs`
  at sample level and `crates/stride-core/src/fingerprint.rs` at lap level —
  are the known case; say what one rule would look like and where it would
  live).
* **Strings** — `crates/stride-core/src/strings.rs` and
  `frontend/src/i18n/catalogue.ts`. Which catalogue entries lose their user
  after the cut, and which are needed by the new ledger strings. The lint in
  `scripts/check-contract-greps.sh` and the catalogue CI check will care.
* **Drift between the documents and the code**: `PLAN.md` says four
  destinations; `README.md` says four; `frontend/src/app/destinations.ts`
  ships five (Profile). Find every other such drift you can — a rule in
  `PLAN.md` that the code no longer obeys, a module the plan describes that
  does not exist, a contract clause (`docs/contract/`) that the code breaks.
  List them as **facts with paths**, and say which need the user's approval
  to change a document.

### 2. The changes the new features need, as cleanup, not as features

The consolidated chart and the ledger both need small changes to existing
things. Price them as part of the cleanup:

* `SamplePoint` gains `distance_m`, `altitude_m` and the movement kind, and
  the separate `elevation` array goes (`crates/stride-api/src/dto.rs`,
  `crates/stride-api/src/routes/series.rs`) — what else reads it?
* **One window definition**: the Progress totals use calendar weeks, the
  rolling route uses a rolling N-day window. Say which one the product
  should standardise on, what it costs, and what breaks.
* **One zone-model rule**: each run measured with the zone model in force on
  the day of the run (the store keeps dated models in `hr_zone_models`),
  instead of the most recent model. Name every reader of the zone model
  (`crates/stride-core/src/load.rs`, `zones.rs`, `rolling.rs`, the api routes)
  and say what changes for each. This one is large — price it honestly.
* The API change the ledger needs (`SamplePoint` decoded per run; 61 file
  decodes on a 12-week window) and where a cache would go.

### 3. The order, and the gates

* A **numbered order of execution**, each step small enough to land as one
  commit, each with: the files it touches, **what could break**, and the
  test that proves it.
* The **gates** each step must pass — read `scripts/gate.sh`,
  `scripts/check-deps.sh`, `scripts/check-contract-greps.sh`,
  `scripts/privacy-rules.sh` and `CONTRIBUTING.md`, and list the checks that
  apply to a cleanup step (the dependency DAG, the string lint, the privacy
  scan, the clippy lints: no unwrap, no expect, no panic, no todo).
* A **do-not-touch list**: what must not change during a cleanup (the
  contract's voice rules, the schema, the sync behaviour, the FIT retention,
  the privacy posture).

### 4. Risks and traps

* Which cleanups are **behaviour changes disguised as tidying** (they are:
  say which ones and how to tell the user apart).
* Which cleanups would **break the Android or desktop hosts** if the API
  changes shape (the FFI seam, the desktop bundle sync script
  `scripts/sync-desktop-bundle.sh`).
* Where the **generated or frozen** things are (`frontend/src/i18n/catalogue.ts`
  is checked by CI; `crates/stride-core/src/strings.json` is exported by the
  core) and what must be regenerated rather than hand-edited.

## Rules

* **Read-only.** Do not modify anything under `crates/`, `frontend/`,
  `android/`, `desktop/`, `PLAN.md`, `docs/` or the contract. The only file
  you create is `exploration/cleanup-plan.md`.
* Do not commit or push.
* Every claim names a path. If you are unsure whether something is used,
  say so and say how to check (`rg` for the symbol, the route table, the
  catalogue).
* No code. This is a plan a reviewer can approve line by line.

## Report back (last message, under ~400 words)

* the file you created,
* the **top ten items** to clean, in order, with the path for each,
* the two items where the cleanup is really a behaviour change and the user
  must be told,
* the drift you found between the documents and the code,
* the gates a cleanup commit has to pass.
