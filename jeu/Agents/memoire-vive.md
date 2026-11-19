# Mémoire vive — lue au début de chaque session JEU

Règle d'admission (voir `Agents/PROTOCOLE-ERREURS.md`, sections 3 et 5) : une entrée n'est ici que si son gain attendu en tokens dépasse son coût. Toute nouvelle entrée passe le test ; l'audit d'une session peut rétrograder une entrée vers `Agents/solutions.md`.

---

### Économie de re-lecture des documents de référence
- Info : le routeur `AGENTS.md` section 3 fixe la hiérarchie (cadrage v7.8/v7.8.1 > conventions > spec de palier > dossier Agents). Les conventions (27 règles, ~1 200 tokens) tiennent en mémoire de travail dès le début d'une session de code ; le cadrage de 6 000+ tokens se lit **par section ciblée uniquement** (recherche par titre de chapitre, jamais lecture intégrale pour une tâche ponctuelle).
- Gain : ~6 000 tokens par tâche (pas de re-lecture intégrale du cadrage), suppression des hésitations sur la préséance des docs.
- Coût : ~120 tokens (cette entrée).
- Origine : AGENTS.md, PILOTAGE.md.

---

### P-1 — Synchronisation cache après write_file (promu P-1)
- Info : Après `write_file`, toujours faire `git add <fichier> && git commit -m "..."` pour forcer la synchronisation du cache. Sans commit, `cat`/`wc -l` peut montrer l'ancien contenu.
- Gain : Évite 1-2h de débogage par incident (recherche de bug fantôme, re-écriture inutile).
- Coût : ~50 tokens.
- Origine : solutions.md P-1, session Tranche 1.

<!-- Nouvelles entrées ici (info ou problème promu), format PROTOCOLE-ERREURS.md section 5 -->

### T6 — Tranche 6 : Marché du travail implémentée
- Info : La tranche 6 (marché du travail, métiers, parrainage) est implémentée dans `src/labor.rs` avec : Q(a) d'emploi modulée par les traits du décideur, engagement employé↔firme (0-1000), décisions ponctuelles (~1/mois), chocs 3 et 4, deux régimes (firme patiente vs firme au churn), transition du bouchon, présentéisme et contagion. Tous les tests T6.2-T6.7 passent (73 tests total dans labor.rs). Le benchmark P1 headless (10 ans, graine 42) valide tous les invariants.
- Gain : Évite la re-lecture complète de la spec §18 et de la structure labor.rs pour les tâches T6. Référence rapide pour les prochaines implémentations.
- Coût : ~200 tokens.
- Origine : Session implémentation T6, commit 5bbdb2d.
- Tests : 73 tests pass (cargo test), benchmark P1 valide (I2, I4, I5, I8, I9).

### T7 — Tranche 7 : implémentation existante, correctif requis
- Info : `src/salary.rs` délègue maintenant le calcul canonique à `src/griefs.rs`, avec l'équation du cadrage v7.8/v7.8.1 (0,09u + 0,02p − 0,03q) et une intégration événementielle. `verify_i8_concordance` dans `src/entities.rs` utilise encore une moyenne entière pour les non-nommés et n'assure pas la concordance pour toutes les cohortes ; ne pas considérer I8 comme fermé.
- Gain : évite de confondre les tests de l'ancienne implémentation avec les critères de design lors du correctif T7-T8.
- Coût : ~100 tokens.
- Origine : spec corrective T7-T8 v1.2, diagnostic historique archivé et cadrage v7.8.

### T8 — Tranche 8 : implémentation existante, intégration à corriger
- Info : `src/strike.rs` contient un gestionnaire de grèves et des tests, mais la spec corrective T7-T8 v1.2 ne couvre que la tranche A. Le risque, les grèves et leur validation restent ouverts ; les anciens rapports et les tests existants ne ferment pas à eux seuls le contrat cible de la spec P1 §35.
- Gain : évite de traiter les tests unitaires existants comme preuve de fermeture du basculement T8.
- Coût : ~75 tokens.
- Origine : spec corrective T7-T8 v1.2 et spec P1 §35 ; diagnostic v3.1 conservé dans les archives pour l'historique.
