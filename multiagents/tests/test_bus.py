from __future__ import annotations

import asyncio
import json
import os
import subprocess
import sys
from pathlib import Path

import httpx
from fastapi.testclient import TestClient

from duo.api import create_app
from duo.bus import Bus, message_visible_to
from duo.config import Settings

HEADERS = {"X-Duo-Request": "1"}


def _client(tmp_path: Path) -> TestClient:
    data = tmp_path / "data"
    data.mkdir()
    settings = Settings(data_dir=data, allowed_roots=(tmp_path,))
    return TestClient(create_app(settings))


def test_room_starts_with_the_task_and_separate_tokens(tmp_path: Path):
    with _client(tmp_path) as client:
        denied = client.post("/api/rooms", json={"task": "Ajouter add(a, b)."})
        assert denied.status_code == 403
        created = client.post(
            "/api/rooms",
            headers=HEADERS,
            json={"task": "Ajouter add(a, b)."},
        )
        assert created.status_code == 200
        body = created.json()
        assert body["room"]["messages"][0]["sender"] == "humain"
        assert body["room"]["agents_count"] == 1
        assert body["room"]["roles"] == ["pilote", "agent1"]
        tokens = body["tokens"]
        assert tokens["pilote"] != tokens["agent1"]
        assert tokens["pilote"] in body["consignes"]["pilote"]
        assert "etat.md" in body["consignes"]["pilote"]
        assert "@agent1" in body["consignes"]["pilote"]
        assert "timeout=90" in body["consignes"]["agent1"]
        hidden = client.get(f"/api/rooms/{body['room']['id']}/consignes")
        assert hidden.status_code == 403


def test_three_agents_have_distinct_tokens_and_prompts(tmp_path: Path):
    with _client(tmp_path) as client:
        created = client.post(
            "/api/rooms",
            headers=HEADERS,
            json={
                "task": "Répartir le travail.",
                "agents_count": 3,
                "slots": [
                    {"window": "cursor"},
                    {"window": "vscode"},
                    {"window": "cursor"},
                ],
            },
        )
        assert created.status_code == 200
        body = created.json()
        assert body["room"]["agents_count"] == 3
        assert body["room"]["roles"] == ["pilote", "agent1", "agent2", "agent3"]
        tokens = body["tokens"]
        assert len(set(tokens.values())) == 4
        for role in body["room"]["roles"]:
            assert role in body["consignes"]
            assert tokens[role] in body["consignes"][role]


def test_addressing_wakes_only_target_agent(tmp_path: Path):
    with _client(tmp_path) as client:
        created = client.post(
            "/api/rooms",
            headers=HEADERS,
            json={
                "task": "Adresser.",
                "agents_count": 2,
                "slots": [{"window": "vscode"}, {"window": "cursor"}],
            },
        ).json()
        room_id = created["room"]["id"]
        tokens = created["tokens"]
        posted = client.post(
            f"/api/rooms/{room_id}/messages",
            headers=HEADERS,
            json={"token": tokens["pilote"], "text": "@agent2 fais la revue seulement."},
        )
        assert posted.status_code == 200
        assert posted.json()["sender"] == "pilote"
        agent2 = client.get(
            f"/api/rooms/{room_id}/wait",
            params={"token": tokens["agent2"], "after": 1, "timeout": 1},
        )
        assert agent2.json()["timed_out"] is False
        assert agent2.json()["messages"][0]["text"].startswith("@agent2")
        agent1 = client.get(
            f"/api/rooms/{room_id}/wait",
            params={"token": tokens["agent1"], "after": 1, "timeout": 1},
        )
        assert agent1.json()["timed_out"] is True
        assert agent1.json()["messages"] == []


def test_message_visibility_helpers():
    assert message_visible_to("pilote", "bonjour à tous", "agent1")
    assert not message_visible_to("pilote", "bonjour à tous", "pilote")
    assert message_visible_to("pilote", "@agent2 go", "agent2")
    assert not message_visible_to("pilote", "@agent2 go", "agent1")
    assert message_visible_to("pilote", "@agent2 go", "pilote") is False
    assert message_visible_to("agent1", "@pilote done", "pilote")
    assert message_visible_to("humain", "STOP", "agent3")


