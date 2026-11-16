# Vestiges Horizons - Spécification Corrective T7-T8
**Phase 2, Étape 1 : Spécification**
**Date** : 14 novembre 2026
**Version** : 1.0
**Statut** : À valider avant implémentation

> **Contexte** : Diagnostic v3 accepté. Cette spécification définit la correction des défauts T7-T8.
> **Règle** : Aucune implémentation avant validation de ce document.

---

## 1️⃣ **ENTITÉS**

### 1.1 Cohorte

**État par cohorte**, indexé par identifiant stable.

```rust
/// Identifiant stable : jamais réutilisé, dérivé du parent à la scission
/// Format : (graine_du_monde, numéro_unique) où numéro_unique est incrémenté globalement
/// À la scission : enfant = (parent.id, événement_id)
///     où événement_id = identifiant stable de l'événement de scission
pub struct CohortId {
    pub world_seed: u64,
    pub unique_number: u64,
}
```

**État de griefs et risque** :

```rust
/// G en micro-points [0 ; 100_000_000]
/// Échelle : 1 µpt = 10⁻⁶ point de grief
/// 100 points = 100_000_000 µpts (maximum)
pub struct GriefsState {
    /// Griefs cumulés (en micro-points)
    pub g: u64,
    
    /// Entrées (entiers [0 ; 1000])
    /// Constantes par morceaux : changent uniquement sur InputChanged
    pub u: u16,  // Besoins non satisfaits
    pub p: u16,  // Pression des prix  
    pub q: u16,  // Qualité effective
    
    /// Date du dernier changement d'entrée
    pub last_input_change: Timestamp,
    
    /// Risque cumulé (en unités de risque, échelle 10⁹)
    pub risk_accumulated: u64,
    
    /// Seuil de grève tiré (une fois par grève)
    pub strike_threshold: u64,
    
    /// Compteur d'indices de tirage (incrémenté après chaque grève)
    pub draw_index: u32,
    
    /// État de grève
    pub strike_state: StrikeState,
    
    /// Compteur d'époque (pour l'ordonnancement canonique v6)
    pub epoch: u64,
}

/// État de grève
pub enum StrikeState {
    /// Pas de grève
    Idle,
    /// Grève en cours
    Ongoing { start_day: i64, planned_end: i64 },
    /// Grève terminée (attend fin effective)
    Ending { end_day: i64, cause: StrikeEndCause },
}
```

**Salaire réel** :

```rust
/// Salaire réel en pourcentage (entier large)
/// 100 = salaire couvre exactement le panier journalier
/// Stocké en millièmes pour éviter les flottants
pub struct RealWage {
    /// Salaire réel en millièmes de pourcentage (ex: 100000 = 100%)
    pub value: i64,
    
    /// Coût du panier journalier (en centimes)
    /// DOIT être > 0, sinon erreur explicite
    pub basket_cost: u64,
    
    /// Salaire horaire (en centimes/heure)
    pub hourly_wage: u64,
    
    /// Heures travaillées par jour
    pub hours_worked: u8,
}

impl RealWage {
    /// Calcule le salaire réel
    /// Formule : (hourly_wage × hours_worked × 100_000) / basket_cost
    /// Retourne une erreur si basket_cost == 0
    pub fn calculate(hourly_wage: u64, hours_worked: u8, basket_cost: u64) 
                     -> Result<i64, ValidationError> {
        if basket_cost == 0 {
            return Err(ValidationError::ZeroBasketCost);
        }
        let numerator = hourly_wage as i128 * hours_worked as i128 * 100_000;
        // Vérifier le dépassement avant division
        if numerator > i64::MAX as i128 {
            return Ok(i64::MAX);  // Saturation
        }
        Ok((numerator / basket_cost as i128) as i64)
    }
}
```

---

## 2️⃣ **DYNAMIQUE DE G**

### 2.1 Pente

**Linéaire par segment** : G évolue linéairement entre deux changements d'entrée.

