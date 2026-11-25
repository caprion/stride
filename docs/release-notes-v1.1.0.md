# Stride 1.1.0

This is the Android release of the current build. The phone and the desktop run
**the same application, the same Rust core and the same router**: what the rig
shows on a wide screen and what the phone shows in the hand are two widths of
one product, not two products.

It keeps package `app.stride.running` and the signing certificate the earlier
releases were signed with, so it **installs over 1.0.0** and the athlete's saved
runs, health history, plans and notes stay where they are.

**Back up important history before upgrading.** Install over the existing app;
do not uninstall to work around an upgrade error, because uninstalling removes
local history. The [v1.0.0 notes](https://github.com/caprion/stride/blob/v1.1.0/docs/release-notes-v1.0.0.md)
describe what the earlier release carried.

## What is included

- **The wall of eleven cards.** Progress carries six questions about the work —
  this week, the load, the weeks, where the time went, the pace and the bests —
  and the athlete's profile carries five about the long pattern: the shape, the
  months, the schedule, the habit and the goals. Every card is one question, and
  the panel behind it holds the answer.
- **One chart on the run.** The run detail draws a single chart over the run's
  own elapsed time. Pace and heart rate share the cursor, and the table beneath
  it keeps the effort, the splits and the line that says where the numbers came
  from.
- **The ledger, in two bases.** The time in zones has two measurements and the
  card draws both, side by side, and never adds them up: the time in the
  athlete's own bands over the time they were running, and the watch's own zone
  minutes over the whole run. Under them sits the overlap — the stretch where
  both bases exist, side by side, with the distance between them.
- **The ledger's absence rows.** Absence is not zero. A band that held no time
  is white space, and the hole is the finding; a whole block with no value says
  why, in the app's own words, and shows no number at all.
- **The training paces, and the cost curve behind them.** The ladder draws the
  anchors and the rungs — each with its pace, its heart-rate band and the rule
  it came from — with the pace axis running fast on the left, the way a runner
  reads a pace. The curve is a second reading of the same athlete: the share of
  the maximum against the time it is held for, with the athlete's own races, the
  method's sessions and the field test's marker on it, and the pace equivalents
  along the bottom.
- **The threshold's four candidates.** The watch's own zone model (a heart
  rate), a race (a pace), a field test (a heart rate and a pace), and the number
  the athlete brought. The app computes the candidates and the athlete chooses
  one; the choice is a dated fact with its source, and the choices that came
  before it stay in the history.
- **The adopted programmes.** Four programmes are offered — the two 10K tiers and
  the two half-marathon tiers — and the athlete picks. A programme is an intent
  and not a baseline: the app never suggests one. The tier's entry requirement
  and the athlete's own mileage stand side by side, unranked. The paces are
  worked out once, at the adoption, from the threshold in force that day, and
  they are stored with the plan.
- **The week against the plan.** The plan's week is read from the adopted copy,
  and the card carries what happened on the day of each planned session: the
  distance run against the distance planned, and the pace against the plan's
  pace when both of them exist. A session with no pace is compared on its
  distance or on nothing, and never against a pace that is not there. Done and
  not done are facts: there is no grade, no percentage, and the word "missed" is
  not in the app.
- **The zone times in two bases** — as above, and never merged into one number.
- **The day's four parts, in the athlete's own hours.** The morning starts at
  04:00, the afternoon at 12:00, the evening at 17:00 and the night at 21:00;
  the night wraps around midnight, so it runs from 21:00 to 04:00. Each part
  names the window it covers — "Morning (04:00–12:00)", "Night (21:00–04:00)" —
  and the hours are read in the machine's own zone. The athlete may state their
  own offset in Settings › Training, and the screen says which of the two is in
  force. The calendar day itself does not move: a run at 05:10 still belongs to
  that day.
- **The athlete's own heart rates.** Settings › Training › Your heart rates
  takes the maximum heart rate, the resting heart rate, or both, each with where
  it came from and, when they know it, the day it was measured. The card shows
  the declaration in force, the bands it produced, the watch's own floors beside
  them, and the difference stated boundary by boundary — a fact, and never a
  correction. The earlier declarations stay in a dated history and are never
  deleted.
- **The older history, from the phone.** Data & connection now carries **Check
  older history** on the phone as well as on the desktop: the dated,
  checkpointed walk back through the months the phone has not checked yet. The
  window is bounded, the job runs in the phone's own background, and the range
  that has been checked is shown as it grows.

## Upgrade and privacy

**The schema moves to version 8, and the migration is in place.** The store
brings an existing database up to the current version the moment it is opened —
each step in its own transaction, and it is idempotent, so opening it twice
changes nothing. The upgrade from 1.0.0 adds the new tables: the watch's zone
model, the wellness observations, the run update and its settings, the athlete's
race, threshold and run observations, the adopted plan and its dated sessions,
and the athlete's own declared heart rates.

**There is no reset and no lost history.** The runs, the health history, the
plans, the notes and the sync cursor that were there before the upgrade are
there after it. Nothing is re-downloaded because of the upgrade, and nothing is
deleted to make room for the new tables.

**The data stays on the device.** The database is the phone's own encrypted
file, the Garmin key is unwrapped from the phone's own key store for the length
of a pull and dropped again, and the app talks to no server. Nothing in this
release leaves the phone except what the athlete asks for: the readable exports
they choose to save, and the tile requests the optional Street map makes, which
carry their own disclosure.

**Two things are now the athlete's to enter.** The **heart rates** are entered
in Settings › Training › Your heart rates, on the phone and on the desktop, and
until they are entered the app's bands are the watch's own and not the athlete's.
A **race** — the anchor the race paces are worked out from — is typed on the
desktop rig's own page; the phone's app has no screen of a race, so a race
pacing needs the rig. A plan may be adopted before a race is typed: its sessions
then carry no paces until the threshold that paced them is known.

## Known limits

- **The repeated-work row needs a workout the watch recorded.** It is the
  watch's own statement about its own session — two or more of the run's steps
  named `interval` — and the app adds no threshold of its own. A run the watch
  recorded without workout steps leaves the row empty, and the row says so:
  "not recorded — the runs in this period carry no workout steps".
- **The 80/20 base is a specification, not yet a second programme.** Its
  schedule has not been read cell by cell, so there is no 80/20 template in the
  catalogue: the picker holds the four programmes and nothing else. The app
  holds one base at a time.
- **The app's bands are the athlete's declared heart rates, and they may differ
  from the watch's own configuration.** Where the watch's zone model has been
  read, the card draws both sets of floors and states the difference boundary by
  boundary. It never says which of the two is right, and a run before the first
  stored model is not calibrated: the watch's own zone minutes are the only zone
  time there is.
- **The condition axis is not built.** Nothing here reads a run's effort against
  the weather or the temperature, and no such measure is implied by any number
  in the app.
- **The ledger reads a bounded number of runs.** One answer carries at most 500
  runs, and a period with more history than that is read from the most recent
  of them. The card says when there were more.
- **The history import is bounded and resumable.** The twelve-month target is
  what the phone walks back towards, one dated window at a time, and it is not
  proof that every requested record has arrived. Missing wellness observations
  stay missing; the coverage the phone has checked is shown, and the check can
  be asked for again.
- **The race is a desktop act.** The race is typed on the rig's own page, by
  decision, and the phone's app carries no screen for it.
