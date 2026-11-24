# Brief — the stimulus ledger (new work), prototype first

*Written by the parent thread, 24 November 2026. Follow it exactly; ask if
something is contradictory.*

## Why this thread exists

We chose the wedge for Stride (a local-first running log): **audit the
negative space of the training — what the recent weeks contain, what they do
not.** The reasoning is in `exploration/the-wedge.md`; read it first, then
`PLAN.md` (the plan of record) and `docs/contract/voice.md` (the frozen voice
and ban list).

The user has now locked three decisions:

1. **Intensity distribution goes first.** The ledger of training stimulus
   kinds — the intensity mix of the recent weeks, and which kinds are absent
   — is the first deliverable. The environment/condition axis is later.
2. **Weather comes from FIT bytes only.** No external weather source, no
   network lookup. If a watch recorded temperature in the FIT file, we use it;
   if not, there is no value (never zero, never a guess).
3. **History only.** The first version anchors on the athlete's own recorded
   history. The typed/prescribed plan is *not* an anchor in v1.

## What to produce

### 1. A design note (markdown, `exploration/stimulus-ledger.md`)

* **The taxonomy.** Propose four to six stimulus kinds that a runner would
  recognise, each derived **deterministically** from data Stride already has
  or already decodes: time in the athlete's own HR zones (from the dated zone
  model), pace bands, moving time, distance, lap structure, and the run's own
  zones. For each kind write the exact rule, the inputs it needs, and what
  happens when the inputs are missing (a "no value" with a reason, never a
  zero, never band 1). Prefer rules that are stable week to week over rules
  that are clever. Say plainly which rules are **heuristic** and which are
  **derived from a stated fact**.
* **The ledger.** For the Progress window (the same window the rest of the
  product uses — see `crates/stride-core/src/rolling.rs` and the Progress
  window rules), per kind: the time, the share of the moving time, the count
  of runs, **the last date that kind occurred**, and the change against the
  previous window (same length, ending a day earlier — the product's own
  comparison).
* **The wording.** Draft the sentences the ledger would show, in the core's
  own voice. Facts only: no verdict, no score, no readiness, no plan advice.
  Note that the ban list forbids *missed* — the ledger states the **last
  date**, not what was missed. Every sentence must be one a reader can check
  against the data.
* **The states.** What the panel shows with: an empty window, a window with
  one run, a window with no stored zone model, and a window where the ledger
  cannot be trusted (say why).

### 2. A prototype (one HTML file, `exploration/stimulus-ledger.html`)

* Phone width and desktop width, side by side, in the style of the existing
  prototypes in `exploration/` (copy the palette and the phone frame from
  `exploration/information-shapes.html`; use the Stride azure/amber identity,
  not Stitch's).
* Synthetic but plausible data: ~12 weeks of runs for one athlete, with a
  deliberate gap — no run above Z4 for five weeks — so the "absence" reading
  is visible in the prototype.
* Show the ledger, the sentences, and the empty / no-zone-model states as
  separate small frames.
* No JavaScript frameworks, no external assets, no network. Plain HTML/CSS
  (a few lines of JS is fine for toggling, but it must work from `file://`).

### 3. A FIT temperature feasibility note (same markdown file)

Scope it precisely, do not implement it:

* The FIT `record` message carries a temperature field (definition number 13,
  `sint8`, degrees Celsius). Confirm against `crates/stride-ingest/src/fit.rs`
  how the decoder reads record fields today and what it would take to read
  temperature — the FIT original is retained locally, so no re-download would
  be needed.
* State plainly that availability is **device-dependent**: watches with a
  thermometer record it, others do not, so a large share of runs may have no
  temperature value at all.
* Say how absence must be handled in Stride's terms (a missing value, with a
  reason; never zero; never substituted with a season's average) and what
  the condition axis could and could **not** claim with partial coverage.
* Also state what altitude already gives us (`fit.rs` rebases barometric
  altitude) and why altitude alone is not a heat model.

## Rules

* **Prototype only.** Do not modify anything under `crates/` or
  `frontend/src/`. Do not touch `PLAN.md`, the contract, or the schema. Write
  only new files under `exploration/`.
* Do not commit or push. Leave the files in the working tree and report the
  paths.
* Synthetic data only. No real athlete data, no Garmin calls, no network.
* Respect the voice: facts first, no judgements, absence is not zero, no
  score. If a sentence could be read as advice, it is wrong.
* Reuse what exists: `crates/stride-core/src/zones.rs` (zone model, time in
  zone), `pace.rs` (pace bands), `rolling.rs` (window and comparison),
  `fingerprint.rs` (per-run observations), `story.rs` and `statement.rs` (how
  sentences are written and typed). Quote the exact function names you rely
  on so the parent can wire the real thing later.

## Report back (last message, under ~400 words)

* the files you created (paths),
* the taxonomy you chose and the one rule you are least sure about,
* the exact wording of three ledger sentences,
* the FIT temperature findings (one paragraph),
* the open questions you need answered before real code is written.
