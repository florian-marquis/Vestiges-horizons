use std::collections::BTreeMap;
use std::error::Error;
use std::io;

use vestiges_horizons::asset::{Asset, AssetType};
use vestiges_horizons::entities::{CausalJournal, EntityId, EntityManager, EntityType};
use vestiges_horizons::griefs::GriefsState;
use vestiges_horizons::health::{CareType, DispensaryPatient, DispensaryQueue};
use vestiges_horizons::household::{FoodQuality, Household, HousingQuality};
use vestiges_horizons::inputs::{
    calculate_q, calculate_u_after_meal, InputTracker, InputValues, ServiceComponent,
};
use vestiges_horizons::market::{GoodType, MarketManager};
use vestiges_horizons::salary::{PanierComposante, SalaireReelEtGriefsManager};
use vestiges_horizons::strike::{
    delta_h_scaled, slots_between, StrikeEndCause, StrikeManager, StrikePhaseState,
    DEFAULT_ORGANISATION_MILLE, HAZARD_SCALE,
};
use vestiges_horizons::{
    Event, EventData, EventQueue, Id, Phase, Priority, Timestamp, EVENT_TYPE_GRIEF_BOUND_REACHED,
    EVENT_TYPE_INPUT_CHANGED, EVENT_TYPE_STRIKE_COOLDOWN_ENDED, EVENT_TYPE_STRIKE_ENDED,
    EVENT_TYPE_STRIKE_TRIGGERED,
};

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
    S0b,
    S1,
    S2,
    S3,
    S4,
}

impl ScenarioKind {
    fn label(self) -> &'static str {
        match self {
            Self::S0 => "S0 nominal sans choc",
            Self::S0b => "S0b nominal sans choc, q=500",
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
    /// Trajectoire (u, p, q) journalière par cohorte (indexée [cohorte][jour]),
    /// enregistrée sans modifier les calculs A2 — utilisée par les lectures B1.
    input_history: Vec<Vec<InputValues>>,
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
        ScenarioKind::S0b => {
            let population = POPULATIONS[cohort.index];
            (population.saturating_mul(2), population)
        }
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
        let housing_state = if kind == ScenarioKind::S0b {
            500
        } else {
            INITIAL_HOUSING_STATES[index]
        };
        cohorts.push(CohortState {
            index,
            id: cohort_id,
            household: make_household(cohort_id, population),
            housing: Asset::new(
                EntityId::new(EntityType::Asset, cohort_id.number),
                AssetType::Building,
                housing_state,
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
    let mut input_history: Vec<Vec<InputValues>> =
        cohorts.iter().map(|_| Vec::with_capacity(DAYS as usize)).collect();
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
            input_history[cohort.index].push(values);
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
        if kind == ScenarioKind::S0b
            && next_day < DAYS
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
        input_history,
    })
}

// ============================================================================
// TRANCHE B1 — Lectures de grève (Commit 5, spec T7-T8 tranche B1 §6.2)
// ============================================================================
//
// Pilotage 100 % événementiel : la trajectoire (u, p, q) A2 est rejouée via
// des événements InputChanged à Id déterministe (input_event_id), traités dans
// l'ordre canonique avec les événements de grève 71-74 encodés par
// encode_strike_event_id (bit 63 = 1, disjoint des ids InputChanged).
// Zéro f64/f32/exp/ln : l'intégrale attendue est en u128 via delta_h_scaled.

/// Observation d'une grève déclenchée pendant la lecture B1.
#[derive(Debug, Clone)]
struct StrikeObservation {
    strike_index: u64,
    start: Timestamp,
    duration_days: i64,
    end: Option<(Timestamp, StrikeEndCause)>,
    g_qmicro: i64,
}

/// Lecture de grève d'une cohorte sur l'horizon complet.
/// (champs lus uniquement par les tests -> allow(dead_code) hors test)
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug)]
struct CohortStrikeReading {
    index: usize,
    cohort_id: EntityId,
    o_mille: u16,
    strikes: Vec<StrikeObservation>,
    /// Intégrale analytique entière sur les fenêtres actives (hors OnStrike/Cooldown).
    expected_active_scaled: u128,
    /// Intégrale brute sur tout l'horizon [0, DAYS] (complément lisible).
    expected_brute_scaled: u128,
    g_day_365: Option<i64>,
    g_day_1825: Option<i64>,
    g_day_3650: Option<i64>,
    g_max_qmicro: i64,
}

