# Brief — the athlete's heart rates: declared once, then the app looks after them

*Written by the parent thread, 24 November 2026. **Design and prototype.**
A parallel thread is fixing the ledger's read; this one designs the thing the
ledger will read.*

## The athlete's instruction

> "in that case, we should ask the runner to explicitly add the HR setup..
> either RHR or max HR or both .. we should setup that as part of onboarding
> for the first time, thereafter the app should take care of it"

The context that makes it necessary: **Garmin's API does not expose the
zone model's history** — only the configuration as it stands now, read on the
day we read it (verified: `exploration/zone-model-history.md`). So every run
before that day has **no model in force**, and the app's own bands measure
nothing over the athlete's history — while the watch's own zone minutes cover
it all. The athlete's **own declared** heart rates are a dated fact that
covers the whole history, and they are the athlete's to give.

## What to design

### 1. The setup — one small, explicit fact

**The athlete's heart rates**: the **resting heart rate**, the **maximum
heart rate**, or **both**, each with:

* the **value**;
* **where it came from** — a closed list of sources, so nothing is free text
  the app has to interpret: *from my watch · from a test · from a doctor or a
  lab · from my training partner's watch · typed*;
* **when it was measured** (optional but asked for) and **when it was
  entered** (always);
* the ability to **change it later**, with the earlier values kept — the
  history is a history.

**Either, or both.** With only a maximum, the app can work in percentages of
the maximum; with both, it can work in **heart-rate reserve** as well. Say
what each combination makes possible and what it does not.

### 2. The bands — a rule, cited

The app needs five bands. It has two possible sources:

* the **watch's configured floors** (the dated zone model) — where one was in
  force;
* the **athlete's declared pair**, with a **documented rule** for turning it
  into bands.

**Source the rule.** Garmin publishes a default zone ladder (the usual
percentages of heart-rate reserve). Find it, cite it, and write down exactly
what the app would compute from a declared maximum and resting heart rate —
including the band's boundaries, and the fact that **the app's derived bands
are not the watch's configured bands** when the watch was set differently.
Everything the app prints must name which of the two it used.

### 3. Two dated facts, never blended

This is the part that must not go wrong:

* the **watch's model** (when it was read, with its own floors) and
* the **athlete's declared pair** (when it was entered) are **two facts**;
* the app **states** them, and states it when they **differ** — a fact, not
  a correction, and never a verdict on either;
* the app **never mixes** bands from one with bands from the other in the
  same answer, and never silently prefers one;
* **decide, and recommend, which basis the ledger's app-basis uses** — the
  athlete's declared pair applied to every run (one basis, consistent across
  the history, dated) or the watch's model where one was in force and the
  declared pair only for the rest (more faithful to what the watch measured
  against, but two bases inside one answer). Give the trade-off in a short
  paragraph and **recommend one**.

### 4. The onboarding

* where it lives: the **rig's own first-run page** (which already carries the
  athlete's three decisions — a race to type, a threshold to choose, a
  programme to adopt) gains a **fourth**: the heart rates. Or somewhere
  better — say where and why.
* the page's rule holds: the choices are **listed side by side**, unranked,
  the vitals are never linked to them, and **every one can stay empty**.
* the app's own place to change it afterwards (Settings → Training, beside
  the threshold).
* the **states**: no heart rates entered; only a maximum; only a resting rate;
  both; changed later.

### 5. "Thereafter the app should take care of it"

Spell out what the app does with the fact, with no work for the athlete:

* it **derives the bands** and uses them wherever a band is needed;
* it **re-derives** them when the pair changes, and the older answers stay
  as they were, dated;
* it **compares** with the watch's model when one exists, and states the
  difference;
* it **never asks again** — the athlete's entry is not re-prompted, and
  nothing nags.

## Deliverables

1. **`exploration/heart-rate-setup.md`** — the design: the fact and its
   provenance, the band rule with its citation, the two-facts rule, the
   onboarding, the states, the storage shape (a dated table beside the
   existing ones — what it holds, what the export carries), and the trade-off
   with a recommendation.
2. **`exploration/heart-rate-setup.html`** — the prototype: the onboarding
   card, the two inputs with their sources, the derived bands drawn against
   the watch's own floors (so the difference is visible), the states, phone
   and desktop widths. Plain HTML/CSS, no frameworks, no network,
   `file://`. **Numbers from `exploration/athlete.js`** — the zone model read
   3 Nov 2026, max HR 183 — and the **declared pair is new synthetic data
   you add to that file**, clearly marked.

## Rules

* **Facts, never verdicts**; the ban list and the closed direction set hold.
* **Explanation is not product content** (D22) and the surface speaks the
  runner's language (D24): *"your resting heart rate"*, *"your maximum heart
  rate"*, *"your zones"*, *"the bands from the heart rates you entered"*.
* **Absence is not zero**, and a missing heart rate is **no value with a
  reason**, never a default.
* **No new dependencies**; the design only — no code changes outside
  `exploration/`.
* No commit.

## Report back (last message, under ~400 words)

* the two files;
* the **band rule**, its citation, and the boundaries it produces for the
  canonical athlete's declared pair;
* the **two-facts rule** and your **recommendation** for which basis the
  ledger uses, in one paragraph;
* the onboarding's shape, and where the heart rates live afterwards;
* what the athlete types, and what the app then does without being asked.
