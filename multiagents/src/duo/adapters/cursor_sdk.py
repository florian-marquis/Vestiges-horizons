"""Présence du SDK Cursor. Il n'est pas utilisé pour écrire dans le dépôt."""

from __future__ import annotations

import importlib.util
import os

from duo.adapters.base import AgentAdapter
from duo.models import AgentRequest, AgentResult, CancelToken, HealthStatus


class CursorSdkAdapter(AgentAdapter):
    name = "cursor_sdk"

    async def health(self) -> HealthStatus:
        installed = importlib.util.find_spec("cursor_sdk") is not None
        has_key = bool(os.environ.get("CURSOR_API_KEY"))
        if not installed:
            return HealthStatus(
                agent=self.name,
                available=False,
                authenticated=None,
                detail="Le paquet Python cursor-sdk n'est pas installé.",
                unsupported=["run"],
            )
        if not has_key:
            return HealthStatus(
                agent=self.name,
                available=False,
                authenticated=False,
                detail="cursor-sdk est installé, mais CURSOR_API_KEY est absente. La valeur n'est pas affichée.",
                unsupported=["run"],
            )
        return HealthStatus(
            agent=self.name,
            available=False,
            authenticated=True,
            detail=(
                "La clé est présente, mais cet adaptateur n'est pas branché : "
                "un agent SDK local peut modifier des fichiers sans le contrôle d'outils du CLI. "
                "Le pilotage retenu est le CLI `agent`."
            ),
            unsupported=["run"],
        )

    async def run(self, request: AgentRequest, token: CancelToken) -> AgentResult:
        del request, token
        return AgentResult(
            ok=False,
            error="L'adaptateur cursor-sdk n'est pas branché sur l'orchestrateur.",
            unsupported=["run"],
        )
