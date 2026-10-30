"""Plan comparison logic; missing observations stay explicit instead of becoming scores."""
from __future__ import annotations

import statistics
from datetime import date, timedelta
from typing import Any, Mapping

from sqlalchemy import func, select
from sqlalchemy.orm import Session

from ..models import Activity
from ..models_phase2 import DailyPhysiology, PlanPhase, PlanResult, PlanSession, PlanWeek, RunFingerprint
from ..services.analytics import RUN_SPORTS

#: A session that has not happened yet. It is neither a hit nor a miss: it is
#: not evidence of anything, and it does not count in the adherence
#: denominator. A session dated in the future used to be reported as
#: "missed", which made the headline number wrong — and a future rest day
#: was reported as "hit", which was worse.
#:
#: The name is the same in the Rust core (``crates/stride-core/src/plan.rs``)
#: so the two implementations agree on the wire.
NOT_YET_DUE = "not_yet"

#: The verdicts a *prescribed* plan can produce. A baseline (an inferred
#: plan) never says "missed": nobody wrote it down, so nothing was missed.
PRESCRIBED_VERDICTS = ("hit", "missed_pace", "short", "missed", "substituted", "long")
BASELINE_VERDICTS = ("less_than_usual", "more_than_usual", "on_a_usual_day", "no_usual_session")


def _date(s: str) -> date: return date.fromisoformat(s[:10])
def _pace(a: Activity) -> float | None: return a.moving_time_s/(a.distance_m/1000) if a.distance_m and a.moving_time_s else None


def _today(today: date | str | None) -> date:
    """The day the plan is judged against.

    It is an argument, not a reading of the clock, so a test — and the
    golden fixtures — can say which day they mean.
    """
    if today is None:
        return date.today()
    if isinstance(today, date):
        return today
    return _date(str(today))


def _field(obj: Any, name: str, default: Any = None) -> Any:
    """Read a field from a row or from a plain dictionary."""
    if isinstance(obj, Mapping):
        value = obj.get(name, default)
    else:
        value = getattr(obj, name, default)
    return default if value is None else value


def _session_date(session: Any) -> date:
    return _date(str(_field(session, "date", "")))


def _not_yet_due() -> dict[str, Any]:
    """The verdict of a session that has not happened yet."""
    return {
        "verdict": NOT_YET_DUE,
        "distance_hit": False,
        "pace_band_hit": None,
        "distance_delta_m": None,
        "pace_delta_s_km": None,
        "adherence_score": None,
        "cause": None,
        "cause_confidence": None,
    }


def active_block_id(db: Session) -> int | None:
    """The id of the block in force, if there is one.

    Every plan query has to be scoped to it. A plan document can be imported
    more than once — each import archives the previous block and creates a new
    one — and the archived blocks often *overlap* the active one in date.
    Selecting sessions by date alone therefore returns each session once per
    overlapping block: the week strip shows every session twice, the
    adherence counts double, and the "missed" tally is fiction.
    """
    from ..models_phase2 import PlanBlock

    block = db.execute(
        select(PlanBlock)
        .where(PlanBlock.status == "active")
        .order_by(PlanBlock.start_date.desc())
    ).scalars().first()
    if block is None:
        block = db.execute(
            select(PlanBlock).order_by(PlanBlock.start_date.desc())
        ).scalars().first()
    return block.id if block is not None else None


def _planned(db: Session, start: str, end: str) -> list[PlanSession]:
    block_id = active_block_id(db)
    stmt = select(PlanSession).where(
        PlanSession.date >= start, PlanSession.date <= end
    )
    if block_id is not None:
        stmt = stmt.where(PlanSession.block_id == block_id)
    return list(db.execute(stmt.order_by(PlanSession.date, PlanSession.id)).scalars())


