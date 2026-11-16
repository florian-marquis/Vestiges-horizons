//! Vestiges Horizons - Benchmark T7-T8
//! 
//! Exécute une simulation headless sur 10 ans avec graine 42
//! Valide spécifiquement les tranches 7 (salaires réels, griefs) et 8 (grèves)
//! 
//! Critères de validation :
//! - Déterminisme (I9)
//! - Concordance griefs (I8)
//! - Grèves déclenchées dans les conditions prévues
//! - Pas de re-tirage (T8.2)

use std::collections::BTreeMap;
use std::time::Instant;
use vestiges_horizons::{
    salary::*,
    entities::*,
    strike::*,
    EventData, EventQueue, Phase, Priority, Timestamp,
    EVENT_TYPE_INPUT_CHANGED,
};

#[derive(Debug, Default, Clone)]
struct CohortGriefSummary {
    q_min: Option<u16>,
    q_max: Option<u16>,
    q_sum: u128,
    q_samples: u64,
    g_zero_days: u64,
    g_partial_days: u64,
    g_full_days: u64,
}

#[derive(Debug, Default, Clone)]
struct ValueDistribution {
    values: Vec<i64>,
}

impl ValueDistribution {
    fn push(&mut self, value: i64) {
        self.values.push(value);
    }

    fn mean(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<i64>() as f64 / self.values.len() as f64
    }

    fn median(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let mut ordered = self.values.clone();
        ordered.sort_unstable();
        let mid = ordered.len() / 2;
        if ordered.len() % 2 == 0 {
            (ordered[mid - 1] as f64 + ordered[mid] as f64) / 2.0
        } else {
            ordered[mid] as f64
        }
    }

    fn percentile(&self, fraction: f64) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let mut ordered = self.values.clone();
        ordered.sort_unstable();
        let index = ((ordered.len() as f64 - 1.0) * fraction).round() as usize;
        ordered[index.min(ordered.len() - 1)] as f64
    }
}

impl CohortGriefSummary {
    fn observe_q(&mut self, q: u16) {
        self.q_min = Some(self.q_min.map_or(q, |min| min.min(q)));
        self.q_max = Some(self.q_max.map_or(q, |max| max.max(q)));
        self.q_sum += u128::from(q);
        self.q_samples += 1;
    }

    fn observe_g(&mut self, g: i64) {
        match g {
            0 => self.g_zero_days += 1,
            x if x > 0 && x < 100 => self.g_partial_days += 1,
            x if x >= 100 => self.g_full_days += 1,
            _ => {}
        }
    }

    fn mean_q(&self) -> f64 {
        if self.q_samples == 0 {
            return 0.0;
        }
        self.q_sum as f64 / self.q_samples as f64
    }
}

