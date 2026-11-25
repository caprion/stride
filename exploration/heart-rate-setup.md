# The athlete's heart rates — one small fact, declared once

*24 November 2026. **Design and prototype only.** No code under `crates/`, `frontend/`, `android/` or
`desktop/` was touched; no schema change was made; no commit was made. The two files are this note and
`exploration/heart-rate-setup.html`. The only change outside them is the addition of the athlete's
declared pair — new, clearly-marked synthetic data — to `exploration/athlete.js`.*

*The surface copy here and in the prototype follows the language pass — `exploration/language.md` — and
D24: the surface speaks the runner's language; the notes carry the rest.*

**Read with:** `exploration/zone-model-history.md` (why the zone model has no history — the fact this
design exists to answer), `exploration/two-bases.md` (the two blocks on the ledger's card), `exploration/
threshold-setup.md` (the dated-fact pattern this follows), `exploration/decisions.md` (**D3** the dated
model, **D11** the picture first, **D17** the two kinds of band, **D19** the threshold's four sources,
**D20** two dates on a typed value, **D21** the canonical athlete, **D22** explanation is not product
content, **D23** no zone-model history, **D24** the runner's language), `docs/contract/voice.md` (the ban
list), `docs/contract/schema.sql` (the frozen schema), and `exploration/athlete.js` (the canonical
athlete — the one source of every number).

---

## 0. The line this design holds

The athlete's sentence:

> "in that case, we should ask the runner to explicitly add the HR setup.. either RHR or max HR or
> both .. we should setup that as part of onboarding for the first time, thereafter the app should take
> care of it"

The design keeps the whole of it. The **setup** is one explicit fact — the athlete's own heart rates,
entered once, with where each number came from and when it was measured and entered. The **"thereafter
the app should take care of it"** is the second half of the sentence and it is the harder half: after
the entry, the app derives the bands, uses them, re-derives them when the pair changes, states the
difference from the watch's own zones, and **never asks again**.

Three negatives hold everywhere below:

1. **The app never computes a heart rate from an age.** There is no birthdate in Stride, and none is
   added. Garmin's own published estimate (220 − age) is **not** used — it is named here only to be
   refused (D19). The watch's own maximum is Garmin's *estimate or auto-detected value*, which is a
   fact about the watch's configuration, not a fact about the athlete's physiology.
2. **The app never blends two bases.** The watch's zones and the athlete's declared heart rates are two
   dated facts, and no answer ever mixes them (§3).
3. **The app never asks again.** The entry is not re-prompted, nothing nags, and no screen says the
   numbers are old (§5).

### 0.1 Why this is needed at all

Garmin's Personal API exposes the heart-rate zone configuration **as it stands now** — one record, no
date, no effective-from (`exploration/zone-model-history.md` §2). So every run before the day the
model was read has **no model in force**, and the app's own bands measure nothing over the athlete's
history, while the watch's own zone minutes cover it all. The athlete's **declared** heart rates are a
dated fact that covers the whole history, and they are the athlete's to give. That is the hole this
design fills.

### 0.2 The three things that must not be confused

The design keeps three different things apart, and the wording keeps them apart too. They are not the
same thing and are never drawn as one:

| | what it is | dated by | who it belongs to |
|---|---|---|---|
| **the watch's zones** | the **configured floors** of the zone model in the watch's settings — `zone1_floor…zone5_floor` | the day it was **read** (3 Nov 2026) | the watch |
| **the watch's own zone minutes** | the **per-run record** the watch made — `hrTimeInZone_1..5` | the run | the watch |
| **the bands from the heart rates you entered** | the app's own bands, **derived** from the athlete's declared pair | the day they were **entered** (24 Nov 2026) | the athlete |

The first and the third are the **two facts** of §3. The second is the source's own record of each run
(the second block on the ledger's card — `exploration/two-bases.md`) and is not a basis the app
computes anything from.

---

## 1. The setup — one small, explicit fact

### 1.1 What the athlete declares

Two values. **Either, or both.**

| value | unit | required | optional companions |
|---|---|---|---|
| **your maximum heart rate** | bpm | no | where it came from · when it was measured |
| **your resting heart rate** | bpm | no | where it came from · when it was measured |

Each value carries four things, and nothing else:

| | |
|---|---|
| **the value** | a whole number of beats per minute |
| **where it came from** | a label from the **closed list** of five (§1.2) |
| **when it was measured** | optional — the day the measurement was taken, when the athlete knows it |
| **when it was entered** | always — the day the athlete put it in |

The two dates are the pair the threshold already carries (D20.2): a typed value carries **the day of the
measurement** (optional) and **the day of the entry** (always). The sentence names the entered day; the
detail shows both.

### 1.2 The closed list of sources — five labels, no free text

```
from my watch · from a test · from a doctor or a lab · from my training partner's watch · typed
```

* **A closed list, so the app has nothing to interpret.** A free-text source would make the app read
  English and decide what a phrase means; a label does not. The source is a **label the athlete chose**,
  and the app stores it and shows it.
* **The app checks nothing about any of them.** "from a test" does not make a number better than
  "typed", and the app never says so. This is the same limitation the threshold's own candidates carry
  (`exploration/threshold-setup.md` §1: *"the app cannot check what was put into the watch"*), and it is
  stated the same way.
* **No source is ranked, and no source is refused.** All five are in the list on the same footing, in
  the order the athlete meets them.
* **The two watch-shaped sources are separate labels** because they are different facts: *from my
  watch* is the athlete's own watch; *from my training partner's watch* is somebody else's. The app
  stores the label; the partner's watch is a number the athlete brought.

### 1.3 The history — a history is a history

* **One declaration is in force.** At most one row carries it (the same shape as
  `threshold_observations.in_force`).
* **A change writes a new declaration, never an edit.** The new declaration carries the value that did
  not change forward, so a declaration is always one coherent statement: *"my maximum is 186 and my
  resting rate is 52"*. §6.1 explains why that matters.
* **The earlier declaration stays**, with its own date, its own sources and its own measured dates.
* **Nothing is deleted and nothing is overwritten.** A declaration from March can still be read in
  December, and any answer given in March names the declaration it was computed from.

### 1.4 What each combination makes possible, and what it does not

| what the athlete entered | the bands the app can derive | what the app can then answer | what stays a no value, with a reason |
|---|---|---|---|
| **both** — a maximum and a resting rate | the ladder on the **heart-rate reserve** (the wider, more informative range) | zone time and shares over any run with a heart rate; the band boundaries; the comparison with the watch's zones | nothing about the intensities |
| **a maximum alone** | the ladder on the **share of the maximum** | zone time and shares; the band boundaries; the comparison with the watch's zones | the bands **as a share of the heart-rate reserve** — the catalogue already has the line for it: *"no resting heart rate — the bands as a share of your heart-rate reserve have no value"* |
| **a resting rate alone** | **none** — there is no range without a maximum | the resting rate is stored and shown; it is a fact the athlete gave | every band, and every band-shaped answer — *"no maximum heart rate"* |
| **neither** | none | the app answers as it does today, with its reasons | every band-shaped answer |

Two consequences worth writing down:

* **A resting rate alone is not a smaller version of a maximum.** It is a different kind of fact, and
  the app says so in the same words it already uses for the missing maximum: *"no maximum heart rate —
  your watch's zones hold none, so the heart-rate bands have no value"* (the existing catalogue line,
  `crates/stride-core/src/strings.rs`).
* **The plan's own session bands are a separate rule.** The book's bands are stated as shares of the
  maximum *and* of the reserve (general aerobic 70–81 % maxHR / 62–75 % HRR; recovery below 76 % maxHR
  / 70 % HRR — `exploration/pfitzinger-spine.md` §3). With a maximum alone, the **% maxHR** side
  computes; with both, the **% HRR** side computes too. With a resting rate alone, neither side does.
  The plan's bands are never the app's intensity bands (D17) — they are two different rules and they
  are labelled as two different rules.

---

## 2. The bands — one rule, cited

### 2.1 The citation

Garmin publishes the ladder. Two of Garmin's own pages carry it:

**[G1] Garmin device manual — *Forerunner 245/245 Music Owner's Manual*, "About Heart Rate Zones" →
"Heart Rate Zone Calculations".** `garmin.com/manuals` (the same section is in the Forerunner 965,
fēnix 8, vívoactive and Forerunner 55 manuals under the same heading).

> "Generally, heart rate zones are calculated based on percentages of your maximum heart rate."

and the table it publishes:

| Zone | % of Maximum Heart Rate |
|---|---|
| 1 | 50–60 % |
| 2 | 60–70 % |
| 3 | 70–80 % |
| 4 | 80–90 % |
| 5 | 90–100 % |

and the definition of the other range it can be expressed on:

> "Select %HRR to view and edit the zones as a percentage of your heart rate reserve (maximum heart
> rate minus resting heart rate)."

and, on what the device does by itself:

> "The default settings allow the device to detect your maximum heart rate and set your heart rate
> zones as a percentage of your maximum heart rate."

**[G2] Garmin — "How You Can Train by Heart Rate Zones Using Garmin"** (`garmin.com/en-AU/blog/how-you-can-train-by-heart-rate-zones-using-garmin`):

> "Here's how our default heart rate zones break down, using running as an example. If you customise
> your zones, then they may not match these descriptions."

> "**Zone 1** (warm-up, 50-60% of max heart rate) … **Zone 2** (easy, 60-70%) … **Zone 3** (aerobic,
> 70-80%) … **Zone 4** (threshold, 80-90%) … **Zone 5** (maximum, 90-100%)".

**[G3] Garmin — the same page, on the age estimate the app refuses:**

> "Garmin will estimate it starting with this formula: 220 minus your current age. … However, your
> Garmin product will update this estimate over time using data from your workouts."

**What the citation does and does not cover.** Garmin publishes the five-zone ladder, and publishes
the two ranges it can be expressed on — the maximum heart rate, and the heart-rate reserve (maximum
minus resting). It does **not** publish a second table for the reserve; the app therefore applies
**the same published percentages** to **the range Garmin defines**. That is the rule below, and it is
one rule with two ranges — not two rules.

### 2.2 The rule, exactly

> **The five bands are Garmin's published five-zone ladder — 50 %, 60 %, 70 %, 80 %, 90 % — applied to
> the range the athlete's declaration supports.**

**With both numbers** (a maximum `M` and a resting rate `R`):

```
reserve = M − R
floor(Z1) = R + 0.50 × reserve
floor(Z2) = R + 0.60 × reserve
floor(Z3) = R + 0.70 × reserve
floor(Z4) = R + 0.80 × reserve
floor(Z5) = R + 0.90 × reserve
top(Z5)   = M
```

**With a maximum alone**:

```
floor(Z1) = 0.50 × M
floor(Z2) = 0.60 × M
floor(Z3) = 0.70 × M
floor(Z4) = 0.80 × M
floor(Z5) = 0.90 × M
top(Z5)   = M
```

**With a resting rate alone, or with nothing**: no bands (§1.4).

**Rounding.** Each floor is rounded to the nearest whole beat per minute, halves up. The rounding is
part of the rule, so two runs of the app on the same number give the same bands.

**Which range the app uses.** When both numbers are present the app uses the **heart-rate reserve**,
because the resting rate is part of the declaration and the reserve is the range Garmin defines for a
pair. When only a maximum is present the app uses the **share of the maximum**. The choice is a rule of
the app, it is fixed, and **every answer names the range it used** — so no answer is ever read as a
band on a range the athlete did not give.

**The wording of the range on the surface** is the runner's, not the rulebook's: *"from your maximum
heart rate and your resting heart rate"* and *"from your maximum heart rate"*. The precise term — *the
heart-rate reserve* — belongs to the notes (D24).

**The derived bands are not the watch's bands.** The bands above are computed from the athlete's
declaration. The watch's zones are the watch's own configuration. Where the two differ, they are two
different sets of numbers, and the app says so (§3).

### 2.3 What the rule produces for the canonical athlete

The declaration in force (`exploration/athlete.js` → `heartRates.inForce` — **synthetic data, added for
this prototype**):

| | value | where it came from | measured | entered |
|---|---|---|---|---|
| **your maximum heart rate** | **186 bpm** | from a test | 8 Nov 2026 | 24 Nov 2026 |
| **your resting heart rate** | **52 bpm** | from a doctor or a lab | 12 Nov 2026 | 24 Nov 2026 |

`reserve = 186 − 52 = 134`

| band | the share | the floor | the band |
|---|---|---|---|
| Z1 | 50 % of the reserve | **119 bpm** | 119–131 |
| Z2 | 60 % | **132 bpm** | 132–145 |
| Z3 | 70 % | **146 bpm** | 146–158 |
| Z4 | 80 % | **159 bpm** | 159–172 |
| Z5 | 90 % | **173 bpm** | 173–186 |

**The watch's own floors, read on 3 Nov 2026** (the dated fact, untouched): **115 · 128 · 143 · 159 ·
165**, with the watch's own maximum of 183 bpm.

**The two sets, side by side:**

| boundary | from the heart rates you entered | your watch's zones | the difference |
|---|---|---|---|
| Z1 | 119 | 115 | **+4** |
| Z2 | 132 | 128 | **+4** |
| Z3 | 146 | 143 | **+3** |
| Z4 | 159 | 159 | **0** |
| Z5 | 173 | 165 | **+8** |

The maximums differ too: **186** declared against the watch's **183**.

**The same rule on a maximum alone** (186 bpm, no resting rate): **93 · 112 · 130 · 149 · 167**, top
186.

**And the earlier declaration** (a maximum of 183 from the athlete's watch, entered 3 Nov 2026, with no
resting rate): **92 · 110 · 128 · 146 · 165**, top 183.

### 2.4 Where the numbers live

The derived bands are **not stored**. They are a pure function of the declaration in force (§6.2), the
same rule the rest of the app already follows: *"derived series are recomputed on read, never stored"*
(`docs/contract/schema.sql` header).

### 2.5 The source line every answer carries

The app already has a source line for the watch's model: *"your zones · running · max heart rate 183
bpm · taken from your watch on 3 Nov 2026"*. The declared basis needs its own, and it names the range
it used:

| the basis | the line |
|---|---|
| the watch's zones | *"your zones · running · max heart rate 183 bpm · taken from your watch on 3 Nov 2026"* (the existing line) |
| the declared pair, both numbers | *"your zones · the bands from the heart rates you entered · maximum heart rate 186 bpm · resting heart rate 52 bpm · entered on 24 Nov 2026"* |
| the declared pair, a maximum alone | *"your zones · the bands from the heart rates you entered · maximum heart rate 186 bpm · entered on 24 Nov 2026"* |

The settings card carries a shorter form of the same line, because the card shows the values
underneath it: *"the bands from the heart rates you entered · from your maximum heart rate and your
resting heart rate · entered on 24 Nov 2026"*. Both forms name the basis and the date; neither names a
verdict.

Both lines name **what** the bands are and **when** they were dated. Neither says the other is wrong.

---

## 3. Two dated facts, never blended

### 3.1 The two facts

| | **the watch's model** | **the athlete's declared pair** |
|---|---|---|
| what it is | the five floors in the watch's settings | the maximum and/or the resting rate the athlete entered |
| dated by | the day it was **read** — 3 Nov 2026 | the day it was **entered** — 24 Nov 2026 |
| where it comes from | the source, read once per sync | the athlete |
| it changes when | the watch's settings change | the athlete enters a new declaration |

They are **two facts**. The app **states** them, and **states that they differ** when they differ.

### 3.2 The five rules

1. **Both are stated.** Any answer that uses a band names the basis it used (§2.5). The other fact is
   stated beside it — the watch's model on the settings card, the declared pair wherever the bands
   appear — so the athlete can always see both.
2. **The difference is a fact, and it is stated as one.** When the two differ, the app says they differ
   and by how much, boundary by boundary (+4, +4, +3, 0, +8 bpm for the canonical athlete). **It is not
   a correction.** The app does not say the watch is mis-set, does not say the watch's number is stale,
   and does not say the athlete's number supersedes the watch's.
3. **Neither is a verdict on the other.** "Your watch's zones" and "the bands from the heart rates you
   entered" are two sets of numbers with two dates. Nothing in the app ranks them, and the ban list
   holds: no "good", "poor", "stale", "wrong".
4. **Bands are never mixed.** No answer, no chart and no card ever puts a band from one fact beside a
   band from the other. The derived bands are derived whole; the watch's floors are the watch's whole.
   A chart that showed Z3 as 146 on one axis and 143 on another would be exactly the blending this
   forbids.
5. **One is never silently preferred.** Where the app must pick a basis for an answer, the choice is a
   **rule** (§3.4), the rule is written down, and the answer names the basis. Silence is the thing
   forbidden here, not the choice.

### 3.3 The wording, where it appears

| where | what it says |
|---|---|
| the settings card | *"Your watch's zones · read from your watch on 3 Nov 2026"* and *"The bands from the heart rates you entered · entered on 24 Nov 2026"* |
| the comparison under the two band pictures | *"Your watch's zones and the bands from the heart rates you entered are different."* |
| every answer that uses a band | the source line of §2.5 |

The comparison sentence is the only place the difference is written out; everywhere else the picture
carries it (D11), and the source line carries the fact.

### 3.4 Which basis the ledger's app-basis uses — the trade-off, and the recommendation

**Option 1 — the athlete's declared pair, applied to every run.** One ladder, one date, over the whole
history: every run with a heart rate is measured with the bands derived from the declaration in force,
and the answer names the declaration's date.

**Option 2 — the watch's model where one was in force, the declared pair for the rest.** More faithful
to what the watch measured against on the days a model was in force; two bases inside one answer.

**The trade-off.** Option 1 gives the ledger **one comparable series over the whole window**: a band
share in September means the same range as a band share in November, which is the property that makes
the twelve-week strip and the intensity rows readable at all. Its cost is that the app's bands are the
athlete's declared bands and **not** the watch's configuration, so where the two differ (§2.3) the
app's band boundaries differ from the boundaries the watch itself used — the app's Z3 is 146–158 bpm
while the watch's zone 3 began at 143 — and the app must say so rather than let the two be read as one
number. Option 2 is more faithful to the watch on the days a model was in force, but it puts **two
bases inside one answer**: the same band name would carry two different boundary sets in the same
chart and the same column, which is the thing §3.2.4 forbids; and it leaves the whole pre-model
history with **no app-side zone time at all**, so the ledger's intensity rows stay empty over exactly
the runs the athlete most wants to see — the hole `exploration/zone-model-history.md` opened.

**The recommendation: Option 1 — the athlete's declared pair, applied to every run.** One basis, one
date, over the whole history, with the watch's model kept as the **other fact** and stated wherever
the two are compared. Three reasons, in the order they matter: (a) the ledger's intensity rows become
complete and comparable across the window, which is the hole this whole thread exists to close;
(b) one answer carries one basis, so nothing is ever blended; (c) the watch's own model is not lost —
it stays a dated fact, it is compared with the declared pair wherever the bands are shown, and the
watch's own per-run zone minutes stay a separate block with their own basis, so the athlete can see
all three.

**What this costs, stated plainly.** The app's zone minutes for a run will not equal the watch's own
zone minutes for that run, because the boundaries differ and the two are measured over different spans
(moving time against elapsed time). That is the sibling design's trade-off, unchanged — the two blocks
stay separate blocks, and the overlap check becomes available over the **whole history** rather than
over the eight runs after 3 Nov 2026, because the app now holds bands for every run.

**This is a change to Z's rule** (D3, D18.3 — "every run is measured with the model in force on the day
of the run"). It is the parent's decision; §8 lists it as the first open question.

---

## 4. The onboarding

### 4.1 Where it lives

**On the rig's own page, as the fourth of the choices the athlete can make** — the section that today
reads *"Three things only you can give it"* and carries the three cards side by side (**a race**, **a
threshold**, **a programme**) becomes four, and the fourth is **your heart rates**.

* The rig's page is the right home because it is where the athlete's own choices already are, listed
  side by side and unranked, and because a fresh rig leaves all of them empty and asks for nothing
  (`desktop/src/rig.rs`, the `decisions_section`).
* **The heart rates are entered in the app, not on the rig's page.** The app needs a screen for them
  anyway — the place to change them afterwards (§4.4) — and two places to type the same two numbers is
  two truths. The rig's card follows the pattern the threshold already uses: it says what is in force
  and **which screen the choice is made on**. Only the race is typed on the rig's page, because the app
  has no screen for it.
* So the rig's fourth card reads, when nothing is entered: *"No heart rates entered."* and *"Your heart
  rates are in the app, under Settings › Training."* When something is in force: *"In force since 24
  Nov 2026: your maximum heart rate 186 bpm · your resting heart rate 52 bpm."*

**The page's rule holds.** The four cards are listed side by side, in the same shape, unranked; **the
vitals are never linked to them**; nothing on the page grades anything; and **every one of them can
stay empty**. The heart-rate card adds no step, no wizard, no "next", and no order.

### 4.2 The card in the app

**Settings → Training → Your heart rates**, beside the threshold. The app's settings page already
carries a **Training** section (`/settings?section=training`) — the challenges, the threshold, the goal
and the pause live there — and the heart rates are the same kind of thing as the threshold: the
athlete's own training fact. They belong beside it, in that section.

The card carries, in the app's own voice:

* the heading **"Your heart rates"**;
* two fields — **"Your maximum heart rate"** and **"Your resting heart rate"** — each with a number
  box, a **"Where it came from"** picker carrying the five labels, and a **"When it was measured"**
  date that may be left empty;
* a line naming the entry: *"entered on 24 Nov 2026"*, and, where a measurement date was given, the
  measurement date beside it;
* **the bands it produced**, drawn against the watch's own floors (§4.3);
* **the history**: the earlier declarations, each with its own date, kept.

**Either field may be left empty.** The card saves what is there and says what is missing, in the
words the catalogue already uses.

### 4.3 What the card shows — the difference, visible

The card draws the two sets of bands **on one axis, one above the other**: the bands from the heart
rates the athlete entered as filled segments, and the watch's floors as ticks on a line beneath them.
The differences are then a picture before they are a sentence, which is the ledger's own rule (D11) and
the reason the sentence under it can be short.

The **prototype** (`exploration/heart-rate-setup.html`) draws exactly this, on the phone and on the
desktop.

### 4.4 The app's own place to change it afterwards

**Settings → Training → Your heart rates.** The same card, in edit mode:

* the field that is changing is the only one that changes; the other value is carried forward, so the
  declaration stays one coherent statement (§1.3);
* saving writes a **new declaration**, dated today, and the earlier one stays in the history;
* the bands are re-derived at once, and every answer that uses them now names the new date (§5).

### 4.5 The states

| state | what the card says | what the bands say |
|---|---|---|
| **no heart rates entered** | *"No heart rates entered."* + the two fields, empty | no bands — *"no maximum heart rate"*, and the app's answers keep the reasons they have today |
| **only a maximum** | the maximum, its source, its date | the ladder on the share of the maximum — **93 · 112 · 130 · 149 · 167** for 186 bpm |
| **only a resting rate** | the resting rate, its source, its date | **no bands** — the reserve has no maximum, and the app says so |
| **both** | both values, both sources, both dates | the ladder on the reserve — **119 · 132 · 146 · 159 · 173** |
| **changed later** | the new declaration at the top, the earlier one below it with its own date | the bands of the declaration in force, and the history carrying the earlier one |

---

## 5. "Thereafter the app should take care of it"

What the app does with the fact, with **no work for the athlete**:

1. **It derives the bands** (§2.2) and uses them **wherever a band is needed**: the ledger's intensity
   rows (zone time, the shares, the last date in each band), the run detail's own band minutes, the
   band boundaries wherever they are drawn. The derivation happens on read (§2.4) — nothing is stored,
   nothing is typed again.
2. **It re-derives them when the pair changes**, and **the older answers stay as they were, dated**.
   Every answer carries the date of the declaration it was computed from, so a March answer is read
   with the March declaration, and the declaration itself stays in the history (§1.3). The athlete
   changes nothing but the declaration.
3. **It compares with the watch's model when one exists, and states the difference** — the side-by-side
   picture, the difference boundary by boundary, and the two source lines (§3.3). The comparison is
   made whenever both exist, and the app does not make it when there is nothing to compare.
4. **It never asks again.** The entry is not re-prompted; no screen says the numbers are old; no
   notification, no banner, no "update your heart rates". The rig's card names the screen and says
   what is in force; that is all it does.

**What the declaration does *not* touch** (so that nothing moves quietly):

| | |
|---|---|
| **the watch's own zone minutes** | the source's own record of each run — never recomputed, never re-banded |
| **the watch's configured floors** | the watch's own fact — never overwritten by the declaration |
| **the load model** | it keeps the watch's dated model as its reference (D19.3, D3 stage 2). The load's own line names the maximum it used. Moving the load to the declared pair is a separate decision, with its own announcement |
| **the plan's session bands** | the book's own rule, unchanged (D17) |
| **the threshold** | the athlete's own choice, unchanged; the declaration does not move it and the threshold does not move the declaration |

---

## 6. The storage shape

### 6.1 The table — beside the existing dated facts

One new table, beside `race_observations` and `threshold_observations`, in the frozen schema's own
style and vocabulary.

```sql
-- Old: (nothing - the old app held no heart rate the athlete gave it).
-- v8: the athlete's own heart rates: the maximum and/or the resting rate,
-- with where each came from, the day it was measured and the day it was
-- entered.
--
-- ONE ROW IS ONE DECLARATION. A maximum and a resting rate can therefore
-- never be read from two different declarations - the pair is one coherent
-- statement, and a change writes a new row that carries the unchanged
-- value forward.
--
-- At most one row carries in_force = 1. The earlier declarations stay, with
-- their own dates and their own sources.
--
-- A missing value is a missing value: the column is NULL and never 0. The
-- derived bands are NOT stored - they are recomputed on read from the
-- declaration in force, by the rule in exploration/heart-rate-setup.md.
CREATE TABLE heart_rate_observations (
    id                      INTEGER NOT NULL PRIMARY KEY,
    max_heart_rate_bpm     REAL,             -- NULL when the declaration carries no maximum
    max_source             VARCHAR(24),      -- 'watch' | 'test' | 'doctor_lab' | 'partner_watch' | 'typed'
    max_measured_on        VARCHAR(10),      -- the day the measurement was taken; NULL when not stated
    resting_heart_rate_bpm REAL,             -- NULL when the declaration carries no resting rate
    resting_source         VARCHAR(24),      -- the same five labels
    resting_measured_on    VARCHAR(10),
    entered_on             VARCHAR(10) NOT NULL,  -- the day the declaration was made
    updated_on             VARCHAR(10) NOT NULL,  -- the day the row was last changed
    in_force               INTEGER NOT NULL DEFAULT 0  -- 1 = the declaration in use now
);
```

**What it holds.** One row per declaration. The two heart rates, each with its source label and its
optional measurement date; the entry date; the update date; and which declaration is in force.

**What it does not hold.** No derived band, no zone, no score, no age, no birthdate, no free text, and
no verdict. The bands are recomputed from the row on read, by the one rule.

**The schema version.** `SCHEMA_VERSION` is 7 today (`crates/stride-store/src/schema.rs`); the table
makes it **8**, with a migration entry, and `docs/contract/schema.sql` and the crate's DDL both carry
it. The schema is frozen, so this is a plan change and travels with a `PLAN.md` edit in the same pull
request (`docs/contract/schema.sql` header).

### 6.2 What the export carries

The export copies **every table of the frozen schema** — `FROZEN_TABLES` in
`crates/stride-store/src/export.rs` — so the new table joins that list and its rows are carried in the
exported database, and counted in the manifest's per-table counts. Nothing about the export changes
shape: it is one more table in a list that is already "every table".

The export's own summary line on the tools page (`desktop/pages/tools.html`) names the tables the
athlete cares about; the heart rates are the athlete's own training facts and belong on that line the
way the races and the plan do — a one-line change, for the parent to take or leave.

---

## 7. What the surface says, in the runner's words

| the thing | the line |
|---|---|
| the athlete's own pair | *"your maximum heart rate"* · *"your resting heart rate"* |
| where it came from | *"from my watch"* · *"from a test"* · *"from a doctor or a lab"* · *"from my training partner's watch"* · *"typed"* |
| the bands it produced | *"the bands from the heart rates you entered"* |
| the watch's own | *"your watch's zones"* · *"read from your watch on 3 Nov 2026"* |
| the difference | *"Your watch's zones and the bands from the heart rates you entered are different."* |
| the entry | *"entered on 24 Nov 2026"* |
| a missing value | *"no maximum heart rate"* · *"no resting heart rate"* — the catalogue's own lines, never a zero and never a default |

**No wording on any surface says** the watch's zones are wrong, that the athlete's numbers are
better, that the numbers are old, or that either one should be changed.

---

## 8. Open questions for the parent

1. **The basis.** §3.4 recommends the declared pair for every run. That changes Z's rule (D3, D18.3)
   and it is the parent's decision. The alternative — the watch's model where one was in force, the
   declared pair for the rest — is designed alongside it and costs one switch.
2. **The load model.** The declaration does not touch it (§5). Moving the load to the declared maximum
   is D3's stage 2 and its own announcement.
3. **The rig's card.** The rig's fourth card says where the heart rates are entered; if the rig's page
   should *type* them as well as point at them (like the race), that is the rig's own decision, and the
   two places must not become two truths (§4.1).
4. **The label of the declaration's basis in the source line** — the wording in §2.5 is a proposal, and
   the catalogue's own `zones.model.source` line is the pattern it follows.
5. **The export's summary line** (§6.2) — one line, or nothing.

---

## 9. The prototype

`exploration/heart-rate-setup.html` — plain HTML/CSS, no frameworks, no network, `file://`. It carries:

1. the **rig's page** with the four choices side by side, the heart-rate card among them;
2. the **app's card** — the two inputs with their sources, the entry dates and the history;
3. the **two sets of bands on one axis**, so the difference is a picture before it is a sentence;
4. the **five states** (§4.5);
5. the phone and the desktop widths, and the notes toggle, off by default (D22).

Every number is read from `exploration/athlete.js`; the markup holds none of its own.
