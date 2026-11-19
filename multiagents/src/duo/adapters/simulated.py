"""Agents simulés, déterministes et indépendants de tout service externe."""

from __future__ import annotations

from duo.adapters.base import AgentAdapter
from duo.models import AgentRequest, AgentResult, CancelToken, HealthStatus
from duo.protocol import dumps_payload

CALC_OK = '''def add(a: int, b: int) -> int:
    """Retourne la somme de deux entiers."""
    return a + b
'''

CALC_BAD = '''def add(a: int, b: int) -> int:
    """Retourne la somme de deux entiers."""
    return a - b
'''

TEST_CALC = '''import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from calc import add


def test_add():
    assert add(1, 2) == 3


def test_add_negatives():
    assert add(-1, 1) == 0
'''

LOOP_TEXT = "Proposition inchangée sans argument nouveau."


def _preferred(expected: frozenset[str]) -> str:
    order = (
        "PROPOSAL",
        "CRITIQUE",
        "RESPONSE",
        "REVISION",
        "IMPLEMENTATION_REPORT",
        "AGREEMENT",
        "DISAGREEMENT",
        "QUESTION",
    )
    for name in order:
        if name in expected:
            return name
    return "RESPONSE"


def _result(payload: dict) -> AgentResult:
    return AgentResult(ok=True, text=dumps_payload(payload), exit_code=0)


class SimulatedAdapter(AgentAdapter):
    def __init__(self, name: str):
        if name not in {"cursor", "copilot", "vibe", "antigravity"}:
            raise ValueError("Agent simulé inconnu.")
        self.name = name

    async def health(self) -> HealthStatus:
        return HealthStatus(
            agent=self.name,
            available=True,
            authenticated=True,
            detail="Adaptateur simulé local, sans appel réseau.",
            unsupported=["cout", "reprise_cli"],
        )

    async def run(self, request: AgentRequest, token: CancelToken) -> AgentResult:
        if token.cancelled:
            return self._cancelled(token)
        if request.scenario == "panne" and request.incident_count == 0 and request.agent == request.proposer:
            return AgentResult(
                ok=False,
                exit_code=127,
                error="Échec de lancement de l'agent simulé.",
            )
        if request.scenario == "lent" and request.incident_count == 0 and request.agent == request.proposer:
            await token.event.wait()
            return self._cancelled(token)
        if request.scenario == "invalide" and request.attempt == 1:
            return AgentResult(ok=True, text="Réponse sans bloc JSON exploitable.", exit_code=0)
        if request.scenario == "boucle":
            return _result(
                {
                    "message_type": _preferred(request.expected_types),
                    "content": LOOP_TEXT,
                    "blocking_issues": [],
                    "agreement": False,
                    "references": [],
                    "files": [],
                }
            )
        return _result(self._payload(request))

    def _cancelled(self, token: CancelToken) -> AgentResult:
        if token.reason == "timeout":
            return AgentResult(ok=False, timed_out=True, error="Délai dépassé")
        return AgentResult(ok=False, cancelled=True, error="Interrompu")

    def _payload(self, request: AgentRequest) -> dict:
        phase = request.phase
        if phase in {"implement", "correct"} or "IMPLEMENTATION_REPORT" in request.expected_types:
            bad = request.scenario == "tests_fail" and phase == "implement"
            return {
                "message_type": "IMPLEMENTATION_REPORT",
                "content": (
                    "J'ai écrit calc.py avec une soustraction, le test doit échouer."
                    if bad
                    else "J'ai ajouté calc.add et test_calc.py. Le contrat vérifie add(1, 2) == 3."
                ),
                "blocking_issues": [],
                "agreement": True,
                "references": [],
                "files": [
                    {"path": "calc.py", "content": CALC_BAD if bad else CALC_OK},
                    {"path": "test_calc.py", "content": TEST_CALC},
                ],
            }
        if phase == "review":
            return {
                "message_type": "AGREEMENT",
                "content": "La relecture du diff montre une fonction pure et un test automatisé. Je n'ai pas de point bloquant.",
                "blocking_issues": [],
                "agreement": True,
                "references": [],
                "files": [],
            }
        if request.scenario == "desaccord" and phase in {"critique", "revise"}:
            if phase == "critique":
                content = (
                    "Je suis en désaccord : la fonction pure ne conserve aucune trace des décisions. "
                    "Sans journal local, le résultat n'est pas vérifiable."
                )
            else:
                content = (
                    "Le désaccord reste entier, pour une raison différente : il n'y a toujours pas de reprise "
                    "après arrêt. Le journal évoqué au tour précédent ne règle pas ce point."
                )
            return {
                "message_type": "DISAGREEMENT",
                "content": content,
                "blocking_issues": ["désaccord non résolu"],
                "agreement": False,
                "references": [],
                "files": [],
            }
        if phase == "analyze":
            return {
                "message_type": "PROPOSAL",
                "content": (
                    "Je propose une fonction pure add(a, b) dans calc.py, couverte par test_calc.py. "
                    "Aucun des deux agents n'écrit en même temps : l'implémentation viendra ensuite, puis la revue."
                ),
                "blocking_issues": [],
                "agreement": False,
                "references": [],
                "files": [],
            }
        if phase == "critique":
            return {
                "message_type": "CRITIQUE",
                "content": (
                    "La proposition est cohérente, mais le contrat de test n'est pas encore fixé. "
                    "J'exige add(1, 2) == 3 et add(-1, 1) == 0 avant de considérer la tâche terminée."
                ),
                "blocking_issues": ["contrat de test non fixé"],
                "agreement": False,
                "references": [],
                "files": [],
            }
        if phase == "respond":
            return {
                "message_type": "RESPONSE",
                "content": (
                    "J'intègre le contrat demandé : test_calc.py vérifiera add(1, 2) == 3 et add(-1, 1) == 0. "
                    "La fonction reste une addition d'entiers, sans effet de bord."
                ),
                "blocking_issues": [],
                "agreement": False,
                "references": [],
                "files": [],
            }
        return {
            "message_type": "AGREEMENT",
            "content": (
                "Le contrat de test répond au point bloquant. J'accepte la solution : "
                "fonction pure, test automatisé, un seul écrivain à la fois."
            ),
            "blocking_issues": [],
            "agreement": True,
            "references": [],
            "files": [],
        }
