//! Tests pour T7-T8 Tranche A2 - Lot 1 (Agent 3)
//!
//! Tests des calculs de u, p, q selon les spécifications de l'ordre de mission.
//!
//! Correspond aux tests #1, #2, #4, #5, #6 de l'ordre de mission.

use vestiges_horizons::{
    entities::EntityId,
    health::{CareType, DispensaryPatient, DispensaryQueue},
    household::{FoodQuality, Household, HousingQuality},
    inputs::{
        calculate_p, calculate_q, calculate_q_from_components, calculate_u, ServiceComponent,
        Q_WEIGHTS,
    },
    salary::SalaireReelEtGriefs,
};

// ============================================================================
// TEST #1 : u = 0 tant que la couverture est positive ; après rupture,
// u égale non servis ÷ demande, exactement, sur un cas non divisible (7/19)
// ============================================================================

#[test]
fn test_u_zero_when_coverage_positive() {
    // Cas : stock = 100, demande = 100 → coverage positive
    let u = calculate_u(0, 100);
    assert_eq!(
        u, 0,
        "u doit être 0 quand non_servis = 0 (coverage positive)"
    );
}

#[test]
fn test_u_zero_when_demande_zero() {
    // Cas : demande = 0 → éviter division par zéro
    let u = calculate_u(50, 0);
    assert_eq!(u, 0, "u doit être 0 quand demande = 0");
}

#[test]
fn test_u_exact_7_over_19() {
    // Cas de l'ordre de mission : 7 non servis sur 19 demande
    // u = (7 * 1000) / 19 = 7000 / 19 = 368 (troncature entière)
    let u = calculate_u(7, 19);
    assert_eq!(u, 368, "u doit être 368 pour 7 non servis sur 19 demande");
}

#[test]
fn test_u_from_household_exact_7_over_19() {
    // Cas avec Household : besoin = 19, stock = 12
    // non_servis = 19 - 12 = 7
    let mut household = Household::new(
        EntityId::new(vestiges_horizons::entities::EntityType::Household, 1),
        EntityId::new(vestiges_horizons::entities::EntityType::Cohort, 1),
        HousingQuality::Base,
    );
    household.add_food(FoodQuality::Base, 12);

    let u = household.calculate_u(19);
    assert_eq!(
        u, 368,
        "Household::calculate_u doit retourner 368 pour 12 stock sur 19 besoin"
    );
}

#[test]
fn test_u_max_value() {
    // u = 1000 quand non_servis >= demande
    let u = calculate_u(100, 100);
    assert_eq!(u, 1000, "u doit être 1000 quand non_servis = demande");

    let u = calculate_u(200, 100);
    assert_eq!(
        u, 1000,
        "u doit être capped à 1000 quand non_servis > demande"
    );
}

// ============================================================================
// TEST #2 : Demande nulle → u = 0 ; u, p, q toujours dans [0, 1000]
// ============================================================================

#[test]
fn test_demande_nulle_u_zero() {
    // Demande nulle → u = 0
    let u = calculate_u(50, 0);
    assert_eq!(u, 0, "u doit être 0 quand demande = 0");
}

#[test]
fn test_all_inputs_in_range() {
    // Test que u, p, q restent toujours dans [0, 1000]

    // u dans [0, 1000]
    for non_servis in [0, 50, 100, 200] {
        for demande in [1, 10, 100, 1000] {
            let u = calculate_u(non_servis, demande);
            assert!(
                u <= 1000,
                "u={} doit être <= 1000 (non_servis={}, demande={})",
                u,
                non_servis,
                demande
            );
        }
    }

    // p dans [0, 1000]
    for cout_panier in [0, 100, 1000, 10000] {
        for revenu in [0, 500, 1000, 5000, 10000] {
            let p = calculate_p(cout_panier, revenu);
            assert!(
                p <= 1000,
                "p={} doit être <= 1000 (cout_panier={}, revenu={})",
                p,
                cout_panier,
                revenu
            );
        }
    }

    // q dans [0, 1000]
    for taux_soins in [0, 250, 500, 750, 1000] {
        for taux_logements in [0, 250, 500, 750, 1000] {
            let q = calculate_q_from_components(taux_soins, taux_logements);
            assert!(
                q <= 1000,
                "q={} doit être <= 1000 (soins={}, logements={})",
                q,
                taux_soins,
                taux_logements
            );
        }
    }
}

