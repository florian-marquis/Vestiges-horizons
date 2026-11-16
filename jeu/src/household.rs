//! Vestiges Horizons - Tranche 3 : Foyer et Consommation
//! 
//! Implémentation des foyers (households) selon Docs/spec P1 - quartier vivant.md §15.2
//! 
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::entities::{EntityId, EntityType};
use crate::time::TimeBudget;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Nombre de repas par jour
pub const REPAS_PAR_JOUR: u32 = 3;

/// Heures de corvées par jour pour un adulte (en demi-heures)
pub const CORVEE_ADULTE_BASE: u32 = 4; // 2 heures

/// Réduction des corvées pour un logement de qualité 1
pub const CORVEE_REDUCTION_QUALITE_1: u32 = 1; // 0.5 heure

/// Réduction des corvées pour un logement de qualité 2
pub const CORVEE_REDUCTION_QUALITE_2: u32 = 2; // 1 heure

// ============================================================================
// QUALITÉ DE NOURRITURE
// ============================================================================

/// Qualité de la nourriture (affecte la santé)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FoodQuality {
    /// Nourriture de base (qualité 0)
    Base = 0,
    /// Nourriture de qualité moyenne (qualité 1)
    Moyenne = 1,
    /// Nourriture de qualité supérieure (qualité 2)
    Superieure = 2,
}

impl FoodQuality {
    /// Retourne le modificateur de santé pour cette qualité
    pub fn health_modifier(&self) -> i32 {
        match self {
            FoodQuality::Base => 0,
            FoodQuality::Moyenne => 10,
            FoodQuality::Superieure => 20,
        }
    }
    
    /// Retourne le coût relatif (en %)
    pub fn relative_cost(&self) -> u32 {
        match self {
            FoodQuality::Base => 100,
            FoodQuality::Moyenne => 150,
            FoodQuality::Superieure => 250,
        }
    }
}

// ============================================================================
// QUALITÉ DE LOGEMENT
// ============================================================================

/// Qualité du logement (affecte les corvées et le confort)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HousingQuality {
    /// Logement de base (qualité 0) - pas de réduction de corvées
    Base = 0,
    /// Logement de qualité 1 - réduction de 0.5h de corvées
    Qualite1 = 1,
    /// Logement de qualité 2 - réduction de 1h de corvées
    Qualite2 = 2,
}

impl HousingQuality {
    /// Retourne la réduction de corvées en demi-heures
    pub fn corvee_reduction(&self) -> u32 {
        match self {
            HousingQuality::Base => 0,
            HousingQuality::Qualite1 => CORVEE_REDUCTION_QUALITE_1,
            HousingQuality::Qualite2 => CORVEE_REDUCTION_QUALITE_2,
        }
    }
    
    /// Retourne le modificateur de confort
    pub fn comfort_modifier(&self) -> i32 {
        match self {
            HousingQuality::Base => 0,
            HousingQuality::Qualite1 => 5,
            HousingQuality::Qualite2 => 15,
        }
    }
}

// ============================================================================
// FOYER (HOUSEHOLD)
// ============================================================================

/// Foyer - unité domestique
/// 
/// §15.2 : "u du foyer → u des individus/cohorte"
/// Le foyer gère la consommation collective et les corvées.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Household {
    pub id: EntityId,
    /// Propriétaire (individu nommé ou cohorte)
    pub owner_id: EntityId,
    /// Membre du foyer (peuvent être plusieurs individus nommés)
    pub members: Vec<EntityId>,
    /// Qualité du logement
    pub housing_quality: HousingQuality,
    /// Stock de nourriture (en unités)
    pub food_stock: BTreeMap<FoodQuality, u32>,
    /// Stock d'eau (en unités)
    pub water_stock: u32,
    /// Jour de dernière consommation
    pub last_consumption_day: i64,
    /// budget de temps partagé (optionnel)
    pub shared_time_budget: Option<TimeBudget>,
}

