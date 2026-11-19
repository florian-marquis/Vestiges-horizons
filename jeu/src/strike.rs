//! Vestiges Horizons - Tranche 8 : Basculements (Grève)
//! 
//! Implémentation des grèves selon Docs/spec P1 - quartier vivant.md §20
//! 
//! - Basculement au franchissement d'un seuil de risque cumulé
//! - Tirage unique par clé stable (jamais re-tirable)
//! - Grève planifiée comme événement
//! - Fin de grève par concession ou durée écoulée

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crate::entities::{CausalJournal, DecisionType, EntityId, JournalEntry, Modification};
use crate::{Event, EventData, EventQueue, Id, Phase, Priority, Timestamp};
use crate::{
    EVENT_TYPE_GRIEF_BOUND_REACHED, EVENT_TYPE_STRIKE_COOLDOWN_ENDED,
    EVENT_TYPE_STRIKE_ENDED, EVENT_TYPE_STRIKE_TRIGGERED,
};

// ============================================================================
// TYPES
// ============================================================================

/// Cause de fin de grève
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrikeEndCause {
    /// La firme a cédé (décision Q(a))
    Concession,
    /// La durée de la grève est écoulée
    DurationElapsed,
}

/// Grève en cours ou historique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strike {
    /// Cohorte en grève
    pub cohort: EntityId,
    /// Jour de début de la grève
    pub start_day: i64,
    /// Durée prévue en jours (tirée à clé stable)
    pub planned_duration: i64,
    /// Cause de fin (si terminée)
    pub end_cause: Option<StrikeEndCause>,
    /// Jour de fin (si terminée)
    pub end_day: Option<i64>,
    /// Seuil de risque cumulé tiré pour cette grève (clé stable)
    pub risk_threshold: u64,
}

impl Strike {
    pub fn new(cohort: EntityId, start_day: i64, risk_threshold: u64, planned_duration: i64) -> Self {
        Strike {
            cohort,
            start_day,
            planned_duration,
            end_cause: None,
            end_day: None,
            risk_threshold,
        }
    }
    
    /// Vérifie si la grève est terminée
    pub fn is_ended(&self) -> bool {
        self.end_day.is_some()
    }
    
    /// Termine la grève avec une cause
    pub fn end(&mut self, end_day: i64, cause: StrikeEndCause) {
        self.end_day = Some(end_day);
        self.end_cause = Some(cause);
    }
    
    /// Retourne le jour de fin prévu (début + durée)
    pub fn planned_end_day(&self) -> i64 {
        self.start_day + self.planned_duration
    }
}

// ============================================================================
// GESTIONNAIRE DE GRÈVE
// ============================================================================

/// Gestionnaire des grèves
/// 
/// Garantit :
/// - Un seul tirage par grève (décision 43)
/// - La grève est planifiée comme événement
/// - La fin de grève a toujours une cause
pub struct StrikeManager {
    /// Grèves en cours et historiques
    strikes: BTreeMap<EntityId, Strike>,
    /// Graine du monde (pour les tirages déterministes)
    world_seed: u64,
    /// Automate événementiel par cohorte (Tranche B1, Commit 2 — spec §4)
    cohorts: BTreeMap<EntityId, CohortStrikeState>,
}

impl StrikeManager {
    pub fn new(world_seed: u64) -> Self {
        StrikeManager {
            strikes: BTreeMap::new(),
            world_seed,
            cohorts: BTreeMap::new(),
        }
    }
    
    /// Vérifie si une cohorte est en grève à un jour donné
    pub fn is_on_strike(&self, cohort: EntityId, day: i64) -> bool {
        if let Some(strike) = self.strikes.get(&cohort) {
            return !strike.is_ended()
                && day >= strike.start_day
                && day < strike.planned_end_day();
        }
        false
    }
    
    /// Retourne la grève pour une cohorte (si elle existe)
    pub fn get_strike(&self, cohort: EntityId) -> Option<&Strike> {
        self.strikes.get(&cohort)
    }
    
    /// Retourne toutes les grèves
    pub fn all_strikes(&self) -> impl Iterator<Item = (&EntityId, &Strike)> {
        self.strikes.iter()
    }
    
}

// ============================================================================
// MODULE DE CALCUL PUR — RISQUE CUMULÉ (Tranche B1, spec T7-T8 tranche B1 §3)
// ============================================================================
//
// Arithmétique 100 % entière (règle 26) : zéro f64/f32/exp/ln.
// Chaque arrondi est déclaré : troncature vers zéro ou plafond entier.

/// Échelle du risque cumulé : 10^12 picorisques (1.0 = 10^12).
pub const HAZARD_SCALE: u128 = 1_000_000_000_000;
/// Taux maximal annuel : 2_000 millièmes d'an⁻¹ (LAMBDA_MAX = 2,0).
pub const LAMBDA_MAX_MILLE: u64 = 2_000;
/// Organisation par défaut des 5 cohortes, en millièmes.
pub const DEFAULT_ORGANISATION_MILLE: [u16; 5] = [200, 400, 600, 600, 400];
/// Répit post-grève en jours (240 créneaux).
pub const STRIKE_COOLDOWN_DAYS: i64 = 60;
/// Créneaux par jour (réexport canonique depuis `griefs`, règle 26).
pub use crate::griefs::SLOTS_PER_DAY;
/// Créneaux par an (365 × 4).
pub const SLOTS_PER_YEAR: u64 = SLOTS_PER_DAY * 365;
/// Plafond de G, en quarts de micro-point (M = 100 pts) — réexport canonique depuis `griefs`.
pub use crate::griefs::MAX_GRIEFS_QMICRO;
/// Dénominateur analytique : (3 · 1460 · 10^6 · M²) / 10^12.
pub const HAZARD_INTEGRAL_DENOM: u128 = 700_800_000_000_000;
/// ln(2) · 10^12 arrondi : 693_147_180_560.
pub const LN2_SCALED: u64 = 693_147_180_560;
/// Tag de domaine pour le tirage du seuil de grève ("GREVE_SE" en ASCII hex).
pub const TAG_GREVE_SEUIL: u64 = 0x4752_4556_455F_5345;
/// Tag de domaine pour le tirage de la durée de grève ("GREVE_DU" en ASCII hex).
pub const TAG_GREVE_DUREE: u64 = 0x4752_4556_455F_4455;
/// Types de décision du journal causal (spec §5.2, règle 14).
pub const DECISION_INPUT_CHANGED: DecisionType = 70;
pub const DECISION_STRIKE_TRIGGERED: DecisionType = 80;
pub const DECISION_STRIKE_ENDED: DecisionType = 81;

