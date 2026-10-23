# Stride Android: local-first, native core, reused web UI

Decision date: 2026-10-23. Status: proposed architecture; no Android implementation or device validation performed. Repository inspected at commit `a83091d`; unrelated untracked showcase files were left alone.

## 1. Decision and rationale

**Choose Capacitor with the existing React/Vite UI bundled into the APK, and a Kotlin core for encrypted storage, Garmin networking, FIT decoding and WorkManager.** Do not wrap the remotely hosted website. Do not run FastAPI or Python on the phone.

This preserves Stride's expensive product assets: activity charts, comparison presentation, plan-vs-actual screens, physiology screens, formatting and design language. It also accepts the real cost: the current Python backend contains most of the product's computation. A WebView wrapper alone provides neither offline Stride nor secure credential handling. That backend logic must be ported regardless of which UI framework wins.

The phone database is authoritative. There is no Stride account, hosted API dependency, desktop proxy, Azure Key Vault dependency, Tailscale route switching or required Garmin connection. Garmin Connect still supplies watch-recorded activities when the athlete opts in; this is not direct Bluetooth watch sync.

### Options compared

| Option | Existing frontend retained | Offline and background sync | FIT, notifications and per-run GPS | Change and verdict |
|---|---|---|---|---|
| **Capacitor + Kotlin core** | React DOM pages, Tailwind, Recharts, Leaflet and formatter code survive; transport and platform settings change | Bundled assets plus native SQLCipher; Kotlin WorkManager runs without a WebView | Kotlin SAF importer, decoder and notification APIs; a future phone recorder needs a location foreground service | Smallest UI change, substantial backend port. **Choose.** Native plugin access is documented [S1]; WebView timers are not a scheduler [S2] |
| **React Native / Expo** | Plain TypeScript DTOs and formatters survive. DOM components, Recharts, react-leaflet and CSS do not directly port | Native SQLite; Expo supports SQLCipher with native configuration, and BackgroundTask uses WorkManager [S6, S7] | Document picker and notification integrations; sustained recording needs suitable native location integration, not periodic BackgroundTask | Viable runner-up if WebView interaction fails. Rewrites the entire rendered product without removing backend-port work |
| **Kotlin + Compose** | Design values and domain contracts can be translated; no direct reuse of TypeScript UI or runtime client | Direct Room/SQLCipher and WorkManager integration | Best direct access to SAF, notifications and foreground location services | Cleanest single-platform implementation, largest UI rewrite. Reserve for proven WebView limitations, not speculative polish |
| **Tauri mobile** | Similar web UI reuse to Capacitor | Rust storage/domain core possible; Android scheduling still needs platform integration | Kotlin mobile plugins can expose Android APIs [S8] | Adds Rust plus Kotlin to a Python/TS project. No measured Stride APK-size advantage; do not assume it is smaller than Capacitor, which also uses the system WebView |
| **PWA only** | Most UI survives | Requires a local data/domain rewrite, service worker and durable browser storage; browser caching is not an authoritative SQLite design, and background scheduling is browser-controlled [S9] | User-mediated file access and web notifications; no equivalent direct Android Keystore integration or reliable screen-off run recorder | Useful companion web experience, not the phone source of truth for this brief |

All packaged options face the same Play health-data, permission and minimum-functionality obligations. Using a native-rendered UI does not waive them. PWA-only avoids store review but not privacy or browser constraints. Apple is not an initial target; if added, guideline 4.2 requires utility beyond a repackaged website [S15]. Local encrypted history, offline maps, imports and native scheduling make a stronger app than a remote website shell, but approval is not guaranteed.

### Exact reuse boundary

Paths below are current files, not files changed by this design.