impl Household {
    pub fn new(id: EntityId, owner_id: EntityId, housing_quality: HousingQuality) -> Self {
        let mut food_stock = BTreeMap::new();
        food_stock.insert(FoodQuality::Base, 0);
        food_stock.insert(FoodQuality::Moyenne, 0);
        food_stock.insert(FoodQuality::Superieure, 0);
        
        Household {
            id,
            owner_id,
            members: Vec::new(),
            housing_quality,
            food_stock,
            water_stock: 0,
            last_consumption_day: 0,
            shared_time_budget: None,
        }
    }
    
    /// Ajoute un membre au foyer
    pub fn add_member(&mut self, member_id: EntityId) {
        if !self.members.contains(&member_id) {
            self.members.push(member_id);
        }
    }
    
    /// Retire un membre du foyer
    pub fn remove_member(&mut self, member_id: EntityId) -> bool {
        if let Some(pos) = self.members.iter().position(|&id| id == member_id) {
            self.members.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Retourne le nombre total de membres (propriétaire + membres)
    pub fn member_count(&self) -> usize {
        1 + self.members.len() // +1 pour le propriétaire
    }
    
    /// Consomme de la nourriture pour un jour
    /// 
    /// §15.2 : "concrétisation de l'un des besoins financiers : la nourriture.
    /// On stocke de la nourriture, de deux qualités"
    pub fn consume_food(&mut self, day: i64) -> bool {
        // Vérifier qu'on a assez de nourriture
        let total_food: u32 = self.food_stock.values().sum();
        
        if total_food < self.member_count() as u32 {
            // Pas assez de nourriture
            self.last_consumption_day = day;
            return false;
        }
        
        // Consommer la nourriture (priorité à la qualité supérieure)
        let mut remaining = self.member_count() as u32;
        
        // Commencer par la meilleure qualité
        if let Some(superieur) = self.food_stock.get_mut(&FoodQuality::Superieure) {
            let to_consume = remaining.min(*superieur);
            *superieur -= to_consume;
            remaining -= to_consume;
        }
        
        if remaining > 0 {
            if let Some(moyenne) = self.food_stock.get_mut(&FoodQuality::Moyenne) {
                let to_consume = remaining.min(*moyenne);
                *moyenne -= to_consume;
                remaining -= to_consume;
            }
        }
        
        if remaining > 0 {
            if let Some(base) = self.food_stock.get_mut(&FoodQuality::Base) {
                let to_consume = remaining.min(*base);
                *base -= to_consume;
                remaining -= to_consume;
            }
        }
        
        self.last_consumption_day = day;
        remaining == 0
    }
    
    /// Ajoute de la nourriture au stock
    pub fn add_food(&mut self, quality: FoodQuality, amount: u32) {
        *self.food_stock.entry(quality).or_insert(0) += amount;
    }
    
    /// Retourne la quantité totale de nourriture
    pub fn total_food(&self) -> u32 {
        self.food_stock.values().sum()
    }
    
    /// Retourne la quantité de nourriture d'une qualité spécifique
    pub fn get_food_stock(&self, quality: FoodQuality) -> Option<&u32> {
        self.food_stock.get(&quality)
    }
    
    /// Retourne la qualité moyenne de la nourriture
    pub fn avg_food_quality(&self) -> f32 {
        let total: u32 = self.food_stock.values().sum();
        if total == 0 {
            return 0.0;
        }
        
        let weighted: u32 = self.food_stock.iter()
            .map(|(q, &amt)| (q.clone() as u32) * amt)
            .sum();
        
        weighted as f32 / total as f32
    }
    
    /// Calcule les corvées totales pour le foyer
    /// 
    /// §15.2 : "u du foyer → u des individus/cohorte"
    /// Les corvées du foyer sont réparties entre les membres.
    /// 
    /// §15.3 : "Corvées : 0.5-1.5h par enfant (8-14 ans), plafond 2h"
    pub fn calculate_total_corvee(&self) -> u32 {
        // Corvées de base pour un adulte : 2 heures (4 demi-heures)
        // Réduites par la qualité du logement
        let base_corvee = CORVEE_ADULTE_BASE.saturating_sub(self.housing_quality.corvee_reduction());
        
        // Pour l'instant, on multiplie par le nombre de membres
        // (les enfants seront gérés séparément)
        base_corvee * (self.member_count() as u32)
    }
    
    /// Répartit les corvées entre les membres
    pub fn distribute_corvee(&self) -> Vec<(EntityId, u32)> {
        let total = self.calculate_total_corvee();
        let per_member = total / (self.member_count() as u32);
        
        self.members.iter()
            .map(|&member| (member, per_member))
            .collect()
    }

    /// Calcule u (besoins alimentaires non servis / demande) pour ce foyer
    ///
    /// Formule (spec T7-T8 A2 §3) :
    /// - u = 0 si stock > 0 ou demande == 0
    /// - u = ((non_servis * 1000) / demande).min(1000) sinon (troncature entière)
    ///
    /// @param besoin_total Besoin total en nourriture pour le foyer
    /// @returns u en millièmes [0, 1000]
    pub fn calculate_u(&self, besoin_total: u32) -> u16 {
        crate::inputs::calculate_u_from_household(self.total_food(), besoin_total)
    }
}

// ============================================================================
// ENFANTS CONTRIBUTEURS
// ============================================================================

/// Range d'âge des enfants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChildAgeGroup {
    /// Enfant de 0-7 ans (ne travaille pas)
    YoungChild,
    /// Enfant de 8-14 ans (peut faire des corvées)
    ChildContributor,
    /// Jeune adulte de 15-17 ans (corvées + travail partiel)
    YoungAdult,
    /// Adulte (18+)
    Adult,
}

impl ChildAgeGroup {
    pub fn from_age(age: u32) -> Self {
        match age {
            0..=7 => ChildAgeGroup::YoungChild,
            8..=14 => ChildAgeGroup::ChildContributor,
            15..=17 => ChildAgeGroup::YoungAdult,
            _ => ChildAgeGroup::Adult,
        }
    }
    
