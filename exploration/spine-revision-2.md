Parent: two updates for the spine.

## 1. SCOPE — the plan programmes are cut

The user: *"just have 10k and HM plans, cut even 5k for now."*

So the product ships **two plan templates: the 10K and the half marathon.** The
5K and 8K plan structures are **out of scope for the first drop**.

Everything else you are extracting still stands — the run types and their
rules, the hard/easy principle, the recovery rules, the pace rules, the
periodisation logic, the "adjusting the training plan" guidance — because the
10K and HM plans are built from the same components.

Say plainly in the document which plan structures you extracted but are **not
shipping**, so the knowledge is kept without the scope creeping back in. The
audience is confirmed as **intermediate runners**, and the app holds **one base
at a time** (Pfitzinger first; 80/20 as the second, separate method).

## 2. PLACEHOLDERS — replace them with sourced numbers

The pace-engine thread (`thr_crvvnnhwtz`, done) has built the ladder with
provisional rule values, which it marked as placeholders for your numbers:

| placeholder | the pace-engine thread's provisional value | what it is |
|---|---|---|
| `EASY` | +120 s/km off marathon pace | the easy / general-aerobic pace |
| `THRESHOLD` | −45 s/km off marathon pace | the threshold pace |
| `LONG_START` | +90 s/km, `FINISH` 0 | where the long run starts and where it ends |
| `RECOVERY` | +180 s/km | the recovery pace |

Those are **placeholders, not sourced values**. Your job: replace each with
the **sourced number and its citation**, or state that the method does not fix
it and the app must let the athlete choose it — the two bases differ here, and
the athlete's own choice is legitimate.

Note for the ladder's own sake: the pace engine's chain (race time → VDOT →
marathon pace → the paces) and Riegel's prediction form are **not**
placeholders — they are the engine's own, sourced elsewhere.

Report back as before, plus: the final value for each of the five rows above
(with its citation, or "the method does not fix it"), and which plan
structures you extracted but are not shipping.
