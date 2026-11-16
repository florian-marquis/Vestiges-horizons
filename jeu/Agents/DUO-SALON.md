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
3. Créer le salon (N agents, fenêtres)
4. Coller les consignes ; le Pilote parle en premier avec `@agentN`
5. Maintenir `etat.md` à la racine du dossier ouvert ; un seul rédacteur code

## Rappel tokens

L'attente bloquante ne consomme presque pas de tokens. Un timeout vide → rappeler `attendre` tout de suite, sans réfléchir.
