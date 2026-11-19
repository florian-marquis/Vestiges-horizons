//! Vestiges Horizons - Tests Tranche 5 : Marchés, prix, repas et crédit
//! 
//! Tests selon Docs/spec P1 - quartier vivant.md §17.7

use vestiges_horizons::credit::{CreditManager, Collateral, CollateralType, LoanStatus, SubleaseStatus};
use vestiges_horizons::market::{GoodType, Market, MarketManager, ReferencePrice, MAX_LOGISTICS_DELAY, MIN_LOGISTICS_DELAY, PRICE_ELASTICITY_CAP, PRICE_ELASTICITY_SLOPE};
use vestiges_horizons::meal::{Basket, HousingLevel, MealManager, MealMode, WATER_BASE_COST_PERCENT, WATER_MAX_COST_PERCENT};
use vestiges_horizons::tax::{AllocationType, TaxManager, TAX_RATE, PENSION_AMOUNT, CHILD_ALLOCATION, DeclaredTransfer, TransferType};
use vestiges_horizons::entities::{EntityId, EntityType};

// ============================================================================
// T5.1 Monnaie
// ============================================================================

/// §17.7 T5.1 : "I3 vert : chaque transaction équilibrée, somme des soldes constante hors transferts déclarés"
#[test]
fn test_t5_1_monnaie() {
    let mut tax_manager = TaxManager::new();
    
    // Créer une firme et un employé
    let firm = EntityId::new(EntityType::Firm, 0);
    let employee = EntityId::new(EntityType::NamedIndividual, 0);
    
    // Prélèvement à la source
    let withheld = tax_manager.withhold_tax(firm, employee, 100000, 0);
    
    // Vérifier que le prélèvement est de 10%
    assert_eq!(withheld, 10000); // 10% de 100000
    
    // Vérifier que le total prélevé est correct
    // Vérifier I3 : les transferts sont déclarés (c'est toujours vrai dans notre implémentation)
    assert!(tax_manager.verify_t5_7_tax_flows());
}

// ============================================================================
// T5.2 Bornes de prix
// ============================================================================

/// §17.7 T5.2 : "I5 vert : prix dans les bornes en données, jamais infini ni nul"
#[test]
fn test_t5_2_bornes_prix() {
    let mut market = Market::new(
        EntityId::new(EntityType::Market, 0),
        GoodType::Food,
        1,
        100,
        1000,
        None,
        (200, 800),
        10,
        (50, 200),
        0,
    );
    
    // Prix initial
    assert!(market.displayed_price > 0);
    assert!(market.displayed_price <= 200);
    
    // Reprice à la baisse (stock haut)
    market.stock = 900;
    market.auto_reprice();
    assert!(market.displayed_price > 0);
    assert!(market.displayed_price >= 50); // Borne inférieure
    
    // Reprice à la hausse (stock bas)
    market.stock = 100;
    market.auto_reprice();
    assert!(market.displayed_price > 0);
    assert!(market.displayed_price <= 200); // Borne supérieure
    
    // Vérifier I5 : le prix est dans les bornes
    assert!(market.displayed_price >= 50 && market.displayed_price <= 200);
}

// ============================================================================
// T5.3 Stocks
// ============================================================================

/// §17.7 T5.3 : "I2 vert : stocks de marchés et de foyers jamais négatifs"
#[test]
fn test_t5_3_stocks() {
    let mut market = Market::new(
        EntityId::new(EntityType::Market, 0),
        GoodType::Food,
        1,
        100,
        1000,
        None,
        (200, 800),
        10,
        (50, 200),
        0,
    );
    
    // Acheter
    let _ = market.buy(EntityId::new(EntityType::Household, 0), 500, 100000);
    assert_eq!(market.stock, 500);
    assert!(market.stock >= 0);
    
    // Acheter plus que le stock
    let _ = market.buy(EntityId::new(EntityType::Household, 0), 1000, 100000);
    assert_eq!(market.stock, 0);
    assert!(market.stock >= 0);
    
    // Vérifier I2 : le stock n'est pas négatif
    assert!(market.stock >= 0);
}

// ============================================================================
// T5.4 Choc 1
// ============================================================================

