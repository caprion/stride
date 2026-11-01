# Security policy

Stride is local-first. There is no server, no account, and no cloud service — so the security model is
small, explicit, and auditable. This document says what it is, and how to report a problem.

## The model

| what | how |
|---|---|
| **Your Garmin key** | Typed into a native screen on Android, wrapped by a non-exportable Keystore key, stored in `noBackupFilesDir`. It never enters the WebView, never enters the database, and is never returned by an endpoint. |
| **Your data** | In a SQLCipher database. The key is supplied by the caller, never logged, never persisted to a state file, never returned by an API. |
| **The first-run conversion** | The desktop's first-run migration encrypts a plaintext database in place and **leaves the plaintext copy beside it**, at `<database>.plaintext-backup`. The file was plaintext before, and the copy is the recovery path if the conversion is wrong. It is the user's file: delete it once the encrypted database opens. |
| **The desktop rig** | Loopback is always allowed. Additional networks are opt-in via `STRIDE_TRUSTED_NETWORKS` (a CIDR list). Everything else is refused. There are **no pairing codes, no QR codes, no device tokens** — deliberately. |

The threat model is therefore: *someone on your network, or someone with your unlocked phone*. There is no
account to phish and no session to steal, because there is no account and no session.

**During development, the key is not written down.** It is fetched from Azure Key Vault at runtime through
the machine's managed identity, held in one process's memory, and discarded with the process — see
[CONTRIBUTING.md](CONTRIBUTING.md#credentials-for-development). The scanner is the backstop, not the
control.

## What we will not accept

- Any report that requires the Garmin key to leave the Keystore or the database.
- Any report that requires the database key to be logged, returned, or written to disk in the clear.
- Any report that lets a caller on an untrusted network read the API.
- Any report that lets a path traversal escape the intended directory.
- Any report that puts a secret, a file path, or an athlete's notes into a tool's output.

## Reporting a vulnerability

**Do not open a public issue.** Use GitHub's private reporting:

> <https://github.com/caprion/stride/security/advisories/new>

Please include the request or the code path that reaches it, the version or commit, and what an attacker
gains. We will acknowledge within a few days, and we will credit you in the advisory unless you would
rather we did not.

If private reporting is not available to you, open an issue that says only "security report available on
request" with no details.

## Supported versions

The project is pre-1.0 and moving quickly. Security fixes land on `master` and in the latest release.
There is no long-term support branch; there is one product, and it is maintained by one person.

## The gates

The repository's CI runs, on every pull request:

- the Rust test suite, with `unwrap`, `expect` and `panic` denied in library code;
- **corpus integrity** — the frozen FIT files match their recorded SHA-256, and the privacy gate
  decodes every FIT position (the full golden-parity differ arrives with Slice 2);
- a **secret scan** — no key, token, keystore or database file may be tracked or printed;
- a **privacy scan** — no real coordinates, athlete names, device names or notes in tracked files.

The **viewport matrix** (six-width UI checks) lands with Slice 9; it is not a gate yet.

Release-only, and landing with Slice 10 (the Android pipeline): the check that the published APK carries the
Rust core it was built with, and the live security check against the trust model. Neither exists yet — the
release workflow is a placeholder that runs the gate and refuses a tag off `master`. This paragraph says so, so
that no reader is told about a gate that is not there.
