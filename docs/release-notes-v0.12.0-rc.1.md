# Stride 0.12.0-rc.1

This is a signed Android pre-release for phone testing, not a production-accepted
release. It retains the existing app identity and signing certificate. Back up
important history before testing an upgrade. Do not uninstall the existing app
to work around an installation error: uninstalling removes its local data.

Azure light is the default across the app and native setup/data screens. Today
shows recorded activity across the week. Body explores selected dates and
7/28/90/365-day calendar windows; charts keep gaps rather than joining missing
measurements. Absent signals stay in collapsed disclosure, while available
history, activity heart rate and routes remain visible. Progress includes a
52-week window. The native calendar follows the phone's current timezone offset.

Initial history targets approximately twelve months through bounded, resumable
jobs. Activity and wellness checkpoints are independent. Partial failures remain
visible; incomplete scans do not authorize deletion. Mobile data is allowed.
Older history can be requested without restarting completed work.

Settings opens native encrypted backup, restore and readable-export controls.
A backup uses a user-chosen passphrase independent of the original phone's
Keystore. Stride does not save that passphrase, and cannot recover a backup
without it. The Android document picker chooses the destination. Restore merges:
matching records can update, and unmatched records remain. Garmin credentials
are not transferred. CSV/JSON exports are unencrypted numeric observations with
explicit units, not recovery backups; they exclude names, coordinates and notes.

Existing run stories, split explanations, computed zones and Compare are retained.
No Garmin future-plan access is claimed; typed or imported plans remain separate.
No live Garmin sync is performed as part of release verification.

## Phone checks still required

Physical-device upgrade, document-provider delivery, restore on a second phone,
locking during transfer and process-death behavior have not been accepted.
Interrupted restore/readable export asks for a restart rather than saving
passphrases or plaintext contents. Pending encrypted archives can resume through
activity recreation. Orphaned encrypted staging older than one day is cleaned
when the data screen opens; active and pending transfers are preserved.

Keep this build on the pre-release track until those checks pass. The previous
stable release remains available.
