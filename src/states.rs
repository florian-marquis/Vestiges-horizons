//! Vestiges Horizons - Tranche 3 : États déclenchés
//! 
//! Implémentation des états déclenchés selon Docs/spec P1 - quartier vivant.md §15.5
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::entities::EntityId;
use crate::time::{TimeBudget, TimeUsage};

// ============================================================================
// CONSTANTES DES ÉTATS
// ============================================================================

/// Nombre de variables cachées (5 selon la spec)
pub const NUM_HIDDEN_VARIABLES: usize = 5;

/// Valeur maximale d'une variable cachée
pub const MAX_VARIABLE_VALUE: u32 = 1000;

/// Seuil par défaut pour déclencher un état
pub const DEFAULT_TRIGGER_THRESHOLD: u32 = 800;

/// Seuil pour déclencher l'angoisse (anticipatoire)
pub const ANXIETY_THRESHOLD: u32 = 700;

/// Seuil pour déclencher la dépression (stress persistant)
pub const DEPRESSION_THRESHOLD: u32 = 600;

// ============================================================================
// IDENTIFIANTS DES VARIABLES CACHÉES
// ============================================================================

/// Identifiant des 5 variables cachées
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HiddenVariable {
    /// Variable 1 : Stress économique
    EconomicStress = 0,
    /// Variable 2 : Tension sociale
    SocialTension = 1,
    /// Variable 3 : Pression temporelle
    TimePressure = 2,
    /// Variable 4 : Incertitude sur l'avenir
    FutureUncertainty = 3,
    /// Variable 5 : Fatigue accumulée
    AccumulatedFatigue = 4,
}

impl HiddenVariable {
    /// Retourne toutes les variables cachées
    pub fn all() -> [HiddenVariable; NUM_HIDDEN_VARIABLES] {
        [
            HiddenVariable::EconomicStress,
            HiddenVariable::SocialTension,
            HiddenVariable::TimePressure,
            HiddenVariable::FutureUncertainty,
            HiddenVariable::AccumulatedFatigue,
        ]
    }
    
    /// Retourne la valeur par défaut pour cette variable
    pub fn default_value(&self) -> u32 {
        0
    }
}

// ============================================================================
// TYPES D'ÉTATS DÉCLENCHÉS
// ============================================================================

/// Type d'état déclenché
/// 
/// §15.5 : "5 états déclenchés + angoisse/dépression"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TriggeredStateType {
    /// État 1 : Colère (tension sociale élevée)
    Anger = 0,
    /// État 2 : Peur (incertitude élevée)
    Fear = 1,
    /// État 3 : Épuisement (fatigue accumulée)
    Exhaustion = 2,
    /// État 4 : Désespoir (stress économique + incertitude)
    Despair = 3,
    /// État 5 : Euphorie (situation exceptionnellement positive)
    Euphoria = 4,
    /// Angoisse (anticipatoire)
    Anxiety = 5,
    /// Dépression (stress persistant)
    Depression = 6,
}

impl TriggeredStateType {
    /// Retourne la description de l'état
    pub fn description(&self) -> &'static str {
        match self {
            TriggeredStateType::Anger => "Colère",
            TriggeredStateType::Fear => "Peur",
            TriggeredStateType::Exhaustion => "Épuisement",
            TriggeredStateType::Despair => "Désespoir",
            TriggeredStateType::Euphoria => "Euphorie",
            TriggeredStateType::Anxiety => "Angoisse",
            TriggeredStateType::Depression => "Dépression",
        }
    }
    
    /// Retourne si cet état est permanent (angoisse/dépression) ou temporaire
    pub fn is_persistent(&self) -> bool {
        match self {
            TriggeredStateType::Anxiety | TriggeredStateType::Depression => true,
            _ => false,
        }
    }
    
    /// Retourne l'impact sur le comportement
    /// Valeur positive = impact négatif, négative = impact positif
    pub fn behavior_impact(&self) -> i32 {
        match self {
            TriggeredStateType::Anger => 50,    // +50% agressivité
            TriggeredStateType::Fear => 30,     // +30% prudence
            TriggeredStateType::Exhaustion => 70, // +70% ralentissement
            TriggeredStateType::Despair => 80,   // +80% passivité
            TriggeredStateType::Euphoria => -20, // -20% prudence (plus audacieux)
            TriggeredStateType::Anxiety => 40,   // +40% anticipation
            TriggeredStateType::Depression => 60, // +60% passivité
        }
    }
}

