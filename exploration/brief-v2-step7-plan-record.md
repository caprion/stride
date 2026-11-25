# Brief — step 7: the plan of record, reconciled with what shipped

*Written by the parent thread, 24 November 2026. **Documentation** — plus two
small code fixes the build surfaced.*

## What to do

### 1. Reconcile `PLAN.md` with the whole build

Steps 1–6 have landed on `rig/v2` (38 commits). The plan of record must now
describe **what exists**. Read the commits (`git log --oneline 42e58a0..rig/v2`)
and the files, and:

* **reconcile every amendment** with the code that shipped — the file paths,
  the function names, the routes and their shapes;
* **add the amendments the build made necessary**, in the file's style: the
  **storage** (schema v6 and v7 — the races, the threshold choice, the source's
  own facts, the adopted plan and its sessions), the **decode cache** (keyed on
  the activity id and the file's digest, bounded at 64 runs, and what it does
  not cache), the **plan bases** (the catalogue, the adoption, the dated
  copy), the **rig's own surface** (`/rig` — the index, the status, the
  storage, the connection, and the three first-run decisions), and the
  **desktop bundle check** (`scripts/check-desktop-bundle.sh`, now a gate);
* **correct the passages that the build invalidated** — the plan-bases passage
  that still says *"the catalogue is empty"*, and anything else the build
  changed;
* **retire what is retired**, marking rather than deleting: the pace page's
  route, the old `/insights/pace`.

### 2. Two loose ends the build surfaced

* **The plan-bases card omits the mileage row when the athlete has no runs.**
  That changes an answer's shape, so it needs a **contract note** in
  `docs/contract/api-paths.md` and a `PLAN.md` note beside it. Write both —
  and say plainly in the card's own wording that with no runs there is no
  mileage to compare against, which is a fact and not a failure.
* **The rig's own pages are outside the catalogue**, so the voice lint does
  not reach them: the tools page is still titled *"Backup and restore"* while
  the app says *"Backup, restore and export"*. **Fix the host's wording to
  match the app's**, and add a short amendment saying the host's surfaces are
  outside the catalogue and the lint — a known gap, with the recommendation
  that they come under it.

### 3. The two open items, stated honestly

* the ledger's **repeated-work row** needs a real workout file from the
  athlete's watch — no file in the corpus carries a workout step, and the
  ingest does not yet decode the step-name field, so the row is a no value
  with the core's reason;
* the **80/20 base** is a spec, not a second base: its schedule has not been
  read cell by cell, its cells are minutes rather than miles, and its
  threshold candidate is an LTHR.

### 4. The approval paragraph

The last block of `PLAN.md` should be the paragraph the user reads to approve
the whole set: every amendment by title, the corrections, the rules and the
refusals, the two open items, and explicitly **not** permission to ship.
Everything stays marked *proposal, not approved* until the user says so.

## Rules

* **`PLAN.md`**, `docs/contract/api-paths.md` where the contract's own process
  requires it, and **the desktop host's wording** for the one title fix. No
  other code.
* Follow the file's existing structure and voice; three of its amendments are
  the model.
* Do not commit — the parent commits after the user's approval.

## Report back (last message, under ~350 words)

* what you changed, and the amendments' final titles;
* the reconciliation, per amendment, in one line each;
* the contract note you wrote for the mileage row;
* the two open items as they now read;
* the approval paragraph, in full, so the parent can put it to the user.
