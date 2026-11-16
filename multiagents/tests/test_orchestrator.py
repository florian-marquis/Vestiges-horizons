from __future__ import annotations

from duo.adapters.base import AgentAdapter
from duo.adapters.copilot_cli import CopilotCliAdapter
from duo.adapters.cursor_cli import CursorCliAdapter
from duo.adapters.simulated import SimulatedAdapter
from duo.models import AgentResult, CancelToken, HealthStatus
from duo.protocol import opinion_flip, parse_payload, similarity

from tests.conftest import make_stack, run, session_fields


class Down(AgentAdapter):
    def __init__(self, name: str):
        self.name = name

    async def health(self) -> HealthStatus:
        return HealthStatus(self.name, False, None, f"{self.name} introuvable dans cet environnement.", ["run"])

    async def run(self, request, token) -> AgentResult:
        del request, token
        return AgentResult(ok=False, error="indisponible")


class AlwaysInvalid(SimulatedAdapter):
    async def run(self, request, token) -> AgentResult:
        del request, token
        return AgentResult(ok=True, text="Ceci n'est pas un message protocolaire.", exit_code=0)


def test_parse_fenced_and_embedded_json():
    text = 'brouillon\n```json\n{"message_type": "CRITIQUE", "content": "manque un test", "blocking_issues": ["test"], "agreement": false}\n```'
    payload = parse_payload(text)
    assert payload["message_type"] == "CRITIQUE"
    assert payload["blocking_issues"] == ["test"]


def test_similarity_and_flip():
    assert similarity("Bonjour   le monde", "bonjour le monde") == 1
    assert opinion_flip(
        {"content": "même texte", "agreement": False},
        {"content": "même texte", "agreement": True},
        0.9,
    )
    assert not opinion_flip(
        {"content": "une architecture en services", "agreement": False},
        {"content": "j'accepte après ajout des tests manquants et du contrat", "agreement": True},
        0.92,
    )


def test_cli_args_keep_metacharacters_out_of_the_shell():
    request_task = "Ajouter add & calc.exe | rm -rf /"
    from duo.models import AgentRequest

    request = AgentRequest(
        session_id="s",
        agent="copilot",
        role="critiquer",
        expected_types=frozenset({"CRITIQUE"}),
        task=request_task,
        subject="sujet",
        history=[],
        turn=1,
        workspace="C:/bac",
        allow_write=False,
        timeout=5,
        resume_id=None,
        phase="critique",
    )
    adapter = CopilotCliAdapter(command=["node", "loader.js"])
    args = adapter.build_args(request, "C:/bac")
    assert args[0] == "node"
    assert "cmd.exe" not in [item.lower() for item in args]
    assert any("calc.exe" in item and "&" in item for item in args)
    assert args.count("-p") == 1
    cursor = CursorCliAdapter(command=["agent.exe"])
    cursor_args = cursor.build_args(request, "C:/bac")
    assert cursor_args[0] == "agent.exe"
    assert "--force" not in cursor_args
    assert "--mode" in cursor_args
    request.allow_write = True
    forced = cursor.build_args(request, "C:/bac")
    assert "--force" in forced
    assert "--mode" not in forced
    request.resume_id = "session-123"
    resumed = cursor.build_args(request, "C:/bac")
    assert resumed[resumed.index("--resume") + 1] == "session-123"


def test_message_is_transmitted_before_the_next_agent_reads_it(tmp_path):
    _, db, orch = make_stack(tmp_path)
    seen = []

    class Watch(SimulatedAdapter):
        async def run(self, request, token):
            seen.append([item["delivery_state"] for item in db.list_messages(request.session_id)])
            return await super().run(request, token)

    session = orch.create_session(session_fields())
    orch._adapters[session["id"]] = {
        "cursor": Watch("cursor"),
        "copilot": Watch("copilot"),
    }
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "succeeded"
    assert seen[0] == ["transmitted"]
    assert "persisted" not in seen[1]
    critique = next(item for item in db.list_messages(session["id"]) if item["type"] == "CRITIQUE")
    assert critique["references"]
    db.close()


def test_invalid_then_valid_continues(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(scenario="invalide"))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "succeeded"
    assert any(item["type"] == "ERROR" for item in db.list_messages(session["id"]))
    db.close()


def test_persistent_invalid_response_stops(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields())
    orch._adapters[session["id"]] = {
        "cursor": AlwaysInvalid("cursor"),
        "copilot": SimulatedAdapter("copilot"),
    }
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "error"
    assert result["next_action"] == "analyze"
    assert any(item["type"] == "ERROR" for item in db.list_messages(session["id"]))
    db.close()


def test_launch_error_then_retry(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(scenario="panne"))
    failed = run(orch.run_session(session["id"]))
    assert failed["status"] == "error"
    assert "lancement" in failed["last_error"].lower()
    recovered = run(orch.retry(session["id"]))
    assert recovered["status"] == "succeeded"
    db.close()


def test_timeout(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(scenario="lent", agent_timeout_seconds=1, max_duration_seconds=30))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "error"
    assert "délai" in result["last_error"].lower()
    db.close()


