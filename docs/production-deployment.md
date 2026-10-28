# The production instance

The app on `:8000` is the real thing: the athlete's own Garmin history, the
real training plan, no synthetic data, and a Garmin key that never touches
disk in plain text.

This is how it is built, run and checked.

---

## The shape of it

```
~/stride/backend/data/            0700
├── stride.db                     0600   160 activities, 371 physiology days
├── stride.db-wal                 0600
├── stride.db-shm                 0600
├── state.json                    0600   the session token, the key pointer
└── tile-cache/                   0700   the OSM tiles, proxied
```

The Garmin Personal API Key comes from **Azure Key Vault** (`claw-agent` →
`garmin-api`) through Managed Identity. It is never written to disk, never
sent to the browser, and never appears in a log. The app only ever reports
`key_source: key_vault`.

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
| backend tests | 120 pass |
| security gate | 22 pass |
| the instance answers | it is actually up |

---

## What `setup` does

1. **Creates the schema** and runs the migrations.
2. **Resolves the key** from Key Vault and validates it against Garmin before
   anything is written.
3. **Removes the synthetic demo activities** and their children.
4. **Pulls the history** — activities and wellness.
5. **Imports the training plan** and anchors it to the activity history.

The demo dataset is now **opt-in** (`STRIDE_SEED_DEMO=1`) and off by default.
A production instance starts empty and fills only from Garmin. That is the
whole point: a database that could contain a synthetic run is a database you
cannot trust.

### The plan's name

The plan document's title is *"20-Week Integrated Training Plan: HM Sub-2:00
Campaign"*. The block used to be called `plan`, because that was the file
name. It now takes the campaign from the title, so it matches the earlier
blocks: **HM Sub-2:00 Campaign**.

---

## Two things the setup found

Both were live bugs, and neither would have shown up in a demo.

### 1. The schema had drifted from the model

`activity.visibility` — the column from the removed social layer — was still
in the database, `NOT NULL`, with no default. The model no longer declared
it, so every INSERT omitted it, and **every single new activity import
failed**.

The app kept working, because it was serving the 158 activities it already
had. The failure only appeared the moment something new was written.

The fix is a migration that drops removed columns, and a test that asserts the
invariant forever:

> **no NOT NULL column may exist in the database that the ORM model does not
> know about**

That test is in `tests/test_robustness.py` and it runs in the gate.

### 2. 57 activities had no fingerprint, and no way to get one

The Personal API's activity *list* returns about a year — 103 activities,
back to October 2025. The database holds 160, because older ones were
imported earlier through a different path. Those 57 had Garmin's
**downsampled chart series** (about 125 points) instead of the FIT's
**per-record stream**, so they had no segmentation, no fingerprint and no
drift — and the sync never noticed, because an activity with *some* stream
data looks imported.

Two changes:

- **The FIT marker.** An activity is "imported" only if it carries a
  `fit_imported` marker or has per-second resolution. The resolution check is
  the important part: presence was never enough.
- **The backfill.** Garmin's download service will still hand over the
  original FIT for *any* activity id we hold, including ones outside the
  list's window. `tools.production` walks the activities we have, asks for
  each one's FIT by id, and rebuilds it.

All 57 were rebuilt. **Every one of the 160 activities now has a
fingerprint** — up from 103.

```bash
cd backend && .venv/bin/python -m tools.production backfill-fit
```

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
    "last_sync_at": "2026-10-26T11:15:01.733816+00:00"
  },
  "data": {
    "activities": 160,
    "from_garmin": 160,
    "physiology_days": 371,
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
restore. The snapshot taken before this work is `20261028T060708Z` — it holds
the pre-purge state, so the demo data can be brought back if anyone wants it.

---

## Before you trust it again

The gate is the contract:

```bash
./scripts/security_check.sh      # 22 passed, 0 failed, 0 warnings
./scripts/prod.sh doctor         # every check green
```

And the human checklist in `docs/security/CHECKLIST.md` is still the
authority for the things a script cannot judge — that the app works in
airplane mode, that the key is unreachable from the browser, that a share
link goes to Garmin and not to Stride.