def match_sessions(db: Session, from_date: str, to_date: str, today: date | str | None = None, prescribed: bool = True) -> list[dict[str, Any]]:
    """The sessions of a range, with the verdict on each.

    ``today`` is the day the plan is judged against. A session dated after it
    has not happened yet: it is neither a hit nor a miss, and it is not
    evidence of anything.

    ``prescribed`` says what kind of plan this is. A prescribed plan is
    judged ``hit``/``missed``; a baseline — a plan inferred from the
    athlete's own habits — is judged ``less than usual`` and never says
    ``missed``.
    """
    today = _today(today)
    planned = _planned(db, from_date, to_date)
    acts = list(db.execute(select(Activity).where(Activity.local_date >= (date.fromisoformat(from_date)-timedelta(days=2)).isoformat(), Activity.local_date <= (date.fromisoformat(to_date)+timedelta(days=2)).isoformat())).scalars())
    claimed: set[int] = set(); out=[]
    for s in planned:
        target = date.fromisoformat(s.date); candidates = [a for a in acts if a.id not in claimed and a.local_date]
        same = [a for a in candidates if a.local_date == s.date and a.sport_type in RUN_SPORTS]
        chosen = min(same, key=lambda a: abs((a.start_time.date()-target).days)) if same else None
        if not chosen and s.session_type in ("rest", "strength"):
            wanted = [a for a in candidates if a.local_date == s.date and ("strength" in a.sport_type.lower() if s.session_type == "strength" else a.sport_type not in RUN_SPORTS)]
            chosen = wanted[0] if wanted else None
        if not chosen and s.session_type not in ("rest", "strength"):
            near = [a for a in candidates if a.sport_type in RUN_SPORTS and abs((_date(a.local_date)-target).days)<=2]
            if near: chosen=min(near,key=lambda a:(abs((_date(a.local_date)-target).days), abs((a.distance_m or 0)-(s.target_distance_m or a.distance_m))))
        if chosen: claimed.add(chosen.id)
        verdict = verdict_for_source(s, chosen, today=today, prescribed=prescribed)
        if s.session_type == "rest" and chosen is None and verdict["verdict"] != NOT_YET_DUE:
            # A logged absence is the intended outcome of a rest prescription.
            verdict = {"verdict":"hit","distance_hit":True,"pace_band_hit":None,"distance_delta_m":None,"pace_delta_s_km":None,"adherence_score":100.0,"cause":None,"cause_confidence":None}
        phys = db.get(DailyPhysiology, s.date)
        baseline = list(db.execute(select(DailyPhysiology).where(DailyPhysiology.date >= (target-timedelta(days=30)).isoformat(), DailyPhysiology.date < s.date)).scalars())
        def mean_metric(field):
            values=[getattr(r,field) for r in baseline if getattr(r,field) is not None]
            return statistics.mean(values) if values else None
        physiology = {"day":phys,"sleep_baseline_s":mean_metric("sleep_s"),"hrv_baseline_ms":mean_metric("hrv_ms"),"resting_hr_baseline":mean_metric("resting_hr")}
        if chosen:
            fp=db.get(RunFingerprint,chosen.id)
            prior=list(db.execute(select(RunFingerprint).join(Activity,Activity.id==RunFingerprint.activity_id).where(Activity.local_date >= (target-timedelta(days=30)).isoformat(),Activity.local_date < s.date)).scalars())
            cv=[x.cadence_cv for x in prior if x.cadence_cv is not None]
            physiology["cadence_cv"] = fp.cadence_cv if fp else None
            physiology["cadence_cv_baseline"] = statistics.mean(cv) if cv else None
        cause, confidence = explain_cause(s, chosen, physiology if phys or physiology.get("cadence_cv") is not None else None, verdict.get("verdict"))
        out.append({"session":_session_dict(s),"activity":_activity_dict(chosen) if chosen else None,**verdict,"cause":cause,"cause_confidence":confidence})
    return out


def _session_dict(s):
    return {"id":s.id,"date":s.date,"session_type":s.session_type,"title":s.title,"details":s.details,"target_distance_m":s.target_distance_m,"pace_low_s_km":s.pace_low_s_km,"pace_high_s_km":s.pace_high_s_km}
def _activity_dict(a):
    return {"id":a.id,"date":a.local_date,"name":a.name,"sport_type":a.sport_type,"distance_m":a.distance_m,"pace_s_km":_pace(a)}


