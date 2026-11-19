# Duo

Deux façons de faire collaborer Cursor et VS Code sur cette machine.

**Salon (le mode à utiliser).** VS Code reste dans une fenêtre, Cursor dans une autre. Vous ouvrez un salon, vous copiez la consigne dans chaque agent, et vous leur donnez la tâche. Chaque agent appelle une attente : l'appel (MCP `attendre` ou `python -m duo.wait_room`) ne revient que sur un message visible ou la clôture. Il répond, puis attend de nouveau. L'application ne lance pas les agents et ne tape pas dans les fenêtres.

**Orchestrateur.** L'application appelle elle-même les CLI (`agent`, `copilot`, `vibe`, `agy`), tour par tour. Utile sans les fenêtres, à condition que les CLI choisies soient authentifiées. Le mode simulé fonctionne sans réseau.

**Comptes.** Page `/comptes` : connexion OAuth via les CLI officielles uniquement. Les jetons restent dans le coffre OS / stores CLI ; Duo n'en copie aucun. Quotas restants en best-effort (sinon `unavailable`).

Le mode simulé est opérationnel sans réseau. Les CLI réelles sont branchées, mais sur cette machine elles ne sont pas authentifiées : une session « CLI réelles » s'arrête avec la marche à suivre, elle ne simule pas un succès.

## Documentation Pilote

- [docs/PILOTE.md](docs/PILOTE.md) — utilisation du salon pour l'agent Pilote
- [docs/CODE.md](docs/CODE.md) — carte du code
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — schémas salon / orchestrateur

## Installation

Prérequis : Python 3.11 ou plus (testé avec 3.14), Git, et les CLI seulement si vous utilisez le mode réel.

```bat
cd /d "chemin\vers\multiagents"
start.bat
```

`start.bat` crée `.venv`, installe les dépendances et ouvre l'application sur [http://127.0.0.1:8765](http://127.0.0.1:8765).

Équivalent manuel :

```bat
py -3 -m venv .venv
.venv\Scripts\python -m pip install -r requirements.txt
set PYTHONPATH=src
.venv\Scripts\python -m duo
```

Copiez `config.example.toml` vers `duo.toml` pour changer le port, les limites ou les racines de fichiers autorisées. L'écoute n'accepte que `127.0.0.1`, `localhost` ou `::1`.

## Salon

