//! Vestiges Horizons - Types communs partagés
//! 
//! Ce module contient les types utilisés par plusieurs modules pour éviter les dépendances circulaires

use serde::{Deserialize, Serialize};

// ============================================================================
// POLITIQUE D'ENTRETIEN
// ============================================================================

/// Politique d'entretien pour les actifs
/// 
/// §16.3 : "Entretien : déclaration mensuelle, choix de politique pour l'actif 
/// (intensif, normal, léger, abandonné) réduisant la usure (70%, 100%, 140%, 200%)"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MaintenancePolicy {
    /// Abandonné : usure à 200% (pas d'entretien)
    Abandoned = 0,
    /// Léger : usure à 140%
    Light = 1,
    /// Normal : usure à 100%
    Normal = 2,
    /// Intensif : usure à 70%
    Intensive = 3,
}

impl MaintenancePolicy {
    /// Multiplicateur du taux d'usure
    pub fn wear_rate_modifier(&self) -> f32 {
        match self {
            MaintenancePolicy::Abandoned => 2.0,    // +100% d'usure
            MaintenancePolicy::Light => 1.4,        // +40% d'usure
            MaintenancePolicy::Normal => 1.0,       // usure normale
            MaintenancePolicy::Intensive => 0.7,    // -30% d'usure
        }
    }
    
    /// Temps requis par jour pour l'entretien (en heures)
    pub fn time_required(&self) -> u32 {
        match self {
            MaintenancePolicy::Abandoned => 0,
            MaintenancePolicy::Light => 0,
            MaintenancePolicy::Normal => 1,
            MaintenancePolicy::Intensive => 2,
        }
    }
    
    /// Coût par jour (en centimes)
    pub fn daily_cost(&self) -> u64 {
        match self {
            MaintenancePolicy::Abandoned => 0,
            MaintenancePolicy::Light => 10,
            MaintenancePolicy::Normal => 50,
            MaintenancePolicy::Intensive => 150,
        }
    }
}

// ============================================================================
// ÉCHELLE DE PRODUCTION
// ============================================================================

/// Échelle de production d'une firme
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProductionScale {
    /// Petite firme (1-5 employés)
    Small = 0,
    /// Moyenne firme (6-20 employés)
    Medium = 1,
    /// Grande firme (21+ employés)
    Large = 2,
}

impl ProductionScale {
    pub fn max_workers(&self) -> usize {
        match self {
            ProductionScale::Small => 5,
            ProductionScale::Medium => 20,
            ProductionScale::Large => 100,
        }
    }
    
    pub fn base_capacity(&self) -> u32 {
        match self {
            ProductionScale::Small => 100,
            ProductionScale::Medium => 500,
            ProductionScale::Large => 2000,
        }
    }
}

// ============================================================================
// TYPE DE RESSOURCE
// ============================================================================

/// Type de ressource pour les recettes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResourceType {
    /// Matière première
    RawMaterial = 0,
    /// Composant
    Component = 1,
    /// Travail (heures)
    Labor = 2,
    /// Énergie
    Energy = 3,
    /// Eau
    Water = 4,
}

// ============================================================================
// TYPE DE PRODUIT
// ============================================================================

/// Type de produit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProductType {
    /// Bien manufacturé
    Manufactured = 0,
    /// Nourriture
    Food = 1,
    /// Service
    Service = 2,
}
