# Showcase

The screenshots of the app as it is now, and the palette comparison that
chose the colours.

- **`stride-collage.png`** — the shareable collage (3520×6168, 2× device scale).
  Open it, drop it in a deck, or send it to someone.
- **`stride-collage@1x.png`** — the same collage at 1760×3084. Use this
  for chat, email or a slide.
- **`stride-collage.jpg`** — progressive JPEG, for anywhere that resists
  a large PNG.
- **`collage.html`** — the source of the collage. Open it in a browser to read
  it as a page, or print it to PDF.
- **`shots/`** — the individual screenshots, captured from the running app:
  Today, Plan, an activity, Body, Running form, the training log, Runs,
  Compare and Progress.
- **`review/`** — every screen on **both** hosts, 18 of them, against the real
  data: ten desktop and eight phone. The phone set is the four destinations
  plus the plan, the form, the statistics, an activity and the settings, so
  **Compare** and the **training calendar** are absent — that is the cut, and
  the sheet says so.
- **`palette-lab/`** — the five candidate palettes, side by side.

**The promise is on the collage**, in the tagline, in one line:

> **Your data stays on this phone.**

## Regenerating the shots and the collage

```bash
cd ~/stride
node scripts/showcase.mjs http://127.0.0.1:8000   # capture the screenshots
python3 scripts/build_collage.py                   # write collage.html
node scripts/render_collage.mjs                    # render collage.html to PNG/JPEG
node scripts/review.mjs http://127.0.0.1:8000 showcase/review   # every screen, both hosts
```

`scripts/showcase.mjs` drives the live app with headless Chromium, including
the interactive states (the comparison page needs runs selected).

**Two things to know before you re-run it.**

1. **The activity page needs about fifteen seconds.** The map tiles come
   through the server's tile proxy, and the first fetch of a tile is a round
   trip. A shot taken at three seconds is a blank page with a spinner in it.
2. **The app has to be running with the athlete's data** for the shots to
   mean anything: a review against invented data tells you nothing about
   density, legibility or the empty states. The screenshots in here are
   the athlete's own history.

   **The live database is not touched.** Copy it first and point the app at
   the copy:

   ```bash
   mkdir -p /tmp/stride-showcase/data && chmod 700 /tmp/stride-showcase/data
   cp backend/data/{stride.db,state.json,stride-desktop.key} /tmp/stride-showcase/data/
   STRIDE_DATA_DIR=/tmp/stride-showcase/data \
   STRIDE_DB_PATH=/tmp/stride-showcase/data/stride.db \
   STRIDE_STATE_PATH=/tmp/stride-showcase/data/state.json \
   STRIDE_FRONTEND_DIST=$PWD/frontend/dist \
   ./target/release/stride serve
   ```

   The app converts the copy into an encrypted database in that directory;
   the original is left exactly as it was.

3. **The pages wait for the DOM, not for the network.** The map tiles come
   through the server's proxy and the first fetch of one is a round trip to
   the tile host, so "the network is idle" can time out on a page with a
   map on it.

## The palette lab

`palette-lab/` is the record of the colour decision: five candidate
palettes, the same pages in each, and the comparison sheet.

**It has not been regenerated**, and it should not be run against the live
tree: `scripts/palette_lab.py` rewrites `frontend/src/lib/palette.ts` — which
today holds the `METRIC` colour language the design review introduced — and
`frontend/tailwind.config.js`, and restores both at the end. Run it with
another agent editing the frontend, it can lose their work; the activity
page alone takes about three minutes per candidate, because of the map
tiles.

The decision it records still stands: **azure and amber, and the metric
language**. `docs/design-review.md` §2 has the reasoning.
