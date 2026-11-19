//! Vestiges Horizons - Tranche 3 : Budget de Temps
//! 
//! Implémentation du budget de temps quotidien (32 demi-heures)
//! selon Docs/spec P1 - quartier vivant.md §15.1
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::entities::EntityId;
use crate::entities::NamedIndividual;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Budget total d'éveil en demi-heures (16 heures = 32 demi-heures)
pub const BUDGET_EVEIL: u32 = 32;

/// Nombre de demi-heures dans une journée complète
pub const DEMI_HEURES_PAR_JOUR: u32 = 32;

// ============================================================================
// TYPES D'USAGE DU TEMPS
// ============================================================================

/// Type d'usage du temps (pour le budget)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TimeUsage {
    /// Sommeil (nuit)
    Sommeil = 0,
    /// Repas
    Repas = 1,
    /// Trajet
    Trajet = 2,
    /// Corvées (ménage, tâches domestiques)
    Corvee = 3,
    /// Travail
    Travail = 4,
    /// Soin (maladie, blessure)
    Soin = 5,
    /// Formation (acquisition de compétences)
    Formation = 6,
    /// Recherche (d'emploi, d'information)
    Recherche = 7,
    /// Loisir
    Loisir = 8,
    /// Urgence (interruption critique)
    Urgence = 9,
}

impl TimeUsage {
    /// Retourne le coût en demi-heures pour un usage standard
    pub fn default_duration(&self) -> u32 {
        match self {
            TimeUsage::Sommeil => 16,   // 8 heures de sommeil = 16 demi-heures
            TimeUsage::Repas => 2,      // 1 heure par repas = 2 demi-heures
            TimeUsage::Trajet => 1,      // 30 minutes de trajet
            TimeUsage::Corvee => 2,     // 1 heure de corvées
            TimeUsage::Travail => 0,     // Variable, déterminé par l'emploi
            TimeUsage::Soin => 2,       // 1 heure par soin
            TimeUsage::Formation => 2,  // 1 heure de formation
            TimeUsage::Recherche => 2,   // 1 heure de recherche
            TimeUsage::Loisir => 1,      // 30 minutes de loisir
            TimeUsage::Urgence => 0,     // Variable, consomme ce qu'il faut
        }
    }
    
    /// Retourne si cet usage est un besoin fondamental
    pub fn is_besoin_fondamental(&self) -> bool {
        match self {
            TimeUsage::Sommeil | TimeUsage::Repas | TimeUsage::Trajet | TimeUsage::Corvee => true,
            _ => false,
        }
    }
}

// ============================================================================
// BUDGET DE TEMPS INDIVIDUEL
// ============================================================================

/// Budget de temps pour un individu nommé
/// 
/// Représente l'allocation des 32 demi-heures d'éveil pour un jour.
/// **Invariant I4** : La somme des usages doit être <= BUDGET_EVEIL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBudget {
    /// Identifiant de l'individu
    pub individual_id: EntityId,
    /// Allocation par type d'usage (index = TimeUsage as usize)
    pub allocations: [u32; 10], // 10 types d'usage
    /// Jour concerné
    pub day: i64,
}

impl TimeBudget {
    pub fn new(individual_id: EntityId, day: i64) -> Self {
        TimeBudget {
            individual_id,
            allocations: [0; 10],
            day,
        }
    }
    
    /// Alloue du temps à un usage
    pub fn allocate(&mut self, usage: TimeUsage, duration: u32) {
        self.allocations[usage as usize] += duration;
    }
    
    /// Désalloue du temps d'un usage
    pub fn deallocate(&mut self, usage: TimeUsage, duration: u32) {
        if self.allocations[usage as usize] >= duration {
            self.allocations[usage as usize] -= duration;
        }
    }
    
    /// Retourne le temps alloué à un usage
    pub fn get_allocation(&self, usage: TimeUsage) -> u32 {
        self.allocations[usage as usize]
    }
    
    /// Vérifie l'invariant I4 : somme des usages d'éveil <= 32 demi-heures
    /// 
    /// **I4** : La somme des usages de temps d'éveil (tout sauf Sommeil) 
    /// doit être <= BUDGET_EVEIL (32 demi-heures = 16 heures)
    /// Le sommeil est compté séparément.
    pub fn verify_i4(&self) -> bool {
        let eveil_used: u32 = self.allocations
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != TimeUsage::Sommeil as usize)
            .map(|(_, &val)| val)
            .sum();
        