1. Lancez `start.bat`, puis ouvrez [http://127.0.0.1:8765](http://127.0.0.1:8765).
2. Choisissez le **nombre d'agents** (1 à 5) en plus du Pilote, et la fenêtre de chacun (VS Code ou Cursor).
3. Décrivez la tâche et ouvrez le salon.
4. Collez chaque consigne dans la fenêtre correspondante (Pilote, Agent 1, Agent 2…).
5. Le Pilote parle en premier et assigne avec `@agent1`, `@agent2`, etc. Seul le Pilote met à jour `etat.md` (lecture seule pour les agents) ; les agents n'écrivent le code que s'ils sont désignés comme rédacteur actuel.
6. L'attente dure jusqu'à 90 s ; si elle expire sans message, l'agent doit la rappeler immédiatement (presque pas de tokens).
7. Un message `STOP` demande un bilan. Clôturer empêche ensuite toute nouvelle publication.

Si un agent termine son tour trop tôt : « Continue. Rappelle l'attente du salon. »

Les outils MCP `lire_salon`, `dire` et `attendre` sont déclarés pour ce dépôt. L'identité vient du jeton de la consigne. `attendre` (et `python -m duo.wait_room` sans MCP) enchaîne les polls jusqu'à un message ou la clôture.

## Orchestrateur

1. Saisissez une tâche et laissez le mode **Simulé** pour un dialogue automatique.
2. Les messages de Cursor, Copilot et de l'orchestrateur se distinguent par leur couleur.
3. Le panneau de droite montre le tour, l'agent actif, le temps restant, les tâches, les tests et le bilan.
4. Pause, reprise, arrêt immédiat et relance après erreur sont disponibles.
5. L'historique reste dans `data/duo.sqlite`. Les exports JSON et Markdown sont dans la barre d'outils.

Les profils simulés (collaboration, désaccord, boucle, réponse invalide, panne, délai, tests en échec) servent à observer l'orchestrateur sans quota.

En mode **CLI réelles**, les providers choisis (proposant / critique / implémenteur) doivent être installés et authentifiés. L'implémentation, si elle est autorisée, n'écrit que dans `data/sandboxes/<session>` ou dans un worktree Git confirmé explicitement. Les agents n'écrivent pas en même temps.

## Authentification (OAuth CLI)

Préférez la page [http://127.0.0.1:8765/comptes](http://127.0.0.1:8765/comptes) : Duo lance la CLI et vous guide. Équivalents manuels :

| Provider | Login CLI | Quotas |
| --- | --- | --- |
| Cursor | `agent login` | usage-summary si jeton CLI local lisible, sinon unavailable |
| Copilot | `copilot login` | `gh api /copilot_internal/user` si `gh` auth, sinon unavailable |
| Mistral Vibe | `vibe --setup` | souvent unavailable → console Mistral |
| Antigravity | `agy` (OAuth navigateur) | souvent unavailable → `/usage` dans agy |

Duo ne stocke aucun secret. Métadonnées non secrètes seulement (`dernier contrôle`, statut).

## Tests

```bat
set PYTHONPATH=src
.venv\Scripts\python -m pytest
```

La dernière exécution locale a donné 47 tests réussis. La suite couvre la persistance, la reprise, les tours automatiques, les réponses invalides, les pannes, les délais, l'arrêt, les boucles, le plafond de tours, les désaccords, la convergence, la sécurité des commandes, les providers OAuth/quotas et l'absence d'un agent. Les tests d'orchestration utilisent les adaptateurs simulés.

## Limites

- Le CLI `cursor` de l'éditeur ouvre l'IDE. Il ne lance pas l'agent. Seul le CLI `agent` est piloté.
- Aucune frappe simulée dans les fenêtres.
- Le SDK Python `cursor-sdk` n'est pas branché : un agent local pourrait écrire en dehors du contrôle d'outils retenu.
- Le coût n'est pas mesuré : les CLI ne fournissent pas un budget exploitable ici. La limite de coût reste inactive.
- Duo n'exécute pas les commandes proposées par les agents. Il lance seulement `pytest` dans le bac à sable, avec l'interpréteur courant.
- `reset --hard`, `clean` et `push --force` sont refusés.
- Un accord des agents ne clôt une tâche avec implémentation que si pytest a réellement réussi.

## Exécutable Windows

Le lanceur `start.bat` est le chemin fiable. Un exécutable PyInstaller est possible, mais il n'est pas généré par défaut et n'a pas été vérifié ici :

```bat
.venv\Scripts\python -m pip install pyinstaller
.venv\Scripts\pyinstaller --onefile --name duo --paths src --add-data "src\duo\static;duo\static" src\duo\__main__.py
```

L'exécutable aurait encore besoin de Git, et des CLI Node, sur la machine.

## Dépannage

| Symptôme | Action |
| --- | --- |
| La page ne s'ouvre pas | Vérifiez que rien d'autre n'écoute le port 8765, ou changez `port` dans `duo.toml`. |
| Cursor « non authentifié » | `agent status`, puis `agent login`. Rouvrez Duo ensuite. |
| Copilot « pas authentifié » | `copilot login`. Un appel de test : `python -m duo --probe`. |
| Session réelle en échec immédiat | Normal tant que les deux CLI ne sont pas authentifiées. Utilisez le mode simulé. |
| Les tests du bac échouent | Le bilan indique le code pytest. Le dépôt utilisateur n'est pas modifié. |
| Reprise après fermeture | Rouvrez l'application et cliquez Reprendre sur une session en pause ou interrompue. |

`python -m duo --probe` interroge `agent status` et un prompt Copilot sans afficher de jeton. Un code de sortie non nul signifie qu'au moins une CLI n'est pas prête.
