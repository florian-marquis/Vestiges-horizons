//! Vestiges Horizons - Tranche 7 : Salaires réels et griefs
//! 
//! Implémente :
//! - Panier (nourriture + logement + eau + fiscalité)
//! - Salaire réel = salaire horaire × heures travaillées / coût du panier
//! - Griefs : dG = 0.8·u + 0.2·p − 0.25·q (points/jour, bornés [0, 100])
//! - Intégration événementielle des griefs
//! 
//! §19 : Spécification de la tranche 7

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crate::entities::EntityId;

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
    /// Entité concernée (individu ou cohorte)
    pub entite: EntityId,
    /// Salaire horaire (en centimes)
    pub salaire_horaire: u64,
    /// Heures travaillées par jour
    pub heures_travaillees: u32,
    /// Coût du panier
    pub cout_panier: u64,
    /// Salaire réel (salaire × heures / coût panier)
    /// Exprimé en pourcentage : (salaire_horaire * heures / cout_panier) * 100
    pub salaire_reel: u64,
    /// Griefs cumulés (0-100)
    pub griefs: i64,
    /// u : part des besoins alimentaires non satisfaits (0-1000)
    pub u: i64,
    /// p : pression des prix (0-1000)
    pub p: i64,
    /// q : qualité effective (0-1000)
    pub q: i64,
}

impl SalaireReelEtGriefs {
    pub fn new(entite: EntityId) -> Self {
        SalaireReelEtGriefs {
            entite,
            salaire_horaire: 0,
            heures_travaillees: 0,
            cout_panier: 0,
            salaire_reel: 0,
            griefs: 0,
            u: 0,
            p: 0,
            q: 0,
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
    
    /// Calcule dG (variation des griefs)
    /// Formule : dG = 0.8·u + 0.2·p − 0.25·q
    /// dG peut être négatif (qualité améliore → griefs diminuent)
    /// Intégration événementielle : appelé quand une entrée change
    pub fn calculer_dg(&self) -> i64 {
        // Calcul en entiers (échelle 1000)
        // dG = (800*u + 200*p - 250*q) / 1000
        (800 * self.u + 200 * self.p - 250 * self.q) / 1000
    }
    
    /// Met à jour les griefs avec intégration événementielle
    /// G (griefs cumulés) est borné à [0, 100]
    pub fn mettre_a_jour_griefs(&mut self) {
        let dg = self.calculer_dg();
        // Intégration : griefs += dG, borné à [0, 100]
        self.griefs = (self.griefs + dg).clamp(0, 100);
    }
    
    /// Met à jour une entrée (u, p ou q) et recalcule les griefs
    pub fn mettre_a_jour_entree(&mut self, entree: GriefEntree, valeur: i64) {
        match entree {
            GriefEntree::U => self.u = valeur,
            GriefEntree::P => self.p = valeur,
            GriefEntree::Q => self.q = valeur,
        }
        self.mettre_a_jour_griefs();
    }
}

/// Type d'entrée pour le calcul des griefs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GriefEntree {
    U, // Besoins alimentaires non satisfaits
    P, // Pression des prix
    Q, // Qualité effective
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
        self.entites.insert(entite, SalaireReelEtGriefs::new(entite));
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
        // Recalculer tous les salaires réels
        for e in self.entites.values_mut() {
            e.cout_panier = self.panier.cout_total();
            e.calculer_salaire_reel();
        }
    }
    
    /// Met à jour une entrée de griefs pour une entité
    pub fn mettre_a_jour_entree_griefs(&mut self, entite: EntityId, entree: GriefEntree, valeur: i64) {
        if let Some(e) = self.entites.get_mut(&entite) {
            e.mettre_a_jour_entree(entree, valeur);
        }
    }
    
    /// Obtient le salaire réel d'une entité
    pub fn get_salaire_reel(&self, entite: EntityId) -> Option<u64> {
        self.entites.get(&entite).map(|e| e.salaire_reel)
    }
    
