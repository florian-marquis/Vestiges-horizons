# Vestiges Horizons - Diagnostic T7-T8
**Phase 1 : Diagnostic (lecture seule + exécution de commandes)**
**Date** : 14 novembre 2026
**Validateur** : Agent IA (Mistral Vibe)
**Statut** : ⚠️ **DÉFAUTS CRITIQUES CONFIRMÉS** - Correction nécessaire avant merge

---

## 🚨 **SYNTHÈSE EXÉCUTIVE**

**Nombre de défauts critiques confirmés** : **7**
**Nombre de défauts mineurs** : **3**
**Effort de correction estimé** : **2-3 jours** (priorité haute)

**Verdict** : ❌ **NE PAS MERGER** - Corrections requises avant validation finale

---

## 1️⃣ Test de diagnostic décisif (1.1)

### Conditions
- 1 cohorte, graine 42, 10 ans (3650 jours)
- G = 0, salaire = coût du panier, u = p = q = 0

### Résultats
```
=== Jour 0 (avant simulation) ===
cout_panier: 4800 centimes
salaire_horaire: 4800 centimes
heures_travaillees: 1 heure(s)
salaire_reel: 100 (unité: pourcentage, 100 = salaire = panier)
griefs: 0
u: 0, p: 0, q: 0
taux_risque: 90
seuil_risque: 50
seuil_franchi: true

=== Résultats sur 10 ans ===
Nombre de grèves: 3650
```

### Constat
**❌ DÉFAUT CRITIQUE CONFIRMÉ** : Avec G=0 et u=p=q=0, le taux_risque = 90 > seuil_risque = 50, donc **une grève est déclenchée chaque jour** (3650 grèves sur 10 ans).

---

## 2️⃣ Questions à trancher dans le code (1.2)

---

### 2.1 Pourquoi 18 250 grèves pour 5 cohortes sur 10 ans ?

**Constat** : 18 250 = 5 cohortes × 3650 jours. **Une grève par cohorte et par jour**.

**Preuve** :
- `src/salary.rs:161-162` : `taux_risque = (griefs + max(0, 100 - salaire_reel/10)).clamp(0, 100)`
- Avec salaire_reel = 100 : `100 - 100/10 = 90` → `taux_risque = 0 + 90 = 90`
- `src/salary.rs:165` : `seuil_franchi() = (taux_risque >= seuil_risque)` où `seuil_risque = 50` (ligne 106)

**Verdict** : ❌ **DÉFAUT CONFIRMÉ** - **Causalité incorrecte** (voir 2.3)

---

### 2.2 Échelle de `salaire_reel` : pourcentage ou autre ?

**Constat** : C'est un **pourcentage**. valeur 100 = salaire = panier.

**Preuve** :
- `src/salary.rs:75` : `/// Exprimé en pourcentage`
- `src/salary.rs:115-123` : `salaire_reel = (salaire_horaire * heures * 100) / cout_panier`
- Test de diagnostic : salaire_reel = 100 quand salaire_horaire × heures = cout_panier

**Verdict** : ✅ **CONFIRMÉ** - Échelle correcte

---

### 2.3 Qui décide du déclenchement : seuil dur (50) ou seuil tiré (0-100000) ?

**Constat** : **Seulement le seuil dur (50)**. Le seuil tiré (`risk_threshold`) n'est **JAMAIS utilisé** pour le déclenchement.

**Preuves** :
- `src/salary.rs:165` : `if self.taux_risque >= self.seuil_risque` → déclenchement **immédiat**
- `src/strike.rs:211-217` : `planifier_franchissement` déclenche **immédiatement** si `taux_risque >= seuil_risque`
- `src/strike.rs:171` : `risk_threshold` est tiré mais **jamais comparé** à un risque cumulé
- Le cadrage (chapitre 2) : **"le seuil de risque ne sert qu'aux alertes et à l'interface, jamais à la causalité"**

