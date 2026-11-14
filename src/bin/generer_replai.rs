//! Générateur de replai de référence pour Vestiges Horizons
//! 
//! Ce programme génère des replais déterministes pour les tests de non-régression.
//! Les replais sont basés sur une graine (seed) et sont stockés dans src/replai/.
//! 
//! Usage : cargo run --bin generer_replai -- <seed> <tranche>

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};
use vestiges_horizons::entities::{EntityId, EntityType};
use vestiges_horizons::time::{TimeBudget, TimeUsage, TimeManager};
use vestiges_horizons::household::{Household, HouseholdManager, HousingQuality, FoodQuality};
use vestiges_horizons::health::{HealthState, HealthIssue, HealthIssueType, Severity};
use vestiges_horizons::states::{IndividualStates, HiddenVariable, TriggeredStateType};
use vestiges_horizons::market::{MarketManager, GoodType, ReferencePrice};
use vestiges_horizons::credit::{CreditManager, Collateral, CollateralType, LoanStatus};
use vestiges_horizons::tax::{TaxManager, AllocationType};
use vestiges_horizons::meal::{MealManager, MealMode, HousingLevel};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage : cargo run --bin generer_replai -- <seed> <tranche>");
        eprintln!("Exemple : cargo run --bin generer_replai -- 42 t3");
        std::process::exit(1);
    }
    
    let seed: u64 = args[1].parse().expect("Seed doit être un entier");
    let tranche = &args[2];
    
    match tranche.as_str() {
        "t3" => generer_replai_tranche3(seed),
        "t4" => generer_replai_tranche4(seed),
        "t5" => generer_replai_tranche5(seed),
        _ => {
            eprintln!("Tranche inconnue : {}", tranche);
            eprintln!("Tranches disponibles : t3, t4, t5");
            std::process::exit(1);
        }
    }
}

