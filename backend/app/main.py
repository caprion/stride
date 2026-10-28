"""Stride — FastAPI application entrypoint.

Serves the JSON API under ``/api/v1`` and, in production, the built SPA from
``frontend/dist`` on the same origin. One command, one port, no proxy config.
"""

from __future__ import annotations

import logging
from pathlib import Path
import os
from contextlib import asynccontextmanager
from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, JSONResponse, Response
from fastapi.staticfiles import StaticFiles

from .api import auth as auth_api
from .api import activities as activities_api
from .api import misc as misc_api
from .api import stats as stats_api
from .config import FRONTEND_DIST, state
from .db import SessionLocal, init_db
from .security import (
    BOOTSTRAP_PATH,
    SecurityMiddleware,
    auth_enabled,
    bootstrap_permitted,
    cors_origins,
    ensure_safe_bind_configuration,
    harden_storage,
    install_log_redaction,
    is_loopback_client,
    redacted_http_exception_handler,
    redact_text,
    set_capability_cookie,
)

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)-7s %(name)s: %(message)s",
)
log = logging.getLogger("stride")

API_PREFIX = "/api/v1"
_cors_origins = cors_origins()
install_log_redaction()


@asynccontextmanager
async def lifespan(app: FastAPI):
    ensure_safe_bind_configuration()
    init_db()
    # init_db may create the database and SQLite's WAL/SHM sidecars.
    harden_storage()
    _bootstrap()
    _start_scheduler()
    if not auth_enabled():
        log.warning(
            "STRIDE API AUTHENTICATION IS DISABLED. This is suitable only for local development/tests; "
            "never expose this process to a network."
        )
    log.info(
        "Stride is ready. Open http://%s:%s",
        os.getenv("STRIDE_BIND", "127.0.0.1"),
        os.getenv("STRIDE_PORT", "8000"),
    )
    yield


app = FastAPI(
    title="Stride",
    description=(
        "A private, local-first running log with native Garmin ingest. "
        "Your data stays on your machine; sharing is Garmin's business, not ours."
    ),
    version="1.0.0",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=_cors_origins,
    # The production SPA is same-origin. A configured development origin gets
    # credentialed CORS only while the capability gate is enabled.
    allow_credentials=auth_enabled(),
    allow_methods=["*"],
    allow_headers=["*"],
)
app.add_middleware(SecurityMiddleware, allowed_origins=_cors_origins)
app.add_exception_handler(HTTPException, redacted_http_exception_handler)

app.include_router(auth_api.router, prefix=API_PREFIX)
app.include_router(activities_api.router, prefix=API_PREFIX)
app.include_router(stats_api.router, prefix=API_PREFIX)
app.include_router(misc_api.calendar_router, prefix=API_PREFIX)
app.include_router(misc_api.sync_router, prefix=API_PREFIX)
app.include_router(misc_api.network_router, prefix=API_PREFIX)
app.include_router(misc_api.health_router, prefix=API_PREFIX)


@app.get(BOOTSTRAP_PATH, status_code=204, include_in_schema=False)
def bootstrap_capability(request: Request) -> Response:
    """Set the HttpOnly browser capability without ever returning it.

    The middleware permits this endpoint without prior API authentication, but
    only a loopback browser at Stride's own origin can use it. Tailnet clients
    must already possess and send the explicit bearer capability.
    """
    if auth_enabled() and not bootstrap_permitted(request, _cors_origins):
        raise HTTPException(status_code=403, detail="Local same-origin bootstrap required.")
    response = Response(status_code=204)
    if auth_enabled():
        set_capability_cookie(response, request)
    return response


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
_mount_optional("app.api.tiles")


def _bootstrap() -> None:
    """Prepare a fresh database, then connect Garmin.

    The demo dataset is only seeded when explicitly asked for; a production
    instance starts empty and fills from the athlete's own Garmin account.
    """
    from .services.persistence import bootstrap_demo

    db = SessionLocal()
    try:
        seeded = bootstrap_demo(db)
        if seeded:
            log.info("Seeded %d demo activities — the app is ready to explore.", seeded)
    except Exception as exc:  # pragma: no cover
        log.warning("Demo bootstrap failed: %s", redact_text(exc))
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
        log.warning("Garmin key from %s did not validate: %s", source, redact_text(exc))
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
        log.warning("Could not start the scheduler: %s", redact_text(exc))


