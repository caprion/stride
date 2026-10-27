# Packaging and deploying the Stride Android app

Pure Android, no server. The app is the data source; the athlete's history never
leaves the phone unless they export it.

**This is the build and ship plan for the v1 described in
`docs/android-v1.md`.** Decisions here are the ones that were genuinely open.

---

## 1. The shape of the APK

```
com.stride.running  (arm64-v8a only)
├── WebView                 the existing React SPA, bundled as assets
├── Kotlin core             Garmin client · FIT parser · engines · storage · sync
├── SQLCipher               one ABI
└── no Chromium, no Python, no node_modules
```

**Capacitor 8** (released May 2026) — `minSdkVersion` 24, AGP 8.14.3, and
`targetSdk` 36, which is what Play now requires. Capacitor 7 needs SDK 35;
we want 36 anyway.

| Component | Size |
|---|---:|
| Capacitor shell + resources | ~1.2 MB |
| Web bundle (JS 280 KB + CSS 12 KB gzipped) | ~0.3 MB |
| Kotlin core (our classes) | ~0.2 MB |
| SQLCipher, one ABI | ~2.4 MB |
| **APK** | **~4.8 MB** |

One ABI only: `armeabi-v7a` and `x86_64` would add ~4.7 MB and leave nothing
spare. `x86_64` is built as a **debug-only** variant for the emulator and is
never published.

---

## 2. How the Kotlin core talks to the SPA

Two options. I recommend the first.

### (a) A Capacitor plugin bridge — **recommended**

The Kotlin core registers a plugin; the SPA calls it through a thin adapter in
`src/lib/api.ts`. The SPA keeps one `api` surface with two backends:

```
src/lib/api.ts
├── apiHttp      → fetch('/api/v1/…')            the desktop app
└── apiBridge    → Capacitor.Plugins.Stride.*   the Android app
```

The bridge is a **narrow typed surface**, not a generic database port. Per
`docs/harness-integration.md`: no "run this SQL" method, no file reader, no
shell. Just the same operations the REST API exposes, with the same shapes.

**Why:** no listening port to secure, no local HTTP server to attack, and the
security boundary is the bridge's method list — which is auditable.

**Cost:** one adapter in the frontend, and the Kotlin side must expose roughly
40 methods matching the REST surface.

### (b) The core serves the API on loopback

The Kotlin core runs a tiny HTTP server on `127.0.0.1` and serves both the
bundled assets and `/api/v1`, so the SPA is 100 % unchanged. `http://127.0.0.1`
is a *potentially trustworthy* origin, so a `https://localhost` WebView may call
it without mixed-content problems.

**Why not:** it opens a port and needs its own auth, CSP and CSRF story —
exactly the surface we spent the last phase closing on the desktop.

---

## 3. What has to be ported to Kotlin

Everything currently in Python that the phone must own. Rough sizes:

| Module | Lines | Notes |
|---|---:|---|
| Garmin Personal API client | ~300 | token exchange, refresh, 8 endpoints, the 24 h token cache |
| FIT parser | ~500 | definition + data messages, records/laps/session, **semicircle positions**, **halved running cadence** |
| Segmentation | ~200 | run / walk / stand classification, running-only laps |
| Fingerprint | ~300 | cadence CV and drift, HR drift, decoupling, efficiency, form score |
| Readiness | ~250 | 7 drivers, renormalised over what exists |
| Plan engine | ~700 | importer, matching, verdicts, causes, adherence |
| Storage (Room + SQLCipher) | ~800 | 17 tables, migrations, the 0640-file discipline |
| WorkManager sync | ~250 | periodic pull, 429 backoff, resumable imports |
| Bridge plugin | ~400 | the typed method surface |
| **Total** | **~3,700** | |

The FIT parser and the two Garmin quirks are the parts worth writing
twice — everything else is mechanical.

**The desktop Python app stays**, for the athlete who wants a big screen. The
two must share the schema exactly, so the phone can export and the desktop can
import (and vice versa) — the `snapshot.sh` model already does this.

---

## 4. Repository layout

```
stride/
├── frontend/                    the SPA — shared by desktop and mobile
│   ├── src/lib/api.ts           two backends: http | bridge
│   └── …
├── android/                     the Capacitor project
│   ├── app/
│   │   ├── src/main/java/app/stride/
│   │   │   ├── MainActivity.kt
│   │   │   ├── core/           garmin · fit · segmentation · engines
│   │   │   ├── data/           room · sqlcipher · migrations
│   │   │   ├── sync/           workmanager
│   │   │   └── bridge/         the capacitor plugin
│   │   └── build.gradle.kts
│   ├── capacitor.settings.gradle
│   └── build.gradle.kts
├── backend/                     the desktop server (unchanged)
└── scripts/
```

`capacitor.config.ts`:

```ts
const config: CapacitorConfig = {
  appId: 'app.stride.running',
  appName: 'Stride',
  webDir: '../frontend/dist',
  android: { allowMixedContent: false },
  server: { androidScheme: 'https' },   // the WebView origin is https://localhost
}
```

---

## 5. Build variants

| Variant | Web assets | Backend | Purpose |
|---|---|---|---|
| `debug` | loaded from the Vite dev server (hot reload) | the desktop instance on :8000 | development |
| `release` | bundled in the APK | the on-device Kotlin core | the real thing |

A Gradle source set switches the API base: in debug the bridge is a thin proxy
to the desktop API; in release it calls the on-device core. The SPA never knows
the difference.

