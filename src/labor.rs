//! Vestiges Horizons - Tranche 6 : Marché du travail, métiers et parrainage
//!
//! Implémentation selon Docs/spec P1 - quartier vivant.md §18 et §34
//! Source de vérité : sections 18 (spécification), 6.3, 6.11, 6.15, 6.20, 6.22
//!
//! Conventions : Docs/Conventions du projet (pour agents).md §1-27

use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, VecDeque};
use crate::entities::EntityId;

// ============================================================================
// Q(A) - TABLE D'ACTIONS
// ============================================================================

/// Type d'action Q(a)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ActionType {
    // Actions d'emploi
    Embaucher = 0,
    Licencier = 1,
    MonterSalaire = 2,
    BaisserSalaire = 3,
    // Actions de production
    Investir = 4,
    Entretenir = 5,
    Fermer = 6,
    // Actions de prix
    MonterPrix = 7,
    BaisserPrix = 8,
    TenirPrix = 9,
}

/// Entrée de la table Q(a) : action + poids
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaEntry {
    pub action: ActionType,
    /// Poids de base (en millièmes pour éviter les flottants)
    pub poids: u32,
}

/// Table Q(a) pour une firme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaTable {
    /// Entrées de la table
    pub entries: Vec<QaEntry>,
    /// Modulateurs par traits (pour l'empathie)
    pub modulateurs: BTreeMap<ActionType, i32>,
}

impl QaTable {
    pub fn new() -> Self {
        QaTable {
            entries: Vec::new(),
            modulateurs: BTreeMap::new(),
        }
    }
    
    /// Ajoute une entrée à la table
    pub fn add_entry(&mut self, action: ActionType, poids: u32) {
        self.entries.push(QaEntry { action, poids });
    }
    
    /// Applique la modulation par les traits du décideur
    /// §6.20 : L'empathie module les poids
    pub fn appliquer_traits(&mut self, traits: &TraitsDecideur) {
        // Moduler les actions selon l'empathie
        // Exemple : l'empathie réduit le poids de "Licencier"
        // et augmente le poids de "Embaucher", "MonterSalaire"
        
        let empathy_factor = traits.empathie as i32;
        
        // Licencier : poids réduit proportionnellement à l'empathie
        self.modulateurs.insert(ActionType::Licencier, -empathy_factor / 2);
        
        // Embaucher : poids augmenté
        self.modulateurs.insert(ActionType::Embaucher, empathy_factor / 2);
        
        // MonterSalaire : poids augmenté
        self.modulateurs.insert(ActionType::MonterSalaire, empathy_factor / 3);
        
        // BaisserSalaire : poids réduit
        self.modulateurs.insert(ActionType::BaisserSalaire, -empathy_factor / 3);
    }
    
    /// Évalue une action : retourne le poids final
    /// §18.2 : Q(a) d'emploi modulée par traits
    pub fn evaluer(&self, action: ActionType) -> i64 {
        // Trouver la base
        let base = self.entries.iter()
            .find(|e| e.action == action)
            .map(|e| e.poids as i64)
            .unwrap_or(0);
        
        // Appliquer le modulateur
        let modulateur = self.modulateurs.get(&action).copied().unwrap_or(0);
        
        // Poids final = base + modulateur
        (base + modulateur as i64).max(0) // Ne pas descendre en dessous de 0
    }
    
    /// Sélectionne la meilleure action
    pub fn selectionner_meilleure(&self) -> Option<ActionType> {
        let mut meilleure_action: Option<ActionType> = None;
        let mut meilleur_poids = i64::MIN;
        
        for entry in &self.entries {
            let poids = self.evaluer(entry.action);
            if poids > meilleur_poids {
                meilleur_poids = poids;
                meilleure_action = Some(entry.action);
            }
        }
        
        meilleure_action
    }
}

/// Contexte de décision pour Q(a)
#[derive(Debug, Clone)]
pub struct DecisionContext {
    /// Firme concernée
    pub firme: EntityId,
    /// Cohorte visée
    pub cohorte: EntityId,
    /// Salaire réel actuel
    pub salaire_reel: u64,
    /// Salaire cible
    pub salaire_cible: u64,
    /// Trésorerie de la firme
    pub tresorerie: u64,
    /// Demande actuelle
    pub demande: u32,
    /// Offre actuelle
    pub offre: u32,
}

// ============================================================================
// TYPES PRINCIPAUX
// ============================================================================

