Parent: a **bigger cause** than the boundary, found while checking the
athlete's own numbers — and the boundary change is still wanted.

## What the review found on the live rig

The athlete's schedule card:

```
Morning 6 -> 12   9 sessions   75.8 km
Afternoon 12 -> 17  3 sessions  15.3 km
Evening 17 -> 23    0 sessions   0 km
Night 23 -> 6      93 sessions  884.6 km
```

Ninety-three sessions in "night" and none in "evening" is not a boundary
problem. **The hours are in UTC.**

## The cause

`crates/stride-api/src/state.rs` holds `utc_offset_minutes`, and its default
is **0**. The **phone** sets it — `crates/stride-api/src/ffi.rs` exposes
`set_utc_offset_minutes` and the Android app calls it with the device's
offset — but **the desktop host never sets it**. So on the desktop rig every
"local hour" is the UTC hour.

The athlete is in IST (+5:30). A run at **05:00 IST** is **23:30 UTC** — and
23:30 falls in the night bucket under *any* boundary. That is the whole of
the complaint: *night* was not a boundary that was wrong, it was an hour that
was wrong. Moving the night to 21:00 would have left the symptom in place.

**And the days are right while the hours are wrong** — the calendar day comes
from Garmin's own local date, so the load, the ledger and the plan all bucket
the run correctly; only the *part of the day* was computed in the wrong zone.
That inconsistency is why the athlete's own history looked impossible.

## What to do

1. **The desktop host sets the offset.** At startup, from the **host
   machine's own timezone** (`chrono::Local`'s offset), and — as the phone
   does — **refresh it** rather than fixing it once: read it when the rig
   starts and when the sync runs, so a machine that changes zone (or a
   daylight-saving change) does not leave the hours behind. The athlete may
   travel; the offset must follow the machine.
2. **An override, if the machine's zone is not the athlete's.** A setting in
   **Settings › Training** (beside the heart rates) where the athlete can
   state their offset, with the machine's value as the default and the fact
   that the app is using it stated on the screen. It is a fact about the
   machine, not a verdict — and it must say **which** is in force.
3. **Keep the athlete's boundary change** — night **21:00**, morning
   **04:00**, afternoon 12:00, evening 17:00 — as they asked. With the
   offset fixed, a 05:00 run is a **morning** run under both the old and the
   new boundaries; the change is still right on its own terms (a 21:30 run
   is an evening run, not a night one), and it is theirs.
4. **The labels carry the window**, as the brief said: "Morning (04:00–12:00)",
   "Afternoon (12:00–17:00)", "Evening (17:00–21:00)", "Night
   (21:00–04:00)" — **rendered from the cut points**, never typed.
5. **Add a test that would have caught this**: a run whose local start time is
   05:00 in a +5:30 zone must land in the **morning** bucket, and the same
   instant in UTC must not. A test at offset 0 that passes for the wrong
   reason is what let this through.

## Also

* `crates/stride-api/src/update.rs` and `crates/stride-api/src/routes.rs`
  read the same offset — check what they use it for (the run update's
  timestamps, and the day's own answer), and say whether the wrong offset
  reached anything else.
* **`PLAN.md`** — a note for the change: the offset is the host's (or the
  athlete's), the four parts are the athlete's own hours, and the labels say
  which window they mean.
* The athlete's own history was **wrong on the desktop rig** since it began:
  say in your report how many answers it touched (the schedule card, the run
  update's timestamps, anything else) — the athlete deserves to know the
  scale of it.

## Rules

* The gates, per commit, as before: fmt, clippy `-D warnings`, the workspace
  tests, check-deps, the contract greps, the secret and privacy scans and
  their self-tests, `tokens:check`, `lint:voice`, typecheck, unit, viewport.
* **Commit on `rig/v2`**.
* Do not touch `crates/stride-core/src/stimulus.rs` or
  `crates/stride-api/src/routes/stimulus.rs` — the ledger thread's.

## Report back

* where the offset is now set, and how it is kept fresh;
* the override, and how the screen says which offset is in force;
* the test that would have caught it, and the boundary change beside it;
* **what the athlete's own numbers become** with the offset fixed — the
  daytime split for the same window, before and after;
* everything else the wrong offset reached.