/// Table statique précalculée hors moteur :
/// `LN1P_TABLE[i] = round(ln(1 + i/256) · 10^12)` pour i ∈ 0..=256,
/// avec `LN1P_TABLE[0] = 0` et `LN1P_TABLE[256] = LN2_SCALED`.
pub const LN1P_TABLE: [u64; 257] = [
    0, 3898640416, 7782140442, 11650617220, 15504186536, 19342962843, 23167059282, 26976587698,
    30771658667, 34552381507, 38318864302, 42071213921, 45809536031, 49533935122, 53244514519, 56941376400,
    60624621816, 64294350705, 67950661909, 71593653187, 75223421238, 78840061708, 82443669211, 86034337342,
    89612158690, 93177224854, 96729626459, 100269453164, 103796793682, 107311735789, 110814366340, 114304771280,
    117783035656, 121249243633, 124703478501, 128145822692, 131576357789, 134995164538, 138402322859, 141797911860,
    145182009844, 148554694323, 151916042026, 155266128911, 158605030177, 161932820269, 165249572895, 168555361030,
    171850256927, 175134332128, 178407657473, 181670303108, 184922338494, 188163832418, 191394853000, 194615467700,
    197825743330, 201025746061, 204215541429, 207395194346, 210564769107, 213724329398, 216873938301, 220013658305,
    223143551314, 226263678650, 229374101065, 232474878743, 235566071313, 238647737850, 241719936887, 244782726418,
    247836163905, 250880306286, 253915209981, 256940930898, 259957524437, 262965045501, 265963548497, 268953087346,
    271933715484, 274905485873, 277868451003, 280822662901, 283768173131, 286705032804, 289633292583, 292553002686,
    295464212894, 298366972552, 301261330578, 304147335467, 307025035295, 309894477723, 312755710004, 315608778986,
    318453731119, 321290612454, 324119468654, 326940344996, 329753286372, 332558337300, 335355541921, 338144944009,
    340926586971, 343700513853, 346466767346, 349225389785, 351976423157, 354719909103, 357455888922, 360184403575,
    362905493689, 365619199561, 368325561159, 371024618128, 373716409794, 376400975164, 379078352935, 381748581491,
    384411698910, 387067742968, 389716751140, 392358760603, 394993808241, 397621930647, 400243164127, 402857544701,
    405465108108, 408065889808, 410659924985, 413247248550, 415827895144, 418401899139, 420969294644, 423530115506,
    426084395311, 428632167390, 431173464818, 433708320422, 436236766775, 438758836208, 441274560805, 443783972410,
    446287102628, 448783982827, 451274644139, 453759117467, 456237433482, 458709622627, 461175715122, 463635740963,
    466089729925, 468537711563, 470979715219, 473415770017, 475845904870, 478270148481, 480688529346, 483101075751,
    485507815782, 487908777319, 490303988045, 492693475443, 495077266798, 497455389203, 499827869556, 502194734567,
    504556010752, 506911724445, 509261901790, 511606568749, 513945751102, 516279474448, 518607764208, 520930645624,
    523248143765, 525560283523, 527867089621, 530168586609, 532464798869, 534755750616, 537041465897, 539321968596,
    541597282433, 543867430967, 546132437598, 548392325566, 550647117953, 552896837687, 555141507541, 557381150134,
    559615787935, 561845443263, 564070138285, 566289895023, 568504735353, 570714681003, 572919753562, 575119974471,
    577315365035, 579505946415, 581691739635, 583872765581, 586049045004, 588220598517, 590387446602, 592549609607,
    594707107747, 596859961108, 599008189646, 601151813189, 603290851438, 605425323967, 607555250225, 609680649537,
    611801541106, 613917944012, 616029877216, 618137359555, 620240409752, 622339046409, 624433288012, 626523152931,
    628608659422, 630689825626, 632766669571, 634839209173, 636907462237, 638971446458, 641031179421, 643086678603,
    645137961374, 647185044995, 649227946625, 651266683315, 653301272013, 655331729563, 657358072708, 659380318089,
    661398482245, 663412581617, 665422632545, 667428651272, 669430653943, 671428656605, 673422675212, 675412725620,
    677398823592, 679380984796, 681359224808, 683333559112, 685304003099, 687270572071, 689233281239, 691192145724,
    693147180560,
];

/// Finaliseur SplitMix64 (bijection u64 → u64).
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hachage pur à clé stable : mélange bijectif (SplitMix64 en chaîne, zéro
/// allocation de chaîne) de `(world_seed, cohort_id, domain_tag, draw_index)`.
///
/// Pour les trois premiers champs fixés, la map `draw_index → sortie` est une
/// bijection 64 bits (chaque étape est inversible en le champ qu'elle xor).
pub fn stable_u64(world_seed: u64, cohort_id: u64, domain_tag: u64, draw_index: u64) -> u64 {
    let mut h = splitmix64(world_seed);
    h = splitmix64(h ^ cohort_id);
    h = splitmix64(h ^ domain_tag);
    h = splitmix64(h ^ draw_index);
    h
}

/// E_scaled = −ln(u) · 10^12 à partir de `raw`, sans f64 ni exp/ln.
///
/// `raw_65 = (raw << 1) | 1` représente u = raw_65 / 2^65 ∈ (0, 1).
/// Réduction exacte u = 2^(−k) (1 + f), puis ln(1+f) par table + interpolation
/// linéaire entière.
/// **Arrondi déclaré (règle 26)** : troncature entière vers zéro sur
/// `(delta * rem) >> 32` ; `E_scaled` borné à `max(1)`.
fn e_scaled_from_raw(raw: u64) -> u64 {
    if raw == 0 {
        // k = 65, frac = 0 ⇒ E = 65 · LN2_SCALED
        return 65 * LN2_SCALED;
    }
    let raw_65: u128 = ((raw as u128) << 1) | 1;
    let k = raw.leading_zeros() + 1; // 1..=64
    let shifted = raw_65 << (k - 1); // bit 64 = bit de tête, < 2^65
    let frac64 = (shifted & 0xFFFF_FFFF_FFFF_FFFF) as u64;
    let idx = (frac64 >> 56) as usize; // 0..=255
    let rem = (frac64 >> 24) & 0xFFFF_FFFF; // 0..2^32
    let delta = LN1P_TABLE[idx + 1] - LN1P_TABLE[idx];
    let ln1p = LN1P_TABLE[idx] + (((delta as u128) * (rem as u128)) >> 32) as u64;
    (u64::from(k) * LN2_SCALED).saturating_sub(ln1p).max(1)
}

/// Tire le seuil exponentiel E = −ln(u) mis à l'échelle 10^12 pour une cohorte
/// et un indice de tirage donnés. Garanti `>= 1` (durée nulle impossible).
pub fn draw_exponential_threshold_scaled(world_seed: u64, cohort_id: u64, draw_index: u64) -> u64 {
    let raw = stable_u64(world_seed, cohort_id, TAG_GREVE_SEUIL, draw_index);
    e_scaled_from_raw(raw)
}

/// I(k) = 3 · ∫₀ᵏ g(t)² dt en unités entières, avec g(t) = g0 + s·t clampé
/// aux butées 0 et M. Aucun écrêtage de k_full (troncature entière déclarée).
fn integral_g2_x3(g0: i64, s: i64, k: u64) -> u128 {
    if k == 0 {
        return 0;
    }
    let m = MAX_GRIEFS_QMICRO as i128;
    let g0c = (g0 as i128).clamp(0, m);
    let sc = s as i128;
    if sc == 0 {
        return 3u128 * k as u128 * (g0c * g0c) as u128;
    }
    let (bound, abs_s) = if sc > 0 { (m, sc) } else { (0i128, -sc) };
    let distance = (bound - g0c).abs();
    // Troncature entière déclarée (règle 26) : aucun écrêtage.
    let k_full = (distance / abs_s) as u64;
    let g_full = g0c + k_full as i128 * sc;
    if k <= k_full {
        let g1 = g0c + k as i128 * sc;
        return k as u128 * ((g0c * g0c + g0c * g1 + g1 * g1) as u128);
    }
    // Dernier créneau partiellement écrêté vers la butée.
    let k_bound = k_full + 1;
    let i_full = k_full as u128 * ((g0c * g0c + g0c * g_full + g_full * g_full) as u128);
    let partial = (g_full * g_full + g_full * bound + bound * bound) as u128;
    let mut i = i_full + partial;
    if k > k_bound && bound == m {
        // Plateau à M : 3 · M² par créneau supplémentaire.
        i += 3u128 * (k - k_bound) as u128 * (m * m) as u128;
    }
    // bound == 0 : contribution nulle après k_bound.
    i
}

/// Incrément de risque cumulé mis à l'échelle `HAZARD_SCALE` sur `k` créneaux,
/// avec g(t) = g0 + slope·t (pente en qµpt/créneau) et organisation `o_mille`.
///
/// **Arrondi déclaré (règle 26)** : troncature entière vers zéro en u128 sur
/// `num / HAZARD_INTEGRAL_DENOM` ; multiplications du numérateur saturantes.
pub fn delta_h_scaled(g0: i64, slope: i64, o_mille: u64, k: u64) -> u128 {
    let i = integral_g2_x3(g0, slope, k);
    let num = (LAMBDA_MAX_MILLE as u128)
        .saturating_mul(o_mille as u128)
        .saturating_mul(i);
    num / HAZARD_INTEGRAL_DENOM
}

/// Résultat de la recherche du plus petit créneau de franchissement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossingSlot {
    /// Plus petit k ≥ 1 tel que `delta_h_scaled(k) >= h_rem`.
    StrikeAtSlot(u64),
    /// Aucun franchissement avant la butée atteinte au créneau `k`.
    BoundReached { k: u64, bound_qmicro: i64 },
}

