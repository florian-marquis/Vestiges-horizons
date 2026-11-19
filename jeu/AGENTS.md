# AGENTS.md — Routeur des agents de développement (`jeu/`)

Ce fichier est le point d'entrée unique des sous-agents d'implémentation (Cursor, Copilot, Vibe) travaillant dans le workspace `jeu/`. Il définit les instructions d'exécution à charger pour développer et tester le code du jeu « Vestiges Horizons ».

## 1. Chargement obligatoire en début de session

Charger dans cet ordre :
1. `AGENTS.md` (ce fichier)
2. `Agents/DEV-JEU.md`
3. `Agents/memoire-vive.md`
4. `Docs/Conventions du projet (pour agents).md` (règles d'exécution v2.3)
5. `Agents/DUO-SALON.md` *(uniquement si la session se déroule dans un salon multi-agents Duo)*

## 2. Hiérarchie des documents de référence (workspace `jeu/`)

En cas de conflit, dans l'ordre décroissant :
1. Les `Docs/Conventions du projet (pour agents).md` (règles d'exécution, version v2.3 — non négociables)
2. La spécification active, l'ordre de mission ou le ticket assigné :
   - `Docs/spec P1 - quartier vivant.md`
   - `Docs/spec/P1b - scene locale et immersion - spec.md`
   - `Docs/spec/T7-T8 correctif - spec v1.2.md`
   - `Docs/spec/T7-T8 tranche A2 - ordre de mission.md`
   - `Docs/tickets/`
3. Les instructions et mémoires du dossier `Agents/` (`Agents/DEV-JEU.md`, `Agents/memoire-vive.md`)

## 3. Mémoire : les deux registres

- **Mémoire vive** : `Agents/memoire-vive.md` — chargée en début de chaque session de développement. Ne contient que des entrées dont le gain en tokens dépasse leur coût (règle économique dans `Agents/PROTOCOLE-ERREURS.md`, section 5).
- **Journal des solutions** : `Agents/solutions.md` — registre froid des problèmes résolus. **Jamais chargé en contexte**, sauf en cas d'erreur ou pour enrichissement (`Agents/PROTOCOLE-ERREURS.md`, sections 1 et 2).

## 4. Règles transverses

- Langue de travail et des documents : **français**.
- Ne modifier les fichiers de `Agents/` que pour les enrichissements prévus par `Agents/PROTOCOLE-ERREURS.md`.
- Toute entrée ajoutée à la mémoire vive obéit au test économique : son gain en tokens doit dépasser son coût à chaque session.
- Ne jamais créer de fichier résumant un autre fichier sans ce test économique.
