# Brief — implement the athlete's declared heart rates

*Written by the parent thread, 24 November 2026. **Implementation.** The
design is done: `exploration/heart-rate-setup.md` and
`exploration/heart-rate-setup.html`. Decision **D27** in
`exploration/decisions.md` settles the trade-off: **the declared pair is the
bands, applied to every run.**

Two other threads are working in this tree right now — one fixing the
ledger's read, one moving the day's four parts. **Do not touch**
`crates/stride-api/src/routes/stimulus.rs` or
`crates/stride-core/src/stimulus.rs` (the ledger thread's), and keep out of
the daytime constants and their strings (the day's-parts thread's). If you
need something from either, **report it** rather than editing it.

## What to build

### 1. The storage — schema version 8

A dated declaration, kept in the history:

* `heart_rate_declarations` — the **kind** (`max` or `resting`), the **value**,
  the **source** from the closed list (*my watch · a test · a doctor or a lab
  · my partner's watch · typed*), the day it was **measured** (nullable), the
  day it was **entered**, the day it was last changed, and the **declaration
  it belongs to** — so a pair entered together is one declaration with two
  rows, and a later pair is a new declaration.

Follow the repository's migration story exactly (`crates/stride-store/src/
schema.rs`, `legacy.rs`, `docs/contract/schema.sql`): **idempotent**, one
transaction for an older database, an older export still refused, the tables
in the **export and the backup** with their counts in the manifest, and the
privacy rules honoured (these are the athlete's own facts, held locally).

### 2. The core — `crates/stride-core/src/heart_rates.rs`

* **the declaration in force** — the most recent declaration with a value;
* **the derived bands** — Garmin's published five-zone ladder, cited in the
  design note, in heart-rate reserve when **both** numbers are given
  (`resting + p × (max − resting)`), in percentages of the maximum when only
  the maximum is, and **no bands at all** when only a resting rate is given —
  with the core's own reason;
* **the comparison** with the watch's configured floors: the two, the
  difference, and **never a blend** — a function that returns the two
  separately and says which one a given answer used;
* the **missing-value reasons**, in the core's own words.

### 3. The zone computation — one seam, two sources

Today the band boundaries come from the **stored zone model** (D3's rule: the
model in force on the run's day). Under D27 the rule becomes:

* **the athlete's declared bands** where a declaration exists — applied to
  **every** run, so the series is comparable across the whole window;
* **the watch's model** where one was in force and no declaration exists.

**Every answer names the basis it used**, and the two are never mixed inside
one answer. Put the selection in the new core module and keep the seam
small — the zone computation takes the floors it is given and does not care
where they came from.

### 4. The routes

| route | what it does |
|---|---|
| `GET /athlete/heart-rates` | the declarations in force, the derived bands, the watch's own floors, and the difference between them |
| `POST /athlete/heart-rates` | the athlete declares: the maximum, the resting rate, or both, with the source and the day it was measured |
| the zone and ledger routes | the bands they now use, and **which basis** — the existing answers change shape, so the **contract** (`docs/contract/api-paths.md`) and a **`PLAN.md` note** travel with it |

**One declaration in force at a time**; the earlier ones stay in the history,
and the answers computed under them stay as they were, dated.

### 5. The app

* **Settings › Training › Your heart rates** — beside the threshold: the
  maximum, the resting rate, each with its source and its dates; the
  derived bands drawn; the watch's own floors beside them, with the
  difference stated; the history.
* **the rig's first-run page** — its three choices become **four**, under
  *"four things only you can give it"*: a race to type, a threshold to
  choose, a programme to adopt, **your heart rates**. Side by side,
  unranked, every one can stay empty, and no vital is linked to any of them.
* The **labels and wording** come from the core's catalogue, in the runner's
  language (D24): *"your resting heart rate"*, *"your maximum heart rate"*,
  *"the bands from the heart rates you entered"*, *"your watch's zones"*.
* `strings.json` is **generated**, never hand-edited; the frontend's
  catalogue is CI-checked.

### 6. The documentation

* `PLAN.md` — a **note** for the change: the declared pair as the bands, the
  cited ladder, the two-facts rule, and the change to Z's rule.
* `exploration/decisions.md` — D27 is already written; add the implementation's
  own line if the shape changed anything.

## Rules

* **Facts, never verdicts**; the ban list and the closed direction set hold.
* **Absence is not zero**: no maximum → no bands, with the reason; a missing
  resting rate → percentages of the maximum, and the answer says which.
* **No new dependencies**; no production data in tests; no Garmin writes.
* **The gates**, per commit: fmt, clippy `-D warnings`, `cargo test
  --workspace`, check-deps, the contract greps, the secret and privacy scans
  and their self-tests, `tokens:check`, `lint:voice`, typecheck, unit,
  viewport.
* **Commit on `rig/v2`**, one commit per piece, and say which is which.

## Report back (last message, under ~450 words)

* the migration and the table;
* the core module's functions, and the **bands it derives** for the declared
  pair (186 / 52) and for a maximum alone;
* the rule's citation, and how the two bases are kept apart;
* the routes and the shapes that changed, with the contract note;
* the app's two surfaces (Settings and the rig's first-run page);
* the commits and the gates each passed;
* anything you needed from the two threads working in parallel.
