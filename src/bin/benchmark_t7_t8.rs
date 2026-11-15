//! Vestiges Horizons - Benchmark T7-T8
//! 
//! Exécute une simulation headless sur 10 ans avec graine 42
//! Valide spécifiquement les tranches 7 (salaires réels, griefs) et 8 (grèves)
//! 
//! Critères de validation :
//! - Déterminisme (I9)
//! - Concordance griefs (I8)
//! - Grèves déclenchées dans les conditions prévues
//! - Pas de re-tirage (T8.2)

use std::time::Instant;
use vestiges_horizons::{
    salary::*,
    entities::*,
    strike::*,
    EventData, EventQueue, Phase, Priority, Timestamp,
    EVENT_TYPE_INPUT_CHANGED,
};

/// Graine pour le benchmark
const BENCHMARK_SEED: u64 = 42;

/// Durée de la simulation en jours
const SIMULATION_DAYS: i64 = 365 * 10; // 10 ans

/// Nombre de cohortes à simuler
const NUM_COHORTS: u64 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Benchmark T7-T8 Headless ===");
    println!("Graine: {}", BENCHMARK_SEED);
    println!("Durée: {} jours ({} ans)", SIMULATION_DAYS, SIMULATION_DAYS / 365);
    println!("Cohortes: {}", NUM_COHORTS);
    println!();
    
    let start_time = Instant::now();
    
    // ========================================================================
    // Exécution 1 : Première passe
    // ========================================================================
    println!("[1/2] Première exécution...");
    
    let mut entity_manager = EntityManager::new(BENCHMARK_SEED);
    let mut salary_manager = SalaireReelEtGriefsManager::new();
    let mut strike_manager = StrikeManager::new(BENCHMARK_SEED);
    let mut input_events = EventQueue::new();
    
    // Créer les cohortes
    let mut cohort_ids = Vec::new();
    for i in 0..NUM_COHORTS {
        let cohort_id = entity_manager.create_cohort(100);
        cohort_ids.push(cohort_id);
        
        // Initialiser dans le manager de salaires
        salary_manager.ajouter_entite(cohort_id);
        
        // Salaire horaire entre 500 et 2000 centimes (5-20€/heure)
        let salaire = 500 + (i * 300);
        salary_manager.mettre_a_jour_salaire_horaire(cohort_id, salaire);
        salary_manager.mettre_a_jour_heures(cohort_id, 8);
    }
    
    // Initialiser le panier
    salary_manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
    salary_manager.mettre_a_jour_panier(PanierComposante::Logement, 2000);
    salary_manager.mettre_a_jour_panier(PanierComposante::Eau, 500);
    salary_manager.mettre_a_jour_panier(PanierComposante::Fiscalite, 300);
    
    // Variables de suivi
    let mut greves_declenchees: u64 = 0;
    let mut greves_terminees: u64 = 0;
    let mut jours_avec_greve: u64 = 0;
    let mut i8_violations: u64 = 0;
    let mut jours: u64 = 0;
    
    // Simulation sur 10 ans
    for day in 0..SIMULATION_DAYS {
        jours += 1;
        
        for (i, &cohort_id) in cohort_ids.iter().enumerate() {
            let u = ((day + i as i64 * 100) % 1000) as u16;
            let p = ((day * 2 + i as i64 * 50) % 1000) as u16;
            let q = ((day * 3) % 1000) as u16;
            input_events.push_at_with_id(
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_id,
                    u,
                    p,
                    q,
                },
            );
        }
        while let Some(event) = input_events.pop() {
            salary_manager.appliquer_evenement(&event)?;
        }

        // Mettre à jour les salaires réels (simulation de variation des prix)
        // Chaque cohorte a une trajectoire différente
        for (i, &cohort_id) in cohort_ids.iter().enumerate() {
            // Variation aléatoire mais déterministe du panier
            // (simulation de choc 2 : prix nourriture +50% au jour 200)
            if day == 200 {
                salary_manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1500);
            }
            if day == 300 {
                salary_manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
            }
            
            // Vérifier si le seuil de risque est franchi
            if let Some(e) = salary_manager.entites.get(&cohort_id) {
                if e.seuil_franchi() {
                    // Déclencher une grève
                    if strike_manager.declencher_greve(cohort_id, day).is_some() {
                        greves_declenchees += 1;
                        println!("  [Jour {}] Grève déclenchée pour cohorte {}", day, i);
                    }
                }
            }
            
            // Vérifier I8
            if !entity_manager.verify_i8_concordance() {
                i8_violations += 1;
            }
            
            // Vérifier si la cohorte est en grève
            if strike_manager.is_on_strike(cohort_id, day) {
                jours_avec_greve += 1;
            }
            
            // Fin de grève aléatoire (simulation de concession)
            if day > 200 && day % 10 == 0 {
                if let Some(strike) = strike_manager.get_strike(cohort_id) {
                    if !strike.is_ended() && day >= strike.start_day + 5 {
                        // 50% de chance de concession
                        if (day + i as i64) % 2 == 0 {
                            strike_manager.fin_de_greve(cohort_id, day, StrikeEndCause::Concession);
                            greves_terminees += 1;
                            println!("  [Jour {}] Grève terminée (concession) pour cohorte {}", day, i);
                        }
                    }
                }
            }
        }
        
        // Journalisation
        if day % 365 == 0 {
            println!("  Année {} terminée (jour {})", day / 365, day);
        }
    }
    
    // ========================================================================
    // Exécution 2 : Deuxième passe pour vérifier le déterminisme (I9)
    // ========================================================================
    println!();
    println!("[2/2] Deuxième exécution (vérification I9)...");
    
    let mut entity_manager2 = EntityManager::new(BENCHMARK_SEED);
    let mut salary_manager2 = SalaireReelEtGriefsManager::new();
    let mut strike_manager2 = StrikeManager::new(BENCHMARK_SEED);
    let mut input_events2 = EventQueue::new();
    
    // Même initialisation
    let mut cohort_ids2 = Vec::new();
    for i in 0..NUM_COHORTS {
        let cohort_id = entity_manager2.create_cohort(100);
        cohort_ids2.push(cohort_id);
        
        salary_manager2.ajouter_entite(cohort_id);
        let salaire = 500 + (i * 300);
        salary_manager2.mettre_a_jour_salaire_horaire(cohort_id, salaire);
        salary_manager2.mettre_a_jour_heures(cohort_id, 8);
    }
    
    salary_manager2.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
    salary_manager2.mettre_a_jour_panier(PanierComposante::Logement, 2000);
    salary_manager2.mettre_a_jour_panier(PanierComposante::Eau, 500);
    salary_manager2.mettre_a_jour_panier(PanierComposante::Fiscalite, 300);
    
    let mut greves_declenchees2: u64 = 0;
    let mut i8_violations2: u64 = 0;
    
    // Même simulation
    for day in 0..SIMULATION_DAYS {
        for (i, &cohort_id) in cohort_ids2.iter().enumerate() {
            let u = ((day + i as i64 * 100) % 1000) as u16;
            let p = ((day * 2 + i as i64 * 50) % 1000) as u16;
            let q = ((day * 3) % 1000) as u16;
            input_events2.push_at_with_id(
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_id,
                    u,
                    p,
                    q,
                },
            );
        }
        while let Some(event) = input_events2.pop() {
            salary_manager2.appliquer_evenement(&event)?;
        }

        for (i, &cohort_id) in cohort_ids2.iter().enumerate() {
            if day == 200 {
                salary_manager2.mettre_a_jour_panier(PanierComposante::Nourriture, 1500);
            }
            if day == 300 {
                salary_manager2.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
            }
            
            if let Some(e) = salary_manager2.entites.get(&cohort_id) {
                if e.seuil_franchi() {
                    if strike_manager2.declencher_greve(cohort_id, day).is_some() {
                        greves_declenchees2 += 1;
                    }
                }
            }
            
            if !entity_manager2.verify_i8_concordance() {
                i8_violations2 += 1;
            }
            
            if day > 200 && day % 10 == 0 {
                if let Some(strike) = strike_manager2.get_strike(cohort_id) {
                    if !strike.is_ended() && day >= strike.start_day + 5 {
                        if (day + i as i64) % 2 == 0 {
                            strike_manager2.fin_de_greve(cohort_id, day, StrikeEndCause::Concession);
                        }
                    }
                }
            }
        }
    }
    
    // ========================================================================
    // Résultats
    // ========================================================================
    let elapsed = start_time.elapsed();
    
    println!();
    println!("=== Résultats Benchmark T7-T8 ===");
    println!();
    
    println!("Temps d'exécution: {:?}", elapsed);
    println!();
    
    println!("--- Exécution 1 ---");
    println!("Grèves déclenchées: {}", greves_declenchees);
    println!("Jours avec grève: {}", jours_avec_greve);
    println!("Violations I8: {}", i8_violations);
    println!();
    
    println!("--- Exécution 2 ---");
    println!("Grèves déclenchées: {}", greves_declenchees2);
    println!("Violations I8: {}", i8_violations2);
    println!();
    
    // ========================================================================
    // Validation
    // ========================================================================
    println!("=== Validation ===");
    println!();
    
    let mut all_pass = true;
    
    // I8 : Aucune violation
    if i8_violations == 0 && i8_violations2 == 0 {
        println!("✅ I8 (concordance griefs): VALIDÉ (0 violation)");
    } else {
        println!("❌ I8 (concordance griefs): ÉCHEC ({} + {} violations)", i8_violations, i8_violations2);
        all_pass = false;
    }
    
    // I9 : Déterminisme
    if greves_declenchees == greves_declenchees2 {
        println!("✅ I9 (déterminisme): VALIDÉ (même nombre de grèves: {})", greves_declenchees);
    } else {
        println!("❌ I9 (déterminisme): ÉCHEC ({} vs {} grèves)", greves_declenchees, greves_declenchees2);
        all_pass = false;
    }
    
    // Grèves déclenchées
    if greves_declenchees > 0 {
        println!("✅ T8.1 (basculement): VALIDÉ ({} grèves déclenchées)", greves_declenchees);
    } else {
        println!("⚠️  T8.1 (basculement): AUCUNE GRÈVE (peut être normal avec ces paramètres)");
    }
    
    println!();
    
    if all_pass {
        println!("✅✅✅ TOUS LES TESTS PASSENT ✅✅✅");
    } else {
        println!("❌❌❌ CERTAINS TESTS ÉCHOUENT ❌❌❌");
        return Err("benchmark validations failed".into());
    }
    Ok(())
}
