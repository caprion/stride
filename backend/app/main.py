"""Stride — FastAPI application entrypoint.

Serves the JSON API under ``/api/v1`` and, in production, the built SPA from
``frontend/dist`` on the same origin. One command, one port, no proxy config.
"""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, JSONResponse
from fastapi.staticfiles import StaticFiles

from .api import auth as auth_api
from .api import activities as activities_api
from .api import misc as misc_api
from .api import stats as stats_api
from .config import FRONTEND_DIST, state
from .db import SessionLocal, init_db

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)-7s %(name)s: %(message)s",
)
log = logging.getLogger("stride")

API_PREFIX = "/api/v1"


@asynccontextmanager
async def lifespan(app: FastAPI):
    init_db()
    _bootstrap()
    _start_scheduler()
    log.info("Stride is ready. Open http://localhost:%s", os.getenv("STRIDE_PORT", "8000"))
    yield


app = FastAPI(
    title="Stride",
    description="A Strava-class running webapp with native Garmin ingest.",
    version="1.0.0",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "http://localhost:8000",
        "http://127.0.0.1:8000",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(auth_api.router, prefix=API_PREFIX)
app.include_router(activities_api.router, prefix=API_PREFIX)
app.include_router(stats_api.router, prefix=API_PREFIX)
app.include_router(misc_api.calendar_router, prefix=API_PREFIX)
app.include_router(misc_api.sync_router, prefix=API_PREFIX)
app.include_router(misc_api.network_router, prefix=API_PREFIX)
app.include_router(misc_api.health_router, prefix=API_PREFIX)


def _mount_optional(module_name: str, attr: str = "router") -> None:
    """Register a router if the module is present.

    The Phase-2 routers are built by parallel workstreams, so the API must
    start even while some of them are still being written. A missing module is
    logged, not fatal.
    """
    import importlib

    try:
        module = importlib.import_module(module_name)
    except ModuleNotFoundError as exc:
        logging.getLogger("stride").warning(
            "Optional router %s is not available yet (%s)", module_name, exc
        )
        return
    router = getattr(module, attr, None)
    if router is None:
        return
    app.include_router(router, prefix=API_PREFIX)


_mount_optional("app.api.plan")
_mount_optional("app.api.physiology")
_mount_optional("app.api.fitness")
_mount_optional("app.api.compare")


def _bootstrap() -> None:
    """Seed the demo dataset on first run so the app is never empty."""
    from .services.persistence import bootstrap_demo

    db = SessionLocal()
    try:
        seeded = bootstrap_demo(db)
        if seeded:
            log.info("Seeded %d demo activities — the app is ready to explore.", seeded)
    except Exception as exc:  # pragma: no cover
        log.warning("Demo bootstrap failed: %s", exc)
    finally:
        db.close()
    _bootstrap_garmin()


def _bootstrap_garmin() -> None:
    """Connect Garmin from the environment or Key Vault if one is available.

    A Personal API Key is preferred over the other methods because it needs no
    developer registration and covers wellness as well as activities.
    """
    if state.garmin_connected and state.personal_api_key:
        return
    if os.getenv("STRIDE_KEYVAULT_ENABLED", "true").lower() in ("false", "0", "no"):
        # Tests and offline runs must never reach out to Azure.
        return
    from .secrets import resolve_garmin_personal_key

    key, source = resolve_garmin_personal_key()
    if not key:
        return
    try:
        from .ingest.garmin_personal import GarminPersonalClient

        client = GarminPersonalClient(key)
        client.validate()
    except Exception as exc:
        log.warning("Garmin key from %s did not validate: %s", source, exc)
        return
    state.update(
        personal_api_key=key,
        garmin_connected=True,
        garmin_method="personal",
    )
    log.info("Connected Garmin via the Personal API (key from %s).", source)


def _start_scheduler() -> None:
    """Periodic auto-sync."""
    if not state.auto_sync:
        return
    try:
        from apscheduler.schedulers.background import BackgroundScheduler

        from .services import sync

        scheduler = BackgroundScheduler()
        scheduler.add_job(
            sync.run_sync,
            "interval",
            minutes=state.sync_interval_minutes,
            id="garmin-sync",
            replace_existing=True,
        )
        scheduler.start()
        log.info("Auto-sync scheduled every %d minutes.", state.sync_interval_minutes)
    except Exception as exc:  # pragma: no cover
        log.warning("Could not start the scheduler: %s", exc)


@app.get(f"{API_PREFIX}/health")
def health() -> dict:
    return {"ok": True, "service": "stride"}


# ---------------------------------------------------------------- SPA -----

if FRONTEND_DIST.exists():
    assets = FRONTEND_DIST / "assets"
    if assets.exists():
        app.mount("/assets", StaticFiles(directory=assets), name="assets")

    @app.get("/{full_path:path}", include_in_schema=False)
    async def spa(full_path: str, request: Request):
        if full_path.startswith("api/"):
            return JSONResponse({"detail": "Not found"}, status_code=404)
        candidate = FRONTEND_DIST / full_path
        if full_path and candidate.is_file():
            return FileResponse(candidate)
        return FileResponse(FRONTEND_DIST / "index.html")

else:

    @app.get("/", include_in_schema=False)
    async def dev_root() -> dict:
        return {
            "service": "stride",
            "message": "The frontend has not been built yet.",
            "build": "cd frontend && npm install && npm run build",
            "dev": "cd frontend && npm run dev",
            "api": f"{API_PREFIX}/docs",
        }
