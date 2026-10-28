# Rust, or what we have now

An evaluation. **Nothing has been changed** — this is a decision document,
not a plan.

---

## 1. The question, restated so it can be answered

"Rust for the whole app" can mean three different things, and they have
different answers:

| | what it means | verdict below |
|---|---|---|
| **A** | A Rust core replacing the **Python** backend; the React UI stays; the phone keeps Capacitor + a Kotlin shell | Partly useful |
| **B** | A Rust core replacing **both** Python and Kotlin, via **Tauri 2** for desktop and mobile | The interesting one |
| **C** | Rust for the core *and* a native Rust UI, dropping React | Not worth it |

The short answer: **Rust does not replace Kotlin. It replaces Python.** And
that is a bigger deal than it sounds, because of what it does to the
*desktop*.

---

## 2. What actually decides it

Not "is Rust good". This:

> **How many implementations of the domain logic do you want?**

Today the plan is:

- **Python** on the desktop — the FIT parser, segmentation, fingerprint,
  readiness, plan engine, the Garmin client, the database.
- **Kotlin** on the phone — the same six things, again.

**Two implementations of the same logic.** That is the single largest risk in
`docs/android-packaging.md`, and it is why that document says *"port the
Python tests, not the Python code"* — because a hand-port of a FIT parser
that gets the halved cadence or the semicircle positions wrong produces
plausible-looking numbers that are not the athlete's.

Rust lets you have **one**. The core is a library; the desktop builds it as a
binary and the phone builds it as an `.so`. Same source, one implementation,
and a compiler that will not let the two drift apart.

So the real question is: **is one implementation worth a rewrite of the
working desktop app?**

---

## 3. What Rust removes, and what it cannot

### It removes Python — and that is the prize

This is the part I had not appreciated until I laid it out.

The desktop app is a **server**. Python is a server framework, so Stride's
desktop is FastAPI + uvicorn + a venv + a process that must be running +
a port to bind + a capability token to authenticate the browser that is
sitting on the same machine. All of that exists only because the domain
logic is in Python.

A Rust core with an embedded SQLite turns the desktop into what it should
have been: **an app**. The React SPA becomes a static bundle; the core
opens the database; there is no port, no token, no process to keep alive,
and no Python to install.

That also answers *"make the whole setup as light and quick as possible"* in
a way Kotlin cannot: the desktop becomes **one binary**.

### It does not remove Kotlin

The Android app needs three things Rust cannot reach:

| need | why it is Java |
|---|---|
| **Android Keystore** — the non-exportable AES-GCM key that wraps the Garmin credential | the Keystore *is* a Java API; Rust reaches it only through JNI |
| **WorkManager** — background sync, with the system's own scheduling | a Java API |
| **Activity lifecycle, foreground services, notifications** | Java |

