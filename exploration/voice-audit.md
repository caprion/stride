# The voice audit — what Progress says, and who it leaves the work to

**An audit, not a proposal to ship.** Nothing here is implemented. The
approvals are the user's to give.

It looks at one thing: **who is doing the reading.** The athlete opens
Progress, and either the app has understood their training and says so,
or the app hands them six datasets and asks them to work it out.

The short answer: **the app hands it to the athlete, and it did not have
to.**

---

## 1. The one number

| | |
|---|---|
| app strings that are labels (`status`) | **529** of 556 |
| app strings that are sentences (`statement`) | **15** |
| app strings that ask the athlete a question | **3** |
| **core strings that ask the athlete a question** | **0** of 198 |

The core never asks. Every sentence it owns is a statement: *"the form
turns positive in 4 days"*, *"the acute-to-chronic ratio is expected back
inside 0.8 to 1.3 in 1 day"*, *"fitter than 8 weeks ago"*. The app asks:
*"Am I training consistently?"*, *"How has my pace behaved?"*, *"How was my
sleep?"*.

**The question and the sentence live in different halves of the
product**, and only one of them has a voice.

---

## 2. Finding one — the coach's sentence exists, and it is disconnected

The plan specifies it, the contract permits it, the core implements it,
the core tests it twice — and the api never calls it.

`crates/stride-core/src/statement.rs`:

```rust
/// The Progress headline and the evidence under it.
pub struct ProgressHeadline {
    /// The headline: "fitter than 8 weeks ago".
    pub headline: String,
    /// The evidence clause, when there is one: "same heart rate, 24 s/km
    /// faster".
    pub evidence: Option<String>,
}
```

Three wordings, with the evidence in a clause — exactly what PLAN §0
specifies:

```
statement.progress.fitter     | fitter than {ago}
statement.progress.slower     | slower than {ago}
statement.progress.unchanged  | the same pace as {ago}
statement.progress.evidence.faster | same heart rate, {pace} s/km faster
```

And here is the api, `crates/stride-api/src/routes/insight.rs:224`:

```rust
// Independent weekly medians are descriptive, not matched-effort evidence.
let headline = if weekly_volume.is_empty() {
    statement::progress_headline(None).headline
} else {
    stride_core::strings::text("status.saved_running_history")
        .unwrap_or_default()
        .to_string()
};
```

So the Progress screen's lead sentence is **`status.saved_running_history`
— "Your saved activity history"** — and `headline_note` is hard-coded
`None` three lines later.

**"Your saved activity history" is a chart's title.** It is what a
heading says, not what a fellow runner says. And it sits directly under
the page's title, which asks *"Am I training consistently?"*

> **Progress**
> **Am I training consistently?**
> **Your saved activity history**
> 23 Feb – 13 May
> 58.4 km · 5 h 11 m · 6 sessions · 2 weeks
> *Saved activity history; gaps are not confirmed rest weeks.*

The question is never answered. What follows it is a title and four
figures.

### The reason the sentence was withheld is correct

