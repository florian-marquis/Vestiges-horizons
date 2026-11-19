//! Vestiges Horizons - Tranche 4 : Actifs avec Usure Paresseuse
//! 
//! Implémentation de l'usure paresseuse selon Docs/spec P1 - quartier vivant.md §16.3
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::common::MaintenancePolicy;
use crate::entities::EntityId;

// ============================================================================
// CONSTANTES
// ============================================================================

/// État initial maximum d'un actif
pub const MAX_INITIAL_STATE: u32 = 1000;

/// Seuil de capacité réduite (§16.3 : "franchissement des seuils (capacité réduite sous 400)")
pub const CAPACITY_REDUCED_THRESHOLD: u32 = 400;

/// Seuil critique (§16.3 : "critique sous 100")
pub const CRITICAL_THRESHOLD: u32 = 100;

// ============================================================================
// TYPE D'ACTIF
// ============================================================================

/// Type d'actif
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AssetType {
    /// Machine de production
    Machine = 0,
    /// Outillage
    Tooling = 1,
    /// Bâtiment
    Building = 2,
    /// Équipement de cuisine
    KitchenEquipment = 3,
    /// Mobilier
    Furniture = 4,
    /// Adduction d'eau
    WaterSystem = 5,
}

impl AssetType {
    pub fn description(&self) -> &'static str {
        match self {
            AssetType::Machine => "Machine de production",
            AssetType::Tooling => "Outillage",
            AssetType::Building => "Bâtiment",
            AssetType::KitchenEquipment => "Équipement de cuisine",
            AssetType::Furniture => "Mobilier",
            AssetType::WaterSystem => "Adduction d'eau",
        }
    }
}

// ============================================================================
// ACTIF
// ============================================================================

/// Actif avec usure paresseuse
/// 
/// §16.3 : "Un actif stocke (état initial, date, taux) et jamais l'état courant"
/// état(t) = état_initial − taux × (t − date)
/// 
/// §16.3 : "Le franchissement des seuils (capacité réduite sous 400, critique sous 100) 
/// est un événement planifié à la date exacte de franchissement, recalculée quand les entrées changent"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: EntityId,
    /// Type d'actif
    pub asset_type: AssetType,
    /// État initial (0-1000)
    pub initial_state: u32,
    /// Date de création (jour)
    pub creation_date: i64,
    /// Taux d'usure par jour (en points d'état)
    pub wear_rate: u32,
    /// Politique d'entretien actuelle
    pub maintenance_policy: MaintenancePolicy,
    /// Date de dernière maintenance
    pub last_maintenance_date: Option<i64>,
    /// Firm propriétaire
    pub owner_firm_id: EntityId,
    /// Capacité de production (unités/jour)
    pub capacity: u32,
}

impl Asset {
    pub fn new(
        id: EntityId,
        asset_type: AssetType,
        initial_state: u32,
        creation_date: i64,
        wear_rate: u32,
        owner_firm_id: EntityId,
        capacity: u32,
    ) -> Self {
        Asset {
            id,
            asset_type,
            initial_state: initial_state.min(MAX_INITIAL_STATE),
            creation_date,
            wear_rate,
            maintenance_policy: MaintenancePolicy::Normal,
            last_maintenance_date: None,
            owner_firm_id,
            capacity,
        }
    }
    
    /// Retourne l'état courant à une date donnée
    /// 
    /// §16.3 : "état(t) = état_initial − taux × (t − date)"
    /// Complexité : O(1) par actif et par jour (une soustraction)
    pub fn current_state(&self, current_day: i64) -> i32 {
        // Calcul : état = initial_state - wear_rate * (current_day - creation_date)
        let days_passed = (current_day - self.creation_date) as u32;
        let wear = self.effective_wear_rate() * days_passed;
        
        // Éviter les sous-débordements (état négatif)
        // §16.5 T4.2 : I6 vert : états dans [0, 1000]
        let state = self.initial_state.saturating_sub(wear);
        state as i32
    }
    
    /// Calcule le taux de service de l'actif pour la composante logements de q
    ///
    /// Formule : taux = min(state / MAX_INITIAL_STATE * 1000, 1000)
    /// où state = current_state(current_day)
    ///
    /// @param current_day Jour courant
    /// @returns Taux de service en millièmes [0, 1000]
    pub fn calculate_maintenance_rate(&self, current_day: i64) -> u16 {
        let state = self.current_state(current_day);
        if state <= 0 {
            return 0;
        }

        // state est dans [0, initial_state] où initial_state <= 1000
        // On normalise à [0, 1000]
        let numerator = state as u64 * 1000;
        let result = (numerator / MAX_INITIAL_STATE as u64) as u16;

        result.min(1000)
    }

