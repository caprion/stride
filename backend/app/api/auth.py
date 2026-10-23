"""Authentication + ingest endpoints.

Three ways to get data in, all first-class:

* **Developer key** — paste a Garmin Consumer Key/Secret, authorise in the
  browser, done. No password ever touches our disk beyond the secret itself.
* **Account** — email + password, with an inline MFA step.
* **FIT upload** — no credentials at all.
"""

from __future__ import annotations

import logging
import os
import threading
import time
import uuid
from typing import Any

from fastapi import APIRouter, Depends, File, HTTPException, Query, UploadFile
from fastapi.responses import HTMLResponse
from pydantic import BaseModel, Field
from sqlalchemy.orm import Session

from ..config import DATA_DIR, state
from ..db import get_db
from ..ingest.fit_importer import parse_fit
from ..ingest.garmin_account import GarminAccountClient, GarminAccountError, MfaBroker
from ..ingest.garmin_oauth import GarminAuthError, GarminOAuthClient
from ..services.persistence import store_activity
from .. import network

log = logging.getLogger(__name__)
router = APIRouter(prefix="/auth", tags=["auth"])

# Pending MFA logins, keyed by an opaque id handed to the browser.
_mfa_sessions: dict[str, dict[str, Any]] = {}
_mfa_lock = threading.Lock()


class DeveloperKeyBody(BaseModel):
    consumer_key: str = Field(min_length=8)
    consumer_secret: str = Field(min_length=8)
    # Where Garmin should send the athlete after they approve. "oob" makes
    # Garmin display the verification code on a page instead of redirecting.
    callback: str | None = None


class ExchangeBody(BaseModel):
    oauth_token: str = ""
    oauth_verifier: str = ""


class LoginBody(BaseModel):
    email: str = Field(min_length=3)
    password: str = Field(min_length=1)


class MfaBody(BaseModel):
    mfa_token: str
    code: str = Field(min_length=4)


# ------------------------------------------------------------------ status


@router.get("/status")
def auth_status() -> dict[str, Any]:
    payload = state.public_status()
    payload["network"] = network.status()
    return payload


# --------------------------------------------------------- developer key


@router.post("/garmin/developer-key")
def start_developer_key(body: DeveloperKeyBody) -> dict[str, Any]:
    """Start the OAuth1 three-legged handshake and return the authorise URL."""
    client = GarminOAuthClient(body.consumer_key, body.consumer_secret)
    callback = body.callback or default_callback()
    try:
        request_token, request_secret = client.request_token(callback)
    except GarminAuthError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc

    state.update(
        consumer_key=body.consumer_key,
        consumer_secret=body.consumer_secret,
        pending_oauth_token=request_token,
        pending_oauth_token_secret=request_secret,
    )
    return {
        "authorize_url": client.authorize_url(request_token),
        "request_token": request_token,
        "callback": callback,
        "instructions": (
            "Open the link, sign in to Garmin, and approve the application. "
            "Garmin will send you back here with an authorisation code — "
            "paste it below if it is shown to you."
        ),
    }


def default_callback() -> str:
    """The URL Garmin redirects to after approval."""
    base = os.getenv("STRIDE_PUBLIC_URL", "http://localhost:8000").rstrip("/")
    return f"{base}/api/v1/auth/garmin/callback"


@router.get("/garmin/callback")
def garmin_callback(
    oauth_token: str = Query(default=""),
    oauth_verifier: str = Query(default=""),
):
    """OAuth1 redirect target. Garmin lands here after the athlete approves."""
    if not oauth_verifier:
        return _callback_page(
            "Almost there",
            "Garmin did not include an authorisation code. Start the connection again "
            "and copy the code Garmin shows you.",
            success=False,
        )
    try:
        payload = exchange(ExchangeBody(oauth_token=oauth_token, oauth_verifier=oauth_verifier))
    except HTTPException as exc:
        return _callback_page("Connection failed", str(exc.detail), success=False)
    return _callback_page(
        "Garmin connected",
        "Your watch data will sync automatically. You can close this tab.",
        success=True,
    )


