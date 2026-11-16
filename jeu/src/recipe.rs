//! Vestiges Horizons - Tranche 4 : Recettes et Production
//! 
//! Implémentation des recettes de production selon Docs/spec P1 - quartier vivant.md §16.2
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Qualités possibles pour les biens produits
pub const NUM_QUALITIES: u32 = 3;

// ============================================================================
// QUALITÉ
// ============================================================================

/// Niveau de qualité d'un bien ou d'une production
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Quality {
    /// Qualité de base / médiocre
    Base = 0,
    /// Qualité standard / moyenne
    Standard = 1,
    /// Qualité supérieure / premium
    Premium = 2,
}

impl Quality {
    /// Retourne la description
    pub fn description(&self) -> &'static str {
        match self {
            Quality::Base => "Base",
            Quality::Standard => "Standard",
            Quality::Premium => "Premium",
        }
    }
    
    /// Retourne le coefficient de rendement (multiplicateur de production)
    pub fn yield_multiplier(&self) -> f32 {
        match self {
            Quality::Base => 0.8,
            Quality::Standard => 1.0,
            Quality::Premium => 1.2,
        }
    }
    
    /// Retourne le coefficient de coût (multiplicateur de coût en intrants)
    pub fn cost_multiplier(&self) -> f32 {
        match self {
            Quality::Base => 0.8,
            Quality::Standard => 1.0,
            Quality::Premium => 1.5,
        }
    }
}

// ============================================================================
// RECETTE
// ============================================================================

/// Recette de production
/// 
/// §16.2 : "Une recette est en données : (intrants, heures par poste, production, qualité)"
/// Les intrants sont un import (bouchon à prix fixe en tranche 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Identifiant unique
    pub id: u64,
    /// Nom de la recette
    pub name: String,
    /// Type de recette (production, service, etc.)
    pub recipe_type: RecipeType,
    /// Intrants nécessaires (bien_id -> quantité)
    pub inputs: BTreeMap<u64, u32>,
    /// Heures de travail par poste (en demi-heures)
    pub labor_hours_per_post: u32,
    /// Nombre de postes requis
    pub num_posts: u32,
    /// Production (bien_id -> quantité)
    pub output: (u64, u32), // (good_id, quantity)
    /// Qualité de la production
    pub quality: Quality,
    /// Niveau de maîtrise requis (0-10)
    pub required_mastery: u8,
    /// Coefficient de rendement de base
    pub base_yield: f32,
}

impl Recipe {
    pub fn new(
        id: u64,
        name: String,
        recipe_type: RecipeType,
        inputs: BTreeMap<u64, u32>,
        labor_hours_per_post: u32,
        num_posts: u32,
        output_good_id: u64,
        output_quantity: u32,
        quality: Quality,
        required_mastery: u8,
    ) -> Self {
        Recipe {
            id,
            name,
            recipe_type,
            inputs,
            labor_hours_per_post,
            num_posts,
            output: (output_good_id, output_quantity),
            quality,
            required_mastery,
            base_yield: 1.0,
        }
    }
    
    /// Retourne le total des heures de travail nécessaires
    pub fn total_labor_hours(&self) -> u32 {
        self.labor_hours_per_post * self.num_posts
    }
    
    /// Retourne la quantité de production avec un niveau de maîtrise donné
    /// 
    /// §16.2 : "Le niveau de maîtrise des porteurs module le rendement (coefficients en données)"
    pub fn get_yield(&self, mastery_level: u8) -> f32 {
        // Le rendement est basé sur la qualité et le niveau de maîtrise
        // mastery_level : 0-10
        // Coefficient : 1 + (mastery_level / 100.0) * mastery_bonus
        // Pour l'instant, bonus linéaire de 0-10%
        let mastery_bonus = mastery_level as f32 / 100.0;
        self.base_yield * self.quality.yield_multiplier() * (1.0 + mastery_bonus)
    }
    
    /// Retourne si la recette peut être exécutée avec les stocks donnés
    pub fn can_produce(&self, input_stock: &BTreeMap<u64, u32>) -> bool {
        for (good_id, &required) in &self.inputs {
            let available = input_stock.get(good_id).copied().unwrap_or(0);
            if available < required {
                return false;
            }
        }
        true
    }
    
    /// Retourne la quantité maximale produisable avec les stocks donnés
    pub fn max_producible(&self, input_stock: &BTreeMap<u64, u32>) -> u32 {
        let mut max_prod = u32::MAX;
        
        for (good_id, &required) in &self.inputs {
            let available = input_stock.get(good_id).copied().unwrap_or(0);
            // Combien de fois peut-on produire avec cet intrant ?
            let prod_for_this_input = available / required;
            max_prod = max_prod.min(prod_for_this_input);
        }
        
        max_prod
    }
    
