# Brief — setting up the athlete's threshold (design + prototype)

*Written by the parent thread, 24 November 2026. **Prototype only** — no
code under `crates/`, `frontend/`, `android/` or `desktop/`.*

## The user's idea, and the shape I want

*"We could help runners set up their current threshold pace/HR somewhere and
calculate all of this for them."*

The app needs a threshold: it is the anchor for the ladder's lactate-
threshold rung, the boundary the 80/20 base bands its easy days against, and
the number a Pfitzinger plan's LT session is paced from.

**The product's line, which this design must hold:** the app computes
**candidates**; the athlete **chooses** one; the choice is stored as a dated
fact with its source. The app never says which candidate is right, never
warns that one is stale, and never computes a threshold from an age.

So: **not** "we calculate your threshold". **"Here are the numbers your data
gives, and here is the one you use."**

## What to produce

### 1. `exploration/threshold-setup.md` — the design note

#### The candidates — one section each

For each: **what it is**, **what it needs**, **how it is computed** (the
formula, named), **what it cannot know** (its limits), **how it is stored**,
and **how it is named in a sentence**.

1. **The watch's own zone model.** Stride already stores it as a **dated
   fact** (`hr_zone_models` — the watch's configuration, read once, dated).
   The threshold HR the athlete has today is, in effect, one of the stored
   zone floors (zone 4's floor or zone 5's floor, depending on the watch's
   configuration) — so a threshold HR may already exist and may need **no
   typing at all**. Say exactly which floor is which in the stored model,
   and how the athlete can accept the watch's own number in one action.
   Its limit: it is whatever the athlete (or a formula, or a lab) put into
   the watch; the app cannot check it.
2. **From a race.** The pace engine's chain (`exploration/pace-engine.md`)
   already gives race-pace equivalents from a typed race time; the spine
   (`exploration/pfitzinger-spine.md`) gives the book's rule for **LT pace**
   — "the pace you could race for about an hour", ≈ a 15K–10-mile race pace,
   or 10K pace + 10–15 s/mi for a shorter-distance athlete. That is a
   **pace** candidate, derived from a race time the athlete typed, with the
   rule cited.
3. **From a field test.** The candidate that needs the app's help. Design it:
   * which test(s) the athlete runs — the **30-minute time trial** (the
     standard field test: run hard for 30 minutes; the threshold heart rate is
     the average heart rate over the **final 20 minutes**), and, if you can
     source it, a **threshold-pace test** in the book's or RCRS's style;
   * the app's part: the athlete syncs the run, and the app computes the
     estimate **from the stored FIT** (the samples it already has) — the
     average heart rate over the last 20 minutes, the pace over the same
     window, and the heart rate's drift across the window;
   * **the rule is named in the sentence** — an estimate, from a named rule,
     with the formula and its limits stated ("this assumes a steady effort
     across the window");
   * the limits are real and must be written down: the estimate inherits the
     run's conditions, the athlete's pacing, and the watch's heart-rate
     accuracy; a test run is a run, and it is in the ledger like any other.
4. **Typed by the athlete.** From a lab, a coach, another app — a number the
   athlete brings. Stored with its provenance: the value, the day entered,
   the day last changed, and **where it came from** (a label the athlete
   picks, not free text the app interprets).

#### The flow

* The athlete **sees the candidates side by side**, each with its source and
  its date — never ranked, never with a "recommended" chip. The athlete
  picks one; the app stores it as **a dated fact with its source**, the way
  the zone model and the race time are stored.
* **Later changes are facts, not alarms.** If the athlete changes it, the app
  can show both: "your threshold on 12 November was 168 bpm; on 3 March it
  was 172 bpm". No "your threshold has changed", no "is it still valid".
* **A disagreement is shown, not resolved.** If the watch's zone floor says
  170 and the field test says 166, the app shows both, with their sources
  and dates, and does not reconcile them. If the athlete picks one, the other
  is still visible.
* **The setup may be skipped.** With no threshold chosen, every answer that
  depends on one is a **no value with a reason**, and the states say what is
  missing and where it would come from.

#### Where the threshold is used — and where it must not leak

This is the subtle part; write it carefully:

* the **ladder's LT rung** — the anchor the spine's rules are applied to;
* the **80/20 base's easy-day band** — the boundary the gray zone is measured
  against;
* the **plan's LT session pacing** — the pace a prescribed LT session runs at;
* the **ledger's absence rows** — here is the trap: the ledger's rows answer
  against the athlete's **own stored zones**, not against this threshold. So
  the ledger must **not** silently switch to the chosen threshold. Say
  plainly: the watch's zone model and the athlete's threshold are **two
  different numbers**, each named where it is used, and the app does not
  merge them.

#### What it must never do

* compute a threshold (or a maximum heart rate) from an **age** — the core's
  standing rule, and there is no birthdate in Stride;
* assert which candidate is right, or which to pick;
* warn that a candidate is stale, or that a threshold needs retesting;
* show a verdict, a score, a readiness, a colour that judges, or a
  recommendation;
* use a banned word (`docs/contract/voice.md`'s ban list) — no *should*,
  *must*, *good*, *bad*, *missed*, *recommended*, *ready*.

### 2. `exploration/threshold-setup.html` — the prototype

Picture-first (`exploration/decisions.md` D11):

* **The candidates list** — one card per candidate: the value, its source,
  its date, and a one-line statement of what it cannot know. Azure for the
  athlete's own data, amber for the method's rule, no judgement colour.
* **The choice** — the athlete picks; the picked one is marked, the others
  stay visible.
* **The history** — the threshold's dates and values over time.
* **The disagreement state** — two candidates that disagree, both shown.
* **The empty state** — no threshold chosen; what is missing, and the four
  ways it could arrive.
* Phone and desktop widths, in the style of
  `exploration/information-shapes.html` and `exploration/consolidated-chart.html`
  (copy their palette and phone frame). Plain HTML/CSS/vanilla JS, no
  frameworks, no network, works from `file://`. Synthetic data.

### 3. The connection to the plan and the ledger (one short section)

* Where a **threshold session's pace** comes from in the plan engine
  (`crates/stride-core/src/plan.rs`), and what it is when the threshold is
  absent.
* What the **80/20 base** needs from the threshold, and what it does without
  one.

## Rules

* Same two files only. Nothing under `crates/`, `frontend/`, `android/`,
  `desktop/`, `PLAN.md` or `docs/`. No commit.
* Read `exploration/decisions.md` (D1–D16), `docs/contract/voice.md`,
  `exploration/pace-engine.md` and `exploration/pfitzinger-spine.md` first.
* **Source the field-test rules.** The 30-minute time trial and the
  "average heart rate over the final 20 minutes" rule are published; cite
  them. If you cannot source a rule, mark it unverified and do not present it
  as the method's.
* Every sentence survives the ban list.

## Report back (last message, under ~400 words)

* the two files created,
* the four candidates and the exact wording of the sentence that names each,
* how the "choose" flow works in three steps, and what the app does when the
  athlete does not choose,
* how the watch's zone model and the athlete's threshold are kept apart,
* the open questions, especially anything about the field test's rules that
  you could not source.
