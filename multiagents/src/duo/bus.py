"""Salon local : Pilote + Agent 1..N se parlent via le bus."""

from __future__ import annotations

import asyncio
import json
import re
import secrets
import sqlite3
from datetime import datetime, timezone
from pathlib import Path
from threading import RLock

from duo.db import new_id, utcnow
from duo.security import redact

WINDOW_KINDS = frozenset({"vscode", "cursor", "antigravity"})
PROVIDER_KINDS = frozenset({"antigravity", "copilot", "vibe", "cursor"})
DEFAULT_WINDOWS_BY_PROVIDER = {
    "antigravity": "antigravity",
    "copilot": "vscode",
    "vibe": "vscode",
    "cursor": "cursor",
}
DEFAULT_MODELS_BY_PROVIDER = {
    "antigravity": "gemini-pro-orchestrator-g1-pro",
    "copilot": "gpt-4.1",
    "vibe": "devstral-medium-latest",
    "cursor": "auto",
}
MIN_AGENTS = 1
MAX_AGENTS = 5
CONNECTED_SECONDS = 70
MENTION_RE = re.compile(r"@(pilote|agent[1-5])\b", re.IGNORECASE)

LEGACY_SENDERS = {
    "cursor": "pilote",
    "master": "pilote",
    "vscode": "agent1",
    "partner": "agent1",
}


def base_url(host: str, port: int) -> str:
    shown = f"[{host}]" if ":" in host else host
    return f"http://{shown}:{port}"


def agent_role(index: int) -> str:
    return f"agent{index}"


def label_for(role: str) -> str:
    role = LEGACY_SENDERS.get(role, role)
    if role == "humain":
        return "Vous"
    if role == "pilote":
        return "Pilote"
    if role.startswith("agent") and role[5:].isdigit():
        return f"Agent {role[5:]}"
    return role


def roles_for_count(agents_count: int) -> list[str]:
    return ["pilote", *[agent_role(i) for i in range(1, agents_count + 1)]]


def normalize_claim(claimed: str | None) -> str | None:
    if not claimed:
        return None
    value = claimed.strip().lower()
    mapped = LEGACY_SENDERS.get(value, value)
    if mapped == "pilote" or (mapped.startswith("agent") and mapped[5:].isdigit()):
        return mapped
    raise PermissionError("Fenêtre inconnue.")


def mentions_in(text: str) -> set[str]:
    return {match.group(1).lower() for match in MENTION_RE.finditer(text or "")}


def message_visible_to(sender: str, text: str, viewer: str) -> bool:
    """Broadcast (pas de @) : tous sauf l'auteur. Mentions : destinataires + pilote (suivi)."""
    if sender == viewer:
        return False
    if sender == "humain":
        return True
    tagged = mentions_in(text)
    if not tagged:
        return True
    if viewer in tagged:
        return True
    # Le pilote voit les messages adressés pour piloter.
    if viewer == "pilote":
        return True
    return False


def _default_provider_for_slot(role: str, window: str) -> str:
    if window == "antigravity":
        return "antigravity"
    if window == "cursor":
        return "cursor"
    if role == "agent3":
        return "vibe"
    return "copilot"


def _default_pilote_slot() -> dict:
    return {
        "role": "pilote",
        "window": "antigravity",
        "provider": "antigravity",
        "model": DEFAULT_MODELS_BY_PROVIDER["antigravity"],
    }


def window_hint_for(role: str, window: str, provider: str | None = None) -> str:
    tool = provider or _default_provider_for_slot(role, window)
    if role == "pilote":
        return (
            f"Colle cette consigne dans la fenêtre Pilote ({window}, outil {tool}), en mode Agent."
        )
    if window == "cursor":
        return (
            "Ouvre une fenêtre Cursor dédiée sur le même dossier "
            "(Fichier > Nouvelle fenêtre si besoin), puis colle cette consigne."
        )
    if window == "antigravity":
        return (
            "Ouvre une session Google Antigravity dédiée sur le même dossier, "
            "puis colle cette consigne."
        )
    return f"Colle cette consigne dans l'agent de la fenêtre VS Code (outil {tool}), en mode Agent."