    /// Retourne les intrants nécessaires pour une production donnée
    pub fn inputs_for_production(&self, quantity: u32) -> BTreeMap<u64, u32> {
        let mut result = BTreeMap::new();
        for (good_id, &required) in &self.inputs {
            result.insert(*good_id, required * quantity);
        }
        result
    }
}

// ============================================================================
// TYPE DE RECETTE
// ============================================================================

/// Type de recette
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RecipeType {
    /// Production de nourriture
    FoodProduction = 0,
    /// Production de biens manufacturés
    Manufacturing = 1,
    /// Service (restauration, etc.)
    Service = 2,
    /// Entretien / Réparation
    Maintenance = 3,
    /// Production domestique (maisonnée)
    Domestic = 4,
}

impl RecipeType {
    pub fn description(&self) -> &'static str {
        match self {
            RecipeType::FoodProduction => "Production alimentaire",
            RecipeType::Manufacturing => "Fabrication",
            RecipeType::Service => "Service",
            RecipeType::Maintenance => "Entretien",
            RecipeType::Domestic => "Domestique",
        }
    }
}

// ============================================================================
// GESTIONNAIRE DE RECETTES
// ============================================================================

/// Gestionnaire des recettes
/// 
/// Stocke les recettes disponibles et permet de les rechercher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeManager {
    /// Recettes par id
    recipes: BTreeMap<u64, Recipe>,
    /// Recettes par type
    recipes_by_type: BTreeMap<RecipeType, Vec<u64>>,
    /// Compteur d'ids
    next_recipe_id: u64,
}

impl RecipeManager {
    pub fn new() -> Self {
        RecipeManager {
            recipes: BTreeMap::new(),
            recipes_by_type: BTreeMap::new(),
            next_recipe_id: 0,
        }
    }
    
    /// Crée une nouvelle recette
    pub fn create_recipe(
        &mut self,
        name: String,
        recipe_type: RecipeType,
        inputs: BTreeMap<u64, u32>,
        labor_hours_per_post: u32,
        num_posts: u32,
        output_good_id: u64,
        output_quantity: u32,
        quality: Quality,
        required_mastery: u8,
    ) -> u64 {
        let id = self.next_recipe_id;
        self.next_recipe_id += 1;
        
        let recipe = Recipe::new(
            id,
            name,
            recipe_type,
            inputs,
            labor_hours_per_post,
            num_posts,
            output_good_id,
            output_quantity,
            quality,
            required_mastery,
        );
        
        self.recipes.insert(id, recipe);
        self.recipes_by_type.entry(recipe_type).or_default().push(id);
        
        id
    }
    
    /// Retourne une recette par id
    pub fn get_recipe(&self, id: u64) -> Option<&Recipe> {
        self.recipes.get(&id)
    }
    
    /// Retourne toutes les recettes d'un type donné
    pub fn get_recipes_by_type(&self, recipe_type: RecipeType) -> Vec<&Recipe> {
        self.recipes_by_type.get(&recipe_type)
            .map(|ids| ids.iter().filter_map(|&id| self.recipes.get(&id)).collect())
            .unwrap_or_default()
    }
    
    /// Retourne toutes les recettes
    pub fn get_all_recipes(&self) -> Vec<&Recipe> {
        self.recipes.values().collect()
    }
    
    /// Vérifie si une recette existe
    pub fn has_recipe(&self, id: u64) -> bool {
        self.recipes.contains_key(&id)
    }
    