/// Résultat de la lecture de grève d'un scénario.
#[derive(Debug)]
struct StrikeReading {
    kind: ScenarioKind,
    cohorts: Vec<CohortStrikeReading>,
    events_processed: u64,
    verify_i7: bool,
}

/// Miroir de lecture pour l'intégrale analytique attendue (u128, zéro flottant).
struct ExpectedMirror {
    last_ts: Timestamp,
    g_last: i64,
    slope_last: i64,
    expected_active: u128,
    expected_brute: u128,
    g_max: i64,
    g_day_365: Option<i64>,
    g_day_1825: Option<i64>,
    g_day_3650: Option<i64>,
}

impl ExpectedMirror {
    fn new() -> Self {
        ExpectedMirror {
            last_ts: Timestamp::new(0, 0),
            g_last: 0,
            slope_last: 0,
            expected_active: 0,
            expected_brute: 0,
            g_max: 0,
            g_day_365: None,
            g_day_1825: None,
            g_day_3650: None,
        }
    }

    /// Intègre le segment [last_ts, ts] dans les intégrales attendues.
    /// La fenêtre active (Accumulating/PendingNextSlot, hors OnStrike/Cooldown)
    /// est déterminée par `phase_before`.
    fn integrate(
        &mut self,
        phase_before: StrikePhaseState,
        o_mille: u16,
        ts: Timestamp,
        g_now: i64,
        slope_now: i64,
    ) {
        // Segment intégral tronqué à l'horizon de lecture [0, DAYS].
        let capped = if ts.day > DAYS { Timestamp::new(DAYS, 0) } else { ts };
        let k = slots_between(self.last_ts, capped);
        if k > 0 {
            let d = delta_h_scaled(self.g_last, self.slope_last, u64::from(o_mille), k);
            self.expected_brute += d;
            if phase_before == StrikePhaseState::Accumulating
                || phase_before == StrikePhaseState::PendingNextSlot
            {
                self.expected_active += d;
            }
        }
        self.last_ts = ts;
        self.g_last = g_now;
        self.slope_last = slope_now;
        // G observé uniquement dans l'horizon (comme A2, qui observe à next_day <= DAYS).
        if ts.day <= DAYS {
            self.g_max = self.g_max.max(g_now);
            if ts.day == 365 {
                self.g_day_365 = Some(g_now);
            }
            if ts.day == 1_825 {
                self.g_day_1825 = Some(g_now);
            }
            if ts.day == DAYS {
                self.g_day_3650 = Some(g_now);
            }
        }
    }

    /// Enregistre G à un jalon [365, 1_825, DAYS] franchi (même sans événement ce jour-là).
    fn record_milestone(&mut self, day: i64, g: i64) {
        self.g_max = self.g_max.max(g);
        match day {
            365 => self.g_day_365 = Some(g),
            1_825 => self.g_day_1825 = Some(g),
            DAYS => self.g_day_3650 = Some(g),
            _ => {}
        }
    }
}

/// Avance `gs` à `ts` en capturant G aux jalons [365, 1_825, DAYS] franchis,
/// sans jamais revenir en arrière.
fn advance_observed(
    gs: &mut GriefsState,
    m: &mut ExpectedMirror,
    ts: Timestamp,
) -> Result<(), Box<dyn Error>> {
    for milestone in [365i64, 1_825, DAYS] {
        let ms_ts = Timestamp::new(milestone, 0);
        if gs.last_update() < ms_ts && ts >= ms_ts {
            gs.advance_to(ms_ts)?;
            m.record_milestone(milestone, gs.g_qmicro());
        }
    }
    if gs.last_update() < ts {
        gs.advance_to(ts)?;
    }
    Ok(())
}

