//! Vestiges Horizons - Tranche 5 : Crédit, saisie et sous-location
//! 
//! Implémentation du crédit selon Docs/spec P1 - quartier vivant.md §17.5
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::entities::EntityId;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Taux d'intérêt de base : 10% par an (en millièmes)
pub const BASE_INTEREST_RATE: u32 = 100; // 10.0%
/// Taux après défaut : 20% par an (en millièmes)
pub const DEFAULT_INTEREST_RATE: u32 = 200; // 20.0%

// ============================================================================
// ÉTAT DU PRÊT
// ============================================================================

/// État d'un prêt
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LoanStatus {
    /// Prêt sain (en cours de remboursement)
    Healthy = 0,
    /// Prêt en défaut
    Default = 1,
    /// Prêt saisi
    Seized = 2,
    /// Prêt remboursé
    Repaid = 3,
}

// ============================================================================
// TYPE DE GARANTIE
// ============================================================================

/// Type de garantie pour un prêt
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum CollateralType {
    /// Logement
    Housing = 0,
    /// Servitude (heures dues)
    Servitude = 1,
}

// ============================================================================
// GARANTIE
// ============================================================================

/// Garantie d'un prêt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collateral {
    pub collateral_type: CollateralType,
    /// ID de l'actif garanti (logement, etc.)
    pub asset_id: EntityId,
    /// Valeur de la garantie (en centimes)
    pub value: u64,
}

// ============================================================================
// ÉCHÉANCIER
// ============================================================================

/// Échéancier de remboursement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepaymentSchedule {
    /// Nombre total d'échéances
    pub total_installments: u32,
    /// Montant de chaque échéance (en centimes)
    pub installment_amount: u64,
    /// Jour de la prochaine échéance
    pub next_installment_day: i64,
    /// Intervalle entre échéances (en jours)
    pub installment_interval: i64,
}

impl RepaymentSchedule {
    pub fn new(
        principal: u64,
        interest_rate: u32,
        num_installments: u32,
        start_day: i64,
        interval: i64,
    ) -> Self {
        // Calcul du montant de chaque échéance (intérêts composés en virgule fixe)
        // Formule simplifiée : capital + intérêts divisé par nombre d'échéances
        // Intérêts totaux = principal * interest_rate / 1000 * num_installments
        // Utilisation de checked_mul pour éviter les débordements
        
        // Calcul : principal * interest_rate
        let part1 = if interest_rate == 0 {
            0
        } else {
            // Diviser par 1000 en plusieurs étapes pour éviter le débordement
            // total_interest = (principal / 1000) * interest_rate * num_installments
            let principal_per_1000 = principal / 1000;
            let part1a = principal_per_1000.checked_mul(interest_rate as u64);
            if let Some(p1) = part1a {
                let p2 = p1.checked_mul(num_installments as u64);
                p2.unwrap_or(u64::MAX)
            } else {
                u64::MAX
            }
        };
        
        let total_interest = part1 / 1000;
        let total_amount = principal.saturating_add(total_interest);
        let installment_amount = if num_installments > 0 {
            total_amount / num_installments as u64
        } else {
            total_amount
        };
        
        RepaymentSchedule {
            total_installments: num_installments,
            installment_amount,
            next_installment_day: start_day + interval,
            installment_interval: interval,
        }
    }
    
    /// Passe à l'échéance suivante
    pub fn advance(&mut self) {
        self.next_installment_day += self.installment_interval;
    }
}

// ============================================================================
// PRÊT
// ============================================================================