    /// Charge des recettes par défaut pour la Tranche 4
    pub fn load_default_recipes(&mut self) {
        // Recette 1 : Production de pain (base)
        let mut inputs = BTreeMap::new();
        inputs.insert(1, 10); // 10 unités de farine
        inputs.insert(2, 5);  // 5 unités d'eau
        self.create_recipe(
            "Pain de base".to_string(),
            RecipeType::FoodProduction,
            inputs,
            4,  // 2 heures de travail (4 demi-heures)
            1,  // 1 poste
            3,  // Pain (bien_id = 3)
            20, // 20 unités de pain
            Quality::Base,
            0,  // Aucune maîtrise requise
        );
        
        // Recette 2 : Production de pain standard
        let mut inputs = BTreeMap::new();
        inputs.insert(1, 15); // 15 unités de farine
        inputs.insert(2, 8);  // 8 unités d'eau
        inputs.insert(4, 2); // 2 unités de beurre
        self.create_recipe(
            "Pain standard".to_string(),
            RecipeType::FoodProduction,
            inputs,
            6,  // 3 heures de travail
            1,
            5,  // Pain standard
            25,
            Quality::Standard,
            3,  // Niveau 3 de maîtrise requis
        );
        
        // Recette 3 : Service de restauration
        let mut inputs = BTreeMap::new();
        inputs.insert(3, 10); // 10 pains
        inputs.insert(5, 5);  // 5 légumes
        inputs.insert(6, 3);  // 3 viandes
        self.create_recipe(
            "Repas complet".to_string(),
            RecipeType::Service,
            inputs,
            8,  // 4 heures de travail
            2,  // 2 postes (cuisinier + serveur)
            7,  // Repas servi
            1,   // 1 repas par production
            Quality::Standard,
            5,  // Niveau 5 de maîtrise
        );
    }
}

impl Default for RecipeManager {
    fn default() -> Self {
        let mut manager = RecipeManager::new();
        manager.load_default_recipes();
        manager
    }
}

// ============================================================================
// PRODUCTION
// ============================================================================

/// Résultat d'une production
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionResult {
    /// Recette utilisée
    pub recipe_id: u64,
    /// Quantité produite
    pub quantity_produced: u32,
    /// Facteur contraignant (capacité, heures, intrants)
    pub limiting_factor: LimitingFactor,
    /// Message de journalisation
    pub log_message: String,
}

/// Facteur contraignant pour la production
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LimitingFactor {
    /// Pas de facteur contraignant (production complète)
    None,
    /// Limité par la capacité des actifs (valeur = capacité disponible)
    AssetCapacity(u32),
    /// Limité par le rendement de la recette (valeur = yield en %, 0-1000)
    RecipeYield(u32),
    /// Limité par les intrants disponibles
    Inputs,
    /// Limité par les heures de travail disponibles
    LaborHours,
    /// Limité par le nombre de postes pourvus
    StaffedPosts,
    /// Manque de main d'œuvre
    InsufficientLabor,
    /// Actif manquant
    MissingAsset,
}

impl LimitingFactor {
    /// Retourne la sévérité du facteur limitant (plus c'est bas, plus c'est limitant)
    pub fn severity(&self) -> f32 {
        match self {
            LimitingFactor::None => 1.0,
            LimitingFactor::AssetCapacity(cap) => *cap as f32 / 1000.0,
            LimitingFactor::RecipeYield(yield_percent) => *yield_percent as f32 / 1000.0,
            LimitingFactor::Inputs => 0.5,
            LimitingFactor::LaborHours => 0.6,
            LimitingFactor::StaffedPosts => 0.7,
            LimitingFactor::InsufficientLabor => 0.8,
            LimitingFactor::MissingAsset => 0.0,
        }
    }
    
    /// Retourne la valeur numérique associée
    pub fn value(&self) -> u32 {
        match self {
            LimitingFactor::None => 0,
            LimitingFactor::AssetCapacity(cap) => *cap,
            LimitingFactor::RecipeYield(yield_percent) => *yield_percent,
            LimitingFactor::Inputs => 0,
            LimitingFactor::LaborHours => 0,
            LimitingFactor::StaffedPosts => 0,
            LimitingFactor::InsufficientLabor => 0,
            LimitingFactor::MissingAsset => 0,
        }
    }
}

impl std::fmt::Display for LimitingFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LimitingFactor::None => write!(f, "Aucun"),
            LimitingFactor::AssetCapacity(cap) => write!(f, "Capacité des actifs ({})", cap),
            LimitingFactor::RecipeYield(yield_percent) => write!(f, "Rendement ({:.1}%)", *yield_percent as f32 / 10.0),
            LimitingFactor::Inputs => write!(f, "Intrants"),
            LimitingFactor::LaborHours => write!(f, "Heures de travail"),
            LimitingFactor::StaffedPosts => write!(f, "Postes pourvus"),
            LimitingFactor::InsufficientLabor => write!(f, "Manque de main d'œuvre"),
            LimitingFactor::MissingAsset => write!(f, "Actif manquant"),
        }
    }
}

/// Gestionnaire de production (basé sur les recettes)
/// 
/// §16.2 : "Exécution quotidienne en phase 1 : production = min(capacité des actifs, heures disponibles, intrants disponibles)"
/// §16.5 T4.5 : "Facteur contraignant : ateliers bridés tour à tour par capacité, heures, intrants"
pub struct RecipeProductionManager {
    /// Recettes disponibles
    recipe_manager: RecipeManager,
}