        eveil_used <= BUDGET_EVEIL
    }
    
    /// Temps restant disponible pour l'éveil
    pub fn remaining_eveil(&self) -> u32 {
        let eveil_used: u32 = self.allocations
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != TimeUsage::Sommeil as usize)
            .map(|(_, &val)| val)
            .sum();
        
        BUDGET_EVEIL.saturating_sub(eveil_used)
    }
    
    /// Réalloue tout le temps libre vers un usage spécifique
    /// Note: Le temps libre est calculé comme le temps restant après les autres allocations
    pub fn allocate_remaining(&mut self, usage: TimeUsage) -> u32 {
        let remaining = self.remaining_eveil();
        self.allocate(usage, remaining);
        remaining
    }
    
    /// Réinitialise le budget
    pub fn reset(&mut self) {
        self.allocations = [0; 10];
    }
}

impl Default for TimeBudget {
    fn default() -> Self {
        TimeBudget {
            individual_id: EntityId::new(crate::entities::EntityType::NamedIndividual, 0),
            allocations: [0; 10],
            day: 0,
        }
    }
}

// ============================================================================
// GESTIONNAIRE DE BUDGET DE TEMPS
// ============================================================================

/// Gestionnaire des budgets de temps pour tous les individus
pub struct TimeManager {
    /// Budgets par individu et par jour
    budgets: BTreeMap<(EntityId, i64), TimeBudget>,
    /// Doctrine par individu (poids pour l'arbitrage)
    doctrines: BTreeMap<EntityId, Doctrine>,
}

/// Doctrine personnelle : poids pour l'arbitrage du temps
/// 
/// Ces poids déterminent l'ordre de résolution des usages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doctrine {
    /// Poids pour les besoins fondamentaux (repas, trajet, corvées)
    pub besoins: u32,
    /// Poids pour le travail
    pub travail: u32,
    /// Poids pour la formation
    pub formation: u32,
    /// Poids pour le loisir
    pub loisir: u32,
}

impl Doctrine {
    pub fn new() -> Self {
        Doctrine {
            besoins: 100,
            travail: 80,
            formation: 50,
            loisir: 30,
        }
    }
    
    /// Doctrine par défaut pour un individu sans préférence
    pub fn default_individual() -> Self {
        Doctrine::new()
    }
    
    /// Doctrine pour un individu ambitieux (priorité travail/formation)
    pub fn ambitious() -> Self {
        Doctrine {
            besoins: 80,
            travail: 120,
            formation: 100,
            loisir: 20,
        }
    }
    
    /// Doctrine pour un individu économe
    pub fn economique() -> Self {
        Doctrine {
            besoins: 90,
            travail: 110,
            formation: 40,
            loisir: 10,
        }
    }
}

impl Default for Doctrine {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeManager {
    pub fn new() -> Self {
        TimeManager {
            budgets: BTreeMap::new(),
            doctrines: BTreeMap::new(),
        }
    }
    
    /// Crée ou retourne le budget pour un individu et un jour
    pub fn get_or_create_budget(&mut self, individual_id: EntityId, day: i64) -> &mut TimeBudget {
        self.budgets
            .entry((individual_id, day))
            .or_insert_with(|| TimeBudget::new(individual_id, day))
    }
    
    /// Définit la doctrine pour un individu
    pub fn set_doctrine(&mut self, individual_id: EntityId, doctrine: Doctrine) {
        self.doctrines.insert(individual_id, doctrine);
    }
    
