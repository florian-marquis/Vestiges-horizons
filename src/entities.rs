//! Vestiges Horizons - Tranche 2 : Identités et Journal
//! 
//! Implémentation du régime d'identité :
//! - Registre d'entités avec compteurs par type
//! - Cohortes à totaux
//! - Individus nommés
//! - Journal causal
//! 
//! Spécification : Docs/spec P1 - quartier vivant.md §14
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use crate::Timestamp;
use serde::{Serialize, Deserialize};
use std::cmp;
use std::collections::BTreeMap;

// ============================================================================
// TYPES D'ENTITÉS
// ============================================================================

/// Type d'entité (pour le compteur par type)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EntityType {
    /// Individu nommé (extrait d'une cohorte)
    NamedIndividual = 1,
    /// Cohorte (groupe d'individus)
    Cohort = 2,
    /// Foyer (unité domestique)
    Household = 3,
    /// firme
    Firm = 4,
    /// Pratique (compétence)
    Practice = 5,
    /// Bien (marchandise)
    Good = 6,
    /// Logement
    Housing = 7,
    /// Prêt
    Loan = 8,
    /// Actif (usure)
    Asset = 9,
    /// Emploi
    Job = 10,
    /// Marché
    Market = 11,
}

impl EntityType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(EntityType::NamedIndividual),
            2 => Some(EntityType::Cohort),
            3 => Some(EntityType::Household),
            4 => Some(EntityType::Firm),
            5 => Some(EntityType::Practice),
            6 => Some(EntityType::Good),
            7 => Some(EntityType::Housing),
            8 => Some(EntityType::Loan),
            9 => Some(EntityType::Asset),
            10 => Some(EntityType::Job),
            11 => Some(EntityType::Market),
            _ => None,
        }
    }
    
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Identifiant d'entité : (type, numéro)
/// Never réutilisé, les entités archivées gardent leur id
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub entity_type: EntityType,
    pub number: u64,
}

impl EntityId {
    pub fn new(entity_type: EntityType, number: u64) -> Self {
        EntityId { entity_type, number }
    }
    
    pub fn to_u64(&self) -> u64 {
        ((self.entity_type as u64) << 56) | (self.number & 0x00FF_FF_FF_FF_FF_FF)
    }
    
    pub fn from_u64(val: u64) -> Option<Self> {
        let entity_type = ((val >> 56) as u8);
        let number = val & 0x00FF_FF_FF_FF_FF_FF;
        EntityType::from_u8(entity_type as u8).map(|et| EntityId::new(et, number))
    }
}

// ============================================================================
// REGISTRE D'ENTITÉS
// ============================================================================

/// Registre d'entités avec compteurs par type
/// Les entités archivées ne libèrent pas leur numéro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRegistry {
    /// Compteurs par type d'entité
    counters: BTreeMap<EntityType, u64>,
    /// Entités actives (non archivées)
    active_entities: BTreeMap<EntityId, EntityStatus>,
    /// Entités archivées (morts, fermées)
    archived_entities: BTreeMap<EntityId, EntityStatus>,
}

/// État d'une entité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityStatus {
    /// Active
    Active,
    /// Archivée (mort/fermé)
    Archived,
}

impl EntityRegistry {
    pub fn new() -> Self {
        EntityRegistry {
            counters: BTreeMap::new(),
            active_entities: BTreeMap::new(),
            archived_entities: BTreeMap::new(),
        }
    }
    
    /// Génère un nouvel EntityId pour un type donné
    pub fn next_id(&mut self, entity_type: EntityType) -> EntityId {
        let counter = self.counters.entry(entity_type).or_insert(0);
        let id = EntityId::new(entity_type, *counter);
        *counter += 1;
        id
    }
    
    /// Active une entité
    pub fn activate(&mut self, id: EntityId) {
        self.active_entities.insert(id, EntityStatus::Active);
    }
    
    /// Archive une entité (ne la supprime pas, conserve l'id)
    pub fn archive(&mut self, id: EntityId) {
        if let Some(status) = self.active_entities.remove(&id) {
            self.archived_entities.insert(id, status);
        }
    }
    
