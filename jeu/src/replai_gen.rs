//! Generation deterministe des replais de reference T1-T5.
//!
//! Aucune ecriture dans `src/replai/` : le chemin de sortie est toujours
//! fourni explicitement par l'appelant (`--output-dir` ou tests).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::entities::*;
use crate::{Clock, EventData, Phase, Priority, Timestamp};
use crate::time::{TimeBudget, TimeUsage, TimeManager};
use crate::household::{HouseholdManager, HousingQuality, FoodQuality};
use crate::health::{HealthState, HealthIssue, HealthIssueType, Severity};
use crate::states::{IndividualStates, HiddenVariable};
use crate::market::{MarketManager, GoodType};
use crate::credit::{CreditManager, Collateral, CollateralType};
use crate::tax::{TaxManager, AllocationType};
use crate::meal::{MealManager, HousingLevel};

/// Graine historique des replais de reference.
pub const DEFAULT_SEED: u64 = 42;

/// Nombre d'evenements du replai T1 de reference.
pub const T1_NUM_EVENTS: usize = 10;

/// Nom de fichier canonique : `reference_seed{seed}_{tranche}.json`
/// (tranche deja prefixee `t`, ex. `t3` -> pas de doublon `tt3`).
pub fn replai_filename(seed: u64, tranche: &str) -> String {
    format!("reference_seed{}_{}.json", seed, tranche)
}

/// Ecrit les octets d'un replai dans `output_dir` (cree le repertoire si besoin).
pub fn write_replai(
    output_dir: &Path,
    seed: u64,
    tranche: &str,
    bytes: &[u8],
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let path = output_dir.join(replai_filename(seed, tranche));
    let mut file = fs::File::create(&path)?;
    file.write_all(bytes)?;
    Ok(path)
}

/// Genere les octets JSON pour une tranche `t1`..`t5`.
pub fn generate_bytes(seed: u64, tranche: &str) -> Result<Vec<u8>, String> {
    match tranche {
        "t1" => Ok(generate_t1_bytes(seed, T1_NUM_EVENTS)),
        "t2" => Ok(generate_t2_bytes(seed)),
        "t3" => Ok(generate_t3_bytes(seed)),
        "t4" => Ok(generate_t4_bytes(seed)),
        "t5" => Ok(generate_t5_bytes(seed)),
        _ => Err(format!(
            "Tranche inconnue : {}\nTranches disponibles : t1, t2, t3, t4, t5",
            tranche
        )),
    }
}

/// Genere et ecrit un replai dans `output_dir`.
pub fn generate_and_write(
    output_dir: &Path,
    seed: u64,
    tranche: &str,
) -> Result<PathBuf, String> {
    let bytes = generate_bytes(seed, tranche)?;
    write_replai(output_dir, seed, tranche, &bytes).map_err(|e| e.to_string())
}


