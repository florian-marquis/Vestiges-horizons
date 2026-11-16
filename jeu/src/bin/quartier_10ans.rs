use std::collections::BTreeMap;
use std::error::Error;
use std::io;

use vestiges_horizons::asset::{Asset, AssetType};
use vestiges_horizons::entities::{EntityId, EntityManager, EntityType};
use vestiges_horizons::health::{CareType, DispensaryPatient, DispensaryQueue};
use vestiges_horizons::household::{FoodQuality, Household, HousingQuality};
use vestiges_horizons::inputs::{calculate_q, InputTracker, InputValues, ServiceComponent};
use vestiges_horizons::market::{GoodType, MarketManager};
use vestiges_horizons::salary::{PanierComposante, SalaireReelEtGriefsManager};
use vestiges_horizons::{EventQueue, Id, Phase, Priority, Timestamp};

const SEED: u64 = 42;
const DAYS: i64 = 3_650;
const FOOD_GOOD_ID: u64 = 10;
const BASE_IMPORT_PER_DAY: u32 = 550;
const SHOCK_START_DAY: i64 = 200;
const SHOCK_END_DAY: i64 = 290;
const DAILY_IMPORT_DURING_SHOCK: u32 = BASE_IMPORT_PER_DAY / 2;
const POPULATIONS: [u32; 5] = [60, 80, 100, 120, 140];
const HOURLY_WAGES: [u64; 5] = [100, 200, 300, 400, 500];
const INITIAL_HOUSING_STATES: [u32; 5] = [1_000, 900, 800, 700, 600];
const INITIAL_FOOD_STOCK: u32 = 5_000;

#[derive(Debug, Default)]
struct CohortStats {
    q_min: u16,
    q_max: u16,
    q_sum: u64,
    q_samples: u64,
    days_g_zero: u64,
    days_g_partial: u64,
    days_g_full: u64,
    input_changed_events: u64,
    days_without_food_coverage: u64,
    days_rationed: u64,
    u_history: Vec<u16>,
}

impl CohortStats {
    fn observe(&mut self, q: u16, g: i64, u: u16) {
        if self.q_samples == 0 {
            self.q_min = q;
        } else {
            self.q_min = self.q_min.min(q);
        }
        self.q_max = self.q_max.max(q);
        self.q_sum += u64::from(q);
        self.q_samples += 1;
        match g {
            0 => self.days_g_zero += 1,
            1..=99 => self.days_g_partial += 1,
            _ => self.days_g_full += 1,
        }
        if u > 0 {
            self.days_without_food_coverage += 1;
        }
        self.u_history.push(u);
    }

    fn mean_q_tenths(&self) -> u64 {
        self.q_sum.saturating_mul(10) / self.q_samples
    }
}

#[derive(Debug)]
struct CohortState {
    id: EntityId,
    household: Household,
    dispensary: DispensaryQueue,
    housing: Asset,
    daily_care_demand: u32,
    current_inputs: InputValues,
    stats: CohortStats,
}

#[derive(Debug)]
struct ScenarioReport {
    cohorts: BTreeMap<EntityId, CohortStats>,
    import_history: Vec<u32>,
    stock_history: Vec<u32>,
    full_coverage_history: Vec<bool>,
    market_stockout_days: u64,
}

fn make_household(cohort_id: EntityId, population: u32) -> Household {
    let mut household = Household::new(
        EntityId::new(EntityType::Household, cohort_id.number),
        cohort_id,
        HousingQuality::Base,
    );
    for member in 1..population {
        household.add_member(EntityId::new(
            EntityType::NamedIndividual,
            cohort_id.number * 1_000 + u64::from(member),
        ));
    }
    household
}

fn input_event_id(day: i64, cohort_index: usize) -> Id {
    Id::new(1 + day as u64 * POPULATIONS.len() as u64 + cohort_index as u64)
}

