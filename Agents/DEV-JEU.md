# DEV-JEU — Développement de « Vestiges horizons » 100% en mode agent

Ce fichier s'applique en mode JEU (voir `AGENTS.md`, section 2). Le jeu est développé entièrement par des agents IA : spécification, implémentation, tests, documentation. L'humain garde les points de validation listés en section 4 et décide du passage de palier.

## 0. Chargement en début de session JEU

Dans cet ordre, uniquement les sections utiles à la tâche du jour :
1. `Agents/memoire-vive.md` (entier — il ne contient que des entrées rentables)
2. `Docs/suivi du projet - todo.md` (état des tâches)
3. `Docs/spec P1 - quartier vivant.md` (palier courant) et la section du cadrage visée
4. `Docs/Conventions du projet (pour agents).md` (règles d'exécution)

## 1. Sources de vérité

- Design : `Docs/cadrage vestiges horizons - v7.7.md`. En silence sur un point, la spec de palier tranche ; en silence encore, l'agent propose et l'humain décide.
- Exécution : `Docs/Conventions du projet (pour agents).md` — déterminisme, tests, méthode de travail, observabilité. Non négociables.
- État d'avancement : `Docs/suivi du projet - todo.md` et `Docs/registre de completude.md`.

## 2. Cycle de travail d'une tâche (mode agent)

1. **Spécification courte d'abord** (convention 9 et 25) : entités, événements, invariants, critères de réussite, en référence au cadrage. Consignée dans la spec de palier si elle la modifie.
2. **Tranche verticale** (convention 10) : une entité, quelques événements, un test headless, livrés en une session.
3. **Implémentation** selon les conventions déterministes (entiers/virgule fixe, pas de `HashMap` itéré, tirages à clé stable, ordonnancement canonique).
4. **Tests headless obligatoires** avant de clore la tranche (conventions 7, 18, 19, 22, 23, 26) ; les tests de propriété et d'invariants tournent avec chaque changement concerné.
5. **Observabilité entretenue avec le code** (convention 27) : pas de couche simulée sans son tableau de bord textuel ni ses séries.
6. **Clôture** : mettre à jour `Docs/suivi du projet - todo.md` (et le registre de complétude si une entrée y correspond), committer.

## 3. Erreurs

Appliquer `Agents/PROTOCOLE-ERREURS.md`, sections 1 et 2 : consulter le journal des solutions avant la deuxième tentative, enrichir le journal à la résolution, évaluer la promotion en mémoire vive, escalader après deux échecs consécutifs malgré le journal.

## 4. Validation humaine (l'agent prépare, l'humain décide)

L'agent prépare la fiche de revue (conventions 15 à 17 et 24) et s'arrête net sur :
1. Design et résultats de simulation : courbes de griefs, stocks, équilibre des bots, chronologie des crises.
2. Tout changement de comportement constaté dans un replai.
3. Toute demande de modification d'un replai de référence (convention 6 : jamais sans validation explicite documentée).
4. Le passage de palier et l'ouverture de la spec suivante.

L'agent ne « valide » jamais ces points lui-même ; il formule les anomalies constatées et des recommandations.

## 5. Sous-agents

Pour une tranche isolable (implémentation, tests, rédaction), un sous-agent peut être délégué : lui passer les chemins des instructions (`AGENTS.md`, ce fichier, sections utiles des conventions) et la description de la tranche. Le sous-agent applique lui-même le protocole d'erreurs. La tranche revient validée par tests, sinon elle n'est pas intégrée.

## 6. Interdits du mode JEU

- Modifier le cadrage ou les conventions sans validation humaine.
- Régénérer un replai de référence pour faire passer un test (convention 6).
- Modifier les fichiers du dossier `Agents/` hors des enrichissements prévus par le protocole d'erreurs.
- Ajouter de la configuration de projet, des dépendances ou un changement de stack sans en avoir fait la demande explicite.
