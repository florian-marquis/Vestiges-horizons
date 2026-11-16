"""Protocole de messages entre agents."""

from __future__ import annotations

import json
import re
from difflib import SequenceMatcher

MESSAGE_TYPES = frozenset(
    {
        "TASK",
        "PROPOSAL",
        "CRITIQUE",
        "QUESTION",
        "RESPONSE",
        "REVISION",
        "IMPLEMENTATION_REPORT",
        "TEST_REPORT",
        "AGREEMENT",
        "DISAGREEMENT",
        "FINAL_REPORT",
        "ERROR",
    }
)

PHASE_TYPES = {
    "analyze": frozenset({"PROPOSAL"}),
    "critique": frozenset({"CRITIQUE", "QUESTION", "AGREEMENT", "DISAGREEMENT"}),
    "respond": frozenset({"RESPONSE", "REVISION", "QUESTION"}),
    "revise": frozenset(
        {"REVISION", "CRITIQUE", "AGREEMENT", "DISAGREEMENT", "QUESTION"}
    ),
    "implement": frozenset({"IMPLEMENTATION_REPORT"}),
    "review": frozenset({"CRITIQUE", "AGREEMENT", "DISAGREEMENT"}),
    "correct": frozenset({"IMPLEMENTATION_REPORT", "REVISION"}),
}


def dumps_payload(payload: dict) -> str:
    body = json.dumps(payload, ensure_ascii=False, indent=2)
    return f"```json\n{body}\n```"


def _balanced_objects(text: str) -> list[str]:
    found: list[str] = []
    start = 0
    while True:
        opening = text.find("{", start)
        if opening < 0:
            break
        depth = 0
        in_string = False
        escaped = False
        closed_at = None
        for index in range(opening, len(text)):
            char = text[index]
            if in_string:
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    in_string = False
                continue
            if char == '"':
                in_string = True
            elif char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
                if depth == 0:
                    closed_at = index
                    break
        if closed_at is None:
            break
        found.append(text[opening : closed_at + 1])
        start = closed_at + 1
    return found


def parse_payload(text: str) -> dict | None:
    if not text:
        return None
    chunks = re.findall(r"```json\s*([\s\S]*?)```", text, flags=re.IGNORECASE)
    candidates = []
    for chunk in chunks:
        candidates.extend(_balanced_objects(chunk))
    if not candidates:
        candidates = _balanced_objects(text)
    for raw in reversed(candidates):
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            continue
        if isinstance(data, dict) and data.get("message_type") in MESSAGE_TYPES:
            return data
    return None


def payload_is_valid(payload: dict | None, allowed: frozenset[str]) -> bool:
    if not payload:
        return False
    if payload.get("message_type") not in allowed:
        return False
    content = payload.get("content")
    return isinstance(content, str) and bool(content.strip())


def blocking_issues(payload: dict | None) -> list[str]:
    if not payload:
        return []
    raw = payload.get("blocking_issues") or []
    if not isinstance(raw, list):
        return []
    return [str(item).strip() for item in raw if str(item).strip()]


def agreement_flag(payload: dict | None) -> bool | None:
    if not payload or "agreement" not in payload:
        return None
    return bool(payload.get("agreement"))


def normalize_text(text: str) -> str:
    return re.sub(r"\s+", " ", (text or "").strip().lower())


def similarity(left: str, right: str) -> float:
    a = normalize_text(left)
    b = normalize_text(right)
    if not a and not b:
        return 1.0
    return SequenceMatcher(None, a, b).ratio()


def opinion_flip(previous: dict | None, current: dict | None, threshold: float) -> bool:
    if not previous or not current:
        return False
    before = agreement_flag(previous)
    after = agreement_flag(current)
    if before is None or after is None or before == after:
        return False
    return similarity(str(previous.get("content", "")), str(current.get("content", ""))) >= threshold


def build_prompt(request) -> str:
    history_lines = []
    for item in request.history:
        history_lines.append(
            "\n".join(
                [
                    f"- id: {item['id']}",
                    f"  émetteur: {item['sender']}",
                    f"  type: {item['type']}",
                    f"  tour: {item['turn']}",
                    f"  contenu: {item['content']}",
                ]
            )
        )
    history = "\n".join(history_lines) if history_lines else "(aucun message précédent)"
    allowed = ", ".join(sorted(request.expected_types))
    diff = request.diff.strip() if request.diff else "(aucune différence Git)"
    return f"""Tu participes à une revue technique structurée. Réponds uniquement avec un bloc ```json.
N'utilise pas d'outil si une réponse textuelle suffit. N'invente pas de commande à exécuter par l'orchestrateur : il ignore les commandes proposées dans le texte.

Tâche globale :
{request.task}

Sujet : {request.subject}
Tour : {request.turn}
Phase : {request.phase}
Ton rôle : {request.role}
Types de message autorisés pour cette phase : {allowed}

Contraintes :
- Le champ message_type doit être l'un des types autorisés.
- Le champ content est rédigé en français, avec des arguments vérifiables.
- Le champ blocking_issues est une liste de points bloquants encore ouverts (liste vide si aucun).
- Le champ agreement est true seulement en cas d'accord motivé.
- Le champ references reprend les id des messages auxquels tu réponds.
- Si tu proposes des fichiers, utilise files: [{{"path": "relatif.py", "content": "..."}}] avec des chemins relatifs au bac à sable.

Historique transmis explicitement (ne te fie pas à une mémoire implicite) :
{history}

Différences Git observables :
{diff}

Schéma attendu :
{{
  "message_type": "{sorted(request.expected_types)[0]}",
  "content": "...",
  "blocking_issues": [],
  "agreement": false,
  "references": [],
  "files": []
}}
"""
