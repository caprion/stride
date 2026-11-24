# Methods and products — what runners say, and how coaching methods map to data

*Research note for the parent thread and the two design threads. Written 24 November 2026.*
*Research only — no code, no changes to `crates/`, `frontend/`, `android/`, `PLAN.md`, the contract or the schema.*

---

## 0. How this was gathered, and what to distrust

**Fetching.** `reddit.com` returned **HTTP 403** to the Tavily extractor and to a plain `curl` with a browser UA during this research; `old.reddit.com` was also 403. The **Parallel** extractor retrieved Reddit pages successfully (OP + comment threads). Two consequences:

* Where a quote appears below, it was read in a fetched page. Where a **thread title** appears below but no body, it came from Reddit's own "People also ask" / "More posts you may like" listings on a page I *did* fetch, and the thread itself was **not opened**. Those are marked *(title only)*.
* Several archived threads (older than ~2 years) returned the **OP only**, with no comment section extracted. Those are marked *(OP only)*. No comment was quoted from an unopened page.
* One guessed r/Garmin URL resolved to an unrelated subreddit; nothing from it is used here.

**Labels used throughout.** `[research]` = published, peer-reviewed or lab evidence, cited to the paper. `[convention]` = a coaching method's published rule, taken from the method's own book/site or a faithful summary of it. `[folklore]` = forum belief, repeated widely, not traceable to a study. `[product]` = what a product's own documentation says it computes.

**Nothing below is invented.** Where I could not verify a claim, it says so. The three things I could not verify are listed in §4.

---

# Section 1 — What runners say (user language)

## 1.1 What they use today, and for which question

