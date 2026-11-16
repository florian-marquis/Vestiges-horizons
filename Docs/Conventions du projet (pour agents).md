
# Conventions du projet — référence pour agents — v2.1 (2024-11-13)

Document court, à fournir à Cursor en contexte système. Le cadrage reste la source de vérité du design ; celui-ci fixe les règles d'exécution.

## Déterminisme

1. Entiers ou virgule fixe pour tout calcul reproductible. Pas de flottants dans l'état canonique ni dans les tirages.
2. **Jamais d'itération sur `HashMap`** (ordre non spécifié, graine aléatoire par défaut). Utiliser `BTreeMap`, `IndexMap`, ou un hasher à graine fixe.
3. Tout tirage passe par une clé stable : graine du monde, identifiant de l'entité, événement, indice du tirage. Jamais de hasard global, jamais de dépendance à l'ordre des threads.
4. Parallélisme par région : fusion des résultats dans un ordre fixe (régions triées par identifiant), résultats agrégés de façon associative et commutative, sinon séquentiel.
5. Le temps de calcul disponible ne change jamais le résultat : il décide seulement de quand une zone est prête.

## Replais de référence et tests

6. Les replais de référence sont **en lecture seule pour l'agent**. Un test qui échoue se corrige dans le code, jamais en régénérant le replai. Toute modification d'un replai de référence exige une validation humaine explicite, avec motif documenté.
7. Chaque palier est fermé par une suite de tests headless qui doit passer avant d'ajouter une couche.
8. Jamais de grande refactoration sans replai de comparaison avant/après.

## Méthode de travail

9. Spécification d'abord : chaque tâche commence par un texte court (entités, événements, invariants, critères de réussite), tiré du cadrage.
10. Petites tranches verticales : une entité, quelques événements, un test, livrés en une session.
11. Tout est piloté par les données : recettes, actions, régimes, tilesets sont des tables, pas du code.
12. Séparation stricte : état canonique / commandes / événements du moteur / rendu. Le cœur tourne sans rendu.
13. Couches découplées : pas d'appels croisés, communication par événements et variables agrégées partagées.
14. Chaque événement important du journal de causalité enregistre : identifiant, date, acteur, décision, valeurs modifiées, événements antérieurs, date d'apprentissage par le joueur.

## Vérification humaine (le développeur relit)

15. Le design et les résultats de simulation : courbes de griefs, stocks, équilibre des bots, chronologie des crises.
16. Tout changement de comportement constaté dans un replai.
17. Toute modification de replai de référence (voir règle 6).

## Conventions v2 (déterminisme et validation)

18. **Ordonnancement déterministe des événements** : appliquer la règle canonique du cadrage (chapitre 9). Ordre total défini, événements générés pendant un traitement reportés au prochain instant logique, aucune dépendance à l'ordre d'insertion dans les conteneurs. Test : même scénario, ordres d'insertion permutés, état et journal causaux identiques.
19. **Tests de déterminisme parallèle** : toute couche parallélisée est testée en exécution séquentielle contre parallèle, avec des nombres de threads différents, des partitions régionales différentes et des charges artificiellement déséquilibrées. État canonique, événements, agrégats et journal strictement identiques. Une opération non associative et commutative est séquentielle ou utilise une réduction déterministe spécifiée.
20. **Auditabilité des replais de référence** : chaque replai de référence documente version du moteur et des données, graine, configuration de simulation, horizon, empreinte de l'état canonique, événements et indicateurs suivis, et règle d'acceptation des différences. La comparaison porte sur les éléments définis par le test, pas sur chaque détail non canonique. Tout changement volontaire reste soumis à validation humaine explicite et documentée (règle 6).
21. **Benchmark standard par palier et par couche** : mesurer et documenter graine et configuration, entités par niveau de détail, événements traités, décisions d'agents, échanges interrégionaux, mémoire, temps réel, horizon simulé, indicateurs. Scénarios représentatifs et scénarios de stress. Aucun seuil de performance fixé avant les premiers benchmarks.
22. **Invariants des transitions de détail** : tester à chaque palier les cycles agrégé → détaillé → agrégé sur la même graine : conservation des identifiants, des quantités et ressources, absence de duplication ou disparition, préservation des relations et contrats nécessaires, cohérence des agrégats, conservation des événements causaux pertinents. Les champs conservés par type d'entité sont listés dans les spécifications de palier.
23. **Contrôles du journal causal** : identifiant stable et unique, références causales valides et ordonnées, distinction date de production / date d'apprentissage, cohérence des valeurs modifiées avec l'état canonique, compactage qui ne détruit pas les références nécessaires à l'explication. La granularité reste compatible avec le budget mémoire : pas de journalisation de toutes les micro-opérations.
24. **Fiche de revue humaine par palier** : comportements attendus, anomalies à rechercher, courbes à examiner, scénarios de crise, différences de comportement entre versions, décision documentée (accepté, rejeté, à investiguer). Appliquée par palier, pas par petite tranche. Elle détecte les anomalies et documente les arbitrages ; elle ne transforme pas des comportements émergents en résultats attendus rigides.
25. **Obligations de spécification de tâche** : toute tâche touchant au moteur ou à une couche indique, quand pertinent : entités et champs canoniques concernés, événements entrants et sortants, variables agrégées lues ou écrites, invariants, niveau de détail concerné, règle d'identité, coût attendu et mécanisme de plafonnement, données configurables, tests headless, dépendances avec les autres couches. Appliquée avec discernement : pas de checklist exhaustive pour une tâche triviale.