| Existing asset | Capacitor disposition | RN / Compose consequence |
|---|---|---|
| `frontend/src/types.ts` | Reuse activity, lap, stream, zone, calendar and analytics DTOs. Replace server-specific `AuthStatus`, `NetworkStatus` and sync-state assumptions in the eventual mobile contract; `garmin_method` currently omits the Personal API method | RN can share TypeScript; Compose needs equivalent Kotlin DTOs |
| `frontend/src/types_phase2.ts` | Reuse plan, physiology, fingerprint and comparison response shapes, units, nullability and verdict semantics | RN can share these; Compose translates them |
| `frontend/src/lib/api.ts` | Reuse public method signatures and response contracts, **not** the HTTP implementation. `BASE='/api/v1'`, `request()`, `fetch`, multipart FIT upload, password/MFA/developer-key endpoints and network-route operations are not the mobile data path | Even RN cannot reuse this unchanged for a local-first app; there is no server to call |
| `frontend/src/lib/format.ts` | Reuse pure formatting and sport-label helpers; audit date handling against activity-local dates, travel and DST | Reusable in RN; translate for Kotlin |
| `frontend/tailwind.config.js`, `frontend/src/index.css` | Retain palette, typography, radii, spacing, shadows and Leaflet styling; add mobile safe-area, font scaling and touch behavior later | Values survive RN/Compose, CSS implementation does not |
| `frontend/src/components/ui.tsx`, `ActivityCard.tsx`, `RouteThumb.tsx` | Retain DOM/SVG component implementations. RouteThumb is a schematic GPS trace with decorative background, **not** a street basemap | Rewrite native views/SVG in RN or Compose |
| `frontend/src/components/charts.tsx` | Retain `ChartFrame`, `VolumeChart`, `PaceTrendChart`, `StreamChart`, `ZoneBarChart`, `MonthlyBars`. Recharts renders in the WebView; touch tooltips and long-stream rendering require device proof | Recharts DOM implementation does not directly port to RN or Compose |
| `frontend/src/pages/ComparePage.tsx` | Retain selection, metric formatting, matrix and inline Recharts presentation. Its profile data still needs a local implementation of comparison service logic | Rewrite rendered UI in either native framework |
| `frontend/src/pages/ActivityPage.tsx` | Retain stat strip, laps, edit UI and chart composition. Replace external OSM `TileLayer` dependency with local tiles; use local repository methods for all reads/writes | Map/chart UI needs new implementations in RN/Compose |
| `frontend/src/pages/{DashboardPage,FeedPage,PlanPage,TrainingPage,PhysiologyPage,FitnessPage,StatsPage}.tsx` | Retain presentation and query call sites as their local methods become available. Each method must be implemented locally before its screen ships | Domain intent survives; native rendered screens are rewrites |
| `frontend/src/components/AppLayout.tsx` | Adapt desktop navigation into phone navigation, Android Back, safe areas and native sync events. Remove its 15/30-second server polling | Native navigation rewrite |
| `frontend/src/pages/SettingsPage.tsx` | Substantially replace server credential/network settings with native key setup entry, app lock, local import/export/delete, sync consent and capability status | New screen in every option |
| `frontend/src/App.tsx`, `main.tsx`, `frontend/vite.config.ts`, `frontend/index.html` | Retain React bootstrapping and routes with mobile lifecycle/router handling. Vite's development proxy is not packaged infrastructure. Remove Google Fonts requests; bundle licensed font assets or use system fonts | Native frameworks replace bootstrapping/build tooling |

TanStack Query remains a view cache, never the source of truth. Configure local reads to run offline rather than inheriting network-pausing defaults; on native commit events invalidate affected query keys. Do not persist a second health-data cache in localStorage, IndexedDB or service-worker caches.

### Proposed runtime boundary

```text
Bundled React UI + typed local repository facade
                 | narrow Capacitor commands / DTOs / change events
Kotlin application core
  native credential entry + lock gate
  Garmin HTTPS client + token cache
  bounded FIT decoder + normalization + domain calculations
  Room DAOs + SQLCipher + versioned migrations
  WorkManager + shared sync coordinator
                 |
Private credential-encrypted app storage, excluded from backup
```

Expose named operations such as `activities`, `activity`, `patchActivity`, `compareRuns`, `runSync`, `importFit` and plan operations. Do not expose arbitrary SQL, arbitrary URLs, filesystem paths, a token getter or a key getter to JavaScript. Native UI handles key entry; React gets connection state and redacted errors only. Initial native modules would live in a new Android project/core, with the existing frontend subsequently gaining a platform adapter; this document authorizes no implementation changes.

**Initial supported floor: Android 15 / API 35.** This deliberately narrows device coverage so unlocked-device-required Keystore keys can use the documented fixed behavior [S4]. Target the Play-required SDK current at release; minimum supported SDK and target SDK are different. Supporting Android 12-14 later needs a separately tested key-access policy, not silently weaker guarantees.

## 2. Security-first design

### Threat model and trust boundaries

Protect the Garmin credential, its bearer token, precise GPS history/home location, physiology, plans and annotations from a lost locked phone, ordinary other apps, backup/transfer leakage, accidental exports, malicious imports, network interception and injected WebView content. Also protect data integrity against process death, retries and failed migrations.

Trust Android's sandbox, verified OS and Keystore, a maintained system WebView, signed Stride releases and Garmin's HTTPS service. An unlocked rooted device, privileged instrumentation, compromised OS, malicious keyboard/accessibility service or compromised app release can observe plaintext or invoke authorized operations. Encryption is not a claim of safety on a compromised endpoint. No root check or remote attestation substitutes for this boundary.

**Literal constraint clarification:** the Personal API key must be transmitted to Garmin's token endpoint to authenticate. Therefore "never leaves the device" must mean **never sent to Stride, the desktop, telemetry or any party except Garmin over TLS**. If even Garmin must never receive it, the requested authentication flow is impossible. Confirm this wording in onboarding and the parallel key-flow design.

### Credential storage and bearer lifecycle

Use Android Keystore for a non-exportable AES-256-GCM wrapping key, **not as a general string store for the Garmin key**. Encrypt one credential envelope in private `noBackupFilesDir`: Personal API key, cached access token and expiry metadata. Use platform cryptographic APIs, fresh random GCM nonces for each encryption, authenticated format/version context and atomic replacement. Do not use deprecated `EncryptedSharedPreferences` for a new design [S3, S5]. Plain DataStore or SharedPreferences is not encryption.