/// État d'un emploi
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EmploiEtats {
    /// Poste ouvert, en attente de candidat
    Ouvert = 0,
    /// Poste pourvu par un employé
    Pourvu = 1,
    /// Poste rompu (licenciement, démission)
    Rompu = 2,
}

/// Créneau de travail (4 créneaux par jour)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Creneau {
    /// 8h-12h
    Matin = 0,
    /// 12h-16h
    ApresMidi = 1,
    /// 16h-20h
    Soir = 2,
    /// 20h-8h (nuit)
    Nuit = 3,
}

impl Creneau {
    pub fn as_str(&self) -> &'static str {
        match self {
            Creneau::Matin => "Matin",
            Creneau::ApresMidi => "Après-midi",
            Creneau::Soir => "Soir",
            Creneau::Nuit => "Nuit",
        }
    }
}

/// Poste d'emploi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emploi {
    pub id: EntityId,
    /// Firme qui propose l'emploi
    pub firme: EntityId,
    /// Cohorte visée (ou individu nommé)
    pub cohorte_visee: EntityId,
    /// Heures de travail par jour
    pub heures: u16,
    /// Salaire horaire en centimes
    pub salaire_horaire: u64,
    /// Créneau de travail
    pub creneau: Creneau,
    /// État du poste
    pub etat: EmploiEtats,
    /// Pratique requise (optionnel)
    pub pratique_requise: Option<u64>,
}

impl Emploi {
    pub fn new(
        id: EntityId,
        firme: EntityId,
        cohorte_visee: EntityId,
        heures: u16,
        salaire_horaire: u64,
        creneau: Creneau,
        pratique_requise: Option<u64>,
    ) -> Self {
        Emploi {
            id,
            firme,
            cohorte_visee,
            heures,
            salaire_horaire,
            creneau,
            etat: EmploiEtats::Ouvert,
            pratique_requise,
        }
    }
    
    /// Passe le poste à l'état Pourvu
    pub fn pourvoir(&mut self) {
        self.etat = EmploiEtats::Pourvu;
    }
    
    /// Passe le poste à l'état Rompu
    pub fn rompre(&mut self) {
        self.etat = EmploiEtats::Rompu;
    }
}

/// Traits du décideur (patron de firme)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitsDecideur {
    /// Empathie : 0-1000 (0 = sans pitié, 1000 = très empathique)
    pub empathie: u16,
}

impl TraitsDecideur {
    pub fn new(empathie: u16) -> Self {
        TraitsDecideur { empathie: empathie.min(1000) }
    }
    
    /// Modifie le seuil de licenciement selon l'empathie
    /// Formule : seuil × (1 + empathie/1000)
    pub fn seuil_licenciement(&self, seuil_base: u32) -> u32 {
        if self.empathie == 0 {
            return seuil_base;
        }
        // Calcul en entiers : seuil_base * (1000 + empathie) / 1000
        let numerator = seuil_base as u64 * (1000 + self.empathie as u64);
        (numerator / 1000) as u32
    }
}

/// Engagement entre un employé et une firme (0-1000)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engagement {
    pub employe: EntityId,
    pub firme: EntityId,
    /// Valeur d'engagement : 0-1000
    pub valeur: u16,
    /// Dernière mise à jour (jour)
    pub dernier_mis_a_jour: i64,
}

impl Engagement {
    pub fn new(employe: EntityId, firme: EntityId, valeur: u16) -> Self {
        Engagement {
            employe,
            firme,
            valeur: valeur.min(1000),
            dernier_mis_a_jour: 0,
        }
    }
    
    /// Met à jour l'engagement avec demi-vie (1-2 ans = 365-730 jours)
    /// nouvelle_valeur = valeur_ancienne + delta, borné à [0, 1000]
    pub fn mise_a_jour(&mut self, delta: i16, current_day: i64, demi_vie: i64) {
        // Décroissance selon demi-vie
        let jours_ecoules = (current_day - self.dernier_mis_a_jour).max(0) as f64;
        let facteur = (-0.693 / demi_vie as f64 * jours_ecoules).exp(); // e^(-ln(2)*t/half_life)
        let valeur_ancienne = (self.valeur as f64 * facteur) as u16;
        
        // Appliquer le delta
        let nouvelle_valeur = (valeur_ancienne as i32 + delta as i32).clamp(0, 1000) as u16;
        
        self.valeur = nouvelle_valeur;
        self.dernier_mis_a_jour = current_day;
    }
}