def verdict_for(session: PlanSession, activity: Activity | None, today: date | str | None = None) -> dict[str, Any]:
    """The verdict on one session of a prescribed plan.

    A session that has not happened yet is ``not_yet``: it is neither a hit
    nor a miss, it does not count in the adherence denominator, and it is not
    evidence of anything. Reporting it as ``missed`` says the athlete failed
    to do something they have not been given the chance to do yet.

    The session may be a row or a plain dictionary, so the same function
    judges a stored plan and a plan a source produced.
    """
    today = _today(today)
    if _session_date(session) > today:
        return _not_yet_due()
    target=_field(session,"target_distance_m") or 0; actual=activity.distance_m if activity else None
    delta=(actual-target) if actual is not None and target else None
    pace=_pace(activity) if activity else None
    low=_field(session,"pace_low_s_km"); high=_field(session,"pace_high_s_km")
    band=bool(pace is not None and (low is None or pace>=low) and (high is None or pace<=high)) if low is not None or high is not None else None
    distance_hit=bool(actual is not None and (not target or actual>=target*.95 or abs(actual-target)<=actual*.05))
    session_type=_field(session,"session_type")
    if not activity: verdict="missed"
    elif session_type in ("rest","strength") and activity.sport_type in RUN_SPORTS: verdict="substituted"
    elif session_type=="tempo" and _looks_interval(activity): verdict="substituted"
    elif session_type=="interval" and _looks_long(activity, target): verdict="substituted"
    elif target and actual < target*.8: verdict="short"
    elif target and session_type!="long" and actual>target*1.3 and actual-target>=2000: verdict="long"
    elif distance_hit and band is False: verdict="missed_pace"
    elif distance_hit and band is not False: verdict="hit"
    else: verdict="short"
    dist_score=100 if not target or actual is None else max(0,min(100,actual/target*100))
    pace_score=100 if band is None else (100 if band else max(0,100-min(abs((pace or 0)-(low or high or 0))/60*10,100)))
    type_score=0 if verdict=="substituted" else 100
    score=None if not activity else round(.5*dist_score+.4*pace_score+.1*type_score,1)
    return {"verdict":verdict,"distance_hit":distance_hit if activity else False,"pace_band_hit":band,"distance_delta_m":delta,"pace_delta_s_km":(pace-statistics.mean([x for x in (low,high) if x is not None])) if pace is not None and (low is not None or high is not None) else None,"adherence_score":score,"cause":None,"cause_confidence":None}


def verdict_for_source(session: Any, activity: Activity | None, today: date | str | None = None, prescribed: bool = True) -> dict[str, Any]:
    """The verdict on one session, chosen by the kind of plan it belongs to.

    A **prescribed** plan — one somebody wrote down — is judged ``hit``,
    ``missed`` and the rest. A **baseline** — a plan inferred from the
    athlete's own habits — is judged ``less than usual``,
    ``more than usual`` and ``on a usual day``, and it never says
    ``missed``: nobody wrote it down, so nothing was missed.

    A session that has not happened yet is ``not_yet`` either way.
    """
    if _session_date(session) > _today(today):
        return _not_yet_due()
    if prescribed:
        return verdict_for(session, activity, today=today)
    from . import plan_sources

    session_dict = session if isinstance(session, Mapping) else _session_dict(session)
    baseline = plan_sources.baseline_verdict(session_dict, activity)
    return {
        "verdict": baseline.get("verdict", "on_a_usual_day"),
        "distance_hit": None,
        "pace_band_hit": None,
        "distance_delta_m": baseline.get("delta_m"),
        "pace_delta_s_km": baseline.get("pace_delta_s_km"),
        "adherence_score": None,
        "cause": None,
        "cause_confidence": None,
    }


def _looks_interval(a):
    """Whether the activity looks like an interval session.

    Laps alone are not evidence: a watch's default auto-lap splits every
    steady run into kilometre laps. What marks intervals is the *pace*
    alternating. Over the laps of 200 metres or more, the spread between the
    fastest and slowest lap is at least 15% of the median lap, and splitting
    at the middle of that spread, there are at least two fast laps, two slow
    laps and three switches between them (a warm-up and a cool-down around a
    steady run make only two). A name that says so also counts.
    """
    if any(x in (a.name or "").lower() for x in ("interval","repeats","fartlek")): return True
    paces=[l.duration_s/(l.distance_m/1000) for l in (a.laps or []) if l.distance_m and l.distance_m>=200 and l.duration_s and l.duration_s>0]
    if len(paces)<4: return False
    mid=statistics.median(paces); low=min(paces); high=max(paces)
    if mid<=0 or (high-low)/mid<.15: return False
    split=(low+high)/2; fast=[p<split for p in paces]
    switches=sum(1 for x,y in zip(fast,fast[1:]) if x!=y)
    return sum(fast)>=2 and len(fast)-sum(fast)>=2 and switches>=3
def _looks_long(a, target): return bool(a.distance_m>14000 and target and a.distance_m>target*1.25)


