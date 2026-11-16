# Vestiges Horizons - Plan de Tranches Verticales T7-T8 v1.1
**Phase 2, Étape 1 : Plan d'exécution**
**Date** : 14 novembre 2026
**Version** : 1.1 (réorganisé selon spec v1.1)
**Statut** : À valider avec la spécification

> **Règle** : Une tranche = une entité, quelques événements, un test.
> **Ordre** : Chaque tranche doit être validée avant de passer à la suivante.
> **Aucune implémentation** avant validation de la spécification v1.1.

---

## 🎯 **PRINCIPE**

**Tranche verticale** = fonctionnalité complète de bout en bout :
1. **Une entité** (ou modification d'entité)
2. **Quelques événements** associés
3. **Un ou plusieurs tests** qui échouent sur le code actuel et passent après correction
4. **Aucune modification des replais de référence** (règle 6)

**Nouvel ordre** (selon spec v1.1 B.10) :
- A : Infrastructure (une cohorte, G, bandes, époque)
- B : Risque et tirage
- C : Grèves
- D : I8 et extraction
- E : Salaire réel
- F : Benchmark et scénario en données

---

## 📦 **TRANCHE A : Infrastructure de base**
**Objectif** : Implémenter une cohorte, G en µpts, bandes de λ, époque.

### 📌 Périmètre
- `CohortId` : identifiant stable avec dérive déterministe
- `GriefsState` : G en µpts [0 ; 100_000_000], u/p/q [0 ; 1000], last_update
- Coefficients de pente en **données** (pas en code)
- Bandes de λ en **données** (unités/jour : 0 ; 50 000 ; 200 000 ; 500 000)
- `calculate_slope` : pente en µpt/jour (tout en entiers)
- `lambda_for_g` : λ en unités/jour
- `advance_to` : intégration unifiée G et R
- Événement `InputChanged`

### 📝 Modifications de code
| Fichier | Modifications |
|---------|---------------|
| `src/entities.rs` | Ajouter `CohortId` avec `derive_child` |
| `src/salary.rs` | Remplacer `SalaireReelEtGriefs` par `GriefsState` |
| `src/lib.rs` | Ajouter types pour événements T7-T8 |
| `src/data/` | Créer `grief_coefficients.json`, `risk_bands.json` |

### ✅ Tests (doivent échouer sur code actuel, passer après)
```rust
// Test 1: Intégration exacte
#[test] fn test_g_integration_exact() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    state.u = 100; state.p = 100; state.q = 100;
    state.last_update = Timestamp::new(0, 0);
    
    let coeffs = load_grief_coefficients();
    advance_to(&mut state, Timestamp::new(10, 0), &coeffs);
    
    // pente = 100*100 + 25*100 - 30*100 - 20_000 = 7500 µpt/jour
    // G = 0 + 7500 * 10 = 75_000 µpts
    assert_eq!(state.g, 75_000);
}

// Test 2: Clamp maximum
#[test] fn test_g_clamp_max() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    state.g = 100_000_000; // max
    state.u = 1000; state.p = 1000; state.q = 0; // pente positive
    
    let coeffs = load_grief_coefficients();
    advance_to(&mut state, Timestamp::new(1, 0), &coeffs);
    
    assert_eq!(state.g, 100_000_000); // toujours max
}

// Test 3: Bandes de λ
#[test] fn test_lambda_bands() {
    assert_eq!(lambda_for_g(0), 0);
    assert_eq!(lambda_for_g(30_000_000), 50_000);
    assert_eq!(lambda_for_g(60_000_000), 200_000);
    assert_eq!(lambda_for_g(80_000_000), 500_000);
    assert_eq!(lambda_for_g(100_000_000), 500_000);
}

// Test 4: État par cohorte
#[test] fn test_per_cohort_state() {
    let state1 = GriefsState::new(cohort1, seed);
    let state2 = GriefsState::new(cohort2, seed);
    
    // ... modifier u/p/q différemment
    advance_to(&mut state1, Timestamp::new(1, 0), &coeffs);
    advance_to(&mut state2, Timestamp::new(1, 0), &coeffs);
    
    assert_ne!(state1.g, state2.g);
}

// Test 5: Époque
#[test] fn test_epoch_increment() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    let initial_epoch = state.epoch;
    
    advance_to(&mut state, Timestamp::new(1, 0), &coeffs);
    assert_eq!(state.epoch, initial_epoch + 1);
}
```

### 🎯 Critère de sortie
- ✅ Tous les tests de la tranche passent
- ✅ Pas de modification des replais
- ✅ Benchmark P1 toujours valide

---

## 📦 **TRANCHE B : Risque et tirage**
**Objectif** : Implémenter accumulation du risque, tirage exponentiel, planification.

### 📌 Périmètre
- `risk_accumulated` : en unités de risque (1 événement = 365 000 000 unités)
- `draw_strike_threshold` : tirage exponentiel avec table 256 entrées + interpolation
- `draw_strike_duration` : durée aléatoire avec hash nommé (xxHash)
- `StrikeThresholdCrossed` : événement de déclenchement
- Planification exacte : `Δt = ceil((seuil − G)/pente)` et `ceil((S − R)/λ)`
- Époque sur les événements

### 📝 Modifications de code
| Fichier | Modifications |
|---------|---------------|
| `src/strike.rs` | Remplacer par nouveau système de tirage |
| `src/lib.rs` | Ajouter `StrikeThresholdCrossed` |
| `src/data/` | Ajouter `exponential_table.json` (256 entrées) |
| `src/utils/` | Créer `stable_hash.rs` (xxHash) |

### ✅ Tests
```rust
// Test 1: Tirage exponentiel
#[test] fn test_exponential_draw() {
    let threshold = draw_strike_threshold(seed, cohort_id, 0);
    assert!(threshold <= 365_000_000); // 1 événement max
    
    // Déterminisme
    assert_eq!(draw_strike_threshold(seed, cohort_id, 0), threshold);
}

// Test 2: Distribution (approximative)
#[test] fn test_exponential_distribution() {
    let mut counts = vec![0; 10];
    for i in 0..1000 {
        let t = draw_strike_threshold(42, CohortId::new(42, i), 0);
        let bucket = (t * 10 / 365_000_000) as usize;
        counts[bucket.min(9)] += 1;
    }
    // Vérifier que la distribution est décroissante
    for i in 0..8 {
        assert!(counts[i] > counts[i + 1]);
    }
}

// Test 3: Planification exacte
#[test] fn test_exact_scheduling() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    state.g = 0;
    state.u = 100; state.p = 0; state.q = 0;
    // pente = 100*100 - 20_000 = 8000 µpt/jour
    
    let coeffs = load_grief_coefficients();
    let target_g = 30_000_000;
    
    // Δt = ceil((30M - 0) / 8000) = ceil(3750) = 3750 jours
    let mut date = Timestamp::new(0, 0);
    let mut remaining = target_g;
    let slope = calculate_slope(&coeffs, state.u, state.p, state.q);
    let dt = (remaining as i128 + slope - 1) / slope as i128;
    
    date.day += dt as i64;
    advance_to(&mut state, date, &coeffs);
    
    assert_eq!(state.g, 30_000_000);
}
```

### 🎯 Critère de sortie
- ✅ Tous les tests passent
- ✅ Déterminisme vérifié
- ✅ Planification exacte validée

---

## 📦 **TRANCHE C : Grèves**
**Objectif** : Implémenter le cycle complet des grèves.

### 📌 Périmètre
- `StrikeState` : Idle, Ongoing, Ending
- `StrikeEnded` : événement de fin
- **λ = 0 pendant StrikeState::Ongoing**
- Réinitialisation de `risk_accumulated` à la fin de grève
- Nouveau seuil tiré **après chaque grève** (jamais 0, jamais paresseusement)

### 📝 Modifications de code
| Fichier | Modifications |
|---------|---------------|
| `src/strike.rs` | Nouveau système de gestion des grèves |
| `src/lib.rs` | Ajouter `StrikeEnded` |

### ✅ Tests
```rust
// Test 1: Déclenchement
#[test] fn test_strike_trigger() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    state.risk_accumulated = state.strike_threshold;
    
    assert!(check_strike_trigger(&state));
}

// Test 2: λ = 0 pendant grève
#[test] fn test_lambda_zero_during_strike() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    state.strike_state = StrikeState::Ongoing;
    state.g = 80_000_000; // devrait donner λ = 500_000
    
    let lambda = effective_lambda(&state);
    assert_eq!(lambda, 0);
}

// Test 3: Fin de grève
#[test] fn test_strike_end() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    state.strike_state = StrikeState::Ongoing;
    state.risk_accumulated = 100_000_000;
    
    end_strike(&mut state, Timestamp::new(10, 0), StrikeEndCause::DurationElapsed);
    
    assert_eq!(state.risk_accumulated, 0);
    assert!(matches!(state.strike_state, StrikeState::Ending));
    assert_ne!(state.strike_threshold, 0); // Nouveau seuil tiré
}

// Test 4: Cycle complet
#[test] fn test_full_strike_cycle() {
    let mut state = GriefsState::new(cohort_id, world_seed);
    
    // Déclencher
    state.risk_accumulated = state.strike_threshold;
    trigger_strike(&mut state, Timestamp::new(10, 0));
    assert!(matches!(state.strike_state, StrikeState::Ongoing));
    
    // Finir
    end_strike(&mut state, Timestamp::new(20, 0), StrikeEndCause::Concession);
    assert!(matches!(state.strike_state, StrikeState::Ending));
    
    // Vérifier nouveau seuil
    let old_threshold = state.strike_threshold;
    state.risk_accumulated = old_threshold;
    trigger_strike(&mut state, Timestamp::new(30, 0));
    assert_ne!(state.strike_threshold, old_threshold);
}
```

### 🎯 Critère de sortie
- ✅ Tous les tests passent
- ✅ Cycle complet validé

---

## 📦 **TRANCHE D : I8 et extraction**
**Objectif** : Implémenter I8 comme propriété avec conservation stricte.

### 📌 Périmètre
- `CohortGriefs` : total, population, named_count, anonymous_count
- Extraction : individu avec grief = G (δ = 0)
- Réinjection : conservation stricte
- I8 comme **propriété** (toujours vrai par construction)
- **Migration** : signaler le lien avec Cohort.total_griefs existant

### 📝 Modifications de code
| Fichier | Modifications |
|---------|---------------|
| `src/entities.rs` | Remplacer `Cohort.total_griefs` par `CohortGriefs` |
| `src/entities.rs` | Modifier `extract` et `reinject` pour conservation stricte |
| `src/entities.rs` | `verify_i8` devient un test de propriété |

### ✅ Tests
```rust
// Test 1: Conservation stricte
#[test] fn test_strict_conservation() {
    let mut griefs = CohortGriefs::new();
    griefs.anonymous_total = 100;
    griefs.anonymous_count = 7;
    griefs.population = 7;
    griefs.total = 100;
    
    let total_before = griefs.total;
    
    let individual = extract_individual(&mut griefs, 50_000_000); // G = 50
    
    let total_after = griefs.total + individual.grief;
    assert_eq!(total_before, total_after);
}

// Test 2: Totaux non divisibles
#[test] fn test_non_divisible_totals() {
    let mut griefs = CohortGriefs::new();
    griefs.anonymous_total = 100;
    griefs.anonymous_count = 7;
    griefs.population = 7;
    griefs.total = 100;
    
    let individual = extract_individual(&mut griefs, 50_000_000);
    
    // grief = floor(100 / 7) = 14
    assert_eq!(individual.grief, 14);
    // anonymous_total = 100 - 14 = 86
    assert_eq!(griefs.anonymous_total, 86);
    assert_eq!(griefs.total, 86);
}

// Test 3: Propriété I8
#[test] fn test_i8_property() {
    let mut griefs = CohortGriefs::new();
    let g = 50_000_000;
    
    // 100 individus à G
    griefs.total = 100 * g as i64;
    griefs.population = 100;
    griefs.anonymous_count = 100;
    
    // Extraire 10 individus
    let mut individuals = Vec::new();
    for _ in 0..10 {
        individuals.push(extract_individual(&mut griefs, g));
    }
    
    // I8 doit toujours être vrai
    assert!(griefs.verify_i8(&individuals, g));
    
    // Réinjecter avec des griefs modifiés
    let modified = NamedIndividual {
        grief: g as i64 + 1000,
        ..individuals.pop().unwrap()
    };
    reinject_individual(&mut griefs, modified);
    
    assert!(griefs.verify_i8(&individuals, g));
}

// Test 4: Propriété sur 50 graines
#[test] fn test_i8_property_50_seeds() {
    for seed in 0..50 {
        let mut griefs = CohortGriefs::new();
        let g = (seed * 1_000_000) as u64;
        
        // ... générer des séquences aléatoires mais déterministes
        // ... extractions, réinjections, évolutions
        
        assert!(griefs.verify_i8(&[], g));
    }
}
```

### 🎯 Critère de sortie
- ✅ Tous les tests passent
- ✅ I8 valide comme propriété
- ✅ Migration depuis Cohort.total_griefs signalée

---

## 📦 **TRANCHE E : Salaire réel**
**Objectif** : Implémenter salaire réel avec validation et lien vers p.

### 📌 Périmètre
- `RealWage` : value (millièmes), basket_cost (centimes), hourly_wage (centimes), hours_worked
- Calcul du salaire réel avec **erreur explicite** si basket_cost = 0
- Courbe p = f(salaire_reel) en **données** avec interpolation entière
- **Unités cohérentes** : tout en centimes ou millièmes

### 📝 Modifications de code
| Fichier | Modifications |
|---------|---------------|
| `src/salary.rs` | Ajouter `RealWage` avec validation |
| `src/salary.rs` | Supprimer `SalaireReelEtGriefs` (remplacé par GriefsState + RealWage) |
| `src/data/` | Créer `p_curve.json` (courbe en données) |

### ✅ Tests
```rust
// Test 1: Rejet si panier = 0
#[test] fn test_zero_basket_rejection() {
    let result = RealWage::calculate(1000, 8, 0);
    assert!(matches!(result, Err(ValidationError::ZeroBasketCost)));
}

// Test 2: Calcul
#[test] fn test_real_wage_calculation() {
    let wage = RealWage::calculate(1000, 8, 8000).unwrap();
    // (1000 * 8 * 100_000) / 8000 = 100_000 (100%)
    assert_eq!(wage, 100_000);
}

// Test 3: Saturation (après division)
#[test] fn test_saturation() {
    let wage = RealWage::calculate(u64::MAX, u8::MAX, 1).unwrap();
    assert_eq!(wage, i64::MAX);
}

// Test 4: Courbe p
#[test] fn test_p_curve() {
    assert_eq!(calculate_p_from_real_wage(0), 1000);      // ≤ 50%
    assert_eq!(calculate_p_from_real_wage(50_000), 1000); // ≤ 50%
    assert_eq!(calculate_p_from_real_wage(100_000), 500);  // 100%
    assert_eq!(calculate_p_from_real_wage(150_000), 200);  // 150%
    assert_eq!(calculate_p_from_real_wage(250_000), 0);     // ≥ 250%
}
```

### 🎯 Critère de sortie
- ✅ Tous les tests passent
- ✅ Validation des données intégrée

---

## 📦 **TRANCHE F : Benchmark et validation finale**
**Objectif** : Intégrer au noyau, valider tous les critères.

### 📌 Périmètre
- Remplacer la boucle quotidienne par des événements
- Connecter à `EventQueue` de lib.rs
- Scénario du benchmark avec **entrées plausibles en données** (pas de modulos)
- Valider que benchmark P1 **reste inchangé**

### ✅ Tests (critères de réussite)

**Critère 1 : Zéro grève sans stress**
```rust
#[test] fn test_criterion_1() {
    // G = 0, salaire = panier, u = p = q = 0, 50 ans
    // → zéro grève sur 20 graines
    let mut results = Vec::new();
    for seed in 0..20 {
        let greves = run_simulation(seed, |_| { /* u=p=q=0 */ });
        results.push(greves);
    }
    assert!(results.iter().sum::<u64>() == 0);
}
```

**Critère 2 : Pénurie durable**
```rust
#[test] fn test_criterion_2() {
    // u = 800 (constant), p = 0, q = 0
    // → moyenne empirique = intégrale analytique de λ ±20 % sur 50 graines
    let mut total_greves = 0;
    let lambda_integral = calculate_analytical_integral(800, 0, 0, 365 * 50);
    
    for seed in 0..50 {
        let greves = run_simulation(seed, |_| { /* u=800 */ });
        total_greves += greves;
    }
    
    let average = total_greves as f64 / 50.0;
    let tolerance = lambda_integral * 0.2;
    assert!((average - lambda_integral).abs() <= tolerance);
}
```

**Critère 3 : Invariance par scission**
```rust
#[test] fn test_criterion_3() {
    // Scinder un segment constant en deux
    // → ne change ni G ni la date de franchissement
    let original = run_simulation_with_segment(42, &segment);
    let (seg1, seg2) = split_segment(&segment);
    let split = run_simulation_with_segments(42, &[seg1, seg2]);
    
    assert_eq!(original.g, split.g);
    assert_eq!(original.franchissement_date, split.franchissement_date);
}
```

**Critère 4 : Proportionnalité événements/entrées**
```rust
#[test] fn test_criterion_4() {
    // Nombre d'événements traité ∝ aux changements d'entrée
    // + franchissements de bande + grèves, pas aux jours simulés
    let event_count = count_events(&simulation);
    let input_count = count_input_changes(&simulation);
    let band_crossings = count_band_crossings(&simulation);
    let strikes = count_strikes(&simulation);
    
    let expected = input_count + band_crossings + strikes;
    assert!(event_count == expected);
}
```

**Critère 5 : I8 en test de propriété**
```rust
// Déjà dans Tranche D
```

**Critère 6 : Cas limites**
```rust
#[test] fn test_criterion_6_limits() {
    // u = p = 1000, q = 0
    // G = 100_000_000 (max)
    // G = 0 avec soulagement
    // salaire très élevé (saturation)
    // panier = 0 (rejeté)
    
    // Test u=p=1000, q=0
    let mut state = GriefsState::new(cohort_id, seed);
    state.u = 1000; state.p = 1000; state.q = 0;
    
    let coeffs = load_grief_coefficients();
    advance_to(&mut state, Timestamp::new(1000, 0), &coeffs);
    assert_eq!(state.g, 100_000_000); // saturation
    
    // Test panier = 0
    let result = RealWage::calculate(1000, 8, 0);
    assert!(matches!(result, Err(ValidationError::ZeroBasketCost)));
}
```

**Critère 7 : Indépendance des cohortes**
```rust
#[test] fn test_criterion_7() {
    // Deux cohortes aux entrées différentes évoluent indépendamment
    let mut state1 = GriefsState::new(cohort1, seed);
    let mut state2 = GriefsState::new(cohort2, seed);
    
    state1.u = 100; state1.p = 100; state1.q = 100;
    state2.u = 500; state2.p = 500; state2.q = 500;
    
    let coeffs = load_grief_coefficients();
    advance_to(&mut state1, Timestamp::new(10, 0), &coeffs);
    advance_to(&mut state2, Timestamp::new(10, 0), &coeffs);
    
    assert_ne!(state1.g, state2.g);
}
```

**Critère 8 : Performance**
```rust
#[test] fn test_criterion_8() {
    // 100 cohortes × 100 ans
    let start = Instant::now();
    
    let mut states = Vec::new();
    for i in 0..100 {
        let mut state = GriefsState::new(CohortId::new(42, i), 42);
        // ... simulation 100 ans
        states.push(state);
    }
    
    let elapsed = start.elapsed();
    let events_per_second = total_events as f64 / elapsed.as_secs_f64();
    let ms_per_year = elapsed.as_millis() as f64 / (100 * 100) as f64;
    
    // Critère : > 10 000 événements/seconde, < 10 ms/an simulé
    assert!(events_per_second > 10_000.0);
    assert!(ms_per_year < 10.0);
}
```

**Critère 9 : Scénario benchmark**
```rust
#[test] fn test_criterion_9() {
    // Scénario avec entrées plausibles en données
    // (remplace les modulos du benchmark original)
    let entries = load_plausible_entries(); // depuis données
    
    let original = run_benchmark_original();
    let new = run_benchmark_with_entries(&entries);
    
    // Benchmark P1 doit rester inchangé
    assert!(original == new);
}
```

**Critère 10 : Suite de 1000 InputChanged = oracle jour par jour**
```rust
#[test] fn test_criterion_10() {
    // Générer une suite de 1000 InputChanged aléatoires
    let inputs = generate_random_inputs(1000, 42);
    
    // Version « avance jour par jour » (oracle)
    let oracle = run_oracle_simulation(&inputs);
    
    // Version « événementielle »
    let event_based = run_event_based_simulation(&inputs);
    
    // Résultats identiques
    assert_eq!(oracle.final_g, event_based.final_g);
    assert_eq!(oracle.final_r, event_based.final_r);
    assert_eq!(oracle.strike_count, event_based.strike_count);
}
```

### 🎯 Critère de sortie
- ✅ Tous les critères 1-10 passent
- ✅ Benchmark P1 inchangé
- ✅ Aucun replai modifié

---

## 📅 **CALENDRIER ESTIMÉ**

| Tranche | Complexité | Effort | Durée | Priorité |
|---------|------------|--------|-------|----------|
| A | Moyenne | 3h | 1/2 j | 1 |
| B | Élevée | 5h | 1 j | 2 |
| C | Élevée | 3h | 1/2 j | 3 |
| D | Moyenne | 3h | 1/2 j | 4 |
| E | Faible | 2h | 1/4 j | 5 |
| F | Élevée | 4h | 1/2 j | 6 |
| **Total** | | **20h** | **2-3 j** | |

---

## 🎯 **RÈGLES D'EXÉCUTION**

1. **Une tranche à la fois** : Ne pas commencer N+1 avant validation de N
2. **Tests d'abord** : Écrire les tests **avant** l'implémentation
3. **Pas de commit** : Ne pas commiter avant validation explicite
4. **Pas de modification des replais** : Si nécessaire, **signaler et s'arrêter**
5. **Validation continue** : Exécuter `cargo test` + benchmark P1 après chaque tranche

---

## ✅ **VALIDATION**

**À faire avant de commencer** :
- [ ] Valider le diagnostic v3.1
- [ ] Valider la spécification v1.1
- [ ] Valider ce plan v1.1
- [ ] Confirmer que labor.rs sera traité en ticket séparé

**Attendez ma validation avant toute implémentation.**

---

*Document généré par Agent IA - Phase 2, Étape 1 : Plan de tranches v1.1*