    /// Retourne les heures de corvées pour cet âge (en demi-heures)
    /// §15.3 : "Corvées : 0.5-1.5h par enfant (8-14 ans), plafond 2h"
    pub fn corvee_hours(&self) -> (u32, u32) {
        match self {
            ChildAgeGroup::YoungChild => (0, 0),
            ChildAgeGroup::ChildContributor => (1, 3), // 0.5-1.5h = 1-3 demi-heures
            ChildAgeGroup::YoungAdult => (2, 4), // 1-2h = 2-4 demi-heures
            ChildAgeGroup::Adult => (4, 4), // 2h = 4 demi-heures
        }
    }
}

/// Enfant contribuant au foyer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildContributor {
    pub id: EntityId,
    /// Âge de l'enfant
    pub age: u32,
    /// Groupe d'âge
    pub age_group: ChildAgeGroup,
    /// Heures de corvées contribuées (en demi-heures)
    pub corvee_hours: u32,
    /// Heures de travail contribuées (en demi-heures) - pour les 15-17 ans
    pub work_hours: u32,
    /// Indice de santé
    pub health_index: u32,
}

impl ChildContributor {
    pub fn new(id: EntityId, age: u32) -> Self {
        let age_group = ChildAgeGroup::from_age(age);
        let (min_corvee, max_corvee) = age_group.corvee_hours();
        
        // Par défaut, contribution moyenne
        let corvee_hours = if min_corvee == max_corvee {
            min_corvee
        } else {
            min_corvee + ((max_corvee - min_corvee) / 2)
        };
        
        ChildContributor {
            id,
            age,
            age_group,
            corvee_hours,
            work_hours: 0, // Les jeunes adultes peuvent travailler
            health_index: 100,
        }
    }
    
    /// Retourne si l'enfant peut travailler
    pub fn can_work(&self) -> bool {
        match self.age_group {
            ChildAgeGroup::YoungAdult => true,
            _ => false,
        }
    }
    
    /// Met à jour l'âge (appelé chaque année)
    pub fn update_age(&mut self, new_age: u32) {
        self.age = new_age;
        self.age_group = ChildAgeGroup::from_age(new_age);
        
        // Réajuster les heures de corvées
        let (min_corvee, max_corvee) = self.age_group.corvee_hours();
        self.corvee_hours = if min_corvee == max_corvee {
            min_corvee
        } else {
            min_corvee + ((max_corvee - min_corvee) / 2)
        };
    }
}

