"""Pilotage du CLI Mistral Vibe en mode programmatique."""

from __future__ import annotations

import json

from duo.adapters.base import AgentAdapter
from duo.models import AgentRequest, AgentResult, CancelToken, HealthStatus
from duo.process_util import ProcessRegistry, run_command
from duo.protocol import build_prompt
from duo.providers.vibe import _credential_present, resolve_vibe_command
from duo.security import redact

INSTALL_HINT = (
    "CLI `vibe` introuvable. Installez Mistral Vibe, puis `vibe --setup` "
    "(OAuth ou clé — stockée hors Duo)."
)


def extract_vibe_output(stdout: str) -> tuple[str, str | None]:
    """Extrait le dernier message assistant d'une sortie `--output json`."""
    text = stdout.strip()
    if not text:
        return "", None
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return text, None
    session_id = None
    if isinstance(payload, dict):
        session_id = payload.get("session_id") or payload.get("id")
        if isinstance(session_id, str) and session_id.strip():
            session_id = session_id.strip()
        else:
            session_id = None
        messages = payload.get("messages")
        if isinstance(messages, list):
            payload = messages
        else:
            for key in ("result", "text", "content", "response"):
                value = payload.get(key)
                if isinstance(value, str) and value.strip():
                    return value.strip(), session_id
            return text, session_id
    if isinstance(payload, list):
        for item in reversed(payload):
            if not isinstance(item, dict):
                continue
            role = str(item.get("role") or item.get("type") or "").lower()
            content = item.get("content") or item.get("text") or item.get("message")
            if isinstance(content, list):
                parts = []
                for block in content:
                    if isinstance(block, str):
                        parts.append(block)
                    elif isinstance(block, dict) and isinstance(block.get("text"), str):
                        parts.append(block["text"])
                content = "".join(parts)
            if isinstance(content, str) and content.strip():
                if role in {"", "assistant", "model", "agent"}:
                    return content.strip(), session_id
        return text, session_id
    return text, session_id


class VibeCliAdapter(AgentAdapter):
    name = "vibe"

    def __init__(self, command: list[str] | None = None, runner=None, registry: ProcessRegistry | None = None, model: str = ""):
        self.command = command
        self.runner = runner or run_command
        self.registry = registry or ProcessRegistry()
        self.model = model

    def resolved_command(self) -> list[str] | None:
        return self.command if self.command is not None else resolve_vibe_command()

    async def health(self) -> HealthStatus:
        command = self.resolved_command()
        if not command:
            return HealthStatus(
                agent=self.name,
                available=False,
                authenticated=None,
                detail=INSTALL_HINT,
                unsupported=["run"],
            )
        authenticated = True if _credential_present() else False
        return HealthStatus(
            agent=self.name,
            available=True,
            authenticated=authenticated,
            detail=(
                "CLI Vibe présente ; indices d'auth locaux détectés."
                if authenticated
                else "CLI Vibe présente. Exécutez `vibe --setup` (page Comptes)."
            ),
            unsupported=["cout"],
        )

    def build_args(self, request: AgentRequest, workspace: str) -> list[str]:
        command = self.resolved_command()
        if not command:
            raise RuntimeError(INSTALL_HINT)
        prompt = build_prompt(request)
        args = [
            *command,
            "--prompt",
            prompt,
            "--output",
            "json",
            "--trust",
            "--workdir",
            workspace,
            "--max-turns",
            "8",
        ]
        if request.allow_write:
            args.append("--auto-approve")
        else:
            args.extend(["--disabled-tools", "bash*", "--disabled-tools", "write*", "--disabled-tools", "edit*"])
        if self.model:
            args.extend(["--model", self.model])
        return args

    async def run(self, request: AgentRequest, token: CancelToken) -> AgentResult:
        if token.cancelled:
            return AgentResult(ok=False, cancelled=token.reason == "stop", timed_out=token.reason == "timeout", error="Interrompu")
        try:
            args = self.build_args(request, request.workspace)
        except RuntimeError as exc:
            return AgentResult(ok=False, error=str(exc), unsupported=["run"])
        code, stdout, stderr, reason = await self.runner(
            args,
            request.workspace,
            request.timeout,
            token,
            self.registry,
            request.session_id,
        )
        text, cli_session = extract_vibe_output(stdout)
        if reason == "cancelled":
            return AgentResult(ok=False, cancelled=True, error="Interrompu", stderr=redact(stderr), session_id=cli_session)
        if reason == "timeout":
            return AgentResult(ok=False, timed_out=True, error="Délai dépassé", stderr=redact(stderr), session_id=cli_session)
        combined = f"{stdout}\n{stderr}".lower()
        if "api key" in combined or "not authenticated" in combined or "unauthorized" in combined:
            return AgentResult(
                ok=False,
                exit_code=code,
                error="Vibe n'est pas authentifié. Ouvrez /comptes ou exécutez `vibe --setup`.",
                stderr=redact(stderr)[:4000],
                text=text,
                session_id=cli_session,
            )
        if code not in (0, None):
            detail = redact(stderr.strip() or text.strip() or f"code {code}")
            return AgentResult(ok=False, exit_code=code, error=detail[:2000], stderr=redact(stderr)[:4000], text=text, session_id=cli_session)
        return AgentResult(ok=True, text=text, exit_code=code, stderr=redact(stderr)[:4000], session_id=cli_session)
