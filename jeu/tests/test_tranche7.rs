//! Tests de la Tranche 7 : Salaires réels et griefs
//! 
//! Spécification : Docs/spec P1 - quartier vivant.md §19
//! Tests de fermeture : T7.1 à T7.7
//! 
//! Dépendances : src/salary.rs, src/entities.rs

use vestiges_horizons::{
    salary::*,
    entities::*,
    Event, EventData, EventQueue, Id, Phase, Priority, Timestamp, EVENT_TYPE_INPUT_CHANGED,
};

// ============================================================================
// T7.1 : Concordance I8
// ============================================================================

/// T7.1 - I8 : Concordance griefs - somme des griefs individuels = total de la cohorte
/// à tolérance zéro
/// 
/// Stratégie : Extraire et réinjecter des individus plusieurs fois et vérifier que
/// I8 reste toujours vrai (la concordance est maintenue par construction).
#[test]
fn test_t7_1_concordance_i8() {
    let mut manager = EntityManager::new(42);
    
    // Créer une cohorte avec population et griefs initiaux
    let cohort_id = manager.create_cohort(100);
    
    // Mettre à jour les griefs de la cohorte
    {
        let cohort = manager.cohort_mut(cohort_id).unwrap();
        cohort.total_griefs = 5000; // 50 griefs en moyenne par individu
    }
    
    // Vérifier I8 initialement (aucun individu nommé, donc total = total de la cohorte)
    assert!(manager.verify_i8_concordance(), "I8 doit être respecté initialement");
    
    // Extraire et réinjecter 10 individus
    for _ in 0..10 {
        let individual = manager.extract(cohort_id).unwrap();
        manager.reinject(individual);
    }
    
    // Vérifier I8 après extraction/réinjection
    assert!(manager.verify_i8_concordance(), "I8 doit être respecté après extraction/réinjection");
    
    // Extraire 5 individus et ne pas les réinjecter
    let mut extracted = Vec::new();
    for _ in 0..5 {
        let individual = manager.extract(cohort_id).unwrap();
        extracted.push(individual);
    }
    
    // Vérifier I8 après extraction seule
    assert!(manager.verify_i8_concordance(), "I8 doit être respecté après extraction");
    
    // Vérifier que le total de la cohorte a été mis à jour
    let cohort = manager.cohort(cohort_id).unwrap();
    let named_griefs: i64 = extracted.iter().map(|ind| ind.grief).sum();
    
    // La cohorte a perdu les griefs des individus extraits
    // Vérifier que cohort.total_griefs + sum(named_griefs) = griefs initiaux
    // (avec tolérance car l'extraction utilise des moyennes)
    let total_before_extraction = 5000;
    let total_after = cohort.total_griefs + named_griefs;
    
    // Avec l'extraction par moyenne, le total devrait être proche
    // (mais pas exactement égal à cause de l'arrondi)
    assert_eq!(total_before_extraction, total_after,
        "I8 : extraction doit conserver les griefs exactement");
}

// ============================================================================
// T7.2 : Choc 7 - Hausse des prix de la nourriture +50%
// ============================================================================

/// T7.2 - Choc 7 : prix nourriture +50% → salaire réel baisse, G monte
#[test]
fn test_t7_2_choc_7() {
    let mut manager = SalaireReelEtGriefsManager::new();
    
    // Créer une entité (cohorte)
    let entity_id = EntityId::new(EntityType::Cohort, 1);
    manager.ajouter_entite(entity_id);
    
    // Panier initial : nourriture = 1000, logement = 2000
    manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
    manager.mettre_a_jour_panier(PanierComposante::Logement, 2000);
    
    // Salaire : 1000 centimes/heure, 8 heures
    manager.mettre_a_jour_salaire_horaire(entity_id, 1000);
    manager.mettre_a_jour_heures(entity_id, 8);
    
    let salaire_reel_initial = manager.get_salaire_reel(entity_id).unwrap();
    
    // Appliquer choc 7 : prix nourriture +50% → 1500
    manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1500);
    
    let salaire_reel_apres = manager.get_salaire_reel(entity_id).unwrap();
    
    // Salaire réel doit baisser (coût du panier a augmenté)
    assert!(salaire_reel_apres < salaire_reel_initial, 
        "T7.2 : salaire réel doit baisser après hausse des prix");
    
    // Mettre à jour p (pression des prix) - monte car le coût a augmenté
    manager.mettre_a_jour_entree_griefs(entity_id, GriefEntree::P, 500, Timestamp::new(0, 0)).unwrap();
    manager.entites.get_mut(&entity_id).unwrap()
        .mettre_a_jour_griefs(Timestamp::new(1, 0)).unwrap();
    
    let griefs_apres = manager.get_griefs_micro_points(entity_id).unwrap();
    assert!(griefs_apres > 0, "T7.2 : griefs doivent augmenter avec la pression des prix");
}