fn run_scenario() -> Result<ScenarioReport, Box<dyn Error>> {
    let mut entities = EntityManager::new(SEED);
    let mut salary = SalaireReelEtGriefsManager::new();
    let mut market = MarketManager::new();
    let mut tracker = InputTracker::default();
    let mut event_queue = EventQueue::new();
    let mut cohorts = Vec::new();

    market.create_market(
        EntityId::new(EntityType::Market, FOOD_GOOD_ID),
        GoodType::Food,
        FOOD_GOOD_ID,
        100,
        INITIAL_FOOD_STOCK,
        None,
        (0, u32::MAX),
        10,
        (50, 200),
        100,
    );
    market.add_reference_price(FOOD_GOOD_ID, 100, 1_000);

    salary.mettre_a_jour_panier(PanierComposante::Logement, 2_000);
    salary.mettre_a_jour_panier(PanierComposante::Eau, 500);
    salary.mettre_a_jour_panier(PanierComposante::Fiscalite, 300);

    for (index, population) in POPULATIONS.iter().copied().enumerate() {
        let cohort_id = entities.create_cohort(u64::from(population));
        salary.ajouter_entite(cohort_id);
        salary.mettre_a_jour_salaire_horaire(cohort_id, HOURLY_WAGES[index]);
        salary.mettre_a_jour_heures(cohort_id, 8);

        let care_capacity = population / 10;
        cohorts.push(CohortState {
            id: cohort_id,
            household: make_household(cohort_id, population),
            dispensary: DispensaryQueue::new(care_capacity),
            housing: Asset::new(
                EntityId::new(EntityType::Asset, cohort_id.number),
                AssetType::Building,
                INITIAL_HOUSING_STATES[index],
                0,
                1,
                EntityId::new(EntityType::Firm, cohort_id.number),
                1,
            ),
            daily_care_demand: care_capacity,
            current_inputs: InputValues::default(),
            stats: CohortStats::default(),
        });
    }

    let total_daily_demand: u32 = POPULATIONS.iter().sum();
    let mut import_history = Vec::with_capacity(DAYS as usize);
    let mut stock_history = Vec::with_capacity(DAYS as usize);
    let mut full_coverage_history = Vec::with_capacity(DAYS as usize);
    let mut market_stockout_days = 0;

    for day in 0..DAYS {
        market.process_all_markets(day);
        let displayed_price = market
            .get_market(FOOD_GOOD_ID)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent"))?
            .displayed_price;
        salary.mettre_a_jour_panier(
            PanierComposante::Nourriture,
            displayed_price.saturating_mul(10),
        );

        let available_before_sales = market
            .get_market(FOOD_GOOD_ID)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent"))?
            .stock;
        stock_history.push(available_before_sales);
        full_coverage_history.push(available_before_sales >= total_daily_demand);

        for (index, cohort) in cohorts.iter_mut().enumerate() {
            let population = POPULATIONS[index];
            let food_needed = population.saturating_sub(cohort.household.total_food());
            let mut was_rationed = false;
            if food_needed > 0 {
                let market_state = market.get_market_mut(FOOD_GOOD_ID).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent")
                })?;
                if market_state.stock > 0 {
                    if let Some((purchased, _)) =
                        market_state.buy(cohort.household.id, food_needed, u64::MAX)
                    {
                        cohort.household.add_food(FoodQuality::Base, purchased);
                        was_rationed = purchased < food_needed;
                    } else {
                        was_rationed = true;
                    }
                } else {
                    was_rationed = true;
                }
            }

            let u = cohort.household.calculate_u(population);
            cohort.household.consume_food(day);

            for patient_index in 0..cohort.daily_care_demand {
                cohort.dispensary.add_patient(DispensaryPatient {
                    individual_id: EntityId::new(
                        EntityType::NamedIndividual,
                        cohort.id.number * 1_000_000
                            + day as u64 * u64::from(cohort.daily_care_demand)
                            + u64::from(patient_index),
                    ),
                    priority: 50,
                    arrival_day: day,
                    care_type: CareType::Basic,
                });
            }
            let care_rate = cohort.dispensary.calculate_service_rate();
            cohort.dispensary.process_day(day);
            let housing_rate = cohort.housing.calculate_maintenance_rate(day);
            let q = calculate_q(&[
                (ServiceComponent::Soins, care_rate),
                (ServiceComponent::Logements, housing_rate),
            ]);
            let p = salary
                .entites
                .get(&cohort.id)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "cohorte salariale absente")
                })?
                .calculate_p(None);

            let values = InputValues { u, p, q };
            cohort.current_inputs = values;
            if was_rationed {
                cohort.stats.days_rationed += 1;
            }
            if tracker.schedule_if_changed(
                &mut event_queue,
                input_event_id(day, index),
                cohort.id,
                values,
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
            )? {
                cohort.stats.input_changed_events += 1;
            }
        }

        while let Some(event) = event_queue.pop() {
            salary.appliquer_evenement(&event)?;
        }

        let next_day = day + 1;
        for cohort in &mut cohorts {
            let state = &mut salary
                .entites
                .get_mut(&cohort.id)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "cohorte salariale absente")
                })?
                .griefs;
            state.advance_to(Timestamp::new(next_day, 0))?;
            cohort.stats.observe(
                cohort.current_inputs.q,
                state.griefs_points(),
                cohort.current_inputs.u,
            );
        }

        let stock = market
            .get_market(FOOD_GOOD_ID)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent"))?
            .stock;
        if stock == 0 {
            market_stockout_days += 1;
        }

        let import_quantity = if (SHOCK_START_DAY..SHOCK_END_DAY).contains(&day) {
            DAILY_IMPORT_DURING_SHOCK
        } else {
            BASE_IMPORT_PER_DAY
        };
        if market
            .order_import(FOOD_GOOD_ID, import_quantity, day)
            .is_none()
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "prix de référence alimentaire absent",
            )
            .into());
        }
        import_history.push(import_quantity);
    }

    let cohort_stats = cohorts
        .into_iter()
        .map(|cohort| (cohort.id, cohort.stats))
        .collect();
    Ok(ScenarioReport {
        cohorts: cohort_stats,
        import_history,
        stock_history,
        full_coverage_history,
        market_stockout_days,
    })
}