    /// Résout l'arbitrage du temps pour un individu
    /// 
    /// **Algorithme** :
    /// 1. Allouer les besoins fondamentaux d'abord
    /// 2. Allouer le travail
    /// 3. Allouer la formation
    /// 4. Allouer le loisir
    /// 5. Le reste est du temps libre
    /// 
    /// L'ordre peut être modifié par la doctrine.
    pub fn resolve_daily_schedule(
        &mut self,
        individual: &NamedIndividual,
        day: i64,
        has_job: bool,
        job_hours: u32,
        _has_children: bool,
        children_corvee_hours: u32,
    ) -> &TimeBudget {
        let budget = self.get_or_create_budget(individual.id, day);
        budget.reset();
        
        // 1. Besoins fondamentaux (toujours prioritaires)
        // Repas : 2 demi-heures
        budget.allocate(TimeUsage::Repas, 2);
        
        // Trajet : 1 demi-heure
        budget.allocate(TimeUsage::Trajet, 1);
        
        // Corvées : 2 demi-heures (réduites si logement de qualité)
        let corvee_hours = 2 - children_corvee_hours.min(2);
        budget.allocate(TimeUsage::Corvee, corvee_hours);
        
        // 2. Travail (si emploi)
        if has_job {
            let work_hours = job_hours.min(budget.remaining_eveil());
            budget.allocate(TimeUsage::Travail, work_hours);
        }
        
        // 3. Formation (si temps restant)
        if budget.remaining_eveil() > 0 {
            // Pour l'instant, allocation fixe
            let formation_hours = 2.min(budget.remaining_eveil());
            budget.allocate(TimeUsage::Formation, formation_hours);
        }
        
        // 4. Loisir (si temps restant)
        if budget.remaining_eveil() > 0 {
            let loisir_hours = 1.min(budget.remaining_eveil());
            budget.allocate(TimeUsage::Loisir, loisir_hours);
        }
        
        // 5. Recherche (si temps restant) - reste du temps est alloué à la recherche/autres
        if budget.remaining_eveil() > 0 {
            budget.allocate(TimeUsage::Recherche, budget.remaining_eveil());
        }
        
        // Vérifier I4
        assert!(budget.verify_i4(), "I4 violé : budget d'éveil dépassé");
        
        budget
    }
    
    /// Vérifie I4 pour tous les individus et tous les jours
    pub fn verify_i4_all(&self) -> bool {
        for budget in self.budgets.values() {
            if !budget.verify_i4() {
                return false;
            }
        }
        true
    }
    
    /// Retourne le budget pour un individu et un jour (lecture seule)
    pub fn get_budget(&self, individual_id: EntityId, day: i64) -> Option<&TimeBudget> {
        self.budgets.get(&(individual_id, day))
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// JEÛNE ET PERTE DE TEMPS D'ÉVEIL
// ============================================================================

/// Gère les effets du jeûne sur le budget de temps
/// 
/// Selon Spec P1 §15.1 : "Jeûne : −1 demi-heure d'éveil par jour et demi sans nourriture"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingManager {
    /// Nombre de jours sans nourriture pour chaque individu
    fasting_days: BTreeMap<EntityId, u32>,
}

impl FastingManager {
    pub fn new() -> Self {
        FastingManager {
            fasting_days: BTreeMap::new(),
        }
    }
    
    /// Signale qu'un individu n'a pas mangé aujourd'hui
    pub fn record_fasting(&mut self, individual_id: EntityId) {
        *self.fasting_days.entry(individual_id).or_insert(0) += 1;
    }
    
    /// Réinitialise le compteur si l'individu a mangé
    pub fn reset_fasting(&mut self, individual_id: EntityId) {
        self.fasting_days.remove(&individual_id);
    }
    
    /// Retourne la perte de temps d'éveil due au jeûne
    /// 
    /// Formule : perte = (fasting_days / 2) demi-heures
    /// Exemple : 1 jour sans nourriture → 0.5 → 0 (arrondi)
    ///           2 jours sans nourriture → 1 demi-heure
    ///           4 jours sans nourriture → 2 demi-heures
    pub fn get_fasting_penalty(&self, individual_id: EntityId) -> u32 {
        let days = self.fasting_days.get(&individual_id).copied().unwrap_or(0);
        days / 2
    }
    
