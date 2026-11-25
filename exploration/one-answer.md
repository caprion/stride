# One answer per question

*24 November 2026. The brief: `exploration/brief-one-answer.md`.*

The review found the app contradicting itself, on one screen and across
two:

> *"/pace and /progress both know max heart rate 197 bpm · taken from your
> watch on 25 Nov 2026, yet /runs/1 (24 Nov Bengaluru - Supersets) says
> 'No zones yet — your watch's zones were not read for these runs'."*

> *"/progress displays 'No heart rate recorded' right above THE WATCH'S OWN
> ZONES showing 36+ hours of Z1–Z5 HR data."*

And it found a card with no way out:

> *"/pace dead-end UX: empty horizontal bars for every pace row with 'no
> race time entered', no inline button/input, and the 'your own race' legend
> entry has nothing plotted."*

**One basis per answer, and the answer says which basis it used.**

## What changed

### The source line says what it applies to

`zones.model.source` (and its two siblings) now read:

> your zones · HR_MAX · max heart rate 197 bpm · **the watch's
> configuration as it stood on 25 Nov 2026** · **the runs from that day on**

The line names the model and the day it was read (as before) and now says
**what it applies to**: the zones it names are the watch's configuration as
it stood on that day, and they measure the runs from that day on. A run
before it is a different answer, and that answer says so where it is shown.

### A run's own answer states its basis, and says why when it has none

The reason the app's own bands are absent is now the ledger's own sentence,
in the register the vocabulary asks for (`not measured`, not "no zones
yet"):

> not measured — your watch's zones were not read for **this run**; the
> watch's own zone minutes are the only zone time here

The window's reason is the same sentence with "these runs" in it. The run
counts its own runs; the period counts its own.

### A reason belongs to the block it qualifies

The ledger's card has **two blocks** — the app's own bands and the watch's
own zone minutes — and they are two measurements. The reason the app's own
bands are absent now sits **inside the app's own block**, and the watch's
block carries **its own coverage** (how many of the period's runs carried
the watch's own zone minutes). A line above the two blocks read as a
reason about both, and it was about one: the watch's own minutes are
measured over the whole run and are not touched by the app's own bands.

**The rule that keeps it there:** a block's reason and a block's coverage
are drawn inside that block, and a block never carries the other's reason.

### The pace card has a door

The paces hang on a race the athlete types, and the race is typed on the
rig's own page — the app has no screen for it. So the card says where, and
takes the athlete there:

> Your race is typed on the rig's own page, in 'A race'. The paces come
> from that race. — *the rig’s own page* → `/rig#race`

The link lands on the race's own form — the anchor `#race` is the race's
own article — and the card names the place by the name the page gives it,
so the two cannot drift apart.

**A link and not the input here:** the app owns no screen for a race, and a
form in the card would be a second place that writes one, beside the rig's
own. One place writes it, and the card says which.

**The link is the rig's own, so it is drawn where the rig serves the app.**
The phone has no rig's page, and a link that goes nowhere is worse than no
link: there the card says where the race is typed and stops — the same
sentence, no link.

### The legend and the empty bars

- **A legend entry with nothing plotted is not drawn.** The legend lists
  what is on the chart: "your own race" appears only when a pace hangs on
  the athlete's own race.
- **Absence is not zero.** A rung with no pace draws **no bar at all** — not
  an empty track, which reads as a band of zero — and the reason stands
  where the band would have been.

## What is left, and why

- The run's own answer names **why** it has no basis, but not the day the
  watch's zones begin to apply; the day is on the screens that hoist the
  source line. Naming it on the run's page needs one more field on the
  run's zones answer, and the heart-rate thread owns that shape.
- `zones.no_model` ("No zones yet — your watch's heart-rate zones have not
  come through") is still the old register: the ledger's own tests assert
  it, and that file is the ledger thread's.

## The gates

`fmt`, `clippy -D warnings`, the workspace tests, `check-deps`,
`check-corpus`, the contract greps, the secret scan, the scanner
self-tests, `tokens:check`, `lint:voice`, typecheck, unit and viewport are
green.

One is not, and it is the **history scan**: it walks every commit, and it
finds the week thread's **superseded** commit, the one that named the
plan's own sentence `note` — the name the rule that guards the athlete's
free text reads as free text. The shipped commit names it `state_line`,
and the tree's own privacy scan is green with it.

The heart-rate thread's ladder of fractions was read as an unlabelled
coordinate pair; it is written with one decimal now, and this note no
longer writes the pair out. The frozen phone table's missing
`/athlete/heart-rates` is fixed here, because the contract grep compares
the two tables and the branch was red without it.