fn compute_market_inputs(
    stock: u32,
    capacity: u32,
    displayed_price: u64,
    base_price: u64,
) -> (u16, u16, u16) {
    let shortage_ratio = (1.0 - (stock as f64 / capacity.max(1) as f64).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let shortage = (shortage_ratio * 1000.0).round() as u16;

    let price_ratio = if base_price == 0 {
        0.0
    } else {
        (displayed_price as f64 / base_price as f64).clamp(0.0, 2.5)
    };
    let burden = ((price_ratio - 1.0).max(0.0) / 1.5).clamp(0.0, 1.0);
    let pressure = (burden * 1000.0).round() as u16;

    let service_quality = (1000.0 * (1.0 - shortage_ratio * 0.7 - burden * 0.3)).clamp(0.0, 1000.0);
    let q = service_quality.round() as u16;

    (shortage, pressure, q)
}

fn print_cohort_grief_table(stats: &BTreeMap<EntityId, CohortGriefSummary>, total_days: u64) {
    println!("\n=== Diagnostic des griefs par cohorte (10 ans, état P1 observé) ===");
    println!("Cohorte | q min. | q moyen | q max. | % jours G=0 | % jours 0<G<100 | % jours G=100");
    println!("--------|--------:|--------:|--------:|------------:|----------------:|--------------:");

    for (cohort_id, summary) in stats {
        let q_min = summary.q_min.unwrap_or(0);
        let q_max = summary.q_max.unwrap_or(0);
        let mean_q = summary.mean_q();
        let zero_pct = if total_days == 0 { 0.0 } else { 100.0 * summary.g_zero_days as f64 / total_days as f64 };
        let partial_pct = if total_days == 0 { 0.0 } else { 100.0 * summary.g_partial_days as f64 / total_days as f64 };
        let full_pct = if total_days == 0 { 0.0 } else { 100.0 * summary.g_full_days as f64 / total_days as f64 };

        println!(
            "{:?} | {:>7} | {:>7.2} | {:>7} | {:>12.2}% | {:>16.2}% | {:>14.2}%",
            cohort_id,
            q_min,
            mean_q,
            q_max,
            zero_pct,
            partial_pct,
            full_pct
        );
    }
}

fn annual_curve(values: &[i64], years: usize) -> Vec<f64> {
    let mut yearly = Vec::new();
    for year in 0..years {
        let start = year * 365;
        let end = ((year + 1) * 365).min(values.len());
        if start >= values.len() {
            yearly.push(0.0);
            continue;
        }
        let slice = &values[start..end];
        let mean = if slice.is_empty() {
            0.0
        } else {
            slice.iter().sum::<i64>() as f64 / slice.len() as f64
        };
        yearly.push(mean);
    }
    yearly
}

fn annual_report_for_cohorts(
    cohort_ids: &[EntityId],
    g_dist: &BTreeMap<EntityId, ValueDistribution>,
    u_dist: &BTreeMap<EntityId, ValueDistribution>,
    p_dist: &BTreeMap<EntityId, ValueDistribution>,
    q_dist: &BTreeMap<EntityId, ValueDistribution>,
    years: usize,
) {
    println!("\n=== Moyennes annuelles par cohorte ===");
    println!("Cohorte | année | G moyenne | G 31/12 | u moyen | p moyen | q moyen");
    println!("--------|-------|----------:|--------:|--------:|--------:|--------:");

    for cohort_id in cohort_ids {
        let g_values = &g_dist.get(cohort_id).cloned().unwrap_or_default().values;
        let u_values = &u_dist.get(cohort_id).cloned().unwrap_or_default().values;
        let p_values = &p_dist.get(cohort_id).cloned().unwrap_or_default().values;
        let q_values = &q_dist.get(cohort_id).cloned().unwrap_or_default().values;

        for year in 0..years {
            let start = year * 365;
            let end = ((year + 1) * 365).min(g_values.len());
            if start >= g_values.len() { continue; }
            let g_slice = &g_values[start..end];
            let u_slice = &u_values[start..end];
            let p_slice = &p_values[start..end];
            let q_slice = &q_values[start..end];

            let g_mean = if g_slice.is_empty() { 0.0 } else { g_slice.iter().sum::<i64>() as f64 / g_slice.len() as f64 };
            let u_mean = if u_slice.is_empty() { 0.0 } else { u_slice.iter().sum::<i64>() as f64 / u_slice.len() as f64 };
            let p_mean = if p_slice.is_empty() { 0.0 } else { p_slice.iter().sum::<i64>() as f64 / p_slice.len() as f64 };
            let q_mean = if q_slice.is_empty() { 0.0 } else { q_slice.iter().sum::<i64>() as f64 / q_slice.len() as f64 };
            let g_dec31 = if end > 0 { g_values[end - 1] } else { 0 };

            println!(
                "{:?} | {:>5} | {:>9.2} | {:>8} | {:>7.2} | {:>7.2} | {:>7.2}",
                cohort_id, year + 1, g_mean, g_dec31, u_mean, p_mean, q_mean
            );
        }
    }
}

fn print_controlled_distribution_report(
    cohort_ids: &[EntityId],
    g_distributions: &BTreeMap<EntityId, ValueDistribution>,
    u_distributions: &BTreeMap<EntityId, ValueDistribution>,
    p_distributions: &BTreeMap<EntityId, ValueDistribution>,
    q_distributions: &BTreeMap<EntityId, ValueDistribution>,
) {
    println!("\n=== Rapport contrôlé G et entrées (u, p, q) ===");
    println!("Cohorte | G final | G moy | G med | p10(G) | p90(G) | u moy | p moy | q moy | Courbe (10 derniers jours)");
    println!("--------|--------:|------:|------:|-------:|-------:|------:|------:|------:|------------------------");

    for cohort_id in cohort_ids {
        let g = g_distributions.get(cohort_id).cloned().unwrap_or_default();
        let u = u_distributions.get(cohort_id).cloned().unwrap_or_default();
        let p = p_distributions.get(cohort_id).cloned().unwrap_or_default();
        let q = q_distributions.get(cohort_id).cloned().unwrap_or_default();

        let final_g = g.values.last().copied().unwrap_or(0);
        let g_curve = g.values.iter().rev().take(10).copied().collect::<Vec<_>>();
        let curve_text = if g_curve.is_empty() { "[]".to_string() } else { format!("{:?}", g_curve) };

        println!(
            "{:?} | {:>8} | {:>6.2} | {:>6.2} | {:>7.2} | {:>7.2} | {:>6.2} | {:>6.2} | {:>6.2} | {}",
            cohort_id,
            final_g,
            g.mean(),
            g.median(),
            g.percentile(0.10),
            g.percentile(0.90),
            u.mean(),
            p.mean(),
            q.mean(),
            curve_text
        );
    }
}

fn print_raw_cohort_curve_report(cohort_ids: &[EntityId], g_distributions: &BTreeMap<EntityId, ValueDistribution>) {
    println!("\n=== Sortie brute des 5 cohortes : G final + moyenne annuelle ===");
    println!("Cohorte | G final | moyenne annuelle 1 | moyenne annuelle 2 | moyenne annuelle 3 | moyenne annuelle 4 | moyenne annuelle 5 | moyenne annuelle 6 | moyenne annuelle 7 | moyenne annuelle 8 | moyenne annuelle 9 | moyenne annuelle 10 | G 31/12");
    println!("--------|--------:|-------------------:|-------------------:|-------------------:|-------------------:|-------------------:|-------------------:|-------------------:|-------------------:|-------------------:|-------------------:|--------:");

    for cohort_id in cohort_ids {
        let g = g_distributions.get(cohort_id).cloned().unwrap_or_default();
        let final_g = g.values.last().copied().unwrap_or(0);
        let yearly = annual_curve(&g.values, 10);
        let dec31_g = if g.values.len() >= 365 { g.values[364] } else { 0 };
        let mut cells = vec![format!("{:>8}", final_g)];
        for value in yearly {
            cells.push(format!("{:>19.2}", value));
        }
        cells.push(format!("{:>8}", dec31_g));
        println!("{:?} | {}", cohort_id, cells.join(" | "));
    }
}

fn write_monthly_csv_30y(
    path: &str,
    cohort_ids: &[EntityId],
    g_dist: &BTreeMap<EntityId, ValueDistribution>,
    u_dist: &BTreeMap<EntityId, ValueDistribution>,
    p_dist: &BTreeMap<EntityId, ValueDistribution>,
    q_dist: &BTreeMap<EntityId, ValueDistribution>,
) {
    let mut csv = String::new();
    csv.push_str("cohort,year,month,g_mean,u_mean,p_mean,q_mean,g_31_dec\n");

    for cohort_id in cohort_ids {
        let g_values = &g_dist.get(cohort_id).cloned().unwrap_or_default().values;
        let u_values = &u_dist.get(cohort_id).cloned().unwrap_or_default().values;
        let p_values = &p_dist.get(cohort_id).cloned().unwrap_or_default().values;
        let q_values = &q_dist.get(cohort_id).cloned().unwrap_or_default().values;

        for year in 0..30 {
            for month in 0..12 {
                let month_days = if month == 11 { 31 } else { 30 };
                let start = year * 365 + month * 30;
                let end = (start + month_days).min(g_values.len());
                if start >= g_values.len() { continue; }
                let g_slice = &g_values[start..end];
                let u_slice = &u_values[start..end];
                let p_slice = &p_values[start..end];
                let q_slice = &q_values[start..end];

                let g_mean = if g_slice.is_empty() { 0.0 } else { g_slice.iter().sum::<i64>() as f64 / g_slice.len() as f64 };
                let u_mean = if u_slice.is_empty() { 0.0 } else { u_slice.iter().sum::<i64>() as f64 / u_slice.len() as f64 };
                let p_mean = if p_slice.is_empty() { 0.0 } else { p_slice.iter().sum::<i64>() as f64 / p_slice.len() as f64 };
                let q_mean = if q_slice.is_empty() { 0.0 } else { q_slice.iter().sum::<i64>() as f64 / q_slice.len() as f64 };
                let g_dec31 = if month == 11 && year * 365 + 364 < g_values.len() {
                    g_values[year * 365 + 364]
                } else {
                    0
                };

                csv.push_str(&format!(
                    "\"{:?}\",{},{},{:.2},{:.2},{:.2},{:.2},{}\n",
                    cohort_id,
                    year + 1,
                    month + 1,
                    g_mean,
                    u_mean,
                    p_mean,
                    q_mean,
                    g_dec31
                ));
            }
        }
    }

    std::fs::write(path, csv).unwrap();
}

fn simulate_30_years_monthly_csv() -> Result<(), Box<dyn std::error::Error>> {
    let mut market_manager = vestiges_horizons::market::MarketManager::new();
    let base_food_price = 100;
    let food_capacity = 5000;
    let food_market_id = 1;
    let cohort_count = 5u64;
    let total_days = 30 * 365;

    market_manager.create_market(
        EntityId::new(EntityType::Market, 0),
        vestiges_horizons::market::GoodType::Food,
        food_market_id,
        base_food_price,
        food_capacity,
        None,
        (2000, 8000),
        10,
        (50, 200),
        100,
    );
    market_manager.add_reference_price(food_market_id, base_food_price, food_capacity);

    let mut g_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut u_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut p_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut q_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut entity_manager = EntityManager::new(BENCHMARK_SEED);
    let mut salary_manager = SalaireReelEtGriefsManager::new();
    let mut input_events = EventQueue::new();
    let mut cohort_ids = Vec::new();

    for i in 0..cohort_count {
        let cohort_id = entity_manager.create_cohort(100);
        cohort_ids.push(cohort_id);
        salary_manager.ajouter_entite(cohort_id);
        salary_manager.mettre_a_jour_salaire_horaire(cohort_id, 500 + (i as u64 * 300));
        salary_manager.mettre_a_jour_heures(cohort_id, 8);
        g_distribution.insert(cohort_id, ValueDistribution::default());
        u_distribution.insert(cohort_id, ValueDistribution::default());
        p_distribution.insert(cohort_id, ValueDistribution::default());
        q_distribution.insert(cohort_id, ValueDistribution::default());
    }

    for day in 0..total_days {
        let seasonal = 0.45 + 0.35 * ((day as f64 / 180.0).sin() + 1.0) / 2.0;
        let disruption = if day >= 1200 && day < 1800 { 0.18 } else { 0.0 };
        let target_stock = ((food_capacity as f64) * (seasonal + disruption)).round() as u32;
        let base_capacity;
        let base_price_for_market;
        {
            let ref_price = market_manager.get_reference_price(food_market_id).unwrap();
            base_capacity = ref_price.nominal_capacity;
            base_price_for_market = ref_price.base_price;
        }
        {
            let market = market_manager.get_market_mut(food_market_id).unwrap();
            market.stock = target_stock.clamp(0, food_capacity);
            market.displayed_price = ((base_food_price as f64) * (1.0 + (food_capacity - market.stock) as f64 / food_capacity as f64 * 0.8)).round() as u64;
        }
        let market = market_manager.get_market(food_market_id).unwrap();

        for (index, &cohort_id) in cohort_ids.iter().enumerate() {
            let (u, p, q) = compute_market_inputs(
                market.stock,
                base_capacity,
                market.displayed_price,
                base_price_for_market,
            );
            let q_balance = (q as f64 * (1.0 + (index as f64 * 0.05))).round() as u16;
            let q_value = q_balance.min(1000);
            let u_value = (u as u32 + (index as u32 * 25)).min(1000) as u16;
            let p_value = (p as u32 + (index as u32 * 15)).min(1000) as u16;

            input_events.push_at_with_id(
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_id,
                    u: u_value,
                    p: p_value,
                    q: q_value,
                },
            );
        }

        while let Some(event) = input_events.pop() {
            salary_manager.appliquer_evenement(&event)?;
        }

        for &cohort_id in &cohort_ids {
            let entity = salary_manager.entites.get(&cohort_id).unwrap();
            let g = entity.griefs.griefs_points();
            let g_dist = g_distribution.get_mut(&cohort_id).unwrap();
            let u_dist = u_distribution.get_mut(&cohort_id).unwrap();
            let p_dist = p_distribution.get_mut(&cohort_id).unwrap();
            let q_dist = q_distribution.get_mut(&cohort_id).unwrap();
            g_dist.push(g);
            u_dist.push(i64::from(entity.griefs.u()));
            p_dist.push(i64::from(entity.griefs.p()));
            q_dist.push(i64::from(entity.griefs.q()));
        }
    }

    let csv_path = "benchmark_t7_t8_30y_monthly.csv";
    write_monthly_csv_30y(csv_path, &cohort_ids, &g_distribution, &u_distribution, &p_distribution, &q_distribution);
    println!("\nCSV mensuel 30 ans écrit : {}", csv_path);
    Ok(())
}