def _callback_page(title: str, message: str, success: bool) -> HTMLResponse:
    accent = "#1a9c5a" if success else "#d93a2b"
    return HTMLResponse(
        f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"/>
<title>{title} · Stride</title>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<style>
 body{{margin:0;display:grid;place-items:center;min-height:100vh;background:#f7f7f7;
      font-family:Inter,-apple-system,'Segoe UI',Roboto,sans-serif;color:#121212}}
 .card{{background:#fff;border:1px solid #e6e6e6;border-radius:10px;padding:32px 36px;
       max-width:420px;text-align:center;box-shadow:0 4px 16px rgba(0,0,0,.08)}}
 .dot{{width:44px;height:44px;border-radius:50%;background:{accent};margin:0 auto 14px;
      display:grid;place-items:center;color:#fff;font-size:22px;font-weight:700}}
 h1{{font-size:19px;margin:0 0 8px}} p{{font-size:14px;color:#6d7176;margin:0;line-height:1.5}}
 a{{display:inline-block;margin-top:18px;background:#fc5200;color:#fff;text-decoration:none;
   padding:9px 18px;border-radius:6px;font-size:14px;font-weight:600}}
</style></head>
<body><div class="card"><div class="dot">{'✓' if success else '!'}</div>
<h1>{title}</h1><p>{message}</p>
<a href="/">Open Stride</a></div></body></html>"""
    )


@router.post("/garmin/exchange")
def exchange(body: ExchangeBody) -> dict[str, Any]:
    """Complete the handshake with the verifier Garmin showed the athlete."""
    if not state.pending_oauth_token:
        raise HTTPException(status_code=400, detail="No authorisation is in progress.")
    if not body.oauth_verifier:
        raise HTTPException(status_code=400, detail="An authorisation code is required.")
    if body.oauth_token and body.oauth_token != state.pending_oauth_token:
        raise HTTPException(status_code=400, detail="Authorisation token mismatch.")

    client = GarminOAuthClient(state.consumer_key or "", state.consumer_secret or "")
    try:
        token, secret = client.access_token(
            state.pending_oauth_token,
            state.pending_oauth_token_secret or "",
            body.oauth_verifier,
        )
    except GarminAuthError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc

    state.update(
        oauth_token=token,
        oauth_token_secret=secret,
        pending_oauth_token=None,
        pending_oauth_token_secret=None,
        garmin_connected=True,
        garmin_method="developer_key",
    )
    return {"ok": True, "garmin_method": "developer_key"}


# ---------------------------------------------------------------- account


@router.post("/garmin/login")
def login(body: LoginBody) -> dict[str, Any]:
    """Log in with email + password. May return ``mfa_required``."""
    tokenstore = str(DATA_DIR / "garmin_tokens")
    broker = MfaBroker()
    session_id = uuid.uuid4().hex

    result: dict[str, Any] = {}
    done = threading.Event()

    def worker() -> None:
        client = GarminAccountClient(body.email, body.password, tokenstore)
        try:
            profile = client.login(broker)
            result["profile"] = profile
            result["ok"] = True
        except Exception as exc:
            result["error"] = str(exc)
        finally:
            done.set()

    thread = threading.Thread(target=worker, daemon=True)
    thread.start()

    # Give Garmin time to respond; it either succeeds or demands an MFA code.
    deadline = time.monotonic() + 90.0
    while not done.is_set():
        if broker.requested.is_set():
            break
        if time.monotonic() > deadline:
            break
        done.wait(0.3)
    if done.is_set():
        if result.get("ok"):
            state.update(
                garmin_email=body.email,
                garmin_password=body.password,
                garmin_connected=True,
                garmin_method="account",
                garmin_athlete={"name": result["profile"].get("full_name") or body.email},
            )
            return {"status": "ok"}
        raise HTTPException(status_code=401, detail=result.get("error") or "Login failed.")

    if broker.requested.is_set():
        with _mfa_lock:
            _mfa_sessions[session_id] = {"broker": broker, "result": result, "done": done}
        state.update(garmin_email=body.email, garmin_password=body.password)
        return {"status": "mfa_required", "mfa_token": session_id}

    raise HTTPException(status_code=504, detail="Garmin did not respond in time.")


@router.post("/garmin/mfa")
def submit_mfa(body: MfaBody) -> dict[str, Any]:
    with _mfa_lock:
        session = _mfa_sessions.get(body.mfa_token)
    if not session:
        raise HTTPException(status_code=404, detail="Unknown or expired MFA session.")
    session["broker"].submit_code(body.code.strip())
    if not session["done"].wait(120):
        raise HTTPException(status_code=504, detail="Garmin did not accept the code in time.")
    result = session["result"]
    with _mfa_lock:
        _mfa_sessions.pop(body.mfa_token, None)
    if not result.get("ok"):
        raise HTTPException(status_code=401, detail=result.get("error") or "MFA login failed.")
    state.update(
        garmin_connected=True,
        garmin_method="account",
        garmin_athlete={"name": (result.get("profile") or {}).get("full_name") or state.garmin_email},
    )
    return {"status": "ok"}


@router.post("/garmin/logout")
def logout() -> dict[str, Any]:
    state.update(
        garmin_connected=False,
        garmin_method=None,
        oauth_token=None,
        oauth_token_secret=None,
        pending_oauth_token=None,
        pending_oauth_token_secret=None,
    )
    return {"ok": True}


# ------------------------------------------------------------- fit import


@router.post("/garmin/fit-import")
async def fit_import(files: list[UploadFile] = File(...), db: Session = Depends(get_db)) -> dict[str, Any]:
    """Import one or more ``.FIT`` files. No Garmin credentials required."""
    imported: list[dict[str, Any]] = []
    errors: list[dict[str, str]] = []

    for upload in files:
        filename = upload.filename or "activity.fit"
        try:
            data = await upload.read()
            activity = parse_fit(data, filename=filename)
            row, created = store_activity(db, activity)
            imported.append(
                {
                    "id": row.id,
                    "name": row.name,
                    "date": row.local_date,
                    "distance_m": row.distance_m,
                    "created": created,
                }
            )
        except Exception as exc:
            log.warning("FIT import failed for %s: %s", filename, exc)
            errors.append({"filename": filename, "error": str(exc)[:300]})
    db.commit()
    return {"imported": len(imported), "activities": imported, "errors": errors}
