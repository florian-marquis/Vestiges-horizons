"""Fournisseur Cursor — auth via `agent login`, quotas via usage-summary best-effort."""

from __future__ import annotations

import json
import os
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from duo.adapters.cli_common import resolve_cursor_agent_command
from duo.models import CancelToken
from duo.process_util import ProcessRegistry, run_command
from duo.providers._spawn import spawn_detached
from duo.providers.base import BaseProvider, LoginResult, ModelInfo, ProviderSnapshot, QuotaItem

LOGIN_HINT = (
    "Duo lance `agent login` (navigateur). Les jetons restent dans le store CLI / "
    "Credential Manager — jamais dans Duo."
)
HELP_QUOTA = "https://cursor.com/settings"

DEFAULT_CURSOR_MODELS: list[ModelInfo] = [
    ModelInfo(id="auto", label="Auto / Cursor 2.5", tier="standard", cost_hint="quota maison"),
    ModelInfo(id="grok-4.7", label="Grok 4.7", tier="standard", cost_hint="quota maison"),
    ModelInfo(id="claude-4.6-sonnet", label="Claude Sonnet (Cursor)", tier="deep", cost_hint="quota extérieur"),
    ModelInfo(id="claude-4.6-opus", label="Claude Opus (Cursor)", tier="critical", cost_hint="quota extérieur"),
    ModelInfo(id="gemini-2.5-pro", label="Gemini 2.5 Pro (Cursor)", tier="deep", cost_hint="quota extérieur"),
]


def _auth_json_paths() -> list[Path]:
    paths: list[Path] = []
    home = Path.home()
    local = os.environ.get("LOCALAPPDATA")
    candidates = [
        home / ".cursor" / "cli-config.json",
        home / ".cursor" / "auth.json",
        home / ".cursor-agent" / "auth.json",
    ]
    if local:
        base = Path(local) / "cursor-agent"
        candidates.append(base / "auth.json")
        versions = base / "versions"
        try:
            if versions.is_dir():
                for child in versions.iterdir():
                    if child.is_dir():
                        candidates.append(child / "auth.json")
        except OSError:
            pass
    for path in candidates:
        if path not in paths:
            paths.append(path)
    return paths


def _read_cli_access_token() -> str | None:
    """Lit un jeton CLI localement sans le journaliser ni le stocker dans Duo."""
    for path in _auth_json_paths():
        try:
            if not path.is_file():
                continue
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError, UnicodeError):
            continue
        if not isinstance(data, dict):
            continue
        for key in ("accessToken", "access_token", "token", "sessionToken"):
            value = data.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
        nested = data.get("auth") or data.get("credentials")
        if isinstance(nested, dict):
            for key in ("accessToken", "access_token", "token"):
                value = nested.get(key)
                if isinstance(value, str) and value.strip():
                    return value.strip()
    return None


def parse_usage_summary(payload: dict) -> list[QuotaItem]:
    """Normalise la réponse usage-summary (fixture ou live)."""
    items: list[QuotaItem] = []
    individual = payload.get("individualUsage") if isinstance(payload, dict) else None
    plan = None
    if isinstance(individual, dict):
        plan = individual.get("plan")
    if not isinstance(plan, dict):
        plan = payload.get("plan") if isinstance(payload.get("plan"), dict) else None
    if isinstance(plan, dict):
        used = plan.get("used")
        limit = plan.get("limit")
        remaining = plan.get("remaining")
        percent = plan.get("totalPercentUsed")
        if remaining is None and isinstance(limit, (int, float)) and isinstance(used, (int, float)):
            remaining = float(limit) - float(used)
        detail = ""
        if isinstance(percent, (int, float)):
            detail = f"{percent:.0f} % du plan utilisé"
        items.append(
            QuotaItem(
                id="plan",
                label="Plan Cursor",
                used=float(used) if isinstance(used, (int, float)) else None,
                limit=float(limit) if isinstance(limit, (int, float)) else None,
                remaining=float(remaining) if isinstance(remaining, (int, float)) else None,
                resets_at=str(plan["billingCycleEnd"]) if plan.get("billingCycleEnd") else None,
                status="live",
                detail=detail or HELP_QUOTA,
            )
        )
    team_usage = payload.get("teamUsage") if isinstance(payload, dict) else None
    if isinstance(team_usage, dict):
        pooled = team_usage.get("pooled")
        if isinstance(pooled, dict) and pooled.get("enabled"):
            items.append(
                QuotaItem(
                    id="team_pool",
                    label="Pool équipe",
                    used=float(pooled["used"]) if isinstance(pooled.get("used"), (int, float)) else None,
                    limit=float(pooled["limit"]) if isinstance(pooled.get("limit"), (int, float)) else None,
                    remaining=float(pooled["remaining"])
                    if isinstance(pooled.get("remaining"), (int, float))
                    else None,
                    status="live",
                    detail="Usage mutualisé équipe",
                )
            )
    if not items:
        items.append(
            QuotaItem(
                id="plan",
                label="Plan Cursor",
                status="unavailable",
                detail=f"Réponse usage-summary inattendue. Voir {HELP_QUOTA}",
            )
        )
    return items