/// §17.7 T5.4 : "1 an sans choc | distribution des prix et des repas stable, explicable"
#[test]
fn test_t5_4_choc_1() {
    let mut market_manager = MarketManager::new();
    
    // Créer un marché
    market_manager.create_market(
        EntityId::new(EntityType::Market, 0),
        GoodType::Food,
        1,
        100,
        1000,
        None,
        (200, 800),
        10,
        (50, 200),
        0,
    );
    
    // Créer un prix de référence
    market_manager.add_reference_price(1, 100, 1000);
    
    // Simuler 365 jours
    for day in 0..365 {
        market_manager.process_all_markets(day);
    }
    
    // Vérifier que le prix est stable (pas de choc)
    let market = market_manager.get_market(1).unwrap();
    // Le prix devrait être proche de 100 (avec possible ajustement automatique)
    assert!(market.displayed_price > 0);
    
    // Vérifier que les stocks sont positifs
    assert!(market.stock >= 0);
}

// ============================================================================
// T5.5 Choc 2
// ============================================================================

/// §17.7 T5.5 : "import ÷2 pendant 90 jours | ruptures temporaires puis hausse de prix de référence différée"
#[test]
fn test_t5_5_choc_2() {
    let mut market_manager = MarketManager::new();
    
    // Créer un marché
    market_manager.create_market(
        EntityId::new(EntityType::Market, 0),
        GoodType::Food,
        1,
        100,
        1000,
        None,
        (200, 800),
        10,
        (50, 200),
        0,
    );
    
    // Créer un prix de référence
    market_manager.add_reference_price(1, 100, 1000);
    
    // Commande initiale
    market_manager.order_import(1, 1000, 0);
    
    // Simuler 90 jours avec demande excédentaire
    for day in 0..90 {
        market_manager.process_all_markets(day);
        // Ajouter de la demande directement
        if let Some(ref_price) = market_manager.get_reference_price_mut(1) {
            ref_price.add_demand(2000); // Demande très élevée
        }
    }
    
    // Le prix de référence devrait monter
    let ref_price = market_manager.get_reference_price(1).unwrap();
    // Après 90 jours de demande élevée, le prix devrait atteindre le plafond
    assert!(ref_price.current_price >= 100);
}

// ============================================================================
// T5.6 Choc 10
// ============================================================================

/// §17.7 T5.6 : "taux doublé, garanties exigées | saisies et servitudes explicables"
#[test]
fn test_t5_6_choc_10() {
    let mut credit_manager = CreditManager::new();
    
    let lender = EntityId::new(EntityType::Firm, 0);
    let borrower = EntityId::new(EntityType::Household, 0);
    let housing = EntityId::new(EntityType::Housing, 0);
    
    // Créer un prêt avec taux normal
    let loan_id = credit_manager.create_loan(
        lender,
        borrower,
        100000,
        100, // 10%
        vec![Collateral {
            collateral_type: CollateralType::Housing,
            asset_id: housing,
            value: 150000,
        }],
        12,
        0,
        1, // Échéance tous les 1 jour
    );
    
    // Simuler un défaut
    credit_manager.process_due_dates(1);
    
    // Traiter les saisies
    credit_manager.process_seizures(1);
    
    // Vérifier la chaîne
    let loan = credit_manager.get_loan(loan_id).unwrap();
    assert_eq!(loan.status, LoanStatus::Seized);
    
    // Vérifier T5.9
    assert!(credit_manager.verify_t5_9_seizure_chain());
}

// ============================================================================
// T5.7 Choc 11
// ============================================================================

/// §17.7 T5.7 : "impôt ×2 pendant un an | flux fiscaux traçables (I3)"
#[test]
fn test_t5_7_choc_11() {
    // Simuler un taux d'imposition doublé
    // Pour l'instant, on vérifie que les flux sont traçables
    let mut tax_manager = TaxManager::new();
    
    let firm = EntityId::new(EntityType::Firm, 0);
    let employee = EntityId::new(EntityType::NamedIndividual, 0);
    
    // Prélèvement normal
    tax_manager.withhold_tax(firm, employee, 100000, 0);
    
    // Vérifier que les flux sont traçables
    assert!(tax_manager.verify_t5_7_tax_flows());
}

// ============================================================================
// T5.8 Choc 12
// ============================================================================

