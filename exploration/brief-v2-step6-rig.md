# Brief — step 6: the rig — the desktop host serves the new version

*Written by the parent thread, 24 November 2026. **Implementation.** Step 6 of
the rig-v2 build; `exploration/build-plan.md` has the order and the rules.
Steps 1–5 have landed: the storage, the core engines, the API routes, the plan
module, and the frontend cards.*

## What this is

The **desktop rig** is the app's face: a local HTTP host that serves the
frontend and the API on the athlete's own machine, with no account and no
network. This step makes the **new version** a working rig — the screens that
were just built, reachable, coherent, and honest about what is missing.

## What to do

1. **The routes and the bundle.** The rig serves the frontend bundle; the
   desktop host is `desktop/`. Make sure the **new cards' routes are served**
   by the host, that the **bundle is regenerated and synced**
   (`scripts/sync-desktop-bundle.sh`), and that the rig boots with the new
   screens present. The desktop host and the API are separate crates in the
   dependency DAG (`desktop → {api, store}`) — `scripts/check-deps.sh` will
   hold you to it.
2. **The rig's own surface.** The desktop host has its own landing surface
   (an index, a status, the storage and the connection). Bring it in line
   with the app: the rig should say what version it is serving, that the data
   is the athlete's own and local, and — where the new screens need a
   **first-run decision** (a race to type, a threshold to choose, a programme
   to adopt) — it should offer them **as the athlete's choices**, unranked,
   never as a recommendation.
3. **The states that matter on a fresh rig.** An athlete with a rig and no
   race typed, no threshold chosen and no plan adopted sees **empty cards
   with reasons** — not a blank screen and not a wizard that presumes. If the
   rig has any onboarding surface, it must follow the same rule as the app:
   the choices are the athlete's, the vitals are never linked to them, and
   nothing is graded.
4. **The phone.** The bridge's route table names the new routes (step 3), so
   the phone does not 404 — but the phone's UI does not call them yet. **Do
   not add phone screens**; this build is the desktop rig. Make sure the
   parity check is green and nothing on the phone breaks.
5. **The rig's own gates.** The desktop build, the bundle check, and the
   viewport matrix at the desktop widths. Report what ran.

## Rules

* **Facts, never verdicts**, everywhere the rig speaks.
* **The surface speaks the runner's language** (D24).
* **No new dependencies** (`scripts/approved-deps.txt`).
* **No publish, no production data, no Garmin writes.** The rig talks to the
  athlete's own local store only.
* **The gates**: fmt, clippy `-D warnings`, `cargo test --workspace`,
  check-deps, the contract greps, the secret and privacy scans and their
  self-tests, `tokens:check`, `lint:voice`, typecheck, unit, viewport, and
  the desktop build.
* **Commit your work** on `rig/v2`.

## Report back (last message, under ~350 words)

* what the rig now serves, and how a fresh rig behaves;
* the first-run decisions it offers, and how they stay the athlete's;
* the bundle and the build;
* the gates, and anything you could not run here;
* anything in the rig that still speaks the old language.
