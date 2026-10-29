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

Any process on the machine, and anything on the operator's network, could
read a year of
sleep, heart rate, stress and GPS routes, and drive syncs that spend the
athlete's Garmin quota.

**Fixed.** A **capability token** now gates every `/api/v1` route except
`/health` and a bootstrap endpoint. The token is minted by the server, stored
`0600`, and handed to the browser as an `HttpOnly`, `SameSite=Strict` cookie —
it is never returned in a response body and never reaches JavaScript. Loading
the SPA from loopback mints the cookie, so **the app needed no frontend change**.
A client on the operator's network must present the token explicitly as a
bearer.

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

---
---

# Second audit — 28 October 2026 — the new surface

The first audit (above) covered the API's authentication, the files on disk and
the browser's cross-origin surface. Everything below was added **after** that
audit and had never been reviewed: the device pairing exchange, the device
credentials, the asset-links route, the two-listener server, the Android app
and the tile proxy.

The audit was run against the **live instance** on this machine (the address
the listener binds, plain HTTP on `:8000`, HTTPS on the TLS port), not only
against the test client, so every "the request that reaches it" below is a
request that was actually sent and answered.

**No public API shape changed.** No path, no request body and no response
field was touched. Three status codes changed, all of them refusals, and they
are listed in *Behaviour changes* at the end of this section.

---

## Critical

### 1. An unauthenticated read of any file the process can open — **critical**

`backend/app/main.py`, the SPA catch-all:

```python
candidate = FRONTEND_DIST / full_path
if full_path and candidate.is_file():
    return FileResponse(candidate)
```

`full_path` is the raw request path, percent-decoded by the server. `..` is
perfectly ordinary in it, and `Path.__truediv__` with an absolute path
discards the base entirely.