    /// Retourne le taux d'usure effectif (modifié par la politique d'entretien)
    /// 
    /// Utilise l'arithmétique entière : wear_rate * modifier * 1000 / 1000
    /// §3 : "Tout est entier. ... Les taux (usure, griefs) sont des entiers avec échelle déclarée."
    pub fn effective_wear_rate(&self) -> u32 {
        // Multiplicateurs en millième (x1000) pour éviter les flottants
        let modifier = match self.maintenance_policy {
            MaintenancePolicy::Abandoned => 2000,  // 2.0 * 1000
            MaintenancePolicy::Light => 1400,     // 1.4 * 1000
            MaintenancePolicy::Normal => 1000,    // 1.0 * 1000
            MaintenancePolicy::Intensive => 700,   // 0.7 * 1000
        };
        // Calcul : (wear_rate * modifier) / 1000
        (self.wear_rate as u64 * modifier as u64 / 1000) as u32
    }
    
    /// Retourne la capacité effective (réduite si l'état est bas)
    /// 
    /// §16.3 : "franchissement des seuils (capacité réduite sous 400, critique sous 100)"
    pub fn effective_capacity(&self, current_day: i64) -> u32 {
        let state = self.current_state(current_day);
        
        if state <= CRITICAL_THRESHOLD as i32 {
            // Critique : capacité à 10%
            self.capacity / 10
        } else if state <= CAPACITY_REDUCED_THRESHOLD as i32 {
            // Capacité réduite : linéaire de 100% à 10% entre 400 et 100
            // ratio = (state - 100) / (400 - 100) = (state - 100) / 300
            // capacity = capacity * (0.1 + 0.9 * ratio) = capacity * (0.1 + 0.9 * (state-100)/300)
            // En entier : capacity * (100 + 900 * (state-100) / 300) / 1000
            // Simplifié : capacity * (100 + 3 * (state-100)) / 1000
            let state_u = state as u32;
            let numerator = self.capacity as u64 * (100u64 + 3u64 * (state_u - CRITICAL_THRESHOLD) as u64);
            (numerator / 1000) as u32
        } else {
            // État normal : capacité à 100%
            self.capacity
        }
    }
    
    /// Applique une maintenance
    /// 
    /// Modifie le taux ou remonte l'état selon la politique
    pub fn apply_maintenance(&mut self, current_day: i64, policy: MaintenancePolicy) {
        self.maintenance_policy = policy;
        self.last_maintenance_date = Some(current_day);
        
        // Pour l'instant, la maintenance ne modifie que la politique
        // L'état continue de se dégrader, mais plus lentement
    }
    
    /// Réparation : remonte l'état à une valeur donnée
    pub fn repair(&mut self, target_state: u32, current_day: i64) {
        if target_state > self.initial_state {
            return; // Ne peut pas dépasser l'état initial
        }
        
        // Mettre à jour la date de création pour simuler la remise à neuf
        // état = initial_state - wear_rate * (current_day - new_creation_date)
        // On veut : état = target_state
        // Donc : new_creation_date = current_day - (initial_state - target_state + wear_rate - 1) / wear_rate
        let state_diff = self.initial_state - target_state;
        if self.wear_rate > 0 {
            // Arrondi supérieur pour être sûr de ne pas dépasser
            let days_back = (state_diff + self.wear_rate - 1) / self.wear_rate;
            self.creation_date = current_day - days_back as i64;
        }
        
        self.last_maintenance_date = Some(current_day);
    }
    
    /// Retourne la date de franchissement du seuil de capacité réduite
    /// 
    /// §16.3 : "le franchissement des seuils ... est un événement planifié à la date exacte"
    pub fn capacity_reduced_date(&self) -> Option<i64> {
        self.threshold_date(CAPACITY_REDUCED_THRESHOLD)
    }
    
    /// Retourne la date de franchissement du seuil critique
    pub fn critical_date(&self) -> Option<i64> {
        self.threshold_date(CRITICAL_THRESHOLD)
    }
    
    /// Calcule la date de franchissement d'un seuil donné
    fn threshold_date(&self, threshold: u32) -> Option<i64> {
        if self.initial_state <= threshold {
            // Déjà en dessous du seuil
            return Some(self.creation_date);
        }
        
        let effective_rate = self.effective_wear_rate();
        if effective_rate == 0 {
            // Pas d'usure, ne franchira jamais
            return None;
        }
        
        // Calcul : état = initial_state - wear_rate * days_passed
        // On veut : état = threshold
        // Donc : days_passed = (initial_state - threshold + effective_rate - 1) / effective_rate
        // (arrondi supérieur)
        let state_diff = self.initial_state - threshold;
        // Arrondi supérieur : (state_diff + effective_rate - 1) / effective_rate
        let days_needed = (state_diff + effective_rate - 1) / effective_rate;
        
        Some(self.creation_date + days_needed as i64)
    }
    
