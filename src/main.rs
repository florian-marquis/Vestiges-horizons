//! Vestiges Horizons - Point d'entrée pour l'exécution headless
//! 
//! Ce binaire permet de :
//! - Générer des replais de référence
//! - Exécuter des tests de déterminisme (I9)
//! - Valider les invariants

use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("Vestiges Horizons - Noyau Temporel (Tranche 1)");
    println!("==============================================\n");
    
    // Test de l'invariant I9 (déterminisme)
    println!("Test de l'invariant I9 (déterminisme du replai)...");
    if vestiges_horizons::verify_i9_determinism(42, 100) {
        println!("✓ I9 VALIDE : Le replai est déterministe");
    } else {
        println!("✗ I9 INVALIDE : Le replai n'est pas déterministe");
        std::process::exit(1);
    }
    
    // Test de l'invariant I1 (conservation)
    println!("\nTest de l'invariant I1 (conservation extraction/réinjection)...");
    if vestiges_horizons::verify_i1_conservation() {
        println!("✓ I1 VALIDE : Conservation vérifiée");
    } else {
        println!("✗ I1 INVALIDE : Conservation échouée");
        std::process::exit(1);
    }
    
    // Générer un replay de référence minimal
    println!("\nGénération du replay de référence (seed=42, 10 événements)...");
    generate_reference_replay(42, 10, "src/replai/reference_seed42.json");
    println!("✓ Replay généré : src/replai/reference_seed42.json");
    
    println!("\nTous les tests passés. Invariants I1 et I9 validés.");
}

/// Génère un replay de référence avec la graine donnée.
fn generate_reference_replay(seed: u64, num_events: usize, output_path: &str) {
    use vestiges_horizons::{Clock, Phase, Priority, EventData};
    
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
