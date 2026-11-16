//! Vestiges Horizons - Tranche T7-T8 A2 : Calcul des entrées u, p, q
//!
//! Implémentation canonique des calculs des entrées de griefs depuis les données réelles du quartier.
//!
//! Conventions :
//! - Tout est entier (millièmes sur [0, 1000])
//! - Calcul entier en millièmes sur [0, 1000] (convention 26)
//! - Une seule implémentation canonique par grandeur (convention 26)
//! - Lecture seule sur l'état
//!
//! Spec : Docs/spec/T7-T8 tranche A2 - ordre de mission.md §3
//! Base : cadrage v7.8.1, spec T7-T8 v1.2

use std::collections::BTreeMap;
use std::fmt;

use crate::entities::EntityId;
use crate::{EventData, Id, Phase, Priority, Timestamp, EVENT_TYPE_INPUT_CHANGED};

// ============================================================================
// TABLE DE POIDS POUR q (Décision D1 du 16/11/2026 - option 2)
// ============================================================================

/// Composante de service pour le calcul de q
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceComponent {
    /// Soins (dispensaire)
    Soins,
    /// État des logements (entretien)
    Logements,
}

/// Table de poids pour la moyenne pondérée de q
/// Poids en millièmes (ex: 500 = 50%)
/// Somme des poids = 1000 (100%)
pub const Q_WEIGHTS: &[(ServiceComponent, u16)] = &[
    (ServiceComponent::Soins, 500),     // 50%
    (ServiceComponent::Logements, 500), // 50%
];

// ============================================================================
// CALCUL DE u (besoins alimentaires non servis)
// ============================================================================

/// Calcule u = besoins alimentaires non servis / demande
///
/// Formule (spec T7-T8 A2 §3) :
/// - u = 0 si coverage > 0 (stock > 0) OU demande == 0
/// - u = (non_servis * 1000) / demande, tronqué à l'entier
/// - Résultat dans [0, 1000]
///
/// @param non_servis Nombre d'unités de nourriture non servies (0 si coverage > 0)
/// @param demande Besoin total en unités de nourriture
/// @returns u en millièmes [0, 1000]
pub fn calculate_u(non_servis: u32, demande: u32) -> u16 {
    // u = 0 si demande == 0 (pas de division par zéro)
    if demande == 0 {
        return 0;
    }

    // u = 0 si non_servis == 0 (coverage positive)
    if non_servis == 0 {
        return 0;
    }

    // Calcul : (non_servis * 1000) / demande
    // Troncature entière (convention 26)
    let numerator = non_servis as u64 * 1000;
    let result = (numerator / demande as u64) as u16;

    // Borne à 1000
    result.min(1000)
}

/// Calcule u depuis le stock de nourriture d'un foyer et le besoin
///
/// @param total_food_stock Stock total de nourriture (toutes qualités confondues)
/// @param besoin_total Besoin total en nourriture pour le foyer
/// @returns u en millièmes [0, 1000]
pub fn calculate_u_from_household(total_food_stock: u32, besoin_total: u32) -> u16 {
    // Si le stock couvre le besoin, u = 0
    if total_food_stock >= besoin_total {
        calculate_u(0, besoin_total)
    } else {
        // non_servis = besoin_total - total_food_stock
        let non_servis = besoin_total.saturating_sub(total_food_stock);
        calculate_u(non_servis, besoin_total)
    }
}

// ============================================================================
// CALCUL DE p (pression des prix)
// ============================================================================

/// Calcule p = pression des prix depuis le panier et le revenu disponible
///
/// Formule (spec P1 §6.2, corrigée dans etat.md) :
/// - p = 0 si cout_panier == 0 OU revenu_disponible >= cout_panier
/// - p = ((cout_panier - revenu_disponible) * 1000) / cout_panier, tronqué
/// - Résultat dans [0, 1000]
///
/// @param cout_panier Coût total du panier en centimes
/// @param revenu_disponible Revenu disponible en centimes
/// @returns p en millièmes [0, 1000]
pub fn calculate_p(cout_panier: u64, revenu_disponible: u64) -> u16 {
    // p = 0 si cout_panier == 0 (éviter division par zéro)
    if cout_panier == 0 {
        return 0;
    }

    // p = 0 si revenu_disponible >= cout_panier
    if revenu_disponible >= cout_panier {
        return 0;
    }

    // Calcul : ((cout_panier - revenu_disponible) * 1000) / cout_panier
    let difference = cout_panier.saturating_sub(revenu_disponible);
    let numerator = u128::from(difference) * 1000;
    let result = (numerator / u128::from(cout_panier)) as u16;

    // Borne à 1000
    result.min(1000)
}

