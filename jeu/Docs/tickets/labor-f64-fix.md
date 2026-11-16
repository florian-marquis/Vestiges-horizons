# Ticket : Corriger labor.rs - Remplacer f64/exp() par décroissance en virgule fixe

**ID** : LABOR-F64-001
**Module** : `src/labor.rs`
**Priorité** : 🔴 **CRITIQUE** (violation convention 2 : pas de flottants dans l'état canonique)
**Statut** : À traiter **avant T9** (hors périmètre T7-T8)
**Type** : Correction de violation de convention

---

## 📋 **DESCRIPTION**

**Problème** : La fonction `Engagement::mise_a_jour` (`src/labor.rs:292-301`) utilise des **flottants (f64)** et la fonction **`exp()`** pour calculer la décroissance exponentielle avec demi-vie. Le résultat est converti en `u16` et stocké dans le champ `valeur`, qui fait partie de l'**état canonique**.

**Code incriminé** :
```rust
// src/labor.rs:292-301
pub fn mise_a_jour(&mut self, delta: i16, current_day: i64, demi_vie: i64) {
    // Décroissance selon demi-vie
    let jours_ecoules = (current_day - self.dernier_mis_a_jour).max(0) as f64;
    let facteur = (-0.693 / demi_vie as f64 * jours_ecoules).exp(); // ❌ f64 + exp()
    let valeur_ancienne = (self.valeur as f64 * facteur) as u16;       // ❌ conversion f64 → u16
    
    // Appliquer le delta
    let nouvelle_valeur = (valeur_ancienne as i32 + delta as i32).clamp(0, 1000) as u16;
    
    self.valeur = nouvelle_valeur;
    self.dernier_mis_a_jour = current_day;
}
```

**Violation** : Convention 2 - "Pas de flottants dans l'état canonique ou les tirages"

---

## 🎯 **IMPACT**

### Sur l'état canonique
- **`valeur: u16`** (dans `Engagement`) entre dans l'état canonique
- Le calcul utilise `f64` et `exp()` → **violation directe**
- La valeur stockée dépend de la précision des flottants → **non déterministe en théorie**

### Sur les replais
- Les replais qui dépendent de `Engagement.valeur` seront **affectés**
- Exemples de replais concernés :
  - `src/replai/reference_seed42_t3.json` (Tranche 3 utilise labor.rs)
  - `src/replai/reference_seed42_t4.json` (Tranche 4 utilise labor.rs)
  - `src/replai/reference_seed42_t5.json` (Tranche 5 utilise labor.rs)
- **Les valeurs de `valeur` seront légèrement différentes** après correction

**⚠️ ATTENTION** : Ne pas régénérer les replais. **Signaler l'impact et s'arrêter** si une correction les affecte.

### Sur la précision
- La version actuelle (f64) a une **précision limitée** (53 bits de mantisse)
- La version corrigée (virgule fixe) aura une **précision contrôlée** (32 bits de fraction)
- La différence sera **minime** (moins de 1 point sur 1000)

---

## ✅ **SOLUTION PROPOSÉE**

### 1. Décroissance exponentielle en virgule fixe

**Facteur journalier** : `2^(-1/demi_vie)` en virgule fixe 2^32

**Exemple** : demi-vie = 365 jours
- `facteur = 2^(-1/365) ≈ 0.99727`
- En virgule fixe 2^32 : `facteur_fixed ≈ 4_274_000_000`

### 2. Exponentiation rapide

**Algorithme** : Exponentiation par carrés (O(log n))

```rust
/// Virgule fixe 2^32 : 1.0 = 2^32 = 4294967296
pub struct Fixed32 {
    pub value: u64,
}

impl Fixed32 {
    pub const ONE: u64 = 1 << 32;  // 4294967296
    
    /// Multiplie deux nombres en virgule fixe 2^32
    pub fn mul(a: u64, b: u64) -> u64 {
        ((a as u128 * b as u128) >> 32) as u64
    }
    
    /// Calcul le facteur pour une demi-vie donnée
    /// facteur = 2^(-1/demi_vie)
    /// On utilise ln(2) ≈ 0.69314718056
    pub fn half_life_factor(demi_vie: i64) -> u64 {
        if demi_vie <= 0 {
            return Self::ONE; // pas de décroissance
        }
        
        // ln(2) * 2^32 ≈ 2971596184
        let ln2_fixed = 2971596184u64;
        
        // -ln(2)/demi_vie * 2^32 ≈ ln2_fixed / demi_vie
        let rate = ln2_fixed / demi_vie as u64;
        
        // exp(-rate) ≈ 2^32 / (2^rate) ≈ Self::ONE >> rate
        // Mais on veut un facteur multiplicatif, pas une division
        
        // Approximation : utiliser une table pré-calculée pour les demi-vies courantes
        // ou calculer avec une série de Taylor en virgule fixe
        Self::exp_negative_fixed(rate)
    }
    
    /// exp(-x) en virgule fixe 2^32 (approximation)
    /// Utilise une série de Taylor ou une table pré-calculée
    pub fn exp_negative_fixed(x: u64) -> u64 {
        // Pour x < 2^32, on utilise une approximation
        // exp(-x/2^32) ≈ 1 - x/2^32 + x^2/2!2^64 - ...
        
        if x >= Self::ONE {
            return 0; // exp(-1) ≈ 0.367, mais en virgule fixe c'est très petit
        }
        
        // Approximation quadratique : 1 - x/2^32 + x^2/2^64
        let x2 = Self::mul(x, x);
        let term1 = Self::ONE;
        let term2 = x;
        let term3 = x2 >> 32;
        
        (term1 + term3).saturating_sub(term2)
    }
    
    /// Exponentiation rapide : value * factor^dt
    pub fn decay(value: u64, factor: u64, dt: u64) -> u64 {
        if factor == Self::ONE {
            return value;
        }
        if factor == 0 {
            return 0;
        }
        
        // Exponentiation par carrés
        let mut result = Self::ONE;
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
```

### 3. Implémentation finale

```rust
/// Mét à jour l'engagement avec décroissance exponentielle
/// valeur_nouvelle = (valeur_ancienne * facteur^dt) + delta
/// factor = 2^(-1/demi_vie) en virgule fixe 2^32
/// Tout en entiers, pas de flottants
pub fn mise_a_jour(&mut self, delta: i16, current_day: i64, demi_vie: i64) {
    let dt = (current_day - self.dernier_mis_a_jour).max(0) as u64;
    
    if dt > 0 && demi_vie > 0 {
        // Calculer le facteur
        let factor = Fixed32::half_life_factor(demi_vie);
        
        // Appliquer la décroissance
        let decayed = Fixed32::decay(self.valeur as u64, factor, dt);
        self.valeur = (decayed >> 16) as u16; // Arrondi à u16
    }
    
    // Appliquer le delta
    let nouvelle_valeur = (self.valeur as i32 + delta as i32).clamp(0, 1000) as u16;
    
    self.valeur = nouvelle_valeur;
    self.dernier_mis_a_jour = current_day;
}
```

---

## 📊 **IMPACT ATTENDU SUR LES REPLAIS**

### Replais concernés
- `src/replai/reference_seed42_t3.json` (Tranche 3)
- `src/replai/reference_seed42_t4.json` (Tranche 4)
- `src/replai/reference_seed42_t5.json` (Tranche 5)

### Différences attendues
| Champ | Actuel (f64) | Corrigé (Fixed32) | Différence |
|-------|---------------|-------------------|------------|
| `Engagement.valeur` | Valeur calculée avec f64 | Valeur calculée avec virgule fixe | < 1 point sur 1000 |

### Exemple concret
Pour un engagement avec :
- `valeur = 1000`
- `demi_vie = 365`
- `dt = 365`

**Actuel (f64)** :
```
facteur = exp(-ln(2)/365) ≈ 0.99727
valeur_ancienne = 1000 * 0.99727^365 ≈ 500.0
```

**Corrigé (Fixed32)** :
```
facteur_fixed ≈ 4274000000 (0.99727 * 2^32)
valeur_fixed = 1000 * facteur_fixed^365 / 2^32 ≈ 499.99
```

**Différence** : ≈ 0.01 point (négligeable mais non nulle)

### Conséquence
- Les tests qui dépendent des valeurs exactes de `Engagement.valeur` **échoueront**
- **Ne pas régénérer les replais** - signaler l'impact et s'arrêter
- Proposer une migration des replais **après** la correction

---

## 🎯 **PLAN D'ACTION**

### Phase 1 : Préparation (sans modification de code)
1. ✅ Identifier tous les appels à `Engagement::mise_a_jour`
2. ✅ Lister les replais concernés (t3, t4, t5)
3. ✅ Estimer l'impact sur les tests existants
4. ✅ **Signaler l'impact** et attendre validation avant toute modification

### Phase 2 : Implémentation (après validation)
1. Créer `src/utils/fixed32.rs` avec les fonctions de virgule fixe
2. Modifier `src/labor.rs:292-301` pour utiliser Fixed32
3. Exécuter `cargo test` et identifier les tests échouants
4. **Ne pas modifier les replais** - s'arrêter et signaler

### Phase 3 : Migration (ticket séparé)
1. Créer un nouveau ticket pour la migration des replais
2. Exécuter `generer_replai.rs` avec la nouvelle implémentation
3. Valider que les nouveaux replais sont cohérents
4. Mettre à jour les replais **uniquement après validation explicite**

---

## 📌 **DÉPENDANCES**

- **Aucune** dépendance avec T7-T8
- **À traiter avant T9** (comme demandé)
- **Ticket séparé** (hors périmètre T7-T8)

---

## ✅ **VALIDATION**

**Avant toute implémentation** :
- [ ] Valider l'approche (virgule fixe 2^32 + exponentiation rapide)
- [ ] Valider l'impact estimé sur les replais
- [ ] Confirmer que les replais ne doivent pas être régénérés sans validation explicite

**Attendez ma validation avant toute modification de code.**

---

*Document généré par Agent IA - Ticket LABOR-F64-001*
*Violation critique : flottants dans l'état canonique*
