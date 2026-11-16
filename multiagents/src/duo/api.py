"""Serveur local FastAPI."""

from __future__ import annotations

import asyncio
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import FileResponse, JSONResponse, PlainTextResponse, StreamingResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, Field, model_validator

from duo import __version__
from duo.adapters.copilot_cli import CopilotCliAdapter
from duo.adapters.cursor_cli import CursorCliAdapter
from duo.adapters.cursor_sdk import CursorSdkAdapter
from duo.bus import Bus
from duo.config import Settings, load_settings
from duo.db import Database
from duo.events import Broker
from duo.gitspace import WorkspaceManager
from duo.orchestrator import Orchestrator
from duo.process_util import ProcessRegistry
from duo.security import mutation_allowed
from duo.snapshot import snapshot, to_markdown

STATIC_DIR = Path(__file__).resolve().parent / "static"


class SessionIn(BaseModel):
    task: str = Field(min_length=1, max_length=20000)
    mode: str = "simulated"
    scenario: str = "collaboration"
    max_turns: int = Field(default=8, ge=1, le=50)
    max_duration_seconds: int = Field(default=600, ge=10, le=86400)
    agent_timeout_seconds: int = Field(default=180, ge=1, le=3600)
    similarity_threshold: float = Field(default=0.92, gt=0.5, le=1)
    max_corrections: int = Field(default=1, ge=0, le=5)
    proposer: str = "cursor"
    critic: str = "copilot"
    implementer: str = "cursor"
    allow_implementation: bool = True
    external_repo: str | None = None
    confirm_external: bool = False
    wait: bool = False

    @model_validator(mode="after")
    def check(self):
        if self.mode not in {"simulated", "live"}:
            raise ValueError("Mode inconnu.")
        if self.scenario not in {
            "collaboration",
            "desaccord",
            "boucle",
            "invalide",
            "panne",
            "lent",
            "tests_fail",
        }:
            raise ValueError("Profil simulé inconnu.")
        if self.mode == "live" and self.scenario != "collaboration":
            raise ValueError("Les profils simulés sont réservés au mode simulé.")
        if self.proposer == self.critic:
            raise ValueError("Le proposant et le critique doivent être distincts.")
        if self.external_repo and not self.confirm_external:
            raise ValueError("Confirmation explicite requise pour un dépôt existant.")
        return self


class SlotIn(BaseModel):
    window: str = "vscode"
    role: str | None = None


class RoomIn(BaseModel):
    task: str = Field(min_length=1, max_length=20000)
    agents_count: int = Field(default=1, ge=1, le=5)
    slots: list[SlotIn] | None = None
    agent1: str | None = None  # alias : fenêtre unique pour tous les agents
    partner: str | None = None  # alias de transition

    @model_validator(mode="after")
    def check_slots(self):
        fallback = self.agent1 or self.partner
        if self.slots is None:
            window = (fallback or "vscode").strip().lower()
            if window not in {"vscode", "cursor"}:
                raise ValueError("La fenêtre d'un agent doit être vscode ou cursor.")
            self.slots = [
                SlotIn(window=window, role=f"agent{index}")
                for index in range(1, self.agents_count + 1)
            ]
            return self
        if len(self.slots) != self.agents_count:
            raise ValueError("Le nombre de slots doit égaler agents_count.")
        normalized: list[SlotIn] = []
        for index, slot in enumerate(self.slots, start=1):
            window = slot.window.strip().lower()
            if window not in {"vscode", "cursor"}:
                raise ValueError("La fenêtre d'un agent doit être vscode ou cursor.")
            expected = f"agent{index}"
            role = (slot.role or expected).strip().lower()
            if role != expected:
                raise ValueError(f"Le rôle attendu est {expected}.")
            normalized.append(SlotIn(window=window, role=expected))
        self.slots = normalized
        return self


class RoomMessageIn(BaseModel):
    text: str = Field(min_length=1, max_length=20000)
    token: str | None = None
    sender: str | None = None

    @model_validator(mode="after")
    def check_sender(self):
        if self.sender == "humain":
            return self
        if self.sender not in {None, "humain"}:
            raise ValueError("L'identité d'un agent vient de son jeton.")
        if not self.token:
            raise ValueError("Jeton de fenêtre requis.")
        return self