#[test]
fn test_p_zero_when_cout_panier_zero() {
    let p = calculate_p(0, 1000);
    assert_eq!(p, 0, "p doit être 0 quand cout_panier = 0");
}

#[test]
fn test_p_zero_when_revenu_suffisant() {
    // revenu >= cout_panier → p = 0
    let p = calculate_p(1000, 1000);
    assert_eq!(p, 0, "p doit être 0 quand revenu >= cout_panier");

    let p = calculate_p(1000, 1500);
    assert_eq!(p, 0, "p doit être 0 quand revenu > cout_panier");
}

#[test]
fn test_p_exact_calculation() {
    // cout_panier = 1000, revenu = 800
    // p = ((1000 - 800) * 1000) / 1000 = 200
    let p = calculate_p(1000, 800);
    assert_eq!(p, 200, "p doit être 200 pour cout_panier=1000, revenu=800");
}

#[test]
fn test_p_capped_at_1000() {
    // revenu = 0, cout_panier = 1000
    // p = ((1000 - 0) * 1000) / 1000 = 1000
    let p = calculate_p(1000, 0);
    assert_eq!(p, 1000, "p doit être 1000 quand revenu = 0");
}

// ============================================================================
// TEST #4 : q suit l'état des services : tous servis → maximum,
// aucun servi → zéro, cas intermédiaire vérifié à la main
// ============================================================================

#[test]
fn test_q_all_services_perfect() {
    // Tous les services à 1000
    let q = calculate_q_from_components(1000, 1000);
    assert_eq!(
        q, 1000,
        "q doit être 1000 quand soins=1000 et logements=1000"
    );
}

#[test]
fn test_q_no_services() {
    // Tous les services à 0
    let q = calculate_q_from_components(0, 0);
    assert_eq!(q, 0, "q doit être 0 quand soins=0 et logements=0");
}

#[test]
fn test_q_weighted_average() {
    // Soins = 800, Logements = 600
    // q = (800 * 500 + 600 * 500) / 1000 = (400000 + 300000) / 1000 = 700
    let q = calculate_q_from_components(800, 600);
    assert_eq!(q, 700, "q doit être 700 pour soins=800, logements=600");
}

#[test]
fn test_q_intermediate_values() {
    // Cas intermédiaire : soins = 500, logements = 500
    let q = calculate_q_from_components(500, 500);
    assert_eq!(q, 500, "q doit être 500 pour soins=500, logements=500");

    // Cas : soins = 1000, logements = 0
    let q = calculate_q_from_components(1000, 0);
    assert_eq!(
        q, 500,
        "q doit être 500 pour soins=1000, logements=0 (poids 50/50)"
    );

    // Cas : soins = 0, logements = 1000
    let q = calculate_q_from_components(0, 1000);
    assert_eq!(
        q, 500,
        "q doit être 500 pour soins=0, logements=1000 (poids 50/50)"
    );
}

#[test]
fn test_q_with_table() {
    // Test avec la table de poids explicite
    let components = vec![
        (ServiceComponent::Soins, 800),
        (ServiceComponent::Logements, 600),
    ];
    let q = calculate_q(&components);
    assert_eq!(q, 700, "q doit être 700 avec la table de poids");
}

#[test]
fn test_q_one_component_missing() {
    // Seulement soins = 500, logements absent
    // total_weight = 500, weighted_sum = 500 * 500 = 250000
    // q = 250000 / 500 = 500
    let components = vec![(ServiceComponent::Soins, 500)];
    let q = calculate_q(&components);
    assert_eq!(q, 500, "q doit être 500 avec seulement soins=500");
}

