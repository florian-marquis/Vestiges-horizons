"""Pont MCP : Cursor et VS Code appellent le salon sans connaître l'autre fenêtre."""

from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request

from duo import __version__

PROTOCOL = "2024-11-05"
TOOLS = (
    {
        "name": "lire_salon",
        "description": "Lit la tâche et tous les messages du salon Duo.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "room_id": {"type": "string"},
                "token": {"type": "string", "description": "Jeton indiqué dans la consigne de cette fenêtre."},
            },
            "required": ["room_id", "token"],
        },
    },
    {
        "name": "dire",
        "description": "Publie un message à destination de l'agent de l'autre fenêtre.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "room_id": {"type": "string"},
                "token": {"type": "string"},
                "text": {"type": "string"},
            },
            "required": ["room_id", "token", "text"],
        },
    },
    {
        "name": "attendre",
        "description": (
            "Bloque jusqu'à un message de l'autre agent ou de l'humain. "
            "Si le délai expire sans message, rappelle cet outil. Ne termine pas la collaboration."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "room_id": {"type": "string"},
                "token": {"type": "string"},
                "after": {
                    "type": "integer",
                    "description": "Dernier seq déjà traité. 0 pour le Pilote, 1 pour un agent.",
                },
                "timeout": {"type": "integer", "description": "Secondes, 90 par défaut, 90 maximum."},
            },
            "required": ["room_id", "token"],
        },
    },
)


def main() -> None:
    base = os.environ.get("DUO_BUS_URL", "http://127.0.0.1:8765").rstrip("/")
    while True:
        message = _read()
        if message is None:
            return
        response = _handle(message, base)
        if response is not None:
            _write(response)


def _handle(message: dict, base: str) -> dict | None:
    method = message.get("method")
    has_id = "id" in message
    if not has_id:
        return None
    request_id = message.get("id")
    try:
        if method == "initialize":
            result = {
                "protocolVersion": PROTOCOL,
                "capabilities": {"tools": {"listChanged": False}},
                "serverInfo": {"name": "duo-bus", "version": __version__},
                "instructions": (
                    "Tu es relié au salon Duo. L'identité vient du token de ta consigne. "
                    "lire_salon lit les messages, dire publie le tien, "
                    "attendre bloque jusqu'au prochain message. "
                    "Après dire, rappelle attendre. Un délai vide veut dire de rappeler attendre."
                ),
            }
        elif method == "ping":
            result = {}
        elif method == "tools/list":
            result = {"tools": list(TOOLS)}
        elif method == "tools/call":
            result = _call_tool(message.get("params") or {}, base)
        else:
            return _error(request_id, -32601, "Méthode inconnue.")
    except Exception as exc:
        return _error(request_id, -32603, str(exc))
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def _call_tool(params: dict, base: str) -> dict:
    name = params.get("name")
    arguments = params.get("arguments") or {}
    room_id = str(arguments.get("room_id") or "").strip()
    token = str(arguments.get("token") or "").strip()
    if not room_id or not token:
        return _tool_error("room_id et token sont requis.")
    try:
        if name == "lire_salon":
            payload = _get(f"{base}/api/rooms/{room_id}", timeout=30)
        elif name == "dire":
            text = arguments.get("text")
            if not isinstance(text, str) or not text.strip():
                return _tool_error("Le texte à publier est vide.")
            payload = _send(
                f"{base}/api/rooms/{room_id}/messages",
                {"token": token, "text": text},
                timeout=30,
            )
        elif name == "attendre":
            after = int(arguments.get("after") or 0)
            timeout = int(arguments.get("timeout") or 90)
            timeout = max(1, min(timeout, 90))
            payload = _get(
                f"{base}/api/rooms/{room_id}/wait?token={_quote(token)}&after={after}&timeout={timeout}",
                timeout=timeout + 15,
            )
        else:
            return _tool_error("Outil inconnu.")
    except RuntimeError as exc:
        return _tool_error(str(exc))
    return {
        "content": [{"type": "text", "text": json.dumps(payload, ensure_ascii=False, indent=2)}],
        "isError": False,
    }


def _quote(token: str) -> str:
    from urllib.parse import quote

    return quote(token, safe="")


def _get(url: str, timeout: int) -> dict:
    request = urllib.request.Request(url, method="GET")
    return _read_response(request, timeout)


def _send(url: str, payload: dict, timeout: int) -> dict:
    body = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=body,
        headers={
            "Content-Type": "application/json",
            "X-Duo-Request": "1",
        },
        method="POST",
    )
    return _read_response(request, timeout)


def _read_response(request: urllib.request.Request, timeout: int) -> dict:
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(detail or f"Le salon a répondu {exc.code}.") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(
            "Le salon Duo ne répond pas. Lancez start.bat, puis réessayez."
        ) from exc


def _tool_error(text: str) -> dict:
    return {"content": [{"type": "text", "text": text}], "isError": True}


def _error(request_id, code: int, message: str) -> dict:
    return {"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}}


def _read() -> dict | None:
    line = sys.stdin.buffer.readline()
    if not line:
        return None
    if not line.strip():
        return _read()
    if line.lower().startswith(b"content-length:"):
        length = int(line.split(b":", 1)[1].strip())
        while True:
            extra = sys.stdin.buffer.readline()
            if extra in (b"\r\n", b"\n", b""):
                break
        body = sys.stdin.buffer.read(length)
        return json.loads(body.decode("utf-8"))
    return json.loads(line.decode("utf-8"))


def _write(message: dict) -> None:
    raw = json.dumps(message, ensure_ascii=False).encode("utf-8") + b"\n"
    sys.stdout.buffer.write(raw)
    sys.stdout.buffer.flush()


if __name__ == "__main__":
    main()
