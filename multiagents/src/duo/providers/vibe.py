"""Fournisseur Mistral Vibe — auth via `vibe --setup`, quotas best-effort."""

from __future__ import annotations

import json
import os
import shutil
import tomllib
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from duo.providers._spawn import spawn_detached
from duo.providers.base import BaseProvider, LoginResult, ModelInfo, ProviderSnapshot, QuotaItem

LOGIN_HINT = (
    "Duo lance `vibe --setup` (navigateur OAuth Mistral ou saisie de clé dans la CLI). "
    "Les secrets restent dans le keyring OS ou `~/.vibe` — jamais dans Duo."
)
HELP_QUOTA = "https://console.mistral.ai/"

DEFAULT_VIBE_MODELS: list[ModelInfo] = [
    ModelInfo(id="devstral-small-latest", label="Devstral Small", tier="light", cost_hint="éco"),
    ModelInfo(id="devstral-medium-latest", label="Devstral Medium", tier="standard", cost_hint="standard"),
    ModelInfo(id="codestral-latest", label="Codestral", tier="standard", cost_hint="standard"),
    ModelInfo(id="mistral-large-latest", label="Mistral Large", tier="deep", cost_hint="élevé"),
    ModelInfo(id="mistral-large-4-0-3", label="Mistral Large 4", tier="deep", cost_hint="élevé"),
]
_KNOWN_VIBE = {m.id: m for m in DEFAULT_VIBE_MODELS}


def _infer_vibe_meta(model_id: str) -> tuple[str, str]:
    known = _KNOWN_VIBE.get(model_id)
    if known is not None:
        return known.tier, known.cost_hint
    low = model_id.lower()
    if "small" in low or "mini" in low or "nemo" in low or "lite" in low:
        return "light", "éco"
    if "large" in low or "opus" in low:
        return "deep", "élevé"
    return "standard", "standard"


def resolve_vibe_command() -> list[str] | None:
    vibe = shutil.which("vibe")
    if not vibe:
        return None
    return [vibe]


def _read_mistral_api_key() -> str | None:
    """Lit la clé Mistral localement sans jamais la journaliser ni la stocker dans Duo."""
    env_key = os.environ.get("MISTRAL_API_KEY")
    if isinstance(env_key, str) and env_key.strip():
        return env_key.strip()
    env_file = Path.home() / ".vibe" / ".env"
    try:
        if not env_file.is_file():
            return None
        text = env_file.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("MISTRAL_API_KEY="):
            val = stripped.split("=", 1)[1].strip().strip("\"'")
            if val:
                return val
    return None


def _credential_present() -> bool:
    if _read_mistral_api_key():
        return True
    config = Path.home() / ".vibe" / "config.toml"
    try:
        return config.is_file()
    except OSError:
        return False


def _merge_vibe_models(primary_ids: list[str], extra: list[ModelInfo] | None = None) -> list[ModelInfo]:
    ordered: list[ModelInfo] = []
    seen: set[str] = set()
    candidates: list[ModelInfo] = []
    for mid in primary_ids:
        known = _KNOWN_VIBE.get(mid)
        if known:
            candidates.append(known)
        else:
            tier, cost = _infer_vibe_meta(mid)
            candidates.append(ModelInfo(id=mid, label=f"Vibe ({mid})", tier=tier, cost_hint=cost))
    if extra:
        candidates.extend(extra)
    candidates.extend(DEFAULT_VIBE_MODELS)
    for model in candidates:
        if model.id not in seen:
            seen.add(model.id)
            ordered.append(model)
    return ordered


def _models_from_config() -> list[ModelInfo]:
    config = Path.home() / ".vibe" / "config.toml"
    try:
        if not config.is_file():
            return list(DEFAULT_VIBE_MODELS)
        text = config.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return list(DEFAULT_VIBE_MODELS)

    primary_ids: list[str] = []
    extra_models: list[ModelInfo] = []
    try:
        parsed = tomllib.loads(text)
    except tomllib.TOMLDecodeError:
        parsed = None

    if isinstance(parsed, dict):
        active = parsed.get("active_model")
        if isinstance(active, str) and active.strip():
            primary_ids.append(active.strip())
        models_cfg = parsed.get("models")
        if isinstance(models_cfg, list):
            for entry in models_cfg:
                if isinstance(entry, dict):
                    name = entry.get("name") or entry.get("alias") or entry.get("id")
                    if isinstance(name, str) and name.strip():
                        tier, cost = _infer_vibe_meta(name.strip())
                        extra_models.append(
                            ModelInfo(id=name.strip(), label=f"Vibe ({name.strip()})", tier=tier, cost_hint=cost)
                        )
                elif isinstance(entry, str) and entry.strip():
                    tier, cost = _infer_vibe_meta(entry.strip())
                    extra_models.append(
                        ModelInfo(id=entry.strip(), label=f"Vibe ({entry.strip()})", tier=tier, cost_hint=cost)
                    )
    else:
        for line in text.splitlines():
            stripped = line.strip()
            if stripped.startswith("active_model"):
                parts = stripped.split("=", 1)
                if len(parts) == 2:
                    alias = parts[1].strip().strip("\"'")
                    if alias:
                        primary_ids.append(alias)

    return _merge_vibe_models(primary_ids, extra_models)


