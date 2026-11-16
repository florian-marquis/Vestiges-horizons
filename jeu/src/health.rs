//! Vestiges Horizons - Tranche 3 : Santé
//! 
//! Implémentation du système de santé selon Docs/spec P1 - quartier vivant.md §15.4
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, VecDeque};

use crate::entities::EntityId;
use crate::time::{TimeBudget, TimeUsage, FastingManager};

// ============================================================================
// CONSTANTES DE SANTÉ
// ============================================================================

/// Santé maximale (1000 points)
pub const MAX_HEALTH: u32 = 1000;

/// Santé minimale avant décès
pub const MIN_HEALTH: u32 = 0;

/// Seuil de santé pour maladie grave
pub const SEVERE_ILLNESS_THRESHOLD: u32 = 200;

/// Seuil de santé pour incapacité de travail
pub const WORK_INCAPACITY_THRESHOLD: u32 = 300;

/// Coût en demi-heures pour un soin de base
pub const BASE_CARE_COST: u32 = 2; // 1 heure

/// Coût en demi-heures pour un soin intensif
pub const INTENSIVE_CARE_COST: u32 = 4; // 2 heures

/// Coût monétaire pour un soin de base (en centimes)
pub const BASE_CARE_MONEY_COST: u64 = 500; // 5€

/// Coût monétaire pour un soin intensif (en centimes)
pub const INTENSIVE_CARE_MONEY_COST: u64 = 2000; // 20€

// ============================================================================
// TYPES DE MALADIES/BLESSURES
// ============================================================================

/// Type de maladie ou blessure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HealthIssueType {
    /// Maladie infectieuse (grippe, etc.)
    Infectious = 0,
    /// Blessure physique (chute, accident)
    Injury = 1,
    /// Maladie chronique (diabète, etc.)
    Chronic = 2,
    /// Stress / Épuisement
    Stress = 3,
    /// Malnutrition
    Malnutrition = 4,
}

/// Gravité de la maladie/blessure
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Severity {
    /// Léger (santé > 800)
    Mild = 1,
    /// Modéré (500 < santé <= 800)
    Moderate = 2,
    /// Sévère (200 < santé <= 500)
    Severe = 3,
    /// Critique (santé <= 200)
    Critical = 4,
}

impl Severity {
    pub fn from_health(health: u32) -> Self {
        match health {
            801..=MAX_HEALTH => Severity::Mild,
            501..=800 => Severity::Moderate,
            201..=500 => Severity::Severe,
            _ => Severity::Critical,
        }
    }
    
    /// Retourne le modificateur de temps de récupération
    pub fn recovery_time_modifier(&self) -> f32 {
        match self {
            Severity::Mild => 0.5,
            Severity::Moderate => 1.0,
            Severity::Severe => 2.0,
            Severity::Critical => 4.0,
        }
    }
}

// ============================================================================
// MALADIE / BLESSURE
// ============================================================================

/// Maladie ou blessure avec clé stable
/// 
/// §15.4 : "Maladie, blessure à clé stable"
/// La clé stable permet de suivre une maladie de manière déterministe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Clé stable (unique par maladie, déterministe)
    pub stable_key: u64,
    /// Type de problème de santé
    pub issue_type: HealthIssueType,
    /// Jour d'apparition
    pub onset_day: i64,
    /// Gravité
    pub severity: Severity,
    ///Santé initiale au moment de l'apparition
    pub initial_health: u32,
    /// Perte de santé par jour (si non soigné)
    pub daily_health_loss: u32,
    /// Temps de récupération en jours (si soigné)
    pub recovery_days: u32,
    /// Coût en temps pour soigner
    pub care_time_cost: u32,
    /// Coût en argent pour soigner
    pub care_money_cost: u64,
    /// Soin terminé ?
    pub is_treated: bool,
    /// Jour de fin du traitement
    pub treatment_end_day: Option<i64>,
}

impl HealthIssue {
    pub fn new(
        stable_key: u64,
        issue_type: HealthIssueType,
        onset_day: i64,
        severity: Severity,
        initial_health: u32,
    ) -> Self {
        let (daily_health_loss, recovery_days, care_time_cost, care_money_cost) = 
            Self::get_issue_params(issue_type, severity);
        
        HealthIssue {
            stable_key,
            issue_type,
            onset_day,
            severity,
            initial_health,
            daily_health_loss,
            recovery_days,
            care_time_cost,
            care_money_cost,
            is_treated: false,
            treatment_end_day: None,
        }
    }
    
