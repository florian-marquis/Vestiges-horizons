//! Tests du noyau temporel - Tranche 1
//! Vérifie le bon fonctionnement du code implémenté

use vestiges_horizons::{
    Clock, Event, EventData, EventQueue, Id, Phase, Priority, SimulationState, Timestamp,
};

#[test]
fn test_event_queue_ordering() {
    let mut queue = EventQueue::new();
    
    // Ajouter des événements dans un ordre non-trié
    let e3 = Event::new(Id::new(3), 1, 0, Phase::FluxPhysiques, Priority::new(0), 0, EventData::Nop);
    let e1 = Event::new(Id::new(1), 0, 0, Phase::Comptes, Priority::new(0), 0, EventData::Nop);
    let e2 = Event::new(Id::new(2), 0, 0, Phase::FluxPhysiques, Priority::new(1), 0, EventData::Nop);
    
    queue.push(e1.clone());
    queue.push(e2.clone());
    queue.push(e3.clone());
    
    // Vérifier l'ordre : e1 (jour 0, phase 2=Comptes) doit venir avant e2 (jour 0, phase 1=FluxPhysiques)? 
    // Non! Phase::FluxPhysiques = 1, Phase::Comptes = 3, donc FluxPhysiques < Comptes
    // Donc l'ordre devrait être: e2 (FluxPhysiques, priorité 1), puis e1 (Comptes, priorité 0)?
    // Non, attendez: e2 a Phase::FluxPhysiques (1) et e1 a Phase::Comptes (3), donc e2 < e1
    // Mais e2 a priorité 1 et e1 a priorité 0, mais la phase vient avant la priorité
    // Donc e2 (phase 1) < e1 (phase 3) regardless de priorité
    
    let mut events = queue.sorted_events();
    assert_eq!(events.len(), 3);
    
    // Le premier devrait être e2 (phase 1 = FluxPhysiques)
    // Puis e1 (phase 3 = Comptes)
    // Puis e3 (jour 1)
    assert_eq!(events[0].id, Id::new(2)); // e2
    assert_eq!(events[1].id, Id::new(1)); // e1
    assert_eq!(events[2].id, Id::new(3)); // e3
}

#[test]
fn test_timestamp_ordering() {
    let t1 = Timestamp::new(0, 1);
    let t2 = Timestamp::new(0, 0);
    let t3 = Timestamp::new(1, 0);
    
    assert!(t2 < t1); // Même jour, seq 0 < seq 1
    assert!(t1 < t3); // Jour 0 < jour 1
}

#[test]
fn test_clock_process_day() {
    let mut clock = Clock::new(42);
    
    // Planifier des événements pour le jour 0
    clock.state.schedule(0, Phase::FluxPhysiques, Priority::new(0), 0, EventData::Nop);
    clock.state.schedule(0, Phase::FluxPhysiques, Priority::new(1), 0, EventData::Nop);
    clock.state.schedule(1, Phase::FluxPhysiques, Priority::new(0), 0, EventData::Nop);
    
    // Traiter le jour 0
    let processed = clock.process_day(0);
    assert_eq!(processed.len(), 2);
    
    // Il reste 1 événement (jour 1)
    assert_eq!(clock.state.event_queue.len(), 1);
}

#[test]
fn test_serialization_roundtrip() {
    use vestiges_horizons::SerializableEvent;
    
    let event = Event::new(
        Id::new(123),
        5,
        10,
        Phase::Social,
        Priority::new(50),
        100,
        EventData::Nop,
    );
    
    let serializable: SerializableEvent = (&event).into();
    let deserialized: Event = serializable.into();
    
    assert_eq!(deserialized.id, event.id);
    assert_eq!(deserialized.timestamp.day, event.timestamp.day);
    assert_eq!(deserialized.timestamp.seq, event.timestamp.seq);
    assert_eq!(deserialized.phase, event.phase);
    assert_eq!(deserialized.priority.0, event.priority.0);
    assert_eq!(deserialized.event_type, event.event_type);
}

#[test]
fn test_determinism() {
    // Deux clocks avec la même graine doivent produire les mêmes événements
    let seed = 42;
    let mut clock1 = Clock::new(seed);
    let mut clock2 = Clock::new(seed);
    
    // Planifier les mêmes événements
    for i in 0..10 {
        let day = (i as i64) / 5;
        let phase = Phase::FluxPhysiques;
        let priority = Priority::new((i % 5) as u8);
        
        clock1.state.schedule(day, phase, priority, 0, EventData::Nop);
        clock2.state.schedule(day, phase, priority, 0, EventData::Nop);
    }
    
    // Traiter et vérifier que les ids sont dans le même ordre
    let mut ids1 = Vec::new();
    let mut ids2 = Vec::new();
    
    for _ in 0..10 {
        if let Some(e) = clock1.state.advance_to_next() {
            ids1.push(e.id);
        }
        if let Some(e) = clock2.state.advance_to_next() {
            ids2.push(e.id);
        }
    }
    
    assert_eq!(ids1, ids2);
}

#[test]
fn test_event_data_variants() {
    let queue = EventQueue::new();
    
    let e1 = Event::new(
        Id::new(1),
        0,
        0,
        Phase::FluxPhysiques,
        Priority::new(0),
        0,
        EventData::Nop,
    );
    
    let e2 = Event::new(
        Id::new(2),
        0,
        0,
        Phase::FluxPhysiques,
        Priority::new(1),
        0,
        EventData::CreateEntity {
            entity_type: "Agent".to_string(),
            entity_id: Id::new(100),
        },
    );
    
    let e3 = Event::new(
        Id::new(3),
        0,
        0,
        Phase::FluxPhysiques,
        Priority::new(2),
        0,
        EventData::UpdateState {
            field: "health".to_string(),
            old_value: 100,
            new_value: 80,
        },
    );
    
    // Just verify we can create different event data variants
    assert!(matches!(e1.data, EventData::Nop));
    assert!(matches!(e2.data, EventData::CreateEntity { .. }));
    assert!(matches!(e3.data, EventData::UpdateState { .. }));
}