Prefer hardware-backed Keystore, inspect the actual security level and surface software-only protection honestly; StrongBox is optional, not assumed [S3]. Separate aliases protect credential envelopes and the database data key. On API 35+, set unlocked-device-required on both. **Do not require an interactive biometric operation for each background unwrap.** Workers defer while the phone is locked. Persist nothing in device-protected/direct-boot storage.

The 24-hour bearer lifetime is a brief/repository assumption, not independently verified Garmin documentation. Current `ingest/garmin_personal.py` caches in process and uses a conservative 23-hour deadline. Mobile should persist its encrypted cache to avoid a new exchange at every worker restart. Use a documented server expiry if available; otherwise use a conservative 23-hour deadline from receipt, refreshing five minutes before that deadline when a request is actually needed. JWT `exp`, if decoded, is only an untrusted scheduling hint unless validated; never infer authorization from locally decoded scopes. Bound expiry by the conservative deadline, invalidate on suspicious clock rollback and refresh lazily.

Serialize refresh across foreground/background sync. On a resource `401`, exchange once and retry once; a second failure records `needs_reconnect` and stops automatic retries. A `403` is a permission/scope failure, not an infinite token-refresh trigger. Handle endpoint-specific capability denial visibly. Never assume there is a refresh token: refresh means exchange the original Personal API key again. Do not wake the phone solely to keep a bearer alive.

No key/token enters React state, WebView storage, WorkManager input/output/progress, URLs, logs, crash reports, notifications or exports. After native paste/save, discard the input buffer; never offer a reveal/copy endpoint. Clear the clipboard only if it still contains the pasted credential and the athlete consents; cannot retract keyboard history, prior clipboard sync or other apps' copies. The parallel onboarding flow must meet this native boundary.

Disconnect cancels queued/in-flight sync, advances a connection generation so late responses cannot commit, and removes the credential envelope/token. Retain history unless the athlete explicitly chooses delete. Explain that local disconnect does not revoke an already issued bearer at Garmin; provide Garmin-side revocation instructions without promising a revocation endpoint.

### Database, files and device theft

Use maintained `sqlcipher-android` integrated with Room, rather than plain SQLite or two competing JS/native SQLite plugins. The project documents Room integration [S10]; pin compatible versions during implementation and verify licensing, supported ABIs and Android 16 KB page-size compatibility. No compatibility claim is made without an actual APK.

Generate a random 256-bit database key; wrap it with its separate Keystore key, never derive it from the Garmin credential, device ID or a hard-coded password. Store the encrypted database and wrapped key under private no-backup storage. Credential rotation must not rekey or orphan activity history. Keep WAL/foreign-key behavior, but test SQLCipher WAL, rollback and temp-file handling so database contents are not written as plaintext sidecars. Do not emit SQL parameters in diagnostics.

Persist original FIT bytes only as independently authenticated encrypted blobs, using a distinct random file key wrapped under the database protection hierarchy. Store a file hash and parser version in the DB. This enables offline reprocessing when segmentation changes. Decrypt as a bounded stream; avoid plaintext temp files. Completed imports are independent of the source picker URI.

Set `allowBackup=false`, explicit legacy backup exclusions and Android 12+ `dataExtractionRules` exclusions for cloud and device transfer. Exclude database, WAL, wrapped keys, credentials, FIT blobs, caches and WebView data. `allowBackup=false` alone is insufficient on some manufacturers [S11]. Do not enroll cross-platform transfer. Verify the merged manifest and actual backup/restore behavior. Keystore keys do not form a portable recovery mechanism.

A stolen locked supported phone cannot normally unwrap these keys. **Unlocked-device-required does not erase a DB key or plaintext already in memory.** On screen lock, cancel network/decode work, close DB handles, clear view/query state and release key buffers; use best-effort buffer zeroing without claiming JVM/WebView memory erasure. Race windows and runtime compromise remain. Root warning is informative, not a destructive wipe or a claim that root detection is reliable.

Key invalidation/corrupt ciphertext must produce a recovery screen, never silently initialize an empty DB over history. Preserve encrypted files for explicit recovery/export decisions. Without a valid device key or an independent export, device loss/uninstall can permanently lose local plans and notes. Say that plainly.

### App lock: yes to screen lock, biometric optional

Require a secure Android device credential before persisting real data, including FIT-only use; no Garmin account is needed. Explain why and open Android settings rather than inventing an app PIN. Allow an empty/local demonstration mode without real data.

Default a native app gate on cold launch and return after 60 seconds in the background; lock immediately when the device locks. Use Android BiometricPrompt with device-credential fallback. Biometrics are a convenience, not a requirement to own compatible biometric hardware. Re-authenticate for export, deletion, replacing credentials and disabling the app gate. Never store an app password or biometric template.

The foreground gate is authorization enforced natively on repository commands, not just a React overlay. A narrowly scoped sync worker may write while the **device is unlocked** even if the foreground gate is closed. Thus this gate prevents casual access on a borrowed unlocked phone; it is **not an independently user-auth-bound database key**. It cannot defend against compromised native code. A stricter auth-per-use encryption mode would disable unattended sync; defer rather than promise incompatible properties.

Use `FLAG_SECURE` and a blank/native recents surface for real-data and credential screens. Notifications contain generic progress or reconnect text, not routes, HRV, activity names or health conclusions. Release builds disable WebView debugging and sensitive logging.

