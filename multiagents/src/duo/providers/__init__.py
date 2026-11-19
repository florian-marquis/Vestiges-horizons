"""Registre des fournisseurs OAuth / quotas."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

from duo.process_util import ProcessRegistry
from duo.providers.antigravity import AntigravityProvider, parse_antigravity_swarm_payload
from duo.providers.base import PROVIDER_IDS, PROVIDER_LABELS, BaseProvider, ModelInfo, ProviderSnapshot, QuotaItem
from duo.providers.copilot import CopilotProvider
from duo.providers.cursor import CursorProvider
from duo.providers.vibe import VibeProvider

__all__ = [
    "PROVIDER_IDS",
    "PROVIDER_LABELS",
    "ProviderRegistry",
    "build_default_providers",
    "compute_routing",
]

VALID_DIFFICULTIES = ("N1", "N2", "N3", "N4")

PROVIDER_DEFAULT_WINDOWS = {
    "antigravity": "antigravity",
    "copilot": "vscode",
    "vibe": "vscode",
    "cursor": "cursor",
}


def build_default_providers(
    *,
    registry: ProcessRegistry | None = None,
    cursor_model: str = "auto",
    copilot_model: str = "",
) -> dict[str, BaseProvider]:
    reg = registry or ProcessRegistry()
    return {
        "cursor": CursorProvider(registry=reg, model=cursor_model),
        "copilot": CopilotProvider(registry=reg, model=copilot_model),
        "vibe": VibeProvider(),
        "antigravity": AntigravityProvider(),
    }


class ProviderRegistry:
    def __init__(self, providers: dict[str, BaseProvider], meta_path: Path | None = None):
        self.providers = providers
        self.meta_path = meta_path

    def get(self, provider_id: str) -> BaseProvider:
        provider = self.providers.get(provider_id)
        if provider is None:
            raise KeyError(provider_id)
        return provider

    def _load_meta(self) -> dict:
        if self.meta_path is None or not self.meta_path.is_file():
            return {}
        try:
            data = json.loads(self.meta_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return {}
        return data if isinstance(data, dict) else {}

    def _save_meta(self, data: dict) -> None:
        if self.meta_path is None:
            return
        self.meta_path.parent.mkdir(parents=True, exist_ok=True)
        self.meta_path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")

    def remember_check(self, provider_id: str, snapshot: ProviderSnapshot) -> None:
        meta = self._load_meta()
        entry = meta.setdefault(provider_id, {})
        entry["last_check_at"] = datetime.now(timezone.utc).isoformat()
        entry["last_status"] = {
            "available": snapshot.available,
            "authenticated": snapshot.authenticated,
            "detail": snapshot.detail[:500],
        }
        # Pas de secrets — modèles préférés éventuels déjà non secrets.
        if snapshot.models and not entry.get("preferred_model"):
            entry["preferred_model"] = snapshot.models[0].id
        self._save_meta(meta)

    async def sync_provider(self, provider_id: str, payload: dict) -> dict:
        self.get(provider_id)
        if not isinstance(payload, dict):
            raise ValueError("Payload de synchronisation invalide.")
        meta = self._load_meta()
        entry = meta.setdefault(provider_id, {})
        now = datetime.now(timezone.utc).isoformat()

        swarm_data = payload.get("swarm") if isinstance(payload.get("swarm"), dict) else payload
        if provider_id == "antigravity" and (
            isinstance(swarm_data.get("models"), list) or isinstance(swarm_data.get("quotas"), dict)
        ):
            models, quotas = parse_antigravity_swarm_payload(swarm_data)
            entry["synced_models"] = [item.as_dict() for item in models]
            if quotas:
                entry["synced_quotas"] = [item.as_dict() for item in quotas]
            entry["synced_at"] = now

        if "quota_override" in payload:
            override = payload.get("quota_override")
            if override is None or (isinstance(override, dict) and override.get("clear")):
                entry.pop("quota_override", None)
            elif isinstance(override, dict):
                limit_val = override.get("limit")
                rem_val = override.get("remaining")
                used_val = override.get("used")
                limit_f = float(limit_val) if isinstance(limit_val, (int, float)) else 100.0
                rem_f = float(rem_val) if isinstance(rem_val, (int, float)) else None
                used_f = float(used_val) if isinstance(used_val, (int, float)) else None
                if used_f is None and rem_f is not None:
                    used_f = max(0.0, round(limit_f - rem_f, 2))
                if rem_f is None and used_f is not None:
                    rem_f = max(0.0, round(limit_f - used_f, 2))
                detail = str(override.get("detail") or "").strip()
                if not detail and rem_f is not None:
                    detail = f"{rem_f:.0f} % restants (manuel)" if limit_f == 100.0 else f"{rem_f:g}/{limit_f:g} restants (manuel)"
                item = QuotaItem(
                    id=str(override.get("id") or "manual_quota"),
                    label=str(override.get("label") or f"Quota manuel ({PROVIDER_LABELS.get(provider_id, provider_id)})"),
                    used=used_f,
                    limit=limit_f,
                    remaining=rem_f,
                    resets_at=str(override["resets_at"]) if override.get("resets_at") else None,
                    status=str(override.get("status") or "manual"),
                    detail=detail or "Défini manuellement",
                )
                entry["quota_override"] = item.as_dict()
                entry["synced_at"] = now

        if isinstance(payload.get("preferred_model"), str) and payload["preferred_model"].strip():
            entry["preferred_model"] = payload["preferred_model"].strip()
            entry["synced_at"] = now

        if "add_models" in payload and isinstance(payload.get("add_models"), list):
            learned = list(entry.get("learned_models") or [])
            by_id: dict[str, dict] = {
                m["id"]: m for m in learned if isinstance(m, dict) and m.get("id")
            }
            for raw_model in payload["add_models"]:
                if isinstance(raw_model, str) and raw_model.strip():
                    mid = raw_model.strip()
                    info = ModelInfo(id=mid, label=mid).as_dict()
                elif isinstance(raw_model, dict) and str(raw_model.get("id") or "").strip():
                    mid = str(raw_model["id"]).strip()
                    tier = str(raw_model.get("tier") or "standard").strip().lower()
                    if tier not in {"light", "standard", "deep", "critical"}:
                        tier = "standard"
                    info = ModelInfo(
                        id=mid,
                        label=str(raw_model.get("label") or mid).strip(),
                        tier=tier,
                        cost_hint=str(raw_model.get("cost_hint") or "").strip(),
                    ).as_dict()
                else:
                    continue
                by_id[mid] = info
            entry["learned_models"] = list(by_id.values())
            entry["synced_at"] = now

        if "remove_models" in payload and isinstance(payload.get("remove_models"), list):
            to_remove = {str(item).strip() for item in payload["remove_models"] if isinstance(item, str)}
            if to_remove:
                entry["learned_models"] = [
                    m
                    for m in (entry.get("learned_models") or [])
                    if isinstance(m, dict) and m.get("id") not in to_remove
                ]
                entry["synced_at"] = now

        self._save_meta(meta)
        return await self.snapshot(provider_id)

    async def snapshot(self, provider_id: str) -> dict:
        provider = self.get(provider_id)
        snap = await provider.status()
        self.remember_check(provider_id, snap)
        payload = snap.as_dict()
        meta = self._load_meta().get(provider_id) or {}
        if isinstance(meta.get("synced_models"), list) and meta["synced_models"]:
            payload["models"] = meta["synced_models"]
        learned = meta.get("learned_models")
        if isinstance(learned, list) and learned:
            by_id: dict[str, dict] = {}
            for m in learned:
                if isinstance(m, dict) and m.get("id"):
                    by_id[m["id"]] = m
            merged: list[dict] = list(by_id.values())
            for m in payload.get("models") or []:
                if isinstance(m, dict) and m.get("id") and m["id"] not in by_id:
                    merged.append(m)
            payload["models"] = merged
        quotas = list(payload.get("quotas") or [])
        all_unavailable = not quotas or all(q.get("status") == "unavailable" for q in quotas)
        if all_unavailable and isinstance(meta.get("synced_quotas"), list) and meta["synced_quotas"]:
            payload["quotas"] = meta["synced_quotas"]
            if provider_id == "antigravity" and payload.get("authenticated") is None:
                payload["authenticated"] = True
                payload["detail"] = "Quotas Antigravity synchronisés depuis la session active."
        override = meta.get("quota_override")
        if isinstance(override, dict) and override:
            filtered = [q for q in payload.get("quotas") or [] if q.get("status") != "unavailable"]
            payload["quotas"] = [override, *filtered]
        payload["last_check_at"] = meta.get("last_check_at")
        payload["preferred_model"] = meta.get("preferred_model")
        payload["synced_at"] = meta.get("synced_at")
        return payload

    async def list_all(self) -> list[dict]:
        return [await self.snapshot(provider_id) for provider_id in PROVIDER_IDS]

    async def recommend_routing(
        self,
        difficulty: str = "N2",
        current_pilote_provider: str = "antigravity",
    ) -> dict:
        snapshots = {provider_id: await self.snapshot(provider_id) for provider_id in PROVIDER_IDS}
        return compute_routing(
            snapshots,
            difficulty=difficulty,
            current_pilote_provider=current_pilote_provider,
        )


def _normalize_difficulty(difficulty: str | None) -> str:
    value = str(difficulty or "N2").strip().upper()
    return value if value in VALID_DIFFICULTIES else "N2"


def _quota_percent_for(snap: dict, model_id: str | None = None) -> tuple[float | None, str]:
    quotas = snap.get("quotas") or []
    live_statuses = {"live", "manual"}
    if model_id:
        for item in quotas:
            if item.get("id") == model_id and item.get("status") in live_statuses:
                pct = _extract_quota_pct(item)
                if pct is not None:
                    return pct, str(item.get("detail") or "")
        low = model_id.lower()
        if snap.get("id") == "antigravity":
            target_id = "weekly" if ("argon" in low or "orchestrator" in low) else "rolling_5h"
            for item in quotas:
                if item.get("id") == target_id and item.get("status") in live_statuses:
                    pct = _extract_quota_pct(item)
                    if pct is not None:
                        return pct, str(item.get("detail") or "")
    for item in quotas:
        if item.get("status") in live_statuses:
            pct = _extract_quota_pct(item)
            if pct is not None:
                return pct, str(item.get("detail") or "")
    return None, ""


def _extract_quota_pct(item: dict) -> float | None:
    rem = item.get("remaining")
    lim = item.get("limit")
    used = item.get("used")
    if isinstance(rem, (int, float)) and isinstance(lim, (int, float)) and lim > 0:
        return max(0.0, min(100.0, (float(rem) / float(lim)) * 100.0))
    if isinstance(rem, (int, float)) and lim is None:
        return max(0.0, min(100.0, float(rem)))
    if isinstance(used, (int, float)) and isinstance(lim, (int, float)) and lim > 0:
        return max(0.0, min(100.0, ((float(lim) - float(used)) / float(lim)) * 100.0))
    return None


def _find_model(snap: dict, model_id: str) -> dict | None:
    for model in snap.get("models") or []:
        if model.get("id") == model_id:
            return model
    return None


def _provider_usable(snap: dict) -> bool:
    if snap.get("authenticated") is False:
        return False
    # Si un quota live/manual existe ou si la CLI/session est disponible, on considère utilisable
    has_live = any(q.get("status") in {"live", "manual"} for q in snap.get("quotas") or [])
    return bool(snap.get("available") or has_live or snap.get("authenticated") is True)


def _is_exhausted(pct: float | None, threshold: float = 5.0) -> bool:
    return pct is not None and pct <= threshold


def _pick_first_available(
    snapshots: dict[str, dict],
    candidates: list[tuple[str, str, str]],
    *,
    min_pct: float = 5.0,
) -> tuple[str, str, str, float | None, str]:
    """Choisit le premier candidat utilisable et non épuisé, sinon le meilleur repli."""
    for require_usable in (True, False):
        for provider_id, model_id, rationale in candidates:
            snap = snapshots.get(provider_id) or {}
            if snap.get("authenticated") is False:
                continue
            if require_usable and not _provider_usable(snap):
                continue
            pct, q_detail = _quota_percent_for(snap, model_id)
            if pct is not None and pct <= min_pct:
                continue
            model = _find_model(snap, model_id)
            label = (model or {}).get("label") or model_id
            return provider_id, model_id, label, pct, rationale or q_detail
    # Repli ultime : premier candidat dont la auth n'est pas False
    for provider_id, model_id, rationale in candidates:
        snap = snapshots.get(provider_id) or {}
        if snap.get("authenticated") is False:
            continue
        pct, _ = _quota_percent_for(snap, model_id)
        model = _find_model(snap, model_id)
        label = (model or {}).get("label") or model_id
        return provider_id, model_id, label, pct, rationale
    provider_id, model_id, rationale = candidates[0]
    return provider_id, model_id, model_id, None, rationale


_PILOTE_CANDIDATES: dict[str, list[tuple[str, str, str]]] = {
    "N1": [
        ("antigravity", "gemini-3.8-flash-high", "Quota 5 h économique, suffisant pour audit/doc/trivial."),
        ("antigravity", "gemini-4-argon-low", "Quota hebdo léger pour coordination N1."),
        ("copilot", "gpt-4.1", "Modèle 0x inclus côté Copilot si quota Antigravity bas."),
        ("vibe", "devstral-small-latest", "Modèle léger Vibe en repli économique."),
        ("cursor", "auto", "Secours Cursor maison."),
    ],
    "N2": [
        ("antigravity", "gemini-pro-orchestrator-g1-pro", "Orchestrator (Argon) adapté au pilotage standard."),
        ("antigravity", "gemini-4-hydrogen-medium", "Quota 5 h équilibré pour découpage et contrôle N2."),
        ("copilot", "gemini-3-flash", "Repli Pilote Copilot 0.33x si Antigravity épuisé."),
        ("copilot", "gpt-4.1", "Repli Pilote Copilot 0x inclus."),
        ("vibe", "devstral-medium-latest", "Repli Pilote Mistral Vibe."),
        ("cursor", "auto", "Secours Cursor maison."),
    ],
    "N3": [
        ("antigravity", "gemini-pro-orchestrator-g1-pro", "Orchestrator (Argon) recommandé pour invariants et architecture."),
        ("antigravity", "gemini-4-hydrogen-high", "Haute capacité de raisonnement (quota 5 h)."),
        ("antigravity", "claude-sonnet-5-5-high", "Alternative solide sur quota 5 h."),
        ("copilot", "claude-sonnet-4.6", "Repli Pilote Copilot 1x si Antigravity épuisé."),
        ("vibe", "mistral-large-latest", "Repli Pilote Mistral Large."),
        ("cursor", "claude-4.6-sonnet", "Escalade Pilote Cursor extérieur."),
    ],
    "N4": [
        ("antigravity", "claude-opus-5-5-high", "Raisonnement maximal pour lot critique ou escalade."),
        ("antigravity", "gemini-4-hydrogen-xhigh", "Alternative XHigh sur quota 5 h."),
        ("antigravity", "gemini-pro-orchestrator-g1-pro", "Orchestrator (Argon) hebdomadaire."),
        ("copilot", "claude-opus-4.6", "Escalade Pilote Copilot 3x si Antigravity épuisé."),
        ("cursor", "claude-4.6-opus", "Escalade ultime Pilote Cursor."),
    ],
}


def compute_routing(
    snapshots: dict[str, dict],
    *,
    difficulty: str = "N2",
    current_pilote_provider: str = "antigravity",
) -> dict:
    """Calcule l'affectation recommandée (outil + modèle) pour le Pilote et les Agents."""
    diff = _normalize_difficulty(difficulty)
    cur_pilote = (current_pilote_provider or "antigravity").strip().lower()
    if cur_pilote not in PROVIDER_IDS:
        cur_pilote = "antigravity"

    candidates = list(_PILOTE_CANDIDATES[diff])
    # Privilégier d'abord un candidat non épuisé de l'outil courant du Pilote (évite les bascules inutiles),
    # puis basculer vers un autre outil si tous les modèles adaptés de l'outil courant sont épuisés.
    same_tool = [c for c in candidates if c[0] == cur_pilote]
    other_tools = [c for c in candidates if c[0] != cur_pilote]
    ordered_pilote = [*same_tool, *other_tools]

    p_prov, p_mod, p_label, p_pct, p_reason = _pick_first_available(snapshots, ordered_pilote)
    fallback_candidates = [c for c in ordered_pilote if not (c[0] == p_prov and c[1] == p_mod)]
    fb_prov, fb_mod, fb_label, fb_pct, _ = _pick_first_available(snapshots, fallback_candidates or ordered_pilote)

    pilote_rec = {
        "role": "pilote",
        "provider": p_prov,
        "window": PROVIDER_DEFAULT_WINDOWS.get(p_prov, "vscode"),
        "model": p_mod,
        "model_label": p_label,
        "quota_remaining_pct": p_pct,
        "switch_tool_recommended": p_prov != cur_pilote,
        "reason": p_reason,
        "fallback_provider": fb_prov,
        "fallback_window": PROVIDER_DEFAULT_WINDOWS.get(fb_prov, "vscode"),
        "fallback_model": fb_mod,
        "fallback_model_label": fb_label,
        "fallback_quota_remaining_pct": fb_pct,
    }

    # Agents d'exécution standard : agent2 = Copilot, agent3 = Vibe, agent1 = Cursor (secours/escalade)
    copilot_snap = snapshots.get("copilot") or {}
    copilot_pct, _ = _quota_percent_for(copilot_snap, "premium_interactions")
    vibe_snap = snapshots.get("vibe") or {}
    vibe_pct, _ = _quota_percent_for(vibe_snap)
    cursor_snap = snapshots.get("cursor") or {}
    cursor_pct, _ = _quota_percent_for(cursor_snap)

    copilot_exhausted = _is_exhausted(copilot_pct)
    vibe_exhausted = _is_exhausted(vibe_pct)

    # Choix modèle Copilot (agent2)
    if diff == "N1":
        cop_model = "gpt-4.1"
    elif diff == "N2":
        cop_model = "gpt-4.1" if (copilot_pct is not None and copilot_pct < 25.0) else "gemini-3-flash"
    elif diff == "N3":
        cop_model = "gpt-4.1" if copilot_exhausted else "claude-sonnet-4.6"
    else:
        if copilot_exhausted:
            cop_model = "gpt-4.1"
        elif copilot_pct is not None and copilot_pct < 20.0:
            cop_model = "claude-sonnet-4.6"
        else:
            cop_model = "claude-opus-4.6"
    cop_info = _find_model(copilot_snap, cop_model) or {}

    # Choix modèle Vibe (agent3)
    vibe_deep = (
        "mistral-large-4-0-3"
        if _find_model(vibe_snap, "mistral-large-4-0-3")
        else next(
            (
                m["id"]
                for m in (vibe_snap.get("models") or [])
                if isinstance(m, dict) and m.get("tier") in {"deep", "critical"} and m.get("id")
            ),
            "mistral-large-latest",
        )
    )
    vibe_map = {
        "N1": "devstral-small-latest",
        "N2": "devstral-medium-latest",
        "N3": vibe_deep,
        "N4": vibe_deep,
    }
    vibe_model = vibe_map[diff]
    vibe_info = _find_model(vibe_snap, vibe_model) or {}

    # Choix modèle Cursor (agent1)
    cursor_map = {
        "N1": "auto",
        "N2": "grok-4.7",
        "N3": "claude-4.6-sonnet",
        "N4": "claude-4.6-opus",
    }
    cur_model = cursor_map[diff]
    cur_info = _find_model(cursor_snap, cur_model) or {}

    # Priorité d'activation
    if diff == "N1":
        cop_prio = "principal"
        vibe_prio = "secondaire"
        cur_prio = "secours (veille)"
    elif diff == "N2":
        vibe_prio = "secondaire (épuisé)" if vibe_exhausted else "principal"
        cop_prio = "principal" if (vibe_exhausted or not copilot_exhausted) else "secondaire (0x)"
        cur_prio = "secours (actif)" if (vibe_exhausted and copilot_exhausted) else "secours (veille)"
    elif diff == "N3":
        cop_prio = "secondaire (quota bas)" if copilot_exhausted else "principal"
        vibe_prio = "principal" if copilot_exhausted else "secondaire (audit)"
        cur_prio = "escalade (actif)" if (vibe_exhausted and copilot_exhausted) else "escalade (veille)"
    else:
        cop_prio = "secondaire" if copilot_exhausted else "principal"
        vibe_prio = "secondaire"
        cur_prio = "escalade (actif)"

    agents_rec = [
        {
            "role": "agent1",
            "provider": "cursor",
            "window": "cursor",
            "model": cur_model,
            "model_label": cur_info.get("label") or cur_model,
            "cost_hint": cur_info.get("cost_hint") or "quota maison",
            "quota_remaining_pct": cursor_pct,
            "active_priority": cur_prio,
        },
        {
            "role": "agent2",
            "provider": "copilot",
            "window": "vscode",
            "model": cop_model,
            "model_label": cop_info.get("label") or cop_model,
            "cost_hint": cop_info.get("cost_hint") or "0x",
            "quota_remaining_pct": copilot_pct,
            "active_priority": cop_prio,
        },
        {
            "role": "agent3",
            "provider": "vibe",
            "window": "vscode",
            "model": vibe_model,
            "model_label": vibe_info.get("label") or vibe_model,
            "cost_hint": vibe_info.get("cost_hint") or "standard",
            "quota_remaining_pct": vibe_pct,
            "active_priority": vibe_prio,
        },
    ]

    # Profil stable de session : 1 seul modèle fixe par agent pour éviter les switchs répétés côté humain.
    if vibe_exhausted and not copilot_exhausted:
        simple_agent = {
            "role": "agent3",
            "provider": "vibe",
            "window": "vscode",
            "model": "devstral-small-latest",
            "model_label": (_find_model(vibe_snap, "devstral-small-latest") or {}).get("label") or "Devstral Small",
            "cost_hint": "éco",
            "handles": ["N1"],
        }
        complex_agent = {
            "role": "agent2",
            "provider": "copilot",
            "window": "vscode",
            "model": "claude-sonnet-4.6",
            "model_label": (_find_model(copilot_snap, "claude-sonnet-4.6") or {}).get("label") or "Claude Sonnet 4.6",
            "cost_hint": "1x",
            "handles": ["N2", "N3"],
        }
        profile_mode = "inverted_vibe_exhausted"
    else:
        simple_agent = {
            "role": "agent2",
            "provider": "copilot",
            "window": "vscode",
            "model": "gpt-4.1",
            "model_label": (_find_model(copilot_snap, "gpt-4.1") or {}).get("label") or "GPT-4.1",
            "cost_hint": "0x (inclus)",
            "handles": ["N1", "N2"],
        }
        complex_agent = {
            "role": "agent3",
            "provider": "vibe",
            "window": "vscode",
            "model": vibe_deep,
            "model_label": (_find_model(vibe_snap, vibe_deep) or {}).get("label") or vibe_deep,
            "cost_hint": (_find_model(vibe_snap, vibe_deep) or {}).get("cost_hint") or "élevé",
            "handles": ["N2", "N3"],
        }
        profile_mode = "stable_session"

    escalation_agent = {
        "role": "agent1",
        "provider": "cursor",
        "window": "cursor",
        "model": "claude-4.6-opus" if diff == "N4" else "auto",
        "model_label": (_find_model(cursor_snap, "claude-4.6-opus" if diff == "N4" else "auto") or {}).get("label")
        or ("Claude Opus (Cursor)" if diff == "N4" else "Auto / Cursor 2.5"),
        "cost_hint": "quota extérieur" if diff == "N4" else "quota maison",
        "handles": ["N4"],
    }

    if diff == "N1":
        assigned_role = simple_agent["role"]
    elif diff == "N2":
        assigned_role = simple_agent["role"]
    elif diff == "N3":
        assigned_role = complex_agent["role"]
    else:
        assigned_role = escalation_agent["role"]

    requires_human_model_switch = bool(
        pilote_rec["switch_tool_recommended"] or (diff == "N4") or (vibe_exhausted and not copilot_exhausted)
    )

    session_profile = {
        "mode": profile_mode,
        "pilote": {
            "role": "pilote",
            "provider": p_prov,
            "window": pilote_rec["window"],
            "model": p_mod,
            "model_label": p_label,
        },
        "simple_agent": simple_agent,
        "complex_agent": complex_agent,
        "escalation_agent": escalation_agent,
    }

    return {
        "difficulty": diff,
        "assigned_role": assigned_role,
        "requires_human_model_switch": requires_human_model_switch,
        "session_profile": session_profile,
        "pilote": pilote_rec,
        "agents": agents_rec,
        "providers": list(snapshots.values()),
    }
