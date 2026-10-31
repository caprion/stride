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
- **Inside the band** → "about usual"; outside → "better than usual" or
  "could be better", depending on which side is the better side.
- **Which side is better is a fact about the measurement**, not an opinion:
  more sleep is better, higher HRV is better, lower resting HR is better.

The direction word comes from a **closed set** — `better than usual` ·
`about usual` · `could be better` — no synonyms, no extensions. Beside it sit
the muted arithmetic ("1 h 10 m less than your usual") and the direction of
travel ("3rd night down", "steady"), which are descriptive, never a verdict.

The "Rest" line on Today is a **count + direction**, not a conclusion: it is a
summary of the three vitals ("2 of 3 signals below your usual"), and its
direction word is a pure function of that count, never an editorial judgment.

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
