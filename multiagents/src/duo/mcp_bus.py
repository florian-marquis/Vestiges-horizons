"""Pont MCP : Cursor et VS Code appellent le salon sans connaître l'autre fenêtre."""

from __future__ import annotations

import json
import os
import sys
from urllib.parse import urlencode

from duo import __version__
from duo.wait_room import http_get, http_patch, http_post, wait_until_message

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
            "Bloque jusqu'au prochain message visible ou à la clôture du salon. "
            "Ne revient pas sur un délai vide : les polls sont enchaînés côté outil. "
            "Ne termine pas la collaboration."
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
                "timeout": {
                    "type": "integer",
                    "description": "Durée d'un poll HTTP en secondes (défaut 90, max 90). L'outil enchaîne les polls.",
                },
            },
            "required": ["room_id", "token"],
        },
    },
    {
        "name": "etat_quotas_modeles",
        "description": (
            "Récupère les modèles disponibles, quotas restants et recommandations de routage "
            "pour le Pilote (@pilote) et les agents (@agent1..N) selon la difficulté (N1..N4). "
            "Peut synchroniser les quotas Antigravity via antigravity_swarm."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "difficulty": {
                    "type": "string",
                    "description": "Niveau de difficulté de la tâche : N1, N2, N3 ou N4 (défaut N2).",
                },
                "pilote_provider": {
                    "type": "string",
                    "description": "Outil courant du Pilote : antigravity, copilot, vibe ou cursor.",
                },
                "antigravity_swarm": {
                    "type": "object",
                    "description": "Sortie JSON optionnelle de swarm(action='models', all=True) à synchroniser.",
                },
            },
        },
    },
    {
        "name": "changer_slot",
        "description": (
            "Change à chaud l'outil (provider/window) et/ou le modèle d'un rôle du salon, "
            "y compris le Pilote (@pilote) ou un agent (@agent1..@agent5), sans perdre le salon ni les jetons."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "room_id": {"type": "string"},
                "token": {"type": "string", "description": "Optionnel (conservé pour cohérence)."},
                "role": {"type": "string", "description": "Rôle ciblé : pilote ou agent1..agent5."},
                "provider": {"type": "string", "description": "antigravity, copilot, vibe ou cursor."},
                "window": {"type": "string", "description": "antigravity, vscode ou cursor."},
                "model": {"type": "string", "description": "Identifiant ou nom du modèle à brancher."},
                "reason": {"type": "string", "description": "Motif du switch (quota, difficulté N1..N4, escalade)."},
            },
            "required": ["room_id", "role"],
        },
    },
    {
        "name": "apprendre_modeles",
        "description": (
            "Enregistre durablement de nouveaux modèles signalés par l'humain pour un fournisseur "
            "(antigravity, copilot, vibe, cursor) afin qu'ils soient utilisables dans Duo et le routage."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "provider": {"type": "string", "description": "antigravity, copilot, vibe ou cursor."},
                "models": {
                    "type": "array",
                    "description": "Liste d'identifiants de modèles ou d'objets {id, label?, tier?, cost_hint?}.",
                    "items": {"type": ["string", "object"]},
                },
            },
            "required": ["provider", "models"],
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
                    "attendre bloque jusqu'au prochain message ou à la clôture. "
                    "Après dire, rappelle attendre. "
                    "Si l'IDE coupe l'outil, rappelle immédiatement attendre avec le même after."
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
    try:
        if name == "etat_quotas_modeles":
            swarm_payload = arguments.get("antigravity_swarm")
            if isinstance(swarm_payload, dict) and swarm_payload:
                http_post(f"{base}/api/providers/antigravity/sync", swarm_payload, timeout=20)
            query = urlencode(
                {
                    "difficulty": str(arguments.get("difficulty") or "N2").strip(),
                    "pilote_provider": str(arguments.get("pilote_provider") or "antigravity").strip(),
                }
            )
            payload = http_get(f"{base}/api/routing?{query}", timeout=30)
        elif name == "changer_slot":
            role = str(arguments.get("role") or "").strip().lower()
            if not room_id or not role:
                return _tool_error("room_id et role sont requis.")
            body = {"role": role}
            for field in ("provider", "window", "model", "reason"):
                val = arguments.get(field)
                if isinstance(val, str) and val.strip():
                    body[field] = val.strip()
            payload = http_patch(f"{base}/api/rooms/{room_id}/slots", body, timeout=30)
        elif name == "apprendre_modeles":
            provider = str(arguments.get("provider") or "").strip().lower()
            models = arguments.get("models")
            if provider not in {"antigravity", "copilot", "vibe", "cursor"}:
                return _tool_error("provider doit être antigravity, copilot, vibe ou cursor.")
            if not isinstance(models, list) or not models:
                return _tool_error("models doit être une liste non vide.")
            payload = http_post(
                f"{base}/api/providers/{provider}/sync",
                {"add_models": models},
                timeout=20,
            )
        elif name in {"lire_salon", "dire", "attendre"}:
            if not room_id or not token:
                return _tool_error("room_id et token sont requis.")
            if name == "lire_salon":
                payload = http_get(f"{base}/api/rooms/{room_id}", timeout=30)
            elif name == "dire":
                text = arguments.get("text")
                if not isinstance(text, str) or not text.strip():
                    return _tool_error("Le texte à publier est vide.")
                payload = http_post(
                    f"{base}/api/rooms/{room_id}/messages",
                    {"token": token, "text": text},
                    timeout=30,
                )
            else:
                after = int(arguments.get("after") or 0)
                timeout = int(arguments.get("timeout") or 90)
                payload = wait_until_message(
                    base,
                    room_id,
                    token,
                    after,
                    poll_timeout=timeout,
                )
        else:
            return _tool_error("Outil inconnu.")
    except RuntimeError as exc:
        return _tool_error(str(exc))
    return {
        "content": [{"type": "text", "text": json.dumps(payload, ensure_ascii=False, indent=2)}],
        "isError": False,
    }


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