/// Bissection entière sur [lo, hi] : plus petit k tel que
/// `delta_h_scaled(k) >= h_rem` (invariant : delta(hi) >= h_rem).
/// `hi <= 4·10^8` ⇒ ≤ 29 itérations.
fn bisect_crossing(g0: i64, s: i64, o_mille: u64, h_rem: u128, mut lo: u64, mut hi: u64) -> u64 {
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if delta_h_scaled(g0, s, o_mille, mid) >= h_rem {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

/// Plus petit créneau k ≥ 1 de franchissement pour `h_rem = E_scaled − H_cum > 0`.
///
/// - `None` si aucun franchissement possible (g0 = 0 et s ≤ 0, ou o_mille = 0).
/// - s = 0 : formule fermée en O(1), **arrondi déclaré : plafond entier**.
/// - s ≠ 0 : plafond entier sur la butée, puis bissection entière.
pub fn find_crossing_slot(g0: i64, slope: i64, o_mille: u64, h_rem: u128) -> Option<CrossingSlot> {
    if o_mille == 0 {
        return None;
    }
    let m = MAX_GRIEFS_QMICRO;
    let g0c = g0.clamp(0, m);
    // Butée haute déjà atteinte : la pente positive se comporte comme un plateau à M.
    let s = if g0c == m && slope > 0 { 0 } else { slope };
    if g0c == 0 && s <= 0 {
        return None;
    }
    if h_rem == 0 {
        return Some(CrossingSlot::StrikeAtSlot(1));
    }
    if s == 0 {
        // A = LAMBDA_MAX_MILLE · o_mille · 3 · g0²
        let a = (LAMBDA_MAX_MILLE as u128)
            .saturating_mul(o_mille as u128)
            .saturating_mul(3u128)
            .saturating_mul((g0c as u128) * (g0c as u128));
        // k_cross = ceil(h_rem · DENOM / A) — plafond entier (arrondi déclaré, règle 26).
        let num = h_rem.saturating_mul(HAZARD_INTEGRAL_DENOM);
        let k = (num.saturating_add(a - 1)) / a;
        return Some(CrossingSlot::StrikeAtSlot(k.max(1).min(u64::MAX as u128) as u64));
    }
    let (bound, abs_s) = if s > 0 { (m, s) } else { (0, -s) };
    let distance = (bound - g0c).abs();
    // Plafond entier (arrondi déclaré, règle 26), borné à ≤ 4·10^8.
    let k_bound = ((distance + abs_s - 1) / abs_s) as u64;
    let d_bound = delta_h_scaled(g0c, s, o_mille, k_bound);
    if s > 0 {
        if d_bound >= h_rem {
            Some(CrossingSlot::StrikeAtSlot(bisect_crossing(g0c, s, o_mille, h_rem, 1, k_bound)))
        } else {
            Some(CrossingSlot::BoundReached { k: k_bound, bound_qmicro: bound })
        }
    } else {
        if d_bound < h_rem {
            Some(CrossingSlot::BoundReached { k: k_bound, bound_qmicro: bound })
        } else {
            Some(CrossingSlot::StrikeAtSlot(bisect_crossing(g0c, s, o_mille, h_rem, 1, k_bound)))
        }
    }
}

/// Tire la durée de grève en jours pour une cohorte, à clé stable.
///
/// **Arrondi déclaré (règle 26)** : troncature entière vers zéro sur
/// `max_duration_days = 1 + (20 · o_mille) / 1_000` (durées max [5, 9, 13, 13, 9]
/// pour `DEFAULT_ORGANISATION_MILLE`), puis `duration = 1 + raw % max`.
/// Résultat toujours dans `[1, 1 + floor(20·O)]`.
pub fn draw_strike_duration_days(world_seed: u64, cohort_id: u64, o_mille: u16, draw_index: u64) -> i64 {
    let max_duration_days = 1 + (20 * u64::from(o_mille)) / 1_000;
    let raw = stable_u64(world_seed, cohort_id, TAG_GREVE_DUREE, draw_index);
    (1 + (raw % max_duration_days)) as i64
}

// ============================================================================
// AUTOMATE ÉVÉNEMENTIEL DE GRÈVE (Tranche B1 — Commit 2, spec §4)
// ============================================================================
//
// Moteur purement événementiel : aucun balayage quotidien. Toute la logique
// passe par les événements 71-74 planifiés dans `EventQueue`.

/// Codes de kind pour l'encodage déterministe des Id d'événements grève (arbitrage 9).
pub const KIND_GRIEF_BOUND: u64 = 1;
pub const KIND_COOLDOWN_END: u64 = 2;
pub const KIND_STRIKE_TRIGGER: u64 = 3;
pub const KIND_STRIKE_END: u64 = 4;

/// Phase d'une cohorte dans l'automate de grève (spec §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrikePhaseState {
    /// Aucun seuil actif (G == 0 et pente <= 0, ou o_mille == 0).
    Dormant,
    /// Seuil actif, H_cum < seuil, prochain événement planifié.
    Accumulating,
    /// Franchissement constaté à un InputChanged : déclenchement à next_slot (règle 18).
    PendingNextSlot,
    /// Grève en cours, fin planifiée.
    OnStrike,
    /// Répit de STRIKE_COOLDOWN_DAYS jours ; seuil suivant déjà tiré, intégration gelée.
    Cooldown,
}

/// Contribution causale d'une entrée de journal (spec §5.1, arbitrage 5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CauseContribution {
    pub journal_entry_id: u64,
    pub delta_h_scaled: u128,
}

/// État automate d'une cohorte (spec §4.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortStrikeState {
    pub phase: StrikePhaseState,
    /// Compteur de tirages, incrémenté à chaque nouveau seuil (crise initiale,
    /// remise à zéro à G = 0, fin de grève).
    pub draw_index: u64,
    /// Indice du tirage actif (alimente à la fois le seuil et la durée).
    pub active_draw_index: u64,
    /// Seuil actif E_scaled (0 = aucun).
    pub active_threshold: u64,
    /// Organisation active, en millièmes.
    pub active_o_mille: u16,
    /// Risque cumulé H_cum à l'échelle 10^12.
    pub h_cum_scaled: u128,
    /// Nombre de grèves déclenchées.
    pub strike_index: u64,
    /// Version de planification : tout événement d'une version différente est
    /// ignoré en O(1) (noop sûr).
    pub schedule_version: u64,
    pub last_update: Timestamp,
    pub g_last_qmicro: i64,
    pub slope_last: i64,
    pub duration_days: i64,
    pub strike_start: Timestamp,
    pub cooldown_end: Timestamp,
    /// Tampon borné (≤ 5) des causes de la crise courante (spec §5.1) :
    /// trié par `delta_h_scaled` décroissant puis `journal_entry_id` croissant.
    pub causes: Vec<CauseContribution>,
    /// Entrée de journal de l'InputChanged actif (dernier segment en date).
    pub last_input_entry_id: Option<u64>,
}

impl CohortStrikeState {
    pub fn dormant(at: Timestamp) -> Self {
        CohortStrikeState {
            phase: StrikePhaseState::Dormant,
            draw_index: 0,
            active_draw_index: 0,
            active_threshold: 0,
            active_o_mille: 0,
            h_cum_scaled: 0,
            strike_index: 0,
            schedule_version: 0,
            last_update: at,
            g_last_qmicro: 0,
            slope_last: 0,
            duration_days: 0,
            strike_start: at,
            cooldown_end: at,
            causes: Vec::new(),
            last_input_entry_id: None,
        }
    }
}

/// Fusionne une contribution dans le tampon borné top-5 (spec §5.1) :
/// même `journal_entry_id` → somme ; tri `delta_h_scaled` DESC puis
/// `journal_entry_id` ASC ; troncature à 5 (coût O(1)).
fn add_cause(st: &mut CohortStrikeState, journal_entry_id: u64, delta: u128) {
    if delta == 0 {
        return;
    }
    if let Some(pos) = st
        .causes
        .iter()
        .position(|c| c.journal_entry_id == journal_entry_id)
    {
        st.causes[pos].delta_h_scaled += delta;
    } else {
        st.causes.push(CauseContribution {
            journal_entry_id,
            delta_h_scaled: delta,
        });
    }
    st.causes.sort_by(|a, b| {
        b.delta_h_scaled
            .cmp(&a.delta_h_scaled)
            .then(a.journal_entry_id.cmp(&b.journal_entry_id))
    });
    st.causes.truncate(5);
}

/// Credite un segment d'intégration H_cum au tampon de causes (entrée
/// d'InputChanged active sur le segment) et cumule H_cum.
fn credit_segment(st: &mut CohortStrikeState, delta: u128) {
    if delta > 0 {
        if let Some(eid) = st.last_input_entry_id {
            add_cause(st, eid, delta);
        }
    }
    st.h_cum_scaled += delta;
}

/// Créneau logique suivant (règle 18) : (day, seq+1) si seq < 3, sinon (day+1, 0).
pub fn next_slot(ts: Timestamp) -> Timestamp {
    if ts.seq < SLOTS_PER_DAY as u32 - 1 {
        Timestamp::new(ts.day, ts.seq + 1)
    } else {
        Timestamp::new(ts.day + 1, 0)
    }
}

