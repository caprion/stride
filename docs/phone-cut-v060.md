# The last three, and the v0.6.0 bundle

**A frozen contract for four agents working at once.** Read this before
writing anything. If something here is wrong, say so in your report rather
than working around it.

---

## 1. The three things

### (a) The phone's Settings is four things

The phone's Settings is currently the desktop's Settings. It offers the
network route, the history windows and the sync interval — **all of which
configure a server.** The phone has no server. It is the app.

On the phone, Settings becomes:

| | |
|---|---|
| **Garmin** | connected or not, when it last synced, **Sync now**, **Forget the key** |
| **About** | what the app is, and the one line: *"Your data stays on this phone."* |

Nothing else. No network route. No history windows. No sync interval. No
developer key. No Garmin password.

**The desktop keeps all of it.** This is a phone cut, not a product cut.

### (b) The phone's Plan screen is one thing

Two things to remove on the phone:

1. **The import box.** "Import a plan" with a Markdown paste area. A markdown
   document is a *desktop* thing — a file somebody wrote somewhere else — and
   it is the only authoring affordance left on that screen.
2. **All eighty-three sessions.** The list is the whole training block, which
   is four screens of scrolling on a phone. It should be the **near future and
   the recent past** — this week, next week, and last week. The desktop keeps
   the full list.

### (c) The map is the hero of an activity

Right now the activity page is: title, stats, **then** the map, then the
analysis. The route is the thing an athlete remembers about a run — the shape
of it, the streets, the park, the long way home. It should be **the first
thing on the page.**

The map does not get smaller and it does not lose its controls. It moves up.

---

## 2. The two new bridge methods

The phone's Settings needs to trigger a pull and to forget the key, and
neither belongs in the web app: the key lives in the **Keystore**, and the
pull is the phone's **background job**.

So the shell exposes two more plugin methods, on the existing
`StrideApiPlugin`:

```kotlin
@PluginMethod
fun syncNow(call: PluginCall)     // enqueue the pull now; resolves with { queued: true }

@PluginMethod
fun forgetGarminKey(call: PluginCall)  // clear the envelope; resolves with { forgotten: true }
```

- **`syncNow`** enqueues a **one-time** `GarminSyncWorker` request — not the
  periodic one — and resolves as soon as it is queued. It does not wait for
  the pull: the app shows "Syncing…" and the count arrives by itself.
- **`forgetGarminKey`** clears the Keystore-wrapped envelope, cancels the
  scheduled work, and clears the core's in-memory key. The imported runs stay
  on the phone — forgetting the key is not forgetting the data.

**Neither method ever returns, logs or transmits the key.** They are about
the key's *existence*.

The SPA calls them the same way it reaches the native setup screen today.

---

## 3. The rules

1. **The desktop keeps everything.** Every cut in this brief is the phone's.
2. **Do not change the API response shapes.** The React app consumes them.
3. **Do not change the database schema.**
4. **Do not edit a file you do not own.** The map is yours.
5. **The copy is short.** "Your data stays on this phone." and then stop.
6. **Report numbers, not adjectives.**
7. **The gates must be green before you report:**
   - `cd frontend && npm run typecheck && npm run build`
   - `./scripts/rust_gate.sh`
   - `./scripts/security_check.sh`
   - `python3 scripts/check_apk.py <the apk>` if you touched the phone's crate