def create_app(settings: Settings | None = None) -> FastAPI:
    settings = settings or load_settings()
    registry = ProcessRegistry()
    db = Database(settings.db_path)
    workspaces = WorkspaceManager(settings.sandbox_root, settings.worktree_root, settings.roots())
    broker = Broker()
    bus = Bus(settings.data_dir / "bus.sqlite")
    live = {
        "cursor": CursorCliAdapter(registry=registry, model=settings.cursor_model),
        "copilot": CopilotCliAdapter(registry=registry, model=settings.copilot_model),
    }
    orchestrator = Orchestrator(db, workspaces, broker=broker, live_adapters=live)

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        yield
        registry.kill_all()
        for task in list(app.state.tasks):
            task.cancel()
        bus.close()
        db.close()

    app = FastAPI(title="Duo", version=__version__, lifespan=lifespan)
    app.state.settings = settings
    app.state.db = db
    app.state.orchestrator = orchestrator
    app.state.broker = broker
    app.state.registry = registry
    app.state.tasks = set()
    app.state.sdk = CursorSdkAdapter()
    app.state.bus = bus

    @app.middleware("http")
    async def guard(request: Request, call_next):
        host_header = (request.headers.get("host") or "").split(",")[0].strip().lower()
        hostname = host_header.split(":")[0].strip("[]")
        if hostname and hostname not in {"127.0.0.1", "localhost", "::1", "testserver"}:
            return JSONResponse({"detail": "Hôte refusé."}, status_code=403)
        if request.url.path.startswith("/api"):
            allowed = mutation_allowed(
                request.method,
                request.headers.get("origin"),
                request.headers.get("x-duo-request"),
                settings.host,
                settings.port,
            )
            if not allowed:
                return JSONResponse({"detail": "Requête refusée."}, status_code=403)
        return await call_next(request)

    @app.get("/")
    async def index():
        return FileResponse(STATIC_DIR / "salon.html")

    @app.get("/orchestre")
    async def orchestre():
        return FileResponse(STATIC_DIR / "index.html")

    app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")

    @app.get("/api/health")
    async def health():
        cursor = await live["cursor"].health()
        copilot = await live["copilot"].health()
        sdk = await app.state.sdk.health()
        return {
            "version": __version__,
            "host": settings.host,
            "port": settings.port,
            "adapters": {
                "cursor": _health_dict(cursor),
                "copilot": _health_dict(copilot),
                "cursor_sdk": _health_dict(sdk),
            },
        }

    @app.get("/api/sessions")
    async def list_sessions():
        from duo.snapshot import session_view

        return [session_view(item) for item in db.list_sessions()]

    @app.post("/api/sessions")
    async def create_session(body: SessionIn):
        try:
            session = orchestrator.create_session(body.model_dump())
        except (ValueError, PermissionError) as exc:
            raise HTTPException(status_code=400, detail=str(exc)) from exc
        if body.wait:
            await orchestrator.run_session(session["id"])
        else:
            task = asyncio.create_task(orchestrator.run_session(session["id"]))
            app.state.tasks.add(task)
            task.add_done_callback(app.state.tasks.discard)
        return snapshot(db, session["id"])

    @app.get("/api/sessions/{session_id}")
    async def get_session(session_id: str):
        _check_id(session_id)
        try:
            return snapshot(db, session_id)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Session introuvable.") from exc

    @app.post("/api/sessions/{session_id}/pause")
    async def pause(session_id: str):
        return await _act(orchestrator.pause, session_id)

    @app.post("/api/sessions/{session_id}/resume")
    async def resume(session_id: str):
        return await _act(orchestrator.resume, session_id, async_call=True)

    @app.post("/api/sessions/{session_id}/stop")
    async def stop(session_id: str):
        return await _act(orchestrator.stop, session_id)

    @app.post("/api/sessions/{session_id}/retry")
    async def retry(session_id: str):
        return await _act(orchestrator.retry, session_id, async_call=True)

    @app.get("/api/sessions/{session_id}/export")
    async def export(session_id: str, format: str = "json"):
        _check_id(session_id)
        try:
            data = snapshot(db, session_id)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Session introuvable.") from exc
        if format == "md":
            return PlainTextResponse(to_markdown(data), media_type="text/markdown; charset=utf-8")
        if format != "json":
            raise HTTPException(status_code=400, detail="Format inconnu.")
        return data

    @app.get("/api/sessions/{session_id}/events")
    async def events(session_id: str):
        _check_id(session_id)
        if db.get_session(session_id) is None:
            raise HTTPException(status_code=404, detail="Session introuvable.")

        async def stream():
            yield "event: ready\ndata: {}\n\n"
            async for payload in broker.subscribe(session_id):
                import json

                yield f"event: update\ndata: {json.dumps(payload, ensure_ascii=False)}\n\n"

        return StreamingResponse(stream(), media_type="text/event-stream")

    @app.get("/api/rooms")
    async def list_rooms():
        return bus.list_rooms()

    @app.post("/api/rooms")
    async def create_room(body: RoomIn):
        try:
            slots = [
                {"role": slot.role or f"agent{index}", "window": slot.window}
                for index, slot in enumerate(body.slots or [], start=1)
            ]
            created = bus.create_room(
                body.task,
                agents_count=body.agents_count,
                slots=slots,
                agent1=body.agent1 or body.partner,
            )
        except ValueError as exc:
            raise HTTPException(status_code=400, detail=str(exc)) from exc
        return _room_created(bus, settings, created)

    @app.get("/api/rooms/{room_id}")
    async def get_room(room_id: str):
        _check_id(room_id)
        try:
            return bus.room(room_id)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Salon introuvable.") from exc

    @app.get("/api/rooms/{room_id}/consignes")
    async def room_prompts(room_id: str, request: Request):
        _check_id(room_id)
        if request.headers.get("x-duo-request") != "1":
            raise HTTPException(status_code=403, detail="Requête refusée.")
        try:
            prompts = bus.consignes(room_id, settings.host, settings.port)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Salon introuvable.") from exc
        return prompts

    @app.post("/api/rooms/{room_id}/messages")
    async def post_room_message(room_id: str, body: RoomMessageIn, request: Request):
        _check_id(room_id)
        claimed = _claimed_agent(request)
        try:
            if body.sender == "humain":
                message = bus.post_humain(room_id, body.text)
            else:
                message = bus.post_token(room_id, body.token or "", body.text, claimed)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Salon introuvable.") from exc
        except PermissionError as exc:
            raise HTTPException(status_code=403, detail=str(exc)) from exc
        except ValueError as exc:
            raise HTTPException(status_code=400, detail=str(exc)) from exc
        return message

    @app.get("/api/rooms/{room_id}/wait")
    async def wait_room(
        room_id: str,
        request: Request,
        token: str,
        after: int = 0,
        timeout: int = 90,
    ):
        _check_id(room_id)
        if after < 0:
            raise HTTPException(status_code=400, detail="Le curseur after est invalide.")
        if not 1 <= timeout <= 90:
            raise HTTPException(status_code=400, detail="Le délai doit être entre 1 et 90 secondes.")
        try:
            return await bus.wait(room_id, token, after, timeout, _claimed_agent(request))
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Salon introuvable.") from exc
        except PermissionError as exc:
            raise HTTPException(status_code=403, detail=str(exc)) from exc

    @app.post("/api/rooms/{room_id}/close")
    async def close_room(room_id: str):
        _check_id(room_id)
        try:
            return bus.close_room(room_id)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Salon introuvable.") from exc

    return app


