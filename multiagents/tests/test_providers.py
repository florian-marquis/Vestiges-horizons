from __future__ import annotations

import json
from pathlib import Path

from fastapi.testclient import TestClient

from duo.adapters.antigravity_cli import extract_agy_output
from duo.adapters.vibe_cli import VibeCliAdapter, extract_vibe_output
from duo.api import create_app
from duo.config import Settings
from duo.models import AgentRequest
from duo.providers import compute_routing
from duo.providers.antigravity import AntigravityProvider, parse_antigravity_swarm_payload
from duo.providers.copilot import CopilotProvider, parse_copilot_user
from duo.providers.cursor import CursorProvider, parse_usage_summary
from duo.providers.vibe import VibeProvider
from tests.conftest import make_stack, run, session_fields


def _app(tmp_path: Path) -> TestClient:
    data = tmp_path / "data"
    data.mkdir()
    settings = Settings(data_dir=data, allowed_roots=(tmp_path,))
    return TestClient(create_app(settings))


def test_providers_list_and_login_localhost_only(tmp_path: Path):
    with _app(tmp_path) as client:
        listing = client.get("/api/providers")
        assert listing.status_code == 200
        ids = [item["id"] for item in listing.json()["providers"]]
        assert ids == ["cursor", "copilot", "vibe", "antigravity"]
        for item in listing.json()["providers"]:
            assert "quotas" in item
            assert "models" in item

        denied_host = client.post(
            "/api/providers/cursor/login",
            headers={"X-Duo-Request": "1", "Host": "evil.example"},
            json={},
        )
        assert denied_host.status_code == 403

        missing = client.post(
            "/api/providers/cursor/login",
            json={},
        )
        assert missing.status_code == 403

        ok = client.post(
            "/api/providers/vibe/login",
            headers={"X-Duo-Request": "1"},
            json={},
        )
        assert ok.status_code == 200
        body = ok.json()
        assert "detail" in body
        assert "started" in body


def test_cursor_usage_summary_fixture_parsing():
    payload = {
        "individualUsage": {
            "plan": {
                "used": 120,
                "limit": 500,
                "remaining": 380,
                "totalPercentUsed": 24,
                "billingCycleEnd": "2026-12-01",
            }
        },
        "teamUsage": {"pooled": {"enabled": True, "used": 10, "limit": 100, "remaining": 90}},
    }
    items = parse_usage_summary(payload)
    assert items[0].status == "live"
    assert items[0].remaining == 380
    assert any(item.id == "team_pool" for item in items)


def test_copilot_quota_fixture_parsing():
    payload = {
        "quota_snapshots": {
            "premium_interactions": {
                "entitlement": 300,
                "remaining": 120,
                "percent_remaining": 40,
                "reset_date": "2026-12-01",
            },
            "chat": {"entitlement": 1000, "remaining": 800, "percent_remaining": 80},
        }
    }
    items = parse_copilot_user(payload)
    assert items[0].id == "premium_interactions"
    assert items[0].status == "live"
    assert items[0].remaining == 120


def test_provider_status_without_cli_is_unavailable(tmp_path: Path, monkeypatch):
    monkeypatch.setattr("duo.providers.cursor.resolve_cursor_agent_command", lambda: None)
    monkeypatch.setattr("duo.providers.vibe.resolve_vibe_command", lambda: None)
    monkeypatch.setattr("duo.providers.antigravity.resolve_agy_command", lambda: None)
    monkeypatch.setattr("duo.providers.copilot.resolve_copilot_command", lambda: None)

    async def _run():
        cursor = await CursorProvider().status()
        vibe = await VibeProvider().status()
        agy = await AntigravityProvider().status()
        copilot = await CopilotProvider().status()
        assert cursor.available is False
        assert vibe.available is False
        assert agy.available is False
        assert copilot.available is False
        assert all(q.status == "unavailable" for q in vibe.quotas)

    run(_run())


def test_vibe_and_agy_output_extractors():
    text, sid = extract_vibe_output(
        json.dumps(
            [
                {"role": "user", "content": "hi"},
                {"role": "assistant", "content": '{"message_type":"PROPOSAL","content":"ok"}'},
            ]
        )
    )
    assert "PROPOSAL" in text
    assert sid is None

    agy_text, agy_sid = extract_agy_output(
        json.dumps(
            {
                "conversation_id": "conv-1",
                "status": "SUCCESS",
                "response": "réponse agent",
            }
        )
    )
    assert agy_text == "réponse agent"
    assert agy_sid == "conv-1"


def test_vibe_cli_args_non_interactive():
    request = AgentRequest(
        session_id="s",
        agent="vibe",
        role="proposer",
        expected_types=frozenset({"PROPOSAL"}),
        task="tâche & spéciale",
        subject="sujet",
        history=[],
        turn=1,
        workspace="C:/bac",
        allow_write=False,
        timeout=5,
        resume_id=None,
        phase="analyze",
    )
    adapter = VibeCliAdapter(command=["vibe.exe"])
    args = adapter.build_args(request, "C:/bac")
    assert args[0] == "vibe.exe"
    assert "--prompt" in args
    assert "--output" in args
    assert "json" in args
    assert "--auto-approve" not in args
    request.allow_write = True
    write_args = adapter.build_args(request, "C:/bac")
    assert "--auto-approve" in write_args


