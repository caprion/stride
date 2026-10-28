# The desktop

**One binary that serves the React app and the API out of the Rust core.**

```bash
cd ~/stride && cargo run --release -p stride-desktop
```

It serves the SPA on `127.0.0.1:8000` and the API beside it, and it opens
the athlete's own database.

## What it is

| | |
|---|---|
| the API | `stride-api`'s router — the same one the phone calls through the FFI. One router, two hosts. |
| the SPA | the built React app in `frontend/dist` — the same files the Python server serves. |
| the database | `backend/data/stride.db`, opened through `stride-store`, with SQLCipher. |
| the loopback | `127.0.0.1` and nothing else. The browser is on this machine; the phone has its own process and its own database. |
| the capability | an HttpOnly cookie, minted for the loopback browser exactly as the Python server mints it. |

## The commands

```
stride                 serve the app and the API on 127.0.0.1:8000
stride convert-db      convert the plain database into an encrypted one
stride --help          the environment variables
```

## The environment

| variable | default | what it does |
|---|---|---|
| `STRIDE_BIND` | `127.0.0.1` | the address to bind. A non-loopback address is **refused**, not warned about. |
| `STRIDE_PORT` | `8000` | the port to bind. |
| `STRIDE_DATA_DIR` | `backend/data` | where the database and the state file live. |
| `STRIDE_DB_PATH` | `<data dir>/stride.db` | the database. |
| `STRIDE_STATE_PATH` | `<data dir>/state.json` | the state file — the capability token lives there. |
| `STRIDE_DB_KEY` | — | the key the database is encrypted with. |
| `STRIDE_KEYVAULT_SECRET` | `stride-db-key` | the Key Vault secret that holds the key. |
| `STRIDE_CORS_ORIGINS` | `http://localhost:5173` | the origins that may call the API from a page. A `*` is refused. |
| `STRIDE_DISABLE_AUTH` | — | set it to turn the capability gate off. |

## The database, and the thing that is wrong in the contract

**The database on this machine is not encrypted.** The Python app writes a
plain SQLite file — `backend/app/db.py` has no cipher in it at all — while
`docs/rust-core-spec.md` and the Android workstream both say the database is
SQLCipher, and the workstream brief for this crate says the file is
encrypted. It is not: the file starts with `SQLite format 3`, and SQLCipher
refuses it with `file is not a database`.

`stride-store` cannot open a plain file — it always applies a key — so the
desktop **converts** it, once, on the first run:

* the plain file is snapshotted with `VACUUM INTO` — SQLite's own answer to
  "copy this database while somebody is writing to it" — and the snapshot is
  read and written into a new, encrypted file, with the same schema and the
  same rows;
* the snapshot is **checked against the database** — the row count of every
  table — and taken again if they do not agree, up to three times;
* the columns are matched **by name**, not by position: three tables
  (`activity`, `daily_physiology`, `run_fingerprint`) have the same columns
  in a different order, and a copy by position would put a timestamp in a
  distance column;
* the rows that were read and the rows that were written are counted, and the
  original is **not** replaced if the two counts disagree;
* the converted file is **read back** with the same key and its rows are
  counted again; if it does not answer, the plain file goes back where it
  was, because a desktop that cannot open the database is a nuisance and an
  empty database is a disaster;
* the plain file is kept as `<name>.plaintext-backup`, owner-only, because it
  holds a year of health data in the clear;
* the conversion reports the tables it copied and any drift between the two
  schemas — on this machine, no drift: the same 17 tables, the same columns.

### The conversion refuses to run while the database is in use

The database is **live**: the reference app is running and syncing from
Garmin into it. Two things would go wrong if the desktop converted it then:

* the copy would be of a database that is being written under the copy, and
  the rows would be a mixture of two states — which is how the first version
  of this conversion lost the plan, the physiology and the fingerprints;
* the rename would take the file away from the writer, and its rows would go
  into the old inode and nowhere else.

So the conversion refuses while another process has the file open, and says
which process:

```
stride: the database at /home/sumit/stride/backend/data/stride.db is open in
another process ([295179]). Stop it before the desktop converts the file: the
conversion replaces it.
```

Stop the reference app, and the desktop converts the file on its next run.

The key comes from `STRIDE_DB_KEY`, then from Azure Key Vault, then — only
because there was nothing else — from `backend/data/stride-desktop.key`,
which is written with mode 0600. **Keep that file with the database**: it is
the key.

`stride convert-db` does the conversion on its own, without serving.

## The layout

| module | what it does |
|---|---|
| `src/config.rs` | the environment: the bind address, the paths, the key. |
| `src/db.rs` | the database: the format, the conversion, the key. |
| `src/http.rs` | a small HTTP/1.1 server, written in-crate. |
| `src/security.rs` | the capability gate, the cookies, the rate limits. |
| `src/server.rs` | the routing: the API, the SPA, the headers. |
| `src/spa.rs` | the static files, and the traversal hole that is not there. |
| `src/state.rs` | the state file and the capability token. |

## Why the HTTP server is hand-written

The approved dependency set has no HTTP server in it. Adding one would add a
crate — and an async runtime, or a TLS stack, or both — to a binary that is
meant to be an app rather than a service. The desktop serves three kinds of
request: a `GET` for a file, a `GET` or `POST` for the API, and nothing
else. `src/http.rs` is a few hundred lines and it bounds everything it reads:
the request head (64 KiB), the request body (64 MiB), the number of live
connections (64) and the time a connection may stay silent (30 s).

## The dependencies

`serde`, `serde_json`, `chrono`, `anyhow`, `thiserror`, `stride-api` and
`stride-store` are the workspace's. The one addition is **`rusqlite`**,
which is already in the approved set and already in the workspace: it is the
plain SQLite connection that reads the reference app's database while it is
being converted — `VACUUM INTO` for the snapshot, `PRAGMA quick_check` for
the soundness check. It is the same crate `stride-store` is built on, with
the same vendored SQLCipher, so the binary carries one SQLite library and
not two.

## The traversal hole

The security audit found a live path-traversal hole in the SPA catch-all:
the path comes off the wire, percent-decoded, so it can hold `..`, an
absolute path, or a symlink pointing anywhere — and the reference's route
served whatever it resolved to. The Python fix resolves the candidate and
refuses anything that is not inside the directory, and `src/spa.rs` ports the
fix rather than the bug: the root and the candidate are both canonicalised,
and the candidate has to be *under* the root. A symlink inside the directory
that points out of it is refused exactly like `..` is.

## The tests

| test | what it proves |
|---|---|
| `tests/openapi.rs` | the router and the OpenAPI document the Python app generates agree: the same 62 paths, the same methods, the same path parameters, and every request body the document promises is accepted. |
| `tests/spa.rs` | the SPA is served for an unknown path, the static files have the right content types, a `..` traversal returns nothing outside `frontend/dist`, and the capability gate answers the way the reference does. |
| `tests/database.rs` | the **real** database opens, every row survives the conversion, the plain file is kept, the wrong key is refused, and the router reads the converted file. |

The database test takes **one** snapshot of the real database for the whole
run, with `VACUUM INTO`, and works on copies of it. The database is live —
the reference app is writing to it — and a snapshot that is taken while the
database changes is a snapshot of two states at once. That is what the first
version of the conversion did, and the test caught it: the plan, the
physiology and the fingerprints came out empty and the streams came out
half.
