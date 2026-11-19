"""Aide commune aux tests."""

from __future__ import annotations

import asyncio
from pathlib import Path

from duo.config import Settings
from duo.db import Database
from duo.gitspace import WorkspaceManager
from duo.orchestrator import Orchestrator


def make_stack(tmp_path: Path, **overrides):
    data = tmp_path / "data"
    settings = Settings(
        host="127.0.0.1",
        port=8765,
        data_dir=data,
        allowed_roots=(tmp_path,),
        **overrides,
    )
    data.mkdir(parents=True, exist_ok=True)
    db = Database(settings.db_path)
    workspaces = WorkspaceManager(settings.sandbox_root, settings.worktree_root, settings.roots())
    return settings, db, Orchestrator(db, workspaces)


def run(coro):
    return asyncio.run(coro)


def session_fields(**overrides):
    fields = {
        "task": "Ajouter une fonction add(a, b) couverte par un test.",
        "mode": "simulated",
        "scenario": "collaboration",
        "max_turns": 8,
        "max_duration_seconds": 600,
        "agent_timeout_seconds": 30,
        "similarity_threshold": 0.92,
        "max_corrections": 1,
        "proposer": "cursor",
        "critic": "copilot",
        "implementer": "cursor",
        "allow_implementation": True,
    }
    fields.update(overrides)
    return fields
