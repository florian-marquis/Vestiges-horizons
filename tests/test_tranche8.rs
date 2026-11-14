//! Tests de la Tranche 8 : Basculements (Grève)
//! 
//! Spécification : Docs/spec P1 - quartier vivant.md §20 et §35
//! Tests de fermeture : T8.1 à T8.5

use vestiges_horizons::{
    strike::*,
    entities::*,
};

// ============================================================================
// T8.1 : Basculement
// ============================================================================

/// T8.1 - Basculement : une grève est déclenchée quand le seuil est franchi
#[test]
fn test_t8_1_basculement() {
    let mut manager = StrikeManager::new(42);
    let cohort = EntityId::new(EntityType::Cohort, 1);
    
    // Déclencher une grève au jour 10
    let strike = manager.declencher_greve(cohort, 10);
    
    assert!(strike.is_some(), "T8.1 : une grève doit être créée");
    let strike = strike.unwrap();
    
    assert_eq!(strike.cohort, cohort);
    assert_eq!(strike.start_day, 10);
    assert!(strike.planned_duration >= 1 && strike.planned_duration <= 30);
    assert_eq!(strike.end_day, None);
    assert!(!strike.is_ended());
}

// ============================================================================
// T8.2 : Tirage unique (jamais re-tirable)
// ============================================================================

/// T8.2 - Tirage unique : impossible de re-tirer pour la même cohorte
/// décision 43 : le tirage est consommé au franchissement et journalisé
#[test]
fn test_t8_2_tirage_unique_never_redrawn() {
    let mut manager = StrikeManager::new(42);
    let cohort = EntityId::new(EntityType::Cohort, 1);
    
    // Premier tirage et grève
    let strike1 = manager.declencher_greve(cohort, 10);
    assert!(strike1.is_some());
    
    // Essayer de re-tirer pour la même cohorte (sans fin de grève)
    // Un nouveau tirage va être créé, mais verify_no_redraw doit le détecter
    manager.draw_strike_parameters(cohort);
    
    // Maintenant la cohorte a une grève en cours ET un tirage en attente
    // C'est une situation de re-tirage !
    assert!(!manager.verify_no_redraw(cohort), 
        "T8.2 : re-tirage détecté (grève en cours + tirage en attente)");
    
    // Après avoir mis fin à la grève, on peut re-tirer
    manager.fin_de_greve(cohort, 20, StrikeEndCause::DurationElapsed);
    assert!(manager.verify_no_redraw(cohort), 
        "T8.2 : pas de re-tirage après fin de grève");
    
    // Maintenant on peut tirer une nouvelle grève
    let strike2 = manager.declencher_greve(cohort, 30);
    assert!(strike2.is_some(), "T8.2 : nouvelle grève autorisée après fin de la précédente");
}

/// T8.2 bis - Vérification que chaque grève a un tirage unique
#[test]
fn test_t8_2_each_strike_has_unique_draw() {
    let mut manager = StrikeManager::new(42);
    let cohort = EntityId::new(EntityType::Cohort, 1);
    
    // Déclencher plusieurs grèves pour la même cohorte (à des jours différents)
    // Chaque grève doit avoir un tirage différent
    let strike1 = manager.declencher_greve(cohort, 10);
    manager.fin_de_greve(cohort, 20, StrikeEndCause::DurationElapsed);
    
    let strike2 = manager.declencher_greve(cohort, 30);
    manager.fin_de_greve(cohort, 40, StrikeEndCause::DurationElapsed);
    
    let strike3 = manager.declencher_greve(cohort, 50);
    
    assert!(strike1.is_some() && strike2.is_some() && strike3.is_some());
    
    let s1 = strike1.unwrap();
    let s2 = strike2.unwrap();
    let s3 = strike3.unwrap();
    
    // Les seuils de risque doivent être différents (ou au moins pas tous identiques)
    // Avec la même graine et cohort, mais des draw_index différents, les résultats diffèrent
    assert_ne!(s1.risk_threshold, s2.risk_threshold, 
        "T8.2 : tirages différents pour grèves différentes");
    assert_ne!(s2.risk_threshold, s3.risk_threshold, 
        "T8.2 : tirages différents pour grèves différentes");
}

// ============================================================================
// T8.3 : Fin de grève par concession
// ============================================================================

