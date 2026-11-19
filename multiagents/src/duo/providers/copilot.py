"""Fournisseur GitHub Copilot — auth via `copilot login`, quotas via gh best-effort."""

from __future__ import annotations

import json
import os
import shutil
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from duo.adapters.cli_common import resolve_copilot_command
from duo.models import CancelToken
from duo.process_util import ProcessRegistry, run_command
from duo.providers._spawn import spawn_detached
from duo.providers.base import BaseProvider, LoginResult, ModelInfo, ProviderSnapshot, QuotaItem

LOGIN_HINT = (
    "Duo lance `copilot login` (navigateur / device flow). "
    "Les jetons restent dans le store Copilot / Credential Manager."
)
HELP_QUOTA = "https://github.com/settings/copilot"

DEFAULT_COPILOT_MODELS: list[ModelInfo] = [
    ModelInfo(id="gpt-4.1", label="GPT-4.1", tier="standard", cost_hint="0x (inclus)"),
    ModelInfo(id="gpt-5-mini", label="GPT-5 mini", tier="light", cost_hint="0x (inclus)"),
    ModelInfo(id="gpt-4o", label="GPT-4o", tier="light", cost_hint="0x (inclus)"),
    ModelInfo(id="gemini-3-flash", label="Gemini 3 Flash", tier="standard", cost_hint="0.33x"),
    ModelInfo(id="claude-haiku-4.5", label="Claude Haiku 4.5", tier="light", cost_hint="0.33x"),
    ModelInfo(id="claude-sonnet-4.6", label="Claude Sonnet 4.6", tier="deep", cost_hint="1x"),
    ModelInfo(id="gemini-2.5-pro", label="Gemini 2.5 Pro", tier="deep", cost_hint="1x"),
    ModelInfo(id="gpt-5", label="GPT-5", tier="deep", cost_hint="1x"),
    ModelInfo(id="claude-opus-4.6", label="Claude Opus 4.6", tier="critical", cost_hint="3x"),
]


def _copilot_token_paths() -> list[Path]:
    paths: list[Path] = []
    local = os.environ.get("LOCALAPPDATA")
    if local:
        base = Path(local) / "github-copilot"
        paths.extend([base / "hosts.json", base / "apps.json"])
    cfg = Path.home() / ".config" / "github-copilot"
    paths.extend([cfg / "hosts.json", cfg / "apps.json"])
    return paths


def _read_local_copilot_token() -> str | None:
    """Lit un jeton OAuth Copilot localement sans jamais le journaliser ni le persister."""
    for key in ("COPILOT_GITHUB_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"):
        value = os.environ.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    for path in _copilot_token_paths():
        try:
            if not path.is_file():
                continue
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError, UnicodeError):
            continue
        if not isinstance(data, dict):
            continue
        for entry in data.values():
            if isinstance(entry, dict):
                token = entry.get("oauth_token") or entry.get("token")
                if isinstance(token, str) and token.strip():
                    return token.strip()
    return None


def parse_copilot_user(payload: dict) -> list[QuotaItem]:
    """Normalise `/copilot_internal/user` (fixture ou live)."""
    items: list[QuotaItem] = []
    quota = payload.get("quota_snapshots") if isinstance(payload, dict) else None
    if not isinstance(quota, dict):
        quota = payload.get("quotas") if isinstance(payload.get("quotas"), dict) else {}
    mapping = [
        ("premium_interactions", "Interactions premium"),
        ("chat", "Chat"),
        ("completions", "Complétions"),
    ]
    for key, label in mapping:
        block = quota.get(key) if isinstance(quota, dict) else None
        if not isinstance(block, dict):
            continue
        # API souvent : percent_remaining + entitlement
        remaining_pct = block.get("percent_remaining")
        entitlement = block.get("entitlement")
        remaining = block.get("remaining")
        limit = entitlement if isinstance(entitlement, (int, float)) else block.get("limit")
        used_val = block.get("used")
        if used_val is None and isinstance(limit, (int, float)) and isinstance(remaining, (int, float)):
            used_val = float(limit) - float(remaining)
        detail = ""
        if isinstance(remaining_pct, (int, float)):
            detail = f"{remaining_pct:.0f} % restants"
        items.append(
            QuotaItem(
                id=key,
                label=label,
                used=float(used_val) if isinstance(used_val, (int, float)) else None,
                limit=float(limit) if isinstance(limit, (int, float)) else None,
                remaining=float(remaining) if isinstance(remaining, (int, float)) else None,
                resets_at=str(block["reset_date"]) if block.get("reset_date") else None,
                status="live",
                detail=detail or HELP_QUOTA,
            )
        )
    if not items and isinstance(payload.get("quota_reset_date"), str):
        items.append(
            QuotaItem(
                id="premium_interactions",
                label="Interactions premium",
                status="unknown",
                resets_at=payload["quota_reset_date"],
                detail="Détail de quota absent dans la réponse.",
            )
        )
    if not items:
        items.append(
            QuotaItem(
                id="premium_interactions",
                label="Interactions premium",
                status="unavailable",
                detail=f"Quota Copilot non exposé. Voir {HELP_QUOTA}",
            )
        )
    return items


