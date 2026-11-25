# Brief — the ledger must cover the window, and must name the right reason

*Written by the parent thread, 24 November 2026. **Bug fix.** Found because the
athlete's twelve months of history finally arrived.*

## What happened

The rig pulled the athlete's **full twelve months** — **105 runs**, 2025-11-25
to 2026-11-25 — and the ledger route answered:

```json
"coverage": { "measured": 0, "runs": 64, "with_heart_rate": 63, "with_source_zones": 63 },
"kinds": [], "weeks": [], "overlap": null,
"unavailable": "No heart rate recorded — none of the runs in this period recorded a heart rate",
"source_weeks": 52, "more_runs": true
```

Three things are wrong.

### 1. The read is bounded by the cache, not by the window

`crates/stride-api/src/routes/stimulus.rs` takes the **newest 64 runs**
(`MAX_LEDGER_RUNS = 64`) and reports `more_runs`. The store holds 105 runs in
a 52-week window, so **the ledger answers for 64 of them** — and the cap is
the decode cache's capacity, an implementation detail, not a fact about the
athlete's training.

**The cache's capacity must never bound the answer.** The window is the
window: the ledger must cover **every run in the window it was asked about**,
and say so in its coverage. The cache may make the second read cheap; it may
not decide what the first read includes.

Decide and implement the right shape, and say what you chose:

* **the cache grows** to hold the window (a 52-week window of ~105 runs is
  small; but a decade would not be) — and the cache's bound stays explicit;
* **or the cache holds the newest N and the rest are decoded without being
  cached** — the answer is complete, the cache stays small, and the second
  read costs more;
* **or the cache is keyed and evicted on a budget** (by decoded bytes, not by
  run count), so a window's worth stays resident.

Whatever you choose: **the coverage line must describe the window, not the
cache**, and `more_runs` must mean "the store holds runs this answer does not
cover" — which, after the fix, should be false for a window the store covers.

### 2. The reason is the wrong one

The answer says *"No heart rate recorded — none of the runs in this period
recorded a heart rate"* — but **63 of the 64 runs do carry heart rate**.

The truth is different, and it is the truth the whole two-bases design rests
on: **there is one zone model, and it was read recently, so no run has a
model in force on its own day.** The app's own bands therefore measure nothing
over this history — while the **watch's own zone minutes cover all 52 weeks**
(`source_weeks: 52`), which is exactly what the two-basis design is for.

So the route must distinguish, and say each in its own words:

| the situation | the reason |
|---|---|
| the runs carry no heart rate | no heart rate was recorded |
| there is no zone model at all | no zone model has been read |
| a model exists but none is in force for these runs | **no zone model was in force for these runs** |
| a model was in force but the run carries no heart rate | the run carries no heart rate |

The third is the one the athlete will meet, and it is the case the product's
two bases exist for: **your watch's own zone minutes are the only zone time
before your watch's configuration was read.** The card's coverage must say
that in the runner's language — not in a failure's language.

### 3. The overlap is empty, and that is a fact

`overlap: null` because, in the 64 runs the answer covered, none had both a
model in force and the source's own zone minutes. Once the read covers the
whole window, the overlap will still be small — and it should be reported as
it is: the period where the app can check its own zone arithmetic against the
watch's. The design note (`exploration/two-bases.md`) says the overlap is
where the app checks itself; **an empty overlap is a fact to state, not a
hole to hide**.

## Rules

* **A bug fix, not a redesign.** The two bases, the ledger's kinds, the
  last-occurrence rule and the coverage line stay as they are.
* **The reason strings come from the core's catalogue**; add what is missing
  and regenerate `strings.json` — never hand-edit it.
* **The vocabulary** (D24): *"your zones"*, *"the time you were running"*,
  *"the whole run"*, *"not measured"*, *"not recorded"*.
* **Absence is not zero**, and **a reason is never a verdict**: the reasons
  name what is missing, and never suggest what to do about it.
* **No new dependencies.**
* **The gates**: fmt, clippy `-D warnings`, `cargo test --workspace`,
  check-deps, the contract greps, the secret and privacy scans and their
  self-tests, `tokens:check`, `lint:voice`, typecheck, unit, viewport.
* **Commit on `rig/v2`**, and **say in your report what the ledger answers for
  the athlete's real twelve months** — the numbers, after the fix.

## Report back (last message, under ~400 words)

* what you changed, and how the read is now bounded (or not);
* the reason strings, and which one the athlete's history actually gets;
* **the ledger's answer over the real 12 months**: the coverage line, the
  weeks the app's own bands cover, the weeks the watch's own minutes cover,
  and the overlap;
* the gates, and any new test that would have caught this.