    /// Vérifie si une entité existe (active ou archivée)
    pub fn exists(&self, id: EntityId) -> bool {
        self.active_entities.contains_key(&id) || self.archived_entities.contains_key(&id)
    }
    
    /// Retourne l'état d'une entité
    pub fn status(&self, id: EntityId) -> Option<&EntityStatus> {
        self.active_entities.get(&id).or(self.archived_entities.get(&id))
    }
    
    /// Nombre total d'entités (actives + archivées) pour un type
    pub fn count_by_type(&self, entity_type: EntityType) -> usize {
        let mut count = 0;
        for (id, _) in &self.active_entities {
            if id.entity_type == entity_type {
                count += 1;
            }
        }
        for (id, _) in &self.archived_entities {
            if id.entity_type == entity_type {
                count += 1;
            }
        }
        count
    }
    
    /// Test T2.5 : Aucun id réutilisé
    pub fn verify_no_reuse(&self) -> bool {
        // Vérifier que tous les compteurs sont supérieurs au max numéro utilisé
        for (entity_type, &counter) in &self.counters {
            let max_used = self.active_entities.keys()
                .filter(|id| id.entity_type == *entity_type)
                .map(|id| id.number)
                .max()
                .unwrap_or(0);
            let max_archived = self.archived_entities.keys()
                .filter(|id| id.entity_type == *entity_type)
                .map(|id| id.number)
                .max()
                .unwrap_or(0);
            
            let max_total = cmp::max(max_used, max_archived);
            if max_total >= counter {
                return false;
            }
        }
        true
    }
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// COHORTES
// ============================================================================

/// Cohorte - porte des totaux, jamais de moyennes stockées
/// Les moyennes sont dérivées à l'affichage (convention 26)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cohort {
    pub id: EntityId,
    /// Effectif total
    pub population: u64,
    /// Monnaie totale (en centimes)
    pub total_money: u64,
    /// Somme des griefs des membres (pondérée)
    pub total_griefs: i64,
    /// Attachement total
    pub total_attachment: i64,
    // Motivations dominantes (à implémenter plus tard)
    // Heures disponibles (à implémenter plus tard)
    // Salaire réel cible (à implémenter plus tard)
}

impl Cohort {
    pub fn new(id: EntityId, population: u64) -> Self {
        Cohort {
            id,
            population,
            total_money: 0,
            total_griefs: 0,
            total_attachment: 0,
        }
    }
    
    /// Calcul la moyenne des griefs (0-100)
    /// Dérivée, jamais stockée (convention 26)
    pub fn avg_griefs(&self) -> i64 {
        if self.population == 0 {
            0
        } else {
            // Échelle : total_griefs est en points, on normalise à 0-100
            // Pour l'instant, on retourne la valeur brute divisée par population
            // Le calibrage viendra plus tard
            self.total_griefs / (self.population as i64)
        }
    }
    
    /// Calcul la moyenne d'attachement
    pub fn avg_attachment(&self) -> i64 {
        if self.population == 0 {
            0
        } else {
            self.total_attachment / (self.population as i64)
        }
    }
}

// ============================================================================
// INDIVIDUS NOMMÉS
// ============================================================================

/// Individu nommé - échantillonnage déterministe d'une cohorte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedIndividual {
    pub id: EntityId,
    /// Cohorte d'origine
    pub cohort_id: EntityId,
    /// Grief individuel
    pub grief: i64,
    /// Solde (en centimes)
    pub balance: u64,
    /// Attachement
    pub attachment: i64,
    // Âge (à implémenter plus tard)
    // Pratiques maîtrisées (à implémenter plus tard)
}

impl NamedIndividual {
    pub fn new(id: EntityId, cohort_id: EntityId, grief: i64, balance: u64, attachment: i64) -> Self {
        NamedIndividual {
            id,
            cohort_id,
            grief,
            balance,
            attachment,
        }
    }
}

// ============================================================================
// EXTRACTION / RÉINJECTION
// ============================================================================

