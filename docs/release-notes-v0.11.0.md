# Stride v0.11.0 — what this release carries, and what an upgrade really does

This is the release after the independent review of v0.10.0. The review's verdict was
**hold**, with the phrase *"the bar is 0.8's depth in 0.9's voice"*. This release is the
answer to the six findings that review raised, and it is honest about the two that needed a
decision rather than a patch.

> **If you are upgrading from v0.8.0, read the second half of this note.** The upgrade
> keeps your app, its key and its database file — and it now carries your history across.

---

## What changed

**The pull is a delta now.** It walks Garmin's pages, asks the store before it downloads
anything, and skips a run it already holds — so a known run is never fetched twice. It
re-parses a run whose numbers changed, continues from where it stopped, spends its budget
on the runs it has not pulled yet (a long history fills over successive pulls rather than
never), resumes after an interruption, and marks a run Garmin no longer lists instead of
bringing it back. A run with no file — see below — is fetched exactly once.

**The heart-rate zones are computed.** Five bands of the run's own maximum heart rate,
taken from the samples that carry a heart rate and from nothing else. The time with no
heart rate is counted separately and never folded into a band. The basis is named
("moving" or "elapsed"). When there is nothing to compute from, the answer says so in
words instead of inventing bands.

**Compare shows your runs.** The screen asked the API which runs there are, instead of
knowing two ids in advance. The picker opens; the pair you choose is the pair the table
renders; the swap turns it round and flips every difference.

**No dead controls.** "Choose a different run" works. "Share" makes the file and the host
delivers it. "Export FIT" fetches your run's original file. Each is offered only where the
host can deliver it — a control that promises a thing and does nothing is worse than no
control at all, and there is now a test that holds that.

**The charts are labelled** with the core's own rendering ("148 bpm", "5:24 /km", "38.6
km"). The pace axis used to be raw seconds.

**The wording is informational, not advice.** The card that used to be headed "Rest" is
headed **"Last night"** — it names the source of the numbers and interprets nothing.

**The version is right.** The About card showed 0.1.0 while the release was 0.10.0. There
is one source of truth now, and it is the build.

---

## Upgrading from v0.8.0: what is carried across

The container survives an upgrade: the same package, the same certificate, a higher
version code — so v0.11.0 installs **over** the app you already have, without an
uninstall, and your key stays where it was.

**Your history was not carried across by the install alone.** v0.10.0 opened your old
database on a new schema, and your old rows sat there unread. **v0.11.0 carries them
across.** Here is exactly what, table by table:

| your old table | what happens |
|---|---|
| `activity` | **carried** — the source, the Garmin id, the title, the sport, the start, the date, the distance, the moving and elapsed times, the elevation, the average and maximum heart rate. **The FIT file is fetched by the first pull**, because the old version kept none. |
| the run's `description` | **carried** — one note per run that had one. The old version had no notes table; the description was the only free text on a run, and your words are worth keeping. |
| `daily_physiology` | **carried** — the date, the sleep and its stages, the awake time, the HRV, the resting heart rate, the average stress, the source. |
| `plan_block` | **carried** — the name, the source, the first and last day, the goal, the status, the notes. |
| `plan_session` | **carried** — the block, the date, the kind, the title, the details, the target distance, the pace band. A session with no block is left behind (the new schema says a session belongs to a block). The old `recovery` kind becomes **unclassified**, because this version has no such kind. |
| `activity_stream`, `activity_lap`, `hr_zone`, `run_fingerprint`, `activity_fact` | **not carried** — these are all derived from the FIT, and the FIT is the source of truth. The app re-derives them. |
| `gear`, `readiness`, `daily_metric`, `athlete`, `plan_phase`, `plan_week`, `plan_result`, `sync_run` | **not carried** — there is no gear ledger, no device score and no stored verdict in this version. |
| the old run's remaining columns (calories, cadence, speed, running and walking time, device, gear, perceived exertion, route, raw record, day of week) | **not carried** — derived from the FIT, or the device's opinion of the run. |

**The synthetic rows are not carried.** The previous version seeded demo runs — synthetic
data at your own coordinates — and threw them away at every start-up. They are left
behind and not carried.

**Nothing is moved and nothing is deleted.** The old tables are read, never written, and
they are still in the file afterwards, with their rows. If you would rather start clean,
uninstalling first gives you an empty history and a fresh pull — and Garmin's Personal API
window is a bounded one: a re-sync restores the recent runs and the recent wellness days
it holds, and not the years before that.

---

## The gates

Every pull request runs, off-device:

- the Rust tests, `fmt` and `clippy` (with `unwrap`, `expect` and `panic` denied in library code);
- the app's own gates — the voice lint, the type check, the unit tests and the six-width viewport matrix;
- the corpus's integrity (every FIT file's SHA-256) and the privacy gate (every FIT file's positions, decoded);
- the contract greps (no pairing route, no string-built SQL, no `unsafe` outside the seam);
- the secret and privacy scans, the pull-request scanner, and the **history scan** — every commit on every ref, every added line, the same rules;
- the APK checks: the Rust core is inside it, it points at nothing, and it is signed with the published certificate.

## Known limits, stated plainly

- **The rig and the phone have not been run against a live Garmin account from this
  release** — the sync was proven against a fake transport and the key's whole life was
  proven by sweep and by test, not by a live pull on a device.
- **The Android floor is 15** (the Keystore's `setUnlockedDeviceRequired` wants it).
- **The zones' band names** are "Z1"…"Z5"; a stored maximum heart rate or threshold is not
  used — the maximum is derived from the run's samples.