#[test]
fn test_q_all_components_missing() {
    // Aucune composante
    let components: Vec<(ServiceComponent, u16)> = vec![];
    let q = calculate_q(&components);
    assert_eq!(q, 0, "q doit être 0 avec aucune composante");
}

// ============================================================================
// TEST #5 : Deux cohortes de situations différentes donnent des u et q
// différents sans aucun décalage codé
// ============================================================================

#[test]
fn test_different_cohorts_different_u() {
    // Cohorte 1 : shortage sévère
    let u1 = calculate_u(50, 100); // 500

    // Cohorte 2 : situation normale (coverage positive)
    let u2 = calculate_u(0, 100); // 0

    assert_ne!(
        u1, u2,
        "Deux cohortes avec des situations différentes doivent avoir des u différents"
    );
}

#[test]
fn test_different_cohorts_different_q() {
    // Cohorte 1 : services parfaits
    let q1 = calculate_q_from_components(1000, 1000); // 1000

    // Cohorte 2 : services médiocres
    let q2 = calculate_q_from_components(500, 500); // 500

    assert_ne!(
        q1, q2,
        "Deux cohortes avec des services différents doivent avoir des q différents"
    );
}

#[test]
fn test_different_cohorts_different_inputs_combined() {
    // Cohorte 1 : shortage sévère, services médiocres, pression prix élevée
    let u1 = calculate_u(50, 100); // 500
    let p1 = calculate_p(2000, 1000); // 500
    let q1 = calculate_q_from_components(600, 400); // 500

    // Cohorte 2 : situation normale
    let u2 = calculate_u(0, 100); // 0
    let p2 = calculate_p(1000, 2000); // 0
    let q2 = calculate_q_from_components(1000, 1000); // 1000

    // Au moins une des valeurs doit être différente
    assert_ne!(u1, u2, "u doit différer");
    assert_ne!(p1, p2, "p doit différer");
    assert_ne!(q1, q2, "q doit différer");
}

// ============================================================================
// Test #6 : Recherche statique : aucun flottant ni exp dans les fichiers
// de calcul de u, q, p
// ============================================================================

#[test]
fn test_input_calculation_sources_have_no_floating_point_or_exp() {
    let sources = [
        include_str!("../src/inputs.rs"),
        include_str!("../src/household.rs"),
        include_str!("../src/salary.rs"),
        include_str!("../src/health.rs"),
        include_str!("../src/asset.rs"),
        include_str!("../src/maintenance.rs"),
    ];

    for source in sources {
        assert!(!source.contains("f64"), "Un calcul d'entrée contient f64");
        assert!(!source.contains("exp("), "Un calcul d'entrée contient exp(");
    }
}

#[test]
fn test_u_uses_integer_arithmetic() {
    // Vérification indirecte : u est toujours un entier
    // Si le calcul utilisait f64, la troncature pourrait donner des résultats inattendus

    // Cas critique : 1 * 1000 / 3 = 333 (troncature)
    let u = calculate_u(1, 3);
    assert_eq!(
        u, 333,
        "u doit utiliser l'arithmétique entière (1/3 * 1000 = 333)"
    );
}

#[test]
fn test_p_uses_integer_arithmetic() {
    // Vérification indirecte : p utilise l'arithmétique entière

    // Cas : cout_panier = 3, revenu = 1
    // p = ((3 - 1) * 1000) / 3 = 2000 / 3 = 666 (troncature)
    let p = calculate_p(3, 1);
    assert_eq!(p, 666, "p doit utiliser l'arithmétique entière");
}

#[test]
fn test_q_uses_integer_arithmetic() {
    // Vérification indirecte : q utilise l'arithmétique entière

    // Cas : soins = 1, logements = 1
    // q = (1 * 500 + 1 * 500) / 1000 = 1000 / 1000 = 1
    let q = calculate_q_from_components(1, 1);
    assert_eq!(q, 1, "q doit utiliser l'arithmétique entière");
}

// ============================================================================
// Tests d'intégration avec les structures existantes
// ============================================================================

