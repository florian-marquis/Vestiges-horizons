//! Vestiges Horizons - Tranche 4 : Production
//! 
//! Implémentation de la production selon Docs/spec P1 - quartier vivant.md §16.4
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::asset::AssetManager;

use crate::entities::EntityId;
use crate::firm::{Firm, FirmManager};
use crate::recipe::{LimitingFactor, RecipeManager};

// ============================================================================
// ORDRE DE PRODUCTION
// ============================================================================

/// Ordre de production
/// 
/// §16.4 : "Production : ordre quotidien (0, 1 ou N) entrepôts pour sortir les marchandises"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub id: EntityId,
    pub firm_id: EntityId,
    pub recipe_id: u64,
    pub quantity: u64,
    pub target_warehouse: Option<EntityId>,
    pub creation_day: i64,
    pub completion_day: Option<i64>,
    pub status: ProductionOrderStatus,
}

/// État d'un ordre de production
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProductionOrderStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Cancelled = 3,
}

impl ProductionOrder {
    pub fn new(
        id: EntityId,
        firm_id: EntityId,
        recipe_id: u64,
        quantity: u64,
        target_warehouse: Option<EntityId>,
        creation_day: i64,
    ) -> Self {
        ProductionOrder {
            id,
            firm_id,
            recipe_id,
            quantity,
            target_warehouse,
            creation_day,
            completion_day: None,
            status: ProductionOrderStatus::Pending,
        }
    }
}

// ============================================================================
// PLAN DE PRODUCTION
// ============================================================================

/// Plan de production pour une firme
/// 
/// §16.4 : "Planification : déclaration mensuelle"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionPlan {
    pub firm_id: EntityId,
    pub orders: Vec<EntityId>,
    pub declaration_day: i64,
}

// ============================================================================
// RÉSULTAT DE PRODUCTION
// ============================================================================

/// Résultat de production
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionResult {
    pub order_id: EntityId,
    pub output_quantity: u64,
    pub limiting_factor: Option<LimitingFactor>,
    pub waste: u64,
}

// ============================================================================
// GESTIONNAIRE DE PRODUCTION
// ============================================================================

/// Gestionnaire de la production
/// 
/// §16.4 : "Production : ordre quotidien (0, 1 ou N) entrepôts pour sortir les marchandises"
pub struct ProductionManager {
    /// Commandes de production
    orders: BTreeMap<EntityId, ProductionOrder>,
    /// Plans de production par firme
    plans: BTreeMap<EntityId, ProductionPlan>,
    /// Résultats de production
    results: Vec<ProductionResult>,
}

impl ProductionManager {
    pub fn new() -> Self {
        ProductionManager {
            orders: BTreeMap::new(),
            plans: BTreeMap::new(),
            results: Vec::new(),
        }
    }
    
    /// Crée un nouvel ordre de production
    pub fn create_order(
        &mut self,
        id: EntityId,
        firm_id: EntityId,
        recipe_id: u64,
        quantity: u64,
        target_warehouse: Option<EntityId>,
        creation_day: i64,
    ) -> EntityId {
        let order = ProductionOrder::new(
            id,
            firm_id,
            recipe_id,
            quantity,
            target_warehouse,
            creation_day,
        );
        
        self.orders.insert(id, order);
        id
    }
    
    /// Crée une commande avec génération automatique d'id
    pub fn create_order_auto(
        &mut self,
        registry: &mut crate::entities::EntityRegistry,
        firm_id: EntityId,
        recipe_id: u64,
        quantity: u64,
        target_warehouse: Option<EntityId>,
        creation_day: i64,
    ) -> EntityId {
        let id = registry.next_id(crate::entities::EntityType::Job);
        self.create_order(
            id,
            firm_id,
            recipe_id,
            quantity,
            target_warehouse,
            creation_day,
        )
    }
    
    /// Planifie la production mensuelle pour une firme
    /// 
    /// §16.4 : "Planification : déclaration mensuelle"
    pub fn plan_production(
        &mut self,
        firm_id: EntityId,
        orders: Vec<EntityId>,
        declaration_day: i64,
    ) {
        let plan = ProductionPlan {
            firm_id,
            orders,
            declaration_day,
        };
        
        self.plans.insert(firm_id, plan);
    }
    
    /// Traite les commandes de production pour un jour donné
    /// 
    /// §16.4 : "Production : ordre quotidien"
    pub fn process_orders(
        &mut self,
        firm_manager: &FirmManager,
        asset_manager: &AssetManager,
        recipe_manager: &RecipeManager,
        current_day: i64,
    ) -> Vec<ProductionResult> {
        let mut day_results = Vec::new();
        
        // Pour chaque firme, traiter ses commandes
        for (firm_id, firm) in firm_manager.firms() {
            // Collecter les ids des commandes pour éviter le borrow conflict
            let order_ids: Vec<EntityId> = self.orders.values()
                .filter(|o| o.firm_id == *firm_id && o.status == ProductionOrderStatus::Pending)
                .map(|o| o.id)
                .collect();
            
            for order_id in order_ids {
                // Extraire l'ordre d'abord
                let order_quantity = {
                    let order = self.orders.get(&order_id);
                    order.map(|o| o.quantity).unwrap_or(0)
                };
                
                if let Some(order) = self.orders.get(&order_id) {
                    let result = self.process_single_order(
                        order,
                        firm,
                        asset_manager,
                        recipe_manager,
                        current_day,
                    );
                    
                    if let Some(r) = result {
                        day_results.push(r.clone());
                        // Mettre à jour l'ordre
                        if let Some(order_mut) = self.orders.get_mut(&order_id) {
                            if r.output_quantity >= order_quantity {
                                order_mut.status = ProductionOrderStatus::Completed;
                                order_mut.completion_day = Some(current_day);
                            } else {
                                order_mut.status = ProductionOrderStatus::InProgress;
                            }
                        }
                    }
                }
            }
        }
        
        self.results.extend(day_results.clone());
        day_results
    }
    