The comment is right. `progress_headline` wants a **matched** comparison
— the pace now and the pace then *at the same heart rate*. The api only
has independent weekly medians: the pace a week ran at, and the heart
rate that week ran at, measured separately. Reading "fitter" off two
independent medians is exactly the error the plan warns about ("Weekly
pace and heart rate are separate medians, not a same-effort fitness
comparison" — the app already says this, on the pace card).

**But nothing was built to replace it.** The gap was filled with figures.

---

## 3. Finding two — the index is a table of contents with numbers in it

The narrow layout is six rows. Each row is: a heading that names a
category, a line of raw values, and a window.

| the row says | what it is |
|---|---|
| "This window against the window before" | a heading, not a question |
| "Acute load {n} · Form {n}" | two of the four numbers the panel shows |
| "The training load" | a category |
| "Acute load {n} · Form {n}" | the same two numbers |
| "The year" | a category |
| "{km} · {n} runs" | two of the four numbers the panel shows |
| "The pace" | a category |
| "{pace} /km at {hr} bpm" | one number and its context |
| "Your zones" | a category |
| "The habit" | a category |
| "{n} day · {n} weeks · {n} months" | three counts |

Three things are wrong with this, and they are the same thing:

1. **The rows are the panels' figures, not the panels' answers.** Tapping
   a row shows more of the same numbers. The index adds no understanding
   to anything — it is a table of contents that happens to print the
   contents.
2. **The headings are nouns, so they cannot be read.** "The training load"
   is a label. A reader cannot learn anything from it.
3. **The athlete is the analyst.** The app hands over six datasets and
   leaves the interpretation to the person who ran them.

And the frame makes it explicit: *"Six questions, and your answer to
each."* The app frames itself as the examiner.

---

## 4. Finding three — the only human voice in the product is the one a model writes

There is exactly one place in Stride where prose is written *about the
athlete's running*, and it is the run update. The register is in the
contract, and it is the best writing in the product:

> *Write the update in four to six sentences.*
>
> *- Say what the run was and how it went.*
> *- If something in the laps or the mechanics stands out, say what stands out.*
> *- Address the runner as "you".*
> *- Observations only: no advice, no verdict, no score, no commands.*

The machinery is all there: the payload (every number the model may use,
and nothing else), the writer and a reviewer of a different family, the
checks, the "ship / repair / discard" verdict, the hash of the payload the
text was written from.

**So the product already knows how to talk like a fellow runner — about
one run. It just never does it about the training.**

That is the asymmetry. The training is where the understanding lives, and
the training is the part with no voice.

---

## 5. This is not a voice change — the contract already wants it

PLAN §0, the first line:

> **Not a data dump, and not a set of orders.**

And the Progress headline is specified:

> The Progress headline is the factual mirror of the evidence, in one of
> three wordings — *fitter than 8 weeks ago*, *the same pace as 8 weeks
> ago*, *slower than 8 weeks ago* — with the evidence in a clause.

The contract bans **judgements** ("good", "bad", "poor", "missed",
"behind", a readiness score) and **commands** ("should", "must", "rest
day", "go easy"). It does not ban **explanations**. "Fitter than 8 weeks
ago, same heart rate, 24 s/km faster" is a fact about two readings, not a
judgement about a person.

So the line the user drew is the line the contract already drew:

| | allowed | not allowed |
|---|---|---|
| explaining | "fitter than 8 weeks ago — same heart rate, 24 s/km faster" | "you are overtraining" |
| advising | — | "ease off this week" |
| comparing | "higher than the 12 weeks before it" | "too high" |

**The gap is not in the rules. It is in the wiring.**

---

## 6. What a Progress that talks would look like

Three parts, in this order: **a reading**, **the picture**, **the way in**.

```
PROGRESS
Your training · 23 Feb – 13 May

  You have covered 58 km in 2 weeks — 21 km more than the 2 weeks
  before it.

  At the same heart rate, you were 24 s/km faster than you were eight
  weeks ago.

  The last seven days carried more load than your body has been used to:
  the acute load is 20 against a chronic 7, and the form has been
  negative for three weeks. With no training at all, the ratio is
  expected back inside 0.8–1.3 in seven days.

  Your pace at 148 bpm has been getting slower: 5:24 /km in the last
  week, 5:31 /km six weeks ago.

  Two thirds of your moving time has been in Z4–Z5 — more than in the 91
  days before it.

  You have run every week for 12 weeks.

  ┌ the radar ─ the picture, two windows on six axes ┐
  │  Six axes, with the shape of each of the two      │  Open →
  │  windows drawn on them.                            │
  └───────────────────────────────────────────────────┘

  The load  ·  The year  ·  The pace  ·  Your zones  ·  The habit  ·
  The window
```

(The numbers are the corpus's, not an athlete's.)

Read every sentence against the rules:

- **No verdict.** "More load than your body has been used to" is a
  comparison against the athlete's own chronic load, not a judgement.
- **No advice.** Nothing says what to do.
- **Every number has its period.** "eight weeks ago", "the 91 days before
  it", "12 weeks".
- **Every number is the core's.** The pace, the load, the scenario, the
  streaks, the zone split — all already computed and already rendered by
  the core in its own words.

The last line changes character: the six panels stop being *answers* and
become *ways in*. That is the whole structural fix — **the index is a
table of contents, so it should read like one**, and the answer belongs
above it.

And the page title stops asking, because the reading answers it.

---

## 7. What it would take

### 7a. The matched comparison — the only new maths

The headline needs the pace now and the pace then *at the same heart
rate*. The app has per-run pace and per-run average heart rate, so it can
fit a pace-at-heart-rate line over the runs and read it off at a common
heart rate. That is the standard way "fitter" is actually measured.

The core has `pace_bands` — the distribution of a window's time *by pace* —
and nothing that normalises by heart rate. So the missing piece is a core
function of the shape:

```
pace_at_hr(runs, hr) -> Option<f64>
```

fitted over a window's runs, read at a given heart rate, with the two
windows compared. Small, and it is the thing that turns "here are your
weekly medians" into "you are fitter than eight weeks ago".

Until it exists, the honest alternative is a headline that does not claim
a change it cannot see — which is what the api does now, correctly, but
with a chart title instead of a sentence.

### 7b. Where the sentences come from

| | what | cost | what it sounds like |
|---|---|---|---|
| **A. the core assembles them** | each sentence is a `statement` the core already knows how to write; the api concatenates | nothing | a form, not a person — the same six sentences every time |
| **B. a model writes it** | the run update's machinery, with a training payload | a call per window, and caching | a fellow, varying with the data |
| **C. a hybrid** | the core writes the fixed facts, a model connects them | a call per window | a fellow who cannot get the facts wrong |

**C is the one I would build**, and the reason is specific: the run
update's discipline — *every number you may use is in this payload, do
not use any other number* — is what makes a written sentence safe. A
model that only sees sentences the core has already computed, each with
its own number and period, can connect them but cannot invent one. That
is the same guarantee the run update gives, at a larger scope.

The cost is real and it is new: the run update is written **once per
run**; a training reading is written **per window**, and the window
changes with the range selector. It would need caching keyed on
(window, last change) and a "reading is being written" state, or it
becomes a spinner on every range click.

### 7c. The register, if a model writes it

The run update's register lives in `docs/contract/voice.md`. A training
reading would need its own — and the plan's rule is that a contract change
travels with a PLAN.md amendment in the same change. The rules would carry
over from the update (payload only, no advice, no verdict, no commands) and
add the training-specific ones: no score, no readiness language, and the
directions limited to the update's own set (faster · slower · higher ·
lower · held).

### 7d. The index becomes a way in

Each row becomes a sentence and a link rather than a line of figures, and
the figures move to the panel they belong to. That is a small change to
`ProgressIndex.tsx` — but it needs the sentences to exist first, which is
7a and 7b.

---

## 8. The decisions, in the order they have to be made

1. **Does the app get a voice about the training at all?** — The contract
   permits it, the plan asks for it, and the mechanism exists. This is the
   question everything else waits on.
2. **Where does the reading come from?** — A, B or C. C if a model is
   allowed to write; A if it is not.
3. **The matched comparison.** — `pace_at_hr` in the core, and then the
   headline can be the plan's own three wordings instead of a chart title.
4. **The index's rows.** — sentences and links, or keep the figures and
   accept that the index is a table of contents.
5. **The page title.** — *"Am I training consistently?"* cannot stay if
   the reading answers it.

---

## 9. What this audit did not look at

- **The run update's own writing.** It was written once, under the old
  rules, and has not been re-read since. It is the product's best prose
  and it deserves its own audit.
- **Today's voice.** Today reads well — *"−0.6 km against the same days
  last week"*, *"Form −12.7"* — because it is facts about *now*, in the
  core's words. It is not a coach; it is a scoreboard. Whether Today should read
  more like a friend is a separate question.
- **The two radars.** `AthleteShape` and `ProfileRadar` draw the same six
  axes over the same two windows, in two different ways, on two different
  screens. One of them should go, and which one is a design decision, not
  a cleanup.
- **The Body screen.** "How was my sleep?" is the third question the app
  asks, and the same finding applies to it.
