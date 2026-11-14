# Solutions Dev - Registre froid des problèmes transverses

**Règle économique** : Chaque entrée doit avoir un gain > coût en tokens à chaque session.
**Ne pas dupliquer** : Ce qui est déjà dans `Docs/spec P1 - quartier vivant.md` ou `Docs/cadrage vestiges horizons - v7.7.md`.

---

## Problèmes transverses résolus

### 1. Débordements dans les calculs économiques
**Date** : 2024-11-14  
**Module** : Tous (market, credit, tax, meal)  
**Problème** : Les calculs monétaires (en centimes) et les calculs de pourcentage (en millièmes) peuvent déborder u64.  
**Solution** : 
- Utiliser `checked_mul` pour les multiplications, avec fallback sur `u64::MAX` ou une valeur plafond
- Utiliser `saturating_add` et `saturating_sub` pour les additions/soustractions
- Décomposer les calculs complexes en étapes avec vérification à chaque étape
- Pour les divisions : diviser tôt pour réduire l'échelle (ex: `panier_cost / 1000 * percent` au lieu de `panier_cost * percent / 1000`)

**Exemple concret** :
```rust
// ❌ Risque de débordement
let total = a * b * c / d;

// ✅ Sécurisé
let part1 = a.checked_mul(b);
if let Some(p1) = part1 {
    let part2 = p1.checked_mul(c);
    if let Some(p2) = part2 {
        p2 / d
    } else {
        u64::MAX
    }
} else {
    u64::MAX
}
```

**Modules concernés** : market.rs, credit.rs, tax.rs, meal.rs

---

### 2. Vérification des invariants I1 et I9
**Date** : 2024-11-14  
**Module** : Tous  
**Problème** : Les invariants critiques (I1, I9) doivent être vérifiés systématiquement.  
**Solution** : 
- Ajouter une méthode `verify_i9_deterministic_<module>()` dans chaque manager
- Cette méthode compare deux instances du manager avec la même graine
- Utiliser ces méthodes dans les tests d'intégration

**Pattern** :
```rust
pub fn verify_i9_deterministic(&self, other: &Self) -> bool {
    // Comparer tous les champs critiques
    self.field1 == other.field1 && self.field2 == other.field2
}
```

---

### 3. Formule de prix de référence incorrecte
**Date** : 2024-11-14  
**Module** : market.rs  
**Problème** : La formule `(slope * excess * 1000) / capacity` appliquait un facteur 1000 en trop car `PRICE_ELASTICITY_SLOPE` est déjà en millièmes.  
**Solution** : 
- Utiliser `(slope * excess) / capacity` directement
- `slope = 500` représente 0.5 (en décimal), donc pas besoin de multiplier par 1000

**Impact** : Correction critique - sans cela, les prix augmentaient de 500% au lieu de 10% pour un excès de 20%

---

## Règles d'ajout

1. **Une entrée = un problème transverse** (impacte ≥ 2 modules)
2. **Format obligatoire** : Date, Module(s), Problème, Solution, Exemple concret
3. **Pas de duplication** avec la spec ou le cadrage
4. **Test économique** : Si l'entrée n'est pas lue dans 3 sessions, la supprimer