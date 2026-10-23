"""Sync Garmin wellness, gear and activity facts into Stride's frozen schema."""
from __future__ import annotations
from datetime import date,timedelta
from typing import Any
from ..db import SessionLocal
from ..models_phase2 import DailyPhysiology,Gear,ActivityFact
from ..ingest.garmin_account import GarminAccountClient,GarminAccountError
from ..ingest.garmin_health import GarminHealthClient
from ..ingest.garmin_workouts import GarminWorkoutClient
from ..config import state


def _account():
    try:
        c=GarminAccountClient(state.garmin_email or '',state.garmin_password or '',state.garmin_tokenstore or '')
        c.login(); return c
    except Exception: return None

def sync_health(days:int=30)->dict:
    account=_account()
    if account is None: return {'days':days,'days_written':0,'errors':['Garmin account unavailable']}
    client=account._client; health=GarminHealthClient(client); workouts=GarminWorkoutClient(client); errors=[]; written=0
    db=SessionLocal()
    try:
        for offset in range(max(0,days)):
            d=(date.today()-timedelta(days=offset)).isoformat()
            try:
                row={'date':d,'source':'garmin'}
                for method,mapping in [('sleep_for',{'duration_s':'sleep_s','score':'sleep_score','deep_s':'deep_sleep_s','rem_s':'rem_s','light_s':'light_sleep_s','awake_s':'awake_s'}),('body_battery',{'min':'body_battery_min','avg':'body_battery_avg','max':'body_battery_max'}),('stress',{'avg':'stress_avg','max':'stress_max'}),('hrv',{'value_ms':'hrv_ms','status':'hrv_status'}),('resting_hr',{'resting_hr':'resting_hr'}),('daily_summary',{'steps':'steps','vo2max':'vo2max'})]:
                    got=getattr(health,method)(d)
                    row.update({dest:got[src] for src,dest in mapping.items() if src in got and got[src] is not None})
                summary=health.training_status(); row.update({k:v for k,v in summary.items() if k in ('training_readiness','acute_load','chronic_load','vo2max','endurance_score','running_tolerance') and v is not None})
                existing=db.get(DailyPhysiology,d)
                if existing is None: existing=DailyPhysiology(date=d); db.add(existing)
                for k,v in row.items(): setattr(existing,k,v)
                written+=1
            except Exception as exc: errors.append(f'{d}: {exc}')
        for g in workouts.gear():
            try:
                if not g.get('uuid'): continue
                obj=db.query(Gear).filter_by(uuid=str(g['uuid'])).one_or_none()
                if obj is None: obj=Gear(uuid=str(g['uuid'])); db.add(obj)
                for k in ('name','brand','model','status'): setattr(obj,k,g.get(k))
            except Exception as exc: errors.append(f'gear: {exc}')
        db.commit()
    except Exception as exc: db.rollback(); errors.append(str(exc))
    finally: db.close()
    return {'days':days,'days_written':written,'errors':errors}

def _find(obj:Any,keys:set[str]):
    if isinstance(obj,dict):
        for k,v in obj.items():
            if k.lower() in keys and v is not None: return v
        for v in obj.values():
            found=_find(v,keys)
            if found is not None:return found
    elif isinstance(obj,list):
        for v in obj:
            found=_find(v,keys)
            if found is not None:return found
    return None

def sync_activity_facts(activity_ids:list[int])->dict:
    account=_account()
    if account is None:return {'written':0,'errors':['Garmin account unavailable']}
    db=SessionLocal(); errors=[]; written=0
    fields={'aerobic_te':{'aerobictrainingeffect','aerobictrainingeffectvalue'},'anaerobic_te':{'anaerobictrainingeffect','anaerobictrainingeffectvalue'},'training_effect_label':{'trainingeffectlabel'},'vo2max':{'vo2max'},'recovery_time_min':{'recoverytime','recoverytimeminutes'},'avg_gct_ms':{'averagegroundcontacttime','avggroundcontacttime'},'avg_stride_ratio':{'averagestridelength','avgstrideratio'},'vertical_ratio':{'verticalratio'},'avg_power':{'averagepower'},'normalized_power':{'normalizedpower'},'avg_grade_adjusted_pace':{'gradeadjustedpace'},'temperature_c':{'averagetemperature','temperature'},'impact_estimate':{'impactloading'},'gear_uuid':{'gearuuid'},'device_name':{'devicename'}}
    try:
        for aid in activity_ids:
            try:
                p=account._client.get_activity_details(aid)
                row=db.get(ActivityFact,aid) or ActivityFact(activity_id=aid)
                db.add(row)
                for dest,keys in fields.items():
                    val=_find(p,keys)
                    if val is not None:setattr(row,dest,val if dest in ('training_effect_label','gear_uuid','device_name') else float(val))
                written+=1
            except Exception as exc:errors.append(f'{aid}: {exc}')
        db.commit()
    except Exception as exc:db.rollback(); errors.append(str(exc))
    finally:db.close()
    return {'written':written,'errors':errors}

def sync_scheduled_workouts(year:int,month:int)->list[dict]:
    account=_account()
    return GarminWorkoutClient(account._client).scheduled_workouts(year,month) if account else []
