/* ==========================================================================
   THE SYNTHETIC ATHLETE — the one source of every number in the end-to-end.

   Every figure the walk-through shows is read from this file. Nothing in
   end-to-end.html types a number that is not here; if a screen needs one
   that is missing, it is added here.

   The athlete: one runner. The window 7 Sep – 24 Nov 2026, 58 runs, 49 h 32 m
   of moving time. The zone model read 3 Nov 2026 (HR_MAX, max heart rate
   183 bpm). A 10K in 41:36 on 12 Oct 2026, typed by the athlete, and a half
   marathon as the goal. The Pfitzinger 10K plan adopted 30 Sep 2026, paced
   with the threshold the athlete chose from the watch's own zone model
   (165 bpm, 3 Nov 2026).

   The numbers come from the existing prototypes and are reused exactly:
     · the window, the run count, the moving time, the zone model
       — exploration/stimulus-ledger.html
     · the run-level table (58 runs), the weekly shape, the eleven cards
       — exploration/cards-as-pictures.html
     · the race time, the ladder, the curve
       — exploration/pace-engine.html
     · the threshold candidates, the choice, the history
       — exploration/threshold-setup.html

   Two corrections were made to the prototype's run table, both to make it
   agree with the ledger the same prototypes ship:
     1. two runs carried an average heart rate that put them in Z4–Z5 while
        their own zone minutes said otherwise (the run of 23 Nov and the run
        of 16 Nov). Their averages were moved into the band their minutes
        place them in, so "no run in Z4–Z5 since 20 Oct 2026" is true of the
        run table as well as of the ledger.
     2. the 21 Nov long run's average was moved to 145 bpm for the same
        reason (its zone minutes are all Z1–Z3).
   ========================================================================== */

