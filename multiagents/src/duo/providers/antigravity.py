"""Fournisseur Google Antigravity — auth via `agy` (OAuth keyring), quotas best-effort."""

from __future__ import annotations

import os
import shutil
from pathlib import Path

from duo.providers._spawn import spawn_detached
from duo.providers.base import BaseProvider, LoginResult, ModelInfo, ProviderSnapshot, QuotaItem

LOGIN_HINT = (
    "Duo lance `agy` : OAuth navigateur + stockage keyring OS. "
    "Duo ne copie aucun jeton."
)
HELP_USAGE = "Ouvrez `agy` puis `/usage` ou synchronisez via `swarm(action=\"models\", all=True)`."

DEFAULT_ANTIGRAVITY_MODELS: list[ModelInfo] = [
    ModelInfo(
        id="gemini-pro-orchestrator-g1-pro",
        label="Orchestrator (Argon)",
        tier="deep",
        cost_hint="quota hebdo",
    ),
    ModelInfo(
        id="gemini-4-argon-high",
        label="Gemini 4 Argon (High)",
        tier="deep",
        cost_hint="quota hebdo",
    ),
    ModelInfo(
        id="gemini-4-argon-medium",
        label="Gemini 4 Argon (Medium)",
        tier="standard",
        cost_hint="quota hebdo",
    ),
    ModelInfo(
        id="gemini-4-argon-low",
        label="Gemini 4 Argon (Low)",
        tier="light",
        cost_hint="quota hebdo",
    ),
    ModelInfo(
        id="gemini-4-hydrogen-xhigh",
        label="Gemini 4 Hydrogen (XHigh)",
        tier="critical",
        cost_hint="quota 5h",
    ),
    ModelInfo(
        id="gemini-4-hydrogen-high",
        label="Gemini 4 Hydrogen (High)",
        tier="deep",
        cost_hint="quota 5h",
    ),
    ModelInfo(
        id="gemini-4-hydrogen-medium",
        label="Gemini 4 Hydrogen (Medium)",
        tier="standard",
        cost_hint="quota 5h",
    ),
    ModelInfo(
        id="gemini-4-hydrogen-low",
        label="Gemini 4 Hydrogen (Low)",
        tier="light",
        cost_hint="quota 5h",
    ),
    ModelInfo(
        id="gemini-3.8-flash-high",
        label="Gemini 3.8 Flash (High)",
        tier="light",
        cost_hint="quota 5h (éco)",
    ),
    ModelInfo(
        id="gemini-3.8-flash-medium",
        label="Gemini 3.8 Flash (Medium)",
        tier="light",
        cost_hint="quota 5h (éco)",
    ),
    ModelInfo(
        id="gemini-3.8-flash-low",
        label="Gemini 3.8 Flash (Low)",
        tier="light",
        cost_hint="quota 5h (éco)",
    ),
    ModelInfo(
        id="claude-sonnet-5-5-high",
        label="Claude Sonnet 5.5 (High)",
        tier="deep",
        cost_hint="quota 5h",
    ),
    ModelInfo(
        id="claude-sonnet-5-5-medium",
        label="Claude Sonnet 5.5 (Medium)",
        tier="deep",
        cost_hint="quota 5h",
    ),
    ModelInfo(
        id="claude-sonnet-5-5-low",
        label="Claude Sonnet 5.5 (Low)",
        tier="standard",
        cost_hint="quota 5h",
    ),
    ModelInfo(
        id="claude-opus-5-5-high",
        label="Claude Opus 5.5 (High)",
        tier="critical",
        cost_hint="quota 5h (coûteux)",
    ),
    ModelInfo(
        id="claude-opus-5-5-medium",
        label="Claude Opus 5.5 (Medium)",
        tier="critical",
        cost_hint="quota 5h (coûteux)",
    ),
    ModelInfo(
        id="claude-opus-5-5-low",
        label="Claude Opus 5.5 (Low)",
        tier="deep",
        cost_hint="quota 5h (coûteux)",
    ),
    ModelInfo(
        id="gpt-oss-120b-medium",
        label="GPT-OSS 120B (Medium)",
        tier="standard",
        cost_hint="quota 5h",
    ),
]