fn print_report(report: &ScenarioReport) {
    println!("=== quartier_10ans — graine {SEED}, {DAYS} jours ===");
    println!(
        "Choc 2 : import ÷2 du jour {SHOCK_START_DAY} au jour {} inclus",
        SHOCK_END_DAY - 1
    );
    println!();
    println!(
        "Cohorte | q min | q moyen | q max | Jours G=0 | Jours 0<G<100 | Jours G=100 | InputChanged | Jours u>0 | Jours rationnés"
    );
    println!(
        "--------|------:|--------:|------:|----------:|---------------:|------------:|-------------:|----------:|---------------:"
    );

    for (cohort_id, stats) in &report.cohorts {
        let q_mean = stats.mean_q_tenths();
        println!(
            "{:>7} | {:>5} | {:>3}.{:01} | {:>5} | {:>10} | {:>15} | {:>12} | {:>13} | {:>10} | {:>13}",
            cohort_id.number,
            stats.q_min,
            q_mean / 10,
            q_mean % 10,
            stats.q_max,
            stats.days_g_zero,
            stats.days_g_partial,
            stats.days_g_full,
            stats.input_changed_events,
            stats.days_without_food_coverage,
            stats.days_rationed
        );
    }
    println!();
    println!(
        "Jours de rupture du stock du marché : {} / {}",
        report.market_stockout_days, DAYS
    );
    let first_recovered_day = report.full_coverage_history[SHOCK_END_DAY as usize..]
        .iter()
        .position(|&covered| covered)
        .map(|offset| offset + SHOCK_END_DAY as usize);
    match first_recovered_day {
        Some(day) => println!(
            "Importations : {} / jour avant le choc, {} / jour pendant, {} / jour après ; couverture complète rétablie au jour {} (stock avant achats : {}).",
            report.import_history[199],
            report.import_history[SHOCK_START_DAY as usize],
            report.import_history[SHOCK_END_DAY as usize],
            day,
            report.stock_history[day]
        ),
        None => println!(
            "Importations revenues à {} / jour au jour {}, mais la couverture complète n'a pas été rétablie.",
            report.import_history[SHOCK_END_DAY as usize],
            SHOCK_END_DAY
        ),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let report = run_scenario()?;
    print_report(&report);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vestiges_horizons::{Event, EventData, EVENT_TYPE_INPUT_CHANGED};

    #[test]
    fn test_3_food_import_shock_causes_shortage_and_recovery() {
        let report = run_scenario().unwrap();
        assert_eq!(report.import_history[199], BASE_IMPORT_PER_DAY);
        assert!(
            report.import_history[SHOCK_START_DAY as usize..SHOCK_END_DAY as usize]
                .iter()
                .all(|&quantity| quantity == DAILY_IMPORT_DURING_SHOCK)
        );
        assert_eq!(
            report.import_history[SHOCK_END_DAY as usize],
            BASE_IMPORT_PER_DAY
        );

        let first_cohort = report.cohorts.values().next().unwrap();
        assert!(first_cohort.u_history[..SHOCK_START_DAY as usize]
            .iter()
            .all(|&u| u == 0));
        let shortage_cohort = report
            .cohorts
            .values()
            .find(|stats| {
                stats.u_history[SHOCK_START_DAY as usize..SHOCK_END_DAY as usize]
                    .iter()
                    .any(|&u| u > 0)
            })
            .expect("le choc doit faire apparaître des besoins alimentaires non servis");
        assert!(
            report.market_stockout_days > 0,
            "le choc doit provoquer des jours de rupture de stock"
        );

        let recovery_day = report.full_coverage_history[SHOCK_END_DAY as usize..]
            .iter()
            .position(|&covered| covered)
            .map(|offset| offset + SHOCK_END_DAY as usize)
            .expect("la couverture de la demande doit revenir après le choc");
        assert_eq!(shortage_cohort.u_history[recovery_day], 0);
        assert!(report.stock_history[recovery_day] >= POPULATIONS.iter().sum());
    }

    fn run_permuted_events(order: &[usize]) -> (Vec<String>, Vec<(EntityId, i64, u16, u16, u16)>) {
        let cohort_a = EntityId::new(EntityType::Cohort, 1);
        let cohort_b = EntityId::new(EntityType::Cohort, 2);
        let events = [
            Event::new(
                Id::new(11),
                0,
                0,
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_a,
                    u: 100,
                    p: 0,
                    q: 500,
                },
            ),
            Event::new(
                Id::new(12),
                0,
                0,
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_b,
                    u: 0,
                    p: 250,
                    q: 750,
                },
            ),
            Event::new(
                Id::new(13),
                0,
                2,
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_a,
                    u: 300,
                    p: 0,
                    q: 500,
                },
            ),
            Event::new(
                Id::new(14),
                1,
                0,
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_b,
                    u: 500,
                    p: 250,
                    q: 500,
                },
            ),
        ];
        let mut queue = EventQueue::new();
        for &index in order {
            queue.push(events[index].clone());
        }

        let mut manager = SalaireReelEtGriefsManager::new();
        manager.ajouter_entite(cohort_a);
        manager.ajouter_entite(cohort_b);
        let mut journal = Vec::new();
        while let Some(event) = queue.pop() {
            journal.push(format!("{:?}|{:?}", event.canonical_order(), event.data));
            manager.appliquer_evenement(&event).unwrap();
        }

        for state in manager.entites.values_mut() {
            state.griefs.advance_to(Timestamp::new(2, 0)).unwrap();
        }
        let states = manager
            .entites
            .iter()
            .map(|(&id, state)| {
                (
                    id,
                    state.griefs.griefs_micro_points(),
                    state.griefs.u(),
                    state.griefs.p(),
                    state.griefs.q(),
                )
            })
            .collect();
        (journal, states)
    }

    #[test]
    fn test_7_event_insertion_permutation_preserves_state_and_journal() {
        let forward = run_permuted_events(&[0, 1, 2, 3]);
        let reversed = run_permuted_events(&[3, 2, 1, 0]);
        assert_eq!(forward, reversed);
    }
}
