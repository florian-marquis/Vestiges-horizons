"""Point d'entrée local."""

from __future__ import annotations

import argparse
import asyncio
import sys

import uvicorn

from duo import __version__
from duo.adapters.copilot_cli import CopilotCliAdapter
from duo.adapters.cursor_cli import CursorCliAdapter
from duo.api import create_app
from duo.config import load_settings
from duo.models import AgentRequest, CancelToken
from duo.protocol import parse_payload
from duo.security import redact


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description="Duo, collaboration locale Cursor et Copilot.")
    parser.add_argument("--probe", action="store_true", help="Sonde les CLI sans afficher de secret.")
    parser.add_argument("--version", action="store_true")
    args = parser.parse_args(argv)
    if args.version:
        print(__version__)
        return
    settings = load_settings()
    if args.probe:
        raise SystemExit(asyncio.run(_probe(settings)))
    uvicorn.run(create_app(settings), host=settings.host, port=settings.port, log_level="info")


async def _probe(settings) -> int:
    cursor = CursorCliAdapter(model=settings.cursor_model)
    copilot = CopilotCliAdapter(model=settings.copilot_model)
    print("Cursor:", (await cursor.health()).detail)
    print("Copilot:", (await copilot.health()).detail)
    code = 0
    for adapter, workspace in ((cursor, settings.data_dir), (copilot, settings.data_dir)):
        health = await adapter.health()
        if not health.available:
            print(f"{adapter.name}: indisponible")
            code = 2
            continue
        request = AgentRequest(
            session_id="probe",
            agent=adapter.name,
            role="répondre sans outil",
            expected_types=frozenset({"RESPONSE"}),
            task="Réponds uniquement par le bloc JSON demandé. N'utilise aucun outil.",
            subject="sonde",
            history=[],
            turn=1,
            workspace=str(settings.data_dir),
            allow_write=False,
            timeout=90,
            resume_id=None,
            phase="respond",
        )
        result = await adapter.run(request, CancelToken())
        text = redact(result.text)[:500]
        error = redact(result.error)[:500]
        parsed = parse_payload(result.text)
        print(
            f"{adapter.name}: ok={result.ok} code={result.exit_code} "
            f"parsed={bool(parsed)} error={error}"
        )
        if text:
            print(f"{adapter.name} extrait: {text}")
        if not result.ok or not parsed:
            code = 1
    return code


if __name__ == "__main__":
    main(sys.argv[1:])
