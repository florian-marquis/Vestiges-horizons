//! Vestiges Horizons - Tranche 7 : Salaires réels et griefs
//! 
//! Implémente :
//! - Panier (nourriture + logement + eau + fiscalité)
//! - Salaire réel = salaire horaire × heures travaillées / coût du panier
//! - Griefs : dG = 0.09·u + 0.02·p − 0.03·q (points/jour, bornés [0, 100])
//! - Intégration événementielle des griefs
//! 
//! §19 : Spécification de la tranche 7

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crate::entities::EntityId;
use crate::{Event, EventData, Timestamp, EVENT_TYPE_INPUT_CHANGED};
pub use crate::griefs::{GriefEntree, GriefsError, GriefsState};

/// Composantes du panier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panier {
    /// Coût de la nourriture (en centimes)
    pub nourriture: u64,
    /// Coût du logement (en centimes)
    pub logement: u64,
    /// Coût de l'eau refacturée (en centimes)
    pub eau: u64,
    /// Coût de la fiscalité (en centimes)
    pub fiscalite: u64,
}

impl Panier {
    pub fn new() -> Self {
        Panier {
            nourriture: 0,
            logement: 0,
            eau: 0,
            fiscalite: 0,
        }
    }
    
    /// Calcul le coût total du panier
    pub fn cout_total(&self) -> u64 {
        self.nourriture + self.logement + self.eau + self.fiscalite
    }
    
    /// Met à jour une composante du panier
    pub fn set_composante(&mut self, composante: PanierComposante, valeur: u64) {
        match composante {
            PanierComposante::Nourriture => self.nourriture = valeur,
            PanierComposante::Logement => self.logement = valeur,
            PanierComposante::Eau => self.eau = valeur,
            PanierComposante::Fiscalite => self.fiscalite = valeur,
        }
    }
}

/// Type de composante du panier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanierComposante {
    Nourriture,
    Logement,
    Eau,
    Fiscalite,
}

/// Salaire réel et griefs pour une entité (individu ou cohorte)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaireReelEtGriefs {
    /// Salaire horaire (en centimes)
    pub salaire_horaire: u64,
    /// Heures travaillées par jour
    pub heures_travaillees: u32,
    /// Coût du panier
    pub cout_panier: u64,
    /// Composantes du panier propres à cette cohorte
    pub panier: Panier,
    /// Salaire réel (salaire × heures / coût panier)
    /// Exprimé en pourcentage : (salaire_horaire * heures / cout_panier) * 100
    pub salaire_reel: u64,
    /// État canonique de griefs, partagé par tous les calculs T7
    pub griefs: GriefsState,
}

impl SalaireReelEtGriefs {
    pub fn new(entite: EntityId) -> Self {
        SalaireReelEtGriefs {
            salaire_horaire: 0,
            heures_travaillees: 0,
            cout_panier: 0,
            panier: Panier::new(),
            salaire_reel: 0,
            griefs: GriefsState::new(entite),
        }
    }
    
    /// Calcule le salaire réel
    /// Formule : (salaire_horaire * heures_travaillees / cout_panier) * 100
    /// Retourne 0 si cout_panier = 0
    pub fn calculer_salaire_reel(&mut self) {
        if self.cout_panier == 0 {
            self.salaire_reel = 0;
            return;
        }
        
        // Calcul en entiers pour éviter les flottants
        // salaire_reel = (salaire_horaire * heures * 100) / cout_panier
        let numerator = self.salaire_horaire * self.heures_travaillees as u64 * 100;
        self.salaire_reel = numerator / self.cout_panier;
    }

    pub fn revenu_journalier(&self) -> u64 {
        self.salaire_horaire
            .saturating_mul(u64::from(self.heures_travaillees))
    }

    pub fn impot_journalier(&self) -> u64 {
        let numerator =
            u128::from(self.revenu_journalier()) * u128::from(crate::tax::TAX_RATE);
        (numerator / 1_000) as u64
    }

    /// Budget nourriture journalier : reste à vivre après logement, eau et impôt,
    /// lus depuis le panier propre à la cohorte (aucune constante locale).
    pub fn budget_nourriture(&self, _jour: i64) -> u64 {
        let charges = self
            .panier
            .logement
            .saturating_add(self.panier.eau)
            .saturating_add(self.impot_journalier());
        self.revenu_journalier().saturating_sub(charges)
    }
    