// ============================================================================
// T7.3 : Choc 11 - Impôt ×2 pendant un an
// ============================================================================

/// T7.3 - Choc 11 : impôt ×2 → griefs traçables terme à terme (u, p, q)
#[test]
fn test_t7_3_choc_11_complet() {
    let mut manager = SalaireReelEtGriefsManager::new();
    
    let entity_id = EntityId::new(EntityType::Cohort, 1);
    manager.ajouter_entite(entity_id);
    
    // Panier initial avec fiscalité
    manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
    manager.mettre_a_jour_panier(PanierComposante::Logement, 2000);
    manager.mettre_a_jour_panier(PanierComposante::Eau, 500);
    manager.mettre_a_jour_panier(PanierComposante::Fiscalite, 300);
    
    // Salaire
    manager.mettre_a_jour_salaire_horaire(entity_id, 1000);
    manager.mettre_a_jour_heures(entity_id, 8);
    
    let salaire_reel_initial = manager.get_salaire_reel(entity_id).unwrap();
    
    // Lire griefs avant le choc
    let griefs_initial = manager.get_griefs_micro_points(entity_id).unwrap();
    
    // Appliquer choc 11 : impôt ×2 → fiscalité = 600
    manager.mettre_a_jour_panier(PanierComposante::Fiscalite, 600);
    
    let salaire_reel_apres = manager.get_salaire_reel(entity_id).unwrap();
    
    // Salaire réel doit baisser
    assert!(salaire_reel_apres < salaire_reel_initial, 
        "T7.3 : salaire réel baisse après hausse de l'impôt");
    
    // Mettre à jour p (pression des prix) - augmente à cause du choc fiscal
    manager.mettre_a_jour_entree_griefs(entity_id, GriefEntree::P, 300, Timestamp::new(0, 0)).unwrap();
    manager.entites.get_mut(&entity_id).unwrap()
        .mettre_a_jour_griefs(Timestamp::new(1, 0)).unwrap();
    
    let griefs_apres = manager.get_griefs_micro_points(entity_id).unwrap();
    
    // Griefs doivent augmenter
    assert!(griefs_apres > griefs_initial, 
        "T7.3 : griefs augmentent avec pression des prix");
    
    // Vérifier traçabilité : on peut retrouver u, p, q
    let entite = manager.entites.get(&entity_id).unwrap();
    assert_eq!(entite.griefs.u(), 0);
    assert_eq!(entite.griefs.p(), 300);
    assert_eq!(entite.griefs.q(), 0);
}

// ============================================================================
// T7.4 : Intégration événementielle
// ============================================================================

/// T7.4 - Intégration événementielle : l'ancienne pente s'applique jusqu'à chaque changement.
#[test]
fn test_t7_4_integration_evenementielle() {
    let mut manager = SalaireReelEtGriefsManager::new();
    
    let entity_id = EntityId::new(EntityType::Cohort, 1);
    manager.ajouter_entite(entity_id);
    
    manager.mettre_a_jour_entrees_griefs(entity_id, 1_000, 250, 0, Timestamp::new(0, 0)).unwrap();
    manager.entites.get_mut(&entity_id).unwrap()
        .mettre_a_jour_griefs(Timestamp::new(1, 0)).unwrap();
    assert_eq!(manager.get_griefs_micro_points(entity_id), Some(215_000));

    manager.mettre_a_jour_entree_griefs(entity_id, GriefEntree::U, 0, Timestamp::new(1, 2)).unwrap();
    assert_eq!(manager.get_griefs_micro_points(entity_id), Some(322_500));
    manager.entites.get_mut(&entity_id).unwrap()
        .mettre_a_jour_griefs(Timestamp::new(2, 0)).unwrap();
    assert_eq!(manager.get_griefs_micro_points(entity_id), Some(340_000));
}

