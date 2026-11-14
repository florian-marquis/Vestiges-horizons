//! Vestiges Horizons - Tranche 4 : Firmes et Production
//! 
//! Implémentation des firmes selon Docs/spec P1 - quartier vivant.md §16
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::MaintenancePolicy;
use crate::entities::{EntityId, EntityType};

// ============================================================================
// CONSTANTES
// ============================================================================

/// Nombre de créneaux par jour (matin, après-midi, soir, nuit)
pub const NUM_SLOTS: usize = 4;

/// Index des créneaux
pub const MORNING: usize = 0;
pub const AFTERNOON: usize = 1;
pub const EVENING: usize = 2;
pub const NIGHT: usize = 3;

// ============================================================================
// TYPES DE FIRMES
// ============================================================================

/// Type de firme selon la spec P1 §4 et §16
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FirmType {
    /// Atelier de production
    Workshop = 0,
    /// Épicerie (vente de nourriture)
    Grocery = 1,
    /// Restaurant
    Restaurant = 2,
    /// Étuve (bains publics)
    Bathhouse = 3,
    /// Propriétaire foncier
    Landlord = 4,
    /// Crédit (banque)
    Credit = 5,
    /// Dispensaire (soins)
    Clinic = 6,
    /// Maisonnée (firme domestique - §16.1 : "Maisonnée = firme")
    HouseholdFirm = 7,
}

impl FirmType {
    /// Retourne si c'est une maisonnée
    pub fn is_household(&self) -> bool {
        *self == FirmType::HouseholdFirm
    }
    
    /// Retourne la description
    pub fn description(&self) -> &'static str {
        match self {
            FirmType::Workshop => "Atelier",
            FirmType::Grocery => "Épicerie",
            FirmType::Restaurant => "Restaurant",
            FirmType::Bathhouse => "Étuve",
            FirmType::Landlord => "Propriétaire foncier",
            FirmType::Credit => "Crédit",
            FirmType::Clinic => "Dispensaire",
            FirmType::HouseholdFirm => "Maisonnée",
        }
    }
}

// ============================================================================
// POSTES DE TRAVAIL (CRÉNEAUX)
// ============================================================================

/// Créneau de travail
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WorkSlot {
    Morning = 0,
    Afternoon = 1,
    Evening = 2,
    Night = 3,
}

impl WorkSlot {
    pub fn all() -> [WorkSlot; NUM_SLOTS] {
        [WorkSlot::Morning, WorkSlot::Afternoon, WorkSlot::Evening, WorkSlot::Night]
    }
    
    pub fn as_index(&self) -> usize {
        *self as usize
    }
    
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(WorkSlot::Morning),
            1 => Some(WorkSlot::Afternoon),
            2 => Some(WorkSlot::Evening),
            3 => Some(WorkSlot::Night),
            _ => None,
        }
    }
}

// ============================================================================
// AFFECTATION DE POSTE
// ============================================================================

/// Affectation d'un employé à un poste
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssignment {
    pub employee_id: EntityId,
    /// Poste (identifiant de la recette ou rôle)
    pub post_id: u64,
    /// Heures par jour (en demi-heures)
    pub daily_hours: u32,
    /// Salaire horaire (en centimes)
    pub hourly_wage: u64,
    /// Créneaux de travail
    pub slots: BTreeSet<WorkSlot>,
    /// Date de début
    pub start_day: i64,
    /// en essai ?
    pub is_trial: bool,
}

impl JobAssignment {
    pub fn new(
        employee_id: EntityId,
        post_id: u64,
        daily_hours: u32,
        hourly_wage: u64,
        slots: BTreeSet<WorkSlot>,
        start_day: i64,
    ) -> Self {
        JobAssignment {
            employee_id,
            post_id,
            daily_hours,
            hourly_wage,
            slots,
            start_day,
            is_trial: false,
        }
    }
    
    /// Retourne le nombre de créneaux
    pub fn num_slots(&self) -> usize {
        self.slots.len()
    }
    
    /// Retourne si l'employé travaille dans un créneau donné
    pub fn works_in_slot(&self, slot: WorkSlot) -> bool {
        self.slots.contains(&slot)
    }
}

// ============================================================================
// STRUCTURE DE LA FIRME
// ============================================================================

