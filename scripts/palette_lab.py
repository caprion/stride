#!/usr/bin/env python3
"""Render the app in several palettes so they can be compared side by side.

    python3 scripts/palette_lab.py            # render the candidates
    python3 scripts/palette_lab.py --keep X   # leave palette X in place

Each candidate rewrites the Tailwind scale and the shared palette module,
rebuilds, and screenshots the dashboard and an activity page. The original
palette is restored at the end unless --keep says otherwise.
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FRONTEND = ROOT / "frontend"
SHOTS = ROOT / "showcase" / "palette-lab"

# name -> (brand 500/600/700/50, accent 400/500/600/50, ink, surface, edge, ramp, zones)
CANDIDATES: dict[str, dict] = {
    "azure-amber": {
        "label": "Azure & Amber",
        "note": "Sky, sun and gravity. The current palette.",
        "brand": ["#1E5E96", "#124A7C", "#0D3A63", "#EEF5FB"],
        "accent": ["#F2B01E", "#D99A0B", "#B57C06", "#FEF8EA"],
        "ink": "#0B1B27",
        "soft": "#F4F7FA",
        "sunken": "#E9EFF4",
        "edge": "#DEE7EE",
        "ramp": ["#F2B01E", "#E08A3C", "#B4557F", "#6E6BA8", "#3F6E9E", "#1E5E96"],
        "zones": ["#A8BCCB", "#2E8B7A", "#6E9E3C", "#D99A0B", "#C0392B"],
    },
    "indigo-coral": {
        "label": "Indigo & Coral",
        "note": "Night running. Electric, high contrast, a little restless.",
        "brand": ["#4338CA", "#3730A3", "#2A2484", "#EEF0FF"],
        "accent": ["#FB7185", "#F43F5E", "#E11D48", "#FFF1F3"],
        "ink": "#14142B",
        "soft": "#F5F5FB",
        "sunken": "#EAEAF5",
        "edge": "#DFDFF0",
        "ramp": ["#FB7185", "#F97316", "#C2410C", "#9A3412", "#7C3AED", "#4338CA"],
        "zones": ["#B4B4CC", "#0EA5A4", "#65A30D", "#F59E0B", "#DC2626"],
    },
    "forest-citron": {
        "label": "Forest & Citron",
        "note": "Off-road. Unusual, grown rather than engineered.",
        "brand": ["#166534", "#14532D", "#103C22", "#EEF7F0"],
        "accent": ["#B8D62E", "#9BBF1A", "#7FA00F", "#F6FBE8"],
        "ink": "#10231A",
        "soft": "#F3F8F4",
        "sunken": "#E7F0E9",
        "edge": "#D8E7DC",
        "ramp": ["#B8D62E", "#84CC16", "#22C55E", "#0D9488", "#0F766E", "#166534"],
        "zones": ["#9CB8A4", "#22C55E", "#84CC16", "#EAB308", "#DC2626"],
    },
    "slate-cyan": {
        "label": "Slate & Cyan",
        "note": "A data instrument. Cold, precise, unfussy.",
        "brand": ["#334155", "#1E293B", "#0F172A", "#F1F5F9"],
        "accent": ["#22D3EE", "#06B6D4", "#0891B2", "#ECFEFF"],
        "ink": "#0F172A",
        "soft": "#F5F7FA",
        "sunken": "#E9EEF3",
        "edge": "#DCE3EA",
        "ramp": ["#22D3EE", "#38BDF8", "#818CF8", "#6366F1", "#475569", "#1E293B"],
        "zones": ["#94A3B8", "#06B6D4", "#22C55E", "#EAB308", "#DC2626"],
    },
    "plum-gold": {
        "label": "Plum & Gold",
        "note": "Warm dark. Premium, a little theatrical.",
        "brand": ["#6D28D9", "#5B21B6", "#4C1D95", "#F5F0FF"],
        "accent": ["#E0A82E", "#C9931F", "#B07C15", "#FDF7E7"],
        "ink": "#1B1030",
        "soft": "#F8F5FC",
        "sunken": "#EFE9F7",
        "edge": "#E3DAF0",
        "ramp": ["#E0A82E", "#D97706", "#B45309", "#9333EA", "#6D28D9", "#4C1D95"],
        "zones": ["#AFA0C9", "#14B8A6", "#84CC16", "#E0A82E", "#DC2626"],
    },
}


DEFAULTS = {"grid": "#E4EBF0", "muted": "#61788D", "faint": "#93A6B7", "planned": "#C7D6E3"}
for _spec in CANDIDATES.values():
    for _key, _value in DEFAULTS.items():
        _spec.setdefault(_key, _value)


def scale(base: str) -> dict[str, str]:
    """Build a 50-900 Tailwind scale from one base colour."""
    return {
        "50": base[3],
        "100": base[3],
        "200": base[3],
        "300": base[3],
        "400": base[0],
        "500": base[0],
        "600": base[1],
        "700": base[2],
        "800": base[2],
        "900": base[2],
    }


def write_tailwind(spec: dict) -> None:
    brand = scale(spec["brand"])
    accent = scale(spec["accent"])
    config = f"""/** @type {{import('tailwindcss').Config}} */
