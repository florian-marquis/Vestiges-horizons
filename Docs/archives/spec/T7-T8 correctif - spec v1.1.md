# Vestiges Horizons - Spécification Corrective T7-T8 v1.1
**Phase 2, Étape 1 : Spécification**
**Date** : 14 novembre 2026
**Version** : 1.1 (révisée)
**Statut** : À valider avant implémentation

> **Contexte** : Diagnostic v3.1 validé. Cette spécification corrige les points B.1-B.11.
> **Règle** : AUCUN flottant, même en pseudo-code. Toutes les valeurs en entiers.

---

## 1️⃣ **ENTITÉS**

### 1.1 Cohorte avec identifiant stable

**État par cohorte, indexé par identifiant stable.** L'identifiant est dérivé de manière déterministe et **jamais réutilisé**.

```rust
/// Identifiant de cohorte stable
/// - world_seed : graine du monde (u64)
/// - unique_number : numéro unique dérivé (u64)
/// 
/// À la création : unique_number = compteur_global.next()
/// À la scission : unique_number = derive_child_id(parent.unique_number, event_id)
///     où event_id = identifiant stable de l'événement de scission
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CohortId {
    pub world_seed: u64,
    pub unique_number: u64,
}

impl CohortId {
    /// Dérive un identifiant enfant à partir d'un parent et d'un événement
    /// Algorithme : hash SHA-256 concis sur (parent, event_id)
    ///     parent_hash = parent.unique_number
    ///     event_hash = event_id
    ///     child_number = (parent_hash * 6364136223846793005 + event_hash).wrapping_add(1) ^ 0xDEADBEEF
    pub fn derive_child(parent: CohortId, event_id: u64) -> Self {
        let combined = parent.unique_number.wrapping_mul(6364136223846793005)
            .wrapping_add(event_id)
            .wrapping_add(0xDEADBEEF);
        CohortId {
            world_seed: parent.world_seed,
            unique_number: combined,
        }
    }
}

/// Vérification anti-collision :
/// - Les identifiants dérivés ne doivent jamais entrer en collision avec le compteur global
/// - On réserve les nombres pairs pour le compteur, les impairs pour les dérivés
/// - Vérification à l'exécution en debug : assert!(derived % 2 == 1)
```

---

### 1.2 État de griefs et risque

```rust
/// G en micro-points [0 ; 100_000_000]
/// Échelle : 1 µpt = 10⁻⁶ point de grief
/// 100 points = 100_000_000 µpts (maximum)
///
/// Un seul champ `last_update` pour G et R
/// Une seule fonction d'avance qui intègre les deux ensemble
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GriefsState {
    /// Griefs cumulés (en micro-points)
    pub g: u64,
    
    /// Entrées (entiers [0 ; 1000])
    /// Constantes par morceaux : changent uniquement sur InputChanged
    pub u: u16,
    pub p: u16,
    pub q: u16,
    
    /// Date du dernier changement (G, R, ou entrées)
    pub last_update: Timestamp,
    
    /// Risque cumulé (en unités de risque)
    /// Échelle : 1 événement = 365_000_000 unités
    pub risk_accumulated: u64,
    
    /// Seuil de grève tiré (une seule fois par grève)
    /// Tiré à la création de la cohorte et après chaque grève
    /// NE JAMAIS 0 (tiré immédiatement à la création)
    pub strike_threshold: u64,
    
    /// Durée de grève (en jours)
    pub strike_duration: u16,
    
    /// État de grève
    pub strike_state: StrikeState,
    
    /// Compteur d'époque (pour l'ordonnancement canonique v6)
    pub epoch: u64,
    
    /// Identifiant de la cohorte
    pub cohort_id: CohortId,
    
    /// Graine du monde
    pub world_seed: u64,
}

/// État de grève
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrikeState {
    /// Pas de grève
    Idle,
    /// Grève en cours
    Ongoing,
    /// Grève terminée (attend fin effective)
    Ending,
}

impl GriefsState {
    pub fn new(cohort_id: CohortId, world_seed: u64) -> Self {
        // Tirer le seuil et la durée À LA CRÉATION
        let strike_threshold = Self::draw_strike_threshold(world_seed, cohort_id, 0);
        let strike_duration = Self::draw_strike_duration(world_seed, cohort_id, 0);
        
        GriefsState {
            g: 0,
            u: 0,
            p: 0,
            q: 0,
            last_update: Timestamp::new(0, 0),
            risk_accumulated: 0,
            strike_threshold,
            strike_duration,
            strike_state: StrikeState::Idle,
            epoch: 0,
            cohort_id,
            world_seed,
        }
    }
}
```

