# The cards as pictures — the wall, redrawn

*24 November 2026. A prototype: `exploration/cards-as-pictures.html`. New files under `exploration/`
only; nothing under `crates/`, `frontend/`, `android/` or `desktop/` was touched, and nothing was
committed.*

**The brief.** The user, looking at the ledger card prototype: *"the cards at this time look very
verbose. we need to find a way to make them look like charts or other things."* D11 answered that
with a direction — the picture first, the sentence second. This note carries it the rest of the way:
**the picture carries the fact; the numbers are labels on the picture, not sentences; absence is
drawn, not described; the caption says only what the picture cannot.** And the rule that decides a
card: **no card may be a list of sentences.** A card that cannot be drawn is merged or it is dropped.

**What was built.** All eleven cards of the cut wall, redrawn, in their ordinary state on the phone
and on the desktop, plus their states — the empty window, the one run, the missing zone model, the
missing race time, the plan not adopted. One synthetic athlete: 7 Sep – 24 Nov 2026, 58 runs,
49 h 32 m of moving time, 564.8 km. The window is the last twelve calendar weeks through today
(D1), and every comparison is the same span one week earlier (D2).

**The data behind the pictures.** The band's own figures are the ledger's, unchanged:
Z1–Z2 low **41 h 32 m · 84% · 58 runs**, Z3 moderate **6 h 52 m · 14% · 58 runs**,
Z4–Z5 high **1 h 08 m · 2% · 4 runs**, the last of them on **20 Oct 2026** — thirty-five days before
the end of the window, and the last five columns of that row are empty. The longest run is
**20.4 km in 1 h 59 m on 21 Nov**; the period before held **19.7 km**. The repeated work is
**2 h 15 m · 3 runs**, the last with a workout step on **15 Oct 2026** — the watch's steps and the
heart rate's answer point at different dates, as the ledger says they do.

---

## The eleven cards

### 1 · This week — Progress

* **The picture.** Seven columns, one per day. Each day's session is drawn to its length and its
  kind (azure shades for endurance and aerobic, amber for the long run, deep azure for the
  intervals session); the plan's own pacing is a dashed outline over it; the previous week's same
  days stand beside it in pale blue, so the comparison is *in* the picture. The signed figure
  (−5.4 km) is a label on the picture, not a sentence.
* **The caption.** `Mon 23 Nov – Tue 24 Nov 2026` — the period, and nothing else.
* **The states.** *No run recorded this week* — seven empty columns, the previous week's bars still
  standing, and the caption carrying the absence. *No plan adopted* — the outlines are gone and the
  caption says why. A state is a fact about the week, not a placeholder.

### 2 · Where the time went — Progress (the centrepiece)

