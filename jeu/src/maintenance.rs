//! Vestiges Horizons - Tranche 4 : Entretien
//! 
//! Implémentation de l'entretien des actifs selon Docs/spec P1 - quartier vivant.md §16.3-16.4
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::asset::{Asset, AssetManager, AssetType};
use crate::common::MaintenancePolicy;
use crate::entities::EntityId;
use crate::firm::FirmManager;

// ============================================================================
// PLAN D'ENTRETIEN
// ============================================================================

/// Plan d'entretien pour un actif
/// 
/// §16.4 : "Entretien : déclaration mensuelle, choix de politique pour l'actif (intensif, normal, léger, abandonné)"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenancePlan {
    pub asset_id: EntityId,
    pub firm_id: EntityId,
    pub policy: MaintenancePolicy,
    /// Coût mensuel de l'entretien
    pub monthly_cost: u64,
    /// Jour de la dernière déclaration
    pub last_declaration_day: i64,
}

impl MaintenancePlan {
    pub fn new(
        asset_id: EntityId,
        firm_id: EntityId,
        policy: MaintenancePolicy,
        monthly_cost: u64,
        current_day: i64,
    ) -> Self {
        MaintenancePlan {
            asset_id,
            firm_id,
            policy,
            monthly_cost,
            last_declaration_day: current_day,
        }
    }
}

// ============================================================================
// GESTIONNAIRE D'ENTRETIEN
// ============================================================================

/// Gestionnaire de l'entretien des actifs
/// 
/// §16.4 : " Entretien : déclaration mensuelle, choix de politique pour l'actif 
/// (intensif, normal, léger, abandonné) réduisant la usure (70%, 100%, 140%, 200%)"
pub struct MaintenanceManager {
    /// Plans d'entretien par actif
    plans: BTreeMap<EntityId, MaintenancePlan>,
    /// Coût total de l'entretien par firme
    costs_by_firm: BTreeMap<EntityId, u64>,
}

impl MaintenanceManager {
    pub fn new() -> Self {
        MaintenanceManager {
            plans: BTreeMap::new(),
            costs_by_firm: BTreeMap::new(),
        }
    }
    
    /// Crée un plan d'entretien pour un actif
    pub fn create_plan(
        &mut self,
        asset_id: EntityId,
        firm_id: EntityId,
        policy: MaintenancePolicy,
        monthly_cost: u64,
        current_day: i64,
    ) -> EntityId {
        let plan = MaintenancePlan::new(
            asset_id,
            firm_id,
            policy,
            monthly_cost,
            current_day,
        );
        
        self.plans.insert(asset_id, plan);
        
        // Mettre à jour les coûts par firme
        *self.costs_by_firm.entry(firm_id).or_default() += monthly_cost;
        
        asset_id
    }
    
    /// Met à jour la politique d'entretien pour un actif
    pub fn update_policy(
        &mut self,
        asset_id: EntityId,
        new_policy: MaintenancePolicy,
        new_monthly_cost: u64,
        current_day: i64,
    ) -> bool {
        if let Some(plan) = self.plans.get_mut(&asset_id) {
            // Réduire le coût de l'ancienne politique
            *self.costs_by_firm.entry(plan.firm_id).or_default() -= plan.monthly_cost;
            
            plan.policy = new_policy;
            plan.monthly_cost = new_monthly_cost;
            plan.last_declaration_day = current_day;
            
            // Ajouter le coût de la nouvelle politique
            *self.costs_by_firm.entry(plan.firm_id).or_default() += new_monthly_cost;
            
            true
        } else {
            false
        }
    }
    
    /// Déclare l'entretien mensuel pour une firme
    /// 
    /// §16.4 : "Entretien : déclaration mensuelle"
    pub fn monthly_declaration(&mut self, firm_id: EntityId, current_day: i64) {
        // Pour tous les actifs de la firme, vérifier s'il faut mettre à jour la politique
        // (par défaut, on garde la politique actuelle)
        
        // Mettre à jour la date de déclaration pour tous les actifs de cette firme
        for plan in self.plans.values_mut() {
            if plan.firm_id == firm_id {
                plan.last_declaration_day = current_day;
            }
        }
    }
    
    /// Applique l'entretien à un actif pour un jour donné
    pub fn apply_maintenance(&self, asset_id: EntityId) -> MaintenancePolicy {
        self.plans.get(&asset_id)
            .map(|p| p.policy)
            .unwrap_or(MaintenancePolicy::Normal)
    }
    
    /// Retourne le coût total d'entretien pour une firme
    pub fn get_total_cost(&self, firm_id: EntityId) -> u64 {
        *self.costs_by_firm.get(&firm_id).unwrap_or(&0)
    }
    
    /// Traite l'entretien pour tous les actifs d'une firme pour un jour donné
    pub fn process_firm_maintenance(
        &self,
        firm_id: EntityId,
        asset_manager: &mut AssetManager,
        current_day: i64,
    ) {
        // Pour chaque actif de la firme, appliquer la politique d'entretien
        let asset_ids: Vec<EntityId> = asset_manager.get_assets_by_firm(firm_id)
            .iter().map(|a| a.id).collect();
        
        for asset_id in asset_ids {
            let policy = self.apply_maintenance(asset_id);
            asset_manager.apply_maintenance(asset_id, current_day, policy);
        }
    }
    