/// T8.3 - La grève se termine par concession de la firme (décision Q(a))
#[test]
fn test_t8_3_fin_par_concession() {
    let mut manager = StrikeManager::new(42);
    let cohort = EntityId::new(EntityType::Cohort, 1);
    
    // Déclencher une grève
    let strike = manager.declencher_greve(cohort, 10).unwrap();
    let planned_end = strike.planned_end_day();
    
    // Mettre fin par concession au jour 15 (avant la fin prévue)
    assert!(manager.fin_de_greve(cohort, 15, StrikeEndCause::Concession));
    
    let strike_after = manager.get_strike(cohort).unwrap();
    assert!(strike_after.is_ended());
    assert_eq!(strike_after.end_day, Some(15));
    assert_eq!(strike_after.end_cause, Some(StrikeEndCause::Concession));
    
    // Vérifier que la grève est terminée avant la fin prévue
    assert!(15 < planned_end, "T8.3 : concession avant fin prévue");
}

// ============================================================================
// T8.4 : Fin de grève par durée écoulée
// ============================================================================

/// T8.4 - La grève se termine quand la durée est écoulée
#[test]
fn test_t8_4_fin_par_duree_ecoulee() {
    let mut manager = StrikeManager::new(42);
    let cohort = EntityId::new(EntityType::Cohort, 1);
    
    // Déclencher une grève
    let strike = manager.declencher_greve(cohort, 10).unwrap();
    let planned_end = strike.planned_end_day();
    
    // Mettre fin par durée écoulée
    assert!(manager.fin_de_greve(cohort, planned_end, StrikeEndCause::DurationElapsed));
    
    let strike_after = manager.get_strike(cohort).unwrap();
    assert!(strike_after.is_ended());
    assert_eq!(strike_after.end_day, Some(planned_end));
    assert_eq!(strike_after.end_cause, Some(StrikeEndCause::DurationElapsed));
}

// ============================================================================
// T8.5 : Déterminisme - même graine → même date de grève
// ============================================================================

/// T8.5 - Même graine → même date de grève et mêmes paramètres
#[test]
fn test_t8_5_determinisme() {
    let seed = 42;
    let cohort = EntityId::new(EntityType::Cohort, 1);
    
    // Créer deux managers avec la même graine
    let mut manager1 = StrikeManager::new(seed);
    let mut manager2 = StrikeManager::new(seed);
    
    // Déclencher des grèves au même jour
    let strike1 = manager1.declencher_greve(cohort, 10).unwrap();
    let strike2 = manager2.declencher_greve(cohort, 10).unwrap();
    
    // Les paramètres doivent être identiques (même graine + même cohort + même jour)
    assert_eq!(strike1.risk_threshold, strike2.risk_threshold, 
        "T8.5 : même seuil de risque avec même graine");
    assert_eq!(strike1.planned_duration, strike2.planned_duration, 
        "T8.5 : même durée avec même graine");
}

/// T8.5 bis - Différentes graines → différents paramètres
#[test]
fn test_t8_5_different_seeds_different_results() {
    let cohort = EntityId::new(EntityType::Cohort, 1);
    
    let mut manager1 = StrikeManager::new(42);
    let mut manager2 = StrikeManager::new(43);
    
    let strike1 = manager1.declencher_greve(cohort, 10).unwrap();
    let strike2 = manager2.declencher_greve(cohort, 10).unwrap();
    
    // Les paramètres doivent être différents (gaines différentes)
    assert_ne!(strike1.risk_threshold, strike2.risk_threshold, 
        "T8.5 : différents seuils avec graines différentes");
    
    // La durée peut être la même par hasard, mais le seuil doit être différent
}

/// T8.5 ter - Même graine, cohortes différentes → tirages différents
#[test]
fn test_t8_5_same_seed_different_cohorts() {
    let seed = 42;
    let cohort1 = EntityId::new(EntityType::Cohort, 1);
    let cohort2 = EntityId::new(EntityType::Cohort, 2);
    
    let mut manager = StrikeManager::new(seed);
    
    let strike1 = manager.declencher_greve(cohort1, 10).unwrap();
    let strike2 = manager.declencher_greve(cohort2, 10).unwrap();
    
    // Les paramètres doivent être différents (cohortes différentes)
    assert_ne!(strike1.risk_threshold, strike2.risk_threshold, 
        "T8.5 : tirages différents pour cohortes différentes (même graine)");
}

// ============================================================================
// T8.6 : Validation complète
// ============================================================================

/// Validation complète de la tranche 8
#[test]
fn test_t8_validation_complete() {
    test_t8_1_basculement();
    test_t8_2_tirage_unique_never_redrawn();
    test_t8_2_each_strike_has_unique_draw();
    test_t8_3_fin_par_concession();
    test_t8_4_fin_par_duree_ecoulee();
    test_t8_5_determinisme();
    test_t8_5_different_seeds_different_results();
    test_t8_5_same_seed_different_cohorts();
}