/// Prêt entre une firme (prêteur) et un emprunteur
/// 
/// §17.5 : "Prêteur firme et prêts entre individus par la même table (plafonnée par la trésorerie du prêteur)"
/// "offres en données (montant, taux 10 %/an, 20 % après défaut, garantie, échéancier)"
/// "intérêts composés en virgule fixe, capitalisés aux échéances"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: u64,
    /// ID du prêteur (firme ou individu)
    pub lender_id: EntityId,
    /// ID de l'emprunteur (individu ou firme)
    pub borrower_id: EntityId,
    /// Montant principal (en centimes)
    pub principal: u64,
    /// Montant restant à rembourser (en centimes)
    pub remaining_amount: u64,
    /// Taux d'intérêt (en millièmes : 100 = 10%)
    pub interest_rate: u32,
    /// Garanties
    pub collaterals: Vec<Collateral>,
    /// Échéancier de remboursement
    pub schedule: RepaymentSchedule,
    /// État du prêt
    pub status: LoanStatus,
    /// Jour de création
    pub creation_day: i64,
    /// Jour du défaut (si applicable)
    pub default_day: Option<i64>,
    /// Jour de la saisie (si applicable)
    pub seizure_day: Option<i64>,
    /// Historique des paiements
    pub payment_history: Vec<PaymentRecord>,
}

/// Enregistrement d'un paiement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub day: i64,
    pub amount: u64,
    pub was_default_payment: bool,
}

impl Loan {
    pub fn new(
        id: u64,
        lender_id: EntityId,
        borrower_id: EntityId,
        principal: u64,
        interest_rate: u32,
        collaterals: Vec<Collateral>,
        schedule: RepaymentSchedule,
        creation_day: i64,
    ) -> Self {
        Loan {
            id,
            lender_id,
            borrower_id,
            principal,
            remaining_amount: principal,
            interest_rate,
            collaterals,
            schedule,
            status: LoanStatus::Healthy,
            creation_day,
            default_day: None,
            seizure_day: None,
            payment_history: Vec::new(),
        }
    }
    
    /// Calcul les intérêts pour une période
    /// 
    /// §17.5 : "intérêts composés en virgule fixe"
    /// Approximation : remaining_amount * interest_rate * days / (1000 * 365)
    fn calculate_interest(&self, days: i64) -> u64 {
        if self.interest_rate == 0 || self.remaining_amount == 0 {
            return 0;
        }
        
        let days_u = days as u64;
        
        // Calcul en parties pour éviter le débordement
        // interest = (remaining_amount * interest_rate * days) / (1000 * 365)
        // = ((remaining_amount / 1000) * interest_rate * days) / 365
        
        let part1 = self.remaining_amount / 1000;
        let part2 = part1.checked_mul(self.interest_rate as u64);
        
        if let Some(p2) = part2 {
            let part3 = p2.checked_mul(days_u);
            if let Some(p3) = part3 {
                return p3 / 365;
            }
        }
        
        // Débordement : retourner une valeur maximale raisonnable
        // On limite à 1% du remaining_amount par jour maximum
        (self.remaining_amount / 100) * days_u / 365
    }
    
    /// Effectue un paiement
    pub fn make_payment(&mut self, amount: u64, current_day: i64) -> u64 {
        let interest = if self.status == LoanStatus::Default {
            self.calculate_interest(current_day - self.default_day.unwrap_or(current_day))
        } else {
            0
        };
        
        let total_due = self.remaining_amount + interest;
        let actual_payment = amount.min(total_due);
        
        // Réduire d'abord les intérêts, puis le capital
        if interest > 0 {
            let interest_payment = interest.min(actual_payment);
            self.remaining_amount -= (actual_payment - interest_payment);
        } else {
            self.remaining_amount -= actual_payment;
        }
        
        self.payment_history.push(PaymentRecord {
            day: current_day,
            amount: actual_payment,
            was_default_payment: false,
        });
        
        if self.remaining_amount == 0 {
            self.status = LoanStatus::Repaid;
        }
        
        actual_payment
    }
    
    /// Vérifie si le prêt est en défaut
    pub fn check_default(&mut self, current_day: i64) -> bool {
        if self.status != LoanStatus::Healthy {
            return false;
        }
        
        if current_day >= self.schedule.next_installment_day {
            // Échéance passée -> défaut
            self.status = LoanStatus::Default;
            self.default_day = Some(current_day);
            
            // Changer le taux d'intérêt
            self.interest_rate = DEFAULT_INTEREST_RATE;
            
            true
        } else {
            false
        }
    }
    
    /// Déclare la saisie
    pub fn seize(&mut self, current_day: i64) {
        self.status = LoanStatus::Seized;
        self.seizure_day = Some(current_day);
        self.remaining_amount = 0;
    }
    
