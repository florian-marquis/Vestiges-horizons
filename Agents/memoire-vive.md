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