    /// Calcule p selon le taux d'effort du panier sur le revenu journalier.
    ///
    /// `None` utilise `revenu_journalier`; le calcul entier et ses bornes sont
    /// définis dans `inputs::calculate_p`.
    ///
    /// @param revenu_disponible Revenu journalier en centimes (par défaut : salaire_horaire * heures_travaillees)
    /// @returns p en millièmes [0, 1000]
    pub fn calculate_p(&self, revenu_disponible: Option<u64>) -> u16 {
        let rev = revenu_disponible.unwrap_or_else(|| self.revenu_journalier());
        crate::inputs::calculate_p(self.cout_panier, rev)
    }

    /// Calcule la pente en micro-points de griefs par jour.
    pub fn calculer_dg(&self) -> i64 {
        self.griefs.slope_micro_points_per_day()
    }
    
    /// Avance les griefs jusqu'à l'horodatage logique.
    pub fn mettre_a_jour_griefs(&mut self, timestamp: Timestamp) -> Result<(), GriefsError> {
        self.griefs.advance_to(timestamp)?;
        Ok(())
    }
    
    /// Avance l'état avec les anciennes entrées, puis applique la nouvelle valeur.
    pub fn mettre_a_jour_entree(
        &mut self,
        entree: GriefEntree,
        valeur: i64,
        timestamp: Timestamp,
    ) -> Result<(), GriefsError> {
        self.griefs.set_input_at(entree, valeur, timestamp)?;
        Ok(())
    }

    /// Applique atomiquement les trois entrées à leur horodatage.
    pub fn mettre_a_jour_entrees(
        &mut self,
        u: u16,
        p: u16,
        q: u16,
        timestamp: Timestamp,
    ) -> Result<(), GriefsError> {
        self.griefs.set_inputs_at(u, p, q, timestamp)?;
        Ok(())
    }
}

/// Gestionnaire des salaires réels et griefs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaireReelEtGriefsManager {
    /// Panier de référence
    pub panier: Panier,
    /// Salaires réels et griefs par entité
    pub entites: BTreeMap<EntityId, SalaireReelEtGriefs>,
    /// Compteur d'IDs
    next_id: u64,
}

impl SalaireReelEtGriefsManager {
    pub fn new() -> Self {
        SalaireReelEtGriefsManager {
            panier: Panier::new(),
            entites: BTreeMap::new(),
            next_id: 0,
        }
    }
    
    /// Ajoute une entité
    pub fn ajouter_entite(&mut self, entite: EntityId) -> EntityId {
        let mut state = SalaireReelEtGriefs::new(entite);
        state.panier = self.panier.clone();
        state.cout_panier = state.panier.cout_total();
        self.entites.insert(entite, state);
        entite
    }
    
    /// Met à jour le salaire horaire d'une entité
    pub fn mettre_a_jour_salaire_horaire(&mut self, entite: EntityId, salaire: u64) {
        if let Some(e) = self.entites.get_mut(&entite) {
            e.salaire_horaire = salaire;
            e.calculer_salaire_reel();
        }
    }
    
    /// Met à jour les heures travaillées d'une entité
    pub fn mettre_a_jour_heures(&mut self, entite: EntityId, heures: u32) {
        if let Some(e) = self.entites.get_mut(&entite) {
            e.heures_travaillees = heures;
            e.calculer_salaire_reel();
        }
    }
    
    /// Met à jour le panier
    pub fn mettre_a_jour_panier(&mut self, composante: PanierComposante, valeur: u64) {
        self.panier.set_composante(composante, valeur);
        for e in self.entites.values_mut() {
            e.panier.set_composante(composante, valeur);
            e.cout_panier = e.panier.cout_total();
            e.calculer_salaire_reel();
        }
    }

    pub fn mettre_a_jour_panier_entite(
        &mut self,
        entite: EntityId,
        composante: PanierComposante,
        valeur: u64,
    ) -> Result<(), GriefsError> {
        let state = self
            .entites
            .get_mut(&entite)
            .ok_or(GriefsError::UnknownEntity(entite))?;
        state.panier.set_composante(composante, valeur);
        state.cout_panier = state.panier.cout_total();
        state.calculer_salaire_reel();
        Ok(())
    }
    
    /// Met à jour une entrée de griefs pour une entité
    pub fn mettre_a_jour_entree_griefs(
        &mut self,
        entite: EntityId,
        entree: GriefEntree,
        valeur: i64,
        timestamp: Timestamp,
    ) -> Result<(), GriefsError> {
        let e = self
            .entites
            .get_mut(&entite)
            .ok_or(GriefsError::UnknownEntity(entite))?;
        e.mettre_a_jour_entree(entree, valeur, timestamp)
    }