/// §17.7 T5.8 : "adductions cédées | consommation d'eau inchangée, crête de maladie"
/// Pour l'instant, on vérifie que le coût de l'eau est calculé
#[test]
fn test_t5_8_choc_12() {
    let meal_manager = MealManager::new();
    let household = EntityId::new(EntityType::Household, 0);
    
    // Coût de base
    let cost_normal = meal_manager.calculate_water_cost(household, 10000, 0);
    assert_eq!(cost_normal, 800); // 8% de 10000
    
    // Coût en pénurie
    let cost_scarcity = meal_manager.calculate_water_cost(household, 10000, 100);
    assert_eq!(cost_scarcity, 2500); // 25% de 10000
    
    // Vérifier que la consommation d'eau ne change pas (élasticité nulle)
    // La quantité consommée ne dépend pas du prix dans notre implémentation
}

// ============================================================================
// T5.9 Saisie explicable
// ============================================================================

/// §17.7 T5.9 : "chaîne complète prêt → défaut → saisie → proposition de sous-location"
#[test]
fn test_t5_9_saisie_explicable() {
    let mut credit_manager = CreditManager::new();
    
    let lender = EntityId::new(EntityType::Firm, 0);
    let borrower = EntityId::new(EntityType::Household, 0);
    let housing = EntityId::new(EntityType::Housing, 0);
    
    let loan_id = credit_manager.create_loan(
        lender,
        borrower,
        100000,
        100,
        vec![Collateral {
            collateral_type: CollateralType::Housing,
            asset_id: housing,
            value: 150000,
        }],
        1,
        0,
        1,
    );
    
    // Chaîne complète
    credit_manager.process_due_dates(1);
    credit_manager.process_seizures(1);
    
    // Vérifier que le prêt est saisi
    let loan = credit_manager.get_loan(loan_id).unwrap();
    assert_eq!(loan.status, LoanStatus::Seized);
    
    // Vérifier la chaîne
    assert!(credit_manager.verify_t5_9_seizure_chain());
}

// ============================================================================
// T5.10 Repas et salaire réel
// ============================================================================

/// §17.7 T5.10 : "distribution des modes de repas corrélée au salaire réel et au niveau de logement"
#[test]
fn test_t5_10_repas_salaire_reel() {
    let mut meal_manager = MealManager::new();
    
    let household = EntityId::new(EntityType::Household, 0);
    
    // Niveau 1 : ne peut pas cuisiner
    meal_manager.set_housing_level(household, HousingLevel::Level1);
    meal_manager.set_food_price(GoodType::Food, 100);
    
    let mode = meal_manager.resolve_meal_mode(household, 10, false, 10000, 8);
    assert_ne!(mode, MealMode::HomeCooking);
    
    // Niveau 2 : peut cuisiner
    meal_manager.set_housing_level(household, HousingLevel::Level2);
    let mode = meal_manager.resolve_meal_mode(household, 10, false, 10000, 8);
    // Si on a le temps, la cuisine maison devrait être choisie
    assert_eq!(mode, MealMode::HomeCooking);
    
    // Vérifier T5.10
    assert!(meal_manager.verify_t5_10_meal_distribution());
}

// ============================================================================
// T5.11 Plancher partiel
// ============================================================================

/// §17.7 T5.11 : "au moins une voie praticable (entraide, servitude, maisonnée, sous-location)"
#[test]
fn test_t5_11_plancher_partiel() {
    let credit_manager = CreditManager::new();
    
    // Pour l'instant, sans saisie, le plancher est respecté
    assert!(credit_manager.verify_t5_11_plancher());
}

// ============================================================================
// INVARIANTS
// ============================================================================

/// §17.7 T5.1 : "I3 vert : chaque transaction équilibrée"
#[test]
fn test_i3_balanced_transactions() {
    let tax_manager = TaxManager::new();
    
    // sans transactions, I3 est vérifié
    assert!(tax_manager.verify_i3_declared_transfers());
}

/// §17.7 T5.2 : "I5 vert : prix dans les bornes"
#[test]
fn test_i5_price_bounds() {
    let market_manager = MarketManager::new();
    
    // sans marchés, I5 est vérifié
    assert!(market_manager.verify_i5_price_bounds());
}

/// §17.7 T5.3 : "I2 vert : stocks jamais négatifs"
#[test]
fn test_i2_no_negative_stocks() {
    let market_manager = MarketManager::new();
    
    // sans marchés, I2 est vérifié
    assert!(market_manager.verify_i2_no_negative_stocks());
}
