"""Interface commune des agents."""

from __future__ import annotations

from abc import ABC, abstractmethod

from duo.models import AgentRequest, AgentResult, CancelToken, HealthStatus


class AgentAdapter(ABC):
    name: str

    @abstractmethod
    async def health(self) -> HealthStatus:
        raise NotImplementedError

    @abstractmethod
    async def run(self, request: AgentRequest, token: CancelToken) -> AgentResult:
        raise NotImplementedError