#[test]
fn test_household_integration() {
    let mut household = Household::new(
        EntityId::new(vestiges_horizons::entities::EntityType::Household, 1),
        EntityId::new(vestiges_horizons::entities::EntityType::Cohort, 1),
        HousingQuality::Base,
    );

    // Ajouter de la nourriture
    household.add_food(FoodQuality::Base, 50);

    // Besoin = 100, stock = 50 → non_servis = 50
    let u = household.calculate_u(100);
    assert_eq!(
        u, 500,
        "Household::calculate_u doit retourner 500 pour 50/100"
    );
}

#[test]
fn test_dispensary_queue_integration() {
    let mut queue = DispensaryQueue::new(10); // Capacité = 10 patients/jour

    // Cinq demandeurs pour une capacité de dix : tous sont servis.
    for i in 0..5 {
        queue.add_patient(DispensaryPatient {
            individual_id: EntityId::new(
                vestiges_horizons::entities::EntityType::NamedIndividual,
                i,
            ),
            priority: 50,
            arrival_day: 0,
            care_type: CareType::Basic,
        });
    }

    let taux = queue.calculate_service_rate();
    assert_eq!(
        taux, 1000,
        "Les cinq demandeurs sont tous servis par capacité 10"
    );
}

#[test]
fn test_dispensary_queue_full_capacity_is_fully_served() {
    let mut queue = DispensaryQueue::new(10);
    for i in 0..10 {
        queue.add_patient(DispensaryPatient {
            individual_id: EntityId::new(
                vestiges_horizons::entities::EntityType::NamedIndividual,
                i,
            ),
            priority: 50,
            arrival_day: 0,
            care_type: CareType::Basic,
        });
    }

    assert_eq!(queue.calculate_service_rate(), 1000);
}

#[test]
fn test_dispensary_queue_demand_over_capacity_is_partial() {
    let mut queue = DispensaryQueue::new(10);
    for i in 0..15 {
        queue.add_patient(DispensaryPatient {
            individual_id: EntityId::new(
                vestiges_horizons::entities::EntityType::NamedIndividual,
                i,
            ),
            priority: 50,
            arrival_day: 0,
            care_type: CareType::Basic,
        });
    }

    assert_eq!(queue.calculate_service_rate(), 666);
}

#[test]
fn test_dispensary_queue_no_demand_is_fully_covered() {
    let queue = DispensaryQueue::new(10);
    assert_eq!(queue.calculate_service_rate(), 1000);
}

#[test]
fn test_salaire_reel_et_griefs_p_integration() {
    let mut sr = SalaireReelEtGriefs::new(EntityId::new(
        vestiges_horizons::entities::EntityType::Cohort,
        1,
    ));

    // Configurer le panier et le salaire
    sr.cout_panier = 1000; // 10€
    sr.salaire_horaire = 500; // 5€/heure
    sr.heures_travaillees = 8; // 8 heures

    // revenu_disponible = 500 * 8 = 4000 centimes = 40€
    // p = 0 car revenu > cout_panier
    let p = sr.calculate_p(None);
    assert_eq!(
        p, 0,
        "SalaireReelEtGriefs::calculate_p doit retourner 0 quand revenu > cout_panier"
    );

    // Avec revenu insuffisant
    sr.salaire_horaire = 100; // 1€/heure
    sr.heures_travaillees = 8; // 8 heures
                               // revenu = 100 * 8 = 800 centimes = 8€
                               // p = ((1000 - 800) * 1000) / 1000 = 200
    let p = sr.calculate_p(None);
    assert_eq!(
        p, 200,
        "SalaireReelEtGriefs::calculate_p doit retourner 200"
    );
}

// ============================================================================
// Vérification des poids q
// ============================================================================

#[test]
fn test_q_weights_sum_to_1000() {
    let sum: u16 = Q_WEIGHTS.iter().map(|(_, w)| w).sum();
    assert_eq!(sum, 1000, "La somme des poids pour q doit être 1000");
}
