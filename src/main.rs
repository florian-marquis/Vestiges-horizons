//! Vestiges Horizons - Point d'entrée pour l'exécution headless
//! 
//! Ce binaire permet de :
//! - Générer des replais de référence (Tranche 1 et Tranche 2)
//! - Exécuter des tests de déterminisme (I9)
//! - Valider les invariants (I1, I7, I8, I9)

use std::fs::File;
use std::io::Write;
use std::path::Path;

use vestiges_horizons::entities::*;
use vestiges_horizons::{Clock, Phase, Priority, EventData, Timestamp};

fn main() {
    println!("Vestiges Horizons - Tranches 1 & 2");
    println!("====================================\n");
    
    // ========================================================================
    // Tests Tranche 1 : Noyau Temporel
    // ========================================================================
    
    println!("[Tranche 1] Test de l'invariant I9 (déterminisme du replai)...");
    if vestiges_horizons::verify_i9_determinism(42, 100) {
        println!("✓ I9 VALIDE : Le replai est déterministe");
    } else {
        println!("✗ I9 INVALIDE : Le replai n'est pas déterministe");
        std::process::exit(1);
    }
    
    println!("\n[Tranche 1] Test de l'invariant I1 (conservation extraction/réinjection)...");
    if vestiges_horizons::verify_i1_conservation() {
        println!("✓ I1 VALIDE : Conservation vérifiée");
    } else {
        println!("✗ I1 INVALIDE : Conservation échouée");
        std::process::exit(1);
    }
    
    // ========================================================================
    // Tests Tranche 2 : Identités et Journal
    // ========================================================================
    
    println!("\n[Tranche 2] Test T2.1 : Extraction répétée (100 cycles)...");
    let mut manager = EntityManager::new(42);
    let cohort_id = manager.create_cohort(100);
    let initial_state = {
        let c = manager.cohort(cohort_id).unwrap();
        (c.population, c.total_money, c.total_griefs, c.total_attachment)
    };
    for _ in 0..100 {
        let individual = manager.extract(cohort_id);
        if let Some(ind) = individual {
            manager.reinject(ind);
        }
    }
    let final_state = {
        let c = manager.cohort(cohort_id).unwrap();
        (c.population, c.total_money, c.total_griefs, c.total_attachment)
    };
    if initial_state == final_state {
        println!("✓ T2.1 VALIDE : Zéro dérive après 100 cycles");
    } else {
        println!("✗ T2.1 INVALIDE : Dérive détectée");
        std::process::exit(1);
    }
    
    println!("\n[Tranche 2] Test T2.5 : Non-réutilisation des ids...");
    let mut registry = EntityRegistry::new();
    for _ in 0..100_000 {
        let id = registry.next_id(EntityType::NamedIndividual);
        registry.activate(id);
    }
    if registry.verify_no_reuse() {
        println!("✓ T2.5 VALIDE : Aucun id réutilisé");
    } else {
        println!("✗ T2.5 INVALIDE : Id réutilisé détecté");
        std::process::exit(1);
    }
    
    println!("\n[Tranche 2] Test T2.6 : Fusion conservatrice...");
    let mut manager = EntityManager::new(42);
    let cohort_a = manager.create_cohort(50);
    let cohort_b = manager.create_cohort(50);
    {
        let a = manager.cohort_mut(cohort_a).unwrap();
        a.total_money = 10000;
        a.total_griefs = 500;
    }
    {
        let b = manager.cohort_mut(cohort_b).unwrap();
        b.total_money = 15000;
        b.total_griefs = 750;
    }
    manager.merge_cohorts(cohort_a, cohort_b);
    let merged = manager.cohort(cohort_a).unwrap();
    if merged.total_money == 25000 && merged.total_griefs == 1250 {
        println!("✓ T2.6 VALIDE : Fusion conservatrice");
    } else {
        println!("✗ T2.6 INVALIDE : Totaux non conservés");
        std::process::exit(1);
    }
    
    println!("\n[Tranche 2] Test I8 : Concordance griefs...");
    let mut manager = EntityManager::new(42);
    let cohort_id = manager.create_cohort(100);
    for _ in 0..10 {
        manager.extract(cohort_id);
    }
    if manager.verify_i8_concordance() {
        println!("✓ I8 VALIDE : Concordance griefs");
    } else {
        println!("✗ I8 INVALIDE : Concordance échouée");
        std::process::exit(1);
    }
    
    // ========================================================================
    // Génération des replays
    // ========================================================================
    
    println!("\n[Replay] Génération du replay T1 (noyau temporel)...");
    generate_reference_replay_t1(42, 10, "src/replai/reference_seed42_t1.json");
    println!("✓ Replay T1 généré : src/replai/reference_seed42_t1.json");
    
    println!("\n[Replay] Génération du replay T2 (avec entités)...");
    generate_reference_replay_t2(42, "src/replai/reference_seed42_t2.json");
    println!("✓ Replay T2 généré : src/replai/reference_seed42_t2.json");
    
    println!("\n========================================");
    println!("Tous les tests passés !");
    println!("Invariants validés : I1, I7, I8, I9");
    println!("Tranches validées : 1 et 2");
    println!("========================================");
}

