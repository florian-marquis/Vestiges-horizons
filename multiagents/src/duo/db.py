"""Persistance SQLite, migrations et transactions."""

from __future__ import annotations

import json
import sqlite3
import uuid
from datetime import datetime, timezone
from pathlib import Path
from threading import RLock

MIGRATIONS: list[tuple[int, str]] = [
    (
        1,
        """
        CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            task_text TEXT NOT NULL,
            status TEXT NOT NULL,
            phase TEXT NOT NULL,
            next_action TEXT NOT NULL,
            turn INTEGER NOT NULL DEFAULT 0,
            max_turns INTEGER NOT NULL,
            max_duration_seconds INTEGER NOT NULL,
            agent_timeout_seconds INTEGER NOT NULL,
            similarity_threshold REAL NOT NULL,
            max_corrections INTEGER NOT NULL,
            corrections_used INTEGER NOT NULL DEFAULT 0,
            proposer TEXT NOT NULL,
            critic TEXT NOT NULL,
            implementer TEXT NOT NULL,
            mode TEXT NOT NULL,
            scenario TEXT NOT NULL,
            allow_implementation INTEGER NOT NULL DEFAULT 0,
            confirm_external INTEGER NOT NULL DEFAULT 0,
            workspace_path TEXT,
            external_repo TEXT,
            active_agent TEXT,
            started_at TEXT,
            finished_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            elapsed_seconds REAL NOT NULL DEFAULT 0,
            slice_started_at TEXT,
            cursor_cli_session TEXT,
            copilot_cli_session TEXT,
            final_report TEXT,
            last_error TEXT,
            blocking_issues TEXT NOT NULL DEFAULT '[]',
            invalid_retries INTEGER NOT NULL DEFAULT 0,
            incident_count INTEGER NOT NULL DEFAULT 0,
            outcome_reason TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            sender TEXT NOT NULL,
            recipient TEXT NOT NULL,
            turn INTEGER NOT NULL,
            type TEXT NOT NULL,
            content TEXT NOT NULL,
            subject TEXT NOT NULL,
            references_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            delivery_state TEXT NOT NULL,
            raw_response TEXT,
            meta_json TEXT NOT NULL DEFAULT '{}',
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE tasks (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            parent_id TEXT,
            kind TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE agent_states (
            session_id TEXT NOT NULL,
            agent TEXT NOT NULL,
            status TEXT NOT NULL,
            detail TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL,
            PRIMARY KEY (session_id, agent)
        );

        CREATE TABLE errors (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            agent TEXT,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE decisions (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            resolved INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE test_results (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            command TEXT NOT NULL,
            exit_code INTEGER,
            stdout TEXT NOT NULL,
            stderr TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE status_changes (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            from_status TEXT NOT NULL,
            to_status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE git_snapshots (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );
        """,
    ),
    (
        2,
        """
        ALTER TABLE sessions ADD COLUMN notes TEXT NOT NULL DEFAULT '';
        CREATE INDEX idx_messages_session_created ON messages(session_id, created_at);
        CREATE INDEX idx_status_changes_session ON status_changes(session_id, created_at);
        """,
    ),
    (
        3,
        """
        ALTER TABLE sessions ADD COLUMN vibe_cli_session TEXT;
        ALTER TABLE sessions ADD COLUMN antigravity_cli_session TEXT;
        """,
    ),
]


def utcnow() -> str:
    return datetime.now(timezone.utc).isoformat()


def new_id() -> str:
    return str(uuid.uuid4())


