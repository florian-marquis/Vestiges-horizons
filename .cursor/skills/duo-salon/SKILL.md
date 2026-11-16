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
- UI : http://127.0.0.1:8765

## Lancer le serveur

```bat
cd /d "C:\Users\Florian\Nextcloud\perso\ia\Vestiges horizons\multiagents"
start.bat
```

Si `.venv` n'existe pas, `start.bat` le crée. Vérifier ensuite http://127.0.0.1:8765.

## Workflow Pilote sur ce projet

1. Workspace ouvert = **Vestiges horizons** (dossier `jeu/` = code du jeu).
2. Lancer Duo depuis `multiagents/`.
3. Créer un salon dans l'UI : tâche + N agents + fenêtres.
4. Coller la consigne **Pilote** ici ; coller les consignes Agent N dans les autres fenêtres ouvertes sur le **même** dossier `jeu` (ou la racine du workspace, selon la tâche).
5. Travailler via le salon : `@agent1`, `etat.md` à la racine du dossier ouvert, un rédacteur à la fois.
6. Attente 90 s ; si timeout vide → rappeler `attendre` immédiatement.

## Commandes API (sans MCP)

Créer (exemple) :

```bat
curl -s -X POST http://127.0.0.1:8765/api/rooms -H "Content-Type: application/json" -H "X-Duo-Request: 1" -d "{\"task\":\"...\",\"agents_count\":2,\"slots\":[{\"window\":\"cursor\"},{\"window\":\"vscode\"}]}"
```

Attendre / publier : suivre les commandes Python dans la consigne Duo (token inclus).

## Règles

- Ne pas confondre avec le mode PILOTAGE des instructions (`agents/PILOTAGE.md`) : Duo pilote des **agents de code**, pas le système d'instruction.
- Le code modifié est celui de Vestiges (`jeu/`), pas celui de `multiagents/`, sauf correctif Duo demandé.
- Lire `multiagents/docs/PILOTE.md` pour le détail du protocole.