def fetch_mistral_api_models_and_limits(api_key: str) -> tuple[list[ModelInfo], list[QuotaItem]]:
    """Interroge `GET https://api.mistral.ai/v1/models` en best-effort (timeout 5 s)."""
    if not api_key:
        return [], []
    try:
        request = Request(
            "https://api.mistral.ai/v1/models",
            headers={
                "Authorization": f"Bearer {api_key}",
                "Accept": "application/json",
                "User-Agent": "duo-vibe/1.0",
            },
            method="GET",
        )
        with urlopen(request, timeout=5) as response:  # noqa: S310 — URL HTTPS fixe Mistral
            raw = response.read().decode("utf-8", errors="replace")
            resp_headers = {k.lower(): v for k, v in response.headers.items()}
        payload = json.loads(raw)
    except (HTTPError, URLError, TimeoutError, json.JSONDecodeError, ValueError, OSError):
        return [], []

    models: list[ModelInfo] = []
    data = payload.get("data") if isinstance(payload, dict) else None
    if isinstance(data, list):
        for item in data:
            if not isinstance(item, dict):
                continue
            mid = item.get("id")
            if not isinstance(mid, str) or not mid.strip():
                continue
            mid = mid.strip()
            # Prioriser les modèles de code / raisonnement utiles à Vibe
            low = mid.lower()
            if not any(tag in low for tag in ("devstral", "codestral", "mistral-large", "mistral-medium", "mistral-small", "vibe")):
                continue
            tier, cost = _infer_vibe_meta(mid)
            known = _KNOWN_VIBE.get(mid)
            label = known.label if known else str(item.get("name") or mid)
            models.append(ModelInfo(id=mid, label=label, tier=tier, cost_hint=cost))

    quotas: list[QuotaItem] = []
    req_rem = resp_headers.get("x-ratelimit-remaining-requests") or resp_headers.get("x-ratelimit-remaining")
    req_lim = resp_headers.get("x-ratelimit-limit-requests") or resp_headers.get("x-ratelimit-limit")
    tok_rem = resp_headers.get("x-ratelimit-remaining-tokens-minute") or resp_headers.get("x-ratelimit-remaining-tokens")
    tok_lim = resp_headers.get("x-ratelimit-limit-tokens-minute") or resp_headers.get("x-ratelimit-limit-tokens")
    for qid, qlabel, rem_raw, lim_raw in (
        ("ratelimit_requests", "Rate-limit requêtes Mistral", req_rem, req_lim),
        ("ratelimit_tokens", "Rate-limit tokens/min Mistral", tok_rem, tok_lim),
    ):
        if rem_raw is None:
            continue
        try:
            rem_val = float(rem_raw)
            lim_val = float(lim_raw) if lim_raw is not None else None
        except ValueError:
            continue
        used_val = (lim_val - rem_val) if lim_val is not None else None
        quotas.append(
            QuotaItem(
                id=qid,
                label=qlabel,
                used=used_val,
                limit=lim_val,
                remaining=rem_val,
                status="live",
                detail="En-tête x-ratelimit API Mistral (non mensuel).",
            )
        )
    return models, quotas


class VibeProvider(BaseProvider):
    id = "vibe"
    label = "Mistral Vibe"

    def _collect_models_and_quotas(self, *, query_api: bool) -> tuple[list[ModelInfo], list[QuotaItem]]:
        config_models = _models_from_config()
        api_models: list[ModelInfo] = []
        api_quotas: list[QuotaItem] = []
        if query_api:
            api_key = _read_mistral_api_key()
            if api_key:
                api_models, api_quotas = fetch_mistral_api_models_and_limits(api_key)
        seen: set[str] = set()
        merged_models: list[ModelInfo] = []
        for model in (*config_models, *api_models):
            if model.id not in seen:
                seen.add(model.id)
                merged_models.append(model)
        org_quota = QuotaItem(
            id="org_usage",
            label="Usage org Mistral",
            status="unavailable",
            detail=(
                "Pas d'endpoint / commande non interactive stable pour le % mensuel. "
                f"Consultez {HELP_QUOTA} ou renseignez un quota manuel."
            ),
        )
        return merged_models, [*api_quotas, org_quota]

    async def status(self) -> ProviderSnapshot:
        command = resolve_vibe_command()
        if not command:
            models, quotas = self._collect_models_and_quotas(query_api=False)
            return self._snapshot(
                available=False,
                authenticated=None,
                detail="CLI `vibe` introuvable. Installez Mistral Vibe CLI.",
                login_hint=LOGIN_HINT,
                models=models,
                quotas=quotas,
            )
        models, quotas = self._collect_models_and_quotas(query_api=True)
        if _credential_present():
            return self._snapshot(
                available=True,
                authenticated=True,
                detail="CLI présente ; indices d'identifiants locaux détectés (valeur non lue).",
                login_hint=LOGIN_HINT,
                models=models,
                quotas=quotas,
            )
        return self._snapshot(
            available=True,
            authenticated=False,
            detail="CLI présente, mais aucun indice d'auth local. Lancez la connexion.",
            login_hint=LOGIN_HINT,
            models=models,
            quotas=quotas,
        )

    async def start_login(self) -> LoginResult:
        command = resolve_vibe_command()
        if not command:
            return LoginResult(ok=False, detail="CLI `vibe` introuvable.", started=False)
        try:
            spawn_detached([*command, "--setup"])
        except OSError as exc:
            return LoginResult(ok=False, detail=str(exc), started=False)
        return LoginResult(
            ok=True,
            started=True,
            detail=(
                "`vibe --setup` démarré. Terminez OAuth ou la saisie de clé dans la CLI, "
                "puis rafraîchissez."
            ),
        )

    async def logout(self) -> LoginResult:
        return LoginResult(
            ok=False,
            detail=(
                "Déconnexion Vibe non exposée de façon stable. "
                "Retirez la clé du keyring / `~/.vibe/.env` ou relancez `vibe --setup`."
            ),
        )

    async def fetch_quotas(self) -> list[QuotaItem]:
        _, quotas = self._collect_models_and_quotas(query_api=True)
        return quotas
