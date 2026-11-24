# Pfitzinger's spine — the numbers, the run types, the plans, and the peer method

*Research note for the parent thread and the pace-engine thread. Written 24 November 2026.*
*Research only — no code, no changes to `crates/`, `frontend/`, `android/`, `desktop/`, `PLAN.md` or `docs/`.*

---

## 0. What this file is, and how it was gathered

### 0.1 The sources, ranked

The rule the parent set: **the book first for anything it covers, then the author's public statements, then third-party summaries (labelled).** That order is applied throughout.

| # | source | what it is | rank |
|---|---|---|---|
| **S1** | Pete Pfitzinger & Philip Latter, ***Faster Road Racing: 5K to Half Marathon***, 2nd ed. (Human Kinetics, 2015) — the EPUB in this thread's storage | the **book**, sub-marathon: training components, the run vocabulary, base training, and the 5K / 8K–10K / 15K–10-mile / half-marathon / multi-distance plans | **primary** |
| **S2** | Pfitzinger & Douglas, "Marathon Advantage", *Runner's World*, 2001 — [runnersworld.com/advanced/a20819355/marathon-advantage](https://www.runnersworld.com/advanced/a20819355/marathon-advantage) | an **author-written** excerpt of *Advanced Marathoning* (1st ed.), with the full 18-week schedule and the definitions of every session type | **primary (for the 1st-edition plan)** |
| **S3** | Mario Fraioli, "Going Long: An Interview with Pete Pfitzinger", *The Morning Shakeout*, 2025 — [themorningshakeout.com/going-long-an-interview-with-pete-pfitzinger](https://themorningshakeout.com/going-long-an-interview-with-pete-pfitzinger) | the author speaking in 2025 about the 4th edition, the long run, marathon-pace runs, LT, VO₂max | **primary (author's own statements)** |
| **S4** | pfitzingercoaching.com — [about-the-plans](https://www.pfitzingercoaching.com/about-the-plans), [faq](https://www.pfitzingercoaching.com/faq), [training-pace-table](https://www.pfitzingercoaching.com/training-pace-table), and the author's own PDF *Lactate Threshold Pace (LT Pace) Based on Race Times* | the author's own current framing: **every workout pace is set relative to LT pace**, and the LT-pace table he publishes | **primary (author's own site)** |
| **S5** | Human Kinetics, *Advanced Marathoning*, 4th ed. product page — [us.humankinetics.com/products/advanced-marathoning-4th-edition](https://us.humankinetics.com/products/advanced-marathoning-4th-edition) | publisher's table of contents: chapter 6 *Tapering*, chapter 8 *Following the Schedules*, Appendix A *Marathon Race-Pace Chart*, Appendix B *Equivalent Race Times* | **primary (publisher)** |
| **T1** | runningwithrock.com — [pfitz-marathon-training-explained](https://runningwithrock.com/pfitz-marathon-training-explained), [pfitz-4th-differences](https://runningwithrock.com/pfitz-4th-differences) | third-party summary of the book's pace and heart-rate tables, and of the 3rd→4th edition changes | **third-party** |
| **T2** | fellrnr.com — [Pfitzinger](https://fellrnr.com/wiki/Pfitzinger), [A Comparison of Marathon Training Plans](https://fellrnr.com/wiki/A_Comparison_of_Marathon_Training_Plans) | third-party plan comparison tables | **third-party** |
| **T3** | buenavida.run — [18-week up to 55 mi/wk](https://buenavida.run/plans/pfitz-am-18wk-up-to-55), [12-week over 85 mi/wk](https://buenavida.run/plans/pfitz-am-12wk-over-85) | third-party review of the 4th-edition schedules, week by week | **third-party** |
| **T4** | joeyp.github.io/Pfitz-Pace-Tool | third-party calculator that claims to reproduce "Table 8.2" and the MP-relative pace bands | **third-party** |
| **T5** | r/AdvancedRunning, "Pfitzinger's Advanced Marathoning 4th Edition Changes" — [1m3eco0](https://www.reddit.com/r/AdvancedRunning/comments/1m3eco0/pfitzingers_advanced_marathoning_4th_edition/) | a reader's comparison of the 3rd and 4th editions, page by page | **third-party (a reader's report)** |
| **R1** | Seiler, *Int J Sports Physiol Perform* 2010 — [PDF](https://umh1617.umh.es/files/2016/05/02-IJSPP_2010_Seiler_0009.pdf) | the 80/20 research | **primary research** |
| **R2** | Stöggl & Sperlich, *Front Physiol* 2015 — [PMC4621419](https://pmc.ncbi.nlm.nih.gov/articles/PMC4621419/) | the polarised-TID trials | **primary research** |
| **R3** | 80/20 Endurance — [intensity guidelines](https://www.8020endurance.com/intensity-guidelines-for-80-20-running), [understanding your plan](https://www.8020endurance.com/understanding-your-8020-run-plan), [marathon plans](https://www.8020endurance.com/training-plans/running/marathon/) | the published polarised plan (Fitzgerald's product) | **product documentation** |

### 0.2 How it was gathered, and what to distrust

* **The book was read from the EPUB in this thread's storage.** It was extracted with Python's `zipfile` (chapters are `OEBPS/xhtml/E6202_chapNN_dNN.html`); the three appendices are **images** inside the EPUB and were rendered and read as images. The book file stays in thread storage; nothing from it was copied into the repository beyond the **rules and numbers** extracted here, each with a chapter citation, plus a handful of short quotations where the wording itself carries the rule.
* **Reddit** returned HTTP 403 to the Tavily extractor and to plain `curl`; the **Parallel** extractor retrieved the thread successfully (OP + comments). No quote here is reconstructed from memory.
* **LetsRun** (`letsrun.com/forum/flat_read.php?thread=10399470`) refused both the Tavily extractor and a direct fetch with HTTP 403. Its content is therefore **not used**; where it is referred to below it is flagged as unreadable.
* **The book is the 2nd edition of *Faster Road Racing* (2015)**, not the marathon book. Its plan structures cover 5K, 8K–10K, 15K–10-mile, half marathon and multi-distance. **The marathon book was not available**, so every marathon number below is sourced to S2 (a 2001 author-written excerpt), S3 (a 2025 interview), S4 (the author's own site) or a labelled third party — never to the book itself.

### 0.3 Confidence labels

* **[book]** — stated in *Faster Road Racing* 2nd ed., with chapter (and section) cited.
* **[author]** — the author's own published statement (interview, his site, the publisher).
* **[author-plan]** — the author's own plan, quoted from an author-written excerpt of the 1st-edition marathon book.
* **[3rd]** — a third-party summary, unverified against the book, with its link given.
* **[research]** — published, peer-reviewed.
* **[derived]** — arithmetic from a cited number, shown so it can be checked.

### 0.4 What this file does not do

It does not reproduce the book's schedules cell by cell. It extracts the **shape**, the **rules** and the **numbers**, because the app will re-express them as dated sessions with paces. Where a schedule's cells were read directly (the half-marathon 46–63 mi/wk and the 8K–10K 30–42 mi/wk schedules) that is said explicitly.

### 0.5 Scope — what ships, and what was extracted but does not ship

**The user's call, 24 November 2026: *"just have 10k and HM plans, cut even 5k for now."*** The product therefore ships **two plan templates — 10K and half marathon** — and holds **one base at a time** (Pfitzinger first; 80/20 as the second, separate method). The audience is the **intermediate runner**.

Everything extracted below that is **not** one of those two templates is kept here as knowledge, marked, and **not** built:

| extracted | ships in the first drop? | why |
|---|---|---|
| **The 8K–10K plan, tiers 1 (30–42 mi/wk) and 2 (45–57 mi/wk)** — ch. 10 | **YES** — the **10K** template | the shipping 10K plan |
| **The half-marathon plan, tiers 1 (31–47 mi/wk) and 2 (46–63 mi/wk)** — ch. 12 | **YES** — the **HM** template | the shipping HM plan |
| The 8K–10K tier 3 (60–76 mi/wk) | no | beyond the intermediate audience |
| The half-marathon tiers 3 (61–84) and 4 (81–100 mi/wk) | no | beyond the intermediate audience |
| **The 5K plan** — ch. 9 | **no** | cut by the user |
| The 15K and 10-mile plan — ch. 11 | no | cut (the 10K and HM templates cover the shipping distances) |
| The multi-distance plan — ch. 13 | no | cut |
| The three base-training schedules — ch. 8 | no, **but referenced** | both shipping plans point a too-low-mileage runner at it ("if you are running fewer than 25 miles per week, follow the base-training schedule in chapter 8 before attempting this schedule") |
| **Every training component** — ch. 1, 2, 3, 6, 7 | **YES** | the 10K and HM plans are built from exactly these components |

**The cut removes plan structures, not rules.** The run vocabulary, the hard/easy principle, the recovery-week and recovery-run rules, the taper rules, the pace rules and the "adjusting the training plan" guidance all still stand, because the 10K and HM plans are assembled from them.

---

# Part 1 — Every number a Pfitzinger plan needs

## 1.1 The book's run types and their rules — the master table

The book names **eight types of runs** on a schedule day: *endurance, general aerobic, recovery, progression long runs, lactate threshold, VO₂max, speed, tune-up races* (ch. 7, "Types of Running Workouts"; ch. 9 lists them in the same order). A ninth, **rest or cross-training**, appears on the calendar but is not a run.

| the rule | the exact number | the unit it is stated in | source | confidence | can Stride compute it, and what it would need |
|---|---|---|---|---|---|
| **Endurance run (the long run)** — heart rate | **74–84 %** of maximal HR; **65–78 %** of HR reserve | % of maxHR / % of HRR | S1 ch. 7 "Endurance Runs"; same figures in ch. 1 "Gaining the Most Benefit From Your Long Runs" | **[book]** | **Yes.** Stride has the athlete's dated zone model (5 floors + `max_heart_rate`, `crates/stride-core/src/zones.rs`, `crates/stride-store/src/model.rs`) and daily `resting_hr` (`repos/wellness.rs`), so HRR is computable. Needs max HR (from Garmin's model) and resting HR (from wellness) to be present; neither is guaranteed. |
| **Endurance run** — pace | **20–33 % slower than 10K race pace**, or **17–29 % slower than 15K-to-half-marathon pace** | % slower than a race pace | S1 ch. 7; ch. 1 table 1.1 | **[book]** | **Yes**, once a race time exists: the athlete types one race time; the engine looks up the equivalent 10K and 15K–HM times (Appendix B) and applies the band. Needs a race result. |
| **Endurance run** — shape | start at the slow end of the range, **gradually increase the pace** through the run; "finish your long runs at a strong pace" | — | S1 ch. 1 (worked example: 10K at 6:00/mi → start just under 8:00, work down to ~7:12); ch. 7 | **[book]** | **Partly.** The band is computable; "gradually increase" is a shape, not a number. Stride's pace bands (§`crates/stride-core/src/pace.rs`) can hold a distribution, so "early vs latter" is expressible if the plan stores two pace windows for the session. |
| **Progression long run** | start at standard long-run intensity; **increase to about 84 % of maxHR by halfway**; continue to **LT pace for about the last 3 miles**. (ch. 7 example: 14-mile; ch. 1 example: 16-mile, ~20 % slower than 10K pace by halfway.) Included **sparingly** | % maxHR; miles at LT pace | S1 ch. 7 "Endurance Runs"; ch. 1 | **[book]** | **Partly.** Needs a race time (for LT pace) and a stored "last N miles at LT" structure on the session. |
| **General aerobic run** | **70–81 %** of maximal HR; **62–75 %** of HR reserve. "The effort should be greater than during a recovery run and less than during a long run… conversational but should not feel like a jog." | % maxHR / % HRR | S1 ch. 7 "General Aerobic Runs"; repeated in ch. 8 "Reading the Base-Training Schedules" | **[book]** | **Yes, as a heart-rate band.** Note: **the book gives GA no pace rule at all** — only a heart-rate band. A pace for GA would be the app's inference, not Pfitzinger's. |
| **Recovery run** | **below 76 %** of maximal HR; **below 70 %** of HR reserve. Or: **about 2 minutes per mile (75 sec/km) slower than 15K-to-half-marathon race pace.** "make sure the pace feels appreciably easier on your recovery runs than on your general aerobic runs… Try to run on relatively flat terrain" | % maxHR / % HRR; min/mi slower | S1 ch. 7 "Recovery Runs"; ch. 2 "How Easy?" (same 76 %/70 % figures with a worked example: maxHR 190, resting 50 → below 144 bpm by %max, below 148 bpm by HRR) | **[book]** | **Yes.** Both forms are computable: the HR ceiling from the zone model, the pace from a race time. |
| **Lactate threshold run (tempo)** — intensity | **80–91 %** of maximal HR; **75–88 %** of HR reserve | % maxHR / % HRR | S1 ch. 7 "Lactate Threshold (Tempo) Runs"; ch. 1 "Lactate threshold" | **[book]** | **Yes.** |
| **Lactate threshold** — what pace it is | LT pace ≈ **the pace you could race for about an hour**. For faster runners LT pace ≈ **15K-to-10-mile race pace**; for elite runners, **15K-to-half-marathon pace**. For slower runners or those racing shorter distances: **10–15 s/mi slower than 10K race pace**, or **20–30 s/mi slower than 5K race pace** | s/mi slower than a race pace | S1 ch. 7; ch. 1 | **[book]** | **Yes**, from a race time: the athlete types a 5K, 10K, 15K, 10-mile or HM time; the engine picks the right offset for that athlete's level. Note the branch — the rule changes with the athlete's race distance, so the app needs to know which distance the athlete races. |
| **Classic tempo run** | **20 to 40 minutes** continuous, at **LT pace to 10 s/mi faster than LT pace**; example structure: 10–20 min warm-up jog, 20–30 min at intensity, cool-down jog | minutes; s/mi faster than LT | S1 ch. 7; ch. 1 "Classic Tempo Runs" | **[book]** | **Yes**, as a duration + pace band, once LT pace is known. |
| **LT intervals (cruise intervals)** | same intensity, steady pace, short jog recovery. Pete's example: **16 min + 12 min + 8 min at LT with 4-min jogs = 36 min at LT** | minutes | S1 ch. 1 "LT Intervals"; ch. 7 | **[book]** | **Yes**, as a structured session. |
| **LT hills** | "just **add 4 beats per minute** to your LT heart rate range" | bpm | S1 ch. 1; ch. 7 | **[book]** | **Yes.** |
| **Change-of-pace tempo** | fast segments **5–15 s/mi faster than LT**, steady segments **up to 10 s/mi slower than LT**; **20–45 min** alternating; the first fast component **at least 4 min**, later fast efforts 1–4 min, steady components **at least 4 min**. Worked example: LT 6:00 → fast 5:45–5:55, slow 6:00–6:10. Heart rate rises 5–6 bpm above LT in the fast part and drops to the middle/lower LT range in the steady part | s/mi, minutes, bpm | S1 ch. 1 "Change-of-Pace Tempo Runs"; ch. 7 | **[book]** | **Yes**, as a structured session with two pace windows. |
| **VO₂max intervals** — intensity | **3K-to-5K race pace** (some 8K/10K-plan workouts run at 8K-to-10K race pace). Heart rate **94–98 %** of maxHR; **92–97 %** of HR reserve | % maxHR / % HRR | S1 ch. 7 "VO₂max Intervals"; ch. 1 | **[book]** | **Yes.** |
| **VO₂max intervals** — shape | intervals of **2 to 6 minutes** at **95–100 % of VO₂max**, total **5,000–8,000 m (3–5 mi)**; for most experienced runners **≈500–1,600 m** per rep. **Recovery jog 50–90 % of the interval time** (example: 6 × 1,000 m in 3:20 → jog 1:40–3:00). Warm-up **10–20 min easy**, last minutes up to LT pace, then drills/stretching and **3–6 strides**; cool-down **10–20 min easy** | minutes, metres, % of interval time | S1 ch. 1 "VO₂max training"; ch. 7 | **[book]** | **Yes**, if the plan stores the interval structure (rep distance, count, recovery rule) — today's `PlanSession` has none of that. |
| **Speed work — strides** | **~100 m**; accelerate to full speed by halfway, hold, then decelerate; usually **after a general aerobic run**. **No heart-rate values** are given (HR is still rising when the runner slows to a jog) | metres | S1 ch. 7 "Speed Training" | **[book]** | **Partly.** The *fact* of strides is verifiable from lap structure (Stride's `repeated work` row, D4, counts steps named `interval`); the intensity is not measurable by HR. |
| **Speed work — uphill sprints** | **~12 seconds**, near all-out; walk back down for full recovery; typically **10–12 s** | seconds | S1 ch. 7 "Speed Training"; "Hills — Short hill repetitions" | **[book]** | **Partly** — same as strides; the watch can see short segments, not the intent. |
| **Speed work — speed repetitions** | **150 to 300 m** at **800 m-to-mile race pace**; recovery jog the same distance or slightly longer, and **may take up to three times as long** as the repetition. Included in the **5K, 8K/10K and multi-distance** plans | metres | S1 ch. 7 "Speed Training" | **[book]** | **Yes**, as a structured session; Appendix C carries the split times. |
| **Hills — VO₂max uphill** | **2 to 6 minutes** uphill at VO₂max effort; recovery slightly longer than the uphill | minutes | S1 ch. 7 "Hills" | **[book]** | **Partly** — pace verifiable, intent not. |
| **Hills — VO₂max up/down** | **4 to 6 efforts of 4 minutes** with the same recovery as a VO₂max workout | minutes | S1 ch. 7 "Hills" | **[book]** | **Partly.** |
| **Tune-up races** | **two per schedule** in the 5K, 8K/10K, 15K/10-mile and half-marathon plans; **one** in the multi-distance plan. Shorter than the goal distance. "you will not be completely fresh… expect residual fatigue" | count per plan | S1 ch. 7 "Tune-Up Races"; ch. 9–13 | **[book]** | **Yes** — a dated race in the plan. Stride has no race store, so it must be a plan session, not an activity type. |
| **Doubles** | one run a day in most plans. **5K schedules include doubles above 55 mi/wk**; **half-marathon schedules stay at one run a day until mileage tops 70 mi/wk**. **At least eight hours between the two runs** | hours | S1 ch. 7 "When to Double" | **[book]** | **Yes** — a plan with two sessions on a day; today's `PlanSession` is one-per-day, so this needs a change to the plan model. |
| **Recovery week** | **no more than 80 % of normal training volume**; long runs **no more than 80 % of normal distance**; **no VO₂max or tempo**; a short speed session is fine; cross-training may replace one or more runs. "Most runners respond well to a cycle of two to four harder training weeks followed by a recovery week" | % of normal volume | S1 ch. 2 "Recovery Weeks" | **[book]** | **Yes**, as a week-level property of the plan. |
| **Hard/easy pattern** | one hard day → one easy day or day off (up to 4 runs/week). Two hard days in a row are allowed **if they are different types** (e.g. VO₂max then long run). Within a seven-day week: **two hard, two easy, two hard, one easy** | — | S1 ch. 2 "Hard/Easy Training Patterns" | **[book]** | **Yes**, as a plan shape. |
| **Mileage progression** | **no more than 10 % per week**, and **no more than three weeks in a row** of increases. Daniels' rule: **no more than 1 extra mile per session per week** (5 runs/week → ≤5 mi/wk) | % per week; miles | S1 ch. 1 (and ch. 8) | **[book]** | **Yes**, as a constraint on a generated plan. |
| **Taper** | **two weeks** is optimal for 5K through half marathon; mileage reduction of **20–60 %**; keep the intensity in **smaller doses** (e.g. 6 × 1,200 m becomes 6 × 800 m at the same pace); **higher-intensity training every three to four days**; a **one-week** taper for moderately important races and a **four-day** mini-taper for minor ones; a well-planned taper is worth **2–3 %** | weeks; % volume reduction | S1 ch. 6 "Duration of Taper", "How to Reduce Your Training" | **[book]** | **Yes.** |
| **Long-run frequency** | "During most training weeks, you will have a long run **as well as another endurance-building run**" | — | S1 ch. 1 | **[book]** | **Yes** — the plan's shape, and the reason Stride's "longest run" row is not the same as "the long run". |

## 1.2 The pace tables the book publishes (the pace engine's raw material)

* **Appendix B — Equivalent Race Performances**: 5K → 8K → 10K → 15K → 10-mile → half marathon, in 30-second steps. Read directly from the book's table image. Example rows: a 20:00 5K → 32:59 8K → 41:36 10K → 1:03:51 15K → 1:09:28 10-mile → 1:32:21 half marathon; a 25:00 5K → 41:14 8K → 52:00 10K → 1:19:49 15K → 1:26:50 10-mile → 1:55:26 HM. **[book]**
* **Appendix C — Workout Paces**: the same rows plus the workout pace ranges — **long run (early per mile / latter per mile / early per km / latter per km), lactate threshold (per mile and per km), VO₂max (per 400 m), speed (300 m and 200 m)**. **The structure is verified; the individual cells were not transcribed.** The table is a low-resolution image that renders transposed, and rows could not be aligned to race times with confidence across crops — so **no cell value is published here**. This costs the product nothing, because the pace engine computes the equivalents itself from the athlete's race time (§5.5); Appendix C matters only if the app ever wants to quote the book's own tabulated paces. **[book, structure only]**
* **Appendix A — Pace Chart**: split times at a constant pace for any distance, for VO₂max sessions, LT sessions, races and race-pace workouts. **[book]**
* **Chapter 7, Table 7.1**: the recommended **heart-rate ranges for every workout type**, in one table. **[book]** (the table itself is an image in the EPUB; the ranges it contains are the ones quoted in the prose above, which is how they are cited here).

**This is the important structural finding:** the book's own pace engine is **race time → equivalent race times (Appendix B) → workout paces (Appendix C)**, plus a **heart-rate table (7.1)**. It is **not** VDOT-based, and it is not marathon-pace-based. The plan's paces are written against the athlete's **own race distances**, and the marathon book is the only place marathon pace appears.

## 1.3 The marathon numbers (the marathon book is not in hand) — **EXTRACTED, NOT SHIPPING**

These are the numbers the original brief asked for, for a **marathon** Pfitzinger plan. **No marathon plan ships in the first drop** (§0.5), so this section is kept for the record and for the day a marathon template is considered — but **none of it is on the path for the 10K or half-marathon plans**, and its MP-relative pace bands are the weakest-sourced numbers in this file. Sourcing is necessarily second-hand: S2 is an author-written excerpt of the 1st edition, S3 and S4 are the author's own statements, and T1–T5 are labelled third parties.

| the rule | the exact number | the unit it is stated in | source | confidence | can Stride compute it, and what it would need |
|---|---|---|---|---|---|
| **General aerobic (easy) runs** | **15 % to 25 % slower than marathon pace**; **70–81 %** of maxHR; **62–75 %** of HRR | % slower than MP | T1 [runningwithrock.com/pfitz-marathon-training-explained](https://runningwithrock.com/pfitz-marathon-training-explained) (summarising the book); the same GA heart-rate band is confirmed independently by the book (S1 ch. 7, ch. 8) | **[3rd]** for the pace band; **[book]** for the HR band | **Yes.** MP comes from a goal marathon time (S5 Appendix A, "Marathon Race-Pace Chart"). The pace band itself is unverified against the 4th edition's text. |
| **Long run** | **10 % to 20 % slower than marathon pace**; **65–78 %** of maxHR; **74–84 %** of HRR | % slower than MP | T1; T2 gives the same "MP+10 % to MP+20 %"; the heart-rate band **74–84 % maxHR / 65–78 % HRR** is independently confirmed by the book (S1 ch. 1, ch. 7) | **[3rd]** for the MP band; **[book]** for the HR band | **Yes**, from a goal marathon time. |
| **Long run shape** | "gradually increase from a slow early pace as you warm up, reaching **20 % slower** than your goal marathon race pace **at five miles** and accelerating to **10 % [slower]** than marathon pace during the **last five miles** of the run." (The 2001 text reads "10 % below marathon pace"; the 2025 interview states "running just say 10 % slower than marathon race pace in the last, say, eight miles".) | % slower than MP, at mile 5 and last 5 miles | S2 [Runner's World, 2001](https://www.runnersworld.com/advanced/a20819355/marathon-advantage); S3 (interview, 2025) | **[author-plan]** for the 1st-edition plan; **[author]** for the 2025 restatement | **Yes** — two pace windows on one session. |
| **Long run length** | "A long run is any run of **17 miles or longer**"; "a 22-mile long run will take approximately the same amount of time as your marathon" | miles | S2 | **[author-plan]** (1st ed.) | **Yes**, as a plan distance. |
| **Medium-long run** | "any run of **11 to 16 miles**"; pace "similar to the pace for long runs" | miles | S2 | **[author-plan]** | **Yes.** |
| **Lactate threshold run** | "at least **20 minutes** at your lactate threshold pace… coincides closely with your current **15K to half-marathon race pace**… tempo runs in this schedule are from **four to seven miles** long"; **82–91 %** of maxHR, **77–88 %** of HRR (T1); the 4th edition prescribes LT by **time** rather than distance (T1, T5) | minutes; miles; % maxHR | S2; T1; T5 | **[author-plan]** for the 1st-edition shape; **[3rd]** for the HR band and the 4th-edition change | **Yes.** The 4th edition's time-based LT (e.g. "11–12 miles with 35–45 minutes LT", where a 7-mile LT used to be) is **third-party**; it matches the book's own 20–40 min tempo rule. |
| **Marathon pace** | the **expected** goal marathon pace; **79–88 %** of maxHR, **73–84 %** of HRR | % maxHR | S2 ("run most of the miles at your goal marathon pace"); T1 for the HR band | **[author-plan]** + **[3rd]** | **Yes**, from the goal time. |
| **VO₂max** | intervals of **600 m to 2,000 m** at **95–100 %** of VO₂max pace, "coincides closely with your current **3K to 5K race pace**"; the optimal duration is **two to six minutes**; no rep longer than 1,600 m; marathoners should stay at **5K pace** rather than 3K pace. The author in 2025: "The workouts we have are at **95 % of VO₂ max**" | metres; % VO₂max | S2; S3 | **[author-plan]** + **[author]** | **Yes**, from a recent 5K/3K time — note the 4th edition **dropped the goal-time → 5K-pace chart**, so the athlete must supply a recent 5K (T3). |
| **Speed / strides** | repetitions of **50 to 150 m**; often at the end of a GA or recovery run; allow plenty of rest | metres | S2 | **[author-plan]** | **Partly.** The 4th edition adds **6 × 10 s hill sprints** (T3). |
| **Recovery run** | "relatively short runs done at a relaxed pace… they should be **noticeably slower** than your other workouts of the week" | — | S2 | **[author-plan]** | **Partly** — no number in the 1st-edition excerpt. T4 gives a recovery ceiling of **< 75 % maxHR / < 66 % HRR** and T5 a "heart-rate ceiling for recovery runs" in the 4th edition; neither number is verified against the book. **The book's own recovery ceiling (76 % maxHR / 70 % HRR, S1) is the one Stride should use** — it is primary. |
| **Marathon vs LT pace** | "you tend to do the marathon just a couple of percent slower than lactate threshold. For at least well-trained runners, probably doing the marathon **two to three percent slower** than lactate threshold" | % slower than LT | S3 (interview, 2025) | **[author]** | **Yes** — a conversion, and a reason to prefer LT pace as the anchor. |
| **Marathon-pace runs** | 18-week programs have **four**; 12-week programs have **three** | count per plan | S3 (interview, 2025) | **[author]** | **Yes.** |
| **Mesocycles** | **four** in the 2001 excerpt (endurance → lactate threshold + endurance → race preparation → tapering); the 4th edition is described as **five** (the fifth, a recovery plan, comes after the marathon) | count | S2; T1 | **[author-plan]** (4) / **[3rd]** (5) | **Yes** — a phase label on a week. |
| **Plan lengths and peak mileage** | **12 or 18 weeks**; peak **55, 70, 85 or over 105 miles/week** | weeks; mi/wk | S5 (publisher); T1; S3 | **[author]** / **[publisher]** / **[3rd]** | **Yes.** |
| **"Eight weeks out, three weeks down"** | **Both halves are Pfitzinger's, but they are different things.** The 2001 schedule has an explicit **"Recovery Week 8"** column (59 miles, at 8 weeks to goal), and the taper is the **last three weeks** (56 → 46 → 30 miles + marathon). The *sharpening* of the long run at eight weeks out is **Bakken's Norwegian method**, not Pfitzinger's | — | S2 (the schedule table); and `exploration/methods-and-products.md` §2.2 for the Bakken attribution | **[author-plan]** + **[3rd]** for the Bakken half | **Yes** — a recovery-week flag at week 8 and a 3-week taper. |
| **Race week** (marathon) | Mon rest; Tue recovery 5 a.m. + 4 p.m.; Wed **dress rehearsal 7 miles with 2 miles at marathon pace**; Thu recovery 5; Fri recovery + speed 5 with 6 × 100 m strides; Sat recovery 4; Sun **goal marathon**. Weekly total **30 miles (6 days pre-race) + the marathon** | miles; sessions | S2 | **[author-plan]** (1st ed.) | **Yes.** The 4th edition returns goal pace to race week as ~**2.5 miles** (T3). |
| **Doubles** (marathon) | "doubles are called for only on the **occasional recovery day**, with a total of **10 miles** for the day… a **6-miler and a 4-miler**" | miles | S2 | **[author-plan]** | **Yes.** The 4th edition reduced doubles further; the **18/70** schedule "no longer references doubles at all" (T1). |

## 1.4 The ten numbers that matter for a Pfitzinger plan

Ranked by how much of the plan depends on them.

| # | the number | source | confidence |
|---|---|---|---|
| **1** | **The athlete's goal race time at one distance (5K, 10K, 15K, 10-mile or half marathon)** — every other pace in the plan hangs off it | the plan's own anchor; S1 App. B, S4 | **[book]** (as the anchor of App. B/C) |
| **2** | **Equivalent race performances 5K→8K→10K→15K→10-mile→HM** (App. B, 30-second steps) | S1 App. B | **[book]** |
| **2b** | **The race-time → workout-pace tables themselves** (App. B + App. C) — the book's own pace engine, which is **not** VDOT and **not** marathon pace | S1 App. B, App. C | **[book]** |
| **3** | **General aerobic = 70–81 % maxHR / 62–75 % HRR** (the most common session in the plan) | S1 ch. 7, ch. 8 | **[book]** |
| **4** | **Recovery = below 76 % maxHR / below 70 % HRR, or 2 min/mi slower than 15K–HM race pace** | S1 ch. 7, ch. 2 | **[book]** |
| **5** | **Long run = 74–84 % maxHR / 65–78 % HRR; 20–33 % slower than 10K pace, 17–29 % slower than 15K–HM pace; start slow, finish fast** | S1 ch. 1, ch. 7 | **[book]** |
| **6** | **LT pace = the pace you could race for about an hour** (15K–10-mile race pace; or 10–15 s/mi slower than 10K pace / 20–30 s/mi slower than 5K pace for shorter-distance runners); **80–91 % maxHR / 75–88 % HRR** | S1 ch. 1, ch. 7 | **[book]** |
| **7** | **LT session = 20–40 min** continuous (or 16/12/8-min intervals with 4-min jogs) | S1 ch. 1, ch. 7 | **[book]** |
| **8** | **VO₂max = 3K–5K race pace; 94–98 % maxHR / 92–97 % HRR; reps of 2–6 min, 5,000–8,000 m total; recovery jog 50–90 % of interval time** | S1 ch. 1, ch. 7 | **[book]** |
| **9** | **Recovery week = ≤ 80 % of normal volume, long run ≤ 80 % of normal distance, no VO₂max/tempo** | S1 ch. 2 | **[book]** |
| **10** | **Taper = 2 weeks, 20–60 % mileage cut, intensity maintained in smaller doses** | S1 ch. 6 | **[book]** |

*For the marathon plans specifically, number 1 becomes a **goal marathon time**, and the MP-relative bands (**GA 15–25 % slower, long run 10–20 % slower**) come in at **[3rd]** confidence; the book's own sub-marathon bands above are the primary ones.*

## 1.5 Where the sources disagree, and where the editions differ

| the point | the disagreement | which to believe |
|---|---|---|
| **GA heart-rate band** | T1: **70–81 %** maxHR / 62–75 % HRR. T4: **72–80 %** / 62–73 %. S1 (book): **70–81 %** / 62–75 % | **The book.** T4's "Table 8.2" claim is a third-party rendering of a *different* book (*Advanced Marathoning*) and disagrees with both T1 and S1. |
| **Long-run HR band** | T1: 65–78 % maxHR / 74–84 % HRR. T4: 75–83 % / 66–77 %. S1: **74–84 %** / 65–78 % | **The book.** (T1 has the HRR and %maxHR figures swapped relative to S1; T4's figures are close to S1's %maxHR but its HRR does not match.) |
| **LT heart-rate band** | T1: 82–91 % / 77–88 %. T4: 83–90 % / 77–87 %. S1: **80–91 %** / 75–88 % | **The book.** |
| **Recovery ceiling** | T4: < 75 % maxHR / < 66 % HRR. S1: **< 76 % / < 70 %** | **The book.** |
| **Number of mesocycles** | 4 (S2, 1st ed.) vs 5 (T1, 4th ed.) | Both, for their editions: the fifth is a **post-race recovery plan**. |
| **LT prescribed by distance or by time** | 3rd edition: miles ("12 miles with 7 miles at LT"). 4th edition: **time** ("11–12 miles with 35–45 minutes at LT") — T1, T5 | The 4th edition, if Stride ships the 4th-edition plan. The book's own tempo rule is **20–40 min**, so the 4th edition is *more* consistent with the book's science than the 3rd. |
| **Doubles** | Present in the 1st-edition marathon plan (6+4 on recovery days, S2); cut back in the 4th edition and removed from 18/70 (T1) | The 4th edition, for a current app. |
| **Marathon pace vs LT pace** | S2: the marathon is "a couple of percent slower than lactate threshold" (2025 interview: 2–3 %); T3 quotes marathoners using MP as their aerobic threshold. | Not a contradiction — S3 is the author's own statement and is the one to cite. |
| **MP-relative vs LT-relative pacing** | The marathon book is written relative to **marathon pace**; the sub-marathon book is written relative to **race pace and heart rate**, and Pfitzinger's own current coaching site (S4) says "The optimal paces for the various types of workouts… are determined relative to **lactate threshold pace**" | A real generational shift. **Stride should decide which anchor it implements** and label it — see §5.4. |

---

# Part 2 — The plan structures, from the book

## 2.1 Base training (ch. 8) — three 10-week schedules — **EXTRACTED, NOT SHIPPING**

Both shipping plans point a too-low-mileage runner at these ("if you are running fewer than 25 miles per week, follow the base-training schedule in chapter 8 before attempting this schedule"), so the knowledge is kept; the templates are not built.

| schedule | starts | ends | runs per week | shape |
|---|---|---|---|---|
| Building up to **30 mi/wk** | **16 mi/wk** | **30 mi/wk** | **4** | Tue GA · Wed rest/cross · Thu GA/LT · Fri rest/cross · Sat GA+strides · Sun long run |
| Building up to **45 mi/wk** | **27 mi/wk** | **45 mi/wk** | **5** | Tue · Wed · Thu · Sat · Sun |
| Building up to **60 mi/wk** | **38 mi/wk** | **60 mi/wk** | **5 → 6** | as above |

Fixed rules of the base schedules, straight from ch. 8: **10 weeks each**; weekly mileage follows "**increasing for two weeks followed by a plateau** before increasing again"; **LT workouts in weeks 5, 7 and 9**; **a set of 100 m strides in weeks 4, 6, 8 and 10**; **Sunday long runs** in every schedule; general aerobic runs are the bulk of the week at **70–81 % maxHR**; "you can jump in wherever your current mileage happens to be" (running 20 mi/wk → start at week 6 of the 30-mile schedule).

## 2.2 The race plans (ch. 9–13) — every distance, in numbers

All are **12 weeks** (ch. 9–12) with **weeks-to-go counted down from 11 to 1 plus race week**; ch. 13 (multiple distances) is **10 weeks**. **Only the 8K–10K and half-marathon rows ship** (see §0.5).

| distance | schedules (mi/wk) | ships? | entry requirement | primary emphasis | tune-up races | after the race |
|---|---|---|---|---|---|---|
| **5K** (ch. 9) | **30–40**, **45–55**, **60–70** | **no — cut** | training 25–35 / 40–50 / 55–65 mi/wk; below that, do the ch. 8 base schedule first | **VO₂max** and two tune-up races; secondary: long runs, mileage, LT, speed | **5K at 4 and 2 weeks to go** (or a **3K time trial** at 2 weeks) | **1 recovery week**; "after a hard 5K race, you need about five days to a week to fully recover" |
| **8K & 10K** (ch. 10) | **30–42**, **45–57**, **60–76** | **YES for tiers 1–2**; tier 3 no | 25–35 / 40–50 / 55–65 mi/wk | VO₂max + speed + LT | two (8K/10K) | recovery + transition |
| **15K & 10-mile** (ch. 11) | **31–45**, **46–60**, **61–80** | **no — cut** | 25–35 / 40–50 / 55–65 mi/wk | LT + long runs; **progression long runs** appear from here on | two | recovery + transition |
| **Half marathon** (ch. 12) | **31–47**, **46–63**, **61–84**, **81–100** | **YES for tiers 1–2**; tiers 3–4 no | 25–35 / 40–50 / 55–65 / 75–85 mi/wk | **LT pace** via tempo runs, LT intervals, **three long progression runs**, two tune-up races; secondary: mileage, VO₂max | **8K–10K at 4 and 2 weeks to go** | **2 weeks**: a recovery week then a transition week; "after the half marathon, you need about **10 days to two weeks** to fully recover" |
| **Multiple distances** (ch. 13) | three levels | **no — cut** | — | balanced preparation across 5K–HM | **one** tune-up race | — |

Other fixed rules across the race plans:

* **A minimum of 8 weeks**: "you can still get into racing shape by joining a training program **no less than 8 weeks out**" (ch. 9, ch. 12).
* **A recovery week is printed on the schedule** — in the half-marathon 46–63 schedule it is the week 8 column; the book's general rule is two to four hard weeks then a recovery week.
* **The goal race's day is fixed in the schedule**: 5K on **Saturday** (add a GA run on Wednesday for a Sunday race); half marathon on **Sunday** (for a Saturday race, drop one GA run in race week and shift everything a day earlier). **[book]**
* **Hilly races**: replace the LT sessions at 10 and 7 weeks to go with **LT hills**, and the VO₂max session at 6 weeks to go with **6–8 × 3 minutes uphill** (ch. 12; ch. 9 gives the 5K equivalents: LT at 11 and 9 weeks, VO₂max at 5 and 3 weeks → 5–7 × 3 min uphill).
* **Race strategy** (ch. 9, ch. 12): 5K is run at **~95 % of VO₂max pace**; the half marathon is run **5–15 s/mi slower than LT pace**; even pacing, with the pace chart in Appendix A as the guide.

## 2.3 The two shipping programmes, read cell by cell

**All four shipping schedules have now been read cell by cell from the book's schedule images**, and every daily cell reconciles with the printed weekly mileage (the check is in each table's last column). Legend: **Rest** = "Rest or cross-training"; **GA*n*** = general aerobic, *n* miles; **GA+s** = general aerobic + speed (strides and/or uphill); **End *n*** = endurance (the long run), *n* miles; **Prog *n* (*k* @LT)** = progression long run, last *k* miles at LT pace; **LT *n*** = lactate threshold, *n* miles total; **VO₂ *n*** = VO₂max, *n* miles total; **SR *n*** = speed repetitions, *n* miles total; **Rec *n*** = recovery, *n* miles; **RP *n*** = race pace.

### 2.3.1 The 10K programme, tier 1 — 30 to 42 miles (48–67 km) per week — ch. 10

| week | Mon | Tue | Wed | Thu | Fri | Sat | Sun | volume |
|---|---|---|---|---|---|---|---|---|
| 11 | Rest | GA+s 6 (6×12 s uphill + 6×100 m strides) | End 8 | Rest | **LT 7** (3 × 8 min, jog 3) | Rest | End 9 | **30** |
| 10 | Rest | SR 6 (2 sets of 4 × 200 m @ 800 m–mile race pace, jog 200 m, 4 min between sets) | End 8 | Rest | **LT 8** (10 min, jog 3, 10 min, jog 3, 8 min) | Rest | End 10 | **32** |
| 9 | Rest | GA+s 7 (6×12 s uphill + 6×100 m strides) | End 9 | Rest | **VO₂ 8** (6 × 3 min uphill @ 3K–5K race effort, jog downhill) | Rest | End 10 | **34** |
| **8 (recovery)** | Rest | SR 7 (2 sets of 4 × 200 m) | GA 7 | Rest | GA 7 | Rest | GA 9 | **30** |
| 7 | Rest | GA+s 7 | End 9 | Rest | **VO₂ 8** (5 × 1,000 m @ 3K–5K race pace, jog 50–90 %) | Rec 3 | End 10 | **37** |
| 6 | Rest | SR 8 (2 sets of 5 × 200 m) | End 9 | Rest | **LT 8** (12 min, jog 4, 10 min, jog 4, 10 min) | Rec 3 | End 11 | **39** |
| 5 | Rest | GA+s 8 (2 sets of 6 × 100 m strides) | End 9 | Rest | **RP 9** (4 × 1,200 m, 2 × 1,000 m) | Rec | End | **?** |
| 4 | Rest | GA+s 7 (2 sets of 4 × 150 m strides, jog 250 m, 4 min between sets) | GA 7 | Rest | Rec 3 + 4×100 m strides | **5K tune-up race 8** | End 9 | **34** |
| 3 | Rest | GA+s 8 (8 × 100 m strides) | End 9 | Rest | **VO₂ 9** (3 × 1,000 m + 3 × 800 m @ 3K–5K) | Rec 5 | End 11 | **42** |
| 2 | Rest | GA+s 7 (2 sets of 4 × 150 m strides) | GA 7 | Rest | Rec 3 | **5K tune-up race 8** | End 9 | **34** |
| 1 | Rest | GA+s 6 (8 × 100 m strides) | Rec 4 | **VO₂ 8** (4 × 800 m + 2 × 600 m @ 3K–5K) | Rest | Rec 4 | GA 8 | **30** |
| **race week** | **RP+s 6** (4 × 100 m strides, 1,200 m @ 8K–10K race pace) | Rec 3 | Rec 4 | Rest | Rec 3 + 4×100 m strides | **8K or 10K goal race 11** | Rec 4 | **31** |
| postrace | Rest | Rec 5 | Rec 5 | Rec 6 | — | — | — | — |

* **Long run:** Sunday, every week — 9 → 10 → 11 miles, and shorter (general aerobic) in the recovery and race weeks. A second endurance run sits on Wednesday.
* **Quality sessions:** one a week, and it **moves** — Friday in the first half (LT, VO₂max), then the last mile of the plan is a mix of LT (weeks 6), VO₂max (weeks 1, 3) and race pace (week 5). **The 10K plan has no progression long runs** — that session type arrives with the 15K and HM plans.
* **Cutback:** the **recovery week is week 8** (30 miles, no LT, no VO₂max, no speed reps; the Sunday run becomes a general aerobic run).
* **Tune-up races:** **5K at 4 weeks and 2 weeks to go**; the goal race is the 8K or the 10K.
* **Taper and race week:** volume 42 → 34 → 30 → 31; the race week opens with a race-pace session carrying **1,200 m at 8K–10K race pace**.

### 2.3.2 The 10K programme, tier 2 — 45 to 57 miles (72–91 km) per week — ch. 10

| week | Mon | Tue | Wed | Thu | Fri | Sat | Sun | volume |
|---|---|---|---|---|---|---|---|---|
| 11 | Rest | GA 8 | **LT 8** (10 min, jog 3, 8 min, jog 3, 8 min) | End 9 | Rec 3 | GA+s 7 (6×12 s uphill + 8×100 m strides) | End 10 | **45** |
| 10 | Rest | GA 8 | **LT 9** (12 min, jog 4, 10 min, jog 4, 8 min) | End 9 | Rec 3 | SR 7 (2 sets of 5 × 200 m @ 800 m–mile race pace) | End 11 | **47** |
| 9 | Rest | GA 8 | **VO₂ 9** (7 × 3 min uphill @ 3K–5K race effort) | End 9 | Rec 4 | GA+s 8 (6×12 s uphill + 8×100 m strides) | End 11 | **49** |
| **8 (recovery)** | Rest | GA 7 | GA 9 | SR 8 (2 sets of 5 × 200 m) | Rec 3 | GA 8 | End 10 | **45** |
| 7 | Rest | GA 9 | **VO₂ 9** (6 × 1,000 m @ 3K–5K race pace) | End 10 | Rec 4 | GA+s 8 (6×12 s uphill + 8×100 m strides) | End 12 | **52** |
| 6 | Rest | GA 9 | **LT 10** (14 min, jog 4, 10 min, jog 4, 10 min) | End 10 | Rec 5 | SR 8 (3 sets of 4 × 200 m) | End 12 | **54** |
| 5 | Rest | GA 9 | **RP 10** (2 sets of 1,600 m, 1,200 m, 1,000 m @ 8K–10K race pace, jog 50 %) | End 11 | Rec 5 | GA+s 9 (2 sets of 6 × 100 m strides, jog 3 min between sets) | End 12 | **56** |
| 4 | Rest | GA+s 9 (2 sets of 5 × 150 m strides) | GA 8 | Rec 7 | Rec 4 | **5K tune-up race 9** | End 10 | **47** |
| 3 | Rest | GA 9 | **VO₂ 10** (3 × 1,200 m + 3 × 800 m @ 3K–5K) | End 11 | Rec 5 | GA+s 9 (2 sets of 6 × 100 m strides) | End 13 | **57** |
| 2 | Rest | GA+s 9 (2 sets of 5 × 150 m strides) | GA 8 | Rec 7 | Rec 4 | **5K tune-up race 9** | End 10 | **47** |
| 1 | Rest | GA 7 | Rec 5 | **VO₂ 9** (4 × 800 m + 3 × 600 m @ 3K–5K) | Rec 4 | GA+s 6 (8 × 100 m strides) | GA 9 | **40** |
| **race week** | **RP+s 7** (4 × 100 m strides, 1,200 m @ 8K–10K race pace) | GA 6 | Rec 5 | Rest | Rec 4 + 4×100 m strides | **8K–10K goal race 11** | Rec 5 | **38** |
| postrace | Rest | Rec 5 | Rec 6 | Rec 6 | Rec 4 | GA+s 8 (8 × 100 m strides) | GA 10 | **39** |

* **Long run:** **two** endurance runs a week — Wednesday and Sunday — from the first week (End 9 + End 10), the long one on Sunday at 10 → 13 miles.
* **Quality sessions:** one a week, **Tuesday or Wednesday**: LT in weeks 11, 10, 6; VO₂max in weeks 9, 7, 3, 1; race pace in week 5. **A pace change comes in the second half**: the week-5 race-pace session is at **8K–10K race pace** (the 10K plan's own distance), while the VO₂max sessions stay at 3K–5K race pace.
* **Cutback:** the recovery week is **week 8** (45 miles).
* **Taper:** 57 → 47 → 40 → 38 (race week, including the 11-mile race).
* **Tune-up races:** 5K at 4 and 2 weeks to go.

### 2.3.3 The half-marathon programme, tier 1 — 31 to 47 miles (50–76 km) per week — ch. 12

| week | Mon | Tue | Wed | Thu | Fri | Sat | Sun | volume |
|---|---|---|---|---|---|---|---|---|
| 11 | Rest | GA+s 6 (8 × 100 m strides) | End 8 | Rest | **LT 7** (14 min, jog 4, 12 min) | Rest | End 10 | **31** |
| 10 | Rest | GA+s 7 (6×12 s uphill + 6×100 m strides) | End 8 | Rest | **LT 8** (18 min, jog 4, 12 min) | Rest | End 11 | **34** |
| 9 | Rest | GA+s 8 (6×12 s uphill + 6×100 m strides) | End 9 | Rest | **LT 8** (18 min, jog 4, 15 min) | Rest | **Prog 12 (2 @ LT)** | **37** |
| **8 (recovery)** | Rest | GA+s 8 (2 sets of 5 × 150 m strides, jog 250 m, 4 min between sets) | GA 7 | Rest | GA 8 | Rest | GA 9 | **32** |
| 7 | Rest | **LT 8** (20 min, jog 4, 16 min) | End 9 | Rest | GA+s 7 (10 × 100 m strides) | Rec 4 | **Prog 12 (3 @ LT)** | **40** |
| 6 | Rest | GA+s 8 (6×12 s uphill + 6×100 m strides) | End 9 | Rest | **VO₂ 9** (6 × 1,000 m @ 3K–5K) | Rec 5 | End 12 | **43** |
| 5 | Rest | **LT 9** (38 min tempo run) | End 10 | Rest | GA+s 8 (10 × 100 m strides) | Rec 5 | **Prog 13 (3 @ LT)** | **45** |
| 4 | Rest | GA+s 8 (2 sets of 4 × 150 m strides) | GA 7 | Rest | Rec 3 + 4×100 m strides | **8K–10K tune-up race 10** | End 10 | **38** |
| 3 | Rest | GA 8 | End 11 | Rest | **VO₂ 10** (2 sets of 2 × 1,200 m + 1 × 800 m @ 3K–5K) | Rec 4 | End 14 | **47** |
| 2 | Rest | GA+s 8 (2 sets of 4 × 150 m strides) | GA 7 | Rest | Rec 3 + 4×100 m strides | **8K–10K tune-up race 10** | End 10 | **38** |
| 1 | Rest | GA+s 6 (2 sets of 6 × 100 m strides, jog 3 min between sets) | Rec 4 | Rest | **VO₂ 9** (2 × 1,200 m, 2 × 1,000 m, 1 × 800 m @ 3K–5K) | Rec 3 | End 10 | **32** |
| **race week** | Rest | **RP+s 7** (4 × 100 m strides, 2 miles @ half-marathon race pace) | Rec 5 | Rec 4 | Rest | Rec 3 + 4×100 m strides | **half marathon goal race 16** | **35** |
| recovery | Rec 4 | Rest | Rec 5 | Rec 6 | Rest | GA+s 7 (8 × 100 m strides) | GA 8 | **30** |
| transition | Rest | GA 7 | GA 8 | Rest | GA+s 8 (2 sets of 6 × 100 m strides) | Rec 5 | GA 10 | **38** |

* **Long run:** Sunday — 10, 11, **12 (progression, last 2 mi at LT)**, 12 (progression, last 3), 13 (progression, last 3), then 14 at the peak and 10 in the taper. **Three progression long runs, in weeks 9, 7 and 5**, each ending at LT pace.
* **Quality sessions:** one a week, and it **moves from Friday to Tuesday** as the plan progresses — Friday LT in weeks 11–9, Tuesday LT in weeks 7 and 5, Friday VO₂max in weeks 6, 3 and 1.
* **Cutback:** the recovery week is **week 8** (32 miles, no LT, no VO₂max, no progression run).
* **Taper:** 47 → 38 → 32 → 35 (race week, including the 16-mile race); the race week carries **2 miles at half-marathon race pace**.
* **After the race:** a **recovery week** (30 miles) and a **transition week** (38 miles).

### 2.3.4 The half-marathon programme, tier 2 — 46 to 63 miles (73–102 km) per week — ch. 12

| week | Mon | Tue | Wed | Thu | Fri | Sat | Sun | volume |
|---|---|---|---|---|---|---|---|---|
| 11 | Rest | GA 9 | **LT 8** (15 min, jog 4, 12 min) | End 10 | Rest | GA+s 7 (2 sets of 6 × 100 m strides, jog 3 min between sets) | End 12 | **46** |
| 10 | Rest | GA 9 | **LT 9** (18 min, jog 4, 14 min) | End 11 | Rest | GA+s 7 (6×12 s uphill + 8×100 m strides) | End 13 | **49** |
| 9 | Rest | GA 9 | **LT 10** (20 min, jog 4, 16 min) | End 11 | Rest | GA+s 8 (6×12 s uphill + 8×100 m strides) | **Prog 14 (2 @ LT)** | **52** |
| **8 (recovery)** | Rest | GA 8 | GA 10 | GA 9 (2 sets of 5 × 150 m strides, jog 250 m, 4 min between sets) | Rest | GA+s 9 | End 11 | **47** |
| 7 | Rest | GA 9 | **LT 10** (22 min, jog 4, 18 min) | End 11 | Rest | GA+s 8 (2 sets of 6 × 100 m strides) | **Prog 14 (3 @ LT)** | **55** |
| 6 | Rest | GA 9 | **VO₂ 10** (3 × 1,200 m + 3 × 1,000 m @ 3K–5K) | End 12 | Rest | GA+s 8 (6×12 s uphill + 8×100 m strides) | End 12 | **58** |
| 5 | Rest | GA 9 | **LT 10** (40 min tempo run) | End 12 | Rest | GA+s 9 (10 × 100 m strides) | **Prog 16 (3 @ LT)** | **61** |
| 4 | Rest | GA+s 10 (2 sets of 5 × 150 m strides) | GA 9 | Rec 7 | Rest | GA+s 8 | End 12 | **63** |
| 3 | Rest | GA 9 | End 11 | **VO₂ 11** (6 × 1,200 m @ 3K–5K) | Rest | **8K–10K tune-up race 10** | End 12 | **52** |
| 2 | Rest | GA+s 10 (2 sets of 5 × 150 m strides) | GA 9 | Rec 7 | Rest | GA+s 8 | End 12 | **52** |
| 1 | Rest | GA 7 | Rec 5 | **VO₂ 10** (2 × 1,200 m + 4 × 800 m @ 3K–5K) | Rest | GA+s 6 (2 sets of 6 × 100 m strides) | End 11 | **43** |
| **race week** | Rest | **RP+s 8** (6 × 100 m strides, 2 miles @ half-marathon race pace) | Rec 7 | Rec 6 | Rec 4 + 4×100 m strides | Rest | **half marathon goal race 17** | **42** |
| recovery | Rec 4 | Rest | Rec 6 | Rec 6 | Rec 4 + 4×100 m strides | GA+s 8 | GA 9 | **37** |
| transition | Rest | GA 9 | GA 7 | GA 10 | Rec 5 | GA+s 8 (2 sets of 6 × 100 m strides) | End 11 | **50** |

* **Long run:** Thursday **and** Sunday — two endurance days a week, Sunday being the long one (12 → 13 → 14 → 14 → 16 → 12 → 11), with **three progression long runs** (weeks 9, 7 and 5, the last 2–3 miles at LT pace).
* **Quality sessions:** one a week, **Wednesday** throughout — LT in weeks 11, 10, 9, 7, 5 and VO₂max in weeks 6, 3, 1.
* **Cutback:** the recovery week is **week 8** (47 miles, no LT, no VO₂max, no progression run).
* **Taper:** 63 → 52 → 43 → 42 (race week, including the 17-mile race).
* **After the race:** a **recovery week** (37 miles) and a **transition week** (50 miles).

### 2.3.5 What the four schedules share, and what was **not** read

* **Shared shape:** Monday is **rest or cross-training** in every week of every tier; the plan runs **12 weeks**, weeks-to-go counted down from 11, then **race week**; a **recovery week is printed at 8 weeks to go** in all four; the plan ends with a **recovery week** and a **transition week** after the race (the 10K tier 1 has a single post-race recovery week).
* **The quality session is one a week** in every tier, and its **type changes with the phase**: LT first, VO₂max later, race pace in the last third — which is the book's own periodisation in miniature.
* **The long run is on Sunday** in every tier, with a second endurance day (Wednesday in the 10K tier 1, Thursday in the HM tier 2).
* **Not read cell by cell:** the 10K tier 1's post-race week beyond Thursday; **Table 7.2**; the cut plans (5K, 15K/10-mile, multi-distance), the higher-mileage tiers and the base-training schedules — all extracted, none shipping (§0.5).

## 2.4 The taper (ch. 6)

* **Two weeks is optimal** for 5K through half marathon; a **one-week** taper for moderately important races; a **four-day** mini-taper for minor ones.
* **Reduce volume, keep intensity**: 20–60 % mileage reduction; keep VO₂max, LT and speed in **smaller doses** at the same pace (6 × 1,200 m → 6 × 800 m); **higher-intensity training every three to four days**; non-running training pared back; a well-planned taper is worth **2–3 %** of race time.
* **Run frequency**: lower-mileage runners keep their run count; high-mileage runners drop **one or two runs per week**.
* **Sleep and food**: get into a regular sleep routine early in the taper; in a two-week taper, reduce caloric intake to match reduced expenditure (expect **2–4 lb** of glycogen-and-water gain).
* **Sample two-week taper** (ch. 6, Table 6.3): recovery runs Monday and Tuesday, a **reduced-volume VO₂max session** Wednesday, three easy days, a moderate endurance run, then race week = recovery run, a **reduced-duration tempo run** Tuesday, four easy days including a day off two days before, strides on Thursday to retain leg speed.

## 2.5 Adjusting the plan when life intervenes (ch. 7)

* **Move the session rather than skip it**: "if you have a VO₂max workout scheduled for Wednesday but your child falls ill, move the workout to Thursday instead of skipping the workout entirely… do not try to make up for lost time by stringing together too many hard days in a row."
* **Fit in the highest training priority for the week** rather than everything.
* **Weather**: GA and recovery runs can be done in poor conditions; VO₂max and speed sessions need good footing and tolerable temperatures.
* **Illness**: the "above the chest / below the chest" rule of thumb; most minor illnesses resolve in a few days.
* **Injury**: cross-train (pool, bike) to maintain fitness; **"if you're not able to resume full training more than four weeks out from a goal race, you should strongly consider finding a new one."**
* **Masters runners**: more recovery days, but "avoid backing off the intensity of training on hard days or cutting out too much mileage in the name of recovery" (ch. 7; ch. 5).
* **Base training**: "if you fall behind… do not try to make up for lost time; simply start where you left off and build up from there."

---

# Part 3 — The mapping the ledger needs: run type → our stimulus kind → what the watch can verify

Our stimulus kinds (from `exploration/stimulus-ledger.md` and D12/D14): **Z1–Z2 low · Z3 moderate · Z4–Z5 high · the longest run · repeated work**.

The third column is the point of this table: **which rows are evidence and which are the plan's words.**

| the book's run type | its rule (pace, duration, frequency) | our stimulus kind | what the watch can verify | what only the plan or the athlete can |
|---|---|---|---|---|
| **Endurance run (long run)** | 74–84 % maxHR / 65–78 % HRR; 20–33 % slower than 10K pace, 17–29 % slower than 15K–HM pace; start slow, finish faster | **the longest run** (and Z1–Z2 for its easy part) | the run exists, its distance and moving time, its time in each zone, its pace distribution (early vs latter) | that it was *this* week's long run; that the pace was in the band; that it was progressive |
| **Progression long run** | standard long-run intensity → ~84 % maxHR by halfway → LT pace for the last ~3 miles | the longest run + **Z3** (the LT segment) | the distance; the pace profile; the time in Z3 near the end | that the last miles were *meant* to be at LT pace |
| **General aerobic run** | 70–81 % maxHR / 62–75 % HRR; harder than recovery, lighter than a long run | **Z1–Z2 low** | zone time, pace bands, distance | that it was a GA run; the book gives GA **no pace rule** — only a heart-rate band |
| **Recovery run** | below 76 % maxHR / 70 % HRR, or ~2 min/mi slower than 15K–HM pace | **Z1–Z2 low** (the slow end) | zone time; whether the HR ceiling was respected | that it was a recovery run; that it was flat terrain; that it was a double |
| **Lactate threshold run** | 80–91 % maxHR / 75–88 % HRR; 20–40 min at LT pace to +10 s/mi; or 16/12/8-min intervals; or change-of-pace; or LT hills (+4 bpm) | **Z3 moderate** | zone time (with the known HR-lag caveat); the session's duration | that it was at LT pace; the interval structure; whether it was a tempo, an LT-interval set, a change-of-pace or LT hills |
| **VO₂max intervals** | 3K–5K race pace; 94–98 % maxHR / 92–97 % HRR; reps of 2–6 min; 5,000–8,000 m total | **Z4–Z5 high** and **repeated work** | the lap structure (the watch's own `interval` steps — D4); total hard distance; the recovery-to-hard ratio | the prescribed rep count and distance; whether the pace was 3K–5K pace. **Note the HR-lag warning: time-in-zone under-reports hard work** (Seiler 2010) |
| **Speed repetitions** | 150–300 m at 800 m–mile race pace; recovery jog the same distance or slightly longer, up to 3× the rep time | **repeated work** (and Z4–Z5) | short repeated segments in the lap structure | the prescribed rep count; that it was a speed session rather than a stray burst. **The book gives no HR values for this work** |
| **Strides** | ~100 m, usually after a GA run | **repeated work** (short) | short segments at the end of a run | that they were strides rather than surges; they are too brief to carry HR |
| **Uphill sprints** | ~12 s, all out, walk back | **repeated work** (short) + elevation | segments + elevation gain | the intent |
| **Hills — LT hills** | LT effort on climbs, jog the descents; +4 bpm on LT | **Z3** + elevation | the climb/descend structure; HR on climbs | the intent |
| **Hills — VO₂max uphill / up-and-down** | 2–6 min uphill at VO₂max; or 4–6 × 4 min up-and-down | **Z4–Z5** + elevation | segments, elevation, recovery | the prescribed structure |
| **Tune-up race** | shorter than the goal distance; raced near maximal effort | **Z4–Z5 high** (all of it) | distance, time, HR, pace | that it was a race; that it was a tune-up; the expected residual fatigue |
| **Rest or cross-training** | replaces a run; must be aerobic and easy | **nothing in the record** | — | that the day was a rest day and not a forgotten watch (**"no run recorded", never "rest day"**) |
| **Race (the goal race)** | the distance and time the plan was built for | **Z4–Z5 high** | distance, time, HR, pace | that it was the goal race |

**Reading of the table.** Five of the book's run types (**GA, recovery, LT, VO₂max, speed repetitions**) are **verifiable as bands** — the watch can say how much time sat in each band. Three (**endurance, progression long run, tune-up race**) are identifiable by shape. Two (**strides, uphill sprints**) are too short for heart rate to mean anything and can only be seen as repeated segments. One (**rest or cross-training**) leaves no trace at all — and per §3.4 of `exploration/methods-and-products.md` the app must say "no run recorded", never "rest day".

**Consequence for the ledger:** the plan's session kinds are *intent*; the watch's zones and lap structure are *evidence*. A row that says "the plan had an LT session on 14 October; the last run in Z3 was 15 October" is a statement about **two different things** and must be rendered as such — never as "you missed it".

---

# Part 4 — 80/20 as a peer plan

## 4.1 What the research says

**Seiler 2010** (R1) — the source of the 80/20 rule:

* "about **80 % of training sessions are LIT intensity** and the remaining 20 % are performed as ThT or HIT";
* LIT = stable lactate **< ~2 mM**; ThT = **2–4 mM**; HIT = **≥ 4 mM**;
* "about **two HIT training sessions per week** seems to be sufficient for inducing physiological adaptations and performance gains without inducing excessive stress over the long term";
* "Additional increases in HIT frequency do not induce further improvements and tend to induce symptoms of overreaching/overtraining";
* the randomised comparison (Esteve-Lanao 2007): the group with more zone-1 running improved more (time trial **−157 ± 13 s vs −121.5 ± 7.1 s, P = .03**);
* and the warning that matters most for Stride: "heart rate time-in-zone tends to **underestimate the time spent working at high intensity** (due to heart rate lag time during intervals)"; "typical software-based heart rate analysis methods **overestimate the amount of time spent training at low intensity**"; the paper's alternative, the **session-goal approach**, "yields better matching between heart rate analysis and athlete perception of session effort".

**Stöggl & Sperlich 2015** (R2) — the polarised trial: HVLIT **83/16/1**, THR **46/54/0**, HIT **43/0/57**, polarised **68/6/26** (LIT/ThT/HIT by the session-goal approach, 48 well-trained athletes, 9 weeks). The polarised group increased VO₂peak **+11.7 %**, time to exhaustion **+17.4 %** and peak performance **+5.1 %** the most. **Munoz 2014** (in R2): 30 recreational runners, polarised **77/3/20** vs THR **46/35/19**, 10K improvement **5.0 % vs 3.5 %**. **Neal 2013** (cyclists): polarised **80/0/20** vs THR **57/43/0**.

**What the same review says about the limits:** "the combination of HVLIT with HIT may improve endurance performance with potentially less autonomic and hormonal stress"; but "an 'optimal' TID cannot be identified", and "most retrospective studies … report a pyramidal TID". The 2024 *Sports Medicine* large-sample marathon dataset (151,813 marathons, 119,452 runners) reported in `exploration/methods-and-products.md` §2.5 finds Z1 share rising with performance while Z2/Z3 stay stable.

## 4.2 The published polarised week (R3, 80/20 Endurance)

* **The rule**: "Spend about **80 percent of your weekly running time at low intensity (Zones 1 and 2)** and about **20 percent at moderate to high intensity (Zones 3 to 5)**. The split is **by time, not by number of runs**, and it is measured **across a week** rather than within one workout."
* **Seven zones set from threshold, not from maxHR**: Z1 72–81 % LTHR, Z2 81–90 %, **X 90–95 %**, Z3 95–100 %, **Y 100–102 %**, Z4 102–105 %, Z5 >105 %; pace bands are 60–76 %, 76–87 %, 87–93 %, 93–100 %, 100–102 %, 102–115 %, >115 % of threshold speed.
* **X and Y are the zones to avoid** — "Zone X sits between low and moderate, Zone Y between moderate and high, so a workout can tell you when you have left the zone it asked for." (Note: 80/20's own plans *do* put marathon pace in Zone X.)
* **Threshold inputs**: a recent race time at 5K/10K/HM, a 20-minute time trial, or the **talk test**; "retest every 6 to 8 weeks"; "recovery weeks fall **every third week** in all of our run plans".
* **The week** (Level 1 marathon plan, 18 weeks, 6 runs): Mon rest · Tue run 35 min · Wed run 30 min · Thu run 30 min · Fri run 32 min · Sat run 30 min · **Sun long run 77 min**. Level 0 = 5 runs and 2 rest days; Level 2 = 7 runs; Level 3 = 9 runs including doubles.
* **Cross-training**: "replace any of your Recovery Runs or Foundation Runs with a nonimpact cardio alternative of equal duration and intensity" — and all cross-training must be at Zones 1–2.
* **The hard work**: marathon-pace runs, critical-velocity runs (the fastest pace sustainable for 30 minutes), hill repetitions, fartlek.
* **After the race**: tune-up half marathon or 10K in week 15.

## 4.3 The two bases side by side — the same app, one plan at a time

**Pfitzinger's week (from S1, read from the half-marathon 46–63 schedule):**

> Six run days, **one** quality session (Wednesday LT), two long-ish endurance days (Thursday and Sunday), one general-aerobic + speed day, and two rest/cross-training days that become recovery runs near the race. A **recovery week** every fourth week at ≤ 80 % volume. One progression long run every three weeks. Two hard sessions are allowed only if they are *different types*.

**The polarised week (from R3):**

> Four to seven run days, **two** quality sessions a week (one in Zone 3–4, one in Zone 4–5), everything else in Zones 1–2, a **recovery week every third week**, a long run on Day 7 set by distance, and the *moderate* band (Zone X) is deliberately avoided — except that marathon-pace work is scheduled inside Zone X.

**Three lines of difference, as the brief asks for them:**

1. **Hard days:** Pfitzinger runs **one** hard session a week (LT, VO₂max or a progression long run) with a hard/easy pattern of "two hard, two easy, two hard, one easy"; 80/20 runs **two** hard sessions a week and puts them near the extremes, with the middle deliberately unused.
2. **Easy days:** Pfitzinger's easy days are **banded by heart rate against the athlete's own zones** (GA 70–81 % maxHR, recovery < 76 %) and there is **no pace rule for them at all**; 80/20's easy days are **banded against the athlete's threshold** (Zones 1–2) with an explicitly *avoided* gray zone.
3. **Pacing anchor:** Pfitzinger paces from **the athlete's own race distance** (10K / 15K / HM) and the **race-time tables**; 80/20 paces from **the athlete's threshold** (LTHR / threshold pace / rFTP), with the marathon-pace band deliberately living in the gray zone.

**How the app keeps them apart.** The plan is a single active object with a **method identity**. Concretely, from what Stride already has:

* `PlanSource` (`crates/stride-core/src/plan.rs`) is already `Baseline | Prescribed | Typed` — it needs **one more value** for the method, e.g. `Pfitzinger | EightyTwenty`, so the plan card can name the base it is following.
* `SessionKind` is already `Easy | Long | Quality | Rest | Other(String)` — it needs the book's vocabulary as named kinds (endurance, general aerobic, recovery, progression long run, lactate threshold, VO₂max, speed, tune-up race, race), because **the plan's kind is what the ledger compares the watch's evidence against**.
* The app holds **one plan at a time**: adopting the other base is an explicit switch by the athlete, and the plan's banner names the base. The athlete's history is never re-read as "the other method" without them asking.

## 4.4 What the athlete's own history would have to show — facts, not a verdict

For either base to be plausible, the following are **facts the history can contain or not contain**. No row of this table is a judgement, and the app must not turn one into a score.

| the fact | Pfitzinger's week | the polarised week | where it comes from in Stride |
|---|---|---|---|
| the share of moving time in the athlete's own Z1–Z2 | high — the plan is mostly GA and recovery runs | very high — the plan's target is 80 % of time in Zones 1–2 | time in zone (D3: the model in force on the day of the run) |
| the share in Z3 | one session a week | deliberately small | time in zone |
| the share in Z4–Z5 | one VO₂max session a week, plus the progression long run's last miles | ~20 % of time, concentrated in two sessions | time in zone, **plus** lap structure (the `interval` steps, D4) — because HR lag under-reports this work (Seiler 2010) |
| the number of days with a run | 6 (up to 7 at higher mileage) | 4–7 | days with a recorded run |
| days with **no run recorded** | 1–2 per week, and the plan says "rest or cross-training" | 1–2 per week | days with no run — **stated as "no run recorded", never "rest day"** |
| the longest run of the week | one, on Sunday, sometimes preceded by a mid-week endurance run | one, on Day 7, set by distance | the longest run in the window |
| repeated work | 100 m strides most weeks; VO₂max reps 3–6 weeks out | hill reps and intervals in two sessions a week | the watch's own workout steps |
| how long the athlete has been doing this | the base schedules ask for 25–65 mi/wk of prior training | the 80/20 levels ask for a sustainable weekly hours figure | the athlete's own history |
| a race time at a distance the plan needs | the plan's anchor | the plan's anchor | **must be typed** — Stride has no race store |

---

# Part 5 — What this means for the pace engine

*The scope here is the two shipping templates: the **10K** and the **half marathon** (§0.5). The marathon book's MP-relative rules are in §1.3 for the record and are **not** on the path for either of them.*

## 5.1 The inputs, and where each one lives today

Stride today, from the code: a **dated zone model** with five floors and `max_heart_rate` (`crates/stride-core/src/zones.rs`; `crates/stride-store/src/model.rs`); per-activity `avg_hr` and `max_hr`; daily wellness with `resting_hr`, `hrv_ms`, sleep and stress (`crates/stride-store/src/repos/wellness.rs`); **pace bands** on a 30 s/km grid (`crates/stride-core/src/pace.rs`); and a **plan engine** whose `PlanSession` is `{ date, kind, target_distance_m }` (`crates/stride-core/src/plan.rs`). There is **no race store, no VO₂max, no VDOT, no FTP, and no place to store a workout's interval structure**.

| the rule | the input it needs | in Stride today? | derivable from what Stride has? | must be typed by the athlete |
|---|---|---|---|---|
| **general aerobic** | maxHR (or the zone model's max HR) + the band 70–81 % | maxHR: **yes**, inside the dated zone model | band → bpm: **yes** | no |
| **recovery** | maxHR + resting HR (for the HRR form); **or** a 15K–HM race time (for the pace form) | resting HR: **yes** (wellness, may be missing on some days) | band → bpm: **yes** | the race time, if the pace form is wanted |
| **endurance / long run** | a race time at 10K or 15K–HM; maxHR | maxHR: yes | equivalent race times (App. B): **yes, once one race time exists** | **the race time** |
| **progression long run** | LT pace (→ a race time); the session's "last N at LT" structure | **no** — `PlanSession` has no structure field | LT pace from a race time: yes | **the race time**; the structure comes from the plan |
| **lactate threshold** | a race time (5K/10K/15K/10-mile/HM) to place LT; maxHR | maxHR: yes | LT pace: **yes** (App. B + the book's offsets) | **the race time** |
| **VO₂max** | a recent 5K (or 3K) time; maxHR | maxHR: yes | VO₂max pace: **yes** (App. C) | **the race time** — and in the marathon book, a *recent* 5K, because the 4th edition dropped the goal-time → 5K-pace chart (T3) |
| **speed repetitions** | an 800 m–mile race pace | no | App. C gives 200 m and 300 m splits | **the race time** |
| **strides / uphill sprints** | nothing but the run they attach to | — | — | nothing |
| **tune-up race / goal race** | the race distance and the plan's date | **no race store** | — | **the distance and date** |
| **recovery week** | the week's plan (≤ 80 % of normal volume) | plan exists | from the plan | nothing |
| **taper** | the race date | plan exists | — | **the race date** |
| **marathon variants** (GA 15–25 % slower than MP, long run 10–20 % slower) | a **goal marathon time** | no | MP: yes, from the goal time | **the goal marathon time** — **not needed for the two shipping plans** (§0.5) |

## 5.2 The five placeholders, with their final values

The pace-engine thread built its ladder with provisional values and marked them as placeholders. Each row below is the **final value**, with its citation — or the statement that **the method does not fix it**.

> **One structural fact first, because it changes all five.** The pace engine's rule shape is *"MP + Δ, a signed constant in seconds per kilometre"*, where MP is the **equivalent marathon pace** of the athlete's own race (`exploration/pace-engine.md` §2.3). **Pfitzinger's sub-marathon rules are not fixed constants and are not anchored to marathon pace.** They are **percentages of a named race pace** — the 10K pace, the 15K–HM pace, the LT pace — and the marathon book (which *is* written in MP) is not shipping (§0.5). So each placeholder becomes: **a named anchor + a percentage rule**, and the app's existing MP node stays as the normalisation it already is, not as the anchor of these rules.
>
> The rule form the engine needs for these five rungs is therefore **"anchor pace × factor"** (or **"anchor pace + fixed offset"**, for recovery), where the anchor is one of: *the 10K race pace*, *the 15K–HM race pace*, *LT pace*. Two consequences for the ladder:
>
> * **A fixed s/km constant is wrong for every athlete except by accident** — a percentage rule scales with the athlete; a constant does not.
> * **The ladder needs one more link**: *race time → the athlete's 10K pace, 15K pace and HM pace*. The engine already inverts Daniels' relations to get the equivalent marathon pace from a VDOT; the same inversion gives the equivalent 10K/15K/HM paces. **Pfitzinger's own version of that link is Appendix B** (§5.3, P2), and using it makes the resulting paces the book's own numbers rather than Daniels'.

| placeholder | the engine's provisional value | **final value** | citation | the shape of the answer |
|---|---|---|---|---|
| **`EASY`** (general aerobic) | +120 s/km off marathon pace | **THE METHOD DOES NOT FIX A PACE.** Pfitzinger defines general aerobic runs **only by heart rate: 70–81 % of maximal HR, or 62–75 % of heart-rate reserve**, and by feel — "conversational but should not feel like a jog"; harder than a recovery run, lighter than a long run. | S1 ch. 7 "General Aerobic Runs"; repeated in ch. 8 "Reading the Base-Training Schedules" | **A heart-rate band, not an offset.** If the app must show a pace, it has two honest routes: (i) derive the pace from the athlete's **own** pace-at-heart-rate relation inside the band — Stride stores HR and pace samples per run, so pace-at-HR is derivable; or (ii) **let the athlete type their easy pace**, which is legitimate, since the method leaves it to them. There is **no sourced s/km value**, and the provisional +120 s/km off MP has **no citation**. |
| **`THRESHOLD`** | −45 s/km off marathon pace | **LT pace = "the pace you could race for about an hour."** For faster runners, LT pace ≈ **15K-to-10-mile race pace**; for elite runners, 15K-to-HM pace; for shorter-distance runners, **10–15 s/mi slower than 10K race pace**, or **20–30 s/mi slower than 5K race pace**. Heart rate **80–91 % maxHR / 75–88 % HRR**. | S1 ch. 1 "Lactate threshold"; ch. 7 "Lactate Threshold (Tempo) Runs"; App. C | **An anchor, not an offset.** The anchor is **LT pace**, and the engine derives it from the race time (§5.5). The rule branches on the athlete's racing distance, so the app needs to know it. The provisional −45 s/km off MP has **no citation**. |
| **`LONG_START`** | +90 s/km | **The long run's early pace = 20–33 % slower than the athlete's 10K race pace.** Heart rate **74–84 % maxHR / 65–78 % HRR**. "begin at a conversational pace and gradually increasing your tempo as the run progresses." | S1 ch. 1 "Gaining the Most Benefit From Your Long Runs"; ch. 7 "Endurance Runs"; App. C | **A percentage of the 10K race pace.** The reference point is the **10K race pace**, not marathon pace. |
| **`FINISH`** | 0 (i.e. at marathon pace) | **The long run's latter pace = 17–29 % slower than the athlete's 15K-to-HM race pace** — i.e. **still slower than race pace**, just less slow than the start. In the **progression long run** (ch. 11, 12, 13 — and in both shipping half-marathon tiers), the run ends at **LT pace for the last ~3 miles** ("increase effort steadily throughout run, last 3 miles @ LT pace"). | S1 ch. 1; ch. 7 "Endurance Runs"; App. C | **Two different finishes, and the plan says which.** Standard endurance run: **15K–HM pace × 1.17 to × 1.29**. Progression long run: **LT pace** for the last 2–3 miles. Neither is "marathon pace", so the provisional `FINISH 0` has **no citation** for these two plans. |
| **`RECOVERY`** | +180 s/km | **The recovery run = 2 minutes per mile (75 s/km) slower than the athlete's 15K-to-half-marathon race pace**, and **below 76 % of maximal HR / below 70 % of heart-rate reserve**; "make sure the pace feels appreciably easier on your recovery runs than on your general aerobic runs." | S1 ch. 2 "How Easy?"; ch. 7 "Recovery Runs" | **A fixed offset from race pace: +120 s/mi (+75 s/km) slower than the 15K–HM race pace.** Note the reference point changes from marathon pace to race pace, and note the book gives **two** forms — a pace form and a heart-rate ceiling — where the provisional value had one. |

**Summary for the pace-engine thread:** **four of the five placeholders become an anchor + a percentage (or one fixed offset), and one (EASY) becomes a heart-rate band with the pace left to the athlete's own data or the athlete's hand.** None of the five can be expressed as a fixed offset from marathon pace.

### 5.2.1 The same five, written in the ladder's own shape

So that the pace-engine thread can wire them without re-deriving anything. **Anchor** = a pace the app must be able to produce; **rule** = what the book says; **form** = how the ladder must express it.

| rung | anchor | rule | form | the provisional value, and what it should be |
|---|---|---|---|---|
| **Easy pace** (general aerobic) | — | **none** | **no pace rung.** A heart-rate rung instead: **70–81 % maxHR / 62–75 % HRR** | the `EASY_OFFSET_S_PER_KM = +120` constant **disappears**. The rung is a heart-rate band, and the pace is either derived from the athlete's own pace-at-HR or typed by them. |
| **Threshold pace** | **LT pace** | "the pace you could race for about an hour"; ≈ 15K–10-mile race pace for faster runners; for shorter-distance runners **10K pace + 10–15 s/mi** or **5K pace + 20–30 s/mi** | anchor lookup (App. C), or anchor + fixed offset by level | `THRESHOLD_OFFSET_S_PER_KM = −45` **disappears** as a constant. If the ladder insists on an MP-anchored rung, the derived equivalent is **LT ≈ MP − 6 to −8 s/km** ([derived] from Pfitzinger's own statement that the marathon is **2–3 % slower than LT**, S3: LT = MP ÷ 1.025), so **−45 s/km is several times too slow** for a typical athlete. |
| **Long run — start** | **10K race pace** | **× 1.20 to × 1.33** ("20 to 33 percent slower than your 10K race pace") | anchor × factor | `LONG_START_OFFSET_S_PER_KM = +90` **disappears** as a constant; it becomes a **range** scaled by the athlete's 10K pace. |
| **Long run — finish** | **15K–HM race pace**, or **LT pace** for a progression long run | standard long run: **× 1.17 to × 1.29** ("17 to 29 percent slower"); progression long run: **LT pace for the last ~3 miles** | anchor × factor, or a second anchor for the last miles | `LONG_FINISH_OFFSET_S_PER_KM = 0` **disappears**. The long run is **one rung with two ends** (the engine already models it that way), and neither end is at the equivalent marathon pace. |
| **Recovery pace** | **15K–HM race pace** | **+ 120 s/mi (+ 75 s/km)**; and **below 76 % maxHR / below 70 % HRR** | anchor + fixed offset (a *pace* offset, not a percentage) | `RECOVERY_OFFSET_S_PER_KM = +180` **disappears**; the rule is **+75 s/km off the athlete's 15K–HM race pace**. The rung **is** present — the book fixes it — so it is drawn, not absent. |

**One extra thing the ladder needs, which is not a placeholder:** the app must be able to say, for a given run, **which zone band the session was in** — the book's own bands (GA 70–81 % maxHR, recovery < 76 %, LT 80–91 %, VO₂max 94–98 %, long run 74–84 %) are the method's rule, and they are what the ledger compares the watch's evidence against (§Part 3).

## 5.3 The remaining rules the ladder needs (the full rule → pace list)

| # | the rule | input | the pace it produces | source |
|---|---|---|---|---|
| **P1** | **race time** | the anchor of every pace | the athlete's race result at 5K, 10K, 15K, 10-mile or HM | S1 App. B |
| **P2** | **equivalent race times** | one race time → all the others | the athlete's 10K, 15K, 10-mile, HM times | S1 App. B |
| **P3** | **long run, early** | the 10K race pace | **× 1.20 to × 1.33** | S1 ch. 1, ch. 7 |
| **P4** | **long run, latter** | the 15K–HM race pace | **× 1.17 to × 1.29** | S1 ch. 1, ch. 7 |
| **P5** | **LT pace** | a race time + the athlete's racing distance | the hour pace (§5.5); heart rate 80–91 % maxHR / 75–88 % HRR | S1 ch. 1, ch. 7 |
| **P6** | **VO₂max pace** | the 5K/3K race pace | the 3K–5K race pace; 94–98 % maxHR / 92–97 % HRR | S1 ch. 1, ch. 7 |
| **P7** | **speed repetition splits** | an 800 m–mile race pace | the 200 m and 300 m splits at that pace | S1 ch. 7 |
| **P8** | **recovery pace** | the 15K–HM race pace | +120 s/mi (+75 s/km); below 76 % maxHR | S1 ch. 2, ch. 7 |
| **P9** | **general aerobic** | maxHR; resting HR | **70–81 % maxHR / 62–75 % HRR — no pace rule** | S1 ch. 7, ch. 8 |
| **P10** | **race pace (the goal race)** | the goal time | the pace the athlete must hold | S1 ch. 10, ch. 12 |
| **P11** | **progression long run's finish** | LT pace | LT pace for the last ~3 miles | S1 ch. 7; the ch. 11–13 schedules |
| **P12** | **LT-relative (the author's current coaching framing)** | LT pace | the whole plan's paces set relative to LT pace | S4 — and the author publishes his own **LT pace table** (5K/10K/15K time → LT pace → LT training pace ±5 s/mi) |

*(The marathon-book variants — MP, GA 15–25 % slower than MP, long run 10–20 % slower — stay in §1.3 for the record; they are not on the path for the two shipping plans.)*

## 5.4 The shortest honest list of what the athlete must supply

*(The chain that produces these paces is settled in **§5.5**; the rules behind it are in **§5.2**.)*

To pace a **Pfitzinger week** on the 10K or half-marathon plan, the athlete must type:

1. **The goal race** — distance (10K or half marathon) and date. The date anchors the 12-week plan, the recovery week and the taper.
2. **One race time**, at a distance the plan uses — a **10K** or **half-marathon** time (or a 5K/15K time, which Appendix B converts). The book's LT rule branches on the athlete's racing distance, so the app needs to know it.
3. **Maximal heart rate** — or Stride uses the value Garmin already holds in the zone model. **Not** derived from age.
4. **Resting heart rate** — Stride already stores it daily from wellness; it may be absent on some days.
5. **Which plan** (distance + tier) and **its start date**.

Everything else — the equivalent race times, the long-run band, the LT pace, the VO₂max pace, the speed splits, the recovery pace, the heart-rate bands, and the GA band — is **computed from those five inputs**.

**That is the whole list.** No VDOT, no VO₂max estimate, no lactate, no age formula, and no guessing a race time from training. **And note: no marathon time is needed** for either shipping plan.

## 5.5 The chain — **resolved**

The decision, from the pace-engine thread (`exploration/pace-engine.md`) and confirmed against this file:

```
race time (typed, dated) → VDOT (the engine's own) → race-pace equivalents
(VDOT/Riegel) → LT pace (this file's rule) → the rungs (this file's percentages)
```

**Our chain replaces the book's lookup tables.** Appendix B and Appendix C are the book's own tabulation of the same relation (race time → equivalents → workout paces); the app computes the 10K, 15K and half-marathon paces from the athlete's race time instead. **The rules are still Pfitzinger's, quoted and cited from the book** — the percentages, the offsets, the heart-rate bands, the session shapes. That is what this file is for.

**And this settles the question this file had left open.** The evidence for the resolution:

* **The book is sub-marathon.** Its rules are anchored to the athlete's own race distances (10K, 15K, HM, LT) — *not* to marathon pace. The marathon book's MP-relative bands are the weakest-sourced numbers in this file (§1.3, third-party), and the marathon plan is not shipping (§0.5).
* **Therefore MP is at most one more rung, never the anchor.** The engine may keep MP as the normalisation it already is (the equivalent marathon pace of the athlete's race); every Pfitzinger rule anchors to a race pace the athlete has.

### 5.5.1 The LT pace rule — confirmed, with one correction

The parent reads the LT anchor as *"the pace for a ~15K (10-mile) race from the same VDOT, or 10K pace + 10–15 s/mi for a shorter-distance athlete."* **Confirmed**, with the sharpening below.

**The book's own words (S1 ch. 1, "Lactate threshold"):**

> "For experienced runners, LT pace is the pace you could race for about an hour."
> "For faster runners, your LT pace is approximately your race pace for 15K or 10 miles, and for elite runners LT pace typically is between 15K and half marathon pace."
> "If your experience is mostly with shorter races, LT pace is generally 10 to 15 seconds per mile slower than 10K race pace or 20 to 30 seconds per mile slower than 5K race pace."

**Correction / sharpening — three points.**

1. **The book's anchor is a band, not a point.** It is the **15K-to-10-mile race pace** — two paces, not one (and for elite runners, 15K-to-HM). A "15K pace" alone is the **fast edge** of the band's middle. The 10-mile pace is slightly slower than the 15K pace for the same athlete, so the band is narrow but real; the engine should carry it as a band, not collapse it silently.
2. **The rule's true statement is "the hour pace."** The 15K/10-mile route is the book's *approximation* of it for faster runners. The engine's job is to produce a pace the athlete could hold for about an hour — and since the chain already computes a 15K pace and a half-marathon pace, the band sits between them (and slightly faster than the 15K pace for most athletes, whose 15K is a little over an hour).
3. **The offset rule is the one for shorter-distance athletes, and it is a floor, not a substitute.** 10K pace + 10–15 s/mi (or 5K pace + 20–30 s/mi) is what the book says for runners whose racing is short-distance. Note the direction: for a shorter-distance athlete the LT pace is **slower** than their race pace, because their race distance is shorter than the hour.

**Does the book say anything else about finding LT pace?** Yes — **five methods**, and they are the complete answer (S1 ch. 1 and ch. 7):

| # | the method | what it gives | source |
|---|---|---|---|
| 1 | **A lactate test** — six 4-minute treadmill stages with a 1-minute rest and a blood sample at each; the pace and HR where lactate turns up | the pace and the heart rate | ch. 1 |
| 2 | **A race of about 45–60 minutes** — "a very good estimate of your LT pace" | the pace (the 15K/10-mile route) | ch. 1, ch. 7 |
| 3 | **A 45-minute time trial** in good conditions on a flat course — "your pace and heart rate during the last 30 minutes" | the pace and the heart rate | ch. 1, ch. 7 |
| 4 | **An estimate from training software** — "more data will generally lead to more accurate estimates"; use a chest strap | the pace | ch. 1, ch. 7 |
| 5 | **An estimate from heart rate** — LT pace "generally occurs at about **82 to 90 percent of maximal heart rate**, which coincides with about **76 to 87 percent of heart rate reserve**" | the heart rate, and the pace implied by it | ch. 1 |

*(Method 5's heart-rate band is quoted from ch. 1; ch. 7 states the wider **80–91 % maxHR / 75–88 % HRR**. The difference between the two statements is the book's own — see §1.5.)*

**The consequence for the engine:** LT pace is computed from the race time using the athlete's **racing distance** to choose the branch (15K–10-mile pace, or 10K + 10–15 s/mi, or 5K + 20–30 s/mi), and the app should name the branch it used — the same way it names the race it came from.

---

# Part 6 — What is not verified, and what a reader should therefore not trust

1. **The marathon book's own text.** *Advanced Marathoning* was not available. Every marathon number above comes from S2 (an author-written 2001 excerpt of the **1st** edition), S3 (a 2025 interview about the **4th** edition), S4 (the author's site) or a labelled third party. The **MP-relative pace bands** (GA 15–25 % slower, long run 10–20 % slower) are **[3rd]** and are unverified against the 4th edition.
2. **The 4th edition's schedule numbers** — time-based LT, mileage ranges, the taper's drop to 25–32 pre-race miles, the removal of doubles from 18/70, the dropped goal-time → 5K-pace chart — are all **[3rd]** (T1, T3, T5). They are plausible and internally consistent, but they are a reviewer's reading of the book, not the book.
3. **The heart-rate table (Table 7.1) is an image in the EPUB.** The figures quoted in §1.1 are the ones stated in the surrounding prose, which is how they are cited. The table itself was not transcribed cell by cell, so **a run type's band that appears only in the table (rather than in the prose) is not yet pinned.**
4. **The plan schedules are images, and all four shipping schedules have now been read cell by cell** (§2.3): the **8K–10K tier 1 (30–42 mi/wk)**, the **8K–10K tier 2 (45–57 mi/wk)**, the **half-marathon tier 1 (31–47 mi/wk)** and the **half-marathon tier 2 (46–63 mi/wk)**. In each, every daily cell reconciles with the printed weekly mileage. **Not read cell by cell:** the 10K tier 1's post-race week beyond Thursday; Table 7.2; and every schedule that is **not shipping** — the 5K, the 15K/10-mile, the multi-distance, the higher-mileage tiers and the three base-training schedules (extracted, not built — §0.5).
5. **Table 7.2** (the book's guidance on revising goals after time lost to illness or injury) was not read.
6. **The 2018 Stöggl/Seiler marathon study** remains **unverified** (carried forward from `exploration/methods-and-products.md` §4). The polarised evidence here is Stöggl & Sperlich 2014/2015 and Munoz 2014, all cited above.
7. **LetsRun's Pfitzinger thread could not be fetched (HTTP 403)** and is not used. **Reddit required the Parallel extractor**; no quote is reconstructed from memory.
8. **80/20's own zone numbers are product documentation**, not research: the 7-zone scale, the talk-test protocol and the "recovery weeks every third week" are Fitzgerald's published method (R3). Seiler's own rule is stated in blood lactate, and the paper's heart-rate method contradicts it — which is why 80/20 was refused as a lens and is here only a **plan**.
9. **A reader should not trust any statement in this file that says "the plan should…"** — every rule here is the *method's* rule, quoted or paraphrased, not advice from Stride. `docs/contract/voice.md` still applies to anything the app renders.
10. **The scope of the first drop is two templates** — the **10K** and the **half marathon** (§0.5). The 5K, 15K/10-mile and multi-distance plan structures were extracted and are kept here as knowledge, but they are **not** built, and nothing in this file should be read as a plan to build them.
11. **The `EASY` placeholder has no pace.** Pfitzinger's general aerobic run is a heart-rate rule. If the app shows a pace on an easy day, that pace is **the athlete's own data or the athlete's own choice**, and the app must say which. It must not present it as the method's number.
12. **Appendix C's cells are not transcribed.** The table renders transposed at low resolution and rows could not be aligned to race times with confidence, so **no cell value from Appendix C is published** (§1.2). The app's VDOT chain does not need them; only a future feature that quotes the book's own tabulated paces would.

---

## 7. What the Beta may and may not claim as a plan skeleton

**May claim (verified, cell by cell in §2.3):**

* A **12-week** plan with **weeks-to-go counted 11 → 1, then race week**, and a **recovery week at 8 weeks to go** in every tier.
* **Monday is rest or cross-training** in every week of every tier.
* **One quality session a week**, and its **type changes with the phase**: lactate threshold in the early weeks, VO₂max later, race pace in the last third. In the HM tiers the quality day moves (Friday → Tuesday → Friday); in the 10K tiers it stays on Friday or Tuesday.
* **The long run is on Sunday** in every tier, with the length and the progression-long-run placement as printed in §2.3, and the **last 2–3 miles at LT pace** in the half-marathon tiers' three progression long runs (weeks 9, 7 and 5).
* **Two tune-up races** in the 10K plans (5K at 4 and 2 weeks to go) and in the half-marathon plans (8K–10K at 4 and 2 weeks to go).
* **The race week's shape**, including the race-pace rehearsal (**1,200 m at 8K–10K race pace** in the 10K plans; **2 miles at half-marathon race pace** in the HM plans) and the goal-race distance and its weekly volume.
* **A recovery week and a transition week after the race** (the 10K tier 1 has a single post-race recovery week).

**May not claim:**

* Any **weekly skeleton for a tier that is not shipping** (5K, 15K/10-mile, multi-distance, the higher-mileage tiers) — those are extracted, not built (§0.5), and their cells were not read.
* **A pace for the general aerobic run** — the method fixes a heart-rate band only (§5.2, `EASY`).
* **Any marathon-plan claim** from this file's numbers — the marathon book's own text was not available and its MP-relative bands are third-party (§1.3, §6.1).
* **A cell value from Appendix C** (§6.12).
* **That the athlete's plan was followed, or not.** The plan's cells are intent; the watch's zones and lap structure are evidence, and the two are different things (Part 3).

---

## 8. The ten numbers that matter — one more time, with the source and the confidence

1. **The athlete's goal race time** — the anchor of every pace — **[book]** (App. B is the book's own tabulation of this relation; the app computes the equivalents from the race time, §5.5).
2. **Equivalent race performances, 5K→8K→10K→15K→10-mile→HM** — **[book]** (App. B; **structure read, cells not transcribed** — §1.2, §6.12).
3. **General aerobic: 70–81 % maxHR / 62–75 % HRR, and no pace rule** — **[book]** (ch. 7, ch. 8).
4. **Recovery: below 76 % maxHR / below 70 % HRR, or 2 min/mi slower than 15K–HM race pace** — **[book]** (ch. 7, ch. 2).
5. **Endurance run: 74–84 % maxHR / 65–78 % HRR; 20–33 % slower than 10K pace; 17–29 % slower than 15K–HM pace; start slow, finish faster** — **[book]** (ch. 1, ch. 7).
6. **LT pace = the pace you could race for about an hour; 80–91 % maxHR / 75–88 % HRR** — **[book]** (ch. 1, ch. 7).
7. **LT session = 20–40 min continuous, or 16/12/8-min intervals with 4-min jogs** — **[book]** (ch. 1, ch. 7).
8. **VO₂max: 3K–5K race pace; 94–98 % maxHR / 92–97 % HRR; 2–6-min reps, 5,000–8,000 m total** — **[book]** (ch. 1, ch. 7).
9. **Recovery week ≤ 80 % of normal volume; long run ≤ 80 % of normal distance; no VO₂max/tempo** — **[book]** (ch. 2).
10. **Taper = 2 weeks, 20–60 % mileage cut, intensity kept in smaller doses** — **[book]** (ch. 6).
