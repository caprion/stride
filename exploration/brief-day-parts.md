# Brief — the day's four parts: night becomes 21:00–04:00, and the labels say so

*Written by the parent thread, 24 November 2026. **Implementation.** Found by
the athlete, on their own history.*

## What the athlete said

> "another sort of a bug is that night time is taken as 23:00 to 6:00. A lot of
> my runs would have started at around 5ish. So everything gets reported to
> night time. We should move night to 21:00 to 4:00 am. And label it that way.
> Adjust others as required."

The athlete is an **early-morning runner**: a run at 05:10 falls into the
night bucket today, because the night runs from 23:00 to 06:00.

## What to change

### The cut points — `crates/stride-core/src/rolling.rs`

The four constants and the partition they define:

| bucket | now | after |
|---|---|---|
| morning | 06:00 | **04:00** |
| afternoon | 12:00 | 12:00 (unchanged) |
| evening | 17:00 | 17:00 (unchanged) |
| night | 23:00 | **21:00** |

The night **wraps around midnight**: with the night starting at 21:00, the
next bucket after it is the morning, so the morning must start at **04:00** —
that is what makes the night 21:00–04:00, seven hours, and the morning
04:00–12:00, eight. The `DaytimeCuts::end_hour` rule already takes the next
bucket's start, so the partition follows from the constants; **check that the
wrap still works** (an hour of 23:00 must be night, an hour of 03:00 must be
night, an hour of 04:00 must be morning, an hour of 21:00 must be night) and
say so in a test.

### The labels — and this is the part the athlete asked for twice

**"label it that way"** — the label must carry the window, so a reader knows
what "night" means:

* the core's catalogue (`crates/stride-core/src/strings.rs`) — the four
  `daytime.*` labels become, for example, **"Morning (04:00–12:00)"**,
  **"Afternoon (12:00–17:00)"**, **"Evening (17:00–21:00)"**,
  **"Night (21:00–04:00)"**. The hours are the product's own cut points, so
  the label must be **rendered from the constants**, not typed into the
  template — otherwise the label and the partition can drift, which is the
  bug in a new coat. If the catalogue's format cannot take the hours, say so
  and propose the shape that can.
* `strings.json` is **generated**, never hand-edited.
* the **frontend's catalogue** (`frontend/src/i18n/catalogue.ts`) is
  CI-checked against the core's — it must carry the same labels.

### Everything else that names the old window

Search the whole repository for the old boundary and fix what is a **fact**
about the product:

* the doc comments in `rolling.rs` and anywhere else that says "23:00" or
  "06:00" or "the night may wrap around midnight" — they must describe the
  partition as it now is;
* the **tests** in the core and the api that assert the old hours
  (`rolling.rs` asserts `split[0].from_hour == 6` and `split[3].from_hour == 23`;
  the api's test asserts `from_hour == 23`) — they are asserting the product's
  partition, so they change with it;
* the **fixtures** regenerate from the router;
* the **frontend**: the schedule card draws the four buckets with their hours
  (`from_hour` / `to_hour` travel in the answer) — check that the card shows
  the window it now means, and that the labels it prints are the catalogue's.

**And check what else depends on the night boundary** — search for any place
that buckets a *sleep* record, a *wellness* day, or a run by hour: the
attribution of a night's sleep to a day, the run's "night before it", the
Body page's sleep window. The **calendar day** must not move: a 05:10 run
still belongs to that calendar day for the load, the ledger and the plan. Only
the **four parts of a day** change. If something *else* is bucketed by hour
and the athlete's complaint reaches it, say so and fix it the same way — and
tell the parent what you found.

### The documentation

* `PLAN.md` — a **note** (the file's convention for a change that touches a
  frozen file or a stated fact): the day's four parts, the hours, and the
  reason — the athlete's own runs start around five, so a night that begins
  at 23:00 reported them as night.
* `exploration/decisions.md` — add the decision (D26) so the record holds it.

## Rules

* **Facts, never verdicts**; the ban list and the closed direction set are
  untouched.
* **The vocabulary** (D24): the labels are plain words plus the hours.
* **The gates**: fmt, clippy `-D warnings`, `cargo test --workspace`,
  check-deps, the contract greps, the secret and privacy scans and their
  self-tests, `tokens:check`, `lint:voice`, typecheck, unit, viewport. **A
  gate that did not run is a gate that did not pass.**
* **Commit on `rig/v2`**, one commit for the partition and its labels, one for
  the documentation — and say which is which.

## Report back (last message, under ~350 words)

* the four constants before and after, and the test that proves the wrap;
* **the labels**, exactly as they will read on the schedule card;
* every other place you found that named the old window, and what you did
  with it;
* anything else bucketed by hour that the athlete's complaint reaches;
* the commits and the gates each passed.
