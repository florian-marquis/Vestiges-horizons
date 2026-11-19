"""Serveur local FastAPI."""

from __future__ import annotations

import asyncio
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any

from fastapi import Body, FastAPI, HTTPException, Request
from fastapi.responses import FileResponse, JSONResponse, PlainTextResponse, StreamingResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, Field, model_validator

from duo import __version__
from duo.adapters.antigravity_cli import AntigravityCliAdapter
from duo.adapters.copilot_cli import CopilotCliAdapter
from duo.adapters.cursor_cli import CursorCliAdapter
from duo.adapters.cursor_sdk import CursorSdkAdapter
from duo.adapters.vibe_cli import VibeCliAdapter
from duo.bus import Bus
from duo.config import Settings, load_settings
from duo.db import Database
from duo.events import Broker
from duo.gitspace import WorkspaceManager
from duo.orchestrator import AGENTS, Orchestrator
from duo.process_util import ProcessRegistry
from duo.providers import PROVIDER_IDS, ProviderRegistry, build_default_providers
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
        for name in (self.proposer, self.critic, self.implementer):
            if name not in AGENTS:
                raise ValueError("Agent inconnu.")
        if self.external_repo and not self.confirm_external:
            raise ValueError("Confirmation explicite requise pour un dépôt existant.")
        return self


_VALID_WINDOWS = {"vscode", "cursor", "antigravity"}
_VALID_PROVIDERS = {"antigravity", "copilot", "vibe", "cursor"}
_DEFAULT_WINDOWS_BY_PROVIDER = {
    "antigravity": "antigravity",
    "copilot": "vscode",
    "vibe": "vscode",
    "cursor": "cursor",
}


class SlotIn(BaseModel):
    window: str | None = None
    role: str | None = None
    provider: str | None = None
    model: str | None = None


class SlotUpdateIn(BaseModel):
    role: str
    window: str | None = None
    provider: str | None = None
    model: str | None = None
    reason: str = ""