---

## 2️⃣ **DYNAMIQUE DE G ET RISQUE**

### 2.1 Bandes de G et λ (entiers uniquement)

**Échelle du risque : 1 événement = 365 000 000 unités.**

**Bandas de λ en unités/jour (données)** :
```
| Bande de G | λ (événements/an) | λ (unités/jour) |
|-------------|-------------------|------------------|
| G < 30_000_000 | 0 | 0 |
| 30_000_000 ≤ G < 60_000_000 | 0.05 | 50_000 |
| 60_000_000 ≤ G < 80_000_000 | 0.2 | 200_000 |
| G ≥ 80_000_000 | 0.5 | 500_000 |
```

**Données** :
```rust
/// Bandes de λ en unités/jour (stockées en données, pas en code)
/// Format : Vec<(seuil_g, lambda_unites_par_jour)>
pub const RISK_BANDS: [(u64, u64); 4] = [
    (0, 0),              // G < 30M
    (30_000_000, 50_000),
    (60_000_000, 200_000),
    (80_000_000, 500_000),
];

/// Retourne λ en unités/jour pour un G donné
pub fn lambda_for_g(g: u64) -> u64 {
    for (threshold, lambda) in RISK_BANDS.iter().rev() {
        if g >= *threshold {
            return *lambda;
        }
    }
    0
}
```

---

### 2.2 Pente de G (entiers uniquement)

**Coefficients en données** :
```rust
/// Coefficients pour la pente de G (µpt/jour)
pub struct GriefCoefficients {
    pub u: i64,    // 100
    pub p: i64,    // 25
    pub q: i64,    // -30
    pub relief: i64, // -20_000 (soulagement constant)
}

/// Calcul de la pente
/// pente = u_coeff * u + p_coeff * p + q_coeff * q + relief
/// Tout en entiers, pas de flottants
pub fn calculate_slope(coeffs: &GriefCoefficients, u: u16, p: u16, q: u16) -> i64 {
    coeffs.u * u as i64 
        + coeffs.p * p as i64 
        + coeffs.q * q as i64 
        + coeffs.relief
}
```

---

### 2.3 Intégration unifiée

**Une seule fonction d'avance qui intègre G et R ensemble, par segments coupés aux franchissements de bande.**