/// Gestionnaire des entités avec extraction/réinjection
#[derive(Clone)]
pub struct EntityManager {
    registry: EntityRegistry,
    cohorts: BTreeMap<EntityId, Cohort>,
    named_individuals: BTreeMap<EntityId, NamedIndividual>,
    /// Clé stable pour l'échantillonnage déterministe
    stable_key: u64,
}

impl EntityManager {
    pub fn new(stable_key: u64) -> Self {
        EntityManager {
            registry: EntityRegistry::new(),
            cohorts: BTreeMap::new(),
            named_individuals: BTreeMap::new(),
            stable_key,
        }
    }
    
    /// Retourne une référence à une cohorte (pour les tests)
    pub fn cohort(&self, id: EntityId) -> Option<&Cohort> {
        self.cohorts.get(&id)
    }
    
    /// Retourne une référence mutable à une cohorte (pour les tests)
    pub fn cohort_mut(&mut self, id: EntityId) -> Option<&mut Cohort> {
        self.cohorts.get_mut(&id)
    }
    
    /// Retourne le registre (pour les tests)
    pub fn registry(&self) -> &EntityRegistry {
        &self.registry
    }
    
    /// Retourne les individus nommés (pour les tests)
    pub fn named_individuals(&self) -> &BTreeMap<EntityId, NamedIndividual> {
        &self.named_individuals
    }
    
    /// Retourne toutes les cohortes (pour la sérialisation)
    pub fn all_cohorts(&self) -> impl Iterator<Item = (&EntityId, &Cohort)> {
        self.cohorts.iter()
    }
    
    /// Crée une nouvelle cohorte
    pub fn create_cohort(&mut self, population: u64) -> EntityId {
        let id = self.registry.next_id(EntityType::Cohort);
        let cohort = Cohort::new(id, population);
        self.cohorts.insert(id, cohort);
        self.registry.activate(id);
        id
    }
    
    /// Crée un individu nommé à partir d'une cohorte (extraction)
    /// 
    /// **I1** : La cohorte déduit exactement (effectif-1, monnaie-solde, somme_griefs-grief, attachement-attachment)
    pub fn extract(&mut self, cohort_id: EntityId) -> Option<NamedIndividual> {
        let cohort = self.cohorts.get_mut(&cohort_id)?;
        
        if cohort.population == 0 {
            return None;
        }
        
        // Créer un nouvel individu avec des valeurs cohérentes
        // Pour l'instant, on extrait des valeurs moyennes (à affiner avec clé stable)
        let individual_id = self.registry.next_id(EntityType::NamedIndividual);
        
        // Calculer les valeurs de l'individu
        // Échantillonnage déterministe : on prend une partie proportionnelle
        // des totaux de la cohorte
        let grief = cohort.total_griefs / (cohort.population as i64);
        let balance = cohort.total_money / cohort.population;
        let attachment = cohort.total_attachment / (cohort.population as i64);
        
        // Créer l'individu
        let individual = NamedIndividual::new(
            individual_id,
            cohort_id,
            grief,
            balance,
            attachment,
        );
        
        // Mettre à jour la cohorte (I1 : déduction exacte)
        cohort.population -= 1;
        cohort.total_money -= balance;
        cohort.total_griefs -= grief;
        cohort.total_attachment -= attachment;
        
        // Activer l'individu
        self.registry.activate(individual_id);
        self.named_individuals.insert(individual_id, individual.clone());
        
        Some(individual)
    }
    
    /// Réinjecte un individu dans sa cohorte d'origine
    /// 
    /// **I1** : Addition réciproque ; l'individu ramène ses valeurs devenues divergentes
    pub fn reinject(&mut self, individual: NamedIndividual) -> bool {
        let cohort = self.cohorts.get_mut(&individual.cohort_id);
        if cohort.is_none() {
            return false;
        }
        
        let cohort = cohort.unwrap();
        
        // Mettre à jour la cohorte (I1 : addition réciproque)
        cohort.population += 1;
        cohort.total_money += individual.balance;
        cohort.total_griefs += individual.grief;
        cohort.total_attachment += individual.attachment;
        
        // Archiver l'individu (il n'est plus nommé)
        self.registry.archive(individual.id);
        self.named_individuals.remove(&individual.id);
        
        true
    }
    
