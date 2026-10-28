# What the renderer holds: a memory audit of the app

**Status:** measurements taken on the production build served by
`scripts/run.sh` (`frontend/dist`, built by `npm run build`, served from
`http://127.0.0.1:8000`), on the data set in this repository: 160
activities, 371 days of physiology, one training block.

**The question.** The Android app is a WebView. The athlete's phone pays
for the whole renderer — the Blink/V8 process, the decoded image cache,
the code cache, the service worker's cache and the retained DOM — and only
one of those is visible in a JavaScript heap profile. The measured JS heap
of a screen is 3.3–10.6 MB; the renderer's real cost on a phone is several
times that. This document measures the layers separately, finds the levers,
pulls the cheap ones, and says what was left alone.

---

## 1. The harness

`scripts/measure.mjs` was extended rather than replaced. It now measures
five layers separately, because they are paid for separately and they are
fixed by different code:

| Layer | How it is read |
|---|---|
| JS heap | CDP `HeapProfiler.collectGarbage`, then `Runtime.getHeapUsage` |
| DOM | `document.querySelectorAll('*')`, with a histogram by tag name |
| Decoded images | `naturalWidth × naturalHeight × 4` for every `<img>` |
| Query cache | `queryClient.getQueryCache()`, with the serialised size of each entry's data |
| Service worker cache | `caches.keys()` → `cache.keys()` → the size of every cached response |

Plus the network (`performance.getEntriesByType('resource')` for transfer
sizes and the decoded-body sizes), the long tasks
(`PerformanceObserver` for `longtask`, installed before the first document
script) and the CDP performance domain for scripting time.

Two modes:

```
node scripts/measure.mjs                 # every route in a fresh context
node scripts/measure.mjs --session       # one renderer, every screen in order
node scripts/measure.mjs --json out.json # machine-readable
```

Three companion scripts:

| Script | What it answers |
|---|---|
| `scripts/measure-map.mjs` | How many map tiles are fetched, at what device pixel ratio, what is decoded, and what is left behind when the route unmounts |
| `scripts/measure-heap.mjs` | Where a screen's heap actually goes, from a heap snapshot |
| `scripts/measure-sw.mjs` | What the service worker's cache holds, and what it keeps from a past build |

Two things the harness needs from the app:

* `window.__strideQueryClient` — one property on `window`, set in
  `App.tsx`, so the harness can read the query cache. No dependency, no
  behaviour change.
* the app must be the **production** build. The harness measures
  `http://127.0.0.1:8000`, which serves `frontend/dist`; the Vite dev
  server is not used anywhere in these numbers.

**Noise.** Heap, DOM, decoded-image, cache and transfer figures repeat to
within a few percent. Long-task counts and scripting totals do not: two
runs of the *same build* differed by up to 3× (physiology: 4042 ms vs
2853 ms of long tasks). Those two numbers are reported, but no conclusion
is drawn from them.

---

## 2. Layer by layer, before

Every route in a fresh context, so the figures are the cost of *opening*
that screen rather than everything opened before it.

| Route | JS heap | DOM nodes | Decoded | Query cache | API transfer |
|---|---|---|---|---|---|
| dashboard | 6.06 MB | 1327 | 0 KB | 24 KB | 27 KB |
| plan | 5.18 MB | 934 | 0 KB | 129 KB | 133 KB |
| feed | 3.92 MB | 1375 | 0 KB | 40 KB | 42 KB |
| activity | 9.16 MB | 986 | **1024 KB** | 638 KB | 790 KB |
| training | 4.46 MB | 702 | 0 KB | 14 KB | 16 KB |
| physiology | 10.69 MB | 1360 | 0 KB | 474 KB | 487 KB |
| fitness | 5.10 MB | 1432 | 0 KB | 95 KB | 100 KB |
| compare | 3.61 MB | 3333 | 0 KB | 41 KB | 43 KB |
| stats | 4.25 MB | 574 | 0 KB | 23 KB | 26 KB |
| settings | 3.32 MB | 580 | 0 KB | 7 KB | 10 KB |

### The DOM is not the problem

375 to 3333 nodes. The heaviest DOM is the compare page, and its histogram
says why: **1140 `<span>` and 164 `<button>`** — the candidate list, one
entry per run — against 43 `<path>` from the charts. No screen has a
hidden measurement container: every `.recharts-wrapper` on every screen has
a real width and height and exactly one `<svg>` surface.

### Decoded images are the map, and only the map