// ============================================================================
// ÉTAT DÉCLENCHÉ
// ============================================================================

/// État déclenché pour un individu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredState {
    pub state_type: TriggeredStateType,
    /// Intensité de l'état (0-100)
    pub intensity: u8,
    /// Jour de déclenchement
    pub trigger_day: i64,
    /// Durée prévue en jours (0 = permanent)
    pub duration_days: u32,
    /// Jour de fin prévu (None = permanent)
    pub end_day: Option<i64>,
    /// Causes (références aux variables cachées)
    pub causes: Vec<HiddenVariable>,
}

impl TriggeredState {
    pub fn new(
        state_type: TriggeredStateType,
        intensity: u8,
        trigger_day: i64,
        duration_days: u32,
        causes: Vec<HiddenVariable>,
    ) -> Self {
        let end_day = if duration_days > 0 {
            Some(trigger_day + duration_days as i64)
        } else {
            None
        };
        
        TriggeredState {
            state_type,
            intensity,
            trigger_day,
            duration_days,
            end_day,
            causes,
        }
    }
    
    /// Vérifie si l'état est actif à un jour donné
    pub fn is_active(&self, current_day: i64) -> bool {
        match self.end_day {
            Some(end_day) => current_day >= self.trigger_day && current_day <= end_day,
            None => current_day >= self.trigger_day, // Permanent
        }
    }
    
    /// Retourne si l'état est terminé
    pub fn is_ended(&self, current_day: i64) -> bool {
        match self.end_day {
            Some(end_day) => current_day > end_day,
            None => false, // Permanent ne se termine jamais
        }
    }
    
    /// Met à jour l'intensité (peut changer au fil du temps)
    pub fn update_intensity(&mut self, new_intensity: u8) {
        self.intensity = new_intensity;
    }
}

// ============================================================================
// ÉTATS D'UN INDIVIDU
// ============================================================================

/// Gestionnaire des états pour un individu
/// 
/// §15.5 : "5 variables cachées (0-1000), seuils en données → événements phase 6"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualStates {
    pub individual_id: EntityId,
    /// Valeurs des 5 variables cachées
    pub hidden_variables: [u32; NUM_HIDDEN_VARIABLES],
    /// États actuellement actifs
    pub active_states: BTreeMap<TriggeredStateType, TriggeredState>,
    /// Historique des états
    pub state_history: Vec<TriggeredState>,
    /// Seuil de déclenchement pour chaque variable (peut être personnalisé)
    pub trigger_thresholds: [u32; NUM_HIDDEN_VARIABLES],
    /// Jour de dernière mise à jour
    pub last_update_day: i64,
}

impl IndividualStates {
    pub fn new(individual_id: EntityId) -> Self {
        IndividualStates {
            individual_id,
            hidden_variables: [0; NUM_HIDDEN_VARIABLES],
            active_states: BTreeMap::new(),
            state_history: Vec::new(),
            trigger_thresholds: [DEFAULT_TRIGGER_THRESHOLD; NUM_HIDDEN_VARIABLES],
            last_update_day: 0,
        }
    }
    
    /// Retourne la valeur d'une variable cachée
    pub fn get_variable(&self, variable: HiddenVariable) -> u32 {
        self.hidden_variables[variable as usize]
    }
    
    /// Met à jour la valeur d'une variable cachée
    /// 
    /// §15.5 : "5 variables cachées (0-1000)"
    pub fn set_variable(&mut self, variable: HiddenVariable, value: u32) {
        self.hidden_variables[variable as usize] = value.min(MAX_VARIABLE_VALUE);
    }
    
