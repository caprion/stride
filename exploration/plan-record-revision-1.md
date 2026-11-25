Parent: the nine amendments you wrote are on `rig/v2` (with the prototypes).
Since then the **build has landed three steps**, so please reconcile the plan
of record with what actually shipped, and prepare the set for the user's
approval.

## What has shipped since your draft (on `rig/v2`)

* **step 1 · the storage** — schema **v6**: `race_observations`,
  `threshold_observations`, `activity_observations`. The sync captures the
  source's own fields (`hrTimeInZone_1..5`, training effect, effort) from the
  **search payload it already holds** — **no extra request**, proved by a
  test that compares the request counts. The races and the threshold choice
  are the athlete's own and are never synced. The export and the backup carry
  all three tables.
* **step 2 · the core engines** — `vo2.rs` (the effort model: VDOT from a
  race time, the race-pace equivalents, the cost curve), `training_paces.rs`
  (the ladder's rungs, each with its value, its band and its rule and
  citation), `threshold.rs` (the four candidates, the choice in force, the
  dated history), `stimulus.rs` (the ledger, the weeks matrix, the two
  bases, the overlap).
* **step 3 · the API routes** — in progress: `/insights/paces`,
  `/insights/pacing-curve`, `/insights/threshold`, `/insights/stimulus`,
  the athlete's races, the threshold choice, and a **bounded decode cache**
  so the ledger's read is not sixty file decodes per request.

## What to do

1. **Reconcile each of the nine amendments with what shipped** — the rule it
   serves is unchanged, but the *file paths*, the function names and the
   routes must be the ones that exist. Say, for each amendment, what changed
   since your draft.
2. **Add the amendments that the build made necessary**, in the same style:
   * the **storage** (the three tables, the dated facts, the no-extra-request
     rule, and that the athlete's own facts are never synced);
   * the **decode cache** (one decode per run, bounded — and the rule that a
     derived value is recomputed on read and never stored);
   * the **plan bases** (the four programmes as a versioned catalogue, the
     athlete's tier, the entry requirement stated beside their mileage, the
     paces worked out once at adoption from the threshold in force, and that a
     later threshold does not re-pace a running plan).
3. **Fix the stale statements** you found: the four-versus-five destinations,
   the five string types against the contract's seven, the 17-entry ban list
   against the contract's 23, and §2's scope in both directions. These are
   **corrections**, not proposals — the user has already decided them.
4. **Retire the five superseded amendments** you identified, by marking them
   as retired with the date and the amendment that replaces each — do not
   delete them; the file's history is its value.
5. **Note the two open items honestly**: the ledger's repeated-work row needs
   a real workout file from the athlete's watch (no file in the corpus
   carries a workout step), and the 80/20 base is a **spec** — its schedule
   has not been read cell by cell, so it is not a second base yet.
6. Keep everything marked **proposal, not approved** — the user has not
   approved the set yet. Say what the approval would cover, in one
   paragraph, so the user can approve it in one go.

## Rules

* **`PLAN.md` only**, plus `docs/` if a contract change is genuinely required
  (it should not be). No code.
* Follow the file's existing structure and voice; three of its amendments
  are the model.
* Do not commit — the parent commits after the user's approval.

## Report back (last message, under ~350 words)

* what you changed, and the amendments' final titles;
* the reconciliation, per amendment, in one line each;
* the paragraph the user can read to approve the set;
* anything in the shipped code you think the plan does not yet describe.
