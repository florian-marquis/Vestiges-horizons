use std::collections::BTreeMap;
use std::error::Error;
use std::io;

use vestiges_horizons::asset::{Asset, AssetType};
use vestiges_horizons::entities::{EntityId, EntityManager, EntityType};
use vestiges_horizons::health::{CareType, DispensaryPatient, DispensaryQueue};
use vestiges_horizons::household::{FoodQuality, Household, HousingQuality};
use vestiges_horizons::inputs::{
    calculate_q, calculate_u_after_meal, InputTracker, InputValues, ServiceComponent,
};
use vestiges_horizons::market::{GoodType, MarketManager};
use vestiges_horizons::salary::{PanierComposante, SalaireReelEtGriefsManager};
use vestiges_horizons::{EventQueue, Id, Phase, Priority, Timestamp};

const SEED: u64 = 42;
const DAYS: i64 = 3_650;
const FOOD_GOOD_ID: u64 = 10;
const INITIAL_FOOD_STOCK: u32 = 2_000;
const DAILY_IMPORT: u32 = 500;
const MAX_FOOD_STOCK: u32 = 2_500;
const BASE_FOOD_PRICE: u64 = 100;
const MAX_FOOD_PRICE: u64 = 600;
// Repricing runs daily: multiplicative rise of +25%/day below the five-day stock cap
// (floor(P * 125 / 100), capped at 600), unchanged linear fall of -10/day at full stock.
const FOOD_PRICE_STEP: u64 = 10;
const SHOCK_START_DAY: i64 = 200;
const SHOCK_END_DAY: i64 = 290;
const SHOCK_ORDER_DELAY: i64 = 3;
const POPULATIONS: [u32; 5] = [60, 80, 100, 120, 140];
const HOURLY_WAGES: [u64; 5] = [35, 40, 50, 60, 70];
const INITIAL_HOUSING_STATES: [u32; 5] = [600, 700, 800, 900, 1_000];
const MAX_GRIEFS_QMICRO: i64 = 400_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenarioKind {
    S0,
    S1,
    S2,
    S3,
    S4,
}

impl ScenarioKind {
    fn label(self) -> &'static str {
        match self {
            Self::S0 => "S0 nominal sans choc",
            Self::S1 => "S1 imports divises par deux",
            Self::S2 => "S2 imports interrompus",
            Self::S3 => "S3 soins sous-dimensionnes",
            Self::S4 => "S4 degradation longue",
        }
    }

    fn arrival_quantity(self, arrival_day: i64) -> u32 {
        match self {
            Self::S1 if (SHOCK_START_DAY..SHOCK_END_DAY).contains(&arrival_day) => 250,
            Self::S2 if (SHOCK_START_DAY..SHOCK_END_DAY).contains(&arrival_day) => 0,
            Self::S4 if arrival_day >= 1_825 => {
                let reduction = ((arrival_day - 1_825) * 150 / 1_825) as u32;
                DAILY_IMPORT.saturating_sub(reduction)
            }
            _ => DAILY_IMPORT,
        }
    }

    fn has_worn_housing(self) -> bool {
        self == Self::S4
    }
}

#[derive(Debug, Default)]
struct CohortStats {
    q_min: u16,
    q_max: u16,
    q_sum: u64,
    q_samples: u64,
    p_day_zero: Option<u16>,
    budget_per_person: Option<u64>,
    food_price_day_zero: Option<u64>,
    days_g_zero: u64,
    days_g_partial: u64,
    days_g_full: u64,
    days_rationed: u64,
    days_u_positive: u64,
    u_max: u16,
    g_max_qmicro: i64,
    g_day_365: Option<i64>,
    g_day_1825: Option<i64>,
    g_day_3650: Option<i64>,
    u_history: Vec<u16>,
}

impl CohortStats {
    fn observe(&mut self, q: u16, g_qmicro: i64, u: u16, day_count: i64) {
        if self.q_samples == 0 {
            self.q_min = q;
        } else {
            self.q_min = self.q_min.min(q);
        }
        self.q_max = self.q_max.max(q);
        self.q_sum += u64::from(q);
        self.q_samples += 1;
        self.u_max = self.u_max.max(u);
        self.g_max_qmicro = self.g_max_qmicro.max(g_qmicro);
        match g_qmicro {
            0 => self.days_g_zero += 1,
            1..=399_999_999 => self.days_g_partial += 1,
            MAX_GRIEFS_QMICRO => self.days_g_full += 1,
            _ => unreachable!("grief state is bounded by the core"),
        }
        if u > 0 {
            self.days_u_positive += 1;
        }
        if day_count == 365 {
            self.g_day_365 = Some(g_qmicro);
        } else if day_count == 1_825 {
            self.g_day_1825 = Some(g_qmicro);
        } else if day_count == DAYS {
            self.g_day_3650 = Some(g_qmicro);
        }
        self.u_history.push(u);
    }

