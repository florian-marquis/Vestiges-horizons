"""Objets échangés entre l'orchestrateur et les adaptateurs."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field


@dataclass
class CancelToken:
    event: asyncio.Event = field(default_factory=asyncio.Event)
    reason: str | None = None

    def trigger(self, reason: str) -> None:
        if self.reason is None:
            self.reason = reason
        self.event.set()

    @property
    def cancelled(self) -> bool:
        return self.event.is_set()


@dataclass
class AgentRequest:
    session_id: str
    agent: str
    role: str
    expected_types: frozenset[str]
    task: str
    subject: str
    history: list[dict]
    turn: int
    workspace: str
    allow_write: bool
    timeout: float
    resume_id: str | None
    phase: str
    diff: str = ""
    scenario: str = "collaboration"
    attempt: int = 1
    incident_count: int = 0
    proposer: str = "cursor"


@dataclass
class AgentResult:
    ok: bool
    text: str = ""
    error: str = ""
    exit_code: int | None = None
    stderr: str = ""
    session_id: str | None = None
    timed_out: bool = False
    cancelled: bool = False
    unsupported: list[str] = field(default_factory=list)


@dataclass
class HealthStatus:
    agent: str
    available: bool
    authenticated: bool | None
    detail: str
    unsupported: list[str] = field(default_factory=list)
