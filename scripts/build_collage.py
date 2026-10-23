#!/usr/bin/env python3
"""Build the showcase collage.

Renders an HTML page that frames the product's screenshots, then prints the
path it should be screenshotted at. Run `scripts/showcase.mjs` first to
produce the shots.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SHOTS = ROOT / "showcase" / "shots"
OUT = ROOT / "showcase" / "collage.html"

PANELS = [
    {
        "file": "01-dashboard.png",
        "title": "Today",
        "caption": "The plan, the readiness and one recommendation — before you lace up.",
        "span": "wide",
    },
    {
        "file": "03-activity.png",
        "title": "Every run, in full",
        "caption": "Real GPS track, per-second heart rate, and splits rebuilt over the running only.",
        "span": "wide",
    },
    {
        "file": "02-plan.png",
        "title": "Plan vs actual",
        "caption": "Your 20-week plan against your real runs — every session scored, with a reason for every miss.",
    },
    {
        "file": "04-physiology.png",
        "title": "The physiological picture",
        "caption": "Sleep, resting heart rate, stress and body battery, rolled into a readiness score you can see the drivers of.",
    },
    {
        "file": "05-fingerprint.png",
        "title": "Your running fingerprint",
        "caption": "Cadence stability, heart-rate drift, aerobic decoupling and form — across every run you have done.",
    },
    {
        "file": "08-compare.png",
        "title": "Compare any two runs",
        "caption": "Both resampled onto one distance axis, so you can see whether you actually got fitter.",
    },
    {
        "file": "06-training.png",
        "title": "Training log",
        "caption": "Volume, streaks and the plan, week by week.",
    },
    {
        "file": "07-feed.png",
        "title": "Your history",
        "caption": "Every run, with its route.",
    },
]

CHIPS = [
    ("103", "real activities synced"),
    ("15", "parameters per run"),
    ("0", "bytes to a cloud you don't own"),
    ("24 h", "access token, auto-refreshed"),
]

CSS = """
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap');
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: Inter, -apple-system, 'Segoe UI', Roboto, sans-serif;
  background: #0d0d0f;
  color: #f2f2f3;
  -webkit-font-smoothing: antialiased;
}
.sheet { width: 1760px; padding: 56px 56px 44px; }

/* ---- header ---- */
.head { display: flex; align-items: flex-end; justify-content: space-between; gap: 40px; }
.brand { display: flex; align-items: center; gap: 14px; }
.mark {
  width: 52px; height: 52px; border-radius: 14px; background: #fc5200;
  display: grid; place-items: center;
}
.mark svg { width: 28px; height: 28px; }
.wordmark { font-size: 30px; font-weight: 800; letter-spacing: -.02em; }
.wordmark span { color: #fc5200; }
.tagline {
  font-size: 16px; color: #9aa0a6; margin-top: 10px; max-width: 760px;
  line-height: 1.55;
}
.tagline b { color: #e8e8ea; font-weight: 600; }
.pitch {
  font-size: 34px; font-weight: 800; line-height: 1.18; letter-spacing: -.025em;
  margin-top: 26px; max-width: 1180px;
}
.pitch em { font-style: normal; color: #fc5200; }

/* ---- chips ---- */
.chips { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; width: 340px; flex: none; }
.chip {
  background: #17171a; border: 1px solid #26262b; border-radius: 10px;
  padding: 11px 15px;
}
.chip b { display: block; font-size: 19px; font-weight: 700; color: #fff; letter-spacing: -.01em; }
.chip span { font-size: 11px; color: #8a8f95; letter-spacing: .02em; }

/* ---- grid ---- */
.grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 26px; margin-top: 44px; }
.panel { background: #141416; border: 1px solid #24242a; border-radius: 14px; overflow: hidden; display: flex; flex-direction: column; }
.panel.wide { grid-column: span 3; }
.shot { width: 100%; display: block; background: #fff; }
.panel:not(.wide) .shot { height: 430px; object-fit: cover; object-position: top center; }
.panel.wide .shot { max-height: 545px; object-fit: cover; object-position: top center; }
.cap { padding: 16px 18px 18px; }
.cap h3 { font-size: 15px; font-weight: 700; letter-spacing: -.01em; }
.cap p { font-size: 12.5px; color: #8a8f95; margin-top: 5px; line-height: 1.5; }

/* ---- footer ---- */
.foot {
  margin-top: 34px; padding-top: 22px; border-top: 1px solid #24242a;
  display: flex; align-items: center; justify-content: space-between; gap: 30px;
}
.stack { font-size: 12px; color: #6f757c; line-height: 1.7; }
.stack b { color: #b9bec4; font-weight: 600; }
.note { font-size: 12px; color: #6f757c; text-align: right; line-height: 1.7; }
"""


def main() -> None:
    panels = []
    for panel in PANELS:
        path = SHOTS / panel["file"]
        if not path.exists():
            raise SystemExit(f"missing screenshot: {path}")
        panels.append(
            f"""      <figure class="panel {panel.get('span', '')}">
        <img class="shot" src="shots/{panel['file']}" alt="{panel['title']}" />
        <figcaption class="cap">
          <h3>{panel['title']}</h3>
          <p>{panel['caption']}</p>
        </figcaption>
      </figure>"""
        )

    chips = "\n".join(
        f"""      <div class="chip"><b>{value}</b><span>{label}</span></div>"""
        for value, label in CHIPS
    )

    html = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8" />
<title>Stride — the training intelligence layer for Garmin</title>
<style>{CSS}</style></head>
<body>
  <div class="sheet">
    <header class="head">
      <div class="brand">
        <div class="mark">
          <svg viewBox="0 0 32 32" fill="none" stroke="#fff" stroke-width="2.6" stroke-linejoin="round" stroke-linecap="round">
            <path d="M5 24l7-12 5 7 4-6 6 11" />
          </svg>
        </div>
        <div class="wordmark">STRIDE</div>
      </div>
      <div class="chips">
{chips}
      </div>
    </header>

    <p class="tagline">
      A private training log for runners who want to know <b>what to do next</b> — not
      just what they did. Your Garmin watch writes to your own database. Nothing else.
    </p>

    <h1 class="pitch">
      Strava shows you what you did.<br />
      Stride tells you <em>whether that was right</em> — and what to do tomorrow.
    </h1>

    <div class="grid">
{chr(10).join(panels)}
    </div>

    <footer class="foot">
      <div class="stack">
        <b>Stack</b> — React + TypeScript + Tailwind + Recharts + Leaflet · FastAPI +
        SQLAlchemy + SQLite · Garmin Connect <b>Personal API</b> (OAuth-free, one key)
        · Azure Key Vault for secrets
      </div>
      <div class="note">
        Plan → Execute → Understand → Adapt<br />
        Self-hosted · local-first · your data stays yours
      </div>
    </footer>
  </div>
</body></html>"""

    OUT.write_text(html)
    print(OUT)


if __name__ == "__main__":
    main()