export default {{
  content: ['./index.html', './src/**/*.{{ts,tsx}}'],
  theme: {{
    extend: {{
      colors: {{
        brand: {{
          50: '{brand['50']}', 100: '{brand['100']}', 200: '{brand['200']}', 300: '{brand['300']}',
          400: '{brand['400']}', 500: '{brand['500']}', 600: '{brand['600']}', 700: '{brand['700']}',
          800: '{brand['800']}', 900: '{brand['900']}',
        }},
        accent: {{
          50: '{accent['50']}', 100: '{accent['100']}', 200: '{accent['200']}', 300: '{accent['300']}',
          400: '{accent['400']}', 500: '{accent['500']}', 600: '{accent['600']}', 700: '{accent['700']}',
          800: '{accent['800']}', 900: '{accent['900']}',
        }},
        ink: {{ DEFAULT: '{spec['ink']}', soft: '#334155', muted: '#64748B', faint: '#94A3B8' }},
        surface: {{ DEFAULT: '#FFFFFF', soft: '{spec['soft']}', raised: '#FAFCFD', sunken: '{spec['sunken']}' }},
        edge: '{spec['edge']}',
        positive: '{spec['zones'][1]}',
        negative: '{spec['zones'][4]}',
        warn: '{spec['zones'][3]}',
        ramp: {json.dumps(spec['ramp'])},
      }},
      fontFamily: {{
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'sans-serif'],
        mono: ['"SF Mono"', 'ui-monospace', 'Menlo', 'Consolas', 'monospace'],
      }},
      boxShadow: {{
        card: '0 1px 2px rgba(0,0,0,0.04), 0 1px 3px rgba(0,0,0,0.06)',
        lift: '0 4px 16px rgba(0,0,0,0.10)',
        nav: '0 1px 0 rgba(0,0,0,0.08)',
      }},
      borderRadius: {{ card: '10px' }},
      keyframes: {{
        'fade-up': {{ '0%': {{ opacity: '0', transform: 'translateY(6px)' }}, '100%': {{ opacity: '1', transform: 'translateY(0)' }} }},
        shimmer: {{ '100%': {{ transform: 'translateX(100%)' }} }},
      }},
      animation: {{ 'fade-up': 'fade-up 0.28s ease-out both' }},
    }},
  }},
  plugins: [],
}}
"""
    (FRONTEND / "tailwind.config.js").write_text(config)


def write_palette(spec: dict) -> None:
    brand, accent = spec["brand"], spec["accent"]
    text = f"""/**
 * The palette, in one place.
 *
 * - **Brand** - structure, gravity, primary actions, chrome.
 * - **Accent** - excitement, energy, the athlete's own data.
 * - A natural family carries the semantic series.
 */

export const BRAND = {{
  50: '{brand[3]}', 100: '{brand[3]}', 200: '{brand[3]}', 300: '{brand[3]}',
  400: '{brand[0]}', 500: '{brand[0]}', 600: '{brand[1]}', 700: '{brand[2]}',
  800: '{brand[2]}', 900: '{brand[2]}',
}} as const

