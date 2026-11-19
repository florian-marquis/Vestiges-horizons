---
name: duo-salon
description: >-
  Piloter le salon multi-agents Duo (Pilote + Agent 1..N) pour collaborer sur
  Vestiges horizons. À utiliser quand l'utilisateur demande un salon, plusieurs
  agents, Duo, multiagents, @agent, ou de lancer/contrôler l'app de communication
  inter-agents.
---

# Duo salon — skill Pilote (Vestiges horizons)

## Emplacement

- App : `multiagents/` à la racine de ce workspace
- Doc Pilote : `multiagents/docs/PILOTE.md`
- Carte code : `multiagents/docs/CODE.md`
- UI : http://127.0.0.1:8765 (salon `/`, orchestrateur `/orchestre`, comptes OAuth & quotas `/comptes`)

## Lancer le serveur

```bat
cd /d "C:\dev\vestiges-horizons\multiagents"
start.bat
```

Si `.venv` n'existe pas, `start.bat` le crée. Vérifier ensuite http://127.0.0.1:8765.

## Workflow Pilote sur ce projet

1. **Vérifier quotas & modèles avant ouverture** : appeler `swarm(action="models", all=True)` côté Antigravity puis MCP `etat_quotas_modeles` (ou `GET /api/routing?difficulty=N1..N4`) pour choisir le couple `(outil, modèle)` optimal pour `@pilote` et chaque `@agentN`.
2. **Obligatoire à chaque étape de codage sur `jeu/`** : si aucun salon actif n'est ouvert pour la tâche, démarrer Duo (`multiagents/`), créer/réouvrir un salon standard à 3 agents + Pilote (`POST /api/rooms` avec `pilote` et `slots`), publier le premier message Pilote et **indiquer explicitement à l'utilisateur quel outil et quel modèle brancher sur `@pilote` et chaque `@agentN`** avant de lui demander d'y connecter les agents.
3. Workspace ouvert = **Vestiges horizons** (dossier `jeu/` = code du jeu).
4. Salon standard à **3 agents** + Pilote :
   - **Pilote (`@pilote`)** : Architecte (par défaut `antigravity` `Orchestrator (Argon)`, switchable à chaud vers `copilot`/`vibe`/`cursor` via `changer_slot` ou `PATCH /api/rooms/{id}/slots` selon quotas/difficulté ; lecture seule sur `jeu/src/`).
   - **Agent 1 (`@agent1`)** : Cursor (`cursor`) — **secours / escalade uniquement** (quand quotas des autres épuisés ou tâche trop complexe).
   - **Agent 2 (`@agent2`)** : GitHub Copilot (`vscode`) — principal (`GPT-4.1` 0x sur N1/audit, `Gemini 3 Flash` 0.33x sur N2, `Claude Sonnet 4.6` 1x sur N3).
   - **Agent 3 (`@agent3`)** : Mistral Vibe (`vscode`) — principal (`devstral-small/medium-latest`, `mistral-large-latest`).
5. **Switch à chaud** : si la difficulté change ou qu'un quota (y compris celui du Pilote) tombe bas, appeler `changer_slot` / `PATCH /api/rooms/{id}/slots`, mettre à jour `etat.md` et demander à l'utilisateur de brancher le nouveau modèle/outil (reprise du Pilote sur `etat.md` + `last_seq`).
6. Travailler via le salon : `@agent2` / `@agent3`, `etat.md`, un seul `Rédacteur actuel` à la fois.
7. Attente opaque : `attendre` (MCP) et `python -m duo.wait_room` (avec `PYTHONPATH=multiagents/src`) enchaînent les polls HTTP 90 s et ne reviennent que sur un message ou la clôture ; si l'IDE coupe l'outil, rappeler immédiatement avec le même `SEQ`.

## Commandes API (sans MCP)

Créer le salon standard (exemple) :

```bat
curl -s "http://127.0.0.1:8765/api/routing?difficulty=N2&pilote_provider=antigravity"
curl -s -X POST http://127.0.0.1:8765/api/rooms -H "Content-Type: application/json" -H "X-Duo-Request: 1" -d "{\"task\":\"...\",\"agents_count\":3,\"pilote\":{\"provider\":\"antigravity\",\"window\":\"antigravity\",\"model\":\"gemini-pro-orchestrator-g1-pro\"},\"slots\":[{\"role\":\"agent1\",\"window\":\"cursor\",\"provider\":\"cursor\",\"model\":\"auto\"},{\"role\":\"agent2\",\"window\":\"vscode\",\"provider\":\"copilot\",\"model\":\"gpt-4.1\"},{\"role\":\"agent3\",\"window\":\"vscode\",\"provider\":\"vibe\",\"model\":\"devstral-medium-latest\"}]}"
curl -s -X PATCH http://127.0.0.1:8765/api/rooms/ROOM_ID/slots -H "Content-Type: application/json" -H "X-Duo-Request: 1" -d "{\"role\":\"pilote\",\"provider\":\"copilot\",\"model\":\"claude-sonnet-4.6\",\"reason\":\"Quota bas\"}"
```

Attendre / publier : suivre `python -m duo.wait_room --base http://127.0.0.1:8765 ROOM_ID TOKEN SEQ` (avec `PYTHONPATH=multiagents/src`) et la commande de publication dans la consigne Duo.

## Règles

- Ne pas confondre avec le mode PILOTAGE des instructions (`agents/PILOTAGE.md`) : Duo pilote des **agents de code**, pas le système d'instruction.
- Le code modifié est celui de Vestiges (`jeu/`), pas celui de `multiagents/`, sauf correctif Duo demandé.
- Lire `multiagents/docs/PILOTE.md` pour le détail du protocole.