    /// Ajoute une valeur à une variable cachée
    pub fn add_to_variable(&mut self, variable: HiddenVariable, delta: i32) {
        let current = self.hidden_variables[variable as usize] as i32;
        let new_value = (current + delta).clamp(0, MAX_VARIABLE_VALUE as i32) as u32;
        self.hidden_variables[variable as usize] = new_value;
    }
    
    /// Vérifie les seuils et déclenche les états si nécessaire
    /// 
    /// §15.5 : "seuils en données → événements phase 6"
    pub fn check_triggers(&mut self, current_day: i64) -> Vec<TriggeredStateType> {
        let mut triggered = Vec::new();
        
        // Vérifier chaque variable par rapport à son seuil
        for variable in HiddenVariable::all() {
            let value = self.get_variable(variable);
            let threshold = self.trigger_thresholds[variable as usize];
            
            if value >= threshold {
                // Déclencher l'état correspondant
                let state_type = match variable {
                    HiddenVariable::EconomicStress => TriggeredStateType::Despair,
                    HiddenVariable::SocialTension => TriggeredStateType::Anger,
                    HiddenVariable::TimePressure => TriggeredStateType::Exhaustion,
                    HiddenVariable::FutureUncertainty => TriggeredStateType::Fear,
                    HiddenVariable::AccumulatedFatigue => TriggeredStateType::Exhaustion,
                };
                
                // Vérifier si l'état est déjà actif
                if !self.active_states.contains_key(&state_type) {
                    let state = TriggeredState::new(
                        state_type,
                        100, // Intensité maximale par défaut
                        current_day,
                        7,   // Durée de 7 jours par défaut
                        vec![variable],
                    );
                    
                    self.active_states.insert(state_type, state.clone());
                    self.state_history.push(state);
                    triggered.push(state_type);
                }
            }
        }
        
        // Vérifier les états persistants (angoisse, dépression)
        self.check_persistent_states(current_day, &mut triggered);
        
        triggered
    }
    
    /// Vérifie les états persistants (angoisse et dépression)
    /// 
    /// §15.5 : "angoisse/dépression"
    /// - Angoisse : état anticipatoire
    /// - Dépression : état persistant du stress
    fn check_persistent_states(&mut self, current_day: i64, triggered: &mut Vec<TriggeredStateType>) {
        // Angoisse : déclenchée si la moyenne des variables > ANXIETY_THRESHOLD
        let avg_variable: u32 = self.hidden_variables.iter().sum::<u32>() / NUM_HIDDEN_VARIABLES as u32;
        
        if avg_variable >= ANXIETY_THRESHOLD {
            if !self.active_states.contains_key(&TriggeredStateType::Anxiety) {
                // Calculer l'intensité comme pourcentage de l'excès par rapport au seuil
                // Si avg = 700, intensity = 0; si avg = 800, intensity = 100/100 = 1; si avg = 1000, intensity = 300/100 = 3
                // Mais on veut une intensité de 0-100, donc : min((avg - threshold) / 3, 100)
                let intensity = ((avg_variable - ANXIETY_THRESHOLD) / 3) as u8;
                let state = TriggeredState::new(
                    TriggeredStateType::Anxiety,
                    intensity.min(100),
                    current_day,
                    0, // Permanent
                    HiddenVariable::all().to_vec(),
                );
                
                self.active_states.insert(TriggeredStateType::Anxiety, state.clone());
                self.state_history.push(state);
                triggered.push(TriggeredStateType::Anxiety);
            }
        }
        
        // Dépression : déclenchée si la moyenne des variables > DEPRESSION_THRESHOLD
        if avg_variable >= DEPRESSION_THRESHOLD {
            if !self.active_states.contains_key(&TriggeredStateType::Depression) {
                // Calculer l'intensité comme pourcentage de l'excès
                let intensity = ((avg_variable - DEPRESSION_THRESHOLD) / 4) as u8;
                let state = TriggeredState::new(
                    TriggeredStateType::Depression,
                    intensity.min(100), // Intensité proportionnelle
                    current_day,
                    0, // Permanent
                    HiddenVariable::all().to_vec(),
                );
                
                self.active_states.insert(TriggeredStateType::Depression, state.clone());
                self.state_history.push(state);
                triggered.push(TriggeredStateType::Depression);
            }
        }
    }
    