impl RecipeProductionManager {
    pub fn new() -> Self {
        RecipeProductionManager {
            recipe_manager: RecipeManager::default(),
        }
    }
    
    /// Exécute la production pour une firme et une recette
    /// 
    /// §16.2 : "production = min(capacité des actifs, heures disponibles, intrants disponibles)"
    /// Chaque facteur contraignant est identifiable au journal.
    /// Les intrants sont consommés avant production, jamais de stock négatif (I2).
    pub fn produce(
        &self,
        firm: &mut crate::firm::Firm,
        recipe_id: u64,
        mastery_levels: &BTreeMap<u64, u8>, // (post_id -> mastery_level)
    ) -> Option<ProductionResult> {
        let recipe = self.recipe_manager.get_recipe(recipe_id)?;
        
        // 1. Vérifier la recette
        if !firm.recipe_ids.contains(&recipe_id) {
            return None;
        }
        
        // 2. Calculer la production maximale pour chaque facteur
        let max_by_inputs = recipe.max_producible(&firm.input_stock);
        
        // 3. Calculer les heures disponibles
        let total_hours_available = firm.total_daily_hours();
        let max_by_labor = recipe.total_labor_hours().min(total_hours_available) / recipe.labor_hours_per_post;
        
        // 4. Calculer les postes pourvus
        let staffed_posts = firm.employee_count() as u32;
        let max_by_staff = recipe.num_posts.min(staffed_posts);
        
        // 5. Capacité des actifs (simplifiée : on suppose que les actifs ont une capacité illimitée pour l'instant)
        // En T4, c'est un bouchon - la capacité des actifs sera gérée en détail plus tard
        let max_by_assets = u32::MAX; // Bouchon : capacité illimitée
        
        // 6. Trouver le facteur contraignant
        let limiting_factor = if max_by_inputs <= max_by_labor && max_by_inputs <= max_by_staff {
            LimitingFactor::Inputs
        } else if max_by_labor <= max_by_inputs && max_by_labor <= max_by_staff {
            LimitingFactor::LaborHours
        } else if max_by_staff <= max_by_inputs && max_by_staff <= max_by_labor {
            LimitingFactor::StaffedPosts
        } else {
            LimitingFactor::None
        };
        
        // 7. Quantité produite = min de tous les facteurs
        let quantity = max_by_inputs.min(max_by_labor).min(max_by_staff);
        
        if quantity == 0 {
            // Ne rien produire
            return Some(ProductionResult {
                recipe_id,
                quantity_produced: 0,
                limiting_factor,
                log_message: format!("Production annulée : facteur contraignant = {}", limiting_factor),
            });
        }
        
        // 8. Consommer les intrants
        let inputs_needed = recipe.inputs_for_production(quantity);
        for (good_id, amount) in inputs_needed {
            firm.consume_input_stock(good_id, amount);
        }
        
        // 9. Calculer la quantité finale avec rendement
        let recipe_yield = recipe.get_yield(*mastery_levels.get(&recipe_id).unwrap_or(&0));
        let final_quantity = (quantity as f32 * recipe_yield) as u32;
        
        // 10. Ajouter les produits
        firm.add_output_stock(recipe.output.0, final_quantity);
        
        Some(ProductionResult {
            recipe_id,
            quantity_produced: final_quantity,
            limiting_factor,
            log_message: format!(
                "Produit {} unités avec recette {} (facteur contraignant : {})",
                final_quantity, recipe_id, limiting_factor
            ),
        })
    }
    
    /// Exécute la production pour toutes les recettes d'une firme
    pub fn produce_all(
        &self,
        firm: &mut crate::firm::Firm,
        mastery_levels: &BTreeMap<u64, u8>,
    ) -> Vec<ProductionResult> {
        let mut results = Vec::new();
        
        // Cloner les recipe_ids pour éviter le borrow conflict
        let recipe_ids = firm.recipe_ids.clone();
        for &recipe_id in &recipe_ids {
            if let Some(result) = self.produce(firm, recipe_id, mastery_levels) {
                results.push(result);
            }
        }
        
        results
    }
    
    /// Retourne le gestionnaire de recettes
    pub fn recipe_manager(&self) -> &RecipeManager {
        &self.recipe_manager
    }
    
    /// Retourne le gestionnaire de recettes (mutable)
    pub fn recipe_manager_mut(&mut self) -> &mut RecipeManager {
        &mut self.recipe_manager
    }
}

impl Default for RecipeProductionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

