# Security policy

Stride is local-first. Android runs the Rust core in-process; desktop serves it
over local HTTP. There is no Stride account or hosted Stride data store.
Garmin sync and explicitly enabled OpenStreetMap tiles are external network
services. This document describes the host boundaries and private reporting.

## The model

| what | how |
|---|---|
| **Your Garmin key** | Typed into a native screen on Android, wrapped by a non-exportable Keystore key, stored in `noBackupFilesDir`. It never enters the WebView, never enters the database, and is never returned by an endpoint. |
| **Desktop Garmin key** | Setup stores an accepted key in the host's private `garmin.key` file, mode `0600` on POSIX. An environment-injected key is process-scoped instead. Neither is returned by an endpoint or logged. |
| **Your data** | In a SQLCipher database. Android wraps its database key with Keystore; desktop persists its separate live `db.key` in the private data directory. Neither key is returned by an API or logged. |
| **Recovery backups** | Explicit native export to an Android document destination, encrypted with a passphrase-derived key independent of the original phone. Stride does not save that passphrase. The ZIP metadata and manifest are readable; the database is encrypted. See `android/README.md` for the format and acceptance boundaries. |
| **Desktop export bundles** | The encrypted database travels beside its export-only `db.key`, plus a manifest. Anyone with the complete folder can open it. Keep the whole directory private; it is not the native passphrase-archive format. |
| **Readable exports** | Explicit dated CSV/JSON delivery. These files are not encrypted and are not recovery backups. The destination is chosen by the user; routes, names, notes and credentials are excluded. |
| **The first-run conversion** | The desktop's first-run migration encrypts a plaintext database in place and **leaves the plaintext copy beside it**, at `<database>.plaintext-backup`. The file was plaintext before, and the copy is the recovery path if the conversion is wrong. It is the user's file: delete it once the encrypted database opens. |
| **The desktop rig** | Loopback is always allowed. Additional networks are opt-in via `STRIDE_TRUSTED_NETWORKS` (a CIDR list). Everything else is refused. There are **no pairing codes, no QR codes, no device tokens** — deliberately. |

Relevant threats include network callers, access to an unlocked device or its
private files, and disclosure through exports or external map requests. Garmin
credentials and tokens still need protection even though Stride has no account.
The local outline does not request tiles; choosing Street map discloses the area
being viewed to OpenStreetMap.

**Development credentials never belong in source, shell history or transcripts.**
Use synthetic fixtures or a runtime secret-manager injection; see
[CONTRIBUTING.md](CONTRIBUTING.md#credentials-for-development). Public
demonstration media uses fabricated observations, not an athlete's history.

## Properties the implementation must preserve

- Android Garmin credentials do not enter the WebView or database.
- No endpoint or log discloses a credential or live database key.
- Untrusted network callers cannot read the API.
- File operations cannot escape their intended directories.
- Diagnostics do not disclose secrets or an athlete's notes.

Reports of violations are welcome through the private channel below.

## Reporting a vulnerability

**Do not open a public issue.** Use GitHub's private reporting:

> <https://github.com/caprion/stride/security/advisories/new>

Please include the request or the code path that reaches it, the version or commit, and what an attacker
gains. Reports are reviewed privately, without a response-time guarantee.
We will credit you in an advisory unless you would rather we did not.

If private reporting is not available to you, open an issue that says only "security report available on
request" with no details.

## Supported versions

The project is pre-1.0 and moving quickly. Security fixes land on `master` and in the latest release.
There is no long-term support branch; there is one product, and it is maintained by one person.

## The gates

The repository's CI runs, on every pull request:

- the Rust test suite, with `unwrap`, `expect` and `panic` denied in library code;
- **corpus integrity** — the frozen FIT files match their recorded SHA-256, and the privacy gate
  decodes every FIT position, and the corpus's SHA-256s are checked;
- a **secret scan** — no key, token, keystore or database file may be tracked or printed;
- a **privacy scan** — no real coordinates, athlete names, device names or notes in tracked files.

The **viewport matrix** — the UI checks at six widths, in a real browser — is a gate.

Release-only: the check that the published APK carries the Rust core it was built with, and that it is
signed with the certificate published in `android/assetlinks.json`. Both exist — they are `scripts/check_apk.py`
and the release workflow — and apply to the current signed candidate as well. The live check against the trust model is the
rig's own test (`desktop/tests/`), not a release step.