class Database:
    def __init__(self, path: Path):
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._lock = RLock()
        self.conn = sqlite3.connect(self.path, check_same_thread=False)
        self.conn.row_factory = sqlite3.Row
        self.conn.execute("PRAGMA foreign_keys = ON")
        self.conn.execute("PRAGMA journal_mode = WAL")
        self.migrate()

    def close(self) -> None:
        with self._lock:
            self.conn.close()

    @property
    def version(self) -> int:
        with self._lock:
            row = self.conn.execute(
                "SELECT COALESCE(MAX(version), 0) AS version FROM schema_migrations"
            ).fetchone()
            return int(row["version"])

    def migrate(self) -> None:
        with self._lock:
            self.conn.execute(
                """
                CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL
                )
                """
            )
            current = self.conn.execute(
                "SELECT COALESCE(MAX(version), 0) AS version FROM schema_migrations"
            ).fetchone()["version"]
            for version, script in MIGRATIONS:
                if version <= current:
                    continue
                try:
                    self.conn.executescript(script)
                    self.conn.execute(
                        "INSERT INTO schema_migrations(version, applied_at) VALUES (?, ?)",
                        (version, utcnow()),
                    )
                    self.conn.commit()
                except Exception:
                    self.conn.rollback()
                    raise

    def _one(self, query: str, params: tuple = ()) -> dict | None:
        row = self.conn.execute(query, params).fetchone()
        return dict(row) if row else None

    def _all(self, query: str, params: tuple = ()) -> list[dict]:
        return [dict(row) for row in self.conn.execute(query, params).fetchall()]

    def create_session(self, fields: dict) -> dict:
        with self._lock:
            now = utcnow()
            sid = fields.get("id") or new_id()
            title = fields["title"]
            self.conn.execute(
                """
                INSERT INTO sessions (
                    id, title, task_text, status, phase, next_action, turn,
                    max_turns, max_duration_seconds, agent_timeout_seconds,
                    similarity_threshold, max_corrections, corrections_used,
                    proposer, critic, implementer, mode, scenario,
                    allow_implementation, confirm_external, workspace_path,
                    external_repo, active_agent, started_at, finished_at,
                    created_at, updated_at, elapsed_seconds, slice_started_at,
                    cursor_cli_session, copilot_cli_session, final_report,
                    last_error, blocking_issues, invalid_retries, incident_count,
                    outcome_reason, notes
                ) VALUES (
                    ?, ?, ?, 'created', 'received', 'analyze', 0,
                    ?, ?, ?,
                    ?, ?, 0,
                    ?, ?, ?, ?, ?,
                    ?, ?, ?,
                    ?, NULL, NULL, NULL,
                    ?, ?, 0, NULL,
                    NULL, NULL, NULL,
                    '', '[]', 0, 0,
                    '', ''
                )
                """,
                (
                    sid,
                    title,
                    fields["task_text"],
                    fields["max_turns"],
                    fields["max_duration_seconds"],
                    fields["agent_timeout_seconds"],
                    fields["similarity_threshold"],
                    fields["max_corrections"],
                    fields["proposer"],
                    fields["critic"],
                    fields["implementer"],
                    fields["mode"],
                    fields["scenario"],
                    1 if fields.get("allow_implementation") else 0,
                    1 if fields.get("confirm_external") else 0,
                    fields.get("workspace_path"),
                    fields.get("external_repo"),
                    now,
                    now,
                ),
            )
            self.conn.execute(
                """
                INSERT INTO status_changes(id, session_id, from_status, to_status, created_at)
                VALUES (?, ?, '', 'created', ?)
                """,
                (new_id(), sid, now),
            )
            agents = {
                fields["proposer"],
                fields["critic"],
                fields["implementer"],
                "orchestrator",
            }
            for agent in sorted(agents):
                self.conn.execute(
                    """
                    INSERT INTO agent_states(session_id, agent, status, detail, updated_at)
                    VALUES (?, ?, 'available', '', ?)
                    """,
                    (sid, agent, now),
                )
            self.conn.commit()
            return self._one("SELECT * FROM sessions WHERE id = ?", (sid,))

    def get_session(self, session_id: str) -> dict | None:
        with self._lock:
            return self._one("SELECT * FROM sessions WHERE id = ?", (session_id,))

    def list_sessions(self) -> list[dict]:
        with self._lock:
            return self._all("SELECT * FROM sessions ORDER BY created_at DESC")

    def update_session(self, session_id: str, **fields) -> dict:
        allowed = {
            "title",
            "status",
            "phase",
            "next_action",
            "turn",
            "corrections_used",
            "workspace_path",
            "active_agent",
            "started_at",
            "finished_at",
            "elapsed_seconds",
            "slice_started_at",
            "cursor_cli_session",
            "copilot_cli_session",
            "vibe_cli_session",
            "antigravity_cli_session",
            "final_report",
            "last_error",
            "blocking_issues",
            "invalid_retries",
            "incident_count",
            "outcome_reason",
            "notes",
            "scenario",
        }
        unknown = set(fields) - allowed
        if unknown:
            raise ValueError(f"Champ de session inconnu : {', '.join(sorted(unknown))}")
        with self._lock:
            current = self._one("SELECT * FROM sessions WHERE id = ?", (session_id,))
            if current is None:
                raise KeyError(session_id)
            now = utcnow()
            if "status" in fields and fields["status"] != current["status"]:
                self.conn.execute(
                    """
                    INSERT INTO status_changes(id, session_id, from_status, to_status, created_at)
                    VALUES (?, ?, ?, ?, ?)
                    """,
                    (new_id(), session_id, current["status"], fields["status"], now),
                )
            assignments = ["updated_at = ?"]
            values: list = [now]
            for key, value in fields.items():
                assignments.append(f"{key} = ?")
                values.append(value)
            values.append(session_id)
            self.conn.execute(
                f"UPDATE sessions SET {', '.join(assignments)} WHERE id = ?",
                values,
            )
            self.conn.commit()
            return self._one("SELECT * FROM sessions WHERE id = ?", (session_id,))

    def insert_message(self, fields: dict) -> dict:
        with self._lock:
            mid = fields.get("id") or new_id()
            now = fields.get("created_at") or utcnow()
            references = fields.get("references") or []
            meta = fields.get("meta") or {}
            self.conn.execute(
                """
                INSERT INTO messages (
                    id, session_id, sender, recipient, turn, type, content,
                    subject, references_json, created_at, delivery_state,
                    raw_response, meta_json
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    mid,
                    fields["session_id"],
                    fields["sender"],
                    fields["recipient"],
                    fields["turn"],
                    fields["type"],
                    fields["content"],
                    fields.get("subject") or "",
                    json.dumps(references, ensure_ascii=False),
                    now,
                    fields.get("delivery_state") or "persisted",
                    fields.get("raw_response"),
                    json.dumps(meta, ensure_ascii=False),
                ),
            )
            self.conn.commit()
            return self.get_message(mid)

    def get_message(self, message_id: str) -> dict | None:
        with self._lock:
            row = self._one("SELECT * FROM messages WHERE id = ?", (message_id,))
            return _hydrate_message(row) if row else None

    def mark_transmitted(self, message_id: str) -> None:
        with self._lock:
            self.conn.execute(
                "UPDATE messages SET delivery_state = 'transmitted' WHERE id = ?",
                (message_id,),
            )
            self.conn.commit()

    def list_messages(self, session_id: str) -> list[dict]:
        with self._lock:
            rows = self._all(
                """
                SELECT * FROM messages
                WHERE session_id = ?
                ORDER BY created_at ASC, rowid ASC
                """,
                (session_id,),
            )
            return [_hydrate_message(row) for row in rows]

    def insert_task(self, fields: dict) -> dict:
        with self._lock:
            tid = fields.get("id") or new_id()
            now = utcnow()
            self.conn.execute(
                """
                INSERT INTO tasks(id, session_id, title, status, parent_id, kind, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    tid,
                    fields["session_id"],
                    fields["title"],
                    fields.get("status") or "pending",
                    fields.get("parent_id"),
                    fields["kind"],
                    now,
                    now,
                ),
            )
            self.conn.commit()
            return self._one("SELECT * FROM tasks WHERE id = ?", (tid,))

    def update_task(self, task_id: str, status: str) -> None:
        with self._lock:
            self.conn.execute(
                "UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?",
                (status, utcnow(), task_id),
            )
            self.conn.commit()

    def list_tasks(self, session_id: str) -> list[dict]:
        with self._lock:
            return self._all(
                "SELECT * FROM tasks WHERE session_id = ? ORDER BY created_at ASC",
                (session_id,),
            )

    def upsert_agent_state(self, session_id: str, agent: str, status: str, detail: str = "") -> None:
        with self._lock:
            now = utcnow()
            self.conn.execute(
                """
                INSERT INTO agent_states(session_id, agent, status, detail, updated_at)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(session_id, agent) DO UPDATE SET
                    status = excluded.status,
                    detail = excluded.detail,
                    updated_at = excluded.updated_at
                """,
                (session_id, agent, status, detail[:500], now),
            )
            self.conn.commit()

    def list_agent_states(self, session_id: str) -> list[dict]:
        with self._lock:
            return self._all(
                "SELECT * FROM agent_states WHERE session_id = ? ORDER BY agent ASC",
                (session_id,),
            )

    def insert_error(self, session_id: str, message: str, agent: str | None = None) -> dict:
        with self._lock:
            eid = new_id()
            now = utcnow()
            self.conn.execute(
                "INSERT INTO errors(id, session_id, agent, message, created_at) VALUES (?, ?, ?, ?, ?)",
                (eid, session_id, agent, message, now),
            )
            self.conn.commit()
            return self._one("SELECT * FROM errors WHERE id = ?", (eid,))

    def list_errors(self, session_id: str) -> list[dict]:
        with self._lock:
            return self._all(
                "SELECT * FROM errors WHERE session_id = ? ORDER BY created_at ASC",
                (session_id,),
            )

    def insert_decision(self, session_id: str, summary: str, resolved: bool) -> dict:
        with self._lock:
            did = new_id()
            now = utcnow()
            self.conn.execute(
                """
                INSERT INTO decisions(id, session_id, summary, resolved, created_at)
                VALUES (?, ?, ?, ?, ?)
                """,
                (did, session_id, summary, 1 if resolved else 0, now),
            )
            self.conn.commit()
            return self._one("SELECT * FROM decisions WHERE id = ?", (did,))

    def resolve_open_decisions(self, session_id: str) -> None:
        with self._lock:
            self.conn.execute(
                "UPDATE decisions SET resolved = 1 WHERE session_id = ? AND resolved = 0",
                (session_id,),
            )
            self.conn.commit()

    def list_decisions(self, session_id: str) -> list[dict]:
        with self._lock:
            return self._all(
                "SELECT * FROM decisions WHERE session_id = ? ORDER BY created_at ASC",
                (session_id,),
            )

    def insert_test_result(
        self,
        session_id: str,
        command: str,
        exit_code: int | None,
        stdout: str,
        stderr: str,
    ) -> dict:
        with self._lock:
            tid = new_id()
            now = utcnow()
            self.conn.execute(
                """
                INSERT INTO test_results(id, session_id, command, exit_code, stdout, stderr, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (tid, session_id, command, exit_code, stdout, stderr, now),
            )
            self.conn.commit()
            return self._one("SELECT * FROM test_results WHERE id = ?", (tid,))

    def list_test_results(self, session_id: str) -> list[dict]:
        with self._lock:
            return self._all(
                "SELECT * FROM test_results WHERE session_id = ? ORDER BY created_at ASC",
                (session_id,),
            )

    def list_status_changes(self, session_id: str) -> list[dict]:
        with self._lock:
            return self._all(
                "SELECT * FROM status_changes WHERE session_id = ? ORDER BY created_at ASC",
                (session_id,),
            )

    def insert_git_snapshot(self, session_id: str, kind: str, payload: str) -> dict:
        with self._lock:
            gid = new_id()
            now = utcnow()
            self.conn.execute(
                """
                INSERT INTO git_snapshots(id, session_id, kind, payload, created_at)
                VALUES (?, ?, ?, ?, ?)
                """,
                (gid, session_id, kind, payload, now),
            )
            self.conn.commit()
            return self._one("SELECT * FROM git_snapshots WHERE id = ?", (gid,))

    def list_git_snapshots(self, session_id: str) -> list[dict]:
        with self._lock:
            return self._all(
                "SELECT * FROM git_snapshots WHERE session_id = ? ORDER BY created_at ASC",
                (session_id,),
            )


def _hydrate_message(row: dict) -> dict:
    item = dict(row)
    item["references"] = json.loads(item.pop("references_json") or "[]")
    item["meta"] = json.loads(item.pop("meta_json") or "{}")
    return item