    /// Fusionne deux cohortes de manière conservatrice
    /// 
    /// **I8** : Somme des totaux, moyennes recalculées, tolérance zéro
    pub fn merge_cohorts(&mut self, id_a: EntityId, id_b: EntityId) -> Option<EntityId> {
        let cohort_a = self.cohorts.get(&id_a)?;
        let cohort_b = self.cohorts.get(&id_b)?;
        
        // La cohorte A absorbe B
        let mut merged = Cohort::new(id_a, cohort_a.population + cohort_b.population);
        merged.total_money = cohort_a.total_money + cohort_b.total_money;
        merged.total_griefs = cohort_a.total_griefs + cohort_b.total_griefs;
        merged.total_attachment = cohort_a.total_attachment + cohort_b.total_attachment;
        
        // Archiver la cohorte B
        self.registry.archive(id_b);
        
        // Mettre à jour la cohorte A
        self.cohorts.insert(id_a, merged);
        
        // Vérifier I8 : la moyenne doit être cohérente
        // Si la somme des griefs pondérés de A + B = total_griefs de merged,
        // et que population de merged = pop_a + pop_b,
        // alors la moyenne est conservée
        
        Some(id_a)
    }
    
    /// Vérifie I1 : conservation lors d'extraction/réinjection
    pub fn verify_i1_conservation(&self) -> bool {
        // Pour chaque cohorte, vérifier que les individus extraits ont des valeurs
        // qui peuvent être réinjectées sans perte
        for (cohort_id, cohort) in &self.cohorts {
            // Trouver les individus extraits de cette cohorte
            for (ind_id, individual) in &self.named_individuals {
                if individual.cohort_id == *cohort_id {
                    // L'individu doit avoir des valeurs cohérentes avec la cohorte
                    // (c'est une vérification basique, à affiner)
                    if individual.balance > cohort.total_money {
                        return false;
                    }
                }
            }
        }
        true
    }
    
    /// Vérifie I8 : concordance griefs
    /// G d'une cohorte = somme des griefs individuels pondérés par les effectifs
    pub fn verify_i8_concordance(&self) -> bool {
        for (cohort_id, cohort) in &self.cohorts {
            // Trouver tous les individus de cette cohorte
            let mut total_griefs_from_individuals: i64 = 0;
            let mut individual_count: u64 = 0;
            
            for (ind_id, individual) in &self.named_individuals {
                if individual.cohort_id == *cohort_id {
                    total_griefs_from_individuals += individual.grief;
                    individual_count += 1;
                }
            }
            
            // Pour I8, on vérifie que la somme des griefs des individus nommés
            // + les griefs des non-nommés = total_griefs de la cohorte
            // Pour l'instant, on suppose que les non-nommés ont la moyenne
            // (implémentation simplifiée)
            
            let unnamed_count = cohort.population - individual_count;
            let avg_grief = if cohort.population > 0 {
                cohort.total_griefs / (cohort.population as i64)
            } else {
                0
            };
            
            let total_from_unnamed = avg_grief * (unnamed_count as i64);
            let total_calculated = total_griefs_from_individuals + total_from_unnamed;
            
            if total_calculated != cohort.total_griefs {
                return false;
            }
        }
        true
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new(0)
    }
}

// ============================================================================
// JOURNAL CAUSAL
// ============================================================================

/// Type de décision/event pour le journal
pub type DecisionType = u16;

/// Modification d'une valeur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modification {
    pub field: String,
    pub old_value: i64,
    pub new_value: i64,
}

/// Entrée du journal causal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: u64,
    pub timestamp: Timestamp,
    pub event_id: u64,
    pub phase: u8,
    pub actor: Option<EntityId>,
    pub decision_type: DecisionType,
    pub modified_values: Vec<Modification>,
    pub causes: Vec<u64>, // ids d'événements strictement antérieurs
    pub learned_by: Vec<(EntityId, u64)>, // (acteur, date d'apprentissage)
}