### WebView and network boundaries

Bundle all HTML/JS/CSS/fonts; no remote application URL, CDN scripts, remote hot updates, ad SDK, analytics, crash uploader or telemetry. Ship code updates through signed app releases. Use a restrictive CSP, no inline script/eval, block arbitrary frame/navigation loads and open approved external help/privacy links in the system browser without the bridge. Prevent file/content navigation into the privileged WebView.

Render notes and imported plan text as text, not HTML. Validate bridge method inputs, authorization, page sizes, IDs and result sizes natively. Keep authentication and sync allowlists separate from untrusted content. Even without a secret getter, injected JavaScript could read athlete DTOs or invoke writes; native validation and preventing script injection remain essential.

Only the native Garmin client makes credentialed requests. Use platform TLS certificate/hostname validation, disallow cleartext, do not bypass certificate errors or add user-controlled proxy URLs. No initial certificate pinning: rotation-related outages are not worth an unmaintained pin lifecycle here. Never forward authorization across a redirect to another origin; reject unexpected redirects until a verified necessary host is explicitly supported. Disable network disk caches for sensitive responses.

### Permissions, maps and what is not stored

Imported Garmin GPS coordinates need **no phone location permission**. Read FIT files through `ACTION_OPEN_DOCUMENT` / SAF; support file providers' `content://` URIs rather than assuming real paths [S12]. No all-files access, media-library, Bluetooth, body-sensor, Health Connect or background-location permissions are needed.

If phone-recorded GPS runs become a feature, build a user-started native `location` foreground service with appropriate while-in-use location permissions and persistent notification [S14]. WorkManager and a WebView geolocation watch are not run-recording engines. Do not request always-on location preemptively.

Current `ActivityPage.tsx` fetches public OSM tiles and `index.html` fetches Google Fonts. Both contradict a zero-third-party-network default. Disable those requests in mobile. OSM's public tile servers explicitly prohibit offline prefetch packages [S13]. An SVG trace is useful but must be labeled route-only; it does not satisfy an offline street-map claim. Use a licensed locally imported/generated tile pack, described in milestone below, and retain attribution. Map packages can disclose an athlete's region too.

Do not store Garmin passwords, MFA values, OAuth consumer secrets, Azure credentials, a second Personal API key in the athlete DB, full profile payloads, nutrition/weight data not used by Stride, advertising identifiers or telemetry. Keep only minimal profile/timezone and whitelisted provider fields required for domain calculations. Original FIT is deliberate retained source data, encrypted and deletable, not a debug dump. Local sync logs contain bounded redacted categories/statuses, not HTTP bodies or raw exception strings.

### Threat table

| Asset | Risk | Control and residual limit |
|---|---|---|
| Personal API key / bearer | File theft, JS exposure, backup leak | Keystore-wrapped single envelope, native-only requests, exclusions, no getters/logging; Garmin necessarily receives authentication |
| GPS, physiology, plans | Locked-phone theft and ordinary app access | Private SQLCipher DB, encrypted FIT/maps, device screen lock, unlocked-device key restriction; unlocked/rooted runtime remains exposed |
| Health screens | Borrowed unlocked phone, recents/screenshots | Native app gate, secure window, blank recents, generic notifications; accessibility/OS compromise not solved |
| DB integrity | Duplicate pulls, process death, migration failure | Unique provider identity, per-activity transactions, durable checkpoints, versioned non-destructive migrations, visible partial status |
| Web/native bridge | Untrusted JS invokes privileged methods | Bundled assets, CSP/navigation restrictions, narrow validated commands, native gate, no generic SQL/HTTP/file API |
| FIT / ZIP / map / plan imports | Resource exhaustion, malformed parser input, path traversal | Native bounded decoding, no archive path extraction, size/entry/sample limits, cancellation and explicit failure |
| Network credentials/data | MITM, redirects, error-body leakage | TLS, exact host policy, no cleartext or debug trust override, sanitized errors |
| Exports | Accidental sharing or lost device-bound keys | Explicit authenticated encrypted export, no credentials, independent passphrase; selected destination outside app control |
| Release artifact | Compromised dependency/signing/update | Minimal pinned dependencies, protected release signing, signed packaged assets, no remote code loading; supply-chain compromise remains high impact |

This table is a prospective architecture threat model, not a vulnerability audit of the existing server.

### Play data-safety and health disclosure

Google defines collection as transmitting data off the device; on-device health storage alone does not automatically mean developer collection [S16]. **Do not publish a blanket "no data collected" answer based only on SQLCipher.** Garmin authentication sends a credential and request metadata off-device, and any online map/font/SDK could send additional data. Audit the exact release's traffic and dependencies before completing the form. User-directed transfer exceptions, where applicable, must be evaluated against the full policy, not assumed from the word local.

Inventory fitness information, health information, precise location and account/authentication information. Explain optional Garmin access, direct communication with Garmin, local processing, no Stride servers, retention, deletion, backup exclusions and explicit exports in the privacy policy. Distinguish health data downloaded from Garmin from health data uploaded to a developer. Classification of authentication and any policy exemptions remains a release-time disclosure question, not resolved legal advice.