    /// Met à jour les états actifs (supprime ceux qui sont terminés)
    pub fn update_states(&mut self, current_day: i64) {
        let mut to_remove = Vec::new();
        
        for (state_type, state) in &self.active_states {
            if state.is_ended(current_day) {
                to_remove.push(*state_type);
            }
        }
        
        for state_type in to_remove {
            self.active_states.remove(&state_type);
        }
    }
    
    /// Retourne si un état est actif
    pub fn has_state(&self, state_type: TriggeredStateType) -> bool {
        self.active_states.contains_key(&state_type)
    }
    
    /// Retourne l'état actif de ce type (si présent)
    pub fn get_state(&self, state_type: TriggeredStateType) -> Option<&TriggeredState> {
        self.active_states.get(&state_type)
    }
    
    /// Retourne tous les états actifs
    pub fn get_active_states(&self) -> Vec<&TriggeredState> {
        self.active_states.values().collect()
    }
    
    /// Termine un état manuellement (par exemple, après une intervention)
    pub fn end_state(&mut self, state_type: TriggeredStateType, current_day: i64) {
        if let Some(mut state) = self.active_states.remove(&state_type) {
            state.end_day = Some(current_day);
            self.state_history.push(state);
        }
    }
    
    /// Met à jour les intensités des états en fonction des variables
    pub fn update_intensities(&mut self) {
        // Collecter les variables et seuils dans des variables locales
        let hidden_variables = self.hidden_variables;
        let trigger_thresholds = self.trigger_thresholds;
        
        // Collecter les types d'états et leurs causes
        let states_to_update: Vec<(TriggeredStateType, Vec<HiddenVariable>)> = 
            self.active_states.iter()
                .map(|(state_type, state)| (*state_type, state.causes.clone()))
                .collect();
        
        for (state_type, causes) in states_to_update {
            if let Some(state) = self.active_states.get_mut(&state_type) {
                // Calculer l'intensité en fonction des variables causales
                let mut total_intensity: u32 = 0;
                
                for &variable in &causes {
                    let value = hidden_variables[variable as usize];
                    let threshold = trigger_thresholds[variable as usize];
                    
                    if value > threshold {
                        // Intensité proportionnelle à l'excès par rapport au seuil
                        total_intensity += (value - threshold) / 10;
                    }
                }
                
                // Normaliser à 0-100
                let avg_intensity = if causes.is_empty() {
                    0
                } else {
                    (total_intensity / causes.len() as u32) as u8
                };
                state.update_intensity(avg_intensity.min(100));
            }
        }
    }
    
