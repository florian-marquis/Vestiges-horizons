"""Machine à états de la collaboration entre agents."""

from __future__ import annotations

import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from duo.adapters.simulated import SimulatedAdapter
from duo.db import Database, utcnow
from duo.gitspace import GitError, WorkspaceManager, write_declared_files
from duo.models import AgentRequest, AgentResult, CancelToken
from duo.protocol import (
    PHASE_TYPES,
    agreement_flag,
    blocking_issues,
    opinion_flip,
    parse_payload,
    payload_is_valid,
    similarity,
)
from duo.security import redact

AGENT_ACTIONS = frozenset(
    {"analyze", "critique", "respond", "revise", "implement", "review", "correct"}
)
LIMIT_REASONS = frozenset({"max_turns", "loop", "disagreement", "duration"})
TERMINAL = frozenset({"succeeded", "failed", "interrupted", "needs_arbitration"})
AGENTS = frozenset({"cursor", "copilot", "vibe", "antigravity"})

CLI_SESSION_COLUMNS = {
    "cursor": "cursor_cli_session",
    "copilot": "copilot_cli_session",
    "vibe": "vibe_cli_session",
    "antigravity": "antigravity_cli_session",
}


def _parse_time(value: str | None) -> datetime | None:
    if not value:
        return None
    return datetime.fromisoformat(value)


def _other(session: dict, name: str) -> str:
    if name == session["proposer"]:
        return session["critic"]
    if name == session["critic"]:
        return session["proposer"]
    for candidate in (session["proposer"], session["critic"], session["implementer"]):
        if candidate != name:
            return candidate
    raise ValueError(f"Pas d'autre agent pour {name}")