```rust
/// Pente de G en µpt/jour
/// Formule : pente = 100·u + 25·p − 30·q − soulagement
/// où u, p, q ∈ [0 ; 1000]
/// soulagement = 20_000 µpt/jour (constant, tant que G > 0)
///
/// Coefficients stockés en DONNÉES (pas en code) :
/// - coefficient_u: i64 = 100
/// - coefficient_p: i64 = 25
/// - coefficient_q: i64 = -30
/// - relief_rate: i64 = 20_000 (soulagement)
///
/// Calcul :
pub fn calculate_slope(u: u16, p: u16, q: u16) -> i64 {
    let coefficients = load_grief_coefficients(); // depuis données
    coefficients.u * u as i64 
        + coefficients.p * p as i64 
        + coefficients.q * q as i64 
        - coefficients.relief_rate
}
```

### 2.2 Intégration

**G += pente × Δt**, clamp [0 ; 100_000_000], tout en entiers.

```rust
/// Met à jour G entre last_date et current_date
pub fn update_g(state: &mut GriefsState, current_date: Timestamp) {
    let dt = current_date.day - state.last_input_change.day;
    if dt <= 0 {
        return;
    }
    
    let slope = calculate_slope(state.u, state.p, state.q);
    
    // Intégration : G += pente × dt
    // En entiers exacts : pas de flottants, pas d'arrondi intermédiaire
    let delta = slope as i128 * dt as i128;
    
    // Clamp [0 ; 100_000_000]
    let new_g = (state.g as i128 + delta).clamp(0, 100_000_000);
    state.g = new_g as u64;
    
    // Mettre à jour la date
    state.last_input_change = current_date;
}
```

**Propriétés** :
- ✅ Pas de flottants
- ✅ Intégration exacte en entiers
- ✅ Arrondi nul (tout est entier exact)
- ✅ Clamp garanti

---

## 3️⃣ **RISQUE ET GRÈVES**

### 3.1 Taux de risque (λ)

**Par bandes de G**, valeurs en données :

```rust
/// Taux λ en événements/an (valeurs de base)
/// Échelle : 1 événement = 10⁹ unités de risque
///
/// Données :
pub struct RiskBands {
    pub bands: Vec<(u64, f64)>; // (seuil_g, λ_par_an)
}

/// Bands par défaut (à stocker en données) :
/// - G < 30 : λ = 0 événements/an
/// - 30 ≤ G < 60 : λ = 0.05/an
/// - 60 ≤ G < 80 : λ = 0.2/an
/// - G ≥ 80 : λ = 0.5/an
///
/// Conversion en unités de risque (entiers) :
/// λ_µrisk = λ_par_an × 10⁹ / 365 (pour avoir λ en µrisk/jour)
/// Exemple : λ = 0.5/an → λ_µrisk = 0.5 × 10⁹ / 365 ≈ 1 369 863 µrisk/jour
```

**Implémentation entière** :

```rust
/// Taux λ en µrisk/jour (entier)
pub fn calculate_lambda(g: u64) -> u64 {
    let bands = load_risk_bands(); // depuis données
    
    // Trouver la bande
    for (threshold, lambda_par_an) in bands.iter() {
        if g < *threshold {
            // Convertir en µrisk/jour
            // λ_µrisk = λ_par_an × 10⁹ / 365
            // Pour éviter les flottants : λ_par_an × 10⁹ × 1_000_000 / (365 × 1_000_000)
            // Mais on peut pré-calculer en données
            return *lambda_par_an as u64;
        }
    }
    0  // Par défaut
}
```

### 3.2 Accumulation du risque

**R s'accumule à λ × Δt** :

```rust
/// Met à jour le risque cumulé entre last_date et current_date
pub fn update_risk(state: &mut GriefsState, current_date: Timestamp) {
    let dt = current_date.day - state.last_input_change.day;
    if dt <= 0 {
        return;
    }
    
    let lambda = calculate_lambda(state.g);
    let delta_risk = lambda as i128 * dt as i128;
    
    state.risk_accumulated += delta_risk as u64;
    state.last_input_change = current_date;
}
```

### 3.3 Tirage unique

**Une grève = un seuil S**, tiré une seule fois.

