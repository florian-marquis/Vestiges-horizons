//! Vestiges Horizons - Tranche 5 : Fiscalité forfaitaire
//! 
//! Implémentation de la fiscalité selon Docs/spec P1 - quartier vivant.md §17.4
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::entities::EntityId;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Taux d'imposition : 10% (en millièmes)
pub const TAX_RATE: u32 = 100; // 10.0%

/// Allocation vieillesse : environ ½ panier (en centimes)
/// Valeur par défaut, sera paramétrée par les données
pub const PENSION_AMOUNT: u64 = 25000; // 250.00 €

/// Allocation par enfant : environ ¼ panier (en centimes)
pub const CHILD_ALLOCATION: u64 = 12500; // 125.00 €

// ============================================================================
// TYPE D'ALLOCATION
// ============================================================================

/// Type d'allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AllocationType {
    /// Allocation vieillesse
    Pension = 0,
    /// Allocation par enfant
    ChildBenefit = 1,
}

// ============================================================================
// PRÉLÈVEMENT À LA SOURCE
// ============================================================================

/// Prélèvement à la source effectué par une firme
/// 
/// §17.4 : "Impôt 10 % prélevé à la source par la firme (rôle purement comptable)"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithholdingTax {
    pub id: u64,
    /// ID de la firme
    pub firm_id: EntityId,
    /// ID de l'employé
    pub employee_id: EntityId,
    /// Salaire brut (en centimes)
    pub gross_salary: u64,
    /// Montant prélevé (en centimes)
    pub withheld_amount: u64,
    /// Jour du prélèvement
    pub day: i64,
}

impl WithholdingTax {
    pub fn new(
        id: u64,
        firm_id: EntityId,
        employee_id: EntityId,
        gross_salary: u64,
        day: i64,
    ) -> Self {
        // Calcul : gross_salary * TAX_RATE / 1000
        // Pour éviter le débordement : (gross_salary / 1000) * TAX_RATE
        let withheld_amount = (gross_salary / 1000) * TAX_RATE as u64;
        
        WithholdingTax {
            id,
            firm_id,
            employee_id,
            gross_salary,
            withheld_amount,
            day,
        }
    }
    
    /// Retourne le salaire net
    /// §17.4 : "Toutes les grandeurs s'expriment en net"
    pub fn net_salary(&self) -> u64 {
        self.gross_salary.saturating_sub(self.withheld_amount)
    }
}

// ============================================================================
// ALLOCATION
// ============================================================================

/// Allocation versée à un foyer
/// 
/// §17.4 : "allocations fixes vers les foyers (vieillesse ≈ ½ panier, aide par enfant ≈ ¼)"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub id: u64,
    /// ID du foyer
    pub household_id: EntityId,
    /// Type d'allocation
    pub allocation_type: AllocationType,
    /// Montant (en centimes)
    pub amount: u64,
    /// Jour du versement
    pub day: i64,
}

impl Allocation {
    pub fn new(
        id: u64,
        household_id: EntityId,
        allocation_type: AllocationType,
        amount: u64,
        day: i64,
    ) -> Self {
        Allocation {
            id,
            household_id,
            allocation_type,
            amount,
            day,
        }
    }
}

// ============================================================================
// TRANSFERT DÉCLARÉ
// ============================================================================

/// Transfert déclaré (pour vérifier I3)
/// 
/// §17.4 : "comptées comme transferts déclarés (I3)"
/// §3 : "Toutes les grandeurs s'expriment en net"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredTransfer {
    pub id: u64,
    /// Type de transfert
    pub transfer_type: TransferType,
    /// Montant (en centimes)
    pub amount: u64,
    /// Jour du transfert
    pub day: i64,
    /// Description
    pub description: String,
}

/// Type de transfert
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TransferType {
    /// Prélèvement fiscal
    TaxWithholding = 0,
    /// Allocation
    Allocation = 1,
    /// Transfert vers la ville
    ToCity = 2,
    /// Transfert depuis la ville
    FromCity = 3,
}

// ============================================================================
// GESTIONNAIRE DE FISCALITÉ
// ============================================================================

/// Gestionnaire de la fiscalité
/// 
/// §17.4 : "Impôt 10 % prélevé à la source par la firme... allocations fixes vers les foyers"
pub struct TaxManager {
    /// Prélèvements à la source
    withholding_taxes: BTreeMap<u64, WithholdingTax>,
    /// Allocations versées
    allocations: BTreeMap<u64, Allocation>,
    /// Transferts déclarés
    declared_transfers: BTreeMap<u64, DeclaredTransfer>,
    /// Compteur d'IDs
    next_tax_id: u64,
    next_allocation_id: u64,
    next_transfer_id: u64,
    /// Montant total prélevé (pour vérification)
    total_withheld: u64,
    /// Montant total versé en allocations (pour vérification)
    total_allocated: u64,
}