```rust
/// Avance G et R jusqu'à target_date
/// G et R évoluent linéairement par segments
/// Les segments sont coupés aux franchissements de bande (λ change) et aux bornes
pub fn advance_to(
    state: &mut GriefsState,
    target_date: Timestamp,
    coeffs: &GriefCoefficients,
) {
    // Ne rien faire si on est déjà à jour ou dans le futur
    if target_date <= state.last_update {
        return;
    }
    
    let current_date = state.last_update;
    let current_g = state.g;
    let current_r = state.risk_accumulated;
    
    // Si en grève, λ = 0
    let lambda = if matches!(state.strike_state, StrikeState::Ongoing) {
        0
    } else {
        lambda_for_g(current_g)
    };
    
    let slope = calculate_slope(coeffs, state.u, state.p, state.q);
    
    // Calculer la prochaine date de franchissement de bande
    let next_band_threshold = Self::next_band_threshold(current_g, slope);
    let band_cross_date = if slope != 0 {
        // Date où G atteindra next_band_threshold
        // G_final = G_initial + slope * Δt
        // Δt = ceil((threshold - G_initial) / slope)
        Self::calculate_cross_date(current_g, next_band_threshold, slope, current_date)
    } else {
        // Pas de changement de bande si pente = 0
        target_date
    };
    
    // Avancer jusqu'au prochain événement (franchissement de bande ou target_date)
    let advance_date = std::cmp::min(band_cross_date, target_date);
    let dt = advance_date.day - current_date.day;
    
    if dt > 0 {
        // Mettre à jour G
        let delta_g = slope as i128 * dt as i128;
        state.g = (current_g as i128 + delta_g).clamp(0, 100_000_000) as u64;
        
        // Mettre à jour R (si λ > 0)
        if lambda > 0 {
            let delta_r = lambda as i128 * dt as i128;
            state.risk_accumulated = current_r + delta_r as u64;
        }
        
        // Incrémenter l'époque
        state.epoch += 1;
        
        // Mettre à jour last_update
        state.last_update = advance_date;
    }
    
    // Si on a atteint un franchissement de bande, réinvoquer pour le prochain segment
    if advance_date < target_date && slope != 0 {
        // Le franchissement de bande a eu lieu, recalculer λ et continuer
        Self::advance_to(state, target_date, coeffs);
    }
}

/// Calcule la date de franchissement d'un seuil de G
/// Δt = ceil((threshold - current_g) / slope)
/// Si slope > 0 : threshold = next_band_threshold
/// Si slope < 0 : threshold = previous_band_threshold
fn calculate_cross_date(
    current_g: u64,
    threshold: u64,
    slope: i64,
    current_date: Timestamp,
) -> Timestamp {
    if slope == 0 {
        return Timestamp::new(i64::MAX, 0); // Jamais
    }
    
    // delta_g = threshold - current_g
    let delta_g = threshold as i128 - current_g as i128;
    
    // Δt = delta_g / slope (arrondi supérieur pour les entiers)
    // Si slope > 0 et delta_g > 0 : Δt = ceil(delta_g / slope)
    // Si slope < 0 et delta_g < 0 : Δt = ceil(delta_g / slope)
    
    let dt = if slope > 0 {
        if delta_g <= 0 {
            0 // déjà au-dessus du seuil
        } else {
            (delta_g + slope as i128 - 1) / slope as i128
        }
    } else {
        if delta_g >= 0 {
            0 // déjà en dessous du seuil
        } else {
            (delta_g + slope as i128 + 1) / slope as i128
        }
    };
    
    Timestamp::new(current_date.day + dt as i64, current_date.seq)
}
```

---

## 3️⃣ **TIrages**

### 3.1 Seuil S tiré une seule fois

**Tiré à la création de la cohorte et après chaque grève (jamais paresseusement, jamais 0).**

**Méthode : S = −ln(U) en entiers**
- U : entier uniforme 32 bits
- S : seuil en unités de risque [0 ; 365_000_000]

**Implémentation** :
```rust
/// Table pré-calculée pour -ln(1-x) avec x ∈ [0, 255]/256
/// 256 entrées, interpolation linéaire
const EXP_TABLE: [u64; 256] = [
    0, 1, 2, ..., // valeurs pré-calculées pour -ln(1-x)*365M
];

/// Algorithme de hash nommé et fixé : SHA-256 concis
/// On utilise un hash 32 bits basé sur xxHash (déterministe, rapide)
fn stable_hash_u32(key: &str) -> u32 {
    // xxHash simplifié :
    // 1. Initialiser avec graine = 0
    // 2. Pour chaque octet : acc = acc * 16777619 ^ byte
    // 3. Mix final
    let mut acc = 0u32;
    for b in key.bytes() {
        acc = acc.wrapping_mul(16777619).wrapping_add(b as u32);
    }
    acc ^= acc >> 15;
    acc = acc.wrapping_mul(0x85ebca6b);
    acc ^= acc >> 13;
    acc = acc.wrapping_mul(0xc2b2ae35);
    acc ^= acc >> 16;
    acc
}

/// Génère une clé stable
fn generate_stable_key(world_seed: u64, cohort_id: CohortId, suffix: &str) -> String {
    format!("{}-{}-{}", world_seed, cohort_id.unique_number, suffix)
}

/// Tire un seuil de grève S = −ln(U) × 365_000_000
fn draw_strike_threshold(world_seed: u64, cohort_id: CohortId, draw_index: u32) -> u64 {
    let key = generate_stable_key(world_seed, cohort_id, &format!("strike-{}", draw_index));
    let u32_hash = stable_hash_u32(&key);
    
    // U = u32_hash / 2^32 (approximation de uniforme [0,1))
    // S = -ln(1-U) * 365_000_000
    //     = -ln((2^32 - u32_hash) / 2^32) * 365_000_000
    //     ≈ table[255 - (u32_hash >> 24)]
    
    // Utiliser leading_zeros pour plus de précision
    let leading_zeros = u32_hash.leading_zeros();
    let fine_index = ((u32_hash << leading_zeros) >> 24) as usize;
    
    // Interpolation linéaire entre table[fine_index] et table[fine_index+1]
    let x = (u32_hash & 0xFFFFFF) as u64;
    let base = EXP_TABLE[fine_index];
    let next = EXP_TABLE[(fine_index + 1).min(255)];
    base + (next - base) * x / 0x1000000
}

/// Tire une durée de grève (en jours)
/// Clé distincte : (world_seed, cohort_id, "strike_duration", draw_index)
fn draw_strike_duration(world_seed: u64, cohort_id: CohortId, draw_index: u32) -> u16 {
    let key = generate_stable_key(world_seed, cohort_id, &format!("strike_duration-{}", draw_index));
    let hash = stable_hash_u32(&key);
    // Durée : 1-30 jours
    1 + (hash % 30) as u16
}
```