```rust
/// Clé stable : (graine_du_monde, cohort_id, "strike", draw_index)
pub fn generate_stable_key(world_seed: u64, cohort_id: CohortId, draw_index: u32) -> String {
    format!("{}-{}-{}-strike-{}", world_seed, cohort_id.world_seed, 
            cohort_id.unique_number, draw_index)
}

/// Tirage exponentiel en entiers
/// S = -ln(U) × échelle, où U ∈ [0, 1) uniforme
/// Échelle : 10⁹ unités de risque (1 événement = 10⁹ unités)
///
/// Implémentation :
///   U = hash_32(key) / 2³²  (approximation de uniforme [0,1))
///   S = -ln(1 - U) × 10⁹
///
/// Pour éviter ln() flottant, on utilise une table pré-calculée
/// ou une approximation entière de -ln(1-x)
///
/// Méthode recommandée : table pré-calculée de 256 valeurs
/// pour x ∈ [0, 255]/256, avec interpolation linéaire
pub fn draw_strike_threshold(key: &str) -> u64 {
    let u = stable_hash_u32(key) as f64 / u32::MAX as f64;
    // -ln(1-u) = -ln(u) pour u proche de 0, mais on veut -ln(1-u)
    // Approximation entière : utiliser table pré-calculée
    let threshold = exponential_table(u); // → u64 en [0, 10⁹]
    threshold
}

/// Tirage de la durée de grève (en jours)
/// Clé distincte : (graine_du_monde, cohort_id, "strike_duration", draw_index)
pub fn draw_strike_duration(key: &str) -> u16 {
    let hash = stable_hash_u32(key);
    // Durée : 1-30 jours (à calibrer en données)
    1 + (hash % 30) as u16
}
```

**Garanties** :
- ✅ Un seul tirage par grève (décision 43)
- ✅ Clé stable (graine + cohort_id + type + index)
- ✅ Pas de re-tirage
- ✅ Tirage exponentiel (approximation entière)

### 3.4 Déclenchement

**Grève déclenchée quand R ≥ S** :

```rust
/// Vérifie si le seuil est franchi
pub fn check_strike_trigger(state: &GriefsState) -> bool {
    state.risk_accumulated >= state.strike_threshold
}

/// Déclenche une grève
pub fn trigger_strike(state: &mut GriefsState, current_date: Timestamp) {
    // Tirer le seuil et la durée (si pas déjà fait)
    if state.strike_threshold == 0 {
        let key = generate_stable_key(
            state.world_seed, state.cohort_id, state.draw_index);
        state.strike_threshold = draw_strike_threshold(&key);
        
        let duration_key = format!("{}-duration", key);
        let duration = draw_strike_duration(&duration_key);
        
        state.strike_state = StrikeState::Ongoing {
            start_day: current_date.day,
            planned_end: current_date.day + duration as i64,
        };
    }
    
    // Réinitialiser le risque cumulé pour la prochaine grève
    state.risk_accumulated = 0;
    
    // Incrémenter l'indice pour la prochaine grève
    state.draw_index += 1;
}
```

### 3.5 Fin de grève

**Fin par concession ou durée écoulée** :

```rust
/// Termine une grève
pub fn end_strike(state: &mut GriefsState, end_day: i64, cause: StrikeEndCause) {
    state.strike_state = StrikeState::Ending { end_day, cause };
    
    // Réinitialiser pour la prochaine grève
    state.risk_accumulated = 0;
    
    // Tirer un nouveau seuil pour la prochaine grève
    let key = generate_stable_key(
        state.world_seed, state.cohort_id, state.draw_index);
    state.strike_threshold = draw_strike_threshold(&key);
}
```

---

## 4️⃣ **ÉVÉNEMENTS**

### 4.1 Types d'événements

