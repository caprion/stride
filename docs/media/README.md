# Stride screenshots and demo

These are captures of the actual **v0.12.0-rc.3 desktop application**, including
its compact layout. They are not Stitch renders or screenshots of personal
history. Android uses the same frontend, but browser captures do not establish
physical-phone acceptance.

The same UI is promoted to stable in **v1.0.0** without feature changes.
The media retains its original rc.3 capture provenance and asset URLs.

## Short video

[Watch/download the 52-second screen-focused MP4](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-desktop-demo.mp4).
It covers Today, the run calendar, run detail with a loaded OpenStreetMap view,
HR/pace and splits, Body, Progress, Compare, Plan, Settings and a compact layout.
There is no spoken audio or long title sequence.
The [dataset and chapter metadata](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/demo-info.json)
records the capture period, duration and synthetic-data boundaries.

[![Stride run detail with fictional data and OpenStreetMap](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-run.png)](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-desktop-demo.mp4)

## Screens

Select an image to open its full-size release asset.

| Today | Saved runs |
|---|---|
| ![Today with invented weekly activity](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-today.png) | ![Calendar and saved runs](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-runs.png) |

| Body | Progress |
|---|---|
| ![Invented sleep stages, HRV and resting HR](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-body.png) | ![Period totals and weekly history](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-progress.png) |

| Compare | Plan |
|---|---|
| ![Comparing two invented runs](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-compare.png) | ![Explicitly entered fictional plan](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-plan.png) |

| Settings | Compact Today |
|---|---|
| ![Local storage and Garmin connection](https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-settings.png) | <img src="https://github.com/caprion/stride/releases/download/v0.12.0-rc.3/stride-mobile.png" alt="The same Today screen at a compact width" width="260"> |

## Data provenance and map privacy

The isolated demo store contains **23 runs and 39 nights**, on an invented
calendar from 2026-09-28 through 2026-11-05, spanning six calendar weeks.
Distances, times, names, notes, plan, HR, HRV, resting HR and sleep observations
were generated from scratch. No personal database, FIT file, Garmin credential
or browser profile supplied the dataset. No Garmin account was connected.

GPS is an illustrative, mathematically generated loop around a public London
park. It is not a recorded person's route or a claim that the displayed
sessions occurred there. This replaces the earlier ocean-only demo geometry so
the street-map interaction has visible geographic context. It does not change
the synthetic anchor or accepted expectations in the frozen test corpus.

The recording explicitly selects **Street map**. Those tile requests reach
OpenStreetMap; the normal local outline makes no tile requests and remains
available offline. Map imagery is credited to
[OpenStreetMap contributors](https://www.openstreetmap.org/copyright), with
attribution retained in the screenshot and recording.

Media is hosted as assets on the
[rc.3 GitHub release](https://github.com/caprion/stride/releases/tag/v0.12.0-rc.3),
not as large binary blobs in Git. These assets document that version; a media
refresh does not rebuild or replace its signed APK. Original wireframes remain
separately labelled as [historical design material](../plan.html).