export const ACCENT = {{
  50: '{accent[3]}', 100: '{accent[3]}', 200: '{accent[3]}', 300: '{accent[3]}',
  400: '{accent[0]}', 500: '{accent[1]}', 600: '{accent[2]}', 700: '{accent[2]}',
  800: '{accent[2]}', 900: '{accent[2]}',
}} as const

/** The data colours, named rather than indexed. */
export const ACCENT_DATA = ACCENT[400]
export const ACCENT_WARM = ACCENT[500]
export const ACCENT_DEEP = ACCENT[600]
export const ACCENT_SOFT = ACCENT[50]

/** Chart furniture: grid lines and the two muted text tones. */
export const GRID = '{spec['grid']}'
export const INK_MUTED = '{spec['muted']}'
export const INK_FAINT = '{spec['faint']}'
export const SURFACE_SUNKEN = '{spec['sunken']}'

export const RAMP = {json.dumps(spec['ramp'])} as const

export const ZONE_COLORS = {json.dumps(spec['zones'])} as const

export const SERIES = {{
  /** The paler tint behind the most recent bar. */
  actualSoft: ACCENT[200],
  /** A cool grey for planned volume and de-emphasised furniture. */
  plannedSoft: '{spec['planned']}',
  heartRate: '{spec['ramp'][-1]}',
  pace: '{accent[2]}',
  cadence: '{spec['zones'][1]}',
  elevation: '{spec['ramp'][3]}',
  planned: '{spec['planned']}',
  actual: '{accent[0]}',
  accent: '{accent[0]}',
}} as const

export const VERDICT_COLORS = {{
  hit: '{spec['zones'][1]}',
  missed_pace: '{spec['zones'][3]}',
  short: '{spec['zones'][3]}',
  missed: '{spec['zones'][4]}',
  substituted: '{spec['ramp'][1]}',
  bonus: '{accent[0]}',
}} as const

export const SPORT_COLORS: Record<string, string> = {{
  running: '{spec['ramp'][-1]}',
  trail_running: '{spec['zones'][1]}',
  treadmill_running: '{spec['ramp'][3]}',
  virtualrun: '{spec['ramp'][1]}',
  walking: '{spec['zones'][2]}',
  hiking: '{spec['zones'][3]}',
  cycling: '{spec['zones'][1]}',
  swimming: '{spec['ramp'][1]}',
  strength_training: '#7E8FA0',
}}
"""
    (FRONTEND / "src" / "lib" / "palette.ts").write_text(text)


def main() -> None:
    keep = None
    if "--keep" in sys.argv:
        keep = sys.argv[sys.argv.index("--keep") + 1]
    if keep not in CANDIDATES:
        keep = None

    SHOTS.mkdir(parents=True, exist_ok=True)
    backup = Path("/tmp/stride-palette-backup")
    if backup.exists():
        shutil.rmtree(backup)
    backup.mkdir(parents=True)
    shutil.copy(FRONTEND / "tailwind.config.js", backup / "tailwind.config.js")
    shutil.copy(FRONTEND / "src" / "lib" / "palette.ts", backup / "palette.ts")

    try:
        for name, spec in CANDIDATES.items():
            print(f"→ {name}: {spec['label']}")
            write_tailwind(spec)
            write_palette(spec)
            subprocess.run(["npm", "run", "build"], cwd=FRONTEND, check=True,
                           capture_output=True)
            subprocess.run(
                ["node", str(ROOT / "scripts" / "palette_shot.mjs"), name, str(SHOTS)],
                check=True,
            )
    finally:
        if keep is None:
            shutil.copy(backup / "tailwind.config.js", FRONTEND / "tailwind.config.js")
            shutil.copy(backup / "palette.ts", FRONTEND / "src" / "lib" / "palette.ts")
            print("restored the original palette")
        else:
            print(f"left '{keep}' in place — rebuild to apply")
    print(f"\nshots in {SHOTS}")


if __name__ == "__main__":
    main()
