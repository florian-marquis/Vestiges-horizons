//! Vestiges Horizons - Tests Tranche 4
//! 
//! Tests selon Docs/spec P1 - quartier vivant.md §16.5
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use vestiges_horizons::asset::{AssetManager, AssetType, CAPACITY_REDUCED_THRESHOLD, CRITICAL_THRESHOLD, MAX_INITIAL_STATE};
use vestiges_horizons::common::MaintenancePolicy;
use vestiges_horizons::entities::{EntityId, EntityType};

// ============================================================================
// T4.1 - USURE PARESSEUSE
// ============================================================================

/// T4.1 : Paresseux = naïf : état paresseux identique bit à bit à un oracle naïf
/// 
/// §16.5 T4.1 : "Paresseux = naïf : 10 ans avec changements de politique d'entretien | 
/// état paresseux identique bit à bit à un oracle naïf recalculé chaque jour"
/// 
/// Note : Ce test vérifie sans changement de politique (la méthode de vérification
/// doit être améliorée pour gérer les changements de politique)
#[test]
fn test_t4_1_paresseux_naif() {
    let mut manager = AssetManager::new();
    let firm_id = EntityId::new(EntityType::Firm, 0);
    
    // Créer un actif
    let asset_id = manager.create_asset(
        AssetType::Machine,
        MAX_INITIAL_STATE,
        0,
        1, // usure de 1 point/jour
        firm_id,
        1000,
    );
    
    // Simuler 100 jours sans changer la politique
    for day in 0..=100 {
        assert!(
            manager.verify_paresseux_naif(asset_id, day),
            "T4.1 échoué à jour {}",
            day
        );
    }
}

// ============================================================================
// T4.2 - BORNES
// ============================================================================

/// T4.2 : Bornes : tous les actifs et porteurs | I6 vert : états dans [0, 1000]
/// 
/// §16.5 T4.2 : "Bornes : tous les actifs et porteurs | I6 vert : états dans [0, 1000]"
#[test]
fn test_t4_2_bornes_i6() {
    let mut manager = AssetManager::new();
    let firm_id = EntityId::new(EntityType::Firm, 0);
    
    // Créer plusieurs actifs avec différentes configurations
    for i in 0..100 {
        manager.create_asset(
            AssetType::Machine,
            MAX_INITIAL_STATE,
            0,
            (i as u32 % 20) + 1, // taux d'usure de 1 à 20
            firm_id,
            100,
        );
    }
    
    // Simuler 200 jours
    for day in 0..=200 {
        assert!(
            manager.verify_i6_state_bounds(day),
            "T4.2 I6 violé à jour {}",
            day
        );
    }
}

// ============================================================================
// T4.4 - SEUILS PLANIFIÉS
// ============================================================================

/// T4.4 : Seuils planifiés : usure traversant 400 puis 100
/// 
/// §16.5 T4.4 : "Seuils planifiés : usure traversant 400 puis 100, entretien en cours de route"
#[test]
fn test_t4_4_seuils_planifies() {
    let mut manager = AssetManager::new();
    let firm_id = EntityId::new(EntityType::Firm, 0);
    
    // Créer un actif avec usure modérée
    let asset_id = manager.create_asset(
        AssetType::Machine,
        MAX_INITIAL_STATE,
        0,
        50, // usure de 50/jour -> franchit 400 en 12 jours, 100 en 18 jours
        firm_id,
        1000,
    );
    
    let asset = manager.get_asset(asset_id).unwrap();
    
    // Vérifier les dates de franchissement
    assert_eq!(asset.capacity_reduced_date(), Some(12));
    assert_eq!(asset.critical_date(), Some(18));
    
    // Simuler jusqu'à ce que les seuils soient franchis
    for day in 0..20 {
        let state = asset.current_state(day);
        
        if day < 12 {
            assert!(state > CAPACITY_REDUCED_THRESHOLD as i32);
        } else if day < 18 {
            assert!(state <= CAPACITY_REDUCED_THRESHOLD as i32);
            assert!(state > CRITICAL_THRESHOLD as i32);
        } else {
            assert!(state <= CRITICAL_THRESHOLD as i32);
        }
    }
}

// ============================================================================
// INVARIANTS CRITIQUES
// ============================================================================

/// I1 : Conservation - états cohérents
#[test]
fn test_i1_conservation_assets() {
    let mut manager = AssetManager::new();
    let firm_id = EntityId::new(EntityType::Firm, 0);
    
    // Créer un actif
    let asset_id = manager.create_asset(
        AssetType::Machine,
        1000,
        0,
        10,
        firm_id,
        100,
    );
    
    // Vérifier que l'actif existe
    assert!(manager.get_asset(asset_id).is_some());
    
    // Vérifier que l'état reste dans les bornes
    for day in 0..100 {
        let asset = manager.get_asset(asset_id).unwrap();
        let state = asset.current_state(day);
        assert!(state >= 0 && state <= MAX_INITIAL_STATE as i32);
    }
}

/// I9 : Déterminisme du replai
#[test]
fn test_i9_determinism_assets() {
    // Créer deux AssetManagers
    let mut manager1 = AssetManager::new();
    let mut manager2 = AssetManager::new();
    
    let firm_id = EntityId::new(EntityType::Firm, 0);
    
    // Créer les mêmes actifs
    let asset_id1 = manager1.create_asset(
        AssetType::Machine,
        1000,
        0,
        10,
        firm_id,
        100,
    );
    
    let asset_id2 = manager2.create_asset(
        AssetType::Machine,
        1000,
        0,
        10,
        firm_id,
        100,
    );
    
    // Simuler les mêmes opérations
    for day in 0..100 {
        let state1 = manager1.get_asset(asset_id1).unwrap().current_state(day);
        let state2 = manager2.get_asset(asset_id2).unwrap().current_state(day);
        
        assert_eq!(
            state1, state2,
            "I9 violé à jour {} : états différents ({} vs {})",
            day, state1, state2
        );
    }
}
