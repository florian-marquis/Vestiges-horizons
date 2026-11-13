//! Tests du noyau temporel - Invariant I9 (déterminisme)

extern crate vestiges_horizons;

use vestiges_horizons::*;

#[test]
fn test_i9_determinisme_de_base() {
    assert!(verify_i9_determinism(42, 100));
}

#[test]
fn test_i9_determinisme_plusieurs_graines() {
    for seed in [0, 42, 100, 999, u64::MAX] {
        assert!(
            verify_i9_determinism(seed, 50),
            "I9 échoué pour la graine {}",
            seed
        );
    }
}

#[test]
fn test_i9_ordre_insertion_permute() {
    // Deux clocks avec les mêmes événements mais ordre d'insertion différent
    let seed = 12345;
    
    let mut clock1 = Clock::new(seed);
    let mut clock2 = Clock::new(seed);
    
    // Inserer dans l'ordre normal
    for i in 0..20 {
        let day = (i as i64) / 10;
        let phase = Phase::from_u8((i % 7 + 1) as u8).unwrap();
        clock1.state.schedule(day, phase, Priority::new(i as u8), i as u16, EventData::Nop);
    }
    
    // Inserer dans un ordre permuté (inversé)
    let mut order: Vec<usize> = (0..20).collect();
    order.reverse();
    for &i in &order {
        let day = (i as i64) / 10;
        let phase = Phase::from_u8((i % 7 + 1) as u8).unwrap();
        clock2.state.schedule(day, phase, Priority::new(i as u8), i as u16, EventData::Nop);
    }
    
    // Les deux clocks devraient produire les mêmes événements dans le même ordre
    let mut events1: Vec<u64> = Vec::new();
    let mut events2: Vec<u64> = Vec::new();
    
    for _ in 0..20 {
        if let Some(e) = clock1.state.advance_to_next() {
            events1.push(e.id.as_u64());
        }
        if let Some(e) = clock2.state.advance_to_next() {
            events2.push(e.id.as_u64());
        }
    }
    
    assert_eq!(events1, events2, "I9 : même graine = même journal indépendamment de l'ordre d'insertion");
}

#[test]
fn test_i1_conservation_basique() {
    assert!(verify_i1_conservation());
}

#[test]
fn test_i1_conservation_avec_plusieurs_phases() {
    let mut queue = EventQueue::new();
    
    // Créer un ensemble d'événements avec différentes phases
    let phases = [
        Phase::FluxPhysiques,
        Phase::Marches,
        Phase::Comptes,
        Phase::MarcheDuTravail,
    ];
    
    let ids: Vec<Id> = (0..50)
        .map(|_| queue.next_id())
        .collect();
    
    // Ajouter des événements avec différentes phases et priorités
    for (i, &id) in ids.iter().enumerate() {
        let day = (i / 10) as i64;
        let phase = phases[i % phases.len()];
        let priority = Priority::new((i % 5) as u8);
        
        queue.push(Event::new(
            id,
            day,
            i as u32,
            phase,
            priority,
            (i % 100) as u16,
            EventData::CreateEntity {
                entity_type: format!("Type_{}", i % 5),
                entity_id: Id::new(i as u64),
            },
        ));
    }
    
    // Extraire tous les événements
    let mut extracted = Vec::new();
    while let Some(event) = queue.pop() {
        extracted.push(event);
    }
    
    // Vérifier que les ids sont bien conservés et dans l'ordre attendu
    // (l'ordre devrait être déterministe : jour -> phase -> priorité -> id)
    for i in 0..extracted.len() - 1 {
        assert!(
            extracted[i] < extracted[i + 1],
            "Les événements ne sont pas dans l'ordre canonique"
        );
    }
    
    // Vérifier que tous les ids sont présents
    let mut found_ids: Vec<Id> = extracted.iter().map(|e| e.id).collect();
    found_ids.sort();
    
    let mut expected_ids = ids.clone();
    expected_ids.sort();
    
    assert_eq!(found_ids, expected_ids, "I1 : tous les ids doivent être conservés");
}

#[test]
fn test_ordre_canonique_phases() {
    // Vérifier que Phase::FluxPhysiques (1) < Phase::Marches (2) < ...
    assert!(Phase::FluxPhysiques < Phase::Marches);
    assert!(Phase::Marches < Phase::Comptes);
    assert!(Phase::Comptes < Phase::MarcheDuTravail);
    assert!(Phase::MarcheDuTravail < Phase::DecisionsAgents);
    assert!(Phase::DecisionsAgents < Phase::Social);
    assert!(Phase::Social < Phase::Journal);
}

#[test]
fn test_event_queue_maintient_ordre() {
    let mut queue = EventQueue::new();
    
    // Ajouter des événements dans un ordre aléatoire
    let events_to_add = vec![
        (1, Phase::FluxPhysiques, 3),
        (0, Phase::FluxPhysiques, 1),
        (0, Phase::Comptes, 0),
        (0, Phase::FluxPhysiques, 2),
        (2, Phase::FluxPhysiques, 0),
        (0, Phase::Marches, 0),
    ];
    
    let ids: Vec<Id> = events_to_add.iter()
        .map(|_| queue.next_id())
        .collect();
    
    for ((day, phase, priority), &id) in events_to_add.iter().zip(ids.iter()) {
        queue.push(Event::new(
            id,
            *day,
            0,
            *phase,
            Priority::new(*priority),
            0,
            EventData::Nop,
        ));
    }
    
    // Extraire et vérifier l'ordre
    let mut extracted = Vec::new();
    while let Some(event) = queue.pop() {
        extracted.push(event);
    }
    
    // L'ordre attendu : 
    // jour 0: Phase::FluxPhysiques priorité 1 (id=1), puis priorité 2 (id=3), puis priorité 3 (id=0)
    // puis Phase::Comptes priorité 0 (id=2), puis Phase::Marches priorité 0 (id=5)
    // puis jour 1: Phase::FluxPhysiques priorité 0 (id=4)
    // puis jour 2: Phase::FluxPhysiques priorité 0 (id=?)
    
    // Vérifier que l'extraction suit bien l'ordre canonique
    for i in 0..extracted.len() - 1 {
        assert!(
            extracted[i] < extracted[i + 1],
            "Ordre canonique violé entre {} et {}",
            i, i + 1
        );
    }
}