**The request that reaches it** (no capability token, no cookie, plain HTTP,
from the operator's network or the Azure VNet):

```
GET /%2e%2e%2f%2e%2e%2fbackend%2fdata%2fstate.json
→ 200, 3563 bytes:
{
  "garmin_connected": true,
  "garmin_method": "personal",
  "personal_api_key": "GCDC_5c8ab97b...",
  "capability_token": "...",
  "devices": [ ... "token_hash": ... ],
  "pairing_code": "..."
}
```

Also confirmed, in the same session:

```
GET /%2e%2e%2f%2e%2e%2f%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd   → 200, /etc/passwd
GET /%2e%2e%2f%2e%2e%2fbackend%2fdata%2fstride.db                  → 200, the whole database (18 MB)
GET /%2e%2e%2f%2e%2e%2fbackend%2fapp%2fsecurity.py                → 200, the source
```

This defeats **every** control the first audit installed: the capability
token, the `0600` on `state.json`, the `0700` on the data directory. It is
the Garmin key, the master capability and the device token hashes, handed to
anybody who can reach the port. The `StaticFiles` mount at `/assets` was
already safe — Starlette resolves and confines it — which is why this one
route was missed.

**Fixed.** The path is resolved and then checked to be *inside*
`frontend/dist`; anything that escapes — by `..`, by being absolute, or
through a symlink — is not served.

```python
def _dist_file(full_path: str) -> Path | None:
    if not full_path:
        return None
    root = FRONTEND_DIST.resolve(strict=False)
    candidate = (root / full_path.lstrip("/")).resolve(strict=False)
    if candidate != root and root not in candidate.parents:
        return None
    if not candidate.is_file():
        return None
    return candidate
```

Re-verified after the fix: all four requests above now return the SPA shell
(2 496 bytes of `index.html`), and `/`, `/index.html` and `/sw.js` still
serve normally.

---

## High

### 2. The "no unauthenticated non-loopback bind" guard was looking at the wrong variable — **high**

The first audit's claim was that a non-loopback bind *refuses to start*
without authentication. `ensure_safe_bind_configuration()` reads
`configured_bind()`, which read `STRIDE_BIND` (default `127.0.0.1`). But the
server that actually runs in production is `python -m app.serve`, and it binds
`STRIDE_HOST` (default `0.0.0.0`). The two never met.

**The request that reaches it:**

```
$ STRIDE_DISABLE_AUTH=true STRIDE_PORT=8123 python -m app.serve
$ ss -ltn | grep 8123
LISTEN 0 2048  0.0.0.0:8123  0.0.0.0:*
$ curl http://127.0.0.1:8123/api/v1/activities
200 {"items":[],"total":0,...}
```

The whole API, unauthenticated, on every interface — while the guard
believed the bind was `127.0.0.1` and said nothing.

**Fixed.** `ensure_safe_bind_configuration(bind=...)` now takes the address
the server is *about* to bind, and `app.serve` passes its own. `configured_bind()`
also learned about `STRIDE_HOST`, so the two agree however the app is started.
The same command now refuses to start:

```
RuntimeError: Refusing to bind 0.0.0.0 while STRIDE_DISABLE_AUTH is enabled.
```

### 3. The loopback check looked at the peer, not at the name — **high**

`is_loopback_client()` decided "is this request from this machine?" from the
TCP peer address alone. That is not enough for a server on `127.0.0.1`, which
is the default bind: a page the athlete happens to have open can point a name
it controls at `127.0.0.1` (DNS rebinding) and then talk to Stride as the
origin it came from. The peer is loopback; the Host header is the attacker's
name; the server cannot tell the difference.

Two endpoints mint authority and both were reachable that way: the bootstrap
(which hands out the **master capability** as an `HttpOnly` cookie) and the
pairing code. The SPA path in the middleware mints the same cookie.

**The request that reaches it** (the shape a rebound name produces):

```
GET /api/v1/bootstrap        Host: evil.example:8000   → was 204 + Set-Cookie: stride_capability=<master>
GET /api/v1/auth/pair-code   Host: evil.example:8000   → was 200 + {"code": "...."}
```

**Fixed.** `is_loopback_client()` now requires two independent facts: the
peer is a loopback address **and** the Host header names a loopback host
(`127.0.0.1`, `localhost`, `::1`). A rebound name fails the second test.
Both requests above now answer `403`.

---

## Medium

### 4. The paired-device list was edited without a lock — **medium**

`touch_device()` (in `security.py`), `pair()` and `revoke_device()` (in
`auth.py`) each read `state.devices`, edited a copy and wrote the copy back
with `state.update(devices=...)`. Nothing held a lock across the
read-modify-write, so two of them interleaving lost one of the edits.

**The code path that reaches it** — a "last seen" refresh for device A is in
flight while device B pairs:

```python
devices = [dict(d) for d in (state.devices or [])]   # A reads the list
...                                                  # B pairs and writes [A, B, new]
state.update(devices=devices)                        # A writes its stale copy back
```

Reproduced against the old code: the newly paired device **disappears from
`state.json`**, so the phone is signed out the moment it pairs. The same
interleaving the other way round writes a **revoked device back into the
file** — the "lost phone" case, where the athlete revokes a phone and the
phone keeps working.

**Fixed.** `AppState.atomic()` (in `config.py`) runs a read-modify-write under
one lock hold and persists it before releasing the lock. All three call sites
use it. The state lock became an `RLock` so that the write can happen inside
the same hold. Re-run of the interleaving: the newly paired device survives.

### 5. The pairing code was tested and cleared in two steps — **medium**

`pair()` did `if not _pair_code_valid(code): ...` and then, separately,
`state.update(pairing_code=None)`. Two simultaneous exchanges with the same
code could both pass the test before either cleared it, and both would mint a
device. The window is a few microseconds wide, so it is not easy to hit — 40
rounds of 32 simultaneous exchanges did **not** reproduce it here — but the
code path is a textbook time-of-check-to-time-of-use and the fix costs
nothing.

**Fixed.** The test and the clear now happen inside the same `state.atomic()`
hold, so a single-use code is single use.

### 6. A pairing code with a non-ASCII character crashed the exchange — **medium**

`hmac.compare_digest()` refuses non-ASCII *strings* with a `TypeError`, and
nothing caught it.

**The request that reaches it** (while a code is pending):

```
POST /api/v1/auth/pair  {"code": "ÄÄÄÄ"}   → 500 Internal Server Error
```

It also made "a pairing code is pending" distinguishable from "no pairing
code is pending" by the status code alone, which is a small state oracle on
the one endpoint that is deliberately unauthenticated.

**Fixed.** The comparison is done on bytes, and a code of the wrong length is
refused before the comparison. The same request now answers `401` with the
same detail as every other wrong code.

### 7. A device credential could be minted for — and then carried over — plain HTTP — **medium**

`set_device_cookie()` set `Secure` only when the request arrived over HTTPS.
The pairing exchange accepted a plain-HTTP request from a remote client. So a
phone that paired over `http://<the address the listener binds>:8000` was
handed a
**year-long bearer token** (`Max-Age=31536000`) that it then sent, in the
clear, on every request. On the operator's network the transport is
encrypted,
but the plain listener binds `0.0.0.0`, which is also the Azure VNet
interface, where it is not.

**Fixed.** A pairing from a client that is not on this machine is refused
over plain HTTP:

```
POST /api/v1/auth/pair   (peer <the operator's address>, scheme http)
→ 403 "Pair this device over HTTPS. A device credential must not be carried
        over plain HTTP — use the TLS listener."
```

and the device cookie is marked `Secure` for any device that is not on this
machine. Over the TLS listener the same exchange works and the cookie comes
back `HttpOnly; SameSite=strict; Secure; Max-Age=31536000`.

The master capability cookie is unaffected: it is only ever minted for a
loopback client, and it is a *session* cookie there.

### 8. uvicorn trusted `X-Forwarded-For` from `127.0.0.1` — **medium**

`app.serve` used uvicorn's defaults, which are `proxy_headers=True` and
`forwarded_allow_ips=127.0.0.1`. Stride is not behind a proxy, but if
anything ever were — a reverse proxy on this box, a tunnel, or a port
forward pointed at `:8000` — then a remote client could set `X-Forwarded-For:
127.0.0.1` and be *seen* as a loopback client by every check in this
document: the pairing code, the bootstrap, the capability cookie.

Nothing proxies to `:8000` today, so this was not reachable; it is one proxy
away from being reachable, and the check it would defeat is the one the
pairing flow is built on.

**Fixed.** Both listeners are started with `proxy_headers=False`.

### 9. The tile proxy followed redirects and buffered the body before capping it — **medium**

`backend/app/api/tiles.py` fetched with `requests.get(TILE_URL...)`, which
follows redirects by default, and read `response.content` — the whole body —
before checking it against `MAX_TILE_BYTES`. The upstream host is a constant
and the coordinates are parsed integers, so there is **no SSRF today** (see
*Verified, not a finding*), but a redirect would have turned the tile proxy
into a general-purpose fetcher, aimed wherever the redirect pointed.

**Fixed.** `allow_redirects=False`, the body is read in chunks and capped
*while* it is read, and the URL is built from the parsed integers so nothing
off the wire is ever interpolated into it.

---

## Low, and one build-time invariant

### 10. The asset-links file was served over cleartext to the whole local network — **low**

`/.well-known/assetlinks.json` was served on every host, including the plain
HTTP listener.

**This is not a leak.** The file names the package (`app.stride.running`) and
the SHA-256 fingerprint of the signing certificate — both of which are in
every APK, and the fingerprint is public by definition. There is nothing in it
that is not already published.

It is also **useless** over plain HTTP: Digital Asset Links is defined over
HTTPS, so a cleartext copy can never satisfy a check.

**Fixed anyway**, because a copy that cannot be used does not need to be on
the wire in the clear: it is now served over HTTPS, or to a loopback client,
and answers `404` otherwise. The response already carried the CSP,
`X-Content-Type-Options`, `Referrer-Policy`, `X-Frame-Options` and
`Permissions-Policy` headers and `Cache-Control: public, max-age=3600`.

### 11. The published signing fingerprint was never checked against the key — **low**

The keystore is not in the repository (`.secrets/` and
`android/keystore.properties` are gitignored; CI writes the key to
`$RUNNER_TEMP`), and the committed fingerprint was checked here against the
development key:

```
$ keytool -list -v -keystore .secrets/keystore.jks -alias stride
SHA256: 87:E5:7B:81:F8:ED:7C:FE:1B:A6:0B:8F:B5:E2:E7:3F:B3:E6:41:C8:71:08:46:ED:87:74:1B:7B:9B:23:BC:26
```

which is exactly the fingerprint in `backend/app/assetlinks.json`. But nothing
*enforced* it, and the CI key is a secret that cannot be inspected from here.
When the two drift, Digital Asset Links stops matching and the Trusted Web
Activity degrades into a browser tab with a web page in it — which looks like
a broken app, not like a broken signature.

**Fixed.** `android/app/build.gradle` gains a `verifyAssetLinksFingerprint`
task that reads the certificate out of the keystore with `keytool` (the
password is handed over in the environment, not on the command line) and
compares it with the published file. It runs before `preReleaseBuild`, and it
fails the build when they differ.

### 12. The WebView could be pointed at a cleartext origin — **low**

`capacitor.config.ts` takes `server.url` from `STRIDE_SERVER_URL` and used it
without checking it. The WebView is the app's only surface, and the origin it
loads is the origin the Capacitor bridge is injected into, with the
paired-device cookie for that origin in the WebView's jar.

**Fixed.** The config now refuses to build with a non-`https://` URL, so an
APK cannot ship with a cleartext WebView target. `cleartext: false`,
`allowMixedContent: false` and `androidScheme: 'https'` were already set and
are unchanged.

---

## Verified, not a finding

These were checked because they were asked about, and they hold.

**The pairing code really is loopback-only, and the check is on the route,
not only in the middleware.** The middleware's `CAPABILITY_FREE` set does
include `/api/v1/auth/pair-code`, but the route itself refuses a client that
is not on this machine — `pair_code()` calls `is_loopback_client()` and
answers `403`. Verified live: `GET http://<the operator's address>:8000/api/v1/auth/pair-code`
→ `403`, from a remote address, with no token. The middleware's allowlist
only decides *that the middleware does not demand a token*; the route decides
*who gets the code*.

**The pairing exchange cannot be driven cross-origin.** The same-origin check
in the middleware covers `POST` on every `/api/v1` path, the pairing exchange
included — it is not on an exemption list. Verified live: `Origin:
https://evil.example` → `403 Same-origin request required.`; a request with
**no** `Origin` header at all → `403` as well (a non-browser client has to
say which origin it is acting for).

**The device token is strong enough and is stored correctly.** It is
`secrets.token_urlsafe(32)` — 32 bytes from the OS CSPRNG, 256 bits — and only
its SHA-256 is stored. The listing returns `id`, `name`, `created_at` and
`last_seen_at` and nothing else: no token, and no hash either. The hash is
not a credential (a SHA-256 of 256 random bits is not invertible), but it is
not returned either, and there is no reason for it to be.

**The master capability does not leak through a device flow.** The pairing
response is `204` with one `Set-Cookie` header carrying the *device* token;
the master capability is never in a pairing response, and the only other
place it is minted is the bootstrap, which is loopback-only. `has_valid_capability()`
accepts either credential, and the two never meet: a device token is checked
against the device list, the capability against the capability.

**The delete-by-id route cannot be abused.** `DELETE /api/v1/auth/devices/{id}`
filters the list on an exact id match, so it can only remove a device — it
cannot touch the master capability or anything else. The ids are twelve hex
characters; the `404` versus `200` distinction says only whether an id
exists, which is not worth enumerating.

**The code's expiry is not client-controlled.** It is compared against the
server's own clock (`time.time()`), stored in the state file when the code is
minted, and the code is valid for ten minutes.

**`_close_inherited_descriptors()` cannot close something the app then
needs.** It runs first thing in `main()`, before the app is imported and
before uvicorn opens its socket, and it keeps `0`, `1` and `2`. Nothing the
application opens afterwards is inherited, and nothing it inherits is a
descriptor it needs — the log file is already `stdout`/`stderr`.

**The Android WebView is not left in a debuggable state.** Capacitor 8 reads
`android.webContentsDebuggingEnabled` from the generated
`assets/capacitor.config.json`, and an explicit `false` wins over the
"release build is debuggable" default (`CapConfig.java`: the explicit value
is used, `isDebug` is only the fallback). Navigation to another host is
handed to the system browser by Capacitor's own `launchIntent`, so the WebView
does not wander.

**The tile proxy cannot be turned into an SSRF.** The upstream URL is a
constant with no user-controlled part, the coordinates are parsed integers
and bounds-checked against the zoom's grid, and the response is served from
the athlete's own cache.

---

## Deliberately not fixed

1. **`android/app/src/main/AndroidManifest.xml` has
   `android:allowBackup="true"`.** The WebView's cookie jar — which holds the
   paired-device credential — is inside the app's data directory and can be
   included in an automatic backup. The manifest is outside the files this
   audit was allowed to edit. The fix is one line:
   `android:allowBackup="false"` (and, if backups are wanted for anything
   else, a `dataExtractionRules` XML that excludes the WebView data).
2. **`.gitignore` covers `android/app/*.jks` and `android/app/*.keystore`,
   but not `*.jks` / `*.keystore` anywhere else in the tree.** The key that
   is actually on this machine lives in `.secrets/` (ignored) and the
   properties file that points at it is ignored, so nothing is exposed today.
   `*.jks`, `*.keystore` and `keystore.properties` as bare patterns would
   close the gap. `.gitignore` was outside this audit's edit scope.
3. **The automated gate gained no new checks.** `scripts/security_check.sh`
   was outside this audit's edit scope. The checks that belong in it:
   - a request for `/%2e%2e%2f%2e%2e%2fbackend%2fdata%2fstate.json` must not
     return the state file;
   - `GET /api/v1/auth/pair-code` with a non-loopback `Host` header must be
     refused;
   - `POST /api/v1/auth/pair` with a non-ASCII code must be `401`, not `500`;
   - `python -m app.serve` with `STRIDE_DISABLE_AUTH=true` must refuse to
     start.
4. **No regression tests were added**, because `backend/tests/` was outside
   this audit's edit scope. The cases the suite should grow:
   `test_the_spa_catch_all_cannot_escape_the_dist_directory`,
   `test_a_rebound_host_cannot_mint_a_capability`,
   `test_a_pairing_code_is_consumed_atomically`,
   `test_revoking_a_device_is_not_undone_by_a_concurrent_last_seen_refresh`.
5. **The plain HTTP listener used to bind `0.0.0.0`**, which is the Azure VNet
   interface as well as the operator's network. This was fixed while the audit was
   running, by the same change that split the two listeners: the plain one is
   now on loopback and the TLS one on the operator's address, so nothing is
   bound on the public interface any more. The gate's remote pairing check
   was moved to the TLS listener at the same time. (It picks the first
   non-loopback address, which on this machine is the VNet address rather
   than the TLS one, so that check currently warns instead of running;
   pointing it at the TLS address would make it run. The route's behaviour
   was verified
   by hand over the TLS listener: a request from a remote address is
   refused with `403`.)
6. **The tile cache is unbounded on disk and upstream fetches are not rate
   limited.** Both are reachable only with a valid credential (a paired
   phone). A map needs a generous request budget, so a rate limit here
   trades a DoS against a broken map; a disk-usage ceiling on
   `backend/data/tile-cache` would be the better half of the trade, and is
   left for a later change.
7. **`scripts/prod.sh` puts the pairing code in a URL fragment**
   (`#pair=CODE`). A fragment is never sent to the server, so the code does
   not reach a log or an access log — but it does land in the phone's browser
   history. The script was outside this audit's edit scope.
8. **The device list grows without bound.** There were nine paired devices on
   this machine before the audit began, most of them months old. That is a
   housekeeping matter, not a vulnerability: each one is a revocable,
   individually-named credential.

---

## In flight while this audit was running: "trust the operator's network"

A second change landed in the same files during the audit: an opt-in
`STRIDE_TRUST_REMOTE` mode in which a device on one of the operator's
configured networks may read and drive the API without a capability token or
a pairing code, and the server binds its plain listener to loopback and its
TLS listener to the operator's address. The networks are the ones named in
`STRIDE_TRUSTED_NETWORKS`; the default list is empty, so nothing extra is
trusted until the operator says so.

It is not this audit's work and it was not reviewed here, but it touches the
findings above, so:

- **The pairing exchange is still loopback-only.** `pair-code` asks
  `is_loopback_client()`, not `is_trusted_client()`, so a trusted peer still
  gets `403` — verified over the TLS listener.
- **The capability token is no longer the boundary in that mode.** In
  `STRIDE_TRUST_REMOTE` mode the boundary is the operator's network itself.
  Findings 7 and 9 are unaffected (a device credential is still never minted
  over cleartext, and the tile proxy still fetches one allowlisted URL), but
  the "the API needs a token" property of the first audit is deliberately
  relaxed in that mode.
- **One hole was closed.** The mode skipped the same-origin check for
  trusted clients outright, which meant that any page the athlete's phone
  visited could make simple cross-origin `POST`/`DELETE` requests to the API
  — a page cannot read the responses (CORS still blocks that), but it can
  revoke a device or trigger a sync. The check now applies whenever the
  request carries an `Origin` header; only a request with **no** `Origin`
  (a native client) is let through from a trusted network. That keeps the
  feature's intent — the operator's configuration says who is calling —
  without letting a web page borrow the phone's network position.

---

## Behaviour changes

No path, request body or response field changed. Three refusals were added:

| Request | Was | Now |
|---|---|---|
| `POST /api/v1/auth/pair` from another machine over plain HTTP | `204` + a device cookie | `403` — pair over HTTPS |
| `GET /.well-known/assetlinks.json` over plain HTTP from another machine | `200` | `404` (it is served over HTTPS, and to loopback clients) |
| `GET /api/v1/bootstrap`, `GET /api/v1/auth/pair-code` with a non-loopback `Host` | `204` / `200` | `403` |

And one launch-time refusal, which is the first audit's rule finally being
enforced rather than merely intended: `python -m app.serve` with
`STRIDE_DISABLE_AUTH=true` now refuses to start, because it binds an address
that is not loopback — and the check covers *both* listeners, so a loopback
plain listener with a TLS listener on a remote address is still caught.

One credential was rotated during the audit: the master capability token was
printed in full by one of the probes below, so it was replaced. The athlete's
browser picks up the new cookie on the next page load — the server re-mints
it on every HTML request from the machine — and paired devices are unaffected
(they hold their own token). The Garmin key was exposed only as a twelve
character prefix by the same probe, which is not usable.
