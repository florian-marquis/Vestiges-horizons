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
cd /d "C:\Users\Florian\Nextcloud\perso\ia\Vestiges horizons\multiagents"
start.bat
```

Si `.venv` n'existe pas, `start.bat` le crée. Vérifier ensuite http://127.0.0.1:8765.

## Workflow Pilote sur ce projet

1. **Obligatoire à chaque étape de codage sur `jeu/`** : si aucun salon actif n'est ouvert pour la tâche, démarrer Duo (`multiagents/`), créer/réouvrir un salon standard à 3 agents + Pilote, publier le premier message Pilote et **demander à l'utilisateur d'y connecter les autres agents** avec leurs consignes.
2. Workspace ouvert = **Vestiges horizons** (dossier `jeu/` = code du jeu).
3. Salon standard à **3 agents** + Pilote :
   - **Pilote** : Architecte (`vscode`, lecture seule sur `jeu/src/`).
   - **Agent 1 (`@agent1`)** : Cursor (`cursor`) — **secours / escalade uniquement** (quand quotas des autres épuisés ou tâche trop complexe).
   - **Agent 2 (`@agent2`)** : GitHub Copilot Fusion (`vscode`) — principal.
   - **Agent 3 (`@agent3`)** : Mistral Vibe (`vscode`) — principal.
4. Utiliser `@agent3` et `@agent2` en parallèle si pertinent (un seul `Rédacteur actuel` sur le code), ou jusqu'à épuisement de quota avant bascule. Préserver les quotas Cursor (`@agent1`).
5. Coller la consigne **Pilote** ici ; coller les consignes Agent 1..3 dans leurs fenêtres ouvertes sur `jeu/` (ou la racine selon le rôle).
6. Travailler via le salon : `@agent2` / `@agent3`, `etat.md`, un seul `Rédacteur actuel` à la fois.
7. Attente opaque : `attendre` (MCP) et `python -m duo.wait_room` (avec `PYTHONPATH=multiagents/src`) enchaînent les polls HTTP 90 s et ne reviennent que sur un message ou la clôture ; si l'IDE coupe l'outil, rappeler immédiatement avec le même `SEQ`.

## Commandes API (sans MCP)

Créer le salon standard (exemple) :

```bat
curl -s -X POST http://127.0.0.1:8765/api/rooms -H "Content-Type: application/json" -H "X-Duo-Request: 1" -d "{\"task\":\"...\",\"agents_count\":3,\"slots\":[{\"role\":\"agent1\",\"window\":\"cursor\"},{\"role\":\"agent2\",\"window\":\"vscode\"},{\"role\":\"agent3\",\"window\":\"vscode\"}]}"
```

Attendre / publier : suivre `python -m duo.wait_room --base http://127.0.0.1:8765 ROOM_ID TOKEN SEQ` (avec `PYTHONPATH=multiagents/src`) et la commande de publication dans la consigne Duo.

## Règles

- Ne pas confondre avec le mode PILOTAGE des instructions (`agents/PILOTAGE.md`) : Duo pilote des **agents de code**, pas le système d'instruction.
- Le code modifié est celui de Vestiges (`jeu/`), pas celui de `multiagents/`, sauf correctif Duo demandé.
- Lire `multiagents/docs/PILOTE.md` pour le détail du protocole.
