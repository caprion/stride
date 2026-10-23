# Stride for Android — v1: the small, secure, Garmin-only app

The brief: **under 10 MB, connect with Garmin, as simple as possible, still
differentiated.** This is the cut-down plan. It supersedes the full
`android-app.md` for what actually ships first.

---

## 1. The size budget is the design

10 MB is not a target to hit at the end — it is the constraint that decides the
architecture. Everything below is sized.

| Component | arm64-v8a | Notes |
|---|---:|---|
| Capacitor shell + AndroidManifest + resources | ~1.2 MB | The WebView is a *system* component, so no Chromium is bundled |
| Our web bundle (JS 280 KB + CSS 12 KB, gzipped) | ~0.3 MB | The whole SPA. `node_modules` (203 MB) never ships |
| Kotlin core (Garmin client, Keystore, WorkManager) | ~0.2 MB | Our own classes |
| SQLCipher native library | ~2.4 MB | One ABI only |
| FIT parser (optional, see §4) | ~0.15 MB | Kotlin classes only |
| **Total** | **~4.3 MB** | |
| Release signing, Play Store metadata, `resources` | ~0.5 MB | |
| **APK** | **~4.8 MB** | Comfortably under 10 MB |

**The ABI decision that buys the headroom:** ship **arm64-v8a only** in the
release build. Adding `armeabi-v7a` (+2.2 MB) and `x86_64` (+2.5 MB) for
emulators would take the APK to ~9.5 MB — under the limit, but with nothing
spare. So: **release builds are arm64 only; an x86_64 debug variant exists for
the emulator and is never published.** Nearly every device in use is arm64.

**What would blow the budget** (and therefore does not happen):
- Bundling Python (Chaquopy) — +15–30 MB. Out.
- Bundling a Chromium WebView — +40 MB. Out; use the system WebView.
- Shipping `node_modules` — obviously out; only the built bundle ships.
- Two or three database engines (SQLite *and* SQLCipher *and* a JS store) — one,
  SQLCipher.

---

## 2. What v1 includes, and what it does not

The differentiator is **plan vs actual, with causes** — that is what Garmin
Connect, Strava, Runna and TrainingPeaks each fail to do. Everything in v1
serves it; everything that does not serve it is cut.

| Feature | v1 | Why |
|---|---|---|
| **Garmin connect** (Personal API key) | ✅ | the whole point |
| **Today card** — plan, readiness, one recommendation | ✅ | *the* differentiator, and it is the app's home screen |
| **Plan import** (paste a markdown plan) | ✅ | local, no dependencies, and it is what makes plan-vs-actual possible |
| **Plan vs actual** — verdict + cause per session | ✅ | the differentiator |
| **Activity list + detail** (splits, HR, cadence per km) | ✅ | from the API's `splitSummaries` — no FIT needed |
| **Physiology** — sleep, RHR, stress, body battery → readiness | ✅ | from the wellness endpoints; needed for the causes |
| **Weekly volume / pace trend / records** | ✅ | aggregate, from splits |
| **Per-lap fingerprint** — cadence stability, HR drift, decoupling | ✅ | from per-lap `averageHR` / `avgStepFrequency` / `averageSpeed` |
| **Run comparison** (two runs on a distance axis) | ✅ | from splits; a strong standalone feature |
| Offline access to everything already pulled | ✅ | local SQLite |
| **GPS route map** | ❌ v1 | needs the FIT file |
| **Per-second HR / pace / elevation traces** | ❌ v1 | needs the FIT file |
| **Precise run/walk/stand segmentation** | ⚠️ coarser | Garmin's own `splitType` instead of per-record analysis |
| Training calendar / plan editor | ❌ v1 | plan comes in read-only from an import |
| Social, comments, kudos | ❌ | not our differentiator |

**The honest trade:** v1 has no route map and no per-second traces. The
dashboard, the plan engine, the physiology and the fingerprint are all
intact — those are the differentiator. The map is table stakes that every
other app already has, so it is the right thing to cut first.

---

## 3. Security: the minimum that is not negotiable

Everything here is cheap and each item has a reason.

| Control | Why |
|---|---|
| **Garmin key never enters the WebView.** It is pasted into a **native** Kotlin field, encrypted with an AES-256-GCM key held in the Android **Keystore**, and stored in `noBackupFilesDir`. The WebView never sees it, and there is no endpoint that returns it. | A WebView is a browser. Anything in JavaScript is exposed to the page, to XSS, and to a future dependency. The credential should not be there. |
| **Only Garmin receives the key.** It goes to Garmin's token endpoint over TLS, and nowhere else. | The honest promise. "Never leaves the device" is impossible — the key must reach Garmin to authenticate. Say *that* in onboarding. |
| **Bearer token in memory only**, refreshed from the key when needed (24 h lifetime), never persisted. | A short-lived token on disk is a liability with no upside. |
| **Database encrypted**: SQLCipher with a random 256-bit key, itself wrapped by a Keystore key. | Plain SQLite on a rooted device is a readable file. The DB key is derived independently of the Garmin key, so rotating one never orphans history. |
| **Backups excluded**: `allowBackup=false` plus explicit `dataExtractionRules` exclusions for the database, WAL, keys, FIT blobs and WebView data. | `allowBackup=false` alone is not honoured on every manufacturer. A restored backup would carry the key and the history to another device. |
| **App lock** — a native screen-lock gate, biometric optional. | A phone left unlocked in a bag should not expose a year of health data. |
| **No telemetry, no analytics, no crash uploads carrying payloads.** | The privacy claim has to be true, not aspirational. |
| **Release signing, no debuggable release build.** | |
| **Loopback-only, scoped capability token** for any local API the app exposes. | The desktop app currently binds `0.0.0.0` with no auth — the phone must not inherit that mistake. |