/// Rejoue la trajectoire A2 du scénario en pilotage 100 % événementiel et
/// produit la lecture de grève B1 (observé vs attendu analytique, détail des grèves).
fn run_strike_reading(kind: ScenarioKind) -> Result<StrikeReading, Box<dyn Error>> {
    // Trajectoire (u, p, q) A2 inchangée, enregistrée jour par jour.
    let report = run_scenario(kind)?;
    let cohort_ids: Vec<EntityId> = report.cohorts.iter().map(|(id, _)| *id).collect();
    let index_of = |id: EntityId| cohort_ids.iter().position(|x| *x == id).unwrap();

    let mut queue = EventQueue::new();
    let mut sm = StrikeManager::new(SEED);
    let mut journal = CausalJournal::new();
    let mut griefs: BTreeMap<EntityId, GriefsState> = BTreeMap::new();
    let mut mirrors: BTreeMap<EntityId, ExpectedMirror> = BTreeMap::new();
    let mut strikes: BTreeMap<EntityId, Vec<StrikeObservation>> = BTreeMap::new();

    for (ci, cid) in cohort_ids.iter().enumerate() {
        griefs.insert(*cid, GriefsState::new(*cid));
        mirrors.insert(*cid, ExpectedMirror::new());
        sm.cohort_state_mut(*cid);
        for day in 0..DAYS {
            let v = report.input_history[ci][day as usize];
            // Émission événementielle stricte : un InputChanged uniquement sur
            // changement réel des entrées (jour 0 ou v != v précédent).
            let changed = day == 0 || v != report.input_history[ci][(day - 1) as usize];
            if changed {
                queue.push(Event::new(
                    input_event_id(day, *cid),
                    day,
                    0,
                    Phase::Comptes,
                    Priority::new(0),
                    EVENT_TYPE_INPUT_CHANGED,
                    EventData::InputChanged {
                        entity_id: *cid,
                        u: v.u,
                        p: v.p,
                        q: v.q,
                    },
                ));
            }
        }
    }

    let mut events_processed = 0u64;

    // Garde-fou d'horizon : tout événement hors [0, DAYS + 100] est ignoré.
    // (Un k de franchissement géant peut déborder en jour négatif via add_slots ;
    // on ne traite que l'horizon de lecture, ce qui garantit la terminaison.)
    const HORIZON_MARGIN: i64 = 100;

    while let Some(ev) = queue.pop() {
        // add_slots est saturé en i128 (Commit 6) : aucun jour négatif n'est possible.
        assert!(ev.timestamp.day >= 0, "add_slots saturé : aucun jour négatif");
        if ev.timestamp.day > DAYS + HORIZON_MARGIN {
            // La file est ordonnée par timestamp : tout le reste est hors horizon.
            break;
        }
        events_processed += 1;
        match ev.event_type {
            EVENT_TYPE_INPUT_CHANGED => {
                if let EventData::InputChanged { entity_id, u, p, q } = ev.data {
                    let o = DEFAULT_ORGANISATION_MILLE[index_of(entity_id)];
                    let gs = griefs.get_mut(&entity_id).unwrap();
                    advance_observed(gs, mirrors.get_mut(&entity_id).unwrap(), ev.timestamp)?;
                    let g_now = gs.g_qmicro();
                    gs.set_inputs_at(u, p, q, ev.timestamp)?;
                    let new_slope = gs.slope_micro_points_per_day();
                    let phase_before = sm.phase_of(entity_id);
                    mirrors
                        .get_mut(&entity_id)
                        .unwrap()
                        .integrate(phase_before, o, ev.timestamp, g_now, new_slope);
                    sm.on_input_changed(entity_id, ev.timestamp, g_now, new_slope, o, &mut queue);
                    sm.journal_input_changed(
                        entity_id,
                        ev.timestamp,
                        ev.id.as_u64(),
                        u,
                        p,
                        q,
                        &mut journal,
                    );
                }
            }
            EVENT_TYPE_STRIKE_TRIGGERED => {
                if let EventData::StrikeTriggered { entity_id, draw_index, strike_index, schedule_version } = ev.data {
                    let o = DEFAULT_ORGANISATION_MILLE[index_of(entity_id)];
                    let gs = griefs.get_mut(&entity_id).unwrap();
                    advance_observed(gs, mirrors.get_mut(&entity_id).unwrap(), ev.timestamp)?;
                    let (g, sl) = (gs.g_qmicro(), gs.slope_micro_points_per_day());
                    let phase_before = sm.phase_of(entity_id);
                    mirrors
                        .get_mut(&entity_id)
                        .unwrap()
                        .integrate(phase_before, o, ev.timestamp, g, sl);
                    if sm.on_strike_triggered(entity_id, ev.timestamp, g, sl, draw_index, strike_index, schedule_version, &mut queue) {
                        let dur = sm.cohort_state(entity_id).unwrap().duration_days;
                        sm.journal_strike_triggered(entity_id, ev.timestamp, ev.id.as_u64(), g, &mut journal);
                        // On ne retient que les grèves démarrant dans l'horizon [0, DAYS].
                        if ev.timestamp.day <= DAYS {
                            strikes.entry(entity_id).or_default().push(StrikeObservation {
                                strike_index: strike_index + 1,
                                start: ev.timestamp,
                                duration_days: dur,
                                end: None,
                                g_qmicro: g,
                            });
                        }
                    }
                }
            }
            EVENT_TYPE_STRIKE_ENDED => {
                if let EventData::StrikeEnded { entity_id, strike_index, cause, schedule_version } = ev.data {
                    let o = DEFAULT_ORGANISATION_MILLE[index_of(entity_id)];
                    let gs = griefs.get_mut(&entity_id).unwrap();
                    advance_observed(gs, mirrors.get_mut(&entity_id).unwrap(), ev.timestamp)?;
                    let (g, sl) = (gs.g_qmicro(), gs.slope_micro_points_per_day());
                    let phase_before = sm.phase_of(entity_id);
                    mirrors
                        .get_mut(&entity_id)
                        .unwrap()
                        .integrate(phase_before, o, ev.timestamp, g, sl);
                    if sm.on_strike_ended(entity_id, ev.timestamp, g, sl, strike_index, cause, schedule_version, &mut queue) {
                        sm.journal_strike_ended(entity_id, ev.timestamp, ev.id.as_u64(), strike_index, cause, &mut journal);
                        if let Some(v) = strikes.get_mut(&entity_id) {
                            if let Some(obs) = v.iter_mut().find(|s| s.strike_index == strike_index) {
                                obs.end = Some((ev.timestamp, cause));
                            }
                        }
                    }
                }
            }
            EVENT_TYPE_STRIKE_COOLDOWN_ENDED => {
                if let EventData::StrikeCooldownEnded { entity_id, schedule_version } = ev.data {
                    let o = DEFAULT_ORGANISATION_MILLE[index_of(entity_id)];
                    let gs = griefs.get_mut(&entity_id).unwrap();
                    advance_observed(gs, mirrors.get_mut(&entity_id).unwrap(), ev.timestamp)?;
                    let (g, sl) = (gs.g_qmicro(), gs.slope_micro_points_per_day());
                    let phase_before = sm.phase_of(entity_id);
                    mirrors
                        .get_mut(&entity_id)
                        .unwrap()
                        .integrate(phase_before, o, ev.timestamp, g, sl);
                    sm.on_cooldown_ended(entity_id, ev.timestamp, g, sl, schedule_version, &mut queue);
                }
            }
            EVENT_TYPE_GRIEF_BOUND_REACHED => {
                if let EventData::GriefBoundReached { entity_id, bound_qmicro, schedule_version } = ev.data {
                    let o = DEFAULT_ORGANISATION_MILLE[index_of(entity_id)];
                    let gs = griefs.get_mut(&entity_id).unwrap();
                    advance_observed(gs, mirrors.get_mut(&entity_id).unwrap(), ev.timestamp)?;
                    let (g, sl) = (gs.g_qmicro(), gs.slope_micro_points_per_day());
                    let phase_before = sm.phase_of(entity_id);
                    mirrors
                        .get_mut(&entity_id)
                        .unwrap()
                        .integrate(phase_before, o, ev.timestamp, g, sl);
                    sm.on_grief_bound_reached(entity_id, ev.timestamp, bound_qmicro, schedule_version, g, sl, &mut queue);
                }
            }
            _ => {}
        }
    }

    // Avance finale à (DAYS, 0) avec capture des jalons : G en fin d'horizon, identique à A2.
    let final_ts = Timestamp::new(DAYS, 0);
    let mut cohorts_out = Vec::with_capacity(cohort_ids.len());
    for (ci, cid) in cohort_ids.iter().enumerate() {
        let o = DEFAULT_ORGANISATION_MILLE[ci];
        let gs = griefs.get_mut(cid).unwrap();
        let m = mirrors.get_mut(cid).unwrap();
        advance_observed(gs, m, final_ts)?;
        let past_horizon = gs.last_update() > final_ts;
        // Si des événements ont dépassé l'horizon, G à (DAYS, 0) a été enregistré au passage.
        let g_end = if past_horizon { m.g_day_3650.unwrap_or_else(|| gs.g_qmicro()) } else { gs.g_qmicro() };
        let sl_end = gs.slope_micro_points_per_day();
        let phase_before = sm.phase_of(*cid);
        m.integrate(phase_before, o, final_ts, g_end, sl_end);
        if m.g_day_3650.is_none() {
            m.g_day_3650 = Some(g_end);
        }
        let m = mirrors.remove(cid).unwrap();
        cohorts_out.push(CohortStrikeReading {
            index: ci,
            cohort_id: *cid,
            o_mille: o,
            strikes: strikes.remove(cid).unwrap_or_default(),
            expected_active_scaled: m.expected_active,
            expected_brute_scaled: m.expected_brute,
            g_day_365: m.g_day_365,
            g_day_1825: m.g_day_1825,
            g_day_3650: m.g_day_3650,
            g_max_qmicro: m.g_max,
        });
    }

    Ok(StrikeReading {
        kind,
        cohorts: cohorts_out,
        events_processed,
        verify_i7: journal.verify_i7(),
    })
}