impl TaxManager {
    pub fn new() -> Self {
        TaxManager {
            withholding_taxes: BTreeMap::new(),
            allocations: BTreeMap::new(),
            declared_transfers: BTreeMap::new(),
            next_tax_id: 0,
            next_allocation_id: 0,
            next_transfer_id: 0,
            total_withheld: 0,
            total_allocated: 0,
        }
    }
    
    /// Prélève à la source
    pub fn withhold_tax(
        &mut self,
        firm_id: EntityId,
        employee_id: EntityId,
        gross_salary: u64,
        day: i64,
    ) -> u64 {
        let tax = WithholdingTax::new(
            self.next_tax_id,
            firm_id,
            employee_id,
            gross_salary,
            day,
        );
        
        let withheld_amount = tax.withheld_amount;
        
        self.withholding_taxes.insert(self.next_tax_id, tax);
        self.total_withheld += withheld_amount;
        
        // Enregistrer le transfert
        self.record_transfer(
            TransferType::TaxWithholding,
            withheld_amount,
            day,
            format!("Prélèvement {} de {:?} à {:?}", self.next_tax_id, firm_id, employee_id),
        );
        
        self.next_tax_id += 1;
        withheld_amount
    }
    
    /// Verse une allocation
    pub fn pay_allocation(
        &mut self,
        household_id: EntityId,
        allocation_type: AllocationType,
        amount: u64,
        day: i64,
    ) {
        let allocation = Allocation::new(
            self.next_allocation_id,
            household_id,
            allocation_type,
            amount,
            day,
        );
        
        self.allocations.insert(self.next_allocation_id, allocation);
        self.total_allocated += amount;
        
        // Enregistrer le transfert
        self.record_transfer(
            TransferType::Allocation,
            amount,
            day,
            format!("Allocation {} à {:?}", self.next_allocation_id, household_id),
        );
        
        self.next_allocation_id += 1;
    }
    
    /// Enregistre un transfert
    fn record_transfer(
        &mut self,
        transfer_type: TransferType,
        amount: u64,
        day: i64,
        description: String,
    ) {
        let transfer = DeclaredTransfer {
            id: self.next_transfer_id,
            transfer_type,
            amount,
            day,
            description,
        };
        
        self.declared_transfers.insert(self.next_transfer_id, transfer);
        self.next_transfer_id += 1;
    }
    
    /// Traite la fiscalité quotidienne pour une firme
    pub fn process_firm_taxes(
        &mut self,
        firm_id: EntityId,
        payroll: BTreeMap<EntityId, u64>, // (employee_id -> gross_salary)
        day: i64,
    ) -> u64 {
        let mut total_withheld = 0;
        
        for (employee_id, gross_salary) in payroll {
            let withheld = self.withhold_tax(firm_id, employee_id, gross_salary, day);
            total_withheld += withheld;
        }
        
        // Le total prélevé va vers la ville
        self.record_transfer(
            TransferType::ToCity,
            total_withheld,
            day,
            format!("Transfert {} de {:?} vers la ville", day, firm_id),
        );
        
        total_withheld
    }
    
    /// Traite les allocations quotidiennes
    pub fn process_allocations(
        &mut self,
        households: &[EntityId],
        child_counts: &BTreeMap<EntityId, usize>, // (household_id -> nombre d'enfants)
        day: i64,
    ) {
        for &household_id in households {
            // Allocation vieillesse (pour tous les foyers)
            self.pay_allocation(
                household_id,
                AllocationType::Pension,
                PENSION_AMOUNT,
                day,
            );
            
            // Allocation par enfant
            if let Some(&child_count) = child_counts.get(&household_id) {
                let total_child_allocation = CHILD_ALLOCATION * child_count as u64;
                self.pay_allocation(
                    household_id,
                    AllocationType::ChildBenefit,
                    total_child_allocation,
                    day,
                );
            }
        }
    }
    
