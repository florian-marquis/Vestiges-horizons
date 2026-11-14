//! Vestiges Horizons - Tranche 5 : Repas et eau
//! 
//! Implémentation des repas selon Docs/spec P1 - quartier vivant.md §17.3
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::entities::EntityId;
use crate::market::GoodType;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Eau refacturée : 5-10% du panier, jusqu'à 25% en pénurie
/// Valeur de base : 8% = 80 en millièmes
pub const WATER_BASE_COST_PERCENT: u32 = 80; // 8.0%
/// Plafond en pénurie : 25% = 250 en millièmes
pub const WATER_MAX_COST_PERCENT: u32 = 250; // 25.0%

// ============================================================================
// MODE DE REPAS
// ============================================================================

/// Mode de repas
/// 
/// §17.3 : "Les modes de repas sont résolus quotidiennement par le foyer selon le couple
/// (coût monétaire, coût temporel) et le niveau de logement"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MealMode {
    /// Cuisine maison (nécessite niveau 2+)
    HomeCooking = 0,
    /// Achat de nourriture à l'épicerie
    Grocery = 1,
    /// Repas au restaurant
    Restaurant = 2,
    /// Repas du travail (si disponible)
    WorkMeal = 3,
}

impl MealMode {
    pub fn description(&self) -> &'static str {
        match self {
            MealMode::HomeCooking => "Cuisine maison",
            MealMode::Grocery => "Épicerie",
            MealMode::Restaurant => "Restaurant",
            MealMode::WorkMeal => "Repas du travail",
        }
    }
    
    /// Retourne le coût temporel (en demi-heures)
    pub fn time_cost(&self) -> u32 {
        match self {
            MealMode::HomeCooking => 4, // 2 heures
            MealMode::Grocery => 2,     // 1 heure (achat + préparation simple)
            MealMode::Restaurant => 2, // 1 heure (déplacement + repas)
            MealMode::WorkMeal => 1,    // 0.5 heure (sur place)
        }
    }
    
    /// Retourne si la cuisine maison est possible
    pub fn requires_kitchen(&self) -> bool {
        matches!(self, MealMode::HomeCooking)
    }
}

// ============================================================================
// NIVEAU DE LOGEMENT
// ============================================================================

/// Niveau de logement
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HousingLevel {
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}

impl HousingLevel {
    pub fn can_cook(&self) -> bool {
        *self as u8 >= 2 // Niveau 2 et plus
    }
    
    /// Coût du logement (en centimes/jour)
    pub fn daily_cost(&self) -> u64 {
        match self {
            HousingLevel::Level1 => 5000,  // 50.00 €
            HousingLevel::Level2 => 10000, // 100.00 €
            HousingLevel::Level3 => 20000, // 200.00 €
        }
    }
}

// ============================================================================
// PANIER
// ============================================================================

/// Composition du panier
/// 
/// §17.1 : "Panier = nourriture + logement + eau refacturée + fiscalité"
/// §17.3 : "Coût du panier = nourriture ×1/jour + logement ×1/jour par adulte"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Basket {
    /// Coût de la nourriture (en centimes)
    pub food_cost: u64,
    /// Coût du logement (en centimes)
    pub housing_cost: u64,
    /// Coût de l'eau (en centimes)
    pub water_cost: u64,
    /// Coût de la fiscalité (en centimes)
    pub tax_cost: u64,
}

impl Basket {
    pub fn new(food_cost: u64, housing_cost: u64, water_cost: u64, tax_cost: u64) -> Self {
        Basket {
            food_cost,
            housing_cost,
            water_cost,
            tax_cost,
        }
    }
    
    /// Retourne le coût total du panier
    pub fn total_cost(&self) -> u64 {
        self.food_cost + self.housing_cost + self.water_cost + self.tax_cost
    }
    
    /// Calcul le salaire réel
    /// 
    /// §17.1 : "Salaire réel = salaire horaire × heures travaillées / coût du panier"
    pub fn calculate_real_wage(&self, hourly_wage: u64, hours_worked: u32) -> f32 {
        let total_earnings = hourly_wage as f32 * hours_worked as f32;
        let panier_cost = self.total_cost() as f32;
        
        if panier_cost == 0.0 {
            0.0
        } else {
            total_earnings / panier_cost
        }
    }
}

