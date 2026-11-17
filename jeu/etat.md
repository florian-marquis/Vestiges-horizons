# Objectif
Branche `feat/t7-t8-griefs-realistes` — Correctif T7-T8 Tranche A2 réaliste (Lots A, B, C, D) : équation `v7.8.2`, budget reste-à-vivre (Option 1), achat partiel/rationnement prorata (Mode A), consommation partielle `R/N` et scénarios de référence **S0** puis **S1–S4**. Spec complète verrouillée : `jeu/Docs/spec/T7-T8 correctif - spec v1.3.md`.

# Décisions
- **Relais Rédacteur (17/11/2026) :** `@agent3` n'ayant pas répondu, **`@agent2` (GitHub Copilot Fusion)** devient **`Rédacteur actuel`** unique sur `jeu/` pour exécuter dans l'ordre **Lots A, B, C, puis D (S0 d'abord, puis S1–S4)**. `@agent3` passe en lecture seule (ne rien modifier dans `jeu/`).
- **Verrouillage technique B.3 (prix et seuils de stock) :**
  - Stock initial `2 000` rations ; pipeline `GoodType::Food` (`délai 3 j`) pré-amorcé à `t = 0` avec `500` rations livrées aux jours `0, 1, 2`.
  - Chaque matin après arrivages, stock du marché alimentaire plafonné à `5` jours (`2 500` rations).
  - `Market::auto_reprice()` appelé une seule fois par jour **avant** les achats des cohortes, avec `stock_thresholds = (2_500, 2_499)`, `price_bounds = (100, 300)` (`P_base = 100`, plafond `3 × P_base = 300`) et `price_step = 10`.
- **Verrouillage des scénarios S0–S4 (Lot D, graine 42, 3 650 j, données figées Annexe spec v1.3) :**
  - **S0–S3 logements :** `INITIAL_HOUSING_STATES = [600, 700, 800, 900, 1000]`, `wear_rate = 0` (entretenus). En **S0, S1, S2**, dispensaire nominal (`soins = 1000`) → `q = [800, 850, 900, 950, 1000]`.
  - **S1 & S2 fenêtre choc :** arrivages effectifs réduits exactement du jour `200` au jour `289` inclus (commandes réduites `jours 197..=286` avec délai `3` j) : `250/j` (S1) et `0/j` (S2), retour à `500/j` dès le jour `290`.
  - **S3 :** sans faim (`500/j`), logements entretenus (`wear_rate = 0`), cohorte `0` avec `care_capacity = 6`, `daily_care_demand = 30` (`soins = 200`, `q = 400 < 500`) ; cohortes `1–4` à `soins = 1000`.
  - **S4 :** logements non entretenus `[600, 700, 800, 900, 1000]` avec `wear_rate = 1` ; soins déclinant linéairement (`demande = population`, `capacite(day) = (population * (3650 - day)) / 3650`, `soins` de `1000` à `0`) ; salaires gelés `[35, 40, 50, 60, 70]` et arrivages effectifs contractés à partir du jour `1 825` : `500 - (((day - 1825) * 150) / 1825)` vers `350/j` (`P` monte vers `300`).
- **Règle d'arrêt S0 :** exécuter **S0** en premier après Lots A–C ; si une cohorte atteint `G=100` en moins de `3` ans (`< 1 095` j) sans choc → **STOP** immédiat et rapport à `@pilote` sans lancer S1–S4. Sinon enchaîner **S1–S4**.
- **Règles dures :** `src/replai/*.json` (T1–T5) en lecture seule ; interdit de toucher `src/strike.rs`, risque dans `src/salary.rs`, `src/labor.rs`, I8 ; aucun `f64`/`exp(` ; `meal.rs:330` inchangé.
- **Résultats vérifiés (graine 42, 3 650 j) :** S0 passe sans déclencher le garde-fou ; `p0 = 0`, ratios budget/P `[1,22; 1,58; 2,30; 3,02; 3,74]`, `u max = 0`, `G max = 0`, aucune ration rationnée ni rupture, `P max = 100`. S1/S2/S3/S4 exécutés ; Pmax respectifs `300/300/100/300`, jours de rupture `82/86/0/401`. S3 cohorte 0 : `q=400`, `G(10a)=87 600 000 µpt`. S4 cohortes 0–1 atteignent le plafond ; cohortes 2–4 restent sous le plafond. Tableaux complets et séries `u` S1/S2 transmis au Pilote dans le salon.

# TODO
- [x] Branche `feat/t7-t8-griefs-realistes` + `cadrage v7.8.2` + `spec v1.3` verrouillée (`B.3`, `S0–S4`).
- [x] **`@agent2` (`Rédacteur actuel`) — Étape 1 (Lots A, B.1–B.4, C + tests)** :
  - Corriger consommation partielle `R/N` et `u` post-repas (`household.rs`, `inputs.rs`) ; fonction pure canonique `budget_nourriture` et cohérence `cout_panier` par cohorte (`P` unitaire + `110 + 20 + impot(c)`, même source revenu que `p`, zéro écriture `Cohort.total_money`) ; Mode A prorata + plus grand reste + `EntityId` (zéro ordre index, zéro priorité joueur) ; équation v7.8.2 `s = 180*u + 20*p + 60*(500 - q)` (`griefs.rs`).
  - Écrire/adapter les 13 tests Lot C (`1–8` recalculés + `9–13` v1.2 dont `test_replais`) + tous les tests Lot B (`cargo test` complet vert, commits atomiques).
- [x] **`@agent2` (`Rédacteur actuel`) — Étape 2 (Lot B.5 + Lot D)** :
  - Implémenter `S0` (STOP si `G=100 < 1095` j) puis `S1, S2, S3, S4` dans `quartier_10ans.rs`, commits atomiques, rapport complet + séries `u` S1/S2.
- [ ] **@pilote** : contrôle final lecture seule + présentation à l'humain.

# Rédacteur actuel
@agent2 (GitHub Copilot Fusion — Lots A, B, C, D)

# Destinataire suivant
@pilote
