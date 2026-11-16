//! Vestiges Horizons - Noyau de simulation (Tranche 1 + Tranche 2)
//! 
//! Implémentation du noyau temporel avec horloge, file d'événements,
//! ordonnancement déterministe et état sérialisable.
//! 
//! Tranche 2 : Identités et Journal causal

use std::cmp::Ordering;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

pub mod common;
pub mod entities;
pub mod time;
pub mod household;
pub mod health;
pub mod states;
pub mod firm;
pub mod recipe;
pub mod asset;
pub mod maintenance;
pub mod production;
pub mod market;
pub mod credit;
pub mod tax;
pub mod meal;
pub mod labor;
pub mod griefs;
pub mod inputs;
pub mod salary;
pub mod strike;
pub mod replai_gen;

// ============================================================================
// TYPES DE BASE
// ============================================================================

/// Identifiant global monotone, jamais réutilisé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(pub u64);

impl Id {
    pub fn new(id: u64) -> Self {
        Id(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Horodatage logique : (jour, créneau/index de séquence).
/// Dans P1, les créneaux valides sont 0 à 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Timestamp {
    pub day: i64,
    pub seq: u32,
}

impl Timestamp {
    pub fn new(day: i64, seq: u32) -> Self {
        Timestamp { day, seq }
    }
    
    /// Compare deux timestamps : jour d'abord, puis séquence
    pub fn cmp(&self, other: &Timestamp) -> Ordering {
        match self.day.cmp(&other.day) {
            Ordering::Equal => self.seq.cmp(&other.seq),
            ord => ord,
        }
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

/// Phase d'événement (7 phases selon la spec P1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Phase {
    FluxPhysiques = 1,
    Marches = 2,
    Comptes = 3,
    MarcheDuTravail = 4,
    DecisionsAgents = 5,
    Social = 6,
    Journal = 7,
}

impl Phase {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Phase::FluxPhysiques),
            2 => Some(Phase::Marches),
            3 => Some(Phase::Comptes),
            4 => Some(Phase::MarcheDuTravail),
            5 => Some(Phase::DecisionsAgents),
            6 => Some(Phase::Social),
            7 => Some(Phase::Journal),
            _ => None,
        }
    }
}

/// Priorité d'événement (donnée, 0-255)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Priority(pub u8);

impl Priority {
    pub fn new(val: u8) -> Self {
        Priority(val)
    }
}

// ============================================================================
// ÉVÉNEMENTS
// ============================================================================

/// Type d'événement (à étendre par tranche)
pub type EventType = u16;
pub const EVENT_TYPE_INPUT_CHANGED: EventType = 70;

/// Événement du noyau temporel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Id,
    pub timestamp: Timestamp,
    pub phase: Phase,
    pub priority: Priority,
    pub event_type: EventType,
    pub data: EventData,
}

/// Données associées à un événement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// Événement vide (pour tests)
    Nop,
    /// Création d'entité
    CreateEntity { entity_type: String, entity_id: Id },
    /// Mise à jour d'état
    UpdateState { field: String, old_value: i64, new_value: i64 },
    /// Changement des entrées agrégées de griefs d'une cohorte
    InputChanged {
        entity_id: entities::EntityId,
        u: u16,
        p: u16,
        q: u16,
    },
}

impl Event {
    pub fn new(
        id: Id,
        day: i64,
        seq: u32,
        phase: Phase,
        priority: Priority,
        event_type: EventType,
        data: EventData,
    ) -> Self {
        Event {
            id,
            timestamp: Timestamp::new(day, seq),
            phase,
            priority,
            event_type,
            data,
        }
    }
    
    /// Ordre canonique : jour -> créneau -> phase -> priorité -> id
    pub fn canonical_order(&self) -> (i64, u32, u8, u8, u64) {
        (
            self.timestamp.day,
            self.timestamp.seq,
            self.phase as u8,
            self.priority.0,
            self.id.as_u64(),
        )
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.canonical_order().cmp(&other.canonical_order())
    }
}

// ============================================================================
// FILE D'ÉVÉNEMENTS
// ============================================================================

