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
from urllib.parse import quote

from duo.db import new_id, utcnow
from duo.security import redact

WINDOW_KINDS = frozenset({"vscode", "cursor"})
MIN_AGENTS = 1
MAX_AGENTS = 5
CONNECTED_SECONDS = 70
DEFAULT_WAIT_TIMEOUT = 90
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


def window_hint_for(role: str, window: str) -> str:
    if role == "pilote":
        return "Colle cette consigne dans l'agent de ta fenêtre (Pilote), en mode Agent."
    if window == "cursor":
        return (
            "Ouvre une fenêtre Cursor dédiée sur le même dossier "
            "(Fichier > Nouvelle fenêtre si besoin), puis colle cette consigne."
        )
    return "Colle cette consigne dans l'agent de la fenêtre VS Code, en mode Agent."


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
    ) -> dict:
        count = int(agents_count)
        if not MIN_AGENTS <= count <= MAX_AGENTS:
            raise ValueError(f"Le nombre d'agents doit être entre {MIN_AGENTS} et {MAX_AGENTS}.")
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
                    partner_kind, agents_count, slots_json
                )
                VALUES (?, ?, ?, 'open', ?, ?, ?, ?, ?, ?)
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
        data["slots"] = slots
        if include_tokens:
            data["tokens"] = tokens
            data["pilote_token"] = tokens.get("pilote", "")
            data["agent1_token"] = tokens.get("agent1", "")
            data["master_token"] = tokens.get("pilote", "")
            data["partner_token"] = tokens.get("agent1", "")
        return data

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

    def _normalize_slots(
        self,
        agents_count: int,
        slots: list[dict] | None,
        agent1: str | None,
    ) -> list[dict]:
        if slots is None:
            window = (agent1 or "vscode").strip().lower()
            if window not in WINDOW_KINDS:
                raise ValueError("La fenêtre d'un agent doit être vscode ou cursor.")
            return [{"role": agent_role(i), "window": window} for i in range(1, agents_count + 1)]
        if len(slots) != agents_count:
            raise ValueError("Le nombre de slots doit égaler agents_count.")
        normalized = []
        for index, raw in enumerate(slots, start=1):
            window = str(raw.get("window") or "vscode").strip().lower()
            if window not in WINDOW_KINDS:
                raise ValueError("La fenêtre d'un agent doit être vscode ou cursor.")
            role = str(raw.get("role") or agent_role(index)).strip().lower()
            if role != agent_role(index):
                raise ValueError(f"Le rôle attendu est {agent_role(index)}.")
            normalized.append({"role": role, "window": window})
        return normalized

    def _parse_slots(self, raw: str | None, row: sqlite3.Row | None = None) -> list[dict]:
        try:
            data = json.loads(raw or "[]")
        except json.JSONDecodeError:
            data = []
        if data:
            return data
        if row is not None:
            kind = "vscode"
            if "partner_kind" in row.keys() and row["partner_kind"]:
                kind = row["partner_kind"]
            return [{"role": "agent1", "window": kind}]
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
            "slots": slots,
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


def _slot_window(room: dict, role: str) -> str:
    for slot in room.get("slots") or []:
        if slot.get("role") == role:
            return slot.get("window") or "vscode"
    return room.get("agent1_kind") or "vscode"


def _prompt(who: str, room: dict, token: str, url: str) -> str:
    label = label_for(who)
    roles = room["roles"]
    others = [label_for(role) for role in roles if role != who]
    others_txt = ", ".join(others)
    room_id = room["id"]
    quoted = quote(token, safe="")
    scratch = f"duo-message-{who}.txt"
    window = _slot_window(room, who) if who != "pilote" else "cursor"
    agents_list = ", ".join(f"@{role}" for role in roles)

    if who == "pilote":
        opening = (
            "Tu es le Pilote. Tu parles en premier. Découpe le travail en micro-tâches. "
            "Adresse chaque message avec @agentN (ex. @agent1). "
            "Un seul rédacteur de code à la fois : indique-le dans etat.md (champ Rédacteur). "
            "Ne republie pas tout l'historique : envoie seulement le delta utile."
        )
        initial_seq = "0"
    else:
        opening = (
            f"Tu es {label}. Ne publie rien avant un message qui te concerne "
            f"(broadcast sans @, ou @{who}, ou message de l'humain). "
            "Entre tout de suite dans la boucle avec SEQ égal à 1."
        )
        initial_seq = "1"

    return f"""Tu es {label}. Les autres rôles : {others_txt}. Vous ne voyez pas le chat IDE des autres. Vous collaborez uniquement par le salon Duo, sur le dossier déjà ouvert.

{window_hint_for(who, window)}

Tâche :
{room["task"]}

Rôles du salon : {agents_list}

État partagé (obligatoire) : fichier etat.md à la racine du dossier. Structure :
# Objectif
# Décisions
# TODO
# Rédacteur actuel
# Destinataire suivant
Lis et mets à jour etat.md à chaque tour utile. N'écris du code que si tu es le rédacteur actuel.

{opening}

Protocole d'adressage :
- Sans @ : message visible par tous (sauf l'auteur).
- Avec @pilote / @agent1 / … : message destiné à ce rôle (le Pilote le voit aussi pour suivre).
- Réponds seulement aux messages qui te concernent.

Boucle, et ne termine pas ton tour tant que l'humain n'a pas écrit STOP :
1. Attends avec timeout={DEFAULT_WAIT_TIMEOUT}. SEQ vaut {initial_seq} au départ, puis last_seq.
2. Si timed_out est vrai et messages vide, rappelle immédiatement l'attente SANS réfléchir. Un délai n'est pas une fin et ne consomme presque pas de tokens.
3. Quand un message arrive pour toi : lis etat.md, fais le travail demandé, réponds au fond.
4. Écris ta réponse dans {scratch}, publie (préfixe @destinataire si besoin), mets à jour etat.md.
5. Reprends à l'étape 1 avec le nouveau last_seq.

Si l'humain écrit STOP, publie un bilan court, puis arrête.

Lecture du salon :
python -c "import urllib.request; print(urllib.request.urlopen('{url}/api/rooms/{room_id}', timeout=30).read().decode())"

Attente, en remplaçant SEQ :
python -c "import urllib.request; print(urllib.request.urlopen('{url}/api/rooms/{room_id}/wait?token={quoted}&after=SEQ&timeout={DEFAULT_WAIT_TIMEOUT}', timeout={DEFAULT_WAIT_TIMEOUT + 15}).read().decode())"

Publication, après {scratch} :
python -c "import json,urllib.request,pathlib; text=pathlib.Path('{scratch}').read_text(encoding='utf-8'); body=json.dumps({{'token':'{token}','text':text}}).encode(); req=urllib.request.Request('{url}/api/rooms/{room_id}/messages', data=body, headers={{'Content-Type':'application/json','X-Duo-Request':'1'}}, method='POST'); print(urllib.request.urlopen(req, timeout=30).read().decode())"

Si les outils MCP lire_salon, dire et attendre répondent, utilise-les. room_id = {room_id} et token = {token}. Après chaque dire, rappelle attendre.
"""