* **The picture.** Four drawings in one card, each carrying one fact:
  * **the zone split** — one stacked bar, printed once, with the five zones named beneath it
    (D11(d): the polarised roll-up is not printed again; the roll-up *is* the matrix's rows);
  * **the weeks × bands matrix** — twelve columns (the weeks), three rows (the bands), each cell
    filled in proportion to the time that band held that week, so the mix is a picture and a week
    with no hard running is **white space**. Each row is scaled to its own busiest week, so the
    shape of a row is readable even when the row is small. A marker on the last filled cell of each
    row, with the date under it;
  * **the row labels** carry the numbers, compactly and as labels, never as clauses:
    `Z4–Z5 high` / `1 h 08 m · 2%` / `4 runs · 0 m`. The signed figure is the change against the
    period before, and a key at the top of the picture says so — a muted figure, no arrow, no
    colour that judges;
  * **the two shape rows** — *the longest run* as a bar series of the window's long runs with the
    largest held out, and the previous period's longest as a faint reference line across them;
    *repeated work* as marks on the strip of weeks where the watch recorded a workout step, with the
    empty weeks white.
* **The caption.** `7 Sep – 24 Nov 2026 · measured over the moving time` — the period and the
  denominator (D5), and no number the picture already shows.
* **The states.** *One run in the window* — one filled cell in the low row, every other cell white;
  the caption says `one run recorded`. *No zone model* — the split bar is an empty outline and the
  three band rows read **no value / no zone model read**, while **the shape rows still answer**
  (the longest run and the repeated work need no zone model), exactly as the ledger's §4.3 says.
  *No workout step recorded* — the repeated-work row has no marks and says so in one short line
  (`no value` / `no workout steps`); the row is not a zero.

### 3 · The load — Progress

* **The picture.** The fitness and fatigue curves over the window — azure solid, amber dashed —
  with the **form filled as the area between them**, today's value a dot, and the two curves'
  values and the form as labels on the picture.
* **The caption.** `Banister k = 1.92 · the ratio is inside 0.8 to 1.3` — the model's own constants
  and the scenario (the scenario sentence changes with the ratio: the ratio above 1.3 gets the
  product's "with no training for 7 days…" line, below 0.8 gets its mirror). The zone model is not
  named here: it is the screen's one source line (D18.3).
* **The states.** *No run recorded in the window* — an empty frame, no curves. *No zone model* — the
  same empty frame, because the load model needs the model's maximum heart rate.

### 4 · The weeks — Progress

* **The picture.** Twelve weekly bars, the **typical week as a line across them**, and under each
  bar the days that held a run as small marks. One picture for what used to be two panels
  (`volume` + `consistency`).
* **The caption.** `12 weeks · 7 Sep – 24 Nov 2026`.
* **The states.** *No run recorded in the window* — twelve empty columns and no typical-week line
  (a median of nothing is not a number).

### 5 · The pace — Progress

* **The picture.** The athlete's own scatter — pace against heart rate, one point per run — with
  the **training paces as the ladder's marks** on the pace axis (recovery, easy, long run,
  threshold), each with its name.
* **The caption.** `7 Sep – 24 Nov 2026 · the training paces from the 10 km of 49:12 on 18 Nov 2026`
  — the period, and the race the ladder came from, which the picture cannot show.
* **The states.** *No race time entered* — the scatter is drawn and the ladder is not; the caption
  says the training paces have no value. A race time is an observation (D13): Stride never invents
  one, so the ladder is simply absent.

### 6 · Bests & milestones — Progress

* **The picture.** A **timeline** — distance against date — with every run as a dot and the three
  records marked, each with its own shape (circle, square, diamond) so the meaning is carried by
  the shape and not only by colour; the key at the top names them, and each mark is labelled with
  its distance, its time and its date.
* **The caption.** `7 Sep – 24 Nov 2026`.
* **The states.** *No run recorded in the window* — an empty axis with the scale and the period and
  no marks: a list of records with no records is a sentence about nothing.

### 7 · Your shape — Profile

* **The picture.** The seven facets as **seven horizontal gauges on one scale**, each labelled with
  its value, each with the way it moved against the previous window — a signed figure and a
  direction glyph, muted, no colour that judges. This is the radar's replacement (D7 retired the
  radar; a polygon over fixed anchors is a claim, seven bars on a common scale are not).
* **The caption.** `12 weeks, against the same span one week earlier` — the period and the
  comparison, which the picture does not show.
* **The states.** *No run recorded in the window* — no gauges, an empty frame.

### 8 · The months — Profile

* **The picture.** Twelve columns of monthly volume, **each column split by the zone mix** — low,
  moderate, high — so the year's shape and the year's mix are one drawing. November is the partial
  month, to the 24th.
* **The caption.** `the last 12 months · November to the 24th`.
* **The states.** *No zone model* — the columns carry distance and the mix has no value; the key
  changes to say so.

### 9 · The schedule — Profile

* **The picture.** A **week × hour grid** — seven rows, twenty-four columns — with each run a mark
  sized by its distance. That *is* the shape of the athlete's week; three tables of numbers
  (weekday, daytime, distances) was not a picture.
* **The caption.** `7 Sep – 24 Nov 2026 · the runs by weekday and hour`.
* **The states.** *No run recorded in the window* — an empty grid.

### 10 · The habit — Profile

* **The picture.** Two drawings: the **streak as a row of day marks** across the window's days (a
  mark for a day that held a run), the current streak held at the end with its count as a label; and
  the **Eddington as a ladder of ticks** — one tick per distance, each as tall as the number of days
  that reached it, with the Eddington's own tick marked (12, here: thirteen days of twelve
  kilometres).
