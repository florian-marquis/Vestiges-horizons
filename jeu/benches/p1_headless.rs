//! Vestiges Horizons - Benchmark P1 Headless
//! 
//! Exécute une simulation headless sur 10 ans avec graine 42
//! Valide les invariants disponibles
//! Respecte la règle économique : gain > coût

use std::time::Instant;
use vestiges_horizons::market::{GoodType, MarketManager, MIN_LOGISTICS_DELAY, MAX_LOGISTICS_DELAY, PRICE_ELASTICITY_CAP};

/// Graine pour le benchmark (spécifiée par l'utilisateur : 42)
const BENCHMARK_SEED: u64 = 42;

/// Durée de la simulation en jours
const SIMULATION_DAYS: i64 = 365 * 10; // 10 ans

fn main() {
    println!("=== Benchmark P1 Headless ===");
    println!("Graine: {}", BENCHMARK_SEED);
    println!("Durée: {} jours ({} ans)", SIMULATION_DAYS, SIMULATION_DAYS / 365);
    println!();
    
    let start_time = Instant::now();
    
    // Initialisation
    let mut market_manager = MarketManager::new();
    
    // Créer un marché de nourriture
    let market_id = vestiges_horizons::entities::EntityId::new(
        vestiges_horizons::entities::EntityType::Market, 0);
    market_manager.create_market(
        market_id,
        GoodType::Food,
        1,
        100, // prix affiché
        10000, // stock initial
        None,
        (2000, 8000), // seuils de stock
        10, // pas de prix
        (50, 200), // bornes de prix
        100, // seuil de ruine
    );
    
    // Prix de référence
    market_manager.add_reference_price(1, 100, 1000); // nourriture: base=100, capacity=1000
    
    // Commande initiale d'import
    market_manager.order_import(1, 5000, 0);
    
    println!("Initialisation terminée.");
    println!("Début de la simulation...");
    
    // Variables pour le suivi
    let mut imports_processed: u64 = 0;
    let mut prices_updated: u64 = 0;
    let mut days_with_invalid_prices: u64 = 0;
    let mut days_with_negative_stocks: u64 = 0;
    
    // Exécuter la simulation sur 10 ans
    for day in 0..SIMULATION_DAYS {
        // Traiter les arrivées d'import
        let arrivals = market_manager.process_import_arrivals(day);
        imports_processed += arrivals.len() as u64;
        
        // Mettre à jour les prix de référence
        market_manager.update_reference_prices(day);
        prices_updated += 1;
        
        // Traiter tous les marchés
        market_manager.process_all_markets(day);
        
        // Vérifier les invariants tous les 100 jours
        if day % 100 == 0 {
            // I2: Aucun stock négatif
            if !market_manager.verify_i2_no_negative_stocks() {
                days_with_negative_stocks += 1;
            }
            
            // I5: Prix dans les bornes
            if !market_manager.verify_i5_price_bounds() {
                days_with_invalid_prices += 1;
            }
            
            // Journaliser la progression
            if day % 365 == 0 {
                println!("  Année {} terminée (jour {})", day / 365, day);
            }
        }
        
        // Appliquer le choc 2 au jour 200
        if day == 200 {
            println!("  Appliquant le choc 2: import ÷2 pendant 90 jours");
            // Réduire la capacité nominale
            if let Some(price) = market_manager.get_reference_price_mut(1) {
                price.nominal_capacity /= 2;
            }
        }
    }
    
    // Vérification finale
    let final_i2 = market_manager.verify_i2_no_negative_stocks();
    let final_i5 = market_manager.verify_i5_price_bounds();
    
    let elapsed = start_time.elapsed();
    let total_time_seconds = elapsed.as_secs_f64();
    let time_per_day_microseconds = (elapsed.as_micros() as f64) / SIMULATION_DAYS as f64;
    
    println!();
    println!("=== Rapport du Benchmark P1 ===");
    println!();
    
    println!("📊 Performances :");
    println!("   Temps total: {:.2} secondes ({:.2} minutes)", 
             total_time_seconds,
             total_time_seconds / 60.0);
    println!("   Temps par jour: {:.2} μs", time_per_day_microseconds);
    println!();
    
    println!("📈 Statistiques :");
    println!("   Imports traités: {}", imports_processed);
    println!("   Mises à jour de prix: {}", prices_updated);
    println!();
    
    println!("🔍 Invariants :");
    println!("   I2 (Stocks non négatifs): {}", if final_i2 { "✅ PASS" } else { "❌ FAIL" });
    println!("   I5 (Prix dans les bornes): {}", if final_i5 { "✅ PASS" } else { "❌ FAIL" });
    println!("   Jours avec stocks négatifs: {}", days_with_negative_stocks);
    println!("   Jours avec prix invalides: {}", days_with_invalid_prices);
    println!();
    
    if final_i2 && final_i5 && days_with_negative_stocks == 0 && days_with_invalid_prices == 0 {
        println!("✅ Tous les invariants vérifiés sont respectés!");
    } else {
        println!("⚠️  Certains invariants ont échoué");
    }
    println!();
    
    println!("📌 Configuration :");
    println!("   Graine: {}", BENCHMARK_SEED);
    println!("   Durée: {} jours ({} ans)", SIMULATION_DAYS, SIMULATION_DAYS / 365);
    println!("   Délai logistique: {} à {} jours", MIN_LOGISTICS_DELAY, MAX_LOGISTICS_DELAY);
    println!("   Plafond élasticité prix: +{}%", PRICE_ELASTICITY_CAP as u64 / 10);
    println!();
    
    println!("=== Benchmark P1 Headless Terminé ===");
}