class CursorProvider(BaseProvider):
    id = "cursor"
    label = "Cursor"

    def __init__(self, runner=None, registry: ProcessRegistry | None = None, model: str = "auto"):
        self.runner = runner or run_command
        self.registry = registry or ProcessRegistry()
        self.model = model or "auto"

    def _models(self) -> list[ModelInfo]:
        models = list(DEFAULT_CURSOR_MODELS)
        if not self.model or self.model == "auto":
            return models
        configured = next((m for m in models if m.id == self.model), None)
        others = [m for m in models if m.id != self.model]
        if configured is not None:
            return [configured, *others]
        return [ModelInfo(id=self.model, label=f"CLI ({self.model})", tier="standard"), *models]

    async def status(self) -> ProviderSnapshot:
        command = resolve_cursor_agent_command()
        if not command:
            return self._snapshot(
                available=False,
                authenticated=None,
                detail=(
                    "CLI `agent` introuvable. Installez-le via "
                    "irm 'https://cursor.com/install?win32=true' | iex"
                ),
                login_hint=LOGIN_HINT,
                models=self._models(),
                quotas=[
                    QuotaItem(
                        id="plan",
                        label="Plan Cursor",
                        status="unavailable",
                        detail="CLI absente.",
                    )
                ],
            )
        token = CancelToken()
        code, stdout, stderr, reason = await self.runner(
            [*command, "status"],
            str(Path.cwd()),
            20,
            token,
            self.registry,
            "provider-cursor-status",
        )
        blob = f"{stdout}\n{stderr}".lower()
        quotas = await self.fetch_quotas()
        if reason == "timeout":
            return self._snapshot(
                available=True,
                authenticated=None,
                detail="`agent status` n'a pas répondu à temps.",
                login_hint=LOGIN_HINT,
                models=self._models(),
                quotas=quotas,
            )
        if "not logged in" in blob:
            return self._snapshot(
                available=True,
                authenticated=False,
                detail="Pas de session CLI. Lancez la connexion OAuth.",
                login_hint=LOGIN_HINT,
                models=self._models(),
                quotas=quotas,
            )
        if code == 0 and "logged in" in blob:
            return self._snapshot(
                available=True,
                authenticated=True,
                detail="CLI authentifiée (compte non affiché).",
                login_hint=LOGIN_HINT,
                models=self._models(),
                quotas=quotas,
            )
        return self._snapshot(
            available=True,
            authenticated=None,
            detail="CLI présente ; authentification non lue.",
            login_hint=LOGIN_HINT,
            models=self._models(),
            quotas=quotas,
        )

    async def start_login(self) -> LoginResult:
        command = resolve_cursor_agent_command()
        if not command:
            return LoginResult(ok=False, detail="CLI `agent` introuvable.", started=False)
        try:
            spawn_detached([*command, "login"])
        except OSError as exc:
            return LoginResult(ok=False, detail=f"Impossible de lancer agent login : {exc}", started=False)
        return LoginResult(
            ok=True,
            started=True,
            detail=(
                "`agent login` a démarré. Terminez OAuth dans le navigateur, "
                "puis rafraîchissez le statut. Les jetons restent hors Duo."
            ),
        )

    async def logout(self) -> LoginResult:
        command = resolve_cursor_agent_command()
        if not command:
            return LoginResult(ok=False, detail="CLI `agent` introuvable.")
        try:
            spawn_detached([*command, "logout"])
        except OSError as exc:
            return LoginResult(ok=False, detail=str(exc))
        return LoginResult(
            ok=True,
            started=True,
            detail="`agent logout` lancé. Vérifiez ensuite le statut.",
        )

    async def fetch_quotas(self) -> list[QuotaItem]:
        token = _read_cli_access_token()
        if not token:
            return [
                QuotaItem(
                    id="plan",
                    label="Plan Cursor",
                    status="unavailable",
                    detail=(
                        "Aucun jeton CLI lisible localement. Connectez-vous via OAuth, "
                        f"ou consultez {HELP_QUOTA}"
                    ),
                )
            ]
        try:
            request = Request(
                "https://cursor.com/api/usage-summary",
                headers={
                    "Cookie": f"WorkosCursorSessionToken={token}",
                    "User-Agent": "cursor-agent/duo",
                    "Accept": "application/json",
                },
                method="GET",
            )
            with urlopen(request, timeout=15) as response:  # noqa: S310 — URL fixe HTTPS
                raw = response.read().decode("utf-8", errors="replace")
            payload = json.loads(raw)
            if not isinstance(payload, dict):
                raise ValueError("payload non objet")
            return parse_usage_summary(payload)
        except (HTTPError, URLError, TimeoutError, json.JSONDecodeError, ValueError, OSError):
            return [
                QuotaItem(
                    id="plan",
                    label="Plan Cursor",
                    status="unavailable",
                    detail=f"usage-summary inaccessible. Voir {HELP_QUOTA}",
                )
            ]