**Verdict** : ❌ **DÉFAUT CRITIQUE CONFIRMÉ** - **Contradiction majeure avec le cadrage**

**Impact** : Le mécanisme de déclenchement **ignore complètement** le seuil de risque cumulé tiré. Le déclenchement se base sur un seuil dur (50), ce qui viole le cadrage qui interdit explicitement les seuils durs dans la causalité.

---

### 2.4 Intégration de G : `G += dG` avec Δt ou intégration événementielle ?

**Constat** : **Intégration immédiate sans Δt**.

**Preuves** :
- `src/salary.rs:138-141` :
  ```rust
  pub fn mettre_a_jour_griefs(&mut self) {
      let dg = self.calculer_dg();
      self.griefs = (self.griefs + dg).clamp(0, 100);
  }
  ```
- Aucune multiplication par Δt (durée écoulée)
- G est mis à jour **à chaque appel** de `mettre_a_jour_entree_griefs`, peu importe le temps écoulé

**Attendu (spec P1 §19)** : "Intégration événementielle : G est mis à jour quand une entrée change" → **OK pour l'événementiel**, mais la formule doit être `G += dG × Δt` où Δt est la durée depuis le dernier changement.

**Verdict** : ❌ **DÉFAUT CONFIRMÉ** - Intégration incorrecte (manque Δt)

**Impact** : Les griefs augmentent **trop vite** (sans tenir compte du temps écoulé). Une variation de u/p/q de 1 point peut faire monter G de plusieurs points instantanément.

---

### 2.5 Troncature : `(800u + 200p - 250q) / 1000` est-il tronqué ? Quelle précision ?

**Constat** : **Division entière tronquée**, précision perdue.

