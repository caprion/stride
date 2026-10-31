# Security policy

Stride is local-first. There is no server, no account, and no cloud service — so the security model is
small, explicit, and auditable. This document says what it is, and how to report a problem.

## The model

| what | how |
|---|---|
| **Your Garmin key** | Typed into a native screen on Android, wrapped by a non-exportable Keystore key, stored in `noBackupFilesDir`. It never enters the WebView, never enters the database, and is never returned by an endpoint. |
| **Your data** | In a SQLCipher database. The key is supplied by the caller, never logged, never persisted to a state file, never returned by an API. |
| **The desktop rig** | Loopback is always allowed. Additional networks are opt-in via `STRIDE_TRUSTED_NETWORKS` (a CIDR list). Everything else is refused. There are **no pairing codes, no QR codes, no device tokens** — deliberately. |
| **The harness / MCP tools** | Read-only, bounded output, and every answer is a disclosure. Only the fields an athlete would want in a model's context are returned: no credentials, no file paths, no notes. |

The threat model is therefore: *someone on your network, or someone with your unlocked phone*. There is no
account to phish and no session to steal, because there is no account and no session.

## What we will not accept

- Any report that requires the Garmin key to leave the Keystore or the database.
- Any report that requires the database key to be logged, returned, or written to disk in the clear.
- Any report that lets a caller on an untrusted network read the API.
- Any report that lets a path traversal escape the intended directory.
- Any report that puts a secret, a file path, or an athlete's notes into a tool's output.

## Reporting a vulnerability

**Do not open a public issue.** Use GitHub's private reporting:

> <https://github.com/caprion/stride-v2/security/advisories/new>

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
- **golden parity** against the frozen corpus;
- a **secret scan** — no key, token, keystore or database file may be tracked or printed;
- a **privacy scan** — no real coordinates, athlete names, device names or notes in tracked files;
- the **viewport matrix** — the UI checks at six widths.

Release-only: the check that the published APK carries the Rust core it was built with, and the live
security check against the trust model.
