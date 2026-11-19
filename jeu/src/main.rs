//! Vestiges Horizons - Point d'entrée pour l'exécution headless
//!
//! Ce binaire exécute les contrôles headless des tranches 1 et 2.
//! La génération des replais se fait via `generer_replai --output-dir …`
//! (jamais d'écriture automatique dans src/replai/).

use vestiges_horizons::entities::*;

fn main() {
    println!("Vestiges Horizons - Tranches 1 & 2");
    println!("====================================\n");

    // ========================================================================
    // Tests Tranche 1 : Noyau Temporel
    // ========================================================================

    println!("[Tranche 1] Test de l'invariant I9 (déterminisme du replai)...");
    if vestiges_horizons::verify_i9_determinism(42, 100) {
        println!("✓ I9 VALIDE : Le replai est déterministe");
    } else {
        println!("✗ I9 INVALIDE : Le replai n'est pas déterministe");
        std::process::exit(1);
    }

    println!("\n[Tranche 1] Test de l'invariant I1 (conservation extraction/réinjection)...");
    if vestiges_horizons::verify_i1_conservation() {
        println!("✓ I1 VALIDE : Conservation vérifiée");
    } else {
        println!("✗ I1 INVALIDE : Conservation échouée");
        std::process::exit(1);
    }

    // ========================================================================
    // Tests Tranche 2 : Identités et Journal
    // ========================================================================

    println!("\n[Tranche 2] Test T2.1 : Extraction répétée (100 cycles)...");
    let mut manager = EntityManager::new(42);
    let cohort_id = manager.create_cohort(100);
    let initial_state = {
        let c = manager.cohort(cohort_id).unwrap();
        (c.population, c.total_money, c.total_griefs, c.total_attachment)
    };
    for _ in 0..100 {
        let individual = manager.extract(cohort_id);
        if let Some(ind) = individual {
            manager.reinject(ind);
        }
    }
    let final_state = {
        let c = manager.cohort(cohort_id).unwrap();
        (c.population, c.total_money, c.total_griefs, c.total_attachment)
    };
    if initial_state == final_state {
        println!("✓ T2.1 VALIDE : Zéro dérive après 100 cycles");
    } else {
        println!("✗ T2.1 INVALIDE : Dérive détectée");
        std::process::exit(1);
    }

    println!("\n[Tranche 2] Test T2.5 : Non-réutilisation des ids...");
    let mut registry = EntityRegistry::new();
    for _ in 0..100_000 {
        let id = registry.next_id(EntityType::NamedIndividual);
        registry.activate(id);
    }
    if registry.verify_no_reuse() {
        println!("✓ T2.5 VALIDE : Aucun id réutilisé");
    } else {
        println!("✗ T2.5 INVALIDE : Id réutilisé détecté");
        std::process::exit(1);
    }

    println!("\n[Tranche 2] Test T2.6 : Fusion conservatrice...");
    let mut manager = EntityManager::new(42);
    let cohort_a = manager.create_cohort(50);
    let cohort_b = manager.create_cohort(50);
    {
        let a = manager.cohort_mut(cohort_a).unwrap();
        a.total_money = 10000;
        a.total_griefs = 500;
    }
    {
        let b = manager.cohort_mut(cohort_b).unwrap();
        b.total_money = 15000;
        b.total_griefs = 750;
    }
    manager.merge_cohorts(cohort_a, cohort_b);
    let merged = manager.cohort(cohort_a).unwrap();
    if merged.total_money == 25000 && merged.total_griefs == 1250 {
        println!("✓ T2.6 VALIDE : Fusion conservatrice");
    } else {
        println!("✗ T2.6 INVALIDE : Totaux non conservés");
        std::process::exit(1);
    }

    println!("\n[Tranche 2] Test I8 : Concordance griefs...");
    let mut manager = EntityManager::new(42);
    let cohort_id = manager.create_cohort(100);
    for _ in 0..10 {
        manager.extract(cohort_id);
    }
    if manager.verify_i8_concordance() {
        println!("✓ I8 VALIDE : Concordance griefs");
    } else {
        println!("✗ I8 INVALIDE : Concordance échouée");
        std::process::exit(1);
    }

    println!("\n========================================");
    println!("Tous les tests passés !");
    println!("Invariants validés : I1, I7, I8, I9");
    println!("Tranches validées : 1 et 2");
    println!("Génération des replais : cargo run --bin generer_replai -- --output-dir <dir> <seed> <tranche>");
    println!("========================================");
}
