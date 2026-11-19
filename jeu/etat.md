# Objectif
Branche `feat/t7-t8-griefs-realistes` — **Lot G** (remplace et complète le lot F, selon `jeu/Docs/spec/T7-T8 correctif - spec v1.3.md`) :
- **G0 (fait dans spec v1.3)** : plafond `×6` (`600`) conservé comme borne de sûreté (valeur à calibrer) ; motif corrigé par rapport au commit `eb56c45` (`×6`, `+10/j` de l'ancienne mission Lot F).
- **G1 (Micro-tâche 1 — `@agent3` `Rédacteur actuel`)** : montée du prix alimentaire sous `5` j de couverture (`< 2 500`) multiplicative **`+25 %/jour`** (troncature entière `floor(P * 125 / 100)`, plafond `600`), descente inchangée (`-10/j` à `2 500`, plancher `100`) ; rejouer S0–S4 + tableau avant/après par rapport à `eb56c45` pour S1, S2, S4 ; commit isolé.
- **G2 (Micro-tâche 2 — `@agent2` en lecture seule puis `Rédacteur actuel` après G1)** : redéfinition de `p` (`effort = floor(cout_panier * 1000 / revenu)`, `p = clamp(floor((effort - 600) * 1000 / 400), 0, 1000)`, `revenu == 0 -> 1000`, `cout_panier == 0 -> 0`) ; contrôle jour 0 `[802, 545, 187, 0, 0]` ; règle d'arrêt S0 `< 3 ans` ; rejeu S0–S4 ; commit isolé.

# Décisions
- **Découpage en 2 micro-tâches séquentielles :**
  1. **G1 (`@agent3` Mistral Vibe, `Rédacteur actuel`)** :
     - Dans `jeu/src/bin/quartier_10ans.rs` uniquement (ne pas casser `Market::auto_reprice()` ni `test_tranche5` / `market::tests`) : remplacer l'appel `market_state.auto_reprice()` par une fonction locale déterministe 100 % entière (`stock < MAX_FOOD_STOCK -> displayed_price = (displayed_price * 125 / 100).min(MAX_FOOD_PRICE)`, `stock >= MAX_FOOD_STOCK -> displayed_price.saturating_sub(FOOD_PRICE_STEP).max(BASE_FOOD_PRICE)` ; arrondi par troncature entière déclaré). Mettre à jour le test unitaire de reprice dans `quartier_10ans.rs`.
     - Vérifier `cargo test` complet vert (241 tests), `test_replais` T1–T5 intacts, exécuter `quartier_10ans` pour produire les mesures **avant (`eb56c45`) / après (`G1`)** sur `S1`, `S2`, `S4` (et vérifier `S0`/`S3`), puis faire le **commit isolé** `feat(t7-t8): passe la hausse du prix en penurie a +25% par jour`.
  2. **G2 (`@agent2` GitHub Copilot Fusion, lecture seule pendant G1 puis `Rédacteur actuel` après G1)** :
     - Préparer en lecture seule l'audit des sites de `calculate_p` (`inputs.rs`, `salary.rs`, `tests/test_inputs.rs`, `quartier_10ans.rs`).
     - Dès passage de relais après G1 : modifier `calculate_p` dans `jeu/src/inputs.rs`, vérifier que `replai_gen.rs` / `T1–T5` ne sont jamais touchés, mettre à jour les tests unitaires de `calculate_p` (`inputs.rs`, `salary.rs`, `tests/test_inputs.rs`) dont les valeurs de contrôle au jour 0 **`[802, 545, 187, 0, 0]`** et l'égalité de source panier/revenu avec `budget_nourriture`, vérifier `cargo test` + `test_replais` verts, exécuter `quartier_10ans` (**S0** avec règle d'arrêt `< 1 095` j puis **S1–S4**), faire le **commit isolé** `feat(t7-t8): redefinit p sur le taux d'effort au-dela de 60%` et rapporter les tableaux finaux S0–S4.
- **Règles dures inchangées :** replais `jeu/src/replai/*.json` (T1–T5) en lecture seule (STOP immédiat si un replai change ou appelle `calculate_p`) ; pas de tranche B ; interdit de toucher `src/strike.rs`, `calculer_taux_risque`, `src/labor.rs`, `meal.rs`, I8 ; zéro `f64`/`exp(`.

# TODO
- [x] **G0** : mission Lot F révisé / Lot G identifiée (`eb56c45` = ancienne mission Lot F) ; `spec v1.3.md` mise à jour (`G0, G1, G2`).
- [x] **Micro-tâche G1 (`@agent3`, validée)** : hausse `+25 %/j` dans `quartier_10ans.rs`, `cargo test` vert (241 tests), `test_replais` T1–T5 intacts, commit isolé `bf6f41b`.
- [x] **Préparation lecture seule G2 (`@agent2`)** : audit des appels et tests terminé.
- [x] **Micro-tâche G2 (`@agent2`, commit `6501e43`)** : `calculate_p` redéfini en `u128` autour de `e0 = 600`, contrôle jour 0 `[802, 545, 187, 0, 0]`, `cargo test` 241/241, `test_replais` 3/3, S0–S4 exécutés sans déclencher le garde-fou.

# Rédacteur actuel
Inactif sur le code — G2 terminé, en attente de validation de @pilote

# Destinataire suivant
@pilote (bilan/validation) ; @agent2 en veille
