# The review

Every screen, both hosts, the real data, and the onboarding end to end.

**The artefact:** `showcase/review/index.html` — 18 screens, contact sheet,
real data, no console errors. Regenerate it with `node scripts/review.mjs`.

---

## 1. What the review found

### The Plan screen was wrong about the future

A session that has **not happened yet** was reported as **MISSED**. The card
read *"Not yet MISSED"*. Worse, a future rest day read **HIT**, because a rest
day with no activity is "hit" by the rule.

Measured on the live data, 29 October, the block running 15 June to 1 November:

| verdict | past | future |
|---|---:|---:|
| hit | 12 | **1** ← a future rest day, calling itself a hit |
| missed_pace | 8 | 0 |
| short | 15 | 0 |
| missed | 39 | **3** ← sessions that have not happened |
| substituted | 5 | 0 |

So the headline said 42 missed and three of them had not happened yet. The
adherence denominator was 83; it is 80.

**Fixed.** A future session is `not_yet` — neither a hit nor a miss, and it
does not count in the denominator. It renders in a neutral grey, because a
session that has not happened is not a failure.

### The sync was not incremental, and the Rust one was much worse

Same window, same account, same machine:

| | activities | time | detail calls | FIT downloads |
|---|---:|---:|---:|---:|
| the Python reference | 29 | **3.3 s** | 0 | 0 |
| the Rust core | 29 | **89.1 s** | 29 | 29 |

The Rust sync re-fetched the detail and re-downloaded the FIT for every
activity on every sync. Over a 90-day window on a phone that is 160 FIT
downloads — about 16 MB — every sync, over mobile data, forever.

**Fixed**: the reference's rule is now in the Rust too — reuse the stored
summary, and only fetch a FIT for an activity that does not have its streams.

### The wellness pull re-fetched every day, every time

| | time | requests | days pulled | days skipped |
|---|---:|---:|---:|---:|
| before | **82.1 s** | 121 | 30 | 0 |
| after | **3.8 s** | 5 | 1 | 29 |

The rule is now *do not fetch what you already hold*. A day with a measurement
in it is skipped; today is always pulled, because the day is not over.

### The Plan screen was empty for most athletes

A plan is *intent* — a dated list of sessions with a target. Until now there
was exactly one source: **a markdown document somebody writes**. So for
anyone who does not plan in a text file, the app's reason to exist showed
nothing. There was also a client that could fetch Garmin's scheduled workouts
and **nothing called it**.

**Fixed.** Two sources, both views rather than management:

1. **The plan you already have, inferred from your own history.** Every
   runner has an implicit plan: their weekly volume (the median of the last
   twelve whole weeks), their rest days, their long-run day, their quality
   day, and their usual session per weekday with a pace band. **Zero setup,
   and it works for everybody.**

   And it declines honestly when there is not enough history: *"Not enough
   history to infer a usual week: 14 runs over 4 weeks. Stride needs at least
   12 runs over 8 weeks."*

2. **The plan you wrote down — Garmin's scheduled workouts.** The client is
   already written; it needed wiring.

And the two are **never mixed**: the payload carries `prescribed: true|false`,
an inferred baseline says *less than usual / more than usual / on a usual day*
and **never "missed"**, and a Garmin prescription keeps *hit / missed / short*.

### And the finding underneath all of them

**39 of 79 past sessions were "missed".** That is the athlete compared against
a campaign plan they were not following, so it is accurate and nearly
meaningless. It is the strongest argument for the inferred plan: comparing the
athlete against *their own* habitual week is the comparison that says
something.

### Smaller things

- The gate was leaving a paired device behind on every run. Seven had piled
  up in the Settings screen, every one of them a "Browser" paired by the
  gate's own test. The gate now cleans up after itself.
- The phone's Plan screen still carries an **"Import a plan" markdown box** —
  authoring, on a screen that should have one thing on it. Being removed.