/// Génère un replai de référence pour la Tranche 3
fn generer_replai_tranche3(seed: u64) {
    println!("Génération du replai de référence pour la Tranche 3 (seed={})...", seed);
    
    let mut events = Vec::new();
    let mut event_id = 1u64;
    
    // Jour 0 : Création des entités
    let individual_id = EntityId::new(EntityType::NamedIndividual, 0);
    let household_id = EntityId::new(EntityType::Household, 0);
    
    // Événement : Création de l'individu
    events.push(json!({
        "id": event_id,
        "event_type": 100,
        "day": 0,
        "phase": 1,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "CreateEntity",
            "entity_id": individual_id.to_u64(),
            "entity_type": format!("{:?}", individual_id.entity_type)
        }
    }));
    event_id += 1;
    
    // Événement : Création du foyer
    events.push(json!({
        "id": event_id,
        "event_type": 101,
        "day": 0,
        "phase": 1,
        "priority": 0,
        "seq": 1,
        "data": {
            "type": "CreateEntity",
            "entity_id": household_id.to_u64(),
            "entity_type": format!("{:?}", household_id.entity_type)
        }
    }));
    event_id += 1;
    
    // Événement : État de santé initial
    let mut health_state = HealthState::new(individual_id, 1000);
    events.push(json!({
        "id": event_id,
        "event_type": 102,
        "day": 0,
        "phase": 2,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "HealthStatus",
            "entity_id": individual_id.to_u64(),
            "health": health_state.health,
            "max_health": health_state.max_health
        }
    }));
    event_id += 1;
    
    // Événement : Budget de temps initial
    let mut time_manager = TimeManager::new();
    let individual = vestiges_horizons::entities::NamedIndividual::new(
        individual_id,
        EntityId::new(EntityType::Cohort, 0),
        0,
        0,
        0,
    );
    let budget = time_manager.resolve_daily_schedule(
        &individual,
        0,
        true,
        8,
        false,
        0,
    );
    
    events.push(json!({
        "id": event_id,
        "event_type": 103,
        "day": 0,
        "phase": 3,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "TimeBudget",
            "entity_id": individual_id.to_u64(),
            "time_usage": {
                "Travail": budget.get_allocation(TimeUsage::Travail),
                "Repas": budget.get_allocation(TimeUsage::Repas),
                "Trajet": budget.get_allocation(TimeUsage::Trajet),
                "Corvee": budget.get_allocation(TimeUsage::Corvee),
                "Formation": budget.get_allocation(TimeUsage::Formation),
                "Recherche": budget.get_allocation(TimeUsage::Recherche),
                "Loisir": budget.get_allocation(TimeUsage::Loisir),
                "Sommeil": budget.get_allocation(TimeUsage::Sommeil)
            }
        }
    }));
    event_id += 1;
    
    // Événement : Stock de nourriture initial
    let mut household_manager = HouseholdManager::new();
    let h_id = household_manager.create_household(individual_id, HousingQuality::Base);
    let household = household_manager.get_household_mut(h_id).unwrap();
    household.add_food(FoodQuality::Base, 10);
    household.add_food(FoodQuality::Moyenne, 5);
    household.add_food(FoodQuality::Superieure, 2);
    
    events.push(json!({
        "id": event_id,
        "event_type": 104,
        "day": 0,
        "phase": 4,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "HouseholdFood",
            "entity_id": h_id.to_u64(),
            "food_stock": {
                "Base": household.get_food_stock(FoodQuality::Base).unwrap_or(&0),
                "Moyenne": household.get_food_stock(FoodQuality::Moyenne).unwrap_or(&0),
                "Superieure": household.get_food_stock(FoodQuality::Superieure).unwrap_or(&0)
            }
        }
    }));
    event_id += 1;
    
    // Événement : Variables cachées initiales
    let mut states = IndividualStates::new(individual_id);
    events.push(json!({
        "id": event_id,
        "event_type": 105,
        "day": 0,
        "phase": 5,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "HiddenVariables",
            "entity_id": individual_id.to_u64(),
            "hidden_variables": {
                "EconomicStress": states.get_variable(HiddenVariable::EconomicStress),
                "SocialTension": states.get_variable(HiddenVariable::SocialTension),
                "TimePressure": states.get_variable(HiddenVariable::TimePressure),
                "FutureUncertainty": states.get_variable(HiddenVariable::FutureUncertainty),
                "AccumulatedFatigue": states.get_variable(HiddenVariable::AccumulatedFatigue)
            }
        }
    }));
    event_id += 1;
    
    // Événement : États déclenchés initiaux
    events.push(json!({
        "id": event_id,
        "event_type": 106,
        "day": 0,
        "phase": 6,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "TriggeredStates",
            "entity_id": individual_id.to_u64(),
            "triggered_states": []
        }
    }));
    event_id += 1;
    
    // Jour 1 : Maladie et soins
    let issue = HealthIssue::new(
        42,
        HealthIssueType::Infectious,
        1,
        Severity::Mild,
        1000,
    );
    health_state.add_health_issue(issue);
    
    events.push(json!({
        "id": event_id,
        "event_type": 107,
        "day": 1,
        "phase": 2,
        "priority": 1,
        "seq": 0,
        "data": {
            "type": "HealthIssue",
            "entity_id": individual_id.to_u64(),
            "health_issue": {
                "stable_key": 42u64,
                "issue_type": "Infectious",
                "severity": "Mild",
                "onset_day": 1i64
            }
        }
    }));
    event_id += 1;
    
    // Jour 2 : Soins
    let mut budget_day2 = TimeBudget::new(individual_id, 2);
    budget_day2.allocate(TimeUsage::Travail, 6);
    budget_day2.allocate(TimeUsage::Repas, 2);
    
    let mut money = 10000u64;
    let success = health_state.treat_health_issue(42, &mut budget_day2, &mut money);
    
    events.push(json!({
        "id": event_id,
        "event_type": 108,
        "day": 2,
        "phase": 3,
        "priority": 1,
        "seq": 0,
        "data": {
            "type": "HealthCare",
            "entity_id": individual_id.to_u64(),
            "care_time": budget_day2.get_allocation(TimeUsage::Soin),
            "care_money": 10000 - money,
            "success": success
        }
    }));
    event_id += 1;
    
    // Jour 2 : Mise à jour santé
    health_state.update(2, true, true, 0, 0);
    
    events.push(json!({
        "id": event_id,
        "event_type": 109,
        "day": 2,
        "phase": 4,
        "priority": 1,
        "seq": 1,
        "data": {
            "type": "HealthUpdate",
            "entity_id": individual_id.to_u64(),
            "health": health_state.health
        }
    }));
    event_id += 1;
    
    // Sauvegarder le replai
    let filename = format!("src/replai/reference_seed{}_t3.json", seed);
    let path = Path::new(&filename);
    
    let json_value: Value = json!(events);
    let json_string = serde_json::to_string_pretty(&json_value).expect("Impossible de sérialiser en JSON");
    
    let mut file = File::create(path).expect("Impossible de créer le fichier de replai");
    file.write_all(json_string.as_bytes()).expect("Impossible d'écrire dans le fichier");
    
    println!("Replai généré : {}", filename);
    println!("  {} événements", events.len());
    println!("  Dernier event_id : {}", event_id - 1);
}