/// Ajoute `k` créneaux à `ts`, en saturant au plafond sûr
/// `max_total = (i64::MAX/8) * spd + (spd-1)` (calcul i128, jamais de débordement :
/// un `k` géant venant de la formule fermée de `find_crossing_slot` ne produit
/// jamais un instant négatif ou dans le passé).
pub fn add_slots(ts: Timestamp, k: u64) -> Timestamp {
    let spd = SLOTS_PER_DAY as i128;
    let max_total = (i64::MAX as i128 / 8) * spd + (spd - 1);
    let total = ((ts.day as i128) * spd + i128::from(ts.seq) + i128::from(k)).min(max_total);
    Timestamp::new(total.div_euclid(spd) as i64, total.rem_euclid(spd) as u32)
}

/// Nombre de créneaux entre `a` et `b`, borné à `0..=u64::MAX` (calcul i128).
pub fn slots_between(a: Timestamp, b: Timestamp) -> u64 {
    let spd = SLOTS_PER_DAY as i128;
    let diff = (b.day as i128 - a.day as i128) * spd + (i128::from(b.seq) - i128::from(a.seq));
    if diff <= 0 { 0 } else { diff.min(u64::MAX as i128) as u64 }
}

/// Encodage déterministe d'un Id d'événement grève (arbitrage 9, spec §4.2) :
/// bit 63 = 1 (namespace disjoint des ids 0..2^63), bits 60..62 = kind_code,
/// bits 28..59 = cohort.number, bits 0..27 = schedule_version.
/// Jamais de compteur partagé entre cohortes.
pub fn encode_strike_event_id(kind_code: u64, cohort: EntityId, schedule_version: u64) -> Id {
    Id::new(
        (1u64 << 63)
            | ((kind_code & 0x7) << 60)
            | ((cohort.number & 0xFFFF_FFFF) << 28)
            | (schedule_version & 0x0FFF_FFFF),
    )
}

impl StrikeManager {
    // ------------------------------------------------------------------
    // Automate B1 (Commit 2)
    // ------------------------------------------------------------------

    /// État automate d'une cohorte (None si jamais vue).
    pub fn cohort_state(&self, cohort: EntityId) -> Option<&CohortStrikeState> {
        self.cohorts.get(&cohort)
    }

    /// État automate mutable (crée l'état Dormant si absent).
    pub fn cohort_state_mut(&mut self, cohort: EntityId) -> &mut CohortStrikeState {
        self.cohorts
            .entry(cohort)
            .or_insert_with(|| CohortStrikeState::dormant(Timestamp::new(0, 0)))
    }

    /// Phase courante d'une cohorte (Dormant si inconnue).
    pub fn phase_of(&self, cohort: EntityId) -> StrikePhaseState {
        self.cohorts
            .get(&cohort)
            .map_or(StrikePhaseState::Dormant, |s| s.phase)
    }

    /// Tire un nouveau seuil (draw_index incrémenté) et remet H_cum à zéro.
    fn draw_new_threshold(&mut self, cohort: EntityId) {
        if let Some(st) = self.cohorts.get_mut(&cohort) {
            st.draw_index += 1;
            st.active_draw_index = st.draw_index;
            st.active_threshold =
                draw_exponential_threshold_scaled(self.world_seed, cohort.to_u64(), st.active_draw_index);
            st.h_cum_scaled = 0;
        }
    }

    /// Planifie `StrikeTriggered` à `t` en Phase::Social, Priority(2).
    fn schedule_trigger(
        &mut self,
        cohort: EntityId,
        t: Timestamp,
        draw_index: u64,
        strike_index: u64,
        queue: &mut EventQueue,
    ) {
        let st = self.cohorts.get_mut(&cohort).expect("cohort registered");
        st.schedule_version += 1;
        let sv = st.schedule_version;
        let id = encode_strike_event_id(KIND_STRIKE_TRIGGER, cohort, sv);
        queue.push(Event::new(
            id,
            t.day,
            t.seq,
            Phase::Social,
            Priority(2),
            EVENT_TYPE_STRIKE_TRIGGERED,
            EventData::StrikeTriggered {
                entity_id: cohort,
                draw_index,
                strike_index,
                schedule_version: sv,
            },
        ));
    }

    /// Planifie `GriefBoundReached` à `t` en Phase::Social, Priority(0).
    fn schedule_bound(&mut self, cohort: EntityId, t: Timestamp, bound_qmicro: i64, queue: &mut EventQueue) {
        let st = self.cohorts.get_mut(&cohort).expect("cohort registered");
        st.schedule_version += 1;
        let sv = st.schedule_version;
        let id = encode_strike_event_id(KIND_GRIEF_BOUND, cohort, sv);
        queue.push(Event::new(
            id,
            t.day,
            t.seq,
            Phase::Social,
            Priority(0),
            EVENT_TYPE_GRIEF_BOUND_REACHED,
            EventData::GriefBoundReached {
                entity_id: cohort,
                bound_qmicro,
                schedule_version: sv,
            },
        ));
    }

    /// Planifie le prochain événement d'une cohorte Accumulating à partir de `ts`
    /// (spec §3.4.3, §4.3). Incrémente `schedule_version` (invalide l'ancien événement).
    fn plan_next(&mut self, cohort: EntityId, ts: Timestamp, g: i64, slope: i64, queue: &mut EventQueue) {
        let (h_rem, o_mille, active_draw_index, strike_index) = {
            let st = match self.cohorts.get_mut(&cohort) {
                Some(s) => s,
                None => return,
            };
            if st.phase != StrikePhaseState::Accumulating {
                return;
            }
            (
                (st.active_threshold as u128).saturating_sub(st.h_cum_scaled),
                u64::from(st.active_o_mille),
                st.active_draw_index,
                st.strike_index,
            )
        };
        if h_rem == 0 {
            // Seuil déjà atteint : règle 18, déclenchement au prochain créneau.
            if let Some(st) = self.cohorts.get_mut(&cohort) {
                st.phase = StrikePhaseState::PendingNextSlot;
            }
            self.schedule_trigger(cohort, next_slot(ts), active_draw_index, strike_index, queue);
            return;
        }
        match find_crossing_slot(g, slope, o_mille, h_rem) {
            None => {
                // Aucun franchissement possible (o_mille == 0, ou G == 0 et pente <= 0) :
                // retour en Dormant, seuil abandonné (invalide l'éventuel événement en attente).
                if let Some(st) = self.cohorts.get_mut(&cohort) {
                    st.schedule_version += 1;
                    st.phase = StrikePhaseState::Dormant;
                    st.h_cum_scaled = 0;
                    st.active_threshold = 0;
                    st.causes.clear();
                }
            }
            Some(CrossingSlot::StrikeAtSlot(k)) => {
                let t = add_slots(ts, k.max(1));
                self.schedule_trigger(cohort, t, active_draw_index, strike_index, queue);
            }
            Some(CrossingSlot::BoundReached { k, bound_qmicro }) => {
                let t = add_slots(ts, k.max(1));
                self.schedule_bound(cohort, t, bound_qmicro, queue);
            }
        }
    }

    /// Traitement d'un `InputChanged` à `ts` (Phase::Comptes) pour une cohorte.
    ///
    /// `g_now` = G après avance de GriefsState à `ts` avec les ANCIENNES entrées ;
    /// `new_slope` = pente (qµpt/créneau) APRÈS application des nouvelles entrées ;
    /// `o_mille` = organisation courante. Intègre H_cum sur [last_update, ts]
    /// avec les anciennes entrées, puis applique la règle 18 ou replanifie.
    pub fn on_input_changed(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        g_now: i64,
        new_slope: i64,
        o_mille: u16,
        queue: &mut EventQueue,
    ) {
        if !self.cohorts.contains_key(&cohort) {
            self.cohorts.insert(cohort, CohortStrikeState::dormant(ts));
        }
        // 1. Intégrer H_cum sur [last_update, ts] avec les anciennes entrées.
        {
            let st = self.cohorts.get_mut(&cohort).unwrap();
            if st.phase == StrikePhaseState::Accumulating
                || st.phase == StrikePhaseState::PendingNextSlot
            {
                let k = slots_between(st.last_update, ts);
                if k > 0 {
                    let delta = delta_h_scaled(
                        st.g_last_qmicro,
                        st.slope_last,
                        u64::from(st.active_o_mille),
                        k,
                    );
                    credit_segment(st, delta);
                }
            }
            // 2. Observer le nouvel état.
            st.last_update = ts;
            st.g_last_qmicro = g_now.clamp(0, MAX_GRIEFS_QMICRO);
            st.slope_last = new_slope;
            st.active_o_mille = o_mille;
        }
        // 3. Transition d'état.
        let phase = self.cohorts[&cohort].phase;
        match phase {
            StrikePhaseState::Dormant => {
                if g_now > 0 || new_slope > 0 {
                    self.draw_new_threshold(cohort);
                    if let Some(st) = self.cohorts.get_mut(&cohort) {
                        st.phase = StrikePhaseState::Accumulating;
                    }
                    self.plan_next(cohort, ts, g_now, new_slope, queue);
                }
            }
            StrikePhaseState::Accumulating => {
                let crossed = {
                    let st = &self.cohorts[&cohort];
                    st.h_cum_scaled >= u128::from(st.active_threshold)
                };
                if crossed {
                    // Règle 18 : jamais d'exécution dans le même instant logique.
                    let (di, si) = {
                        let st = &self.cohorts[&cohort];
                        (st.active_draw_index, st.strike_index)
                    };
                    if let Some(st) = self.cohorts.get_mut(&cohort) {
                        st.phase = StrikePhaseState::PendingNextSlot;
                    }
                    self.schedule_trigger(cohort, next_slot(ts), di, si, queue);
                } else {
                    self.plan_next(cohort, ts, g_now, new_slope, queue);
                }
            }
            // PendingNextSlot : aucun replan ne peut annuler le déclenchement imminent.
            // OnStrike / Cooldown : intégration gelée, aucun replan.
            _ => {}
        }
    }

