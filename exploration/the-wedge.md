# The wedge — what Stride should be that no one else is

*24 November 2026. Written before any work, so the choice can be checked
against what was chosen.*

## The question

The learning goal behind this: understand training properly — what the recent
months actually did to the athlete, whether the training is running too hot or
too cold, whether a kind of workout is absent. The reference material in the
thread (RUNALYZE, OpenAthlete, TrainingLoad, Trainingload.ai) is a list of
models an app *could* compute. Before implementing any of them: **what is the
one thing Stride can own that those four cannot?**

## 1. What Stride already has (do not rebuild this)

| paste layer | Stride status |
|---|---|
| TRIMP, CTL, ATL, form, ratio, monotony, strain | **exists** — `crates/stride-core/src/load.rs` |
| Time in zone, from a dated zone model | **exists** — `crates/stride-core/src/zones.rs` |
| Pace bands (distribution of moving time by pace) | **exists** — `crates/stride-core/src/pace.rs` |
| Polarised split (low / moderate / high shares) | **exists** — `crates/stride-core/src/rolling.rs` (`polarised_shares`) |
| Aerobic decoupling, cadence drift, HR drift, pace drift, CV, efficiency factor, stride length | **exists, per run** — `crates/stride-core/src/fingerprint.rs` |
| Fitness / fatigue chart, weekly impulse | **exists** — `load.rs`, `/insights/load` |
| Intensity balance | **exists as shares only** — no ledger, no gaps |
| Workout-type classification (easy / long / tempo / interval) | **absent** |
| Riegel, VDOT, effective VO2max, critical speed, marathon shape | **absent** |
| Condition adjustment (heat / humidity / altitude) | **absent** (altitude is decoded; temperature is not) |
| "Readiness" score | **deliberately absent** — banned by `docs/contract/voice.md` |

So the paste is, for this codebase, mostly a description of a competitor.
Stride has already done Layer A and most of Layer C *per run*. The missing
parts are not the mathematics; they are the **framing**.

## 2. The competitive picture (checked 2026-11-24)

| product | what it does that Stride does not |
|---|---|
| RUNALYZE | ML activity-type classification (beta), marathon-shape race predictions, VO2max estimation |
| TrainingPeaks / PEAK | interval detection, intensity-zone thresholds, weekly load distribution charts |
| Garmin Connect | training status, training load, training readiness, intensity minutes, performance conditions |
| Strava | population-ML fitness and race predictions |
| heat-calculator sites (Running Writings, McMillan) | heat-adjusted pace models, but one-off calculators with manually entered weather |

Nobody among them answers the **negative space**: what the last six weeks
*did not contain*. And nobody shows conditions inside the training history,
so "my easy pace got slower" and "it got 8°C hotter" are indistinguishable in
every one of them.

## 3. Candidate wedges, scored

| candidate | answers the runner's question? | data available today? | moat | effort | fits the voice contract? |
|---|---|---|---|---|---|
| **A. Stimulus ledger** — what the period contains, and what it lacks | yes: "missing a type of workout", "too hot / too cold" in the intensity sense | yes: zones, pace bands, laps, plan intent | medium | medium | yes — a gap is a date and a count, not a verdict |
| **B. Condition-adjusted effort** — heat, humidity, altitude tax on every run | yes: "running too hot / too cold", literally | partly: altitude yes, temperature/humidity no | high | high, plus a privacy decision | yes — "this pace cost 5 bpm more than in October" is a fact |
| **C. Self-validating predictions** — race predictions checked against the athlete's own results | partly (fitness trend, race goals) | needs manual race entries | medium | medium | needs care to stay a fact |
| **D. Session execution analysis** — interval detection, rep drift | partly | yes | low (intervals.icu, RUNALYZE, TrainingPeaks all do it) | medium | yes |

D is a feature, not a wedge. C is a good later layer. A and B are the
compass.

## 4. The wedge

> **Stride audits the negative space of your training: what your recent weeks
> contain, what they do not, and how much of what you feel belongs to the
> conditions. Every number is a dated fact from your own watch, shown with its
> assumptions — never a score, never a prescription.**

Three pillars, in build order:

**Pillar 1 — the stimulus ledger.** For a period, an inventory of the
stimuli acquired, each with its evidence and its last date: easy volume
(Z1–Z2), quality (Z4–Z5), threshold work, the long run, the pace range, the
recovery days. The answer to "is a kind of workout missing" is not an opinion
— it is *"last run above Z4: 41 days ago"*, *"longest run in the last 8
weeks: 14 km"*, *"92% of the time was Z1–Z2"*. The plan engine
(`crates/stride-core/src/plan.rs`) already knows the athlete's **intent**
(typed, prescribed or inferred baseline) — intent is the anchor the ledger
measures against, and a baseline stays labelled a baseline.

**Pillar 2 — the condition axis.** Every run's effort read against the day it
happened: heat, humidity, altitude. This is what makes Pillar 1 honest across
seasons — otherwise a hot summer reads as "no easy running", and a
cool October reads as "fitter". Without it, the intensity-share comparison is
confounded, and the app's own comparisons between windows are subtly unfair.

**Pillar 3 — the self-check.** Later. Race predictions (Riegel / VDOT) scored
against the athlete's own results, so the app states its own error instead of
claiming fitness.

### Why this fits Stride and not a generic dashboard

* the **dated zone model** is already a stored fact with provenance — stimulus
  boundaries do not move per run;
* the **FIT originals are retained**, so pillars 1–2 are recomputable forever
  from local bytes, with no re-download and no account;
