#!/usr/bin/env python3
"""Compose the palette candidates into one comparison sheet."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SHOTS = ROOT / "showcase" / "palette-lab"
OUT = ROOT / "showcase" / "palette-lab"

LABELS = {
    "azure-amber": ("Azure & Amber", "Sky, sun and gravity. The current palette."),
    "indigo-coral": ("Indigo & Coral", "Night running. Electric, high contrast, a little restless."),
    "forest-citron": ("Forest & Citron", "Off-road. Unusual, grown rather than engineered."),
    "slate-cyan": ("Slate & Cyan", "A data instrument. Cold, precise, unfussy."),
    "plum-gold": ("Plum & Gold", "Warm dark. Premium, a little theatrical."),
}

ORDER = ["azure-amber", "indigo-coral", "forest-citron", "slate-cyan", "plum-gold"]

SWATCHES = {
    "azure-amber": ["#1E5E96", "#124A7C", "#0D3A63", "#F2B01E", "#D99A0B", "#B57C06", "#2E8B7A", "#6E9E3C", "#C0392B"],
    "indigo-coral": ["#4338CA", "#3730A3", "#2A2484", "#FB7185", "#F43F5E", "#E11D48", "#0EA5A4", "#65A30D", "#DC2626"],
    "forest-citron": ["#166534", "#14532D", "#103C22", "#B8D62E", "#9BBF1A", "#7FA00F", "#22C55E", "#84CC16", "#DC2626"],
    "slate-cyan": ["#334155", "#1E293B", "#0F172A", "#22D3EE", "#06B6D4", "#0891B2", "#06B6D4", "#22C55E", "#DC2626"],
    "plum-gold": ["#6D28D9", "#5B21B6", "#4C1D95", "#E0A82E", "#C9931F", "#B07C15", "#14B8A6", "#84CC16", "#DC2626"],
}

CSS = """
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap');
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: Inter, -apple-system, sans-serif; background: #081420; color: #eef2f7; padding: 44px; }
h1 { font-size: 32px; font-weight: 800; letter-spacing: -.02em; }
.lede { font-size: 15px; color: #8fa3b6; margin-top: 8px; max-width: 780px; line-height: 1.6; }
.grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 30px; margin-top: 32px; }
.card { background: #0f1e2c; border: 1px solid #1d3145; border-radius: 14px; overflow: hidden; }
.card.wide { grid-column: span 2; }
.head { display: flex; align-items: center; justify-content: space-between; gap: 18px; padding: 16px 18px; border-bottom: 1px solid #1d3145; }
.name { font-size: 17px; font-weight: 700; }
.note { font-size: 12.5px; color: #8fa3b6; margin-top: 3px; }
.sw { display: flex; gap: 4px; }
.sw i { width: 17px; height: 17px; border-radius: 4px; display: block; }
.shot { width: 100%; display: block; }
.foot { padding: 12px 18px; font-size: 12px; color: #6b8098; }
"""


def main() -> None:
    cards = []
    for name in ORDER:
        label, note = LABELS[name]
        swatches = "".join(f'<i style="background:{c}"></i>' for c in SWATCHES[name])
        dashboard = SHOTS / f"{name}-dashboard.png"
        activity = SHOTS / f"{name}-activity.png"
        if not dashboard.exists():
            continue
        cards.append(f"""      <div class="card">
        <div class="head">
          <div><div class="name">{label}</div><div class="note">{note}</div></div>
          <div class="sw">{swatches}</div>
        </div>
        <img class="shot" src="{name}-dashboard.png" />
        <div class="foot">Dashboard — the Today card, hero stats, weekly volume.</div>
      </div>
      <div class="card">
        <div class="head">
          <div><div class="name">{label} · activity</div><div class="note">{note}</div></div>
          <div class="sw">{swatches}</div>
        </div>
        <img class="shot" src="{name}-activity.png" />
        <div class="foot">Activity — map, stat strip, running-only splits.</div>
      </div>""")

    html = f"""<!doctype html>
<html><head><meta charset="utf-8" /><title>Stride — palette candidates</title>
<style>{CSS}</style></head><body>
  <h1>Five palettes, one product.</h1>
  <p class="lede">Every candidate keeps the same structure: <b>brand</b> carries the app
  (chrome, primary actions, gravity) and <b>accent</b> carries the athlete's data
  (excitement, energy), with a natural family for the semantic series. The data,
  the pages and the layout are identical — only the colour changes.</p>
  <div class="grid">
{chr(10).join(cards)}
  </div>
</body></html>"""

    page = OUT / "palette-lab.html"
    page.write_text(html)

    node = r"""
const { chromium } = require('/home/sumit/node_modules/playwright');
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 1700, height: 1200 }, deviceScaleFactor: 2 });
  await p.goto('file://%s', { waitUntil: 'networkidle' });
  await p.waitForTimeout(2500);
  await p.screenshot({ path: '%s', fullPage: true });
  await b.close();
})();
""" % (page, OUT / "palette-lab.png")
    subprocess.run(["node", "-e", node], check=True)
    print(f"wrote {OUT / 'palette-lab.png'}")


if __name__ == "__main__":
    main()