    /// Retourne les paramètres par défaut pour un type de maladie et une gravité
    fn get_issue_params(issue_type: HealthIssueType, severity: Severity) -> (u32, u32, u32, u64) {
        let severity_modifier = severity as u32;
        
        match issue_type {
            HealthIssueType::Infectious => {
                let base_loss = 50 * severity_modifier;
                let base_days = 7 * severity_modifier;
                let base_time = 2 * severity_modifier;
                let base_money = 1000 * severity_modifier as u64;
                (base_loss, base_days, base_time, base_money)
            }
            HealthIssueType::Injury => {
                let base_loss = 30 * severity_modifier;
                let base_days = 14 * severity_modifier;
                let base_time = 4 * severity_modifier;
                let base_money = 1500 * severity_modifier as u64;
                (base_loss, base_days, base_time, base_money)
            }
            HealthIssueType::Chronic => {
                let base_loss = 10 * severity_modifier;
                let base_days = 30 * severity_modifier; // Les maladies chroniques durent plus longtemps
                let base_time = 2 * severity_modifier;
                let base_money = 500 * severity_modifier as u64;
                (base_loss, base_days, base_time, base_money)
            }
            HealthIssueType::Stress => {
                let base_loss = 20 * severity_modifier;
                let base_days = 5 * severity_modifier;
                let base_time = 1 * severity_modifier;
                let base_money = 200 * severity_modifier as u64;
                (base_loss, base_days, base_time, base_money)
            }
            HealthIssueType::Malnutrition => {
                let base_loss = 40 * severity_modifier;
                let base_days = 10 * severity_modifier;
                let base_time = 2 * severity_modifier;
                let base_money = 800 * severity_modifier as u64;
                (base_loss, base_days, base_time, base_money)
            }
        }
    }
    
    /// Met à jour la maladie (appelé chaque jour)
    /// Retourne la perte de santé pour ce jour
    pub fn update(&mut self, current_day: i64, is_being_treated: bool) -> u32 {
        if self.is_treated {
            // Si le traitement est terminé, vérifier si la maladie est guérie
            if let Some(end_day) = self.treatment_end_day {
                if current_day >= end_day {
                    // La maladie est guérie
                    return 0;
                }
            }
            // Pendant le traitement, la perte est réduite
            (self.daily_health_loss / 2).max(1)
        } else if is_being_treated {
            // Commencer le traitement
            self.is_treated = true;
            self.treatment_end_day = Some(current_day + self.recovery_days as i64);
            // Première journée de traitement : perte réduite
            (self.daily_health_loss / 2).max(1)
        } else {
            // Pas de traitement : perte complète
            self.daily_health_loss
        }
    }
    
    /// Retourne si la maladie est guérie
    pub fn is_cured(&self, current_day: i64) -> bool {
        self.is_treated && 
        self.treatment_end_day.map_or(false, |end_day| current_day >= end_day)
    }
}

// ============================================================================
// SÉQUELLES
// ============================================================================

/// Séquelle permanente après une maladie/blessure
/// 
/// §15.4 : "séquelles"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequela {
    /// Type de séquelle
    pub sequela_type: SequelaType,
    /// Gravité de la séquelle (1-100)
    pub severity: u8,
    /// Impact sur la santé maximale
    pub max_health_reduction: u32,
    /// Impact sur la capacité de travail (en %)
    pub work_capacity_reduction: u8,
    /// Jour d'apparition
    pub onset_day: i64,
}

/// Type de séquelle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SequelaType {
    /// Réduction de la mobilité
    ReducedMobility,
    /// Douleur chronique
    ChronicPain,
    /// Fatigue chronique
    ChronicFatigue,
    /// Problèmes cognitifs
    CognitiveIssues,
    /// Problèmes respiratoires
    RespiratoryIssues,
}

impl Sequela {
    pub fn new(sequela_type: SequelaType, severity: u8, onset_day: i64) -> Self {
        let (health_reduction, work_reduction) = match sequela_type {
            SequelaType::ReducedMobility => (50 * severity as u32 / 100, severity / 2),
            SequelaType::ChronicPain => (30 * severity as u32 / 100, severity / 3),
            SequelaType::ChronicFatigue => (40 * severity as u32 / 100, severity / 3),
            SequelaType::CognitiveIssues => (20 * severity as u32 / 100, severity / 4),
            SequelaType::RespiratoryIssues => (35 * severity as u32 / 100, severity / 3),
        };
        
        Sequela {
            sequela_type,
            severity,
            max_health_reduction: health_reduction,
            work_capacity_reduction: work_reduction,
            onset_day,
        }
    }
}