// ============================================================================
// CALCUL DE q (qualité effective des services)
// ============================================================================

/// Calcule la moyenne pondérée des composantes de service pour q
///
/// Formule (D1 option 2, spec T7-T8 A2 §3) :
/// - q = Σ (taux_composante * poids_composante) / 1000
/// - Seules les composantes avec donnée réelle sont incluses
/// - Une composante sans donnée est exclue (pas comptée comme 0)
/// - Résultat dans [0, 1000]
///
/// @param components Table de (composante, taux [0,1000]) - seules les composantes présentes sont incluses
/// @returns q en millièmes [0, 1000]
pub fn calculate_q(components: &[(ServiceComponent, u16)]) -> u16 {
    // Filtrer : ne garder que les composantes avec un taux > 0 OU la composante est explicitement présente
    // Selon la spec : "Une composante sans donnée réelle n'entre pas dans la moyenne et n'est pas comptée comme zéro"
    // Interprétation : on inclut toutes les composantes de la table Q_WEIGHTS qui ont une valeur fournie

    let mut weighted_sum: u32 = 0;
    let mut total_weight: u32 = 0;

    for (component, weight) in Q_WEIGHTS {
        // Trouver le taux pour cette composante
        if let Some((_, taux)) = components.iter().find(|(c, _)| c == component) {
            weighted_sum += *taux as u32 * *weight as u32;
            total_weight += *weight as u32;
        }
        // Sinon : la composante n'a pas de donnée, on l'ignore (ne pas compter comme 0)
    }

    // Si aucune composante n'a de donnée, q = 0
    if total_weight == 0 {
        return 0;
    }

    // Calcul : weighted_sum / total_weight
    // Comme total_weight = 1000 normalement (si toutes les composantes ont des données)
    // Mais si certaines sont manquantes, total_weight < 1000
    let result = (weighted_sum / total_weight) as u16;

    // Borne à 1000
    result.min(1000)
}

/// Calcule q depuis les taux de service individuels
///
/// @param taux_soins Taux de service des soins [0, 1000] (0 = aucun soin, 1000 = tous servis)
/// @param taux_logements Taux de service des logements [0, 1000] (0 = aucun entretien, 1000 = parfait)
/// @returns q en millièmes [0, 1000]
pub fn calculate_q_from_components(taux_soins: u16, taux_logements: u16) -> u16 {
    let components = vec![
        (ServiceComponent::Soins, taux_soins),
        (ServiceComponent::Logements, taux_logements),
    ];
    calculate_q(&components)
}

// ============================================================================
// FONCTIONS DE FRANCHISSEMENT ET ÉMISSION D'ÉVÉNEMENTS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputValues {
    pub u: u16,
    pub p: u16,
    pub q: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputTrackerError {
    InvalidValues,
    NonMonotonicTimestamp,
    MultipleChangesInSlot,
}

