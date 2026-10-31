# The frozen contract

These four files are the **contract freeze** — the entire point of Slice 0. They
are data, not code: the shapes and rules every later slice is measured against.

| file | what it freezes | source |
|---|---|---|
| [`schema.sql`](schema.sql) | the seven tables, columns, types, indexes, migration order | derived from the old implementation (`backend/app/models.py`, `backend/app/models_phase2.py`, `crates/stride-store/src/schema.rs`), reshaped to the plan's §6.1 |
| [`api-paths.md`](api-paths.md) | the route table, including the cut routes | `PLAN.md` §6.2 (with the discrepancies it resolved, recorded in the file) |
| [`core-contract.md`](core-contract.md) | units, rounding, "no value", the direction band, human rendering | adapted from the old repo's `docs/core-contract.md` |
| [`voice.md`](voice.md) | string types, the closed direction set, the ban list, the per-type rules | `PLAN.md` §0 and `docs/plan.html` §5.9 |

## What "frozen" means

These files do not change after Slice 0. A change to any of them is a **plan
change**, not an implementation detail: it must travel with an edit to
`PLAN.md` (and the matching `docs/plan.html` section) in the **same pull
request**, so the diff shows the plan and the contract moving together.

The reason this matters is the old repo's single largest loss: it started
writing code before the contract was frozen, and the "schema drift" and
"stale-golden" bugs came out of exactly that. The frozen corpus (`fixtures/`)
and the parity differ (`crates/stride-testkit`, wired in a later slice) only
mean something if the contract they encode cannot silently move.

## Where each came from

- **schema.sql** is *not* the old schema copied forward. The old schema had
  score columns, a gear ledger, a per-run sync log and a readiness table; v2
  drops all of them (FIT is the source of truth, views are computed on read,
  no score is ever stored). The mapping is documented in the header of each
  `CREATE TABLE`.
- **api-paths.md** is `PLAN.md` §6.2 verbatim, with the two places the plan
  disagreed with itself resolved and recorded in the file (see "Discrepancies").
- **core-contract.md** keeps the old contract's units/rounding/no-value and
  adds the direction band and the human-rendering rules, because those are what
  make the voice's numbers unambiguous. The old contract remains the oracle for
  the byte-level FIT decoder.
- **voice.md** is the machine-checkable half of the voice model; the lint that
  enforces it is built in Slice 3 (the string catalogue does not exist yet, and
  no product strings are written in Slice 0).