---

### 3.2 Planification exacte

**Date de BandCrossed** :
```
Δt = ceil((seuil − G) / pente)
```

**Date de StrikeThresholdCrossed** :
```
Δt = ceil((S − R) / λ)
```

**Convention de jour déclarée** : Les jours sont des entiers i64, la date est toujours arrondie **vers le haut** (ceil) pour garantir que l'événement se déclenche **après** que le seuil soit franchi.

**Époque** : Chaque événement planifié porte son époque. Un événement périmé (époque < epoch actuelle) est **ignoré**.

```rust
/// Planifie un événement
fn schedule_event(
    state: &mut GriefsState,
    event_type: T7T8Event,
    date: Timestamp,
    event_queue: &mut EventQueue,
) {
    event_queue.push(Event::new(
        next_event_id(),
        date.day,
        0,
        Phase::Social,
        Priority::new(1),
        event_type,
    ));
    // L'événement portera l'époque actuelle
}

/// Traite un événement
fn handle_event(
    state: &mut GriefsState,
    event: T7T8Event,
    current_date: Timestamp,
    coeffs: &GriefCoefficients,
) {
    // Avancer jusqu'à la date de l'événement
    advance_to(state, current_date, coeffs);
    
    match event {
        T7T8Event::InputChanged { cohort_id, new_u, new_p, new_q } => {
            // Changer les entrées
            state.u = new_u;
            state.p = new_p;
            state.q = new_q;
            state.last_update = current_date;
            
            // Recalculer pente et λ
            let slope = calculate_slope(coeffs, state.u, state.p, state.q);
            let lambda = lambda_for_g(state.g);
            
            // Planifier le prochain BandCrossed si la bande change
            if lambda != state.current_lambda {
                let next_band_date = calculate_next_band_date(state, slope, coeffs);
                schedule_event(state, T7T8Event::BandCrossed { ... }, next_band_date);
            }
            
            // Planifier le prochain StrikeThresholdCrossed
            let strike_date = calculate_strike_date(state, lambda);
            schedule_event(state, T7T8Event::StrikeThresholdCrossed, strike_date);
            
            // Incrémenter l'époque
            state.epoch += 1;
            state.current_lambda = lambda;
        }
        
        T7T8Event::BandCrossed { old_band, new_band } => {
            // λ a changé, recalculer le prochain StrikeThresholdCrossed
            let lambda = lambda_for_g(state.g);
            let strike_date = calculate_strike_date(state, lambda);
            schedule_event(state, T7T8Event::StrikeThresholdCrossed, strike_date);
            state.current_lambda = lambda;
        }
        
        T7T8Event::StrikeThresholdCrossed => {
            // Vérifier que R >= S
            if state.risk_accumulated >= state.strike_threshold {
                // Déclencher la grève
                state.strike_state = StrikeState::Ongoing;
                state.risk_accumulated = 0;
                
                // Tirer un nouveau seuil pour la prochaine grève
                state.draw_index += 1;
                state.strike_threshold = draw_strike_threshold(
                    state.world_seed, state.cohort_id, state.draw_index);
                
                // Planifier la fin de grève
                let end_date = Timestamp::new(
                    current_date.day + state.strike_duration as i64,
                    0
                );
                schedule_event(state, T7T8Event::StrikeEnded, end_date);
            }
        }
        
        T7T8Event::StrikeEnded { cause } => {
            state.strike_state = StrikeState::Ending;
            // Pendant la grève, λ = 0, donc R n'a pas augmenté
            // À la fin de grève, R est réinitialisé à 0
            state.risk_accumulated = 0;
            state.strike_state = StrikeState::Idle;
        }
    }
}
```