/// Génère un replai de référence pour la Tranche 4
fn generer_replai_tranche4(seed: u64) {
    use vestiges_horizons::asset::{AssetManager, AssetType, MAX_INITIAL_STATE};
    use vestiges_horizons::common::MaintenancePolicy;
    use vestiges_horizons::firm::{FirmManager, FirmType};
    use vestiges_horizons::recipe::{RecipeManager, RecipeType, Quality};
    use vestiges_horizons::maintenance::MaintenanceManager;
    use vestiges_horizons::production::ProductionManager;
    
    println!("Génération du replai de référence pour la Tranche 4 (seed={})...", seed);
    
    let mut events = Vec::new();
    let mut event_id = 1u64;
    
    // Jour 0 : Création d'une firme
    let mut registry = vestiges_horizons::entities::EntityRegistry::new();
    let mut firm_manager = FirmManager::new();
    
    let owner_id = registry.next_id(EntityType::NamedIndividual);
    let firm_id = firm_manager.create_firm(
        FirmType::Workshop,
        owner_id,
        0,
    );
    
    events.push(json!({
        "id": event_id,
        "event_type": 200,
        "day": 0,
        "phase": 1,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "CreateFirm",
            "firm_id": firm_id.to_u64(),
            "owner_id": owner_id.to_u64(),
            "firm_type": "Workshop",
            "production_scale": "Small"
        }
    }));
    event_id += 1;
    
    // Jour 0 : Création d'actifs
    let mut asset_manager = AssetManager::new();
    let asset1_id = asset_manager.create_asset(
        AssetType::Machine,
        MAX_INITIAL_STATE,
        0,
        10, // usure de 10/jour
        firm_id,
        1000,
    );
    
    events.push(json!({
        "id": event_id,
        "event_type": 201,
        "day": 0,
        "phase": 1,
        "priority": 1,
        "seq": 0,
        "data": {
            "type": "CreateAsset",
            "asset_id": asset1_id.to_u64(),
            "firm_id": firm_id.to_u64(),
            "asset_type": "Machine",
            "initial_state": MAX_INITIAL_STATE,
            "wear_rate": 10u32,
            "capacity": 1000u32
        }
    }));
    event_id += 1;
    
    // Jour 0 : Déclaration d'entretien
    let mut maintenance_manager = MaintenanceManager::new();
    maintenance_manager.create_plan(
        asset1_id,
        firm_id,
        MaintenancePolicy::Normal,
        50,
        0,
    );
    
    events.push(json!({
        "id": event_id,
        "event_type": 202,
        "day": 0,
        "phase": 2,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "MaintenanceDeclaration",
            "asset_id": asset1_id.to_u64(),
            "policy": "Normal",
            "monthly_cost": 50u64
        }
    }));
    event_id += 1;
    
    // Jour 0 : Création de recettes
    let mut recipe_manager = RecipeManager::new();
    let recipe_id = recipe_manager.create_recipe(
        "Production de base".to_string(),
        RecipeType::Manufacturing,
        std::collections::BTreeMap::new(),
        1,
        1,
        registry.next_id(EntityType::Good).to_u64(),
        10,
        Quality::Standard,
        0,
    );
    
    events.push(json!({
        "id": event_id,
        "event_type": 203,
        "day": 0,
        "phase": 1,
        "priority": 2,
        "seq": 0,
        "data": {
            "type": "CreateRecipe",
            "recipe_id": recipe_id,
            "name": "Production de base",
            "recipe_type": "Manufacturing",
            "output_quantity": 10u32
        }
    }));
    event_id += 1;
    
    // Jour 1 : Création d'un ordre de production
    let mut production_manager = ProductionManager::new();
    let order_id = production_manager.create_order_auto(
        &mut registry,
        firm_id,
        recipe_id,
        50,
        None,
        1,
    );
    
    events.push(json!({
        "id": event_id,
        "event_type": 204,
        "day": 1,
        "phase": 3,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "CreateProductionOrder",
            "order_id": order_id.to_u64(),
            "firm_id": firm_id.to_u64(),
            "recipe_id": recipe_id,
            "quantity": 50u64
        }
    }));
    event_id += 1;
    
    // Jour 1 : Production
    let results = production_manager.daily_production(
        firm_id,
        &firm_manager,
        &asset_manager,
        &recipe_manager,
        1,
    );
    
    for result in &results {
        events.push(json!({
            "id": event_id,
            "event_type": 205,
            "day": 1,
            "phase": 4,
            "priority": 0,
            "seq": event_id as u32 % 100,
            "data": {
                "type": "ProductionResult",
                "order_id": result.order_id.to_u64(),
                "output_quantity": result.output_quantity,
                "waste": result.waste,
                "limiting_factor": format!("{:?}", result.limiting_factor)
            }
        }));
        event_id += 1;
    }
    
    // Jour 30 : Déclaration mensuelle d'entretien
    maintenance_manager.monthly_declaration(firm_id, 30);
    
    events.push(json!({
        "id": event_id,
        "event_type": 206,
        "day": 30,
        "phase": 2,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "MonthlyMaintenanceDeclaration",
            "firm_id": firm_id.to_u64(),
            "total_cost": maintenance_manager.get_total_cost(firm_id)
        }
    }));
    event_id += 1;
    
    // Jour 100 : État de l'actif
    let asset = asset_manager.get_asset(asset1_id).unwrap();
    let state = asset.current_state(100);
    let capacity = asset.effective_capacity(100);
    
    events.push(json!({
        "id": event_id,
        "event_type": 207,
        "day": 100,
        "phase": 1,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "AssetStatus",
            "asset_id": asset1_id.to_u64(),
            "state": state,
            "capacity": capacity
        }
    }));
    event_id += 1;
    
    // Sauvegarder le replai
    let filename = format!("src/replai/reference_seed{}_t4.json", seed);
    let path = Path::new(&filename);
    
    let json_value: Value = json!(events);
    let json_string = serde_json::to_string_pretty(&json_value).expect("Impossible de sérialiser en JSON");
    
    let mut file = File::create(path).expect("Impossible de créer le fichier de replai");
    file.write_all(json_string.as_bytes()).expect("Impossible d'écrire dans le fichier");
    
    println!("Replai T4 généré : {}", filename);
    println!("  {} événements", events.len());
    println!("  Dernier event_id : {}", event_id - 1);
}