    fn mean_q_tenths(&self) -> u64 {
        if self.q_samples == 0 {
            0
        } else {
            self.q_sum.saturating_mul(10) / self.q_samples
        }
    }
}

#[derive(Debug)]
struct CohortState {
    index: usize,
    id: EntityId,
    household: Household,
    housing: Asset,
    daily_care_demand: u32,
    care_capacity: u32,
    current_inputs: InputValues,
    stats: CohortStats,
}

#[derive(Debug)]
struct ScenarioReport {
    kind: ScenarioKind,
    cohorts: Vec<(EntityId, CohortStats)>,
    import_history: Vec<u32>,
    price_history: Vec<u64>,
    market_stockout_days: u64,
    early_stop_day: Option<i64>,
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

fn input_event_id(day: i64, cohort_id: EntityId) -> Id {
    Id::new(1 + day as u64 * POPULATIONS.len() as u64 + cohort_id.number)
}

fn payable_rations(budget: u64, price: u64, need: u32) -> Result<u32, io::Error> {
    if price == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "le prix alimentaire doit rester strictement positif",
        ));
    }
    Ok(need.min((budget / price).min(u64::from(u32::MAX)) as u32))
}

fn allocate_rations(payables: &[(EntityId, u32)], stock: u32) -> BTreeMap<EntityId, u32> {
    let mut allocation: BTreeMap<EntityId, u32> = payables.iter().map(|(id, _)| (*id, 0)).collect();
    let total_payable: u128 = payables.iter().map(|(_, amount)| u128::from(*amount)).sum();
    if total_payable == 0 {
        return allocation;
    }
    if total_payable <= u128::from(stock) {
        return payables.iter().map(|(id, amount)| (*id, *amount)).collect();
    }

    let mut remainders = Vec::with_capacity(payables.len());
    let mut base_total = 0u32;
    for (id, payable) in payables {
        let numerator = u128::from(stock) * u128::from(*payable);
        let base = (numerator / total_payable) as u32;
        let remainder = numerator % total_payable;
        allocation.insert(*id, base);
        base_total += base;
        remainders.push((*id, remainder));
    }

    remainders.sort_by(|(left_id, left_remainder), (right_id, right_remainder)| {
        right_remainder
            .cmp(left_remainder)
            .then_with(|| left_id.cmp(right_id))
    });
    let remaining = stock.saturating_sub(base_total) as usize;
    for (id, _) in remainders.into_iter().take(remaining) {
        if let Some(amount) = allocation.get_mut(&id) {
            *amount += 1;
        }
    }
    allocation
}

fn daily_care_rate(cohort_id: EntityId, day: i64, demand: u32, capacity: u32) -> u16 {
    let mut queue = DispensaryQueue::new(capacity);
    for patient_index in 0..demand {
        queue.add_patient(DispensaryPatient {
            individual_id: EntityId::new(
                EntityType::NamedIndividual,
                cohort_id.number * 1_000_000
                    + day as u64 * u64::from(demand)
                    + u64::from(patient_index),
            ),
            priority: 50,
            arrival_day: day,
            care_type: CareType::Basic,
        });
    }
    let rate = queue.calculate_service_rate();
    queue.process_day(day);
    rate
}

fn scenario_care(kind: ScenarioKind, cohort: &CohortState, day: i64) -> (u32, u32) {
    match kind {
        ScenarioKind::S3 if cohort.index == 0 => (30, 6),
        ScenarioKind::S4 => {
            let capacity =
                (u64::from(POPULATIONS[cohort.index]) * (DAYS - day) as u64 / DAYS as u64) as u32;
            (POPULATIONS[cohort.index], capacity)
        }
        _ => (cohort.daily_care_demand, cohort.care_capacity),
    }
}

fn schedule_initial_imports(market: &mut MarketManager) -> Result<(), io::Error> {
    for arrival_day in 0..SHOCK_ORDER_DELAY {
        if market
            .order_import(FOOD_GOOD_ID, DAILY_IMPORT, arrival_day - SHOCK_ORDER_DELAY)
            .is_none()
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "prix de référence alimentaire absent",
            ));
        }
    }
    Ok(())
}