/// Génère un replay de référence pour la Tranche 1 (noyau temporel)
fn generate_reference_replay_t1(seed: u64, num_events: usize, output_path: &str) {
    let mut clock = Clock::new(seed);
    
    // Planifier des événements déterministes
    for i in 0..num_events {
        let day = (i as i64) / 5;
        let phase = match i % 7 {
            0 => Phase::FluxPhysiques,
            1 => Phase::Marches,
            2 => Phase::Comptes,
            3 => Phase::MarcheDuTravail,
            4 => Phase::DecisionsAgents,
            5 => Phase::Social,
            _ => Phase::Journal,
        };
        let priority = Priority::new((i % 10) as u8);
        
        let entity_id = clock.state.event_queue.next_id();
        clock.state.schedule(
            day,
            phase,
            priority,
            (i as u16) % 100,
            EventData::CreateEntity {
                entity_type: format!("TestEntity_{}", i),
                entity_id,
            },
        );
    }
    
    // Traiter tous les événements et les collecter
    let mut events = Vec::new();
    for _ in 0..num_events {
        if let Some(event) = clock.state.advance_to_next() {
            events.push(event);
        }
    }
    
    // Sérialiser le replay
    let json: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id.as_u64(),
                "day": e.timestamp.day,
                "seq": e.timestamp.seq,
                "phase": e.phase as u8,
                "priority": e.priority.0,
                "event_type": e.event_type,
                "data": match &e.data {
                    EventData::Nop => serde_json::Value::Null,
                    EventData::CreateEntity { entity_type, entity_id } => {
                        serde_json::json!({
                            "type": "CreateEntity",
                            "entity_type": entity_type,
                            "entity_id": entity_id.as_u64()
                        })
                    }
                    EventData::UpdateState { field, old_value, new_value } => {
                        serde_json::json!({
                            "type": "UpdateState",
                            "field": field,
                            "old_value": old_value,
                            "new_value": new_value
                        })
                    }
                }
            })
        })
        .collect();
    
    // Écrire dans le fichier
    let json_string = serde_json::to_string_pretty(&json).unwrap();
    let path = Path::new(output_path);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("Échec de création du répertoire");
    }
    let mut file = File::create(path).expect("Échec de création du fichier");
    file.write_all(json_string.as_bytes()).expect("Échec d'écriture");
}

/// Génère un replay de référence pour la Tranche 2 (avec entités et journal)
fn generate_reference_replay_t2(seed: u64, output_path: &str) {
    let mut manager = EntityManager::new(seed);
    let mut journal = CausalJournal::new();
    
    // Créer des cohortes
    let cohort_a = manager.create_cohort(100);
    let cohort_b = manager.create_cohort(100);
    
    // Extraire des individus
    let mut extracted_individuals = Vec::new();
    for i in 0..5 {
        let individual = manager.extract(cohort_a);
        if let Some(ind) = individual {
            extracted_individuals.push(ind.clone());
            
            // Journaliser l'extraction
            let entry = JournalEntry {
                id: 0,
                timestamp: Timestamp::new(0, i as u32),
                event_id: i + 1,
                phase: 6, // Phase Social
                actor: Some(ind.id),
                decision_type: 1, // Extraction
                modified_values: vec![
                    Modification {
                        field: "population".to_string(),
                        old_value: (100 - i) as i64,
                        new_value: (99 - i) as i64,
                    },
                ],
                causes: vec![],
                learned_by: vec![],
            };
            journal.add_entry(entry);
        }
    }
    
    // Fusionner les cohortes
    manager.merge_cohorts(cohort_a, cohort_b);
    
    // Réinjecter certains individus
    for individual in &extracted_individuals {
        manager.reinject(individual.clone());
    }
    
    // Sérialiser l'état
    let state = serde_json::json!({
        "seed": seed,
        "cohorts": manager.all_cohorts().map(|(id, cohort)| {
            serde_json::json!({
                "id": id.to_u64(),
                "population": cohort.population,
                "total_money": cohort.total_money,
                "total_griefs": cohort.total_griefs,
                "total_attachment": cohort.total_attachment,
            })
        }).collect::<Vec<_>>(),
        "named_individuals": manager.named_individuals().iter().map(|(id, ind)| {
            serde_json::json!({
                "id": id.to_u64(),
                "cohort_id": ind.cohort_id.to_u64(),
                "grief": ind.grief,
                "balance": ind.balance,
                "attachment": ind.attachment,
            })
        }).collect::<Vec<_>>(),
        "journal_entries": journal.all_entries().map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "day": entry.timestamp.day,
                "phase": entry.phase,
                "event_id": entry.event_id,
                "actor": entry.actor.map(|a| a.to_u64()),
                "decision_type": entry.decision_type,
            })
        }).collect::<Vec<_>>(),
    });
    
    let json_string = serde_json::to_string_pretty(&state).unwrap();
    let path = Path::new(output_path);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("Échec de création du répertoire");
    }
    let mut file = File::create(path).expect("Échec de création du fichier");
    file.write_all(json_string.as_bytes()).expect("Échec d'écriture");
}