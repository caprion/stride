# Stride 0.12.0-rc.3

This signed Android testing pre-release retains the existing app identity and
signing certificate. Back up important history before upgrading. Do not uninstall
the existing app to work around an upgrade error: uninstalling removes local
history. Stable remains v0.11.0.

This update addresses the desktop/mobile interaction gaps found after rc.2.
The layouts use the same compact Azure/amber design and shared navigation.

## What changed

- Today puts the named latest run and weekly running history before wellness on
  compact screens. Each weekday opens its saved runs. Body and Plan have direct
  entry points, including when no plan has been imported.
- Runs uses compact rows, real local route previews and a monthly calendar.
  Inclusive date filters now search stored history, not just loaded pages.
  Text search remains explicitly scoped to loaded results. FIT import stays
  available in a disclosure, and successful imports refresh cached screens.
- Run detail uses the activity's actual name, opens splits and elevation, and
  keeps the selected history window when returning. Splits show core-rendered
  pace. Heart-rate time bars disclose that their bands use that run's maximum,
  not personal training zones. Sleep follows run analysis and notes.
- Body shares date selection through its URL, offers seven clickable dates with
  previous/next week navigation, and links to runs on that date. Changing dates
  preserves the chosen history window.
- Progress restores the wide two-column layout and shows selected-period
  distance, moving time, sessions and weeks with saved activity. Weekly bars and
  a week selector open the corresponding stored history.
- Compare restores desktop columns, aligns its metric table and renders visible
  chart lines after compact A/B switches. Both runs can be explored by heart
  rate or pace. Links back to runs retain the selected history window.
- Data & connection separates storage, training preferences, import and About.
  Desktop backup opens the working host-owned backup/restore tools. Desktop
  connection and backup pages use the shared Azure styling. Native passphrase
  handling and document delivery stay in the Android host.
- The checked-in desktop bundle is refreshed too, so a clean Rust-only desktop
  build does not fall back to the older screens.

## Data and privacy

There is no database reset, storage migration or change to Garmin credentials.
Feed route previews use at most 64 points decoded locally from saved FIT files.
The detailed route remains separately bounded. Local outlines make no tile
requests; the optional Street map action retains its external-tile disclosure
and local fallback.

Zero saved runs is not proof of a rest day or complete Garmin coverage. Missing
wellness observations remain missing. Historical import is still bounded and
resumable; this release does not establish that the entire requested history has
arrived. No new readiness score or matched-effort fitness claim is introduced.

## Acceptance boundary

The shared UI was checked against local desktop history at desktop and compact
widths, including linked date/history journeys, actual route previews, Compare
chart lines, selected-period totals and failed street tiles. The automated
release gate covers the Rust workspace, frontend and responsive viewport matrix.

Physical-phone upgrade/startup, background sync, map gestures, locking,
document-provider delivery and cross-phone recovery still require device
acceptance. This remains a testing pre-release. It does not publish a PWA or
replace the stable release.
