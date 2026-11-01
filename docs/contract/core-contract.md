# Stride v2 — the core contract (units, rounding, no value)

FROZEN. Part of the contract freeze (Slice 0). This is adapted from the old
repo's `docs/core-contract.md` (the byte-level spec of the Python reference),
trimmed to what v2 keeps: the **units**, the **rounding**, and the
**representation of "no value"** — plus the two things that make the voice
unambiguous: the **direction band** and the **human rendering** rules. This is
what makes "6 h 10 m" and "14 below your usual" mean exactly one thing.

The old `docs/core-contract.md` remains the oracle for the byte-level FIT
decoder (which messages and fields are read, and the exact arithmetic). Where
this document is silent, the old contract applies; where they differ, this one
wins, because v2 drops the readiness score, the verdicts and the advice that the
old contract specified.

## 1. Units

Everything is stored and computed in these units; conversion happens only at the
display boundary.

| quantity | unit | note |
|---|---|---|
| distance | metres | FIT stores centimetres (scale 100) |
| time | seconds | FIT stores milliseconds |
| speed | metres per second | FIT stores mm/s (scale 1000); pace is derived as `s/km` |
| heart rate | beats per minute | truncated to an integer where the old contract says so |
| cadence | steps per minute | running cadence is halved by the halving rule; the stored value is used for classification |
| elevation | metres | FIT stores `(raw / 5) − 500` (scale 5, offset 500) |
| position | decimal degrees | FIT stores semicircles: `round(semicircles × 180 / 2³¹, 7)` |
| timestamps | seconds since 1989-12-31T00:00:00Z | always UTC; a port must treat them as UTC unconditionally |

Preferences: `speed` before `enhanced_speed`; `altitude` before
`enhanced_altitude`. A field whose value is out of range is absent, not zero:
heart rate outside 0–300, cadence outside 0–300, speed outside 0–150 m/s,
altitude outside −1000–10 000 m, distance outside 0–10 000 000 m.

Sport normalisation: `running`, `trail_running`, `treadmill_running`,
`virtualrun`, `street_running`, `track_running` → `running`; `walking`,
`hiking`, `cycling`, `swimming`, `strength_training` map to themselves; anything
ending in `running` maps to `running`; anything else maps to itself, lower-cased
with spaces replaced by underscores.

## 2. No value

`null` is the only representation of "absent". There is no sentinel value in any
stored or returned structure: a missing heart rate is `null`, never `0`. JSON
output carries no `NaN`, no `Infinity`, and no trailing `.0` on an
integer-valued float.

FIT has its own "no value" sentinels, and the rule is: **a sentinel value is an
absent field, and it is absent before any range check is applied.**

| FIT base type | the invalid value |
|---|---|
| `uint8` / `uint16` / `uint32` | all bits set — `0xFF`, `0xFFFF`, `0xFFFFFFFF` |
| `sint8` / `sint16` / `sint32` | the most positive value — `0x7F`, `0x7FFF`, `0x7FFFFFFF` |
| `string` / `byte` | a zero byte |
| `float32` / `float64` | `NaN` |

A decoder that reads `0xFFFF` as a number turns a missing session average speed
into 65.535 m/s — plausible, wrong, and inside the range. The sentinel rule is
what keeps that out.

## 3. Rounding

A port must round in the same place the reference does, and with **round-half-to-even**
(Python `round`; a port that rounds halves away from zero differs on exact ties).

| value | rounding |
|---|---|
| `running_time_s`, `walking_time_s`, `stopped_time_s`, `moving_time_s`, `elapsed_time_s` | 1 decimal |
| `running_distance_m` | 1 decimal |
| lap `running_distance_m` | 2 decimals |
| lap `avg_cadence`, `max_cadence` | 1 decimal |
| `pace_s_per_km` (and any derived pace) | 2 decimals |
| GPS degrees | 7 decimals |
| heart rate | truncated to an integer |
| distance and pace sums reported in Progress (`km` totals, `delta_km`) | 2 decimals |

Dropped from the old contract: `readiness.score`, `adherence_score`,
`overall_pct`, `adherence_pct`. v2 has no readiness score and no adherence
percentage — adherence is a count ("7 of 8 sessions run"), never a percentage.

