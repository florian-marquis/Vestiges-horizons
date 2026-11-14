# Template de Test d'Invariant

**Usage** : Copier ce template pour créer de nouveaux tests d'invariants.
**Localisation** : À placer dans le module concerné (ex: `src/market.rs`, `src/credit.rs`) ou dans `tests/`.

---

## Template de base pour I9 (Déterminisme)

```rust
#[test]
fn test_i9_<module>_<scenario>() {
    // 1. Setup : Créer deux instances identiques avec la même graine
    let seed = 42;  // Graine déterministe
    
    let mut manager1 = <ModuleManager>::new_with_seed(seed);
    let mut manager2 = <ModuleManager>::new_with_seed(seed);
    
    // 2. Action : Appliquer les mêmes opérations aux deux instances
    // (Utiliser des données déterministes, pas de random)
    manager1.operation_1(value);
    manager2.operation_1(value);
    
    manager1.operation_2(value);
    manager2.operation_2(value);
    
    // 3. Assertion : Vérifier que les deux instances sont identiques
    assert!(
        manager1.verify_i9_deterministic(&manager2),
        "I9 échoué : les états divergent après les mêmes opérations"
    );
    
    // Optionnel : Vérifier des champs spécifiques
    assert_eq!(manager1.field1, manager2.field1);
    assert_eq!(manager1.field2, manager2.field2);
}
```

---

## Template pour I1 (Extraction/Réinjection)

```rust
#[test]
fn test_i1_<module>_<scenario>() {
    // 1. Setup : Créer un état initial
    let mut manager = <ModuleManager>::new();
    let initial_state = manager.get_state_snapshot();
    
    // 2. Action : Extraire et réinjecter
    let extracted = manager.extract_entity(entity_id);
    manager.reinject_entity(extracted);
    
    // 3. Assertion : Vérifier la conservation
    let final_state = manager.get_state_snapshot();
    
    // Vérifier que tous les totaux sont conservés
    assert_eq!(
        initial_state.total_effectif, 
        final_state.total_effectif,
        "I1 échoué : effectif non conservé"
    );
    assert_eq!(
        initial_state.total_monnaie, 
        final_state.total_monnaie,
        "I1 échoué : monnaie non conservée"
    );
    assert_eq!(
        initial_state.total_griefs, 
        final_state.total_griefs,
        "I1 échoué : griefs non conservés"
    );
    assert_eq!(
        initial_state.total_attachement, 
        final_state.total_attachement,
        "I1 échoué : attachement non conservé"
    );
}
```

---

## Template pour I3 (Équilibre comptable)

```rust
#[test]
fn test_i3_<module>_<scenario>() {
    // 1. Setup : Créer une transaction
    let mut manager = <ModuleManager>::new();
    let initial_balance = manager.get_total_balance();
    
    // 2. Action : Effectuer une transaction
    manager.process_transaction(sender, receiver, amount);
    
    // 3. Assertion : Vérifier que le total reste inchangé
    let final_balance = manager.get_total_balance();
    assert_eq!(
        initial_balance, 
        final_balance,
        "I3 échoué : déséquilibre comptable de {}",
        final_balance - initial_balance
    );
    
    // Optionnel : Vérifier que le débit = crédit
    assert_eq!(
        manager.get_debit_total(),
        manager.get_credit_total(),
        "I3 échoué : débit ≠ crédit"
    );
}
```

---

## Template combiné (I1 + I9)

```rust
#[test]
fn test_<module>_<scenario>_full_invariants() {
    // Setup avec graine déterministe
    let seed = 42;
    let mut manager1 = <ModuleManager>::new_with_seed(seed);
    let mut manager2 = <ModuleManager>::new_with_seed(seed);
    
    // Snapshot initial pour I1
    let initial_state1 = manager1.get_state_snapshot();
    let initial_state2 = manager2.get_state_snapshot();
    
    // Appliquer les opérations
    manager1.extract_and_reinject(entity_id);
    manager2.extract_and_reinject(entity_id);
    
    // Vérifier I1 : conservation
    let final_state1 = manager1.get_state_snapshot();
    let final_state2 = manager2.get_state_snapshot();
    
    assert_eq!(initial_state1.total_effectif, final_state1.total_effectif);
    assert_eq!(initial_state1.total_monnaie, final_state1.total_monnaie);
    
    // Vérifier I9 : déterminisme
    assert!(
        manager1.verify_i9_deterministic(&manager2),
        "I9 échoué après extraction/réinjection"
    );
}
```

---

## Bonnes pratiques

### 1. Graines déterministes
- Toujours utiliser une graine fixe pour les tests I9
- Éviter `rand::random()` dans les tests d'invariants

### 2. Données déterministes
- Utiliser des valeurs fixes, pas de générateurs aléatoires
- Si nécessaire, utiliser des générateurs à graine fixe

### 3. Nommage des tests
- Préfixer par l'invariant : `test_i1_...`, `test_i9_...`
- Inclure le module : `test_i9_market_pricing`
- Inclure le scénario : `test_i9_market_high_demand`

### 4. Messages d'erreur clairs
- Toujours inclure le contexte dans les assertions
- Exemple : `"I9 échoué : prix divergent après update_price()"`

### 5. Couverture des edge cases
- Tester les valeurs limites : 0, u64::MAX, values proches des seuils
- Tester les cas de débordement
- Tester les cas de sous-débordement (saturating_sub)

---

## Exemples concrets

### Exemple 1 : Test I9 pour MarketManager
```rust
#[test]
fn test_i9_market_reference_price_update() {
    let seed = 42;
    let mut market1 = MarketManager::new_with_seed(seed);
    let mut market2 = MarketManager::new_with_seed(seed);
    
    // Ajouter de la demande
    market1.add_demand_to_reference(GoodType::Food, 120);
    market2.add_demand_to_reference(GoodType::Food, 120);
    
    // Mettre à jour les prix
    market1.update_reference_prices(0);
    market2.update_reference_prices(0);
    
    // Vérifier I9
    assert!(market1.verify_i9_deterministic_pricing(&market2));
}
```

### Exemple 2 : Test I1 pour CreditManager
```rust
#[test]
fn test_i1_credit_loan_extraction() {
    let mut credit = CreditManager::new();
    let loan_id = credit.create_loan(lender, borrower, 100000, 100, vec![], 12, 0, 30);
    
    // Snapshot avant extraction
    let initial_total = credit.get_total_principal();
    
    // Extraire et réinjecter le prêt
    let loan = credit.extract_loan(loan_id);
    credit.reinject_loan(loan);
    
    // Vérifier I1 : conservation du principal total
    assert_eq!(initial_total, credit.get_total_principal());
}
```

---

## Vérification de la couverture

Avant de valider un module, vérifier que :
- [ ] Tous les invariants applicables ont au moins un test
- [ ] Les tests couvrent les edge cases
- [ ] Les tests utilisent des graines déterministes
- [ ] Les messages d'erreur sont clairs et actionnables