/// Formate une grandeur à l'échelle 10^12 avec 3 décimales par division entière (X.YYY).
fn format_thousandths_scaled(scaled: u128) -> String {
    format!(
        "{}.{:03}",
        scaled / HAZARD_SCALE,
        (scaled % HAZARD_SCALE) / 1_000_000_000u128
    )
}

/// Affiche la section de lecture de grève B1 d'un scénario.
fn print_strike_reading(reading: &StrikeReading) {
    println!(
        "--- {} — lectures de grève B1 (graine {SEED}, {DAYS} j) ---",
        reading.kind.label()
    );
    println!(
        "events_processed = {}, verify_i7 = {}",
        reading.events_processed, reading.verify_i7
    );
    for c in &reading.cohorts {
        println!(
            "cohorte {} (O = {}) : grèves observées = {}, attendu fenêtres actives = {}, attendu brut = {}",
            c.index,
            c.o_mille,
            c.strikes.len(),
            format_thousandths_scaled(c.expected_active_scaled),
            format_thousandths_scaled(c.expected_brute_scaled)
        );
        for s in &c.strikes {
            let end = match &s.end {
                Some((ts, cause)) => format!(" fin=({}, {}) cause={:?}", ts.day, ts.seq, cause),
                None => " fin=-".to_string(),
            };
            println!(
                "  grève #{} : début=({}, {}), durée={} j,{}, G au déclenchement = {} qµpt ({} pts)",
                s.strike_index,
                s.start.day,
                s.start.seq,
                s.duration_days,
                end,
                s.g_qmicro,
                s.g_qmicro / 4_000_000
            );
        }
    }
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
        if report.kind == ScenarioKind::S0b {
            println!("ARRÊT OBLIGATOIRE S0b : G=100 atteint au jour {day} (< 3650).");
        } else {
            println!("ARRÊT OBLIGATOIRE S0 : G=100 atteint au jour {day} (< 1095).");
        }
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

    let s0b = run_scenario(ScenarioKind::S0b)?;
    print_report(&s0b);
    if s0b.early_stop_day.is_some() {
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

    // Lectures de grève Tranche B1 (sorties A2 ci-dessus inchangées).
    println!();
    println!("=== Lectures de grève Tranche B1 (graine {SEED}, {DAYS} j) ===");
    for kind in [
        ScenarioKind::S0b,
        ScenarioKind::S1,
        ScenarioKind::S2,
        ScenarioKind::S4,
    ] {
        let reading = run_strike_reading(kind)?;
        print_strike_reading(&reading);
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
        let expected_p_day_zero = [802, 545, 187, 0, 0];
        for (index, (_, stats)) in report.cohorts.iter().enumerate() {
            assert_eq!(stats.p_day_zero, Some(expected_p_day_zero[index]));
            assert_eq!(stats.u_max, 0);
            assert_eq!(stats.days_rationed, 0);
            assert_eq!(stats.g_max_qmicro, 0);
        }
    }

    #[test]
    fn s0b_keeps_q_at_500_with_nominal_inputs() {
        assert_eq!(
            calculate_q(&[
                (ServiceComponent::Soins, 500),
                (ServiceComponent::Logements, 500),
            ]),
            500
        );

        let report = run_scenario(ScenarioKind::S0b).unwrap();
        assert_eq!(report.early_stop_day, None);
        assert!(report
            .import_history
            .iter()
            .all(|&quantity| quantity == DAILY_IMPORT));
        assert!(report
            .price_history
            .iter()
            .all(|&price| price == BASE_FOOD_PRICE));

        let expected_p_day_zero = [802, 545, 187, 0, 0];
        for (index, (_, stats)) in report.cohorts.iter().enumerate() {
            assert_eq!(stats.q_min, 500);
            assert_eq!(stats.q_max, 500);
            assert_eq!(stats.p_day_zero, Some(expected_p_day_zero[index]));
            assert_eq!(stats.u_max, 0);
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

    /// B1 §6.2 — déterminisme bit à bit des lectures de grève + verify_i7 vert
    /// sur S0b, S1, S2, S4 (graine 42, 3650 j).
    #[test]
    fn strike_readings_are_deterministic_and_i7_clean() {
        for kind in [
            ScenarioKind::S0b,
            ScenarioKind::S1,
            ScenarioKind::S2,
            ScenarioKind::S4,
        ] {
            let a = run_strike_reading(kind).unwrap();
            let b = run_strike_reading(kind).unwrap();
            assert_eq!(
                format!("{:?}", a),
                format!("{:?}", b),
                "déterminisme bit à bit pour {:?}",
                kind
            );
            assert!(a.verify_i7, "verify_i7 vert pour {:?}", kind);
            assert_eq!(a.events_processed, b.events_processed);
            if kind == ScenarioKind::S0b {
                // S0b : rien ne change après le jour 0 -> coût événementiel, pas quotidien.
                assert!(
                    a.events_processed <= 20,
                    "S0b : coût proportionnel aux événements (<= 20), obtenu {}",
                    a.events_processed
                );
            }
        }
    }

    /// B1 §6.2 — invariance de G : la lecture de grève ne modifie pas G par rapport
    /// au run A2 (jours 365, 1825, 3650 et g_max) pour S0b, S1, S2, S4.
    #[test]
    fn strike_reading_preserves_a2_griefs() {
        for kind in [
            ScenarioKind::S0b,
            ScenarioKind::S1,
            ScenarioKind::S2,
            ScenarioKind::S4,
        ] {
            let report = run_scenario(kind).unwrap();
            let reading = run_strike_reading(kind).unwrap();
            for (ci, (cid, stats)) in report.cohorts.iter().enumerate() {
                let rc = &reading.cohorts[ci];
                assert_eq!(rc.cohort_id, *cid, "{:?}", kind);
                assert_eq!(rc.g_day_365, stats.g_day_365, "{:?} cohorte {} G365", kind, ci);
                assert_eq!(rc.g_day_1825, stats.g_day_1825, "{:?} cohorte {} G1825", kind, ci);
                assert_eq!(rc.g_day_3650, stats.g_day_3650, "{:?} cohorte {} G3650", kind, ci);
                assert_eq!(rc.g_max_qmicro, stats.g_max_qmicro, "{:?} cohorte {} Gmax", kind, ci);
            }
        }
    }
}