/// Lien de parrainage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LienParrainage {
    pub parrain: EntityId,
    pub filleul: EntityId,
    /// Coût en demi-heures par jour
    pub cout: u8,
    /// Date de début
    pub date_debut: i64,
}

impl LienParrainage {
    pub fn new(parrain: EntityId, filleul: EntityId) -> Self {
        LienParrainage {
            parrain,
            filleul,
            cout: 4, // 2 demi-heures par jour (à calibrer)
            date_debut: 0,
        }
    }
}

/// Décision ponctuelle de métier (incident, raccourci, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPonctuelle {
    pub id: u64,
    /// Type de décision
    pub type_decision: u8,
    /// Options possibles
    pub options: Vec<u64>,
    /// Clé stable pour le déterminisme
    pub cle_stable: u64,
}

/// Recette multi-postes (avec goulot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecetteMultiPostes {
    pub id: u64,
    /// Étages de la recette (poste → capacité en unités/demi-journée)
    pub etages: BTreeMap<u64, u32>,
    /// Production totale = min des capacités des étages pourvus
    pub production: u32,
}

impl RecetteMultiPostes {
    pub fn new(id: u64) -> Self {
        RecetteMultiPostes {
            id,
            etages: BTreeMap::new(),
            production: 0,
        }
    }
    
    /// Ajoute un étage à la recette
    pub fn ajouter_etage(&mut self, poste_id: u64, capacite: u32) {
        self.etages.insert(poste_id, capacite);
    }
    
    /// Calcule la production actuelle basée sur les étages pourvus
    /// Production = min des capacités des étages pourvus
    pub fn calculer_production(&self, postes_pourvus: &BTreeMap<u64, bool>) -> u32 {
        let mut min_capacite = u32::MAX;
        
        for (poste_id, capacite) in &self.etages {
            if let Some(&pourvu) = postes_pourvus.get(poste_id) {
                if pourvu {
                    min_capacite = min_capacite.min(*capacite);
                } else {
                    // Si un étage n'est pas pourvu, production = 0
                    return 0;
                }
            } else {
                // Poste non présent = non pourvu
                return 0;
            }
        }
        
        min_capacite
    }
    
    /// Retourne le goulot (poste limitant) et son amplitude
    pub fn goulot(&self, postes_pourvus: &BTreeMap<u64, bool>) -> Option<(u64, u32)> {
        let mut min_capacite = u32::MAX;
        let mut goulot_id = 0;
        
        for (poste_id, capacite) in &self.etages {
            if let Some(&pourvu) = postes_pourvus.get(poste_id) {
                if pourvu {
                    if *capacite < min_capacite {
                        min_capacite = *capacite;
                        goulot_id = *poste_id;
                    }
                } else {
                    return None; // Pas de production
                }
            } else {
                return None;
            }
        }
        
        Some((goulot_id, min_capacite))
    }
}

// ============================================================================
// MARCHÉ DU TRAVAIL
// ============================================================================

/// Marché du travail pour le quartier
/// §18.1 : Cinq grandeurs du marché du travail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarcheDuTravail {
    /// Disponibilité par cohorte (heures disponibles)
    pub disponibilite: BTreeMap<EntityId, u32>,
    /// Volonté par cohorte (heures offertes au salaire courant)
    pub volonte: BTreeMap<EntityId, u32>,
    /// Demande : postes publiés par les firmes
    pub demande: VecDeque<Emploi>,
    /// Postes pourvus par cohorte
    pub postes_pourvus: BTreeMap<EntityId, u32>,
    /// Heures travaillées par cohorte
    pub heures_travaillees: BTreeMap<EntityId, u32>,
    /// Traits des décideurs (par firme)
    pub traits_decideurs: BTreeMap<EntityId, TraitsDecideur>,
    /// Engagements (employé → firme → valeur)
    pub engagements: BTreeMap<(EntityId, EntityId), Engagement>,
    /// Liens de parrainage
    pub parrainages: Vec<LienParrainage>,
    /// Recettes multi-postes
    pub recettes: BTreeMap<u64, RecetteMultiPostes>,
    /// Compteur d'IDs
    next_id: u64,
}

impl MarcheDuTravail {
    pub fn new() -> Self {
        MarcheDuTravail {
            disponibilite: BTreeMap::new(),
            volonte: BTreeMap::new(),
            demande: VecDeque::new(),
            postes_pourvus: BTreeMap::new(),
            heures_travaillees: BTreeMap::new(),
            traits_decideurs: BTreeMap::new(),
            engagements: BTreeMap::new(),
            parrainages: Vec::new(),
            recettes: BTreeMap::new(),
            next_id: 0,
        }
    }
    