_KNOWN_ANTIGRAVITY = {model.id: model for model in DEFAULT_ANTIGRAVITY_MODELS}


def _infer_antigravity_meta(model_id: str) -> tuple[str, str]:
    known = _KNOWN_ANTIGRAVITY.get(model_id)
    if known is not None:
        return known.tier, known.cost_hint
    low = model_id.lower()
    is_weekly = "argon" in low or "orchestrator" in low
    cost_hint = "quota hebdo" if is_weekly else "quota 5h"
    if "opus" in low or "xhigh" in low:
        tier = "critical"
        if not is_weekly:
            cost_hint = "quota 5h (coûteux)"
    elif "flash" in low or low.endswith("-low"):
        tier = "light"
        if "flash" in low:
            cost_hint = "quota 5h (éco)"
    elif "high" in low or "sonnet" in low or "orchestrator" in low:
        tier = "deep"
    else:
        tier = "standard"
    return tier, cost_hint


def _parse_percent(value: object) -> float | None:
    if isinstance(value, (int, float)):
        return max(0.0, min(100.0, float(value)))
    if isinstance(value, str):
        cleaned = value.strip().rstrip("%").strip().replace(",", ".")
        if not cleaned:
            return None
        try:
            return max(0.0, min(100.0, float(cleaned)))
        except ValueError:
            return None
    return None


def parse_antigravity_swarm_payload(payload: dict) -> tuple[list[ModelInfo], list[QuotaItem]]:
    """Parse la sortie de `swarm(action='models', all=True)` en ModelInfo + QuotaItem."""
    if not isinstance(payload, dict):
        return list(DEFAULT_ANTIGRAVITY_MODELS), []
    display_names = payload.get("display_names") if isinstance(payload.get("display_names"), dict) else {}
    quotas_map = payload.get("quotas") if isinstance(payload.get("quotas"), dict) else {}
    raw_models = payload.get("models")
    if isinstance(raw_models, list):
        model_ids = [str(item).strip() for item in raw_models if isinstance(item, str) and item.strip()]
    else:
        model_ids = []
    for key in (*display_names.keys(), *quotas_map.keys()):
        if isinstance(key, str) and key.strip() and key.strip() not in model_ids:
            model_ids.append(key.strip())

    models: list[ModelInfo] = []
    for model_id in model_ids:
        tier, cost_hint = _infer_antigravity_meta(model_id)
        known = _KNOWN_ANTIGRAVITY.get(model_id)
        label = str(display_names.get(model_id) or (known.label if known else model_id))
        models.append(ModelInfo(id=model_id, label=label, tier=tier, cost_hint=cost_hint))
    if not models:
        models = list(DEFAULT_ANTIGRAVITY_MODELS)

    per_model_quotas: list[QuotaItem] = []
    five_h_values: list[tuple[float, str | None]] = []
    weekly_values: list[tuple[float, str | None]] = []
    for model in models:
        raw_q = quotas_map.get(model.id)
        if not isinstance(raw_q, dict):
            continue
        pct = _parse_percent(raw_q.get("quota_remaining"))
        if pct is None:
            pct = _parse_percent(raw_q.get("remaining"))
        if pct is None:
            continue
        resets = raw_q.get("quota_resets") or raw_q.get("resets_at")
        resets_str = str(resets) if resets else None
        per_model_quotas.append(
            QuotaItem(
                id=model.id,
                label=model.label,
                used=round(100.0 - pct, 2),
                limit=100.0,
                remaining=round(pct, 2),
                resets_at=resets_str,
                status="live",
                detail=f"{pct:.0f} % restants ({model.cost_hint or 'Antigravity'})",
            )
        )
        if "hebdo" in model.cost_hint:
            weekly_values.append((pct, resets_str))
        else:
            five_h_values.append((pct, resets_str))

    summary_quotas: list[QuotaItem] = []
    if five_h_values:
        min_pct, resets_5h = min(five_h_values, key=lambda item: item[0])
        summary_quotas.append(
            QuotaItem(
                id="rolling_5h",
                label="Quota glissant 5 h",
                used=round(100.0 - min_pct, 2),
                limit=100.0,
                remaining=round(min_pct, 2),
                resets_at=resets_5h,
                status="live",
                detail=f"{min_pct:.0f} % min restant (fenêtre 5 h)",
            )
        )
    if weekly_values:
        min_pct_w, resets_w = min(weekly_values, key=lambda item: item[0])
        summary_quotas.append(
            QuotaItem(
                id="weekly",
                label="Quota hebdomadaire",
                used=round(100.0 - min_pct_w, 2),
                limit=100.0,
                remaining=round(min_pct_w, 2),
                resets_at=resets_w,
                status="live",
                detail=f"{min_pct_w:.0f} % min restant (fenêtre hebdo)",
            )
        )
    return models, [*summary_quotas, *per_model_quotas]


