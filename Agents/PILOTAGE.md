# PILOTAGE — Processus de travail sur le système d'instruction des agents

Ce fichier s'applique en mode PILOTAGE (voir `AGENTS.md`, section 2). Le projet courant est le système d'instruction lui-même : on conçoit, corrige et versionne la façon dont les agents IA travaillent sur ce dépôt. Le jeu n'est pas touché dans ce mode.

## 1. Périmètre

- **Dans le périmètre** : `AGENTS.md`, tout le dossier `Agents/`, les `Docs/Conventions du projet (pour agents).md` (proposition de l'agent, validation humaine pour les changements de fond), et tout nouveau fichier d'instruction ou de protocole.
- **Hors périmètre** : code du jeu, specs de palier, cadrage, documents de travail de design.

## 2. Processus par changement d'instruction

1. **Motif explicite** : chaque changement d'instruction part d'un problème observé (erreur répétée, friction, ambiguïté) ou d'une demande de l'utilisateur, jamais d'une amélioration spéculative.
2. **Cohérence avant écriture** : vérifier que la nouvelle règle ne contredit ni la hiérarchie de `AGENTS.md` section 3, ni les instructions système critiques de l'agent, ni une convention existante. En cas de conflit : le résoudre d'abord, ou expliciter la préséance dans le texte.
3. **Forme canonique d'une règle** : impérative, testable (on peut dire quand elle est violée), bornée (elle dit quand elle ne s'applique pas), sans doublon d'une règle existante.
4. **Versionnement additif** : les conventions se numérotent à la suite (le prochain numéro libre de `Conventions du projet (pour agents).md`), avec intitulé de version et date en sous-titre de groupe. Une règle existante se corrige en la remplaçant, pas en la contredisant ailleurs.
5. **Retour d'expérience** : toute friction constatée lors de l'application des instructions (règle ignorée, interprétations divergentes, coût excessif) est un motif de changement recevable ; la révision passe par ce processus.

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

## Historique

- [initial] Création du système : routeur (`AGENTS.md`), `Agents/PILOTAGE.md`, `Agents/DEV-JEU.md`, `Agents/PROTOCOLE-ERREURS.md`, `Agents/solutions.md`, `Agents/memoire-vive.md`, `Agents/etat-mode.md`.