Complete the Health apps declaration and Data safety form for public/closed/open distribution as applicable; publish an active public non-PDF privacy policy and include it in-app. Health-related functionality still triggers the health policy even without Health Connect [S17]. Readiness and fatigue explanations are training estimates with visible inputs/confidence, not diagnosis or medical guarantees. Preserve missing-data warnings; never fabricate physiology. No account-deletion backend is needed for a nonexistent Stride account; provide local erase/disconnect and explain that Garmin-held data is managed at Garmin.

## 3. Local-first data model and sync

### Reuse the schema's meaning, not SQLAlchemy runtime or the server DB file

Port table semantics from `backend/app/models.py` and `models_phase2.py` into Room entities/explicit SQLite migrations. Keep integer primary keys for the existing UI contract, foreign keys, nullability, units and unique constraints. JSON arrays remain reasonable for this personal workload; do not explode each second into a separate row without measurement.

| Current tables / fields | Phone ownership and treatment |
|---|---|
| `athlete` | One local athlete, editable display name, timezone; no email/account requirement. Keep activity-local date distinct from UTC start instant |
| `activity`, `activity_lap`, `activity_stream`, `hr_zone` | Durable imported source facts. Preserve `running_time_s`, `walking_time_s`, `stopped_time_s`, `running_distance_m`, lap `elapsed_s`, units, null gaps and route geometry |
| `activity.raw` | Replace unrestricted debug payload retention with versioned whitelisted source facts; encrypted original FIT supports reprocessing |
| `daily_metric`, `daily_physiology` | Make `daily_physiology` canonical for imported recovery facts. Preserve meaningful legacy daily fields through migration/derived compatibility, not two independently writable physiology stores |
| `activity_fact`, `gear` | Locally stored provider facts and user gear. Actual singular table is `activity_fact`, despite plural wording in parts of DESIGN |
| `plan_block`, `plan_phase`, `plan_week`, `plan_session` | Authoritative local intent; manual/imported plans work without Garmin. Garmin calendar import is one-way provenance, not plan write-back |
| `plan_result`, `run_fingerprint`, `readiness` | Recomputable local derivatives with algorithm version and input revision. Invalidate when activities, physiology, plan, timezone or algorithms change |
| `sync_run` | Local bounded/redacted operational history. Add partial, paused, rate-limited, needs-reconnect and cancelled states rather than representing all nonfatal outcomes as success |

New local metadata: `schema_version`/Room migrations; provider connection generation and opaque account identity; sync cursor/window/page and durable per-item queue; `detail_state`/`fit_state` independent of GPS presence; source hash/revision and parser/algorithm version; per-activity local-edit overrides; suppression tombstones for deliberately deleted provider activities. Secrets stay outside this DB.

Keep uniqueness on `(source, external_id)` and use `garmin_personal` consistently for this path. Bind a connected provider account to a dataset: replacing a key for the same account preserves history; a different account must be explicitly rejected or trigger a separately confirmed dataset switch/reset. Never mix athletes silently. FIT import deduplicates by content hash and validated provider identity when available; filename alone is not a durable ID, and heuristic matches require review.

Keep user names/notes/effort overrides separate from provider fields so subsequent pulls cannot erase edits. Deleting an imported run creates a local suppression entry so the next pull does not resurrect it. No write-back to Garmin; Garmin deletions do not automatically erase the athlete's local history.

Normalize UTC instants, original activity-local date and timezone/offset provenance separately. Current `personal_sync._parse_dt()` attaches UTC to some local timestamps; do not reproduce that as a correct instant conversion. Respect actual activity dates, not the current phone timezone, when matching plans.

Domain ports are required for `ingest/normalize.py`, `fit_importer.py`, `plan_importer.py`, `services/personal_sync.py`, `persistence.py`, `analytics.py`, `plan_engine.py`, `fingerprint.py`, `readiness.py` and `comparison.py`. The relevant `api/activities.py`, `stats.py`, `misc.py`, `plan.py`, `physiology.py`, `fitness.py` and `compare.py` supply DTO-shaping reference, not a phone HTTP server. `db.py`'s WAL, foreign keys and migration intent survive; its SQLAlchemy engine and additive migration implementation do not. Server auth, scheduler, `network.py`, Key Vault access and demo auto-seeding do not ship.

Preserve running-only segmentation, cadence unit handling by sport/device, summary-versus-FIT precedence, missing sensor behavior and comparison's 120-point distance profiles. Port expected behavior with curated golden fixtures rather than blindly transcribing Python. In particular, align sparse streams by timestamp before mapping them to distance; independent filtered arrays are not guaranteed index-aligned. Pin regression cases for treadmill, walk-only, cycling, missing cadence/speed, stops and short/empty streams. Do not label ported analytics as verified until compared against reviewed fixtures.

### Desktop relationship: no live protocol in v1

**No automatic phone/desktop sync, no desktop connection, no database-file sharing.** Desktop remains an independent instance. Shared SQLite files, including WAL, are not a multi-device protocol. Pairing, conflict resolution, remote auth and LAN discovery are deliberately absent. Never send the phone's Garmin key to the desktop.