## Conventions v2.1 (arithmétique et observabilité)

26. **Implémentation canonique unique et arrondi déclaré** : toute grandeur agrégée ou cumulée a une seule implémentation canonique ; recalculer la même grandeur par deux chemins est un défaut. La politique d'arrondi est déclarée par opération (division : troncature ou arrondi, direction fixe, jamais alternée) ; les cumuls longs passent par des entiers à échelle déclarée, jamais par des sommes de quotients réarrangés. Les tests de propriété (permutation d'insertion, découpage alternatif, extraction répétée, exécution pas à pas contre batch) s'exécutent en continu comme les invariants.
27. **Observabilité de première classe** : les outils de diagnostic sont entretenus avec le code de simulation, pas ajoutés après coup : séries synthétiques à pas fixe (quotidien et décennal) des agrégats, tableau de bord textuel avec diff entre deux dates, reprise depuis un snapshot, requête « explique » qui liste les contributions terme à terme à une variation d'agrégat. Le journal causal explique un événement ; les séries disent quand regarder.

---

## Gestion de version et git

28. **Commit systématique** : Toute modification de fichier source, spec, document de référence ou configuration doit être commité. Un commit par changement logique et cohérent, pas par fichier.
29. **Messages de commit** : Format impératif court en une ligne : `<type>(<scope>): <description>`. Types autorisés : `feat` (fonctionnalité), `fix` (correction), `docs` (documentation), `refactor` (refactoring sans changement de comportement), `chore` (maintenance). Scope = module ou palier concerné. Exemple : `feat(p1-quartier): ajoute système de griefs des habitants`.
30. **Branches** : `main` est la branche stable. Toute nouvelle fonctionnalité ou palier utilise une branche `feat/<nom>` ou `paliers/<pX>-<description>`, fusionnée par PR après validation humaine. Les corrections urgentes utilisent `hotfix/<problème>`.
31. **Synchronisation** : `git pull` au début de chaque session pour intégrer les changements distants. `git push` après chaque commit local pour partager les modifications.
32. **Interdiction des force-push** : Aucun `git push --force` ou `--force-with-lease` sans validation humaine explicite et documentée.
33. **Atomicité des commits** : Un commit doit être atomique (un seul changement logique) et laisser le code dans un état compilable et testable. Si un changement casse les tests, il doit être corrigé avant le commit.
34. **Validation humaine compréhensible** : Toute demande de décision à l'humain est formulée en français courant, sans jargon non expliqué. Elle commence par dire ce qui va changer et ce que la personne doit vérifier. Elle montre des exemples concrets du résultat attendu, distingue les faits vérifiés des choix de conception, et pose une question à laquelle on peut répondre sans lire le code ni recalculer une formule. Si plusieurs options existent, leurs effets visibles sont expliqués. La réponse attendue est explicite (par exemple : « oui, cela correspond », « non, voici ce qui ne convient pas » ou « je ne sais pas encore »). Les détails techniques restent disponibles en référence, mais ne sont pas un prérequis à la décision.

## Historique des versions

- **v2.2** (2025-01-XX) : Ajout des règles 28-33 (gestion de version et git).
- **v2.3** (2026-11-16) : Ajout de la règle 34 pour rendre les demandes de validation humaine faciles à comprendre et à vérifier.
- **v2.1** (2024-11-13) : Ajout des règles 26-27 (arithmétique et observabilité).
- **v2.0** (2024-11-XX) : Ajout des règles 18-25 (déterminisme et validation).
- **v1.0** (2024-11-XX) : Version initiale (règles 1-17).