    /// Applique la pénalité de jeûne au budget
    /// 
    /// §15.1 : "Jeûne : −1 demi-heure d'éveil par jour et demi sans nourriture"
    /// La pénalité réduit les activités discrétionnaires (loisir, formation, recherche)
    /// et ajoute du sommeil (repos forcé)
    pub fn apply_fasting_penalty(&self, budget: &mut TimeBudget) {
        let penalty = self.get_fasting_penalty(budget.individual_id);
        if penalty > 0 {
            // Réduire le loisir d'abord
            let loisir = budget.get_allocation(TimeUsage::Loisir);
            if loisir >= penalty {
                budget.deallocate(TimeUsage::Loisir, penalty);
                // Réallouer une partie à Sommeil (repos forcé)
                budget.allocate(TimeUsage::Sommeil, penalty.min(16));
            } else {
                // Si pas assez de loisir, réduire la formation
                let formation = budget.get_allocation(TimeUsage::Formation);
                let to_remove = penalty.saturating_sub(loisir);
                if formation >= to_remove {
                    budget.deallocate(TimeUsage::Formation, to_remove);
                    budget.allocate(TimeUsage::Sommeil, to_remove.min(16));
                } else {
                    // Si pas assez de formation, réduire la recherche
                    let recherche = budget.get_allocation(TimeUsage::Recherche);
                    let to_remove2 = to_remove.saturating_sub(formation);
                    if recherche >= to_remove2 {
                        budget.deallocate(TimeUsage::Recherche, to_remove2);
                        budget.allocate(TimeUsage::Sommeil, to_remove2.min(16));
                    }
                }
            }
        }
    }
}

impl Default for FastingManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityId;
    
    #[test]
    fn test_budget_creation() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let budget = TimeBudget::new(individual_id, 0);
        
        assert_eq!(budget.day, 0);
        assert_eq!(budget.get_allocation(TimeUsage::Repas), 0);
    }
    
    #[test]
    fn test_allocate_and_verify_i4() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut budget = TimeBudget::new(individual_id, 0);
        
        // Allouer des usages
        budget.allocate(TimeUsage::Repas, 2);
        budget.allocate(TimeUsage::Trajet, 1);
        budget.allocate(TimeUsage::Corvee, 2);
        budget.allocate(TimeUsage::Travail, 8);
        budget.allocate(TimeUsage::Formation, 4);
        budget.allocate(TimeUsage::Loisir, 2);
        budget.allocate(TimeUsage::Recherche, 13);
        
        // Total = 2+1+2+8+4+2+13 = 32
        assert!(budget.verify_i4());
        assert_eq!(budget.remaining_eveil(), 0);
    }
    
    #[test]
    fn test_i4_violation() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut budget = TimeBudget::new(individual_id, 0);
        
        // Allouer trop de temps
        budget.allocate(TimeUsage::Travail, 33);
        
        // I4 doit être violé
        assert!(!budget.verify_i4());
    }
    
    #[test]
    fn test_time_manager_resolve() {
        let mut manager = TimeManager::new();
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let cohort_id = EntityId::new(crate::entities::EntityType::Cohort, 1);
        let individual = crate::entities::NamedIndividual::new(
            individual_id,
            cohort_id,
            0,
            0,
            0,
        );
        
        let budget = manager.resolve_daily_schedule(
            &individual,
            0,
            true,
            8,
            false,
            0,
        );
        
        // Vérifier que le total est correct
        assert!(budget.verify_i4());
        
        // Vérifier les allocations minimales
        assert_eq!(budget.get_allocation(TimeUsage::Repas), 2);
        assert_eq!(budget.get_allocation(TimeUsage::Trajet), 1);
        assert_eq!(budget.get_allocation(TimeUsage::Corvee), 2);
        assert_eq!(budget.get_allocation(TimeUsage::Travail), 8);
    }
    
    #[test]
    fn test_fasting_penalty() {
        let mut fasting_manager = FastingManager::new();
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        
        // 0 jour de jeûne
        assert_eq!(fasting_manager.get_fasting_penalty(individual_id), 0);
        
        // 1 jour de jeûne
        fasting_manager.record_fasting(individual_id);
        assert_eq!(fasting_manager.get_fasting_penalty(individual_id), 0);
        
        // 2 jours de jeûne
        fasting_manager.record_fasting(individual_id);
        assert_eq!(fasting_manager.get_fasting_penalty(individual_id), 1);
        
        // 4 jours de jeûne
        fasting_manager.record_fasting(individual_id);
        fasting_manager.record_fasting(individual_id);
        assert_eq!(fasting_manager.get_fasting_penalty(individual_id), 2);
        
        // Réinitialiser
        fasting_manager.reset_fasting(individual_id);
        assert_eq!(fasting_manager.get_fasting_penalty(individual_id), 0);
    }
}