def _health_dict(health) -> dict:
    return {
        "agent": health.agent,
        "available": health.available,
        "authenticated": health.authenticated,
        "detail": health.detail,
        "unsupported": health.unsupported,
    }


def _room_created(bus: Bus, settings: Settings, created: dict) -> dict:
    prompts = bus.consignes(created["id"], settings.host, settings.port)
    tokens = created.get("tokens") or {}
    public = {
        key: value
        for key, value in created.items()
        if key not in {"tokens"} and not key.endswith("_token")
    }
    return {
        "room": public,
        "tokens": tokens,
        "pilote_token": tokens.get("pilote", created.get("pilote_token", "")),
        "agent1_token": tokens.get("agent1", created.get("agent1_token", "")),
        "master_token": tokens.get("pilote", ""),
        "partner_token": tokens.get("agent1", ""),
        "consignes": prompts,
    }


def _claimed_agent(request: Request) -> str | None:
    claimed = (request.headers.get("x-duo-agent") or "").strip().lower()
    if not claimed:
        return None
    allowed = {"cursor", "vscode", "master", "partner", "pilote"} | {
        f"agent{index}" for index in range(1, 6)
    }
    if claimed not in allowed:
        raise HTTPException(status_code=400, detail="Fenêtre inconnue.")
    return claimed


def _check_id(session_id: str) -> None:
    if len(session_id) > 64 or any(char not in "0123456789abcdef-" for char in session_id.lower()):
        raise HTTPException(status_code=404, detail="Session introuvable.")


async def _act(func, session_id: str, async_call: bool = False):
    _check_id(session_id)
    try:
        if async_call:
            await func(session_id)
        else:
            func(session_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="Session introuvable.") from exc
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    except PermissionError as exc:
        raise HTTPException(status_code=403, detail=str(exc)) from exc
    from duo.snapshot import snapshot as snap

    # L'application courante n'est pas en closure ici : on relit via l'orchestrateur.
    database = func.__self__.db
    return snap(database, session_id)