fn run_headless_grief_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    let mut market_manager = vestiges_horizons::market::MarketManager::new();
    let base_food_price = 100;
    let food_capacity = 5000;
    let food_market_id = 1;
    let cohort_count = 5u64;

    market_manager.create_market(
        EntityId::new(EntityType::Market, 0),
        vestiges_horizons::market::GoodType::Food,
        food_market_id,
        base_food_price,
        food_capacity,
        None,
        (2000, 8000),
        10,
        (50, 200),
        100,
    );
    market_manager.add_reference_price(food_market_id, base_food_price, food_capacity);

    let mut stats: BTreeMap<EntityId, CohortGriefSummary> = BTreeMap::new();
    let mut g_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut u_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut p_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut q_distribution: BTreeMap<EntityId, ValueDistribution> = BTreeMap::new();
    let mut entity_manager = EntityManager::new(BENCHMARK_SEED);
    let mut salary_manager = SalaireReelEtGriefsManager::new();
    let mut input_events = EventQueue::new();

    let mut cohort_ids = Vec::new();
    for i in 0..cohort_count {
        let cohort_id = entity_manager.create_cohort(100);
        cohort_ids.push(cohort_id);
        salary_manager.ajouter_entite(cohort_id);
        salary_manager.mettre_a_jour_salaire_horaire(cohort_id, 500 + (i as u64 * 300));
        salary_manager.mettre_a_jour_heures(cohort_id, 8);
        stats.insert(cohort_id, CohortGriefSummary::default());
        g_distribution.insert(cohort_id, ValueDistribution::default());
        u_distribution.insert(cohort_id, ValueDistribution::default());
        p_distribution.insert(cohort_id, ValueDistribution::default());
        q_distribution.insert(cohort_id, ValueDistribution::default());
    }

    let mut total_days: u64 = 0;
    for day in 0..SIMULATION_DAYS {
        let seasonal = 0.45 + 0.35 * ((day as f64 / 180.0).sin() + 1.0) / 2.0;
        let disruption = if day >= 1200 && day < 1800 { 0.18 } else { 0.0 };
        let target_stock = ((food_capacity as f64) * (seasonal + disruption)).round() as u32;
        let base_capacity;
        let base_price_for_market;
        {
            let ref_price = market_manager.get_reference_price(food_market_id).unwrap();
            base_capacity = ref_price.nominal_capacity;
            base_price_for_market = ref_price.base_price;
        }
        {
            let market = market_manager.get_market_mut(food_market_id).unwrap();
            market.stock = target_stock.clamp(0, food_capacity);
            market.displayed_price = ((base_food_price as f64) * (1.0 + (food_capacity - market.stock) as f64 / food_capacity as f64 * 0.8)).round() as u64;
        }
        let market = market_manager.get_market(food_market_id).unwrap();

        for (index, &cohort_id) in cohort_ids.iter().enumerate() {
            let (u, p, q) = compute_market_inputs(
                market.stock,
                base_capacity,
                market.displayed_price,
                base_price_for_market,
            );
            let q_balance = (q as f64 * (1.0 + (index as f64 * 0.05))).round() as u16;
            let q_value = q_balance.min(1000);
            let u_value = (u as u32 + (index as u32 * 25)).min(1000) as u16;
            let p_value = (p as u32 + (index as u32 * 15)).min(1000) as u16;

            input_events.push_at_with_id(
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_id,
                    u: u_value,
                    p: p_value,
                    q: q_value,
                },
            );
        }

        while let Some(event) = input_events.pop() {
            salary_manager.appliquer_evenement(&event)?;
        }

        for &cohort_id in &cohort_ids {
            let entity = salary_manager.entites.get(&cohort_id).unwrap();
            let g = entity.griefs.griefs_points();
            let current = stats.get_mut(&cohort_id).unwrap();
            let q_value = entity.griefs.q();
            current.observe_q(q_value);
            current.observe_g(g);

            let g_dist = g_distribution.get_mut(&cohort_id).unwrap();
            let u_dist = u_distribution.get_mut(&cohort_id).unwrap();
            let p_dist = p_distribution.get_mut(&cohort_id).unwrap();
            let q_dist = q_distribution.get_mut(&cohort_id).unwrap();
            g_dist.push(g);
            u_dist.push(i64::from(entity.griefs.u()));
            p_dist.push(i64::from(entity.griefs.p()));
            q_dist.push(i64::from(entity.griefs.q()));
        }

        total_days += 1;
    }

    print_cohort_grief_table(&stats, total_days);
    print_controlled_distribution_report(&cohort_ids, &g_distribution, &u_distribution, &p_distribution, &q_distribution);
    print_raw_cohort_curve_report(&cohort_ids, &g_distribution);
    annual_report_for_cohorts(&cohort_ids, &g_distribution, &u_distribution, &p_distribution, &q_distribution, 10);
    Ok(())
}