    pub fn mettre_a_jour_entrees_griefs(
        &mut self,
        entite: EntityId,
        u: u16,
        p: u16,
        q: u16,
        timestamp: Timestamp,
    ) -> Result<(), GriefsError> {
        let e = self
            .entites
            .get_mut(&entite)
            .ok_or(GriefsError::UnknownEntity(entite))?;
        e.mettre_a_jour_entrees(u, p, q, timestamp)
    }

    pub fn appliquer_evenement(&mut self, event: &Event) -> Result<(), GriefsError> {
        if event.event_type != EVENT_TYPE_INPUT_CHANGED {
            return Err(GriefsError::UnsupportedEvent);
        }
        match &event.data {
            EventData::InputChanged {
                entity_id,
                u,
                p,
                q,
            } => self.mettre_a_jour_entrees_griefs(
                *entity_id,
                *u,
                *p,
                *q,
                event.timestamp,
            ),
            _ => Err(GriefsError::UnsupportedEvent),
        }
    }
    
    /// Obtient le salaire réel d'une entité
    pub fn get_salaire_reel(&self, entite: EntityId) -> Option<u64> {
        self.entites.get(&entite).map(|e| e.salaire_reel)
    }
    
    /// Obtient les griefs d'une entité
    /// Retourne leur projection entière en points; utiliser `get_griefs_micro_points`
    /// pour conserver la précision sous-point.
    pub fn get_griefs(&self, entite: EntityId) -> Option<i64> {
        self.entites
            .get(&entite)
            .map(|e| e.griefs.griefs_points())
    }