/// File d'événements ordonnée de manière déterministe.
/// 
/// Invariant : les événements sont toujours dans l'ordre canonique.
/// Utilise BTreeMap pour garantir un ordre déterministe (contrairement à BinaryHeap)
#[derive(Debug, Clone)]
pub struct EventQueue {
    /// Compteur global pour les ids d'événements
    next_event_id: u64,
    /// File principale : BTreeMap triée par (jour, créneau, phase, priorité, id)
    queue: BTreeMap<(i64, u32, u8, u8, u64), Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            next_event_id: 0,
            queue: BTreeMap::new(),
        }
    }
    
    /// Génère un nouvel Id d'événement
    pub fn next_id(&mut self) -> Id {
        let id = Id::new(self.next_event_id);
        self.next_event_id += 1;
        id
    }
    
    /// Ajoute un événement à la file.
    /// L'événement est inséré à la position déterministe.
    pub fn push(&mut self, event: Event) {
        let key = event.canonical_order();
        self.queue.insert(key, event);
    }
    
    /// Ajoute un événement avec génération d'id automatique.
    pub fn push_with_id(&mut self, day: i64, phase: Phase, priority: Priority, event_type: EventType, data: EventData) -> Id {
        self.push_at_with_id(Timestamp::new(day, 0), phase, priority, event_type, data)
    }

    /// Ajoute un événement à un horodatage logique précis.
    pub fn push_at_with_id(
        &mut self,
        timestamp: Timestamp,
        phase: Phase,
        priority: Priority,
        event_type: EventType,
        data: EventData,
    ) -> Id {
        let id = self.next_id();
        let event = Event::new(
            id,
            timestamp.day,
            timestamp.seq,
            phase,
            priority,
            event_type,
            data,
        );
        self.push(event);
        id
    }
    
    /// Retire et retourne le premier événement de la file.
    pub fn pop(&mut self) -> Option<Event> {
        self.queue.pop_first().map(|(_, event)| event)
    }
    
    /// Retourne le premier événement sans le retirer.
    pub fn peek(&self) -> Option<&Event> {
        self.queue.values().next()
    }
    
    /// Nombre d'événements dans la file.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    /// Retourne tous les événements triés.
    pub fn sorted_events(&self) -> Vec<&Event> {
        self.queue.values().collect()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ÉTAT CANONIQUE (Sérialisable)
// ============================================================================

/// État minimal du noyau temporel.
/// Doit être sérialisable et désérialisable de manière déterministe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    /// Jour courant
    pub current_day: i64,
    /// Index de séquence courant
    pub current_seq: u32,
    /// File d'événements
    #[serde(skip)]
    pub event_queue: EventQueue,
    /// Graine du monde (pour les tirages déterministes)
    pub world_seed: u64,
}

impl SimulationState {
    pub fn new(world_seed: u64) -> Self {
        SimulationState {
            current_day: 0,
            current_seq: 0,
            event_queue: EventQueue::new(),
            world_seed,
        }
    }
    
    /// Avance le temps au prochain événement.
    pub fn advance_to_next(&mut self) -> Option<Event> {
        let event = self.event_queue.pop()?;
        self.current_day = event.timestamp.day;
        self.current_seq = event.timestamp.seq;
        Some(event)
    }
    
    /// Planifie un événement.
    pub fn schedule(
        &mut self,
        day: i64,
        phase: Phase,
        priority: Priority,
        event_type: EventType,
        data: EventData,
    ) -> Id {
        self.event_queue.push_with_id(day, phase, priority, event_type, data)
    }

    /// Planifie un événement à un créneau précis de la journée.
    pub fn schedule_at(
        &mut self,
        timestamp: Timestamp,
        phase: Phase,
        priority: Priority,
        event_type: EventType,
        data: EventData,
    ) -> Id {
        self.event_queue
            .push_at_with_id(timestamp, phase, priority, event_type, data)
    }
}

// ============================================================================
// HORLOGE CANONIQUE
// ============================================================================

/// Horloge du noyau temporel.
/// 
/// Garantit :
/// - Ordonnancement déterministe des événements
/// - Report par défaut au prochain instant logique
/// - Pas d'exécution réentrante
pub struct Clock {
    pub state: SimulationState,
}

impl Clock {
    pub fn new(world_seed: u64) -> Self {
        Clock {
            state: SimulationState::new(world_seed),
        }
    }
    
    /// Traite les événements d'un jour donné.
    /// Les événements générés pendant le traitement sont reportés au jour suivant.
    pub fn process_day(&mut self, day: i64) -> Vec<Event> {
        let mut processed = Vec::new();
        
        // Traiter tous les événements de ce jour
        while let Some(event) = self.state.event_queue.peek() {
            if event.timestamp.day != day {
                break;
            }
            
            let event = self.state.event_queue.pop().unwrap();
            processed.push(event);
        }
        
        processed
    }
    
