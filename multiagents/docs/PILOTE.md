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
Comptes OAuth : [http://127.0.0.1:8765/comptes](http://127.0.0.1:8765/comptes)

## Utilisation du salon

1. Dans l’UI, saisir la tâche, choisir le niveau de difficulté (`N1`–`N4`) ou cliquer **Conseiller outils & modèles selon quotas** pour pré-remplir `@pilote` et `@agent1`…`@agentN`.
2. Choisir l'outil (`antigravity`, `copilot`, `vibe`, `cursor`) et le modèle du **Pilote** puis des **Agents** (1–5).
3. Ouvrir le salon.
4. Copier la consigne **Pilote** dans sa fenêtre (mode Agent) et indiquer à l'utilisateur quel outil et quel modèle brancher sur chaque fenêtre.
5. Copier chaque consigne **Agent N** dans sa fenêtre.
6. Tu parles en premier : micro-tâche + `@agentN` + mise à jour de `etat.md` (incluant `# Configuration active (Pilote & Agents)`).
7. Boucle `attendre` → travail → `dire` → `attendre`. L’attente ne revient que sur message ou clôture ; si l’IDE coupe l’outil, rappeler `attendre` avec le même SEQ.
8. **Switch à chaud (Pilote ou Agent)** : à tout moment (changement de difficulté N1–N4 ou quota épuisé), utiliser MCP `changer_slot` ou `PATCH /api/rooms/{id}/slots` (ou le panneau de droite dans l’UI) pour changer l’outil et/ou le modèle de `@pilote` ou d’un `@agentN` sans perdre les jetons. Si l’outil de `@pilote` change, le nouveau Pilote relit `etat.md` et reprend avec `SEQ = last_seq`.
9. `STOP` (humain ou toi) pour un bilan ; Clôturer ferme le salon.

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
# Configuration active (Pilote & Agents)
```

**Seul le Pilote est autorisé à modifier `etat.md`** (écrivain unique). Les agents `@agent1`…`@agentN` le consultent en lecture seule et rendent compte sur le salon. N’écris du code que si tu es le **Rédacteur actuel**. Sinon : lire, critiquer, proposer.

### Routage modèles & quotas (N1–N4) et spécialisation de session

- Avant l’ouverture ou un changement de lot, appeler MCP `etat_quotas_modeles` (avec éventuellement la sortie de `swarm(action="models", all=True)` dans `antigravity_swarm`) ou `GET /api/routing?difficulty=N1..N4&pilote_provider=...`.
- **Spécialisation stable (`session_profile`)** : les agents IDE ne pouvant pas changer seuls leur sélecteur de modèle, fixer **1 seul modèle par agent pour la session** à l'ouverture (ex. `@agent2` Copilot sur `gpt-4.1` **0x** pour les tâches simples `N1`–`N2` ; `@agent3` Vibe sur `mistral-large-4-0-3` / `mistral-large-latest` pour les tâches complexes `N2`–`N3` ; `@agent1` Cursor en veille `N4`) afin de router vers le bon agent sans solliciter l’humain en cours de session.
- **Apprentissage des modèles** : si l’utilisateur indique de nouveaux modèles disponibles dans son outil, les enregistrer immédiatement avec MCP `apprendre_modeles` (`POST /api/providers/{id}/sync` avec `add_models`).
- **Toujours annoncer à l’utilisateur quel outil et quel modèle brancher** à l'ouverture sur `@pilote` et chaque `@agentN`, et ne demander un switch manuel (`changer_slot`) qu'en cas de quota épuisé ou d'escalade exceptionnelle.

### Économie de tokens

- L’attente bloquante ne consomme presque pas de tokens : MCP `attendre` et `python -m duo.wait_room` enchaînent les polls jusqu’à un message ou la clôture.
- Si l’IDE coupe l’outil, rappelle immédiatement l’attente avec le même SEQ.
- Ne republie pas tout l’historique : envoie le delta.
- Micro-tâches ciblées plutôt qu’un roman.
- Un seul rédacteur à la fois.

### API utile (localhost)

| Action | Méthode | Chemin |
| --- | --- | --- |
| Créer un salon | `POST` | `/api/rooms` body `{task, agents_count, pilote, slots}` + header `X-Duo-Request: 1` |
| Lire | `GET` | `/api/rooms/{id}` |
| Consignes | `GET` | `/api/rooms/{id}/consignes` + `X-Duo-Request: 1` |
| Switch slot (Pilote/Agent) | `PATCH` | `/api/rooms/{id}/slots` body `{role, provider, window, model, reason}` + `X-Duo-Request: 1` |
| Routage quotas & modèles | `GET` | `/api/routing?difficulty=N1..N4&pilote_provider=antigravity` |
| Sync quotas / manuel | `POST` | `/api/providers/{id}/sync` + `X-Duo-Request: 1` |
| Publier | `POST` | `/api/rooms/{id}/messages` body `{token, text}` |
| Attendre (poll) | `GET` | `/api/rooms/{id}/wait?token=…&after=SEQ&timeout=90` |
| Attendre (opaque) | CLI | `python -m duo.wait_room --base URL room_id token SEQ` |
| Clôturer | `POST` | `/api/rooms/{id}/close` |

MCP (si configuré) : outils `lire_salon`, `dire`, `attendre`, `etat_quotas_modeles`, `changer_slot`, `apprendre_modeles`. L’identité vient du **token** de ta consigne. `attendre` ne revient que sur message ou clôture.

## Carte du code

| Fichier | Rôle |
| --- | --- |
| `src/duo/bus.py` | Salon : rooms, jetons, wait adressé, consignes, `etat.md` dans le protocole |
| `src/duo/api.py` | FastAPI : routes `/api/rooms*`, UI `/`, orchestrateur `/orchestre`, comptes `/comptes` |
| `src/duo/mcp_bus.py` | Pont MCP stdio vers le salon |
| `src/duo/wait_room.py` | Attente opaque (retry des polls) pour CLI et MCP |
| `src/duo/static/salon.html` + `salon.js` | UI création N agents + consignes |
| `src/duo/static/comptes.html` + `comptes.js` | Auth OAuth via CLI + quotas best-effort |
| `src/duo/providers/*` | Statut / login / quotas Cursor, Copilot, Vibe, Antigravity |
| `src/duo/orchestrator.py` | Mode orchestrateur (CLI), distinct du salon |
| `src/duo/adapters/*` | Adaptateurs Cursor / Copilot / Vibe / Antigravity (orchestrateur) |
| `docs/ARCHITECTURE.md` | Schéma salon + orchestrateur |
| `tests/test_bus.py` | Couverture multi-agents et adressage |

## Intégration dans un projet cible

1. Copier (ou ouvrir) ce dossier `multiagents` à côté du projet.
2. Lancer le serveur Duo.
3. Ouvrir le **même dossier de travail** (le jeu / l’app) dans chaque fenêtre d’agent.
4. Le Pilote colle sa consigne ; les agents collent les leurs.
5. Le code modifié est celui du projet cible ; Duo ne fait que relayer les messages.

Pour Vestiges horizons, la copie locale vit sous `multiagents/` à la racine du workspace, avec MCP et skill Pilote.