    /// Traite une commande unique
    fn process_single_order(
        &self,
        order: &ProductionOrder,
        firm: &Firm,
        asset_manager: &AssetManager,
        recipe_manager: &RecipeManager,
        current_day: i64,
    ) -> Option<ProductionResult> {
        // Récupérer la recette
        let _recipe = recipe_manager.get_recipe(order.recipe_id)?;
        
        // Calculer la capacité de production disponible
        let available_capacity = self.calculate_available_capacity(
            firm.id,
            asset_manager,
            current_day,
        );
        
        // Calculer le facteur limitant principal
        // Pour l'instant, on vérifie simplement si la capacité est suffisante
        let limiting_factor = if available_capacity < order.quantity as u64 {
            Some(LimitingFactor::AssetCapacity(available_capacity as u32))
        } else {
            None
        };
        
        // Calculer la quantité réellement produite
        let output_quantity = if let Some(ref lf) = limiting_factor {
            // Appliquer la limitation
            match lf {
                LimitingFactor::None => order.quantity,
                LimitingFactor::AssetCapacity(capacity) => {
                    order.quantity.min(*capacity as u64)
                }
                LimitingFactor::RecipeYield(yield_percent) => {
                    // yield_percent est en pourmille (0-1000)
                    // Calcul entier : order.quantity * yield_percent / 1000
                    order.quantity as u64 * *yield_percent as u64 / 1000
                }
                LimitingFactor::Inputs => {
                    // Limité par les intrants
                    order.quantity
                }
                LimitingFactor::LaborHours => {
                    // Limité par les heures
                    order.quantity
                }
                LimitingFactor::StaffedPosts => {
                    // Limité par les postes
                    order.quantity
                }
                LimitingFactor::MissingAsset => {
                    0
                }
                LimitingFactor::InsufficientLabor => {
                    // Pour l'instant, on retourne 0
                    0
                }
            }
        } else {
            order.quantity
        };
        
        // Calculer les déchets
        let waste = order.quantity.saturating_sub(output_quantity);
        
        // Créer le résultat
        let result = ProductionResult {
            order_id: order.id,
            output_quantity,
            limiting_factor,
            waste,
        };
        
        Some(result)
    }
    
    /// Calcule la capacité disponible pour une firme
    fn calculate_available_capacity(
        &self,
        firm_id: EntityId,
        asset_manager: &AssetManager,
        current_day: i64,
    ) -> u64 {
        let mut total_capacity: u64 = 0;
        
        for asset in asset_manager.get_assets_by_firm(firm_id) {
            total_capacity += asset.effective_capacity(current_day) as u64;
        }
        
        total_capacity
    }
    
    /// Calcule la production quotidienne pour une firme
    /// 
    /// §16.4 : "Production : ordre quotidien (0, 1 ou N) entrepôts pour sortir les marchandises"
    pub fn daily_production(
        &mut self,
        firm_id: EntityId,
        firm_manager: &FirmManager,
        asset_manager: &AssetManager,
        recipe_manager: &RecipeManager,
        current_day: i64,
    ) -> Vec<ProductionResult> {
        // Traiter toutes les commandes de la firme pour ce jour
        let firm = match firm_manager.get_firm(firm_id) {
            Some(f) => f,
            None => return Vec::new(),
        };
        
        let order_ids: Vec<EntityId> = self.orders.values()
            .filter(|o| o.firm_id == firm_id && o.status == ProductionOrderStatus::Pending)
            .map(|o| o.id)
            .collect();
        
        let mut results = Vec::new();
        
        for order_id in order_ids {
            // Extraire l'ordre d'abord
            let order_quantity = {
                let order = self.orders.get(&order_id);
                order.map(|o| o.quantity).unwrap_or(0)
            };
            
            if let Some(order) = self.orders.get(&order_id) {
                let result = self.process_single_order(
                    order,
                    firm,
                    asset_manager,
                    recipe_manager,
                    current_day,
                );
                
                if let Some(r) = result {
                    // Mettre à jour l'ordre d'abord
                    if let Some(order_mut) = self.orders.get_mut(&order_id) {
                        if r.output_quantity >= order_quantity {
                            order_mut.status = ProductionOrderStatus::Completed;
                            order_mut.completion_day = Some(current_day);
                        } else {
                            order_mut.status = ProductionOrderStatus::InProgress;
                        }
                    }
                    results.push(r);
                }
            }
        }
        
        results
    }
    
    /// Annule une commande
    pub fn cancel_order(&mut self, order_id: EntityId) -> bool {
        if let Some(order) = self.orders.get_mut(&order_id) {
            order.status = ProductionOrderStatus::Cancelled;
            true
        } else {
            false
        }
    }
}

impl Default for ProductionManager {
    fn default() -> Self {
        Self::new()
    }
}