    /// Traitement d'un événement `StrikeTriggered` pompé à `ts`.
    /// `g`/`slope` = observation GriefsState avancée à `ts`.
    /// Retourne false si l'événement est obsolète (schedule_version) — noop sûr.
    pub fn on_strike_triggered(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        g: i64,
        slope: i64,
        draw_index: u64,
        strike_index: u64,
        schedule_version: u64,
        queue: &mut EventQueue,
    ) -> bool {
        let st = match self.cohorts.get_mut(&cohort) {
            Some(s) => s,
            None => return false,
        };
        if schedule_version != st.schedule_version {
            return false;
        }
        if st.phase != StrikePhaseState::PendingNextSlot && st.phase != StrikePhaseState::Accumulating {
            return false;
        }
        if draw_index != st.active_draw_index || strike_index != st.strike_index {
            return false;
        }
        // Intégrer le segment final avec les anciennes entrées.
        let k = slots_between(st.last_update, ts);
        if k > 0 {
            let delta = delta_h_scaled(st.g_last_qmicro, st.slope_last, u64::from(st.active_o_mille), k);
            credit_segment(st, delta);
        }
        st.last_update = ts;
        st.g_last_qmicro = g.clamp(0, MAX_GRIEFS_QMICRO);
        st.slope_last = slope;
        // Déclenchement : G inchangé, strike_index += 1, durée tirée à clé stable.
        st.phase = StrikePhaseState::OnStrike;
        st.strike_index += 1;
        st.duration_days = draw_strike_duration_days(
            self.world_seed,
            cohort.to_u64(),
            st.active_o_mille,
            st.active_draw_index,
        );
        st.strike_start = ts;
        let (duration, si, threshold) = (st.duration_days, st.strike_index, st.active_threshold);
        st.schedule_version += 1;
        let sv = st.schedule_version;
        // Planifier StrikeEnded(DurationElapsed) à (day + duration, seq).
        let end = Timestamp::new(ts.day + duration, ts.seq);
        let id = encode_strike_event_id(KIND_STRIKE_END, cohort, sv);
        queue.push(Event::new(
            id,
            end.day,
            end.seq,
            Phase::Social,
            Priority(3),
            EVENT_TYPE_STRIKE_ENDED,
            EventData::StrikeEnded {
                entity_id: cohort,
                strike_index: si,
                cause: StrikeEndCause::DurationElapsed,
                schedule_version: sv,
            },
        ));
        self.strikes
            .insert(cohort, Strike::new(cohort, ts.day, threshold, duration));
        true
    }

    /// Traitement d'un événement `StrikeEnded` à `ts`. Passe en Cooldown,
    /// tire immédiatement le seuil suivant, planifie `StrikeCooldownEnded`.
    /// Retourne false si obsolète.
    pub fn on_strike_ended(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        g: i64,
        slope: i64,
        strike_index: u64,
        cause: StrikeEndCause,
        schedule_version: u64,
        queue: &mut EventQueue,
    ) -> bool {
        if !self
            .strikes
            .get(&cohort)
            .map_or(false, |strike| !strike.is_ended())
        {
            return false;
        }
        let ok = {
            let st = match self.cohorts.get_mut(&cohort) {
                Some(s) => s,
                None => return false,
            };
            if schedule_version != st.schedule_version
                || st.phase != StrikePhaseState::OnStrike
                || strike_index != st.strike_index
            {
                false
            } else {
                st.last_update = ts;
                st.g_last_qmicro = g.clamp(0, MAX_GRIEFS_QMICRO);
                st.slope_last = slope;
                st.phase = StrikePhaseState::Cooldown;
                st.h_cum_scaled = 0;
                st.cooldown_end = add_slots(ts, (STRIKE_COOLDOWN_DAYS * SLOTS_PER_DAY as i64) as u64);
                // Tirage immédiat du nouveau seuil (draw_index incrémenté).
                st.draw_index += 1;
                st.active_draw_index = st.draw_index;
                st.active_threshold = draw_exponential_threshold_scaled(
                    self.world_seed,
                    cohort.to_u64(),
                    st.active_draw_index,
                );
                true
            }
        };
        if !ok {
            return false;
        }
        self.strikes
            .get_mut(&cohort)
            .expect("active Strike record for OnStrike state")
            .end(ts.day, cause);
        // Planifier StrikeCooldownEnded à cooldown_end, Phase::Social, Priority(1).
        let (ce, sv) = {
            let st = self.cohorts.get_mut(&cohort).unwrap();
            st.schedule_version += 1;
            (st.cooldown_end, st.schedule_version)
        };
        let id = encode_strike_event_id(KIND_COOLDOWN_END, cohort, sv);
        queue.push(Event::new(
            id,
            ce.day,
            ce.seq,
            Phase::Social,
            Priority(1),
            EVENT_TYPE_STRIKE_COOLDOWN_ENDED,
            EventData::StrikeCooldownEnded {
                entity_id: cohort,
                schedule_version: sv,
            },
        ));
        true
    }

    /// Traitement d'un événement `StrikeCooldownEnded` à `ts`.
    /// Si G > 0 ou pente > 0 : reprise sur le seuil déjà tiré ; sinon Dormant (remise à zéro).
    pub fn on_cooldown_ended(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        g: i64,
        slope: i64,
        schedule_version: u64,
        queue: &mut EventQueue,
    ) -> bool {
        {
            let st = match self.cohorts.get_mut(&cohort) {
                Some(s) => s,
                None => return false,
            };
            if schedule_version != st.schedule_version || st.phase != StrikePhaseState::Cooldown {
                return false;
            }
            // Intégration gelée pendant le répit : on avance juste l'observation.
            st.last_update = ts;
            st.g_last_qmicro = g.clamp(0, MAX_GRIEFS_QMICRO);
            st.slope_last = slope;
        }
        if g > 0 || slope > 0 {
            if let Some(st) = self.cohorts.get_mut(&cohort) {
                st.phase = StrikePhaseState::Accumulating;
            }
            self.plan_next(cohort, ts, g, slope, queue);
        } else {
            // G == 0 sans grève : remise à zéro.
            if let Some(st) = self.cohorts.get_mut(&cohort) {
                st.phase = StrikePhaseState::Dormant;
                st.h_cum_scaled = 0;
                st.active_threshold = 0;
            }
        }
        true
    }

    /// Traitement d'un événement `GriefBoundReached` à `ts`.
    /// Butée 0 sans grève : remise à zéro (Dormant). Butée M : replanifier sur le
    /// plateau G = M, pente 0, sans nouveau tirage.
    pub fn on_grief_bound_reached(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        bound_qmicro: i64,
        schedule_version: u64,
        g: i64,
        slope: i64,
        queue: &mut EventQueue,
    ) -> bool {
        {
            let st = match self.cohorts.get_mut(&cohort) {
                Some(s) => s,
                None => return false,
            };
            if schedule_version != st.schedule_version {
                return false;
            }
            if st.phase != StrikePhaseState::Accumulating
                && st.phase != StrikePhaseState::PendingNextSlot
            {
                return false;
            }
            // Crediter H_cum sur le segment (delta_h gère la butée en interne).
            let k = slots_between(st.last_update, ts);
            if k > 0 {
                let delta = delta_h_scaled(
                    st.g_last_qmicro,
                    st.slope_last,
                    u64::from(st.active_o_mille),
                    k,
                );
                credit_segment(st, delta);
            }
            st.last_update = ts;
            st.g_last_qmicro = g.clamp(0, MAX_GRIEFS_QMICRO);
            st.slope_last = slope;
        }
        if bound_qmicro == 0 {
            // Remise à zéro à G = 0 sans grève (spec §4.3) : le suivi des causes est vidé.
            {
                let st = self.cohorts.get_mut(&cohort).unwrap();
                st.h_cum_scaled = 0;
                st.active_threshold = 0;
                st.phase = StrikePhaseState::Dormant;
                st.causes.clear();
            }
            if g > 0 || slope > 0 {
                self.draw_new_threshold(cohort);
                if let Some(st) = self.cohorts.get_mut(&cohort) {
                    st.phase = StrikePhaseState::Accumulating;
                }
                self.plan_next(cohort, ts, g, slope, queue);
            }
        } else {
            // Butée M atteinte : replanifier sur le plateau (formule fermée O(1)).
            if let Some(st) = self.cohorts.get_mut(&cohort) {
                st.g_last_qmicro = MAX_GRIEFS_QMICRO;
                st.slope_last = 0;
            }
            self.plan_next(cohort, ts, MAX_GRIEFS_QMICRO, 0, queue);
        }
        true
    }

