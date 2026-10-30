# Stride: what it is for, and what v1 is

## The thesis: Plan → Execute → Understand → Adapt

A runner's week is a loop:

1. **Plan**: what am I trying to do this week, and today?
2. **Execute**: run.
3. **Understand**: what did that run mean? Was it easy enough, too long, a
   sign of getting fitter, a bad night's sleep showing up?
4. **Adapt**: so what should I do tomorrow?

The watch does step 2 brilliantly. Most running apps stop there: a social
log of what was recorded. Stride exists for steps 3 and 4, and for closing
the loop back to 1. Every screen should answer a question in that loop. A
screen that only shows a number does not belong.

## Three jobs, in order

| job | screen | the question it answers |
|---|---|---|
| **Decide** | Today | Should I run today, and how much? |
| **Understand** | Runs | What did that run mean? |
| **Stay motivated** | Progress | Is this working? |

**Body** is the evidence behind Decide: sleep, HRV and resting HR against
your own normal. Settings is short.

## Principles

Each one rules something in or out, and each says what it rests on:
**evidence** (a published study), **practice** (consensus in coaching and
sports-science practice, not one decisive study), or **judgment** (a
product decision we own). None comes from a survey of runners; the
direction comes from the literature, app-store and forum complaints about
running apps, and dogfooding.

1. **A decision with a reason beats a score.** Today leads with a sentence
   ("Go easy today, or take the day.") and the reason ("Slept 5h36 vs your
   usual 7h12."), not a 0-100 number. Never two signals that disagree: Body
   and Today read the same "normal" from the core, and when a written plan
   has a session today, Today names it and says how the call relates to it
   ("Your plan says Intervals. Recovery says go easy: keep it gentle, or
   swap it for an easy run."). *Judgment.*
2. **One guardrail, the one with evidence.** A single run more than 10%
   longer than your longest of the last 30 days is the spike that predicted
   injury in the 2025 Garmin-RUNSAFE cohort; weekly load ratios (ACWR) did
   not. Stride shows the spike limit and nothing that pretends to be an
   injury score. The limit is where observed risk rose, not a safe zone:
   the study is observational, its authors suggest a lower cut-off may be
   worth considering, and runs that creep up just under the line compound.
   The screen says where the number comes from and that under it is not
   proven safe. *Evidence:* Frandsen et al., BJSM 2025 (5,205 runners);
   the framing is *judgment*.
3. **Your normal, not a population's.** HRV, resting HR and sleep are read
   against your own recent range. Efficiency (speed per heartbeat) is
   compared with your own runs of a similar distance. *Practice:* HRV and
   resting HR are read against individual baselines in the recovery
   literature; population norms vary too widely to judge one person.
4. **Compare only with your past self.** No leaderboards, no age grades.
   Progress is "faster at the same heart rate than six weeks ago".
   *Judgment.*
5. **Forgiving consistency.** Weeks, not days. One quiet week in six is a
   grace week. A pause (travel, sick, injured) freezes the streak instead
   of breaking it. Nothing is red, nothing says "missed". *Judgment*,
   informed by habit research on how broken streaks end habits.
6. **Never make up a behind week.** A low-volume week never produces advice
   to cram the difference into the weekend. *Evidence:* the same spike
   finding; cramming is how a single long run spikes.
7. **Show what the watch records, hide what it does not.** A card whose
   data is missing is not drawn; it does not show zeros. *Judgment.*
8. **Honest about confidence.** Single runs are noisy (heat, hills, wrist
   HR). Trends use medians over weeks; a run's story says "likely" when it
   is inferred. *Practice.*

## Local-first, and why it is non-negotiable

**Your data stays on this phone.** The Rust core, the encrypted database and
the Garmin key all live on the device. There is no Stride server, no
account, no analytics. The desktop binary is the same core, serving
`127.0.0.1` only.

Consequences we accept:

- Stride uses the official **Garmin Personal API key** only. Garmin's own
  computed scores (Training Readiness, VO2max, Training Status) are only
  reachable with your Garmin password through unofficial endpoints, so
  Stride computes its own from the raw data instead.
- Strava's 2026 API terms forbid storing history, combining sources and AI
  use, so Strava data comes in only as files you export yourself (FIT).
  COROS has a self-service OAuth route that fits the same model and is the
  next connector (not in v1).

## What v1 is

**v1 reads, decides, and takes three small inputs.**

In:

- Connect a Garmin Personal API key; import FIT files.
- **Today**: one call, one reason, one guardrail; the week as seven dots
  against your goal or usual week; last night in one line; recent runs.
- **Runs**: every run gets a chip ("Longer than usual", "Fitter than
  usual", "1-year best: 10K") and a short story under the map.
- **Progress**: pace at the same heart rate, consistency with grace weeks,
  best runs by distance, race ranges, lifetime milestones; Stats and
  Running form one tap away.
- **Body**: sleep, HRV, resting HR against your own normal.
- Three inputs that change what the app should say: a run's name and note,
  an optional weekly goal, and a pause. The goal and the pause live in the
  app's own storage on the device, not in the encrypted database: the
  database schema is shared with the desktop export and frozen for v1, and
  neither is training data. They move into the database (and into the
  core's advice) when the schema next changes; clearing the app's storage
  clears them.
- Desktop: the same, plus the Plan calendar, Compare and full Settings.

Out of v1 (deliberately):

- Writing a training plan on the phone. Plans arrive from Garmin or a file;
  the Plan screen stays on the desktop until it reads well.
- Editing a run's numbers.
- Garmin's proprietary scores, and anything needing the Garmin password.
- Social features, leaderboards, coaching chat, on-device AI.
- Strava and COROS connectors (FIT import covers both for now).
- Judging a run's purpose. Compare marks "strongest" only between runs of
  a similar distance; a 10K easy run and a 10K time trial can still be
  compared. Purpose will come from a written plan's session type when
  there is one; guessing it from the run itself is not reliable enough.

## How we decide what to add

A feature earns its place if it answers a question in the loop, uses data
the watch actually records, and keeps every byte on the device. When in
doubt, a sentence with a reason beats a chart.
