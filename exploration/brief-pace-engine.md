# Brief — the pace engine: race time → marathon pace → the training paces

*Written by the parent thread, 24 November 2026. **Prototype only** — no
code in `crates/`, `frontend/`, `android/` or `desktop/`. This is a design
note and a prototype, like the other threads' work.*

## Why this thread exists

The user has chosen **Pfitzinger's plan** as the product's first method. A
Pfitzinger plan is written in **marathon pace** — MP+2 for the easy days,
threshold pace, the long-run pace rule — and **Stride has no marathon pace
today**: no VDOT, no race times, no race predictions. That is the gap.

A parallel thread (`thr_nxc9e9nxms`) is pinning Pfitzinger's exact numbers
and their sources into `exploration/pfitzinger-spine.md`. **Do not wait for
it, and do not duplicate it** — design the *engine* and use clearly-marked
placeholders for the rule values, with a comment saying which row of
`exploration/pfitzinger-spine.md` will fill it in.

## What to produce

### 1. `exploration/pace-engine.md` — the design note

**The chain, and each link's source:**

| link | how it is computed | what it needs | what happens when the input is missing |
|---|---|---|---|
| **race time → VDOT** | the standard Daniels formulation | the athlete's own race: distance + time + date | no value, with a reason |
| **VDOT → marathon pace** | the marathon equivalent of that VDOT | the above | no value |
| **MP → the training paces** | the rules (placeholders) | MP | no value |
| **MP → a race prediction** | the Riegel form | MP (or any race) | no value |

**The paces to design for** (the values are placeholders until the research
lands; the *shape* is yours):

* **MP** — the anchor;
* **the easy pace** (MAFS / general aerobic) — the rule from the spine file;
* **the threshold pace**;
* **the long-run pace** — where the long run starts and where it finishes;
* and, if the spine file supports it, the **recovery pace**.

**Rules that must hold:**

* **A race time is an observation, not a verdict.** The athlete types it
  (distance, time, date, optionally the race name). It is stored with its
  provenance, like `wellness_observations`: the value, the day it was
  entered, the day it was last changed. Stride never invents a race time, and
  it never guesses one from the training.
* **Every derived number names its source.** "Marathon pace 3:48 /km — from
  your 3:12:00 marathon on 12 October 2026." If the race time is old, the
  card says how old; if it is changed, the answers change with it, and the
  card says which race time it used.
* **No score, no grade, no verdict.** A pace is a pace. The product does not
  say the athlete is "ready" for a distance, and does not rank races.
* **No age, no birthday.** The core's standing rule: a maximum heart rate is
  never an age formula. Maffetone's `180 − age` is the athlete's own typed
  number, never computed.
* **Absence is not zero.** No race time → no MP → no paces → **no value**,
  and the state says what is missing and where it would come from.

**The states to design:** no race entered; one race entered; a race entered
and later replaced; a race old enough that the athlete should be told its age
(as a fact, not a warning); paces that cannot be computed because the rule
needs something the athlete has not supplied.

### 2. `exploration/pace-engine.html` — the prototype

Picture-first (see `exploration/decisions.md` D11: the card is the picture,
the sentence is the caption):

* **A pace ladder** — the four or five paces drawn as a ladder, with the
  marathon pace as the anchor rung, the training paces labelled with the
  rule that produced them, and each rung's value inline. No red, no green,
  no "easy/hard" judgement colour; azure and amber only, and the athlete's
  own zones if a zone boundary happens to fall on a rung.
* **The source line** under it, naming the race the paces came from and the
  day it was entered.
* **The no-race state** — same card, empty, saying what is missing and what
  the athlete can type in.
* Phone and desktop widths, in the style of
  `exploration/information-shapes.html` and `exploration/consolidated-chart.html`
  (copy their palette and phone frame). Plain HTML/CSS, no frameworks, no
  network, works from `file://`.
* Synthetic data: one plausible athlete with a 3:12 marathon.

### 3. The connection to the ledger and the plan (a short section)

One paragraph each:

* **How the plan uses the paces.** The plan engine
  (`crates/stride-core/src/plan.rs`) holds dated sessions with a source
  (typed, prescribed, or an inferred baseline). A Pfitzinger plan is
  **prescribed** — and its sessions carry the pace the method prescribes for
  that week. Say how a session's target pace is expressed and stored, and
  how the plan's pacing changes when the underlying race time changes.
* **How the ledger uses the plan.** The ledger's rows answer against the
  athlete's own history (intensity bands, longest run, repeated work) and
  **now** against the plan's intent: the plan's session kinds are the
  vocabulary, and "the plan had a threshold session on 14 October; the last
  run in Z4–Z5 was on 15 October" is the negative space the wedge is about.
  Say what the plan anchor adds, and what it must never become (a judgement
  about whether the athlete did what they were told).

## Rules

* **Prototype only.** Only new files under `exploration/`. Do not touch
  `crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md`, `docs/` or the
  contract. Do not commit or push.
* Read `exploration/decisions.md` (D1–D12), `docs/contract/voice.md` (the
  ban list), `crates/stride-core/src/plan.rs`, `crates/stride-core/src/load.rs`
  and `crates/stride-core/src/units.rs` before you start, and reuse the
  existing rendering helpers rather than inventing new formatting.
* Every sentence in the prototype must survive the ban list; the note must
  say how you checked.
* Synthetic data only.

## Report back (last message, under ~400 words)

* the files you created,
* the chain, in four lines, with the formula you used for each link,
* the pace ladder you drew and how it reads,
* the "no race entered" state in one sentence,
* the placeholders you left for the spine thread's numbers, and where they
  plug in.
