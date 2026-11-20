# Stride v2 — the voice (string types and rules)

FROZEN. Part of the contract freeze (Slice 0). This is the machine-checkable
half of the voice model in `PLAN.md` §0 and `docs/plan.html` §5.9. Every
user-facing string is **typed** in the core or frontend catalogue. The core
exports strings and shared rules as `crates/stride-core/strings.json`; the
frontend catalogue lives in `frontend/src/i18n/catalogue.ts`. CI lints both.

The rule in one line: **facts first, no judgements at all** — the vitals, the
direction each moved, and the plan's own options. The app never tells the
athlete what to do, never scores them, and never links an option to a vital.

## The seven string types

There are seven string types: the four **registers** — `vital`, `direction`,
`option`, `pattern` — plus the three supporting types, `statement`, `status`
and `instruction`. All seven are typed in the catalogue; the four registers
are the content, `statement` carries the sentences that state a fact about a
comparison, `status` covers connect / sync / staleness / empty states, and
`instruction` is the product's own instruction to a model — the run update's
prompt, and nothing else.

| type | what it is | where it lives |
|---|---|---|
| `vital` | label · value · unit — "Sleep — 6 h 10 m" | Today, Body |
| `direction` | a word from the closed set, beside muted arithmetic and travel — "could be better · 1 h 10 m less than your usual · 3rd night down" | Today, Body |
| `option` | a session or alternative from the athlete's own plan, listed unranked, never conditioned on the vitals | Today |
| `statement` | a factual sentence about a comparison — "2.4 km more than the 10 km target", "fitter than 8 weeks ago", "Same distance, 3 weeks apart" | Today, Progress, Compare |
| `pattern` | what usually happened to this athlete, as a fact about the past, always with its sample size | Body only (never Today, never next to a suggestion) |
| `status` | connect/sync/staleness and empty-state lines ("last synced 2 days ago", "not enough data for a comparison yet") | anywhere |
| `instruction` | the product's instruction to a model — the run update's prompt: the register, and the rule that only the numbers given may be used | the run update |

The `instruction` strings are the one kind the app never shows: they are
sent to a model, and they are the product's own words. Like every other
kind, they are checked for the banned words — a prompt that names a banned
word would teach the model to use one.

Order on Today is fixed: vitals (with the Rest summary) → the plan's sessions →
the other options → the last run. Nothing draws a conclusion.

## The closed direction set

`better than usual` · `about usual` · `could be better`

A `direction` string must come from this set and **may not be extended**. The
band the word is measured against is defined in `core-contract.md` §4 (median of
the last 30 nights, band = median ± 1.5 × MAD). The "Rest" summary line on Today
is a count + direction ("Rest — could be better · 2 of 3 signals below your
usual") — a pure function of the three vitals' counts, never an editorial
judgement.

## The ban list

The lint fails the build on any of these words or constructions:

```
should   must      take the day   rest day   rest up   rest today   go easy
missed   bad       poor           fail       behind    good        keep it easy
ready to recommended   better to  need to   red        readiness score
not recovered   you are ready   you are not ready
```

`good` is on the list because the plan lists it as a verdict ("no good/bad");
`red` because nothing may be coloured as a problem; `readiness score` because
there is no score; `not recovered` / `you are ready` because they are
conclusions the app does not draw.

`rest` is banned **as a command** (`rest day`, `rest up`, `rest today`) — never
as the noun. The **`Rest`** summary label on Today ("Rest — could be better ·
2 of 3 signals below your usual") is the name of the row, not an instruction,
and is exempt.

## Per-type rules the lint enforces

- **`direction`** — must contain a word from the closed set, nothing else in its
  place. The arithmetic ("14 below your usual") and travel ("3rd night down",
  "steady") are descriptive tails, never verdicts.
- **`pattern`** — must carry its sample size **n** ("the 5 other times …");
  never emitted from fewer than **three** comparable days (two is "twice so
  far", not a pattern).
- **`statement`** — a factual sentence about a comparison. The ban list applies
  to it like to every other kind; it must not be a directive; and **a rendered
  statement may not carry a hole** — the values it compares are already in the
  sentence. A template hole in a statement is where a second implementation
  grows back. Unlike an `option`, a statement **may** name a vital: a choice
  is not a fact, and this is a fact.
- **`option`** — must never reference a vital. "because your sleep was low" is
  the banned link — listing an option *because* a vital was poor would be the
  app judging the vitals. Options are unranked: no "recommended" chip, no
  highlighted choice.
- **no conclusions** — the app never combines the vitals into a verdict ("you
  are not recovered", "you are ready"). It reports each vital's direction; the
  athlete reads the combination.
- **no commands, no verdicts, no shame** — no imperative addressed to the
  athlete, no "good/bad", no red, no "should have".

## The run update's own register

The run update is the one place where a **model** writes the words, so it has
its own rules, and they are the app's rules: the model arranges the facts
the app computed and may not do anything else.

**The direction words are a different set from the closed one.** The closed
set — `better than usual` · `about usual` · `could be better` — is a *vital's*
vocabulary, and a run update is not about a vital: it is about a run. The
words the update may use for a direction are:

```
faster   slower   higher   lower   held
```

They are the app's own words (`update.direction.*` in the catalogue), and
they may not be extended. A model told to avoid the closed set can still
write "better" or "worse" about a run; these five are what it says instead.

**The signs are the app's, and they are fixed.** The app computes the drifts
and the model may only repeat them:

| the app says | the sign means |
|---|---|
| pace drift | second half minus first half — **negative is a faster second half** |
| heart rate drift | second half minus first half, in beats per minute |
| cadence drift | the change in the mean cadence, in percent |
| aerobic decoupling | the heart rate's rise against the pace's — a rise is a decoupling |

A model that reverses a sign is the defect this register exists to catch: the
number is grounded and the meaning is wrong.

**A lap is not a kilometre.** The payload's laps are numbered from one, and
the number counts the laps the watch recorded — it is not a distance marker.
The payload says "lap 4", never a bare "4", and a fragment (a lap under
three hundred metres) is left out of the payload altogether, because a model
that is shown one narrates it.

**The payload carries no title.** The run's title is the athlete's own
words, and a title like "Easy Run" is a word the ban list does not want in
the model's prose — so the title is not in the payload, and a model cannot
quote what it was never given.

## What this replaces

| old product | v2 |
|---|---|
| "Take the day, or make it very easy." | "Rest — could be better · 2 of 3 signals below your usual" — and the plan's options, unlinked |
| "Keep it easy today." | "Sleep — 6 h 10 m · could be better · 1 h 10 m less than your usual" |
| "You missed Tuesday's 10 km." | "Tuesday — 10 km planned · not done" |
| Readiness 63/100 | no score exists |

"easy" and "very easy" survive **as the plan's own vocabulary** (session
types), never as commands and never linked to the vitals.
