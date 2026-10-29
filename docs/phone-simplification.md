# The phone: one screen, and half as much

**A frozen contract for the next wave of work.** Five agents are working
against this at once, on disjoint files. If something here is wrong, say so
in your report rather than working around it — five people following a wrong
contract is worse than one.

---

## 1. The problem, in the athlete's words

> When I add my Garmin key it takes me to another screen where I again add my
> key but then it doesn't sync the data.

Both halves are real bugs, and both are in the seam between the native screen
and the web app.

**The key is asked for twice.** The SPA shows a Garmin key form. On the phone
that route is *refused* — the phone connects with a Personal API key and never
asks for a password — so the app intercepts the attempt and brings the native
`GarminSetupActivity` to the front. The athlete pastes the key into the web
form, the app throws them onto a native screen, and they paste it again. The
second time it works. It is two screens for one key, and it looks broken.

**The sync does not appear to happen.** `/api/v1/sync/status` answers **501
Not Implemented** in the Rust router. That is the route the dashboard's
"Synced 2 days ago" and the sync button read. So the pull may well be
succeeding and the app has no way to say so.

---

## 2. What the onboarding must be

**One screen. Add the key. The sync follows. It says connected.**

```
┌──────────────────────────────────┐
│  Connect Garmin                  │
│                                  │
│  ┌────────────────────────────┐  │
│  │  ••••••••••••••••••••••    │  │
│  └────────────────────────────┘  │
│                                  │
│  Get a key from Garmin            │
│                                  │
│  [ Connect ]                      │
│                                  │
│  Your data stays on this phone.   │
└──────────────────────────────────┘
```

After **Connect**:

| state | what it says |
|---|---|
| validating | "Checking…" |
| pulling | "Syncing…" and a progress line — "12 runs" |
| done | "Connected · 160 runs" and the screen goes away |
| refused | "Garmin did not accept this key." and the field stays |

**No second screen, ever.** The athlete pastes a key once and watches it work.

### The rules

1. **The key is entered once.** The web app must not offer a key form on the
   phone. It offers a **Connect Garmin** button that opens the native screen.
   That is the only route to the key.
2. **The sync follows the key automatically.** No second button. Saving the
   key *is* the request to sync.
3. **The state is visible.** One place says whether Garmin is connected, and
   it is true within a second of the key being accepted.
4. **The copy is short.** The screen says **"Your data stays on this phone."**
   and nothing else about privacy. No paragraphs.
5. **The key never enters the WebView.** It is typed into a native field,
   validated, wrapped by a Keystore key and written to `noBackupFilesDir`.
   That is already true and must stay true.

---

## 3. Half as much

The app has ten surfaces. **On the phone, five.**

| keep | why |
|---|---|
| **Today** | what to do now |
| **Plan** | the plan against what was actually done — the reason the app exists |
| **Runs** | the activity list |
| **Body** | readiness, sleep, heart rate |
| **Form** | the running fingerprint — the thing nothing else does |

| cut from the phone | why |
|---|---|
| **Compare** | two runs side by side is a laptop activity |
| **Stats** | totals, records and the year heatmap are a laptop activity |
| **Training calendar** | the Plan screen carries the schedule |
| **The Garmin forms** | one native screen, already built |
| **Network route, history windows** | they configure a *server*. The phone has no server. |

**Settings** stays, behind the avatar, and shrinks to: **connected or not**,
**sync now**, **forget the key**, and **about**. Four things.

### The desktop keeps everything

This is a **phone** cut, not a product cut. The desktop keeps Compare, Stats
and the calendar — it has the room, and they are good there.

So the SPA needs to know which host it is on. `isNativeApp()` already exists
in `frontend/src/lib/api.ts` and is the answer:

- the phone shows five tabs; the desktop shows its full rail;
- a phone that is somehow navigated to a cut route lands on Today rather
  than a screen that does not belong there.

**One codebase, two surfaces.** The desktop is not a different product; it is
the same one with more room.

---

## 4. The configuration, with no vendor in it

The repository must not name a network vendor. It configures a network.

| variable | meaning | default |
|---|---|---|
| `STRIDE_TLS_HOST` | the address the HTTPS listener binds | `0.0.0.0` |
| `STRIDE_TLS_PORT` | the port for it | `8443` |
| `STRIDE_TRUST_REMOTE` | when true, clients from the trusted networks are trusted without a pairing code | `false` |
| `STRIDE_TRUSTED_NETWORKS` | comma-separated CIDRs those clients come from | empty — loopback only |

The old `STRIDE_TRUST_TAILNET` becomes `STRIDE_TRUST_REMOTE`. The
hard-coded `100.64.0.0/10` becomes `STRIDE_TRUSTED_NETWORKS`, and **empty
means nothing extra is trusted**.

The listener must not *detect* anything: it binds `STRIDE_TLS_HOST` and
nothing else. No vendor's CLI is run at startup.

**No host name appears anywhere in the repository.** Not in the config, not
in the docs, not in a test. The address is the operator's business.

---

## 5. The rules

1. **One screen for the key.** No exceptions, no "advanced".
2. **The sync follows the key.** There is no separate sync step in the
   onboarding.
3. **Short copy.** "Your data stays on this phone." and then stop.
4. **No vendor in the repository.** Configuration, not a named network.
5. **The desktop keeps its surfaces.** The cut is the phone's.
6. **Do not change the API shapes.** The React app consumes them.
7. **Do not change the schema.** A desktop export has to open on the phone.
8. **Numbers, not adjectives.** Report what a test asserts, not that it
   "works".