/// Firme - unité de production
/// 
/// §16.1 : "La firme porte les champs de la section 4"
/// §16.1 : "son décideur est un individu nommé"
/// §16.1 : "En tranche 4, les décisions de production sont des politiques en données"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Firm {
    pub id: EntityId,
    /// Type de firme
    pub firm_type: FirmType,
    /// Décideur (individu nommé)
    pub owner_id: EntityId,
    /// Employés (cohorte ou individus nommés)
    pub employees: BTreeMap<EntityId, JobAssignment>,
    /// Actifs de la firme
    pub assets: Vec<EntityId>,
    /// Stock d'intrants (bien_id -> quantité)
    pub input_stock: BTreeMap<u64, u32>,
    /// Stock de produits (bien_id -> quantité)
    pub output_stock: BTreeMap<u64, u32>,
    /// Recettes disponibles (ids)
    pub recipe_ids: Vec<u64>,
    /// Trésorerie (en centimes)
    pub cash: u64,
    /// Dette d'entretien
    pub maintenance_debt: u64,
    /// Créneaux d'ouverture (4 booléens)
    pub opening_slots: [bool; NUM_SLOTS],
    /// Politique d'entretien par défaut
    pub default_maintenance_policy: MaintenancePolicy,
    /// Date de création
    pub creation_day: i64,
}

impl Firm {
    pub fn new(
        id: EntityId,
        firm_type: FirmType,
        owner_id: EntityId,
        creation_day: i64,
    ) -> Self {
        Firm {
            id,
            firm_type,
            owner_id,
            employees: BTreeMap::new(),
            assets: Vec::new(),
            input_stock: BTreeMap::new(),
            output_stock: BTreeMap::new(),
            recipe_ids: Vec::new(),
            cash: 0,
            maintenance_debt: 0,
            opening_slots: [true; NUM_SLOTS], // Ouvert par défaut dans tous les créneaux
            default_maintenance_policy: MaintenancePolicy::Normal,
            creation_day,
        }
    }
    
    /// Ajoute un employé
    pub fn add_employee(&mut self, assignment: JobAssignment) {
        self.employees.insert(assignment.employee_id, assignment);
    }
    
    /// Retire un employé
    pub fn remove_employee(&mut self, employee_id: EntityId) -> Option<JobAssignment> {
        self.employees.remove(&employee_id)
    }
    
    /// Retourne le nombre d'employés
    pub fn employee_count(&self) -> usize {
        self.employees.len()
    }
    
    /// Retourne les heures totales travaillées par jour
    pub fn total_daily_hours(&self) -> u32 {
        self.employees.values().map(|a| a.daily_hours).sum()
    }
    
    /// Retourne la masse salariale quotidienne
    pub fn daily_payroll(&self) -> u64 {
        self.employees.values()
            .map(|a| a.hourly_wage * a.daily_hours as u64)
            .sum()
    }
    
    /// Ajoute un actif
    pub fn add_asset(&mut self, asset_id: EntityId) {
        if !self.assets.contains(&asset_id) {
            self.assets.push(asset_id);
        }
    }
    
