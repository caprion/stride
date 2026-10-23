"""Normalise Garmin's planned workouts and shoe rotation for plan comparison."""
from __future__ import annotations
import re
from typing import Any
from .normalize import safe_float

class GarminWorkoutClient:
    def __init__(self, client: Any = None): self.client=client
    def _call(self,name,*args,**kwargs):
        try:
            fn=getattr(self.client,name,None); return fn(*args,**kwargs) if callable(fn) else None
        except Exception: return None
    @staticmethod
    def _items(x):
        if isinstance(x,list): return x
        if isinstance(x,dict):
            for k in ('workouts','scheduledWorkouts','calendarItems','items','gear'):
                if isinstance(x.get(k),list): return x[k]
        return []
    def scheduled_workouts(self,year:int,month:int)->list[dict]:
        items=self._call('get_scheduled_workouts', year, month)
        out=[]
        for x in self._items(items):
            if not isinstance(x,dict): continue
            out.append({'workout_id':x.get('workoutId') or x.get('id'),'name':x.get('workoutName') or x.get('name'),'date':x.get('date') or x.get('scheduledDate'),'sport':x.get('sportType') or x.get('sportTypeKey'),'steps':x.get('workoutSteps') or x.get('steps') or [],'status':x.get('status')})
        return out
    def workout_library(self,limit:int)->list[dict]: return [x for x in self._items(self._call('get_workouts',0,max(1,int(limit)))) if isinstance(x,dict)]
    def workout_detail(self,workout_id:int)->dict:
        p=self._call('get_workout_by_id',workout_id)
        if not isinstance(p,dict): return {}
        steps=p.get('workoutSteps') or p.get('steps') or []
        title=str(p.get('workoutName') or p.get('name') or '')
        text=(title+' '+' '.join(str(s.get('stepType') or s.get('description') or s.get('stepName') or '') for s in steps if isinstance(s,dict))).lower()
        typ='easy'
        if 'race' in text: typ='race'
        elif any(w in text for w in ('tempo','threshold','lactate')): typ='tempo'
        elif any(w in text for w in ('repeat','interval','recovery')) or len(steps)>2: typ='interval'
        elif 'strength' in text: typ='strength'
        elif 'rest' in text: typ='rest'
        elif 'recovery' in text: typ='recovery'
        dist=0; paces=[]
        for s in steps:
            if not isinstance(s,dict): continue
            dist+=safe_float(s.get('endConditionValue') if 'DISTANCE' in str(s.get('endCondition','')).upper() else 0,0) or 0
            for k in ('targetValueOne','targetValueTwo','targetPace','pace'):
                v=s.get(k)
                if isinstance(v,str) and ':' in v:
                    try: paces.append((int(v.split(':')[0])*60+float(v.split(':')[1]))*60)
                    except ValueError: pass
                elif (n:=safe_float(v)) is not None: paces.append(n*1000 if n<20 else n)
        if typ=='easy' and (dist>12000 or any((safe_float(s.get('endConditionValue'),0) or 0)>3600 for s in steps if isinstance(s,dict))): typ='long'
        return {'session_type':typ,'target_distance_m':dist or None,'target_pace_low_s_km':min(paces) if paces else None,'target_pace_high_s_km':max(paces) if paces else None,'hr_zone_target':None,'title':title,'details':str(steps)}
    def gear(self)->list[dict]:
        p=self._call('get_gear',getattr(self.client,'profile_id',''))
        out=[]
        for x in self._items(p):
            if isinstance(x,dict): out.append({'uuid':x.get('uuid') or x.get('gearUUID'),'name':x.get('name') or x.get('gearName'),'brand':x.get('brand'),'model':x.get('model'),'status':x.get('status') or 'active'})
        return out
