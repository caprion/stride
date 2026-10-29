# Showcase

The screenshots of the app as it is now, and the palette comparison that
chose the colours.

- **`stride-collage.png`** — the shareable collage (3520×5904, 2× device scale).
  Open it, drop it in a deck, or send it to someone.
- **`stride-collage@1x.png`** — the same collage at 1760×2952. Use this
  for chat, email or a slide.
- **`stride-collage.jpg`** — progressive JPEG, for anywhere that resists
  a large PNG.
- **`collage.html`** — the source of the collage. Open it in a browser to read
  it as a page, or print it to PDF.
- **`shots/`** — the individual screenshots, captured from the running app.
- **`review/`** — every screen on **both** hosts, 18 of them, against the real
  data: ten desktop and eight phone. The phone set is the six destinations
  plus an activity and the settings, so **Compare** and the **training
  calendar** are absent — that is the cut, and the sheet says so.
- **`palette-lab/`** — the five candidate palettes, side by side.

**The promise is on the collage**, in the tagline, in one line:

> **Your data stays on this phone.**

## Regenerating the shots and the collage

```bash
cd ~/stride
node scripts/showcase.mjs http://127.0.0.1:8000   # capture the screenshots
python3 scripts/build_collage.py                   # write collage.html
node /tmp/collage_shot.mjs                         # render collage.html to PNG/JPEG
```

`scripts/showcase.mjs` drives the live app with headless Chromium, including
the interactive states (the comparison page needs runs selected).

**Two things to know before you re-run it.**

1. **The activity page needs about fifteen seconds.** The map tiles come
   through the server's tile proxy, and the first fetch of a tile is a round
   trip. A shot taken at three seconds is a blank page with a spinner in it.
2. **`scripts/build_collage.py` overwrites `collage.html`, and it still holds
   the old copy** — the chip that said "103 real activities synced", the
   stack line that named FastAPI, and the footer that said "your data stays
   yours". The copy in this directory is the corrected one; the generator
   needs the same three edits (it is in `scripts/`, which the documentation
   workstream does not own).

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
