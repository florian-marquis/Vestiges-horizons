# PILOTAGE — Processus de travail sur le système d'instruction des agents

Ce fichier s'applique en mode PILOTAGE (voir `AGENTS.md` à la racine). Le projet courant est le système d'instruction et le pilotage macro-architectural : on conçoit, corrige et versionne la façon dont les agents IA travaillent sur ce dépôt. Le code du jeu dans `jeu/` n'est jamais modifié en direct depuis ce workspace (accès en lecture seule).

## 1. Périmètre

- **Dans le périmètre** : `AGENTS.md` (racine), `agents/`, `docs/`, les `jeu/Docs/Conventions du projet (pour agents).md` (proposition de l'agent, validation humaine pour les changements de fond), l'outillage de coordination multi-agents (`multiagents/`, `.cursor/skills/duo-salon/SKILL.md`) et tout nouveau fichier d'instruction ou de protocole.
- **Hors périmètre en modification directe** : code source du jeu (`jeu/src/`, `jeu/tests/`, `jeu/benches/`, `jeu/Cargo.toml`), réservé aux sous-agents d'implémentation après génération de tickets.

**Distinction des rôles de pilotage** :
- **`agents/PILOTAGE.md`** : Gouvernance des instructions, conventions et architecture documentaire.
- **`multiagents/docs/PILOTE.md`** : Coordination opérationnelle temps réel — **obligatoire à chaque étape de codage sur `jeu/`** — via le salon multi-agents Duo standard (`@pilote` en VS Code, `@agent1` Cursor en secours/escalade uniquement, `@agent2` GitHub Copilot Fusion et `@agent3` Mistral Vibe en exécution principale, un seul `Rédacteur actuel` à la fois ; si aucun salon ouvert, le Pilote crée/réouvre un salon et demande à l'utilisateur d'y connecter les agents).

**Périmètre précis** :
- **`agents/PILOTAGE.md`** : Gère les **règles globales** (conventions, processus) qui impactent **toutes les sessions** (ex. : nouvelle convention dans `jeu/Docs/Conventions du projet (pour agents).md`).
- **`jeu/Agents/PROTOCOLE-ERREURS.md`** : Gère les **solutions locales** (problèmes ponctuels, mémoire vive) ou les **règles liées aux erreurs** pour les agents de développement.
  *Une règle qui modifie le comportement des agents pour toutes les tâches (ex. : "Toujours spécifier les invariants avant de coder") relève de `agents/PILOTAGE.md`. Une solution à un bug de déterminisme relève de `jeu/Agents/PROTOCOLE-ERREURS.md`.*

## 2. Processus par changement d'instruction

1. **Motif explicite** : chaque changement d'instruction part d'un problème observé (erreur répétée, friction, ambiguïté) ou d'une demande de l'utilisateur, jamais d'une amélioration spéculative.
2. **Cohérence avant écriture** : vérifier que la nouvelle règle ne contredit **ni le cadrage (`docs/cadrage vestiges horizons - v7.8.md`, révision v7.8.1)**, ni la hiérarchie de `AGENTS.md` section 3.4, ni les instructions système critiques de l'agent, ni une convention existante. En cas de conflit avec le cadrage, **la règle est rejetée ou reformulée pour être compatible**. Une dérogation explicite doit être validée par l’humain et documentée dans le cadrage. En cas de conflit entre autres règles : le résoudre d'abord, ou expliciter la préséance dans le texte.
3. **Forme canonique d'une règle** : impérative, testable (on peut dire quand elle est violée), bornée (elle dit quand elle ne s'applique pas), sans doublon d'une règle existante.
4. **Versionnement additif** : les conventions se numérotent à la suite (le prochain numéro libre de `jeu/Docs/Conventions du projet (pour agents).md`), avec intitulé de version et date en sous-titre de groupe. Une règle existante se corrige en la remplaçant, pas en la contredisant ailleurs.
5. **Retour d'expérience** : toute friction constatée lors de l'application des instructions (règle ignorée, interprétations divergentes, coût excessif) est un motif de changement recevable ; la révision passe par ce processus.