// ============================================================================
// ÉTAT DE SANTÉ INDIVIDUEL
// ============================================================================

/// État de santé d'un individu
/// 
/// Intègre avec le système de santé de la spec P1 §15.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthState {
    /// Identifiant de l'individu
    pub individual_id: EntityId,
    /// Santé actuelle (0-1000)
    pub health: u32,
    /// Santé maximale (peut être réduite par les séquelles)
    pub max_health: u32,
    /// Maladies/blessures en cours
    pub health_issues: BTreeMap<u64, HealthIssue>,
    /// Séquelles permanentes
    pub sequelae: Vec<Sequela>,
    /// Dernier jour de mise à jour
    pub last_update_day: i64,
    /// Gestionnaire du jeûne (lié à la santé)
    pub fasting_manager: FastingManager,
}

impl HealthState {
    pub fn new(individual_id: EntityId, initial_health: u32) -> Self {
        HealthState {
            individual_id,
            health: initial_health.min(MAX_HEALTH),
            max_health: MAX_HEALTH,
            health_issues: BTreeMap::new(),
            sequelae: Vec::new(),
            last_update_day: 0,
            fasting_manager: FastingManager::new(),
        }
    }
    
    /// Met à jour l'état de santé pour un jour donné
    /// 
    /// Calcule les pertes de santé dues aux maladies et au jeûne.
    /// Applique les soins si disponibles.
    pub fn update(
        &mut self,
        current_day: i64,
        has_eaten: bool,
        is_receiving_care: bool,
        care_time_available: u32,
        care_money_available: u64,
    ) -> u32 {
        let days_passed = (current_day - self.last_update_day) as u32;
        self.last_update_day = current_day;
        
        // Mettre à jour le gestionnaire de jeûne
        if !has_eaten {
            self.fasting_manager.record_fasting(self.individual_id);
        } else {
            self.fasting_manager.reset_fasting(self.individual_id);
        }
        
        let mut total_health_loss: u32 = 0;
        
        // Mettre à jour chaque maladie
        let mut issues_to_remove = Vec::new();
        for (key, issue) in &mut self.health_issues {
            let is_being_treated = is_receiving_care && 
                care_time_available >= issue.care_time_cost &&
                care_money_available >= issue.care_money_cost;
            
            let daily_loss = issue.update(current_day, is_being_treated);
            total_health_loss += daily_loss * days_passed;
            
            // Si la maladie est guérie, la marquer pour suppression
            if issue.is_cured(current_day) {
                issues_to_remove.push(*key);
            }
        }
        
        // Supprimer les maladies guéries
        for key in issues_to_remove {
            self.health_issues.remove(&key);
        }
        
        // Appliquer la pénalité de jeûne
        let fasting_penalty = self.fasting_manager.get_fasting_penalty(self.individual_id);
        total_health_loss += fasting_penalty * 10; // 10 points de santé par demi-heure de pénalité
        
        // Appliquer la perte de santé
        self.health = self.health.saturating_sub(total_health_loss);
        
        // Vérifier si la santé est trop basse (décès)
        if self.health == 0 {
            // Mort
            self.health = 0;
        }
        
        total_health_loss
    }
    
    /// Ajoute une maladie/blessure
    pub fn add_health_issue(&mut self, issue: HealthIssue) {
        self.health_issues.insert(issue.stable_key, issue);
    }
    
    /// Ajoute une séquelle
    pub fn add_sequela(&mut self, sequela: Sequela) {
        let max_reduction = sequela.max_health_reduction;
        self.sequelae.push(sequela);
        // Réduire la santé maximale
        self.max_health = self.max_health.saturating_sub(max_reduction);
        // S'assurer que la santé actuelle ne dépasse pas la nouvelle max
        if self.health > self.max_health {
            self.health = self.max_health;
        }
    }
    
    /// Soigne une maladie si possible
    /// Retourne le temps et l'argent consommés
    pub fn treat_health_issue(
        &mut self,
        stable_key: u64,
        time_budget: &mut TimeBudget,
        money_available: &mut u64,
    ) -> bool {
        if let Some(issue) = self.health_issues.get_mut(&stable_key) {
            // Vérifier qu'on a assez de temps et d'argent
            if time_budget.remaining_eveil() >= issue.care_time_cost &&
               *money_available >= issue.care_money_cost {
                
                // Consommer le temps
                time_budget.allocate(TimeUsage::Soin, issue.care_time_cost);
                *money_available -= issue.care_money_cost;
                
                // Marquer comme traité
                issue.is_treated = true;
                issue.treatment_end_day = Some(self.last_update_day + issue.recovery_days as i64);
                
                return true;
            }
        }
        false
    }
    
