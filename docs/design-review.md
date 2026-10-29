# Design review

A review of how Stride looks, how it behaves on a phone, and how much it
costs to run — measured, not guessed, and with the fixes that came out of
it already applied.

**Method.** Every route rendered at 390×844 (a phone) and at 1440×1000 (a
desktop), in headless Chromium. The heap was read from the debugger after a
garbage collection, because Chrome's `performance.memory` is bucketed and
reports the same number on every screen. Storage was measured with
`dbstat`. Every number below is from a script in `scripts/`, not from a
judgement call.

**Status.** §5 says what was done; §6 says what is still open. The Rust port
did not change the front end, so the measurements in §1–§3 still describe the
app the athlete sees.

---

## 1. The footprint

### What it costs now

| | before | after |
|---|---:|---:|
| Initial bundle | 220 KB (56 KB gzip) | **60 KB (18.8 KB gzip)** |
| First load of a simple screen | 1,093 KB | **813 KB** |
| JS heap, a typical screen | 9.5 MB* | **3.3 – 6.2 MB** |
| JS heap, physiology | 24.8 MB | **10.7 MB** |
| DOM nodes, physiology | 3,087 | **1,200** |
| Long tasks over 50 ms | 0 | **0** |
| Database | 18 MB / 160 activities | — |

\* the 9.5 MB was the *bucketed* figure Chrome reports; the real number was
higher on every screen.

**The CPU is fine.** Not one long task on any screen. The phone is not
struggling with the maths — it is struggling with the DOM.

### The two changes that mattered

**Every screen is loaded on demand.** The pages pull in Recharts and
Leaflet, and a phone that opens one chart should not pay for the map.
Splitting the routes took the shell from 220 KB to 60 KB. The two heavy
libraries are still the payload — Recharts is 112.7 KB gzipped and Leaflet
44.9 KB, together 54% of everything — but they are now *per screen*.

**The physiology page was drawing 365 days of stacked sleep bars** — 642 SVG
paths, 1,503 group wrappers — and a circle for each of 365 readiness
scores. It now has a range selector (30 days / 3 months / year) and the
readiness year is a single gradient strip. **57% less heap, 61% fewer
nodes** — and the sleep chart is readable for the first time.

### Storage: the interesting one

| table | size | share |
|---|---:|---:|
| `activity_stream` | 12.1 MB | **67%** |
| `activity` (routes + raw payloads) | 4.2 MB | 23% |
| everything else | 1.8 MB | 10% |

115 KB per activity, of which the *exploded per-second series* is the bulk.
The original FIT file for the same activity is **20 KB**.

> **The phone should keep the FIT and derive from it.** Six times smaller,
> and the FIT is the source of truth anyway. The desktop can afford to
> explode it because the desktop has the disk.

**This is still open.** It is item 1 in §6.

---

## 2. The colour

### What was actually wrong

Not the brand. Azure and amber are a good pair and they are not "dull" —
they are *underused*.

The real finding is this. The palette module already defines a metric
colour system:

```ts
heartRate: '#1E5E96',  pace: '#B57C06',  cadence: '#2E8B7A',  elevation: '#6E6BA8',
```

And it was used in **7 places**, against **36 direct palette literals** in
the charts. The activity page — the one place where colour carries the most
meaning — had its own local constants:

```ts
const HR_COLOR = ACCENT_DATA      // amber
const PACE_COLOR = RAMP[5]         // azure
const ELEVATION_COLOR = RAMP[1]    // orange
```

So heart rate was **amber**, exactly the colour of the volume chart on the
dashboard and the accent of every button. Pace was the brand's own azure.
Elevation was nearly the same amber as heart rate.

**There was no colour language.** The same hue meant different things on
different screens, which is exactly what "dull" feels like from the inside.

### The fix: a metric language

```ts
export const METRIC = {
  pace:       '#1E5E96',   // azure  — the spine; everything is measured against it
  heartRate:  '#D64A4A',   // crimson — heat
  elevation:  '#6E6BA8',   // violet  — terrain
  cadence:    '#2E8B7A',   // teal    — rhythm
  drift:      '#B4557F',   // plum    — the gap between expected and actual
  effort:     '#F2B01E',   // amber   — what it cost
}
```

One family, similar chroma and weight, so a screen with five of them reads
as one palette rather than five. **Amber is reserved for effort** — the
volume you spent, the bar that is yours — so the accent colour keeps a
single meaning instead of being decoration.

Applied, the activity page now reads at a glance: a red heart-rate line, a
blue pace line, a violet elevation line. You know what you are looking at
before you read the title.

---

## 3. Every screen, on a phone

Screens are counted in phone viewports (844 px tall).

| screen | height | verdict |
|---|---:|---|
| dashboard | 10.0 | good. The first screen answers "what is today". |
| plan | 9.2 | good. The week strip and the verdict reasons are the point of the app. |
| feed | 9.8 | good. |
| activity | 8.9 | good, once the charts have their own colours. |
| training | 4.4 | good. |
| physiology | 6.9 | was 14.8. Fixed by the range selector. |
| fitness | 14.8 | **still too long** — see below. |
| compare | 3.0 | good. |
| stats | 5.8 | good. |
| settings | 8.2 | good. |

**The phone shows six of them.** Today, Plan, Runs, Body, Form and Stats.
**Compare** and the **training calendar** are cut from the phone and kept on
the desktop — the desktop has the room, and they are good there — along with
the server settings. The cut is the phone's, not the product's; the contract
is `docs/phone-simplification.md`.

### The bug that was hiding