    /// Applique les effets des états au budget de temps
    /// 
    /// Les états peuvent modifier l'allocation du temps (ex : épuisement réduit le travail)
    pub fn apply_state_effects_to_time(&self, budget: &mut TimeBudget) {
        for state in self.get_active_states() {
            match state.state_type {
                TriggeredStateType::Anger => {
                    // La colère peut augmenter les corvées (agitation)
                    // et réduire le travail (difficulté de concentration)
                    let anger_reduction = (state.intensity as u32 * 10 / 100).min(budget.get_allocation(TimeUsage::Travail));
                    let corvee_increase = (state.intensity as u32 * 10 / 100).min(4);
                    
                    budget.deallocate(TimeUsage::Travail, anger_reduction);
                    budget.allocate(TimeUsage::Corvee, corvee_increase);
                }
                TriggeredStateType::Fear => {
                    // La peur réduit les activités à risque (travail, déplacement)
                    let fear_reduction = (state.intensity as u32 * 15 / 100).min(budget.get_allocation(TimeUsage::Travail));
                    budget.deallocate(TimeUsage::Travail, fear_reduction);
                    budget.allocate(TimeUsage::Sommeil, fear_reduction / 2);
                }
                TriggeredStateType::Exhaustion => {
                    // L'épuisement réduit fortement le travail et les corvées
                    let exhaustion_reduction = (state.intensity as u32 * 25 / 100);
                    let travail = budget.get_allocation(TimeUsage::Travail);
                    let corvee = budget.get_allocation(TimeUsage::Corvee);
                    
                    let travail_reduction = exhaustion_reduction.min(travail);
                    let corvee_reduction = (exhaustion_reduction / 2).min(corvee);
                    
                    budget.deallocate(TimeUsage::Travail, travail_reduction);
                    budget.deallocate(TimeUsage::Corvee, corvee_reduction);
                    budget.allocate(TimeUsage::Sommeil, travail_reduction + corvee_reduction);
                }
                TriggeredStateType::Despair => {
                    // Le désespoir réduit toutes les activités
                    let despair_reduction = (state.intensity as u32 * 30 / 100);
                    let total_reduction = despair_reduction.min(budget.remaining_eveil() + budget.get_allocation(TimeUsage::Travail) + budget.get_allocation(TimeUsage::Corvee));
                    
                    // Réduire proportionnellement
                    let travail = budget.get_allocation(TimeUsage::Travail);
                    let corvee = budget.get_allocation(TimeUsage::Corvee);
                    
                    if total_reduction > 0 && (travail + corvee) > 0 {
                        let travail_ratio = travail as f32 / (travail + corvee) as f32;
                        let corvee_ratio = corvee as f32 / (travail + corvee) as f32;
                        
                        let travail_reduction = (total_reduction as f32 * travail_ratio) as u32;
                        let corvee_reduction = (total_reduction as f32 * corvee_ratio) as u32;
                        
                        budget.deallocate(TimeUsage::Travail, travail_reduction);
                        budget.deallocate(TimeUsage::Corvee, corvee_reduction);
                    }
                }
                TriggeredStateType::Euphoria => {
                    // L'euphorie augmente le travail et réduit le sommeil
                    let euphoria_boost = (state.intensity as u32 * 10 / 100).min(4);
                    
                    if budget.remaining_eveil() >= euphoria_boost {
                        budget.deallocate(TimeUsage::Sommeil, euphoria_boost);
                        budget.allocate(TimeUsage::Travail, euphoria_boost);
                    }
                }
                TriggeredStateType::Anxiety => {
                    // L'angoisse augmente la recherche et réduit le travail
                    let anxiety_reduction = (state.intensity as u32 * 10 / 100).min(budget.get_allocation(TimeUsage::Travail));
                    
                    budget.deallocate(TimeUsage::Travail, anxiety_reduction);
                    budget.allocate(TimeUsage::Recherche, anxiety_reduction);
                }
                TriggeredStateType::Depression => {
                    // La dépression réduit fortement toutes les activités
                    let depression_reduction = (state.intensity as u32 * 40 / 100);
                    let total_activity = budget.get_allocation(TimeUsage::Travail) + 
                                        budget.get_allocation(TimeUsage::Corvee) + 
                                        budget.get_allocation(TimeUsage::Formation) + 
                                        budget.get_allocation(TimeUsage::Loisir);
                    
                    let reduction = depression_reduction.min(total_activity);
                    
                    if reduction > 0 && total_activity > 0 {
                        // Réduire proportionnellement toutes les activités
                        let travail_ratio = budget.get_allocation(TimeUsage::Travail) as f32 / total_activity as f32;
                        let corvee_ratio = budget.get_allocation(TimeUsage::Corvee) as f32 / total_activity as f32;
                        let formation_ratio = budget.get_allocation(TimeUsage::Formation) as f32 / total_activity as f32;
                        let loisir_ratio = budget.get_allocation(TimeUsage::Loisir) as f32 / total_activity as f32;
                        
                        let travail_reduction = (reduction as f32 * travail_ratio) as u32;
                        let corvee_reduction = (reduction as f32 * corvee_ratio) as u32;
                        let formation_reduction = (reduction as f32 * formation_ratio) as u32;
                        let loisir_reduction = (reduction as f32 * loisir_ratio) as u32;
                        
                        budget.deallocate(TimeUsage::Travail, travail_reduction);
                        budget.deallocate(TimeUsage::Corvee, corvee_reduction);
                        budget.deallocate(TimeUsage::Formation, formation_reduction);
                        budget.deallocate(TimeUsage::Loisir, loisir_reduction);
                        
                        // Ajouter au sommeil
                        budget.allocate(TimeUsage::Sommeil, reduction);
                    }
                }
            }
        }
    }
}