---

## 2. The onboarding, end to end

**One screen. One key. The sync follows.**

```
┌────────────────────────────────┐
│  Connect Garmin                │
│                                │
│  ┌──────────────────────────┐  │
│  │  ••••••••••••••••••••    │  │
│  └──────────────────────────┘  │
│  Show the key   Get a key from Garmin │
│                                │
│  Checking…                     │
│  12 runs                       │
│                                │
│  [ Connect ]                   │
│                                │
│  Your data stays on this phone. │
└────────────────────────────────┘
```

That is the whole screen. The layout has no server field, no paragraph about
what local-first means, and one line about privacy.

**The flow, and what is verified:**

| step | what happens | verified |
|---|---|---|
| 1. launch | the core opens its own encrypted database, and the native setup screen comes up | ✓ `mCurrentFocus=GarminSetupActivity`, `StrideCore: the core is open on …/no_backup/stride.db` |
| 2. paste the key | into a **native** field — never the WebView | ✓ the field is `textPassword`, the screen is `FLAG_SECURE` |
| 3. Connect | the key is checked against Garmin, and **only then** kept | ✓ a refused key writes no credential file |
| 4. sync | it starts on its own — there is no second button | ✓ the foreground service starts with `foregroundServiceType="dataSync"` |
| 5. progress | "12 runs", then "160 runs" | the count as it arrives |
| 6. done | "Connected · 160 runs", and the screen goes away | |
| 7. the app | the dashboard renders from the Rust core | ✓ verified in the earlier run |

**Why the key is entered natively and not in the WebView:** a WebView is a
browser. Anything in it is one injection away from a stranger. The key is
typed in the app, wrapped by a Keystore key, and written to `noBackupFilesDir`
— and there is no endpoint that returns it.

---

## 3. The screens

Eighteen of them, in `showcase/review/`.

**What is good.** The dashboard reads well at 390 px — the plan for today,
the readiness, the week, the recent runs — and the volume bars now encode
how close each week came to the goal instead of being sixteen identical
amber rectangles. The phone's Plan screen fits its week strip and its
verdict bar.

**All three of these are now done** (v0.6.0):

1. ~~The phone's Settings is the desktop's Settings.~~ **It is two cards
   now** — Garmin (connected, when it last synced, Sync now, Forget the key)
   and About. The desktop keeps every server setting.
2. ~~The phone's Plan screen still has the import box, and still lists all
   eighty-three sessions.~~ **Gone.** The import box is the desktop's, and
   the phone's list is the near future. The counts follow: **3 of 8 sessions
   and 50%**, where the whole block said 38 of 83 and 25%.
3. ~~The map is not the hero of an activity.~~ **It is.** The route, then
   the title and the numbers, then the analysis — with the heart rate red,
   the pace blue and the elevation violet.

### And the harness was lying about the phone

The review captured the "phone" by rendering the desktop's web app at 390
pixels. **The phone's surfaces only exist inside the WebView**, where
Capacitor injects `window.Capacitor` — so the phone's Settings, its Plan
screen and its six tabs were all invisible, and a phone-only change could
look like it had landed when it had not.

The harness now injects `window.Capacitor` for the phone context, so every
screen in `showcase/review/` is the surface that actually ships.

---

## 4. The measurements

| | |
|---|---|
| the APK | **8.6 MB**, arm64-v8a, 8.57 MB of it the Rust core |
| the Rust gate | **9 of 9** — 355 tests, 38 golden assertions |
| the backend | 178 pass, 2 skip, 2 xfail, 1 xpass |
| the security gate | **27 passed, 0 failed** |
| the route parity | 38 routes, **0 differences** against the Python reference |
| the desktop binary | 7.0 MB, one process, two listeners |
| the first sync | activities **3.3 s**, wellness **3.8 s** (was 76 s) |
| the JS heap | 3.3–6.2 MB a screen, no long tasks |