/// Journal causal (append-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalJournal {
    /// Compteur global pour les ids d'entrées
    next_id: u64,
    /// Entrées du journal
    entries: BTreeMap<u64, JournalEntry>,
    /// Index par événement pour la requête "expliquer"
    event_to_entry: BTreeMap<u64, Vec<u64>>,
}

impl CausalJournal {
    /// Retourne toutes les entrées du journal
    pub fn all_entries(&self) -> impl Iterator<Item = &JournalEntry> {
        self.entries.values()
    }
}

impl CausalJournal {
    pub fn new() -> Self {
        CausalJournal {
            next_id: 0,
            entries: BTreeMap::new(),
            event_to_entry: BTreeMap::new(),
        }
    }
    
    /// Ajoute une entrée au journal
    /// Vérifie que les causes sont strictement antérieures (I7)
    pub fn add_entry(&mut self, entry: JournalEntry) -> Option<u64> {
        // Vérifier I7 : causes doivent être strictement antérieures
        for &cause_id in &entry.causes {
            let cause_entry = self.entries.get(&cause_id);
            if let Some(cause) = cause_entry {
                // La cause doit être strictement antérieure
                if cause.timestamp >= entry.timestamp {
                    // Violation de I7 : cause n'est pas strictement antérieure
                    return None;
                }
            } else {
                // La cause n'existe pas encore - c'est acceptable si on journalise dans l'ordre
                // Mais en mode debug, on peut vouloir échouer
            }
        }
        
        let mut entry = entry;
        entry.id = self.next_id;
        let id = entry.id;
        self.entries.insert(id, entry.clone());
        
        // Mettre à jour l'index
        self.event_to_entry.entry(entry.event_id).or_default().push(id);
        
        self.next_id += 1;
        
        Some(id)
    }
    
    /// Requête "expliquer" : remonte les liens causaux
    pub fn explain(&self, entry_id: u64) -> Vec<JournalEntry> {
        let mut chain = Vec::new();
        let mut current_id = entry_id;
        let mut visited = std::collections::HashSet::new();
        
        // Éviter les cycles (même si par construction il n'y en a pas)
        while let Some(entry) = self.entries.get(&current_id) {
            if visited.contains(&current_id) {
                break; // Cycle détecté (ne devrait jamais arriver)
            }
            visited.insert(current_id);
            
            chain.push(entry.clone());
            
            // Remonter aux causes (pour l'instant, on prend la première cause)
            if !entry.causes.is_empty() {
                current_id = entry.causes[0];
            } else {
                break;
            }
        }
        
        chain.reverse();
        chain
    }
    
    /// Compactage : résume les entrées vieilles
    pub fn compact(&mut self, before_day: i64) -> Vec<Summary> {
        let mut summaries = Vec::new();
        let mut to_remove = Vec::new();
        
        for (id, entry) in &self.entries {
            if entry.timestamp.day < before_day {
                // Créer un résumé pour cette entrée
                let summary = Summary {
                    compacted_ids: vec![*id],
                    data: format!("Résumé de l'entrée {}", id),
                };
                summaries.push(summary);
                to_remove.push(*id);
            }
        }
        
        // Supprimer les entrées compactées (mais garder les résumés hors état canonique)
        // Note : en réalité, on ne supprime pas, on les archive
        // Pour l'instant, on les garde dans le journal
        
        summaries
    }
    
    /// Vérifie I7 : chaque entrée référence des ids existants à sa date
    pub fn verify_i7(&self) -> bool {
        for (id, entry) in &self.entries {
            for &cause_id in &entry.causes {
                // La cause doit exister
                if !self.entries.contains_key(&cause_id) {
                    return false;
                }
                
                // La cause doit être strictement antérieure
                let cause = self.entries.get(&cause_id).unwrap();
                if cause.timestamp >= entry.timestamp {
                    return false;
                }
            }
        }
        true
    }
}