    // ------------------------------------------------------------------
    // Journalisation causale (Commit 3, spec §5)
    // ------------------------------------------------------------------

    /// Journalise un événement `InputChanged` en `Phase::Comptes`
    /// (DECISION_INPUT_CHANGED = 70) et mémorise son `journal_entry_id` comme
    /// entrée active pour l'attribution des prochains segments H_cum.
    pub fn journal_input_changed(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        event_id: u64,
        u: u16,
        p: u16,
        q: u16,
        journal: &mut CausalJournal,
    ) -> Option<u64> {
        let entry = JournalEntry {
            id: 0,
            timestamp: ts,
            event_id,
            phase: Phase::Comptes as u8,
            actor: Some(cohort),
            decision_type: DECISION_INPUT_CHANGED,
            modified_values: vec![
                Modification { field: "input_u".into(), old_value: 0, new_value: i64::from(u) },
                Modification { field: "input_p".into(), old_value: 0, new_value: i64::from(p) },
                Modification { field: "input_q".into(), old_value: 0, new_value: i64::from(q) },
            ],
            causes: vec![],
            learned_by: vec![],
        };
        let id = journal.add_entry(entry)?;
        if let Some(st) = self.cohorts.get_mut(&cohort) {
            st.last_input_entry_id = Some(id);
        }
        Some(id)
    }

    /// Journalise un `StrikeTriggered` en `Phase::Social` (règle 14, spec §5.2) :
    /// tous les champs obligatoires, `causes` = top-5 du tampon, puis le tampon
    /// est vidé (crise clôturée par la grève).
    pub fn journal_strike_triggered(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        event_id: u64,
        g_qmicro: i64,
        journal: &mut CausalJournal,
    ) -> Option<u64> {
        let (si, di, o, thr, dur, causes_ids) = {
            let st = self.cohorts.get_mut(&cohort)?;
            (
                st.strike_index,
                st.active_draw_index,
                u64::from(st.active_o_mille),
                st.active_threshold,
                st.duration_days,
                st.causes.iter().map(|c| c.journal_entry_id).collect::<Vec<u64>>(),
            )
        };
        let g = g_qmicro.clamp(0, MAX_GRIEFS_QMICRO);
        // lambda_mille_per_year = LAMBDA_MAX_MILLE · o · g² / (1000 · M²), troncature entière (u128).
        let lambda_mille = ((LAMBDA_MAX_MILLE as u128)
            * (o as u128)
            * (g as u128).pow(2))
            / (1000u128 * (MAX_GRIEFS_QMICRO as u128).pow(2));
        let entry = JournalEntry {
            id: 0,
            timestamp: ts,
            event_id,
            phase: Phase::Social as u8,
            actor: Some(cohort),
            decision_type: DECISION_STRIKE_TRIGGERED,
            modified_values: vec![
                Modification { field: "strike_index".into(), old_value: 0, new_value: si as i64 },
                Modification { field: "draw_index".into(), old_value: 0, new_value: di as i64 },
                Modification { field: "g_qmicro".into(), old_value: 0, new_value: g },
                Modification { field: "lambda_mille_per_year".into(), old_value: 0, new_value: lambda_mille as i64 },
                Modification { field: "threshold_scaled".into(), old_value: 0, new_value: thr as i64 },
                Modification { field: "planned_duration_days".into(), old_value: 0, new_value: dur },
            ],
            causes: causes_ids,
            learned_by: vec![(cohort, ts.day as u64)],
        };
        let id = journal.add_entry(entry)?;
        // Tampon vidé au déclenchement de la grève (spec §5.1).
        if let Some(st) = self.cohorts.get_mut(&cohort) {
            st.causes.clear();
        }
        Some(id)
    }

