from __future__ import annotations

from pathlib import Path

from fastapi.testclient import TestClient

from duo.api import create_app
from duo.config import Settings


def _app(tmp_path: Path) -> TestClient:
    data = tmp_path / "data"
    data.mkdir()
    settings = Settings(data_dir=data, allowed_roots=(tmp_path,))
    return TestClient(create_app(settings))


def test_api_rejects_cross_site_and_missing_header(tmp_path: Path):
    with _app(tmp_path) as client:
        denied = client.post("/api/sessions", json={"task": "x"})
        assert denied.status_code == 403
        evil = client.post(
            "/api/sessions",
            json={"task": "x"},
            headers={"X-Duo-Request": "1", "Origin": "https://evil.example"},
        )
        assert evil.status_code == 403
        health = client.get("/api/health", headers={"Origin": "https://evil.example"})
        assert health.status_code == 403
        ok = client.get("/api/health")
        assert ok.status_code == 200
        assert "cursor" in ok.json()["adapters"]
        assert "copilot" in ok.json()["adapters"]


def test_api_simulated_session_export_and_history(tmp_path: Path):
    with _app(tmp_path) as client:
        created = client.post(
            "/api/sessions",
            headers={"X-Duo-Request": "1"},
            json={
                "task": "Concevoir add(a, b) et le vérifier.",
                "mode": "simulated",
                "wait": True,
            },
        )
        assert created.status_code == 200
        body = created.json()
        assert body["session"]["status"] == "succeeded"
        assert len(body["messages"]) >= 4
        session_id = body["session"]["id"]
        listing = client.get("/api/sessions")
        assert any(item["id"] == session_id for item in listing.json())
        exported = client.get(f"/api/sessions/{session_id}/export?format=md")
        assert exported.status_code == 200
        assert "add(a, b)" in exported.text
        again = client.post(
            f"/api/sessions/{session_id}/resume",
            headers={"X-Duo-Request": "1"},
        )
        assert again.status_code == 400