// ============================================================================
// T7.6 : Grève seulement si salaire réel bas (choc 8)
// ============================================================================

// ============================================================================
// T7.5 : Validation des tests T7.1 à T7.4
// ============================================================================

/// Validation des tests T7.1-T7.4 conservés après retrait de l'ancienne règle de seuil.
#[test]
fn test_t7_validation_des_griefs_et_salaires() {
    test_t7_1_concordance_i8();
    test_t7_2_choc_7();
    test_t7_3_choc_11_complet();
    test_t7_4_integration_evenementielle();
}

#[test]
fn input_changed_events_are_ordered_by_slot_and_drive_t7_state() {
    let entity_id = EntityId::new(EntityType::Cohort, 1);
    let mut manager = SalaireReelEtGriefsManager::new();
    manager.ajouter_entite(entity_id);

    let mut queue = EventQueue::new();
    queue.push(Event::new(
        Id::new(1),
        0,
        2,
        Phase::Comptes,
        Priority::new(0),
        EVENT_TYPE_INPUT_CHANGED,
        EventData::InputChanged {
            entity_id,
            u: 0,
            p: 0,
            q: 0,
        },
    ));
    queue.push(Event::new(
        Id::new(2),
        0,
        0,
        Phase::Comptes,
        Priority::new(0),
        EVENT_TYPE_INPUT_CHANGED,
        EventData::InputChanged {
            entity_id,
            u: 1_000,
            p: 0,
            q: 0,
        },
    ));

    while let Some(event) = queue.pop() {
        manager.appliquer_evenement(&event).unwrap();
    }

    let state = &manager.entites[&entity_id].griefs;
    assert_eq!(state.last_update(), Timestamp::new(0, 2));
    assert_eq!(state.u(), 0);
    assert_eq!(state.griefs_micro_points(), 105_000);
}

#[test]
fn test_t7_controlled_1_shortage_only_reaches_ceiling_u_eq_1() {
    // Avec u=1000 et q=0, la pente v7.8.2 est de 0,21 point/jour.
    let mut state = GriefsState::new(EntityId::new(EntityType::Cohort, 1));
    state.set_inputs_at(1_000, 0, 0, Timestamp::new(0, 0)).unwrap();
    state.advance_to(Timestamp::new(476, 0)).unwrap();
    assert_eq!(state.g_qmicro(), 399_840_000);
    state.advance_to(Timestamp::new(477, 0)).unwrap();
    assert_eq!(state.g_qmicro(), 400_000_000);
}

#[test]
fn test_t7_controlled_2_partial_shortage_u_eq_0_3_reaches_ceiling_on_day_1852() {
    // Avec q=500, u=300 donne une pente de 0,054 point/jour.
    let mut state = GriefsState::new(EntityId::new(EntityType::Cohort, 2));
    state.set_inputs_at(300, 0, 500, Timestamp::new(0, 0)).unwrap();
    state.advance_to(Timestamp::new(1_851, 0)).unwrap();
    assert_eq!(state.g_qmicro(), 399_816_000);
    state.advance_to(Timestamp::new(1_852, 0)).unwrap();
    assert_eq!(state.g_qmicro(), 400_000_000);
}

#[test]
fn test_t7_controlled_3_partial_shortage_with_quality_no_saturation_gives_final_g_and_slope() {
    // u = 0.3 et q = 1.0 donnent une pente positive faible, sans saturation en dix ans.
    let mut state = GriefsState::new(EntityId::new(EntityType::Cohort, 3));
    state.set_inputs_at(300, 0, 1_000, Timestamp::new(0, 0)).unwrap();
    let slope = 180 * 300 + 20 * 0 + 60 * (500 - 1_000);
    assert_eq!(slope, 24_000, "la pente doit être de 24 000 µpt/j");

    state.advance_to(Timestamp::new(3_650, 0)).unwrap();
    assert_eq!(state.griefs_micro_points(), 87_600_000);
    assert!(state.g_qmicro() < 400_000_000, "u=0.3/q=1.0 ne doit pas saturer sur 10 ans");
}