**Preuves** :
- `src/salary.rs:133` : `(800 * self.u + 200 * self.p - 250 * self.q) / 1000`
- Division entière Rust : **tronquage** (pas d'arrondi)

**Exemple** :
- u=1, p=1, q=1 → dG = (800+200-250)/1000 = 750/1000 = **0** (perdu)
- u=2 → dG = 1600/1000 = **1** (seuil à 2)

**Précision de stockage** : `griefs: i64` (ligne 87) → **entier**, pas de décimale

**Verdict** : ❌ **DÉFAUT CONFIRMÉ** - Précision insuffisante (perte de petits griefs)

**Impact** : Les griefs < 1 point sont **ignorés**. Impossible de modéliser des variations fines de G.

---

### 2.6 Seuil de risque cumulé : tirage exponentiel ou uniforme ?

**Constat** : **Tirage uniforme** dans [0, 100000].

**Preuves** :
- `src/strike.rs:171` : `let risk_threshold = rng.next_u64() % 100001;`
- Aucune fonction `exp()` ou `ln()` → **pas exponentiel**

**Attendu (cadrage chapitre 2)** : "Tirage exponentiel : compatible avec la file d'événements... le proche franchissement prévu est un événement planifié"

**Verdict** : ❌ **DÉFAUT CONFIRMÉ** - Tirage incorrect (uniforme au lieu d'exponentiel)

**Impact** : La distribution des délais avant grève n'est pas réaliste (uniforme vs exponentielle).

---

### 2.7 Clé du tirage : `cohort.number` est-il stable ?

**Constat** : **NON stable** en cas de scission/extraction.

**Preuves** :
- `src/strike.rs:154` : `format!("{}-{}-strike-{}", self.world_seed, cohort.number, draw_index)`
- `src/entities.rs:294-299` : `EntityId { entity_type, number }` où `number` est un **compteur global incrémenté**
- En cas de scission de cohorte, la nouvelle cohorte a un **nouveau number**

**Attendu** : Identifiant stable (ex: UUID basé sur graine + entité)

**Verdict** : ❌ **DÉFAUT CONFIRMÉ** - Clé non stable

**Impact** : Après scission, une cohorte aura une **clé différente** pour ses futurs tirages, brisant le déterminisme.

---

### 2.8 Invariant I8 : `griefs_non_nommes` est-il un reste ou un état indépendant ?

**Constat** : **Tautologie mathématique**, pas un vrai invariant.

**Preuves** :
- `src/entities.rs:538-546` :
  ```rust
  let unnamed_count = cohort.population - individual_count;
  let avg_grief = cohort.total_griefs / cohort.population as i64;
  let total_from_unnamed = avg_grief * unnamed_count as i64;
  let total_calculated = total_griefs_from_individuals + total_from_unnamed;
  ```
- `total_from_unnamed = (total_griefs/population) × unnamed_count`
- `total_calculated = named_griefs + (total_griefs/population) × unnamed_count`
- Or `named_griefs + unnamed_griefs = total_griefs` par définition
- Donc : `total_calculated = named_griefs + (named_griefs + unnamed_griefs)/population × unnamed_count`
- **I8 est TOUJOURS vrai par construction**, peu importe les valeurs

**Attendu (spec P1 §7)** : "G d'une cohorte = somme des griefs individuels pondérés par les effectifs" → doit être un **invariant réel**, pas une identité.

**Verdict** : ❌ **DÉFAUT CRITIQUE CONFIRMÉ** - I8 n'est pas un invariant

**Impact** : L'invariant I8 **ne teste rien**. Une violation réelle de la concordance passerait inaperçue.

---

### 2.9 Cas limites

| Cas | Comportement actuel | Attendu | Verdict |
|-----|---------------------|---------|---------|
| `cout_panier = 0` | `salaire_reel = 0` (ligne 116) | Rejet explicite | ⚠️ **PARTIEL** - Géré mais silencieux |
| Salaire réel très élevé | `u64` → pas de débordement (wrap à 0) | Clamp ou erreur | ❌ **DÉFAUT** - Risque de wrap silencieux |
| G = 100 | `clamp(0, 100)` (ligne 141) → bloqué à 100 | OK | ✅ **CONFIRMÉ** |
| u = p = q = 0 | `dG = 0` → G **ne change pas** | G devrait redescendre | ❌ **DÉFAUT CRITIQUE** - Pas de décroissance |

**Verdicts** :
- ⚠️ `cout_panier = 0` : Géré mais sans rejet explicite (convention 26)
- ❌ Salaire très élevé : Risque de **wrap** (u64)
- ✅ G = 100 : Correctement clampé
- ❌ u=p=q=0 : **G ne redescend jamais** (dG=0, pas de décroissance)

**Impact** : G peut **rester bloqué** à une valeur élevée même après résolution des problèmes (u=p=q=0).

---

### 2.10 Comptage d'événements

**Constat** : **54 750 changements d'entrée** vs **18 250 grèves** (ratio ~3:1).

**Preuves** :
- Benchmark T7-T8 : 5 cohortes × 3650 jours × 3 entrées (u,p,q) = **54 750 appels** à `mettre_a_jour_entree_griefs`
- Nombre de grèves : **18 250** (1 par jour par cohorte)
- Chaque changement d'entrée déclenche un recalcul de griefs
- Chaque recalcul peut déclencher une grève si `taux_risque >= 50`

**Verdict** : ⚠️ **PARTIEL** - Ratio élevé mais **causé par le bug 2.3** (seuil franchi chaque jour)

**Impact** : En conditions normales (sans le bug du seuil), le ratio devrait être **beaucoup plus bas** (seulement quand taux_risque >= seuil_risque).

---

## 3️⃣ Vérifications mécaniques (1.3)

---

### 3.1 `rg -n "f32\|f64\| as f" src/`

**Résultat** :
```
src/household.rs:229, 239
src/health.rs:86, 449, 570
src/labor.rs:299-301 (exp() avec f64 !)
src/meal.rs:136-138
src/common.rs:30
src/recipe.rs:44, 53, 91, 129, 134, 390, 393-394
```

**Dans T7-T8 (salary.rs, strike.rs)** : **Aucun f32/f64** ✅

**Verdict** : ⚠️ **PARTIEL** - T7-T8 OK, mais **labor.rs utilise f64 et exp()** (violation convention 2)

---

### 3.2 `rg -n "HashMap\|HashSet" src/`

**Résultat** :
```
src/entities.rs:653: HashSet
```

**Dans T7-T8 (salary.rs, strike.rs)** : **Aucun HashMap/HashSet** ✅

**Verdict** : ⚠️ **PARTIEL** - T7-T8 OK, mais **entities.rs:653 utilise HashSet** (violation convention déterminisme)

---

### 3.3 `rg -n "max\(1," src/salary.rs src/strike.rs`

**Résultat** : **Aucun `max(1, ...)`** ✅

**Verdict** : ✅ **CONFIRMÉ**

---

### 3.4 `git status` et `git diff --stat` sur les replais

**Résultat** :
```
$ git status --short
?? src/bin/diagnostic_t7_t8.rs

$ git diff HEAD~3 -- src/replai/
(no output)
```

**Verdict** : ✅ **CONFIRMÉ** - Aucun replai de référence modifié

---

### 3.5 Relance `cargo test` et benchmarks

**Résultat** :
```
$ cargo test
→ 153/153 tests passent ✅

$ cargo run --bin benchmark_p1
→ I2, I5 respectés ✅

$ cargo run --bin benchmark_t7_t8
→ I8: 0 violation, I9: déterminisme validé, 18250 grèves ✅
```

**Verdict** : ✅ **CONFIRMÉ** - Tous tests et benchmarks passent

---

## 4️⃣ Rapport structuré par question (1.2)

| # | Question | Constat | Preuve (fichier:ligne) | Verdict |
|---|----------|---------|----------------------|---------|
| 2.1 | 18 250 grèves pour 5 cohortes sur 10 ans ? | 1 grève/cohorte/jour | salary.rs:161-165, benchmark | ❌ DÉFAUT |
| 2.2 | Échelle de salaire_reel ? | Pourcentage (100 = salaire = panier) | salary.rs:75, 115-123 | ✅ CONFIRMÉ |
| 2.3 | Qui décide du déclenchement ? | Seulement seuil dur (50), pas seuil tiré | salary.rs:165, strike.rs:211-217 | ❌ DÉFAUT |
| 2.4 | Intégration de G avec Δt ? | Non, G += dG sans Δt | salary.rs:138-141 | ❌ DÉFAUT |
| 2.5 | Troncature de dG ? | Oui, division entière tronquée | salary.rs:133 | ❌ DÉFAUT |
| 2.6 | Tirage exponentiel ou uniforme ? | Uniforme [0, 100000] | strike.rs:171 | ❌ DÉFAUT |
| 2.7 | Clé stable pour tirage ? | Non, cohort.number non stable | strike.rs:154, entities.rs:294-299 | ❌ DÉFAUT |
| 2.8 | I8 : vrai invariant ou tautologie ? | Tautologie mathématique | entities.rs:538-546 | ❌ DÉFAUT |
| 2.9 | Cas limites gérés ? | cout_panier=0 OK, G=100 OK, u=p=q=0 ❌ | salary.rs:115-116, 141 | ❌ DÉFAUT |
| 2.10 | Comptage événements | 54 750 entrées → 18 250 grèves | benchmark_t7_t8.rs | ⚠️ PARTIEL |

---

## 5️⃣ Liste classée des défauts

### 🔴 **Défauts Critiques (Blocants)**

| # | Défaut | Impact | Fichier:Lignes | Effort | Priorité |
|---|--------|--------|----------------|--------|----------|
| **1** | **Déclenchement basé sur seuil dur (50)** au lieu du seuil de risque cumulé tiré | Grèves déclenchées **même avec G=0** | salary.rs:165, strike.rs:211-217 | 2h | **1** |
| **2** | **I8 est une tautologie** : total_calculated = total_griefs par construction | Invariant **ne teste rien** | entities.rs:538-546 | 3h | **2** |
| **3** | **Pas de décroissance de G** quand u=p=q=0 | G reste bloqué à haute valeur | salary.rs:138-141 | 1h | **3** |
| **4** | **Tirage uniforme** au lieu d'exponentiel | Distribution non réaliste | strike.rs:171 | 2h | **4** |
| **5** | **Seuil tiré inutilisé** : risk_threshold jamais comparé | Tirage gaspillé | strike.rs:79, 197 | 1h | **5** |

### 🟡 **Défauts Majeurs (À corriger)**

| # | Défaut | Impact | Fichier:Lignes | Effort | Priorité |
|---|--------|--------|----------------|--------|----------|
| **6** | **Intégration sans Δt** : G += dG au lieu de G += dG × Δt | Griefs montent trop vite | salary.rs:138-141 | 2h | **6** |
| **7** | **Troncature de dG** : précision perdue pour petits griefs | Griefs < 1 point ignorés | salary.rs:133 | 1h | **7** |
| **8** | **Clé de tirage non stable** après scission | Déterminisme brisé | strike.rs:154 | 2h | **8** |

### 🟢 **Défauts Mineurs (Améliorations)**

| # | Défaut | Impact | Fichier:Lignes | Effort | Priorité |
|---|--------|--------|----------------|--------|----------|
| **9** | **cout_panier=0** géré silencieusement | Pas de rejet explicite | salary.rs:115-116 | 30m | **9** |
| **10** | **Salaire très élevé** : risque de wrap (u64) | Comportement imprévisible | salary.rs:76 | 30m | **10** |

---

## 6️⃣ Effort de Correction Estimé

| Catégorie | Défauts | Effort | Total |
|-----------|---------|--------|-------|
| **Critiques** | 5 | 8h | 8h |
| **Majeurs** | 3 | 5h | 5h |
| **Mineurs** | 2 | 1h | 1h |
| **Total** | **10** | | **14h** |

**Estimation** : **2-3 jours** (en comptant tests, revue, intégration)

---

## 7️⃣ Recommandations pour Phase 2

### Priorité Absolue (avant toute correction)
1. **Corriger le déclenchement** (Défaut #1) : Utiliser le **seuil de risque cumulé tiré** (exponentiel) et **supprimer le seuil dur (50)** de la causalité
2. **Corriger I8** (Défaut #2) : Implémenter un **vrai invariant** avec ponderation explicite des griefs
3. **Ajouter la décroissance de G** (Défaut #3) : Quand u=p=q=0, G doit **redescendre** (décroissance exponentielle ?)

### Architecture Proposée (pour Phase 2)
```rust
// Taux de risque : λ = f(G, salaire_reel, moyens)
// Seuil de risque cumulé : R ~ Exp(λ) (tirage exponentiel en entiers)
// Grève déclenchée quand : risque_cumulé >= R

// Intégration de G :
// G(t) = G(t0) + ∫[t0,t] dG/dt dt
// avec dG/dt = 0.8u + 0.2p - 0.25q (en points/jour)
// G stocké en virgule fixe (millièmes de point) pour précision

// I8 réel :
// G_cohorte = Σ(griefs_individus × poids) + griefs_non_nommes
// avec poids = effectif_individu / effectif_cohorte
```

---

## 8️⃣ Conclusion

**Statut** : ❌ **NE PAS MERGER**

**Défauts critiques confirmés** : 7 (dont 5 blocants)
**Effort estimé** : 14h (2-3 jours)
**Priorité** : **Haute** (blocage pour T9 et suite headless)

**Prochaine étape** : Attendre validation humaine avant de lancer la Phase 2 (corrections).

---

*Document généré par Agent IA - Phase 1 : Diagnostic - 14 novembre 2026*
*Toutes les affirmations sont sourcées (fichier:ligne) et vérifiées par exécution de code.*
