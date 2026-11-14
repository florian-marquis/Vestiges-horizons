# Mémoire Dev - Décisions architecturales récurrentes

**Règle économique** : Ne contient que des entrées dont le gain > coût en tokens à chaque session.
**Chargé en début de chaque session JEU** pour éviter de répéter les débats.

---

## Décisions architecturales

### 1. Arithmétique entière systématique
**Décision** : TOUS les calculs économiques utilisent des entiers (u64 pour les montants, u32 pour les pourcentages en millièmes).

**Rationale** : 
- Spécification §3 : "Tout est entier"
- Évite les imprécisions des flottants
- Plus rapide et déterministe

**Pattern à suivre** :
```rust
// ❌ à éviter
let price = base_price * 1.10;  // flottant

// ✅ à utiliser
let price = base_price * 1100 / 1000;  // entier
```

**Exceptions** : Aucune - même les pourcentages sont en millièmes (ex: 10% = 100)

---

### 2. Gestion des débordements
**Décision** : Tous les calculs qui peuvent déborder sont protégés.

**Hiérarchie des protections** :
1. `checked_mul` / `checked_add` → Return `None` ou `u64::MAX`
2. `saturating_add` / `saturating_sub` → Sature aux bornes
3. Division précoce pour réduire l'échelle

**Exemple** :
```rust
// Pour : (a * b * c) / d
let part1 = a.checked_mul(b).unwrap_or(u64::MAX);
let part2 = part1.checked_mul(c).unwrap_or(u64::MAX);
part2 / d
```

---

### 3. Vérification des invariants
**Décision** : Chaque manager de module T5 doit implémenter une méthode `verify_i9_deterministic_<module>()`.

**Rationale** : 
- I9 : "même graine, mêmes données = même journal, bit à bit"
- Permet de valider le déterminisme à tout moment

**Modules concernés** : MarketManager, CreditManager, TaxManager, MealManager

---

### 4. Structure des tests
**Décision** : Structure uniforme pour tous les tests.

**Template** :
```rust
#[test]
fn test_<module>_<scenario>_<invariants>() {
    // 1. Setup avec graine déterministe
    let seed = 42;
    
    // 2. Action
    // ...
    
    // 3. Assertions
    // - Vérification des invariants (I1, I9)
    // - Vérification des résultats attendus
}
```

---

### 5. Nommage des fonctions de vérification
**Décision** : Préfixer les fonctions de vérification par `verify_` + nom de l'invariant.

**Exemples** :
- `verify_i9_deterministic_pricing()`
- `verify_i1_conservation()`
- `verify_t5_9_seizure_chain()`

---

## Règles de maintenance

1. **Ajouter une entrée** seulement si elle est citée ≥ 2 fois dans une session
2. **Supprimer une entrée** si elle n'est pas lue dans 3 sessions consécutives
3. **Ne jamais dupliquer** avec `solutions-dev.md`, la spec, ou le cadrage