def explain_cause(session: PlanSession, activity: Activity | None, physiology: DailyPhysiology | None, verdict: str = "") -> tuple[str | None, float | None]:
    """Why a session went the way it did.

    A cause only makes sense for a session that did not go to plan. Attaching
    "short sleep" to a session that was hit exactly would be noise, so a
    clean result carries no cause at all.
    """
    if verdict in ("hit", "not_yet"):
        # A hit needs no explanation, and a session that has not happened yet
        # has nothing to explain: it is not evidence of anything.
        return None, None
    if not activity: return ("No activity was logged for the planned session.",0.0)
    if verdict=="long":
        # Nothing physiological explains running further than planned; the
        # point worth making is the distance itself.
        return ("Ran well past the planned distance; keep the next few days easy.",0.0)
    if not physiology: return ("Physiology data is unavailable, so the reason for this result is unknown.",0.0)
    factors=[]
    get = (lambda k: physiology.get(k)) if isinstance(physiology,dict) else (lambda k: getattr(physiology,k,None))
    day=get("day")
    def metric(field):
        if day is not None: return getattr(day,field,None)
        return get(field)
    sleep=get("sleep_baseline_s")
    sleep_s=metric("sleep_s")
    if sleep_s is not None and sleep is not None and sleep_s < sleep-3600: factors.append(("short sleep",.85))
    hrv=get("hrv_baseline_ms")
    hrv_day=metric("hrv_ms")
    if hrv_day is not None and hrv and hrv_day < hrv*.9: factors.append(("elevated fatigue",.7))
    rhr=get("resting_hr_baseline")
    rhr_day=metric("resting_hr")
    if rhr_day is not None and rhr is not None and rhr_day > rhr+4: factors.append(("illness or overreaching",.6))
    cv=get("cadence_cv"); cvbase=get("cadence_cv_baseline")
    if cv is not None and cvbase is not None and cv>cvbase: factors.append(("mechanical inefficiency",.55))
    pace=_pace(activity)
    if session.session_type=="race" and pace is not None and session.pace_low_s_km is not None and not session.pace_low_s_km<=pace<=(session.pace_high_s_km or session.pace_low_s_km): factors.append(("intended race-week load",.55))
    factors.sort(key=lambda x:x[1],reverse=True)
    if factors: return ("Likely influenced by "+" and ".join(x[0] for x in factors[:2])+".",max(x[1] for x in factors[:2]))
    if any(get(k) is not None for k in ("sleep_baseline_s","hrv_baseline_ms","resting_hr_baseline","cadence_cv_baseline")):
        return ("No measured factor explains this one.",0.0)
    if any(metric(k) is not None for k in ("sleep_s","hrv_ms","resting_hr")):
        return ("Physiology is available, but there is not enough baseline data to explain this result.",.1)
    return ("Physiology data is unavailable, so the result is unexplained and may reflect fitness or an off-plan effort.",.1)


def adherence(db: Session, from_date: str, to_date: str, today: date | str | None = None, prescribed: bool = True) -> dict[str, Any]:
    today = _today(today)
    matches=match_sessions(db,from_date,to_date,today,prescribed); weeks={}
    for row in matches:
        # A session that has not happened yet is not evidence of anything: it
        # does not count as planned, as completed, as missed or as hit.
        if row["verdict"] == NOT_YET_DUE: continue
        s=row["session"]; key=_date(s["date"]).isocalendar(); wk=f"{key.year}-W{key.week:02d}"
        w=weeks.setdefault(wk,{"week_key":wk,"start_date":(date.fromisocalendar(key.year,key.week,1)).isoformat(),"phase":None,"planned_km":0.,"actual_km":0.,"planned_sessions":0,"completed_sessions":0,"missed_sessions":0,"hit_sessions":0})
        w["planned_sessions"]+=1
        if s["target_distance_m"]: w["planned_km"]+=s["target_distance_m"]/1000
        if row["activity"]:
            w["completed_sessions"]+=1; w["actual_km"]+=row["activity"]["distance_m"]/1000
        else: w["missed_sessions"]+=1
        if row["verdict"] in ("hit","missed_pace"): w["hit_sessions"]+=1
    rows=list(weeks.values())
    for w in rows: w["delta_km"]=round(w["actual_km"]-w["planned_km"],2); w["adherence_pct"]=round(w["completed_sessions"]/w["planned_sessions"]*100,1) if w["planned_sessions"] else None
    due=[r for r in matches if r["verdict"] != NOT_YET_DUE]
    if not due: return {"overall_pct":None,"planned_km":0,"actual_km":0,"planned_sessions":0,"completed_sessions":0,"hit_sessions":0,"missed_sessions":0,"extra_sessions":0,"by_verdict":{},"rows":[]}
    counts={k:sum(1 for r in due if r["verdict"]==k) for k in ("hit","missed_pace","short","missed","substituted","long")}
    # Adherence is showing up: the share of due sessions that got a run. How
    # each run went is the verdict's job, not this percentage's.
    return {"overall_pct":round(sum(bool(r["activity"]) for r in due)/len(due)*100,1),"planned_km":round(sum(w["planned_km"] for w in rows),2),"actual_km":round(sum(w["actual_km"] for w in rows),2),"planned_sessions":len(due),"completed_sessions":sum(bool(r["activity"]) for r in due),"hit_sessions":counts["hit"]+counts["missed_pace"],"missed_sessions":counts["missed"],"extra_sessions":len(extra_activities(db,from_date,to_date)),"by_verdict":counts,"rows":rows}