    /// Retire un actif
    pub fn remove_asset(&mut self, asset_id: EntityId) -> bool {
        if let Some(pos) = self.assets.iter().position(|&id| id == asset_id) {
            self.assets.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Retourne si la firme est ouverte dans un créneau
    pub fn is_open_in_slot(&self, slot: WorkSlot) -> bool {
        self.opening_slots[slot.as_index()]
    }
    
    /// Définit les créneaux d'ouverture
    pub fn set_opening_slots(&mut self, slots: [bool; NUM_SLOTS]) {
        self.opening_slots = slots;
    }
    
    /// Ajoute du stock d'intrants
    pub fn add_input_stock(&mut self, good_id: u64, amount: u32) {
        *self.input_stock.entry(good_id).or_insert(0) += amount;
    }
    
    /// Consomme du stock d'intrants
    /// Retourne la quantité réellement consommée
    pub fn consume_input_stock(&mut self, good_id: u64, amount: u32) -> u32 {
        let available = self.input_stock.get(&good_id).copied().unwrap_or(0);
        let consumed = amount.min(available);
        if consumed > 0 {
            *self.input_stock.get_mut(&good_id).unwrap() -= consumed;
            if self.input_stock[&good_id] == 0 {
                self.input_stock.remove(&good_id);
            }
        }
        consumed
    }
    
    /// Ajoute du stock de produits
    pub fn add_output_stock(&mut self, good_id: u64, amount: u32) {
        *self.output_stock.entry(good_id).or_insert(0) += amount;
    }
    
    /// Vend des produits
    pub fn sell_output(&mut self, good_id: u64, amount: u32, price: u64) -> u32 {
        let available = self.output_stock.get(&good_id).copied().unwrap_or(0);
        let sold = amount.min(available);
        if sold > 0 {
            *self.output_stock.get_mut(&good_id).unwrap() -= sold;
            if self.output_stock[&good_id] == 0 {
                self.output_stock.remove(&good_id);
            }
            self.cash += sold as u64 * price;
        }
        sold
    }
    
    /// Achete des intrants
    pub fn buy_input(&mut self, good_id: u64, amount: u32, price: u64) -> u32 {
        let cost = amount as u64 * price;
        if self.cash >= cost {
            self.cash -= cost;
            self.add_input_stock(good_id, amount);
            amount
        } else {
            // Acheter ce qu'on peut
            let affordable = (self.cash / price) as u32;
            if affordable > 0 {
                self.cash -= affordable as u64 * price;
                self.add_input_stock(good_id, affordable);
            }
            affordable
        }
    }
}

// ============================================================================
// GESTIONNAIRE DE FIRMES
// ============================================================================

/// Gestionnaire des firmes
pub struct FirmManager {
    /// Firmes par id
    firms: BTreeMap<EntityId, Firm>,
    /// Firmes par type
    firms_by_type: BTreeMap<FirmType, BTreeSet<EntityId>>,
    /// Compteur pour les ids
    next_firm_id: u64,
}

impl FirmManager {
    pub fn new() -> Self {
        FirmManager {
            firms: BTreeMap::new(),
            firms_by_type: BTreeMap::new(),
            next_firm_id: 0,
        }
    }
    
    /// Crée une nouvelle firme
    pub fn create_firm(
        &mut self,
        firm_type: FirmType,
        owner_id: EntityId,
        day: i64,
    ) -> EntityId {
        let id = EntityId::new(EntityType::Firm, self.next_firm_id);
        self.next_firm_id += 1;
        
        let firm = Firm::new(id, firm_type, owner_id, day);
        self.firms.insert(id, firm);
        self.firms_by_type.entry(firm_type).or_default().insert(id);
        
        id
    }
    
    /// Retourne une firme par id
    pub fn get_firm(&self, id: EntityId) -> Option<&Firm> {
        self.firms.get(&id)
    }
    
    /// Retourne une firme par id (mutable)
    pub fn get_firm_mut(&mut self, id: EntityId) -> Option<&mut Firm> {
        self.firms.get_mut(&id)
    }
    
    /// Retourne toutes les firmes
    pub fn firms(&self) -> impl Iterator<Item = (&EntityId, &Firm)> {
        self.firms.iter()
    }
    
    /// Retourne toutes les firmes d'un type donné
    pub fn get_firms_by_type(&self, firm_type: FirmType) -> Vec<&Firm> {
        self.firms_by_type.get(&firm_type)
            .map(|ids| ids.iter().filter_map(|&id| self.firms.get(&id)).collect())
            .unwrap_or_default()
    }
    
    /// Ajoute un employé à une firme
    pub fn add_employee_to_firm(
        &mut self,
        firm_id: EntityId,
        assignment: JobAssignment,
    ) -> bool {
        if let Some(firm) = self.firms.get_mut(&firm_id) {
            firm.add_employee(assignment);
            true
        } else {
            false
        }
    }
    
    /// Retire un employé d'une firme
    pub fn remove_employee_from_firm(
        &mut self,
        firm_id: EntityId,
        employee_id: EntityId,
    ) -> Option<JobAssignment> {
        self.firms.get_mut(&firm_id).and_then(|firm| firm.remove_employee(employee_id))
    }
    
    /// Retourne le nombre total de firmes
    pub fn firm_count(&self) -> usize {
        self.firms.len()
    }
    
    /// Vérifie I2 : aucun stock négatif
    /// 
    /// §16.5 T4.1 : "Stocks : 1 an de production agressive sous bouchons | 
    /// I2 vert : aucun stock négatif, jamais"
    pub fn verify_i2_no_negative_stocks(&self) -> bool {
        for firm in self.firms.values() {
            for (_, &stock) in &firm.input_stock {
                if stock < 0 {
                    return false;
                }
            }
            for (_, &stock) in &firm.output_stock {
                if stock < 0 {
                    return false;
                }
            }
        }
        true
    }
}

impl Default for FirmManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