impl Default for IndividualStates {
    fn default() -> Self {
        IndividualStates::new(EntityId::new(crate::entities::EntityType::NamedIndividual, 0))
    }
}

// ============================================================================
// GESTIONNAIRE D'ÉTATS
// ============================================================================

/// Gestionnaire des états pour tous les individus
pub struct StatesManager {
    /// États par individu
    individual_states: BTreeMap<EntityId, IndividualStates>,
    /// Seuil global pour chaque variable (peut être différent par individu)
    global_thresholds: [u32; NUM_HIDDEN_VARIABLES],
}

impl StatesManager {
    pub fn new() -> Self {
        StatesManager {
            individual_states: BTreeMap::new(),
            global_thresholds: [DEFAULT_TRIGGER_THRESHOLD; NUM_HIDDEN_VARIABLES],
        }
    }
    
    /// Crée ou retourne les états pour un individu
    pub fn get_or_create_states(&mut self, individual_id: EntityId) -> &mut IndividualStates {
        self.individual_states
            .entry(individual_id)
            .or_insert_with(|| IndividualStates::new(individual_id))
    }
    
    /// Met à jour tous les états pour un jour donné
    pub fn update_all(&mut self, current_day: i64) {
        for states in self.individual_states.values_mut() {
            states.update_states(current_day);
            states.check_triggers(current_day);
            states.update_intensities();
        }
    }
    
    /// Vérifie les états pour tous les individus
    pub fn check_all_triggers(&mut self, current_day: i64) -> BTreeMap<EntityId, Vec<TriggeredStateType>> {
        let mut triggered = BTreeMap::new();
        
        for (individual_id, states) in &mut self.individual_states {
            let triggered_states = states.check_triggers(current_day);
            if !triggered_states.is_empty() {
                triggered.insert(*individual_id, triggered_states);
            }
        }
        
        triggered
    }
    
    /// Retourne les états pour un individu
    pub fn get_states(&self, individual_id: EntityId) -> Option<&IndividualStates> {
        self.individual_states.get(&individual_id)
    }
    
    /// Définit un seuil global pour une variable
    pub fn set_global_threshold(&mut self, variable: HiddenVariable, threshold: u32) {
        self.global_thresholds[variable as usize] = threshold;
    }
}

impl Default for StatesManager {
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
    use crate::time::{TimeBudget, TimeUsage, BUDGET_EVEIL};
    
    #[test]
    fn test_individual_states_creation() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let states = IndividualStates::new(individual_id);
        
        assert_eq!(states.individual_id, individual_id);
        
