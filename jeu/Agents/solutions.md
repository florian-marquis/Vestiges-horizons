# Journal des solutions — registre froid

Manipulation régie par `Agents/PROTOCOLE-ERREURS.md` : lu uniquement sur incident, enrichi uniquement à la résolution d'un problème. Une entrée par problème, au format de la section 1.

---

### P-1 — Incohérence fichier après write_file
- Symptôme : `wc -l` ou `cat` affiche le ancien contenu du fichier après un `write_file` réussie. Le fichier semble inchangé malgré la confirmation d'écriture.
- Contexte : Fichier `src/lib.rs` pendant la Tranche 1. Commande `write_file` retourne succès, mais la lecture montre l'ancien contenu ou un contenu partiel.
- Cause : Conflit de cache dans l'environnement d'exécution temporaire. Le système de fichiers de la sandbox conserve une version en cache qui n'est pas synchronisée avec Nextcloud/le filesystem réel.
- Résolution : Effectuer un `git add <fichier>` suivi d'un `git commit` force la synchronisation du cache. Après commit, le contenu est correctement persistant et lisible.
- Occurrences : 2 (session Tranche 1, jour 2)
- Statut MV : **promu** (2024-11-13) → mémoire-vive.md

---

<!-- Nouvelles entrées ici, numérotation séquentielle P-2, P-3, ... -->
