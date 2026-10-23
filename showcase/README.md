# Showcase

- **`stride-collage.png`** — the shareable collage (3520×5904, 2× device scale).
  Open it, drop it in a deck, or send it to someone.
- **`stride-collage@1x.png`** — the same collage at 1760×2952 (1.9 MB). Use this
  for chat, email or a slide.
- **`stride-collage.jpg`** — 1.6 MB progressive JPEG, for anywhere that resists
  a large PNG.
- **`collage.html`** — the source of the collage. Open it in a browser to read
  it as a page, or print it to PDF.
- **`shots/`** — the individual screenshots, captured from the running app.

## Regenerating

```bash
cd ~/stride
node scripts/showcase.mjs          # capture the screenshots
python3 scripts/build_collage.py   # write collage.html
node -e "…"                        # render collage.html to stride-collage.png
```

`scripts/showcase.mjs` drives the live app with headless Chromium, including
the interactive states (the comparison page needs runs selected).