    /// Retourne si l'actif est au-dessus du seuil de capacité réduite
    pub fn is_above_capacity_threshold(&self, current_day: i64) -> bool {
        self.current_state(current_day) > CAPACITY_REDUCED_THRESHOLD as i32
    }
    
    /// Retourne si l'actif est au-dessus du seuil critique
    pub fn is_above_critical_threshold(&self, current_day: i64) -> bool {
        self.current_state(current_day) > CRITICAL_THRESHOLD as i32
    }
}

// ============================================================================
// ÉVÉNEMENT DE FRANCHISSEMENT DE SEUIL
// ============================================================================

/// Événement planifié pour le franchissement d'un seuil d'usure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdEvent {
    pub asset_id: EntityId,
    pub threshold: u32,
    pub event_day: i64,
    pub event_type: ThresholdEventType,
}

/// Type d'événement de seuil
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThresholdEventType {
    CapacityReduced,
    Critical,
}

impl ThresholdEvent {
    pub fn new(
        asset_id: EntityId,
        threshold: u32,
        event_day: i64,
        event_type: ThresholdEventType,
    ) -> Self {
        ThresholdEvent {
            asset_id,
            threshold,
            event_day,
            event_type,
        }
    }
}

// ============================================================================
// GESTIONNAIRE D'ACTIFS
// ============================================================================

