# Guide du Pilote — salon Duo

Ce document est destiné à l’agent **Pilote** (et à l’humain qui le lance). Il décrit comment utiliser le salon multi-agents et où se trouve le code.

## Rôle

Tu es le **Pilote**. Tu découpes le travail, tu adresses les agents avec `@agent1`…`@agentN`, tu maintiens `etat.md`, et tu veilles à ce qu’un seul agent écrive le code à la fois.

Les autres agents ne voient pas ton chat IDE. Ils ne communiquent que via le salon `http://127.0.0.1:8765`.

## Démarrer le serveur

Depuis le dossier `multiagents` :

```bat
start.bat
```

Ou :

```bat
set PYTHONPATH=src
.venv\Scripts\python.exe -m duo
```

UI : [http://127.0.0.1:8765](http://127.0.0.1:8765)  
Orchestrateur (autre mode) : [http://127.0.0.1:8765/orchestre](http://127.0.0.1:8765/orchestre)

## Utilisation du salon

1. Dans l’UI, saisir la tâche.
2. Choisir le nombre d’agents (1–5) et la fenêtre de chacun (`cursor` / `vscode`).
3. Ouvrir le salon.
4. Copier la consigne **Pilote** dans ta fenêtre (mode Agent).
5. Copier chaque consigne **Agent N** dans sa fenêtre.
6. Tu parles en premier : micro-tâche + `@agentN` + mise à jour de `etat.md`.
7. Boucle `attendre` → travail → `dire` → `attendre`. Timeout 90 s : si vide, rappeler `attendre` immédiatement sans réfléchir.
8. `STOP` (humain ou toi) pour un bilan ; Clôturer ferme le salon.

### Adressage

| Message | Qui le reçoit à l’attente |
| --- | --- |
| Sans `@` | Tous sauf l’auteur |
| `@agent2 …` | Agent 2 (+ Pilote pour suivi) |
| `@pilote …` | Pilote |
| Message humain | Tous |

### `etat.md` (racine du workspace cible)

```markdown
# Objectif
# Décisions
# TODO
# Rédacteur actuel
# Destinataire suivant
```

N’écris du code que si tu es le **Rédacteur actuel**. Sinon : lire, critiquer, proposer.

### Économie de tokens

- L’attente bloquante ne consomme presque pas de tokens.
- Ne republie pas tout l’historique : envoie le delta.
- Micro-tâches ciblées plutôt qu’un roman.
- Un seul rédacteur à la fois.

### API utile (localhost)

| Action | Méthode | Chemin |
| --- | --- | --- |
| Créer un salon | `POST` | `/api/rooms` body `{task, agents_count, slots}` + header `X-Duo-Request: 1` |
| Lire | `GET` | `/api/rooms/{id}` |
| Consignes | `GET` | `/api/rooms/{id}/consignes` + `X-Duo-Request: 1` |
| Publier | `POST` | `/api/rooms/{id}/messages` body `{token, text}` |
| Attendre | `GET` | `/api/rooms/{id}/wait?token=…&after=SEQ&timeout=90` |
| Clôturer | `POST` | `/api/rooms/{id}/close` |

MCP (si configuré) : outils `lire_salon`, `dire`, `attendre`. L’identité vient du **token** de ta consigne.

## Carte du code

| Fichier | Rôle |
| --- | --- |
| `src/duo/bus.py` | Salon : rooms, jetons, wait adressé, consignes, `etat.md` dans le protocole |
| `src/duo/api.py` | FastAPI : routes `/api/rooms*`, UI `/`, orchestrateur `/orchestre` |
| `src/duo/mcp_bus.py` | Pont MCP stdio vers le salon |
| `src/duo/static/salon.html` + `salon.js` | UI création N agents + consignes |
| `src/duo/orchestrator.py` | Mode orchestrateur (CLI), distinct du salon |
| `src/duo/adapters/*` | Adaptateurs Cursor / Copilot CLI (orchestrateur) |
| `docs/ARCHITECTURE.md` | Schéma salon + orchestrateur |
| `tests/test_bus.py` | Couverture multi-agents et adressage |

## Intégration dans un projet cible

1. Copier (ou ouvrir) ce dossier `multiagents` à côté du projet.
2. Lancer le serveur Duo.
3. Ouvrir le **même dossier de travail** (le jeu / l’app) dans chaque fenêtre d’agent.
4. Le Pilote colle sa consigne ; les agents collent les leurs.
5. Le code modifié est celui du projet cible ; Duo ne fait que relayer les messages.

Pour Vestiges horizons, la copie locale vit sous `multiagents/` à la racine du workspace, avec MCP et skill Pilote.
