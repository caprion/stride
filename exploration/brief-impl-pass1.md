# Brief — cleanup Pass 1: the correctness changes (W, Z, C1, C2)

*Written by the parent thread, 24 November 2026. This is **implementation** —
the first code change of the cleanup. Follow `exploration/cleanup-plan.md`.*

## What is authorised

The user has approved, by name:

* **Pass 2 of the cleanup (the surface cut)** — the consolidated run-detail
  chart is approved explicitly ("thumbsup"); the wall cut, the `/pace`
  retirement and the radar retirement follow it as one change.
* **W — the window migration** (decision 2 of `exploration/pending-decisions.md`).
* **Z — the zone model, stage 1** (decision 3), **with one change**: the
  model and its read date are **hoisted to one source line per screen**, not
  repeated per answer. An answer that used a different model still says so.
* **The plan tier: option A** — the plan card states the tier's entry
  requirement beside the athlete's own weekly mileage, and the athlete picks.

**This brief is Pass 1 only** (W, Z, C1, C2). Pass 2 (the cut) and Pass 3
(the new activity observations) come after it, and they will be briefed
separately.

## The four changes

Follow `exploration/cleanup-plan.md` for the detail — the paths, the risks and
the gates are all named there. In short:

* **W — the window.** One window definition for the whole Progress wall: the
  last *N* calendar weeks, from the start of the earliest one **through
  today**; the compared period is **the same span ending seven days
  earlier**. The Progress totals and the pace page already compute the window
  this way; the **rolling route** is the one to migrate. The core's window
  type gains two ends. **The profile's volume-a-week and consistency change**
  (the denominator becomes the window's length — 79 days for 12 weeks on a
  Wednesday, not 84), and the year's comparison moves to a week.
* **Z — the zone model, stage 1.** Every run is measured with **the zone
  model in force on the day of the run**, for the **zone-time** answers: the
  run detail's zone table, the window's zone split, the ledger's rows, the
  fingerprint's efficiency factor. The store already keeps the dated models
  in `hr_zone_models`. The **load model does not move** — that is stage 2, a
  separate change with its own announcement.
  * **The source line is hoisted**: one per screen, naming the model and its
    read date, with the details on a click through. An answer that used a
    different model (a run predating the first stored model) still says so.
  * Every zone number changes retroactively. **Say so in the change's own
    words** — in the commit message and in whatever the app shows where a
    number changed meaning — without making it a judgement.
* **C1 — one walking rule.** `crates/stride-ingest/src/segment.rs` decides
  walking at sample level; `crates/stride-core/src/fingerprint.rs` decides it
  at lap level, with different thresholds. They become **one rule with two
  views of it**, living in the core (the ingest classifies; the core reads).
* **C2 — one zone-model reader.** `rolling.rs:266`, `fingerprint.rs:185` and
  `load.rs:160` each read the model; they become one seam.

## Rules

* **Implement; do not re-plan.** The plan names every path, the risk and the
  gate. If the plan and the code disagree, **stop and report** rather than
  improvising — and say which one you believe is right.
* **Small commits, one change each**, in the plan's order: W, Z, C1, C2. Each
  commit passes the gates on its own. Do not bundle them.
* **The gates are in `exploration/cleanup-plan.md`** — read them before you
  start and run them per commit: `cargo fmt --check`, `clippy -D warnings`
  (no unwrap, no expect, no panic, no todo, no unimplemented),
  `cargo test --workspace`, `scripts/check-deps.sh` (the crate DAG),
  `scripts/check-contract-greps.sh`, `scripts/privacy-rules.sh`, the secret
  and privacy scans and their self-tests, `scripts/check-corpus.sh` (the FIT
  fixtures' SHA-256), and the frontend checks (`tokens:check`, `lint:voice`,
  `typecheck`, `test:unit`). **Run the ones that apply to each change and
  report their results** — a gate you did not run is a gate you did not pass.
* **The contract is frozen.** If a change needs a contract change, it is a
  **plan change**: it must travel with a `PLAN.md` note in the same change,
  and `docs/contract/README.md` says what that means. Read it first.
* **The voice is frozen.** No new string type, no new direction word, nothing
  off the ban list (`docs/contract/voice.md`). Every user-facing string comes
  from the core's catalogue (`crates/stride-core/src/strings.json`), and the
  core renders it — the UI does not write sentences.
* **No production data, no Garmin writes, no publish, no network.** Synthetic
  fixtures only; the FIT corpus is checked by SHA-256, so do not regenerate
  it.
* **New dependencies need approval** (`scripts/approved-deps.txt`). Assume
  the answer is no.
* **Do not commit or push.** Leave the changes in the working tree; the
  parent reviews and commits. Say in your report which files changed, so the
  commits can be made in the right order.

## Report back (last message, under ~450 words)

* **W** — what you changed, the two windows' dates for a 12-week window as of
  today, and what the profile's volume-a-week and consistency now read;
* **Z** — the seam, the per-run model lookup, the hoisted source line, and
  the one place a run's different model still says so;
* **C1 / C2** — the single rule and where it lives;
* the commits you would make, in order, and the gates each passed;
* anything where the plan and the code disagreed, and what you did.