/// Reprice quotidien du scénario (Lot G1) : sous 5 jours de couverture, la hausse
/// est multiplicative (`+25 %/jour`, troncature entière, plafond `MAX_FOOD_PRICE`) ;
/// à couverture pleine, la descente reste linéaire (`-10/jour`, plancher `BASE_FOOD_PRICE`).
fn reprice_quotidien(stock: u32, displayed_price: u64) -> u64 {
    if stock < MAX_FOOD_STOCK {
        (displayed_price.saturating_mul(125) / 100).min(MAX_FOOD_PRICE)
    } else {
        displayed_price
            .saturating_sub(FOOD_PRICE_STEP)
            .max(BASE_FOOD_PRICE)
    }
}

fn run_scenario(kind: ScenarioKind) -> Result<ScenarioReport, Box<dyn Error>> {
    let mut entities = EntityManager::new(SEED);
    let mut salary = SalaireReelEtGriefsManager::new();
    let mut market = MarketManager::new();
    let mut tracker = InputTracker::default();
    let mut event_queue = EventQueue::new();
    let mut cohorts = Vec::with_capacity(POPULATIONS.len());

    market.create_market(
        EntityId::new(EntityType::Market, FOOD_GOOD_ID),
        GoodType::Food,
        FOOD_GOOD_ID,
        BASE_FOOD_PRICE,
        INITIAL_FOOD_STOCK,
        None,
        (MAX_FOOD_STOCK, MAX_FOOD_STOCK - 1),
        FOOD_PRICE_STEP,
        (BASE_FOOD_PRICE, MAX_FOOD_PRICE),
        0,
    );
    market.add_reference_price(FOOD_GOOD_ID, BASE_FOOD_PRICE, MAX_FOOD_STOCK);
    schedule_initial_imports(&mut market)?;

    for (index, population) in POPULATIONS.iter().copied().enumerate() {
        let cohort_id = entities.create_cohort(u64::from(population));
        salary.ajouter_entite(cohort_id);
        salary.mettre_a_jour_salaire_horaire(cohort_id, HOURLY_WAGES[index]);
        salary.mettre_a_jour_heures(cohort_id, 8);

        let income_tax = salary
            .entites
            .get(&cohort_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cohorte salariale absente"))?
            .impot_journalier();
        salary.mettre_a_jour_panier_entite(cohort_id, PanierComposante::Logement, 110)?;
        salary.mettre_a_jour_panier_entite(cohort_id, PanierComposante::Eau, 20)?;
        salary.mettre_a_jour_panier_entite(cohort_id, PanierComposante::Fiscalite, income_tax)?;

        let care_capacity = match (kind, index) {
            (ScenarioKind::S3, 0) => 6,
            _ => population,
        };
        let daily_care_demand = match (kind, index) {
            (ScenarioKind::S3, 0) => 30,
            _ => population,
        };
        cohorts.push(CohortState {
            index,
            id: cohort_id,
            household: make_household(cohort_id, population),
            housing: Asset::new(
                EntityId::new(EntityType::Asset, cohort_id.number),
                AssetType::Building,
                INITIAL_HOUSING_STATES[index],
                0,
                if kind.has_worn_housing() { 1 } else { 0 },
                EntityId::new(EntityType::Firm, cohort_id.number),
                1,
            ),
            daily_care_demand,
            care_capacity,
            current_inputs: InputValues::default(),
            stats: CohortStats::default(),
        });
    }

    let mut import_history = Vec::with_capacity(DAYS as usize);
    let mut price_history = Vec::with_capacity(DAYS as usize);
    let mut market_stockout_days = 0;
    let mut early_stop_day = None;

    for day in 0..DAYS {
        let next_arrival_day = day + SHOCK_ORDER_DELAY;
        if next_arrival_day < DAYS
            && market
                .order_import(FOOD_GOOD_ID, kind.arrival_quantity(next_arrival_day), day)
                .is_none()
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "prix de référence alimentaire absent",
            )
            .into());
        }

        market.update_reference_prices(day);
        let arrivals = market.process_import_arrivals(day);
        let arrived_quantity: u32 = arrivals
            .iter()
            .filter(|(good_id, _)| *good_id == FOOD_GOOD_ID)
            .map(|(_, quantity)| *quantity)
            .sum();
        if arrived_quantity != kind.arrival_quantity(day) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "arrivage inattendu au jour {day}: {arrived_quantity}, attendu {}",
                    kind.arrival_quantity(day)
                ),
            )
            .into());
        }

        let market_state = market
            .get_market_mut(FOOD_GOOD_ID)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent"))?;
        market_state.stock = market_state.stock.min(MAX_FOOD_STOCK);
        market_state.displayed_price =
            reprice_quotidien(market_state.stock, market_state.displayed_price);
        let displayed_price = market_state.displayed_price;
        price_history.push(displayed_price);
        import_history.push(arrived_quantity);

        let mut payables = Vec::with_capacity(cohorts.len());
        let mut budgets = BTreeMap::new();
        let opening_household_food: BTreeMap<EntityId, u32> = cohorts
            .iter()
            .map(|cohort| (cohort.id, cohort.household.total_food()))
            .collect();
        for cohort in &mut cohorts {
            salary.mettre_a_jour_panier_entite(
                cohort.id,
                PanierComposante::Nourriture,
                displayed_price,
            )?;
            let state = salary.entites.get(&cohort.id).ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "cohorte salariale absente")
            })?;
            let per_person_budget = state.budget_nourriture(day);
            let total_budget = u128::from(per_person_budget)
                .checked_mul(u128::from(POPULATIONS[cohort.index]))
                .and_then(|value| u64::try_from(value).ok())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "budget de cohorte débordé")
                })?;
            let need = POPULATIONS[cohort.index].saturating_sub(cohort.household.total_food());
            let payable = payable_rations(total_budget, displayed_price, need)?;
            budgets.insert(cohort.id, total_budget);
            payables.push((cohort.id, payable));

            if day == 0 {
                cohort.stats.p_day_zero = Some(state.calculate_p(None));
                cohort.stats.budget_per_person = Some(per_person_budget);
                cohort.stats.food_price_day_zero = Some(displayed_price);
            }
        }

        let available_stock = market
            .get_market(FOOD_GOOD_ID)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent"))?
            .stock;
        let allocations = allocate_rations(&payables, available_stock);
        let mut purchased_today = 0u32;
        let mut purchases_by_cohort = BTreeMap::new();
        for cohort in &mut cohorts {
            let amount = allocations.get(&cohort.id).copied().unwrap_or(0);
            if amount == 0 {
                continue;
            }
            let cost = u64::from(amount)
                .checked_mul(displayed_price)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "coût alimentaire débordé")
                })?;
            let budget = budgets[&cohort.id];
            if cost > budget {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "allocation alimentaire supérieure au budget",
                )
                .into());
            }
            let market_state = market.get_market_mut(FOOD_GOOD_ID).ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent")
            })?;
            let (purchased, actual_cost) = market_state
                .buy(cohort.household.id, amount, budget)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "achat alimentaire impossible")
                })?;
            if purchased != amount || actual_cost != cost {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "achat alimentaire différent de l'allocation déterministe",
                )
                .into());
            }
            purchased_today += purchased;
            purchases_by_cohort.insert(cohort.id, purchased);
            cohort.household.add_food(FoodQuality::Base, purchased);
        }
        let remaining_market_stock = market
            .get_market(FOOD_GOOD_ID)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent"))?
            .stock;
        if remaining_market_stock + purchased_today != available_stock {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "la sortie du stock de marché ne correspond pas aux achats",
            )
            .into());
        }

        for cohort in &mut cohorts {
            let need = POPULATIONS[cohort.index];
            let rations_eaten = cohort.household.consume_food_rations(day);
            let food_balance_before = u64::from(opening_household_food[&cohort.id])
                + u64::from(purchases_by_cohort.get(&cohort.id).copied().unwrap_or(0));
            let food_balance_after =
                u64::from(rations_eaten) + u64::from(cohort.household.total_food());
            if food_balance_before != food_balance_after {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "la conservation des rations du foyer est violée",
                )
                .into());
            }
            let u = calculate_u_after_meal(rations_eaten, need);
            let (care_demand, care_capacity) = scenario_care(kind, cohort, day);
            let care_rate = daily_care_rate(cohort.id, day, care_demand, care_capacity);
            let housing_rate = cohort.housing.calculate_maintenance_rate(day);
            let q = calculate_q(&[
                (ServiceComponent::Soins, care_rate),
                (ServiceComponent::Logements, housing_rate),
            ]);
            let salary_state = salary.entites.get(&cohort.id).ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "cohorte salariale absente")
            })?;
            let p = salary_state.calculate_p(None);
            let values = InputValues { u, p, q };
            cohort.current_inputs = values;
            let rationed = rations_eaten < need;
            if rationed {
                cohort.stats.days_rationed += 1;
            }
            let _event_scheduled = tracker.schedule_if_changed(
                &mut event_queue,
                input_event_id(day, cohort.id),
                cohort.id,
                values,
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
            )?;
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
                state.g_qmicro(),
                cohort.current_inputs.u,
                next_day,
            );
        }

        if market
            .get_market(FOOD_GOOD_ID)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "marché alimentaire absent"))?
            .stock
            == 0
        {
            market_stockout_days += 1;
        }

        if kind == ScenarioKind::S0
            && next_day < 1_095
            && cohorts.iter().any(|cohort| {
                salary
                    .entites
                    .get(&cohort.id)
                    .map(|state| state.griefs.g_qmicro() == MAX_GRIEFS_QMICRO)
                    .unwrap_or(false)
            })
        {
            early_stop_day = Some(next_day);
            break;
        }
    }

    Ok(ScenarioReport {
        kind,
        cohorts: cohorts
            .into_iter()
            .map(|cohort| (cohort.id, cohort.stats))
            .collect(),
        import_history,
        price_history,
        market_stockout_days,
        early_stop_day,
    })
}