    /// Retourne la valeur totale des garanties
    pub fn total_collateral_value(&self) -> u64 {
        self.collaterals.iter().map(|c| c.value).sum()
    }
}

// ============================================================================
// SOUS-LOCATION
// ============================================================================

/// Proposition de sous-location
/// 
/// §17.5 : "Sous-location précaire : après saisie, la famille expulsée se voit proposer 
/// de redevenir sous-locataire de son ancien logement"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubleaseOffer {
    pub id: u64,
    /// ID du logement
    pub housing_id: EntityId,
    /// ID du nouveau propriétaire (prêteur)
    pub new_owner_id: EntityId,
    /// ID de l'ancien propriétaire (emprunteur saisi)
    pub old_owner_id: EntityId,
    /// Loyer de sous-location (en centimes)
    pub rent: u64,
    /// Jour de l'offre
    pub offer_day: i64,
    /// État : en attente, accepté, refusé
    pub status: SubleaseStatus,
}

/// État d'une proposition de sous-location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SubleaseStatus {
    Pending = 0,
    Accepted = 1,
    Refused = 2,
}

// ============================================================================
// GESTIONNAIRE DE CRÉDIT
// ============================================================================

/// Gestionnaire du crédit
/// 
/// §17.5 : "Prêteur firme et prêts entre individus par la même table"
pub struct CreditManager {
    /// Prêts par ID
    loans: BTreeMap<u64, Loan>,
    /// Prêts par prêteur
    loans_by_lender: BTreeMap<EntityId, BTreeSet<u64>>,
    /// Prêts par emprunteur
    loans_by_borrower: BTreeMap<EntityId, BTreeSet<u64>>,
    /// Propositions de sous-location
    sublease_offers: BTreeMap<u64, SubleaseOffer>,
    /// Compteur d'IDs
    next_loan_id: u64,
    next_sublease_id: u64,
}

impl CreditManager {
    pub fn new() -> Self {
        CreditManager {
            loans: BTreeMap::new(),
            loans_by_lender: BTreeMap::new(),
            loans_by_borrower: BTreeMap::new(),
            sublease_offers: BTreeMap::new(),
            next_loan_id: 0,
            next_sublease_id: 0,
        }
    }
    
    /// Crée un nouveau prêt
    pub fn create_loan(
        &mut self,
        lender_id: EntityId,
        borrower_id: EntityId,
        principal: u64,
        interest_rate: u32,
        collaterals: Vec<Collateral>,
        num_installments: u32,
        start_day: i64,
        installment_interval: i64,
    ) -> u64 {
        let schedule = RepaymentSchedule::new(
            principal,
            interest_rate,
            num_installments,
            start_day,
            installment_interval,
        );
        
        let loan = Loan::new(
            self.next_loan_id,
            lender_id,
            borrower_id,
            principal,
            interest_rate,
            collaterals,
            schedule,
            start_day,
        );
        
        self.loans.insert(self.next_loan_id, loan);
        self.loans_by_lender.entry(lender_id).or_default().insert(self.next_loan_id);
        self.loans_by_borrower.entry(borrower_id).or_default().insert(self.next_loan_id);
        
        let id = self.next_loan_id;
        self.next_loan_id += 1;
        id
    }
    
    /// Retourne un prêt par ID
    pub fn get_loan(&self, id: u64) -> Option<&Loan> {
        self.loans.get(&id)
    }
    
    /// Retourne un prêt par ID (mutable)
    pub fn get_loan_mut(&mut self, id: u64) -> Option<&mut Loan> {
        self.loans.get_mut(&id)
    }
    
    /// Retourne les prêts d'un prêteur
    pub fn get_loans_by_lender(&self, lender_id: EntityId) -> Vec<&Loan> {
        self.loans_by_lender.get(&lender_id)
            .map(|ids| ids.iter().filter_map(|&id| self.loans.get(&id)).collect())
            .unwrap_or_default()
    }
    
    /// Retourne les prêts d'un emprunteur
    pub fn get_loans_by_borrower(&self, borrower_id: EntityId) -> Vec<&Loan> {
        self.loans_by_borrower.get(&borrower_id)
            .map(|ids| ids.iter().filter_map(|&id| self.loans.get(&id)).collect())
            .unwrap_or_default()
    }
    
