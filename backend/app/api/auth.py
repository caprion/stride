"""Authentication + ingest endpoints.

Three ways to get data in, all first-class:

* **Developer key** — paste a Garmin Consumer Key/Secret, authorise in the
  browser, done. No password ever touches our disk beyond the secret itself.
* **Account** — email + password, with an inline MFA step.
* **FIT upload** — no credentials at all.
"""

from __future__ import annotations

import hashlib
import hmac
import logging
import os
import secrets
import threading
import time
import uuid
from datetime import datetime, timezone
from typing import Any

from fastapi import APIRouter, Depends, File, HTTPException, Query, Request, Response, UploadFile
from fastapi.responses import HTMLResponse
from pydantic import BaseModel, Field
from sqlalchemy.orm import Session

from ..config import DATA_DIR, state
from ..db import get_db
from ..security import is_loopback_client, set_capability_cookie, set_device_cookie
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


class PersonalKeyBody(BaseModel):
    api_key: str = Field(min_length=20)


# ---------------------------------------------------------------- personal


@router.post("/garmin/personal-key")
def connect_personal_key(body: PersonalKeyBody) -> dict[str, Any]:
    """Connect with a Garmin Connect Personal API Key.

    This is the recommended path: no developer registration, no consumer
    secret, and it covers activities *and* wellness (sleep, HRV, body
    battery, resting heart rate).
    """
    from ..ingest.garmin_personal import GarminPersonalClient, PersonalApiError

    client = GarminPersonalClient(body.api_key.strip())
    try:
        result = client.validate()
    except PersonalApiError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    except Exception as exc:  # pragma: no cover - network
        raise HTTPException(status_code=502, detail=f"Could not reach Garmin: {exc}") from exc

    state.update(
        personal_api_key=body.api_key.strip(),
        garmin_connected=True,
        garmin_method="personal",
    )
    profile = result.get("profile") or {}
    return {
        "ok": True,
        "garmin_method": "personal",
        "athlete": _athlete_from_personal(profile),
        "sources": result.get("sources") or {},
    }


def _athlete_from_personal(profile: dict[str, Any]) -> dict[str, Any]:
    """Pull a display name out of Garmin's profile payload, whatever it is called."""
    for key in ("displayName", "fullName", "userName", "name"):
        value = profile.get(key)
        if value:
            return {"name": str(value)}
    return {}


@router.post("/garmin/personal-key/test")
def test_personal_key(body: PersonalKeyBody) -> dict[str, Any]:
    """Validate a key without storing it."""
    from ..ingest.garmin_personal import GarminPersonalClient, PersonalApiError

    client = GarminPersonalClient(body.api_key.strip())
    try:
        result = client.validate()
    except PersonalApiError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    return {
        "ok": True,
        "athlete": _athlete_from_personal(result.get("profile") or {}),
    }


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
        # The Personal API key is a long-lived credential; drop it on
        # disconnect rather than leaving it lying in the state file.
        personal_api_key=None,
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


# --------------------------------------------------------------- pairing
#
# A browser on the tailnet is not a loopback client, so loading the app
# cannot mint a capability cookie — by design, because "any device on my
# tailnet can read my health data" is not a property anyone wants. The
# missing half was a way for the athlete to *pair* a device deliberately.
#
# The operator runs ``scripts/prod.sh pair`` on the machine, which asks the
# server for a fresh code over loopback. The code is short, short-lived and
# single-use. It is exchanged for the same HttpOnly, SameSite=Strict cookie
# that a loopback browser receives — so the device ends up with exactly the
# same authority, and the code itself never becomes a long-lived secret that
# JavaScript has to hold.


PAIR_CODE_TTL_SECONDS = 600
_PAIR_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"  # no look-alikes


def _new_pair_code() -> str:
    """A code that is easy to read aloud and type on a phone keyboard."""
    return "-".join(
        "".join(secrets.choice(_PAIR_ALPHABET) for _ in range(4))
        for _ in range(2)
    )


def _pair_code_valid(code: str) -> bool:
    expected = state.pairing_code
    expires = state.pairing_expires_at
    if not expected or not expires:
        return False
    if time.time() > float(expires):
        return False
    return hmac.compare_digest(str(code).strip().upper(), str(expected).upper())


@router.get("/pair-code")
def pair_code(request: Request) -> dict[str, Any]:
    """Mint a pairing code. Loopback only — this is the operator's console."""
    if not is_loopback_client(request):
        raise HTTPException(
            status_code=403,
            detail="Pairing codes are issued on the Stride machine only.",
        )
    code = _new_pair_code()
    expires = time.time() + PAIR_CODE_TTL_SECONDS
    state.update(pairing_code=code, pairing_expires_at=str(expires))
    log.info("pairing code issued; valid for %d minutes", PAIR_CODE_TTL_SECONDS // 60)
    return {
        "code": code,
        "expires_in": PAIR_CODE_TTL_SECONDS,
        "instructions": (
            "Open Stride on the other device and enter this code. "
            "It works once, and expires in 10 minutes."
        ),
    }


class PairBody(BaseModel):
    code: str = Field(..., min_length=4, max_length=32)


@router.post("/pair")
def pair(body: PairBody, request: Request) -> Response:
    """Exchange a pairing code for a device credential.

    The device gets its **own** token, not the master capability. That is
    what makes a lost phone a solved problem: revoke it in Settings and the
    athlete's own machine is untouched.
    """
    if not _pair_code_valid(body.code):
        raise HTTPException(
            status_code=401,
            detail="That code is wrong, expired, or already used.",
        )
    # Single use. The code is consumed the moment it works.
    state.update(pairing_code=None, pairing_expires_at=None)

    token = secrets.token_urlsafe(32)
    device = {
        "id": uuid.uuid4().hex[:12],
        "name": _device_name(request),
        "token_hash": hashlib.sha256(token.encode()).hexdigest(),
        "created_at": datetime.now(timezone.utc).isoformat(),
        "last_seen_at": datetime.now(timezone.utc).isoformat(),
    }
    devices = [d for d in (state.devices or []) if d.get("id") != device["id"]]
    devices.append(device)
    state.update(devices=devices)

    response = Response(status_code=204)
    set_device_cookie(response, request, token)
    log.info("paired device %s (%s)", device["id"], device["name"])
    return response


def _device_name(request: Request) -> str:
    """A label the athlete can recognise in the device list."""
    agent = request.headers.get("user-agent", "")
    for marker, label in (
        ("iPhone", "iPhone"),
        ("iPad", "iPad"),
        ("Android", "Android"),
        ("Macintosh", "Mac"),
        ("Windows", "Windows"),
        ("CrOS", "ChromeOS"),
        ("Linux", "Linux"),
    ):
        if marker in agent:
            return label
    return "Browser"


@router.get("/devices")
def list_devices() -> dict[str, Any]:
    """The paired devices. Tokens are never returned — only their hashes."""
    return {
        "devices": [
            {
                "id": device.get("id"),
                "name": device.get("name"),
                "created_at": device.get("created_at"),
                "last_seen_at": device.get("last_seen_at"),
            }
            for device in (state.devices or [])
        ]
    }


@router.delete("/devices/{device_id}")
def revoke_device(device_id: str) -> dict[str, Any]:
    """Revoke a paired device."""
    devices = [d for d in (state.devices or []) if d.get("id") != device_id]
    if len(devices) == len(state.devices or []):
        raise HTTPException(status_code=404, detail="No such device.")
    state.update(devices=devices)
    return {"ok": True, "remaining": len(devices)}