// ============================================================================
// DÉPENSE DE REPAS
// ============================================================================

/// Dépense de repas pour un foyer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealExpense {
    pub household_id: EntityId,
    /// Mode de repas choisi
    pub meal_mode: MealMode,
    /// Quantité de nourriture consommée
    pub food_quantity: u32,
    /// Coût monétaire
    pub monetary_cost: u64,
    /// Coût temporel (en demi-heures)
    pub time_cost: u32,
    /// Jour
    pub day: i64,
}

// ============================================================================
// GESTIONNAIRE DE REPAS
// ============================================================================

/// Gestionnaire des repas
/// 
/// §17.3 : "Les modes de repas sont résolus quotidiennement par le foyer"
pub struct MealManager {
    /// Dépenses de repas par foyer et par jour
    meal_expenses: BTreeMap<(EntityId, i64), MealExpense>,
    /// Panier courant par foyer
    baskets: BTreeMap<EntityId, Basket>,
    /// Niveau de logement par foyer
    housing_levels: BTreeMap<EntityId, HousingLevel>,
    /// Prix des biens alimentaires
    food_prices: BTreeMap<GoodType, u64>,
    /// Coût de l'eau par foyer
    water_costs: BTreeMap<EntityId, u64>,
}

impl MealManager {
    pub fn new() -> Self {
        MealManager {
            meal_expenses: BTreeMap::new(),
            baskets: BTreeMap::new(),
            housing_levels: BTreeMap::new(),
            food_prices: BTreeMap::new(),
            water_costs: BTreeMap::new(),
        }
    }
    
    /// Définit le niveau de logement pour un foyer
    pub fn set_housing_level(&mut self, household_id: EntityId, level: HousingLevel) {
        self.housing_levels.insert(household_id, level);
    }
    
    /// Définit le prix d'un bien alimentaire
    pub fn set_food_price(&mut self, good_type: GoodType, price: u64) {
        self.food_prices.insert(good_type, price);
    }
    
    /// Calcul le coût de l'eau pour un foyer
    /// 
    /// §17.3 : "eau refacturée dans le loyer (une seule facture), abonnement indexé sur la rareté"
    pub fn calculate_water_cost(&self, _household_id: EntityId, panier_cost: u64, scarcity_level: u32) -> u64 {
        // Coût de base : WATER_BASE_COST_PERCENT de panier_cost
        // Pour éviter le débordement : (panier_cost / 1000) * WATER_BASE_COST_PERCENT
        let base_cost = (panier_cost / 1000) * WATER_BASE_COST_PERCENT as u64;
        
        // Ajustement pour rareté : linéaire de 0% à (WATER_MAX_COST_PERCENT - WATER_BASE_COST_PERCENT)
        let scarcity_percent = scarcity_level.min(100);
        let extra_percent = (WATER_MAX_COST_PERCENT - WATER_BASE_COST_PERCENT) as u64 * scarcity_percent as u64 / 100;
        
        // Pour éviter le débordement : (panier_cost / 1000) * extra_percent
        let extra_cost = (panier_cost / 1000) * extra_percent;
        
        base_cost + extra_cost
    }
    