fn format_qmicro(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_hundredths(value: u128) -> String {
    format!("{}.{:02}", value / 100, value % 100)
}

fn print_report(report: &ScenarioReport) {
    let maximum_price = report.price_history.iter().copied().max().unwrap_or(0);
    println!(
        "=== {} — graine {SEED}, {} jours simulés ===",
        report.kind.label(),
        report.import_history.len()
    );
    println!(
        "Stock initial {INITIAL_FOOD_STOCK}, imports nominaux {DAILY_IMPORT}/j, capacité {MAX_FOOD_STOCK}; prix max {maximum_price}; jours de rupture marché {}",
        report.market_stockout_days
    );
    if let Some(day) = report.early_stop_day {
        println!("ARRÊT OBLIGATOIRE S0 : G=100 atteint au jour {day} (< 1095).");
    }
    println!(
        "idx | p0 | budget/P | u max | G365 qµpt | G1825 qµpt | G3650 qµpt | Gmax qµpt | G=0 | partiel | G=100 | q min/moy/max | j rationnés"
    );
    for (index, (_, stats)) in report.cohorts.iter().enumerate() {
        let ratio_hundredths = stats
            .budget_per_person
            .zip(stats.food_price_day_zero)
            .map(|(budget, price)| u128::from(budget) * 100 / u128::from(price))
            .unwrap_or(0);
        let q_mean = stats.mean_q_tenths();
        println!(
            "{index:>3} | {:>3} | {:>7} | {:>5} | {:>10} | {:>11} | {:>11} | {:>10} | {:>4} | {:>7} | {:>6} | {:>4}/{:>5}.{:01}/{:>4} | {:>11}",
            stats.p_day_zero.unwrap_or(0),
            format_hundredths(ratio_hundredths),
            stats.u_max,
            format_qmicro(stats.g_day_365),
            format_qmicro(stats.g_day_1825),
            format_qmicro(stats.g_day_3650),
            stats.g_max_qmicro,
            stats.days_g_zero,
            stats.days_g_partial,
            stats.days_g_full,
            stats.q_min,
            q_mean / 10,
            q_mean % 10,
            stats.q_max,
            stats.days_rationed
        );
    }
}

fn print_shock_u(report: &ScenarioReport) {
    if !matches!(report.kind, ScenarioKind::S1 | ScenarioKind::S2) {
        return;
    }
    println!("Série u jour par jour, fenêtre d'arrivages effectifs 200..=289 :");
    for cohort_index in [0usize, 2usize] {
        let stats = &report.cohorts[cohort_index].1;
        let values = stats
            .u_history
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                let day = *index as i64;
                (SHOCK_START_DAY..SHOCK_END_DAY).contains(&day)
            })
            .map(|(index, value)| format!("{index}:{value}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("Cohorte {cohort_index}: {values}");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let s0 = run_scenario(ScenarioKind::S0)?;
    print_report(&s0);
    if s0.early_stop_day.is_some() {
        return Ok(());
    }

    for kind in [
        ScenarioKind::S1,
        ScenarioKind::S2,
        ScenarioKind::S3,
        ScenarioKind::S4,
    ] {
        let report = run_scenario(kind)?;
        print_report(&report);
        print_shock_u(&report);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vestiges_horizons::{Event, EventData, EVENT_TYPE_INPUT_CHANGED};

    #[test]
    fn rationing_uses_largest_remainder_and_stable_entity_id_ties() {
        let first = EntityId::new(EntityType::Cohort, 1);
        let second = EntityId::new(EntityType::Cohort, 2);
        let forward = allocate_rations(&[(second, 3), (first, 3)], 5);
        let reversed = allocate_rations(&[(first, 3), (second, 3)], 5);
        assert_eq!(forward, reversed);
        assert_eq!(forward[&first], 3);
        assert_eq!(forward[&second], 2);
        assert_eq!(forward.values().sum::<u32>(), 5);
    }

    #[test]
    fn payable_rations_respect_each_cohorts_aggregate_budget() {
        assert_eq!(payable_rations(122 * 60, 150, 60).unwrap(), 48);
        assert_eq!(payable_rations(374 * 140, 150, 140).unwrap(), 140);
        assert_eq!(
            payable_rations(100, 0, 1).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn market_reprice_tracks_full_stock_and_reverses_to_base_price() {
        // Couverture sous 5 jours : montée multiplicative +25 %/jour (troncature entière, plafond 600).
        let mut price = BASE_FOOD_PRICE;
        for expected in [125, 156, 195, 243, 303, 378, 472, 590] {
            price = reprice_quotidien(MAX_FOOD_STOCK - 1, price);
            assert_eq!(price, expected);
        }
        price = reprice_quotidien(MAX_FOOD_STOCK - 1, price);
        assert_eq!(price, MAX_FOOD_PRICE);
        price = reprice_quotidien(MAX_FOOD_STOCK - 1, price);
        assert_eq!(price, MAX_FOOD_PRICE, "le plafond 600 est stable");

        // Couverture pleine : descente inchangée -10/jour, plancher 100.
        while price > BASE_FOOD_PRICE {
            price = reprice_quotidien(MAX_FOOD_STOCK, price);
        }
        assert_eq!(price, BASE_FOOD_PRICE);
        price = reprice_quotidien(MAX_FOOD_STOCK, price);
        assert_eq!(price, BASE_FOOD_PRICE, "le plancher 100 est stable");
        price = reprice_quotidien(MAX_FOOD_STOCK - 1, price);
        assert_eq!(price, 125);
    }

    #[test]
    fn shock_arrivals_are_applied_to_the_locked_days() {
        assert_eq!(ScenarioKind::S1.arrival_quantity(199), 500);
        assert_eq!(ScenarioKind::S1.arrival_quantity(200), 250);
        assert_eq!(ScenarioKind::S1.arrival_quantity(289), 250);
        assert_eq!(ScenarioKind::S1.arrival_quantity(290), 500);
        assert_eq!(ScenarioKind::S2.arrival_quantity(200), 0);
        assert_eq!(ScenarioKind::S2.arrival_quantity(289), 0);
        assert_eq!(ScenarioKind::S2.arrival_quantity(290), 500);
        assert_eq!(ScenarioKind::S4.arrival_quantity(1_825), 500);
        assert_eq!(ScenarioKind::S4.arrival_quantity(3_650), 350);
    }

    #[test]
    fn undersized_s3_dispensary_produces_the_locked_service_and_q_rates() {
        let cohort_id = EntityId::new(EntityType::Cohort, 1);
        assert_eq!(daily_care_rate(cohort_id, 0, 30, 6), 200);
        assert_eq!(
            calculate_q(&[
                (ServiceComponent::Soins, 200),
                (ServiceComponent::Logements, 600),
            ]),
            400
        );
    }

    #[test]
    fn nominal_run_serves_every_cohort_at_base_price_without_griefs() {
        let report = run_scenario(ScenarioKind::S0).unwrap();
        assert_eq!(report.early_stop_day, None);
        assert_eq!(report.import_history.len(), DAYS as usize);
        assert!(report
            .import_history
            .iter()
            .all(|&quantity| quantity == DAILY_IMPORT));
        assert!(report
            .price_history
            .iter()
            .all(|&price| price == BASE_FOOD_PRICE));
        for (_, stats) in &report.cohorts {
            assert_eq!(stats.p_day_zero, Some(0));
            assert_eq!(stats.u_max, 0);
            assert_eq!(stats.days_rationed, 0);
            assert_eq!(stats.g_max_qmicro, 0);
        }
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
    fn event_insertion_permutation_preserves_state_and_journal() {
        let forward = run_permuted_events(&[0, 1, 2, 3]);
        let reversed = run_permuted_events(&[3, 2, 1, 0]);
        assert_eq!(forward, reversed);
    }
}
