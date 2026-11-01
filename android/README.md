# Stride — the Android phone shell

The phone is the product. This directory is the shell around it: a
**Capacitor 8** WebView that hosts the web app, and the **Kotlin** that
owns everything the page must not touch — the credential, the database, the
Garmin pull and the background work.

## The rules this shell keeps

| rule | where it is kept |
|---|---|
| The Garmin key is typed into a **native** field and never enters the WebView | `setup/GarminSetupActivity.kt`, `bridge/PhoneRoutes.kt` |
| The key is wrapped by a **non-exportable** Keystore AES-GCM key and lives in `noBackupFilesDir` | `security/KeystoreKeys.kt`, `security/EnvelopeFile.kt` |
| The bridge has **one** method, `apiCall(path, body)`, and may call the frozen routes only | `bridge/StrideApiPlugin.kt`, `bridge/PhoneRoutes.kt` |
| The Garmin-key routes are refused, **including the query-string forms** | `bridge/PhoneRoutes.kt` (and its test) |
| A locked screen gets no real data | `bridge/StrideApiPlugin.kt`, `security/AppLock.kt` |
| The pull is **resumable** and a 401 gets **exactly one** re-exchange and one retry | `sync/SyncEngine.kt`, `sync/PullPolicy.kt` |
| "Synced 12 activities" is the notification, and it says nothing about the athlete | `sync/SyncNotifications.kt` |
| `FLAG_SECURE`, `allowBackup=false`, backup rules, minimal intent filters, nothing exported but the launcher | `MainActivity.kt`, `AndroidManifest.xml`, `res/xml/` |
| `unsafe` is allowed in the FFI seam and nowhere else | `app/src/main/rust/stride-ffi/` |

## The bridge the web app talks to

The web app is the same app on both hosts: the desktop rig talks to the
router over loopback HTTP, and the phone talks to the Rust core in the same
process. The app does not know which — it calls one function, and the
transport behind it differs.

**The shell installs the shim.** `app/src/main/assets/public/stride-bridge.js`
is the phone's half of that seam:

```js
window.__stride_bridge__.apiCall({ method, path, body })  // → Promise<string>
```

Capacitor injects the file into the page it serves — after its own runtime,
so the plugin proxy the shim calls already exists, and **before the app's
own scripts**, so the app's first request finds the bridge. The native side
asks for the injection in `StrideApiPlugin.load()`.

The shim does two things and nothing else: it moves the `method` into the
body (the plugin's contract is `apiCall(path, body)`), and it hands the
plugin's answer back. **It adds no capability**: the route table, the
refusal of the Garmin-key routes, the lock check and the redaction are all
in the plugin it calls.

The answer the app is handed is the **response body**, as a JSON string —
the same bytes the desktop's router puts on the wire — so the two hosts
cannot be told apart by the app. A failure is a rejection, not a body.

## The layout

```
android/
├── app/
│   ├── build.gradle              the app, the Rust build, the release rule
│   └── src/
│       ├── main/
│       │   ├── AndroidManifest.xml
│       │   ├── java/app/stride/running/
│       │   │   ├── MainActivity.kt      the WebView, the gate, the setup entry
│       │   │   ├── bridge/              the one method, the route table, the redaction
│       │   │   ├── core/                the Rust core, in this process
│       │   │   ├── security/            the Keystore, the envelopes, the gate
│       │   │   ├── settings/            the phone's settings
│       │   │   ├── setup/               the native Garmin screen
│       │   │   └── sync/                the worker, the scheduler, the notifications
│       │   ├── res/                     the screen, the strings, the icons, the backup rules
│       │   └── rust/
│       │       ├── stride-ffi/           the UniFFI seam and the panic boundary
│       │       └── stride-bindgen/       the Kotlin binding generator
│       └── test/                        the unit tests
├── fetch-capacitor.sh            put the Capacitor Android library where Gradle finds it
├── sync-web.sh                   copy the web bundle into the APK's assets
└── gradlew                       Gradle, without a binary jar in the tree
```

## Building it

The Capacitor Android library is an **npm package**, not a Maven artifact,
so it has to be on disk before Gradle can see it:

```bash
cd android
./fetch-capacitor.sh          # installs @capacitor/android into android/.capacitor
```

The web bundle is the frontend's build output, copied into the APK's
assets:

```bash
./sync-web.sh ../frontend/dist
```

Then the build, under the host's heavy lock:

```bash
flock /run/user/$(id -u)/cf-heavy.lock ./gradlew assembleDebug
flock /run/user/$(id -u)/cf-heavy.lock ./gradlew testDebugUnitTest
```

`assembleDebug` also cross-compiles the Rust core for `arm64-v8a` with
`cargo-ndk`, using the **shared** target directory
`/home/sumit/stride-v2/target` — so the crates are compiled once for the
whole fleet rather than once per worktree.
`-PstrideTargetDir=…` (or `STRIDE_CARGO_TARGET_DIR=…`) moves it, and a
checkout that has no shared directory falls back to its own `target/`.

### Why there is no `gradle-wrapper.jar`

The wrapper is normally a script plus a small binary jar, and the
repository's own pull-request scanner (`scripts/scan-pr.sh`) fails a change
that adds a binary file outside the corpus. So `gradlew` here is a shell
script: it reads the distribution out of
`gradle/wrapper/gradle-wrapper.properties`, finds it in the Gradle cache,
downloads it if it is not there, and runs it. The version is pinned in
exactly the same place as the wrapper pins it.

## The Rust seam

The core is `stride-api` in the workspace at the repository root. It is not
in the tree yet, so the seam is built against a **stub** that answers every
route with a sentence saying so — a stub can refuse, but it cannot be
wrong. The release build refuses to be assembled against the stub.

The seam is six functions, all of them answering with the same JSON
envelope, and a panic boundary in front of every one of them: a panic in the
core becomes an error the app can show, never a process kill. The panic's
*payload* is not logged — a payload is whatever the failing code was
holding, and that might be the key.

The crate is a **member of the repository's workspace**, so the root gate
sees it: `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings` and `cargo test --workspace` all cover the
seam.

**The retry rule has one owner: the shell.** The Garmin client performs
exactly one token exchange and one HTTP attempt per call and answers the
typed failure `needs_reconnect` when the token has expired; the caller —
`PullPolicy` — may make **one** more attempt, and then it stops. See
`docs/contract/core-contract.md` §7.

See `app/src/main/rust/stride-ffi/README.md` for what the core has to
provide.

## The tests

The unit tests are plain JVM tests and need no phone:

```bash
flock /run/user/$(id -u)/cf-heavy.lock ./gradlew testDebugUnitTest
```

They cover the route table (including the query-string forms of the key
routes), the redaction of an answer that carries the key, the one-retry
rule, the setup screen's states and the Garmin key page's host.

The **bridge shim is tested by running it**: `StrideBridgeShimTest`
evaluates the shipped `stride-bridge.js` on a JavaScript engine (Rhino,
test-only) against a fake plugin, and asserts what the web app would see —
the app's `{ method, path, body }` going to the plugin with the method in
the body, the plugin's JSON string coming back, and a failure coming back
as a failure.
