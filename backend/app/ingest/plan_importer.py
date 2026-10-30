"""Parse athlete-authored plans into a format-independent plan tree.

A row whose text names no kind of session is imported as ``unknown``: the
document did not say what kind of session it was, and the importer does not
say so either.
"""
from __future__ import annotations

import csv
import json
import re
from datetime import date, timedelta
from io import StringIO
from pathlib import Path
from typing import Any


def parse_pace(value: str | None) -> tuple[int | None, int | None]:
    """Convert written min:sec paces to seconds/km while preserving absent data."""
    if not value:
        return None, None
    pace_text = re.sub(r"\s*/\s*km\b", "", value, flags=re.I)
    found = re.findall(r"(?<!\d)(\d{1,2}:\d{2})(?!\d)", pace_text)
    if not found:
        return None, None
    nums = [int(x.split(":")[0]) * 60 + int(x.split(":")[1]) for x in found[:2]]
    return (min(nums), max(nums)) if len(nums) == 2 else (nums[0], nums[0])


def parse_distance(value: str | None) -> float | None:
    if not value:
        return None
    m = re.search(r"(?:~\s*)?(\d+(?:\.\d+)?)\s*(?:-\s*\d+(?:\.\d+)?)?\s*km\b", value, re.I)
    if m: return float(m.group(1)) * 1000
    m = re.search(r"\b(\d+(?:\.\d+)?)\s*k\b", value, re.I)
    if m: return float(m.group(1))*1000
    if re.search(r"half\s+marathon", value, re.I): return 21097.5
    return None


def _clean(s: str) -> str:
    """Strip markdown emphasis and emoji from a table cell.

    Cells can be empty or missing entirely in a malformed document, so every
    input is tolerated.
    """
    if not s:
        return ""
    s = re.sub(r"\*\*|__|[*_]", "", s)
    s = re.sub(r"[⭐⚠🏁✅🔴🟡🟢]", "", s)
    return s.replace("→", " to ").replace("—", "-").strip()


def _tables(text: str) -> list[list[list[str]]]:
    tables, rows = [], []
    for line in text.splitlines():
        if "|" not in line:
            if rows: tables.append(rows); rows = []
            continue
        cells = [_clean(x.strip()) for x in line.strip().strip("|").split("|")]
        if all(re.fullmatch(r"[-: ]+", c or "-") for c in cells):
            continue
        rows.append(cells)
    if rows: tables.append(rows)
    return tables


def _phase_key(s: str) -> str:
    s = re.sub(r"\([^)]*\)", "", _clean(s or ""))
    return re.sub(r"\s+", " ", s).strip().lower()


def _week_date(heading: str, year: int) -> tuple[str, str] | None:
    m = re.search(r"\((\w{3,9})\s+(\d{1,2})\s*-\s*(?:(\w{3,9})\s+)?(\d{1,2})\)", heading)
    if not m: return None
    months = {x.lower(): i for i, x in enumerate(("January February March April May June July August September October November December").split(), 1)}
    mon1, d1, mon2, d2 = m.groups(); a = months.get(mon1.lower()); b = months.get((mon2 or mon1).lower())
    if not a or not b: return None
    y2 = year + (1 if b < a else 0)
    return date(year, a, int(d1)).isoformat(), date(y2, b, int(d2)).isoformat()


def _classify(title: str, details: str) -> str:
    s = (title + " " + details).lower()
    t = title.lower()
    if "rest" in t:
        # "Rest or 2km walk" is not a rest day — it is a light recovery
        # session with an optional distance. Calling it "rest" would then
        # report a distance target against a session that may not happen.
        if re.search(r"\b(or|walk|run|jog|shakeout)\b", t):
            return "recovery"
        return "rest"
    if "strength" in s or "session a" in s or "session b" in s:
        return "strength"
    if "race" in t:
        return "race"
    # Distance and the word "long" decide a long run before anything else.
    # "Long 14km easy" contains the word "easy" but it is still a long run.
    if "long" in s or (parse_distance(title) or 0) > 14000:
        return "long"
    if re.search(r"\d+\s*[×x]\s*\d", s) or any(
        x in s for x in ("interval", "fartlek", "sharpener")
    ):
        return "interval"
    if "easy" in t or "recovery" in t:
        return "easy"
    if any(x in s for x in ("tempo", "threshold", "steady")):
        return "tempo"
    if "recovery" in s:
        return "recovery"
    # Nothing in the row names a kind of session — "Session 4", "Zona 3" —
    # so the kind is unknown. It used to be "easy", which is a guess: the
    # plan then said the athlete had an easy run prescribed, and every
    # verdict, every pace band and every comparison downstream treated that
    # guess as the athlete's own words.
    return "unknown"