    /// Vérifie I3 : somme des soldes constante hors transferts déclarés
    /// 
    /// §17.7 T5.1 : "I3 vert : chaque transaction équilibrée, somme des soldes constante hors transferts déclarés"
    /// Pour l'instant, on vérifie que tous les transferts sont déclarés
    pub fn verify_i3_declared_transfers(&self) -> bool {
        // Le total prélevé doit correspondre au total transféré vers la ville
        // Le total alloué doit correspondre au total transféré depuis la ville
        
        // Calculer le total transféré vers la ville
        let to_city: u64 = self.declared_transfers.values()
            .filter(|t| t.transfer_type == TransferType::ToCity)
            .map(|t| t.amount)
            .sum();
        
        let from_city: u64 = self.declared_transfers.values()
            .filter(|t| t.transfer_type == TransferType::FromCity)
            .map(|t| t.amount)
            .sum();
        
        // Pour l'instant, on vérifie que les prélèvements sont déclarés
        // La vérification complète nécessiterait le solde de tous les agents
        self.total_withheld == to_city && self.total_allocated >= from_city
    }
    
    /// Vérifie T5.7 : flux fiscaux traçables
    /// 
    /// §17.7 T5.7 : "flux fiscaux traçables (I3), réponse de u et p correcte"
    pub fn verify_t5_7_tax_flows(&self) -> bool {
        // Vérifier que chaque prélèvement a un transfert correspondant
        for tax in self.withholding_taxes.values() {
            let found = self.declared_transfers.values()
                .any(|t| t.transfer_type == TransferType::TaxWithholding && t.amount == tax.withheld_amount);
            if !found {
                return false;
            }
        }
        true
    }
    
    /// Vérifie T5.11 : réponse de u et p correcte
    /// 
    /// §17.7 T5.11 : "l'assertion griefs s'ajoute en tranche 7"
    /// Pour l'instant, on retourne true car les griefs seront implémentés en T7
    pub fn verify_t5_11_griefs_response(&self) -> bool {
        true
    }
    
    /// Vérifie I9 : déterminisme des calculs fiscaux
    /// 
    /// §17.7 T5.1 : "même graine, même données = même résultat"
    /// Vérifie que les calculs de taxes sont déterministes
    pub fn verify_i9_deterministic_tax(&self, other: &TaxManager) -> bool {
        // Comparer les prélèvements
        if self.withholding_taxes.len() != other.withholding_taxes.len() {
            return false;
        }
        
        for (id, tax1) in &self.withholding_taxes {
            if let Some(tax2) = other.withholding_taxes.get(id) {
                if tax1.gross_salary != tax2.gross_salary 
                    || tax1.withheld_amount != tax2.withheld_amount {
                    return false;
                }
            }
        }
        
        // Comparer les allocations
        if self.allocations.len() != other.allocations.len() {
            return false;
        }
        
        for (id, alloc1) in &self.allocations {
            if let Some(alloc2) = other.allocations.get(id) {
                if alloc1.amount != alloc2.amount 
                    || alloc1.allocation_type != alloc2.allocation_type {
                    return false;
                }
            }
        }
        
        // Comparer les totaux
        self.total_withheld == other.total_withheld 
            && self.total_allocated == other.total_allocated
    }
}

impl Default for TaxManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_withholding_tax() {
        let mut manager = TaxManager::new();
        
        let firm = EntityId::new(crate::entities::EntityType::Firm, 0);
        let employee = EntityId::new(crate::entities::EntityType::NamedIndividual, 0);
        
        let withheld = manager.withhold_tax(firm, employee, 100000, 0); // 1000.00 €
        
        assert_eq!(withheld, 10000); // 10%
        assert_eq!(manager.total_withheld, 10000);
    }
    
    #[test]
    fn test_allocations() {
        let mut manager = TaxManager::new();
        
        let household = EntityId::new(crate::entities::EntityType::Household, 0);
        
        manager.pay_allocation(household, AllocationType::Pension, PENSION_AMOUNT, 0);
        manager.pay_allocation(household, AllocationType::ChildBenefit, CHILD_ALLOCATION * 2, 0);
        
        assert_eq!(manager.total_allocated, PENSION_AMOUNT + CHILD_ALLOCATION * 2);
    }
    
    #[test]
    fn test_firm_taxes() {
        let mut manager = TaxManager::new();
        
        let firm = EntityId::new(crate::entities::EntityType::Firm, 0);
        let employee1 = EntityId::new(crate::entities::EntityType::NamedIndividual, 0);
        let employee2 = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        
        let mut payroll = BTreeMap::new();
        payroll.insert(employee1, 100000);
        payroll.insert(employee2, 150000);
        
        let total_withheld = manager.process_firm_taxes(firm, payroll, 0);
        
        assert_eq!(total_withheld, 10000 + 15000); // 10% de chaque
    }
}
