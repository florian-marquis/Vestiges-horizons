# Vestiges Horizons - Plan de Tranches Verticales T7-T8
**Phase 2, Étape 1 : Plan d'exécution**
**Date** : 14 novembre 2026
**Version** : 1.0
**Statut** : À valider avec la spécification

> **Règle** : Une tranche = une entité, quelques événements, un test.
> **Ordre** : Chaque tranche doit être validée avant de passer à la suivante.
> **Aucune implémentation** avant validation de la spécification et de ce plan.

---

## 🎯 **PRINCIPE**

Chaque tranche est **verticale** : elle implémente une fonctionnalité complète de bout en bout, avec :
1. Une **entité** (ou modification d'entité)
2. Quelques **événements** associés
3. Un ou plusieurs **tests** qui échouent sur le code actuel et passent après correction
4. **Aucune modification des replais de référence** (règle 6)

---

## 📋 **ORDRE D'EXÉCUTION**

### **Prérequis** : Validation de la spécification `Docs/spec/T7-T8 correctif - spec.md`

---

## 📦 **TRANCHE A : Infrastructure de base des griefs**

**Objectif** : Créer la base pour le stockage et l'intégration de G en micro-points.

### Entités
- `CohortId` : Identifiant stable (graine + numéro unique)
- `GriefsState` : G en µpts [0 ; 100_000_000], u/p/q [0 ; 1000], last_input_change, epoch

### Événements
- `InputChanged` : Changement d'entrée (u, p, q)

### Fonctions
- `calculate_slope(u, p, q)` : Pente en µpt/jour avec coefficients en données
- `update_g(state, current_date)` : G += pente × Δt, clamp [0 ; max]

### Tests
1. **Test d'intégration exacte** :
   ```rust
   #[test]
   fn test_g_integration() {
       let mut state = GriefsState::new();
       state.u = 100; state.p = 100; state.q = 100;
       state.last_input_change = Timestamp::new(0, 0);
       
       update_g(&mut state, Timestamp::new(10, 0));
       // pente = 100*100 + 25*100 - 30*100 - 20_000 = 7500 µpt/jour
       // G = 0 + 7500 * 10 = 75_000 µpts
       assert_eq!(state.g, 75_000);
   }
   ```

2. **Test du clamp** :
   ```rust
   #[test]
   fn test_g_clamp() {
       let mut state = GriefsState::new();
       state.g = 100_000_000; // max
       state.u = 0; state.p = 0; state.q = 0; // pente négative
       
       update_g(&mut state, Timestamp::new(1, 0));
       assert_eq!(state.g, 100_000_000); // toujours max
   }
   ```

3. **Test de l'état par cohorte** :
   ```rust
   #[test]
   fn test_per_cohort_state() {
       let mut state1 = GriefsState::new();
       let mut state2 = GriefsState::new();
       state1.u = 100; state2.u = 200;
       
       update_g(&mut state1, Timestamp::new(1, 0));
       update_g(&mut state2, Timestamp::new(1, 0));
       
       assert_ne!(state1.g, state2.g);
   }
   ```

### Critère de sortie
- ✅ Tous les tests de la tranche passent
- ✅ Pas de modification des replais
- ✅ Benchmark P1 toujours valide

---

## 📦 **TRANCHE B : Dynamique du risque et déclenchement**

**Objectif** : Implémenter l'accumulation du risque et le déclenchement des grèves.

### Entités (extension)
- Ajouter `risk_accumulated`, `strike_threshold`, `draw_index`, `strike_state` à `GriefsState`

### Événements
- `StrikeThresholdCrossed` : Seuil de grève franchi

### Fonctions
- `calculate_lambda(g)` : Taux λ par bandes (données)
- `update_risk(state, current_date)` : R += λ × Δt
- `draw_strike_threshold(key)` : Tirage exponentiel en entiers
- `draw_strike_duration(key)` : Tirage de la durée
- `check_strike_trigger(state)` : R ≥ S ?
- `trigger_strike(state, current_date)` : Déclencher une grève

### Tests
1. **Test du taux λ par bandes** :
   ```rust
   #[test]
   fn test_lambda_bands() {
       assert_eq!(calculate_lambda(0), 0);      // < 30
       assert_eq!(calculate_lambda(30_000_000), LAMBDA_30); // 30-60
       assert_eq!(calculate_lambda(80_000_000), LAMBDA_80); // > 80
   }
   ```

2. **Test du tirage exponentiel** :
   ```rust
   #[test]
   fn test_exponential_draw() {
       // Vérifier que le tirage donne des valeurs dans [0, 10⁹]
       let key = "42-1-strike-0";
       let threshold = draw_strike_threshold(key);
       assert!(threshold <= 1_000_000_000);
       
       // Vérifier le déterminisme
       assert_eq!(draw_strike_threshold(key), threshold);
   }
   ```

3. **Test du déclenchement** :
   ```rust
   #[test]
   fn test_strike_trigger() {
       let mut state = GriefsState::new();
       state.risk_accumulated = 500_000_000;
       state.strike_threshold = 500_000_000;
       
       assert!(check_strike_trigger(&state));
       
       trigger_strike(&mut state, Timestamp::new(10, 0));
       assert!(state.risk_accumulated == 0); // Réinitialisé
       assert!(state.draw_index == 1);      // Incrémenté
   }
   ```

4. **Test de la cohérence avec le diagnostic** :
   ```rust
   #[test]
   fn test_no_strike_with_zero_input() {
       // G = 0, u = p = q = 0, 50 ans
       // → zéro grève
       // À implémenter après l'intégration
   }
   ```

### Critère de sortie
- ✅ Tous les tests de la tranche passent
- ✅ Déterminisme vérifié (même graine → même résultat)
- ✅ Pas de modification des replais

---

## 📦 **TRANCHE C : Gestion complète des grèves**

**Objectif** : Implémenter la fin des grèves et le cycle complet.

### Événements
- `StrikeEnded` : Fin de grève (concession ou durée)

### Fonctions
- `end_strike(state, end_day, cause)` : Terminer une grève
- `update_strike(state, current_date)` : Mettre à jour l'état de grève

### Tests
1. **Test de la fin par concession** :
   ```rust
   #[test]
   fn test_strike_end_by_concession() {
       let mut state = GriefsState::new();
       trigger_strike(&mut state, Timestamp::new(10, 0));
       
       end_strike(&mut state, 15, StrikeEndCause::Concession);
       
       assert!(matches!(state.strike_state, StrikeState::Ending { .. }));
       assert!(state.risk_accumulated == 0);
   }
   ```

2. **Test de la fin par durée** :
   ```rust
   #[test]
   fn test_strike_end_by_duration() {
       let mut state = GriefsState::new();
       state.strike_threshold = 500_000_000;
       trigger_strike(&mut state, Timestamp::new(10, 0));
       
       if let StrikeState::Ongoing { planned_end, .. } = state.strike_state {
           end_strike(&mut state, planned_end, StrikeEndCause::DurationElapsed);
       }
       
       assert!(matches!(state.strike_state, StrikeState::Ending { .. }));
   }
   ```

3. **Test du cycle complet** :
   ```rust
   #[test]
   fn test_full_strike_cycle() {
       let mut state = GriefsState::new();
       
       // 1. Pas de grève initialement
       assert!(matches!(state.strike_state, StrikeState::Idle));
       
       // 2. Déclencher une grève
       trigger_strike(&mut state, Timestamp::new(10, 0));
       assert!(matches!(state.strike_state, StrikeState::Ongoing { .. }));
       
       // 3. Terminer la grève
       if let StrikeState::Ongoing { planned_end, .. } = state.strike_state {
           end_strike(&mut state, planned_end, StrikeEndCause::DurationElapsed);
       }
       assert!(matches!(state.strike_state, StrikeState::Ending { .. }));
       
       // 4. Nouveau seuil tiré pour la prochaine grève
       assert!(state.strike_threshold > 0);
   }
   ```

### Critère de sortie
- ✅ Tous les tests de la tranche passent
- ✅ Cycle complet validé

---

## 📦 **TRANCHE D : Bandes de G et bornes**

**Objectif** : Implémenter la détection des franchissements de bandes et des bornes.

### Événements
- `BandCrossed` : Franchissement d'une bande de G (30, 60, 80)
- `GBoundHit` : G atteint 0 ou 100_000_000

### Fonctions
- `calculate_band(g)` : Retourne la bande (0: <30, 1: 30-60, 2: 60-80, 3: >80)
- `calculate_band_cross_date(state, old_band, new_band)` : Date du franchissement
- `calculate_bound_hit_date(state)` : Date d'atteinte de la borne

### Tests
1. **Test du franchissement de bande** :
   ```rust
   #[test]
   fn test_band_crossing() {
       let mut state = GriefsState::new();
       state.u = 1000; state.p = 1000; state.q = 0;
       // pente = 100*1000 + 25*1000 - 0 - 20_000 = 105_000 µpt/jour
       
       update_g(&mut state, Timestamp::new(1, 0));
       // G = 105_000 après 1 jour
       // Bandes: <30_000_000, 30-60M, 60-80M, >80M
       // 105_000 est dans la bande 0 (< 30M)
       
       // Après 300 jours: G = 105_000 * 300 = 31_500_000 (> 30M)
       update_g(&mut state, Timestamp::new(300, 0));
       assert_eq!(calculate_band(state.g), 1); // bande 30-60M
   }
   ```

2. **Test des bornes** :
   ```rust
   #[test]
   fn test_upper_bound() {
       let mut state = GriefsState::new();
       state.u = 1000; state.p = 1000; state.q = 0;
       
       // Après beaucoup de jours, G devrait saturer à 100_000_000
       update_g(&mut state, Timestamp::new(1000, 0));
       assert_eq!(state.g, 100_000_000);
   }
   ```

3. **Test de la planification** :
   ```rust
   #[test]
   fn test_event_scheduling() {
       // Vérifier que les événements BandCrossed et GBoundHit
       // sont planifiés au bon moment
   }
   ```

### Critère de sortie
- ✅ Tous les tests de la tranche passent
- ✅ Bandes et bornes correctement détectées

---

## 📦 **TRANCHE E : I8 redéfini et extraction**

**Objectif** : Implémenter la nouvelle définition de I8 avec total anonyme comme champ propre.

### Entités
- `CohortGriefs` : named_total, anonymous_total, named_count, anonymous_count

### Fonctions
- `extract_individual(cohort, griefs)` : Extraction avec floor
- `reinject_individual(cohort, griefs, individual)` : Réinjection
- `verify_i8(griefs)` : Toujours vrai par construction

### Tests
1. **Test de conservation stricte** :
   ```rust
   #[test]
   fn test_strict_conservation() {
       let mut cohort = Cohort::new();
       let mut griefs = CohortGriefs::new();
       griefs.anonymous_total = 100;
       griefs.anonymous_count = 7;
       
       let total_before = griefs.named_total + griefs.anonymous_total;
       
       let individual = extract_individual(&mut cohort, &mut griefs);
       
       let total_after = griefs.named_total + griefs.anonymous_total + individual.grief;
       assert_eq!(total_before, total_after);
   }
   ```

2. **Test avec totaux non divisibles** :
   ```rust
   #[test]
   fn test_non_divisible_totals() {
       let mut griefs = CohortGriefs::new();
       griefs.anonymous_total = 100;
       griefs.anonymous_count = 7;
       // avg = 100/7 = 14 (floor)
       
       let individual = extract_individual(&mut cohort, &mut griefs);
       assert_eq!(individual.grief, 14);
       assert_eq!(griefs.anonymous_total, 100 - 14); // 86
   }
   ```

3. **Test de propriété I8** :
   ```rust
   #[test]
   fn test_i8_property() {
       let mut griefs = CohortGriefs::new();
       // Toute séquence d'extractions/réinjections
       // ...
       assert!(griefs.verify_i8()); // Toujours vrai
   }
   ```

4. **Test de propriété avec 50 graines** :
   ```rust
   #[test]
   fn test_i8_property_50_seeds() {
       for seed in 0..50 {
           let mut griefs = CohortGriefs::new();
           // Générer des séquences aléatoires mais déterministes
           // ...
           assert!(griefs.verify_i8());
       }
   }
   ```

### Critère de sortie
- ✅ Tous les tests de la tranche passent
- ✅ I8 valide comme propriété

---

## 📦 **TRANCHE F : Salaire réel et validation**

**Objectif** : Implémenter le salaire réel avec validation et lien vers p.

### Entités
- `RealWage` : value (millièmes), basket_cost (centimes), hourly_wage (centimes), hours_worked (heures)

### Fonctions
- `RealWage::calculate()` : Calcule le salaire réel, erreur si basket_cost = 0
- `update_p_from_real_wage(real_wage, inflation)` : Met à jour p

### Tests
1. **Test de rejection si panier = 0** :
   ```rust
   #[test]
   fn test_zero_basket_rejection() {
       let result = RealWage::calculate(1000, 8, 0);
       assert!(matches!(result, Err(ValidationError::ZeroBasketCost)));
   }
   ```

2. **Test du calcul** :
   ```rust
   #[test]
   fn test_real_wage_calculation() {
       let wage = RealWage::calculate(1000, 8, 8000).unwrap();
       // (1000 * 8 * 100_000) / 8000 = 100_000 (100%)
       assert_eq!(wage, 100_000);
   }
   ```

3. **Test de la saturation** :
   ```rust
   #[test]
   fn test_saturation() {
       let wage = RealWage::calculate(u64::MAX, u8::MAX, 1).unwrap();
       assert_eq!(wage, i64::MAX);
   }
   ```

4. **Test du lien vers p** :
   ```rust
   #[test]
   fn test_p_from_real_wage() {
       let wage = RealWage::calculate(1000, 8, 8000).unwrap();
       let p = update_p_from_real_wage(&wage, 0);
       // À calibrer en données
   }
   ```

### Critère de sortie
- ✅ Tous les tests de la tranche passent
- ✅ Validation des données intégrée

---

## 📦 **TRANCHE G : Intégration au noyau et validation finale**

**Objectif** : Connecter T7-T8 à la file d'événements et valider tous les critères.

### Modifications
- Remplacer la boucle quotidienne par des événements
- Connecter à `EventQueue` de lib.rs
- Valider le benchmark P1

### Tests
1. **Critère 1** : Zéro grève sans stress
2. **Critère 2** : Pénurie durable (moyenne empirique)
3. **Critère 3** : Invariance par scission
4. **Critère 4** : Proportionnalité événements/entrées
5. **Critère 5** : I8 en test de propriété
6. **Critère 6** : Cas limites
7. **Critère 7** : Indépendance des cohortes
8. **Critère 8** : Performance
9. **Critère 9** : Scénario benchmark

### Critère de sortie
- ✅ Tous les critères de réussite passent
- ✅ Benchmark P1 inchangé
- ✅ Aucun replai modifié

---

## 📅 **CALENDRIER ESTIMÉ**

| Tranche | Complexité | Effort estimé | Durée estimée |
|---------|------------|---------------|----------------|
| A | Moyenne | 2h | 1/2 journée |
| B | Élevée | 4h | 1 journée |
| C | Moyenne | 2h | 1/2 journée |
| D | Moyenne | 3h | 1/2 journée |
| E | Moyenne | 2h | 1/2 journée |
| F | Faible | 1h | 1/4 journée |
| G | Élevée | 4h | 1 journée |
| **Total** | | **18h** | **2-3 jours** |

---

## 🎯 **RÈGLES D'EXÉCUTION**

1. **Une tranche à la fois** : Ne pas commencer la tranche N+1 avant que la tranche N soit validée
2. **Tests d'abord** : Écrire les tests **avant** l'implémentation
3. **Pas de commit** : Ne pas commiter avant validation explicite
4. **Pas de modification des replais** : Si une correction nécessite de modifier un replai, signaler et s'arrêter
5. **Validation continue** : Exécuter `cargo test` et le benchmark P1 après chaque tranche

---

## ✅ **VALIDATION**

**À faire avant de commencer** :
- [ ] Valider la spécification `Docs/spec/T7-T8 correctif - spec.md`
- [ ] Valider ce plan `Docs/validation/T7-T8 - plan-tranches.md`
- [ ] Confirmer que labor.rs sera traité en ticket séparé

**Attendez ma validation avant toute implémentation.**

---

*Document généré par Agent IA - Phase 2, Étape 1 : Plan de tranches verticales*