        for variable in HiddenVariable::all() {
            assert_eq!(states.get_variable(variable), 0);
        }
    }
    
    #[test]
    fn test_set_variable() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut states = IndividualStates::new(individual_id);
        
        states.set_variable(HiddenVariable::EconomicStress, 500);
        
        assert_eq!(states.get_variable(HiddenVariable::EconomicStress), 500);
    }
    
    #[test]
    fn test_trigger_state() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut states = IndividualStates::new(individual_id);
        
        // Déclencher un état en dépassant le seuil
        states.set_variable(HiddenVariable::EconomicStress, 900);
        
        let triggered = states.check_triggers(0);
        
        assert!(triggered.contains(&TriggeredStateType::Despair));
    }
    
    #[test]
    fn test_persistent_states() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut states = IndividualStates::new(individual_id);
        
        // Déclencher l'angoisse en dépassant le seuil moyen
        for variable in HiddenVariable::all() {
            states.set_variable(variable, 750);
        }
        
        let triggered = states.check_triggers(0);
        
        assert!(triggered.contains(&TriggeredStateType::Anxiety));
    }
    
    #[test]
    fn test_state_effects_on_time() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut states = IndividualStates::new(individual_id);
        
        // Créer un budget de temps
        let mut budget = TimeBudget::new(individual_id, 0);
        budget.allocate(TimeUsage::Travail, 8);
        budget.allocate(TimeUsage::Corvee, 4);
        budget.allocate(TimeUsage::Repas, 2);
        budget.allocate(TimeUsage::Trajet, 1);
        
        // Déclencher l'épuisement
        states.set_variable(HiddenVariable::AccumulatedFatigue, 900);
        states.check_triggers(0);
        
        // Appliquer les effets
        states.apply_state_effects_to_time(&mut budget);
        
        // Le travail devrait avoir été réduit
        assert!(budget.get_allocation(TimeUsage::Travail) < 8);
    }
    
    #[test]
    fn test_anxiety_effects() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut states = IndividualStates::new(individual_id);
        
        // Créer un budget de temps
        let mut budget = TimeBudget::new(individual_id, 0);
        budget.allocate(TimeUsage::Travail, 8);
        budget.allocate(TimeUsage::Corvee, 4);
        budget.allocate(TimeUsage::Repas, 2);
        budget.allocate(TimeUsage::Trajet, 1);
        budget.allocate(TimeUsage::Recherche, 0);
        
        // Déclencher l'angoisse
        for variable in HiddenVariable::all() {
            states.set_variable(variable, 750);
        }
        states.check_triggers(0);
        
        // Appliquer les effets
        states.apply_state_effects_to_time(&mut budget);
        
        // La recherche devrait avoir augmenté
        assert!(budget.get_allocation(TimeUsage::Recherche) > 0);
        // Le travail devrait avoir diminué
        assert!(budget.get_allocation(TimeUsage::Travail) < 8);
    }
    
    #[test]
    fn test_depression_effects() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut states = IndividualStates::new(individual_id);
        
        // Créer un budget de temps
        let mut budget = TimeBudget::new(individual_id, 0);
        budget.allocate(TimeUsage::Travail, 8);
        budget.allocate(TimeUsage::Corvee, 4);
        budget.allocate(TimeUsage::Repas, 2);
        budget.allocate(TimeUsage::Trajet, 1);
        budget.allocate(TimeUsage::Formation, 2);
        budget.allocate(TimeUsage::Loisir, 1);
        
        // Déclencher la dépression
        for variable in HiddenVariable::all() {
            states.set_variable(variable, 800);
        }
        states.check_triggers(0);
        
        // Appliquer les effets
        states.apply_state_effects_to_time(&mut budget);
        
        // Toutes les activités devraient avoir été réduites
        assert!(budget.get_allocation(TimeUsage::Travail) < 8);
        assert!(budget.get_allocation(TimeUsage::Corvee) < 4);
        assert!(budget.get_allocation(TimeUsage::Formation) < 2);
        assert!(budget.get_allocation(TimeUsage::Loisir) < 1);
        // Le sommeil devrait avoir augmenté
        assert!(budget.get_allocation(TimeUsage::Sommeil) > 0);
    }
    
    #[test]
    fn test_state_lifecycle() {
        let individual_id = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let mut states = IndividualStates::new(individual_id);
        
        // Déclencher un état
        states.set_variable(HiddenVariable::SocialTension, 900);
        states.check_triggers(0);
        
        assert!(states.has_state(TriggeredStateType::Anger));
        
        // Avancer dans le temps
        states.update_states(8); // 8 jours plus tard (durée par défaut = 7 jours)
        
        assert!(!states.has_state(TriggeredStateType::Anger));
    }
    
    #[test]
    fn test_triggered_state_type() {
        assert!(TriggeredStateType::Anxiety.is_persistent());
        assert!(TriggeredStateType::Depression.is_persistent());
        assert!(!TriggeredStateType::Anger.is_persistent());
    }
}