def resolve_agy_command() -> list[str] | None:
    agy = shutil.which("agy")
    if not agy:
        return None
    return [agy]


def _settings_dir() -> Path:
    return Path.home() / ".gemini" / "antigravity-cli"


def _credential_hints() -> bool:
    if os.environ.get("GEMINI_API_KEY") or os.environ.get("AGY_ADC_AUTH"):
        return True
    root = _settings_dir()
    try:
        if not root.exists():
            return False
        for name in ("settings.json", "oauth_creds.json", "auth.json"):
            if (root / name).is_file():
                return True
        cache = root / "cache"
        if cache.is_dir() and any(cache.iterdir()):
            return True
    except OSError:
        return False
    return False


class AntigravityProvider(BaseProvider):
    id = "antigravity"
    label = "Google Antigravity"

    async def status(self) -> ProviderSnapshot:
        command = resolve_agy_command()
        quotas = await self.fetch_quotas()
        models = list(DEFAULT_ANTIGRAVITY_MODELS)
        if not command:
            return self._snapshot(
                available=False,
                authenticated=None,
                detail="CLI `agy` introuvable. Installez Antigravity CLI.",
                login_hint=LOGIN_HINT,
                models=models,
                quotas=quotas,
            )
        if _credential_hints():
            return self._snapshot(
                available=True,
                authenticated=True,
                detail="CLI présente ; indices de session / clé locaux (valeur non affichée).",
                login_hint=LOGIN_HINT,
                models=models,
                quotas=quotas,
            )
        return self._snapshot(
            available=True,
            authenticated=False,
            detail="CLI présente sans indice de session. Lancez la connexion OAuth.",
            login_hint=LOGIN_HINT,
            models=models,
            quotas=quotas,
        )

    async def start_login(self) -> LoginResult:
        command = resolve_agy_command()
        if not command:
            return LoginResult(ok=False, detail="CLI `agy` introuvable.", started=False)
        try:
            # Session interactive : ouvre OAuth si besoin.
            spawn_detached([*command])
        except OSError as exc:
            return LoginResult(ok=False, detail=str(exc), started=False)
        return LoginResult(
            ok=True,
            started=True,
            detail=(
                "`agy` a démarré. Terminez OAuth dans le navigateur, "
                "fermez ensuite la session interactive, puis rafraîchissez Duo."
            ),
        )

    async def logout(self) -> LoginResult:
        command = resolve_agy_command()
        if not command:
            return LoginResult(ok=False, detail="CLI `agy` introuvable.")
        # Pas de flag logout CLI stable documenté hors commande slash.
        return LoginResult(
            ok=False,
            detail="Déconnectez-vous via `/logout` dans une session `agy` interactive.",
        )

    async def fetch_quotas(self) -> list[QuotaItem]:
        return [
            QuotaItem(
                id="rolling_5h",
                label="Quota glissant 5 h",
                status="unavailable",
                detail=HELP_USAGE,
            ),
            QuotaItem(
                id="weekly",
                label="Quota hebdomadaire",
                status="unavailable",
                detail=HELP_USAGE,
            ),
        ]
