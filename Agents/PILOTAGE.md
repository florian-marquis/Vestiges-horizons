# PILOTAGE — Processus de travail sur le système d'instruction des agents

Ce fichier s'applique en mode PILOTAGE (voir `AGENTS.md`, section 2). Le projet courant est le système d'instruction lui-même : on conçoit, corrige et versionne la façon dont les agents IA travaillent sur ce dépôt. Le jeu n'est pas touché dans ce mode.

## 1. Périmètre

- **Dans le périmètre** : `AGENTS.md`, tout le dossier `Agents/`, les `Docs/Conventions du projet (pour agents).md` (proposition de l'agent, validation humaine pour les changements de fond), et tout nouveau fichier d'instruction ou de protocole.
- **Hors périmètre** : code du jeu, specs de palier, cadrage, documents de travail de design.

**Périmètre précis** :
- **`PILOTAGE.md`** : Gère les **règles globales** (conventions, processus) qui impactent **toutes les sessions** (ex. : nouvelle convention dans `Docs/Conventions du projet (pour agents).md`).
- **`PROTOCOLE-ERREURS.md`** : Gère les **solutions locales** (problèmes ponctuels, mémoire vive) ou les **règles liées aux erreurs**.
  *Une règle qui modifie le comportement des agents pour toutes les tâches (ex. : "Toujours spécifier les invariants avant de coder") relève de `PILOTAGE.md`. Une solution à un bug de déterminisme relève de `PROTOCOLE-ERREURS.md`.*

## 2. Processus par changement d'instruction

1. **Motif explicite** : chaque changement d'instruction part d'un problème observé (erreur répétée, friction, ambiguïté) ou d'une demande de l'utilisateur, jamais d'une amélioration spéculative.
2. **Cohérence avant écriture** : vérifier que la nouvelle règle ne contredit **ni le cadrage (`Docs/cadrage vestiges horizons - v7.7.md`)**, ni la hiérarchie de `AGENTS.md` section 3, ni les instructions système critiques de l'agent, ni une convention existante. En cas de conflit avec le cadrage, **la règle est rejetée ou reformulée pour être compatible**. Une dérogation explicite doit être validée par l’humain et documentée dans le cadrage. En cas de conflit entre autres règles : le résoudre d'abord, ou expliciter la préséance dans le texte.
3. **Forme canonique d'une règle** : impérative, testable (on peut dire quand elle est violée), bornée (elle dit quand elle ne s'applique pas), sans doublon d'une règle existante.
4. **Versionnement additif** : les conventions se numérotent à la suite (le prochain numéro libre de `Conventions du projet (pour agents).md`), avec intitulé de version et date en sous-titre de groupe. Une règle existante se corrige en la remplaçant, pas en la contredisant ailleurs.
5. **Retour d'expérience** : toute friction constatée lors de l'application des instructions (règle ignorée, interprétations divergentes, coût excessif) est un motif de changement recevable ; la révision passe par ce processus.

---
**Template de proposition de modification** (à remplir par l'agent) :
```markdown
### [PROP-<n>] — <Titre court>
- **Motif** : [Problème observé ou demande utilisateur]
- **Règle proposée** : [Texte exact de la nouvelle règle, format impératif]
- **Test économique** : gain estimé = [X tokens/session], coût = [Y tokens] → **G > C ?** (oui/non)
- **Vérification de cohérence** :
  - [ ] Compatible avec le cadrage (`Docs/cadrage vestiges horizons - v7.7.md`)
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
- Une connaissance utile au mode JEU suit la règle de la mémoire vive (`Agents/PROTOCOLE-ERREURS.md`, section 5) : `Agents/memoire-vive.md` si gain > coût, sinon rien.

## 4. Grille de validation avant de livrer une instruction

1. La règle empêche-t-elle ou accélère-t-elle quelque chose de concret ?
2. Peut-on détecter sa violation en relisant une trace de session ?
3. Un agent qui ne connaît que ce fichier peut-il l'appliquer sans question supplémentaire ?
4. Aucune redondance avec une autre règle (sinon fusionner) ?
5. Le test économique est passé et journalisé ?
6. Retouche minimale : pas de réécriture d'un document entier quand une phrase corrigée suffit.

## 5. Fin de session en mode PILOTAGE

- Mettre à jour `Agents/etat-mode.md` si le mode a changé.
- Ajouter une ligne à l'Historique ci-dessous si un fichier d'instruction a été créé ou restructuré.
- **Commiter et pousser** toute modification de fichier d'instruction selon les conventions 28-33.

## 6. Audit rétroactif des règles existantes

À chaque modification du fichier des conventions (`Docs/Conventions du projet (pour agents).md`), **évaluer les règles existantes** avec le test économique (section 3). Si une règle a un coût > gain :
- **Rétrograder** vers la spec du palier concerné (si locale).
- **Condenser** (si redondante ou trop verbeuse).
- **Supprimer** (si obsolète ou inapplicable).
*Exemple : La règle 25 (spécification de tâche) coûte ~200 tokens. Si elle n'est utilisée que pour 1 tâche sur 10, son gain peut être < coût → à évaluer.*

## 7. Sélection des modèles

Lorsqu'un morceau de travail est jugé trop difficile à réaliser avec le modèle actuel, **estimer le modèle le moins coûteux nécessaire** pour accomplir ce morceau. L'agent obtient cette information en effectuant une recherche internet.

**Catégories de quotas** :
- **Quota modèles maison Cursor** : Cursor 2.5, Grok 4.6, Grok 4.7
- **Quota modèles extérieurs Cursor** : Gemini, Claude, GPT

## 8. Proposition proactive de la suite

Pendant le codage du jeu, **TOUTE réponse finale doit se terminer par une section explicite "Prochaine étape suggérée" ou "Suite du développement"**, même si la tâche est considérée comme complète. Une réponse sans cette section est considérée comme incomplète et doit être corrigée.

Cette section doit **toujours spécifier explicitement** :
- La prochaine étape concrète à accomplir
- **Si une nouvelle conversation est nécessaire ou si on reste dans la conversation actuelle** (ex: "→ On reste dans cette conv" ou "→ Ouvrir une nouvelle conv")

Si une décision de l'utilisateur est nécessaire, **préciser systématiquement un choix suggéré** parmi les options présentées, avec justification en une phrase.

**Test économique** : règle — gain estimé 60 tokens/session (évite les rappels utilisateur, les sessions bloquées et les ambiguïtés de conversation) vs coût 20 tokens → **G > C : oui**.

## Historique

- [initial] Création du système : routeur (`AGENTS.md`), `Agents/PILOTAGE.md`, `Agents/DEV-JEU.md`, `Agents/PROTOCOLE-ERREURS.md`, `Agents/solutions.md`, `Agents/memoire-vive.md`, `Agents/etat-mode.md`.
- [2024-11-14] Renforcement de la section 8 : **Toute réponse finale doit se terminer par une section "Prochaine étape suggérée"**. Motif : règle existante violée (réponse sans suggestion). Test économique : gain 50 tokens/session vs coût 20 tokens.
- [2024-11-14] Ajout à la section 8 : **spécification obligatoire du contexte de conversation** (nouvelle conv ou actuelle). Motif : ambiguïté sur la continuité des sessions. Test économique : gain 60 tokens/session vs coût 20 tokens.