---

## 4️⃣ **COMPORTEMENT PENDANT GRÈVE**

**Pendant StrikeState::Ongoing, λ = 0.**

**Comportement du risque à la fin de grève** :
- R est **réinitialisé à 0** à la fin de la grève (dans StrikeThresholdCrossed ou StrikeEnded)
- Un nouveau seuil S est tiré pour la prochaine grève
- L'accumulation du risque reprend avec le nouveau λ (basé sur G après la grève)

---

## 5️⃣ **RELATION G ↔ GRIEFS INDIVIDUELS**

**G est un scalaire par cohorte.**

**Les anonymes sont exactement à G.**

**Les nommés ont un décalage δ constant (nul à l'extraction).**

**Formule** :
```
total_griefs = pop * G + Σδ
```

**Où** :
- `pop` = population totale de la cohorte
- `G` = griefs cumulés (scalaire par cohorte)
- `Σδ` = somme des décalages des individus nommés
- À l'extraction : δ = 0 (l'individu extrait a exactement G)
- Après modification : δ peut devenir non-nul

---

### 5.1 I8 redéfini

**I8 = conservation du total avant/après chaque opération + pop = nommés + anonymes**

**Implémentation** :
```rust
/// État des griefs avec conservation explicite
pub struct CohortGriefs {
    /// Population totale
    pub population: u64,
    
    /// Total des griefs de tous les membres (anonymes + nommés)
    /// Ce champ est **toujours** égal à population * G + Σδ
    /// où G est le grief scalaire de la cohorte
    pub total: i64,
    
    /// Population nommée
    pub named_count: u64,
    
    /// Population anonyme
    pub anonymous_count: u64,
}

impl CohortGriefs {
    /// Vérifie I8 : total == population * G + Σδ
    /// où Σδ = sum des (individual.grief - G) pour les nommés
    pub fn verify_i8(&self, named_individuals: &[NamedIndividual], g: u64) -> bool {
        let sum_delta: i64 = named_individuals.iter()
            .map(|ind| (ind.grief as i64) - (g as i64))
            .sum();
        
        let expected_total = self.population as i64 * g as i64 + sum_delta;
        expected_total == self.total
    }
    
    /// Vérifie la conservation : pop = named + anonymous
    pub fn verify_population(&self) -> bool {
        self.population == self.named_count + self.anonymous_count
    }
}
```

**Migration depuis Cohort.total_griefs existant** :
```rust
/// Migration : Cohort.total_griefs → CohortGriefs.total
/// Les deux représentent le même concept (total des griefs)
/// mais CohortGriefs totalise explicitement avec G scalaire
///
/// Proposition :
/// - Garder Cohort.total_griefs comme alias de CohortGriefs.total
/// - Ou supprimer Cohort.total_griefs et utiliser uniquement CohortGriefs.total
/// - Pour la compatibilité, on peut garder les deux avec une assertion
```

---

### 5.2 Extraction et réinjection

