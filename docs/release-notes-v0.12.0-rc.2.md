# Stride 0.12.0-rc.2

This signed Android testing pre-release keeps the existing app identity and
signing certificate. It can upgrade the previous installation. Back up important
history before testing; do not uninstall the existing app to resolve an upgrade
error, because uninstalling removes its local data. Stable remains v0.11.0.

The shared phone and desktop UI adopts a compact Azure/amber layout informed by
the Stitch concepts, using recorded observations rather than the prototype's
invented readings or capability claims.

## What changed

- Body shows the selected night's deep, REM, light and awake durations inline.
  History charts have dated axes, recorded-value tooltips and date selection.
  Today and run detail reuse the sleep breakdown; a run uses its own date.
  Missing stages stay missing. A stage-composition bar is not a chronological
  sleep timeline, a quality score or a clinical assessment.
- Runs supports search, date filters and explicit two-run selection for Compare.
  Filters apply to loaded pages, with Load more for older saved activities.
  Detail keeps splits, zones, notes and export, with a shared elapsed-time cursor
  for pace and heart rate. Elevation remains distance-based and separate.
- Full GPS geometry is recovered from stored FIT files, rather than only the
  first and last three positions. Run detail offers an optional OpenStreetMap
  street map with pan and zoom. Opening Street map reveals the viewed area to
  the tile provider. The default local GPS outline makes no tile requests and
  remains available if street tiles fail. No route-to-chart position matching
  is claimed.
- Progress leads with saved activity history instead of the unconditional
  "not enough data yet" headline. It distinguishes the unfinished current week,
  retains gaps and explains that weekly pace/heart-rate medians do not establish
  a matched-effort fitness trend. The sessions-per-week figure is rendered and
  explicitly scoped to weeks containing recorded sessions.
- Data & connection leads with local storage and Garmin's actual connection
  state. Connect to Garmin appears when disconnected; an unavailable status
  check is not reported as disconnected. Refresh uses the existing host sync
  path. On Android it requests background work, not an immediate successful
  sync. Technical connection and coverage details are secondary. Native
  passphrase-based backup, restore and readable export remain available.
- Failed feed/detail/comparison/plan requests now have visible retry states
  instead of empty data or permanent loading. Failed settings saves retain the
  entered changes and never announce success. Empty inferred plans no longer
  display an active plan with meaningless zero-of-zero adherence.
- Old numeric activity-sync cursors are accepted without losing their identity;
  malformed cursors remain explicit errors. This fixes a reproduced desktop
  startup/status failure, not a verified diagnosis of every phone startup issue.

## What remains unchanged

Storage, Garmin credentials and the Rust core stay on the phone. No cloud
fitness-data store, PWA migration, new scoring model or Garmin future-workout
access is introduced. Existing encrypted history and portable backup formats
remain in place; no database reset or migration is required by these changes.

The initial twelve-month history import remains bounded and resumable.
Historical coverage is not guaranteed complete, and a pending checkpoint does
not mean work is actively running. Android continuation still depends on its
network, auto-sync, battery and unlock conditions.

## Phone checks still required

Physical-phone startup after upgrade, background sync, map gestures, locking,
document-provider delivery and cross-phone recovery require device acceptance.
The release checks do not substitute for those checks. This build remains a
pre-release and does not replace the stable release.
