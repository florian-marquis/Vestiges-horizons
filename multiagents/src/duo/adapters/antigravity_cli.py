"""Pilotage du CLI Google Antigravity (`agy`) en mode headless."""

from __future__ import annotations

import json

from duo.adapters.base import AgentAdapter
from duo.models import AgentRequest, AgentResult, CancelToken, HealthStatus
from duo.process_util import ProcessRegistry, run_command
from duo.protocol import build_prompt
from duo.providers.antigravity import _credential_hints, resolve_agy_command
from duo.security import redact

INSTALL_HINT = (
    "CLI `agy` introuvable. Installez Antigravity CLI, puis authentifiez-vous "
    "(page Comptes ou `agy` interactif)."
)


def extract_agy_output(stdout: str) -> tuple[str, str | None]:
    text = stdout.strip()
    if not text:
        return "", None
    # Une ligne JSON envelope, ou NDJSON result final.
    last_obj = None
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        try:
            obj = json.loads(stripped)
        except json.JSONDecodeError:
            continue
        if isinstance(obj, dict):
            last_obj = obj
    if last_obj is None:
        try:
            last_obj = json.loads(text)
        except json.JSONDecodeError:
            return text, None
    if not isinstance(last_obj, dict):
        return text, None
    # Envelope direct ou event result
    payload = last_obj.get("result") if last_obj.get("event") == "result" else last_obj
    if not isinstance(payload, dict):
        payload = last_obj
    session_id = payload.get("conversation_id")
    if not isinstance(session_id, str) or not session_id.strip():
        session_id = None
    else:
        session_id = session_id.strip()
    response = payload.get("response") or payload.get("text") or payload.get("content")
    if isinstance(response, str) and response.strip():
        return response.strip(), session_id
    if payload.get("status") and str(payload.get("status")).upper() not in {"SUCCESS", "OK"}:
        error = payload.get("error") or payload.get("status")
        return (str(error) if error else text), session_id
    return text, session_id


class AntigravityCliAdapter(AgentAdapter):
    name = "antigravity"

    def __init__(self, command: list[str] | None = None, runner=None, registry: ProcessRegistry | None = None, model: str = ""):
        self.command = command
        self.runner = runner or run_command
        self.registry = registry or ProcessRegistry()
        self.model = model

    def resolved_command(self) -> list[str] | None:
        return self.command if self.command is not None else resolve_agy_command()

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
        authenticated = True if _credential_hints() else False
        return HealthStatus(
            agent=self.name,
            available=True,
            authenticated=authenticated,
            detail=(
                "CLI agy présente ; indices de session locaux."
                if authenticated
                else "CLI agy présente. Authentifiez-vous via /comptes ou `agy`."
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
            "-p",
            prompt,
            "--output-format",
            "json",
        ]
        if request.resume_id:
            args.extend(["--conversation", request.resume_id])
        if request.allow_write:
            # Borné au bac Duo : l'orchestrateur fixe déjà le cwd workspace.
            args.append("--dangerously-skip-permissions")
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
        text, cli_session = extract_agy_output(stdout)
        if reason == "cancelled":
            return AgentResult(ok=False, cancelled=True, error="Interrompu", stderr=redact(stderr), session_id=cli_session)
        if reason == "timeout":
            return AgentResult(ok=False, timed_out=True, error="Délai dépassé", stderr=redact(stderr), session_id=cli_session)
        combined = f"{stdout}\n{stderr}".lower()
        if "authentication required" in combined or "not authenticated" in combined or "sign in" in combined:
            return AgentResult(
                ok=False,
                exit_code=code,
                error=(
                    "Antigravity n'est pas authentifié. "
                    "Ouvrez /comptes ou lancez `agy` pour terminer OAuth (keyring OS)."
                ),
                stderr=redact(stderr)[:4000],
                text=text,
                session_id=cli_session,
            )
        if code not in (0, None):
            detail = redact(stderr.strip() or text.strip() or f"code {code}")
            return AgentResult(ok=False, exit_code=code, error=detail[:2000], stderr=redact(stderr)[:4000], text=text, session_id=cli_session)
        return AgentResult(ok=True, text=text, exit_code=code, stderr=redact(stderr)[:4000], session_id=cli_session)