// ============================================================================
// GESTIONNAIRE DE FOYERS
// ============================================================================

/// Gestionnaire des foyers
pub struct HouseholdManager {
    /// Foyers par id
    households: BTreeMap<EntityId, Household>,
    /// Enfants contribuants par foyer
    children: BTreeMap<EntityId, Vec<ChildContributor>>,
    /// Compteur pour les ids de foyer
    next_household_id: u64,
}

impl HouseholdManager {
    pub fn new() -> Self {
        HouseholdManager {
            households: BTreeMap::new(),
            children: BTreeMap::new(),
            next_household_id: 0,
        }
    }
    
    /// Crée un nouveau foyer
    pub fn create_household(
        &mut self,
        owner_id: EntityId,
        housing_quality: HousingQuality,
    ) -> EntityId {
        let id = EntityId::new(EntityType::Household, self.next_household_id);
        self.next_household_id += 1;
        
        let household = Household::new(id, owner_id, housing_quality);
        self.households.insert(id, household);
        self.children.insert(id, Vec::new());
        
        id
    }
    
    /// Ajoute un membre à un foyer
    pub fn add_member_to_household(
        &mut self,
        household_id: EntityId,
        member_id: EntityId,
    ) -> bool {
        if let Some(household) = self.households.get_mut(&household_id) {
            household.add_member(member_id);
            true
        } else {
            false
        }
    }
    
    /// Ajoute un enfant contribuant à un foyer
    pub fn add_child_contributor(
        &mut self,
        household_id: EntityId,
        child_id: EntityId,
        age: u32,
    ) -> Option<EntityId> {
        if !self.households.contains_key(&household_id) {
            return None;
        }
        
        let child = ChildContributor::new(child_id, age);
        
        if let Some(children) = self.children.get_mut(&household_id) {
            children.push(child);
            // Ajouter aussi comme membre du foyer
            if let Some(household) = self.households.get_mut(&household_id) {
                household.add_member(child_id);
            }
            Some(child_id)
        } else {
            None
        }
    }
    
    /// Calcule les corvées pour un foyer avec enfants
    /// 
    /// §15.3 : "Corvées : 0.5-1.5h par enfant (8-14 ans), plafond 2h"
    /// 
    /// Retourne le total des heures de corvées des enfants (en demi-heures)
    pub fn calculate_children_corvee(&self, household_id: EntityId) -> u32 {
        let children = self.children.get(&household_id);
        if let Some(children_list) = children {
            children_list.iter()
                .filter(|c| matches!(c.age_group, ChildAgeGroup::ChildContributor | ChildAgeGroup::YoungAdult))
                .map(|c| c.corvee_hours)
                .sum()
        } else {
            0
        }
    }
    
    /// Vérifie si un foyer a assez de nourriture
    pub fn has_enough_food(&self, household_id: EntityId) -> bool {
        let household = self.households.get(&household_id);
        if let Some(hh) = household {
            let members_count = hh.member_count() as u32;
            let total_food = hh.total_food();
            total_food >= members_count
        } else {
            false
        }
    }
    
    /// Retourne le foyer pour un id donné
    pub fn get_household(&self, id: EntityId) -> Option<&Household> {
        self.households.get(&id)
    }
    
    /// Retourne le foyer pour un id donné (mutable)
    pub fn get_household_mut(&mut self, id: EntityId) -> Option<&mut Household> {
        self.households.get_mut(&id)
    }
    
    /// Retourne les enfants d'un foyer
    pub fn get_children(&self, household_id: EntityId) -> Option<&Vec<ChildContributor>> {
        self.children.get(&household_id)
    }
}

impl Default for HouseholdManager {
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
    use crate::entities::EntityId;
    
    #[test]
    fn test_household_creation() {
        let mut manager = HouseholdManager::new();
        let owner_id = EntityId::new(EntityType::NamedIndividual, 1);
        
        let household_id = manager.create_household(owner_id, HousingQuality::Base);
        
        assert!(manager.households.contains_key(&household_id));
        let household = manager.get_household(household_id).unwrap();
        assert_eq!(household.owner_id, owner_id);
        assert_eq!(household.housing_quality, HousingQuality::Base);
    }
    