One screen has any image in it at all. The activity page draws four
256×256 map tiles:

```
4 tiles × 256 × 256 × 4 bytes = 1024 KB decoded
```

The browser keeps a second copy of each decoded bitmap for the GPU, so the
real cost of that map is **about 2 MB of renderer memory that no heap
profile shows**. Nothing else in the app decodes an image: the route
thumbnails on the feed are inline SVG, not bitmaps.

### The query cache is where the weight was

After one screen, the cache holds what that screen asked for. After a
session, it holds everything:

```
["activities","neighbours",200]   470 KB
["physiology-readiness"]           205 KB
["physiology-daily"]               177 KB
["plan-current"]                   100 KB
["fitness-fingerprint"]             84 KB
["activity",162]                    78 KB
```

Three of those five are payloads the screens never draw.

---

## 3. The session: one renderer, every screen

This is the number that matters on a phone. The athlete browses; the
renderer keeps what every screen asked for.

| | Before |
|---|---|
| Final JS heap | **8.52 MB** |
| Query cache | **1346 KB in 37 queries** |
| Heap at the activity screen | 10.37 MB (809 KB cached) |
| Heap at the physiology screen | 13.83 MB (1287 KB cached) |
| Service worker cache | 295 KB |

One screen at a time never shows the accumulation. The session shows it:
by the last screen the cache holds 1.3 MB of JSON, parsed into objects
that stay alive for as long as the cache does.

---

## 4. The levers

### 4.1 The neighbour list on the activity page — the biggest one

The previous/next links need four numbers per run. The list endpoint
answers with the whole summary, **including the route preview** — 90 GPS
points per activity, about 2.3 KB a run. Fetching 200 of them to find the
two that bracket the current activity cost **473 KB transferred and 470 KB
cached**.