(function () {
  "use strict";

  /* ------------------------------------------------------------------
     1 · THE WINDOW
     ------------------------------------------------------------------ */

  var meta = {
    windowFrom: "7 Sep 2026",
    windowTo: "24 Nov 2026",
    windowLabel: "7 Sep – 24 Nov 2026",
    windowWeeks: 12,
    weeksLabel: "last 12 weeks",
    runs: 58,
    movingMin: 2972,
    movingText: "49 h 32 m",
    km: 564.8,
    priorLabel: "31 Aug – 17 Nov 2026",
    priorText: "49 h 20 m across 58 runs",
    zoneModelLine: "your zones · running · max heart rate 183 bpm · taken from your watch on 3 Nov 2026",
    today: "24 Nov 2026"
  };

  /* ------------------------------------------------------------------
     2 · THE ZONE MODEL — a dated fact about the watch's configuration
     ------------------------------------------------------------------ */

  var zoneModel = {
    method: "HR_MAX",
    maxHr: 183,
    floors: [115, 128, 143, 159, 165],
    readOn: "3 Nov 2026",
    zoneNames: ["Z1", "Z2", "Z3", "Z4", "Z5"]
  };

  /* ------------------------------------------------------------------
     3 · THE RUNS — 58 runs, 12 weeks
     a run: [dow, km, minutes, Z3 minutes, Z4–Z5 minutes, workout-step
             minutes, average heart rate, hour of the day, kind]
     ------------------------------------------------------------------ */

  var weeks = [
    { lab: "7 Sep",  off: 0,  r: [[0,8.0,40,6,0,0,138,6,"endurance"],[1,8.4,42,7,0,0,149,18,"aerobic"],[2,7.2,36,6,0,0,136,6,"endurance"],[3,9.2,46,7,0,0,149,18,"aerobic"],[4,7.6,38,6,0,0,137,7,"endurance"],[5,18.9,114,10,0,0,144,9,"long run"],[6,6.0,30,6,0,0,133,9,"endurance"]] },
    { lab: "14 Sep", off: 7,  r: [[0,8.0,40,6,0,0,137,6,"endurance"],[1,8.4,42,7,0,0,149,18,"aerobic"],[2,7.2,36,6,0,0,135,6,"endurance"],[3,12.4,62,11,14,45,166,17,"intervals"],[5,19.5,114,9,0,0,145,9,"long run"],[6,6.0,30,6,0,0,132,9,"endurance"]] },
    { lab: "21 Sep", off: 14, r: [[0,8.0,40,6,0,0,138,6,"endurance"],[1,8.4,42,7,0,0,150,18,"aerobic"],[2,7.2,36,6,0,0,136,6,"endurance"],[3,9.2,46,7,0,0,148,18,"aerobic"],[5,19.7,115,10,0,0,146,9,"long run"],[6,6.0,30,6,0,0,133,9,"endurance"]] },
    { lab: "28 Sep", off: 21, r: [[0,8.0,40,6,0,0,137,6,"endurance"],[1,8.4,42,7,0,0,149,18,"aerobic"],[2,7.2,36,6,0,0,135,6,"endurance"],[3,12.4,62,8,12,42,166,17,"intervals"],[5,18.9,110,9,0,0,144,9,"long run"],[6,6.0,30,6,0,0,132,9,"endurance"]] },
    { lab: "5 Oct",  off: 28, r: [[0,8.0,40,6,0,0,138,6,"endurance"],[1,8.4,42,7,0,0,150,18,"aerobic"],[2,7.2,36,6,0,0,136,6,"endurance"],[3,9.2,46,7,0,0,149,18,"aerobic"],[4,7.6,38,6,0,0,137,7,"endurance"],[5,19.5,114,10,0,0,145,9,"long run"],[6,6.0,30,6,0,0,133,9,"endurance"]] },
    { lab: "12 Oct", off: 35, r: [[0,8.0,40,6,0,0,137,6,"endurance"],[1,8.4,42,7,0,0,149,18,"aerobic"],[2,7.2,36,6,0,0,135,6,"endurance"],[3,12.4,62,11,12,48,166,17,"intervals"],[5,19.4,115,9,0,0,145,9,"long run"],[6,6.4,32,6,0,0,132,9,"endurance"]] },
    { lab: "19 Oct", off: 42, r: [[0,7.2,36,6,0,0,138,6,"endurance"],[1,12.6,63,13,30,0,167,17,"intervals"],[2,7.2,36,6,0,0,136,6,"endurance"],[3,9.2,46,7,0,0,149,18,"aerobic"],[5,18.9,110,9,0,0,144,9,"long run"],[6,6.0,30,6,0,0,133,9,"endurance"]] },
    { lab: "26 Oct", off: 49, r: [[0,8.0,40,6,0,0,137,6,"endurance"],[1,8.4,42,7,0,0,148,18,"aerobic"],[3,9.2,46,7,0,0,149,18,"aerobic"],[4,8.8,44,6,0,0,140,7,"endurance"],[5,19.6,114,10,0,0,144,8,"long run"],[6,6.0,30,6,0,0,132,9,"endurance"]] },
    { lab: "2 Nov",  off: 56, r: [[6,6.0,30,6,0,0,133,9,"endurance"]] },
    { lab: "9 Nov",  off: 63, r: [] },
    { lab: "16 Nov", off: 70, r: [[0,8.0,40,6,0,0,138,6,"endurance"],[1,8.4,42,7,0,0,150,18,"aerobic"],[2,10.0,49,6,0,0,157,9,"aerobic"],[5,20.4,119,10,0,0,145,8,"long run"],[6,6.0,30,6,0,0,133,9,"endurance"]] },
    { lab: "23 Nov", off: 77, r: [[0,5.0,23,6,0,0,152,8,"aerobic"],[1,6.0,30,6,0,0,134,7,"endurance"]] }
  ];

  /* the week before the window — the comparison period's first week */
  var prior = { lab: "31 Aug", off: -7, r: [[0,8.0,40,6,0,0,137,6,"endurance"],[1,8.4,42,7,0,0,149,18,"aerobic"],[2,7.2,36,6,0,0,135,6,"endurance"],[3,9.2,46,7,0,0,150,18,"aerobic"],[5,18.5,108,9,0,0,144,9,"long run"],[6,6.0,30,6,0,0,132,9,"endurance"]] };

  var DAY = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
  var MONTH = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  var ORIGIN = Date.UTC(2026, 8, 7);

  function dayLabel(off) {
    var d = new Date(ORIGIN + off * 86400000);
    return d.getUTCDate() + " " + MONTH[d.getUTCMonth()];
  }
  function dateFull(off) {
    var d = new Date(ORIGIN + off * 86400000);
    return d.getUTCDate() + " " + MONTH[d.getUTCMonth()] + " " + d.getUTCFullYear();
  }
  function dur(m) {
    m = Math.round(m);
    var h = Math.floor(m / 60), r = m % 60;
    return h ? h + " h " + (r < 10 ? "0" : "") + r + " m" : r + " m";
  }
  function timeText(sec) {
    var h = Math.floor(sec / 3600), m = Math.floor((sec - h * 3600) / 60), s = Math.round(sec % 60);
    if (s === 60) { m += 1; s = 0; }
    if (h) { return h + ":" + (m < 10 ? "0" : "") + m + ":" + (s < 10 ? "0" : "") + s; }
    return m + ":" + (s < 10 ? "0" : "") + s;
  }
  function paceText(sec) {
    if (sec === null || sec === undefined) { return "—"; }
    var m = Math.floor(sec / 60), s = Math.round(sec - m * 60);
    if (s === 60) { m += 1; s = 0; }
    return m + ":" + (s < 10 ? "0" : "") + s;
  }
  function kmText(k) { return (Math.round(k * 10) / 10).toFixed(1) + " km"; }
  function signedKm(k) {
    var v = Math.round(k * 10) / 10;
    return (v < 0 ? "−" : v > 0 ? "+" : "") + Math.abs(v).toFixed(1) + " km";
  }
  function signedMin(m) {
    var s = m < 0 ? "−" : m > 0 ? "+" : "";
    var v = Math.abs(Math.round(m));
    return s + (v >= 60 ? dur(v) : v + " m");
  }

  /* ---- derived: the runs, flat, with their dates ---- */
  var runs = [];
  weeks.forEach(function (w, wi) {
    w.r.forEach(function (r) {
      var low = r[2] - r[3] - r[4];
      runs.push({
        dow: r[0], km: r[1], min: r[2], mod: r[3], hard: r[4], steps: r[5],
        hr: r[6], hour: r[7], kind: r[8],
        low: low, wi: wi, off: w.off + r[0], date: dayLabel(w.off + r[0]),
        dateFull: dateFull(w.off + r[0]),
        pace: Math.round(r[2] * 60 / r[1]),
        week: w.lab
      });
    });
  });
  runs.sort(function (a, b) { return a.off - b.off; });

  function byWeek(ws) {
    return ws.map(function (w) {
      var o = { lab: w.lab, off: w.off, low: 0, mod: 0, hard: 0, km: 0, min: 0, steps: 0, n: 0, days: [] };
      w.r.forEach(function (r) {
        o.low += r[2] - r[3] - r[4]; o.mod += r[3]; o.hard += r[4];
        o.km += r[1]; o.min += r[2]; o.n++; o.days.push(r[0]);
        if (r[5] > 0) { o.steps += r[5]; }
      });
      return o;
    });
  }
  var weekAgg = byWeek(weeks);

  function bands(list) {
    var low = 0, mod = 0, hard = 0, min = 0, km = 0, steps = 0, stepRuns = 0;
    list.forEach(function (r) {
      low += r.low; mod += r.mod; hard += r.hard; min += r.min; km += r.km;
      if (r.steps > 0) { steps += r.steps; stepRuns++; }
    });
    return { low: low, mod: mod, hard: hard, min: min, km: km, steps: steps, stepRuns: stepRuns };
  }
  var bandTotals = bands(runs);

  function lastDayWhere(pred) {
    var best = null;
    runs.forEach(function (r) { if (pred(r) && (best === null || r.off > best)) { best = r.off; } });
    return best === null ? null : dayLabel(best);
  }

  /* the months the window touches, with the months before it */
  var MONTHS_BEFORE = [
    { lab: "Dec", km: 186, mix: [.72, .24, .04] }, { lab: "Jan", km: 214, mix: [.66, .28, .06] },
    { lab: "Feb", km: 198, mix: [.70, .25, .05] }, { lab: "Mar", km: 172, mix: [.74, .22, .04] },
    { lab: "Apr", km: 196, mix: [.78, .19, .03] }, { lab: "May", km: 224, mix: [.68, .26, .06] },
    { lab: "Jun", km: 208, mix: [.71, .24, .05] }, { lab: "Jul", km: 246, mix: [.67, .27, .06] },
    { lab: "Aug", km: 231, mix: [.72, .24, .04] }
  ];
  var monthsSeries = (function () {
    var buckets = {};
    runs.forEach(function (r) {
      var d = new Date(ORIGIN + r.off * 86400000);
      var key = d.getUTCFullYear() + "-" + ("0" + (d.getUTCMonth() + 1)).slice(-2);
      var b = buckets[key] || (buckets[key] = { km: 0, low: 0, mod: 0, hard: 0 });
      b.km += r.km; b.low += r.low; b.mod += r.mod; b.hard += r.hard;
    });
    var out = MONTHS_BEFORE.map(function (x) { return { lab: x.lab, km: x.km, mix: x.mix, partial: false }; });
    [["2026-09", "Sep"], ["2026-10", "Oct"], ["2026-11", "Nov"]].forEach(function (pair) {
      var b = buckets[pair[0]] || { km: 0, low: 0, mod: 0, hard: 0 };
      var t = b.low + b.mod + b.hard;
      out.push({ lab: pair[1], km: b.km, mix: t ? [b.low / t, b.mod / t, b.hard / t] : [0, 0, 0], partial: pair[1] === "Nov" });
    });
    return out;
  })();

  /* the habit: the streak and the Eddington number */
  var habit = (function () {
    var byDay = {};
    runs.forEach(function (r) { byDay[r.off] = (byDay[r.off] || 0) + r.km; });
    var days = Object.keys(byDay).map(Number);
    var counts = [];
    for (var e = 1; e <= 20; e++) {
      counts.push({ e: e, n: days.filter(function (d) { return byDay[d] >= e; }).length });
    }
    var num = 0;
    counts.forEach(function (c) { if (c.n >= c.e) { num = c.e; } });
    var set = {};
    runs.forEach(function (r) { set[r.off] = 1; });
    var s = 0, o = 78;
    while (set[o]) { s++; o--; }
    return { counts: counts, num: num, days: days.length, streak: s };
  })();

  /* the shape: seven facets, on one scale */
  var facets = (function () {
    var kinds = {}, daySet = {};
    runs.forEach(function (r) { kinds[r.kind] = 1; daySet[r.dow] = 1; });
    var consistent = weekAgg.filter(function (w) { return w.n >= 4; }).length;
    return {
      volume: bandTotals.km,
      consistency: consistent / 12,
      intensity: (bandTotals.mod + bandTotals.hard) / bandTotals.min,
      duration: bandTotals.min / runs.length,
      density: runs.length / 12,
      variety: Object.keys(kinds).length,
      days: Object.keys(daySet).length
    };
  })();

  /* the same seven facets over the period before — the same span, one week earlier */
  var facetsPrior = (function () {
    var list = A_prior_runs();
    var low = 0, mod = 0, hard = 0, min = 0, km = 0, kinds = {}, days = {}, n = 0;
    list.forEach(function (r) {
      low += r.low; mod += r.mod; hard += r.hard; min += r.min; km += r.km;
      kinds[r.kind] = 1; days[r.dow] = 1; n++;
    });
    var perWeek = {};
    list.forEach(function (r) {
      var w = Math.floor((r.off + 7) / 7);
      (perWeek[w] = perWeek[w] || []).push(r);
    });
    var consistent = Object.keys(perWeek).filter(function (k) { return perWeek[k].length >= 4; }).length / 12;
    return {
      volume: km, consistency: consistent, intensity: min ? (mod + hard) / min : 0,
      duration: n ? min / n : 0, density: n / 12, variety: Object.keys(kinds).length,
      days: Object.keys(days).length
    };
  })();

  function A_prior_runs() {
    var out = [];
    function push(off, r) {
      out.push({ off:off, dow:r[0], km:r[1], min:r[2], mod:r[3], hard:r[4],
        low:r[2]-r[3]-r[4], kind:r[8] });
    }
    prior.r.forEach(function (r) { push(prior.off + r[0], r); });
    weeks.slice(0, 10).forEach(function (w) {
      w.r.forEach(function (r) { push(w.off + r[0], r); });
    });
    weeks[10].r.filter(function (r) { return r[0] <= 1; }).forEach(function (r) { push(weeks[10].off + r[0], r); });
    return out;
  }

  /* ------------------------------------------------------------------
     4 · THE LEDGER — the window's own answers
     the band times and the last dates are the ledger's; the shares are
     the ledger's rounded split of the same band times
     ------------------------------------------------------------------ */

  var ledger = {
    zoneSplit: [
      { z: "Z1", pct: 42.4 }, { z: "Z2", pct: 41.7 }, { z: "Z3", pct: 13.9 },
      { z: "Z4", pct: 2.0 }, { z: "Z5", pct: 0.3 }
    ],
    outsidePct: 0,
    rollup: { low: 84.1, mod: 13.9, high: 2.3 },
    bands: [
      {
        name: "Z1–Z2", kind: "low", time: "41 h 32 m", mins: bandTotals.low, share: 84,
        runs: 58, last: "24 Nov 2026",
        change: "+37 m", changeText: "37 m more than the same span, one week earlier (41 h 32 m against 40 h 54 m)",
        caveat: null
      },
      {
        name: "Z3", kind: "moderate", time: "6 h 52 m", mins: bandTotals.mod, share: 14,
        runs: 58, last: "24 Nov 2026",
        change: "−8 m", changeText: "8 m less than the same span, one week earlier (6 h 52 m against 7 h 01 m)",
        caveat: null
      },
      {
        name: "Z4–Z5", kind: "high", time: "1 h 08 m", mins: bandTotals.hard, share: 2,
        runs: 4, last: "20 Oct 2026", lastNote: "35 days before the end of this period",
        change: "−15 m", changeText: "15 m less than the same span, one week earlier (1 h 08 m against 1 h 23 m) · 4 runs against 5",
        caveat: "Heart-rate time in zone counts hard running slowly and easy running generously — the last run in Z4–Z5 is what the watch recorded."
      }
    ],
    longest: {
      name: "Longest run", km: 20.4, time: "1 h 59 m", date: "21 Nov 2026", share: 4,
      change: "+0.8 km", changeText: "0.8 km more than the longest run in the same 12 weeks, one week earlier (19.6 km on 31 Oct 2026)"
    },
    repeated: {
      name: "Repeated work", time: "2 h 15 m", share: 5, runs: 3, last: "15 Oct 2026",
      change: "−45 m", changeText: "45 m less than the same span, one week earlier (2 h 15 m against 3 h 00 m) · 3 runs against 4"
    },
    coverage: "58 of 58 runs recorded a heart rate"
  };

  /* ------------------------------------------------------------------
     5 · THE LOAD — the model's own numbers
     ------------------------------------------------------------------ */

  var load = {
    acute: 118, chronic: 84, form: 34, ratio: 1.40, k: 1.92,
    scenario: "with no training for 7 days, 1.40 is expected to come back inside 0.8 to 1.3 in about a day",
    band: [0.8, 1.3]
  };

  /* ------------------------------------------------------------------
     6 · THE RACE, THE GOAL, THE LADDER, THE CURVE
     the chain: a typed race time → VDOT → the race paces → the rules
     ------------------------------------------------------------------ */

  var race = {
    name: "10K", km: 10, sec: 2496, time: "41:36", date: "12 Oct 2026",
    enteredOn: "13 Oct 2026", changedOn: "3 Nov 2026", vdot: 49.6
  };
  var goal = { name: "a half marathon", km: 21.0975 };

  /* Riegel, exponent 1.06, from the typed 10K */
  function riegel(km, sec) { return sec * Math.pow(km / race.km, 1.06); }
  var equivalents = {
    "5 km": riegel(5, race.sec), "10 km": race.sec, "15 km": riegel(15, race.sec),
    "30 km": riegel(30, race.sec), "half marathon": riegel(goal.km, race.sec),
    "marathon": riegel(42.195, race.sec)
  };
  var equivText = {};
  Object.keys(equivalents).forEach(function (k) { equivText[k] = timeText(equivalents[k]); });

  var hmSec = equivalents["half marathon"];
  var hmPace = hmSec / goal.km;
  var k15 = equivalents["15 km"], k15Pace = k15 / 15;
  var mile10 = riegel(16.0934, race.sec), mile10Pace = mile10 / 16.0934;
  var marPace = equivalents["marathon"] / 42.195;
  var tenK = race.sec / race.km;

  var ladder = {
    source: "From your 10K in 41:36 on 12 Oct 2026 · entered 13 Oct 2026 · changed 3 Nov 2026.",
    rungs: [
      {
        name: "Threshold (LT)", value: paceText(k15Pace) + "–" + paceText(hmPace),
        rule: "for your half marathon", bandHr: [146, 167], bandPace: [k15Pace, hmPace],
        owner: "method", note: "the hour pace — the pace you could race for about an hour (ch. 1, ch. 7) — on the half-marathon span, because the adopted plan is a half"
      },
      {
        name: "The goal race", value: paceText(hmPace),
        rule: "a half marathon", bandHr: [155, 181], bandPace: [hmPace, hmPace],
        owner: "athlete", note: "the goal distance — its pace is the equivalent of the typed 10K (Riegel), not a race that was run"
      },
      {
        name: "Marathon pace", value: paceText(marPace),
        rule: "worked out from your race", bandHr: null, bandPace: [marPace, marPace],
        owner: "engine", note: "no method rule ships; no band"
      },
      {
        name: "Long run", value: paceText(tenK * 1.33) + " → " + paceText(tenK * 1.20),
        rule: "your 10K pace × 1.20–1.33", bandHr: [135, 154], bandPace: [tenK * 1.33, tenK * 1.20],
        owner: "method", note: "it starts at the slow end of the early band and points at the faster end of the latter"
      },
      {
        name: "General aerobic", value: "5:35–5:50",
        rule: "yours — no rule of its own", bandHr: [128, 148], bandPace: null,
        owner: "athlete", note: "70–81 % maxHR / 62–75 % HRR — no pace rule (ch. 7, ch. 8)"
      },
      {
        name: "Recovery", value: paceText(hmPace + 75),
        rule: "+75 s/km slower than your half-marathon pace", bandHr: [null, 138], bandPace: [hmPace + 75, hmPace + 75],
        owner: "method", note: "+120 s/mi (+75 s/km) off the 15K–HM race pace; below 76 % maxHR (ch. 2, ch. 7)"
      }
    ],
    legend: [
      { kind: "method", label: "the plan's band" },
      { kind: "athlete", label: "your own band" },
      { kind: "floor", label: "where your zones start" },
      { kind: "dot-method", label: "the plan's pace" },
      { kind: "dot-athlete", label: "your own pace" },
      { kind: "dot-engine", label: "worked out from your race" }
    ]
  };

  /* the physiological-cost curve — the shape the prototype drew, with this
     athlete's own races placed on it at their durations */
  var curve = {
    vdot: race.vdot,
    races: [
      { label: "your 10K of 41:36 on 12 Oct 2026", min: race.sec / 60, kind: "dot" },
      { label: "the goal race — a half marathon", min: hmSec / 60, kind: "ring" }
    ],
    sessions: [
      { label: "the lactate-threshold session", minFrom: 20, minTo: 40, kind: "rect" },
      { label: "the long run", minFrom: 98, minTo: 140, kind: "rect" },
      { label: "the recovery run", minFrom: 27, minTo: 45, kind: "line" }
    ],
    fieldTest: 45,
    equivalents: [
      { label: "5K", time: timeText(equivalents["5 km"]), pace: paceText(equivalents["5 km"] / 5) },
      { label: "10K", time: timeText(equivalents["10 km"]), pace: paceText(equivalents["10 km"] / 10) },
      { label: "15K", time: timeText(equivalents["15 km"]), pace: paceText(equivalents["15 km"] / 15) },
      { label: "HM", time: timeText(equivalents["half marathon"]), pace: paceText(hmPace) },
      { label: "marathon", time: timeText(equivalents["marathon"]), pace: paceText(marPace) }
    ],
    assumes: [
      "Your fitness number " + race.vdot + ", from your 10K in 41:36 on 12 Oct 2026 (entered 13 Oct 2026).",
      "The training plan's paces and durations — the threshold session at the hour-pace band for 20–40 minutes, the long run from 11 to 17 miles, the recovery run at 3–5 miles (book ch. 1, ch. 2, ch. 7).",
      "The times your 10K gives for the other distances: 5K " + timeText(equivalents["5 km"]) + " · 10K " + timeText(equivalents["10 km"]) + " · 15K " + timeText(equivalents["15 km"]) + " · half " + timeText(equivalents["half marathon"]) + " · marathon " + timeText(equivalents["marathon"]) + "."
    ]
  };

  /* ------------------------------------------------------------------
     7 · THE THRESHOLD — four sources, one choice, a dated history
     ------------------------------------------------------------------ */

  var threshold = {
    base: "the Pfitzinger 10K plan",
    chosen: "watch",
    chosenDate: "3 Nov 2026",
    chosenValue: zoneModel.floors[4] + " bpm",
    chosenSource: "the start of your watch's fifth zone · read from your watch on 3 Nov 2026",
    candidates: [
      {
        id: "watch", where: "your watch's zones",
        value: zoneModel.floors[4] + " bpm", hr: zoneModel.floors[4], pace: null,
        source: "the start of your watch's fifth zone · read from your watch on 3 Nov 2026",
        rule: null,
        limit: "the app cannot check what was put into the watch"
      },
      {
        id: "race", where: "from your 10K",
        value: paceText(k15Pace) + "–" + paceText(mile10Pace) + " /km", hr: null,
        paceLow: k15Pace, paceHigh: mile10Pace,
        source: "your 10K in 41:36 on 12 Oct 2026",
        rule: "the plan's rule — the pace you could race for about an hour: the band between your 15K pace (" + timeText(k15) + ") and your 10-mile pace (" + timeText(mile10) + ")",
        limit: "a race is a race, not a test — the pace carries that day's conditions"
      },
      {
        id: "test", where: "from a field test",
        value: "166 bpm · 4:22 /km", hr: 166, pace: 262,
        source: "your 45-minute time trial on 3 Mar 2026",
        rule: "the plan's rule — your pace and heart rate over the last 30 minutes",
        limit: "the estimate carries the run's conditions, the effort across the window, and the watch's heart-rate accuracy; the rule assumes a 45-minute time trial on a flat course"
      },
      {
        id: "typed", where: "entered by you",
        value: "168 bpm · 4:06–4:12 /km", hr: 168, paceLow: 246, paceHigh: 252,
        source: "entered on 3 Mar 2026, from a lab test",
        rule: null,
        limit: "the app cannot check what the number was taken from"
      }
    ],
    history: [
      { date: "3 Nov 2026", value: "165 bpm", source: "the start of your watch's fifth zone, read on 3 Nov 2026" },
      { date: "3 Mar 2026", value: "172 bpm", source: "entered on 3 Mar 2026, from a coach" }
    ],
    ownRuns: [
      { mins: 34, hr: 162 }, { mins: 39, hr: 164 }, { mins: 42, hr: 165 },
      { mins: 49, hr: 168 }, { mins: 56, hr: 170 }
    ],
    disagreement: [165, 168]
  };

  /* ------------------------------------------------------------------
     8 · THE PLAN — the adopted Pfitzinger 10K plan
     ------------------------------------------------------------------ */

  var plan = {
    base: "the Pfitzinger 10K plan",
    adopted: "30 Sep 2026",
    anchor: "your 10K in 41:36 on 12 Oct 2026",
    pacing: "165 bpm · 3 Nov 2026",
    pacingSource: "the threshold you chose from your watch's zones, on 3 Nov 2026",
    weekLabel: "Mon 23 Nov – Sun 29 Nov 2026",
    week: [
      { dow: 0, day: "Mon 23 Nov", kind: "Endurance", km: 8.0, pace: "5:12–5:38 /km", hr: "128–148 bpm", state: "recorded", actual: "5.0 km recorded" },
      { dow: 1, day: "Tue 24 Nov", kind: "Endurance", km: 8.0, pace: "5:12–5:38 /km", hr: "128–148 bpm", state: "recorded", actual: "6.0 km recorded", today: true },
      { dow: 2, day: "Wed 25 Nov", kind: "Endurance", km: 8.0, pace: "5:12–5:38 /km", hr: "128–148 bpm", state: "planned" },
      { dow: 3, day: "Thu 26 Nov", kind: "Intervals", km: 12.0, pace: "4:16–4:21 /km", hr: "146–167 bpm", state: "planned" },
      { dow: 4, day: "Fri 27 Nov", kind: "Endurance", km: 8.0, pace: "5:12–5:38 /km", hr: "128–148 bpm", state: "planned" },
      { dow: 5, day: "Sat 28 Nov", kind: "Long run", km: 18.0, pace: "5:32 → 5:00 /km", hr: "135–154 bpm", state: "planned" },
      { dow: 6, day: "Sun 29 Nov", kind: "Endurance", km: 6.0, pace: "5:12–5:38 /km", hr: "128–148 bpm", state: "planned" }
    ],
    alternatives: [
      { kind: "Endurance", km: 8.0, pace: "5:12–5:38 /km", hr: "128–148 bpm" },
      { kind: "Recovery", km: 6.0, pace: "5:36 /km", hr: "below 138 bpm" }
    ],
    priorLongRun: { day: "Sat 21 Nov", kind: "Long run", km: 18.0 }
  };

  /* ------------------------------------------------------------------
     9 · THE BODY — last night, and the patterns
     ------------------------------------------------------------------ */

  var body = {
    date: "24 Nov 2026",
    night: "23 – 24 Nov 2026",
    rest: { dir: "could be better", count: 2, of: 3 },
    signals: [
      { name: "Sleep", value: "6 h 10 m", dir: "could be better", tail: "1 h 10 m less than your usual", band: [370, 440], valueMin: 370 },
      { name: "Resting heart rate", value: "54 bpm", dir: "about usual", tail: "2 bpm above your usual", band: [50, 58], valueMin: 54 },
      { name: "HRV", value: "48 ms", dir: "could be better", tail: "9 ms below your usual", band: [45, 62], valueMin: 48 }
    ],
    sleepStages: { deep: 78, rem: 94, light: 174, awake: 24, total: 370 },
    /* the night as it happened: minutes from 23:00, and the stage */
    sleepBlocks: [
      { at: 0, min: 12, stage: "awake" }, { at: 12, min: 40, stage: "light" },
      { at: 52, min: 24, stage: "rem" }, { at: 76, min: 36, stage: "light" },
      { at: 112, min: 38, stage: "deep" }, { at: 150, min: 26, stage: "rem" },
      { at: 176, min: 44, stage: "light" }, { at: 220, min: 40, stage: "deep" },
      { at: 260, min: 30, stage: "rem" }, { at: 290, min: 42, stage: "light" },
      { at: 332, min: 14, stage: "rem" }, { at: 346, min: 12, stage: "light" },
      { at: 358, min: 12, stage: "awake" }
    ],
    sleepStart: "23:00",
    patterns: [
      { text: "On the 5 other nights you ran before 06:00, your sleep was 7 h 02 m — longer than your usual.", n: 5 },
      { text: "On the 6 nights before a session with intervals in it, your resting heart rate was 51 bpm — 3 bpm below your usual.", n: 6 },
      { text: "On the 4 nights after a long run, your HRV was 55 ms — 7 ms above your usual.", n: 4 }
    ],
    dates: [
      { date: "24 Nov", sleep: 370, rhr: 54, hrv: 48 },
      { date: "23 Nov", sleep: 342, rhr: 53, hrv: 44 },
      { date: "22 Nov", sleep: 428, rhr: 52, hrv: 58 },
      { date: "21 Nov", sleep: 405, rhr: 55, hrv: 51 },
      { date: "20 Nov", sleep: 388, rhr: 56, hrv: 47 },
      { date: "19 Nov", sleep: 361, rhr: 57, hrv: 45 },
      { date: "18 Nov", sleep: 440, rhr: 53, hrv: 59 },
      { date: "17 Nov", sleep: 412, rhr: 52, hrv: 54 }
    ]
  };

  /* ------------------------------------------------------------------
     10 · THE FEATURED RUN — the long run of 21 Nov 2026
     ------------------------------------------------------------------ */

  var featured = {
    id: "run-21-nov",
    title: "long run",
    date: "21 Nov 2026",
    dateLabel: "Sat 21 Nov",
    km: 20.4, min: 119, moving: "1 h 59 m", elapsed: "2 h 02 m",
    pace: 350, paceText: "5:50 /km", avgHr: 145, maxHr: 158,
    elevationGain: 86, elevationLoss: 82, cadence: 178,
    zones: { z1: 20, z2: 65, z3: 34, z4: 0, z5: 0, outside: 0, notMoving: 3 },
    splits: [
      { lap: 1, km: 4.1, sec: 1424, pace: 347, hr: 141 },
      { lap: 2, km: 4.1, sec: 1432, pace: 349, hr: 143 },
      { lap: 3, km: 4.1, sec: 1445, pace: 352, hr: 144 },
      { lap: 4, km: 4.1, sec: 1451, pace: 354, hr: 145 },
      { lap: 5, km: 4.0, sec: 1388, pace: 347, hr: 143 }
    ],
    form: [
      { name: "cadence", value: "178 spm", dir: "about usual", tail: "4 spm below your usual" },
      { name: "heart-rate variability", value: "4.8 %", dir: "about usual", tail: "0.3 points below your usual" },
      { name: "pace drift", value: "−0:06 /km", dir: "faster", tail: "second half minus first half" },
      { name: "heart-rate drift", value: "+3 bpm", dir: "higher", tail: "second half minus first half" },
      { name: "aerobic decoupling", value: "2.1 %", dir: "about usual", tail: "the heart rate's rise against the pace's" },
      { name: "stride length", value: "1.42 m", dir: "about usual", tail: "9 cm below your usual" },
      { name: "efficiency factor", value: "0.91", dir: "about usual", tail: "0.02 below your usual" }
    ],
    update: "The second half was faster than the first by 0:06 /km, and your heart rate was higher by 3 bpm over the run. Aerobic decoupling was 2.1 %.",
    updateFacts: ["pace drift −0:06 /km", "heart-rate drift +3 bpm", "aerobic decoupling 2.1 %", "lap 5 · 4.0 km"],
    againstPlan: "18.0 km planned · 20.4 km recorded · 2.4 km more than the 18 km target",
    note: "the last 5 km were into a headwind",
    ledgerRow: "Z1–Z2 — 1 h 25 m of this run's 1 h 59 m · Z4–Z5 0 m",
    ledgerNote: "the last run in Z4–Z5 was on 20 Oct 2026 — 35 days before the end of this period",
    sleepLink: "Sleep on 20 – 21 Nov 2026 — 6 h 45 m",
    /* the route: points in metres, relative to the start */
    route: [
      [0,0],[10,4],[22,10],[34,18],[48,24],[62,26],[74,22],[84,14],[88,4],[86,-6],
      [78,-14],[64,-18],[50,-18],[38,-14],[28,-8],[18,-4],[8,-2],[0,0]
    ],
    /* the chart series, for the one chart */
    hrSeries: [132,134,136,138,139,140,141,142,142,143,143,144,144,145,145,145,146,146,146,147,147,148,148,149,149,150,151,152,153,154],
    paceSeries: [344,344,345,346,347,347,348,348,349,349,350,350,351,351,352,352,353,353,354,354,355,355,356,356,357,357,358,358,359,359],
    altSeries: [0,2,5,9,14,20,24,27,29,28,25,21,17,13,10,8,7,9,12,15,17,18,16,13,10,7,5,3,1,0]
  };

  /* the film: the zone ribbon and the playhead of the same run */
  var film = {
    run: "21 Nov 2026",
    ribbon: [
      { z: "Z1", min: 20 }, { z: "Z2", min: 65 }, { z: "Z3", min: 34 }
    ],
    frames: 30,
    playhead: 19
  };

  /* ------------------------------------------------------------------
     11 · COMPARE — two runs, three weeks apart
     ------------------------------------------------------------------ */

  var compare = {
    basis: "Same distance, 3 weeks apart",
    a: {
      id: "run-21-nov", date: "21 Nov 2026", km: 20.4, min: 119, moving: "1 h 59 m",
      pace: "5:50 /km", hr: 145, elevation: 86, cadence: 178, decoupling: "2.1 %"
    },
    b: {
      id: "run-31-oct", date: "31 Oct 2026", km: 19.6, min: 114, moving: "1 h 54 m",
      pace: "5:49 /km", hr: 144, elevation: 74, cadence: 179, decoupling: "2.4 %"
    },
    metrics: [
      { name: "distance", a: "20.4 km", b: "19.6 km", diff: "+0.8 km" },
      { name: "time you were running", a: "1 h 59 m", b: "1 h 54 m", diff: "+5 m" },
      { name: "pace", a: "5:50 /km", b: "5:49 /km", diff: "−0:01 /km" },
      { name: "average heart rate", a: "145 bpm", b: "144 bpm", diff: "+1 bpm" },
      { name: "elevation gain", a: "86 m", b: "74 m", diff: "+12 m" },
      { name: "cadence", a: "178 spm", b: "179 spm", diff: "−1 spm" },
      { name: "aerobic decoupling", a: "2.1 %", b: "2.4 %", diff: "−0.3 points" }
    ]
  };

  /* ------------------------------------------------------------------
     12 · THE YEAR — the months, the calendar, what changed
     ------------------------------------------------------------------ */

  var year = {
    label: "2026",
    months: monthsSeries,
    calendar: {
      /* one entry per day of 2026 that the window touches */
      from: "1 Jan 2026", to: "31 Dec 2026",
      months: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
      days: buildYearDays()
    },
    changed: [
      { what: "distance", a: "1 676 km", b: "565 km" },
      { what: "runs", a: "187", b: "58" },
      { what: "the longest run", a: "22.4 km", b: "20.4 km" },
      { what: "the zone mix", a: "84 % low", b: "84 % low" }
    ],
    changedNote: "The two periods are different lengths — this is the same span of the year, and the nine months before it."
  };

  function buildYearDays() {
    var out = [];
    var first = Date.UTC(2026, 0, 1);
    var last = Date.UTC(2026, 11, 31);
    var byDay = {};
    runs.forEach(function (r) { byDay[r.off] = (byDay[r.off] || 0) + r.km; });
    for (var t = first; t <= last; t += 86400000) {
      var off = Math.round((t - ORIGIN) / 86400000);
      out.push({ off: off, km: byDay[off] || 0, inWindow: off >= 0 && off <= 78 });
    }
    return out;
  }

  /* ------------------------------------------------------------------
     13 · RUNS — the calendar, the filters, the compare tray
     ------------------------------------------------------------------ */

  var runsScreen = {
    month: "November 2026",
    filters: [
      { label: "search", value: "" },
      { label: "from", value: "7 Sep 2026" },
      { label: "through", value: "24 Nov 2026" },
      { label: "scope", value: "runs" }
    ],
    tray: { a: "21 Nov 2026 · 20.4 km", b: "31 Oct 2026 · 19.6 km", action: "compare these two" },
    import: "Import a FIT file",
    groups: [
      {
        label: "This week", runs: runs.filter(function (r) { return r.off >= 77; })
      },
      {
        label: "Last week", runs: runs.filter(function (r) { return r.off >= 70 && r.off < 77; })
      },
      {
        label: "16 – 22 Nov", runs: runs.filter(function (r) { return r.off >= 63 && r.off < 70; })
      }
    ]
  };

  /* ------------------------------------------------------------------
     14 · THE STATES — the absences the screens have to hold
     ------------------------------------------------------------------ */

  var states = {
    noZoneModel: {
      name: "No zones yet",
      says: "The intensity family has no value and says why, in the core's own words. The shape family does not depend on the model, so it still answers.",
      lines: [
        "No zones yet — your watch's heart-rate zones have not come through.",
        "No split, no zone times, no shares and no last dates. The runs themselves are still there."
      ]
    },
    noRace: {
      name: "No race entered yet",
      says: "The ladder and the curve are drawn empty, and each says what it needs and where that would come from.",
      lines: [
        "No race entered yet — the race times and the training paces are not worked out.",
        "A race is entered as distance, time and the day of the race; the name is optional."
      ]
    },
    noPlan: {
      name: "No plan adopted yet",
      says: "The plan's outline is gone from the pictures, the session row is absent, and each says why.",
      lines: [
        "No plan adopted yet — there are no sessions to show.",
        "The runs themselves are still drawn, as they are."
      ]
    },
    noThreshold: {
      name: "No threshold chosen yet",
      says: "Nothing is stored. Every answer that depends on a threshold is a no value with a reason, and the reasons name where the number would come from.",
      lines: [
        "No threshold chosen yet. Everything that needs one says what is missing.",
        "It could come from your watch's zones, a race, a field test, or a number you enter."
      ]
    },
    emptyWindow: {
      name: "No runs recorded",
      says: "One line, in the core's words — never 'rest': a day without a recorded run may be a forgotten watch.",
      lines: [
        "No runs recorded.",
        "The kinds of run, the split and your zones are not shown: there is nothing for them to hold."
      ]
    },
    noHeartRate: {
      name: "No heart rate recorded",
      says: "One family answers and the other does not. The shape rows come from distance, time and the watch's workout steps, and need no heart rate.",
      lines: [
        "No heart rate recorded — none of the runs in this period recorded a heart rate.",
        "Nothing was measured, so there are no zone rows — never 0 m."
      ]
    }
  };

  /* ------------------------------------------------------------------
     15 · THE PRODUCT'S OWN STRINGS
     The numbers the screens show, as words. They live here so that the
     markup holds no numbers of its own: end-to-end.html reads these.
     ------------------------------------------------------------------ */

  var copy = {
    today: {
      hero: {
        title: "The last run", date: "24 Nov 2026",
        dist: "6.0 km", time: "30 m", pace: "5:00 /km", hr: "134 bpm",
        kind: "endurance", band: "Z1–Z2", bandTime: "30 m",
        bandShare: "100 % of the time you were running", hour: "07:00"
      },
      figures: [
        ["distance", "6.0 km"], ["time you were running", "30 m"],
        ["pace", "5:00 /km"], ["average heart rate", "134 bpm"]
      ],
      week: {
        value: "11.0 km", change: "−5.4 km",
        note: "Mon 23 Nov and Tue 24 Nov, against the same two days a week earlier."
      },
      form: { value: "+34", detail: "the difference · last 7 days 118 · last 6 weeks 84 · 1.40 times the last 6 weeks" },
      habit: { streak: "4 days in a row", eddington: "12 km on 13 days",
        days: "58 days with a run recorded, in the last 12 weeks." },
      options: [
        { kind: "Endurance", km: "8.0 km", pace: "5:12–5:38 /km", hr: "128–148 bpm", day: "Tue 24 Nov" },
        { kind: "Recovery", km: "6.0 km", pace: "5:36 /km", hr: "below 138 bpm", day: "Tue 24 Nov" }
      ],
      night: { label: "Rest", dir: "could be better", count: "2 of 3 signals below your usual", date: "23 – 24 Nov 2026" },
      vitals: [
        { name: "Sleep", value: "6 h 10 m", dir: "could be better", tail: "1 h 10 m less than your usual" },
        { name: "Resting heart rate", value: "54 bpm", dir: "about usual", tail: "2 bpm above your usual" },
        { name: "HRV", value: "48 ms", dir: "could be better", tail: "9 ms below your usual" }
      ],
      sleep: { total: "6 h 10 m" },
      link: { text: "see all of it in Body →" }
    },
    progress: {
      window: { label: "7 Sep – 24 Nov 2026", sub: "last 12 weeks",
        line: "58 runs · 49 h 32 m of the time you were running · 564.8 km" }
    },
    runs: {
      filters: [["search", "—"], ["from", "7 Sep 2026"], ["through", "24 Nov 2026"], ["showing", "runs"]],
      tray: { a: "21 Nov 2026 · 20.4 km", b: "31 Oct 2026 · 19.6 km", action: "compare these two →" },
      listCount: "58 runs in the last 12 weeks"
    },
    run: {
      title: "Long run", date: "Sat 21 Nov 2026",
      headline: "20.4 km · 1 h 59 m · 5:50 /km · 145 bpm",
      rows: [["time you were running", "1 h 59 m"], ["the whole run", "2 h 02 m"],
             ["elevation gain", "86 m"], ["average cadence", "178 spm"]],
      sentence: "20.4 km in 1 h 59 m — the longest run of the last 12 weeks.",
      ledgerRow: "Z1–Z2 — 1 h 25 m of this run’s 1 h 59 m · Z4–Z5 0 m",

      cursor: { at: "14:20", distance: "10.2 km", pace: "5:48 /km", hr: "148 bpm", alt: "27 m" },
      effort: [["Z1", "20 m", "17 %"], ["Z2", "65 m", "55 %"], ["Z3", "34 m", "29 %"],
               ["Z4", "0 m", "0 %"], ["Z5", "0 m", "0 %"], ["outside", "0 m", "0 %"],
               ["not moving", "3 m", "—"]],
      splitsNote: "the watch’s own laps",
    },
    pace: {
      axes: "Heart rate on the left, from your own zones; pace on the right, from the race you entered. A heart rate and a pace are different things, and one is never turned into the other.",
      curveRoute: "/insights/pace-curve",
      rule: "The training plan’s heart-rate bands are percentages of your maximum heart rate. This curve is a different measure — the share of your maximum oxygen uptake a duration holds — so the plan’s bands are named here and never drawn on it."
    },
    threshold: {
      used: [["the plan’s pacing", "used"], ["the 80/20 plan’s easy-day bands", "not set yet"],
             ["the threshold pace on the training-paces card", "the plan’s own rule"],
             ["the zone times on your runs", "your own zones"]],
    },
    plan: {
      paces: [["Endurance", "5:12–5:38 /km"], ["Intervals", "4:16–4:21 /km"], ["Long run", "5:32 → 5:00 /km"]],
      source: "Your 10K in 41:36 on 12 Oct 2026.",
    },
    body: {
      signalsNote: "Each signal on its own: the value, its direction against your own usual band, and the arithmetic that produced the direction."
    },
    compare: {
      picker: "31 Oct 2026 · 19.6 km"
    },
    film: {},
    states: {
      noZoneModel: { title: "No zones yet", note: "your watch’s heart-rate zones have not come through." },
      noRace: { title: "No race entered yet", note: "the training paces are not worked out." },
      noPlan: { title: "No plan adopted yet", note: "there are no sessions to show." },
      noThreshold: { title: "No threshold chosen yet.", note: "it could come from your watch’s zones, a race, a field test, or a number you enter." },
      emptyWindow: { title: "No runs recorded." },
      noHeartRate: { title: "No heart rate recorded", note: "none of the runs in this period recorded a heart rate." }
    }
  };

  /* ------------------------------------------------------------------
     EXPORT
     ------------------------------------------------------------------ */

  var A = {
    meta: meta, zoneModel: zoneModel, weeks: weeks, prior: prior, runs: runs,
    weekAgg: weekAgg, bands: bandTotals, dayLabel: dayLabel, dateFull: dateFull,
    dur: dur, paceText: paceText, timeText: timeText, kmText: kmText, signedKm: signedKm, signedMin: signedMin,
    DAY: DAY, ledger: ledger, load: load, race: race, goal: goal, equivalents: equivalents,
    equivText: equivText, ladder: ladder, curve: curve, threshold: threshold, plan: plan,
    body: body, featured: featured, film: film, compare: compare, year: year,
    runsScreen: runsScreen, states: states, copy: copy, habit: habit, facets: facets, facetsPrior: facetsPrior,
    monthsSeries: monthsSeries,
    lastDayWhere: lastDayWhere
  };

  if (typeof window !== "undefined") { window.ATHLETE = A; }
  if (typeof module !== "undefined" && module.exports) { module.exports = A; }
})();