/// Genere les octets JSON du replai T1 (noyau temporel).
pub fn generate_t1_bytes(seed: u64, num_events: usize) -> Vec<u8> {
    let mut clock = Clock::new(seed);
    
    for i in 0..num_events {
        let day = (i as i64) / 5;
        let phase = match i % 7 {
            0 => Phase::FluxPhysiques,
            1 => Phase::Marches,
            2 => Phase::Comptes,
            3 => Phase::MarcheDuTravail,
            4 => Phase::DecisionsAgents,
            5 => Phase::Social,
            _ => Phase::Journal,
        };
        let priority = Priority::new((i % 10) as u8);
        
        let entity_id = clock.state.event_queue.next_id();
        clock.state.schedule(
            day,
            phase,
            priority,
            (i as u16) % 100,
            EventData::CreateEntity {
                entity_type: format!("TestEntity_{}", i),
                entity_id,
            },
        );
    }
    
    let mut events = Vec::new();
    for _ in 0..num_events {
        if let Some(event) = clock.state.advance_to_next() {
            events.push(event);
        }
    }
    
    let json: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id.as_u64(),
                "day": e.timestamp.day,
                "seq": e.timestamp.seq,
                "phase": e.phase as u8,
                "priority": e.priority.0,
                "event_type": e.event_type,
                "data": match &e.data {
                    EventData::Nop => serde_json::Value::Null,
                    EventData::CreateEntity { entity_type, entity_id } => {
                        serde_json::json!({
                            "type": "CreateEntity",
                            "entity_type": entity_type,
                            "entity_id": entity_id.as_u64()
                        })
                    }
                    EventData::UpdateState { field, old_value, new_value } => {
                        serde_json::json!({
                            "type": "UpdateState",
                            "field": field,
                            "old_value": old_value,
                            "new_value": new_value
                        })
                    }
                    EventData::InputChanged { entity_id, u, p, q } => {
                        serde_json::json!({
                            "type": "InputChanged",
                            "entity_id": entity_id.to_u64(),
                            "u": u,
                            "p": p,
                            "q": q
                        })
                    }
                    EventData::GriefBoundReached { entity_id, bound_qmicro, schedule_version } => {
                        serde_json::json!({
                            "type": "GriefBoundReached",
                            "entity_id": entity_id.to_u64(),
                            "bound_qmicro": bound_qmicro,
                            "schedule_version": schedule_version
                        })
                    }
                    EventData::StrikeCooldownEnded { entity_id, schedule_version } => {
                        serde_json::json!({
                            "type": "StrikeCooldownEnded",
                            "entity_id": entity_id.to_u64(),
                            "schedule_version": schedule_version
                        })
                    }
                    EventData::StrikeTriggered { entity_id, draw_index, strike_index, schedule_version } => {
                        serde_json::json!({
                            "type": "StrikeTriggered",
                            "entity_id": entity_id.to_u64(),
                            "draw_index": draw_index,
                            "strike_index": strike_index,
                            "schedule_version": schedule_version
                        })
                    }
                    EventData::StrikeEnded { entity_id, strike_index, cause, schedule_version } => {
                        serde_json::json!({
                            "type": "StrikeEnded",
                            "entity_id": entity_id.to_u64(),
                            "strike_index": strike_index,
                            "cause": format!("{:?}", cause),
                            "schedule_version": schedule_version
                        })
                    }
                }
            })
        })
        .collect();
    
    serde_json::to_string_pretty(&json).unwrap().into_bytes()
}

/// Genere les octets JSON du replai T2 (entites et journal).
pub fn generate_t2_bytes(seed: u64) -> Vec<u8> {
    let mut manager = EntityManager::new(seed);
    let mut journal = CausalJournal::new();
    
    let cohort_a = manager.create_cohort(100);
    let cohort_b = manager.create_cohort(100);
    
    let mut extracted_individuals = Vec::new();
    for i in 0..5 {
        let individual = manager.extract(cohort_a);
        if let Some(ind) = individual {
            extracted_individuals.push(ind.clone());
            
            let entry = JournalEntry {
                id: 0,
                timestamp: Timestamp::new(0, i as u32),
                event_id: i + 1,
                phase: 6,
                actor: Some(ind.id),
                decision_type: 1,
                modified_values: vec![
                    Modification {
                        field: "population".to_string(),
                        old_value: (100 - i) as i64,
                        new_value: (99 - i) as i64,
                    },
                ],
                causes: vec![],
                learned_by: vec![],
            };
            journal.add_entry(entry);
        }
    }
    
    manager.merge_cohorts(cohort_a, cohort_b);
    
    for individual in &extracted_individuals {
        manager.reinject(individual.clone());
    }
    
    let state = serde_json::json!({
        "seed": seed,
        "cohorts": manager.all_cohorts().map(|(id, cohort)| {
            serde_json::json!({
                "id": id.to_u64(),
                "population": cohort.population,
                "total_money": cohort.total_money,
                "total_griefs": cohort.total_griefs,
                "total_attachment": cohort.total_attachment,
            })
        }).collect::<Vec<_>>(),
        "named_individuals": manager.named_individuals().iter().map(|(id, ind)| {
            serde_json::json!({
                "id": id.to_u64(),
                "cohort_id": ind.cohort_id.to_u64(),
                "grief": ind.grief,
                "balance": ind.balance,
                "attachment": ind.attachment,
            })
        }).collect::<Vec<_>>(),
        "journal_entries": journal.all_entries().map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "day": entry.timestamp.day,
                "phase": entry.phase,
                "event_id": entry.event_id,
                "actor": entry.actor.map(|a| a.to_u64()),
                "decision_type": entry.decision_type,
            })
        }).collect::<Vec<_>>(),
    });
    
    serde_json::to_string_pretty(&state).unwrap().into_bytes()
}