class RoomIn(BaseModel):
    task: str = Field(min_length=1, max_length=20000)
    agents_count: int = Field(default=1, ge=1, le=5)
    pilote: SlotIn | None = None
    slots: list[SlotIn] | None = None
    agent1: str | None = None  # alias : fenêtre unique pour tous les agents
    partner: str | None = None  # alias de transition

    @model_validator(mode="after")
    def check_slots(self):
        if self.pilote is not None:
            p_prov = self.pilote.provider.strip().lower() if self.pilote.provider else None
            if p_prov is not None and p_prov not in _VALID_PROVIDERS:
                raise ValueError(f"Outil Pilote inconnu : {p_prov}.")
            p_win = self.pilote.window.strip().lower() if self.pilote.window else None
            if p_win is not None and p_win not in _VALID_WINDOWS:
                raise ValueError("La fenêtre du Pilote doit être antigravity, vscode ou cursor.")
            if p_win is None:
                p_win = _DEFAULT_WINDOWS_BY_PROVIDER.get(p_prov or "antigravity", "antigravity")
            self.pilote = SlotIn(
                role="pilote",
                window=p_win,
                provider=p_prov,
                model=self.pilote.model.strip() if self.pilote.model else None,
            )

        fallback = self.agent1 or self.partner
        if self.slots is None:
            window = (fallback or "vscode").strip().lower()
            if window not in _VALID_WINDOWS:
                raise ValueError("La fenêtre d'un agent doit être antigravity, vscode ou cursor.")
            self.slots = [
                SlotIn(window=window, role=f"agent{index}")
                for index in range(1, self.agents_count + 1)
            ]
            return self
        if len(self.slots) != self.agents_count:
            raise ValueError("Le nombre de slots doit égaler agents_count.")
        normalized: list[SlotIn] = []
        for index, slot in enumerate(self.slots, start=1):
            provider = slot.provider.strip().lower() if slot.provider else None
            if provider is not None and provider not in _VALID_PROVIDERS:
                raise ValueError(f"Outil inconnu : {provider}.")
            window = slot.window.strip().lower() if slot.window else None
            if window is not None and window not in _VALID_WINDOWS:
                raise ValueError("La fenêtre d'un agent doit être antigravity, vscode ou cursor.")
            if window is None:
                window = _DEFAULT_WINDOWS_BY_PROVIDER.get(provider or "copilot", "vscode")
            expected = f"agent{index}"
            role = (slot.role or expected).strip().lower()
            if role != expected:
                raise ValueError(f"Le rôle attendu est {expected}.")
            normalized.append(
                SlotIn(
                    window=window,
                    role=expected,
                    provider=provider,
                    model=slot.model.strip() if slot.model else None,
                )
            )
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
        "vibe": VibeCliAdapter(registry=registry),
        "antigravity": AntigravityCliAdapter(registry=registry),
    }
    providers = ProviderRegistry(
        build_default_providers(
            registry=registry,
            cursor_model=settings.cursor_model,
            copilot_model=settings.copilot_model,
        ),
        meta_path=settings.data_dir / "providers_meta.json",
    )
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
    app.state.providers = providers

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

    @app.get("/comptes")
    async def comptes():
        return FileResponse(STATIC_DIR / "comptes.html")

    app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")

    @app.get("/api/health")
    async def health():
        adapters = {
            name: _health_dict(await live[name].health())
            for name in ("cursor", "copilot", "vibe", "antigravity")
        }
        sdk = await app.state.sdk.health()
        adapters["cursor_sdk"] = _health_dict(sdk)
        return {
            "version": __version__,
            "host": settings.host,
            "port": settings.port,
            "adapters": adapters,
        }

    @app.get("/api/providers")
    async def list_providers():
        return {"providers": await providers.list_all()}

    @app.get("/api/providers/{provider_id}")
    async def get_provider(provider_id: str):
        _check_provider(provider_id)
        try:
            return await providers.snapshot(provider_id)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Fournisseur inconnu.") from exc

    @app.post("/api/providers/{provider_id}/login")
    async def provider_login(provider_id: str):
        _check_provider(provider_id)
        try:
            result = await providers.get(provider_id).start_login()
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Fournisseur inconnu.") from exc
        return {"ok": result.ok, "started": result.started, "detail": result.detail}

    @app.post("/api/providers/{provider_id}/logout")
    async def provider_logout(provider_id: str):
        _check_provider(provider_id)
        try:
            result = await providers.get(provider_id).logout()
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Fournisseur inconnu.") from exc
        return {"ok": result.ok, "started": result.started, "detail": result.detail}

    @app.get("/api/providers/{provider_id}/quotas")
    async def provider_quotas(provider_id: str):
        _check_provider(provider_id)
        try:
            snap = await providers.snapshot(provider_id)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Fournisseur inconnu.") from exc
        return {
            "provider": provider_id,
            "models": snap.get("models", []),
            "quotas": snap.get("quotas", []),
            "last_check_at": snap.get("last_check_at"),
            "synced_at": snap.get("synced_at"),
        }

    @app.post("/api/providers/{provider_id}/sync")
    async def sync_provider(provider_id: str, payload: dict[str, Any] = Body(default_factory=dict)):
        _check_provider(provider_id)
        try:
            return await providers.sync_provider(provider_id, payload)
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Fournisseur inconnu.") from exc
        except ValueError as exc:
            raise HTTPException(status_code=400, detail=str(exc)) from exc

    @app.get("/api/routing")
    async def get_routing(difficulty: str = "N2", pilote_provider: str = "antigravity"):
        return await providers.recommend_routing(
            difficulty=difficulty,
            current_pilote_provider=pilote_provider,
        )

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
                {
                    key: value
                    for key, value in {
                        "role": slot.role or f"agent{index}",
                        "window": slot.window,
                        "provider": slot.provider,
                        "model": slot.model,
                    }.items()
                    if value is not None
                }
                for index, slot in enumerate(body.slots or [], start=1)
            ]
            pilote = body.pilote.model_dump(exclude_none=True) if body.pilote else None
            created = bus.create_room(
                body.task,
                agents_count=body.agents_count,
                slots=slots,
                agent1=body.agent1 or body.partner,
                pilote=pilote,
            )
        except ValueError as exc:
            raise HTTPException(status_code=400, detail=str(exc)) from exc
        return _room_created(bus, settings, created)

    @app.patch("/api/rooms/{room_id}/slots")
    async def update_room_slot(room_id: str, body: SlotUpdateIn):
        _check_id(room_id)
        try:
            return bus.update_slot(
                room_id,
                body.role,
                window=body.window,
                provider=body.provider,
                model=body.model,
                reason=body.reason,
                host=settings.host,
                port=settings.port,
            )
        except KeyError as exc:
            raise HTTPException(status_code=404, detail="Salon introuvable.") from exc
        except ValueError as exc:
            raise HTTPException(status_code=400, detail=str(exc)) from exc

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


def _check_provider(provider_id: str) -> None:
    if provider_id not in PROVIDER_IDS:
        raise HTTPException(status_code=404, detail="Fournisseur inconnu.")


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