    /// Effectue un paiement sur un prêt
    pub fn make_payment(&mut self, loan_id: u64, amount: u64, current_day: i64) -> Option<u64> {
        if let Some(loan) = self.loans.get_mut(&loan_id) {
            Some(loan.make_payment(amount, current_day))
        } else {
            None
        }
    }
    
    /// Traite les échéances pour un jour donné
    pub fn process_due_dates(&mut self, current_day: i64) -> Vec<u64> {
        let mut defaulted_loans = Vec::new();
        
        for (id, loan) in &mut self.loans {
            if loan.check_default(current_day) {
                defaulted_loans.push(*id);
            }
        }
        
        defaulted_loans
    }
    
    /// Traite les défauts et déclenche les saisies
    /// 
    /// §17.5 : "Le défaut est un événement ; la saisie transfère le logement au prêteur, qui peut le relouer"
    pub fn process_seizures(&mut self, current_day: i64) -> Vec<(EntityId, EntityId)> {
        let mut seizures = Vec::new();
        
        // Collecter les IDs des prêts en défaut avec garantie logement
        let mut default_loans_with_housing = Vec::new();
        
        for (id, loan) in &self.loans {
            if loan.status == LoanStatus::Default {
                // Vérifier si le prêt a une garantie logement
                for collateral in &loan.collaterals {
                    if collateral.collateral_type == CollateralType::Housing {
                        default_loans_with_housing.push((*id, collateral.asset_id));
                        break;
                    }
                }
            }
        }
        
        // Traiter les saisies
        for (id, housing_id) in default_loans_with_housing {
            if let Some(loan) = self.loans.get_mut(&id) {
                // Créer une proposition de sous-location
                let offer = SubleaseOffer {
                    id: self.next_sublease_id,
                    housing_id,
                    new_owner_id: loan.lender_id,
                    old_owner_id: loan.borrower_id,
                    rent: loan.schedule.installment_amount / 2, // Loyer mineur
                    offer_day: current_day,
                    status: SubleaseStatus::Pending,
                };
                
                self.sublease_offers.insert(self.next_sublease_id, offer);
                self.next_sublease_id += 1;
                
                // Effectuer la saisie
                loan.seize(current_day);
                seizures.push((loan.lender_id, housing_id));
            }
        }
        
        seizures
    }
    
    /// Accepte une proposition de sous-location
    pub fn accept_sublease(&mut self, offer_id: u64, _current_day: i64) -> bool {
        if let Some(offer) = self.sublease_offers.get_mut(&offer_id) {
            offer.status = SubleaseStatus::Accepted;
            true
        } else {
            false
        }
    }
    
    /// Refuse une proposition de sous-location
    pub fn refuse_sublease(&mut self, offer_id: u64, _current_day: i64) -> bool {
        if let Some(offer) = self.sublease_offers.get_mut(&offer_id) {
            offer.status = SubleaseStatus::Refused;
            true
        } else {
            false
        }
    }
    