/// Génère un replai de référence pour la Tranche 5
/// Inclut : marché, crédit, fiscalité, repas
fn generer_replai_tranche5(seed: u64) {
    use vestiges_horizons::credit::{BASE_INTEREST_RATE, DEFAULT_INTEREST_RATE};
    use vestiges_horizons::tax::{TAX_RATE, PENSION_AMOUNT, CHILD_ALLOCATION};
    use vestiges_horizons::meal::{WATER_BASE_COST_PERCENT, WATER_MAX_COST_PERCENT};
    
    println!("Génération du replai de référence pour la Tranche 5 (seed={})...", seed);
    
    let mut events = Vec::new();
    let mut event_id = 1u64;
    
    // Jour 0 : Initialisation des managers T5
    let mut market_manager = MarketManager::new();
    let mut credit_manager = CreditManager::new();
    let mut tax_manager = TaxManager::new();
    let mut meal_manager = MealManager::new();
    
    // Création des entités
    let firm_id = EntityId::new(EntityType::Firm, 0);
    let household_id = EntityId::new(EntityType::Household, 0);
    let individual_id = EntityId::new(EntityType::NamedIndividual, 0);
    let housing_id = EntityId::new(EntityType::Housing, 0);
    let good_id = 0u64;
    
    // Initialisation du marché pour la nourriture
    market_manager.create_market(firm_id, GoodType::Food, good_id, 1000, 100, None, (200, 800), 10, (50, 200), 50);
    
    events.push(json!({
        "id": event_id,
        "event_type": 501,
        "day": 0,
        "phase": 1,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "MarketCreated",
            "market_id": firm_id.to_u64(),
            "good_type": "Food",
            "good_id": good_id,
            "displayed_price": 1000,
            "stock": 100
        }
    }));
    event_id += 1;
    
    // Prix de référence pour la nourriture
    market_manager.add_reference_price(good_id, 1000, 100);
    
    events.push(json!({
        "id": event_id,
        "event_type": 502,
        "day": 0,
        "phase": 1,
        "priority": 0,
        "seq": 1,
        "data": {
            "type": "ReferencePriceCreated",
            "good_id": good_id,
            "good_type": "Food",
            "base_price": 1000,
            "nominal_capacity": 100
        }
    }));
    event_id += 1;
    
    // Création d'un prêt
    let lender_id = EntityId::new(EntityType::Firm, 1);
    let borrower_id = household_id;
    
    let loan_id = credit_manager.create_loan(
        lender_id,
        borrower_id,
        100000, // 1000.00 €
        BASE_INTEREST_RATE,
        vec![Collateral {
            collateral_type: CollateralType::Housing,
            asset_id: housing_id,
            value: 150000,
        }],
        12, // 12 échéances
        0,
        30, // 30 jours
    );
    
    events.push(json!({
        "id": event_id,
        "event_type": 503,
        "day": 0,
        "phase": 2,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "LoanCreated",
            "loan_id": loan_id,
            "lender_id": lender_id.to_u64(),
            "borrower_id": borrower_id.to_u64(),
            "principal": 100000,
            "interest_rate": BASE_INTEREST_RATE
        }
    }));
    event_id += 1;
    
    // Prélèvement fiscal
    tax_manager.withhold_tax(firm_id, individual_id, 100000, 0);
    
    events.push(json!({
        "id": event_id,
        "event_type": 504,
        "day": 0,
        "phase": 2,
        "priority": 0,
        "seq": 1,
        "data": {
            "type": "TaxWithheld",
            "firm_id": firm_id.to_u64(),
            "employee_id": individual_id.to_u64(),
            "gross_salary": 100000,
            "withheld_amount": 10000 // 10%
        }
    }));
    event_id += 1;
    
    // Allocation
    tax_manager.pay_allocation(household_id, AllocationType::Pension, PENSION_AMOUNT, 0);
    
    events.push(json!({
        "id": event_id,
        "event_type": 505,
        "day": 0,
        "phase": 2,
        "priority": 0,
        "seq": 2,
        "data": {
            "type": "AllocationPaid",
            "household_id": household_id.to_u64(),
            "allocation_type": "Pension",
            "amount": PENSION_AMOUNT
        }
    }));
    event_id += 1;
    
    // Configuration du repas
    meal_manager.set_housing_level(household_id, HousingLevel::Level2);
    meal_manager.set_food_price(GoodType::Food, 100);
    
    let meal_mode = meal_manager.resolve_meal_mode(household_id, 10, false, 10000, 8);
    
    events.push(json!({
        "id": event_id,
        "event_type": 506,
        "day": 0,
        "phase": 3,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "MealModeResolved",
            "household_id": household_id.to_u64(),
            "meal_mode": format!("{:?}", meal_mode),
            "available_food": 10,
            "budget": 10000,
            "available_time": 8
        }
    }));
    event_id += 1;
    
    // Panier
    let basket = meal_manager.calculate_basket(household_id, 5000);
    
    events.push(json!({
        "id": event_id,
        "event_type": 507,
        "day": 0,
        "phase": 3,
        "priority": 0,
        "seq": 1,
        "data": {
            "type": "BasketCalculated",
            "household_id": household_id.to_u64(),
            "food_cost": basket.food_cost,
            "housing_cost": basket.housing_cost,
            "water_cost": basket.water_cost,
            "tax_cost": basket.tax_cost,
            "total_cost": basket.total_cost()
        }
    }));
    event_id += 1;
    
    // Jour 30 : Mise à jour des prix de référence
    if let Some(ref_price) = market_manager.get_reference_price_mut(good_id) {
        ref_price.add_demand(120);
    }
    market_manager.update_reference_prices(30);
    
    let ref_price = market_manager.get_reference_price(good_id).unwrap();
    
    events.push(json!({
        "id": event_id,
        "event_type": 508,
        "day": 30,
        "phase": 2,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "ReferencePriceUpdated",
            "good_type": "Food",
            "old_price": 1000,
            "new_price": ref_price.current_price,
            "demand": 120
        }
    }));
    event_id += 1;
    
    // Jour 30 : Paiement du prêt
    credit_manager.make_payment(loan_id, 10000, 30);
    
    events.push(json!({
        "id": event_id,
        "event_type": 509,
        "day": 30,
        "phase": 2,
        "priority": 0,
        "seq": 1,
        "data": {
            "type": "LoanPayment",
            "loan_id": loan_id,
            "amount": 10000,
            "day": 30
        }
    }));
    event_id += 1;
    
    // Jour 60 : Déclenchement du défaut
    credit_manager.process_due_dates(60);
    credit_manager.process_seizures(60);
    
    let loan = credit_manager.get_loan(loan_id).unwrap();
    
    events.push(json!({
        "id": event_id,
        "event_type": 510,
        "day": 60,
        "phase": 2,
        "priority": 0,
        "seq": 0,
        "data": {
            "type": "LoanStatusUpdated",
            "loan_id": loan_id,
            "status": format!("{:?}", loan.status),
            "remaining_amount": loan.remaining_amount
        }
    }));
    event_id += 1;
    
    // Sauvegarder le replai
    let filename = format!("src/replai/reference_seed{}_t5.json", seed);
    let path = Path::new(&filename);
    
    let json_value: Value = json!(events);
    let json_string = serde_json::to_string_pretty(&json_value).expect("Impossible de sérialiser en JSON");
    
    let mut file = File::create(path).expect("Impossible de créer le fichier de replai");
    file.write_all(json_string.as_bytes()).expect("Impossible d'écrire dans le fichier");
    
    println!("Replai T5 généré : {}", filename);
    println!("  {} événements", events.len());
    println!("  Dernier event_id : {}", event_id - 1);
}