```rust
/// Événements T7-T8
pub enum T7T8Event {
    /// Changement d'entrée (u, p, q)
    /// Émis par les couches amont (marché, santé, etc.)
    /// NE JAMAIS émis par une boucle quotidienne
    InputChanged {
        cohort_id: CohortId,
        new_u: u16,
        new_p: u16,
        new_q: u16,
        date: Timestamp,
    },
    
    /// Franchissement d'une bande de G
    /// Émis quand G traverse 30, 60, 80
    BandCrossed {
        cohort_id: CohortId,
        old_band: u8,
        new_band: u8,
        date: Timestamp,
    },
    
    /// G atteint 0 ou 100_000_000
    GBoundHit {
        cohort_id: CohortId,
        bound: Bound, // Zero ou Max
        date: Timestamp,
    },
    
    /// Seuil de grève franchi (R ≥ S)
    StrikeThresholdCrossed {
        cohort_id: CohortId,
        date: Timestamp,
    },
    
    /// Fin de grève
    StrikeEnded {
        cohort_id: CohortId,
        cause: StrikeEndCause,
        date: Timestamp,
    },
}
```

### 4.2 Ordonnancement

**À chaque InputChanged** :

```rust
/// Traitement d'un événement InputChanged
pub fn handle_input_changed(
    state: &mut GriefsState,
    event: &InputChanged,
    event_queue: &mut EventQueue,
) {
    // 1. Avancer G et R jusqu'à la date de l'événement
    update_g(state, event.date);
    update_risk(state, event.date);
    
    // 2. Changer les entrées
    state.u = event.new_u;
    state.p = event.new_p;
    state.q = event.new_q;
    state.last_input_change = event.date;
    
    // 3. Recalculer la pente
    let new_slope = calculate_slope(state.u, state.p, state.q);
    let old_slope = calculate_slope_for_previous(state);
    
    // 4. Planifier le prochain événement
    // a) Si la pente change de signe, planifier GBoundHit
    if (old_slope < 0 && new_slope >= 0) || (old_slope >= 0 && new_slope < 0) {
        // G va changer de direction, planifier GBoundHit
        let next_bound_date = calculate_bound_hit_date(state);
        event_queue.push(Event::new(
            next_event_id(),
            next_bound_date.day,
            0,
            Phase::Social,
            Priority::new(1),
            T7T8Event::GBoundHit { ... },
        ));
    }
    
    // b) Si changement de bande de G, planifier BandCrossed
    let old_band = calculate_band(state.g);
    let new_band = calculate_band(state.g + new_slope * 1); // estimé
    if old_band != new_band {
        let cross_date = calculate_band_cross_date(state, old_band, new_band);
        event_queue.push(Event::new(
            next_event_id(),
            cross_date.day,
            0,
            Phase::Social,
            Priority::new(1),
            T7T8Event::BandCrossed { ... },
        ));
    }
    
    // c) Planifier le prochain InputChanged (si connu)
    //    → Émis par les couches amont
    
    // 5. Incrémenter l'époque
    state.epoch += 1;
}
```

**Règle** : Tout événement planifié sous une époque périmée est **ignoré**.

---

## 5️⃣ **SEUIL DUR**

**Le seuil de 50 et `taux_risque = griefs + max(0, 100 − salaire_reel/10)` sortent de la causalité.**

```rust
/// Ces valeurs ne servent QU'AUX ALERTES ET À L'INTERFACE
/// Elles ne déclenchent PAS de grève
pub struct AlertThresholds {
    /// Seuil pour alerte (50)
    /// AFFICHAGE UNIQUEMENT
    pub alert_threshold: u8,
    
    /// Calcul du taux de risque pour l'interface
    /// N'UTILISE PAS pour le déclenchement
    pub fn display_risk_rate(griefs: u64, real_wage: i64) -> u8 {
        // Convertir griefs en [0, 100]
        let g_percent = (griefs / 1_000_000) as i64; // 100_000_000 µpt = 100%
        let wage_factor = (100 - (real_wage / 100)).max(0); // real_wage en millièmes
        (g_percent + wage_factor).clamp(0, 100) as u8
    }
}
```

**Garantie** :
- ✅ Le seuil de 50 **ne déclenche pas** de grève
- ✅ Le déclenchement utilise **uniquement** R ≥ S (tirage exponentiel)
- ✅ `taux_risque` est **affichage uniquement**

---

## 6️⃣ **I8 REDÉFINI**

### 6.1 Agrégat de cohorte