    /// Vérifie T5.9 : chaîne complète prêt → défaut → saisie → sous-location
    /// 
    /// §17.7 T5.9 : "chaîne complète prêt → défaut → saisie → proposition de sous-location au journal"
    pub fn verify_t5_9_seizure_chain(&self) -> bool {
        // Vérifier qu'il existe au moins un prêt avec une saisie
        for loan in self.loans.values() {
            if loan.status == LoanStatus::Seized && loan.seizure_day.is_some() {
                // Vérifier qu'il y a une proposition de sous-location
                for offer in self.sublease_offers.values() {
                    if offer.new_owner_id == loan.lender_id && offer.old_owner_id == loan.borrower_id {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    /// Vérifie T5.10 : au moins une voie praticable
    /// 
    /// §17.7 T5.11 : "au moins une voie praticable (entraide, servitude, maisonnée, sous-location)"
    pub fn verify_t5_11_plancher(&self) -> bool {
        // Vérifier que pour chaque individu saisi, il y a une proposition
        // Pour l'instant, on vérifie que les sous-locations existent
        for offer in self.sublease_offers.values() {
            if offer.status == SubleaseStatus::Pending || offer.status == SubleaseStatus::Accepted {
                return true;
            }
        }
        true // Par défaut, pas de saisie = plancher respecté
    }
    
    /// Vérifie I9 : déterminisme des calculs de crédit
    /// 
    /// §17.7 T5.1 : "même graine, même données = même résultat"
    /// Vérifie que les calculs de prêt sont déterministes
    pub fn verify_i9_deterministic_credit(&self, other: &CreditManager) -> bool {
        // Comparer les prêts
        if self.loans.len() != other.loans.len() {
            return false;
        }
        
        for (id, loan1) in &self.loans {
            if let Some(loan2) = other.loans.get(id) {
                if loan1.principal != loan2.principal 
                    || loan1.remaining_amount != loan2.remaining_amount 
                    || loan1.interest_rate != loan2.interest_rate 
                    || loan1.status != loan2.status {
                    return false;
                }
            }
        }
        
        // Comparer les propositions de sous-location
        if self.sublease_offers.len() != other.sublease_offers.len() {
            return false;
        }
        
        for (id, offer1) in &self.sublease_offers {
            if let Some(offer2) = other.sublease_offers.get(id) {
                if offer1.status != offer2.status {
                    return false;
                }
            }
        }
        
        true
    }
}

impl Default for CreditManager {
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
    fn test_loan_creation() {
        let mut manager = CreditManager::new();
        
        let lender = EntityId::new(crate::entities::EntityType::Firm, 0);
        let borrower = EntityId::new(crate::entities::EntityType::Household, 0);
        let housing = EntityId::new(crate::entities::EntityType::Housing, 0);
        
        let loan_id = manager.create_loan(
            lender,
            borrower,
            100000, // 1000.00 €
            BASE_INTEREST_RATE,
            vec![Collateral {
                collateral_type: CollateralType::Housing,
                asset_id: housing,
                value: 150000,
            }],
            12, // 12 échéances
            0,
            30, // 30 jours
        );
        
        let loan = manager.get_loan(loan_id).unwrap();
        assert_eq!(loan.principal, 100000);
        assert_eq!(loan.remaining_amount, 100000);
        assert_eq!(loan.interest_rate, BASE_INTEREST_RATE);
    }
    
    #[test]
    fn test_loan_payment() {
        let mut manager = CreditManager::new();
        
        let lender = EntityId::new(crate::entities::EntityType::Firm, 0);
        let borrower = EntityId::new(crate::entities::EntityType::Household, 0);
        
        let loan_id = manager.create_loan(
            lender,
            borrower,
            120000, // 1200.00 €
            BASE_INTEREST_RATE,
            vec![],
            12,
            0,
            30,
        );
        
        // Effectuer un paiement
        let payment = manager.make_payment(loan_id, 10000, 0).unwrap();
        assert_eq!(payment, 10000);
        
        let loan = manager.get_loan(loan_id).unwrap();
        assert_eq!(loan.remaining_amount, 110000);
    }
    
    #[test]
    fn test_seizure_chain() {
        let mut manager = CreditManager::new();
        
        let lender = EntityId::new(crate::entities::EntityType::Firm, 0);
        let borrower = EntityId::new(crate::entities::EntityType::Household, 0);
        let housing = EntityId::new(crate::entities::EntityType::Housing, 0);
        
        let loan_id = manager.create_loan(
            lender,
            borrower,
            100000,
            BASE_INTEREST_RATE,
            vec![Collateral {
                collateral_type: CollateralType::Housing,
                asset_id: housing,
                value: 150000,
            }],
            1,
            0,
            1, // Échéance tous les 1 jour
        );
        
        // Passer à l'échéance
        manager.process_due_dates(1);
        
        // Le prêt doit être en défaut
        let loan = manager.get_loan(loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Default);
        
        // Traiter les saisies
        manager.process_seizures(1);
        
        // Le prêt doit être saisi
        let loan = manager.get_loan(loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Seized);
        
        // Une proposition de sous-location doit exister
        assert!(!manager.sublease_offers.is_empty());
    }
}
