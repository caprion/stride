# The production instance

The instance on `:8000` is the real thing: the athlete's own Garmin history,
the real training plan, no synthetic data, and a Garmin key that never
touches disk in plain text.

This is how it is built, run and checked.

---

## The shape of it

```
backend/data/                  0700
├── stride.db                  0600   160 activities, 372 physiology days
├── stride.db-wal              0600
├── stride.db-shm              0600
├── state.json                 0600   the session token, the key pointer
├── stride-desktop.key         0600   the SQLCipher key, when Key Vault is not reachable
└── tile-cache/                0700   the map tiles, proxied
```

The Garmin Personal API Key comes from **Azure Key Vault** through Managed
Identity. It is never written to disk, never sent to the browser, and never
appears in a log. The app only ever reports `key_source: key_vault`.

---

## The commands

```bash
./scripts/prod.sh setup    # first run: schema, vault key, real history, plan
./scripts/prod.sh start    # harden permissions, start, health-check
./scripts/prod.sh status   # what is running and what is in the database
./scripts/prod.sh doctor   # the full pre-flight
./scripts/prod.sh stop
```

`doctor` is the one that matters. It checks, and it fails loudly:

| check | what it means |
|---|---|
| data dir 0700 | nobody else can list it |
| state.json 0600 | the token is not readable |
| key from Key Vault | not a pasted-in key, not a file |
| zero demo activities | the database holds only real runs |
| no NOT NULL orphan columns | the schema still matches the model |
| backend tests | the suite passes |
| security gate | the gate passes |
| a caller on an untrusted network is refused | the API is for the operator's machine |
| the instance answers | it is actually up |

---

## The configuration

**No network is named.** The deployment configures a network; the operator's
address is the operator's business.

| variable | default | what it does |
|---|---|---|
| `STRIDE_TLS_HOST` | `0.0.0.0` | the address the HTTPS listener binds |
| `STRIDE_TLS_PORT` | `8443` | the port for it |
| `STRIDE_TLS_DIR` | `data/tls` | where the certificate and the key are. It looks for `stride.crt` and `stride.key` |
| `STRIDE_TRUST_REMOTE` | `false` | when true, clients from the trusted networks are trusted without a capability |
| `STRIDE_TRUSTED_NETWORKS` | empty | comma-separated CIDRs those clients come from. **Empty means nothing extra is trusted** — loopback only |
| `STRIDE_PORT` | `8000` | the plain listener |
| `STRIDE_DATA_DIR` | `backend/data` | the database and the state file |
| `STRIDE_CORS_ORIGINS` | `http://localhost:5173` | the origins that may call the API from a page. A `*` is refused |

**The listener detects nothing.** It binds `STRIDE_TLS_HOST` and nothing
else: no address is discovered at startup and no external command is run to
work out where to listen. The address the operator wants is the address they
configure.

### Reaching it from another device

The phone reaches the desktop over the address the HTTPS listener binds, and
the certificate on that listener has to be one the browser trusts — otherwise
the phone will not install the app, because a service worker only registers
in a secure context.

The trade-off in `STRIDE_TRUST_REMOTE` is worth stating plainly: **trusting
a network means every device on it can read the athlete's health data.**
That is a far smaller group than the whole internet and a far larger group
than one device. It is off by default, because a stranger who clones this
repository should not find it trusting their network.

---

## What `setup` does

1. **Creates the schema** and runs the migrations.
2. **Resolves the key** from Key Vault and validates it against Garmin before
   anything is written.
3. **Removes the synthetic demo activities** and their children.
4. **Pulls the history** — activities and wellness.
5. **Imports the training plan** and anchors it to the activity history.

The demo dataset is **opt-in** (`STRIDE_SEED_DEMO=1`) and off by default.
A production instance starts empty and fills only from Garmin. That is the
whole point: a database that could contain a synthetic run is a database you
cannot trust.

### The plan's name

The plan document's title is *"20-Week Integrated Training Plan: HM Sub-2:00
Campaign"*. The block used to be called `plan`, because that was the file
name. It now takes the campaign from the title: **HM Sub-2:00 Campaign**.

---

## Two things the setup found

Both were live bugs, and neither would have shown up in a demo.

### 1. The schema had drifted from the model

`activity.visibility` — the column from the removed social layer — was still
in the database, `NOT NULL`, with no default. The model no longer declared
it, so every INSERT omitted it, and **every single new activity import
failed**.

The app kept working, because it was serving the activities it already had.
The failure only appeared the moment something new was written.

The fix is a migration that drops removed columns, and a test that asserts
the invariant forever:

> **no NOT NULL column may exist in the database that the ORM model does not
> know about**

That test is in `backend/tests/test_robustness.py` and it runs in the gate.

### 2. 57 activities had no fingerprint, and no way to get one

The Personal API's activity *list* returns about a year — 103 activities.
The database held 160, because older ones were imported earlier through a
different path. Those 57 had Garmin's **downsampled chart series** (about
125 points) instead of the FIT's **per-record stream**, so they had no
segmentation, no fingerprint and no drift — and the sync never noticed,
because an activity with *some* stream data looks imported.