Floating-point comparison (the parity differ's tolerance): strings, booleans,
`null` and integers exactly; floats within `1e-9 × max(1, |golden value|)`.

## 4. The direction band

"Usual" is defined once and drawn on every chart so the athlete can disagree
with it.

- **Usual** = the median of the last **30 nights** (the day itself excluded).
- **Band** = median ± **1.5 × MAD** (configurable).
- **A band needs at least 3 nights** behind it. A band drawn from one night is
  a point, and every reading would then be "outside" it. With fewer than
  three, there is no comparison: the row says "not enough data for a
  comparison yet" and shows the raw value alone.
- **Inside the band** → "about usual"; outside → "better than usual" or
  "could be better", depending on which side is the better side.
- **Which side is better is a fact about the measurement**, not an opinion:
  more sleep is better, higher HRV is better, lower resting HR is better.

The direction word comes from a **closed set** — `better than usual` ·
`about usual` · `could be better` — no synonyms, no extensions. Beside it sit
the muted arithmetic ("1 h 10 m less than your usual") and the direction of
travel ("3rd night down", "steady"), which are descriptive, never a verdict.

- **Direction of travel** is shown when there is a direction to show: a streak
  of **at least two nights** in the same direction ("3rd night down", "3rd night
  up"), or "steady" when the value is outside its band and the last two nights
  are equal. The form is the ordinal one for both directions; "up 3 nights" is
  not a form the app produces.
- **The travel line is never shown when there is no streak.** One night is a
  reading, not a direction.

The "Rest" line on Today is a **count + direction**, not a conclusion: it is a
summary of the three vitals ("2 of 3 signals below your usual"), and its
direction word is a pure function of that count, never an editorial judgment.

- **The count names the side the signals are literally on.** A resting heart
  rate on its worse side is *above* its usual, so the sentence says "above"
  there; when the signals do not agree on a side, it says "outside". The
  sentence must be true as written — it is a fact about the data, and a false
  fact is the one thing this app does not ship.

## 5. Human rendering (the voice's numbers)

Values are rounded to what a human would say, never to what a machine would.

- **Duration**: "6 h 10 m", not "6.166 h"; "42 m" under an hour; minutes rounded
  to the nearest minute.
- **Deviation**: "14 below your usual", not "−13.7%" and not "−1.4σ". The value
  is rounded to the nearest whole number in the metric's own scale.
- **No raw σ, no z, no percentage without a baseline**: a near-zero starting
  value yields no percentage, ever — the row says "not enough data for a
  comparison yet" and shows the raw value alone.
- **Distance comparison**: shown only when the gap from the plan's target
  exceeds **10%**, stated neutrally — "12.4 km · 2.4 km more than the 10 km
  target". Within 10% it is absent (10.2 km against 10 km is not news), and
  neither more nor less is good or bad. Symmetric for over and under.
- **Units written out**: `ms`, `bpm`, `h`, `m`, `km`, `s` — no abbreviations the
  athlete has to learn, no icons standing in for words, no colour carrying
  meaning on its own.
- **Effort** is the athlete's own rating, **0–10 (CR10)**, whole numbers — the
  value the old schema already held in `perceived_exertion`. The app never
  produces one; it only carries what the athlete entered.

## 6. Expected differences vs the frozen corpus

The frozen corpus (`fixtures/golden/*`, `fixtures/stride-api/api_golden.json`)
is the **old core's recorded output**, not the product's. It still contains the
fields and words v2 deliberately drops:

- `readiness.score`, `adherence_score`, `overall_pct`, `adherence_pct`,
  `grade`, `recommendation`, `rationale`;
- the verdict vocabulary (`hit`, `missed`, `short`, `substituted`,
  `missed_pace`) and the word `"missed"` wherever the old plan engine wrote it.

These are **expected differences**. The parity differ (Slice 2+) asserts the
surviving numbers match, and treats the dropped fields and the verdict
vocabulary as deliberately absent from v2 — a difference there is not a failure.
The corpus keeps them because it is the oracle's output and its SHA-256 is
frozen (`scripts/check-corpus.sh`); deleting them would break the oracle's
integrity and the reference the differ measures against.

### The vitals engine has a different oracle

The old core's readiness engine cannot be the oracle for the vitals engine.
It used the **mean** of HRV and resting heart rate as its baseline, ad-hoc
normal ranges (sleep ≥ base − 1 h, HRV ≥ base × 0.85, resting HR < base + 5)
and a 0–100 score — all of which v2 drops, deliberately. For the vitals
engine, the oracle is **this contract**: the band, the closed set, the
arithmetic, the rounding and the words. The old corpus is the oracle for the
*ingest* numbers (the FIT readings, the distances, the times) and for the plan
engine's surviving arithmetic — not for the words the athlete reads.

## 7. The retry rule, and who owns it

**The shell owns the retry. The client does not.**

- `stride-ingest` performs **exactly one** token exchange and **one** HTTP
  attempt per call. If the token has expired, the call returns
  `NeedsReconnect` — it does not re-exchange and try again.
- The caller — the phone's sync engine, or the desktop's — may make **one**
  more attempt, and that attempt may perform one fresh token exchange. A
  second failure stops: the caller surfaces the error and the next scheduled
  sync tries again.

One rule, one owner. If the client also retried, the phone would exchange
and retry twice, and the "exactly one" guarantee would be a coincidence
rather than a rule. Both sides assert it: the client test asserts one
exchange per call, and the shell's test asserts one retry per sync.


## 8. One owner for the sentences and the rounding

The domain sentences and the domain numbers have **one owner: the core**.

- **The core renders them.** The direction word, the arithmetic tail, the
  direction of travel, the "Rest" line and the pattern sentences are produced
  by `stride-core` and travel to the UI **already rendered**. The API sends
  the sentence, not a template with holes in it.
- **The frontend renders app chrome only.** Labels, buttons, empty states, the
  chrome around the sentences. It does not re-implement the domain's
  wording, and it does not re-implement the domain's rounding.
- **The rounding belongs to `units.rs`.** Half-even where the contract says
  half-even, truncation for heart rate, the human form for a duration. A
  second implementation of "how do we say 30 seconds" is the same disease as
  a second implementation of the FIT decoder: it will disagree, and it will
  disagree quietly.

`crates/stride-core/strings.json` is the catalogue's export — it carries the
ban list, the closed direction set and the minimum pattern sample **with** the
strings, so the frontend's lint and the core's lint read the same rules. If
the two ever disagree, the contract's list wins and the export is regenerated.

Two hosts that disagree about a number is a bug. Two hosts that disagree
about a *sentence* is the same bug, and it is worse, because a sentence is
what the athlete reads.