Offer explicit versioned data export/import after the vertical slice, before asking users to trust the app with irreplaceable notes/plans. Export a consistent transactional snapshot of normalized data, optional encrypted FIT sources and provenance, not Keystore aliases or credentials. Use a reviewed authenticated encrypted archive format with an independent passphrase and a mature KDF/library; choose and benchmark its concrete parameters during implementation rather than inventing crypto in this document. No silent plaintext JSON/ZIP backup.

Use SAF create/open document and authenticate before export. Explain that the chosen destination may itself be a cloud provider. Desktop compatibility requires a later reviewed importer; the current FastAPI app does not already understand this proposed archive. Restores validate version, limits and relations in staging, remap local integer IDs and require explicit replace/merge choices. Initial restore supports replace only; do not claim bidirectional merge. Password loss means export loss; no recovery service exists.

### Garmin scheduling and battery policy

1. Without a saved key, enqueue no Garmin work and make no Garmin requests. All local reads, plan edits/import and comparison continue. New installs show honest empty states; optional labeled sample data is never silently treated as real history.
2. After explicit Garmin/auto-sync consent, schedule one unique periodic WorkManager request at a **six-hour desired interval**, network connected and battery-not-low. WorkManager's minimum periodic interval is 15 minutes, but execution is inexact and power constrained [S6, S18]. Six hours is a product choice, not a platform guarantee. No exact alarms, permanent service, WebView timer or battery-optimization exemption request.
3. Foreground "Sync now" and on-open catch-up use the same Kotlin coordinator and queue. Rate-limit on-open checks to at most once per hour, unless explicitly requested. Serialize all Garmin pulls; use one unique queued request plus a native coordinator lock. WorkManager metadata contains opaque IDs only, not secrets/health records.
4. Workers do not decrypt while locked. Mark pending-unlock and enqueue catch-up when the app next resumes unlocked; a later periodic opportunity may also proceed unlocked. No promise of exact sync immediately on every unlock or while force-stopped. After reboot, work awaits ordinary unlock/OS opportunity. Show last successful sync and freshness.
5. First import fixes an inclusive **30 local-calendar-day window: today and the preceding 29 days**, paginates until exhausted and persists discovered item IDs. Do not copy `personal_sync.py`'s single `limit=200` request as complete history. Incremental sync rechecks a three-day overlap for late uploads/edits; weekly lightweight 30-day reconciliation catches older metadata edits. Manual backfill is explicit.
6. Fetch summaries, then required details and original FIT sequentially with bounded batch time. Commit each activity and its laps/streams/zones atomically; incomplete items remain retryable. Use explicit import completeness/hash, not `geo_points && streams`, to avoid redownloading treadmill activities forever. Do not skip every future summary merely because a FIT is cached.
7. Checkpoint batches before normal worker execution limits, initially target no more than two minutes per batch, and continue through persisted queue work. User can cancel; process death does not mark missing FIT as complete. Avoid a long-running foreground worker for the initial version. Long-running WorkManager uses foreground-service machinery and has additional restrictions [S19]; do not assume unlimited background time.
8. On `429`, persist a global `not_before` from `Retry-After` (seconds or HTTP-date); otherwise exponential cooldown starting at one minute, capped at six hours, with positive jitter. Do not shorten a longer server deadline. Schedule delayed continuation rather than sleeping a worker. Every entry point respects the cooldown. Use WorkManager exponential retries for transient transport/5xx failures; cap attempts and leave a visible resumable error. No IP-route switching or immediate refresh storm.
9. No blanket success on detail/FIT failures. List imported summary counts separately from fully offline-ready activities and display retry actions. Unavailable wellness/scope endpoints report capability state; local missing data remains null.

Routine sync needs no notification. Optional completion/reconnect notifications use a low-importance channel and Android 13+ runtime permission requested in context; refusal never disables import/browsing [S20]. Show progress/cancel in-app. Plan reminders are approximate local notifications, not an excuse to request exact-alarm access.

## 4. First milestone: prove the native core, not just the shell

**Deliverable: a signed installable Android 15+ APK that accepts the native Garmin setup flow, fully imports the last 30 days, and cold-opens a real activity's map and charts in airplane mode without a desktop.** No work in this milestone is claimed to have been built here.

### Build scope

| Component | Must exist in the slice |
|---|---|
| Packaged UI | Bundled Vite assets/system fonts; phone feed, activity detail, errors/empty states; Android Back and safe areas; no server URL |
| Native core | Narrow typed Capacitor bridge; native gate; Keystore envelopes; Room/SQLCipher migrations; local list/detail DTOs |
| Garmin integration | Consume the parallel native key-entry flow; HTTPS exchange/cache; paginated 30-day discovery; summary/detail/FIT download; explicit rate limit/reconnect/partial status |
| Decoder | Java-compatible FIT decoder candidate from Garmin's FIT SDK documentation [S21], subject to SDK license and Android compatibility verification; Kotlin normalization plus segmentation/lap/stream ports. Do not confuse the Health/Connect IQ SDK with a FIT file decoder |
| Persistence | Atomic per-activity commits, encrypted retained FIT, explicit completeness, resumable cursor/queue, idempotency and cancellation |
| Offline activity | Existing Recharts stream views and lap table reading native DTOs; panning/zooming route with start/end and **a real offline basemap** for the test region |
| Scheduler | Same import engine callable from UI and a chunked WorkManager job, demonstrated with the WebView destroyed; periodic six-hour scheduling plus lock/cooldown behavior |
| No-key path | Skip Garmin, launch offline, select a real local FIT via SAF and browse the resulting activity; no credential or phone-location permission required |