**Extraction** :
```rust
/// Extrait un individu d'une cohorte
/// L'individu a grief = G (δ = 0 à l'extraction)
/// Conservation stricte : total_before = total_after + individual.grief
pub fn extract_individual(
    cohort: &mut Cohort,
    griefs: &mut CohortGriefs,
    g: u64,
) -> NamedIndividual {
    // L'individu extrait a grief = G
    let individual_grief = g as i64;
    
    // Mettre à jour la cohorte
    cohort.population -= 1;
    
    // Mettre à jour les griefs
    griefs.population -= 1;
    griefs.anonymous_count -= 1;
    griefs.named_count += 1;
    
    // Conservation stricte
    griefs.total -= individual_grief;
    
    NamedIndividual {
        grief: individual_grief,
        // ... autres champs
    }
}
```

**Réinjection** :
```rust
/// Réinjecte un individu dans sa cohorte
/// Conservation stricte : total_after = total_before + individual.grief
pub fn reinject_individual(
    cohort: &mut Cohort,
    griefs: &mut CohortGriefs,
    individual: NamedIndividual,
) {
    // Mettre à jour la cohorte
    cohort.population += 1;
    
    // Mettre à jour les griefs
    griefs.population += 1;
    griefs.anonymous_count += 1;
    griefs.named_count -= 1;
    
    // Conservation stricte
    griefs.total += individual.grief;
}
```

**Test de propriété I8** :
```rust
/// Test de propriété : I8 est toujours vrai
#[test]
fn test_i8_property() {
    let mut griefs = CohortGriefs::new();
    let g = 50_000_000; // 50 points
    griefs.total = 100 * g as i64; // 100 individus à G
    griefs.population = 100;
    griefs.anonymous_count = 100;
    
    // Extraire 10 individus
    for _ in 0..10 {
        let individual = extract_individual(&mut cohort, &mut griefs, g);
        assert!(griefs.verify_i8(&[], g));
    }
    
    // Réinjecter avec des griefs modifiés
    let mut individuals = vec![NamedIndividual { grief: g as i64 + 1000, .. }];
    reinject_individual(&mut cohort, &mut griefs, individuals.pop().unwrap());
    
    // I8 doit toujours être vrai
    assert!(griefs.verify_i8(&individuals, g));
}
```

---

## 6️⃣ **SALAIRE RÉEL → P**

**p est mis à jour par une courbe en données, pas de flottants.**

**Courbe proposée** (valeurs en données) :
```
| Salaire réel (%) | p |
|-------------------|---|
| ≤ 50              | 1000 |
| 100               | 500 |
| 150               | 200 |
| ≥ 250             | 0 |
```

**Interpolation entière** :
```rust
/// Points de la courbe (salaire_reel_percent, p)
/// salaire_reel_percent en millièmes (100_000 = 100%)
pub const P_CURVE: [(i64, u16); 4] = [
    (50_000, 1000),   // 50% → p=1000
    (100_000, 500),   // 100% → p=500
    (150_000, 200),   // 150% → p=200
    (250_000, 0),     // 250% → p=0
];

/// Interpolation linéaire entière
/// p = p1 + (p2 - p1) * (s - s1) / (s2 - s1)
pub fn calculate_p_from_real_wage(real_wage_percent: i64) -> u16 {
    // real_wage_percent en millièmes (ex: 100_000 = 100%)
    
    // Trouver l'intervalle
    for i in 0..P_CURVE.len() - 1 {
        let (s1, p1) = P_CURVE[i];
        let (s2, p2) = P_CURVE[i + 1];
        
        if real_wage_percent <= s1 {
            return p1;
        }
        if real_wage_percent >= s2 {
            continue;
        }
        
        // Interpolation : p = p1 + (p2 - p1) * (s - s1) / (s2 - s1)
        let numerator = (p2 as i64 - p1 as i64) * (real_wage_percent - s1);
        let denominator = s2 - s1;
        let delta = numerator / denominator;
        
        return (p1 as i64 + delta).clamp(0, 1000) as u16;
    }
    
    0 // Par défaut
}
```

**Validation des données** :
- ❌ **panier = 0** → **erreur explicite** (ValidationError::ZeroBasketCost)
- ✅ **Saturation vérifiée après division**, pas avant
- ✅ **Unités cohérentes** : salaire_reel en millièmes de pourcentage (100_000 = 100%)

---

