# Security audit — 23 October 2026

Four auditors, one per surface, with disjoint file ownership:

| Area | Auditor | Files |
|---|---|---|
| API exposure & authentication | `security.py`, `main.py`, `api/misc.py` | local API surface |
| Secrets & data at rest | `config.py`, `secrets.py`, `db.py`, `redaction.py` | credentials, permissions |
| Browser surface | `index.html`, `vite.config.ts`, `lib/tiles.ts` | the SPA and its origins |
| Data pipeline & untrusted input | `ingest/*`, `services/personal_sync.py`, `services/sync.py` | Garmin + FIT |

Every finding below was verified, fixed, and re-verified. The automated gate
is `scripts/security_check.sh`; the human checklist is `CHECKLIST.md`.

---

## Findings that were confirmed and fixed

### 1. The local API had no authentication at all — **critical**

Any process on the machine, and anything on the tailnet, could read a year of
sleep, heart rate, stress and GPS routes, and drive syncs that spend the
athlete's Garmin quota.

**Fixed.** A **capability token** now gates every `/api/v1` route except
`/health` and a bootstrap endpoint. The token is minted by the server, stored
`0600`, and handed to the browser as an `HttpOnly`, `SameSite=Strict` cookie —
it is never returned in a response body and never reaches JavaScript. Loading
the SPA from loopback mints the cookie, so **the app needed no frontend change**.
A tailnet client must present the token explicitly as a bearer.

### 2. The server bound `0.0.0.0` — **high**

Now binds `127.0.0.1` by default. A non-loopback bind (`STRIDE_BIND=0.0.0.0`)
is possible but **refuses to start unauthenticated**, and says so loudly.

### 3. No CSRF defence — **high**

A page the athlete visited could POST to `http://127.0.0.1:8000` and drive the
local API. Now every state-changing request must either carry an explicit
bearer (not ambient browser authority) or be same-origin.

### 4. The Garmin key was world-readable on disk — **high**

`backend/data/state.json` held the live Personal API key, the account password
and the OAuth secrets at mode `0644` — readable by any other user on the VM.
The database and its WAL sidecars were `0644` too, and the data directory was
`0775`.

**Fixed.** The state file is created `0600` (with the mode set at creation, so
it is never briefly world-readable), the directory is `0700`, and the database
and its `-wal`/`-shm` sidecars are `0600`. Existing files are repaired on
startup. The state file also gained a schema version, so a future build cannot
silently mis-parse it.

### 5. The browser told two third parties who was looking — **medium**

The SPA loaded **Google Fonts** and **OpenStreetMap tiles**. Every page view
handed a third party the athlete's IP address and the fact that they were
looking at a running dashboard.

**Fixed.** Inter is now **self-hosted** (one 48 KB variable woff2). Map tiles
are fetched by **this server**, cached on disk, and served back from the same
origin, with a proper identifying `User-Agent` — so the browser makes **zero**
third-party requests. Verified in a headless browser: `THIRD-PARTY requests: NONE`.

### 6. No Content-Security-Policy — **medium**

Now present as a response header *and* a `<meta>` in the static build:

```
default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none';
form-action 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
font-src 'self' data:; img-src 'self' data: blob:; connect-src 'self'
```

Alongside `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`,
`X-Frame-Options: DENY` and `Permissions-Policy: camera=(), microphone=(),
geolocation=()` (the app needs none of them).

### 7. The FIT archive was decompressed without limit — **medium**

Garmin returns the original activity file as a ZIP; it was read with no size
cap, no entry cap and no decompression cap. Now: **5 MB** download, **8**
entries, **20 MB** decompressed, member paths checked for traversal.

### 8. Credentials could be forwarded across a redirect — **medium**

The Garmin client now requires **HTTPS to an allowlisted Garmin host**, verifies
TLS explicitly, uses a **10 s connect / 30 s read** timeout, caps response
bodies at **5 MB**, and strips the credential on a cross-host hop. A `429` gets
three retries at 1, 2 and 4 seconds.

### 9. The FIT parser had no input limit — **low**

Now capped at **20 MB** and **250,000 records**; non-finite values are rejected
before any calculation.

### 10. Unbounded history pulls — **low**

The `months` and `days` inputs are bounded in the **service** layer, not only in
the API layer, so a future caller cannot bypass the limit: **365 days** of
activity history, **12** history months, **300** activities per sync, **90**
wellness days.

### 11. Nothing rate-limited the Garmin routes — **low**

An in-process limiter now covers `/sync/run`, `/sync/history` and
`/auth/garmin/*`, so a runaway loop cannot spend the athlete's Garmin quota.

### 12. The Key Vault lookup could leak — **low**

`az` is invoked with an argument list and no shell, so a vault or secret name
cannot be interpolated into a command. The result is never logged, and a
multi-line result is rejected rather than stored.

### 13. Credentials could appear in logs or the database — **low**

`app/redaction.py` scrubs credential-shaped keys and values from any payload
before it is logged or persisted. A log filter now runs on the root logger, and
the HTTP exception handler redacts.

---

## Disproven

- **"A corrupt FIT can produce a partially written activity."** It cannot. The
  parser either returns a complete result or raises, and the importer catches
  the failure and stores the valid Garmin summary as a summary-only activity.
- **"Non-finite values reach the database."** `safe_float` already rejected
  them; the parser now also discards implausible values before calculating.

---

## Known limitations, stated plainly

1. **An unlocked, rooted, or compromised device can read plaintext that is
   already in memory.** Encryption protects data at rest on a locked device; it
   is not a claim of safety on a compromised endpoint. The privacy note says so.
2. **The harness may still send tool results to its model provider.** Stride
   cannot guarantee "no vendor data sharing" for a third-party agent — see
   `docs/harness-integration.md`.
3. **The tile proxy means this server's IP reaches OpenStreetMap.** That is
   strictly better than the athlete's, and the User-Agent identifies the app.
   It is not zero.
4. **`state.json` still holds the Garmin password** for the account path. It is
   `0600` and it can be dropped once a session token exists — tracked, not done.
5. **The local API is trusted once it holds the capability token.** Anyone who
   can read the cookie can read the athlete's data. That is the same trust
   boundary as the desktop process itself.