    /// Retourne si l'individu peut travailler (santé suffisante)
    pub fn can_work(&self) -> bool {
        self.health >= WORK_INCAPACITY_THRESHOLD
    }
    
    /// Retourne la capacité de travail (en %)
    pub fn work_capacity(&self) -> u8 {
        if !self.can_work() {
            return 0;
        }
        
        let mut capacity = 100u8;
        
        // Réduire par les séquelles
        for sequela in &self.sequelae {
            capacity = capacity.saturating_sub(sequela.work_capacity_reduction);
        }
        
        // Réduire par la santé actuelle
        let health_ratio = self.health as f32 / self.max_health as f32;
        let health_bonus = (health_ratio * 100.0) as u8;
        
        // Capacité finale = min(capacité de base, bonus santé)
        capacity.min(health_bonus)
    }
    
    /// Retourne la gravité globale de la santé
    pub fn overall_severity(&self) -> Severity {
        Severity::from_health(self.health)
    }
}

impl Default for HealthState {
    fn default() -> Self {
        HealthState::new(EntityId::new(crate::entities::EntityType::NamedIndividual, 0), MAX_HEALTH)
    }
}

// ============================================================================
// DISPENSAIRE (FILE D'ATTENTE)
// ============================================================================

/// File d'attente du dispensaire
/// 
/// §15.4 : "file du dispensaire (bouchon)"
/// Les soins peuvent être retardés par la file d'attente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispensaryQueue {
    /// Patients en attente de soins
    patients: VecDeque<DispensaryPatient>,
    /// Capacité du dispensaire (nombre de patients par jour)
    capacity: u32,
    /// Jour courant
    current_day: i64,
}

/// Patient dans la file du dispensaire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispensaryPatient {
    pub individual_id: EntityId,
    /// Priorité (0 = très urgent, 100 = peu urgent)
    pub priority: u8,
    /// Jour d'arrivée dans la file
    pub arrival_day: i64,
    /// Type de soin nécessaire
    pub care_type: CareType,
}

/// Type de soin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CareType {
    /// Soin de base
    Basic,
    /// Soin intensif
    Intensive,
    /// Urgence
    Emergency,
}

impl DispensaryQueue {
    pub fn new(capacity: u32) -> Self {
        DispensaryQueue {
            patients: VecDeque::new(),
            capacity,
            current_day: 0,
        }
    }
    
    /// Ajoute un patient à la file
    pub fn add_patient(&mut self, patient: DispensaryPatient) {
        // Insérer le patient dans la file en fonction de la priorité
        // (priorité plus basse = plus urgent)
        let mut inserted = false;
        for (i, existing) in self.patients.iter().enumerate() {
            if patient.priority < existing.priority {
                self.patients.insert(i, patient.clone());
                inserted = true;
                break;
            }
        }
        
        if !inserted {
            self.patients.push_back(patient);
        }
    }
    
    /// Traite les patients pour le jour courant
    /// Retourne la liste des patients traités et ceux qui restent
    pub fn process_day(&mut self, day: i64) -> (Vec<EntityId>, usize) {
        self.current_day = day;
        
        let mut treated = Vec::new();
        let capacity = self.capacity as usize;
        
        for _ in 0..capacity.min(self.patients.len()) {
            if let Some(patient) = self.patients.pop_front() {
                treated.push(patient.individual_id);
            }
        }
        
        (treated, self.patients.len())
    }
    
    /// Retourne le nombre de patients en attente
    pub fn queue_length(&self) -> usize {
        self.patients.len()
    }
    
    /// Vérifie si la file est vide
    pub fn is_empty(&self) -> bool {
        self.patients.is_empty()
    }
    
    /// Retourne la durée d'attente estimée pour un nouveau patient
    pub fn estimated_wait_time(&self) -> u32 {
        // Durée = (nombre de patients / capacité) jours
        if self.capacity == 0 {
            return 0;
        }
        
        let days = self.patients.len() as f32 / self.capacity as f32;
        days.ceil() as u32
    }

