Parent: D11 is binding — LESS VERBOSE, MORE VISUAL, in the app itself — and here are the decisions on your nine open questions. Please apply both to `exploration/method-lens.md` and `exploration/method-lens.html` (same two files).

## D11 — the picture first, the sentence second

* The card is read as a **picture** before it is read as text; the sentence is the **caption** and the accessible fallback (a number still needs its period and its reason in words).
* **Absence is drawn**: a date marker on a small timeline, or a gap in a strip — the reader sees the hole before the sentence names the date.
* The **rule block and the history block stay visibly separate** — no arrow, no shared axis that implies a comparison the app must not draw. If the method's number and the athlete's numbers appear on one scale, that scale belongs to the **method** and the athlete's data is plotted on it — with **no verdict colour and no label** ("above"/"below" are the method's own arithmetic, never the app's judgement).
* The caption must not repeat what the picture already says.
* Fewer sentences; each still carries its values inline (contract rule).

Concretely, per lens, add a short section to the note: **what the lens draws** — for example, a scale with the method's published band marked, the athlete's time plotted on it, and a small timeline carrying the last date. Then update the prototype so it shows those pictures rather than paragraphs.

## The nine decisions

1. **A rule with two clauses (L3, Bakken: heart rate below 70% of max *and* lactate below 1.0 mmol/l) — compute the half that resolves, and state the half that does not.** The card says the rule has two clauses, names both, says the heart-rate clause resolves against the athlete's own zone model, and says the lactate clause is not recorded by a watch. Nothing is approximated, and the method is not quietly halved.
2. **The refused methods appear in the picker, but not as choices.** They are listed with their reason ("stated in blood lactate"; "stated in marathon pace"; "needs RPE and the talk test") and are not selectable. A refused method must not vanish silently — the reader should learn that it exists and why it is not there. No method is ever the app's recommendation: the list is alphabetical or in the order the note uses, and the app asserts none of them.
3. **Yes — the band or ceiling value may sit in the Shape B header** ("Bakken's easy days · 128 bpm"). It is a fact: 70% of the maximum heart rate in the athlete's own zone model. It is part of the lens's identity, not a verdict.
4–7. **Inherited and already decided** — D1 (one window: last N calendar weeks through today, compared with the same span ending seven days earlier), D3 (stage 1: every run measured with the zone model in force on the day of the run), D5 (each share names its denominator; shares over different denominators are never added), D2 (one comparison rule). Apply them; do not re-open them.
8. **The athlete's own cap is a dated setting, and it behaves like the zone model.** Stride never computes 180 − age — the core's rule is that a maximum heart rate is never an age formula, and no birthdate is stored — so the cap is **entered by the athlete** and stored as a **dated fact** (the value, the day it was entered, the day it last changed), beside the zone model's dated facts. When a window spans a change in the cap, each day is measured against the cap **in force that day** (the same rule as D3), and the card names the cap and the day it was read. If the cap is changed, the answers before it change too — that is the athlete's own number and the app says which one it used.
9. **No lens on a run's own page.** A run's page shows the run's own facts; a lens is a reading of a window. Keeping the run page free of lenses is the same discipline as the surface cut.

## Report back

* the two files updated,
* for each of the three lenses, **what it now draws** — three lines each, no paragraphs,
* how the refused methods appear in the picker,
* how the athlete's dated cap is stored and named in the sentence,
* anything you could not draw in a way that stays fact-only.