    #[test]
    fn test_add_member() {
        let mut manager = HouseholdManager::new();
        let owner_id = EntityId::new(EntityType::NamedIndividual, 1);
        let member_id = EntityId::new(EntityType::NamedIndividual, 2);
        
        let household_id = manager.create_household(owner_id, HousingQuality::Base);
        
        assert!(manager.add_member_to_household(household_id, member_id));
        
        let household = manager.get_household(household_id).unwrap();
        // member_count() inclut le propriétaire, donc 1 (propriétaire) + 1 (membre) = 2
        assert_eq!(household.member_count(), 2);
    }
    
    #[test]
    fn test_child_corvee() {
        let mut manager = HouseholdManager::new();
        let owner_id = EntityId::new(EntityType::NamedIndividual, 1);
        let child_id = EntityId::new(EntityType::NamedIndividual, 2);
        
        let household_id = manager.create_household(owner_id, HousingQuality::Base);
        
        // Ajouter un enfant de 10 ans (groupe ChildContributor)
        manager.add_child_contributor(household_id, child_id, 10);
        
        let corvee = manager.calculate_children_corvee(household_id);
        // Un enfant de 10 ans contribue entre 1 et 3 demi-heures
        // Par défaut, on prend la moyenne : 2 demi-heures
        assert!(corvee >= 1 && corvee <= 3);
    }
    
    #[test]
    fn test_housing_quality_corvee_reduction() {
        let mut manager = HouseholdManager::new();
        let owner_id = EntityId::new(EntityType::NamedIndividual, 1);
        
        // Créer un foyer avec logement de qualité 2
        let household_id = manager.create_household(owner_id, HousingQuality::Qualite2);
        
        let household = manager.get_household(household_id).unwrap();
        
        // La réduction devrait être de 2 demi-heures (1 heure)
        assert_eq!(household.housing_quality.corvee_reduction(), 2);
    }
    
    #[test]
    fn test_food_consumption() {
        let owner_id = EntityId::new(EntityType::NamedIndividual, 1);
        let member_id = EntityId::new(EntityType::NamedIndividual, 2);
        
        let mut household = Household::new(
            EntityId::new(EntityType::Household, 1),
            owner_id,
            HousingQuality::Base,
        );
        
        household.add_member(member_id);
        
        // Ajouter de la nourriture
        household.add_food(FoodQuality::Base, 5);
        
        // member_count() = 2 (propriétaire + 1 membre), donc consommation = 2
        assert!(household.consume_food(0));
        assert_eq!(household.total_food(), 3); // 5 - 2 = 3
        
        // Essayer de consommer sans assez de nourriture (il reste 3, besoin 2)
        // Donc ça devrait réussir
        assert!(household.consume_food(1));
        assert_eq!(household.total_food(), 1); // 3 - 2 = 1
        
        // Maintenant il n'y a plus assez de nourriture (1 < 2)
        assert!(!household.consume_food(2));
    }
    
    #[test]
    fn test_food_quality_average() {
        let owner_id = EntityId::new(EntityType::NamedIndividual, 1);
        
        let mut household = Household::new(
            EntityId::new(EntityType::Household, 1),
            owner_id,
            HousingQuality::Base,
        );
        
        // Ajouter de la nourriture de différentes qualités
        household.add_food(FoodQuality::Base, 2);
        household.add_food(FoodQuality::Moyenne, 3);
        household.add_food(FoodQuality::Superieure, 5);
        
        let avg = household.avg_food_quality();
        // Qualité totale : (0*2 + 1*3 + 2*5) / (2+3+5) = 13 / 10 = 1.3
        assert!((avg - 1.3).abs() < 0.01);
    }
    
    #[test]
    fn test_child_age_groups() {
        assert_eq!(ChildAgeGroup::from_age(5), ChildAgeGroup::YoungChild);
        assert_eq!(ChildAgeGroup::from_age(10), ChildAgeGroup::ChildContributor);
        assert_eq!(ChildAgeGroup::from_age(16), ChildAgeGroup::YoungAdult);
        assert_eq!(ChildAgeGroup::from_age(20), ChildAgeGroup::Adult);
    }
}