For the basemap, use a small test-region tile archive generated from legally obtained OSM data with the applicable attribution/license, or a local pack whose license explicitly permits redistribution/offline use. Import it via native SAF; serve tiles from an exact scoped local asset handler into Leaflet, never from a generic file-reading bridge. Encrypt the pack at rest and enforce read-only bounded tile addressing. **Selecting a reproducible licensed pack and proving the local tile handler is part of the slice, not an assumed dependency.** Never bulk-fetch `tile.openstreetmap.org`.

A labeled tile-free route trace remains available outside installed coverage and when no pack exists, without network fallback. That fallback must not be presented as satisfying the milestone's offline-basemap test. The initial APK need not bundle the world's maps or add a paid map service.

### Acceptance tests on real output

- Fresh install on the athlete's Android 15+ phone: setup device lock, skip Garmin and import a FIT; feed/detail work in airplane mode. No hidden HTTP dependency or mandatory cloud login.
- Connect with the native key flow; confirm against the Garmin list that **all pages in the exact 30-day window** were visited. Report actual discovered/imported/offline-complete counts rather than inventing a fixed activity count. Rerun: no duplicates, annotations preserved. If the window has no GPS run, explicitly import a real GPS FIT for the map check; do not fabricate a route.
- Demonstrate real HR/pace/elevation charts where the file has sensors, laps and an attributed offline basemap/route. Revoke network, terminate the process, cold launch, unlock, open a previously unopened downloaded activity and pan/zoom within the stored map region. A warm WebView cache is not proof.
- Destroy the WebView during a worker import. Reopen and confirm native progress/commits. Separately kill the process mid-batch, reboot, force-stop and resume: no corruption, no duplicate children, documented delays and explicit resume after force-stop.
- Lock during import: no subsequent protected reads/decryption after cancellation completes; unlock/resume continues from a checkpoint. Check a second supported device/OEM before broader release.
- Exercise controlled `401`, denied scope, `429` with both Retry-After forms, offline/5xx, corrupt ZIP/FIT, no-GPS activity, missing sensors and key invalidation. Assert visible state, bounded retries, no data loss. Use test transport fixtures, not extra live Garmin authentication attempts.
- Inspect app-private files, WAL/temp files, WebView storage, WorkManager DB and logcat for plaintext sentinel secrets/data. Confirm plain SQLite cannot open the encrypted DB. Inspect the final manifest/backup rules and perform backup/restore checks; confirm no Google Fonts, OSM tile, telemetry or Stride-server traffic. Instrumented inspection is required, not just code review.
- Measure import wall time, request count, bytes, DB/FIT size, worker duration, memory, battery behavior and chart gestures on the real phone. Proposed UX budgets: activity opens from local storage in under two seconds and chart/route interactions remain responsive on a representative two-hour run; these are acceptance targets, not measured results.

### Deferred, but required before calling the whole app complete

Full local ports for dashboard/stats/calendar, plan import/edit/verdicts, physiology/readiness, fingerprints and multi-run comparison follow the ingestion slice. They must work without Garmin, including imported/local data and explicit missing physiology. Do not ship original routes that silently call FastAPI or show empty fake-success screens. Preserve the Plan -> Execute -> Understand -> Adapt loop; the slice is a technical proof, not the finished product.

Defer desktop live sync, Garmin write-back, password/OAuth fallback, Health Connect, direct watch Bluetooth, phone GPS recording, social/cloud accounts, push infrastructure, global offline map distribution, iOS, remote JS updates and a native UI rewrite. Encrypted export/restore and a release-ready local delete flow are required before a public release with irreplaceable user-entered data; do not defer recovery indefinitely.

### What could invalidate the decision

**Biggest risk: recreating the Python ingest/analytics semantics as a resumable native core, not packaging React.** FIT decoder compatibility/license, sparse-stream alignment, run/walk segmentation and local-date interpretation can silently change the product's conclusions. Reviewed fixture parity and real-device imports are go/no-go gates.

If Recharts/Leaflet fail the measured device responsiveness, accessibility or touch bar after bounded rendering/downsampling work, move the affected chart/map surface native first; reconsider RN/Compose only with measured evidence. If SQLCipher/Room cannot meet ABI/page-size, lifecycle or migration requirements, do not ship an unencrypted fallback. If native app-lock policy cannot coexist with the required sync UX, explicitly choose stronger foreground-only access or a revised documented policy; do not add a second unprotected key.

If Garmin mobile access, API terms, credential availability or public distribution rights do not permit this use, stop the Garmin-connected launch and retain the FIT-only local product. If a licensed offline basemap cannot be reproduced, the map milestone has not passed. If the athlete's phone is below Android 15, the initial platform-floor decision must be revisited before implementation.

## 5. Sources, evidence and verification limits

