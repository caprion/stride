# Stride 1.0.0

This is the stable release of the application published as v0.12.0-rc.3.
It starts the v1.0 release line without new features or changes to stored data.
The APK is rebuilt with version name `1.0.0` and a higher Android version code,
not just renamed. It retains package `app.stride.running` and the original
signing certificate, so it can upgrade existing stable and rc.3 installations.

Back up important history before upgrading. Install over the existing app;
do not uninstall to work around an upgrade error, because uninstalling removes
local history. The [v0.11.0 migration notes](https://github.com/caprion/stride/blob/v1.0.0/docs/release-notes-v0.11.0.md) still
describe the one-time importer for older tables and its limits.

## What is included

- The compact Azure/amber interface shared by Android and desktop: Today, Runs,
  run detail, Body, Progress, Compare, Plan and Settings.
- Linked dates and history windows, a monthly run calendar, inclusive stored
  history filters, selected-period totals and paired run comparisons.
- Local FIT route outlines, pace/heart-rate exploration, splits and elevation.
  The explicit Street map option loads OpenStreetMap tiles with attribution;
  outlines remain available offline without tile requests.
- Bounded, resumable Garmin activity and wellness history import, the legacy
  numeric activity-cursor compatibility fix, and explicit missing-data states.
- Native passphrase-encrypted portable recovery backups, separate readable
  exports and local encrypted storage.

The [rc.1](https://github.com/caprion/stride/blob/v1.0.0/docs/release-notes-v0.12.0-rc.1.md),
[rc.2](https://github.com/caprion/stride/blob/v1.0.0/docs/release-notes-v0.12.0-rc.2.md) and
[rc.3](https://github.com/caprion/stride/blob/v1.0.0/docs/release-notes-v0.12.0-rc.3.md) notes retain the development history.
The [52-second desktop demo and screenshots](https://github.com/caprion/stride/blob/v1.0.0/docs/media/README.md) show this UI
using entirely invented HR, HRV, sleep and GPS observations.

## Upgrade and privacy

There is no new database reset, storage migration or Garmin credential change
relative to rc.3. Saved data stays local; Garmin sync needs network access.
Selecting Street map reveals the viewed area to the tile provider.

## Known limits

Stable publication does not establish physical-phone acceptance.
Upgrade/startup on the affected phone, background sync, locking, map gestures,
document-provider delivery and cross-phone recovery remain unverified on
physical devices. Back up before installing.

History import is bounded and resumable. Its 365-day target is not proof that
every requested record has arrived. Missing wellness observations remain
missing; no new readiness score or matched-effort fitness claim is introduced.

This release is the Android APK with the existing local desktop host.
It does not publish an installable PWA or deploy a hosted connector.
