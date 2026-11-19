"""Interface commune des fournisseurs (auth OAuth CLI + quotas)."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


PROVIDER_IDS = ("cursor", "copilot", "vibe", "antigravity")

PROVIDER_LABELS = {
    "cursor": "Cursor",
    "copilot": "GitHub Copilot",
    "vibe": "Mistral Vibe",
    "antigravity": "Google Antigravity",
}


@dataclass
class QuotaItem:
    id: str
    label: str
    used: float | None = None
    limit: float | None = None
    remaining: float | None = None
    resets_at: str | None = None
    status: str = "unavailable"  # live | unavailable | unknown
    detail: str = ""

    def as_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "label": self.label,
            "used": self.used,
            "limit": self.limit,
            "remaining": self.remaining,
            "resets_at": self.resets_at,
            "status": self.status,
            "detail": self.detail,
        }


@dataclass
class ModelInfo:
    id: str
    label: str
    tier: str = "standard"  # light (N1) | standard (N2) | deep (N3) | critical (N4)
    cost_hint: str = ""

    def as_dict(self) -> dict[str, str]:
        return {
            "id": self.id,
            "label": self.label,
            "tier": self.tier,
            "cost_hint": self.cost_hint,
        }


@dataclass
class ProviderSnapshot:
    id: str
    label: str
    available: bool
    authenticated: bool | None
    detail: str
    login_hint: str = ""
    models: list[ModelInfo] = field(default_factory=list)
    quotas: list[QuotaItem] = field(default_factory=list)

    def as_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "label": self.label,
            "available": self.available,
            "authenticated": self.authenticated,
            "detail": self.detail,
            "login_hint": self.login_hint,
            "models": [item.as_dict() for item in self.models],
            "quotas": [item.as_dict() for item in self.quotas],
        }


@dataclass
class LoginResult:
    ok: bool
    detail: str
    started: bool = False


class BaseProvider:
    id: str = ""
    label: str = ""

    async def status(self) -> ProviderSnapshot:
        raise NotImplementedError

    async def start_login(self) -> LoginResult:
        raise NotImplementedError

    async def logout(self) -> LoginResult:
        return LoginResult(
            ok=False,
            detail="Déconnexion via Duo non supportée. Utilisez la CLI du fournisseur.",
            started=False,
        )

    async def fetch_quotas(self) -> list[QuotaItem]:
        return [
            QuotaItem(
                id="default",
                label="Quota",
                status="unavailable",
                detail="Quota non exposé par une API stable.",
            )
        ]

    def _snapshot(
        self,
        *,
        available: bool,
        authenticated: bool | None,
        detail: str,
        login_hint: str = "",
        models: list[ModelInfo] | None = None,
        quotas: list[QuotaItem] | None = None,
    ) -> ProviderSnapshot:
        return ProviderSnapshot(
            id=self.id,
            label=self.label,
            available=available,
            authenticated=authenticated,
            detail=detail,
            login_hint=login_hint,
            models=models or [],
            quotas=quotas or [],
        )