def _session(title: str, details: str, day: date, paces: dict[str, tuple[int | None, int | None]]) -> dict[str, Any]:
    kind = _classify(title, details)
    pace_source = title + " " + details
    race_pace = re.search(r"(\d{1,2}:\d{2})\s*/\s*km", pace_source, re.I) if kind == "race" else None
    if kind == "race" and not race_pace and "half marathon" in pace_source.lower():
        pace = (round(2*3600/21.0975), round((2*3600+3*60)/21.0975))
    else:
        pace = parse_pace(race_pace.group(1) if race_pace else pace_source) if kind != "race" or race_pace else (None, None)
    if pace == (None, None):
        # The phase's pace bands belong to the kinds they were written for.
        # An unknown session belongs to none of them, so it gets no band:
        # the easy band would be the same guess, one step later.
        group = None if kind == "unknown" else ("easy" if kind in ("easy", "recovery") else "long" if kind == "long" else "tempo" if kind == "tempo" else "interval" if kind == "interval" else "easy")
        pace = paces.get(group, (None, None)) if group else (None, None)
    rep = re.search(r"(\d+)\s*[×x]\s*(\d+(?:\.\d+)?)\s*(km|m)\b", title, re.I) if kind == "interval" else None
    rep_distance = (int(rep.group(1))*float(rep.group(2))*(1000 if rep.group(3).lower()=="km" else 1)) if rep else None
    distance = (parse_distance(details) or rep_distance) if rep else None
    return {"date": day.isoformat(), "session_type": kind, "title": title[:160], "details": details or None,
            "target_distance_m": distance or parse_distance(title) or parse_distance(details),
            "pace_low_s_km": pace[0], "pace_high_s_km": pace[1], "source": "imported"}