    /// Journalise un `StrikeEnded` en `Phase::Social` (DECISION_STRIKE_ENDED = 81).
    pub fn journal_strike_ended(
        &mut self,
        cohort: EntityId,
        ts: Timestamp,
        event_id: u64,
        strike_index: u64,
        cause: StrikeEndCause,
        journal: &mut CausalJournal,
    ) -> Option<u64> {
        let cause_code = match cause {
            StrikeEndCause::Concession => 1,
            StrikeEndCause::DurationElapsed => 2,
        };
        let entry = JournalEntry {
            id: 0,
            timestamp: ts,
            event_id,
            phase: Phase::Social as u8,
            actor: Some(cohort),
            decision_type: DECISION_STRIKE_ENDED,
            modified_values: vec![
                Modification { field: "strike_index".into(), old_value: 0, new_value: strike_index as i64 },
                Modification { field: "end_cause".into(), old_value: 0, new_value: cause_code },
            ],
            causes: vec![],
            learned_by: vec![(cohort, ts.day as u64)],
        };
        journal.add_entry(entry)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tranche B1 — Commit 1 : calcul pur du risque cumulé (spec §3, §6.1.1)
    // ------------------------------------------------------------------------
    
    /// B1.1 - stable_u64 : déterminisme + bijection 64 bits en draw_index
    #[test]
    fn test_b1_stable_u64_determinisme_bijection() {
        for di in 0..64u64 {
            assert_eq!(
                stable_u64(42, 7, TAG_GREVE_SEUIL, di),
                stable_u64(42, 7, TAG_GREVE_SEUIL, di),
                "déterminisme pour draw_index={di}"
            );
        }
        // Bijection : 4096 indices distincts -> 4096 sorties distinctes
        let mut set = std::collections::BTreeSet::new();
        for di in 0..4096u64 {
            set.insert(stable_u64(42, 7, TAG_GREVE_SEUIL, di));
        }
        assert_eq!(set.len(), 4096, "bijection en draw_index");
        // Changer n'importe quel champ change la sortie (échantillon)
        let base = stable_u64(42, 7, TAG_GREVE_SEUIL, 3);
        assert_ne!(stable_u64(43, 7, TAG_GREVE_SEUIL, 3), base, "champ world_seed");
        assert_ne!(stable_u64(42, 8, TAG_GREVE_SEUIL, 3), base, "champ cohort_id");
        assert_ne!(stable_u64(42, 7, TAG_GREVE_DUREE, 3), base, "champ domain_tag");
        assert_ne!(stable_u64(42, 7, TAG_GREVE_SEUIL, 4), base, "champ draw_index");
    }
    
    /// B1.2 - Table LN1P_TABLE : bornes et cohérence du dénominateur analytique
    #[test]
    fn test_b1_table_ln1p_et_denom() {
        assert_eq!(LN1P_TABLE.len(), 257);
        assert_eq!(LN1P_TABLE[0], 0);
        assert_eq!(LN1P_TABLE[256], LN2_SCALED);
        assert!(LN1P_TABLE.windows(2).all(|w| w[0] < w[1]), "table strictement croissante");
        // HAZARD_INTEGRAL_DENOM = (3 · 1460 · 10^6 · M²) / 10^12 (division exacte)
        let m = MAX_GRIEFS_QMICRO as u128;
        assert_eq!(
            3u128 * SLOTS_PER_YEAR as u128 * 1_000_000u128 * m * m / HAZARD_SCALE,
            HAZARD_INTEGRAL_DENOM
        );
        assert_eq!(SLOTS_PER_YEAR, SLOTS_PER_DAY * 365);
    }
    
    /// B1.3 - Seuil exponentiel : points de contrôle connus (référence f64
    /// précalculée hors moteur : -ln(u)·10^12 avec u = raw_65/2^65).
    /// Tolérance 3·10^6 picorisques = erreur max de l'interpolation linéaire
    /// de la table à 257 points (courbure de ln).
    #[test]
    fn test_b1_seuil_exponentiel_points_controle() {
        // (world_seed, cohort_id, draw_index, E_scaled attendu, ref f64 ·10^12)
        let cas: [(u64, u64, u64, u64, u64); 8] = [
            (42, 1, 0, 738_703_545_811, 738_703_213_523),
            (42, 1, 1, 453_646_689_345, 453_645_742_593),
            (42, 2, 0, 1_444_547_145_494, 1_444_547_089_909),
            (7, 3, 5, 16_143_963_589, 16_143_649_108),
            (123_456_789, 4, 99, 941_086_059_217, 941_085_291_020),
            (0, 0, 0, 1_677_463_428_195, 1_677_462_665_527),
            (42, 1, 2, 480_744_131_286, 480_742_917_053),
            (999, 7, 42, 171_057_042_956, 171_056_371_606),
        ];
        for (ws, cid, di, attendu, ref_f64) in cas {
            let e = draw_exponential_threshold_scaled(ws, cid, di);
            assert_eq!(e, attendu, "E_scaled exact (algorithme entier) pour ({ws},{cid},{di})");
            let diff = e.abs_diff(ref_f64 as u64);
            assert!(diff <= 3_000_000, "précision -ln(u) pour ({ws},{cid},{di}) : |{e} - {ref_f64}| = {diff}");
            assert!(e >= 1, "E_scaled >= 1 garanti");
        }
        // Cas limite raw == 0 : k = 65, E = 65 · LN2_SCALED
        assert_eq!(e_scaled_from_raw(0), 65 * LN2_SCALED);
        // Positivité sur un grand échantillon
        for di in 0..10_000u64 {
            assert!(draw_exponential_threshold_scaled(42, 1, di) >= 1);
        }
    }
    
    /// B1.4 - delta_h_scaled : plateau (s = 0), butées et cas nuls
    #[test]
    fn test_b1_delta_h_plateau_et_cas_nuls() {
        let o = 400u64;
        let g0 = 200_000_000i64;
        // s = 0 : I(k) = 3·k·g0², arrondi = troncature entière
        for k in [1u64, 2, 10, 1_460, 100_000] {
            let attendu = (LAMBDA_MAX_MILLE as u128 * o as u128 * 3u128)
                .saturating_mul((g0 as u128).pow(2))
                .saturating_mul(k as u128) / HAZARD_INTEGRAL_DENOM;
            assert_eq!(delta_h_scaled(g0, 0, o, k), attendu, "plateau k={k}");
        }
        // Plateau à M, y compris avec pente positive depuis M (butée atteinte)
        let m = MAX_GRIEFS_QMICRO;
        for k in [1u64, 5, 1_460] {
            let attendu = (LAMBDA_MAX_MILLE as u128 * o as u128 * 3u128)
                .saturating_mul((m as u128).pow(2))
                .saturating_mul(k as u128) / HAZARD_INTEGRAL_DENOM;
            assert_eq!(delta_h_scaled(m, 0, o, k), attendu, "plateau M k={k}");
            assert_eq!(delta_h_scaled(m, 1_000, o, k), attendu, "butée M + pente k={k}");
        }
        // Cas nuls : g0 = 0 avec s <= 0, ou o = 0
        assert_eq!(delta_h_scaled(0, 0, o, 100), 0);
        assert_eq!(delta_h_scaled(0, -5, o, 100), 0);
        assert_eq!(delta_h_scaled(g0, 100, 0, 100), 0);
        assert_eq!(delta_h_scaled(g0, 0, o, 0), 0);
    }
    
    /// B1.5 - I(k) sur segment linéaire et butées 0 / M
    #[test]
    fn test_b1_integrale_segment_et_butees() {
        let m = MAX_GRIEFS_QMICRO;
        // Segment s > 0 sans butée : I(k) = k·(g0² + g0·g1 + g1²)
        let (g0, s) = (100_000_000i64, 50_000i64);
        let k_full = ((m - g0) / s) as u64;
        for k in [1u64, 2, 100, k_full] {
            let g1 = g0 + k as i64 * s;
            let attendu = k as u128
                * ((g0 as u128).pow(2) + (g0 as u128) * (g1 as u128) + (g1 as u128).pow(2));
            assert_eq!(integral_g2_x3(g0, s, k), attendu, "segment k={k}");
        }
        // Butée M : créneau final écrêté + plateau 3·M²/créneau ensuite
        let k_bound = k_full + 1;
        let g_full = g0 + k_full as i64 * s;
        let i_full = k_full as u128
            * ((g0 as u128).pow(2) + (g0 as u128) * (g_full as u128) + (g_full as u128).pow(2));
        let partial = (g_full as u128).pow(2) + (g_full as u128) * (m as u128) + (m as u128).pow(2);
        assert_eq!(integral_g2_x3(g0, s, k_bound), i_full + partial, "butée M k_bound");
        assert_eq!(
            integral_g2_x3(g0, s, k_bound + 7),
            i_full + partial + 3u128 * 7u128 * (m as u128).pow(2),
            "plateau après butée M"
        );
        // Butée 0 (s < 0) : contribution nulle après k_bound
        let (g0b, sb) = (100_000_000i64, -30_000i64);
        let k_full_b = (g0b / -sb) as u64;
        let k_bound_b = k_full_b + 1;
        let g_full_b = g0b + k_full_b as i64 * sb;
        let i_full_b = k_full_b as u128
            * ((g0b as u128).pow(2) + (g0b as u128) * (g_full_b as u128) + (g_full_b as u128).pow(2));
        let partial_b = (g_full_b as u128).pow(2);
        assert_eq!(integral_g2_x3(g0b, sb, k_bound_b), i_full_b + partial_b, "butée 0 k_bound");
        assert_eq!(integral_g2_x3(g0b, sb, k_bound_b + 100), i_full_b + partial_b, "nul après butée 0");
        // Monotonie de delta_h_scaled en k
        let mut prec = 0u128;
        for k in 1..=2_000u64 {
            let d = delta_h_scaled(50_000_000, 1_000, 600, k);
            assert!(d >= prec, "monotonie k={k}");
            prec = d;
        }
    }
    
    /// B1.6 - find_crossing_slot : cas sans franchissement possible
    #[test]
    fn test_b1_find_crossing_impossible() {
        assert_eq!(find_crossing_slot(0, -5, 600, 1_000), None);
        assert_eq!(find_crossing_slot(0, 0, 600, 1_000), None);
        assert_eq!(find_crossing_slot(100, 5, 0, 1_000), None, "o_mille = 0");
    }
    
    /// B1.7 - find_crossing_slot sur plateau (s = 0) : formule fermée exacte,
    /// vérifiée contre une recherche linéaire force brute.
    #[test]
    fn test_b1_find_crossing_plateau_formule_fermee() {
        let (g0, o, h_rem) = (123_456_789i64, 400u64, 10_000_000_000u128);
        match find_crossing_slot(g0, 0, o, h_rem) {
            Some(CrossingSlot::StrikeAtSlot(k)) => {
                assert!(delta_h_scaled(g0, 0, o, k) >= h_rem, "delta(k) >= H_rem");
                assert!(k == 1 || delta_h_scaled(g0, 0, o, k - 1) < h_rem, "delta(k-1) < H_rem");
                let mut kb = 1u64;
                while delta_h_scaled(g0, 0, o, kb) < h_rem {
                    kb += 1;
                }
                assert_eq!(k, kb, "formule fermée = force brute");
            }
            other => panic!("franchissement attendu sur plateau, obtenu {other:?}"),
        }
    }
    
    /// B1.8 - find_crossing_slot : exactitude de la bissection (delta(k) >= H_rem
    /// et delta(k-1) < H_rem) et butées 0 / M.
    #[test]
    fn test_b1_find_crossing_bissection_et_butees() {
        let m = MAX_GRIEFS_QMICRO;
        // s > 0 avec franchissement avant la butée (delta(k_bound=1522) ≈ 4,76e11 > h_rem)
        let (g0, s, o, h_rem) = (50_000_000i64, 230_000i64, 600u64, 100_000_000u128);
        match find_crossing_slot(g0, s, o, h_rem) {
            Some(CrossingSlot::StrikeAtSlot(k)) => {
                assert!(delta_h_scaled(g0, s, o, k) >= h_rem, "delta(k) >= H_rem");
                assert!(k == 1 || delta_h_scaled(g0, s, o, k - 1) < h_rem, "delta(k-1) < H_rem");
            }
            other => panic!("franchissement attendu, obtenu {other:?}"),
        }
        // s > 0 sans franchissement avant M -> BoundReached(M)
        match find_crossing_slot(g0, s, o, u128::MAX / 2) {
            Some(CrossingSlot::BoundReached { bound_qmicro, .. }) => assert_eq!(bound_qmicro, m),
            other => panic!("butée M attendue, obtenu {other:?}"),
        }
        // s < 0 avec franchissement avant G = 0 (delta(k_bound=6667) ≈ 3,04e11 > hb)
        let (g0b, sb, ob, hb) = (200_000_000i64, -30_000i64, 400u64, 200_000_000u128);
        match find_crossing_slot(g0b, sb, ob, hb) {
            Some(CrossingSlot::StrikeAtSlot(k)) => {
                assert!(delta_h_scaled(g0b, sb, ob, k) >= hb, "delta(k) >= H_rem");
                assert!(k == 1 || delta_h_scaled(g0b, sb, ob, k - 1) < hb, "delta(k-1) < H_rem");
            }
            other => panic!("franchissement attendu, obtenu {other:?}"),
        }
        // s < 0 sans franchissement avant G = 0 -> BoundReached(0)
        match find_crossing_slot(g0b, sb, ob, u128::MAX / 2) {
            Some(CrossingSlot::BoundReached { bound_qmicro, k }) => {
                assert_eq!(bound_qmicro, 0);
                assert!(delta_h_scaled(g0b, sb, ob, k) < u128::MAX / 2);
            }
            other => panic!("butée 0 attendue, obtenu {other:?}"),
        }
        // Cohérence bisection sur 50 cas déterministes variés
        for i in 0..50u64 {
            let g0c = (stable_u64(7, i, 1, 0) % (m as u64)) as i64;
            let sc = (stable_u64(7, i, 2, 0) % 260_000) as i64 - 30_000;
            let oc = [200u64, 400, 600][(i % 3) as usize];
            let h = (stable_u64(7, i, 3, 0) % 40_000_000_000_000) as u128 + 1;
            if let Some(CrossingSlot::StrikeAtSlot(k)) = find_crossing_slot(g0c, sc, oc, h) {
                assert!(delta_h_scaled(g0c, sc, oc, k) >= h, "cas {i} : delta(k) >= H_rem");
                assert!(
                    k == 1 || delta_h_scaled(g0c, sc, oc, k - 1) < h,
                    "cas {i} : delta(k-1) < H_rem"
                );
            }
        }
    }
    
    /// B1.10 (bonus relecture Agent 2) - Butée haute avec pente positive NON divisible :
    /// distance = M - g0 non multiple de s, k_full tronqué, créneau final écrêté + plateau.
    #[test]
    fn test_b1_integrale_butee_haute_non_divisible() {
        let m = MAX_GRIEFS_QMICRO;
        let (g0, s) = (100_000_001i64, 50_000i64);
        let distance = m - g0; // 299_999_999, non divisible par 50_000
        assert_eq!(distance % s, 49_999);
        let k_full = (distance / s) as u64; // 5999 (troncature)
        let k_bound = k_full + 1; // 6000 (plafond)
        let g_full = g0 + k_full as i64 * s; // 399_950_001
        let i_full = k_full as u128
            * ((g0 as u128).pow(2) + (g0 as u128) * (g_full as u128) + (g_full as u128).pow(2));
        let partial = (g_full as u128).pow(2) + (g_full as u128) * (m as u128) + (m as u128).pow(2);
        assert_eq!(integral_g2_x3(g0, s, k_bound), i_full + partial, "créneau final écrêté");
        assert_eq!(
            integral_g2_x3(g0, s, k_bound + 3),
            i_full + partial + 3u128 * 3u128 * (m as u128).pow(2),
            "plateau M après butée"
        );
        // Cohérence de delta_h_scaled avec I
        let o = 600u64;
        assert_eq!(
            delta_h_scaled(g0, s, o, k_bound),
            (LAMBDA_MAX_MILLE as u128 * o as u128 * (i_full + partial)) / HAZARD_INTEGRAL_DENOM
        );
    }

    /// B1.11 - Tampon top-5 des causes : fusion par entry_id, tri delta DESC puis id ASC,
    /// troncature à 5 (spec §5.1, arbitrage 5).
    #[test]
    fn test_b1_tampon_causes_top5() {
        let mut st = CohortStrikeState::dormant(Timestamp::new(0, 0));
        let contributions = [
            (10u64, 500u128),
            (11, 900),
            (12, 100),
            (13, 900), // même delta que 11 -> id ASC
            (14, 2_000),
            (15, 50),
            (16, 700),
        ];
        for (eid, delta) in contributions {
            add_cause(&mut st, eid, delta);
        }
        assert_eq!(st.causes.len(), 5, "tronqué à 5");
        // attendu : 14 (2000), 11 (900), 13 (900), 16 (700), 10 (500)
        let ids: Vec<u64> = st.causes.iter().map(|c| c.journal_entry_id).collect();
        assert_eq!(ids, vec![14, 11, 13, 16, 10], "tri delta DESC puis id ASC");
        // fusion : même entry_id -> somme
        add_cause(&mut st, 10, 10_000);
        let pos = st.causes.iter().position(|c| c.journal_entry_id == 10).unwrap();
        assert_eq!(st.causes[pos].delta_h_scaled, 10_500, "fusion même entry_id");
        assert_eq!(st.causes[0].journal_entry_id, 10, "10 passe en tête (10500 > 2000)");
        assert_eq!(st.causes.len(), 5, "toujours tronqué à 5");
        // delta nul ignoré
        add_cause(&mut st, 99, 0);
        assert!(!st.causes.iter().any(|c| c.journal_entry_id == 99));
    }

    /// B1.12 - Saturation de add_slots / slots_between : aucun débordement i64,
    /// jamais d'instant passé ou négatif, même avec k = u64::MAX.
    #[test]
    fn test_b1_add_slots_saturation_sans_debordement() {
        let ts = Timestamp::new(100, 2);
        let s1 = add_slots(ts, u64::MAX);
        assert!(s1 > ts, "add_slots(u64::MAX) reste futur");
        assert!(s1.day >= 100 && s1.seq < 4, "jour >= 100, seq < 4 : {:?}", s1);
        let s2 = add_slots(ts, i64::MAX as u64 + 1);
        assert!(s2 > ts && s2.day >= 100 && s2.seq < 4, "add_slots(i64::MAX+1) saturé : {:?}", s2);
        // find_crossing_slot à G faible + plateau + H_rem = HAZARD_SCALE -> k géant,
        // add_slots doit rester futur (jamais négatif ni passé).
        if let Some(CrossingSlot::StrikeAtSlot(k)) = find_crossing_slot(1, 0, 200, HAZARD_SCALE) {
            let t = add_slots(Timestamp::new(100, 0), k);
            assert!(t > Timestamp::new(100, 0), "toujours futur, jamais passé : {:?}", t);
            assert!(t.day >= 0);
        }
        // slots_between ne panique pas avec le timestamp saturé
        let _ = slots_between(Timestamp::new(0, 0), add_slots(Timestamp::new(100, 0), u64::MAX));
        // aller-retour cohérent pour les petites valeurs
        assert_eq!(slots_between(Timestamp::new(3, 1), Timestamp::new(4, 1)), 4);
        assert_eq!(slots_between(Timestamp::new(4, 1), Timestamp::new(3, 1)), 0);
    }

    /// B1.9 - Durée de grève : dans [1, 1 + floor(20·O)], bornes atteintes,
    /// déterministe. Durées max [5, 9, 13, 13, 9] pour DEFAULT_ORGANISATION_MILLE.
    #[test]
    fn test_b1_duree_greve_bornes() {
        let max_attendus = [5u64, 9, 13, 13, 9];
        for (i, o) in DEFAULT_ORGANISATION_MILLE.iter().enumerate() {
            let maxd = max_attendus[i];
            assert_eq!(1 + (20 * u64::from(*o)) / 1_000, maxd, "max_duration O={o}");
            let mut min_d = i64::MAX;
            let mut max_d = 0i64;
            for di in 0..2_000u64 {
                let d = draw_strike_duration_days(42, 1, *o, di);
                assert!((1..=maxd as i64).contains(&d), "durée {d} dans [1, {maxd}] pour O={o}");
                min_d = min_d.min(d);
                max_d = max_d.max(d);
            }
            assert_eq!(min_d, 1, "borne inf atteinte pour O={o}");
            assert_eq!(max_d, maxd as i64, "borne sup atteinte pour O={o}");
            assert_eq!(
                draw_strike_duration_days(42, 1, *o, 7),
                draw_strike_duration_days(42, 1, *o, 7),
                "déterminisme durée"
            );
        }
    }
}