* **The caption.** `7 Sep – 24 Nov 2026`.
* **The states.** *No run recorded in the window* — no marks, and the ladder has no value.
  *Too few days* — two days recorded, the ladder is not drawn, and the caption says how many days
  there are, because "the Eddington has no value" without the count cannot be acted on.

### 11 · The goals — Profile

* **The picture.** The challenge weeks as a **small grid of met and not-met marks** — a filled
  square where the challenge was met, an empty square where it was not, **never red** — and, under
  it, the plan's own sessions for the week as marks: a filled square for a run recorded, a dashed
  outline for a planned session, a plain outline for a planned session whose day has gone by. The
  adherence line is drawn *into* the card; it is not printed beside the grid.
* **The caption.** `12 weeks · the plan adopted 30 Sep 2026`.
* **The states.** *No plan adopted* — the session row is absent, and the caption says why. *No run
  recorded in the window* — the challenge grid is empty and the count reads `0 of 12`.

---

## What was dropped or merged

**Nothing was dropped.** All eleven cards survive, because every one of them can be drawn. Three
were merged into their neighbours rather than kept as pictures of their own:

* **the radar** — retired by D7; the seven facets are now seven gauges on one scale (*your shape*);
* **the `/pace` page** — retired; its cloud and its bands are the scatter and the ladder on *the
  pace*;
* **the polarised split** — retired into the ledger card; the five zone shares are one bar, printed
  once, and the roll-up is the matrix's rows (D11(d)).

**The card that came closest to resisting a picture was *the goals*.** Its challenge grid was
already a picture, but its adherence line was a sentence ("9 of 10 sessions done") and a sentence
cannot be drawn next to a grid without becoming a second fact in two registers. So the sentence was
drawn into the grid: the week's sessions became a row of marks, and the grid and the line are now
one drawing.

---

## The one sentence to add to D11

> **A number the picture can carry is a label on the picture; a caption names the period and the
> reason something is missing, and nothing else.**

D11 said the picture comes first and the caption second. This exercise found the sharper edge: the
failure is not a caption that is too long but a caption that is doing the picture's job — and the
test is mechanical, **no sentence repeats a number the picture shows**. A card is finished when
every number in it is either drawn or is the period, and when the absences are white space rather
than clauses.

---

## The states, and what they say

Every card must state its absence, and the absence is a fact, not a placeholder. The five states
the wall needs, and the words they use:

| the state | what the picture does | what the caption says |
|---|---|---|
| **an empty window** | the frame, the strip or the grid, empty | `no run recorded in the window` |
| **one run in the window** | one cell filled, the rest white | `one run recorded` |
| **no zone model** | the band rows read `no value / no zone model read`; the shape rows still answer | `no zone model has been read — the band rows have no value` |
| **no race time** | the scatter without the ladder | `no race time entered — the training paces have no value` |
| **no plan adopted** | the plan's row and outlines are absent | `no plan adopted` |
| **no workout step recorded** | the repeated-work row has no marks and says `no value` | the period, and the row's own short line |

Two of these were the hardest to draw, and both turned on the same thing — **an absence that has a
count is a fact, and an absence that has no count is only a reason**:

* **no zone model.** The band rows cannot say "0 m" (that would be a wrong number) and cannot say
  "no value" alone (nobody can act on that). So the row says **no value** and the card's caption
  says **why**, and the shape rows beside it keep answering, because the longest run and the
  repeated work never needed the model. The card that can only partly answer says which part.
* **too few days for the Eddington.** "The Eddington has no value" is a sentence about nothing; the
  caption says **two days recorded** instead, which is the fact the athlete can use. A state that
  cannot be given a number has to be given a reason instead — and the reason has to name the
  absence, not blame it.

One rule the states taught the pictures: **never draw an absence that the caption is also
describing.** The empty frames in *the load* and *your shape* carry no words at all; the caption
carries the reason. Where the absence needs a number of its own — the days recorded, the runs
counted — the number goes on the picture and the caption names the period.