Three changes:

- **The FIT marker.** An activity is "imported" only if it carries a
  `fit_imported` marker. A resolution check is *not* enough: an older watch
  samples its records every few seconds, so a genuine record stream and
  Garmin's chart series are the same length. The marker is the only reliable
  signal.
- **The marker survives a re-import.** The stored payload is carried forward.
- **The backfill.** Garmin's download service will still hand over the
  original FIT for *any* activity id we hold, including ones outside the
  list's window.

All 57 were rebuilt. **Every one of the 160 activities now has a
fingerprint** — up from 103.

```bash
cd backend && .venv/bin/python -m tools.production backfill-fit
```

It is idempotent: a second run rebuilds nothing and downloads nothing.

#### Two traps worth knowing about

**`compute_many` does not commit.** It flushes and leaves the transaction to
its caller. A caller that forgets looks exactly like success — the count is
right inside the process and the rows are gone after it exits. The backfill
commits explicitly.

**SQLAlchemy does not track mutations to a JSON column.** Marking an
activity by mutating its `raw` dict in place is silently discarded on
commit. The column has to be *replaced* with a new object. Both are covered
by tests in `tests/test_robustness.py`.

### The answer to the retention question

Garmin's Personal API exposes a **rolling window on the list**, not on the
files:

- the activity list returns roughly the last 12 months;
- the download service serves the FIT for any id you already hold, for as
  long as Garmin keeps it.

So Stride can hold more history than Garmin's list shows — as long as it
keeps the ids it has already seen. The retention limit is on *discovery*, not
on *retrieval*.

---

## The sync log

The sync history on the Settings page is an **operational log**, not an audit
record. During the `visibility` bug it grew seventeen rows of the same
failure, one every three hours.

```bash
cd backend && .venv/bin/python -m tools.production prune-sync-history --keep 8
```

A failure is *resolved* once a later run has succeeded, and resolved failures
are dropped.

---

## Connecting another device

**The phone app does not need this.** It has its own database and its own
Garmin key. This is about a *browser* on another device reaching the
desktop.

A browser on the machine that runs Stride is given a session cookie
automatically, because the server mints it when it serves the app to a
loopback client. A browser somewhere else is a different machine. It cannot
mint that cookie, and it should not be able to: if every device on the
network could, then the network *is* the trust boundary, and that is a much
bigger group than "the athlete's own devices".

**There is no second credential.** The app has **one**: the Garmin API
key. The phone types it into one native screen and it goes straight into
the Keystore; the desktop has the same field on its own screen, and the
browser on the machine is signed in automatically.

A browser on another machine is signed in the same way — **if the operator
has trusted the network it is on** (`STRIDE_TRUST_REMOTE` and
`STRIDE_TRUSTED_NETWORKS`). Both default to the safe answer: loopback
only, and nobody else trusted.

There is no pairing code, no QR code and no second device to revoke: the
capability the browser holds is the machine's own, it is a session
cookie, and closing the browser signs it out.

### Properties

| property | why |
|---|---|
| the machine's own, or a trusted network | a browser elsewhere is refused |
| a session cookie | closing the browser signs it out |
| the capability never leaves the machine | it is in the state file, mode 600 |

The security gate checks the first of these on the live service: a caller
on an untrusted network is refused.

"> **A trusted network is a network the operator named.** The peer address
> *and* the `Host` header have to say so: a page the athlete happens to
> have open can point a name it controls at 127.0.0.1 and then talk to
> Stride as the origin it came from. A caller on any other network is
> refused, and the security gate checks that over TLS.

---

## Two listeners, one process

| listener | who | why |
|---|---|---|
| `:8000` plain HTTP | the operator, on the machine | the tools and the local browser |
| the HTTPS listener | another device | a secure context, so the app can be installed |

One process means one scheduler and one state file — two processes would
mean two auto-syncs racing each other against the Garmin API.

The service worker caches the **shell** only. The API is never cached: health
data is not something to serve out of a cache, and the app already promises
that nothing leaves the machine.

---

## Health

```bash
curl -s http://127.0.0.1:8000/api/v1/health | python3 -m json.tool
```

```json
{
  "ok": true,
  "service": "stride",
  "garmin": {
    "connected": true,
    "method": "personal",
    "key_source": "key_vault",
    "last_sync_at": "2026-10-29T04:02:23.128379+00:00"
  },
  "data": {
    "activities": 160,
    "from_garmin": 160,
    "physiology_days": 372,
    "plan_sessions": 332
  }
}
```

It reports the key's *source*, never the key.

---

## Snapshots

```bash
./scripts/snapshot.sh
```

Verified, checksummed snapshots under `backend/data-snapshots/`, with a safe
restore.

---

## Before you trust it again

The gate is the contract:

```bash
./scripts/security_check.sh      # 26 passed, 0 failed, 1 warning
./scripts/prod.sh doctor         # every check green
```

And the human checklist in `docs/security/CHECKLIST.md` is still the
authority for the things a script cannot judge — that the app works in
airplane mode, that the key is unreachable from the browser, that a share
link goes to Garmin and not to Stride.