def extra_activities(db: Session, from_date: str, to_date: str) -> list[dict[str, Any]]:
    acts=list(db.execute(select(Activity).where(Activity.local_date>=from_date,Activity.local_date<=to_date,Activity.sport_type.in_(RUN_SPORTS))).scalars())
    planned=_planned(db,from_date,to_date); claimed=set()
    for s in planned:
        options=[a for a in acts if a.id not in claimed and abs((_date(a.local_date)-_date(s.date)).days)<=2]
        if options: claimed.add(min(options,key=lambda a:abs((_date(a.local_date)-_date(s.date)).days)).id)
    return [{**_activity_dict(a),"verdict":"bonus"} for a in acts if a.id not in claimed]


def _session_name(session: Any) -> str:
    """The name of a session, for the words the screen reads.

    A session whose kind is not known is named by its title, or by
    "session": "unknown" is not a kind of session, and a sentence that says
    "the planned unknown session" reads as one.
    """
    title = getattr(session, "title", None)
    if title:
        return str(title)
    kind = str(getattr(session, "session_type", "") or "").strip()
    if not kind or kind.lower() == "unknown":
        return "session"
    return kind


def today(db: Session) -> dict[str, Any]:
    d=date.today().isoformat(); sessions=_planned(db,d,d); readiness=db.execute(select(DailyPhysiology).where(DailyPhysiology.date==d)).scalar_one_or_none()
    if not sessions: return {"date":d,"session":None,"week":None,"readiness":getattr(readiness,"training_readiness",None),"recommendation":"No plan is defined for today; add or import a plan to get a recommendation.","rationale":"There is no planned session to compare or adjust.","has_plan":False}
    s=sessions[0]; week=db.get(PlanWeek,s.week_id) if s.week_id else None; low=bool(readiness and readiness.training_readiness is not None and readiness.training_readiness<40)
    behind=False
    if week and week.target_km and week.start_date:
        logged=db.execute(select(func.coalesce(func.sum(Activity.distance_m),0)).where(Activity.local_date>=week.start_date,Activity.local_date<=d,Activity.sport_type.in_(RUN_SPORTS))).scalar_one()
        elapsed=max(1,(_date(d)-_date(week.start_date)).days+1)
        behind=(logged/1000) < week.target_km*min(elapsed/7,1)*.75
    adjust=low or behind
    why="readiness is low" if low else "this week's running volume is behind target"
    rec=f"Shorten today's session or swap quality for an easy run because {why}." if adjust else f"Complete the planned {_session_name(s)} session."
    rationale="; ".join(x for x,yes in (("readiness is below 40",low),("week-to-date running volume is behind its target",behind)) if yes) or "No low-readiness or behind-volume signal was available."
    return {"date":d,"session":_session_dict(s),"week":{"id":week.id,"week_number":week.week_number,"target_km":week.target_km} if week else None,"readiness":readiness.training_readiness if readiness else None,"recommendation":rec,"rationale":rationale,"has_plan":True}


def current(db: Session):
    block=db.execute(select(__import__("app.models_phase2",fromlist=["PlanBlock"]).PlanBlock).where(__import__("app.models_phase2",fromlist=["PlanBlock"]).PlanBlock.status=="active")).scalars().first()
    return None if not block else {"id":block.id,"name":block.name}


def week(db: Session, day: str):
    d=date.fromisoformat(day); start=(d-timedelta(days=d.weekday())).isoformat(); end=(d+timedelta(days=6-d.weekday())).isoformat()
    return {"start_date":start,"end_date":end,"items":[_session_dict(s) for s in _planned(db,start,end)]}