    /// Génère un nouvel ID
    fn next_id(&mut self) -> EntityId {
        let id = EntityId::new(crate::entities::EntityType::Job, self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Publie un poste d'emploi (action de la firme)
    /// §34.3 : actions Q(a)
    pub fn publier_poste(
        &mut self,
        firme: EntityId,
        cohorte_visee: EntityId,
        heures: u16,
        salaire_horaire: u64,
        creneau: Creneau,
        pratique_requise: Option<u64>,
    ) -> EntityId {
        let id = self.next_id();
        let emploi = Emploi::new(
            id,
            firme,
            cohorte_visee,
            heures,
            salaire_horaire,
            creneau,
            pratique_requise,
        );
        self.demande.push_back(emploi);
        id
    }
    
    /// Embauche un individu pour un poste
    pub fn embaucher(
        &mut self,
        poste_id: EntityId,
        employe: EntityId,
        _day: i64,
    ) -> bool {
        // Trouver le poste
        if let Some(pos) = self.demande.iter().position(|e| e.id == poste_id) {
            let mut emploi = self.demande.remove(pos).unwrap();
            
            // Mettre à jour l'état
            emploi.pourvoir();
            
            // Mettre à jour les stats
            let cohorte = emploi.cohorte_visee;
            *self.postes_pourvus.entry(cohorte).or_insert(0) += 1;
            *self.heures_travaillees.entry(cohorte).or_insert(0) += emploi.heures as u32;
            
            // Créer un engagement initial (valeur neutre)
            let engagement = Engagement::new(employe, emploi.firme, 500);
            self.engagements.insert((employe, emploi.firme), engagement);
            
            // Remettre le poste dans la demande (pourvu)
            self.demande.push_back(emploi);
            
            return true;
        }
        false
    }
    
    /// Licencie un employé
    pub fn licencier(
        &mut self,
        poste_id: EntityId,
        employe: EntityId,
        day: i64,
        _traits: &TraitsDecideur,
    ) -> bool {
        // Trouver le poste
        if let Some(pos) = self.demande.iter().position(|e| e.id == poste_id) {
            let mut emploi = self.demande.remove(pos).unwrap();
            
            // Mettre à jour l'état
            emploi.rompre();
            
            // Mettre à jour les stats
            let cohorte = emploi.cohorte_visee;
            *self.postes_pourvus.entry(cohorte).or_insert(0) -= 1;
            *self.heures_travaillees.entry(cohorte).or_insert(0) -= emploi.heures as u32;
            
            // Mettre à jour l'engagement : licenciement pour maladie = -20 à -40
            // Ici, on applique un delta négatif
            if let Some(engagement) = self.engagements.get_mut(&(employe, emploi.firme)) {
                // Delta = -20 à -40 selon la gravité
                let delta = -30; // Valeur moyenne
                engagement.mise_a_jour(delta, day, 500); // demi-vie de 500 jours
            }
            
            // Remettre le poste dans la demande (rompu)
            self.demande.push_back(emploi);
            
            return true;
        }
        false
    }
    
    /// Appariement événementiel (phase 4)
    /// §18.1 : volontés × demande par cohorte et pratique, événement par événement
    pub fn apparier(&mut self, _day: i64) -> Vec<(EntityId, EntityId)> {
        let mut embauches = Vec::new();
        
        // Pour l'instant, appariement simple : premier poste, premier candidat
        // À améliorer : appariement par cohorte et pratique
        
        // Parcourir la demande
        for emploi in self.demande.iter_mut() {
            if emploi.etat == EmploiEtats::Ouvert {
                // Chercher une cohorte avec de la volonté disponible
                let cohorte = emploi.cohorte_visee;
                if let Some(&volonte) = self.volonte.get(&cohorte) {
                    if volonte > 0 && self.disponibilite.get(&cohorte).map_or(0, |&d| d) > 0 {
                        // Trouver un employé (simplifié : on utilise la cohorte comme représentant)
                        let employe = cohorte; // À améliorer : gérer les individus
                        
                        // Embaucher
                        emploi.pourvoir();
                        *self.postes_pourvus.entry(cohorte).or_insert(0) += 1;
                        *self.heures_travaillees.entry(cohorte).or_insert(0) += emploi.heures as u32;
                        
                        // Créer engagement
                        let engagement = Engagement::new(employe, emploi.firme, 500);
                        self.engagements.insert((employe, emploi.firme), engagement);
                        
                        embauches.push((emploi.id, employe));
                        
                        // Réduire la disponibilité
                        *self.disponibilite.entry(cohorte).or_insert(0) -= emploi.heures as u32;
                    }
                }
            }
        }
        
        embauches
    }
    
    /// Parraine un journalier
    /// §34.3 : condition : pratique ≥ 3
    pub fn parrainer(
        &mut self,
        parrain: EntityId,
        filleul: EntityId,
        _day: i64,
    ) -> bool {
        // Vérifier que le parrain a une pratique ≥ 3
        // (À implémenter quand on aura le système de pratiques)
        
        let parrainage = LienParrainage::new(parrain, filleul);
        self.parrainages.push(parrainage);
        true
    }
    
    /// Calcule la production avec goulot
    /// §34.3 : production = min des capacités des étages pourvus
    pub fn production_goulot(&self, recette_id: u64, postes_pourvus: &BTreeMap<u64, bool>) -> (u32, Option<(u64, u32)>) {
        if let Some(recette) = self.recettes.get(&recette_id) {
            let production = recette.calculer_production(postes_pourvus);
            let goulot = recette.goulot(postes_pourvus);
            (production, goulot)
        } else {
            (0, None)
        }
    }
    
    /// Met à jour la disponibilité d'une cohorte
    pub fn mettre_a_jour_disponibilite(&mut self, cohorte: EntityId, heures: u32) {
        *self.disponibilite.entry(cohorte).or_insert(0) = heures;
    }
    
    /// Met à jour la volonté d'une cohorte
    pub fn mettre_a_jour_volonte(&mut self, cohorte: EntityId, heures: u32) {
        *self.volonte.entry(cohorte).or_insert(0) = heures;
    }
    
    /// Q(a) d'emploi : évalue une action pour une firme
    /// §18.2 : Q(a) d'emploi modulée par les traits du décideur
    pub fn qa_emploi(&self, firme: EntityId, action: ActionType) -> i64 {
        // Récupérer la table Q(a) de la firme
        // Pour l'instant, on utilise une table par défaut
        // À améliorer : chaque firme a sa propre table
        
        let table = self.get_qa_table(firme);
        table.evaluer(action)
    }
    
    /// Retourne la table Q(a) pour une firme
    pub fn get_qa_table(&self, firme: EntityId) -> QaTable {
        // Pour l'instant, table par défaut
        // À améliorer : stocker les tables par firme
        let mut table = QaTable::new();
        
        // Ajouter les actions de base avec des poids par défaut
        table.add_entry(ActionType::Embaucher, 500);
        table.add_entry(ActionType::Licencier, 300);
        table.add_entry(ActionType::MonterSalaire, 400);
        table.add_entry(ActionType::BaisserSalaire, 200);
        
        // Appliquer les traits du décideur
        if let Some(traits) = self.traits_decideurs.get(&firme) {
            table.appliquer_traits(traits);
        }
        
        table
    }
    
    /// Prend une décision ponctuelle de métier
    /// §18.3 : ~1/mois, à clé stable, même table pour PNJ et joueur
    pub fn decision_ponctuelle(
        &self,
        individu: EntityId,
        contexte: &DecisionPonctuelle,
        cle_stable: u64,
    ) -> ActionType {
        // Pour l'instant, sélection simple basée sur la clé
        // À améliorer : utiliser une table de décisions ponctuelles
        
        // Utiliser la clé stable pour déterminer l'option
        let index = (cle_stable % contexte.options.len() as u64) as usize;
        
        // Mapper l'option à une action
        // Pour l'instant, on retourne une action par défaut
        // À améliorer : mapping propre
        contexte.options.get(index).map(|&opt| {
            match opt {
                0 => ActionType::Embaucher,
                1 => ActionType::Licencier,
                2 => ActionType::MonterSalaire,
                3 => ActionType::BaisserSalaire,
                _ => ActionType::Embaucher,
            }
        }).unwrap_or(ActionType::Embaucher)
    }
    
    /// Met à jour l'engagement après une décision observable
    /// §6.20 : Mis à jour par les décisions observées de la firme
    pub fn mettre_a_jour_engagement_par_decision(
        &mut self,
        firme: EntityId,
        decision: ActionType,
        day: i64,
    ) {
        // Trouver tous les employés de cette firme
        for ((employe, f), engagement) in self.engagements.iter_mut() {
            if *f == firme {
                // Calculer le delta basé sur la décision
                let delta = match decision {
                    ActionType::Embaucher => 10, // Décision positive
                    ActionType::MonterSalaire => 15,
                    ActionType::Entretenir => 10,
                    ActionType::Licencier => -20, // Décision négative
                    ActionType::BaisserSalaire => -15,
                    ActionType::Fermer => -30,
                    _ => 0,
                };
                
                engagement.mise_a_jour(delta, day, 500);
            }
        }
    }
    
    /// Pondère l'arrêt maladie en fonction de l'engagement
    /// §6.20 : engagement bas → présentéisme, engagement haut → repos
    pub fn ponderer_arret_maladie(&self, employe: EntityId, firme: EntityId) -> bool {
        // Récupérer l'engagement
        if let Some(engagement) = self.engagements.get(&(employe, firme)) {
            // Seuil : engagement < 300 → présentéisme (continue à travailler)
            // engagement >= 300 → repos
            engagement.valeur >= 300
        } else {
            // Pas d'engagement enregistré → repos par défaut
            true
        }
    }
    
    /// Bascule du bouchon de travail (tranche 4 → tranche 6)
    /// §18.1 : Transition sans saut
    pub fn basculer_dotation(&mut self) {
        // Pour l'instant, rien à faire car on n'a pas de bouchon
        // À implémenter quand on intégrera avec la tranche 4
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
    fn test_emploi_creation() {
        let id = EntityId::new(crate::entities::EntityType::Job, 0);
        let firme = EntityId::new(crate::entities::EntityType::Firm, 1);
        let cohorte = EntityId::new(crate::entities::EntityType::Cohort, 2);
        
        let mut emploi = Emploi::new(id, firme, cohorte, 8, 100, Creneau::Matin, None);
        
        assert_eq!(emploi.etat, EmploiEtats::Ouvert);
        assert_eq!(emploi.heures, 8);
        assert_eq!(emploi.salaire_horaire, 100);
        
        emploi.pourvoir();
        assert_eq!(emploi.etat, EmploiEtats::Pourvu);
        
        emploi.rompre();
        assert_eq!(emploi.etat, EmploiEtats::Rompu);
    }
    
    #[test]
    fn test_traits_decideur_empathie() {
        let traits = TraitsDecideur::new(500); // empathie moyenne
        
        // seuil_base = 100, avec empathie 500
        // seuil = 100 * (1000 + 500) / 1000 = 100 * 1500 / 1000 = 150
        assert_eq!(traits.seuil_licenciement(100), 150);
        
        let traits_0 = TraitsDecideur::new(0);
        assert_eq!(traits_0.seuil_licenciement(100), 100);
        
        let traits_1000 = TraitsDecideur::new(1000);
        // 100 * (1000 + 1000) / 1000 = 200
        assert_eq!(traits_1000.seuil_licenciement(100), 200);
    }
    
    #[test]
    fn test_engagement_mise_a_jour() {
        let employe = EntityId::new(crate::entities::EntityType::NamedIndividual, 0);
        let firme = EntityId::new(crate::entities::EntityType::Firm, 1);
        
        let mut engagement = Engagement::new(employe, firme, 500);
        
        // Mise à jour avec delta positif
        engagement.mise_a_jour(100, 10, 500);
        assert!(engagement.valeur >= 500);
        
        // Mise à jour avec delta négatif
        engagement.mise_a_jour(-100, 20, 500);
        assert!(engagement.valeur < 500);
    }
    
    #[test]
    fn test_recette_goulot() {
        let mut recette = RecetteMultiPostes::new(1);
        recette.ajouter_etage(100, 10); // Poste 100 : capacité 10
        recette.ajouter_etage(101, 15); // Poste 101 : capacité 15
        
        // Tous les postes pourvus
        let mut postes_pourvus = BTreeMap::new();
        postes_pourvus.insert(100, true);
        postes_pourvus.insert(101, true);
        
        let production = recette.calculer_production(&postes_pourvus);
        let goulot = recette.goulot(&postes_pourvus);
        assert_eq!(production, 10); // min(10, 15) = 10
        assert_eq!(goulot, Some((100, 10))); // poste 100 est le goulot
        
        // Un poste non pourvu
        postes_pourvus.insert(100, false);
        let production = recette.calculer_production(&postes_pourvus);
        let goulot = recette.goulot(&postes_pourvus);
        assert_eq!(production, 0);
        assert_eq!(goulot, None);
    }
    
    #[test]
    fn test_marche_du_travail_publier() {
        let mut marche = MarcheDuTravail::new();
        
        let firme = EntityId::new(crate::entities::EntityType::Firm, 1);
        let cohorte = EntityId::new(crate::entities::EntityType::Cohort, 2);
        
        let id = marche.publier_poste(firme, cohorte, 8, 100, Creneau::Matin, None);
        
        assert_eq!(marche.demande.len(), 1);
        assert_eq!(marche.demande[0].id, id);
        assert_eq!(marche.demande[0].etat, EmploiEtats::Ouvert);
    }
    
    #[test]
    fn test_marche_du_travail_embauche() {
        let mut marche = MarcheDuTravail::new();
        
        let firme = EntityId::new(crate::entities::EntityType::Firm, 1);
        let cohorte = EntityId::new(crate::entities::EntityType::Cohort, 2);
        
        // Publier un poste
        let id = marche.publier_poste(firme, cohorte, 8, 100, Creneau::Matin, None);
        
        // Mettre à jour disponibilité
        marche.mettre_a_jour_disponibilite(cohorte, 16);
        marche.mettre_a_jour_volonte(cohorte, 10);
        
        // Embaucher
        let success = marche.embaucher(id, cohorte, 0);
        assert!(success);
        
        // Vérifier les stats
        assert_eq!(marche.postes_pourvus.get(&cohorte), Some(&1));
        assert_eq!(marche.heures_travaillees.get(&cohorte), Some(&8));
        assert_eq!(marche.demande[0].etat, EmploiEtats::Pourvu);
    }
    
    /// §34.4 T6.1 : Décisions d'emploi expliquées
    #[test]
    fn test_t6_1_decisions_emploi_expliquees() {
        let mut marche = MarcheDuTravail::new();
        
        let firme = EntityId::new(crate::entities::EntityType::Firm, 1);
        let cohorte = EntityId::new(crate::entities::EntityType::Cohort, 2);
        
        // Publier un poste
        let id = marche.publier_poste(firme, cohorte, 8, 100, Creneau::Matin, None);
        
        // Mettre à jour disponibilité et volonté
        marche.mettre_a_jour_disponibilite(cohorte, 16);
        marche.mettre_a_jour_volonte(cohorte, 10);
        
        // Embaucher
        let embauches = marche.apparier(0);
        
        // Vérifier que l'embauche est traçable
        assert!(!embauches.is_empty());
        assert_eq!(embauches[0].0, id); // Le poste est pourvu
    }
    
    /// §34.4 T6.4 : Parrainage
    #[test]
    fn test_t6_4_parrainage() {
        let mut marche = MarcheDuTravail::new();
        
        let parrain = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let filleul = EntityId::new(crate::entities::EntityType::NamedIndividual, 2);
        
        // Parrainer
        let success = marche.parrainer(parrain, filleul, 0);
        assert!(success);
        assert_eq!(marche.parrainages.len(), 1);
        assert_eq!(marche.parrainages[0].parrain, parrain);
        assert_eq!(marche.parrainages[0].filleul, filleul);
    }
    
    /// §34.4 T6.8 : Goulot multi-postes
    #[test]
    fn test_t6_8_goulot_multi_postes() {
        let mut marche = MarcheDuTravail::new();
        
        // Créer une recette multi-postes
        let mut recette = RecetteMultiPostes::new(1);
        recette.ajouter_etage(100, 10); // Cuisinier : 10 plats/demi-journée
        recette.ajouter_etage(101, 15); // Serveur : 15 couverts/demi-journée
        marche.recettes.insert(1, recette);
        
        // Aucun poste pourvu
        let mut postes_pourvus = BTreeMap::new();
        let (production, _goulot) = marche.production_goulot(1, &postes_pourvus);
        assert_eq!(production, 0);
        
        // Cuisinier seul
        postes_pourvus.insert(100, true);
        let (production, _goulot) = marche.production_goulot(1, &postes_pourvus);
        assert_eq!(production, 0); // Il manque le serveur
        
        // Cuisinier + serveur
        postes_pourvus.insert(101, true);
        let (production, goulot) = marche.production_goulot(1, &postes_pourvus);
        assert_eq!(production, 10); // Goulot = cuisinier (10)
        assert_eq!(goulot, Some((100, 10)));
    }
    
    // ============================================================================
    // Tests Q(a) - §18.2
    // ============================================================================
    
    /// §18.2 : Q(a) d'emploi modulée par traits
    #[test]
    fn test_qa_emploi_modulation() {
        let mut marche = MarcheDuTravail::new();
        
        let firme_patiente = EntityId::new(crate::entities::EntityType::Firm, 1);
        let firme_churn = EntityId::new(crate::entities::EntityType::Firm, 2);
        
        // Ajouter des traits
        marche.traits_decideurs.insert(firme_patiente, TraitsDecideur::new(800));
        marche.traits_decideurs.insert(firme_churn, TraitsDecideur::new(200));
        
        // Évaluer les actions
        let qa_patiente_licencier = marche.qa_emploi(firme_patiente, ActionType::Licencier);
        let qa_churn_licencier = marche.qa_emploi(firme_churn, ActionType::Licencier);
        
        let qa_patiente_embaucher = marche.qa_emploi(firme_patiente, ActionType::Embaucher);
        let qa_churn_embaucher = marche.qa_emploi(firme_churn, ActionType::Embaucher);
        
        // La firme patiente a un poids plus faible pour licencier
        assert!(qa_patiente_licencier < qa_churn_licencier);
        
        // La firme patiente a un poids plus élevé pour embaucher
        assert!(qa_patiente_embaucher > qa_churn_embaucher);
    }
    
    /// §18.2 : Q(a) table complète
    #[test]
    fn test_qa_table_selection() {
        let mut table = QaTable::new();
        
        // Ajouter des entrées avec différents poids
        table.add_entry(ActionType::Embaucher, 500);
        table.add_entry(ActionType::Licencier, 300);
        table.add_entry(ActionType::MonterSalaire, 400);
        
        // Sans modulation, Embaucher a le poids le plus élevé
        let meilleure = table.selectionner_meilleure();
        assert_eq!(meilleure, Some(ActionType::Embaucher));
        
        // Appliquer les traits (empathie élevée)
        table.appliquer_traits(&TraitsDecideur::new(1000));
        
        // Après modulation, Embaucher devrait toujours être la meilleure
        let meilleure = table.selectionner_meilleure();
        assert_eq!(meilleure, Some(ActionType::Embaucher));
    }
    
    /// §6.20 : Engagement mis à jour par décisions observées
    #[test]
    fn test_engagement_mis_a_jour_par_decision() {
        let mut marche = MarcheDuTravail::new();
        
        let firme = EntityId::new(crate::entities::EntityType::Firm, 1);
        let employe1 = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let employe2 = EntityId::new(crate::entities::EntityType::NamedIndividual, 2);
        
        // Créer des engagements initiaux
        marche.engagements.insert((employe1, firme), Engagement::new(employe1, firme, 500));
        marche.engagements.insert((employe2, firme), Engagement::new(employe2, firme, 500));
        
        // La firme prend une décision positive (embaucher)
        marche.mettre_a_jour_engagement_par_decision(firme, ActionType::Embaucher, 0);
        
        // L'engagement devrait augmenter
        let engagement1 = marche.engagements.get(&(employe1, firme)).unwrap();
        assert!(engagement1.valeur >= 500);
        
        // La firme prend une décision négative (licencier)
        marche.mettre_a_jour_engagement_par_decision(firme, ActionType::Licencier, 10);
        
        // L'engagement devrait diminuer
        let engagement1 = marche.engagements.get(&(employe1, firme)).unwrap();
        assert!(engagement1.valeur < 500);
    }
    
    /// §6.20 : Pondération arrêt maladie par engagement
    #[test]
    fn test_ponderer_arret_maladie() {
        let mut marche = MarcheDuTravail::new();
        
        let firme = EntityId::new(crate::entities::EntityType::Firm, 1);
        let employe_engage = EntityId::new(crate::entities::EntityType::NamedIndividual, 1);
        let employe_desengage = EntityId::new(crate::entities::EntityType::NamedIndividual, 2);
        
        // Engagement élevé
        marche.engagements.insert((employe_engage, firme), Engagement::new(employe_engage, firme, 800));
        
        // Engagement faible
        marche.engagements.insert((employe_desengage, firme), Engagement::new(employe_desengage, firme, 200));
        
        // L'employé engagé choisit le repos
        assert!(marche.ponderer_arret_maladie(employe_engage, firme));
        
        // L'employé désengagé choisit le présentéisme (continue à travailler)
        assert!(!marche.ponderer_arret_maladie(employe_desengage, firme));
    }
}