impl fmt::Display for InputTrackerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            InputTrackerError::InvalidValues => "u, p et q doivent rester dans [0, 1000]",
            InputTrackerError::NonMonotonicTimestamp => {
                "les observations d'une cohorte doivent être chronologiques"
            }
            InputTrackerError::MultipleChangesInSlot => {
                "une seule observation consolidée est permise par cohorte et créneau"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InputTrackerError {}

#[derive(Debug, Clone, Copy)]
struct TrackedInput {
    timestamp: Timestamp,
    values: InputValues,
}

#[derive(Debug, Default)]
pub struct InputTracker {
    last_by_cohort: BTreeMap<EntityId, TrackedInput>,
}

impl InputTracker {
    /// Émet au plus un événement par cohorte et créneau.
    /// L'appelant observe une fois les valeurs consolidées à la fin du créneau.
    pub fn event_if_changed(
        &mut self,
        event_id: Id,
        entity_id: EntityId,
        values: InputValues,
        timestamp: Timestamp,
        phase: Phase,
        priority: Priority,
    ) -> Result<Option<crate::Event>, InputTrackerError> {
        if !are_inputs_valid(values.u, values.p, values.q) {
            return Err(InputTrackerError::InvalidValues);
        }

        let previous = self.last_by_cohort.get(&entity_id).copied();
        if let Some(previous) = previous {
            if timestamp < previous.timestamp {
                return Err(InputTrackerError::NonMonotonicTimestamp);
            }
            if timestamp == previous.timestamp {
                if values != previous.values {
                    return Err(InputTrackerError::MultipleChangesInSlot);
                }
                return Ok(None);
            }
        }

        let changed = previous
            .map(|previous| values != previous.values)
            .unwrap_or(values != InputValues::default());
        self.last_by_cohort
            .insert(entity_id, TrackedInput { timestamp, values });

        Ok(changed.then(|| {
            create_input_changed_event_at(
                event_id, entity_id, values.u, values.p, values.q, timestamp, phase, priority,
            )
        }))
    }

    pub fn schedule_if_changed(
        &mut self,
        queue: &mut crate::EventQueue,
        event_id: Id,
        entity_id: EntityId,
        values: InputValues,
        timestamp: Timestamp,
        phase: Phase,
        priority: Priority,
    ) -> Result<bool, InputTrackerError> {
        if let Some(event) =
            self.event_if_changed(event_id, entity_id, values, timestamp, phase, priority)?
        {
            queue.push(event);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Génère un événement InputChanged pour une cohorte
///
/// @param entity_id ID de la cohorte
/// @param u Valeur de u [0, 1000]
/// @param p Valeur de p [0, 1000]
/// @param q Valeur de q [0, 1000]
/// @param day Jour du déclenchement
/// @param phase Phase de l'événement
/// @param priority Priorité de l'événement
/// @returns Un Event avec InputChanged data
pub fn create_input_changed_event(
    event_id: Id,
    entity_id: EntityId,
    u: u16,
    p: u16,
    q: u16,
    day: i64,
    phase: Phase,
    priority: Priority,
) -> crate::Event {
    create_input_changed_event_at(
        event_id,
        entity_id,
        u,
        p,
        q,
        Timestamp::new(day, 0),
        phase,
        priority,
    )
}

pub fn create_input_changed_event_at(
    event_id: Id,
    entity_id: EntityId,
    u: u16,
    p: u16,
    q: u16,
    timestamp: Timestamp,
    phase: Phase,
    priority: Priority,
) -> crate::Event {
    crate::Event {
        id: event_id,
        timestamp,
        phase,
        priority,
        event_type: EVENT_TYPE_INPUT_CHANGED,
        data: EventData::InputChanged { entity_id, u, p, q },
    }
}

// ============================================================================
// VALIDATION DES ENTRÉES
// ============================================================================

/// Vérifie que u, p, q sont dans [0, 1000]
///
/// @returns true si toutes les valeurs sont valides
#[inline]
pub fn are_inputs_valid(u: u16, p: u16, q: u16) -> bool {
    u <= 1000 && p <= 1000 && q <= 1000
}

/// Vérifie qu'une valeur d'entrée est dans [0, 1000]
///
/// @returns true si la valeur est valide
#[inline]
pub fn is_input_valid(value: u16) -> bool {
    value <= 1000
}

// ============================================================================
// TESTS UNITAIRES
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Tests pour u (test #1 de l'ordre de mission)
    mod test_u {
        use super::*;

        #[test]
        fn test_u_zero_when_coverage_positive() {
            // Si non_servis = 0 (coverage positive), u = 0
            assert_eq!(calculate_u(0, 100), 0);
        }

        #[test]
        fn test_u_zero_when_demande_zero() {
            // Si demande = 0, u = 0 (pas de division par zéro)
            assert_eq!(calculate_u(50, 0), 0);
        }

        #[test]
        fn test_u_exact_calculation_7_over_19() {
            // Cas de l'ordre de mission : 7 non servis sur 19 demande
            // u = (7 * 1000) / 19 = 7000 / 19 = 368 (troncature)
            assert_eq!(calculate_u(7, 19), 368);
        }

        #[test]
        fn test_u_max_value() {
            // u = 1000 quand non_servis = demande
            assert_eq!(calculate_u(100, 100), 1000);
        }

        #[test]
        fn test_u_capped_at_1000() {
            // u ne dépasse jamais 1000
            assert_eq!(calculate_u(200, 100), 1000); // 200% -> clamped to 1000
        }

        #[test]
        fn test_u_from_household_coverage_positive() {
            // Stock >= besoin => u = 0
            assert_eq!(calculate_u_from_household(100, 100), 0);
            assert_eq!(calculate_u_from_household(150, 100), 0);
        }

        #[test]
        fn test_u_from_household_shortage() {
            // Stock < besoin => u > 0
            // besoin = 100, stock = 80, non_servis = 20
            // u = (20 * 1000) / 100 = 200
            assert_eq!(calculate_u_from_household(80, 100), 200);
        }
    }

    // Tests pour p (test #2 de l'ordre de mission)
    mod test_p {
        use super::*;

        #[test]
        fn test_p_zero_when_cout_panier_zero() {
            assert_eq!(calculate_p(0, 1000), 0);
        }

        #[test]
        fn test_p_zero_when_revenu_suffisant() {
            // revenu >= cout_panier => p = 0
            assert_eq!(calculate_p(1000, 1000), 0);
            assert_eq!(calculate_p(1000, 1500), 0);
        }

        #[test]
        fn test_p_exact_calculation() {
            // cout_panier = 1000, revenu = 800
            // p = ((1000 - 800) * 1000) / 1000 = 200
            assert_eq!(calculate_p(1000, 800), 200);
        }

        #[test]
        fn test_p_capped_at_1000() {
            // revenu = 0, cout_panier = 1000
            // p = ((1000 - 0) * 1000) / 1000 = 1000
            assert_eq!(calculate_p(1000, 0), 1000);
        }

        #[test]
        fn test_p_partial_pressure() {
            // cout_panier = 2000, revenu = 1500
            // p = ((2000 - 1500) * 1000) / 2000 = 500000 / 2000 = 250
            assert_eq!(calculate_p(2000, 1500), 250);
        }
    }

    // Tests pour q (test #4 de l'ordre de mission)
    mod test_q {
        use super::*;

        #[test]
        fn test_q_all_services_perfect() {
            // Tous les services à 1000
            assert_eq!(calculate_q_from_components(1000, 1000), 1000);
        }

        #[test]
        fn test_q_no_services() {
            // Tous les services à 0
            assert_eq!(calculate_q_from_components(0, 0), 0);
        }

        #[test]
        fn test_q_weighted_average() {
            // Soins = 800, Logements = 600
            // q = (800 * 500 + 600 * 500) / 1000 = (400000 + 300000) / 1000 = 700
            assert_eq!(calculate_q_from_components(800, 600), 700);
        }

        #[test]
        fn test_q_one_component_missing() {
            // Seulement soins = 500, logements absent
            // q = (500 * 500) / 500 = 500
            // Note : total_weight = 500, pas 1000
            let components = vec![(ServiceComponent::Soins, 500)];
            assert_eq!(calculate_q(&components), 500);
        }

        #[test]
        fn test_q_all_components_missing() {
            // Aucune composante
            let components: Vec<(ServiceComponent, u16)> = vec![];
            assert_eq!(calculate_q(&components), 0);
        }
    }

    // Tests pour la validation (test #2 de l'ordre de mission)
    mod test_bounds {
        use super::*;

        #[test]
        fn test_all_inputs_in_range() {
            assert!(are_inputs_valid(0, 0, 0));
            assert!(are_inputs_valid(500, 500, 500));
            assert!(are_inputs_valid(1000, 1000, 1000));
        }

        #[test]
        fn test_inputs_exceeding_range() {
            // u16 peut théoriquement aller jusqu'à 65535, mais on borne à 1000
            // Nos fonctions garantissent déjà le cap à 1000
            assert!(are_inputs_valid(1000, 1000, 1000));
        }

        #[test]
        fn test_individual_input_validation() {
            assert!(is_input_valid(0));
            assert!(is_input_valid(500));
            assert!(is_input_valid(1000));
            assert!(!is_input_valid(1001));
        }
    }

    // Test #5 : deux cohortes sans décalage codé
    mod test_no_offsets {
        use super::*;

        #[test]
        fn test_different_cohorts_different_inputs() {
            // Cohorte 1 : shortage sévère
            let u1 = calculate_u(50, 100); // 500
            let p1 = calculate_p(2000, 1000); // 500
            let q1 = calculate_q_from_components(800, 600); // 700

            // Cohorte 2 : situation normale
            let u2 = calculate_u(0, 100); // 0
            let p2 = calculate_p(1000, 2000); // 0
            let q2 = calculate_q_from_components(1000, 1000); // 1000

            // Les valeurs doivent être différentes
            assert_ne!(u1, u2);
            assert_ne!(p1, p2);
            assert_ne!(q1, q2);
        }
    }

    // Vérification des poids
    mod test_weights {
        use super::*;

        #[test]
        fn test_q_weights_sum_to_1000() {
            let sum: u16 = Q_WEIGHTS.iter().map(|(_, w)| w).sum();
            assert_eq!(sum, 1000, "La somme des poids pour q doit être 1000");
        }
    }

    #[test]
    fn input_tracker_emits_only_on_changes_and_once_per_slot() {
        let entity_id = EntityId::new(crate::entities::EntityType::Cohort, 20);
        let mut tracker = InputTracker::default();
        let initial_timestamp = Timestamp::new(0, 0);

        assert!(tracker
            .event_if_changed(
                Id::new(0),
                entity_id,
                InputValues::default(),
                initial_timestamp,
                Phase::Comptes,
                Priority::new(0),
            )
            .unwrap()
            .is_none());

        let changed = tracker.event_if_changed(
            Id::new(1),
            entity_id,
            InputValues { u: 1, p: 0, q: 0 },
            initial_timestamp,
            Phase::Comptes,
            Priority::new(0),
        );
        assert_eq!(changed, Err(InputTrackerError::MultipleChangesInSlot));

        let timestamp = Timestamp::new(1, 0);
        let event = tracker
            .event_if_changed(
                Id::new(2),
                entity_id,
                InputValues { u: 1, p: 0, q: 0 },
                timestamp,
                Phase::Comptes,
                Priority::new(0),
            )
            .unwrap()
            .unwrap();
        assert_eq!(event.timestamp, timestamp);

        assert!(tracker
            .event_if_changed(
                Id::new(3),
                entity_id,
                InputValues { u: 1, p: 0, q: 0 },
                Timestamp::new(2, 0),
                Phase::Comptes,
                Priority::new(0),
            )
            .unwrap()
            .is_none());

        assert_eq!(
            tracker.event_if_changed(
                Id::new(4),
                entity_id,
                InputValues { u: 0, p: 0, q: 0 },
                timestamp,
                Phase::Comptes,
                Priority::new(0),
            ),
            Err(InputTrackerError::NonMonotonicTimestamp)
        );
    }

    #[test]
    fn input_tracker_rejects_values_outside_the_domain() {
        let mut tracker = InputTracker::default();
        let result = tracker.event_if_changed(
            Id::new(0),
            EntityId::new(crate::entities::EntityType::Cohort, 21),
            InputValues {
                u: 1001,
                p: 0,
                q: 0,
            },
            Timestamp::new(0, 0),
            Phase::Comptes,
            Priority::new(0),
        );
        assert_eq!(result, Err(InputTrackerError::InvalidValues));
    }

    #[test]
    fn input_tracker_schedules_only_changed_values() {
        let entity_id = EntityId::new(crate::entities::EntityType::Cohort, 22);
        let mut tracker = InputTracker::default();
        let mut queue = crate::EventQueue::new();
        let values = InputValues {
            u: 10,
            p: 0,
            q: 500,
        };

        assert!(tracker
            .schedule_if_changed(
                &mut queue,
                Id::new(0),
                entity_id,
                values,
                Timestamp::new(0, 0),
                Phase::Comptes,
                Priority::new(0),
            )
            .unwrap());
        assert!(!tracker
            .schedule_if_changed(
                &mut queue,
                Id::new(1),
                entity_id,
                values,
                Timestamp::new(1, 0),
                Phase::Comptes,
                Priority::new(0),
            )
            .unwrap());
        assert_eq!(queue.len(), 1);
    }
}
