# Brief — step 1: the storage (schema v6) and the sync

*Written by the parent thread, 24 November 2026. **Implementation.** This is
step 1 of the rig-v2 build; see `exploration/build-plan.md` for the order and
the rules.*

## What to build

**A.** **The athlete's races** — the races the athlete typed, which every
later step reads. A row per race: the distance (in the app's own unit), the
time, the date it was run, the date it was entered, the day it last changed,
and the label the athlete gave it (a closed list: "10K", "half marathon", …).
The store's model is `crates/stride-store/src/model.rs`; the schema is
`docs/contract/schema.sql` and `crates/stride-store/src/schema.rs`.

**B.** **The athlete's threshold** — the choice, dated, tied to the base it was
made for (D19, D20). A row per choice: the kind (a pace band, a heart rate, or
both — a typed candidate may carry a **pair**), the value(s), the **source**
(where the number came from: the watch's zone model, a race, a field test, or
typed), the **rule** (the published rule it was computed under, where one
applied), the base it was made for, the date entered, the day last changed.
**One choice is in force at a time**; the older ones stay in the history and
are read by the dated rule — a plan adopted in March is paced by March's
number, and the card says so.

**C.** **The activity observations** — the source's own facts about an
activity, which arrive on the **search payload we already fetch** (D10, and
the fourth path for the imported history):

* `hrTimeInZone_1..5` — **the watch's own zone minutes** (the elapsed basis),
  which is what makes the imported history readable at all;
* aerobic and anaerobic **training effect** and their messages,
  `trainingEffect`, `directWorkoutRpe`, `directWorkoutFeel`, moderate and
  vigorous minutes, respiration, steps.

**One row per (activity, field)** — the shape `wellness_observations` already
uses: the value, the day the response carried it, the day it was read, and
the source. A field never sent has **no row**; a field sent as null has a row
with a null value; **never a zero for "no time"**.

**Judgements stay out**: VO₂ max, training stress score, activity training
load, lactate threshold, intensity factor, stress, body battery — the
contract excludes Garmin's estimates.

## The migration

**Schema version 6** (the store is at 5). Read
`crates/stride-store/src/schema.rs`, `crates/stride-store/src/legacy.rs` and
`docs/contract/schema.sql` first — the repo has a **migration story** and a
**legacy** path for older databases, and this change must follow it exactly.
The export and backup carry every table (`crates/stride-store/src/export.rs`),
so the new tables are in the export and in the backup, with their counts in
the manifest. The **privacy** rules (`scripts/privacy-rules.sh`) apply: these
are the athlete's own facts, stored locally, and the export must say so.

## The sync

* the search payload Stride already fetches now carries `hrTimeInZone_1..5`
  and the other fields — **capture them in the listing loop; no extra
  request**. The sync's request budget (`crates/stride-api/src/state.rs`) is
  deliberately small and **must not grow**;
* the athlete's typed races and threshold choices are **not** synced — they
  are entered in the app, and they are the athlete's own;
* a failed or partial read leaves the previous values in place, as
  `wellness_observations` already does.

## Rules

* **Implement; do not re-plan.** The plan and the decisions are the
  specification. Where the code and the plan disagree, **stop and report**.
* **Small commits**: the migration and the tables; the store's model and
  repositories; the export and the backup; the sync mapping; the tests.
* **The gates**, per commit: fmt, clippy `-D warnings` (no unwrap, no
  expect, no panic, no todo), `cargo test --workspace`, `scripts/check-deps.sh`,
  `scripts/check-contract-greps.sh`, `scripts/privacy-rules.sh`, the secret and
  privacy scans and their self-tests, `scripts/check-corpus.sh` (the FIT
  fixtures' SHA-256 must not change), `scripts/scan-history.sh`, and the
  frontend's `tokens:check`, `lint:voice`, `typecheck`, `test:unit`.
* **The contract is frozen** — a schema change travels with a `PLAN.md` note
  and with `docs/contract/schema.sql` beside it. Read
  `docs/contract/README.md` for what a change to a frozen file requires.
* **No new dependencies** (`scripts/approved-deps.txt`). Synthetic data only;
  no Garmin writes; no publish; no network beyond the app's own sync.
* **Commit your work** on `rig/v2`, one commit per step, with a message that
  says what changed and which gates passed — the parent reviews the history
  in the morning.

## Report back (last message, under ~450 words)

* the migration, in three sentences: what version, what tables, what happens
  to an existing database;
* the new tables and their columns, and where each is written;
* how the sync captures the search payload's fields, and the proof that **no
  extra request** was added;
* how a field that was never sent, and one sent as null, are stored;
* the export and backup changes;
* the commits, and the gates each passed;
* anything the plan and the code disagreed about.
