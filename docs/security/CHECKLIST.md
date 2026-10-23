# Security checklist

Every change to Stride ends with a green `./scripts/security_check.sh` and a
pass through this checklist. It is part of the working model, not an optional
extra — see `docs/working-agreement.md` §3.

The automated script covers the mechanical checks. This file covers the
judgement calls a script cannot make.

---

## 1. The threat model in one paragraph

The athlete's data is a year of sleep, resting heart rate, stress, body
battery, heart-rate zones and GPS routes — plus a **live Garmin Personal API
key** that can read and write their Garmin account. The realistic adversaries
are: another process on the machine, a page the athlete visits in a browser
that tries to drive the local API, someone with the tailnet, and a lost or
compromised phone. A determined attacker with root on the device is **out of
scope** — say so in the privacy note rather than pretending otherwise.

## 2. Credentials

- [ ] The Garmin Personal API key is never written to a log, an exception
      message, a URL, a query string, or a browser-visible API response.
- [ ] `backend/data/state.json` is `0600`; `backend/data/` is `0700`.
- [ ] The database and its `-wal`/`-shm` sidecars are `0600`.
- [ ] The key is only ever sent to a Garmin host, over TLS, with verification
      on — and never forwarded across a redirect to another host.
- [ ] The key is not in the frontend bundle, not in `localStorage`, not in a
      cookie, and not in the git history.
- [ ] Secrets come from Azure Key Vault via Managed Identity, or from the
      environment — never from a file in the repository.
- [ ] `state.json` has a schema version and refuses a future one.

## 3. The API surface

- [ ] The server binds loopback by default. A non-loopback bind is possible
      only with authentication switched on, and says so loudly.
- [ ] Every `/api/v1` route requires the capability token, except an explicit
      allowlist (`/health`, the bootstrap).
- [ ] State-changing requests are rejected cross-origin, so a malicious page
      cannot drive the local API.
- [ ] CORS is not a wildcard and does not combine `allow_credentials` with a
      broad origin list.
- [ ] Expensive routes (sync, history, connect) are rate limited.
- [ ] Responses carrying health data are `Cache-Control: no-store`.

## 4. The browser

- [ ] A Content-Security-Policy is present, on the static build and as a
      response header, and the map still renders under it.
- [ ] The app loads **no** third-party resources: no Google Fonts, no
      analytics, no error reporting, no CDN. The only permitted outbound
      request is to the athlete's own Garmin.
- [ ] The tile policy is a single source of truth and its default does not
      tell a third party the athlete's IP address.
- [ ] No `dangerouslySetInnerHTML`, no `eval`, and athlete-supplied text
      (activity names, descriptions) is escaped.
- [ ] Numbers that reach the DOM — GPS coordinates, paces, heart rates — are
      coerced to finite numbers, so a corrupt value cannot become markup.

## 5. Untrusted input

- [ ] The FIT archive is size-capped, entry-capped, and its extracted member
      is decompression-capped; member names are checked for traversal.
- [ ] The FIT parser caps input size and record count, and rejects
      non-finite values before they reach a calculation.
- [ ] Garmin responses are body-capped, timeout-bounded, and follow only
      bounded redirects.
- [ ] The history endpoints are bounded in the **service** layer, not only in
      the API layer, so a future caller cannot bypass the limit.
- [ ] A corrupt or hostile input produces a clean error, never a partial
      write.

## 6. Privacy

- [ ] No telemetry, no analytics, no crash reporting that carries payloads.
- [ ] `Activity.raw` and the sync history are scrubbed of anything
      credential-shaped before they are persisted.
- [ ] The privacy note states plainly that the Garmin key must reach Garmin
      itself over TLS — "never leaves your device" is not a promise the app
      can keep.
- [ ] The app states that it cannot erase text already returned to a
      third-party model's context.

## 7. Supply chain

- [ ] `pip-audit` and `npm audit --omit=dev` are clean.
- [ ] The production build ships no source maps and no stray files.
- [ ] No `VITE_*` environment variable can carry a secret into the bundle.

## 8. Before shipping a change

```bash
cd ~/stride
./scripts/security_check.sh          # the automated gate
```

- [ ] The gate is green.
- [ ] The backend suite and the frontend typecheck pass.
- [ ] The change is committed with the reasoning in the message.
- [ ] If the change touched the API surface, the auth behaviour was re-tested
      by hand: a request without a token is rejected; a cross-origin POST is
      rejected; a request with a token still works.