    /// Calcule le taux de service des soins en millièmes [0, 1000]
    ///
    /// Formule : patients servis sur les patients demandeurs, en millièmes.
    /// - Si aucun patient ne demande de soin, retourne 1000
    /// - Sinon, les soins servis sont bornés par la capacité quotidienne
    /// - Résultat dans [0, 1000]
    ///
    /// @returns Taux de service en millièmes
    pub fn calculate_service_rate(&self) -> u16 {
        let demand = self.patients.len() as u64;
        if demand == 0 {
            return 1000;
        }

        let treated = demand.min(u64::from(self.capacity));
        ((treated * 1000) / demand) as u16
    }
}

impl Default for DispensaryQueue {
    fn default() -> Self {
        DispensaryQueue::new(10) // Capacité par défaut : 10 patients/jour
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
    fn test_health_state_creation() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let health_state = HealthState::new(individual_id, MAX_HEALTH);
        
        assert_eq!(health_state.individual_id, individual_id);
        assert_eq!(health_state.health, MAX_HEALTH);
        assert_eq!(health_state.max_health, MAX_HEALTH);
        assert!(health_state.health_issues.is_empty());
        assert!(health_state.sequelae.is_empty());
    }
    
    #[test]
    fn test_health_issue_creation() {
        let issue = HealthIssue::new(
            1,
            HealthIssueType::Infectious,
            0,
            Severity::Mild,
            MAX_HEALTH,
        );
        
        assert_eq!(issue.issue_type, HealthIssueType::Infectious);
        assert_eq!(issue.severity, Severity::Mild);
        assert!(!issue.is_treated);
    }
    
    #[test]
    fn test_health_update_with_fasting() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut health_state = HealthState::new(individual_id, MAX_HEALTH);
        
        // Simuler 4 jours sans nourriture (2 jours de jeûne = 1 demi-heure de pénalité)
        for _ in 0..4 {
            health_state.update(1, false, false, 0, 0);
        }
        
        // Après 4 jours de jeûne, pénalité = 2 demi-heures = 20 points de santé/jour
        // Total perte = 4 * 20 = 80 points
        assert!(health_state.health < MAX_HEALTH);
    }
    
    #[test]
    fn test_health_update_with_illness() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut health_state = HealthState::new(individual_id, MAX_HEALTH);
        
        // Ajouter une maladie
        let issue = HealthIssue::new(
            1,
            HealthIssueType::Infectious,
            0,
            Severity::Moderate,
            MAX_HEALTH,
        );
        health_state.add_health_issue(issue);
        
        // Simuler un jour
        health_state.update(1, true, false, 0, 0);
        
        // La santé devrait avoir diminué
        assert!(health_state.health < MAX_HEALTH);
    }
    
    #[test]
    fn test_dispensary_queue() {
        let mut queue = DispensaryQueue::new(3);
        
        // Ajouter des patients
        for i in 0..5 {
            let patient = DispensaryPatient {
                individual_id: EntityId::new(crate::entities::EntityType::NamedIndividual, i),
                priority: (i * 10) as u8,
                arrival_day: 0,
                care_type: CareType::Basic,
            };
            queue.add_patient(patient);
        }
        
        assert_eq!(queue.queue_length(), 5);
        
        // Traiter un jour
        let (treated, remaining) = queue.process_day(1);
        assert_eq!(treated.len(), 3); // Capacité = 3
        assert_eq!(remaining, 2);
    }
    
    #[test]
    fn test_severity_from_health() {
        assert_eq!(Severity::from_health(900), Severity::Mild);
        assert_eq!(Severity::from_health(600), Severity::Moderate);
        assert_eq!(Severity::from_health(300), Severity::Severe);
        assert_eq!(Severity::from_health(100), Severity::Critical);
    }
    
    #[test]
    fn test_can_work() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        
        // Santé normale
        let health_state_1 = HealthState::new(individual_id, MAX_HEALTH);
        assert!(health_state_1.can_work());
        
        // Santé faible
        let mut health_state_2 = HealthState::new(individual_id, WORK_INCAPACITY_THRESHOLD);
        assert!(health_state_2.can_work());
        
        // Santé trop faible
        let health_state_3 = HealthState::new(individual_id, WORK_INCAPACITY_THRESHOLD - 1);
        assert!(!health_state_3.can_work());
    }
    
    #[test]
    fn test_work_capacity_with_sequelae() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut health_state = HealthState::new(individual_id, MAX_HEALTH);
        
        // Ajouter une séquelle
        let sequela = Sequela::new(SequelaType::ReducedMobility, 50, 0);
        health_state.add_sequela(sequela);
        
        // La capacité de travail devrait être réduite
        assert!(health_state.work_capacity() < 100);
    }
}