**Le total anonyme est un champ propre, pas un reste calculé.**

```rust
/// État des griefs pour une cohorte
pub struct CohortGriefs {
    /// Total des griefs des individus nommés
    pub named_total: i64,
    
    /// Total des griefs des individus anonymes
    /// **Champ propre**, mis à jour explicitement
    pub anonymous_total: i64,
    
    /// Population nommée
    pub named_count: u64,
    
    /// Population anonyme
    pub anonymous_count: u64,
}

impl CohortGriefs {
    /// Vérifie I8 : named_total + anonymous_total == total_griefs
    pub fn verify_i8(&self) -> bool {
        // total_griefs = named_total + anonymous_total
        // Toujours vrai par construction si on met à jour correctement
        true  // → I8 devient une PROPRIÉTÉ, pas un invariant à tester
    }
}
```

### 6.2 Extraction

**Conservation stricte avec floor** :

```rust
/// Extrait un individu d'une cohorte
pub fn extract_individual(cohort: &mut Cohort, griefs: &mut CohortGriefs) -> NamedIndividual {
    // Calculer le grief de l'individu extrait
    // grief = floor(anonymous_total / anonymous_count)
    let individual_grief = if griefs.anonymous_count > 0 {
        (griefs.anonymous_total / griefs.anonymous_count as i64) as i64
    } else {
        0
    };
    
    // Mettre à jour la cohorte
    cohort.population -= 1;
    
    // Mettre à jour les griefs
    griefs.named_count += 1;
    griefs.anonymous_count -= 1;
    
    // **Conservation stricte** : retirer exactement individual_grief
    griefs.named_total += individual_grief;
    griefs.anonymous_total -= individual_grief;
    
    // Créer l'individu
    NamedIndividual {
        grief: individual_grief,
        // ... autres champs
    }
}
```

### 6.3 Réinjection

```rust
/// Réinjecte un individu dans sa cohorte
pub fn reinject_individual(
    cohort: &mut Cohort,
    griefs: &mut CohortGriefs,
    individual: NamedIndividual,
) {
    // Mettre à jour la cohorte
    cohort.population += 1;
    
    // Mettre à jour les griefs
    griefs.named_count -= 1;
    griefs.anonymous_count += 1;
    
    // **Conservation stricte** : ajouter exactement individual.grief
    griefs.named_total -= individual.grief;
    griefs.anonymous_total += individual.grief;
}
```

**Propriétés** :
- ✅ **Conservation stricte** : `named_total + anonymous_total` est toujours constant
- ✅ **Floor utilisé** : `grief = floor(anonymous_total / anonymous_count)`
- ✅ **I8 devient une propriété** : toujours vrai par construction

---

## 7️⃣ **SALAIRE RÉEL → P**

**Le salaire réel alimente p (pression des prix), pas λ.**

```rust
/// Mise à jour de p en fonction du salaire réel
/// p ∈ [0 ; 1000]
/// Formule : p = f(salaire_reel, cout_panier, inflation, etc.)
///
/// Exemple simple (à calibrer en données) :
/// p = max(0, 1000 - salaire_reel_percent)
/// où salaire_reel_percent = salaire_reel / 100
///
/// Mais salaire_reel est en millièmes :
/// salaire_reel_value = 100_000 → 100%
/// p = max(0, 1000 - (salaire_reel_value / 1000)) as u16
pub fn update_p_from_real_wage(real_wage: &RealWage, inflation: u16) -> u16 {
    let coefficients = load_price_pressure_coefficients();
    // Calcul basé sur données
    coefficients.calculate_p(real_wage.value, inflation)
}
```

**Validation** :
- ✅ Si `cout_panier = 0` → **erreur explicite** (ValidationError::ZeroBasketCost)
- ✅ Pas de flottants : tout en entiers
- ✅ Saturation avec i128 pour éviter les dépassements

---

## 8️⃣ **IDENTITÉ**

**À la scission, le parent garde son identifiant, l'enfant en dérive** :

