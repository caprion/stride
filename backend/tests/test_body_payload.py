"""The Body screen's payload: what the daily endpoints answer, and what they do not.

The Body screen is three things — today's readiness, sleep, and resting heart
rate with HRV. Body battery, stress and load come off it, and they come off
the payload with it: the daily endpoints used to answer with every Garmin
column of every day, and almost none of it was drawn.

These tests hold that trim in place, and they hold the readiness score where
it is: the score still counts the stress, the body battery and the load,
because a number that quietly changes meaning between two versions of the
app is worse than a screen with two extra series on it.
"""

from __future__ import annotations

import json
from datetime import date, timedelta
from pathlib import Path

import pytest

from app.api.physiology import BODY_FIELDS
from app.models_phase2 import DailyPhysiology

REPO = Path(__file__).resolve().parents[2]
SCENARIOS = REPO / "backend" / "tests" / "golden" / "scenarios" / "readiness.json"

#: The columns that are no longer drawn, and so are no longer sent.
REMOVED = (
    'max_hr',
    'stress_avg',
    'stress_max',
    'body_battery_min',
    'body_battery_avg',
    'body_battery_max',
    'steps',
    'training_readiness',
    'acute_load',
    'chronic_load',
    'vo2max',
    'endurance_score',
    'running_tolerance',
    'source',
    'updated_at',
)


def seed_year(db, days: int = 365) -> None:
    """One row per day, with values in every column that can hold one."""
    first = date.today() - timedelta(days=days - 1)
    for offset in range(days):
        day = (first + timedelta(days=offset)).isoformat()
        db.add(
            DailyPhysiology(
                date=day,
                sleep_s=27000.0,
                sleep_score=80.0,
                deep_sleep_s=5400.0,
                rem_s=4500.0,
                light_sleep_s=15000.0,
                awake_s=600.0,
                hrv_ms=55.0,
                hrv_status='BALANCED',
                resting_hr=48.0,
                max_hr=180.0,
                stress_avg=22.0,
                stress_max=60.0,
                body_battery_min=40.0,
                body_battery_avg=78.0,
                body_battery_max=90.0,
                steps=9000.0,
                training_readiness=70.0,
                acute_load=320.0,
                chronic_load=300.0,
                vo2max=52.0,
                endurance_score=6000.0,
                running_tolerance=40.0,
            )
        )
    db.flush()


def test_the_daily_endpoint_answers_only_the_body_fields(client, db):
    """The default answer is the ten fields the screen draws, and no others."""
    seed_year(db, days=3)
    response = client.get("/api/v1/physiology/daily")
    assert response.status_code == 200, response.text
    items = response.json()['items']
    assert items, 'the seeded days are missing'
    for item in items:
        assert set(item) == set(BODY_FIELDS)
        for removed in REMOVED:
            assert removed not in item


def test_a_named_field_is_still_answered(client, db):
    """A caller that asks for a field gets it, including one the screen dropped."""
    seed_year(db, days=3)
    response = client.get(
        "/api/v1/physiology/daily",
        params={'fields': 'date,stress_avg,body_battery_avg,acute_load,chronic_load,steps,source'},
    )
    assert response.status_code == 200, response.text
    for item in response.json()['items']:
        assert set(item) == {
            'date',
            'stress_avg',
            'body_battery_avg',
            'acute_load',
            'chronic_load',
            'steps',
            'source',
        }
        assert item['stress_avg'] == 22.0
        assert item['source'] == 'garmin'


def test_the_latest_day_answers_only_the_body_fields(client, db):
    """The latest day is trimmed the same way the daily rows are."""
    seed_year(db, days=3)
    response = client.get("/api/v1/physiology/latest")
    assert response.status_code == 200, response.text
    payload = response.json()
    assert set(payload) == {'day', 'readiness'}
    assert set(payload['day']) == set(BODY_FIELDS)


def test_a_year_of_days_is_seventy_kilobytes(client, db):
    """The payload of a year of days, measured rather than asserted.

    With every column populated, a row of every Garmin column is 520 bytes
    and a row of the ten fields the screen draws is 191: a year of the first
    is 189,811 bytes, a year of the second is 70,091 — and the endpoint
    answers the second.
    """
    seed_year(db, days=365)
    response = client.get("/api/v1/physiology/daily")
    assert response.status_code == 200, response.text
    body = response.content
    assert len(response.json()['items']) == 365
    assert len(body) < 75_000, f'the year of days is {len(body)} bytes'

    every_column = ','.join(column.name for column in DailyPhysiology.__table__.columns)
    every = client.get("/api/v1/physiology/daily", params={'fields': every_column})
    assert every.status_code == 200, every.text
    assert len(every.content) > 180_000, f'the full year is {len(every.content)} bytes'
    assert len(body) < len(every.content) * 0.4


def test_the_readiness_score_counts_only_the_four_shown_drivers(db):
    """Since v1 the score is the four drivers the screen draws, and nothing else."""
    from app.services import readiness

    scenario = next(
        case for case in json.loads(SCENARIOS.read_text())['scenarios'] if case['id'] == 'full_recovery'
    )
    for row in scenario['physiology']:
        db.add(DailyPhysiology(date=row['date'], **{k: v for k, v in row.items() if k != 'date'}))
    db.flush()
    result = readiness.compute_readiness(db, scenario['day'])
    assert result['score'] == pytest.approx(98.0)
    assert result['grade'] == 'peak'
    keys = {driver['key'] for driver in result['drivers']}
    assert keys == {'sleep', 'sleep_score', 'hrv', 'resting_hr'}


def test_the_screen_shows_four_of_the_seven_drivers():
    """The drivers the screen draws, and the three it hides."""
    from app.services.readiness import SHOWN_DRIVERS

    assert SHOWN_DRIVERS == ('sleep', 'sleep_score', 'hrv', 'resting_hr')
    assert set(SHOWN_DRIVERS) & {'stress', 'body_battery', 'load'} == set()