    pub fn get_griefs_micro_points(&self, entite: EntityId) -> Option<i64> {
        self.entites
            .get(&entite)
            .map(|e| e.griefs.griefs_micro_points())
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
    fn test_panier_cout_total() {
        let mut panier = Panier::new();
        panier.nourriture = 1000;
        panier.logement = 2000;
        panier.eau = 500;
        panier.fiscalite = 300;
        
        assert_eq!(panier.cout_total(), 3800);
    }
    
    #[test]
    fn test_salaire_reel_calcul() {
        let mut sr = SalaireReelEtGriefs::new(EntityId::new(crate::entities::EntityType::Cohort, 1));
        sr.salaire_horaire = 1000; // 10€/heure en centimes
        sr.heures_travaillees = 8;
        sr.cout_panier = 2000; // 20€/jour en centimes
        
        sr.calculer_salaire_reel();
        // salaire_reel = (1000 * 8 * 100) / 2000 = 400%
        assert_eq!(sr.salaire_reel, 400);
    }

    #[test]
    fn food_budget_and_price_pressure_share_per_cohort_income() {
        let mut manager = SalaireReelEtGriefsManager::new();
        let cohorts = [
            (35, 280, 28, 122),
            (40, 320, 32, 158),
            (50, 400, 40, 230),
            (60, 480, 48, 302),
            (70, 560, 56, 374),
        ];

        let expected_p = [802, 545, 187, 0, 0];
        for (index, (hourly_wage, income, tax, food_budget)) in cohorts.into_iter().enumerate() {
            let entity = EntityId::new(crate::entities::EntityType::Cohort, index as u64 + 1);
            manager.ajouter_entite(entity);
            manager.mettre_a_jour_salaire_horaire(entity, hourly_wage);
            manager.mettre_a_jour_heures(entity, 8);
            manager
                .mettre_a_jour_panier_entite(entity, PanierComposante::Nourriture, 100)
                .unwrap();
            manager
                .mettre_a_jour_panier_entite(entity, PanierComposante::Logement, 110)
                .unwrap();
            manager
                .mettre_a_jour_panier_entite(entity, PanierComposante::Eau, 20)
                .unwrap();
            manager
                .mettre_a_jour_panier_entite(entity, PanierComposante::Fiscalite, tax)
                .unwrap();

            let state = manager.entites.get(&entity).unwrap();
            assert_eq!(state.revenu_journalier(), income);
            assert_eq!(state.impot_journalier(), tax);
            assert_eq!(state.budget_nourriture(0), food_budget);
            assert_eq!(state.cout_panier, 100 + 110 + 20 + tax);
            assert_eq!(state.calculate_p(None), expected_p[index]);
        }
    }

    #[test]
    fn entity_specific_basket_updates_do_not_change_other_cohorts() {
        let mut manager = SalaireReelEtGriefsManager::new();
        let first = EntityId::new(crate::entities::EntityType::Cohort, 1);
        let second = EntityId::new(crate::entities::EntityType::Cohort, 2);
        manager.ajouter_entite(first);
        manager.ajouter_entite(second);

        manager
            .mettre_a_jour_panier_entite(first, PanierComposante::Nourriture, 150)
            .unwrap();

        assert_eq!(manager.entites.get(&first).unwrap().cout_panier, 150);
        assert_eq!(manager.entites.get(&second).unwrap().cout_panier, 0);
    }

    /// Le budget nourriture suit les charges logées dans le panier de la cohorte
    /// (Logement et Eau) : toute variation du panier se répercute sur le reste à vivre.
    #[test]
    fn food_budget_follows_basket_rent_and_water_updates() {
        let mut manager = SalaireReelEtGriefsManager::new();
        let entity = EntityId::new(crate::entities::EntityType::Cohort, 1);
        manager.ajouter_entite(entity);
        manager.mettre_a_jour_salaire_horaire(entity, 60);
        manager.mettre_a_jour_heures(entity, 8);
        // Revenu 480, impôt 48 : budget attendu 480 - 110 - 20 - 48 = 302
        manager
            .mettre_a_jour_panier_entite(entity, PanierComposante::Logement, 110)
            .unwrap();
        manager
            .mettre_a_jour_panier_entite(entity, PanierComposante::Eau, 20)
            .unwrap();
        manager
            .mettre_a_jour_panier_entite(entity, PanierComposante::Fiscalite, 48)
            .unwrap();
        assert_eq!(manager.entites.get(&entity).unwrap().budget_nourriture(0), 302);

        // Logement +10 : budget 480 - 120 - 20 - 48 = 292
        manager
            .mettre_a_jour_panier_entite(entity, PanierComposante::Logement, 120)
            .unwrap();
        assert_eq!(manager.entites.get(&entity).unwrap().budget_nourriture(0), 292);

        // Eau +5 : budget 480 - 120 - 25 - 48 = 287
        manager
            .mettre_a_jour_panier_entite(entity, PanierComposante::Eau, 25)
            .unwrap();
        assert_eq!(manager.entites.get(&entity).unwrap().budget_nourriture(0), 287);

        // Charges supérieures au revenu : budget borné à zéro, jamais négatif
        manager
            .mettre_a_jour_panier_entite(entity, PanierComposante::Eau, 500)
            .unwrap();
        assert_eq!(manager.entites.get(&entity).unwrap().budget_nourriture(0), 0);
    }

    /// §19.2 T7.1 : Concordance - I8 vert à tolérance zéro
    #[test]
    fn test_t7_1_concordance_i8() {
        let mut manager = SalaireReelEtGriefsManager::new();
        
        let entite = EntityId::new(crate::entities::EntityType::Cohort, 1);
        manager.ajouter_entite(entite);
        
        // Mettre à jour le panier
        manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
        manager.mettre_a_jour_panier(PanierComposante::Logement, 2000);
        
        // Mettre à jour salaire et heures
        manager.mettre_a_jour_salaire_horaire(entite, 1000);
        manager.mettre_a_jour_heures(entite, 8);
        
        // Mettre à jour entrées de griefs (valeurs normales)
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::U, 100, Timestamp::new(0, 0)).unwrap();
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::P, 200, Timestamp::new(0, 0)).unwrap();
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::Q, 300, Timestamp::new(0, 0)).unwrap();
        
        // Vérifier que les griefs sont dans [0, 100]
        let griefs = manager.get_griefs(entite).unwrap();
        assert!(griefs >= 0 && griefs <= 100);
    }
    
    /// §19.2 T7.2 : Choc 7 - prix nourriture +50%
    /// → salaire réel en baisse, G monte
    #[test]
    fn test_t7_2_choc_7() {
        let mut manager = SalaireReelEtGriefsManager::new();
        
        let entite = EntityId::new(crate::entities::EntityType::Cohort, 1);
        manager.ajouter_entite(entite);
        
        // Panier initial
        manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
        manager.mettre_a_jour_panier(PanierComposante::Logement, 2000);
        
        // Salaire et heures
        manager.mettre_a_jour_salaire_horaire(entite, 1000);
        manager.mettre_a_jour_heures(entite, 8);
        
        let salaire_reel_initial = manager.get_salaire_reel(entite).unwrap();
        
        // Appliquer choc 7 : prix nourriture +50%
        manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1500);
        
        let salaire_reel_apres = manager.get_salaire_reel(entite).unwrap();
        
        // Salaire réel doit baisser
        assert!(salaire_reel_apres < salaire_reel_initial);
        
        // Mettre à jour p (pression des prix monte)
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::P, 500, Timestamp::new(0, 0)).unwrap();
        manager.entites.get_mut(&entite).unwrap()
            .mettre_a_jour_griefs(Timestamp::new(1, 0)).unwrap();
        
        // Griefs doivent monter (p a augmenté)
        let griefs = manager.get_griefs_micro_points(entite).unwrap();
        assert!(griefs > 0);
    }
    
    #[test]
    fn test_griefs_slope_conversion() {
        let entite = EntityId::new(crate::entities::EntityType::Cohort, 1);
        let mut sr = SalaireReelEtGriefs::new(entite);
        sr.mettre_a_jour_entrees(1_000, 0, 0, Timestamp::new(0, 0)).unwrap();
        assert_eq!(sr.calculer_dg(), 210_000);
    }
    
    /// §19.2 T7.3 : Choc 11 complet - impôt ×2 pendant un an
    /// → griefs traçables terme à terme (u, p, q)
    #[test]
    fn test_t7_3_choc_11_complet() {
        let mut manager = SalaireReelEtGriefsManager::new();
        
        let entite = EntityId::new(crate::entities::EntityType::Cohort, 1);
        manager.ajouter_entite(entite);
        
        // Initialisation : panier normal, salaire normal
        manager.mettre_a_jour_panier(PanierComposante::Nourriture, 1000);
        manager.mettre_a_jour_panier(PanierComposante::Logement, 2000);
        manager.mettre_a_jour_panier(PanierComposante::Eau, 500);
        manager.mettre_a_jour_panier(PanierComposante::Fiscalite, 300);
        
        manager.mettre_a_jour_salaire_horaire(entite, 1000);
        manager.mettre_a_jour_heures(entite, 8);
        
        manager.mettre_a_jour_entrees_griefs(entite, 100, 100, 0, Timestamp::new(0, 0)).unwrap();
        manager.entites.get_mut(&entite).unwrap()
            .mettre_a_jour_griefs(Timestamp::new(1_000, 0)).unwrap();
        
        let griefs_initial = manager.get_griefs(entite).unwrap();
        let salaire_reel_initial = manager.get_salaire_reel(entite).unwrap();
        
        // Appliquer choc 11 : impôt ×2 (composante fiscalité du panier)
        manager.mettre_a_jour_panier(PanierComposante::Fiscalite, 600);
        
        let salaire_reel_apres = manager.get_salaire_reel(entite).unwrap();
        
        // Salaire réel doit baisser (coût du panier a augmenté)
        assert!(salaire_reel_apres < salaire_reel_initial);
        
        // Mettre à jour p (pression des prix monte à cause du choc fiscal)
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::P, 300, Timestamp::new(1_000, 0)).unwrap();
        manager.entites.get_mut(&entite).unwrap()
            .mettre_a_jour_griefs(Timestamp::new(2_000, 0)).unwrap();
        
        let griefs_apres = manager.get_griefs(entite).unwrap();
        
        // Griefs doivent monter (p a augmenté)
        assert!(griefs_apres > griefs_initial);
        
        // Vérifier que les griefs sont traçables : on peut retrouver u, p, q
        if let Some(e) = manager.entites.get(&entite) {
            assert_eq!(e.griefs.u(), 100);
            assert_eq!(e.griefs.p(), 300);
            assert_eq!(e.griefs.q(), 0);
        }
    }
    
    /// §19.2 T7.4 : l'état est avancé jusqu'à chaque changement d'entrée.
    #[test]
    fn test_t7_4_integration_evenementielle() {
        let mut manager = SalaireReelEtGriefsManager::new();
        
        let entite = EntityId::new(crate::entities::EntityType::Cohort, 1);
        manager.ajouter_entite(entite);
        
        manager.mettre_a_jour_entrees_griefs(entite, 1_000, 250, 0, Timestamp::new(0, 0)).unwrap();
        manager.entites.get_mut(&entite).unwrap()
            .mettre_a_jour_griefs(Timestamp::new(1, 0)).unwrap();
        assert_eq!(manager.get_griefs_micro_points(entite), Some(215_000));

        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::U, 0, Timestamp::new(1, 2)).unwrap();
        assert_eq!(manager.get_griefs_micro_points(entite), Some(322_500));

        manager.entites.get_mut(&entite).unwrap()
            .mettre_a_jour_griefs(Timestamp::new(2, 0)).unwrap();
        assert_eq!(manager.get_griefs_micro_points(entite), Some(340_000));
    }
    
}
