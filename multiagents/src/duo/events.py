"""Diffusion locale des mises à jour de session."""

from __future__ import annotations

import asyncio
from collections import defaultdict


class Broker:
    def __init__(self) -> None:
        self._subscribers: dict[str, list[asyncio.Queue]] = defaultdict(list)

    def publish(self, session_id: str, payload: dict) -> None:
        for queue in list(self._subscribers.get(session_id, ())):
            queue.put_nowait(payload)

    async def subscribe(self, session_id: str):
        queue: asyncio.Queue = asyncio.Queue()
        self._subscribers[session_id].append(queue)
        try:
            while True:
                payload = await queue.get()
                yield payload
        finally:
            self._subscribers[session_id].remove(queue)