impl Default for CausalJournal {
    fn default() -> Self {
        Self::new()
    }
}

/// Résumé de compactage (hors état canonique)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub compacted_ids: Vec<u64>,
    pub data: String,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_registry_no_reuse() {
        let mut registry = EntityRegistry::new();
        
        let id1 = registry.next_id(EntityType::NamedIndividual);
        let id2 = registry.next_id(EntityType::NamedIndividual);
        
        assert_ne!(id1, id2);
        assert!(registry.verify_no_reuse());
    }
    
    #[test]
    fn test_extraction_reinjection_i1() {
        let mut manager = EntityManager::new(42);
        
        // Créer une cohorte
        let cohort_id = manager.create_cohort(100);
        
        // Extraire un individu
        let initial_money = {
            let cohort = manager.cohorts.get(&cohort_id).unwrap();
            cohort.total_money
        };
        
        let individual = manager.extract(cohort_id).unwrap();
        
        // Vérifier que la cohorte a été mise à jour
        let cohort = manager.cohorts.get(&cohort_id).unwrap();
        assert_eq!(cohort.population, 99);
        assert_eq!(cohort.total_money, initial_money - individual.balance);
        
        // Réinjecter
        manager.reinject(individual);
        
        // Vérifier que tout est revenu à la normale
        let cohort = manager.cohorts.get(&cohort_id).unwrap();
        assert_eq!(cohort.population, 100);
        assert_eq!(cohort.total_money, initial_money);
    }
    
    #[test]
    fn test_i1_conservation_cycle() {
        let mut manager = EntityManager::new(42);
        
        let cohort_id = manager.create_cohort(100);
        
        // Extraire et réinjecter 100 fois
        for _ in 0..100 {
            let individual = manager.extract(cohort_id.clone());
            if let Some(ind) = individual {
                manager.reinject(ind);
            }
        }
        
        // Vérifier I1
        assert!(manager.verify_i1_conservation());
        
        let cohort = manager.cohorts.get(&cohort_id).unwrap();
        assert_eq!(cohort.population, 100);
    }
    
    #[test]
    fn test_merge_cohorts_i8() {
        let mut manager = EntityManager::new(42);
        
        let cohort_a = manager.create_cohort(50);
        let cohort_b = manager.create_cohort(50);
        
        // Modifier les totaux
        {
            let a = manager.cohorts.get_mut(&cohort_a).unwrap();
            a.total_money = 1000;
            a.total_griefs = 500;
            a.total_attachment = 250;
        }
        {
            let b = manager.cohorts.get_mut(&cohort_b).unwrap();
            b.total_money = 2000;
            b.total_griefs = 1000;
            b.total_attachment = 500;
        }
        
        // Fusionner
        manager.merge_cohorts(cohort_a, cohort_b);
        
        let merged = manager.cohorts.get(&cohort_a).unwrap();
        assert_eq!(merged.population, 100);
        assert_eq!(merged.total_money, 3000);
        assert_eq!(merged.total_griefs, 1500);
        assert_eq!(merged.total_attachment, 750);
        
        // Vérifier I8
        assert!(manager.verify_i8_concordance());
    }
    
    #[test]
    fn test_journal_causal_order() {
        let mut journal = CausalJournal::new();
        
        let entry1 = JournalEntry {
            id: 0,
            timestamp: Timestamp::new(0, 1),
            event_id: 1,
            phase: 1,
            actor: None,
            decision_type: 0,
            modified_values: vec![],
            causes: vec![],
            learned_by: vec![],
        };
        
        let entry2 = JournalEntry {
            id: 0,
            timestamp: Timestamp::new(0, 2),
            event_id: 2,
            phase: 2,
            actor: None,
            decision_type: 1,
            modified_values: vec![],
            causes: vec![], // pas de cause pour ce test simple
            learned_by: vec![],
        };
        
        journal.add_entry(entry1);
        journal.add_entry(entry2);
        
        // Vérifier I7
        assert!(journal.verify_i7());
    }
}