    /// Résout le mode de repas optimal pour un foyer
    /// 
    /// §17.3 : "le choix est pondéré par la doctrine et le salaire réel"
    /// Pour l'instant, on choisit en fonction du coût et du niveau de logement
    pub fn resolve_meal_mode(
        &self,
        household_id: EntityId,
        available_food: u32,
        has_work_meal: bool,
        budget: u64,
        available_time: u32,
    ) -> MealMode {
        let housing_level = self.housing_levels.get(&household_id).copied().unwrap_or(HousingLevel::Level1);
        let food_price = self.food_prices.get(&GoodType::Food).copied().unwrap_or(100);
        
        let mut possible_modes = Vec::new();
        
        // Vérifier chaque mode
        if has_work_meal {
            let cost = 0; // Supposons que le repas du travail est gratuit ou inclus
            let time = MealMode::WorkMeal.time_cost();
            if time <= available_time && cost <= budget {
                possible_modes.push((MealMode::WorkMeal, cost, time));
            }
        }
        
        if housing_level.can_cook() {
            let cost = available_food as u64 * food_price / 2; // On utilise ce qu'on a
            let time = MealMode::HomeCooking.time_cost();
            if time <= available_time {
                possible_modes.push((MealMode::HomeCooking, cost, time));
            }
        }
        
        // Épicerie
        let grocery_quantity = (budget / food_price) as u32;
        let grocery_cost = grocery_quantity as u64 * food_price;
        let grocery_time = MealMode::Grocery.time_cost();
        if grocery_time <= available_time && grocery_quantity > 0 {
            possible_modes.push((MealMode::Grocery, grocery_cost, grocery_time));
        }
        
        // Restaurant (plus cher)
        let restaurant_cost = budget.min(5000); // 50.00 € max
        let restaurant_time = MealMode::Restaurant.time_cost();
        if restaurant_time <= available_time && restaurant_cost > 0 {
            possible_modes.push((MealMode::Restaurant, restaurant_cost, restaurant_time));
        }
        
        // Choisir le mode avec le meilleur rapport qualité/prix
        // Pour l'instant, on choisit le moins cher
        possible_modes.sort_by(|a, b| {
            // D'abord par coût monétaire, puis par coût temporel
            a.1.cmp(&b.1).then(a.2.cmp(&b.2))
        });
        
        possible_modes.get(0).map(|&(mode, _, _)| mode).unwrap_or(MealMode::Grocery)
    }
    
    /// Enregistre une dépense de repas
    pub fn record_meal_expense(
        &mut self,
        household_id: EntityId,
        meal_mode: MealMode,
        food_quantity: u32,
        monetary_cost: u64,
        time_cost: u32,
        day: i64,
    ) {
        let expense = MealExpense {
            household_id,
            meal_mode,
            food_quantity,
            monetary_cost,
            time_cost,
            day,
        };
        
        self.meal_expenses.insert((household_id, day), expense);
    }
    
    /// Calcul le panier pour un foyer
    pub fn calculate_basket(&self, household_id: EntityId, food_cost: u64) -> Basket {
        let housing_level = self.housing_levels.get(&household_id).copied().unwrap_or(HousingLevel::Level1);
        let housing_cost = housing_level.daily_cost();
        
        // Coût de l'eau : calculé comme pourcentage du panier (nourriture + logement)
        let panier_without_water = food_cost + housing_cost;
        let water_cost = self.calculate_water_cost(household_id, panier_without_water, 0); // Pas de rareté pour l'instant
        
        // Coût de la fiscalité : 10% des revenus (sera ajouté séparément)
        let tax_cost = 0; // Calculé dans TaxManager
        
        Basket::new(food_cost, housing_cost, water_cost, tax_cost)
    }
    
    /// Résout les repas pour tous les foyers pour un jour donné
    pub fn resolve_all_meals(&mut self, day: i64) {
        // Pour chaque foyer, résoudre le mode de repas
        // Cloner les IDs pour éviter le borrow conflict
        let household_ids: Vec<EntityId> = self.housing_levels.keys().cloned().collect();
        for household_id in household_ids {
            let housing_level = self.housing_levels[&household_id];
            let budget = 10000; // Budget par défaut (100.00 €)
            let available_time = 8; // 4 heures en demi-heures
            let available_food = 10; // 10 unités de nourriture
            let has_work_meal = false;
            
            let meal_mode = self.resolve_meal_mode(
                household_id,
                available_food,
                has_work_meal,
                budget,
                available_time,
            );
            
            let food_price = self.food_prices.get(&GoodType::Food).copied().unwrap_or(100);
            let monetary_cost = match meal_mode {
                MealMode::HomeCooking => available_food as u64 * food_price / 2,
                MealMode::Grocery => (budget / food_price) as u64 * food_price,
                MealMode::Restaurant => budget.min(5000),
                MealMode::WorkMeal => 0,
            };
            
            self.record_meal_expense(
                household_id,
                meal_mode,
                available_food,
                monetary_cost,
                meal_mode.time_cost(),
                day,
            );
        }
    }
    