def test_agent_sees_the_other_and_not_itself(tmp_path: Path):
    with _client(tmp_path) as client:
        created = client.post(
            "/api/rooms",
            headers=HEADERS,
            json={"task": "Relire le plan."},
        ).json()
        room_id = created["room"]["id"]
        posted = client.post(
            f"/api/rooms/{room_id}/messages",
            headers=HEADERS,
            json={"token": created["pilote_token"], "text": "Je propose une fonction pure."},
        )
        assert posted.status_code == 200
        assert posted.json()["sender"] == "pilote"
        agent1 = client.get(
            f"/api/rooms/{room_id}/wait",
            params={"token": created["agent1_token"], "after": 1, "timeout": 1},
        )
        assert agent1.json()["timed_out"] is False
        own = client.get(
            f"/api/rooms/{room_id}/wait",
            params={"token": created["pilote_token"], "after": 1, "timeout": 1},
        )
        assert own.json()["timed_out"] is True
        stolen = client.post(
            f"/api/rooms/{room_id}/messages",
            headers={**HEADERS, "X-Duo-Agent": "agent1"},
            json={"token": created["pilote_token"], "text": "usurpation"},
        )
        assert stolen.status_code == 403


def test_wait_unblocks_when_the_other_speaks(tmp_path: Path):
    asyncio.run(_wait_unblocks(tmp_path))


async def _wait_unblocks(tmp_path: Path):
    data = tmp_path / "data"
    data.mkdir()
    app = create_app(Settings(data_dir=data))
    transport = httpx.ASGITransport(app=app)
    async with httpx.AsyncClient(transport=transport, base_url="http://testserver") as client:
        created = await client.post(
            "/api/rooms",
            headers=HEADERS,
            json={"task": "Se répondre."},
        )
        body = created.json()
        room_id = body["room"]["id"]

        async def speak():
            await asyncio.sleep(0.2)
            response = await client.post(
                f"/api/rooms/{room_id}/messages",
                headers=HEADERS,
                json={"token": body["pilote_token"], "text": "Premier plan."},
            )
            assert response.status_code == 200

        speaker = asyncio.create_task(speak())
        waited = await client.get(
            f"/api/rooms/{room_id}/wait",
            params={"token": body["agent1_token"], "after": 1, "timeout": 5},
        )
        await speaker
        payload = waited.json()
        assert payload["timed_out"] is False
        assert payload["messages"][0]["sender"] == "pilote"


def test_closed_room_wakes_without_waiting_the_whole_delay(tmp_path: Path):
    bus = Bus(tmp_path / "bus.sqlite")
    room = bus.create_room("Finir.")

    async def scenario():
        async def shut():
            await asyncio.sleep(0.1)
            bus.close_room(room["id"])

        closer = asyncio.create_task(shut())
        result = await bus.wait(room["id"], room["agent1_token"], after=1, timeout=30)
        await closer
        return result

    result = asyncio.run(scenario())
    assert result["closed"] is True
    bus.close()


def test_mcp_lists_tools_and_reports_a_down_salon():
    env = dict(os.environ)
    root = Path(__file__).resolve().parents[1]
    env["PYTHONPATH"] = str(root / "src")
    env["DUO_BUS_URL"] = "http://127.0.0.1:9"
    payload = "\n".join(
        [
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {"protocolVersion": "2024-11-05"},
                }
            ),
            json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            json.dumps({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "tools/call",
                    "params": {
                        "name": "dire",
                        "arguments": {"room_id": "x", "token": "y", "text": "bonjour"},
                    },
                }
            ),
        ]
    )
    completed = subprocess.run(
        [sys.executable, "-m", "duo.mcp_bus"],
        input=payload.encode("utf-8"),
        capture_output=True,
        env=env,
        timeout=20,
        check=False,
    )
    assert completed.returncode == 0
    lines = [json.loads(line) for line in completed.stdout.decode("utf-8").splitlines() if line.strip()]
    assert lines[0]["id"] == 1
    names = [tool["name"] for tool in lines[1]["result"]["tools"]]
    assert names == ["lire_salon", "dire", "attendre"]
    assert lines[2]["result"]["isError"] is True
