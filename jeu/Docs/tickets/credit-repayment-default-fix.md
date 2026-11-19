# Ticket : Corriger credit.rs — Double division d'intérêt et détection de défaut sans suivi des échéances

**ID** : CREDIT-FIX-001
**Module** : `jeu/src/credit.rs`
**Priorité** : 🟡 **MOYENNE** (hors périmètre T7-T8 / Tranche B1 — à traiter avant raccordement du crédit aux ménages)
**Statut** : Ouvert — **NE PAS CORRIGER en Tranche B1**
**Type** : Correction de bogues logiques et arithmétiques

---

## 📋 **DESCRIPTION**

L'audit du module `jeu/src/credit.rs` réalisé lors du pré-travail de la Tranche B1 a relevé deux défauts distincts :

### 1. Double division par `1000` dans `RepaymentSchedule::new` (`jeu/src/credit.rs:103, 113`)
```rust
// jeu/src/credit.rs:103-114
let principal_per_1000 = principal / 1000; // 1re division par 1000
let part1a = principal_per_1000.checked_mul(interest_rate as u64);
...
let total_interest = part1 / 1000;       // 2e division par 1000 ❌
```
- **Conséquence :** les intérêts totaux et `installment_amount` sont divisés par `1 000 000` au lieu de `1 000`, réduisant les intérêts d'un facteur `1 000` (et annulant les intérêts pour tout `principal < 1 000`).
- **Piste de correction (règle 26) :** calculer en `u128` sans pré-division ni perte de précision, avec politique d'arrondi déclarée.

### 2. `Loan::check_default` ignore les paiements effectués (`jeu/src/credit.rs:272-289`)
```rust
// jeu/src/credit.rs:277-285
if current_day >= self.schedule.next_installment_day {
    // Échéance passée -> défaut ❌ ne vérifie pas si l'échéance a été payée
    self.status = LoanStatus::Default;
    ...
}
```
- **Conséquence :** même si l'emprunteur a payé son échéance à temps via `make_payment(...)`, `make_payment` ne fait pas avancer `schedule.advance()` et `check_default(current_day)` fait basculer le prêt en `LoanStatus::Default` (puis en saisie) dès que `current_day >= next_installment_day` (sauf remboursement intégral ayant déjà passé le statut à `Repaid`).

---

## 🎯 **IMPACT SUR LES TESTS ET REPLAIS (Règle 6)**

- **Replai concerné :** [`jeu/src/replai/reference_seed42_t5.json`](../src/replai/reference_seed42_t5.json) (`generate_t5_bytes` dans `jeu/src/replai_gen.rs:771, 915, 934-950` crée un prêt, appelle `make_payment(loan_id, 10000, 30)` puis `process_due_dates(60)` et `process_seizures(60)`).
- **Alerte Règle 6 :** modifier `RepaymentSchedule::new` ou `Loan::check_default` risque de changer le statut ou les montants du prêt dans `T5` et de faire échouer `cargo test --test test_replais`. **Ne jamais régénérer `reference_seed42_t5.json` sans validation humaine explicite.**
- **Tests existants :** `jeu/src/credit.rs:638` (`test_loan_payment`), `jeu/src/credit.rs:664` (`test_seizure_chain`), `jeu/tests/test_tranche5.rs`.

---

## 📌 **PLAN D'ACTION (hors B1)**

1. **Ne pas toucher `credit.rs` pendant la Tranche B1** (Option A provisoire sans perte de salaire retenue en B1).
2. Lors de l'ouverture dédiée de ce ticket :
   - corriger `RepaymentSchedule::new` en arithmétique `u128` avec arrondi déclaré ;
   - suivre l'échéancier payé / avancement de `next_installment_day` pour `Loan::check_default` ;
   - vérifier l'impact exact sur `test_replais` (`T5`) et soumettre toute divergence à validation humaine avant toute suite.