@app.get(f"{API_PREFIX}/health")
def health() -> dict:
    """Readiness for the supervisor: is it up, and is it actually useful?

    Reports the shape of the instance without leaking anything — the key's
    source, never the key.
    """
    from .models import Activity
    from .models_phase2 import DailyPhysiology, PlanSession

    db = SessionLocal()
    try:
        activities = db.query(Activity).count()
        real = db.query(Activity).filter(Activity.source != "demo").count()
        physiology = db.query(DailyPhysiology).count()
        sessions = db.query(PlanSession).count()
    except Exception:  # pragma: no cover - a broken database is not healthy
        return {"ok": False, "service": "stride", "error": "database unavailable"}
    finally:
        db.close()

    return {
        "ok": True,
        "service": "stride",
        "garmin": {
            "connected": state.garmin_connected,
            "method": state.garmin_method,
            "key_source": "key_vault" if state.personal_api_key else "none",
            "last_sync_at": state.last_sync_at,
        },
        "data": {
            "activities": activities,
            "from_garmin": real,
            "physiology_days": physiology,
            "plan_sessions": sessions,
        },
    }


# ------------------------------------------------------- asset links ----

ASSETLINKS_PATH = Path(__file__).with_name("assetlinks.json")


@app.get("/.well-known/assetlinks.json", include_in_schema=False)
async def assetlinks(request: Request) -> Response:
    """Prove this origin is allowed to launch the Android app.

    A Trusted Web Activity will only open as a real app — full screen, no
    browser, with the app's own icon and splash — if the origin can show
    this file and it names the app's package and signing certificate.

    Without it the APK opens a browser tab that looks like the app, which is
    worse than no app at all.

    The file is only served where it can be *used*: Digital Asset Links is
    defined over HTTPS, so a copy handed out over the cleartext listener is
    inert. The contents are public anyway — the package name and the
    signing certificate are in every APK — but there is no reason to put
    them on the wire in the clear for the local network.
    """
    if request.url.scheme != "https" and not is_loopback_client(request):
        return JSONResponse({"detail": "Not found"}, status_code=404)
    if not ASSETLINKS_PATH.exists():
        return JSONResponse({"detail": "Not found"}, status_code=404)
    return Response(
        content=ASSETLINKS_PATH.read_text(),
        media_type="application/json",
        headers={"Cache-Control": "public, max-age=3600"},
    )


# ---------------------------------------------------------------- SPA -----

def _dist_file(full_path: str) -> Path | None:
    """Resolve a request path to a file inside ``frontend/dist``.

    Returns ``None`` when the path escapes the directory — by ``..``, by
    being absolute, or through a symlink — and when it is not a file.
    """
    if not full_path:
        return None
    try:
        root = FRONTEND_DIST.resolve(strict=False)
        candidate = (root / full_path.lstrip("/")).resolve(strict=False)
    except (OSError, ValueError):
        return None
    if candidate != root and root not in candidate.parents:
        return None
    if not candidate.is_file():
        return None
    return candidate


if FRONTEND_DIST.exists():
    assets = FRONTEND_DIST / "assets"
    if assets.exists():
        app.mount("/assets", StaticFiles(directory=assets), name="assets")

    @app.get("/{full_path:path}", include_in_schema=False)
    async def spa(full_path: str, request: Request):
        if full_path.startswith("api/"):
            return JSONResponse({"detail": "Not found"}, status_code=404)
        # The SPA catch-all serves files out of frontend/dist. The path comes
        # straight off the wire, percent-decoded, so it can hold "..", an
        # absolute path, or a symlink pointing anywhere. Everything has to be
        # resolved and then checked to be *inside* the directory before it is
        # served — otherwise this route is an unauthenticated read of any
        # file the process can open, including the state file with the
        # Garmin key and the capability token in it.
        candidate = _dist_file(full_path)
        if candidate is not None:
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


if __name__ == "__main__":  # pragma: no cover - convenience entrypoint
    import uvicorn

    uvicorn.run(
        "app.main:app",
        host=os.getenv("STRIDE_BIND", "127.0.0.1"),
        port=int(os.getenv("STRIDE_PORT", "8000")),
    )