### Archivage des documents remplacés

- Avant de modifier une spécification ou un cadrage, inventorier les documents actifs sur le même sujet et désigner explicitement la version de référence selon `AGENTS.md`, section 3.4.
- Quand une nouvelle version remplace une ancienne, déplacer l'ancienne sous `docs/archives/`, dans un sous-dossier par type (`cadrages`, `spec`, `validation`), sans la supprimer ni altérer son contenu.
- Ne pas archiver un document qui contient une décision ou un état encore utile et absent des références actives : intégrer d'abord cette information dans la source active ou conserver le document comme référence historique clairement signalée.
- Après déplacement, corriger les liens des documents actifs vers la version retenue ; les archives ne sont ni chargées ni citées comme source de vérité, sauf pour une question historique.
- Vérifier que les versions actives et leurs renvois restent accessibles et qu'aucun doublon obsolète ne demeure dans les dossiers actifs.
- **Motif** : demande explicite d'harmoniser le cadrage et les spécifications et d'archiver les versions obsolètes.
- **Test économique** : gain estimé 200 tokens par session de remise en cohérence documentaire (recherches et relectures évitées), coût estimé 100 tokens par session ; **G > C : oui**.

---
**Template de proposition de modification** (à remplir par l'agent) :
```markdown
### [PROP-<n>] — <Titre court>
- **Motif** : [Problème observé ou demande utilisateur]
- **Règle proposée** : [Texte exact de la nouvelle règle, format impératif]
- **Test économique** : gain estimé = [X tokens/session], coût = [Y tokens] → **G > C ?** (oui/non)
- **Vérification de cohérence** :
  - [ ] Compatible avec le cadrage (`docs/cadrage vestiges horizons - v7.8.md`, révision v7.8.1)
  - [ ] Compatible avec les conventions existantes
  - [ ] Pas de doublon avec une règle existante
- **Impact** : [Fichiers/modules concernés]
- **Validation humaine requise** : [oui/non] (si changement de fond)
```

## 3. Test économique (règle d'or de ce projet)

Une instruction est un coût payé à **chaque session** ; son gain doit être supérieur à ce coût.

- Coût par session ≈ tokens de la règle, payés à chaque session qui la charge.
- Gain par session = tokens évités (investigation, re-lectures, tentatives ratées, erreurs).
- **Ajouter ou garder une règle uniquement si : gain attendu par session > coût par session.** Une règle existante peut être rétrogradée vers une spec locale ou supprimée si la balance devient négative.
- Journaliser le test à chaque ajout ou retrait, sur une ligne : `règle — gain estimé X tokens/session vs coût Y tokens`.
- Une règle locale à un palier, une couche ou une tâche se met dans la spec de ce palier, **pas** dans les instructions globales.
- Une connaissance utile au mode JEU suit la règle de la mémoire vive (`jeu/Agents/PROTOCOLE-ERREURS.md`, section 5) : `jeu/Agents/memoire-vive.md` si gain > coût, sinon rien.

## 4. Grille de validation avant de livrer une instruction

1. La règle empêche-t-elle ou accélère-t-elle quelque chose de concret ?
2. Peut-on détecter sa violation en relisant une trace de session ?
3. Un agent qui ne connaît que ce fichier peut-il l'appliquer sans question supplémentaire ?
4. Aucune redondance avec une autre règle (sinon fusionner) ?
5. Le test économique est passé et journalisé ?
6. Retouche minimale : pas de réécriture d'un document entier quand une phrase corrigée suffit.

## 5. Fin de session en mode PILOTAGE

- Mettre à jour `agents/etat-mode.md` si le mode a changé.
- Ajouter une ligne à l'Historique ci-dessous si un fichier d'instruction a été créé ou restructuré.
- **Commiter et pousser** toute modification de fichier d'instruction selon les conventions 28-33.

## 6. Audit rétroactif des règles existantes

À chaque modification du fichier des conventions (`jeu/Docs/Conventions du projet (pour agents).md`), **évaluer les règles existantes** avec le test économique (section 3). Si une règle a un coût > gain :
- **Rétrograder** vers la spec du palier concerné (si locale).
- **Condenser** (si redondante ou trop verbeuse).
- **Supprimer** (si obsolète ou inapplicable).
*Exemple : La règle 25 (spécification de tâche) coûte ~200 tokens. Si elle n'est utilisée que pour 1 tâche sur 10, son gain peut être < coût → à évaluer.*

## 7. Sélection des modèles, spécialisation stable par session et apprentissage

Avant d'ouvrir un salon Duo, le Pilote relève les modèles et quotas disponibles (`swarm(action="models", all=True)` côté Antigravity + outil MCP `etat_quotas_modeles` ou `GET /api/routing` côté Duo) et arrête **un modèle fixe par agent pour toute la session (`session_profile`)** :
- **Tâches simples `N1`–`N2`** : routées vers un agent léger/inclus (par défaut `@agent2` Copilot sur `gpt-4.1` **0x**).
- **Tâches complexes `N2`–`N3`** : routées vers un agent fort (par défaut `@agent3` Mistral Vibe sur `mistral-large-4-0-3` / `mistral-large-latest` — ou inversion vers Copilot `claude-sonnet-4.6` `1x` + Vibe léger si Vibe est épuisé).
- **Escalade `N4`** : `@agent1` Cursor (en veille le reste du temps).

Les agents IDE (Copilot, Vibe, Cursor) ne pouvant pas modifier seuls leur sélecteur de modèle, cette spécialisation évite de solliciter l'utilisateur en cours de session : le Pilote demande à l'utilisateur de régler le modèle de chaque fenêtre à l'ouverture, et ne redemande un switch (`changer_slot` / `PATCH /api/rooms/{id}/slots`) qu'en cas d'épuisement de quota (`<= 5 %`) ou d'escalade critique. Si l'utilisateur signale de nouveaux modèles disponibles dans un outil, le Pilote les enregistre immédiatement via MCP `apprendre_modeles` (`POST /api/providers/{id}/sync` avec `add_models`).

| Difficulté | `@pilote` (Antigravity / repli Copilot-Vibe-Cursor) | `@agent2` GitHub Copilot | `@agent3` Mistral Vibe | `@agent1` Cursor (secours/escalade) |
| --- | --- | --- | --- | --- |
| **N1** (Trivial / Audit / Doc) | `gemini-3.8-flash-high` (5h éco) / `gemini-4-argon-low` (hebdo) / repli `gpt-4.1` (0x) | `gpt-4.1` / `gpt-5-mini` (**0x inclus**) | `devstral-small-latest` (éco) | `auto` (veille) |
| **N2** (Standard — lot isolé) | `Orchestrator (Argon)` / `gemini-4-hydrogen-medium` / repli `gemini-3-flash` (0.33x) | `gpt-4.1` (0x) ou `gemini-3-flash` (0.33x) | `devstral-medium-latest` / `mistral-large-4-0-3` | `grok-4.7` / `auto` (secours) |
| **N3** (Complexe — Invariants I1–I9) | `Orchestrator (Argon)` / `gemini-4-hydrogen-high` / repli `claude-sonnet-4.6` (1x) | `claude-sonnet-4.6` (1x) / `gemini-2.5-pro` (1x) | `mistral-large-4-0-3` / `mistral-large-latest` | `claude-4.6-sonnet` (escalade) |
| **N4** (Critique / Escalade) | `claude-opus-5-5-high` / `gemini-4-hydrogen-xhigh` / repli `claude-opus-4.6` (3x) | `claude-opus-4.6` (3x) / `claude-sonnet-4.6` | `mistral-large-4-0-3` / `mistral-large-latest` | `claude-4.6-opus` (escalade) |

**Catégories de quotas** :
- **Google Antigravity** : fenêtre glissante **5h** (`Flash`, `Hydrogen`, `Neon`, `Claude`, `GPT-OSS`) vs fenêtre **hebdomadaire** (`Orchestrator (Argon)`, `Gemini 4 Argon`), temps réel via `swarm(action="models", all=True)`.
- **GitHub Copilot** : multiplicateurs premium (`0x` inclus vs `0.33x` vs `1x` vs `3x`), quota live via `/copilot_internal/user`.
- **Mistral Vibe** : `mistral-large-4-0-3`, `devstral-*`, `codestral-latest`, `mistral-large-latest` via `~/.vibe/config.toml` + API `/v1/models` + apprentissage `apprendre_modeles` + quota manuel sur `/comptes`.
- **Cursor** : **Quota modèles maison** (`Auto`, `Cursor 2.5`, `Grok 4.6/4.7`) vs **Quota modèles extérieurs** (`Gemini`, `Claude`, `GPT`), usage live via `usage-summary`.

En cas de switch d'outil du Pilote (`changer_slot`), reprendre sur `etat.md` avec le `last_seq` fourni.

## 8. Proposition proactive de la suite

Pendant le codage du jeu, **TOUTE réponse finale doit se terminer par une section explicite "Prochaine étape suggérée" ou "Suite du développement"**, même si la tâche est considérée comme complète. Une réponse sans cette section est considérée comme incomplète et doit être corrigée.

**Continuation automatique** : L'agent **doit automatiquement continuer** sur les prochaines étapes logiques sans attendre d'intervention utilisateur. **À chaque étape, l'agent doit évaluer si une nouvelle conversation est nécessaire** (ex. : changement de contexte majeur, validation utilisateur requise, action à blast radius). Si une nouvelle conversation est nécessaire, l'agent **doit l'indiquer explicitement** dans la section "Prochaine étape suggérée" et **arrêter** la continuation automatique, car cela nécessite une intervention de l'utilisateur.

Cette section doit **toujours spécifier explicitement** :
- La prochaine étape concrète à accomplir
- **Si une nouvelle conversation est nécessaire ou si on reste dans la conversation actuelle** (ex: "→ On reste dans cette conv" ou "→ Ouvrir une nouvelle conv")

Si une décision de l'utilisateur est nécessaire, **préciser systématiquement un choix suggéré** parmi les options présentées, avec justification en une phrase.

**Test économique** : règle — gain estimé 70 tokens/session (évite les temps morts entre tâches, les rappels utilisateur, les sessions bloquées et les ambiguïtés) vs coût 20 tokens → **G > C : oui**.

## Historique

- [initial] Création du système : routeur (`AGENTS.md`), `Agents/PILOTAGE.md`, `Agents/DEV-JEU.md`, `Agents/PROTOCOLE-ERREURS.md`, `Agents/solutions.md`, `Agents/memoire-vive.md`, `Agents/etat-mode.md`.
- [2024-11-14] Renforcement de la section 8 : **Toute réponse finale doit se terminer par une section "Prochaine étape suggérée"**. Motif : règle existante violée (réponse sans suggestion). Test économique : gain 50 tokens/session vs coût 20 tokens.
- [2024-11-14] Ajout à la section 8 : **spécification obligatoire du contexte de conversation** (nouvelle conv ou actuelle). Motif : ambiguïté sur la continuité des sessions. Test économique : gain 60 tokens/session vs coût 20 tokens.
- [2024-11-14] Ajout à la section 8 : **continuation automatique** sans intervention utilisateur. Motif : optimisation du flux de travail. Test économique : gain 70 tokens/session vs coût 20 tokens.
- [2024-11-14] Ajout à la section 8 : **évaluation systématique du besoin de nouvelle conversation**. Motif : clarification du processus de continuation. Test économique : gain 10 tokens/session vs coût 5 tokens.
- [2026-11-16] Ajout du protocole d'archivage des documents remplacés à la section 2. Motif : demande explicite d'harmonisation documentaire. Test économique : gain estimé 200 tokens/session de remise en cohérence vs coût 100 tokens/session.
- [2026-11-16] Séparation du workspace de pilotage Architecte à la racine (`AGENTS.md`, `agents/`, `docs/`) et du workspace des agents de dev (`jeu/`). Motif : isolation des documents de pilotage hors de portée des sous-agents de codage. Test économique : gain estimé 300 tokens/session (isolation de contexte des sous-agents) vs coût 50 tokens/session.
- [2026-11-16] Intégration de l'outil multi-agents Duo (`multiagents/`) et du protocole Pilote/Salon dans `AGENTS.md`, `agents/PILOTAGE.md`, `jeu/AGENTS.md` et `.cursor/skills/duo-salon/SKILL.md`. Motif : ajout de l'outil de pilotage multi-agents au workspace. Test économique : gain estimé 250 tokens/session (coordination directe par renvoi sans duplication) vs coût 80 tokens/session.
- [2026-11-16] Normalisation du salon Duo standard (Pilote VS Code + `@agent1` Cursor secours/escalade, `@agent2` GitHub Copilot Fusion, `@agent3` Mistral Vibe) dans `AGENTS.md`, `agents/PILOTAGE.md`, `.cursor/skills/duo-salon/SKILL.md` et `jeu/Agents/DUO-SALON.md`. Motif : règle permanente de répartition et d'économie de quotas. Test économique : gain estimé 150 tokens/session (zéro redéfinition des slots et quotas) vs coût 40 tokens/session.
- [2026-11-17] Obligation systématique du salon Duo à chaque étape de codage (réouverture + demande explicite à l'utilisateur de connecter les agents) dans `AGENTS.md`, `agents/PILOTAGE.md` et `.cursor/skills/duo-salon/SKILL.md`. Motif : demande utilisateur explicite pour garantir l'orchestration multi-agents à chaque lot de code. Test économique : gain estimé 180 tokens/session (zéro oubli d'orchestration ou travail isolé non coordonné) vs coût 35 tokens/session.
- [2026-11-17] Prise en compte des évolutions de `multiagents/` (attente opaque `wait_room` anti-raccrochage sur MCP/CLI et page OAuth/quotas `/comptes` pour `cursor`, `copilot`, `vibe`, `antigravity`) dans `AGENTS.md`, `.cursor/skills/duo-salon/SKILL.md` et `jeu/Agents/DUO-SALON.md`. Motif : alignement sur l'outil Duo mis à jour. Test économique : gain estimé 120 tokens/session (évite la boucle sur polls vides et clarifie `wait_room` / `/comptes`) vs coût 30 tokens/session.
- [2026-11-17] Règle Pilote de découpage en micro-tâches courtes et interdiction de réaffecter `Rédacteur actuel` pendant la phase d'analyse d'un agent sans ping + cycle d'attente ou accord explicite (`AGENTS.md` §2.3). Motif : retour utilisateur (`seq=9` sur `@agent3` Mistral Vibe encore en analyse de code). Test économique : gain estimé 150 tokens/session (évite travail perdu et réaffectations prématurées) vs coût 30 tokens/session (`G > C : oui`).
- [2026-11-19] Routage dynamique par quotas/modèles/difficulté (N1–N4) et switch à chaud d'outil/modèle étendu au Pilote (`@pilote`) et aux agents (`@agent1..@agent3`) dans `AGENTS.md` §2.3, `agents/PILOTAGE.md` §7 et `multiagents/`. Motif : demande utilisateur explicite (`[PROP-1]`). Test économique : gain estimé 250 tokens/session + économie de quotas premium (0x/0.33x sur N1–N2, bascule auto si quota épuisé) vs coût 55 tokens/session (`G > C : oui`).
- [2026-11-19] Spécialisation stable par session (`session_profile` : 1 modèle fixe par agent, simple vs complexe, pour éviter les switchs manuels IDE répétés) et apprentissage persistant des modèles signalés par l'humain (`apprendre_modeles`, ajout `mistral-large-4-0-3` sur Vibe) dans `AGENTS.md` §2.3, `agents/PILOTAGE.md` §7 et `multiagents/` (`[PROP-2]`). Motif : retour d'expérience live (`seq=7` Copilot, `seq=12` Vibe) et demande utilisateur. Test économique : gain estimé 200 tokens/session + suppression des allers-retours humains inutiles vs coût 45 tokens/session (`G > C : oui`).
