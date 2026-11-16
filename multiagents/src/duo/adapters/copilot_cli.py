"""Pilotage du GitHub Copilot CLI officiel, sans shell."""

from __future__ import annotations

from duo.adapters.base import AgentAdapter
from duo.adapters.cli_common import extract_agent_output, resolve_copilot_command
from duo.models import AgentRequest, AgentResult, CancelToken, HealthStatus
from duo.process_util import ProcessRegistry, run_command
from duo.protocol import build_prompt
from duo.security import redact


class CopilotCliAdapter(AgentAdapter):
    name = "copilot"

    def __init__(self, command: list[str] | None = None, runner=None, registry: ProcessRegistry | None = None, model: str = ""):
        self.command = command
        self.runner = runner or run_command
        self.registry = registry or ProcessRegistry()
        self.model = model

    def resolved_command(self) -> list[str] | None:
        return self.command if self.command is not None else resolve_copilot_command()

    async def health(self) -> HealthStatus:
        command = self.resolved_command()
        if not command:
            return HealthStatus(
                agent=self.name,
                available=False,
                authenticated=None,
                detail="GitHub Copilot CLI est introuvable. Installez-le avec npm i -g @github/copilot.",
                unsupported=["run"],
            )
        return HealthStatus(
            agent=self.name,
            available=True,
            authenticated=None,
            detail=(
                "CLI détectée. L'authentification n'est pas prouvée tant qu'une sonde "
                "`copilot -p` n'a pas répondu. Aucun jeton n'est lu ni affiché."
            ),
            unsupported=["cout"],
        )

    def build_args(self, request: AgentRequest, workspace: str) -> list[str]:
        command = self.resolved_command()
        if not command:
            raise RuntimeError("GitHub Copilot CLI est indisponible.")
        prompt = build_prompt(request)
        args = [
            *command,
            "-p",
            prompt,
            "-s",
            "--no-ask-user",
            "--output-format",
            "json",
            "--no-color",
            "-C",
            workspace,
            "--add-dir",
            workspace,
        ]
        if self.model:
            args.extend(["--model", self.model])
        if request.resume_id:
            args.extend(["--resume", request.resume_id])
        if request.allow_write:
            args.extend(["--allow-tool", "write"])
        else:
            args.extend(["--deny-tool", "shell", "--deny-tool", "write"])
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
        combined = f"{stdout}\n{stderr}"
        if "no authentication information found" in combined.lower():
            return AgentResult(
                ok=False,
                exit_code=code,
                error=(
                    "GitHub Copilot CLI n'est pas authentifié. "
                    "Exécutez `copilot login`, ou définissez COPILOT_GITHUB_TOKEN "
                    "(jeton fin à permission Copilot Requests). Aucun jeton n'est affiché."
                ),
                stderr=redact(stderr)[:4000],
                text=text,
                session_id=cli_session,
            )
        if code not in (0, None):
            detail = redact(stderr.strip() or text.strip() or f"code {code}")
            return AgentResult(ok=False, exit_code=code, error=detail[:2000], stderr=redact(stderr)[:4000], text=text, session_id=cli_session)
        return AgentResult(ok=True, text=text, exit_code=code, stderr=redact(stderr)[:4000], session_id=cli_session)