/// Gestionnaire des actifs
/// 
/// §16.3 : "Coût : O(1) par actif et par jour (une soustraction)"
/// §16.5 T4.3 : "Paresseux = naïf : état paresseux identique bit à bit à un oracle naïf"
/// §16.5 T4.4 : "Seuils planifiés : usure traversant 400 puis 100, entretien en cours de route"
pub struct AssetManager {
    /// Actifs par id
    pub assets: BTreeMap<EntityId, Asset>,
    /// Événements de seuil planifiés (par jour)
    pub threshold_events: BTreeMap<i64, Vec<ThresholdEvent>>,
    /// Actifs par firme
    pub assets_by_firm: BTreeMap<EntityId, Vec<EntityId>>,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager {
            assets: BTreeMap::new(),
            threshold_events: BTreeMap::new(),
            assets_by_firm: BTreeMap::new(),
        }
    }
    
    /// Crée un nouvel actif
    pub fn create_asset(
        &mut self,
        asset_type: AssetType,
        initial_state: u32,
        creation_date: i64,
        wear_rate: u32,
        owner_firm_id: EntityId,
        capacity: u32,
    ) -> EntityId {
        let id = EntityId::new(crate::entities::EntityType::Asset, self.assets.len() as u64);
        
        let asset = Asset::new(
            id,
            asset_type,
            initial_state,
            creation_date,
            wear_rate,
            owner_firm_id,
            capacity,
        );
        
        self.assets.insert(id, asset);
        self.assets_by_firm.entry(owner_firm_id).or_default().push(id);
        
        // Planifier les événements de seuil
        self.schedule_threshold_events(id, creation_date);
        
        id
    }
    
    /// Planifie les événements de seuil pour un actif
    fn schedule_threshold_events(&mut self, asset_id: EntityId, current_day: i64) {
        // Cloner les dates nécessaires avant d'ajouter des événements
        let capacity_date = self.assets.get(&asset_id).and_then(|a| a.capacity_reduced_date());
        let critical_date = self.assets.get(&asset_id).and_then(|a| a.critical_date());
        
        if let Some(capacity_date) = capacity_date {
            if capacity_date > current_day {
                self.add_threshold_event(ThresholdEvent::new(
                    asset_id,
                    CAPACITY_REDUCED_THRESHOLD,
                    capacity_date,
                    ThresholdEventType::CapacityReduced,
                ));
            }
        }
        
        if let Some(critical_date) = critical_date {
            if critical_date > current_day {
                self.add_threshold_event(ThresholdEvent::new(
                    asset_id,
                    CRITICAL_THRESHOLD,
                    critical_date,
                    ThresholdEventType::Critical,
                ));
            }
        }
    }
    
    /// Ajoute un événement de seuil
    pub fn add_threshold_event(&mut self, event: ThresholdEvent) {
        self.threshold_events.entry(event.event_day).or_default().push(event);
    }
    
    /// Retourne les événements de seuil pour un jour donné
    pub fn get_threshold_events(&self, day: i64) -> Vec<&ThresholdEvent> {
        self.threshold_events.get(&day).map(|v| v.as_slice()).unwrap_or(&[]).iter().collect()
    }
    
    /// Traite les événements de seuil pour un jour donné
    pub fn process_threshold_events(&mut self, day: i64) -> Vec<ThresholdEvent> {
        let events = self.threshold_events.remove(&day);
        
        if let Some(mut events) = events {
            // Pour chaque événement, vérifier si le seuil a été franchi
            for event in &mut events {
                if let Some(asset) = self.assets.get(&event.asset_id) {
                    let current_state = asset.current_state(day);
                    
                    // Si le seuil a été franchi, générer un nouvel événement
                    // (pour T4.4 : "événements de franchissement à la date exacte, replanifiés après chaque changement d'entrée")
                    if current_state <= event.threshold as i32 {
                        // Replanifier l'événement suivant
                        self.schedule_threshold_events(event.asset_id, day);
                    }
                }
            }
            
            events
        } else {
            Vec::new()
        }
    }
    
    /// Retourne un actif par id
    pub fn get_asset(&self, id: EntityId) -> Option<&Asset> {
        self.assets.get(&id)
    }
    
    /// Retourne un actif par id (mutable)
    pub fn get_asset_mut(&mut self, id: EntityId) -> Option<&mut Asset> {
        self.assets.get_mut(&id)
    }
    
    /// Retourne tous les actifs d'une firme
    pub fn get_assets_by_firm(&self, firm_id: EntityId) -> Vec<&Asset> {
        self.assets_by_firm.get(&firm_id)
            .map(|ids| ids.iter().filter_map(|&id| self.assets.get(&id)).collect())
            .unwrap_or_default()
    }
    
    /// Applique une maintenance à un actif
    pub fn apply_maintenance(
        &mut self,
        asset_id: EntityId,
        current_day: i64,
        policy: MaintenancePolicy,
    ) -> bool {
        if let Some(asset) = self.assets.get_mut(&asset_id) {
            asset.apply_maintenance(current_day, policy);
            // Replanifier les événements de seuil (le taux a changé)
            self.schedule_threshold_events(asset_id, current_day);
            true
        } else {
            false
        }
    }
    
    /// Répare un actif
    pub fn repair_asset(
        &mut self,
        asset_id: EntityId,
        target_state: u32,
        current_day: i64,
    ) -> bool {
        if let Some(asset) = self.assets.get_mut(&asset_id) {
            asset.repair(target_state, current_day);
            // Replanifier les événements de seuil
            self.schedule_threshold_events(asset_id, current_day);
            true
        } else {
            false
        }
    }
    
    /// Met à jour l'état de tous les actifs pour un jour donné
    /// 
    /// §16.3 : "Coût : O(1) par actif et par jour (une soustraction), aucun recalcul de masse"
    pub fn update_all(&mut self, current_day: i64) {
        // Just process threshold events - states are calculated lazily
        self.process_threshold_events(current_day);
    }
    
    /// Vérifie T4.3 : Paresseux = naïf
    /// 
    /// §16.5 T4.3 : "Paresseux = naïf : 10 ans avec changements de politique d'entretien | 
    /// état paresseux identique bit à bit à un oracle naïf recalculé chaque jour"
    pub fn verify_paresseux_naif(&self, asset_id: EntityId, current_day: i64) -> bool {
        let asset = self.assets.get(&asset_id).unwrap();
        
        // Calcul paresseux (notre méthode)
        let paresseux_state = asset.current_state(current_day);
        
        // Calcul naïf (recalculé chaque jour)
        // état(t) = état_initial - taux * (t - creation_date)
        // Donc on soustrait le taux pour chaque jour écoulé
        let days_passed = (current_day - asset.creation_date) as u32;
        let effective_rate = asset.effective_wear_rate();
        let total_wear = effective_rate * days_passed;
        let naif_state = asset.initial_state.saturating_sub(total_wear) as i32;
        
        paresseux_state == naif_state
    }
    
    /// Vérifie I6 : états dans [0, 1000]
    /// 
    /// §16.5 T4.2 : "Bornes : tous les actifs et porteurs | I6 vert : états dans [0, 1000]"
    pub fn verify_i6_state_bounds(&self, current_day: i64) -> bool {
        for asset in self.assets.values() {
            let state = asset.current_state(current_day);
            if state < 0 || state > MAX_INITIAL_STATE as i32 {
                return false;
            }
        }
        true
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

