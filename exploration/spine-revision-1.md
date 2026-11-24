Parent: three changes to your brief, plus the primary source.

## 1. The primary source is in hand — the book itself

The user attached **Pete Pfitzinger, *Faster Road Racing: 5K to Half Marathon***:

`/home/sumit/.bb/thread-storage/thr_yd4irqeycg/Attachments/vdoc.pub_faster-road-racing-5k-to-half-marathon.epub`

It is an EPUB; you can read it with Python's `zipfile` (no `unzip` on this
box): the chapters are `OEBPS/xhtml/E6202_chapNN_dNN.html`, the front matter
is `..._contents_d72.html`, `..._introduction_d75.html`, and there are three
appendices. Strip the tags, and you have the text.

This is now your **primary source** for the sub-marathon levels — it beats
every third-party summary, and it beats the marathon book for the
distances this app cares about. Treat the earlier instruction ("prefer the
author's published statements") as: **the book first for anything it covers,
then the author's public statements, then third-party summaries (labelled)**.

**Copyright discipline (important).** The repository is **public**
(`github.com/caprion/stride`). Do **not** copy the EPUB into the repository,
do not commit it, and do not paste long passages into any file in the
repository. Extract the **rules and facts** — the numbers, the structures,
the conditions — with a **short** quotation where a wording matters, and cite
the chapter. The file stays in thread storage; leave it where it is.

## 2. The product's scope just sharpened — and it changes your Part 1

* **The app targets the intermediate runner.** The user's words: *"mark this
  whole app for intermediate runners. For entry level, Strava/Garmin is
  enough."* The entry-level runner is not the audience; the plan structures,
  the vocabulary and the analytics all assume an athlete who is already
  training, has a base, and is chasing a specific distance.
* **Pfitzinger is the base model** — the app's first method, the plan the
  athlete adopts.
* **80/20 is a *separate second method*, not a lens and not a variant of
  Pfitzinger.** The user was explicit: *"80/20 and Pfitzinger are two
  separate methods, not the same. So those will be two different defaults."*
  So: **two plan bases, one at a time.** The athlete's own history and the
  ledger speak against the base they are following.

## 3. What to add to `exploration/pfitzinger-spine.md`

### Part 4 — The book's training components and its session vocabulary

The book names its run types (general aerobic, tempo/threshold, intervals,
long run, recovery, strides/speed work, and its cross-training). Extract
**each type with its rule** — the pace relative to MP or threshold pace, the
duration, the frequency, the recovery, and any condition ("only if...", "not
more than two per week").

Then produce the table the whole product needs:

| the book's run type | its rule (pace, duration, frequency) | our stimulus kind | what the watch can verify | what only the plan or the athlete can |

Our stimulus kinds are, for reference: **Z1–Z2 low · Z3 moderate · Z4–Z5
high · the longest run · repeated work** (the watch's own workout steps).
The middle two columns are the point: which of the book's run types the app
can **verify** from the watch's data, and which exist only as **intent** in
the plan. That mapping is what the ledger will use to say "the plan had a
tempo session on 14 October; the last run in Z4–Z5 was 15 October" — and to
know which rows are evidence and which are merely the plan's words.

### Part 5 — The plan structures, from the book

The book has training plans for 5K, 8K, 10K and the half marathon, plus an
"Adjusting the Training Plan" section. Extract, for each distance: the
plan's length in weeks, the weekly structure (which days carry what), the
long run's progression, the taper, and **how the plan is adjusted** when life
interrupts. Give the **shape** precisely — the app will re-express these as
dated sessions with paces, so the numbers must be exact, and each with a
chapter citation.

### Part 6 — The 80/20 plan, as a peer

The polarized week (structure, from Seiler's 80/20 and published polarized
plans), and — again — **how it differs from a Pfitzinger week in the mix**,
because the app holds one base at a time and the athlete must be able to see
which one they are following.

## Also

* The pace engine thread (`thr_crvvnnhwtz`) is designing the chain
  **race time → VDOT → MP → the paces** and is leaving placeholders exactly
  where your numbers go. When you have them, say **which placeholder each
  number fills** — a list of "rule → the pace it produces" — so that thread
  can be told to wire them in.
* Read `exploration/decisions.md` (D13 and the new scope note) first.

## Report back (add to the earlier list)

* the book's run types and their rules, each with a chapter citation,
* the run-type → stimulus-kind → verifiable? mapping table,
* the plan structures per distance, in numbers,
* the two plan bases side by side, and how the app keeps them apart,
* which placeholders in the pace engine your numbers fill.