---

## 6. Signing

- Two keys: an **upload key** (yours) and the **app signing key** (Google's, in
  Play App Signing). Keep the upload keystore in CI secrets, never in the repo.
- `.gitignore`: `*.jks`, `*.keystore`, `keystore.properties`, `google-services.json`.
- The release build is reproducible from CI: `./gradlew bundleRelease` produces
  `app/build/outputs/bundle/release/app-release.aab`.

---

## 7. CI

```yaml
# .github/workflows/android.yml
name: android
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 22 }
      - uses: actions/setup-java@v4
        with: { distribution: temurin, java-version: 17 }
      - run: npm ci --prefix frontend
      - run: npm run typecheck --prefix frontend
      - run: npm run build --prefix frontend
      - name: security gate
        run: ./scripts/security_check.sh
      - uses: gradle/actions/setup-gradle@v4
      - run: ./gradlew test
      - run: ./gradlew assembleDebug assembleRelease
      - uses: actions/upload-artifact@v4
        with: { name: stride-apk, path: android/app/build/outputs/apk/release/ }
```

**The security gate runs in CI.** It is the same script we run by hand, and it
is a required phase in `docs/working-agreement.md` — so it gates the pipeline
too, not just the conversation.

---

## 8. Deploying it

### Now — for the athlete and the circle

| Channel | Use | Lead time |
|---|---|---|
| **Firebase App Distribution** | the athlete, friends, testers | minutes, no review |
| **GitHub Releases** | the APK as an artefact; good for the showcase | minutes |
| **Sideload** | the athlete's own phone | seconds |

### Later — the public Play Store

The gatekeeping to plan around:

1. **A personal developer account ($25) cannot publish immediately.** Accounts
   created after 13 November 2023 must run a **closed test with at least 12
   testers opted in continuously for 14 days** before they can apply for
   production access. An **organisation account does not have this
   requirement** — worth deciding early, because it is the long pole and
   *cannot be rushed*.
2. **Target API 36 (Android 16).** Required for new apps and updates since
   31 August 2026.
3. **Health apps declaration.** An app that tracks health must complete
   Play's health-apps declaration.
4. **Data safety form.** Declare: health data, and that it is transmitted off
   the device — to **Garmin** — and stored locally by us. Also declare the
   credential (the Personal API key) and that we do not collect it ourselves.
5. **Privacy policy** — a URL, required, and it must actually describe what
   we do (data stays local; sharing is Garmin's).
6. **Permissions** — `INTERNET`, `ACCESS_NETWORK_STATE`,
   `POST_NOTIFICATIONS` (sync status), and WorkManager's `WAKE_LOCK`/
   `FOREGROUND_SERVICE`. `usesCleartextTraffic=false` plus a network security
   config that permits cleartext **only** to `127.0.0.1`.
7. **"Webviews and website spam" policy.** A WebView shell is allowed, but the
   listing must show value beyond a wrapped site. Ours has native sync,
   offline access, Keystore storage and a local database — a reviewer can see
   that from the listing. Make the screenshots show the *native* value (the
   dashboard, the plan engine), not a browser chrome.

### The release checklist

```bash
./scripts/security_check.sh          # the gate
./gradlew test                      # Kotlin unit tests
# and by hand:
#   - install the release APK
#   - paste the Garmin key in the native field
#   - pull 30 days
#   - enable airplane mode
#   - open an activity: the route, the splits and the charts all render
#   - confirm the key is unreadable to the WebView (adb shell run-as …)
#   - confirm zero third-party requests (the same rule we enforce on the web)
```

---

## 9. The migration story

- The phone's schema version must match the desktop's, and the Kotlin side owns
  the migrations (the desktop has an additive-migration helper today).
- A database written by a newer build must be **refused**, not silently
  mis-parsed.
- "Export my data" on the phone is the `snapshot.sh` model: a file the athlete
  owns, with the database plus the state file, encrypted.

---

## 10. The order to build it

1. **The shell.** Capacitor project, the SPA bundled, the bridge plumbing, and
   a stubbed core that returns canned data. Proves the packaging and the size
   budget before any domain logic is ported.
2. **The bridge contract.** Freeze the method list — it is the same contract
   the REST API already has, and freezing it now means the Kotlin port and the
   SPA adapter can proceed in parallel.
3. **Garmin + storage.** The client, the key in the Keystore, SQLCipher, and
   one activity in, one activity out. This is the risky part; do it early.
4. **The FIT parser and the engines.** The ports, with the Kotlin unit tests
   written against the same fixtures as the Python ones.
5. **Sync.** WorkManager, the 24-hour token cache, resumable imports.
6. **Packaging, signing, CI.**
7. **Start the Play closed-test clock** — it runs 14 days and cannot be
   compressed, so it should start while 3–6 are still in progress.

---

## 11. The one thing that could sink this

**The port.** Not the packaging, not the store — the port of the FIT parser and
the two engines. The FIT quirks are the kind of thing that looks right and is
wrong by a factor of two: positions in semicircles, running cadence stored
halved, lap speed sometimes absent. A port that gets them wrong produces
plausible-looking numbers that are not the athlete's.

Mitigation, and it is not optional: **port the Python tests, not the Python
code.** The segmentation, fingerprint and readiness suites become the Kotlin
suite, running against the same fixtures, with the same expected values. If the
numbers do not match, the port is not done.