def import_plan(content: str, format: str = "markdown", name: str | None = None) -> dict[str, Any]:
    """Normalize markdown, CSV, or JSON plans; warnings expose unrepresented source content."""
    if format == "json":
        data = json.loads(content)
        return data if {"block", "phases", "weeks", "sessions"} <= data.keys() else _normalize_json(data, name)
    if format == "csv":
        items = list(csv.DictReader(StringIO(content)))
        return _normalize_json({"sessions": items, "name": name or "Imported plan"}, name)
    if format != "markdown": raise ValueError("format must be markdown, csv, or json")
    text = content
    year_match = re.search(r"\b(20\d{2})\b", text[:500])
    year = int(year_match.group(1)) if year_match else date.today().year
    phase_rows: dict[str, dict[str, Any]] = {}
    tables = _tables(text)
    for table in tables:
        if table[0] and _phase_key(table[0][0]) == "phase" and len(table[0]) >= 4 and "duration" in " ".join(table[0]).lower():
            for row in table[1:]:
                if len(row) < 4: continue
                m = re.search(r"(\d+)\s*-\s*(\d+)", row[1]); km = re.findall(r"\d+(?:\.\d+)?", row[3])
                phase_rows[_phase_key(row[0])] = {"name": row[0], "week_start": int(m.group(1)) if m else None,
                    "week_end": int(m.group(2)) if m else None, "duration": row[2],
                    "target_weekly_km_min": float(km[0]) if km else None, "target_weekly_km_max": float(km[-1]) if km else None}
        elif table[0] and _phase_key(table[0][0]) == "phase" and "easy" in " ".join(table[0]).lower():
            for row in table[1:]:
                if len(row) >= 5:
                    key = _phase_key(row[0]); p = phase_rows.setdefault(key, {"name": row[0]})
                    for dest, ix in (("easy",1),("tempo",2),("interval",3),("long",4)):
                        p[f"pace_{dest}_low"], p[f"pace_{dest}_high"] = parse_pace(row[ix])
    progression: dict[int, dict[str, Any]] = {}
    for t in tables:
        if t[0] and _phase_key(t[0][0]) == "week" and any("running" in h.lower() for h in t[0]):
            for row in t[1:]:
                try:
                    n = int(re.match(r"\d+", row[0]).group())
                except (ValueError, AttributeError):
                    continue
                # A malformed row must not abort the whole import. Every cell
                # is optional; whatever cannot be read becomes a warning.
                phase = row[1] if len(row) > 1 else None
                km = parse_distance(row[2]) if len(row) > 2 else None
                key_workout = row[4] if len(row) > 4 else None
                if km is None and len(row) > 2:
                    digits = re.search(r"\d+(?:\.\d+)?", str(row[2]))
                    km = float(digits.group()) if digits else None
                progression[n] = {
                    "phase": phase,
                    "target_km": (km / 1000) if km else None,
                    "key_workout": key_workout,
                }
    phases, weeks, sessions, warnings = [], [], [], []
    for i, p in enumerate(phase_rows.values()):
        dates = re.search(r"([A-Z][a-z]{2})\s+(\d+)\s*-\s*(?:([A-Z][a-z]{2})\s+)?(\d+)", p.get("duration", ""))
        if dates:
            d1 = _week_date(f"({dates.group(1)} {dates.group(2)}-{dates.group(3) or dates.group(1)} {dates.group(4)})", year)
        else: d1 = None
        p.update({"start_date": d1[0] if d1 else None, "end_date": d1[1] if d1 else None})
        p["_idx"] = i; phases.append(p)
    week_heading = None
    detail_rows: dict[int, list[tuple[str,str,str]]] = {}
    for line in text.splitlines():
        h = re.search(r"Week\s+(\d+)\s*[-—]\s*(.*?)\s*\(([^)]+)\).*?(\d+(?:\.\d+)?)\s*km", line, re.I)
        if h: week_heading = (int(h.group(1)), h.group(2)); continue
        if re.match(r"^\s*#{1,6}\s", line):
            week_heading = None
            continue
        if week_heading and "|" in line:
            cells = [_clean(x.strip()) for x in line.strip().strip("|").split("|")]
            if cells and cells[0] not in ("Day",) and not all(re.fullmatch(r"[-: ]+", c or "-") for c in cells):
                if len(cells)>=2: detail_rows.setdefault(week_heading[0], []).append((cells[0],cells[1],cells[2] if len(cells)>2 else ""))
    for n, prog in sorted(progression.items()):
        phase_name = prog["phase"]; key = _phase_key(phase_name)
        p = phase_rows.get(key) or next((x for k,x in phase_rows.items() if key.startswith(k) or k.startswith(key)), {})
        p = next((x for x in phase_rows.values() if x.get("week_start") is not None and x["week_start"] <= n <= x.get("week_end", n)), p)
        parent = p.get("_idx", 0); start = None
        if p.get("start_date") and p.get("week_start"):
            start = date.fromisoformat(p["start_date"]) + timedelta(weeks=n-p["week_start"])
        if start is None:
            # The date derives from the plan's declared week progression beginning Jan 5.
            start = date(year, 1, 5) + timedelta(weeks=n-1)
        end = start + timedelta(days=6)
        w = {"week_number": n, "start_date": start.isoformat(), "end_date": end.isoformat(), "target_km": prog["target_km"], "key_workout": prog.get("key_workout"), "phase_index": parent}
        weeks.append(w)
        rows = detail_rows.get(n, [])
        for dayname,title,detail in rows:
            days = {"Mon":0,"Tue":1,"Wed":2,"Thu":3,"Fri":4,"Sat":5,"Sun":6,"Sat/Sun":6}
            offset = days.get(dayname)
            if offset is None:
                warnings.append(f"Week {n}: could not map day {dayname!r}"); continue
            sessions.append(_session(title, detail, start+timedelta(days=offset), {k[5:]: (p.get(k+"_low"),p.get(k+"_high")) for k in ("pace_easy","pace_tempo","pace_interval","pace_long")}))
        if not rows:
            # Early weeks only provide a volume target; use the document's explicit weekly template.
            template = [(1,"Easy run", "6-8km"),(3,"Quality",prog.get("key_workout") or "Easy running"),(5,"Long run","10km"),(6,"Easy run","3-5km")]
            phase_paces = {k: (p.get("pace_"+k+"_low"), p.get("pace_"+k+"_high")) for k in ("easy","tempo","interval","long")}
            for off,title,detail in template:
                sessions.append(_session(title, detail, start+timedelta(days=off), phase_paces))
            warnings.append(f"Week {n}: day sessions generated from the documented weekly template; no dated rows in source")
    for p in phases: p.pop("_idx", None)
    return {"block":{"name":name or "Imported training plan","start_date":min((w["start_date"] for w in weeks), default=None),"end_date":max((w["end_date"] for w in weeks), default=None),"goal":None,"status":"active","source":"imported"},"phases":phases,"weeks":weeks,"sessions":sessions,"warnings":warnings}


def _normalize_json(data: dict[str, Any], name: str | None) -> dict[str, Any]:
    if "block" in data and all(k in data for k in ("phases","weeks","sessions")):
        data.setdefault("warnings", []); return data
    sessions = data.get("sessions", [])
    return {"block":{"name":name or data.get("name","Imported plan"),"start_date":None,"end_date":None,"status":"active","source":"imported"},"phases":data.get("phases",[]),"weeks":data.get("weeks",[]),"sessions":sessions,"warnings":[]}