The fingerprint page rendered **938 CSS pixels wide on a 390-pixel
viewport**. Three separate causes, each a classic:

1. A grid item's default `min-width` is its content's, so the scrolling
   table refused to shrink and blew out the page. (`min-w-0`.)
2. The card header's action slot had `shrink-0`, so the eight-option metric
   selector could never be narrower than its options and ran off the edge.
3. The segmented control itself had no way to scroll.

All three are fixed. **The page is now exactly the viewport wide**, with
the table scrolling inside its own container.

---

## 4. What to pick up, and from where

Not a survey. Five specific things, each from an app that does it well.

| the idea | from | why it fits here |
|---|---|---|
| **A range selector on every trend** | Oura, Whoop | A year of daily bars is not a chart, it is a smear. The selector is now on physiology; it belongs on the fingerprint trend and the pace trend too. |
| **A year as one colour strip** | Oura's sleep history | Replaced 365 SVG circles with one gradient node. Reads better *and* costs a fraction. |
| **A stable colour per metric** | Strava, Garmin Connect | Heart rate is red everywhere it appears, because the point of colour is recognition. |
| **One number, then its drivers** | Whoop | The readiness card: 67, then what moved it. Already built; the drivers need their units. |
| **A map as the hero of an activity** | Strava | The route is the thing you remember. It should be the first thing on the activity page, not the third section. |

### The one to take from Garmin Connect

Garmin colours **body battery** green→amber→red and **stress** blue, because
a watch face has to be read in two seconds in the sun. Stride's readiness
grade does the same job — peak, good, fair, low — and it already has a
scale. It should be a **traffic light**, everywhere: in the strip, in the
badge, in the driver list. Right now the grade colours are one set and the
verdict colours another, and they overlap.

---

## 5. What was done

Each of the five open items from the last review, and where it was fixed.

| the item | the fix | where |
|---|---|---|
| **the fingerprint table is 160 rows** — fourteen screens of scrolling on a phone | one activity per card on a phone, with the four numbers that matter; the table stays for the desktop | `frontend/src/pages/FitnessPage.tsx` — `FingerprintCards` |
| **units are inconsistent** — the driver list showed `6.3` where the line below it said `6h16m` | every driver renders its own unit, and its baseline and delta with it | `frontend/src/pages/PhysiologyPage.tsx` — the driver row renders `d.unit` |
| **the dashboard's weekly volume bars are one amber at low opacity** — every bar the same colour, so the chart carries no information | the bars encode the volume tier: deeper amber is closer to the goal | `frontend/src/pages/DashboardPage.tsx` — the weekly volume chart |
| **the comparison screen says "0/8"** — eight runs on a shared distance axis is a lot to compare | the copy says what to pick: "Two or three reads best — the chart puts them on one distance axis. Eight is the limit." | `frontend/src/pages/ComparePage.tsx` |
| **the readiness score showed `67.39`** | it is rounded: `Math.round(current.score)` | `frontend/src/pages/PhysiologyPage.tsx` |

And the order of work that followed:

| | what | state |
|---|---|---|
| 1 | Fingerprint table as cards on a phone | **done** |
| 2 | Units on every driver number | **done** |
| 3 | Volume bars coloured by tier | **done** |
| 4 | Range selectors on the other trend charts | **done** — the fitness trend has one too |
| 5 | The map as the hero of the activity page | **done** — `RouteHero` is section 1 of the activity page, before the title and the stats |
| 6 | Store the FIT, derive the streams | **open** — see §6 |

---

## 6. What is still open

1. **The phone still stores the exploded per-second series.** 115 KB per
   activity against **20 KB** for the original FIT file — **six times** the
   storage, and the FIT is the source of truth anyway. The fix is in the
   store, not in the UI: keep the FIT, derive the streams on read. The
   desktop can afford to explode it; the phone cannot.
2. **The renderer costs about 100 MB on the phone.** Measured by the memory
   workstream. A native UI is the answer if it matters; it is not yet a
   decision, and the bundled WebView is what ships.
3. **The readiness traffic light is not a traffic light.** The grade colours
   and the verdict colours are still two sets that overlap. It should be one
   scale, everywhere: in the strip, in the badge, in the driver list.
4. **The fingerprint page is still 14.8 screens.** The cards helped; the
   trend on it did not get a range selector until late, and the page is
   still the longest in the app.

---

## 7. The direction, honestly

**What gets better**

- The app has a colour language, so a chart is legible without its title.
- The phone footprint is real: 3–6 MB of heap, no long tasks, 60 KB of
  shell.
- The screens that were broken on a phone are fixed, and the overflow was
  hiding on one page only.
- Nothing about the design is expensive to maintain. The palette is one
  file; the rules are in comments.

**What gets worse**

- Six metric colours is one more thing to keep consistent. The
  `METRIC` object is the only place to change it, but only if the discipline
  holds.
- The range selector adds a control to a screen. A screen with a range
  selector, a metric selector and a table is a lot of furniture.
- Storing the FIT and deriving from it means the phone does more work per
  read. It is the right trade for storage, but it is a real cost in CPU.

**What I would not do**

- **Do not change the brand colours.** Azure and amber are fine. The
  problem was that they were doing five jobs. Fixing that is worth more
  than any new palette.
- **Do not add a social anything.** No sharing sheet, no "your friend also
  ran", no streak-sharing. It would undo the thing that makes this
  different.
- **Do not put a number on the home screen that cannot be explained.** The
  dashboard shows readiness, volume, pace and the plan for today. That is
  the right amount.
