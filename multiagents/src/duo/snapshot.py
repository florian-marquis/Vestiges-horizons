"""Vues publiques des sessions, sans aucun secret."""

from __future__ import annotations

import json
from datetime import datetime, timezone

from duo.db import Database


def _elapsed(session: dict) -> float:
    elapsed = float(session.get("elapsed_seconds") or 0)
    started = session.get("slice_started_at")
    if started:
        elapsed += (datetime.now(timezone.utc) - datetime.fromisoformat(started)).total_seconds()
    return max(0.0, elapsed)


def session_view(session: dict) -> dict:
    elapsed = _elapsed(session)
    remaining_turns = max(0, int(session["max_turns"]) - int(session["turn"]))
    remaining_seconds = max(0, int(session["max_duration_seconds"] - elapsed))
    report = None
    if session.get("final_report"):
        try:
            report = json.loads(session["final_report"])
        except json.JSONDecodeError:
            report = {"solution": session["final_report"]}
    return {
        "id": session["id"],
        "title": session["title"],
        "task": session["task_text"],
        "status": session["status"],
        "phase": session["phase"],
        "next_action": session["next_action"],
        "turn": session["turn"],
        "max_turns": session["max_turns"],
        "turns_remaining": remaining_turns,
        "max_duration_seconds": session["max_duration_seconds"],
        "elapsed_seconds": round(elapsed, 1),
        "seconds_remaining": remaining_seconds,
        "agent_timeout_seconds": session["agent_timeout_seconds"],
        "similarity_threshold": session["similarity_threshold"],
        "max_corrections": session["max_corrections"],
        "corrections_used": session["corrections_used"],
        "proposer": session["proposer"],
        "critic": session["critic"],
        "implementer": session["implementer"],
        "mode": session["mode"],
        "scenario": session["scenario"],
        "allow_implementation": bool(session["allow_implementation"]),
        "workspace_path": session.get("workspace_path"),
        "external_repo": session.get("external_repo"),
        "active_agent": session.get("active_agent"),
        "created_at": session["created_at"],
        "started_at": session.get("started_at"),
        "finished_at": session.get("finished_at"),
        "last_error": session.get("last_error") or "",
        "outcome_reason": session.get("outcome_reason") or "",
        "blocking_issues": json.loads(session.get("blocking_issues") or "[]"),
        "final_report": report,
    }


def snapshot(db: Database, session_id: str) -> dict:
    session = db.get_session(session_id)
    if session is None:
        raise KeyError(session_id)
    return {
        "session": session_view(session),
        "messages": db.list_messages(session_id),
        "tasks": db.list_tasks(session_id),
        "agents": db.list_agent_states(session_id),
        "decisions": db.list_decisions(session_id),
        "tests": db.list_test_results(session_id),
        "errors": db.list_errors(session_id),
        "status_changes": db.list_status_changes(session_id),
        "git": db.list_git_snapshots(session_id),
    }


def to_markdown(data: dict) -> str:
    session = data["session"]
    lines = [
        f"# {session['title']}",
        "",
        f"- Statut : {session['status']}",
        f"- Mode : {session['mode']}",
        f"- Tour : {session['turn']} / {session['max_turns']}",
        f"- Proposant : {session['proposer']}",
        f"- Critique : {session['critic']}",
        f"- Implémenteur : {session['implementer']}",
        "",
        "## Tâche",
        "",
        session["task"],
        "",
        "## Messages",
        "",
    ]
    for message in data["messages"]:
        lines.extend(
            [
                f"### Tour {message['turn']} — {message['sender']} → {message['recipient']} ({message['type']})",
                "",
                message["content"],
                "",
            ]
        )
    report = session.get("final_report") or {}
    lines.extend(["## Bilan", ""])
    if report:
        lines.append(f"Résultat : {report.get('outcome', '')}")
        lines.append("")
        lines.append(report.get("solution") or "")
        lines.append("")
        if report.get("files_modified"):
            lines.append("Fichiers : " + ", ".join(report["files_modified"]))
        if report.get("disagreements_open"):
            lines.append("Désaccords ouverts :")
            lines.extend(f"- {item}" for item in report["disagreements_open"])
        if report.get("limitations"):
            lines.append("Limites :")
            lines.extend(f"- {item}" for item in report["limitations"])
    else:
        lines.append("Bilan non publié.")
    lines.append("")
    return "\n".join(lines)
