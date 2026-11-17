# Objectif
T7-T8 · Tranche A2 — Phase 1 : implémenter les sources réelles de `u`, `p`, `q` en millièmes entiers `[0, 1000]` dans le cœur (`jeu/`), supprimer les décalages manuels et les `f64`, livrer le scénario `quartier_10ans` (tableau spec v1.2 §5 sur 10 ans, graine 42, choc 2) et les 8 tests headless de l'ordre de mission `jeu/Docs/spec/T7-T8 tranche A2 - ordre de mission.md`.

# Décisions
- Audit Phase 0 de Mistral Vibe (`@agent3`) validé avec 4 cadrages impératifs pour la Phase 1 :
  1. **`u` (besoins alimentaires non servis)** : `u = 0` tant que le stock alimentaire disponible est `> 0` ou si `demande == 0` ; sinon `u = ((non_servis * 1000) / demande).min(1000) as u16` (troncature entière, convention 26).
  2. **`p` (pression des prix — correction sur la piste B2 du rapport)** : ne **pas** utiliser `displayed_price / base_price` (qui venait du benchmark synthétique). Appliquer la formule de la Spec P1 §6.2 depuis le panier et le revenu disponible (`src/salary.rs`) : si `cout_panier == 0` ou `revenu_disponible >= cout_panier`, `p = 0` ; sinon `p = (((cout_panier - revenu_disponible) * 1000) / cout_panier).min(1000) as u16` (troncature entière).
  3. **`q` (qualité des services, D1 = option 2)** : moyenne pondérée entière par table de données `[(Soins, 500), (Logements, 500)]`. Taux de soins `[0, 1000]` issu du dispensaire (`health.rs`), taux des logements `[0, 1000]` issu de l'état d'entretien des actifs logements (`asset.rs` / `maintenance.rs`). Toute composante sans donnée réelle est exclue de la moyenne (jamais comptée comme 0).
  4. **Architecture et interdits** : fichier unique `jeu/src/inputs.rs`, exposé dans `jeu/src/lib.rs`. Interdiction stricte de modifier `src/strike.rs`, le calcul de risque dans `src/salary.rs`, ou les replais JSON de référence.
- **Revue Pilote du Lot 1 (`@agent3` Mistral Vibe)** :
  - `cargo test` vert (117 lib + 28 `test_inputs` + 3 `test_replais` + tranches 2/3/4/5/7/8) et `src/strike.rs`, risque dans `src/salary.rs`, `src/replai/*.json` intacts.
  - **2 ajustements mineurs requis pendant le Lot 2 (ou pris en charge dans Lot 2)** :
    1. `DispensaryQueue::calculate_service_rate()` dans `health.rs` calcule `(capacity - queue.len()) / capacity` (il tombe à 0 si `queue.len() >= capacity` alors qu'un dispensaire de capacité 10 sert bien 10 patients/jour sur une file de 10 !). Or la sémantique réelle est : si `patients_en_attente == 0` → `1000` ; sinon s'il y a `N` patients demandeurs, servis = `min(capacity, N)` → taux = `(min(capacity, N) * 1000 / N)`. Corriger en vérifiant s'il faut distinguer `demande_soins` vs `queue.len()` (ou si `capacity` sert jusqu'à `capacity` patients parmi `patients_en_attente + traites`).
    2. Dans `inputs.rs`, ajouter un test #6 statique réel (par ex. qui lit `src/inputs.rs` et vérifie `!source.contains("f64") && !source.contains("exp(")`) et un émetteur sur franchissement de seuil (`InputTracker` ou fonction qui n'émet `InputChanged` que si `(u, p, q)` a changé au moins d'1 millième, au plus 1 fois par cohorte et par créneau — cf. ordre de mission §3).
- Plan Lot 2 validé par le Pilote (`seq=4`) : binaire dédié `src/bin/quartier_10ans.rs`, graine 42, 3 650 jours, choc 2 réel, tableau spec v1.2 §5 + tests #3, #7, #8.
- Répartition des rôles Duo :
  - `@pilote` (VS Code) : Architecte (lecture seule sur `jeu/src/`, pilotage et revue).
  - `@agent1` (Cursor) : Secours / escalade uniquement.
  - `@agent2` (GitHub Copilot Fusion, VS Code) : **`Rédacteur actuel`** pour Lot 2 + 2 corrections mineures du Lot 1.
  - `@agent3` (Mistral Vibe, VS Code) : En veille (lecture seule / relecture si demandé).

# TODO
- [x] **Lot 1 (`@agent3` Mistral Vibe)** : `jeu/src/inputs.rs`, accesseurs, `tests/test_inputs.rs`, `cargo test` vert.
- [x] **Audit Lot 2 (`@agent2` GitHub Copilot Fusion)** : Audit en lecture seule de `benchmark_t7_t8.rs` et `benchmark_p1.rs`.
- [x] **Lot 1 (`@agent3` Mistral Vibe)** : `jeu/src/inputs.rs`, accesseurs, `tests/test_inputs.rs`, `cargo test` vert.
- [x] **Audit Lot 2 (`@agent2` GitHub Copilot Fusion)** : Audit en lecture seule de `benchmark_t7_t8.rs` et `benchmark_p1.rs`.
- [x] **Lot 2 + finitions Lot 1 (`@agent2` GitHub Copilot Fusion)** :
  1. Correction du taux de soins (`health.rs`), test statique #6 réel (`tests/test_inputs.rs`), `InputTracker` (`src/inputs.rs`).
  2. Binaire `jeu/src/bin/quartier_10ans.rs` (graine 42, 3 650 jours, choc 2 de 90 jours) et tableau de mesures spec v1.2 §5.
  3. Tests #3, #7 et #8 (`test_replais`) verts ; `cargo test` complet vert (commit `5758e5a`).

# Rédacteur actuel
Aucun (Phase 1 terminée — en attente de revue humaine du tableau)

# Destinataire suivant
Humain (validation spec v1.2 §5)