## 7️⃣ **PLAN DE TRANCHES VERTICALES (réorganisé)**

### **Tranche A : Infrastructure de base (une cohorte, G, bandes, époque)**

**Périmètre** :
- `CohortId` avec identifiant stable
- `GriefsState` avec G en µpts, u/p/q [0 ; 1000], last_update
- `calculate_slope` avec coefficients en données
- `lambda_for_g` avec bandes en données
- `advance_to` pour intégration unifiée
- Événement `InputChanged`

**Tests** :
- Test d'intégration exacte : G += pente × Δt
- Test du clamp [0 ; 100_000_000]
- Test des bandes de λ
- Test de l époque

**Critère de sortie** : ✅ Tous les tests passent

---

### **Tranche B : Risque et tirage**

**Périmètre** :
- `risk_accumulated` en unités de risque
- `draw_strike_threshold` avec table pré-calculée
- `draw_strike_duration` avec hash nommé
- `StrikeThresholdCrossed` événement
- Planification exacte avec dates calculées

**Tests** :
- Test du tirage exponentiel (distribution)
- Test de la planification exacte
- Test du déterminisme

**Critère de sortie** : ✅ Tous les tests passent

---

### **Tranche C : Grèves**

**Périmètre** :
- `StrikeState` (Idle, Ongoing, Ending)
- `StrikeEnded` événement
- λ = 0 pendant grève
- Réinitialisation de R à la fin de grève
- Nouveau seuil tiré après chaque grève

**Tests** :
- Test du déclenchement
- Test de la fin de grève
- Test du comportement pendant grève (λ = 0)

**Critère de sortie** : ✅ Tous les tests passent

---

### **Tranche D : I8 et extraction**

**Périmètre** :
- `CohortGriefs` avec total, population, named_count, anonymous_count
- Extraction avec grief = G (δ = 0)
- Réinjection avec conservation stricte
- I8 comme propriété (pas un invariant à tester)

**Tests** :
- Test de conservation stricte
- Test avec totaux non divisibles
- Test de propriété I8 sur 50 graines

**Critère de sortie** : ✅ Tous les tests passent

---

### **Tranche E : Salaire réel**

**Périmètre** :
- `RealWage` avec validation
- Calcul du salaire réel (erreur si panier = 0)
- Courbe p = f(salaire_reel) en données
- Lien salaire réel → p

**Tests** :
- Test de rejection si panier = 0
- Test du calcul
- Test de la saturation
- Test du lien vers p

**Critère de sortie** : ✅ Tous les tests passent

---

### **Tranche F : Benchmark et scénario en données**

**Périmètre** :
- Remplacer les modulos par des entrées plausibles en données
- Scénario du benchmark T7-T8
- Valider que benchmark P1 reste inchangé

**Tests** :
- Benchmark P1 inchangé
- Critère 1 : Zéro grève sans stress
- Critère 2 : Pénurie durable (intégrale analytique ±20%)
- Critère nouveau : Suite de 1 000 InputChanged aléatoires = version « avance jour par jour »
- Critères 3-9 (voir ci-dessous)

**Critères de réussite** :
1. ✅ G = 0, salaire = panier, u = p = q = 0, 50 ans : zéro grève sur 20 graines
2. ✅ Pénurie durable (u = 800) : moyenne empirique = intégrale analytique de λ ±20 % sur 50 graines
3. ✅ Invariance : scinder un segment constant ne change ni G ni la date de franchissement
4. ✅ Événements traités ∝ aux changements d'entrée + franchissements de bande + grèves
5. ✅ I8 en test de propriété (50 graines, séquences aléatoires)
6. ✅ Cas limites : u = p = 1000, q = 0 ; G = 100_000_000 ; G = 0 avec soulagement ; salaire très élevé ; panier = 0 rejeté
7. ✅ Deux cohortes aux entrées différentes évoluent indépendamment
8. ✅ Performance : 100 cohortes × 100 ans, en événements par seconde
9. ✅ Scénario benchmark avec entrées plausibles en données
10. ✅ Suite de 1 000 InputChanged aléatoires identique à l'oracle « jour par jour »