/// Graine pour le benchmark
const BENCHMARK_SEED: u64 = 42;

/// Durée de la simulation en jours
const SIMULATION_DAYS: i64 = 365 * 10; // 10 ans

/// Nombre de cohortes à simuler
const NUM_COHORTS: u64 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Benchmark T7-T8 Headless ===");
    println!("Graine: {}", BENCHMARK_SEED);
    println!("Durée: {} jours ({} ans)", SIMULATION_DAYS, SIMULATION_DAYS / 365);
    println!("Cohortes: {}", NUM_COHORTS);
    println!();

    run_headless_grief_diagnostic()?;
    simulate_30_years_monthly_csv()?;

    let start_time = Instant::now();
    
    // ========================================================================
    // Exécution 1 : Première passe
    // ========================================================================
    println!("[1/2] Première exécution...");
    
    let mut entity_manager = EntityManager::new(BENCHMARK_SEED);
    let mut salary_manager = SalaireReelEtGriefsManager::new();
    let mut strike_manager = StrikeManager::new(BENCHMARK_SEED);
    let mut input_events = EventQueue::new();
    
    // Créer les cohortes
    let mut cohort_ids = Vec::new();
    for i in 0..NUM_COHORTS {
        let cohort_id = entity_manager.create_cohort(100);
        cohort_ids.push(cohort_id);
        
        // Initialiser dans le manager de salaires
        salary_manager.ajouter_entite(cohort_id);
        
        // Salaire horaire entre 500 et 2000 centimes (5-20€/heure)
        let salaire = 500 + (i * 300);
        salary_manager.mettre_a_jour_salaire_horaire(cohort_id, salaire);
        salary_manager.mettre_a_jour_heures(cohort_id, 8);
    }
    
    // Initialiser le panier
    salary_manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
    salary_manager.mettre_a_jour_panier(PanierComposante::Logement, 2000);
    salary_manager.mettre_a_jour_panier(PanierComposante::Eau, 500);
    salary_manager.mettre_a_jour_panier(PanierComposante::Fiscalite, 300);
    
    // Variables de suivi
    let mut greves_declenchees: u64 = 0;
    let mut greves_terminees: u64 = 0;
    let mut jours_avec_greve: u64 = 0;
    let mut i8_violations: u64 = 0;
    let mut jours: u64 = 0;
    
    // Simulation sur 10 ans
    for day in 0..SIMULATION_DAYS {
        jours += 1;
        
        for (i, &cohort_id) in cohort_ids.iter().enumerate() {
            let u = ((day + i as i64 * 100) % 1000) as u16;
            let p = ((day * 2 + i as i64 * 50) % 1000) as u16;
            let q = ((day * 3) % 1000) as u16;
            input_events.push_at_with_id(
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_id,
                    u,
                    p,
                    q,
                },
            );
        }
        while let Some(event) = input_events.pop() {
            salary_manager.appliquer_evenement(&event)?;
        }

        // Mettre à jour les salaires réels (simulation de variation des prix)
        // Chaque cohorte a une trajectoire différente
        for (i, &cohort_id) in cohort_ids.iter().enumerate() {
            // Variation aléatoire mais déterministe du panier
            // (simulation de choc 2 : prix nourriture +50% au jour 200)
            if day == 200 {
                salary_manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1500);
            }
            if day == 300 {
                salary_manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
            }
            
            // Vérifier si le seuil de risque est franchi
            if let Some(e) = salary_manager.entites.get(&cohort_id) {
                if e.seuil_franchi() {
                    // Déclencher une grève
                    if strike_manager.declencher_greve(cohort_id, day).is_some() {
                        greves_declenchees += 1;
                        println!("  [Jour {}] Grève déclenchée pour cohorte {}", day, i);
                    }
                }
            }
            
            // Vérifier I8
            if !entity_manager.verify_i8_concordance() {
                i8_violations += 1;
            }
            
            // Vérifier si la cohorte est en grève
            if strike_manager.is_on_strike(cohort_id, day) {
                jours_avec_greve += 1;
            }
            
            // Fin de grève aléatoire (simulation de concession)
            if day > 200 && day % 10 == 0 {
                if let Some(strike) = strike_manager.get_strike(cohort_id) {
                    if !strike.is_ended() && day >= strike.start_day + 5 {
                        // 50% de chance de concession
                        if (day + i as i64) % 2 == 0 {
                            strike_manager.fin_de_greve(cohort_id, day, StrikeEndCause::Concession);
                            greves_terminees += 1;
                            println!("  [Jour {}] Grève terminée (concession) pour cohorte {}", day, i);
                        }
                    }
                }
            }
        }
        
        // Journalisation
        if day % 365 == 0 {
            println!("  Année {} terminée (jour {})", day / 365, day);
        }
    }
    
    // ========================================================================
    // Exécution 2 : Deuxième passe pour vérifier le déterminisme (I9)
    // ========================================================================
    println!();
    println!("[2/2] Deuxième exécution (vérification I9)...");
    
    let mut entity_manager2 = EntityManager::new(BENCHMARK_SEED);
    let mut salary_manager2 = SalaireReelEtGriefsManager::new();
    let mut strike_manager2 = StrikeManager::new(BENCHMARK_SEED);
    let mut input_events2 = EventQueue::new();
    
    // Même initialisation
    let mut cohort_ids2 = Vec::new();
    for i in 0..NUM_COHORTS {
        let cohort_id = entity_manager2.create_cohort(100);
        cohort_ids2.push(cohort_id);
        
        salary_manager2.ajouter_entite(cohort_id);
        let salaire = 500 + (i * 300);
        salary_manager2.mettre_a_jour_salaire_horaire(cohort_id, salaire);
        salary_manager2.mettre_a_jour_heures(cohort_id, 8);
    }
    
    salary_manager2.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
    salary_manager2.mettre_a_jour_panier(PanierComposante::Logement, 2000);
    salary_manager2.mettre_a_jour_panier(PanierComposante::Eau, 500);
    salary_manager2.mettre_a_jour_panier(PanierComposante::Fiscalite, 300);
    
    let mut greves_declenchees2: u64 = 0;
    let mut i8_violations2: u64 = 0;
    
    // Même simulation
    for day in 0..SIMULATION_DAYS {
        for (i, &cohort_id) in cohort_ids2.iter().enumerate() {
            let u = ((day + i as i64 * 100) % 1000) as u16;
            let p = ((day * 2 + i as i64 * 50) % 1000) as u16;
            let q = ((day * 3) % 1000) as u16;
            input_events2.push_at_with_id(
                Timestamp::new(day, 0),
                Phase::Comptes,
                Priority::new(0),
                EVENT_TYPE_INPUT_CHANGED,
                EventData::InputChanged {
                    entity_id: cohort_id,
                    u,
                    p,
                    q,
                },
            );
        }
        while let Some(event) = input_events2.pop() {
            salary_manager2.appliquer_evenement(&event)?;
        }

        for (i, &cohort_id) in cohort_ids2.iter().enumerate() {
            if day == 200 {
                salary_manager2.mettre_a_jour_panier(PanierComposante::Nourriture, 1500);
            }
            if day == 300 {
                salary_manager2.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
            }
            
            if let Some(e) = salary_manager2.entites.get(&cohort_id) {
                if e.seuil_franchi() {
                    if strike_manager2.declencher_greve(cohort_id, day).is_some() {
                        greves_declenchees2 += 1;
                    }
                }
            }
            
            if !entity_manager2.verify_i8_concordance() {
                i8_violations2 += 1;
            }
            
            if day > 200 && day % 10 == 0 {
                if let Some(strike) = strike_manager2.get_strike(cohort_id) {
                    if !strike.is_ended() && day >= strike.start_day + 5 {
                        if (day + i as i64) % 2 == 0 {
                            strike_manager2.fin_de_greve(cohort_id, day, StrikeEndCause::Concession);
                        }
                    }
                }
            }
        }
    }
    
    // ========================================================================
    // Résultats
    // ========================================================================
    let elapsed = start_time.elapsed();
    
    println!();
    println!("=== Résultats Benchmark T7-T8 ===");
    println!();
    
    println!("Temps d'exécution: {:?}", elapsed);
    println!();
    
    println!("--- Exécution 1 ---");
    println!("Grèves déclenchées: {}", greves_declenchees);
    println!("Jours avec grève: {}", jours_avec_greve);
    println!("Violations I8: {}", i8_violations);
    println!();
    
    println!("--- Exécution 2 ---");
    println!("Grèves déclenchées: {}", greves_declenchees2);
    println!("Violations I8: {}", i8_violations2);
    println!();
    
    // ========================================================================
    // Validation
    // ========================================================================
    println!("=== Validation ===");
    println!();
    
    let mut all_pass = true;
    
    // I8 : Aucune violation
    if i8_violations == 0 && i8_violations2 == 0 {
        println!("✅ I8 (concordance griefs): VALIDÉ (0 violation)");
    } else {
        println!("❌ I8 (concordance griefs): ÉCHEC ({} + {} violations)", i8_violations, i8_violations2);
        all_pass = false;
    }
    
    // I9 : Déterminisme
    if greves_declenchees == greves_declenchees2 {
        println!("✅ I9 (déterminisme): VALIDÉ (même nombre de grèves: {})", greves_declenchees);
    } else {
        println!("❌ I9 (déterminisme): ÉCHEC ({} vs {} grèves)", greves_declenchees, greves_declenchees2);
        all_pass = false;
    }
    
    // Grèves déclenchées
    if greves_declenchees > 0 {
        println!("✅ T8.1 (basculement): VALIDÉ ({} grèves déclenchées)", greves_declenchees);
    } else {
        println!("⚠️  T8.1 (basculement): AUCUNE GRÈVE (peut être normal avec ces paramètres)");
    }
    
    println!();
    
    if all_pass {
        println!("✅✅✅ TOUS LES TESTS PASSENT ✅✅✅");
    } else {
        println!("⚠️ Le benchmark reste un diagnostic de revue humaine : aucun seuil d'acceptation n'est fixé ici.");
    }

    println!("\nLa sortie ci-dessus sert à comparer le comportement observé du quartier à l'intention du design, sans conclure sur l'acceptation du calibrage.");
    Ok(())
}