All source access/research below occurred on **2026-10-23**. Platform statements use official project/platform documentation unless marked MDN. Access dates describe this investigation, not guarantees of future policy.

| ID | Source | Used for |
|---|---|---|
| S1 | [Capacitor Android Plugin Guide](https://capacitorjs.com/docs/plugins/android) | Java/Kotlin plugins, native bridge methods, lifecycle/navigation integration |
| S2 | [Capacitor Background Runner](https://capacitorjs.com/docs/apis/background-runner) | WebView unavailable for background events; headless runner is separate from UI. Not selected as the DB/sync owner |
| S3 | [Android Keystore system](https://developer.android.com/privacy-and-security/keystore) | Non-exportable keys, hardware support/security levels, compromised-process limits |
| S4 | [KeyGenParameterSpec.Builder](https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec.Builder) | Unlocked-device versus user-auth requirements; documented Android 12-14 issues fixed in Android 15 |
| S5 | [EncryptedSharedPreferences API reference](https://developer.android.com/reference/androidx/security/crypto/EncryptedSharedPreferences) | Deprecation; not a recommended new storage dependency |
| S6 | [Expo BackgroundTask](https://docs.expo.dev/versions/latest/sdk/background-task) | Android WorkManager basis, 15-minute minimum and inexact execution |
| S7 | [Expo SQLite](https://docs.expo.dev/versions/latest/sdk/sqlite) | Persistent native SQLite and SQLCipher build configuration; not evidence current Stride UI ports to RN |
| S8 | [Tauri 2.0 stable release](https://v2.tauri.app/blog/tauri-20) | OS WebView/Rust architecture and native mobile plugin support. Historical release notes, not a current plugin compatibility matrix |
| S9 | [MDN PWA caching](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Caching) | Offline assets/service workers, browser cache management/eviction. No claim that browser storage can never be durable |
| S10 | [SQLCipher for Android](https://github.com/sqlcipher/sqlcipher-android) | Android library, Room integration and logging controls; final dependency/ABI compatibility untested |
| S11 | [Android Auto Backup](https://developer.android.com/guide/topics/data/autobackup), [Android 12 backup behavior changes](https://developer.android.com/about/versions/12/behavior-changes-12) | Default backup coverage, no-backup storage and cloud versus D2D exclusions |
| S12 | [Android Storage Access Framework](https://developer.android.com/guide/topics/providers/document-provider) | User-selected document access/import/export and content URIs |
| S13 | [OSMF Tile Usage Policy](https://operations.osmfoundation.org/policies/tiles) | Public raster tile attribution/caching rules and prohibition on offline prefetch |
| S14 | [Android foreground service types](https://developer.android.com/about/versions/14/changes/fgs-types-required), [Launch a foreground service](https://developer.android.com/develop/background-work/services/fgs/launch) | Future user-started location recorder permissions/types; not required for imported GPS |
| S15 | [Apple App Review Guidelines, 4.2](https://developer.apple.com/app-store/review/guidelines) | Minimum functionality beyond repackaged websites; page reported update 2026-10-16 |
| S16 | [Google Play Data safety guidance](https://support.google.com/googleplay/android-developer/answer/10787469?hl=en) | Off-device collection definition, developer responsibility, health/fitness data categories and distribution-track requirements |
| S17 | [Google Play Health Content and Services](https://support.google.com/googleplay/android-developer/answer/16679511?hl=en) | Health declaration, privacy policy, minimum permissions and limits on health claims |
| S18 | [Android: define work requests](https://developer.android.com/develop/background-work/background-tasks/persistent/getting-started/define-work) | Constraints, periodic work, retry/backoff and deferred execution |
| S19 | [Android long-running workers](https://developer.android.com/develop/background-work/background-tasks/persistent/how-to/long-running) | Foreground-service/notification implications of work beyond ordinary worker limits |
| S20 | [Android notification permission](https://developer.android.com/develop/ui/compose/notifications/notification-permission) | Android 13+ POST_NOTIFICATIONS behavior |
| S21 | [Garmin FIT SDK cookbook](https://developer.garmin.com/fit/cookbook) | Official FIT decoding reference; SDK binary, Android integration and redistribution license not validated |

Repository evidence: `README.md`, `DESIGN.md` including Personal API/segmentation/comparison updates; both ORM model files; `db.py`; frontend source inventory and contracts; the typed API client, activity/map/chart/layout/comparison implementations, styles/build entrypoints; Personal API client and sync implementation; FIT and comparison service logic. Documentation is inconsistent in places: earlier DESIGN health-account requirements are superseded by its later Personal API section; README test counts are historical, not rerun evidence.

**Not verified:** no APK, emulator, physical-device run, native dependency resolution, FIT SDK license review, offline tile-pack artifact, battery benchmark, public Garmin Personal API specification/24-hour contract, actual Garmin API call from this task, Play submission decision or current target-SDK deadline. Search did not yield a dependable public Personal API specification, so endpoints/lifetime are attributed to the brief and existing code rather than presented as independently confirmed platform facts. Some long web extracts omitted sections; no final data-safety classification or legal conclusion is inferred from those omissions.

Only this decision document was created for this task. No frontend/backend source, dependencies, builds or running services were changed.
