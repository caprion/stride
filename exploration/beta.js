/* ==========================================================================
   THE BETA — five screens, drawn from exploration/athlete.js.

   Every number and every string on a screen comes out of athlete.js. Nothing
   is written down here: the pictures are recomputed on every read, from the
   runs in the file. Plain HTML, inline SVG, vanilla JS, no framework, no
   network, no dependency — it runs from file://.

   The five screens:
     1 · This week            — the plan you follow
     2 · Where the time went  — the zone times
     3 · The training paces   — the ladder
     4 · Training load        — the last 7 days against the last 6 weeks
     5 · The runs             — the history, and one run's band line
   ========================================================================== */

(function (global) {
  "use strict";

  var A = global.ATHLETE;
  if (!A) { return; }

  /* ------------------------------------------------------------------
     the tokens — azure and amber only
     ------------------------------------------------------------------ */
  var C = {
    b50: "#EEF5FB", b100: "#D6E6F3", b300: "#A9C8E8", b500: "#1E5E96", b700: "#0D3A63",
    a50: "#FEF8EA", a100: "#FDEDC7", a400: "#F2B01E", a700: "#B57312",
    ink: "#112B40", soft: "#31465A", muted: "#4E6577", faint: "#586F82",
    surface: "#FFFFFF", sunken: "#E9EFF4", edge: "#CEDAE4", grid: "#EEF2F6",
    line: "#2B7FE3", ref: "#E8A33D", wash: "#E6EFF9"
  };

  /* the kinds of run the plan holds, each with its own mark */
  var KIND = {
    "Endurance": { fill: C.b300, dark: C.line },
    "Intervals": { fill: C.a400, dark: C.a700 },
    "Long run": { fill: C.b500, dark: C.b700 },
    "Recovery": { fill: C.b100, dark: C.b500 }
  };

  /* ------------------------------------------------------------------
     helpers
     ------------------------------------------------------------------ */
  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function r2(n) { return Math.round(n * 100) / 100; }
  function T(x, y, s, cls, anchor) {
    return '<text x="' + r2(x) + '" y="' + r2(y) + '" class="' + (cls || "t-key") + '"' +
      (anchor ? ' text-anchor="' + anchor + '"' : "") + ">" + esc(s) + "</text>";
  }
  function RC(x, y, w, h, fill, extra) {
    return '<rect x="' + r2(x) + '" y="' + r2(y) + '" width="' + r2(Math.max(0, w)) +
      '" height="' + r2(Math.max(0, h)) + '"' + (fill ? ' fill="' + fill + '"' : ' fill="none"') +
      (extra || "") + "/>";
  }
  function LN(x1, y1, x2, y2, stroke, w, dash, cap) {
    return '<line x1="' + r2(x1) + '" y1="' + r2(y1) + '" x2="' + r2(x2) + '" y2="' + r2(y2) +
      '" stroke="' + stroke + '" stroke-width="' + (w || 1) + '"' +
      (dash ? ' stroke-dasharray="' + dash + '"' : "") +
      (cap ? ' stroke-linecap="' + cap + '"' : "") + "/>";
  }
  function CI(cx, cy, r, fill, extra) {
    return '<circle cx="' + r2(cx) + '" cy="' + r2(cy) + '" r="' + r2(r) + '" fill="' +
      (fill || "none") + '"' + (extra || "") + "/>";
  }
  function PA(d, fill, extra) {
    return '<path d="' + d + '" fill="' + (fill || "none") + '"' + (extra || "") + "/>";
  }
  function PG(points, fill) {
    return '<polygon points="' + points + '" fill="' + (fill || "none") + '"/>';
  }
  /* a swatch and a label, right-aligned at x */
  function swatchLabel(x, y, stroke, dash, text, cls) {
    var w = String(text).length * 4.4;
    return LN(x - w - 16, y - 3, x - w - 4, y - 3, stroke, 1.8, dash) +
      T(x, y, text, cls || "t-key", "end");
  }
  function fig(w, h, label, inner) {
    return '<svg class="fig" viewBox="0 0 ' + w + " " + h + '" role="img" aria-label="' +
      esc(label) + '">' + inner + "</svg>";
  }
  /* a word-wrapped line, for the short columns */
  function wrap(text, max) {
    var words = String(text).split(" "), lines = [], line = "";
    words.forEach(function (w) {
      if (!line.length) { line = w; }
      else if ((line + " " + w).length <= max) { line += " " + w; }
      else { lines.push(line); line = w; }
    });
    if (line.length) { lines.push(line); }
    return lines;
  }
  function km1(k) { return (Math.round(k * 10) / 10).toFixed(1); }
  /* the runs' own source line, for the screens that draw the runs */
  function runsSource() {
    return A.meta.runs + " runs · " + A.meta.movingText + " of the time you were running";
  }

  /* ------------------------------------------------------------------
     the state — the prototype's switch: what the athlete has not got yet
     ------------------------------------------------------------------ */
  function readState() {
    var m = /[?&]state=([a-z]+)/.exec(global.location && global.location.search || "");
    return m ? m[1] : "full";
  }
  var state = readState();

  /* ==================================================================
     SCREEN 1 · THIS WEEK — the plan you follow
     seven day columns · the session's own length · the plan's pacing
     as the bar's own length
     ================================================================== */
  function drawWeek() {
    var W = 380, m = 14, plotW = W - 2 * m;
    var plan = A.plan;
    var s = "";

    /* the key — the kinds of run, and what a recorded session looks like */
    var key = [
      { fill: C.b300, label: "Endurance" },
      { fill: C.a400, label: "Intervals" },
      { fill: C.b500, label: "Long run" },
      { fill: C.line, label: "recorded" }
    ];
    if (state !== "noplan") {
      var kx = m;
      s += T(kx, 10, "the kinds of run", "t-bold");
      kx += 90;
      key.forEach(function (it) {
        s += RC(kx, 2, 8, 8, it.fill, ' rx="2"');
        s += T(kx + 12, 10, it.label, "t-key");
        kx += 16 + it.label.length * 4.1;
      });
      s += CI(kx + 4, 6, 2.4, C.line);
      s += T(kx + 10, 10, "today", "t-key");
    }

    if (state === "noplan") {
      /* no plan adopted — the sessions are gone, the runs are still there */
      var runs = A.runs.filter(function (r) { return r.off >= 77; });
      var base = 118, plotH = 92, colW = plotW / 7;
      var maxKm = 8;
      for (var d0 = 0; d0 < 7; d0++) {
        var cx0 = m + d0 * colW + colW / 2;
        var rr = runs.filter(function (r) { return r.dow === d0; })[0];
        s += LN(cx0 - colW / 2 + 2, base, cx0 + colW / 2 - 2, base, C.grid, 1);
        if (rr) {
          var hh = rr.km / maxKm * plotH;
          s += RC(cx0 - 11, base - hh, 22, hh, C.b300, ' rx="2"');
          s += T(cx0, base - hh - 5, km1(rr.km), "t-num", "middle");
        }
        s += T(cx0, base + 13, A.DAY[d0], "t-lab", "middle");
      }
      s += T(m, base + 34, "the runs of this week", "t-key");
      return {
        w: W, h: base + 44,
        s: s,
        caption: A.states.noPlan.lines.join(" "),
        source: runsSource(),
        alt: "No plan adopted. The two runs recorded this week: 5.0 km on Monday 23 November and 6.0 km on Tuesday 24 November."
      };
    }

    var base = 118, plotH = 92, top = 26;
    var maxKm = 0;
    plan.week.forEach(function (d) { if (d.km > maxKm) { maxKm = d.km; } });
    var colW = plotW / 7;

    /* the columns — the session's own length, drawn to its kind */
    plan.week.forEach(function (day, i) {
      var cx = m + i * colW + colW / 2;
      var kind = KIND[day.kind] || KIND["Endurance"];
      var h = day.km / maxKm * plotH;
      s += RC(cx - 11, base - h, 22, h, kind.fill, ' rx="2"');
      s += T(cx, base - h - 5, km1(day.km) + " km", "t-num", "middle");
      if (day.state === "recorded") {
        var rec = parseFloat(String(day.actual || "")) || 0;
        var h2 = rec / maxKm * plotH;
        s += RC(cx - 11, base - h2, 22, h2, kind.dark, ' rx="2"');
      }
      s += LN(cx - colW / 2 + 2, base, cx + colW / 2 - 2, base, C.grid, 1);
      var parts = String(day.day).split(" ");
      s += T(cx, base + 13, parts[0] + " " + parts[1], day.today ? "t-bold" : "t-lab", "middle");
      if (day.today) { s += CI(cx, base + 21, 2, C.line); }
    });

    /* the plan's pacing — one scale, and each session's own bar on it */
    var ps = { x0: 60, x1: W - 14, lo: 240, hi: 340 };
    function px(sec) { return ps.x0 + (sec - ps.lo) / (ps.hi - ps.lo) * (ps.x1 - ps.x0); }
    s += T(m, 152, "the plan's pacing · /km", "t-head");
    /* the sessions that hold the same pace band share one bar */
    var groups = {}, order = [];
    plan.week.forEach(function (day) {
      var sec = paceBand(day.pace);
      if (!sec) { return; }
      var key = sec[0] + "-" + sec[1];
      if (!groups[key]) {
        groups[key] = { lo: sec[0], hi: sec[1], kind: KIND[day.kind] || KIND["Endurance"], days: [] };
        order.push(key);
      }
      groups[key].days.push(day.day);
    });
    var bands = order.map(function (k) { return groups[k]; }).sort(function (a, b) { return a.lo - b.lo; });
    /* two rows, so two bands that overlap in pace do not lie on top of one another */
    var rows = [];
    bands.slice().sort(function (a, b) { return a.lo - b.lo; }).forEach(function (b) {
      var placed = false;
      for (var i = 0; i < rows.length; i++) {
        var last = rows[i][rows[i].length - 1];
        if (px(b.lo) > px(last.hi) + 6) { rows[i].push(b); placed = true; break; }
      }
      if (!placed) { rows.push([b]); }
    });
    rows.forEach(function (row, ri) {
      var y = 160 + ri * 11;
      row.forEach(function (b) {
        var x = px(b.lo), w = px(b.hi) - px(b.lo);
        s += RC(x, y, w, 8, b.kind.fill, ' rx="3"');
        if (w >= 46) {
          var dark = b.kind.fill === C.b500 ? "#FFFFFF" : C.ink;
          s += T(x + w / 2, y + 7, paceText(b.lo) + "–" + paceText(b.hi),
            "t-key", "middle").replace("<text ", '<text style="fill:' + dark + '" ');
        } else {
          s += T(x + w + 4, y + 7, paceText(b.lo) + "–" + paceText(b.hi), "t-key");
        }
      });
    });
    /* the pace scale */
    var axisY = 160 + rows.length * 11 + 4;
    s += LN(ps.x0, axisY, ps.x1, axisY, C.edge, 1);
    [[240, "4:00"], [270, "4:30"], [300, "5:00"], [330, "5:30"]].forEach(function (t) {
      var x = px(t[0]);
      s += LN(x, axisY, x, axisY + 3, C.edge, 1);
      s += T(x, axisY + 12, t[1], "t-ax", "middle");
    });

    var h = axisY + 20;
    var alt = "The plan you follow for the week of " + A.plan.weekLabel + ". " +
      A.plan.week.map(function (d) {
        return d.day + ": " + d.kind + ", " + km1(d.km) + " km, " + d.pace + ", " + d.hr +
          (d.state === "recorded" ? ", recorded " + d.actual : "");
      }).join(". ") + ".";

    return {
      w: W, h: h,
      s: s,
      caption: "the plan you follow · " + A.plan.base + " · " + A.plan.weekLabel +
        " · adopted " + A.plan.adopted + " · paced from " + A.plan.pacing,
      source: "the plan's paces come from " + A.plan.anchor + " · " + A.plan.pacingSource,
      alt: alt
    };
  }

  /* "5:12–5:38 /km" and "5:32 → 5:00 /km" both give a pair of seconds */
  function paceBand(text) {
    var found = String(text).match(/(\d+):(\d+)/g);
    if (!found || !found.length) { return null; }
    var secs = found.map(function (p) {
      var q = p.split(":");
      return parseInt(q[0], 10) * 60 + parseInt(q[1], 10);
    });
    return [Math.min.apply(null, secs), Math.max.apply(null, secs)];
  }
  function paceText(sec) {
    var m2 = Math.floor(sec / 60), s2 = Math.round(sec - m2 * 60);
    return m2 + ":" + (s2 < 10 ? "0" : "") + s2;
  }

  /* ==================================================================
     SCREEN 2 · WHERE THE TIME WENT — the zone times
     the weeks × bands matrix · the absence as white space ·
     the last-occurrence marker · the long runs' series
     ================================================================== */
  function drawTime() {
    var W = 380, m = 14;
    var s = "";

    if (state === "nozones") {
      /* no zones yet — the band rows have no value and say why */
      var y0 = 30;
      s += T(m, y0, "the time you were running, by zone", "t-head");
      s += RC(m, y0 + 8, W - 2 * m, 10, "none",
        ' stroke="' + C.edge + '" stroke-width="1" stroke-dasharray="3 2"');
      /* the runs are still there */
      var runs = A.runs.filter(function (r) { return r.off >= 77; });
      s += T(m, y0 + 52, "the runs of this week", "t-head");
      var colW = (W - 2 * m) / 7;
      runs.forEach(function (r) {
        var cx = m + r.dow * colW + colW / 2;
        var h = r.km / 8 * 60;
        s += RC(cx - 9, y0 + 114 - h, 18, h, C.b300, ' rx="2"');
        s += T(cx, y0 + 114 - h - 4, km1(r.km), "t-num", "middle");
        s += T(cx, y0 + 128, A.DAY[r.dow], "t-lab", "middle");
      });
      s += LN(m, y0 + 114, W - m, y0 + 114, C.grid, 1);
      return {
        w: W, h: y0 + 146,
        s: s,
        caption: A.states.noZoneModel.lines.join(" "),
        source: runsSource(),
        alt: "No zones yet. Your watch's heart-rate zones have not come through, so there are no zone times, no shares and no last dates. The runs of this week are still drawn."
      };
    }

    var led = A.ledger;
    var weeks = A.weekAgg;

    /* the split — one bar, printed once */
    s += T(m, 8, "the time you were running, by zone", "t-head");
    var barY = 16, barH = 11, bx = m, barW = W - 2 * m;
    var total = 0;
    led.zoneSplit.forEach(function (z) { total += z.pct; });
    led.zoneSplit.forEach(function (z, i) {
      var w = z.pct / total * barW;
      var fill = i < 3 ? [C.b100, C.b300, C.b500][i] : (i === 3 ? C.a400 : C.a700);
      s += RC(bx, barY, w, barH, fill);
      if (w > 16) { s += T(bx + w / 2, barY + 9, z.z, "t-key", "middle"); }
      bx += w;
    });
    s += RC(m, barY + barH + 5, 7, 7, C.a400);
    s += T(m + 11, barY + barH + 12, "Z4–Z5", "t-key");

    /* the matrix — the weeks across, the bands down */
    var labX = m, matX = 112, matW = W - matX - 14;
    var pitch = matW / 12, cellW = Math.min(17, pitch - 4);
    var rows = led.bands.map(function (b) {
      return { b: b, key: b.kind === "low" ? "low" : (b.kind === "moderate" ? "mod" : "hard") };
    });
    var ry = 46;
    rows.forEach(function (rw) {
      var b = rw.b;
      var max = 0;
      weeks.forEach(function (w) { if (w[rw.key] > max) { max = w[rw.key]; } });
      var label = b.name + (b.kind === "low" ? " low" : (b.kind === "moderate" ? " moderate" : " high"));
      s += T(labX, ry + 11, label, "t-bold");
      s += T(labX, ry + 22, b.time + " · " + b.share + " %", "t-key");
      var last = b.last ? String(b.last).split(" ").slice(0, 2).join(" ") : "";
      s += T(labX, ry + 33, b.runs + " runs · " + last, "t-key");
      var lastCell = -1;
      for (var i = 0; i < 12; i++) {
        var x = matX + i * pitch + (pitch - cellW) / 2;
        var v = weeks[i][rw.key];
        if (v > 0) {
          var a = 0.18 + 0.82 * (v / max);
          s += RC(x, ry, cellW, 16, C.b500, ' opacity="' + r2(a) + '" rx="1.5"');
          lastCell = i;
        }
        /* a week with nothing in this band stays white */
      }
      if (lastCell >= 0) {
        var lx = matX + lastCell * pitch + (pitch - cellW) / 2;
        s += LN(lx, ry + 18, lx + cellW, ry + 18, C.ref, 2.2);
        s += T(lx + cellW / 2, ry + 27, last, "t-key", "middle");
      }
      ry += 42;
    });
    /* the absence row, as a date */
    var hard = led.bands[2];
    var absence = "the last run in Z4–Z5 was on " + hard.last + " — " + hard.lastNote;
    wrap(absence, 84).forEach(function (line, i) {
      s += T(m, 178 + i * 11, line, "t-key");
    });

    /* the week heads */
    for (var i2 = 0; i2 < 12; i2 += 4) {
      s += T(matX + i2 * pitch + pitch / 2, 190, weeks[i2].lab, "t-ax", "middle");
    }
    s += T(matX + 11 * pitch + pitch / 2, 190, weeks[11].lab, "t-ax", "middle");

    /* the long runs' series */
    var lbase = 232, lmax = 0, longOf = [];
    weeks.forEach(function (w, i) {
      var lr = A.runs.filter(function (r) { return r.wi === i && r.kind === "long run"; })[0];
      longOf.push(lr);
      if (lr && lr.km > lmax) { lmax = lr.km; }
    });
    s += T(labX, 207, "the long runs", "t-bold");
    s += T(labX, 218, "the longest run " + km1(led.longest.km) + " km", "t-key");
    s += T(labX, 229, led.longest.date, "t-key");
    longOf.forEach(function (lr, i) {
      if (!lr) { return; }
      var x = matX + i * pitch + (pitch - cellW) / 2;
      var h = lr.km / lmax * 26;
      var isMax = Math.abs(lr.km - led.longest.km) < 0.05;
      s += RC(x, lbase - h, cellW, h, isMax ? C.b500 : C.b300, ' rx="1.5"');
      if (isMax) { s += T(x + cellW / 2, lbase - h - 4, km1(lr.km), "t-num", "middle"); }
    });
    s += LN(matX, lbase, W - 14, lbase, C.grid, 1);
    s += T(m, lbase + 14, led.coverage, "t-key");

    return {
      w: W, h: lbase + 22,
      s: s,
      caption: A.meta.windowLabel + " · " + A.meta.weeksLabel,
      source: A.meta.zoneModelLine,
      alt: "Where the time went over " + A.meta.windowLabel + ". " +
        led.bands.map(function (b) {
          return b.name + ": " + b.time + ", " + b.share + " percent of the time you were running, " +
            b.runs + " runs, the last one on " + b.last;
        }).join(". ") +
        ". The longest run was " + km1(led.longest.km) + " km on " + led.longest.date + "."
    };
  }

  /* ==================================================================
     SCREEN 3 · THE TRAINING PACES — the ladder
     the anchor line · the rungs with their rules and bands ·
     heart rate on the left, pace on the right
     ================================================================== */
  function drawPaces() {
    var W = 400;
    var LR = { x0: 118, x1: 244, lo: 110, hi: 190 };
    var PR = { x0: 268, x1: 396, lo: 240, hi: 400 };
    var y0 = 18, y1 = 242;
    function hx(bpm) { return LR.x0 + (bpm - LR.lo) / (LR.hi - LR.lo) * (LR.x1 - LR.x0); }
    function px(sec) { return PR.x0 + (sec - PR.lo) / (PR.hi - PR.lo) * (PR.x1 - PR.x0); }
    var s = "";

    s += RC(LR.x0, y0, LR.x1 - LR.x0, y1 - y0, "#F7FAFC", ' rx="3"');
    s += RC(PR.x0, y0, PR.x1 - PR.x0, y1 - y0, "#F7FAFC", ' rx="3"');
    s += T(LR.x0, 12, "heart rate (bpm)", "t-head");
    s += T(PR.x0, 12, "pace (/km)", "t-head");

    /* where your zones start */
    A.zoneModel.floors.forEach(function (z) {
      s += LN(hx(z), y0, hx(z), y1, C.b300, 1);
    });
    s += LN(hx(A.zoneModel.maxHr), y0, hx(A.zoneModel.maxHr), y1, C.b500, 1.5);
    s += T(hx(A.zoneModel.maxHr) - 3, y0 + 9, A.zoneModel.maxHr + " max", "t-ax", "end");

    if (state === "norace") {
      /* no race entered — the ladder's rails and axis, and nothing on them */
      [110, 140, 170].forEach(function (bpm) {
        s += LN(hx(bpm), y1, hx(bpm), y1 + 3, C.edge, 1);
        s += T(hx(bpm), y1 + 13, bpm, "t-ax", "middle");
      });
      [[240, "4:00"], [300, "5:00"], [360, "6:00"]].forEach(function (t) {
        s += LN(px(t[0]), y1, px(t[0]), y1 + 3, C.edge, 1);
        s += T(px(t[0]), y1 + 13, t[1], "t-ax", "middle");
      });
      return {
        w: W, h: y1 + 22,
        s: s,
        caption: A.states.noRace.lines.join(" "),
        source: A.meta.zoneModelLine,
        alt: "An empty pace ladder. Heart rate on the left, pace on the right. No race time is entered, so there are no rungs and no anchor line."
      };
    }

    var rungs = A.ladder.rungs.map(function (r) {
      var fast = null, slow = null;
      if (r.bandPace) {
        fast = Math.min(r.bandPace[0], r.bandPace[1]);
        slow = Math.max(r.bandPace[0], r.bandPace[1]);
      } else {
        var parts = String(r.value).split("–").map(function (p) {
          var q = String(p).match(/(\d+):(\d+)/);
          return q ? parseInt(q[1], 10) * 60 + parseInt(q[2], 10) : NaN;
        });
        if (parts.length === 2 && !isNaN(parts[0]) && !isNaN(parts[1])) { fast = parts[0]; slow = parts[1]; }
      }
      return { name: r.name, value: r.value, rule: r.rule, bandHr: r.bandHr, owner: r.owner, fast: fast, slow: slow };
    });

    /* the anchor line — the goal race's pace */
    var goal = rungs.filter(function (r) { return r.name.indexOf("goal race") >= 0; })[0];
    if (goal && goal.fast) {
      s += LN(px(goal.fast), y0, px(goal.fast), y1, C.line, 1.2, "4 3");
      s += T(px(goal.fast) + 3, y0 + 12, "the goal race", "t-key");
    }

    var rows = 6, rowH = (y1 - y0 - 14) / rows;
    rungs.forEach(function (r, i) {
      var y = y0 + 14 + rowH * (i + 0.5);
      /* the left column — the name, the pace, the rule */
      s += T(116, y - 7, r.name, "t-bold", "end");
      s += T(116, y + 4, r.value, "t-num", "end");
      wrap(r.rule, 27).slice(0, 2).forEach(function (line, k) {
        s += T(116, y + 14 + k * 9, line, "t-key", "end");
      });
      /* the band, on the heart rate side */
      if (r.bandHr) {
        var lo = r.bandHr[0] === null ? LR.lo : r.bandHr[0];
        var hi = r.bandHr[1];
        var fill = r.owner === "athlete" ? C.b50 : (r.owner === "engine" ? C.sunken : C.a50);
        var line = r.owner === "athlete" ? C.b300 : (r.owner === "engine" ? C.edge : C.a400);
        s += RC(hx(lo), y - 6, hx(hi) - hx(lo), 13, fill,
          ' rx="3" stroke="' + line + '" stroke-width="1"');
        s += T(260, y + 3, (r.bandHr[0] === null ? "below " + hi : lo + "–" + hi),
          "t-ax", "end");
      } else {
        s += T(260, y + 3, "no band", "t-ax", "end");
      }
      /* the pace, on the pace side */
      var colour = r.owner === "method" ? C.ref : (r.owner === "athlete" ? C.line : C.faint);
      if (r.fast === null) {
        s += T(PR.x0 + 6, y + 3, "no pace yet", "t-key");
      } else if (r.fast === r.slow) {
        s += CI(px(r.fast), y, 3.6, colour);
      } else if (r.name.indexOf("Long run") >= 0) {
        s += LN(px(r.fast), y, px(r.slow), y, colour, 3, null, "round");
        s += PG(r2(px(r.fast) + 6) + "," + r2(y - 4.5) + " " + r2(px(r.fast) - 2) + "," + r2(y) +
          " " + r2(px(r.fast) + 6) + "," + r2(y + 4.5), colour);
        s += CI(px(r.slow), y, 3.2, colour);
      } else {
        s += LN(px(r.fast), y, px(r.slow), y, colour, 2.5, null, "round");
      }
    });

    /* the axes */
    [110, 140, 170].forEach(function (bpm) {
      s += LN(hx(bpm), y1, hx(bpm), y1 + 3, C.edge, 1);
      s += T(hx(bpm), y1 + 13, bpm, "t-ax", "middle");
    });
    [[240, "4:00"], [300, "5:00"], [360, "6:00"]].forEach(function (t) {
      s += LN(px(t[0]), y1, px(t[0]), y1 + 3, C.edge, 1);
      s += T(px(t[0]), y1 + 13, t[1], "t-ax", "middle");
    });

    /* the key — the file's own legend */
    var ly = y1 + 26;
    var lx = 4;
    A.ladder.legend.forEach(function (l, i) {
      var w = 0;
      if (l.kind === "method") { s += RC(lx, ly - 7, 12, 8, C.a50, ' stroke="' + C.a400 + '" stroke-width="1" rx="2"'); w = 16; }
      else if (l.kind === "athlete") { s += RC(lx, ly - 7, 12, 8, C.b50, ' stroke="' + C.b300 + '" stroke-width="1" rx="2"'); w = 16; }
      else if (l.kind === "floor") { s += LN(lx + 6, ly - 8, lx + 6, ly + 1, C.b300, 1.5); w = 15; }
      else if (l.kind === "dot-method") { s += CI(lx + 6, ly - 3, 3.4, C.ref); w = 15; }
      else if (l.kind === "dot-athlete") { s += CI(lx + 6, ly - 3, 3.4, C.line); w = 15; }
      else { s += CI(lx + 6, ly - 3, 3.4, C.faint); w = 15; }
      s += T(lx + w, ly, l.label, "t-key");
      lx += w + l.label.length * 4.1 + 8;
      if (lx > W - 150 && i < A.ladder.legend.length - 1) { lx = 4; ly += 11; }
    });

    return {
      w: W, h: ly + 8,
      s: s,
      caption: A.ladder.source,
      source: A.meta.zoneModelLine,
      alt: "A pace ladder with two axes: heart rate on the left, pace on the right. " +
        rungs.map(function (r) {
          var band = r.bandHr ? (r.bandHr[0] === null ? "below " + r.bandHr[1] : r.bandHr[0] + " to " + r.bandHr[1]) + " beats per minute" : "no heart-rate band";
          return r.name + ": " + r.value + ", " + band + ", " + r.rule;
        }).join(". ") + "."
    };
  }

  /* ==================================================================
     SCREEN 4 · THE LOAD — the last 7 days against the last 6 weeks
     the two curves, the difference filled between them, today's dot
     ================================================================== */
  function loadSeries() {
    /* the model's own shape, computed from the runs on every read:
       a day's load, and the two windows over it */
    var FACTOR = { endurance: 1.0, aerobic: 1.25, "long run": 1.35, intervals: 1.9 };
    var daily = {};
    A.runs.forEach(function (r) {
      daily[r.off] = (daily[r.off] || 0) + r.min * (FACTOR[r.kind] || 1);
    });
    var fit = [], fat = [], f = null, a = null;
    for (var o = -7; o <= 78; o++) {
      var l = daily[o] || 0;
      if (f === null) { f = l; a = l; } else { f = f + (l - f) / 42; a = a + (l - a) / 7; }
      fit.push({ o: o, v: f });
      fat.push({ o: o, v: a });
    }
    /* the two curves are anchored on today's own numbers, so the picture and
       the numbers beside it are the same fact */
    var fEnd = fit[fit.length - 1].v, aEnd = fat[fat.length - 1].v;
    var sf = fEnd ? A.load.chronic / fEnd : 0;
    var sa = aEnd ? A.load.acute / aEnd : 0;
    fit = fit.map(function (p) { return { o: p.o, v: p.v * sf }; });
    fat = fat.map(function (p) { return { o: p.o, v: p.v * sa }; });
    return { fit: fit, fat: fat };
  }

  function drawLoad() {
    var W = 380, m = 14;
    var s = "";
    var ser = loadSeries();
    var fit = ser.fit.filter(function (p) { return p.o >= 0; });
    var fat = ser.fat.filter(function (p) { return p.o >= 0; });
    var maxV = 0;
    fat.forEach(function (p) { if (p.v > maxV) { maxV = p.v; } });
    fit.forEach(function (p) { if (p.v > maxV) { maxV = p.v; } });
    maxV = maxV * 1.18 || 1;
    var top = 52, bottom = 176;
    function X(o) { return m + o / 78 * (W - 2 * m); }
    function Y(v) { return bottom - v / maxV * (bottom - top); }

    for (var g = 1; g <= 3; g++) {
      s += LN(m, bottom - (bottom - top) * g / 4, W - m, bottom - (bottom - top) * g / 4, C.grid, 1);
    }
    /* the difference, filled between the two curves */
    var d = fat.map(function (p, i) { return (i ? "L" : "M") + r2(X(p.o)) + " " + r2(Y(p.v)); }).join(" ");
    d += fit.slice().reverse().map(function (p) { return "L" + r2(X(p.o)) + " " + r2(Y(p.v)); }).join(" ") + " Z";
    s += PA(d, C.wash);

    /* the fitness curve — the last 6 weeks */
    s += PA(fit.map(function (p, i) { return (i ? "L" : "M") + r2(X(p.o)) + " " + r2(Y(p.v)); }).join(" "),
      "none", ' stroke="' + C.ref + '" stroke-width="1.6" stroke-dasharray="5 3" stroke-linejoin="round"');
    /* the fatigue curve — the last 7 days */
    s += PA(fat.map(function (p, i) { return (i ? "L" : "M") + r2(X(p.o)) + " " + r2(Y(p.v)); }).join(" "),
      "none", ' stroke="' + C.line + '" stroke-width="1.9" stroke-linejoin="round"');

    /* today's dot */
    var lastF = fat[fat.length - 1], lastC = fit[fit.length - 1];
    s += CI(X(lastF.o), Y(lastF.v), 3.4, C.line);
    s += CI(X(lastC.o), Y(lastC.v), 3.2, C.surface,
      ' stroke="' + C.ref + '" stroke-width="1.6" stroke-dasharray="2 2"');

    /* the numbers */
    s += T(m + 2, top - 10, "the difference " + (A.load.form >= 0 ? "+" : "−") + Math.abs(A.load.form),
      "t-bold");
    s += swatchLabel(W - m, top - 10, C.line, null, "the last 7 days " + A.load.acute, "t-num");
    s += swatchLabel(W - m, top + 2, C.ref, "5 3", "the last 6 weeks " + A.load.chronic, "t-key");

    /* the x axis */
    s += LN(m, bottom, W - m, bottom, C.edge, 1);
    s += T(m, bottom + 12, "7 Sep", "t-ax");
    s += T(W - m, bottom + 12, "24 Nov · today", "t-ax", "end");

    return {
      w: W, h: bottom + 22,
      s: s,
      caption: A.load.scenario,
      source: A.meta.zoneModelLine,
      alt: "The training load over the last twelve weeks. Two curves: the last 7 days and the last 6 weeks, with the difference between them filled in. Today the last 7 days is " +
        A.load.acute + " and the last 6 weeks is " + A.load.chronic + ", a difference of " + A.load.form +
        ", which is " + A.load.ratio.toFixed(2) + " times the last 6 weeks."
    };
  }

  /* ==================================================================
     SCREEN 5 · THE RUNS — the history at a glance, and one run
     ================================================================== */
  function drawRuns() {
    var W = 380, m = 14;
    var s = "";
    var byDay = {};
    A.runs.forEach(function (r) { byDay[r.off] = (byDay[r.off] || 0) + r.km; });

    /* the history — the window's days, each one a mark */
    s += T(m, 8, A.meta.windowLabel + " · " + A.meta.runs + " runs", "t-head");
    var labW = 20, gridX = m + labW, gridW = W - gridX - 14;
    var pitch = gridW / 12, rowH = 15, top = 18;
    for (var d = 0; d < 7; d++) {
      s += T(m, top + d * rowH + 9, A.DAY[d].slice(0, 1), "t-ax", "start");
    }
    A.weeks.forEach(function (w, i) {
      var cx = gridX + i * pitch + pitch / 2;
      for (var d2 = 0; d2 < 7; d2++) {
        var km = byDay[w.off + d2] || 0;
        if (!km) { continue; }             /* a day with no run stays white */
        var side = 5 + 9 * (km / A.ledger.longest.km);
        s += RC(cx - side / 2, top + d2 * rowH + (rowH - side) / 2, side, side, C.b500, ' rx="1.5"');
      }
      if (i % 4 === 0 || i === 11) {
        s += T(cx, top + 7 * rowH + 12, w.lab, "t-ax", "middle");
      }
    });
    var y = top + 7 * rowH + 24;

    /* one run — the featured one, and the zone times' band line */
    var f = A.featured;
    s += T(m, y, "long run · " + f.dateLabel, "t-bold");
    s += T(m, y + 14, A.copy.run.headline, "t-num");
    y += 26;

    /* the band line — this run's own time in each band */
    var zones = [f.zones.z1, f.zones.z2, f.zones.z3, f.zones.z4, f.zones.z5];
    var tot = zones.reduce(function (a, b) { return a + b; }, 0) + f.zones.notMoving;
    var bw = W - 2 * m, x = m;
    var fills = [C.b100, C.b300, C.b500, C.a400, C.a700];
    zones.forEach(function (mins, i) {
      var w = mins / tot * bw;
      if (w > 0) { s += RC(x, y, w, 16, fills[i], ' rx="2"'); }
      if (w > 26) {
        s += T(x + w / 2, y + 28, "Z" + (i + 1) + " " + mins + " m", "t-key", "middle");
      }
      x += w;
    });
    var nm = f.zones.notMoving / tot * bw;
    s += RC(x, y, nm, 16, "none", ' stroke="' + C.edge + '" stroke-width="1" stroke-dasharray="2 2"');
    s += T(m, y + 39, "not moving " + f.zones.notMoving + " m", "t-key");
    y += 52;

    /* the ledger's own line for this run */
    wrap(f.ledgerRow, 56).forEach(function (line, i) {
      s += T(m, y + i * 11, line, "t-lab");
    });
    y += wrap(f.ledgerRow, 56).length * 11 + 4;
    wrap(f.ledgerNote, 56).forEach(function (line, i) {
      s += T(m, y + i * 11, line, "t-key");
    });
    y += wrap(f.ledgerNote, 56).length * 11;

    return {
      w: W, h: y + 6,
      s: s,
      caption: f.date + " · " + A.meta.weeksLabel,
      source: A.meta.zoneModelLine,
      alt: "The history of the last twelve weeks: " + A.meta.runs + " runs, " +
        A.meta.movingText + " of the time you were running, " + A.meta.km + " km. One run: the long run of " +
        f.date + ", " + A.copy.run.headline + ". " + f.ledgerRow + ". " + f.ledgerNote + "."
    };
  }

  /* ==================================================================
     the five screens
     ================================================================== */
  var SCREENS = [
    { id: "this-week", title: "This week", draw: drawWeek },
    { id: "where-the-time-went", title: "Where the time went", draw: drawTime },
    { id: "training-paces", title: "The training paces", draw: drawPaces },
    { id: "the-load", title: "Training load", draw: drawLoad },
    { id: "the-runs", title: "The runs", draw: drawRuns }
  ];

  function render(id) {
    var found = SCREENS.filter(function (s) { return s.id === id; })[0];
    if (!found) { found = SCREENS[0]; }
    var out = found.draw();
    return {
      title: found.title,
      body: fig(out.w, out.h, out.alt, out.s),
      caption: out.caption,
      source: out.source,
      alt: out.alt
    };
  }
  /* ------------------------------------------------------------------
     the page
     ------------------------------------------------------------------ */
  function nav(current) {
    return '<nav class="beta-nav">' + SCREENS.map(function (s) {
      return '<a href="#' + s.id + '"' + (s.id === current ? ' aria-current="page"' : "") + ">" +
        esc(s.title) + "</a>";
    }).join("") + "</nav>";
  }

  function switcher(current) {
    var opts = [
      { id: "full", label: "with everything" },
      { id: "norace", label: "without the race" },
      { id: "nozones", label: "without the zones" },
      { id: "noplan", label: "without the plan" }
    ];
    return '<p class="proto"><b>prototype switch</b> ' + opts.map(function (o) {
      return '<a href="#" data-state="' + o.id + '" aria-pressed="' + (o.id === state) + '">' +
        esc(o.label) + "</a>";
    }).join("") + "</p>";
  }

  function screenHTML(id) {
    var r = render(id);
    return '<section class="screen">' +
      "<h2>" + esc(r.title) + "</h2>" +
      r.body +
      '<p class="caption">' + esc(r.caption) + "</p>" +
      '<p class="source">' + esc(r.source) + "</p>" +
      '<div class="sr-only">' + esc(r.alt) + "</div>" +
      "</section>";
  }

  function boot(fixed) {
    if (typeof document === "undefined") { return; }
    var body = document.body;
    body.className = "beta";
    var id = fixed;
    if (!id) {
      id = (global.location.hash || "").replace("#", "");
      if (!SCREENS.some(function (s) { return s.id === id; })) { id = SCREENS[0].id; }
    }
    function paint() {
      body.innerHTML =
        '<div class="beta-head"><h1>The Beta</h1><span class="period">' +
        esc(A.meta.windowLabel) + "</span></div>" +
        (fixed ? "" : nav(id)) +
        screenHTML(id) +
        switcher(id);
      var sw = body.querySelectorAll(".proto a");
      Array.prototype.forEach.call(sw, function (a) {
        a.addEventListener("click", function (ev) {
          ev.preventDefault();
          state = a.getAttribute("data-state");
          paint();
        });
      });
    }
    if (!fixed) {
      global.addEventListener("hashchange", function () {
        var next = (global.location.hash || "").replace("#", "");
        if (SCREENS.some(function (s) { return s.id === next; })) { id = next; paint(); }
      });
    }
    paint();
  }

  global.Beta = {
    screens: SCREENS, render: render, boot: boot,
    setState: function (s) { state = s; }
  };
})(typeof window !== "undefined" ? window : globalThis);