class Orchestrator:
    def __init__(self, db: Database, workspaces: WorkspaceManager, broker=None, live_adapters=None):
        self.db = db
        self.workspaces = workspaces
        self.broker = broker
        self.live_adapters = live_adapters or {}
        self._running: set[str] = set()
        self._paused: dict[str, bool] = {}
        self._stop: dict[str, bool] = {}
        self._tokens: dict[str, CancelToken] = {}
        self._adapters: dict[str, dict] = {}

    def create_session(self, fields: dict) -> dict:
        proposer = fields.get("proposer", "cursor")
        critic = fields.get("critic", "copilot")
        implementer = fields.get("implementer", "cursor")
        mode = fields.get("mode", "simulated")
        for name in (proposer, critic, implementer):
            if name not in AGENTS:
                raise ValueError("Agent inconnu.")
        if proposer == critic:
            raise ValueError("Le proposant et le critique doivent être distincts.")
        if mode not in {"simulated", "live"}:
            raise ValueError("Mode de session inconnu.")
        task = str(fields.get("task") or "").strip()
        if not task:
            raise ValueError("La tâche est vide.")
        title = task.splitlines()[0].strip()[:80] or "Session"
        return self.db.create_session(
            {
                "title": title,
                "task_text": task,
                "max_turns": int(fields.get("max_turns", 8)),
                "max_duration_seconds": int(fields.get("max_duration_seconds", 600)),
                "agent_timeout_seconds": int(fields.get("agent_timeout_seconds", 180)),
                "similarity_threshold": float(fields.get("similarity_threshold", 0.92)),
                "max_corrections": int(fields.get("max_corrections", 1)),
                "proposer": proposer,
                "critic": critic,
                "implementer": implementer,
                "mode": mode,
                "scenario": fields.get("scenario") or "collaboration",
                "allow_implementation": bool(fields.get("allow_implementation", True)),
                "confirm_external": bool(fields.get("confirm_external", False)),
                "external_repo": fields.get("external_repo"),
            }
        )

    def _adapters_for(self, session: dict) -> dict:
        sid = session["id"]
        if sid not in self._adapters:
            if session["mode"] == "simulated":
                names = {session["proposer"], session["critic"], session["implementer"]}
                self._adapters[sid] = {name: SimulatedAdapter(name) for name in names}
            else:
                self._adapters[sid] = self.live_adapters
        return self._adapters[sid]

    def _notify(self, session_id: str, kind: str) -> None:
        if self.broker is not None:
            self.broker.publish(session_id, {"session_id": session_id, "kind": kind})

    def _elapsed(self, session: dict) -> float:
        elapsed = float(session["elapsed_seconds"] or 0)
        started = _parse_time(session.get("slice_started_at"))
        if started is not None:
            elapsed += (datetime.now(timezone.utc) - started).total_seconds()
        return elapsed

    def _accrue(self, session_id: str) -> None:
        session = self.db.get_session(session_id)
        if session is None or not session.get("slice_started_at"):
            return
        self.db.update_session(
            session_id,
            elapsed_seconds=self._elapsed(session),
            slice_started_at=None,
        )

    def _begin_slice(self, session_id: str) -> None:
        session = self.db.get_session(session_id)
        if session and not session.get("slice_started_at"):
            fields = {"slice_started_at": utcnow()}
            if not session.get("started_at"):
                fields["started_at"] = utcnow()
            self.db.update_session(session_id, **fields)

    def pause(self, session_id: str) -> dict:
        session = self._require(session_id)
        if session["status"] not in {"created", "running"}:
            raise ValueError("La session n'est pas en cours.")
        self._paused[session_id] = True
        if session["status"] == "running":
            self._accrue(session_id)
            session = self.db.update_session(session_id, status="paused", active_agent=None)
        self._notify(session_id, "status")
        return session

    def stop(self, session_id: str) -> dict:
        self._require(session_id)
        self._stop[session_id] = True
        self._paused[session_id] = False
        token = self._tokens.get(session_id)
        if token is not None:
            token.trigger("stop")
        if session_id not in self._running:
            session = self.db.get_session(session_id)
            if session and session["status"] not in TERMINAL:
                self._accrue(session_id)
                session = self.db.update_session(
                    session_id,
                    status="interrupted",
                    active_agent=None,
                    finished_at=utcnow(),
                )
                self._agent_state(session_id, "orchestrator", "finished", "interrompue")
        self._notify(session_id, "status")
        return self.db.get_session(session_id)

    async def retry(self, session_id: str) -> dict:
        session = self._require(session_id)
        if session["status"] != "error":
            raise ValueError("Seule une session en erreur peut être relancée.")
        self._stop[session_id] = False
        self._paused[session_id] = False
        self.db.update_session(session_id, status="running", last_error="", finished_at=None)
        await self.run_session(session_id)
        return self.db.get_session(session_id)

    async def resume(self, session_id: str) -> dict:
        session = self._require(session_id)
        if session["status"] not in {"paused", "interrupted"}:
            raise ValueError("Cette session ne peut pas être reprise.")
        self._stop[session_id] = False
        self._paused[session_id] = False
        self.db.update_session(session_id, status="running", finished_at=None, last_error="")
        await self.run_session(session_id)
        return self.db.get_session(session_id)

    async def run_session(self, session_id: str) -> dict:
        if session_id in self._running:
            return self._require(session_id)
        self._running.add(session_id)
        try:
            if self._paused.get(session_id):
                self._accrue(session_id)
                self.db.update_session(session_id, status="paused", active_agent=None)
                return self._require(session_id)
            if self._stop.get(session_id):
                self._accrue(session_id)
                self.db.update_session(
                    session_id,
                    status="interrupted",
                    active_agent=None,
                    finished_at=utcnow(),
                )
                return self._require(session_id)
            await self._loop(session_id)
        except Exception as exc:
            message = redact(str(exc)) or "Erreur interne de l'orchestrateur."
            self._fail_technical(session_id, None, message, status="failed")
        finally:
            self._running.discard(session_id)
            self._accrue(session_id)
            self._notify(session_id, "status")
        return self._require(session_id)

    async def _loop(self, session_id: str) -> None:
        self._prepare(session_id)
        if self._stop.get(session_id):
            self.db.update_session(session_id, status="interrupted", active_agent=None, finished_at=utcnow())
            return
        session = self.db.get_session(session_id)
        if session["mode"] == "live" and not await self._live_ready(session):
            return
        while True:
            if self._stop.get(session_id):
                self.db.update_session(
                    session_id,
                    status="interrupted",
                    active_agent=None,
                    finished_at=utcnow(),
                )
                self._agent_state(session_id, "orchestrator", "finished", "interrompue")
                return
            if self._paused.get(session_id):
                self._accrue(session_id)
                self.db.update_session(session_id, status="paused", active_agent=None)
                self._agent_state(session_id, "orchestrator", "waiting", "en pause")
                return
            session = self.db.get_session(session_id)
            if session["status"] in TERMINAL:
                return
            if session["next_action"] == "done":
                return
            if session["next_action"] != "verify" and self._elapsed(session) > session["max_duration_seconds"]:
                self.db.update_session(session_id, outcome_reason=session["outcome_reason"] or "duration", next_action="verify")
                session = self.db.get_session(session_id)
            if session["next_action"] in AGENT_ACTIONS and session["turn"] >= session["max_turns"]:
                self.db.update_session(
                    session_id,
                    outcome_reason=session["outcome_reason"] or "max_turns",
                    next_action="verify",
                )
                session = self.db.get_session(session_id)
            self.db.update_session(session_id, status="running", phase=session["next_action"])
            self._begin_slice(session_id)
            action = self.db.get_session(session_id)["next_action"]
            if action == "verify":
                self._verify(session_id)
                return
            if action == "test":
                await self._run_tests(session_id)
                continue
            outcome = await self._agent_turn(session_id, action)
            if outcome == "stop":
                return

    def _prepare(self, session_id: str) -> None:
        session = self.db.get_session(session_id)
        if not self.db.list_messages(session_id):
            self.db.insert_message(
                {
                    "session_id": session_id,
                    "sender": "user",
                    "recipient": "orchestrator",
                    "turn": 0,
                    "type": "TASK",
                    "content": session["task_text"],
                    "subject": session["title"],
                    "references": [],
                    "delivery_state": "persisted",
                }
            )
            root = self.db.insert_task(
                {
                    "session_id": session_id,
                    "title": session["title"],
                    "kind": "root",
                    "status": "running",
                }
            )
            for kind, title in (
                ("dialogue", "Dialogue"),
                ("implementation", "Implémentation"),
                ("tests", "Tests"),
                ("report", "Bilan"),
            ):
                self.db.insert_task(
                    {
                        "session_id": session_id,
                        "title": title,
                        "kind": kind,
                        "status": "pending",
                        "parent_id": root["id"],
                    }
                )
        if not session.get("workspace_path"):
            path = self.workspaces.create(
                session_id,
                session.get("external_repo"),
                bool(session.get("confirm_external")),
            )
            self.db.insert_git_snapshot(session_id, "baseline", self.workspaces.status(path))
            self.db.update_session(session_id, workspace_path=str(path))
        self._notify(session_id, "message")

    async def _live_ready(self, session: dict) -> bool:
        adapters = self._adapters_for(session)
        problems: list[str] = []
        needed = {session["proposer"], session["critic"], session["implementer"]}
        for name in sorted(needed):
            adapter = adapters.get(name)
            if adapter is None:
                problems.append(f"L'adaptateur {name} est absent.")
                continue
            health = await adapter.health()
            if not health.available or health.authenticated is False:
                self._agent_state(session["id"], name, "error", health.detail)
                problems.append(health.detail)
            else:
                self._agent_state(session["id"], name, "available", health.detail)
        if problems:
            self._fail_technical(session["id"], None, " ".join(problems), status="failed")
            return False
        return True

    async def _agent_turn(self, session_id: str, action: str) -> str:
        session = self.db.get_session(session_id)
        speaker = self._speaker(session, action)
        allowed = PHASE_TYPES[action]
        self._set_task_status(session_id, self._task_kind(action), "running")
        self._agent_state(session_id, speaker, "running", action)
        self._agent_state(session_id, _other(session, speaker), "waiting", "en attente")
        self.db.update_session(session_id, active_agent=speaker, phase=action)
        for message in self.db.list_messages(session_id):
            if message["delivery_state"] == "persisted":
                self.db.mark_transmitted(message["id"])
        for attempt in (1, 2):
            if self._stop.get(session_id):
                self.db.update_session(session_id, status="interrupted", active_agent=None, finished_at=utcnow())
                return "stop"
            request = self._request(session_id, action, speaker, allowed, attempt)
            result = await self._invoke(session_id, speaker, request)
            if self._stop.get(session_id) or result.cancelled:
                self._agent_state(session_id, speaker, "finished", "interrompue")
                self.db.update_session(session_id, status="interrupted", active_agent=None, finished_at=utcnow())
                self._notify(session_id, "status")
                return "stop"
            if result.timed_out:
                self._incident(session_id, speaker, "Délai dépassé pour l'agent.")
                return "stop"
            if not result.ok:
                self._incident(session_id, speaker, result.error or "Échec de lancement de l'agent.")
                return "stop"
            if result.session_id:
                column = CLI_SESSION_COLUMNS.get(speaker)
                if column:
                    self.db.update_session(session_id, **{column: result.session_id})
            payload = parse_payload(result.text)
            if payload_is_valid(payload, allowed):
                self._record_agent_message(session_id, speaker, action, payload, result.text)
                follow = self._after_message(session_id, action, payload)
                self._agent_state(session_id, speaker, "waiting", "réponse enregistrée")
                self.db.update_session(session_id, active_agent=None)
                self._notify(session_id, "message")
                return follow
            self.db.update_session(
                session_id,
                invalid_retries=self.db.get_session(session_id)["invalid_retries"] + 1,
            )
            self._message(
                session_id,
                sender="orchestrator",
                recipient=speaker,
                type_="ERROR",
                content="La réponse de l'agent ne respecte pas le protocole. Un nouvel essai est demandé.",
                turn=self.db.get_session(session_id)["turn"],
            )
        self._incident(session_id, speaker, "Réponse invalide après un nouvel essai.")
        return "stop"

    def _record_agent_message(self, session_id: str, speaker: str, action: str, payload: dict, raw: str) -> None:
        session = self.db.get_session(session_id)
        turn = session["turn"] + 1
        prior = self.db.list_messages(session_id)
        references = [prior[-1]["id"]] if prior else []
        for item in payload.get("references") or []:
            if isinstance(item, str) and item not in references and self.db.get_message(item):
                references.append(item)
        issues = blocking_issues(payload)
        meta = {
            "agreement": agreement_flag(payload),
            "blocking_issues": issues,
        }
        content = str(payload.get("content") or "").strip()
        previous = [item for item in prior if item["sender"] == speaker and item["type"] != "ERROR"]
        loop = False
        if previous:
            last = previous[-1]
            if similarity(last["content"], content) >= session["similarity_threshold"]:
                loop = True
            elif opinion_flip(
                {"content": last["content"], "agreement": (last.get("meta") or {}).get("agreement")},
                {"content": content, "agreement": agreement_flag(payload)},
                session["similarity_threshold"],
            ):
                loop = True
        message_type = payload["message_type"]
        if message_type == "AGREEMENT" and not issues:
            self.db.resolve_open_decisions(session_id)
            self.db.insert_decision(session_id, content[:500], True)
        elif message_type == "DISAGREEMENT":
            self.db.insert_decision(session_id, content[:500], False)
        self.db.update_session(
            session_id,
            turn=turn,
            blocking_issues=json.dumps(issues, ensure_ascii=False),
        )
        self._message(
            session_id,
            sender=speaker,
            recipient=self._recipient(session, action, speaker),
            type_=message_type,
            content=content,
            turn=turn,
            references=references,
            raw=redact(raw)[:20000],
            meta=meta,
        )
        if loop:
            self.db.update_session(session_id, outcome_reason="loop", next_action="verify")
            return
        if self._allow_files(session, action):
            self._apply_files(session_id, payload)
        self._advance(session_id, action, payload)

    def _advance(self, session_id: str, action: str, payload: dict) -> None:
        session = self.db.get_session(session_id)
        message_type = payload["message_type"]
        issues = blocking_issues(payload)
        unresolved = sum(1 for item in self.db.list_decisions(session_id) if not item["resolved"])
        nxt = "verify"
        if action == "analyze":
            nxt = "critique"
        elif action == "critique":
            if message_type == "AGREEMENT" and not issues:
                nxt = "implement"
            elif message_type == "DISAGREEMENT" and unresolved >= 2:
                self.db.update_session(session_id, outcome_reason="disagreement")
                nxt = "verify"
            else:
                nxt = "respond"
        elif action == "respond":
            nxt = "revise"
        elif action == "revise":
            if message_type == "DISAGREEMENT" or issues:
                if message_type == "DISAGREEMENT" and unresolved >= 2:
                    self.db.update_session(session_id, outcome_reason="disagreement")
                    nxt = "verify"
                else:
                    nxt = "respond"
            else:
                nxt = "implement"
        elif action == "implement":
            nxt = "review"
            self._set_task_status(session_id, "dialogue", "done")
            self._set_task_status(session_id, "implementation", "done")
        elif action == "review":
            if (issues or message_type == "DISAGREEMENT") and session["corrections_used"] < session["max_corrections"]:
                self.db.update_session(session_id, corrections_used=session["corrections_used"] + 1)
                nxt = "correct"
            else:
                nxt = "test"
        elif action == "correct":
            nxt = "test"
            self._set_task_status(session_id, "implementation", "done")
        if not session["allow_implementation"] and nxt in {"implement", "review", "test", "correct"}:
            nxt = "verify"
        self.db.update_session(session_id, next_action=nxt, phase=nxt)

    def _after_message(self, session_id: str, action: str, payload: dict) -> str:
        del action, payload
        session = self.db.get_session(session_id)
        if session["status"] == "error":
            return "stop"
        return "continue"

    def _apply_files(self, session_id: str, payload: dict) -> None:
        session = self.db.get_session(session_id)
        root = Path(session["workspace_path"])
        written = write_declared_files(root, payload.get("files") or [])
        if written:
            self.workspaces.commit_all(root, f"duo: modification {session_id[:8]}")
            self.db.insert_git_snapshot(session_id, "write", "\n".join(written))

    async def _run_tests(self, session_id: str) -> None:
        import asyncio
        import subprocess

        session = self.db.get_session(session_id)
        root = Path(session["workspace_path"])
        self._set_task_status(session_id, "tests", "running")
        self._agent_state(session_id, "orchestrator", "running", "tests")
        args = [
            sys.executable,
            "-m",
            "pytest",
            "-q",
            "-p",
            "no:cacheprovider",
            "--noconftest",
            "--confcutdir",
            str(root),
            "--rootdir",
            str(root),
            str(root),
        ]
        env = os.environ.copy()
        for key in list(env):
            if key.startswith("PYTEST") or key.startswith("_PYTEST"):
                env.pop(key, None)
        env["PYTEST_DISABLE_PLUGIN_AUTOLOAD"] = "1"
        completed = await asyncio.to_thread(
            subprocess.run,
            args,
            cwd=str(root),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
            env=env,
        )
        stdout = redact(completed.stdout)[:8000]
        stderr = redact(completed.stderr)[:4000]
        command = "python -m pytest -q"
        self.db.insert_test_result(session_id, command, completed.returncode, stdout, stderr)
        status = "réussi" if completed.returncode == 0 else "en échec"
        self._message(
            session_id,
            sender="orchestrator",
            recipient="user",
            type_="TEST_REPORT",
            content=f"pytest s'est terminé avec le code {completed.returncode} ({status}).\n{stdout}",
            turn=session["turn"],
        )
        self._set_task_status(session_id, "tests", "done" if completed.returncode == 0 else "failed")
        if completed.returncode != 0 and session["corrections_used"] < session["max_corrections"]:
            self.db.update_session(
                session_id,
                corrections_used=session["corrections_used"] + 1,
                next_action="correct",
                phase="correct",
            )
        else:
            self.db.update_session(session_id, next_action="verify", phase="verify")
        self._notify(session_id, "tests")

    def _verify(self, session_id: str) -> None:
        for message in self.db.list_messages(session_id):
            if message["delivery_state"] == "persisted":
                self.db.mark_transmitted(message["id"])
        session = self.db.get_session(session_id)
        messages = self.db.list_messages(session_id)
        decisions = self.db.list_decisions(session_id)
        tests = self.db.list_test_results(session_id)
        issues = json.loads(session["blocking_issues"] or "[]")
        agreements = [item for item in messages if item["type"] == "AGREEMENT"]
        open_disagreements = [item for item in decisions if not item["resolved"]]
        limitations: list[str] = []
        reasons: list[str] = []
        if session["outcome_reason"]:
            reasons.append(session["outcome_reason"])
        if not agreements:
            reasons.append("aucun accord explicite")
        if open_disagreements:
            reasons.append("désaccords ouverts")
        if issues:
            reasons.append("points bloquants")
        last_test = tests[-1] if tests else None
        if session["allow_implementation"]:
            if last_test is None:
                reasons.append("aucun test exécuté")
            elif last_test["exit_code"] != 0:
                reasons.append("tests en échec")
        else:
            limitations.append(
                "L'implémentation n'était pas autorisée : aucun fichier d'agent n'a été écrit et pytest n'a pas été lancé."
            )
        if session["outcome_reason"] in LIMIT_REASONS or open_disagreements:
            outcome = "needs_arbitration"
        elif reasons:
            outcome = "failed"
        else:
            outcome = "succeeded"
        root = Path(session["workspace_path"]) if session.get("workspace_path") else None
        files = self.workspaces.changed_files(root) if root and root.exists() else []
        if root and root.exists():
            self.db.insert_git_snapshot(session_id, "final_diff", self.workspaces.diff(root)[:20000])
        solution = ""
        for item in reversed(messages):
            if item["type"] in {"IMPLEMENTATION_REPORT", "REVISION", "PROPOSAL"}:
                solution = item["content"]
                break
        report = {
            "problem": session["task_text"],
            "solution": solution,
            "outcome": outcome,
            "reasons": reasons,
            "arguments": [
                {"sender": item["sender"], "type": item["type"], "content": item["content"]}
                for item in messages
                if item["type"] in {"PROPOSAL", "CRITIQUE", "RESPONSE", "REVISION", "AGREEMENT", "DISAGREEMENT"}
            ],
            "disagreements_resolved": [item["summary"] for item in decisions if item["resolved"]],
            "disagreements_open": [item["summary"] for item in open_disagreements],
            "files_modified": files,
            "tests": [
                {"command": item["command"], "exit_code": item["exit_code"], "stdout": item["stdout"]}
                for item in tests
            ],
            "limitations": limitations,
            "remaining_actions": self._remaining(outcome, reasons),
        }
        body = json.dumps(report, ensure_ascii=False, indent=2)
        self._message(
            session_id,
            sender="orchestrator",
            recipient="user",
            type_="FINAL_REPORT",
            content=self._report_text(report),
            turn=session["turn"],
            meta=report,
        )
        self._set_task_status(session_id, "root", "done" if outcome == "succeeded" else outcome)
        self._set_task_status(session_id, "report", "done")
        self._set_task_status(session_id, "dialogue", "done")
        for agent in ("cursor", "copilot", "orchestrator"):
            self._agent_state(session_id, agent, "finished", outcome)
        self.db.update_session(
            session_id,
            status=outcome,
            phase="done",
            next_action="done",
            active_agent=None,
            finished_at=utcnow(),
            final_report=body,
            outcome_reason=session["outcome_reason"] or (", ".join(reasons) if outcome != "succeeded" else ""),
        )
        self._notify(session_id, "done")

    def _remaining(self, outcome: str, reasons: list[str]) -> list[str]:
        if outcome == "succeeded":
            return []
        if outcome == "needs_arbitration":
            return ["Arbitrer les points encore ouverts avant une nouvelle session."]
        if "tests en échec" in reasons:
            return ["Corriger le code du bac à sable et relancer pytest."]
        return ["Relire le bilan et relancer une session si la tâche reste ouverte."]

    def _report_text(self, report: dict) -> str:
        lines = [
            f"Résultat : {report['outcome']}.",
            f"Problème : {report['problem']}",
            f"Solution : {report['solution'] or 'aucune formulation retenue'}",
        ]
        if report["disagreements_open"]:
            lines.append("Désaccords ouverts : " + " | ".join(report["disagreements_open"]))
        if report["disagreements_resolved"]:
            lines.append("Accords : " + " | ".join(report["disagreements_resolved"]))
        if report["files_modified"]:
            lines.append("Fichiers : " + ", ".join(report["files_modified"]))
        if report["tests"]:
            last = report["tests"][-1]
            lines.append(f"Tests : code {last['exit_code']}.")
        if report["limitations"]:
            lines.append("Limites : " + " ".join(report["limitations"]))
        if report["remaining_actions"]:
            lines.append("À faire : " + " ".join(report["remaining_actions"]))
        return "\n".join(lines)

    async def _invoke(self, session_id: str, speaker: str, request: AgentRequest) -> AgentResult:
        adapter = self._adapters_for(self.db.get_session(session_id))[speaker]
        token = CancelToken()
        self._tokens[session_id] = token
        task_name = f"{speaker}-{session_id}"
        del task_name
        import asyncio

        started = time.monotonic()
        job = asyncio.create_task(adapter.run(request, token))
        try:
            while not job.done():
                if self._stop.get(session_id):
                    token.trigger("stop")
                elif time.monotonic() - started >= request.timeout:
                    token.trigger("timeout")
                if token.cancelled:
                    try:
                        return await asyncio.wait_for(asyncio.shield(job), timeout=2)
                    except TimeoutError:
                        job.cancel()
                        if token.reason == "stop":
                            return AgentResult(ok=False, cancelled=True, error="Interrompu")
                        return AgentResult(ok=False, timed_out=True, error="Délai dépassé")
                await asyncio.sleep(0.02)
            return await job
        finally:
            self._tokens.pop(session_id, None)

    def _request(self, session_id: str, action: str, speaker: str, allowed: frozenset[str], attempt: int) -> AgentRequest:
        session = self.db.get_session(session_id)
        history = [
            {
                "id": item["id"],
                "sender": item["sender"],
                "type": item["type"],
                "turn": item["turn"],
                "content": item["content"][:4000],
            }
            for item in self.db.list_messages(session_id)[-20:]
        ]
        root = Path(session["workspace_path"])
        diff = ""
        if action in {"review", "correct", "implement"} and root.exists():
            diff = self.workspaces.diff(root)[:12000]
        column = CLI_SESSION_COLUMNS.get(speaker)
        resume = session.get(column) if column else None
        roles = {
            "analyze": "proposer une solution argumentée",
            "critique": "critiquer la proposition de façon indépendante",
            "respond": "répondre à la critique avec des arguments nouveaux",
            "revise": "réviser ta position, ou maintenir un désaccord explicite",
            "implement": "implémenter uniquement dans le bac à sable",
            "review": "relire le diff sans présumer que l'autre agent a raison",
            "correct": "corriger les défauts signalés par les tests ou la revue",
        }
        return AgentRequest(
            session_id=session_id,
            agent=speaker,
            role=roles[action],
            expected_types=allowed,
            task=session["task_text"],
            subject=session["title"],
            history=history,
            turn=session["turn"] + 1,
            workspace=str(root),
            allow_write=self._allow_files(session, action),
            timeout=float(session["agent_timeout_seconds"]),
            resume_id=resume,
            phase=action,
            diff=diff,
            scenario=session["scenario"],
            attempt=attempt,
            incident_count=session["incident_count"],
            proposer=session["proposer"],
        )

    def _allow_files(self, session: dict, action: str) -> bool:
        return bool(session["allow_implementation"]) and action in {"implement", "correct"}

    def _speaker(self, session: dict, action: str) -> str:
        if action in {"analyze", "respond"}:
            return session["proposer"]
        if action in {"critique", "revise"}:
            return session["critic"]
        if action in {"implement", "correct"}:
            return session["implementer"]
        if action == "review":
            return _other(session, session["implementer"])
        raise ValueError(action)

    def _recipient(self, session: dict, action: str, speaker: str) -> str:
        if action in {"implement", "correct"}:
            return _other(session, session["implementer"])
        if action == "review":
            return session["implementer"]
        return _other(session, speaker)

    def _task_kind(self, action: str) -> str:
        if action in {"implement", "correct"}:
            return "implementation"
        if action == "review":
            return "dialogue"
        return "dialogue"

    def _set_task_status(self, session_id: str, kind: str, status: str) -> None:
        for task in self.db.list_tasks(session_id):
            if task["kind"] == kind and task["status"] != status:
                self.db.update_task(task["id"], status)

    def _agent_state(self, session_id: str, agent: str, status: str, detail: str) -> None:
        self.db.upsert_agent_state(session_id, agent, status, redact(detail)[:500])

    def _message(
        self,
        session_id: str,
        sender: str,
        recipient: str,
        type_: str,
        content: str,
        turn: int,
        references: list | None = None,
        raw: str | None = None,
        meta: dict | None = None,
    ) -> dict:
        session = self.db.get_session(session_id)
        return self.db.insert_message(
            {
                "session_id": session_id,
                "sender": sender,
                "recipient": recipient,
                "turn": turn,
                "type": type_,
                "content": redact(content),
                "subject": session["title"],
                "references": references or [],
                "delivery_state": "persisted",
                "raw_response": raw,
                "meta": meta or {},
            }
        )

    def _incident(self, session_id: str, agent: str, message: str) -> None:
        clean = redact(message)[:2000]
        session = self.db.get_session(session_id)
        self.db.insert_error(session_id, clean, agent)
        self._message(
            session_id,
            sender="orchestrator",
            recipient=agent or "user",
            type_="ERROR",
            content=clean,
            turn=session["turn"],
        )
        self._agent_state(session_id, agent or "orchestrator", "error", clean)
        self.db.update_session(
            session_id,
            status="error",
            active_agent=None,
            last_error=clean,
            finished_at=utcnow(),
            incident_count=session["incident_count"] + 1,
        )
        self._notify(session_id, "error")

    def _fail_technical(self, session_id: str, agent: str | None, message: str, status: str) -> None:
        clean = redact(message)[:2000]
        session = self.db.get_session(session_id)
        if session is None:
            return
        self.db.insert_error(session_id, clean, agent)
        if self.db.list_messages(session_id) or True:
            try:
                self._message(
                    session_id,
                    sender="orchestrator",
                    recipient=agent or "user",
                    type_="ERROR",
                    content=clean,
                    turn=session["turn"],
                )
            except Exception:
                pass
        self.db.update_session(
            session_id,
            status=status,
            active_agent=None,
            last_error=clean,
            finished_at=utcnow(),
            phase="done",
            next_action="done",
            final_report=json.dumps(
                {
                    "problem": session["task_text"],
                    "solution": "",
                    "outcome": status,
                    "reasons": [clean],
                    "limitations": [clean],
                    "remaining_actions": ["Rétablir l'agent indisponible ou utiliser le mode simulé."],
                    "arguments": [],
                    "disagreements_resolved": [],
                    "disagreements_open": [],
                    "files_modified": [],
                    "tests": [],
                },
                ensure_ascii=False,
            ),
        )
        self._notify(session_id, "error")

    def _require(self, session_id: str) -> dict:
        session = self.db.get_session(session_id)
        if session is None:
            raise KeyError(session_id)
        return session