| Product | What its own docs say it computes | Who the forums say uses it, and for what |
|---|---|---|
| **TrainingPeaks** | Daily **Training Stress Score (TSS)**; **Fitness (CTL)** = 42-day exponentially weighted average of TSS; **Fatigue (ATL)** = 7-day average; **Form (TSB)** = yesterday's Fitness − yesterday's Fatigue; all drawn on the **Performance Management Chart** `[product]` ([trainingpeaks.com/learn/articles/what-is-the-performance-management-chart](https://www.trainingpeaks.com/learn/articles/what-is-the-performance-management-chart), updated 5 Oct 2026) | The plan plus the load chart. "I’m quite rational analytic by nature and so use tools like TrainingPeaks to monitor fatigue (perceived or otherwise) and adjust training accordingly" — u/Unable-Recording-726, r/AdvancedRunning, *Thoughts on building training intensity*, 3y ago ([16kwqu0](https://www.reddit.com/r/AdvancedRunning/comments/16kwqu0/thoughts_on_building_training_intensity/)) *(OP only)* |
| **RUNALYZE** | **Effective VO2max** from the HR/pace relation; **TRIMP** training load; **ATL (fatigue)** 7-day and **CTL (fitness)** 42-day exponentially weighted averages; **TSB (stress balance)**; **Easy TRIMP**; **Monotony** over 7 days of daily TRIMP; **Training strain** = monotony × sum TRIMP, with a coloured green/orange/red feedback `[product]` ([blog.runalyze.com/tutorial/runalyze-understanding-the-calculations](https://blog.runalyze.com/tutorial/runalyze-understanding-the-calculations)) | The one whose number matches the body. "Runalyze Workload Ratio is the most true to how I feel" and "I find the Garmin training status laughable" — u/BottleCoffee, r/AdvancedRunning, *What training load metric do you find the most 'real'?*, 2y ago ([1asawk7](https://www.reddit.com/r/AdvancedRunning/comments/1asawk7/what_training_load_metric_do_you_find_the_most/)) |
| **Garmin Connect** | **Training Readiness** = a score plus a message, "continuously calculated and updated throughout the day" from sleep score (last night), recovery time, HRV status, acute load, sleep history (last 3 nights), stress history (last 3 days); bands: Purple 95–100 "Prime — Best possible" … **Red 1–24 "Poor — Let your body recover"** `[product]` ([Forerunner 965 owner's manual, Training Readiness](https://www8.garmin.com/manuals/webhelp/GUID-0221611A-992D-495E-8DED-1DD448F7A066/EN-US/GUID-C21BE0C8-A08E-4DA1-B6C6-2E0E2DDDB372.html), April 2026) | The thing that argues with them. Thread titles alone: *"Training Readiness accuracy? My watch is gaslighting me"* (1y ago), *"Poor training readiness despite great sleep."* (5mo ago), *"Why is my Training Readiness at 1 even though my sleep was good?"* (9mo ago), *"Training Readiness: does it change your workouts, or do you mostly go by feel?"* (3mo ago) — r/Garmin *(title only, from a fetched r/Garmin search listing)* |
| **intervals.icu** | — | Chosen because it is the "advanced metrics over time" alternative to a spreadsheet. u/ARehmat, r/running, *Online Logging Software: Runalyze vs interval.icu*, 4y ago ([11vq5ub](https://www.reddit.com/r/running/comments/11vq5ub/online_logging_software_runalyze_vs_intervalicu/)) *(OP only)*: "I do have a spreadsheet where I log each runs (distance, pace, elevation climbed etc) but it is not feasable to log all the data contained in the .fit file the watch produces. As such I am looking into logging software such as Runalyze (free tier) and intervals.icu that I can use to track more advanced metrics over time." |
| **Strava** | — | The social layer and the comparison layer, and the thing that makes runners run too hard. "People on Strava will see me chugging along at something less than other-worldly paces and judge me" — u/robotcrow1878, r/AdvancedRunning, *Why Do You Run Easy Miles Too Hard?*, 3y ago ([179biyh](https://www.reddit.com/r/AdvancedRunning/comments/179biyh/why_do_you_run_easy_miles_too_hard/)) |

## 1.2 What they like — the specific feature and the specific phrase

* **"the load chart" / the PMC** — the Fitness–Fatigue–Form picture is the thing coaches and self-coaches look at. TrainingPeaks' own framing: "consistent training is marked by a steady rise in Fatigue and Fitness, whereas a sharp drop might indicate time off due to sickness or injury" `[product]`.
* **"intensity distribution" / "the percentage of high quality miles"** — "If you NEED a number to go by, look at the percentage of high quality miles you put in. The old rule of thumb is you want about 20% of your total volume to be quality." — same 1asawk7 thread.
* **"monotony" and "strain"** — RUNALYZE users quote them by name; RUNALYZE's own docs say monotony under 1.5 is "recommendable", over 2.0 "critical" `[product]`.
* **"pace at HR" / pace–HR coupling** — the chart runners actually use to decide whether a pace is "really" easy on the day. "Both Garmin and Stryd think my easy pace could be much faster, but I think they're being biased by my (relatively) strong speed work" — u/RektorRicks, r/AdvancedRunning, *Anyone else have a pretty slow easy pace?*, 5y ago ([okz568](https://www.reddit.com/r/AdvancedRunning/comments/okz568/anyone_else_have_a_pretty_slow_easy_pace/)) *(OP only)*.
* **VDOT paces** — "the V dot calculator says according to my HM race time, I could do my 'easy' runs between 8:40-9:32'/mi" — u/an_angry_Moose, r/AdvancedRunning, *Some clarifications on "easy" runs and mileage*, 3y ago ([185f45t](https://www.reddit.com/r/AdvancedRunning/comments/185f45t/some_clarifications_on_easy_runs_and_mileage/)) *(OP only)*.
* **"the plan"** — the plan is the thing they say they actually follow: "Mileage, average pace and gut feel. That, and following what I know to be a properly structured plan if I'm targeting an event." — u/yellow_barchetta, 1asawk7.

## 1.3 What they complain about

**Dashboards that show numbers without meaning.**

* "If you rely on various Apps (including the innate Garmin one) instead of actual understanding of what each number or graph represent, then yea.. you're better of not even using the device." — u/npavcec, r/AdvancedRunning, *How do you think of zone 2 training?*, 2y ago ([166icri](https://www.reddit.com/r/AdvancedRunning/comments/166icri/how_do_you_think_of_zone_2_training/))
* "Every single person who has attempted to define what exactly Zone 2 means has come up with something different." — u/WillGeoghegan, 185f45t.

**Black-box scores that contradict the body.**

* "Garmin will tell me I'm overtraining when I hit my biggest volume weeks right before a taper, which is partly true but useless as a training indicator." — u/Wientje, 1asawk7.
* "I feel like garmin general data, race predictors, recovery, body stress, etc are nearly useless while in the midst of a training block (esp a high volume marathon block)." — u/beagish, 1asawk7.
* "If I listened to suggested rest periods I'd never complete any marathon training block." — u/beagish, 1asawk7.
* "Absolutely none of them lol. Learn to listen to your body. If I want to get my Garmin to tell me I'm super fit, I'll wake up early and run nothing but easy runs 30s/mile faster than my usual easy runs. Doing that with my lower than average early morning HR makes my Garmin think I'm an endurance god. If I start doing intervals with standing rest, or faster track reps, it just starts spouting nonsense." — deleted user, 1asawk7.
* "COROS EvoLabs reckons that I am either 100% recovered or need 10 days before a hard effort." — u/ARehmat, 1asawk7.

**The metrics that cannot see the session.**

* "It can't see intervals or any 'in session' differences." and "Hard intervals, hill sprints or strength sessions don't show up in the data." — u/Wientje, 1asawk7.
* "Strava, TP and Garmin all have a different bias to different runs, and if you want to cheat the system, you can. Also it assumes correct metrics - pace on the treadmill may be off, elevation can be affected by the weather changes, heart rate may be off if you are using a wrist HRM." — deleted user, 1asawk7. **This sentence is, in effect, a specification for Stride's missing-value policy.**

**Over-claiming, and the sense that a number is a verdict.**

* "I quit analyzing anything other than the pure data. I don't take suggestions or anything from any apps. If I feel good then I'm doing good. If I feel bad then I know I need rest. I try to do the KISS method. keep it simple, stupid. Running is intuitive." — u/BiceRidingWorldChamp, 1asawk7.
* "Mileage, average pace and gut feel." — u/yellow_barchetta, 1asawk7.
* "I am not a fan of heart rate as a metric to use *during* my own training, but it is something I look at *after*." — u/nnfbruv, 185f45t.

**Zones treated as universal.** "It's a core value but MASSIVELY over emphasized. 'Zones' are also incredibly personalized and there is no general rule, yet it's treated as such. Too many early stage runners get caught up living and dying by their heart rate number." — u/deezenemious, 166icri.

**Device disagreement used as a source of doubt.** "Imo, easy days should obviously be easy but I don't need some inaccurate device telling me how easy that should be. I'll be running 3:00/mi slower than 5k pace and both my garmin and whoop say I'm in HR zone 4." — u/little_runner_boy, 166icri.

**Plan rigidity.** "It's better to let your actual fitness determine the mpw rather than trying to hit some arbitrary target that you may not respond well to. It's part of the reason why stock training plans are flawed, especially when people follow them to a tee." — u/arksi, 185f45t.

**Not verified in this pass:** paywall/price complaints, privacy complaints, and Strava-specific complaints were asked for in the brief but I did **not** find a clean verbatim thread on them. They exist in quantity on Reddit; I simply did not surface a quotable one. Treat as a gap, not as an absence of complaint.

## 1.4 The questions they actually ask of an analytics tool

This is the highest-value section. These are verbatim thread titles (all fetched-page titles, r/AdvancedRunning unless noted) with the URL and the age Reddit displayed on 24 Nov 2026.

1. *"What training load metric do you find the most 'real'?"* — 2y ago — [1asawk7](https://www.reddit.com/r/AdvancedRunning/comments/1asawk7/what_training_load_metric_do_you_find_the_most/) · *"I got my new Garmin watch that has the training status calculation and started wondering what training load metric works the best in terms of reflecting the stress on the body… In other words, that gives you the best recommendation how to train to keep getting better but not overtrain."*
2. *"Can you run your easy runs too slowly?"* — 6y ago — r/running — [nke23n](https://www.reddit.com/r/running/comments/nke23n/can_you_run_your_easy_runs_too_slowly/) · *"my real question was are the physiological benefits of the easy pace diminished if I run at the low end of the prescribed range vs the high end?"*
3. *"Is there such a thing as 'too easy'?"* — 4y ago — [10kb8so](https://www.reddit.com/r/AdvancedRunning/comments/10kb8so/is_there_such_a_thing_as_too_easy/) · *"how easy should one take their easy runs if trying to go quick on race day?"*
4. *"Why Do You Run Easy Miles Too Hard?"* — 3y ago — [179biyh](https://www.reddit.com/r/AdvancedRunning/comments/179biyh/why_do_you_run_easy_miles_too_hard/)
5. *"How do you think of zone 2 training?"* — 2y ago — [166icri](https://www.reddit.com/r/AdvancedRunning/comments/166icri/how_do_you_think_of_zone_2_training/)
6. *"At what point does easy running become a long run?"* — 6y ago — [m5mnxm](https://www.reddit.com/r/AdvancedRunning/comments/m5mnxm/at_what_point_does_easy_running_become_a_long_run/)
7. *"Anyone else have a pretty slow easy pace?"* — 5y ago — [okz568](https://www.reddit.com/r/AdvancedRunning/comments/okz568/anyone_else_have_a_pretty_slow_easy_pace/)
8. *"minimal effective dose of easy running"* — 5mo ago — [1uspch7](https://www.reddit.com/r/AdvancedRunning/comments/1uspch7/minimal_effective_dose_of_easy_running/) · *"is there a minimal per-session effective dose for easy runs? … can any benefits be expected from a 35-minute easy running session?"*
9. *"How slow is too slow for easy runs?"* — 4mo ago — r/NorwegianSinglesRun — [1tuzb2d](https://www.reddit.com/r/NorwegianSinglesRun/comments/1tuzb2d/how_slow_is_too_slow_for_easy_runs/) · *"my easy run pace has been consistent around 11:00/mile pace. Is this too slow to make progress? I haven't really progressed much"* *(OP only)*
10. *"Am I experiencing overreaching /overtraining symptoms?"* — [1fs253f](https://www.reddit.com/r/AdvancedRunning/comments/1fs253f/am_i_experiencing_overreachingovertraining/) *(title only)* · and *"How to tell the difference between functional overreaching and overtraining?"* — [ubv0cl](https://www.reddit.com/r/AdvancedRunning/comments/ubv0cl/how_to_tell_the_difference_between_functional/) *(title only)*
11. *"Overreaching, cumulative fatigue, and/or external stress -- 'diagnosing' and addressing?"* — [mbvgzq](https://www.reddit.com/r/AdvancedRunning/comments/mbvgzq/overreaching_cumulative_fatigue_andor_external/) *(title only)*
12. *"Is it generally better to overshoot or undershoot Threshold pace in training?"* — [1dvkbga](https://www.reddit.com/r/AdvancedRunning/comments/1dvkbga/is_it_generally_better_to_overshoot_or_undershoot/) *(title only)*
13. *"80% of my training is in HR zones 4 and 5. Dangerous?"* — r/running, 5y ago *(title only, from a fetched r/running "More posts" listing)*
14. *"PSA: How your heart rate zone 2 might be lying to you."* — r/running, 1y ago, 723 upvotes *(title only, same listing)*
15. *"Dealing with Garmin Training Load vs. Real Feelings – Any Tips?"* — r/AdvancedRunning, 2y ago *(title only, from a fetched 1asawk7 listing)*
16. *"Heart Rate Zone Training. Am I doing it wrong?"* — [f71gzx](https://www.reddit.com/r/AdvancedRunning/comments/f71gzx/heart_rate_zone_training_am_i_doing_it_wrong/) *(title only)*
17. *"Trade-off between Slow/Easy Runs and Good Running Technique"* — [ggb88e](https://www.reddit.com/r/AdvancedRunning/comments/ggb88e/tradeoff_between_sloweasy_runs_and_good_running/) *(title only)*
18. *"Easy runs being hard"* — r/Marathon_Training, 10mo ago — [1qk3xvr](https://www.reddit.com/r/Marathon_Training/comments/1qk3xvr/easy_runs_being_hard/) *(title only)* · and *"How easy are easy runs supposed to be?"* — [1vcncti](https://www.reddit.com/r/Marathon_Training/comments/1vcncti/how_easy_are_easy_runs_supposed_to_be/) *(title only)*
19. *"Online Logging Software: Runalyze vs interval.icu"* — r/running, 4y ago — [11vq5ub](https://www.reddit.com/r/running/comments/11vq5ub/online_logging_software_runalyze_vs_intervalicu/) — *"What has your experience been using the aforementioned platforms and are there any others that you would recommend? How do they compare in 2023?"*
20. *"Thoughts on building training intensity"* — [16kwqu0](https://www.reddit.com/r/AdvancedRunning/comments/16kwqu0/thoughts_on_building_training_intensity/) — *"As my training becomes less generic (eg 1x 5 mile easy 1x 3 mile tempo per week) and more specific (eg long runs and intervals) I'm struggling to decide whether to prioritise high aerobic/anaerobic (eg track sessions, intervals, hill reps) or building long run distance."*

**What the list says.** The overwhelming majority of the questions are about **one session** ("is this run easy enough?"), not about a model ("am I progressing?"). The questions about the *history* are rare and mostly about **missing kinds of work** and **being too hard overall**. The "am I training too hard?" question is nearly always asked as *"is my easy run too easy?"* — the runner asks about the **easy end**, not the hard end. That is a strong argument for the stimulus ledger being read as *"what the weeks contain and what they lack"*, not as a fatigue or readiness verdict.

## 1.5 How they check a session

The order is consistent across the threads I read:

1. **Feel / talk test first.** "Easy means easy. You shouldn't have to look at your Garmin to see if you're running easy" — u/ilanarama, 185f45t. "Easy for me just means a comfortable, relaxed feeling run that won't require a recovery run (or day off) afterwards" — u/arksi, 185f45t. "Easy pace should feel boring." — u/jysh1, 185f45t.
2. **Heart rate against the athlete's own band, second half vs first half.** "my max heart rate is 200 and I typically do 140 - 145 for my easy runs. It ends up higher on long runs, but cardiac drift makes it an artificial higher number." — u/BuzzedtheTower, 185f45t.
3. **Drift / decoupling.** "One way could be to took at HR drift to determine intensity." — u/Virtual_Opinion_8630, 1uspch7. "Weather can change my heart rate to pace ratio drastically, so paying attention to hr instead of pace is a good idea." — u/crunchyRoadkill, nke23n. "[HR] swings more [than HRV]… Do a hard work out and it will go down. Do more without recovery and it will keep going down." — u/skiitifyoucan, 1asawk7.
4. **Was the session's *interior* as it was meant to be?** The forum's own framing is "did the last rep fall apart?" expressed as: *"Is it generally better to overshoot or undershoot Threshold pace in training?"* (1dvkbga, title only); *"Can you run tempo runs too fast?"* ([nc4koq](https://www.reddit.com/r/AdvancedRunning/comments/nc4koq/can_you_run_tempo_runs_too_fast/), title only); and from the description of a failed session: *"It felt great going up, but the jogs down did not."* — u/an_angry_Moose, 185f45t (OP only).
5. **Yesterday's feeling.** "I have been running 15 years and I FELT plenty of runs as super easy, but I was always tired the next day." — u/npavcec, 179biyh.
6. **Not the number, the whole picture:** *"How did the speed sessions go? How did your long run go? Sleep? Recovery? Nutrition, hydration? Weight (perhaps!)? Stress, mood? Any niggles, injuries or soreness?"* — deleted user, 1asawk7. **This is the list of things a watch cannot produce.**

## 1.6 Does the runner want comparison to a method?

**Mostly no — but the vocabulary of the methods is already in their heads, and they use it as a *label*, not as a *rule*.**

* They reason in method-relative terms constantly ("MP+2 mins for easy, MP+1min for aerobic, MP-30sec for threshold, MP-1min for short intervals" — u/wofulunicycle, 10kb8so; "a good rule of thumb is MP+2 mins for easy…" — same user; "The usual rule is one to one and a half minutes above your marathon pace" — u/Zealousideal_Put_495, nke23n).
* But they pull those rules apart when they conflict with the body: *"I love to run slow! I'm constantly getting bad unsolicited advice from people who think my easy runs should be at a 7:30 pace based on my race times. I'm very happy with my 9:30-10 min/mile easy runs most days and my high quality speed workout once a week."* — u/sbwithreason, 179biyh.
* And they ask about the method itself only when a method has failed them: *"No love for Phil Maffetone? (Simple calculation to see if MAF is right for you)."* — [o9q4xd](https://www.reddit.com/r/AdvancedRunning/comments/o9q4xd/no_love_for_phil_maffetone_simple_calculation_to/), 5y ago *(OP only)*. The OP's edit is the clearest statement of the folk verdict: *"I misunderstood why the Maffetone method was unpopular among athletes. I thought it was because athletes were worried that it required them to train too slowly. Others on here quickly and gracefully assured me that other training methods are preferable to MAF in terms of simplicity, breadth, and evidentiary support."*
* One post uses the method as an explicit frame and asks for the ledger: *"I am in the beginning of a marathon training plan (Pfitz 18/85, just starting week 3) and I have a few questions about how people approach their easy paces."* — u/nlaframboise, 10kb8so *(OP only)*.

**Design consequence:** the method is a *vocabulary* the runner already uses. A method lens in Stride does not need to teach the method; it needs to answer "what does my history look like when read with that method's own rule?" — as a fact, with the rule quoted.

---

# Section 2 — The methods, mapped

## 2.1 The table

| method | weekly structure | intensity rule | long-run rule | recovery rule | what it asks the runner to *feel* | computable from HR zones + pace + time + distance alone? | needs what it needs |
|---|---|---|---|---|---|---|---|
| **Pfitzinger** (P+D, *Advanced Marathoning*) | 12 or 18 weeks; peak mileage 55/70/85/105 mpw; 5 mesocycles: endurance → LT+endurance → race prep → taper & race → recovery; 6–7 run days incl. rest/cross days; GA, recovery, strides, VO2max, LT, medium-long, long, MP run | GA = 15–25% slower than MP, or 70–81% HRmax / 62–75% HRR; LT = 15k–HM pace, or 82–91% HRmax / 77–88% HRR; VO2max ≈ 5k pace, 93–95% HRmax; MP run = expected MP, 79–88% HRmax `[convention]` | 10–20% slower than MP, or 65–78% HRmax; "start slow, finish faster"; MP runs are a separate session type `[convention]` | named "recovery" days: deliberately the slowest runs of the week, after hard efforts `[convention]` | "easy enough to talk"; the GA run is "comfortable, not effortless" and *faster* than recovery `[convention]`; the runner feels a distinct "recovery" pace that is slower than GA | **Partly.** The *bands* are computable from the athlete's own HR/pace model, but "GA vs recovery" is a **method-defined distinction** with no physiological boundary; MP needs a target race | a target marathon time; the distinction GA/recovery is a convention, not a measurement |
| **Jack Daniels** (VDOT) | "run at the intensity of the workout"; 5–6 quality types: Easy, Marathon Pace, Threshold, Interval, Rep, Fast Reps; Lydian cadence tradition `[convention]` | Easy 59–74% VO2max / 65–79% HRmax; MP 75–84% / 80–90%; Threshold 83–88% / 88–92%; Interval 97–100% / 98–100% `[product]` (vdoto2.com) | Easy is used for long runs; MP is "an alternative to Easy pace running for beginners on long run days" `[product]` | recovery folded into Easy ("recovery runs, recovery running within a workout and generally long runs") `[product]` | "a comfortable, conversational pace, which certainly may vary daily, depending on how you are feeling, and the weather and terrain" `[product]` | **Yes, largely** — VDOT derives every band from one race time; the pace↔HR mapping is exactly the "pace at heart rate" chart | a race time (VDOT input); MP needs a target |
| **Maffetone (MAF)** | aerobic running only, daily; all runs at or below the cap; speed work shelved initially `[convention]` | MAF HR = 180 − age, then −10 / −5 / 0 / +5 by health & training status; first mile 10 bpm lower `[product]` (philmaffetone.com) | no long run defined — the cap governs everything | the cap *is* the recovery rule | "low-intensity effort"; conversational, often uncomfortably slow | **Only if age is known** — and that is the point: it is a *rule* about age, not about the athlete's physiology | age; a chest strap (Maffetone's own advice); MAF test cadence (monthly early, then ~8-weekly) |
| **Seiler / 80/20 polarisation** | ~80% of training *time* low intensity, ~20% threshold-or-hard; ~2 HIT sessions/week is enough `[research]` | 3 zones by lactate: LIT <2 mM, ThT 2–4 mM, HIT ≥4 mM `[research]`; Fitzgerald's 5-zone practical scale with **Zone X** and **Zone Y** marking the "gray zone" to avoid `[convention]`; 80/20 Endurance's 7 zones set from LTHR / threshold speed / rFTP `[product]` | easy; 80/20 Endurance's marathon long run carries 8–16 km at marathon effort, counted toward the 20% `[convention]` | "recovery weeks fall every third week in all of our run plans" `[product]` | low intensity must be *conversational in full sentences*; the gray zone is the failure mode `[convention]` | **Partly.** Zone boundaries need a threshold test; the *share* is computable from time-in-zone — but see the HR-lag caveat in §2.6 | a threshold (lactate, LTHR test, race time, or talk test) |
| **Norwegian — classic (Bakken)** | 2 double-threshold days (morning + evening) + 1 "X element" + easy days; altitude blocks; 160–180 km/wk typical `[convention]` | threshold work at **2–3 mmol/L** ("golden zone 2.3–3.0"), ≈80–87% HRmax; morning = longer reps lower, evening = shorter reps slightly higher, 15–20 s/km apart `[convention]` | long easy run 5–8 h for the elite; easy runs <1.0 mmol/L, HR <70% max; from ~8 weeks out the long run is extended/sharpened, pure long run never beyond 3–3.5 h `[convention]` | "at least one full easy day between sessions is not optional. It is where adaptation happens" `[convention]` | "you should be able to speak in short sentences but not hold a comfortable conversation" `[convention]` | **Partly.** The session *shape* (two sessions, rep structure, gap) is computable; the *band* (2–3 mmol/L) needs lactate, and Bakken explicitly says to cross-check HR with talk test and perceived effort | lactate meter (ideal), or HR + talk test + RPE; a treadmill for controlled comparison; altitude |
| **Norwegian — Singles (2024–26)** | 2–3 single sub-threshold sessions across the week + all other runs easy; "no weekly or every-other-week X-session"; only a test run or race every 4–6 weeks goes above sub-threshold `[convention]` | sub-threshold = **LT1** (≈2 mM; ≈marathon pace for trained); RPE 5–6/10; "you can still speak in full, but short, sentences"; amateurs ≈20–25% of weekly mileage `[convention]` | long, easy run in the sample week; easy days "truly easy, including the long run" `[convention]` | rest day + easy days between sub-threshold sessions | "firm and controlled… If the pace feels like it's tipping toward strain, it *is* too hard" `[convention]` | **Partly.** Session count and the easy/sub-threshold split are computable; the LT1 line needs lactate, and the published guidance says so | the LT1 line (lactate or a calibrated proxy); RPE or talk test to police it |

## 2.2 Pfitzinger

**What it is.** Pete Pfitzinger and Scott Douglas, *Advanced Marathoning*, 4th ed. Plans of 12 or 18 weeks with peak weekly mileage of 55, 70, 85 or 105 miles; five mesocycles; a named vocabulary of run types — rest/cross-training, recovery, general aerobic (GA), strides, VO2max, lactate threshold, medium long, long, marathon pace `[convention]` (summarised from [runningwithrock.com/pfitz-marathon-training-explained](https://runningwithrock.com/pfitz-marathon-training-explained)).

**Intensity rules as published in the plan** (the percentages are given there for each session type) `[convention]`: GA 15–25% slower than MP (70–81% HRmax / 62–75% HRR); long run 10–20% slower than MP (65–78% HRmax / 74–84% HRR); LT at 15k–HM pace (82–91% HRmax / 77–88% HRR); VO2max ~5k pace (93–95% HRmax / 91–93% HRR); MP run at expected MP (79–88% HRmax / 73–84% HRR).

**The "slow down to feel easy" rule.** Widely attributed to Pfitzinger, but **I could not verify a primary-source sentence from the book in this pass** — treat as *attributed convention, not quoted*. What *is* attested, from the runner side, is the behaviour the rule is supposed to produce: "When I started focusing on HR, I noticed how fast I was going, and gave myself permission to go slower. I now have a little beep if I'm pushing too hard on my easy runs… I was going 11 / 11:30 when I first started doing that, and down to 9-9:30 for same HR within about 2 months." — u/Comfortable-Plan2658, nke23n.

**"Eight weeks out / three weeks down."** Two different things are merged in the brief, and the sources disagree about which method owns which. Pfitz's taper is the **last three weeks** before the race (the fourth mesocycle, "taper and race") `[convention]`. The **eight-weeks-out** sharpening of the long run is Bakken's: "about eight weeks before the marathon, you extend and sharpen the long run" — Marius Bakken in [Runner's World, *Norwegian Training Method*](https://www.runnersworld.com/training/a72845651/norwegian-training-method) `[convention]`.

**The complaint: MAFS is too slow for faster runners.** The forum's sharpest version is that at elite volume the easy pace is *not* very slow: "Most elite runners are doing easy about 2 mins slower than MP… most amateurs tend to go too fast on easy runs, rather than too slow. A good rule of thumb is MP+2 mins for easy, MP+1min for aerobic, MP-30sec for threshold, MP-1min for short intervals. If you're needing to run more than 2:30/mile slower than MP, best just take a real rest day." — u/wofulunicycle, 10kb8so `[folklore, but a widely held and internally consistent coaching rule]`. The counter-complaint is about the *gait* cost of very slow running: "at these really slow paces, my gait suffers, and I could feel the shin pain more at the slow paces than I could during a short test pace pickup of 7'/mi" — u/an_angry_Moose, 185f45t `[folklore]`. And the comic version of the same: "Scott Fauble ran the 2018 NYC Marathon at a pace between 4:50 and 5:10/mi. In his book, he wrote about a recovery run: 'If we start running sub 7 minute pace again, I am gonna just pretend like I am having stomach issues and run alone.'" — u/chaosdev, 10kb8so `[folklore, quoted from a book]`.

**Watch-verifiable:** the *bands* (GA vs LT vs VO2max vs MP) are verifiable against the athlete's own zone model — **except** MP, which needs a target time; and **except** the GA/recovery distinction, which is a convention with no measurement behind it.

## 2.3 Jack Daniels

**The anchor.** VDOT is "a measure of your current running ability… Enter a recent or estimated race result to get your VDOT score and current training paces" `[product]` ([vdoto2.com/calculator](https://www.vdoto2.com/calculator/), fetched 24 Nov 2026). Every training band is a percentage of VDOT-equivalent intensity, and the current implementation states them as VO2max and HRmax ranges: **Easy** 59–74% VO2max / 65–79% HRmax; **Marathon Pace** 75–84% / 80–90%; **Threshold** 83–88% / 88–92%; **Interval** 97–100% / 98–100%; **Rep** and **Fast Reps** defined by feel relative to 800/1500/mile pace `[product]`.

**"Run at the intensity of the workout."** The intent is the training pace set by the workout type, not the runner's day. The site's own sentence: "Easy running is a comfortable, conversational pace, which certainly may vary daily, depending on how you are feeling, and the weather and terrain with which you are faced. You may be up to 20 seconds per mile slower or faster than the specified pace on a given day." `[product]`

**Pace at heart rate — the foundation.** Daniels' method derives pace *from* a race equivalent and then maps heart rate onto it; the "pace at HR" chart is the same mapping, plotted, in products like RUNALYZE. Two properties matter for Stride: it is **anchor-dependent** (one race time sets everything) and it is **the chart runners already trust** to decide "is this really easy today?". Note that VDOT O2's own published bands leave a wide tolerance (±20 s/mile) and explicitly refuse to judge the day.

**Recovery.** In this implementation, recovery is not a separate band: "Easy pace running refers to warm-ups, cool-downs, **recovery runs, recovery running within a workout** and generally long runs" `[product]`. Daniels' own book separates recovery; the current commercial implementation folds it in. **Two products, two taxonomies** — a good illustration for Stride that "recovery run" is a label, not a measurement.

**Watch-verifiable:** almost everything, provided a race time (or an equivalent VDOT input) exists. **Not** verifiable: that the athlete ran *at* the intensity (they may have run harder or easier than the band).

## 2.4 Maffetone

**The rule, from the source.** "Subtract your age from 180, then modify from one of the categories below": −10 for major illness, rehabilitation, regular medication, or Stage 3 (chronic) overtraining; −5 for injury, regression, frequent infection, allergies/asthma, being overfat, Stage 1–2 overtraining, or being inconsistent/new; no modification if training consistently ≥4×/week for up to two years; +5 if training more than two years without the listed problems and progressing. Exemptions: individualisation above 65; MAF HR 165 for 16 and under. And Maffetone's own caveat, which is often left out: "The use of the number 180 is not significant other than as a means of finding the end heart rate. Plus, 180 minus age itself is not a meaningful number; for example, it is not associated with VO2max, lactate threshold, or other traditional measurements." `[product]` ([philmaffetone.com/180-formula](https://philmaffetone.com/180-formula), updated July 2020).

**"No workouts with speed?" — stated precisely.** The MAF method as described puts **all aerobic training under the cap**; *speed* work is not part of the method as such. The rule people quote is from the coaching gloss, not from Maffetone: "When you first start MAF Method training, you're supposed to **stop all speed workouts** because every single training run is supposed to be performed at or below your target maximum heart rate. Then, as long as you are progressing and improving on your MAF tests, **you can reintroduce speed workouts using the 80/20 rule after a couple of months**" — [marathonhandbook.com/maf-method-maffetone-method](https://marathonhandbook.com/maf-method-maffetone-method) `[convention]`. Maffetone's own method pages do not set a speed prescription.

**Criticism, stated by a credible secondary source.** The same Marathon Handbook page, in a section labelled "The Honest Truth About The MAF Method": the 180-minus-age cap is "a population-level approximation that doesn't handle the substantial individual variation in maximum heart rate or lactate threshold"; Tanaka et al.'s large-population study found the 220-minus-age formula has a standard deviation of ~11 bpm; and because LT1 typically sits at 78–82% HRmax and LT2 at 88–92% HRmax, "for some runners the 180-minus-age cap lands well below LT1 and represents truly easy effort, while for others it sits at or above LT2 and isn't aerobic-zone training at all" `[research cited, via secondary]`.

**The forum verdict.** From the Maffetone thread's OP edit: "other training methods are preferable to MAF in terms of simplicity, breadth, and evidentiary support… any age-based formula for calculating exercise intensity is statistically unreliable. That the 180-age formula works for me is a coincidence." `[folklore, but consistent with the published criticism]`

**Watch-verifiable:** the cap is computable **if the athlete's age is known** — which is the interesting case for the ledger: this is a method-defined kind that is *arithmetically* checkable but *physiologically* arbitrary.

## 2.5 Seiler's 80/20 polarisation

**Published research.** Seiler, S. *What is best practice for training intensity and duration distribution in endurance athletes?* Int J Sports Physiol Perform. 2010;5(3):276–291 (full text: [umh1617.umh.es](https://umh1617.umh.es/files/2016/05/02-IJSPP_2010_Seiler_0009.pdf)) `[research]`:
* "about 80% of training sessions are LIT intensity and the remaining 20% are performed as ThT or HIT";
* LIT = stable lactate < ~2 mM; ThT = 2–4 mM; HIT = ≥4 mM;
* "about two HIT training sessions per week seems to be sufficient for inducing physiological adaptations and performance gains without inducing excessive stress over the long term";
* "Additional increases in HIT frequency do not induce further improvements and tend to induce symptoms of overreaching/overtraining";
* the randomised comparison (Esteve-Lanao et al. 2007): the group with more zone-1 running improved more (time-trial −157 ± 13 s vs −121.5 ± 7.1 s, P = .03).

**The practice version (Fitzgerald).** Three zones (low below VT1; moderate between VT1 and VT2; high above VT2), turned into five training zones, "Between those zones he marks small in-between ranges that he advises avoiding, so your easy runs stay truly easy" — the **gray zone** `[convention]`, summarised at [runculture.com/learn/running-methods/fitzgerald](https://runculture.com/learn/running-methods/fitzgerald). The operational rules that matter for a ledger: the split is **by time, not distance**; warm-ups, cool-downs and jog recoveries **count as low intensity**; strides are too brief to change the distribution.

**The 7-zone scale (80/20 Endurance, the current product).** Zones set from LTHR / threshold speed / rFTP: Z1 72–81% LTHR; Z2 81–90%; **X 90–95%**; Z3 95–100%; **Y 100–102%**; Z4 102–105%; Z5 >105%. The FAQ states the purpose of X and Y plainly: "They mark the moderate intensities most runners drift into by accident. Zone X sits between low and moderate, Zone Y between moderate and high, so a workout can tell you when you have left the zone it asked for." `[product]` ([8020endurance.com/intensity-guidelines-for-80-20-running](https://www.8020endurance.com/intensity-guidelines-for-80-20-running))

**Where the credible sources disagree.** Fitzgerald's 80/20 deliberately includes moderate work (tempo, marathon pace) inside the 20%; strict polarisation "keeps time between the two thresholds to a minimum". Same source: "For most recreational runners, the differences are academic." **We must not pick a side.**

**The "2018 Stöggl/Seiler marathon study" — could not verify.** See §4. What *is* verifiable and relevant: Stöggl & Sperlich's randomised TID comparison (HVLIT 83/16/1 vs THR 46/54/0 vs HIT 43/0/57 vs **polarised 68/6/26**; the polarised group gained VO2peak +11.7%, time to exhaustion +17.4%) `[research]` ([PMC4621419](https://pmc.ncbi.nlm.nih.gov/articles/PMC4621419), Front Physiol 2015), and the modern large-sample marathon dataset: *The training intensity distribution of marathon runners across performance levels*, Sports Medicine, 2024, DOI 10.1007/s40279-024-02137-7 — 151,813 marathons by 119,452 runners, three-zone model with critical speed as the Z2/Z3 boundary — which reports that "the amount of training time completed in Z2 and Z3 remained relatively stable across performance levels, however, proportion of Z1 was higher in progressively faster groups" `[research]` ([repository.londonmet.ac.uk/9883](https://repository.londonmet.ac.uk/9883)).

## 2.6 The Norwegian methods

### Classic — Bakken's model

**Primary source, in his own words.** [mariusbakken.com/the-norwegian-model.html](https://www.mariusbakken.com/the-norwegian-model.html) (Kristiansand, 23 January 2022) and [mariusbakken.com/double-threshold-training.html](https://www.mariusbakken.com/double-threshold-training.html) (22 March 2026) `[convention — a coach's own account, not a trial]`.

* **The band.** "lowering the lactate level from the standard level of 4.0 mmol/l down initially to below 3.0 – usually staying from 2.3 up to 3.0"; elsewhere "my Golden Zone of 2.3 to 3.0 mmol/L lactate"; for well-trained runners "roughly 80–87% of maximum heart rate".
* **The week.** "Day 1: two threshold sessions/double threshold running, one easy session. Day 2: long easy (5-8 hours workout). Day 3: two threshold sessions, one easy. Day 4: very easy training of about 2 hours easy training of choice." Plus one weekly "X element" of higher intensity (hill work or 5k/10k-specific), plus altitude.
* **The 2006 week (the template that became the Ingebrigtsens')**: Monday easy 15 km + easy 12 km; **Tuesday threshold + threshold**; Wednesday easy 16 + 10; **Thursday threshold + threshold**; Friday easy 15 + 9 + strength; Saturday higher-intensity hill workout + 10 easy; Sunday easy 21 km — 178 km total.
* **Easy days.** "In terms of lactate, we are below 1.0 mml/l, heart rate below 70 % of max for the easy runs."
* **Morning vs evening.** Longer reps (6–10 min) in the morning at the lower end; shorter reps (400–1000 m) in the evening, "usually right at or right above the anaerobic threshold", "typically spans 15 to 20 seconds per kilometer" apart.
* **Control.** "Heart rate is the most practical substitute, but it should not be used alone. The talk test is a useful cross-check… Perceived effort tells you something too, though it is the least reliable of the three on its own… If your heart rate is in range but the effort feels harder than it should, or if you cannot pass the talk test at a pace that looks correct on paper, treat that as a signal rather than noise. Move down in pace." — Bakken, 22 March 2026 `[convention]`.
* **Volume.** "120 [km] which is a minimum for long/middle-distances for many, 160-180 km which is for most will work the best and 170-220 which is a level that some will work for, but not all."

**Published research it now leans on.** The 2024 NTNU study (Kjøsen Talsnes et al., *Frontiers in Physiology*, 2024, doi 10.3389/fphys.2024.1428536): 14 well-trained athletes, one session of 6×10 min vs two sessions of 3×10 min with 6.5 h between. Single session: second half 2.8% higher HR and 0.41 mmol/L higher lactate; Borg RPE 13.4 → 14.8. Split: evening session 2.5% *lower* HR and 0.58 mmol/L lower lactate; RPE stable at 13–14; next-day fatigue 8.0 (split) vs 7.0 (single); soreness 7.0 vs 6.0 `[research]` — as summarised by Bakken, 22 March 2026.

**The critique, from the method's own architect.** Runner's World, *Can the Norwegian Training Method Help You Run More and Faster—While Recovering Better?* (Caitlin Carlson) `[convention/published interview]`:
* "Running too fast/hard is actually the most common mistake Bakken sees runners making. 'Threshold training at an intensity that is not threshold is worse than [overdoing] conventional training [like easy runs] because you will just go constantly a little bit too hard and push a little bit too much and you're doing it with more load in total,' he says."
* "You need to go below 70 percent [max HR] on the easy days… It's a nice day outside and they run a little bit hard at the end of the runs and then you get into problems."
* "The more intuitive thing is to just run faster and obviously that's the challenge and that's the beauty of it."
* Coach Greg Laraia: "Most of the time we want to go faster than that."
* On recovery: "Muscle soreness and elevated muscle tone peak the day after a hard session, not during it. That is why the rhythm is hard day, fully protected easy day below 70 percent of max heart rate, then the next hard day."

### Current — Norwegian Singles

* **Origin.** Developed by James Copeland (*Norwegian Singles Method*, [norwegiansingles.run](https://norwegiansingles.run/)); endorsed by Bakken — Runner's World: "The 'singles' approach, developed by James Copeland… is a slightly tamer version of the traditional Norwegian Method that is also endorsed by Bakken." `[convention]`
* **Structure.** "There are three single sub-threshold sessions across the week, plus easy runs. 'There's not always a second daily session, but there's also no weekly or every-other-week X-session. The only thing that lifts you above sub-threshold is a test run or race roughly every four to six weeks.'" — Bakken, via Runner's World.
* **The band.** Sub-threshold = **LT1** (~2 mM; ≈ marathon pace for trained runners), per [marathonhandbook.com/norwegian-singles-training](https://marathonhandbook.com/norwegian-singles-training): "The defining feature of Norwegian singles is that 'threshold' sessions sit at LT1 … not LT2"; RPE 5–6/10; "A sub-lactate pace is one in which you can still speak in full, but short, sentences."
* **The week** (from the same page): Monday sub-threshold 3×10 min, Tuesday easy, Wednesday sub-threshold 5×6 min, Thursday easy, Friday optional third session or easy, Saturday long easy, Sunday rest. "Run your easy days *truly* easy, including the long run."
* **Amateur dose.** "For amateur runners, this typically translates to **20–25% of weekly mileage** at sub-threshold intensity."
* **The proxy problem is stated plainly.** "He points out that heart rate isn't a reliable tool mid-workout, and running power is still too inconsistent to anchor a session. **Pace**, on the other hand, gives you a steady, repeatable proxy for sub-threshold."
* **What it removes.** "Compared to other coaching methods, such as Jack Daniels, Norwegian singles removes much of the traditional intensity variety. Instead of a VO2 max day, a threshold day, and a long-run-hard-finish day, you get two or three moderate but highly controlled aerobic sessions and then lots of easy running."
* **The literature, honestly read.** "The honest reading: Norwegian singles is a high-end optimisation framework for runners already at high volume, not a beginner-to-intermediate intervention… Most amateur runners would benefit more from base volume, polarised intensity distribution, and consistency than from copying the methodological details of Ingebrigtsen-style training."
* **Inside-the-family disagreement.** The "Singles" page itself describes "the classic Norwegian-singles week typically includes two double-threshold days" — i.e. singles is presented both as the *simplification* of double-threshold and as *containing* it. Bakken's own double-threshold page says the "singles" variant is "a tamer version… endorsed by Bakken" (Runner's World). The two structures are not the same thing; a ledger that treats "Norwegian" as one kind would be wrong.
* **Elite results.** The Ingebrigtsen brothers (Jakob, Henrik) and Bashir Abdi's European marathon record are the method's public evidence — *reported results*, not a controlled trial. Andreas Almgren's 2026 New York City Marathon win on a double-threshold-based plan is reported by Marathon Handbook `[folklore-adjacent: press reporting, not a study]`.

### What I could not verify about the Norwegian family

* **Ørnskæg's "Fthreshold 2:1 and 3:2 ratios."** Not verifiable from any source I reached. Marked **unverified**.
* **Ørnskæg's classic method as described in the brief** (4-week cycles of 4×4 / 2×4 / 3×4 minutes, threshold 3×/week). The 4×4 that circulates under the name "Norwegian 4×4" is **NTNU research** (Haugen et al.; 4 minutes at 85–95% of max HR, 3 min recovery), not Ørnskæg's method — this is a genuine mis-attribution in the brief and I could not verify the Ørnskæg-specific cycle structure. Marked **unverified / likely mis-attributed**.

## 2.7 Where the credible sources disagree — the table that matters for Stride

| The same 45-minute run at 78% HRmax is… | Pfitzinger | Daniels (VDOT) | Maffetone | Seiler / 80/20 | Norwegian (Bakken) | Norwegian Singles |
|---|---|---|---|---|---|---|
| "GA run" | **yes** (70–81% HRmax) | **yes** (Easy, 65–79%) | **yes** (below the cap, if the cap is ≥78) | **yes** (Z1–Z2 low intensity) | **yes** (below 70% max is the *rule* — 78% would be too fast) | **yes** (below LT1) |
| "gray zone" | n/a | n/a — Easy band is wide | n/a — everything is aerobic | **this is the zone to avoid** (Zone X) | n/a | n/a |
| "sub-threshold quality work" | no | no | no | no | **yes** — this *is* the quality session | **yes** — this *is* the quality session |
| "recovery" | a **separate, slower** run type | folded into Easy | n/a | a separate, slower run type | the day after a threshold day | rest day + easy days |
| "threshold" | LT at 82–91% HRmax | Threshold at 88–92% HRmax | never | a small, deliberate slice of the 20% | 2–3 mmol/L ≈ 80–87% HRmax | **LT1**, ~78–82% HRmax |

**The single biggest disagreement: what "threshold" means.** Daniels' threshold sits at 88–92% of max HR (LT2). The Norwegian singles method sits at LT1, ~78–82% of max HR. The *same run* is "threshold" to one and "sub-threshold" to the other, and the two methods differ by roughly a full zone. A ledger that says "threshold volume" without saying whose threshold is meaningless.

**Second: what "easy" means.** Bakken's easy days are below 70% HRmax; Pfitz's GA is 70–81%; Daniels' Easy is 65–79% but his *recovery* is folded in; 80/20's low intensity is below VT1 with an explicit gray zone to avoid. A run at 72% HRmax is GA under Pfitz, Easy under Daniels, low intensity under 80/20, and **too hard under Bakken**.

**Third: the long run.** Pfitz's long run is 10–20% slower than MP, pushed ("start slow, finish faster"); 80/20's marathon long run ends with 8–16 km at marathon effort; Bakken's long run is *fully* easy (<70% HRmax) and, from ~8 weeks out, becomes a progressive or combination run; a Hanson-style marathon-pace long run is a different session again (the brief names Hanson; I did not research it — **out of scope, unverified here**).

## 2.8 What a watch can and cannot verify — per method

* **Verifiable from HR + pace + time + distance:** that a run fell in a named band; the *counts*, *shares*, *dates* of runs in each band; the shape of a session (repeated hard segments from lap structure); that a week contained N sub-threshold sessions and M easy days; that a session's HR drifted; that pace and HR decoupled.
* **Not verifiable from a watch, ever:** blood lactate (so LT1 vs LT2 vs "golden zone" is an inference, never a measurement); RPE; the talk test; how the run felt; muscle tone or soreness; terrain and conditions beyond what the FIT records (and the FIT record's temperature field is device-dependent); motivation; sleep debt beyond what is stored; whether the athlete intended the session as prescribed; whether a day with no recorded run was a rest day or a forgotten watch.

**And the research warns us about HR time-in-zone specifically.** Seiler 2010 `[research]`: "heart rate time-in-zone tends to **underestimate the time spent working at high intensity** (due to heart rate lag time during intervals)"; it "does not seem to correspond well with perceived effort for a given workout"; and "typical software-based heart rate analysis methods **overestimate the amount of time spent training at low intensity and underestimate the time spent at very high workloads**, compared with athlete perception of effort for a training bout." The paper's alternative — the "session goal approach" (allocate a session by the intensity of its primary part) — "yields better matching between heart rate analysis and athlete perception of session effort".

**This is the single most important line in this document for the stimulus ledger:** a naive time-in-zone ledger will systematically **over-report easy volume and under-report interval work** on exactly the runs where the watch's HR lags. Stride must not read "no run above Z4 for five weeks" as "no interval work happened" without saying that HR is the input.

A second research warning, from the same paper: Foster et al. found "the tendency for midlevel athletes to **train harder than planned on easy days and at lower intensity than planned on hard days**, relative to coach prescriptions" `[research, cited in Seiler 2010]`. So a ledger's "absence" of a stimulus kind may be a **plan-versus-reality gap**, not a stimulus gap — and a fact-only product must be able to say both without picking.

---

# Section 3 — What this means for our stimulus ledger

## 3.1 The taxonomy: method-agnostic kinds vs method-defined kinds

**Method-agnostic** — derivable from the athlete's own dated zone model, pace bands, moving time, distance and lap structure, with no external rule:

| kind | rule sketch | notes |
|---|---|---|
| **Easy volume** | time in the athlete's own Z1–Z2 (or below the athlete's threshold pace band) | the single most robust kind; *both* Pfitz GA and Bakken easy days land here for most athletes, at different edges |
| **Steady / sub-threshold volume** | time in the athlete's upper-Z2 band (below their threshold pace) | method-agnostic **as a band**; "sub-threshold" as a *label* belongs to the Norwegian method |
| **Threshold-ish volume** | time in the athlete's threshold pace band / upper Z3 | band is derivable; whether it is "threshold" depends on whose threshold |
| **High-intensity work** | time in Z4+, *plus* runs whose lap structure contains ≥N sustained segments faster than the athlete's threshold pace | **must not rely on HR alone** — see §2.8; lap structure is the mitigation |
| **Long run** | the athlete's longest run(s) of the window by duration, or runs ≥90 min | long ≠ hard; these are orthogonal, and every method agrees they are different axes |
| **Days with no recorded run** | calendar days in the window with no run | must be labelled "no run recorded", never "rest day" — the watch may simply have been off |

**Method-defined** — exist only because a method says so; the ledger can only produce them under an explicit, athlete-chosen lens:

| kind | why it is method-defined | what it needs |
|---|---|---|
| **MAFS / MAF run** | defined by a *rule about age* (180 − age ± adjustments), not by the athlete's physiology | age; the athlete's HR in the run |
| **MP run** | defined against a *goal race* time that does not exist yet | a target time |
| **GA run vs recovery run** | two named, deliberately different paces with no physiological boundary between them | a convention the athlete adopts |
| **Fthreshold / double-threshold day** | defined by a *session ratio* (hard:easy days) and a lactate band | lactate or a calibrated proxy; RPE/talk test to police it |
| **80/20 compliance / gray-zone time** | defined by a *distribution target* and an exclusion band | a threshold definition, and a decision about which runs count as "the 20%" |
| **Lydian cadence session** | defined by a cadence target (180–190 spm) | cadence sensor — **not verified in this pass** |

**The taxonomy rule I would propose:** every ledger row carries **one of two labels** — *your zones* (method-agnostic) or *a named method's rule* (lens). Never both in the same row without saying so.

## 3.2 Where the methods disagree about the same run

Four collisions, all documented above:

1. **Easy** — Bakken (<70% HRmax) vs Pfitz GA (70–81%) vs Daniels Easy (65–79%) vs 80/20 low intensity (below VT1, gray zone excluded). A 72% HRmax run is "GA" under one method, "too hard" under another.
2. **Threshold** — Daniels 88–92% HRmax (LT2) vs Norwegian Singles LT1 78–82% HRmax vs Pfitz LT 82–91%.
3. **Recovery** — a named, slower run type (Pfitz, 80/20) vs folded into Easy (Daniels' current implementation) vs the day *after* a threshold day (Norwegian) vs a cap (Maffetone).
4. **Long run** — Pfitz's pushed long run vs 80/20's marathon-effort finish vs Bakken's fully easy long run vs Hanson's goal-pace long run.

**Is a method-agnostic ledger possible? Yes — for the *kinds*, not for the *judgements*.** What is not possible is a single "easy" bucket that all six methods would agree on. Therefore:

* The **ledger rows** must be method-agnostic (athlete-relative bands, counts, shares, last dates).
* A **method lens** can only be a *second, explicitly-labelled projection* over the same history, computed with the method's own published rule, quoted from the method.

## 3.3 A fact-only method lens, and what it may say

The design constraint: `docs/contract/voice.md` bans verdicts, advice, scores and commands, and the ban list includes *should, must, good, bad, poor, fail, behind, missed, recommended, better to, need to, ready to, red, readiness score, not recovered, you are ready, you are not ready*, plus *rest day / rest up / rest today / go easy / keep it easy* as commands. *"easy"* survives as the plan's own vocabulary.

Two shapes, both fact-only:

**Shape A — the rule and the history, side by side, as two separate facts.** The app states the method's published rule (attributed, quoted) and then states what the athlete's own history contains, in the athlete's own units, with no link between them. The reader draws the conclusion; the app does not.

**Shape B — the athlete chooses a lens; the app reports counts, dates and shares under that lens only.** No rule text, no evaluation. Nothing in the output says whether the result is high or low.

**Example sentences in Stride's voice** (each must survive the ban list; each carries its values inline, per the `statement` rules; each is a `statement`, not a `pattern`, so no n is required — if any of these were re-typed as a `pattern` it would need its sample size):

1. *"Pfitzinger's general aerobic runs: 15% to 25% slower than marathon pace. Your last 12 runs: 6 h 41 m of moving time below your zone 2 ceiling, 2 h 05 m above it."*
2. *"Bakken's easy days: heart rate below 70% of maximum. In the last 8 weeks you recorded 9 runs below that ceiling, 4 runs above it, and 2 runs with no heart-rate data."*
3. *"Under the 80/20 rule, about 80% of training time is spent at low intensity. In the last 8 weeks: 71% of moving time was in your zones 1 and 2, 19% in zone 3, 10% in zones 4 and 5."*
4. *"Daniels' threshold pace sits at 88% to 92% of maximum heart rate. Your last 20 runs: 3 h 12 m inside that band. Your longest run above it was 42 minutes, on 12 October."*
5. *"The last run with time above your zone 4 ceiling was 12 October — 5 weeks and 4 days before today."* (the ledger's *absence* sentence; it states the **last date**, never what was missed)

Note what is **absent** from all five: no verdict, no comparison to "should be", no score, no advice, no link to a vital, no ranking of options. Note also what shape-1 and shape-2 cost: the *rule* is a quote with attribution (a `statement` about a published rule, not about the athlete), and the *history* is a `statement` about the athlete — two facts, no arrow between them.

**Things a lens may never contain** (by ban list and by the `statement` rule "a rendered statement may not carry a hole" — no template holes, so every value must be inline):
* "Your training is 9% short of the 80/20 target." → a verdict.
* "You have not had a threshold run for 5 weeks." → "missed" territory; state the last date.
* "Your easy runs are too fast." → a verdict, and requires the method's threshold to be the arbiter.
* Any sentence containing *should, must, good, bad, poor, red, recommended, need to, better to, ready to, missed*.

**One more rule to consider:** a lens must **not** be presented as a default. If a lens is on by default, the app is implicitly asserting a method; if it is opt-in, the athlete has asserted it. The athlete's own words in §1.6 support this: they use method vocabulary as labels, and they reject method rules that conflict with their body.

## 3.4 What cannot be derived from watch data — say it plainly, never imply it

Never derivable, from a watch, in any case:

* **Blood lactate** — so "LT1", "LT2", "the golden zone 2.3–3.0 mmol/L", "threshold at 4 mmol/L" are all *inferences from the athlete's own zone model*, and must be labelled as such. This is the largest single gap: **three of the six methods define their intensity by a lactate number.** Stride cannot verify any of them.
* **RPE and the talk test** — Bakken's own control protocol requires cross-checking HR against these; the method explicitly says HR alone is insufficient.
* **How the run felt** — "the subjective feeling is a tricky thing" (u/npavcec, 179biyh); runners repeatedly report that their sense of "easy" is wrong and that only the watch, or only the feeling, tells the truth — never both.
* **Terrain and conditions** beyond what the FIT records. Note also that elevation in Stride is barometric-rebased and is not a heat model, and that the FIT temperature field is device-dependent, so a large share of runs may have no temperature at all.
* **Motivation, life stress, sleep debt beyond what is stored** — "Stress, mood? Any niggles, injuries or soreness?" (1asawk7) is the runner's own list of what actually explains the session.
* **Whether a day with no recorded run was a rest day** — it may be a forgotten watch. State "no run recorded".
* **Whether the session was run at the intended intensity** — only the athlete knows this.
* **Consistency across devices** — "heart rate may be off if you are using a wrist HRM" (1asawk7).

And the one methodological caveat the ledger must carry, because it changes how absence should be read: **HR time-in-zone under-counts hard work and over-counts easy work** (Seiler 2010). A ledger that reports "no time above zone 4 in five weeks" is reporting a property of the watch's heart-rate lag as much as of the athlete's training.

---

# Section 4 — What could not be verified

1. **The "2018 Stöggl/Seiler marathon study"** (the brief's "Training Intensity Distribution and Simulated Performance of a Marathon"). Not found via Crossref, OpenAlex or PubMed title search under that title or author filter; **unverified**. The nearest verified items are the polarised-TID randomised comparison (Stöggl & Sperlich, *Front Physiol* 2015, PMC4621419) and the 2024 *Sports Medicine* large-sample marathon TID dataset (DOI 10.1007/s40279-024-02137-7).
2. **Ørnskæg's "Fthreshold 2:1 and 3:2 ratios"** and **Ørnskæg's classic 4-week cycle of 4×4 / 2×4 / 3×4 minutes.** No source reached. Additionally, the "Norwegian 4×4" that circulates publicly traces to **NTNU research** (4 minutes at 85–95% of max HR), not to Ørnskæg — the brief's attribution appears to be a conflation, and I could not verify Ørnskæg's specific structure.
3. **Pfitzinger's "slow down to feel easy"** wording, from the book itself; and the **Lydian cadence** figure (180–190 spm) as Daniels' own prescription. Both are widely repeated; neither was verified against a primary source in this pass.
4. **Paywall, price, privacy and account requirements** as *complaints* — no quotable thread surfaced. Also **Hanson** was named in the brief but not researched.
5. **Several archived Reddit threads** returned OP text only (comment sections not extracted). Any quote from those threads is marked *(OP only)*; where only a title was seen it is marked *(title only)*.
6. **The "am I training too hard?" question in its own words** did not appear as a thread title. The nearest attested forms are *"Am I experiencing overreaching /overtraining symptoms?"* and *"80% of my training is in HR zones 4 and 5. Dangerous?"* — and, more often, the easy-end question: *"is my easy run too easy?"*