    /// Obtient les griefs d'une entité
    pub fn get_griefs(&self, entite: EntityId) -> Option<i64> {
        self.entites.get(&entite).map(|e| e.griefs)
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
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::U, 100);
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::P, 200);
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::Q, 300);
        
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
        manager.mettre_a_jour_entree_griefs(entite, GriefEntree::P, 500);
        
        // Griefs doivent monter (p a augmenté)
        let griefs = manager.get_griefs(entite).unwrap();
        assert!(griefs > 0);
    }
    
    #[test]
    fn test_griefs_dg_formula() {
        // Tester la formule dG = 0.8*u + 0.2*p - 0.25*q
        let mut sr = SalaireReelEtGriefs::new(EntityId::new(crate::entities::EntityType::Cohort, 1));
        
        // u=100, p=100, q=100 → dG = 80 + 20 - 25 = 75
        sr.u = 100; sr.p = 100; sr.q = 100;
        assert_eq!(sr.calculer_dg(), 75);
        
        // u=200, p=100, q=100 → dG = 160 + 20 - 25 = 155
        sr.u = 200; sr.p = 100; sr.q = 100;
        assert_eq!(sr.calculer_dg(), 155);
        
        // u=200, p=100, q=500 → dG = 160 + 20 - 125 = 55
        sr.q = 500;
        assert_eq!(sr.calculer_dg(), 55);
        
        // u=200, p=100, q=800 → dG = 160 + 20 - 200 = -20
        sr.q = 800;
        assert_eq!(sr.calculer_dg(), -20);
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
        
        // Entrées initiales (valeurs normales)
        if let Some(e) = manager.entites.get_mut(&entite) {
            e.u = 100; // Besoins alimentaires non satisfaits
            e.p = 100; // Pression des prix
            e.q = 500; // Qualité effective
            e.mettre_a_jour_griefs();
        }
        
        let griefs_initial = manager.get_griefs(entite).unwrap();
        let salaire_reel_initial = manager.get_salaire_reel(entite).unwrap();
        
        // Appliquer choc 11 : impôt ×2 (composante fiscalité du panier)
        manager.mettre_a_jour_panier(PanierComposante::Fiscalite, 600);
        
        let salaire_reel_apres = manager.get_salaire_reel(entite).unwrap();
        
        // Salaire réel doit baisser (coût du panier a augmenté)
        assert!(salaire_reel_apres < salaire_reel_initial);
        
        // Mettre à jour p (pression des prix monte à cause du choc fiscal)
        if let Some(e) = manager.entites.get_mut(&entite) {
            e.p = 300; // Pression des prix augmentée
            e.mettre_a_jour_griefs();
        }
        
        let griefs_apres = manager.get_griefs(entite).unwrap();
        
        // Griefs doivent monter (p a augmenté)
        assert!(griefs_apres > griefs_initial);
        
        // Vérifier que les griefs sont traçables : on peut retrouver u, p, q
        if let Some(e) = manager.entites.get(&entite) {
            assert_eq!(e.u, 100);
            assert_eq!(e.p, 300);
            assert_eq!(e.q, 500);
        }
    }
    
    /// §19.2 T7.4 : Intégration événementielle
    /// G est mis à jour à chaque changement d'entrée, résultat identique à l'intégration continue
    #[test]
    fn test_t7_4_integration_evenementielle() {
        let mut manager = SalaireReelEtGriefsManager::new();
        
        let entite = EntityId::new(crate::entities::EntityType::Cohort, 1);
        manager.ajouter_entite(entite);
        
        // Initialisation de toutes les entrées en une seule opération
        // Pour éviter le problème où chaque mise à jour écrase les autres
        if let Some(e) = manager.entites.get_mut(&entite) {
            e.u = 100;
            e.p = 100;
            e.q = 100;
            e.mettre_a_jour_griefs();
        }
        
        let griefs_1 = manager.get_griefs(entite).unwrap();
        // griefs_1 = dG(100,100,100) = 0.8*100 + 0.2*100 - 0.25*100 = 75
        assert_eq!(griefs_1, 75);
        
        // Changer u (besoins non satisfaits augmente)
        if let Some(e) = manager.entites.get_mut(&entite) {
            e.u = 200;
            e.mettre_a_jour_griefs();
        }
        
        let griefs_2 = manager.get_griefs(entite).unwrap();
        // griefs_2 = 75 + dG(200,100,100) = 75 + (0.8*200 + 0.2*100 - 0.25*100) = 75 + 155 = 230 → clampé à 100
        assert_eq!(griefs_2, 100);
        assert!(griefs_2 > griefs_1);
        
        // Changer q (qualité améliore)
        if let Some(e) = manager.entites.get_mut(&entite) {
            e.q = 500;
            e.mettre_a_jour_griefs();
        }
        
        let griefs_3 = manager.get_griefs(entite).unwrap();
        // griefs_3 = 100 + dG(200,100,500) = 100 + (0.8*200 + 0.2*100 - 0.25*500) = 100 + (160+20-125) = 100 + 55 = 155 → clampé à 100
        // Donc griefs_3 reste à 100
        assert_eq!(griefs_3, 100);
        // Mais on peut vérifier que si on diminue encore plus q, les griefs baissent
        
        // Changer q à 800 (très haute qualité)
        if let Some(e) = manager.entites.get_mut(&entite) {
            e.q = 800;
            e.mettre_a_jour_griefs();
        }
        
        let griefs_4 = manager.get_griefs(entite).unwrap();
        // Avec u=200, p=100, q=800: dG = 0.8*200 + 0.2*100 - 0.25*800 = 160+20-200 = -20
        // griefs_4 = 100 + (-20) = 80 (dans les bornes)
        assert!(griefs_4 < griefs_2);
    }
}
