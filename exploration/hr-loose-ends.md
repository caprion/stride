Parent: two loose ends from the one-answer thread, both in your territory.
Please close them on `rig/v2` as part of your work.

1. **The run's own zones answer must say when the zones begin to apply.**
   The one-answer thread made the screen's source line say it — *"the
   watch's configuration as it stood on 25 Nov 2026 · the runs from that day
   on"* — and made a run's answer say why it has no basis. But the run's own
   page does not say **the day the zones begin to apply**, so a reader has to
   visit another screen to learn it. Add **one field** to the run's zones
   answer — the day the athlete's zones begin to apply — and say it in the
   answer's own wording, in the runner's language: *"your zones apply to the
   runs from 25 Nov 2026"*, or the equivalent. If there is **no** such day
   (no model has ever been read), the field is absent and the reason stands
   as it is now.
   The shape of that answer is yours (you own the zones route's shape), so
   the **contract** (`docs/contract/api-paths.md`) and the shape test travel
   with it, plus a `PLAN.md` note.

2. **`zones.no_model` is still in the old register.** It reads *"No zones yet
   — your watch's heart-rate zones have not come through"*, while the rest of
   the new vocabulary says **"not measured —"** and names what is missing and
   what else is available. Align it with the register the other reasons
   use — *not measured*, the watch's zones were not read, and the watch's own
   zone minutes are the only zone time here — and keep the ban list and the
   closed direction set untouched.
   The **ledger's tests assert that string** (`crates/stride-core/src/
   stimulus.rs`'s tests), so changing it means updating that expectation
   too. You have been told not to touch `stimulus.rs`; **this one time you
   may change that test's expectation only** — the ledger's behaviour is
   settled and must not change.

3. And the check the one-answer thread asked for: **the phone's frozen route
   table** now names `/athlete/heart-rates` (it was missing and the parity
   check caught it) — please keep that true for any route you add.

## Rules

* The gates, as always; commit on `rig/v2`.
* The vocabulary (D24): *"your zones"*, *"not measured"*, *"the runs from
  that day on"*.

## Report back

* the new field, and the sentence a run's page now prints;
* the new wording of `zones.no_model`;
* the gates.