* the **no-verdict voice** (`docs/contract/voice.md`) is already contract-frozen
  and machine-linted — a gap is a fact, and the ban list makes the wording
  safe ("*missed*" is a banned word: the ledger says the **last date**, not
  what was missed);
* the **absence-is-not-zero** discipline already exists everywhere in the
  core, which is exactly the discipline a ledger needs (a day the record does
  not cover is not a rest day).

### The vocabulary trap to design around

| what a coach says | what Stride may say |
|---|---|
| "you missed threshold work" | "last run above Z4: 41 days ago" |
| "you are overreaching" | "acute load 118, chronic 84, ratio 1.40 — same as the week of 6 May" |
| "run easy tomorrow" | *(nothing — options are unranked and never conditioned)* |
| "you are in good shape" | "form +12; last 10 km at 152 bpm was 11 s/km faster than the same pace in March" |

## 5. Smallest first slice

One new core module (`crates/stride-core/src/stimulus.rs`), one route
(`/insights/stimulus`), one panel on Progress:

1. Classify each stored run into a **stimulus kind** from what is already
   stored — zone distribution of moving time, pace bands, duration, distance,
   and the laps. Deterministic, with the reason attached when the
   classification is not trustworthy.
2. Over the Progress window, produce the **ledger**: per kind — the time, the
   share, the count, the last date it occurred, and the change against the
   previous window.
3. Render it as facts only, in the core's own strings, under the existing
   lint.

No new ingestion, no weather, no model, no LLM. If this does not read as
"useful and honest" on the athlete's own history, no larger wedge will.

## 6. Explicitly out of scope

* a readiness / recovery score (banned, and not validatable for one athlete);
* ML of any kind (no population dataset; the athlete's own history is the
  dataset);
* effective VO2max as a headline number (a number with a model version, not
  a fact about the athlete);
* **the environment axis — dropped, not "later"** (see the clarification in
  §7). No FIT temperature decoding, no weather source, no humidity, no heat
  adjustment. The watch's temperature coverage was measured and is nil
  (`exploration/stimulus-ledger.md` §5: no record message in the fixture
  corpus carries FIT field 13), so the axis had no data to stand on.

## 7. Decisions this memo needs

**Decided by the user, 24 November 2026:**

1. **Intensity distribution goes first** — pillar 1 is the first deliverable.
2. **"Running too hot or too cold" means load, not weather.** The user's
   own words, clarified on 24 November 2026: *"when I say running too hot or
   cold means are you over exerting basically load or not."* So the axis is
   **physiological exertion and training load**, not temperature. The
   environment pillar is therefore **dropped**, not deferred:

   | the runner's question | the product's answer | where it lives |
   |---|---|---|
   | **too hot** — am I over-exerting? | acute load, chronic load, form, the acute-to-chronic ratio, monotony, strain — with the model's scenario | **already exists** — `crates/stride-core/src/load.rs`, the *The load* card |
   | **too cold** — am I under-doing it? | what the weeks contain and what they lack: the last date per band, the longest run, the repeated work | **the new work** — the stimulus ledger |
   | **missing a type of workout** | the same absence rows, read as kinds | **the new work** — the stimulus ledger |

   So the "too hot" side was never missing: the product has answered it with
   a fact and a scenario since v1. The gap the wedge closes is the **cold**
   side — the negative space — and the honest, caveated reading of it
   (heart-rate time in zone under-counts hard running and over-counts easy
   running — Seiler 2010 — so an absence row is corroborated by the run's
   own lap structure, or the card says the lap data is missing).
3. **Weather comes from the FIT bytes only** — *moot*: dropped with the
   environment axis (see decision 2). The FIT `record` message carries
   temperature as definition number 13 (`sint8`, °C), and the decoder in
   `crates/stride-ingest/src/fit.rs` reads records by definition number
   already, so reading it would be a small local change — but the fixture
   corpus shows the athlete's watch records **no** temperature at all, so the
   axis had no data to stand on. The note in `exploration/stimulus-ledger.md`
   §5 stays as the record of why.
4. **History only** — *superseded by D13.* The ledger's rows still answer
   against the athlete's own recorded history; the **plan card** now carries
   the athlete's intent (the adopted base, its sessions, its pacing). The two
   sit side by side and are never merged into one number, and the app never
   says whether the athlete did what they were told. (The research finding
   still stands: runners train harder than planned on easy days and easier
   than planned on hard days — Foster et al. — so a plan anchor answers a
   question a history-only ledger cannot.)

Still open: the exact stimulus taxonomy, the wording of the ledger sentences,
how far the Progress wall is cut, and the method lens's form.

## 8. The work, as threads

* `exploration/brief-stimulus-ledger.md` → thread **thr_efgucevebk** — the
  taxonomy, the ledger, the sentences, the prototype, and the FIT
  temperature feasibility note (now a record of why the environment axis was
  dropped, not a plan for it).
* `exploration/brief-surface-consolidation.md` → thread **thr_q6tcjhp6wm** —
  the surface inventory, the cut list, the one consolidated chart (pace,
  altitude, heart rate, toggled by the user), and the cut Progress wall.
* `exploration/brief-research-methods.md` → thread **thr_5x4cgbmpq5** — what
  runners actually say, and how the coaching methods map to computable data
  (done: `exploration/methods-and-products.md`).
* `exploration/brief-method-lens.md` → thread **thr_rn8egwrawh** — the
  opt-in, fact-only method lens.
* `exploration/brief-cleanup-audit.md` → thread **thr_8xwzkx39xu** — the
  ordered cleanup plan for the code, before any feature code.

Both threads are **prototype only**: new files under `exploration/`, no
changes under `crates/` or `frontend/src/`. The code cleanup happens after
the prototypes are agreed, before any real implementation.