def test_immediate_stop_and_resume(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(scenario="lent", agent_timeout_seconds=30))

    async def flow():
        import asyncio

        task = asyncio.create_task(orch.run_session(session["id"]))
        await asyncio.sleep(0.15)
        orch.stop(session["id"])
        await task
        db.update_session(session["id"], scenario="collaboration")
        return await orch.resume(session["id"])

    result = run(flow())
    assert db.get_session(session["id"])["status"] == "succeeded" or result["status"] == "succeeded"
    changes = [item["to_status"] for item in db.list_status_changes(session["id"])]
    assert "interrupted" in changes
    db.close()


def test_pause_and_resume_after_restart(tmp_path):
    from duo.adapters.simulated import SimulatedAdapter as Sim

    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields())

    class PauseCursor(Sim):
        async def run(self, request, token):
            result = await super().run(request, token)
            if request.phase == "analyze":
                orch.pause(request.session_id)
            return result

    orch._adapters[session["id"]] = {"cursor": PauseCursor("cursor"), "copilot": Sim("copilot")}
    paused = run(orch.run_session(session["id"]))
    assert paused["status"] == "paused"
    assert any(item["type"] == "PROPOSAL" for item in db.list_messages(session["id"]))
    path = db.path
    db.close()
    _, db2, orch2 = make_stack(tmp_path)
    del _
    assert db2.path == path
    resumed = run(orch2.resume(session["id"]))
    assert resumed["status"] == "succeeded"
    db2.close()


def test_loop_detection(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(scenario="boucle"))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "needs_arbitration"
    assert result["outcome_reason"] == "loop"
    db.close()


def test_max_turns(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(max_turns=2, scenario="desaccord"))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "needs_arbitration"
    assert result["turn"] <= 2
    assert "max_turns" in (result["outcome_reason"] or "")
    db.close()


def test_disagreement_ends_cleanly(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(scenario="desaccord"))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "needs_arbitration"
    assert any(item["type"] == "DISAGREEMENT" for item in db.list_messages(session["id"]))
    report = __import__("json").loads(result["final_report"])
    assert report["disagreements_open"]
    db.close()


def test_convergence_and_failing_tests_are_corrected(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(scenario="tests_fail"))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "succeeded"
    tests = db.list_test_results(session["id"])
    assert [item["exit_code"] for item in tests] == [1, 0]
    report = __import__("json").loads(result["final_report"])
    assert report["outcome"] == "succeeded"
    assert report["files_modified"]
    assert report["tests"][-1]["exit_code"] == 0
    db.close()


def test_duration_limit(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(session_fields(max_duration_seconds=0))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "needs_arbitration"
    assert result["outcome_reason"] == "duration"
    db.close()


def test_live_agent_unavailable(tmp_path):
    _, db, orch = make_stack(tmp_path)
    orch.live_adapters = {"cursor": Down("cursor"), "copilot": Down("copilot")}
    session = orch.create_session(session_fields(mode="live"))
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "failed"
    assert "introuvable" in result["last_error"]
    assert any(item["type"] == "ERROR" for item in db.list_messages(session["id"]))
    db.close()


def test_cursor_resolver_prefers_latest_version(tmp_path):
    from duo.adapters.cli_common import resolve_cursor_agent_command

    for name in ("2026.10.9-aaaa", "2026.10.14-bbbb"):
        folder = tmp_path / "versions" / name
        folder.mkdir(parents=True)
        (folder / "node.exe").write_bytes(b"")
        (folder / "index.js").write_text("", encoding="utf-8")
    command = resolve_cursor_agent_command(tmp_path)
    assert command is not None
    assert "2026.10.14-bbbb" in command[1]


def test_copilot_missing_auth_is_explicit():
    from duo.adapters.copilot_cli import CopilotCliAdapter
    from duo.models import AgentRequest, CancelToken

    async def runner(args, cwd, timeout, token, registry, session_id):
        del args, cwd, timeout, token, registry, session_id
        return 1, "", "Error: No authentication information found.", "ok"

    adapter = CopilotCliAdapter(command=["node", "loader.js"], runner=runner)
    request = AgentRequest(
        session_id="s",
        agent="copilot",
        role="répondre",
        expected_types=frozenset({"RESPONSE"}),
        task="ping",
        subject="ping",
        history=[],
        turn=1,
        workspace="C:/bac",
        allow_write=False,
        timeout=5,
        resume_id=None,
        phase="respond",
    )
    result = run(adapter.run(request, CancelToken()))
    assert result.ok is False
    assert "copilot login" in result.error


def test_roles_can_be_swapped(tmp_path):
    _, db, orch = make_stack(tmp_path)
    session = orch.create_session(
        session_fields(proposer="copilot", critic="cursor", implementer="copilot")
    )
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "succeeded"
    proposal = next(item for item in db.list_messages(session["id"]) if item["type"] == "PROPOSAL")
    assert proposal["sender"] == "copilot"
    db.close()
