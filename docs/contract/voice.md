# Stride v2 — the voice (string types and rules)

FROZEN. Part of the contract freeze (Slice 0). This is the machine-checkable
half of the voice model in `PLAN.md` §0 and `docs/plan.html` §5.9. Every
user-facing string lives in one catalogue (to be `strings.toml`/JSON, added in
Slice 3) and is **typed**. The CI lint fails the build on any violation.

The rule in one line: **facts first, no judgements at all** — the vitals, the
direction each moved, and the plan's own options. The app never tells the
athlete what to do, never scores them, and never links an option to a vital.

## The four registers

| type | what it is | where it lives |
|---|---|---|
| `vital` | label · value · unit — "Sleep — 6 h 10 m" | Today, Body |
| `direction` | a word from the closed set, beside muted arithmetic and travel — "could be better · 1 h 10 m less than your usual · 3rd night down" | Today, Body |
| `option` | a session or alternative from the athlete's own plan, listed unranked, never conditioned on the vitals | Today |
| `pattern` | what usually happened to this athlete, as a fact about the past, always with its sample size | Body only (never Today, never next to a suggestion) |
| `status` | connect/sync/staleness and empty-state lines ("last synced 2 days ago", "not enough data for a comparison yet") | anywhere |

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
should   must      take the day   rest      go easy
missed   bad       poor           fail      behind
ready to recommended   better to  need to
```

## Per-type rules the lint enforces

- **`direction`** — must contain a word from the closed set, nothing else in its
  place. The arithmetic ("14 below your usual") and travel ("3rd night down",
  "steady") are descriptive tails, never verdicts.
- **`pattern`** — must carry its sample size **n** ("the 5 other times …");
  never emitted from fewer than **three** comparable days (two is "twice so
  far", not a pattern).
- **`option`** — must never reference a vital. "because your sleep was low" is
  the banned link — listing an option *because* a vital was poor would be the
  app judging the vitals. Options are unranked: no "recommended" chip, no
  highlighted choice.
- **no conclusions** — the app never combines the vitals into a verdict ("you
  are not recovered", "you are ready"). It reports each vital's direction; the
  athlete reads the combination.
- **no commands, no verdicts, no shame** — no imperative addressed to the
  athlete, no "good/bad", no red, no "should have".

## What this replaces

| old product | v2 |
|---|---|
| "Take the day, or make it very easy." | "Rest — could be better · 2 of 3 signals below your usual" — and the plan's options, unlinked |
| "Keep it easy today." | "Sleep — 6 h 10 m · could be better · 1 h 10 m less than your usual" |
| "You missed Tuesday's 10 km." | "Tuesday — 10 km planned · not done" |
| Readiness 63/100 | no score exists |

"easy" and "very easy" survive **as the plan's own vocabulary** (session
types), never as commands and never linked to the vitals.
