Parent: one more thing about `exploration/pace-engine.md`, so it cannot drift again.

The brief thread has settled the chain against your file and found that
**your file still carries the old placeholders as live values**: §2.3's
placeholder table and §12 still say `MP+2:00`, `MP−0:45`, `MP+1:30`,
`MP+3:00`.

When you rebuild the ladder on the new chain, **strike those outright** — do
not leave them in a table as "previous" values. The spine established that
the method does **not** anchor on marathon pace for either shipping
programme, and that the provisional `THRESHOLD −45 s/km` was several times
too slow; a file that still carries the MP offsets is a file someone will
quote later.

Replace them with the settled chain and the spine's numbers:

    race time (typed, dated) → VDOT (the engine's own) → race-pace equivalents
    (VDOT/Riegel) → LT pace (the spine's rule) → the rungs (the spine's percentages)

* **VDOT is the engine; the spine is the rulebook.** Our chain **replaces**
  the book's lookup tables (Appendix B, Appendix C) — we compute the pace
  for a 10K, a 15K and a half from the athlete's race time. The book's
  *rules* are quoted and cited.
* **Anchors:** the goal race pace (10K or HM) and **LT pace** (≈ the pace for
  a 15K / 10-mile race from the same VDOT; or 10K pace + 10–15 s/mi for a
  shorter-distance athlete).
* **Rungs:** general aerobic = **no pace**, only the band (70–81 % maxHR /
  62–75 % HRR); recovery = 15K–HM race pace + 120 s/mi, below 76 % maxHR;
  LT = LT pace, 80–91 % maxHR; long run = 10K pace × 1.20–1.33 → 15K–HM pace
  × 1.17–1.29, or LT pace for the last ~3 miles of a progression run.
* Every rung carries **a band and a pace**, kept apart: the band is the
  method's, the pace is the athlete's own pace-at-HR (or the method's rule),
  and where there is no history at that heart rate the pace is a **no value
  with a reason** while the band stands.

Also confirm in the file that the **run types are the spine's vocabulary**
(endurance, general aerobic, recovery, progression long run, lactate
threshold, VO₂max, speed, tune-up race) and that the ladder shows the paces
subset while the rest belong to the plan's session kinds.

Same two files only; no commit.
