# Brief — the method lens (prototype only)

*Written by the parent thread, 24 November 2026. Follow it exactly.*

## Why this thread exists

`exploration/methods-and-products.md` (the research thread's output) settled an
important thing: runners already think in method vocabulary ("MP+2 for easy",
"threshold at 88–92% of max", "Bakken's 70%"), they use it as a **label**
rather than a rule, and they ask questions like *"Is there such a thing as
'too easy'?"*, *"Can you run your easy runs too slowly?"*, *"80% of my training
is in zones 4 and 5 — dangerous?"*.

It also found the two shapes a fact-only product can use:

* **Shape A — the rule and the history, side by side, as two separate facts.**
  The app states a method's **published rule** (quoted, attributed) and then
  states what the athlete's **own history** contains, in the athlete's own
  units, with **no link between them**. The reader draws the conclusion.
* **Shape B — the athlete chooses a lens; the app reports counts, dates and
  shares under that lens only.** No rule text, no evaluation, nothing that
  says whether the result is high or low.

And two rules that are not negotiable:

* a lens is **opt-in** and **never on by default** — a default lens is the app
  asserting a method;
* a lens must never produce a verdict ("9% short of the 80/20 target",
  "you have not had a threshold run for 5 weeks", "your easy runs are too
  fast"). The last-date form is the only permitted absence sentence.

## What to produce

### 1. A design note — `exploration/method-lens.md`

* **The lens catalogue.** A small set of lenses, each with: the method, the
  **exact published rule** (quoted, with a source link and the edition or
  publication it came from), the units it is stated in, and **whether the rule
  is computable from what Stride has**. Be strict about that last column —
  it is the interesting one. From the research: three of the six methods
  define their intensity by a **lactate number**, which no watch records;
  Bakken's own protocol requires cross-checking heart rate against **RPE and
  the talk test**; Pfitzinger's rules are stated in **marathon pace** and
  **general-aerobic pace**, which need a race time Stride does not have.
  A lens whose rule cannot be computed must say so and offer **nothing** —
  not an approximation.
* **What each lens would ask the app to compute**, using only what Stride
  already has: the stored zone model (max HR, thresholds), time in zone over
  the moving time, the window's totals, the athlete's own pace bands, the
  distance and time of each run. Name the core functions you would call
  (`zones::*`, `rolling::*`, `pace::*`).
* **The two shapes, worked out per lens**, with the **exact sentences** in
  Stride's voice. Every sentence must survive `docs/contract/voice.md`'s ban
  list, must carry its values inline (no template holes), and must not link a
  choice to a vital. Keep the five example sentences in the research's §3.3
  as your starting point and improve them.
* **The caveats the card must carry**, from the research, in the product's
  own voice:
  * **Seiler (2010)**: heart-rate time-in-zone **under-counts hard running
    and over-counts easy running** (HR lag). So an absence of time above Z4
    is partly a property of the watch. This must be stated on the card, not
    buried — it is what keeps the absence honest.
  * the rules that **cannot** be verified from a watch (lactate, RPE, feel,
    whether the session was run as intended) — say plainly that they are not
    checked, never imply they are.
  * the coverage facts (how many runs in the window carry heart-rate data at
    all).
* **The states**: no lens chosen (the default — the app states no rule and
  shows only the athlete's own facts); a lens chosen; a lens chosen whose
  rule cannot be computed; a window with no heart-rate data.

### 2. A prototype — `exploration/method-lens.html`

* Phone and desktop widths, Stride's azure/amber identity, in the style of
  `exploration/information-shapes.html` and `exploration/consolidated-chart.html`
  (copy their palette and phone frame).
* Plain HTML/CSS, no frameworks, no network, works from `file://`.
* Show: the **lens off** (the athlete's own facts only), a lens **on**
  (rule quoted and attributed above, the athlete's facts below, with a clear
  visual separation and **no arrow, no verdict, no colour judgement**), and
  the **not-computable** state.
* The lens choice is a **control the athlete makes** — a select or a list of
  radio-style options, off by default, and it must read as *the athlete's
  choice*, not the app's recommendation.
* Synthetic but plausible data, consistent in style with
  `exploration/stimulus-ledger.html` (same fictional athlete if you like).

## Rules

* **Prototype only.** Do not modify anything under `crates/`, `frontend/`,
  `android/`, `PLAN.md` or the contract. Only new files under `exploration/`.
* Do not commit or push.
* Every rule you quote must carry its source. **Never invent a rule, never
  paraphrase one into a number the source does not state**, and never state a
  threshold as the method's if the source says something vaguer. If you cannot
  source a rule, leave the lens out and say so in the note.
* No banned words. No scores, no grades, no traffic lights, no red, no advice,
  no "recommended", no "should", no "missed".
* Read `exploration/methods-and-products.md` first — especially §1.6, §2.7,
  §3.1, §3.3 and §3.4 — and `exploration/stimulus-ledger.md` so the lens sits
  beside the ledger rather than duplicating it.

## Report back (last message, under ~400 words)

* the files you created,
* the lens catalogue you chose, and **which lenses you refused** and why,
* three finished sentences (Shape A) and one (Shape B),
* how you handled a rule that cannot be computed from a watch,
* the open questions you need answered before any of this is written in code.