class Bus:
    def __init__(self, path: Path):
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._lock = RLock()
        self._conn = sqlite3.connect(self.path, check_same_thread=False)
        self._conn.row_factory = sqlite3.Row
        self._conn.execute("PRAGMA journal_mode = WAL")
        self._conn.executescript(
            """
            CREATE TABLE IF NOT EXISTS rooms (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                task TEXT NOT NULL,
                status TEXT NOT NULL,
                cursor_token TEXT NOT NULL DEFAULT '',
                vscode_token TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS room_messages (
                room_id TEXT NOT NULL,
                seq INTEGER NOT NULL,
                sender TEXT NOT NULL,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (room_id, seq)
            );
            CREATE TABLE IF NOT EXISTS presence (
                room_id TEXT NOT NULL,
                agent TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                PRIMARY KEY (room_id, agent)
            );
            CREATE TABLE IF NOT EXISTS room_tokens (
                room_id TEXT NOT NULL,
                role TEXT NOT NULL,
                token TEXT NOT NULL,
                PRIMARY KEY (room_id, role)
            );
            """
        )
        self._conn.commit()
        self._migrate()
        self._events: dict[str, asyncio.Event] = {}
        self._loop: asyncio.AbstractEventLoop | None = None

    def _migrate(self) -> None:
        with self._lock:
            columns = {
                row["name"]
                for row in self._conn.execute("PRAGMA table_info(rooms)").fetchall()
            }
            if "partner_kind" not in columns:
                self._conn.execute(
                    "ALTER TABLE rooms ADD COLUMN partner_kind TEXT NOT NULL DEFAULT 'vscode'"
                )
            if "agents_count" not in columns:
                self._conn.execute(
                    "ALTER TABLE rooms ADD COLUMN agents_count INTEGER NOT NULL DEFAULT 1"
                )
            if "slots_json" not in columns:
                self._conn.execute(
                    "ALTER TABLE rooms ADD COLUMN slots_json TEXT NOT NULL DEFAULT '[]'"
                )
            if "pilote_slot_json" not in columns:
                self._conn.execute(
                    "ALTER TABLE rooms ADD COLUMN pilote_slot_json TEXT NOT NULL DEFAULT '{}'"
                )
            self._conn.commit()
            # Remplir room_tokens / slots pour les anciens salons duo.
            rooms = self._conn.execute("SELECT * FROM rooms").fetchall()
            for room in rooms:
                existing = self._conn.execute(
                    "SELECT COUNT(*) AS n FROM room_tokens WHERE room_id = ?",
                    (room["id"],),
                ).fetchone()["n"]
                if existing:
                    continue
                slots = self._parse_slots(room["slots_json"], room)
                if not slots:
                    kind = room["partner_kind"] if "partner_kind" in room.keys() else "vscode"
                    slots = [{"role": "agent1", "window": kind or "vscode"}]
                    self._conn.execute(
                        "UPDATE rooms SET agents_count = 1, slots_json = ? WHERE id = ?",
                        (json.dumps(slots), room["id"]),
                    )
                pilote_token = room["cursor_token"] or secrets.token_urlsafe(18)
                self._conn.execute(
                    "INSERT OR IGNORE INTO room_tokens (room_id, role, token) VALUES (?, ?, ?)",
                    (room["id"], "pilote", pilote_token),
                )
                agent1_token = room["vscode_token"] or secrets.token_urlsafe(18)
                self._conn.execute(
                    "INSERT OR IGNORE INTO room_tokens (room_id, role, token) VALUES (?, ?, ?)",
                    (room["id"], "agent1", agent1_token),
                )
                for slot in slots[1:]:
                    self._conn.execute(
                        "INSERT OR IGNORE INTO room_tokens (room_id, role, token) VALUES (?, ?, ?)",
                        (room["id"], slot["role"], secrets.token_urlsafe(18)),
                    )
            self._conn.commit()

    def close(self) -> None:
        with self._lock:
            self._conn.close()

    def create_room(
        self,
        task: str,
        agents_count: int = 1,
        slots: list[dict] | None = None,
        agent1: str | None = None,
        pilote: dict | None = None,
    ) -> dict:
        count = int(agents_count)
        if not MIN_AGENTS <= count <= MAX_AGENTS:
            raise ValueError(f"Le nombre d'agents doit être entre {MIN_AGENTS} et {MAX_AGENTS}.")
        pilote_slot = self._normalize_pilote_slot(pilote)
        normalized = self._normalize_slots(count, slots, agent1)
        text = _clean_text(task)
        room_id = new_id()
        title = text.splitlines()[0][:80]
        now = utcnow()
        tokens = {"pilote": secrets.token_urlsafe(18)}
        for slot in normalized:
            tokens[slot["role"]] = secrets.token_urlsafe(18)
        first_window = normalized[0]["window"]
        with self._lock:
            self._conn.execute(
                """
                INSERT INTO rooms (
                    id, title, task, status, cursor_token, vscode_token, created_at,
                    partner_kind, agents_count, slots_json, pilote_slot_json
                )
                VALUES (?, ?, ?, 'open', ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    room_id,
                    title,
                    text,
                    tokens["pilote"],
                    tokens.get("agent1", ""),
                    now,
                    first_window,
                    count,
                    json.dumps(normalized),
                    json.dumps(pilote_slot),
                ),
            )
            for role, token in tokens.items():
                self._conn.execute(
                    "INSERT INTO room_tokens (room_id, role, token) VALUES (?, ?, ?)",
                    (room_id, role, token),
                )
            self._conn.execute(
                """
                INSERT INTO room_messages (room_id, seq, sender, text, created_at)
                VALUES (?, 1, 'humain', ?, ?)
                """,
                (room_id, text, now),
            )
            self._conn.commit()
        self._signal(room_id)
        return self.room(room_id, include_tokens=True)

    def list_rooms(self) -> list[dict]:
        with self._lock:
            rows = self._conn.execute(
                "SELECT * FROM rooms ORDER BY created_at DESC"
            ).fetchall()
        return [self._public(row) for row in rows]

    def room(self, room_id: str, include_tokens: bool = False) -> dict:
        with self._lock:
            row = self._conn.execute("SELECT * FROM rooms WHERE id = ?", (room_id,)).fetchone()
            if row is None:
                raise KeyError(room_id)
            messages = self._messages(room_id)
            slots = self._parse_slots(row["slots_json"], row)
            pilote_slot = self._parse_pilote_slot(
                row["pilote_slot_json"] if "pilote_slot_json" in row.keys() else "{}"
            )
            roles = roles_for_count(int(row["agents_count"] or 1))
            presence = self._presence(room_id, roles)
            token_rows = self._conn.execute(
                "SELECT role, token FROM room_tokens WHERE room_id = ?",
                (room_id,),
            ).fetchall()
        tokens = {item["role"]: item["token"] for item in token_rows}
        data = self._public(row)
        data["messages"] = messages
        data["presence"] = presence
        data["last_seq"] = messages[-1]["seq"] if messages else 0
        data["labels"] = {role: label_for(role) for role in roles}
        data["labels"]["humain"] = "Vous"
        data["pilote_slot"] = pilote_slot
        data["slots"] = slots
        data["all_slots"] = [pilote_slot, *slots]
        if include_tokens:
            data["tokens"] = tokens
            data["pilote_token"] = tokens.get("pilote", "")
            data["agent1_token"] = tokens.get("agent1", "")
            data["master_token"] = tokens.get("pilote", "")
            data["partner_token"] = tokens.get("agent1", "")
        return data

    def update_slot(
        self,
        room_id: str,
        role: str,
        *,
        window: str | None = None,
        provider: str | None = None,
        model: str | None = None,
        reason: str = "",
        host: str = "127.0.0.1",
        port: int = 8765,
    ) -> dict:
        target_role = str(role or "").strip().lower()
        with self._lock:
            row = self._conn.execute("SELECT * FROM rooms WHERE id = ?", (room_id,)).fetchone()
            if row is None:
                raise KeyError(room_id)
            if row["status"] != "open":
                raise ValueError("Ce salon est clos.")
            roles = roles_for_count(int(row["agents_count"] or 1))
            if target_role not in roles:
                raise ValueError(f"Rôle inconnu dans ce salon : {target_role}.")

            if target_role == "pilote":
                current = self._parse_pilote_slot(
                    row["pilote_slot_json"] if "pilote_slot_json" in row.keys() else "{}"
                )
            else:
                slots = self._parse_slots(row["slots_json"], row)
                current = next((s for s in slots if s.get("role") == target_role), None)
                if current is None:
                    raise ValueError(f"Slot introuvable pour {target_role}.")

            updated = self._merge_slot_update(
                current,
                role=target_role,
                window=window,
                provider=provider,
                model=model,
            )

            if target_role == "pilote":
                self._conn.execute(
                    "UPDATE rooms SET pilote_slot_json = ? WHERE id = ?",
                    (json.dumps(updated), room_id),
                )
            else:
                new_slots = [updated if s.get("role") == target_role else s for s in slots]
                first_window = new_slots[0]["window"] if new_slots else "vscode"
                self._conn.execute(
                    "UPDATE rooms SET slots_json = ?, partner_kind = ? WHERE id = ?",
                    (json.dumps(new_slots), first_window, room_id),
                )
            self._conn.commit()

        updated_room = self.room(room_id)
        last_seq = updated_room["last_seq"]
        consignes = self.consignes(room_id, host, port)
        consigne = consignes[target_role]
        handover = (
            f"Switch {label_for(target_role)} (@{target_role}) -> "
            f"outil={updated['provider']} (fenêtre={updated['window']}), "
            f"modèle={updated['model']}. "
            f"Lire etat.md et reprendre avec SEQ={last_seq} (même token conservé)."
        )
        return {
            "room": updated_room,
            "role": target_role,
            "slot": updated,
            "consigne": consigne,
            "last_seq": last_seq,
            "reason": reason.strip(),
            "handover": handover,
        }

    def consignes(self, room_id: str, host: str, port: int) -> dict[str, str]:
        full = self.room(room_id, include_tokens=True)
        url = base_url(host, port)
        tokens = full["tokens"]
        prompts = {}
        for role in full["roles"]:
            prompts[role] = _prompt(role, full, tokens[role], url)
        prompts["master"] = prompts["pilote"]
        if "agent1" in prompts:
            prompts["partner"] = prompts["agent1"]
        return prompts

    def post_humain(self, room_id: str, text: str) -> dict:
        return self._post(room_id, "humain", text)

    def post_token(self, room_id: str, token: str, text: str, claimed: str | None = None) -> dict:
        agent = self._agent_for_token(room_id, token, claimed)
        self._touch(room_id, agent)
        return self._post(room_id, agent, text)

    async def wait(
        self,
        room_id: str,
        token: str,
        after: int,
        timeout: int,
        claimed: str | None = None,
    ) -> dict:
        agent = self._agent_for_token(room_id, token, claimed)
        self._touch(room_id, agent)
        event = self._event_for(room_id)
        loop = asyncio.get_running_loop()
        deadline = loop.time() + timeout
        while True:
            found = self._newer(room_id, after, agent)
            last_seq = self._last_seq(room_id)
            if found:
                return {"messages": found, "last_seq": last_seq, "timed_out": False, "closed": False}
            if self._status(room_id) != "open":
                return {"messages": [], "last_seq": last_seq, "timed_out": False, "closed": True}
            remaining = deadline - loop.time()
            if remaining <= 0:
                return {"messages": [], "last_seq": last_seq, "timed_out": True, "closed": False}
            event.clear()
            found = self._newer(room_id, after, agent)
            if found:
                return {
                    "messages": found,
                    "last_seq": self._last_seq(room_id),
                    "timed_out": False,
                    "closed": False,
                }
            try:
                await asyncio.wait_for(event.wait(), remaining)
            except TimeoutError:
                found = self._newer(room_id, after, agent)
                return {
                    "messages": found,
                    "last_seq": self._last_seq(room_id),
                    "timed_out": not found,
                    "closed": False,
                }

    def close_room(self, room_id: str) -> dict:
        with self._lock:
            row = self._conn.execute("SELECT status FROM rooms WHERE id = ?", (room_id,)).fetchone()
            if row is None:
                raise KeyError(room_id)
            self._conn.execute("UPDATE rooms SET status = 'closed' WHERE id = ?", (room_id,))
            self._conn.commit()
        self._signal(room_id)
        return self.room(room_id)

    def _normalize_pilote_slot(self, pilote: dict | None) -> dict:
        if not pilote:
            return _default_pilote_slot()
        raw_provider = pilote.get("provider")
        raw_window = pilote.get("window")
        if raw_provider is not None and str(raw_provider).strip():
            provider = str(raw_provider).strip().lower()
            if provider not in PROVIDER_KINDS:
                raise ValueError(f"Outil Pilote inconnu : {provider}.")
        else:
            provider = None
        if raw_window is not None and str(raw_window).strip():
            window = str(raw_window).strip().lower()
            if window not in WINDOW_KINDS:
                raise ValueError("La fenêtre du Pilote doit être antigravity, vscode ou cursor.")
        else:
            window = DEFAULT_WINDOWS_BY_PROVIDER.get(provider or "antigravity", "antigravity")
        if provider is None:
            provider = _default_provider_for_slot("pilote", window)
        model = str(pilote.get("model") or "").strip() or DEFAULT_MODELS_BY_PROVIDER[provider]
        return {"role": "pilote", "window": window, "provider": provider, "model": model}

    def _merge_slot_update(
        self,
        current: dict,
        *,
        role: str,
        window: str | None = None,
        provider: str | None = None,
        model: str | None = None,
    ) -> dict:
        cur_provider = str(current.get("provider") or _default_provider_for_slot(role, current.get("window") or "vscode")).lower()
        cur_window = str(current.get("window") or DEFAULT_WINDOWS_BY_PROVIDER.get(cur_provider, "vscode")).lower()
        cur_model = str(current.get("model") or DEFAULT_MODELS_BY_PROVIDER.get(cur_provider, "auto"))

        next_provider = cur_provider
        if provider is not None and str(provider).strip():
            next_provider = str(provider).strip().lower()
            if next_provider not in PROVIDER_KINDS:
                raise ValueError(f"Outil inconnu : {next_provider}.")

        if window is not None and str(window).strip():
            next_window = str(window).strip().lower()
            if next_window not in WINDOW_KINDS:
                raise ValueError("La fenêtre doit être antigravity, vscode ou cursor.")
        elif provider is not None and next_provider != cur_provider:
            next_window = DEFAULT_WINDOWS_BY_PROVIDER[next_provider]
        else:
            next_window = cur_window

        if provider is None and window is not None and next_window != cur_window:
            next_provider = _default_provider_for_slot(role, next_window)

        if model is not None and str(model).strip():
            next_model = str(model).strip()
        elif next_provider != cur_provider:
            next_model = DEFAULT_MODELS_BY_PROVIDER[next_provider]
        else:
            next_model = cur_model

        return {
            "role": role,
            "window": next_window,
            "provider": next_provider,
            "model": next_model,
        }

    def _normalize_slots(
        self,
        agents_count: int,
        slots: list[dict] | None,
        agent1: str | None,
    ) -> list[dict]:
        if slots is None:
            window = (agent1 or "vscode").strip().lower()
            if window not in WINDOW_KINDS:
                raise ValueError("La fenêtre d'un agent doit être antigravity, vscode ou cursor.")
            result = []
            for i in range(1, agents_count + 1):
                role = agent_role(i)
                provider = _default_provider_for_slot(role, window)
                result.append(
                    {
                        "role": role,
                        "window": window,
                        "provider": provider,
                        "model": DEFAULT_MODELS_BY_PROVIDER[provider],
                    }
                )
            return result
        if len(slots) != agents_count:
            raise ValueError("Le nombre de slots doit égaler agents_count.")
        normalized = []
        for index, raw in enumerate(slots, start=1):
            expected = agent_role(index)
            role = str(raw.get("role") or expected).strip().lower()
            if role != expected:
                raise ValueError(f"Le rôle attendu est {expected}.")
            raw_provider = raw.get("provider")
            provider = None
            if raw_provider is not None and str(raw_provider).strip():
                provider = str(raw_provider).strip().lower()
                if provider not in PROVIDER_KINDS:
                    raise ValueError(f"Outil inconnu pour {role} : {provider}.")
            raw_window = raw.get("window")
            if raw_window is not None and str(raw_window).strip():
                window = str(raw_window).strip().lower()
                if window not in WINDOW_KINDS:
                    raise ValueError("La fenêtre d'un agent doit être antigravity, vscode ou cursor.")
            elif provider is not None:
                window = DEFAULT_WINDOWS_BY_PROVIDER[provider]
            else:
                window = "vscode"
            if provider is None:
                provider = _default_provider_for_slot(role, window)
            model = str(raw.get("model") or "").strip() or DEFAULT_MODELS_BY_PROVIDER[provider]
            normalized.append(
                {
                    "role": role,
                    "window": window,
                    "provider": provider,
                    "model": model,
                }
            )
        return normalized

    def _parse_pilote_slot(self, raw: str | None) -> dict:
        try:
            data = json.loads(raw or "{}")
        except json.JSONDecodeError:
            data = {}
        if not isinstance(data, dict) or not data:
            return _default_pilote_slot()
        window = str(data.get("window") or "antigravity").strip().lower()
        if window not in WINDOW_KINDS:
            window = "antigravity"
        provider = str(data.get("provider") or _default_provider_for_slot("pilote", window)).strip().lower()
        if provider not in PROVIDER_KINDS:
            provider = "antigravity"
        model = str(data.get("model") or DEFAULT_MODELS_BY_PROVIDER[provider]).strip()
        return {"role": "pilote", "window": window, "provider": provider, "model": model}

    def _parse_slots(self, raw: str | None, row: sqlite3.Row | None = None) -> list[dict]:
        try:
            data = json.loads(raw or "[]")
        except json.JSONDecodeError:
            data = []
        if data:
            parsed: list[dict] = []
            for index, item in enumerate(data, start=1):
                if not isinstance(item, dict):
                    continue
                role = str(item.get("role") or agent_role(index)).strip().lower()
                window = str(item.get("window") or "vscode").strip().lower()
                provider = str(item.get("provider") or _default_provider_for_slot(role, window)).strip().lower()
                model = str(item.get("model") or DEFAULT_MODELS_BY_PROVIDER.get(provider, "auto")).strip()
                parsed.append({"role": role, "window": window, "provider": provider, "model": model})
            if parsed:
                return parsed
        if row is not None:
            kind = "vscode"
            if "partner_kind" in row.keys() and row["partner_kind"]:
                kind = row["partner_kind"]
            provider = _default_provider_for_slot("agent1", kind)
            return [
                {
                    "role": "agent1",
                    "window": kind,
                    "provider": provider,
                    "model": DEFAULT_MODELS_BY_PROVIDER[provider],
                }
            ]
        return []

    def _post(self, room_id: str, sender: str, text: str) -> dict:
        cleaned = _clean_text(text)
        now = utcnow()
        with self._lock:
            row = self._conn.execute("SELECT status FROM rooms WHERE id = ?", (room_id,)).fetchone()
            if row is None:
                raise KeyError(room_id)
            if row["status"] != "open":
                raise ValueError("Ce salon est clos.")
            seq = self._conn.execute(
                "SELECT COALESCE(MAX(seq), 0) + 1 AS seq FROM room_messages WHERE room_id = ?",
                (room_id,),
            ).fetchone()["seq"]
            self._conn.execute(
                """
                INSERT INTO room_messages (room_id, seq, sender, text, created_at)
                VALUES (?, ?, ?, ?, ?)
                """,
                (room_id, seq, sender, cleaned, now),
            )
            self._conn.commit()
        self._signal(room_id)
        return {"seq": seq, "sender": sender, "text": cleaned, "created_at": now}

    def _agent_for_token(self, room_id: str, token: str, claimed: str | None) -> str:
        with self._lock:
            row = self._conn.execute(
                "SELECT role FROM room_tokens WHERE room_id = ? AND token = ?",
                (room_id, token),
            ).fetchone()
            if row is None:
                # Compat anciens jetons stockés seulement sur rooms.
                legacy = self._conn.execute(
                    "SELECT cursor_token, vscode_token FROM rooms WHERE id = ?",
                    (room_id,),
                ).fetchone()
                if legacy is None:
                    raise KeyError(room_id)
                if token and token == legacy["cursor_token"]:
                    agent = "pilote"
                elif token and token == legacy["vscode_token"]:
                    agent = "agent1"
                else:
                    raise PermissionError("Jeton de fenêtre refusé.")
            else:
                agent = row["role"]
        claim = normalize_claim(claimed)
        if claim and claim != agent:
            raise PermissionError("Ce jeton appartient à l'autre fenêtre.")
        return agent

    def _touch(self, room_id: str, agent: str) -> None:
        now = utcnow()
        with self._lock:
            self._conn.execute(
                """
                INSERT INTO presence (room_id, agent, last_seen) VALUES (?, ?, ?)
                ON CONFLICT(room_id, agent) DO UPDATE SET last_seen = excluded.last_seen
                """,
                (room_id, agent, now),
            )
            self._conn.commit()

    def _event_for(self, room_id: str) -> asyncio.Event:
        loop = asyncio.get_running_loop()
        with self._lock:
            self._loop = loop
            event = self._events.get(room_id)
            if event is None:
                event = asyncio.Event()
                self._events[room_id] = event
            return event

    def _signal(self, room_id: str) -> None:
        with self._lock:
            event = self._events.get(room_id)
            loop = self._loop
        if event is None or loop is None or loop.is_closed():
            return
        try:
            running = asyncio.get_running_loop()
        except RuntimeError:
            running = None
        if running is loop:
            event.set()
        else:
            loop.call_soon_threadsafe(event.set)

    def _newer(self, room_id: str, after: int, agent: str) -> list[dict]:
        with self._lock:
            rows = self._conn.execute(
                """
                SELECT seq, sender, text, created_at FROM room_messages
                WHERE room_id = ? AND seq > ?
                ORDER BY seq
                """,
                (room_id, after),
            ).fetchall()
        messages = []
        for row in rows:
            item = _normalize_message(row)
            if message_visible_to(item["sender"], item["text"], agent):
                messages.append(item)
        return messages

    def _messages(self, room_id: str) -> list[dict]:
        rows = self._conn.execute(
            """
            SELECT seq, sender, text, created_at FROM room_messages
            WHERE room_id = ? ORDER BY seq
            """,
            (room_id,),
        ).fetchall()
        return [_normalize_message(row) for row in rows]

    def _presence(self, room_id: str, roles: list[str]) -> dict:
        rows = self._conn.execute(
            "SELECT agent, last_seen FROM presence WHERE room_id = ?",
            (room_id,),
        ).fetchall()
        found = {row["agent"]: row["last_seen"] for row in rows}
        for legacy, modern in LEGACY_SENDERS.items():
            if modern not in found and legacy in found:
                found[modern] = found[legacy]
        now = datetime.now(timezone.utc)
        presence = {}
        for agent in roles:
            seen = found.get(agent)
            connected = False
            if seen:
                moment = datetime.fromisoformat(seen)
                connected = (now - moment).total_seconds() < CONNECTED_SECONDS
            presence[agent] = {
                "last_seen": seen,
                "connected": connected,
                "label": label_for(agent),
            }
        return presence

    def _last_seq(self, room_id: str) -> int:
        with self._lock:
            row = self._conn.execute(
                "SELECT COALESCE(MAX(seq), 0) AS seq FROM room_messages WHERE room_id = ?",
                (room_id,),
            ).fetchone()
        return int(row["seq"])

    def _status(self, room_id: str) -> str:
        with self._lock:
            row = self._conn.execute("SELECT status FROM rooms WHERE id = ?", (room_id,)).fetchone()
        if row is None:
            raise KeyError(room_id)
        return row["status"]

    def _public(self, row: sqlite3.Row) -> dict:
        count = int(row["agents_count"] or 1) if "agents_count" in row.keys() else 1
        slots = self._parse_slots(row["slots_json"] if "slots_json" in row.keys() else "[]", row)
        pilote_slot = self._parse_pilote_slot(
            row["pilote_slot_json"] if "pilote_slot_json" in row.keys() else "{}"
        )
        kind = slots[0]["window"] if slots else (
            row["partner_kind"] if "partner_kind" in row.keys() else "vscode"
        )
        return {
            "id": row["id"],
            "title": row["title"],
            "task": row["task"],
            "status": row["status"],
            "created_at": row["created_at"],
            "agents_count": count,
            "pilote_slot": pilote_slot,
            "slots": slots,
            "all_slots": [pilote_slot, *slots],
            "agent1_kind": kind,
            "partner_kind": kind,
            "roles": roles_for_count(count),
        }


def _clean_text(text: str) -> str:
    if not isinstance(text, str):
        raise ValueError("Le message doit être du texte.")
    cleaned = redact(text).strip()
    if not cleaned:
        raise ValueError("Message vide.")
    if len(cleaned) > 20000:
        raise ValueError("Message trop long.")
    return cleaned


def _message(row: sqlite3.Row) -> dict:
    return {
        "seq": row["seq"],
        "sender": row["sender"],
        "text": row["text"],
        "created_at": row["created_at"],
    }


def _normalize_message(row: sqlite3.Row) -> dict:
    data = _message(row)
    data["sender"] = LEGACY_SENDERS.get(data["sender"], data["sender"])
    return data


def _slot_for(room: dict, role: str) -> dict:
    if role == "pilote":
        slot = room.get("pilote_slot")
        if isinstance(slot, dict) and slot:
            return slot
        return _default_pilote_slot()
    for slot in room.get("slots") or []:
        if slot.get("role") == role:
            return slot
    kind = room.get("agent1_kind") or "vscode"
    provider = _default_provider_for_slot(role, kind)
    return {
        "role": role,
        "window": kind,
        "provider": provider,
        "model": DEFAULT_MODELS_BY_PROVIDER[provider],
    }


def _slot_summary_lines(room: dict) -> str:
    all_slots = room.get("all_slots") or [_slot_for(room, "pilote"), *(room.get("slots") or [])]
    lines = []
    for slot in all_slots:
        role = slot.get("role") or "agent1"
        provider = slot.get("provider") or _default_provider_for_slot(role, slot.get("window") or "vscode")
        window = slot.get("window") or "vscode"
        model = slot.get("model") or DEFAULT_MODELS_BY_PROVIDER.get(provider, "auto")
        lines.append(f"- @{role} : outil={provider} (fenêtre={window}), modèle={model}")
    return "\n".join(lines)


def _prompt(who: str, room: dict, token: str, url: str) -> str:
    label = label_for(who)
    roles = room["roles"]
    others = [label_for(role) for role in roles if role != who]
    others_txt = ", ".join(others)
    room_id = room["id"]
    scratch = f"duo-message-{who}.txt"
    slot = _slot_for(room, who)
    window = str(slot.get("window") or ("antigravity" if who == "pilote" else "vscode"))
    provider = str(slot.get("provider") or _default_provider_for_slot(who, window))
    model = str(slot.get("model") or DEFAULT_MODELS_BY_PROVIDER.get(provider, "auto"))
    config_line = f"Configuration de ce rôle : Outil = {provider} (fenêtre {window}) | Modèle à sélectionner = {model}"
    agents_list = ", ".join(f"@{role}" for role in roles)
    last_seq = int(room.get("last_seq") or 0)

    if who == "pilote":
        slots_summary = _slot_summary_lines(room)
        reprise_note = (
            f"Reprise Pilote en cours de salon (last_seq={last_seq}) : lis etat.md avant d'agir et reprends avec SEQ={last_seq}."
            if last_seq > 1
            else "Tu parles en premier."
        )
        opening = (
            f"Tu es le Pilote. {reprise_note} Découpe le travail en micro-tâches. "
            "Adresse chaque message avec @agentN (ex. @agent1). "
            "Un seul rédacteur de code à la fois : indique-le dans etat.md (champ Rédacteur). "
            "Ne republie pas tout l'historique : envoie seulement le delta utile.\n"
            "Routage modèles & quotas : utilise l'outil MCP etat_quotas_modeles (ou GET /api/routing?difficulty=N1..N4) "
            "et changer_slot (ou PATCH /api/rooms/{room_id}/slots) pour adapter l'outil et le modèle de @pilote ou de @agentN "
            "selon la difficulté et les quotas. Indique toujours à l'utilisateur quel outil et quel modèle brancher.\n"
            f"Affectation actuelle des slots :\n{slots_summary}"
        )
        etat_rule = (
            "Seul le Pilote est autorisé à modifier etat.md (écrivain unique). "
            "Mets à jour etat.md avant d'assigner une tâche et après chaque compte-rendu d'agent."
        )
        step4 = f"4. Mets à jour etat.md, écris ton message dans {scratch}, puis publie (préfixe @destinataire si besoin)."
        initial_seq = str(last_seq) if last_seq > 1 else "0"
        mcp_extra = " (ainsi que etat_quotas_modeles et changer_slot pour le routage)"
    else:
        opening = (
            f"Tu es {label}. Ne publie rien avant un message qui te concerne "
            f"(broadcast sans @, ou @{who}, ou message de l'humain). "
            "Entre tout de suite dans la boucle avec SEQ égal à 1."
        )
        etat_rule = (
            "Lis etat.md à chaque tour en LECTURE SEULE "
            "(interdiction de modifier etat.md : seul le Pilote l'édite). "
            "N'écris du code que si tu es désigné dans # Rédacteur actuel."
        )
        step4 = f"4. Écris ta réponse dans {scratch} et publie (préfixe @pilote si besoin), sans modifier etat.md."
        initial_seq = "1"
        mcp_extra = ""

    return f"""Tu es {label}. Les autres rôles : {others_txt}. Vous ne voyez pas le chat IDE des autres. Vous collaborez uniquement par le salon Duo, sur le dossier déjà ouvert.

{window_hint_for(who, window, provider=provider)}
{config_line}

Tâche :
{room["task"]}

Rôles du salon : {agents_list}

État partagé (obligatoire) : fichier etat.md à la racine du dossier. Structure :
# Objectif
# Décisions
# TODO
# Rédacteur actuel
# Destinataire suivant
# Configuration active (Pilote & Agents)
{etat_rule}

{opening}

Protocole d'adressage :
- Sans @ : message visible par tous (sauf l'auteur).
- Avec @pilote / @agent1 / … : message destiné à ce rôle (le Pilote le voit aussi pour suivre).
- Réponds seulement aux messages qui te concernent.

Boucle, et ne termine pas ton tour tant que l'humain n'a pas écrit STOP :
1. Attends un message (MCP attendre, ou python -m duo.wait_room). SEQ vaut {initial_seq} au départ, puis last_seq. L'attente ne revient que sur un message ou la clôture.
2. Si l'IDE coupe l'outil d'attente, rappelle immédiatement attendre / wait_room avec le même SEQ, sans réfléchir.
3. Quand un message arrive pour toi : lis etat.md, fais le travail demandé, réponds au fond.
{step4}
5. Reprends à l'étape 1 avec le nouveau last_seq.

Si l'humain écrit STOP, publie un bilan court, puis arrête.

Lecture du salon :
python -c "import urllib.request; print(urllib.request.urlopen('{url}/api/rooms/{room_id}', timeout=30).read().decode())"

Attente, en remplaçant SEQ :
python -m duo.wait_room --base {url} {room_id} {token} SEQ

Publication, après {scratch} :
python -c "import json,urllib.request,pathlib; text=pathlib.Path('{scratch}').read_text(encoding='utf-8'); body=json.dumps({{'token':'{token}','text':text}}).encode(); req=urllib.request.Request('{url}/api/rooms/{room_id}/messages', data=body, headers={{'Content-Type':'application/json','X-Duo-Request':'1'}}, method='POST'); print(urllib.request.urlopen(req, timeout=30).read().decode())"

Si les outils MCP lire_salon, dire et attendre répondent, utilise-les{mcp_extra}. room_id = {room_id} et token = {token}. Après chaque dire, rappelle attendre.
"""