pub fn generate_t3_bytes(seed: u64) -> Vec<u8> {
    println!("GÃ©nÃ©ration du replai de rÃ©fÃ©rence pour la Tranche 3 (seed={})...", seed);
    
    let mut events = Vec::new();
    let mut event_id = 1u64;
    
    // Jour 0 : CrÃ©ation des entitÃ©s
    let individual_id = EntityId::new(EntityType::NamedIndividual, 0);
    let household_id = EntityId::new(EntityType::Household, 0);
    
    // Ã‰vÃ©nement : CrÃ©ation de l'individu
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
    
    // Ã‰vÃ©nement : CrÃ©ation du foyer
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
    
    // Ã‰vÃ©nement : Ã‰tat de santÃ© initial
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
    
    // Ã‰vÃ©nement : Budget de temps initial
    let mut time_manager = TimeManager::new();
    let individual = crate::entities::NamedIndividual::new(
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
    
    // Ã‰vÃ©nement : Stock de nourriture initial
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
    
    // Ã‰vÃ©nement : Variables cachÃ©es initiales
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
    
    // Ã‰vÃ©nement : Ã‰tats dÃ©clenchÃ©s initiaux
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
    
    // Jour 2 : Mise Ã  jour santÃ©
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

    let json_value: Value = json!(events);
    serde_json::to_string_pretty(&json_value).expect("Impossible de serialiser en JSON").into_bytes()
}

pub fn generate_t4_bytes(seed: u64) -> Vec<u8> {
    use crate::asset::{AssetManager, AssetType, MAX_INITIAL_STATE};
    use crate::common::MaintenancePolicy;
    use crate::firm::{FirmManager, FirmType};
    use crate::recipe::{RecipeManager, RecipeType, Quality};
    use crate::maintenance::MaintenanceManager;
    use crate::production::ProductionManager;
    
    println!("GÃ©nÃ©ration du replai de rÃ©fÃ©rence pour la Tranche 4 (seed={})...", seed);
    
    let mut events = Vec::new();
    let mut event_id = 1u64;
    
    // Jour 0 : CrÃ©ation d'une firme
    let mut registry = crate::entities::EntityRegistry::new();
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
    
    // Jour 0 : CrÃ©ation d'actifs
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
    
    // Jour 0 : DÃ©claration d'entretien
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
    
    // Jour 0 : CrÃ©ation de recettes
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
    
    // Jour 1 : CrÃ©ation d'un ordre de production
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
    
    // Jour 30 : DÃ©claration mensuelle d'entretien
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
    
    // Jour 100 : Ã‰tat de l'actif
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

    let json_value: Value = json!(events);
    serde_json::to_string_pretty(&json_value).expect("Impossible de serialiser en JSON").into_bytes()
}

pub fn generate_t5_bytes(seed: u64) -> Vec<u8> {
    use crate::credit::BASE_INTEREST_RATE;
    use crate::tax::PENSION_AMOUNT;
    
    println!("GÃ©nÃ©ration du replai de rÃ©fÃ©rence pour la Tranche 5 (seed={})...", seed);
    
    let mut events = Vec::new();
    let mut event_id = 1u64;
    
    // Jour 0 : Initialisation des managers T5
    let mut market_manager = MarketManager::new();
    let mut credit_manager = CreditManager::new();
    let mut tax_manager = TaxManager::new();
    let mut meal_manager = MealManager::new();
    
    // CrÃ©ation des entitÃ©s
    let firm_id = EntityId::new(EntityType::Firm, 0);
    let household_id = EntityId::new(EntityType::Household, 0);
    let individual_id = EntityId::new(EntityType::NamedIndividual, 0);
    let housing_id = EntityId::new(EntityType::Housing, 0);
    let good_id = 0u64;
    
    // Initialisation du marchÃ© pour la nourriture
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
    
    // Prix de rÃ©fÃ©rence pour la nourriture
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
    
    // CrÃ©ation d'un prÃªt
    let lender_id = EntityId::new(EntityType::Firm, 1);
    let borrower_id = household_id;
    
    let loan_id = credit_manager.create_loan(
        lender_id,
        borrower_id,
        100000, // 1000.00 â‚¬
        BASE_INTEREST_RATE,
        vec![Collateral {
            collateral_type: CollateralType::Housing,
            asset_id: housing_id,
            value: 150000,
        }],
        12, // 12 Ã©chÃ©ances
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
    
    // PrÃ©lÃ¨vement fiscal
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
    
    // Jour 30 : Mise Ã  jour des prix de rÃ©fÃ©rence
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
    
    // Jour 30 : Paiement du prÃªt
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
    
    // Jour 60 : DÃ©clenchement du dÃ©faut
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

    let json_value: Value = json!(events);
    serde_json::to_string_pretty(&json_value).expect("Impossible de serialiser en JSON").into_bytes()
}