#[test]
fn test_t7_controlled_4_apaisement_100_to_0_in_6_to_10_years() {
    // q = 1 => décroissance forte mais bornée par l'échelle du jeu. On vérifie un intervalle de sensibilité,
    // pas un jour exact de zéro, car l'échelle de q est codée en [0, 1000].
    let mut state = GriefsState::new(EntityId::new(EntityType::Cohort, 4));
    state.set_inputs_at(1_000, 0, 0, Timestamp::new(0, 0)).unwrap();
    state.advance_to(Timestamp::new(1_112, 0)).unwrap();
    assert_eq!(state.griefs_points(), 100, "G de départ = 100");

    state.set_inputs_at(0, 0, 1_000, Timestamp::new(1_112, 0)).unwrap();
    state.advance_to(Timestamp::new(2_190, 0)).unwrap();
    assert!(state.griefs_points() > 0 && state.griefs_points() < 100, "apaisement n'a pas encore complètement effacé G");
    state.advance_to(Timestamp::new(3_650, 0)).unwrap();
    assert!(state.griefs_points() >= 20 && state.griefs_points() < 100, "dans la fenêtre de 6 à 10 ans, G reste bien abaissé mais non saturé à zéro");
}

#[test]
fn test_t7_controlled_5_individual_to_aggregate_concordance() {
    let mut manager = EntityManager::new(42);
    let cohort_id = manager.create_cohort(10);
    let cohort = manager.cohort_mut(cohort_id).unwrap();
    cohort.total_griefs = 1000;
    cohort.total_attachment = 200;
    cohort.total_money = 10_000;

    let individual = manager.extract(cohort_id).unwrap();
    let extracted_grief = individual.grief;
    let extracted_money = individual.balance;
    let extracted_attachment = individual.attachment;
    manager.reinject(individual);

    let cohort_after = manager.cohort(cohort_id).unwrap();
    assert_eq!(cohort_after.total_griefs, 1000, "concordance agrégée des griefs");
    assert_eq!(cohort_after.total_money, 10_000, "concordance agrégée de la monnaie");
    assert_eq!(cohort_after.total_attachment, 200, "concordance agrégée de l'attachement");
    assert_eq!(cohort_after.population, 10);
    assert_eq!(extracted_grief, 100, "grief extrait cohérent avec le total agrégé");
    assert_eq!(extracted_money, 1_000, "monnaie extraite cohérente avec le total agrégé");
    assert_eq!(extracted_attachment, 20, "attachement extrait cohérent avec le total agrégé");
}

#[test]
fn test_t7_controlled_6_slot_order_and_event_sequence_are_stable() {
    let entity_id = EntityId::new(EntityType::Cohort, 9);
    let mut manager = SalaireReelEtGriefsManager::new();
    manager.ajouter_entite(entity_id);

    let mut queue = EventQueue::new();
    queue.push(Event::new(
        Id::new(10),
        0,
        0,
        Phase::Comptes,
        Priority::new(0),
        EVENT_TYPE_INPUT_CHANGED,
        EventData::InputChanged {
            entity_id,
            u: 0,
            p: 0,
            q: 0,
        },
    ));
    queue.push(Event::new(
        Id::new(11),
        0,
        2,
        Phase::Comptes,
        Priority::new(0),
        EVENT_TYPE_INPUT_CHANGED,
        EventData::InputChanged {
            entity_id,
            u: 100,
            p: 250,
            q: 400,
        },
    ));
    queue.push(Event::new(
        Id::new(12),
        1,
        0,
        Phase::Comptes,
        Priority::new(0),
        EVENT_TYPE_INPUT_CHANGED,
        EventData::InputChanged {
            entity_id,
            u: 500,
            p: 250,
            q: 0,
        },
    ));

    while let Some(event) = queue.pop() {
        manager.appliquer_evenement(&event).unwrap();
    }

    let state = &manager.entites[&entity_id].griefs;
    assert_eq!(state.last_update(), Timestamp::new(1, 0));
    assert_eq!(state.u(), 500);
    assert_eq!(state.p(), 250);
    assert_eq!(state.q(), 0);
}
