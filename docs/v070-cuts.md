# v0.7.0 — the last cuts

**A frozen contract for five agents working at once.** Read it before
writing anything. If something here is wrong, say so in your report rather
than working around it.

---

## 1. No sample data. At all.

The demo dataset goes away entirely: the seeding, the flag, the banner, the
filter in every query, and `backend/app/ingest/demo.py`.

It existed so the app would not open empty. It has outgrown that:

- the banner *"you're browsing sample data"* is a confession built into the
  product — it tells the athlete the app is lying, on every launch, for ever;
- the demo is seeded at `12.9716, 77.5946, "Bengaluru"` — **the athlete's
  own coordinates and their own city**. The sample runs look like *their*
  runs and are not, which is worse than obviously-fake data;
- and it hides the real first-run experience, which is the thing this round
  is about.

**What replaces it:** an app that opens empty and says what to do. The
dashboard's empty state should read **"Connect Garmin to see your runs"** and
offer the one obvious next step.

**The demo must not come back** in any form. Not hidden, not behind a flag.
If you need data for a screenshot, use a snapshot of the real thing.

---

## 2. The Body screen is three things

**Today's readiness. Sleep. Resting heart rate and HRV.**

Body battery, stress and load come off the screen. They are Garmin's own
metrics with their own definitions, and on the one screen whose job is
*"how am I?"* they are three more series competing for attention.

They also come off the **payload** — nothing draws them and the plan engine
does not use them (it needs sleep, HRV and resting HR). `/physiology/daily`
currently returns every Garmin column for every day.

---

## 3. The Plan tab comes off the phone

The phone's tabs become five: **Today · Runs · Body · Form · Stats**.

Plan was meant to be the reason the app exists, and the inferred plan has
just made it non-empty for everyone. But the athlete has used it and it does
not work, and the reason is now clear: *39 of 79 past sessions were
"missed"* — the athlete compared against a campaign plan they were not
following. Accurate and nearly meaningless.

So: the Plan tab comes off the **phone** for v1. **The work stays** — the
inferred plan, the verdict logic, the `not_yet` state — and the tab comes
back when it reads well. **The desktop keeps Plan.**

Weekly volume and pace trends stay where they are: on **Today** and on
**Stats**.

---

## 4. The native Garmin screen

Four changes, all on the native screen:

### a. Its own theme, and no splash over it

`GarminSetupActivity` inherits the application's launch theme, which is a
`Theme.SplashScreen`. On Android 12+ every activity launch shows the system
splash, and an activity that does not handle that lifecycle leaves it up —
**an overlay over the screen the athlete is trying to read**.

Give the activity **its own plain theme** — an `AppCompat` theme with the
app's surface colour, no splash — and make sure the splash is dismissed.

### b. "Replace key" → "Save key"

"Replace" reads as destructive. The athlete is saving a key.

### c. One link, below the field

The screen offers **one** link, directly under the key field, that takes the
athlete to Garmin's key page. Not a button in a row with another button.

**The link's label carries the one instruction that matters:**

> **Get a key from Garmin — pick the one-year expiry**

Garmin offers 30, 90, 180 days or a year. A 30-day key means the app stops
working every month and the athlete has to come back and paste a new one. The
link is where they will read it, so the advice belongs in the link.

Open it in a **Custom Tab**, so the athlete comes straight back to the field
they were filling in — not out to a browser they then have to find their way
back from.

### d. Nothing else on the screen

No server field. No paragraph about what local-first means. The status line
says what is happening; the line under it says how many runs have arrived.
One line about privacy: **"Your data stays on this phone."**

---

## 5. The rules

1. **The desktop keeps everything.** Every cut in this brief is the phone's,
   except the demo data, which goes from both.
2. **Do not change the API response shapes** except where this brief says
   to, and where it does, change the frontend that consumes them in the same
   commit.
3. **Do not change the database schema.** A desktop export has to open on
   the phone.
4. **Do not edit a file you do not own.** Five agents are working at once.
5. **Short copy.** "Your data stays on this phone." and then stop.
6. **Report numbers, not adjectives.**

## The gates

```bash
cd frontend && npm run typecheck && npm run build
./scripts/rust_gate.sh
./scripts/security_check.sh
python3 scripts/check_apk.py <apk>
```

All green before you report.
