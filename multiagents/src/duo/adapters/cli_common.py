"""Construction des commandes CLI et lecture de leurs sorties."""

from __future__ import annotations

import json
import os
import re
import shutil
from pathlib import Path


def resolve_copilot_command() -> list[str] | None:
    copilot = shutil.which("copilot")
    if not copilot:
        return None
    path = Path(copilot)
    if path.suffix.lower() == ".exe":
        return [str(path)]
    node = shutil.which("node")
    loader = path.resolve().parent / "node_modules" / "@github" / "copilot" / "npm-loader.js"
    if node and loader.exists():
        return [node, str(loader)]
    return None


def _cursor_version_key(name: str) -> tuple[int, int, int]:
    match = re.match(r"(\d{4})\.(\d{1,2})\.(\d{1,2})", name)
    if not match:
        return (0, 0, 0)
    return tuple(int(part) for part in match.groups())  # type: ignore[return-value]


def resolve_cursor_agent_command(root: Path | None = None) -> list[str] | None:
    """Retourne node.exe et index.js, sans passer par cmd.exe."""
    base = root
    if base is None:
        local = os.environ.get("LOCALAPPDATA")
        base = Path(local) / "cursor-agent" if local else None
    if base is not None:
        versions = base / "versions"
        if versions.is_dir():
            candidates = sorted(
                (path for path in versions.iterdir() if path.is_dir()),
                key=lambda path: _cursor_version_key(path.name),
                reverse=True,
            )
            for candidate in candidates:
                node = candidate / "node.exe"
                index = candidate / "index.js"
                if node.is_file() and index.is_file():
                    return [str(node), str(index)]
    agent = shutil.which("agent")
    if agent and Path(agent).suffix.lower() == ".exe":
        return [agent]
    return None


def _session_from(obj: dict) -> str | None:
    for key in ("sessionId", "session_id", "sessionID"):
        value = obj.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    session = obj.get("session")
    if isinstance(session, dict):
        for key in ("id", "sessionId", "session_id"):
            value = session.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
    return None


def _text_from(obj: dict) -> str:
    message = obj.get("message")
    if isinstance(message, dict):
        content = message.get("content")
        if isinstance(content, str) and content.strip():
            return content
        if isinstance(content, list):
            parts: list[str] = []
            for block in content:
                if isinstance(block, str):
                    parts.append(block)
                elif isinstance(block, dict) and isinstance(block.get("text"), str):
                    parts.append(block["text"])
            joined = "".join(parts).strip()
            if joined:
                return joined
    for key in ("result", "text", "content"):
        value = obj.get(key)
        if isinstance(value, str) and value.strip():
            return value
    if isinstance(message, str) and message.strip():
        return message
    return ""


def extract_agent_output(stdout: str) -> tuple[str, str | None]:
    texts: list[str] = []
    session_id = None
    parsed = False
    for line in stdout.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        try:
            obj = json.loads(stripped)
        except json.JSONDecodeError:
            continue
        if not isinstance(obj, dict):
            continue
        parsed = True
        session_id = session_id or _session_from(obj)
        text = _text_from(obj)
        if text:
            texts.append(text)
    if not parsed:
        try:
            obj = json.loads(stdout)
        except json.JSONDecodeError:
            return stdout.strip(), None
        if isinstance(obj, dict):
            return _text_from(obj) or stdout.strip(), _session_from(obj)
        return stdout.strip(), None
    if texts:
        return texts[-1], session_id
    return stdout.strip(), session_id
