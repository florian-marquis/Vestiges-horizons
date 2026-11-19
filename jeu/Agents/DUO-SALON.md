# DUO-SALON — Collaboration multi-agents via Duo

À charger quand l'utilisateur demande un **salon**, plusieurs agents, Duo / multiagents, ou une collaboration Pilote + Agent N sur le jeu.

Ce n'est **pas** le mode PILOTAGE des instructions (`PILOTAGE.md`). Ici on orchestre des fenêtres d'agents qui codent sur `jeu/`.

## Où est l'app

- Dossier : `../multiagents/` (racine du workspace Vestiges horizons)
- Doc Pilote : `../multiagents/docs/PILOTE.md`
- Skill Cursor : `.cursor/skills/duo-salon/SKILL.md`

## Démarrage minimal

1. `cd ../multiagents` puis `start.bat`
2. Ouvrir http://127.0.0.1:8765
3. Salon standard du projet (3 agents + Pilote) :
   - **Pilote** (`@pilote`, outil configurable `antigravity` / `vscode` / `cursor`, switchable à chaud) : Architecte.
   - **Agent 1** (`@agent1`, fenêtre `cursor`) : **secours / escalade uniquement** (si quotas de Vibe/Copilot épuisés ou tâche trop complexe).
   - **Agent 2** (`@agent2`, fenêtre `vscode`) : GitHub Copilot (principal — `0x`/`0.33x`/`1x` selon difficulté).
   - **Agent 3** (`@agent3`, fenêtre `vscode`) : Mistral Vibe (principal).
4. Vérifier que le modèle réglé dans votre fenêtre IDE correspond à celui indiqué par le Pilote dans votre consigne et `etat.md` (un modèle fixe par agent pour toute la session : simple vs complexe), puis coller les consignes ; vous ne pouvez pas changer seul le sélecteur IDE, signalez-le au Pilote s'il diffère. En cas de switch de modèle ou d'outil (`PATCH /api/rooms/{id}/slots` / `changer_slot`), le jeton du rôle reste inchangé.
5. Maintenir `etat.md` à la racine du dossier ouvert ; un seul `Rédacteur actuel` code à la fois (l'autre agent principal peut auditer ou préparer en lecture seule).

## Rappel tokens et attente opaque

L'attente bloquante (`attendre` MCP ou `python -m duo.wait_room --base http://127.0.0.1:8765 ROOM_ID TOKEN SEQ`, avec `PYTHONPATH=../multiagents/src`) enchaîne les polls HTTP de 90 s côté outil et ne revient que lorsqu'un message visible arrive ou que le salon est clos. Si l'IDE interrompt l'outil, le rappeler tout de suite avec **le même `SEQ`**, sans réfléchir.