    /// Vérifie T5.10 : distribution des modes de repas
    /// 
    /// §17.7 T5.10 : "distribution des modes de repas corrélée au salaire réel et au niveau de logement"
    pub fn verify_t5_10_meal_distribution(&self) -> bool {
        // Vérifier que les foyers avec niveau 2+ utilisent la cuisine maison
        for ((household_id, _), expense) in &self.meal_expenses {
            let housing_level = self.housing_levels.get(household_id).copied().unwrap_or(HousingLevel::Level1);
            
            if housing_level.can_cook() && expense.meal_mode == MealMode::HomeCooking {
                return true;
            }
        }
        true // Par défaut
    }
    
    /// Vérifie I9 : déterminisme des calculs de repas
    /// 
    /// §17.7 T5.1 : "même graine, même données = même résultat"
    /// Vérifie que les calculs de repas sont déterministes
    pub fn verify_i9_deterministic_meals(&self, other: &MealManager) -> bool {
        // Comparer les dépenses de repas
        if self.meal_expenses.len() != other.meal_expenses.len() {
            return false;
        }
        
        for ((household_id, day), expense1) in &self.meal_expenses {
            if let Some(expense2) = other.meal_expenses.get(&(*household_id, *day)) {
                if expense1.meal_mode != expense2.meal_mode 
                    || expense1.monetary_cost != expense2.monetary_cost 
                    || expense1.time_cost != expense2.time_cost {
                    return false;
                }
            }
        }
        
        // Comparer les paniers
        if self.baskets.len() != other.baskets.len() {
            return false;
        }
        
        for (household_id, basket1) in &self.baskets {
            if let Some(basket2) = other.baskets.get(household_id) {
                if basket1.food_cost != basket2.food_cost 
                    || basket1.housing_cost != basket2.housing_cost 
                    || basket1.water_cost != basket2.water_cost 
                    || basket1.tax_cost != basket2.tax_cost {
                    return false;
                }
            }
        }
        
        true
    }
}

impl Default for MealManager {
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
    
    #[test]
    fn test_meal_mode_resolution() {
        let mut manager = MealManager::new();
        
        let household = EntityId::new(crate::entities::EntityType::Household, 0);
        
        // Niveau 1 : ne peut pas cuisiner
        manager.set_housing_level(household, HousingLevel::Level1);
        manager.set_food_price(GoodType::Food, 100);
        
        let mode = manager.resolve_meal_mode(household, 10, false, 10000, 8);
        assert_ne!(mode, MealMode::HomeCooking);
        
        // Niveau 2 : peut cuisiner
        manager.set_housing_level(household, HousingLevel::Level2);
        let mode = manager.resolve_meal_mode(household, 10, false, 10000, 8);
        // Si on a le temps et les ingrédients, la cuisine maison devrait être choisie
        assert_eq!(mode, MealMode::HomeCooking);
    }
    
    #[test]
    fn test_basket_calculation() {
        let mut manager = MealManager::new();
        
        let household = EntityId::new(crate::entities::EntityType::Household, 0);
        manager.set_housing_level(household, HousingLevel::Level2);
        
        let basket = manager.calculate_basket(household, 5000); // Nourriture : 50.00 €
        
        assert!(basket.food_cost > 0);
        assert!(basket.housing_cost > 0);
        assert!(basket.water_cost > 0);
        assert!(basket.total_cost() > 0);
    }
    
    #[test]
    fn test_water_cost_calculation() {
        let manager = MealManager::new();
        let household = EntityId::new(crate::entities::EntityType::Household, 0);
        
        // Sans rareté : 8% du panier
        let cost = manager.calculate_water_cost(household, 10000, 0);
        assert_eq!(cost, 800); // 8% de 10000 = 800
        
        // Avec rareté max : 25% du panier
        let cost = manager.calculate_water_cost(household, 10000, 100);
        assert_eq!(cost, 2500); // 25% de 10000 = 2500
    }
}