**The threat model boundary, stated honestly:** an unlocked, rooted, or
compromised device can read plaintext that is already in memory. Encryption
protects data at rest on a locked device; it is not a claim of safety on a
compromised endpoint, and the app should say so in its privacy note.

---

## 4. The optional FIT parser — the one feature worth reconsidering

The FIT file is what carries the **GPS track**, the **per-second streams** and
the **precise** walk/stand segmentation. It is also the riskiest piece of the
whole app, because the Python implementation has three Garmin quirks baked in:

1. positions are stored in **semicircles** (`2³¹` per 180°),
2. running cadence is stored **halved** (stride cycles, not steps),
3. lap speed is **absent** on some devices and must be derived from distance
   and time.

**Recommendation:** ship v1 without it. Then, as a scoped v1.1, port **only the
record-message reader** — the FIT record messages are a definition message plus
a data message per sample — which yields the route, the streams and exact
segmentation. Budget ~0.15 MB of Kotlin classes and a focused week.

**Why defer rather than parallelise:** the thresholds that decide
walk-vs-run were derived empirically against a real file, and the agent that
drafted the Android design flagged "faithfully porting the Python FIT
ingestion" as the single biggest risk. That is a small, high-value, bounded
piece of work — but it is not something to attempt in the same pass as
everything else.

---

## 5. Architecture

```
┌─────────────────────────────────────────────┐
│  Kotlin core (the only part with real logic)│
│  • Garmin Personal API client               │
│  • Keystore envelope + token lifecycle      │
│  • SQLCipher database (Room)                 │
│  • WorkManager: daily pull, resumable        │
│  • Native key entry, app lock, backup rules │
└────────────────┬────────────────────────────┘
                 │  ▲
   JSON bridge   │  │ native calls (key, crypto, db)
                 ▼  │
┌─────────────────────────────────────────────┐
│  WebView: the existing React SPA             │
│  • reuses every component, chart and token  │
│  • never sees the Garmin key                 │
└─────────────────────────────────────────────┘
```

**Capacitor**, not a rewrite. The React SPA already has the design system, the
charts, the Today card and the plan engine. A rewrite to React Native or Kotlin
Compose would throw that away and cost more than the whole v1.

**Why a Kotlin core rather than a JS plugin:** the credential must not be
reachable from JavaScript, the database must be encrypted at rest, and the
background pull must survive the app being killed. All three are native
concerns, and all three are the security boundary. A Capacitor plugin that
keeps them in JS keeps the key one XSS away from a stranger.

**The bridge stays narrow**: a handful of typed methods
(`getActivities`, `getActivityDetail`, `getPhysiology`, `getPlan`, `loadHistory`,
`syncNow`, `logout`). No generic "query the database" method — that would put
the whole store behind the WebView and undo the point.

---

## 6. Why this is still differentiated with v1's cuts

Garmin Connect is the vendor app: it shows your activities and the watch's own
coaching, but it does not compare the session against *your plan*, does not
explain a miss, and does not connect the day before to the run. Strava is a
social log with no plan and no physiology. TrainingPeaks has plans but no
device data behind them.

v1 has: **your plan, your runs, your sleep, and the sentence that connects
them** — on a phone that fits in 10 MB, stores everything locally, and never
sends your Garmin key to us.

---

## 7. Build order

1. **Shell + Kotlin core + local DB + Garmin connect.** Paste the key, pull 30
   days, browse offline. Proves the risky parts: Keystore, SQLCipher, the
   token lifecycle, background work.
2. **The Today card.** Plan import → today's session → readiness → one
   recommendation. This is the app's reason to exist; get it on the phone
   before anything else.
3. **Plan vs actual + causes.** The engine already exists server-side; port the
   *presentation*, and reuse the same rules.
4. **Physiology + fingerprint + comparison.** Aggregate views over the local
   database.
5. **v1.1: the FIT reader** → route map, per-second traces, exact segmentation.

**Acceptance for step 1:** install the APK on a real device, paste the key
natively, pull 30 days of activities, kill the app, reopen it in airplane
mode, and browse an activity with its splits and charts — with the key
unreadable to anything on the device.