Now: a window of ±30 days around the activity is fetched (11 activities,
**45,250 bytes**, against 484,061 for the whole list), and the whole list
is only asked for when the window cannot answer the question — the run is
not in it, the run sits at its edge, or there are more runs than one page.
The two links are the same two runs: verified against the full list for
activity 162 (previous: 163 "Nagpur Running", next: 321 "Bengaluru -
Benchmark Run").

### 4.2 The physiology payloads

The three heaviest endpoints answer for a year of days with every field
they have, and the readiness row carries seven drivers, each with a
baseline, a delta and a sentence of prose.

| Endpoint | Before | After | Kept |
|---|---|---|---|
| `/physiology/readiness` | 212,644 B | **18,896 B** | date, score, grade |
| `/physiology/daily` | 186,729 B | **42,463 B** | the six fields the table draws |
| `/physiology/hrv` | 41,107 B | **35,997 B** | the five fields the chart draws |

The server gained an optional `fields` parameter on these three
endpoints; the default response shape is unchanged, so any other consumer
of the API still gets what it always got. The frontend asks for the fields
it draws.

The drivers are not lost: "Today's readiness" shows them, and it reads
them from `/physiology/latest`, which still returns the full breakdown for
the latest day. The year strip still covers the year, because the
readiness request still asks for the whole default range — only the
undrawn fields are gone.

### 4.3 The fingerprint list on the activity page

The activity page fetched **every** fingerprint in the year (84 KB) to
find the one for the activity it was showing. It now asks for that one
activity's fingerprint — a few hundred bytes.

### 4.4 The plan's phases

`/plan/current` answers with the block, every phase of it, every week and
every session: 133 KB, of which **107 KB of phases that no screen draws**
(`grep phases frontend/src` finds no reader). The client now drops the
phases before they reach the query cache: 129 KB → 34 KB cached. The
transfer is unchanged — see §6.

### 4.5 The running-form list

A fingerprint row carries 29 fields; the screen draws 13. The list is now
projected to those 13 before it is cached: 95 KB → 48 KB.

### 4.6 The cache's own lifetime

`gcTime` was 5 minutes (the default). A phone that browses for ten
minutes accumulates every screen it visited in the last five. It is now
**2 minutes** — long enough that a glance at another screen and back
still shows the old data while it refetches, short enough that a session
does not pile up.

### 4.7 The service worker's cache

The worker caches the shell, the assets and the fonts. Its cache is named
after the app, not after the build; the assets are content-hashed; and
nothing ever deletes an entry. So every release adds a set of files and
keeps the previous set alive, forever.

Measured with a persistent browser profile:

| | Entries | Size |
|---|---|---|
| One build | 15 | 792 KB |
| Two builds, no reconciliation | 24 | 925 KB |
| Two builds, with reconciliation | 15 | 792 KB |

The client now leaves a marker in the cache recording which assets belong
to the running build, and drops the assets of the build it replaced when
the build changes. The fonts and the shell are not content-hashed and are
kept.

(One subtlety worth recording: the cached responses carry `Vary: Origin`,
so a cached entry can only be deleted by the *stored request* — a bare
URL string does not match it. The first version of this silently deleted
nothing.)

---

## 5. After

### Per route

| Route | JS heap | DOM nodes | Decoded | Query cache | API transfer |
|---|---|---|---|---|---|
| dashboard | 6.16 MB | 1314 | 0 KB | 24 KB | 27 KB |
| plan | 5.10 MB | 923 | 0 KB | **35 KB** | 133 KB |
| feed | 3.94 MB | 1375 | 0 KB | 41 KB | 42 KB |
| activity | **7.19 MB** | 986 | 1024 KB | **86 KB** | **277 KB** |
| training | 4.49 MB | 702 | 0 KB | 14 KB | 16 KB |
| physiology | 10.63 MB | 1360 | 0 KB | **145 KB** | **151 KB** |
| fitness | 5.06 MB | 1432 | 0 KB | **48 KB** | 99 KB |
| compare | 3.62 MB | 3333 | 0 KB | 42 KB | 43 KB |
| stats | 4.20 MB | 574 | 0 KB | 23 KB | 26 KB |
| settings | 3.32 MB | 580 | 0 KB | 7 KB | 10 KB |

API bytes across the ten screens: **1674 KB → 825 KB**.

### The session

| | Before | After | Change |
|---|---|---|---|
| Final JS heap | 8.52 MB | **7.51 MB** | −1.01 MB (−12%) |
| Query cache | 1346 KB / 37 queries | **405 KB / 38 queries** | **−941 KB (−70%)** |
| Heap at the activity screen | 10.37 MB | 8.66 MB | −1.71 MB |
| Heap at the physiology screen | 13.83 MB | 12.78 MB | −1.05 MB |
| Heap at the fitness screen | 8.94 MB | 7.95 MB | −0.99 MB |
| Activity page, one screen | 9.16 MB | 7.19 MB | −1.97 MB (−22%) |

The query cache is now, in full:

```
78 KB  ["activity",162]            the streams the charts draw
44 KB  ["physiology-sleep"]
40 KB  ["physiology-daily"]
37 KB  ["fitness-fingerprint"]
36 KB  ["compare-candidates"]
35 KB  ["activities",1,"all","date",""]
35 KB  ["physiology-hrv"]
18 KB  ["physiology-readiness"]
```

Nothing in that list is a payload the screen does not draw.

---

## 6. Measured, and deliberately not changed

### The tile proxy serves PNG (`backend/app/api/tiles.py`)

Four tiles, 143 KB transferred, 1024 KB decoded. WebP would cut the
transfer roughly in half and **would not change the decoded memory at
all** — a decoded tile is 256×256×4 bytes whatever it was compressed as.
The decoded cost is set by the tile's resolution and by how many are on
screen, not by the codec. The tile proxy is not in this change's
remit.

### The device pixel ratio does not inflate the request

Measured at DPR 1 and DPR 3 (`scripts/measure-map.mjs --scale=3`): the
same four tiles, the same 143 KB, the same 1024 KB decoded. Leaflet only
asks for retina tiles when told to, and Stride does not tell it to.

### Tiles are not decoded that the athlete never sees

After two zoom-ins the map has fetched 16 tiles and holds 6 in the DOM
(1536 KB decoded) — the visible ones plus the buffer Leaflet keeps. Leaflet's
`keepBuffer` was measured at 1 and at the default 2: **no difference** (6
tiles either way), so it was not changed.

### The map is released when the route unmounts

`react-leaflet` 4.2.1 calls `map.remove()` on unmount. Measured: after
leaving the activity route, **0 tile elements and 0 Leaflet nodes** remain
in the document, and the event-listener count returns to the app's
baseline (178, the same as any other screen). Nothing leaked; nothing
needed fixing.

### The API does not compress

`curl -H "Accept-Encoding: gzip, br"` against the activities list
downloads **484,061 bytes** — the server sends uncompressed JSON. Every
number in §2 and §5 is therefore a wire cost as well as a memory cost.
`GZipMiddleware` in `backend/app/main.py` would cut the API's bytes by
roughly an order of magnitude; that file is not in this change's remit.

### The plan's phases still cross the wire

`/plan/current` still sends 107 KB of phases that nothing draws. The
client drops them from the cache, so they cost no memory, but they still
cost transfer and parse time. Fixing it properly means a `fields`
parameter on the plan endpoint (`backend/app/api/plan.py`), which is not
in this change's remit.

### The physiology screen is still 10.1–10.6 MB, and it is not the data

The heap snapshot (`scripts/measure-heap.mjs /physiology`) says where the
10.6 MB is:

```
3.58 MB  array          3.22 MB of it is the storage behind objects' properties
3.05 MB  code           1.32 MB of instruction stream, 0.38 MB of bytecode
3.03 MB  native         1.16 MB of external string data (the chunk's source text)
1.65 MB  object         0.80 MB of plain objects
0.54 MB  number
0.48 MB  string
```

The trimmed payload moved this screen's heap by 0.06 MB — inside the
run-to-run noise. The screen's weight is **Recharts' internal state and
the compiled chart bundle**, not the query data: 3.2 MB of array storage
behind the chart library's own objects, 3 MB of V8 code for the 408 KB
Recharts chunk, and 1.2 MB of that chunk's source text kept as a string.

The only lever there is drawing fewer points, which changes what the
athlete sees — the readiness trend, the sleep stages and the HRV line are
drawn from the full series. Not pulled.

### The activity detail keeps its streams

`["activity",162]` is 78 KB of 1 Hz streams. The three stream charts draw
them. Trimming them would mean drawing a sampled track.

### `StrictMode` is not costing anything in production

`StrictMode` is on in `main.tsx`, but React only double-invokes effects in
a development build. The built bundle contains no double-render path
(`grep -c "renderWithHooks" frontend/dist/assets/react-*.js` → 0), and
every number in this document was taken against `frontend/dist`, served
from the same origin as production. No measurement here is of a
development build.

### Recharts has no hidden measurement container

Every `.recharts-wrapper` on every screen has a real width and height and
one `<svg>` surface. There is nothing mounted off-screen.

### The query cache is still bounded by the payloads it holds

`gcTime` of 2 minutes bounds how *long* the cache keeps things; the
payload trims bound how *big* they are. The largest entry left is the
activity's own streams (78 KB), which the screen draws.

---

## 7. What the athlete will notice

Three things changed that are visible, and one that is not:

1. **The neighbour links on the activity page arrive one request later.**
   The window fetch needs the activity's date, so it waits for the
   activity. The two links are the same two runs.
2. **Coming back to a screen after two minutes shows a brief skeleton**
   instead of the stale data it used to show while it refetched. The data
   shown is the same; it arrives a moment later.
3. **The service worker's cache holds one build, not every build.**
   Nothing is visible; the offline shell is unaffected.
4. Everything else is invisible: the same screens, the same numbers, the
   same charts.

Verified after the change: every route renders with no skeletons left, no
console errors, the physiology screen draws its strip, its trend, its
sleep stages and its ten-row table, the activity screen draws its map, its
four polylines and both neighbour links, and the plan and running-form
screens render their content. `./scripts/security_check.sh`: 27 passed, 0
failed. `npm run typecheck`: clean.

---

## 8. The numbers in one place

| | Before | After |
|---|---|---|
| Activity screen, JS heap | 9.16 MB | **7.19 MB** |
| Activity screen, query cache | 638 KB | **86 KB** |
| Activity screen, API transfer | 790 KB | **277 KB** |
| Physiology screen, query cache | 474 KB | **145 KB** |
| Physiology screen, API transfer | 487 KB | **151 KB** |
| Plan screen, query cache | 129 KB | **35 KB** |
| Fitness screen, query cache | 95 KB | **48 KB** |
| API bytes, ten screens | 1674 KB | **825 KB** |
| **Session, final JS heap** | 8.52 MB | **7.51 MB** |
| **Session, query cache** | **1346 KB** | **405 KB** |
| Service worker cache, two builds | 925 KB / 24 entries | **792 KB / 15 entries** |
| Map tiles decoded | 1024 KB | 1024 KB (unchanged — see §6) |

The biggest lever was the query cache: **941 KB of the 1.0 MB the session
heap fell by**, and 552 KB of the 1.97 MB the activity screen fell by. The
decoded image cache was the biggest single number in the renderer (1 MB of
tiles, about 2 MB with the GPU copy) and it is the one number that did not
move — it is the price of drawing a map, and it is released the moment the
athlete leaves the route.
