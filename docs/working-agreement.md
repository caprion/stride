# Working agreement — running big changes with parallel agents

Distilled from the Stride build (Phase 1, Phase 2, and the three design
sub-threads). This is the operating rule, not a philosophy.

---

## 1. What parallelising actually buys you

**A clean context for a narrow brief.** That is the resource being parallelised —
not CPU, and not wall-clock coding time.

The evidence: in Phase 2 the four agents wrote ~1,400 lines of correct-shaped
code that I would have taken about the same time to write myself, and I spent
the saved time on integration and verification instead. The three design
sub-threads (Android, Garmin-on-device, harness) landed 83 KB of dated,
cited decision documents with **zero** rework — the best return of the whole
session.

The corollary matters: **a sub-thread's value is destroyed if the brief requires
the orchestrator's context.** If I cannot write the brief without pasting half
my own context in, I should just do the work.

---

## 2. When to fan out, and when not to

| | Fan out | Do it inline |
|---|---|---|
| **Design / architecture** | no | one coherent thinking pass — parallel designers produce parallel schemas |
| **Schema, contract, types** | no | freeze these *before* fan-out, always |
| **Independent file ownership** | **yes** — the main win | |
| **Research with a written deliverable** | **yes** | |
| **Tests against a frozen contract** | **yes** | |
| **Anything touching shared files** | no | serialise, or assign one owner |

Rule of thumb: **N agents = N disjoint file sets.** If two agents would edit the
same file, they are one agent.

---

## 3. The three rules that make it work

### Rule 1 — Freeze the contract first

Every parallel failure in this session was contract drift. Agents invented their
own response shapes, and the integration cost landed entirely on the
orchestrator: the plan API had to be rewritten wholesale because its shape did
not match what the frontend was written against.

Before fan-out, write down and share: the schema, the TypeScript types, the
endpoint list, the response shapes, and the file each agent owns. The contract
is the interface; treat it like one.

### Rule 2 — Never trust the completion message

Twice in this session an agent reported success with real problems behind it:
one delivered 85 lines containing a literal `p.get('…')` placeholder; another
reported a finished test file that did not exist. A third produced no reply at
all.

Always: read the diff, run the type-check, run the tests, look at the render.
Treat every agent's report as a claim, not a fact.

### Rule 3 — Assign models to the work, not to the thread

| Model | Good at | Unreliable at |
|---|---|---|
| `codex` / `gpt-6-luna` | mechanical, well-specified code against a frozen contract | writing dense code — it compresses to single lines and skips edge cases |
| `hermes` / `gpt-6.1-sol` | research, design docs, reasoned code | — |
| `claude-code` / `claude-fable-5-1` | *(untested here)* | — |
| `claude-sonnet-5.5` via hermes | — | **produced no reply on 2 of 3 attempts** — do not use |

And for the code a `codex` agent writes: **run Prettier over it afterwards.**
The compression is the single most common quality problem, and the formatter
fixes most of it for free.

---

## 4. Concurrency: set it before you start

The host limit here is **automatic = 2** (two vCPUs). That silently serialised
four agents into two waves and cost a cycle to discover. Raise it up front:

```bash
bb concurrency-limit host <host-id> 5
```

Beyond ~4 concurrent agents, the orchestrator becomes the bottleneck anyway —
it is the only one integrating. Five is the practical ceiling on a two-core
box.

---

## 5. The shape that worked

```
1. DESIGN        one pass, by the orchestrator
                 └─ the schema, the contract, the briefs

2. FREEZE        write the contract down before anyone starts

3. FAN OUT       N agents, disjoint files, briefs that say:
                 - what to read first
                 - which files you own
                 - which files you must NOT touch
                 - how to verify your own work
                 - what to report back

4. INTEGRATE     orchestrator: typecheck, tests, render, review the diff
                 fix the drift, re-run, re-review

5. COMMIT        one commit per coherent phase
```

Step 5 is not optional. Every phase in this repo is one commit with the
reasoning in the message, because the reasoning is what makes the next phase
possible.

---

## 6. What I would *not* parallelise, ever

- **The product design.** One mind, one coherent thesis. The value of
  `DESIGN.md` is that it is one argument, not four.
- **The data model.** A schema is a contract for everything downstream.
- **Anything the user will judge as one thing.** The collage, the dashboard,
  the "Today" card — these are single-piece judgements.
- **The final integration.** Always the orchestrator's.
