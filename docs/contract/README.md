# The frozen contract

These four files define the shared schema, API and domain/voice rules.
They began with the Slice 0 freeze and now include deliberate, recorded
amendments. The implemented Rust core and both hosts are measured against them.

| file | what it freezes | source |
|---|---|---|
| [`schema.sql`](schema.sql) | the seven tables, columns, types, indexes, migration order | derived from the old implementation (`backend/app/models.py`, `backend/app/models_phase2.py`, `crates/stride-store/src/schema.rs`), reshaped to the plan's §6.1 |
| [`api-paths.md`](api-paths.md) | the route table, including cut routes and deliberate additions | original HTML design section 6.2, with recorded discrepancies and approved plan amendments |
| [`core-contract.md`](core-contract.md) | units, rounding, "no value", the direction band, human rendering | adapted from the old repo's `docs/core-contract.md` |
| [`voice.md`](voice.md) | string types, the closed direction set, the ban list, the per-type rules | `PLAN.md` §0 and `docs/plan.html` §5.9 |

## What "frozen" means

These files do not change silently. A change to any of them is a **plan
change**, not an implementation detail: it travels with an edit to
`PLAN.md` (and the matching `docs/plan.html` section) in the **same pull
request**, so the diff shows the plan and the contract moving together.

The reason this matters is the old repo's single largest loss: it started
writing code before the contract was frozen, and the "schema drift" and
"stale-golden" bugs came out of exactly that. The frozen corpus (`fixtures/`)
and the implemented parity differ (`crates/stride-testkit`) only
mean something if the contract they encode cannot silently move.

## Where each came from

- **schema.sql** is *not* the old schema copied forward. The old schema had
  score columns, a gear ledger, a per-run sync log and a readiness table; v2
  drops all of them (FIT is the source of truth, views are computed on read,
  no score is ever stored). The mapping is documented in the header of each
  `CREATE TABLE`.
- **api-paths.md** began with the original design's route table, resolving its
  discrepancies explicitly. It now also records implemented settings,
  connection and history additions; it is not an unamended copy of the first plan.
- **core-contract.md** keeps the old contract's units/rounding/no-value and
  adds the direction band and the human-rendering rules, because those are what
  make the voice's numbers unambiguous. The old contract remains the oracle for
  the byte-level FIT decoder.
- **voice.md** is the machine-checkable half of the voice model. The Rust
  catalogue exports its strings and rules in `crates/stride-core/strings.json`;
  the frontend catalogue and voice lint enforce the same closed types and rules.

Current product scope is in [PLAN.md](../../PLAN.md), including the approved
reset and responsive amendments. [The original HTML design](../plan.html) is
historical; [the media gallery](../media/README.md) shows the current interface.
