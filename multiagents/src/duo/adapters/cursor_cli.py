"""Pilotage du CLI Cursor `agent`, s'il est installé et authentifié."""

from __future__ import annotations

from pathlib import Path

from duo.adapters.base import AgentAdapter
from duo.adapters.cli_common import extract_agent_output, resolve_cursor_agent_command
from duo.models import AgentRequest, AgentResult, CancelToken, HealthStatus
from duo.process_util import ProcessRegistry, run_command
from duo.protocol import build_prompt
from duo.security import redact

INSTALL_HINT = (
    "Le CLI Cursor `agent` est absent. La commande `cursor` de l'IDE ne lance pas l'agent. "
    "Installez le CLI officiel dans PowerShell : irm 'https://cursor.com/install?win32=true' | iex. "
    "Puis exécutez `agent login` ou définissez CURSOR_API_KEY."
)


class CursorCliAdapter(AgentAdapter):
    name = "cursor"

    def __init__(self, command: list[str] | None = None, runner=None, registry: ProcessRegistry | None = None, model: str = "auto"):
        self.command = command
        self.runner = runner or run_command
        self.registry = registry or ProcessRegistry()
        self.model = model or "auto"

    def resolved_command(self) -> list[str] | None:
        return self.command if self.command is not None else resolve_cursor_agent_command()

    async def health(self) -> HealthStatus:
        command = self.resolved_command()
        if not command:
            return HealthStatus(
                agent=self.name,
                available=False,
                authenticated=None,
                detail=INSTALL_HINT,
                unsupported=["run", "auth"],
            )
        token = CancelToken()
        code, stdout, stderr, reason = await self.runner(
            [*command, "status"],
            str(Path.cwd()),
            20,
            token,
            self.registry,
            "health-cursor",
        )
        blob = f"{stdout}\n{stderr}".lower()
        if reason == "timeout":
            return HealthStatus(
                agent=self.name,
                available=True,
                authenticated=None,
                detail="Le CLI agent est installé, mais `agent status` n'a pas répondu à temps.",
                unsupported=["cout"],
            )
        if "not logged in" in blob:
            return HealthStatus(
                agent=self.name,
                available=True,
                authenticated=False,
                detail=(
                    "Le CLI agent est installé. `agent status` indique qu'aucune session n'est ouverte. "
                    "Exécutez `agent login` dans un terminal, ou définissez CURSOR_API_KEY. "
                    "Aucun identifiant n'est affiché."
                ),
                unsupported=["cout"],
            )
        if code == 0 and "logged in" in blob:
            return HealthStatus(
                agent=self.name,
                available=True,
                authenticated=True,
                detail="Le CLI agent répond et une session est ouverte. Le compte n'est pas affiché.",
                unsupported=["cout"],
            )
        return HealthStatus(
            agent=self.name,
            available=True,
            authenticated=None,
            detail="Le CLI agent est installé, mais son statut d'authentification n'a pas pu être lu.",
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
            "--trust",
            "--workspace",
            workspace,
            "--model",
            self.model,
        ]
        if request.resume_id:
            args.extend(["--resume", request.resume_id])
        if request.allow_write:
            args.append("--force")
        else:
            args.extend(["--mode", "ask"])
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
        text, cli_session = extract_agent_output(stdout)
        if reason == "cancelled":
            return AgentResult(ok=False, cancelled=True, error="Interrompu", stderr=redact(stderr), session_id=cli_session)
        if reason == "timeout":
            return AgentResult(ok=False, timed_out=True, error="Délai dépassé", stderr=redact(stderr), session_id=cli_session)
        if code not in (0, None):
            detail = redact(stderr.strip() or text.strip() or f"code {code}")
            return AgentResult(
                ok=False,
                exit_code=code,
                error=detail[:2000],
                stderr=redact(stderr)[:4000],
                text=text,
                session_id=cli_session,
            )
        return AgentResult(ok=True, text=text, exit_code=code, stderr=redact(stderr)[:4000], session_id=cli_session)
