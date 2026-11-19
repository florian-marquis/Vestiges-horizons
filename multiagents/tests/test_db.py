from __future__ import annotations

import sqlite3
from pathlib import Path

from duo.db import MIGRATIONS, Database
from duo.security import redact

from tests.conftest import make_stack, run, session_fields


def test_messages_survive_reopen(tmp_path: Path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields())
    db.insert_message(
        {
            "session_id": session["id"],
            "sender": "user",
            "recipient": "orchestrator",
            "turn": 0,
            "type": "TASK",
            "content": "persistance",
            "subject": "persistance",
            "references": [],
            "delivery_state": "persisted",
        }
    )
    path = db.path
    db.close()
    reopened = Database(path)
    messages = reopened.list_messages(session["id"])
    assert messages[0]["content"] == "persistance"
    assert messages[0]["delivery_state"] == "persisted"
    reopened.mark_transmitted(messages[0]["id"])
    assert reopened.get_message(messages[0]["id"])["delivery_state"] == "transmitted"
    reopened.close()


def test_migrations_apply_on_previous_schema(tmp_path: Path):
    path = tmp_path / "legacy.sqlite"
    conn = sqlite3.connect(path)
    conn.execute(
        "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
    )
    conn.executescript(MIGRATIONS[0][1])
    conn.execute("INSERT INTO schema_migrations(version, applied_at) VALUES (1, '2020-01-01T00:00:00+00:00')")
    conn.commit()
    conn.close()
    db = Database(path)
    assert db.version == 3
    db.conn.execute("SELECT notes FROM sessions").fetchall()
    db.conn.execute("SELECT vibe_cli_session, antigravity_cli_session FROM sessions").fetchall()
    db.close()


def test_redact_tokens():
    text = "github_pat_abcdefghijklmnopqrstuvwxyz et Bearer abcdefghijklmnop"
    cleaned = redact(text)
    assert "github_pat_" not in cleaned
    assert "abcdefghijklmnop" not in cleaned
    assert "secret masqué" in cleaned.lower() or "masqué" in cleaned