    /// Optimise les politiques d'entretien pour une firme
    /// 
    /// §16.4 : "P1.19-21 Entretien (3 à implémenter : maintenance paresseuse + usure)"
    /// 
    /// Stratégie : 
    /// - Si l'actif est proche du seuil critique : politique intensif
    /// - Si l'actif est proche du seuil de capacité réduite : politique normal
    /// - Sinon : politique léger
    pub fn optimize_policies(
        &mut self,
        firm_id: EntityId,
        asset_manager: &AssetManager,
        current_day: i64,
    ) {
        for asset in asset_manager.get_assets_by_firm(firm_id) {
            let state = asset.current_state(current_day);
            
            let new_policy = if state <= crate::asset::CRITICAL_THRESHOLD as i32 {
                MaintenancePolicy::Intensive
            } else if state <= crate::asset::CAPACITY_REDUCED_THRESHOLD as i32 {
                MaintenancePolicy::Normal
            } else {
                MaintenancePolicy::Light
            };
            
            if let Some(plan) = self.plans.get_mut(&asset.id) {
                if plan.firm_id == firm_id {
                    // Mettre à jour la politique
                    let old_cost = plan.monthly_cost;
                    let new_cost = calculate_maintenance_cost(&asset, new_policy);
                    
                    *self.costs_by_firm.entry(firm_id).or_default() -= old_cost;
                    plan.policy = new_policy;
                    plan.monthly_cost = new_cost;
                    plan.last_declaration_day = current_day;
                    *self.costs_by_firm.entry(firm_id).or_default() += new_cost;
                }
            } else {
                // Créer un nouveau plan
                let monthly_cost = calculate_maintenance_cost(&asset, new_policy);
                self.create_plan(
                    asset.id,
                    firm_id,
                    new_policy,
                    monthly_cost,
                    current_day,
                );
            }
        }
    }
    
    /// Vérifie T4.5 : progéniture d'entretien
    /// 
    /// §16.5 T4.5 : "Progéniture d'entretien : entraînement actif usé, arrêt et redémarrage"
    /// Pour l'instant, on vérifie que l'on peut démarrer/arrêter l'entretien
    pub fn verify_maintenance_progeniture(
        &mut self,
        asset_manager: &mut AssetManager,
        firm_id: EntityId,
        current_day: i64,
    ) -> bool {
        // Créer un actif test
        let asset_id = EntityId::new(crate::entities::EntityType::Asset, 9999);
        let asset = Asset::new(
            asset_id,
            AssetType::Machine,
            1000,
            current_day,
            10,
            firm_id,
            100,
        );
        
        // Ajouter l'actif
        asset_manager.assets.insert(asset_id, asset);
        asset_manager.assets_by_firm.entry(firm_id).or_default().push(asset_id);
        
        // Créer un plan d'entretien
        self.create_plan(
            asset_id,
            firm_id,
            MaintenancePolicy::Normal,
            100,
            current_day,
        );
        
        // Arrêter l'entretien (politique Abandonné)
        self.update_policy(
            asset_id,
            MaintenancePolicy::Abandoned,
            0,
            current_day,
        );
        
        // Vérifier que l'actif continue de se dégrader (avec taux * 2)
        let asset = asset_manager.get_asset(asset_id).unwrap();
        let rate_with_abandoned = asset.effective_wear_rate();
        
        // Redémarrer l'entretien (politique Normal)
        self.update_policy(
            asset_id,
            MaintenancePolicy::Normal,
            100,
            current_day + 1,
        );
        
        let asset = asset_manager.get_asset(asset_id).unwrap();
        let rate_with_normal = asset.effective_wear_rate();
        
        // Avec Abandonné : taux * 200%
        // Avec Normal : taux * 100%
        // Donc rate_with_abandoned devrait être > rate_with_normal
        
        // Nettoyer
        asset_manager.assets.remove(&asset_id);
        
        rate_with_abandoned > rate_with_normal
    }
}

/// Calcule le coût mensuel d'entretien pour un actif avec une politique donnée
fn calculate_maintenance_cost(asset: &Asset, policy: MaintenancePolicy) -> u64 {
    // Coût de base basé sur le type d'actif et la capacité
    let base_cost = match asset.asset_type {
        AssetType::Machine => asset.capacity as u64 * 2,
        AssetType::Tooling => asset.capacity as u64 * 1,
        AssetType::Building => asset.capacity as u64 * 5,
        AssetType::KitchenEquipment => asset.capacity as u64 * 1,
        AssetType::Furniture => asset.capacity as u64 * 0,
        AssetType::WaterSystem => asset.capacity as u64 * 3,
    };
    
    // Modifier selon la politique (en entiers)
    match policy {
        MaintenancePolicy::Intensive => base_cost * 2,
        MaintenancePolicy::Normal => base_cost,
        MaintenancePolicy::Light => base_cost / 2,  // Division entière, acceptable pour les coûts
        MaintenancePolicy::Abandoned => 0,
    }
}

impl Default for MaintenanceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