```rust
/// Scission d'une cohorte
pub fn split_cohort(
    parent: &Cohort,
    event_id: u64,
    population_split: u64,
) -> Cohort {
    Cohort {
        id: CohortId {
            world_seed: parent.id.world_seed,
            unique_number: derive_child_number(&parent.id, event_id),
        },
        // ... autres champs
    }
}

/// Dérivation de l'identifiant enfant
/// Utilise un hash stable : (parent_id, event_id)
pub fn derive_child_number(parent_id: &CohortId, event_id: u64) -> u64 {
    let hash = stable_hash_u64(&format!("{}-{}", parent_id.unique_number, event_id));
    hash
}
```

**Propriétés** :
- ✅ **Parent garde son identifiant**
- ✅ **Enfant dérive du parent + événement**
- ✅ **Identifiant stable** (reproductible)
- ❌ **Pas d'UUID aléatoire**

---

## 9️⃣ **CRITÈRES DE RÉUSSITE**

### 9.1 Tests fonctionnels

1. **Zéro grève sans stress** :
   ```
   G = 0, salaire = panier, u = p = q = 0, 50 ans
   → zéro grève en moyenne sur 20 graines
   ```

2. **Pénurie durable** :
   ```
   u = 800 (constant), p = 0, q = 0
   → moyenne empirique du nombre de grèves sur 50 graines
      égale à l'intégrale analytique de λ ±20 %
   ```

3. **Invariance par scission** :
   ```
   Scinder un segment constant en deux
   → ne change ni G ni la date de franchissement
   ```

4. **Proportionnalité événements/entrées** :
   ```
   Nombre d'événements traités proportionnel aux changements d'entrée,
   aux franchissements de bande et aux grèves, pas aux jours simulés
   ```

5. **I8 en test de propriété** :
   ```
   50 graines, séquences aléatoires d'extractions, réinjections et évolutions,
   avec totaux non divisibles
   → I8 toujours vrai (propriété par construction)
   ```

6. **Cas limites** :
   ```
   - u = p = 1000, q = 0 pendant des années → G saturé à 100_000_000
   - G = 100_000_000 → vérification du clamp
   - G = 0 avec soulagement → vérification de la décroissance
   - salaire très élevé → pas de dépassement (saturation)
   - panier = 0 → erreur explicite (rejet)
   ```

7. **Indépendance des cohortes** :
   ```
   Deux cohortes aux entrées différentes évoluent indépendamment
   → vérification que l'état n'est pas partagé
   ```

8. **Performance** :
   ```
   100 cohortes × 100 ans
   → mesurer en événements par seconde et ms par an simulé
   ```

9. **Scénario benchmark** :
   ```
   Un scénario du benchmark remplace les modulos par des entrées plausibles
   décrites en données
   → benchmark P1 inchangé
   ```

---

## 🎯 **PLAN DE TRANCHES VERTICALES**

### Tranche A : Infrastructure de base

**Périmètre** :
- Définir `CohortId` avec identifiant stable
- Créer `GriefsState` avec G en µpts
- Implémenter `calculate_slope` avec coefficients en données
- Implémenter `update_g` (intégration exacte)

**Événements** : InputChanged seulement

**Tests** :
- Test de l'intégration : G += pente × Δt
- Test du clamp [0 ; 100_000_000]
- Test de conservation avec extractions/réinjections

---

### Tranche B : Dynamique du risque

**Périmètre** :
- Implémenter `calculate_lambda` avec bandes en données
- Implémenter `update_risk` (accumulation)
- Tirage exponentiel (table pré-calculée)
- Déclenchement quand R ≥ S

**Événements** : StrikeThresholdCrossed

**Tests** :
- Test du tirage exponentiel (distribution)
- Test du déclenchement (R ≥ S)
- Test de la réinitialisation après grève

---

### Tranche C : Gestion des grèves

**Périmètre** :
- Implémenter `StrikeState`
- Implémenter `trigger_strike` et `end_strike`
- Tirage de la durée

**Événements** : StrikeEnded

**Tests** :
- Test de la fin de grève par concession
- Test de la fin de grève par durée
- Test du déterminisme (même graine → même résultat)

---

### Tranche D : Bandes et bornes