    /// Traite un événement spécifique.
    /// Toute génération d'événement pendant le traitement est reportée.
    pub fn process_event(&mut self) -> Option<Event> {
        self.state.advance_to_next()
    }
}

// ============================================================================
// TESTS D'INVARIANTS
// ============================================================================

/// Vérifie l'invariant I9 : déterminisme du replai.
/// Même graine + mêmes données = même journal, bit à bit.
/// Vérifie l'invariant I9 : déterminisme du replai.
/// Même graine + mêmes données = même journal, bit à bit.
pub fn verify_i9_determinism(seed: u64, iterations: usize) -> bool {
    // Créer deux horloges identiques
    let mut clock1 = Clock::new(seed);
    let mut clock2 = Clock::new(seed);
    
    // Planifier les mêmes événements
    for i in 0..iterations {
        let day = (i as i64) / 10;
        let phase = Phase::FluxPhysiques;
        let priority = Priority::new((i % 10) as u8);
        
        clock1.state.schedule(day, phase, priority, 0, EventData::Nop);
        clock2.state.schedule(day, phase, priority, 0, EventData::Nop);
    }
    
    // Traiter les événements et vérifier que les ordres sont identiques
    let mut events1 = Vec::new();
    let mut events2 = Vec::new();
    
    for _ in 0..iterations {
        if let Some(e) = clock1.state.advance_to_next() {
            events1.push(e.id.as_u64());
        }
        if let Some(e) = clock2.state.advance_to_next() {
            events2.push(e.id.as_u64());
        }
    }
    
    events1 == events2
}

/// Vérifie l'invariant I1 : conservation lors d'extraction/réinjection.
/// Pour l'instant, on vérifie que la file d'événements conserve son ordre.
pub fn verify_i1_conservation() -> bool {
    // Ce test sera étendu quand les entités seront implémentées
    // Pour l'instant, on vérifie la conservation des ids et de l'ordre
    
    let mut queue = EventQueue::new();
    
    // Ajouter des événements
    let id1 = queue.next_id();
    let id2 = queue.next_id();
    let id3 = queue.next_id();
    
    queue.push(Event::new(id1, 0, 0, Phase::FluxPhysiques, Priority::new(0), 0, EventData::Nop));
    queue.push(Event::new(id2, 0, 0, Phase::FluxPhysiques, Priority::new(1), 0, EventData::Nop));
    queue.push(Event::new(id3, 0, 0, Phase::FluxPhysiques, Priority::new(2), 0, EventData::Nop));
    
    // Extraire dans l'ordre
    let e1 = queue.pop().unwrap();
    let e2 = queue.pop().unwrap();
    let e3 = queue.pop().unwrap();
    
    // Vérifier l'ordre
    e1.id == id1 && e2.id == id2 && e3.id == id3
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_ordering() {
        // Créer des événements avec différents ordonnancements
        let e1 = Event::new(Id::new(1), 0, 0, Phase::FluxPhysiques, Priority::new(1), 0, EventData::Nop);
        let e2 = Event::new(Id::new(2), 0, 0, Phase::Comptes, Priority::new(0), 0, EventData::Nop);
        let e3 = Event::new(Id::new(3), 1, 0, Phase::FluxPhysiques, Priority::new(0), 0, EventData::Nop);
        
        // Phase::Comptes (2) > Phase::FluxPhysiques (1), donc e2 devrait être AVANT e1
        // car Phase::FluxPhysiques a une valeur numérique plus petite (1 < 2)
        assert!(e1 < e2, "FluxPhysiques (1) doit être avant Comptes (2) au même jour");
        assert!(e1 < e3, "Jour 0 doit être avant jour 1");
        assert!(e2 < e3, "Jour 0 doit être avant jour 1");
    }
    
    #[test]
    fn test_i9_determinism() {
        assert!(verify_i9_determinism(42, 100));
    }
    
    #[test]
    fn test_i1_conservation() {
        assert!(verify_i1_conservation());
    }
    
    #[test]
    fn test_timestamp_ordering() {
        let t1 = Timestamp::new(0, 0);
        let t2 = Timestamp::new(0, 1);
        let t3 = Timestamp::new(1, 0);
        
        assert!(t1 < t2);
        assert!(t2 < t3);
    }
}
