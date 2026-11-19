"""Supervision des processus enfants."""

from __future__ import annotations

import asyncio
import os
import subprocess
import time
from threading import Lock


def windows_flags() -> int:
    if os.name != "nt":
        return 0
    return subprocess.CREATE_NEW_PROCESS_GROUP | subprocess.CREATE_NO_WINDOW


class ProcessRegistry:
    def __init__(self) -> None:
        self._lock = Lock()
        self._pids: dict[str, set[int]] = {}

    def add(self, session_id: str, pid: int) -> None:
        with self._lock:
            self._pids.setdefault(session_id, set()).add(pid)

    def discard(self, session_id: str, pid: int) -> None:
        with self._lock:
            group = self._pids.get(session_id)
            if not group:
                return
            group.discard(pid)
            if not group:
                self._pids.pop(session_id, None)

    def kill_pid(self, pid: int) -> None:
        if os.name == "nt":
            subprocess.run(
                ["taskkill", "/PID", str(int(pid)), "/T", "/F"],
                capture_output=True,
                text=True,
                check=False,
            )
            return
        try:
            os.kill(pid, 15)
        except OSError:
            return

    def kill_session(self, session_id: str) -> None:
        with self._lock:
            pids = list(self._pids.get(session_id, ()))
        for pid in pids:
            self.kill_pid(pid)

    def kill_all(self) -> None:
        with self._lock:
            pids = [pid for group in self._pids.values() for pid in group]
            self._pids.clear()
        for pid in pids:
            self.kill_pid(pid)


async def run_command(
    args: list[str],
    cwd: str,
    timeout: float,
    token,
    registry: ProcessRegistry,
    session_id: str,
) -> tuple[int | None, str, str, str]:
    """Exécute une commande avec arguments séparés.

    Retourne (code, stdout, stderr, fin) où fin vaut ok, timeout ou cancelled.
    """
    if not args or any(not isinstance(part, str) or part == "" for part in args):
        raise ValueError("Arguments de processus invalides.")
    process = await asyncio.create_subprocess_exec(
        *args,
        cwd=cwd,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
        creationflags=windows_flags(),
    )
    registry.add(session_id, process.pid)
    reason = "ok"
    communicate = asyncio.create_task(process.communicate())
    deadline = time.monotonic() + timeout
    try:
        while not communicate.done():
            if token.cancelled:
                reason = "cancelled" if token.reason == "stop" else "timeout"
                registry.kill_pid(process.pid)
                break
            if time.monotonic() >= deadline:
                reason = "timeout"
                token.trigger("timeout")
                registry.kill_pid(process.pid)
                break
            await asyncio.sleep(0.05)
        try:
            stdout, stderr = await asyncio.wait_for(communicate, timeout=5)
        except TimeoutError:
            stdout, stderr = b"", b""
    finally:
        registry.discard(session_id, process.pid)
    out = (stdout or b"").decode("utf-8", errors="replace")
    err = (stderr or b"").decode("utf-8", errors="replace")
    return process.returncode, out, err, reason