class CopilotProvider(BaseProvider):
    id = "copilot"
    label = "GitHub Copilot"

    def __init__(self, runner=None, registry: ProcessRegistry | None = None, model: str = ""):
        self.runner = runner or run_command
        self.registry = registry or ProcessRegistry()
        self.model = model or ""

    def _models(self) -> list[ModelInfo]:
        models = list(DEFAULT_COPILOT_MODELS)
        if not self.model:
            return models
        configured = next((m for m in models if m.id == self.model), None)
        others = [m for m in models if m.id != self.model]
        if configured is not None:
            return [configured, *others]
        return [ModelInfo(id=self.model, label=f"CLI ({self.model})", tier="standard"), *models]

    async def status(self) -> ProviderSnapshot:
        command = resolve_copilot_command()
        quotas = await self.fetch_quotas()
        if not command:
            return self._snapshot(
                available=False,
                authenticated=None,
                detail="GitHub Copilot CLI introuvable (`npm i -g @github/copilot`).",
                login_hint=LOGIN_HINT,
                models=self._models(),
                quotas=quotas,
            )
        # Pas de `copilot status` fiable : présence CLI + indices d'auth via env / store local / gh.
        has_local_token = _read_local_copilot_token() is not None
        has_live_quota = any(item.status == "live" for item in quotas)
        authenticated: bool | None = None
        detail = "CLI détectée. Authentification non prouvée tant qu'un appel -p n'a pas réussi."
        if has_live_quota or os_environ_has_copilot_token() or has_local_token:
            authenticated = True
            detail = "CLI détectée ; jeton local ou quota live confirmé."
        elif shutil.which("gh"):
            detail = "CLI détectée ; `gh` présent (auth à confirmer via quotas)."
        return self._snapshot(
            available=True,
            authenticated=authenticated,
            detail=detail,
            login_hint=LOGIN_HINT,
            models=self._models(),
            quotas=quotas,
        )

    async def start_login(self) -> LoginResult:
        command = resolve_copilot_command()
        if not command:
            return LoginResult(ok=False, detail="Copilot CLI introuvable.", started=False)
        try:
            spawn_detached([*command, "login"])
        except OSError as exc:
            return LoginResult(ok=False, detail=str(exc), started=False)
        return LoginResult(
            ok=True,
            started=True,
            detail="`copilot login` démarré. Terminez le flux dans le navigateur, puis rafraîchissez.",
        )

    async def logout(self) -> LoginResult:
        command = resolve_copilot_command()
        if not command:
            return LoginResult(ok=False, detail="Copilot CLI introuvable.")
        try:
            spawn_detached([*command, "logout"])
        except OSError as exc:
            return LoginResult(ok=False, detail=str(exc))
        return LoginResult(ok=True, started=True, detail="`copilot logout` lancé si supporté par la CLI.")

    def _fetch_quotas_via_http(self, token: str) -> list[QuotaItem] | None:
        try:
            request = Request(
                "https://api.github.com/copilot_internal/user",
                headers={
                    "Authorization": f"token {token}",
                    "Accept": "application/json",
                    "User-Agent": "duo-copilot/1.0",
                },
                method="GET",
            )
            with urlopen(request, timeout=5) as response:  # noqa: S310 — URL HTTPS fixe GitHub
                raw = response.read().decode("utf-8", errors="replace")
            payload = json.loads(raw)
            if isinstance(payload, dict):
                return parse_copilot_user(payload)
        except (HTTPError, URLError, TimeoutError, json.JSONDecodeError, ValueError, OSError):
            return None
        return None

    async def fetch_quotas(self) -> list[QuotaItem]:
        gh = shutil.which("gh")
        fail_detail = f"`gh` absent — impossible d'interroger /copilot_internal/user. Voir {HELP_QUOTA}"
        if gh:
            token = CancelToken()
            code, stdout, stderr, reason = await self.runner(
                [gh, "api", "/copilot_internal/user"],
                str(Path.cwd()),
                20,
                token,
                self.registry,
                "provider-copilot-quota",
            )
            if reason == "ok" and code in (0, None):
                try:
                    payload = json.loads(stdout)
                except json.JSONDecodeError:
                    payload = None
                if isinstance(payload, dict):
                    return parse_copilot_user(payload)
                fail_detail = "Réponse gh inattendue ou non JSON."
            else:
                fail_detail = f"gh api a échoué (auth gh ?). Voir {HELP_QUOTA}"

        local_token = _read_local_copilot_token()
        if local_token:
            http_items = self._fetch_quotas_via_http(local_token)
            if http_items is not None:
                return http_items

        return [
            QuotaItem(
                id="premium_interactions",
                label="Interactions premium",
                status="unavailable",
                detail=fail_detail,
            )
        ]


def os_environ_has_copilot_token() -> bool:
    for key in ("COPILOT_GITHUB_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"):
        if os.environ.get(key):
            return True
    return False