So **there is a Kotlin layer regardless** — call it 300–500 lines of
platform glue. The question is only whether the *domain logic* lives in
Kotlin (today's plan) or in Rust.

---

## 4. The numbers

Measured where I could measure them; estimated where I could not, and marked.

| | today | Rust core (B) | Kotlin core (today's plan) |
|---|---:|---:|---:|
| **APK size** | **3.58 MB** (measured, v0.2.0) | ~4.5 MB *(est.)* | ~3.9 MB *(est.)* |
| **core binary** | — | ~2.5–3.5 MB `.so` *(est.)* — SQLCipher dominates | ~200–400 KB DEX *(est.)* |
| **APK size vs today** | — | **+1 MB** | **+0.3 MB** |
| **domain-logic memory** | — | ~5–15 MB RSS *(est.)* | ~15–30 MB RSS *(est.)* |
| **desktop footprint** | Python + venv + node + a running server | **one binary + the SPA** | Python + venv + node + a running server |
| **port size** | — | **~3,700 lines** | **~3,700 lines** |
| **cold build** | seconds | minutes *(NDK + cargo)* | ~1 min (Gradle) |
| **GC pauses on the phone** | n/a | none | ART, negligible for this workload |

### The size claim needs honesty

Rust is often sold as "smaller". **For this app it is not.** SQLCipher is
1.8 MB either way, the ART runtime is in the APK either way (the Keystore
glue needs it), and Rust's `std` and panic machinery add roughly what a
Kotlin core would. A Rust APK is about **1 MB larger**, not smaller.

Rust's real wins are **memory** and **one implementation** — not bytes.

### And the elephant

The APK is 3.58 MB but the *phone's* memory is dominated by the **WebView**
that renders the React app — Chrome's renderer process is on the order of
100 MB on Android, plus 3–6 MB of JS heap for the app itself (measured).

So the difference between a 5 MB and a 15 MB core is real but small against
a ~100 MB WebView. If the goal is a *genuinely* light Android app, the
lever is **not using a WebView** — it is a native UI (Compose), with the
core in whatever language. Rust does not do that; it just makes the core
cheaper.

---

## 5. The comparison

| | Python + Kotlin (today's plan) | Rust + Tauri 2 | Rust core + Capacitor + Kotlin glue |
|---|---|---|---|
| **implementations of the domain logic** | **two** | **one** | one (core) + Kotlin glue |
| **desktop** | a server | **an app** | a server |
| **desktop setup** | venv + npm + a process | **cargo + npm → one binary** | venv + npm + a process |
| **APK size** | smallest (~3.9 MB) | largest (~4.5 MB) | ~4.2 MB |
| **phone memory** | highest of the three | lowest | low |
| **FIT parsing** | `fitparse` (Python) | `fitparse` (Rust) or Garmin's SDK bindings | hand-rolled — no good Kotlin library |
| **SQLCipher** | official Android library | `rusqlite` + bundled-sqlcipher | official Android library |
| **Keystore / WorkManager** | native Kotlin | JNI plugin | native Kotlin |
| **NDK in CI** | no | yes | no |
| **Tauri 2 Android maturity** | n/a | **younger than Capacitor** | n/a |
| **the port** | ~3,700 lines | ~3,700 lines | ~3,700 lines |
| **learning curve** | Kotlin is mainstream | Rust is not | Kotlin |

---

## 6. What I actually recommend

**Not Rust yet — but Rust is the right long-term shape, and the reason is
the desktop, not the phone.**

### If the goal is *the phone app, soonest*

**Stay with Kotlin.** The port is the same size either way, Kotlin has
better libraries for everything Android-specific (SQLCipher, WorkManager,
the Keystore), the platform APIs need Java anyway, and a Rust APK is about
1 MB *bigger*. Choosing Rust here buys a smaller memory footprint and
nothing else. If you are comfortable with Rust, it is a fine choice — it is
just not a faster one.

### If the goal is *one codebase for desktop and phone*

**Rust, with Tauri 2.** This is the real prize, and it is bigger than it
looks:

- the desktop stops being a server — no port, no capability token for your
  own browser, no process to keep alive;
- the desktop becomes **one binary**, which is the lightest, quickest
  setup of anything on the table;
- the phone and the desktop share the domain logic, so the numbers cannot
  disagree;
- the React UI is unchanged in both.

The risks are real: Tauri 2's Android support is younger than Capacitor's,
Rust for Android needs the NDK in CI, and the port is not smaller.

### If the goal is *a native Android app, properly light*

**Kotlin + Compose, not a WebView.** The 100 MB renderer is the weight, not
the core. Whatever language the core is in, the UI decision matters more
than the language decision.

### The cheap mitigation, either way

Whichever you choose, the port's correctness risk is the same and it is
cheaply retired:

> **Differential tests.** Run the *same* fixtures through the Python
> implementation and the new one, and assert identical outputs — every
> segment boundary, every fingerprint number, every verdict.

That is a few hundred lines and it converts "two implementations that might
drift" into "two implementations that are proven to agree". It is worth
doing **whatever** you choose, and it is worth doing *first*, because it
turns the port from a leap of faith into a checked step.

---

## 7. The decision, stated plainly

| if you care most about | choose |
|---|---|
| the phone app, soonest | **Kotlin** (today's plan) |
| one codebase, lightest desktop | **Rust + Tauri 2** |
| the smallest APK | **Kotlin** (by ~0.7 MB) |
| the least phone memory | **Rust** (by ~10 MB) |
| the fewest moving parts on the desktop | **Rust** (one binary, no server) |
| the least risk to the schedule | **Kotlin** (Tauri 2 Android is younger) |

**My vote: Kotlin for the phone, and Rust for the *core* — but not both in
the same release.** Sequence it:

1. **Differential tests first.** The Python implementation stays the
   reference; the fixtures and the expected values are frozen.
2. **Port the core to Kotlin** for the phone, with the differential tests
   running against both. The phone ships.
3. **Then** decide whether the desktop follows the core into Rust, or stays
   Python with the tests still proving they agree.

That order gets the phone fastest, keeps the correctness risk retired the
whole way, and leaves the Rust decision open until there is evidence you
want it — rather than betting the whole product on it up front.
