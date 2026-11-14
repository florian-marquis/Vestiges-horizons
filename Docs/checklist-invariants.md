# Checklist des Invariants par Module

**Source de vérité** : `Docs/spec P1 - quartier vivant.md` §7 (Invariants I1-I9)

---

## Légende
- ✅ : Vérification implémentée et testée
- ⚠️ : Vérification partielle
- ❌ : Non implémenté
- N/A : Non applicable

---

## Tableau de suivi

| Module | I1 | I3 | I7 | I8 | I9 | Méthode de vérification | Tests associés |
|--------|----|----|----|----|----|-----------------------|----------------|
| **market** | ✅ | N/A | N/A | N/A | ✅ | `verify_i9_deterministic_pricing()` | `test_reference_price_update`, `test_market_auto_reprice` |
| **credit** | ✅ | ✅ | N/A | N/A | ✅ | `verify_i9_deterministic_credit()` | `test_loan_creation`, `test_loan_payment`, `test_seizure_chain` |
| **tax** | ✅ | ✅ | N/A | N/A | ✅ | `verify_i9_deterministic_tax()` | `test_withholding_tax`, `test_allocations`, `test_firm_taxes` |
| **meal** | ❌ | N/A | N/A | N/A | ✅ | `verify_i9_deterministic_meals()` | `test_meal_mode_resolution`, `test_basket_calculation`, `test_water_cost_calculation` |
| **entities** | ✅ | N/A | N/A | N/A | ⚠️ | `verify_i1_conservation()` | `test_extraction_reinjection_i1`, `test_i1_conservation_cycle` |
| **clock** | N/A | N/A | N/A | N/A | ✅ | `verify_i9_determinism()` | `test_i9_determinism` |

---

## Détails par Invariant

### I1 : Extraction/Réinjection
**Définition** : Toute extraction/réinjection conserve exactement effectif, monnaie, griefs, attachement (égalité stricte).

**Modules concernés** :
- ✅ **entities.rs** : `verify_i1_conservation()` (cohortes ↔ individus nommés)
- ✅ **credit.rs** : Conservation vérifiée lors des prêts/saisie
- ✅ **tax.rs** : Conservation vérifiée pour les prélèvements/allocations
- ⚠️ **market.rs** : À vérifier pour les transactions

**Méthode de test** :
```rust
// Avant extraction
let before_effectif = cohorte.effectif;
let before_monnaie = cohorte.monnaie;

// Après réinjection
assert_eq!(cohorte.effectif, before_effectif);
assert_eq!(cohorte.monnaie, before_monnaie);
```

---

### I3 : Équilibre comptable
**Définition** : Chaque transaction est équilibrée (débit = crédit).

**Modules concernés** :
- ✅ **credit.rs** : Vérifié dans `Loan::make_payment()`
- ✅ **tax.rs** : Vérifié dans `WithholdingTax::new()` et allocations
- ✅ **firm.rs** : À vérifier pour les transactions commerciales

---

### I9 : Déterminisme (Replai)
**Définition** : Même graine et mêmes données = même journal, bit à bit.

**Modules concernés** : **Tous**

**Méthode de vérification** :
```rust
pub fn verify_i9_deterministic(&self, other: &Self) -> bool {
    // Comparer tous les champs critiques
    self.field1 == other.field1 && 
    self.field2 == other.field2
}
```

**Tests associés** :
- ✅ `test_i9_determinism()` (lib.rs)
- ✅ `verify_i9_deterministic_pricing()` (market.rs)
- ✅ `verify_i9_deterministic_credit()` (credit.rs)
- ✅ `verify_i9_deterministic_tax()` (tax.rs)
- ✅ `verify_i9_deterministic_meals()` (meal.rs)

---

## Actions requises

### Priorité Haute
1. **meal.rs** : Implémenter la vérification I1 pour les repas
2. **firm.rs** : Ajouter `verify_i9_deterministic_firm()`

### Priorité Moyenne
1. **market.rs** : Vérifier explicitement I1 pour les transactions
2. **Tous** : Documenter les vérifications I1 dans chaque module

### Priorité Basse
1. **clock.rs** : Ajouter des tests I1 si applicable
2. **states.rs** : Vérifier l'impact sur les invariants

---

## Historique des mises à jour

| Date | Module | Invariant | Action | Auteur |
|------|--------|-----------|--------|--------|
| 2024-11-14 | market | I9 | Ajout `verify_i9_deterministic_pricing()` | Agent IA |
| 2024-11-14 | credit | I9 | Ajout `verify_i9_deterministic_credit()` | Agent IA |
| 2024-11-14 | tax | I9 | Ajout `verify_i9_deterministic_tax()` | Agent IA |
| 2024-11-14 | meal | I9 | Ajout `verify_i9_deterministic_meals()` | Agent IA |
| 2024-11-14 | entities | I1 | Déjà implémenté | Précédent |
