# The merge gate

Nothing merges without passing all six. They are in this order because each
one catches what the previous one cannot.

| # | gate | what it catches | who runs it |
|---|---|---|---|
| 1 | `bash scripts/gate.sh` | a broken build, a broken lint, a broken test, a broken corpus, a scanner that stopped working | CI, and every agent locally |
| 2 | `scripts/scan-pr.sh` | a credential, a coordinate, a Garmin profile value, a free-text note, an unexplained dependency or an unprovenanced file **in the change itself** | CI, on every pull request |
| 3 | the contract greps | a route that reintroduces pairing or a QR; string-built SQL; `unsafe` outside the FFI | CI (`scripts/check-contract-greps.sh`) |
| 3b | the history scan | a credential, a coordinate, a name or a note that a commit added and a later commit removed | CI (`scripts/scan-history.sh`) |
| 4 | an independent review | a **reachable** problem the tests do not cover — a wrong number, a wrong rounding, a missing guard, a false claim in a document | a second agent, on a clean clone, with the brief "report, do not fix" |
| 5 | the integrator's own read of the diff | the things a reviewer and a scanner both miss: a design that does not match the plan, an API shape that drifted, a sentence that breaks the voice | the integrator, every time, before approving the merge |
| 6 | the plan | a change that ought to be a plan change | a plan edit travels in the same pull request, or it does not happen |

## The rules behind it

- **A frozen contract changes only with the plan.** `docs/contract/*` is frozen
  after Slice 0. A change to one travels with the matching edit to `PLAN.md`
  **and** `docs/plan.html`, in the same pull request, and the reason is written
  down.
- **The schema does not change.** A desktop export has to open on the phone.
- **The API shapes do not change.** The frontend consumes them.
- **No new dependency** without a justification in the pull request description.
- **The voice is linted, not reviewed.** No imperatives, no verdicts, no score.
  The direction words are a closed set.

## What "inspect all data" means here

Concretely, before a pull request is merged:

1. **Every added line** has been through `scan-pr.sh` — a credential, a
   coordinate in the plausible band, a Garmin profile value, an email, a phone
   number or a free-text note fails the build.
2. **Every binary in the corpus** has been decoded and its positions asserted
   to be within 1° of the synthetic anchor (`crates/stride-testkit`, run by
   `cargo test`).
3. **Every corpus file** has provenance: where it came from and what was
   changed, in `fixtures/README.md`. A corpus file changed without that log
   fails `scan-pr.sh`.
4. **The repository history** carries no personal data — checked on every
   ref, not just the tip, by `scripts/scan-history.sh`, which walks every
   commit on every ref and applies the same rules to the lines each commit
   added. A leak that was added and later removed is exactly what the tree
   scanners cannot see. This repository was born clean; a regression here is
   severe, because the previous implementation's public history carried the
   athlete's real coordinate pair, name and Garmin id.
5. **The build threads never saw the key.** The Garmin Personal API key is
   fetched at runtime from Azure Key Vault and held in one process's memory.
   Nothing in the tree, in a fixture, in a test or in a transcript contains it.

## The reviewer brief

The reviewer is given the area, the contract that area must satisfy, and the
instruction to report rather than fix. It gets a **clean clone** — never the
builder's worktree — so it reads what the pull request says, not what the
builder intended.

A finding must be a real, reachable problem with the path that reaches it.
"Could be improved" is not a finding. If an area is clean, the reviewer says
so plainly.