def test_orchestrator_simulated_vibe_antigravity(tmp_path: Path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(
        session_fields(proposer="vibe", critic="antigravity", implementer="vibe")
    )
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "succeeded"
    senders = {item["sender"] for item in db.list_messages(session["id"])}
    assert "vibe" in senders
    assert "antigravity" in senders
    db.close()


def test_antigravity_swarm_payload_parsing():
    swarm_payload = {
        "models": [
            "gemini-pro-orchestrator-g1-pro",
            "gemini-3.8-flash-high",
            "claude-opus-5-5-high",
        ],
        "display_names": {
            "gemini-pro-orchestrator-g1-pro": "Orchestrator (Argon)",
            "gemini-3.8-flash-high": "Gemini 3.8 Flash (High)",
            "claude-opus-5-5-high": "Claude Opus 5.5 (High)",
        },
        "quotas": {
            "gemini-pro-orchestrator-g1-pro": {
                "quota_remaining": "80%",
                "quota_resets": "2026-11-25T14:14:26Z",
            },
            "gemini-3.8-flash-high": {
                "quota_remaining": "95%",
                "quota_resets": "2026-11-18T19:15:33Z",
            },
            "claude-opus-5-5-high": {
                "quota_remaining": "10%",
                "quota_resets": "2026-11-18T19:15:33Z",
            },
        },
    }
    models, quotas = parse_antigravity_swarm_payload(swarm_payload)
    assert len(models) == 3
    by_id = {m.id: m for m in models}
    assert by_id["gemini-3.8-flash-high"].tier == "light"
    assert "hebdo" in by_id["gemini-pro-orchestrator-g1-pro"].cost_hint
    q_by_id = {q.id: q for q in quotas}
    assert q_by_id["weekly"].status == "live"
    assert q_by_id["weekly"].remaining == 80.0
    assert q_by_id["rolling_5h"].remaining == 10.0
    assert q_by_id["gemini-3.8-flash-high"].remaining == 95.0


def test_copilot_models_expose_cost_multipliers():
    models = CopilotProvider()._models()
    by_id = {m.id: m for m in models}
    assert "0x" in by_id["gpt-4.1"].cost_hint
    assert "0.33x" in by_id["gemini-3-flash"].cost_hint
    assert "1x" in by_id["claude-sonnet-4.6"].cost_hint
    assert "3x" in by_id["claude-opus-4.6"].cost_hint


def test_compute_routing_switches_pilote_tool_when_exhausted():
    snapshots = {
        "antigravity": {
            "id": "antigravity",
            "available": True,
            "authenticated": True,
            "models": [{"id": "gemini-pro-orchestrator-g1-pro", "label": "Orchestrator (Argon)"}],
            "quotas": [
                {"id": "weekly", "status": "live", "remaining": 0.0, "limit": 100.0},
                {"id": "rolling_5h", "status": "live", "remaining": 0.0, "limit": 100.0},
            ],
        },
        "copilot": {
            "id": "copilot",
            "available": True,
            "authenticated": True,
            "models": [
                {"id": "gpt-4.1", "label": "GPT-4.1", "cost_hint": "0x (inclus)"},
                {"id": "claude-sonnet-4.6", "label": "Claude Sonnet 4.6", "cost_hint": "1x"},
            ],
            "quotas": [
                {"id": "premium_interactions", "status": "live", "remaining": 60.0, "limit": 100.0}
            ],
        },
        "vibe": {
            "id": "vibe",
            "available": True,
            "authenticated": True,
            "models": [{"id": "devstral-medium-latest", "label": "Devstral Medium", "cost_hint": "standard"}],
            "quotas": [{"id": "manual_quota", "status": "manual", "remaining": 80.0, "limit": 100.0}],
        },
        "cursor": {
            "id": "cursor",
            "available": True,
            "authenticated": True,
            "models": [{"id": "auto", "label": "Auto / Cursor 2.5", "cost_hint": "quota maison"}],
            "quotas": [{"id": "plan", "status": "live", "remaining": 100.0, "limit": 100.0}],
        },
    }
    r1 = compute_routing(snapshots, difficulty="N1", current_pilote_provider="antigravity")
    assert r1["pilote"]["switch_tool_recommended"] is True
    assert r1["pilote"]["provider"] == "copilot"
    assert r1["pilote"]["model"] == "gpt-4.1"

    r3 = compute_routing(snapshots, difficulty="N3", current_pilote_provider="antigravity")
    assert r3["pilote"]["switch_tool_recommended"] is True
    assert r3["pilote"]["provider"] == "copilot"
    assert r3["pilote"]["model"] == "claude-sonnet-4.6"
    agents = {a["role"]: a for a in r3["agents"]}
    assert agents["agent2"]["model"] == "claude-sonnet-4.6"
    assert agents["agent3"]["model"] == "mistral-large-latest"


def test_providers_sync_and_routing_api(tmp_path: Path):
    with _app(tmp_path) as client:
        synced_agy = client.post(
            "/api/providers/antigravity/sync",
            headers={"X-Duo-Request": "1"},
            json={
                "models": ["gemini-pro-orchestrator-g1-pro", "gemini-3.8-flash-high"],
                "display_names": {
                    "gemini-pro-orchestrator-g1-pro": "Orchestrator (Argon)",
                    "gemini-3.8-flash-high": "Gemini 3.8 Flash (High)",
                },
                "quotas": {
                    "gemini-pro-orchestrator-g1-pro": {"quota_remaining": "90%"},
                    "gemini-3.8-flash-high": {"quota_remaining": "100%"},
                },
            },
        )
        assert synced_agy.status_code == 200
        assert any(q["id"] == "weekly" and q["remaining"] == 90.0 for q in synced_agy.json()["quotas"])

        synced_vibe = client.post(
            "/api/providers/vibe/sync",
            headers={"X-Duo-Request": "1"},
            json={"quota_override": {"remaining": 75, "limit": 100}, "preferred_model": "devstral-medium-latest"},
        )
        assert synced_vibe.status_code == 200
        assert synced_vibe.json()["quotas"][0]["remaining"] == 75.0

        routing = client.get("/api/routing", params={"difficulty": "N1", "pilote_provider": "antigravity"})
        assert routing.status_code == 200
        body = routing.json()
        assert body["difficulty"] == "N1"
        assert body["pilote"]["provider"] == "antigravity"
        assert body["pilote"]["model"] == "gemini-3.8-flash-high"


def test_learn_models_and_stable_session_profile(tmp_path: Path):
    with _app(tmp_path) as client:
        client.post(
            "/api/providers/antigravity/sync",
            headers={"X-Duo-Request": "1"},
            json={
                "models": ["gemini-pro-orchestrator-g1-pro", "gemini-3.8-flash-high"],
                "quotas": {
                    "gemini-pro-orchestrator-g1-pro": {"quota_remaining": "90%"},
                    "gemini-3.8-flash-high": {"quota_remaining": "92%"},
                },
            },
        )
        learned = client.post(
            "/api/providers/vibe/sync",
            headers={"X-Duo-Request": "1"},
            json={
                "add_models": [
                    {
                        "id": "mistral-large-4-0-3",
                        "label": "Mistral Large 4",
                        "tier": "deep",
                        "cost_hint": "élevé",
                    },
                    {
                        "id": "vibe-next-1",
                        "label": "Vibe Next 1",
                        "tier": "critical",
                        "cost_hint": "expérimental",
                    },
                ]
            },
        )
        assert learned.status_code == 200
        model_ids = [m["id"] for m in learned.json()["models"]]
        assert "mistral-large-4-0-3" in model_ids
        assert "vibe-next-1" in model_ids

        listing = client.get("/api/providers")
        vibe_snap = next(p for p in listing.json()["providers"] if p["id"] == "vibe")
        assert "vibe-next-1" in [m["id"] for m in vibe_snap["models"]]

        r1 = client.get("/api/routing", params={"difficulty": "N1", "pilote_provider": "antigravity"}).json()
        assert r1["assigned_role"] == "agent2"
        assert r1["requires_human_model_switch"] is False
        assert r1["session_profile"]["simple_agent"]["role"] == "agent2"
        assert r1["session_profile"]["simple_agent"]["model"] == "gpt-4.1"
        assert r1["session_profile"]["complex_agent"]["role"] == "agent3"
        assert r1["session_profile"]["complex_agent"]["model"] == "mistral-large-4-0-3"

        r3 = client.get("/api/routing", params={"difficulty": "N3", "pilote_provider": "antigravity"}).json()
        assert r3["assigned_role"] == "agent3"
        assert r3["requires_human_model_switch"] is False

        # Inversion automatique quand Vibe est épuisé
        client.post(
            "/api/providers/vibe/sync",
            headers={"X-Duo-Request": "1"},
            json={"quota_override": {"remaining": 0, "limit": 100}},
        )
        r3_inv = client.get("/api/routing", params={"difficulty": "N3", "pilote_provider": "antigravity"}).json()
        assert r3_inv["session_profile"]["mode"] == "inverted_vibe_exhausted"
        assert r3_inv["session_profile"]["complex_agent"]["role"] == "agent2"
        assert r3_inv["session_profile"]["complex_agent"]["model"] == "claude-sonnet-4.6"
        assert r3_inv["assigned_role"] == "agent2"
        assert r3_inv["requires_human_model_switch"] is True
