# Brief — the plain vocabulary in the product's own strings

*Written by the parent thread, 24 November 2026. **Implementation** — the
prototypes are done; the product's strings are not.*

## Why this thread exists

`exploration/language.md` is the language pass: the rule (**precision belongs
in the notes; the surface speaks the runner's language**), the test, and a
**vocabulary** of the recurring hard words with their plain forms. It swept
every prototype. **The product's own strings have not been swept** — the
catalogue still says "zone model", "moving time", "the acute load", "the
ratio", "the ladder".

The user's words: *"We need to work on everything like that if we are showing
that in the surface."*

## The vocabulary to apply

| as it is now | plain form |
|---|---|
| zone model | **your zones** |
| moving time | **the time you were running** |
| elapsed time | **the whole run** (stops included) |
| the watch's own zone minutes | **the watch's own zones** |
| the model in force | **the zones your watch was set to that day** |
| VDOT | **your fitness number** |
| load | **your training load** |
| acute / chronic | **the last 7 days / the last 6 weeks** |
| form | **the difference** |
| ratio | **times the last 6 weeks** |
| the base | **the plan you follow** |
| a tier | **the level of the plan** |
| session kinds | **the kinds of run** |
| the window | **the last 12 weeks** (or the actual span) |
| the period before | **the same span, one week earlier** |
| no value | **not recorded / not measured / not worked out** |
| the ladder | **the training-paces card** |
| the ledger | **the zone times** |
| the method's rule | **the plan's rule** |
| the engine's own | **worked out from your race** |
| your zone floors | **where your zones start** |

**What stays technical, and why:** the terms the athlete's own watch shows —
**cadence**, **aerobic decoupling**, **efficiency factor**, **zone 1…zone 5** —
because the runner reads those exact words on their watch and would not
recognise our paraphrase. Same for `% maxVO₂` on the cost curve's axis: the
caption says it in full.

## What to change

**The surfaces the runner reads**, in the catalogue:

* the **zone source line** (now hoisted once per screen) — *"your zones ·
  running · max heart rate 183 bpm · taken from your watch on 3 Nov 2026"*;
* the **load card** — *"last 7 days 118 · last 6 weeks 84 · the difference
  +34 · 1.40 times the last 6 weeks"* — and its scenario;
* the **ledger card** — the bands, the coverage lines, the absence reasons;
* the **training-paces card** — the rungs, the source lines, the missing-value
  reasons;
* the **threshold** — the four candidates, their sources, the no-choice state;
* the **plan** — the week, the sessions, the kinds of run, the adoption;
* the **profile cards** — shape, months, schedule, habit, goals;
* the **states** — every "no value" line across the product, in the runner's
  words.

## Rules

* **The keys do not change** — only the rendered text. A key is an internal
  identifier; renaming it is churn with no reader benefit.
* **The core renders the sentences.** `crates/stride-core/src/strings.rs`
  owns the catalogue and `strings.json` is **generated** from it — change the
  core, regenerate, never hand-edit the JSON. The frontend's
  `frontend/src/i18n/catalogue.ts` is CI-checked against the same rule.
* **The contract is frozen**: seven string types, the closed direction set,
  the ban list (`docs/contract/voice.md`). A change that needs a new string
  type or a new direction word is a plan change and must travel with a
  `PLAN.md` note.
* **The ban list applies to every new string** — and note the language pass's
  finding: a note that says *"the app must not say 'recommended'"* still
  contains the banned word. State prohibitions without the word.
* **Numbers and formats do not change.** The app's conventions are in
  `crates/stride-core/src/units.rs` — "1 h 08 m", "84%", "40 days". Words
  only.
* **One wording per idea.** If two catalogue strings say the same thing in
  two ways, they must be brought together — the product should say "the last
  6 weeks" everywhere, never "chronic" in one card and "the last 6 weeks" in
  another.
* Gates, per the usual: fmt, clippy, workspace tests, check-deps, the
  contract greps, the privacy and secret scans, `tokens:check`, `lint:voice`,
  typecheck, unit, viewport. Run them and report the results.

## Report back (last message, under ~400 words)

* the files changed, and how many strings;
* the **before and after** of the ten strings you think matter most, in a
  table;
* any string you could not make plain without losing the fact it carries;
* the places where two strings said the same thing in different words, and
  how you brought them together;
* the gate results.