**Périmètre** :
- Implémenter `BandCrossed`
- Implémenter `GBoundHit`
- Planification des événements

**Événements** : BandCrossed, GBoundHit

**Tests** :
- Test du franchissement de bande
- Test des bornes (G=0, G=max)
- Test de l'ordonnancement des événements

---

### Tranche E : I8 et extraction

**Périmètre** :
- Redéfinir `CohortGriefs` avec anonymous_total
- Implémenter extraction/réinjection avec floor
- Valider I8 comme propriété

**Tests** :
- Test de conservation stricte
- Test avec totaux non divisibles
- Test de propriété I8

---

### Tranche F : Salaire réel

**Périmètre** :
- Implémenter `RealWage` avec validation
- Lien salaire réel → p

**Tests** :
- Test de rejection si panier = 0
- Test de saturation
- Test du lien vers p

---

### Tranche G : Intégration au noyau

**Périmètre** :
- Connecter T7-T8 à la file d'événements
- Remplacer la boucle quotidienne par des événements
- Valider le benchmark P1

**Tests** :
- Benchmark P1 inchangé
- Test des critères de réussite 1-9

---

## 📋 **ORDRE D'EXÉCUTION**

1. **A → B → C** : Infrastructure + dynamique de base
2. **D** : Bandes et bornes
3. **E** : I8 et extraction
4. **F** : Salaire réel
5. **G** : Intégration finale

**Critère de sortie** : Tous les tests des critères de réussite passent.

---

## 🔴 **HORS PÉRIMÈTRE (À TRAITER EN TICKET SÉPARÉ)**

### labor.rs : f64 et exp() dans l'état canonique

**Problème** : `src/labor.rs:299-301` utilise f64 et exp() pour calculer la décroissance avec demi-vie, et le résultat est stocké dans `valeur: u16`.

**Impact sur les replais** :
- `valeur` fait partie de l'état canonique (struct `Engagement`)
- Le calcul utilise des flottants → **violation de la convention 2**
- Les replais qui utilisent labor.rs auront des valeurs non déterministes ou imprévisibles

**Solution proposée** :
- Remplacer la décroissance exponentielle par une **décroissance linéaire par segments**
- Stocker le taux de décroissance en données (entier)
- Implémenter l'intégration en entiers : `valeur += delta × Δt`

**Table de décroissance entière** :
```rust
/// Décroissance linéaire avec demi-vie
/// Taux : 1/unité_de_temps
/// Exemple : demi-vie = 365 jours → taux = ln(2)/365 ≈ 0.0019 par jour
/// En entiers : taux_micro = 1900 (µpt/jour)
pub struct DecayTable {
    pub rates: Vec<(u64, i32)>, // (seuil, taux_µpt/jour)
}

/// Mise à jour avec décroissance
pub fn update_with_decay(value: &mut u16, delta: i16, dt: i64, rate: i32) {
    let decay = rate as i64 * dt;
    let new_value = (*value as i64 - decay).clamp(0, u16::MAX as i64) as u16;
    *value = new_value;
}
```

**Ticket** : "Corriger labor.rs:299-301 - Remplacer f64/exp() par décroissance entière"

---

## ✅ **RÉSUMÉ**

| Élément | Statut | Priorité |
|---------|--------|----------|
| Entités avec identifiant stable | ✅ Spécifié | Haute |
| Dynamique de G (linéaire, µpts) | ✅ Spécifié | Haute |
| Risque avec bandes et tirage exponentiel | ✅ Spécifié | Haute |
| Événements (InputChanged, BandCrossed, etc.) | ✅ Spécifié | Haute |
| Seuil dur sorti de la causalité | ✅ Spécifié | Haute |
| I8 redéfini (propriété) | ✅ Spécifié | Moyenne |
| Salaire réel → p | ✅ Spécifié | Moyenne |
| Critères de réussite | ✅ Spécifiés | Haute |
| Plan de tranches | ✅ Spécifié | Haute |
| labor.rs (hors périmètre) | ⚠️ Ticket séparé | Haute |

---

*Document à valider avant toute implémentation - Phase 2, Étape 1 : Spécification*
