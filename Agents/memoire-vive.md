# Mémoire vive — lue au début de chaque session JEU

Règle d'admission (voir `Agents/PROTOCOLE-ERREURS.md`, sections 3 et 5) : une entrée n'est ici que si son gain attendu en tokens dépasse son coût. Toute nouvelle entrée passe le test ; l'audit d'une session peut rétrograder une entrée vers `Agents/solutions.md`.

---

### Économie de re-lecture des documents de référence
- Info : le routeur `AGENTS.md` section 3 fixe la hiérarchie (cadrage v7.7 > conventions > spec de palier > dossier Agents). Les conventions (27 règles, ~1 200 tokens) tiennent en mémoire de travail dès le début d'une session de code ; le cadrage de 6 000+ tokens se lit **par section ciblée uniquement** (recherche par titre de chapitre, jamais lecture intégrale pour une tâche ponctuelle).
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

### T7 — Tranche 7 : Salaires réels et griefs implémentée
- Info : La tranche 7 (panier, salaire réel, griefs) est implémentée dans `src/salary.rs` avec : calcul du panier (nourriture, logement, eau, fiscalité), salaire réel = (salaire_horaire × heures / coût_panier) × 100, intégration événementielle des griefs (dG = 0.8·u + 0.2·p − 0.25·q), équation de griefs bornée [0, 100], taux de risque fonction de G et salaire réel. Tous les tests T7.1-T7.7 passent (7 tests). L'invariant I8 (concordance griefs cohorte/individus) est implémenté à tolérance zéro dans `src/entities.rs`.
- Gain : Évite la re-lecture de la spec §19. Permet de passer à T8 (basculements) qui dépend de T7.
- Coût : ~150 tokens.
- Origine : Session implémentation T7, commit 85af42a.
- Tests : 7 tests pass (cargo test), I8 vert.

### T8 — Tranche 8 : Basculements implémentée
- Info : La tranche 8 (seuil de risque cumulé, grève) est implémentée dans `src/strike.rs` avec : gestion des grèves (Strike, StrikeManager), tirage unique par clé stable (décision 43, jamais re-tirable), planification du franchissement du seuil, fin de grève par concession (Q(a)) ou durée écoulée. Tous les tests T8.1-T8.5 passent (9 tests). Le déterminisme est validé (même graine → même date de grève).
- Gain : Fermeture du noyau réductible P1 (T1-T8). Prêt pour T9 (vie sociale) ou la suite headless T10.
- Coût : ~150 tokens.
- Origine : Session implémentation T8, commit 85af42a.
- Tests : 9 tests pass (cargo test), déterminisme validé (T8.5).