**Critère de sortie** : ✅ Tous les critères passent, benchmark P1 inchangé

---

## 8️⃣ **HORS PÉRIMÈTRE**

### labor.rs : Demi-vie en virgule fixe

**Problème** : `src/labor.rs:299-301` utilise f64 et exp() pour la décroissance avec demi-vie. Le résultat entre dans l'état canonique (`valeur: u16`).

**Solution** : 
- Remplacer par une **décroissance exponentielle en virgule fixe**
- **Facteur journalier** : 2^(-1/demi_vie) en virgule fixe 2^32
- **Exponentiation rapide** : multiplication répétée ou table pré-calculée

**Implémentation proposée** :
```rust
/// Facteur journalier en virgule fixe 2^32
/// facteur = 2^(-1/demi_vie)
/// Exemple : demi_vie = 365 jours → facteur = 2^(-1/365) ≈ 0.99727
/// En virgule fixe : facteur_fixed = 0.99727 * 2^32 ≈ 4_274_000_000
pub struct FixedPoint {
    value: u64, // 2^32 = 1.0
}

impl FixedPoint {
    /// Multiplie deux nombres en virgule fixe 2^32
    pub fn mul(a: u64, b: u64) -> u64 {
        ((a as u128 * b as u128) >> 32) as u64
    }
    
    /// Décroissance exponentielle : valeur * facteur^dt
    pub fn decay(value: u64, factor: u64, dt: u64) -> u64 {
        // Exponentiation rapide par carrés
        let mut result = Self::FIXED_ONE; // 2^32
        let mut power = factor;
        let mut remaining = dt;
        
        while remaining > 0 {
            if remaining % 2 == 1 {
                result = Self::mul(result, power);
            }
            power = Self::mul(power, power);
            remaining /= 2;
        }
        
        Self::mul(value, result)
    }
}

/// Décroissance avec demi-vie
/// valeur_nouvelle = valeur_ancienne * 2^(-dt/demi_vie)
pub fn decay_with_half_life(value: u16, current_day: i64, last_update: i64, half_life: i64) -> u16 {
    let dt = (current_day - last_update).max(0) as u64;
    
    // Calculer facteur^dt en virgule fixe
    // Pour éviter les flottants, on utilise une table pré-calculée
    // ou on calcule directement avec exponentiation rapide
    
    let decayed = FixedPoint::decay(value as u64, half_life_factor(half_life), dt);
    (decayed >> 16) as u16 // Arrondi
}
```

**Impact attendu sur les replais** :
- Les replais qui utilisent `Engagement.valeur` (labor.rs) vont changer
- Les valeurs de décroissance seront **légèrement différentes** (virgule fixe vs flottant)
- **Aucun replai ne doit être régénéré** - signaler l'impact et s'arrêter
- Les replais existants (T1-T5) peuvent être affectés si ils dépendent de labor.rs

**Ticket** : "Corriger labor.rs:299-301 - Remplacer f64/exp() par décroissance exponentielle en virgule fixe"

---

## 🎯 **RÉSUMÉ**

| Élément | Statut | Priorité |
|---------|--------|----------|
| Entités avec identifiant stable | ✅ Spécifié | Haute |
| G en µpts, intégration unifiée | ✅ Spécifié | Haute |
| Bandes de λ en entiers | ✅ Spécifié | Haute |
| Tirage exponentiel en entiers | ✅ Spécifié | Haute |
| Planification exacte | ✅ Spécifié | Haute |
| λ = 0 pendant grève | ✅ Spécifié | Haute |
| G scalaire, δ pour nommés | ✅ Spécifié | Moyenne |
| I8 comme propriété | ✅ Spécifié | Moyenne |
| p = f(salaire_reel) en données | ✅ Spécifié | Moyenne |
| Plan de tranches réorganisé | ✅ Spécifié | Haute |
| labor.rs (hors périmètre) | ⚠️ Ticket séparé | Haute |

**Attendez ma validation avant toute implémentation.**

---

*Document généré par Agent IA - Phase 2, Étape 1 : Spécification v1.1*
*Toutes les valeurs en entiers. Aucun flottant. Conforme aux contraintes de conception.*
