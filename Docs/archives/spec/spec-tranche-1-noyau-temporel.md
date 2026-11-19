# Spécification détaillée — Tranche 1 : Noyau temporel

**Mode** : JEU  
**Palier** : P1 (Quartier vivant)  
**Réference** : `Docs/spec P1 - quartier vivant.md` §13 (spécification) et §29 (ordre de mission)  
**Statut** : Prêt pour implémentation  
**Auteur** : Agent (session 2024-11-13)  
**Validation humaine requise** : Non (spécification pure, pas de design)  

---

## 🎯 Objectif de la tranche

**Livrer** l’ossature minimaliste du moteur de simulation :
- Horloge logique (jours).
- File d’événements ordonnée de manière **canonique** et **déterministe**.
- Mécanisme de **report** des événements générés pendant le traitement.
- État **sérialisable** et reprise depuis snapshot.
- Harnais de tests headless (T1.1 à T1.7).

**Interdits absolus** (order de mission §29.1) :
- ❌ Aucune sémantique métier (individus, firmes, biens).
- ❌ Aucune horloge réelle (`Date::now`, `time`, etc.).
- ❌ Aucun flottant dans l’état canonique.
- ❌ Aucune itération sur `HashMap` dans l’état ou la sérialisation.
- ❌ Aucune exécution réentrante.
- ❌ Aucun parallélisme (P1 est séquentiel).
- ❌ Aucun rendu.

---

## 📦 Entités concernées

| Entité | Champs | Rôle | Persistance |
|--------|--------|------|-------------|
| `ÉtatNoyau` | horloge: u64, compteur_id: u64, rng: [ÉtatRng; NB_FLUX], file: TasMin<Événement> | État global du noyau | Sérialisée |
| `CléÉvénement` | jour: u64, phase: u8, priorité: i32, id: u64 | Clé de tri totale | Non persistante (reconstruite à la désérialisation) |
| `Événement` | clé: CléÉvénement, type: IdType, payload: Payload | Événement à traiter | Non persistante |
| `RegistresSystèmes` | (vide en tranche 1) | Extensible pour les tranches suivantes | Sérialisée |

**Notes** :
- `IdType` et `Payload` sont **opaque** pour le noyau : définis par les couches métiers (tranche 2+).
- `NB_FLUX` = nombre de flux RNG déclarés (à calibrer, voir registre consolidé O10.2).

---

## 🔄 Événements concernés

### Types d’événements
Le noyau **ne connaît pas** les types d’événements. Il manipule uniquement :
- Une **clé de tri** (`CléÉvénement`).
- Un **identifiant de type** (`IdType`, entier).
- Un **payload** (opaque, sérialisable).

**Responsabilité des couches métiers** (tranche 2+) :
- Définir les `IdType` et leur sérialisation/désérialisation.
- Définir les priorités par type (en données).
- Appeler `générer()` avec le bon `type` et `payload`.

### Phases (ordonnancement)
Les 7 phases sont **fermées** (spec P1 §5) :
1. **Flux physiques** (import, production, consommation, seuils).
2. **Marchés** (transaction, reprice, rupture, faillite).
3. **Comptes** (salaires, loyers, prêts, impôts).
4. **Marché du travail** (offres, embauches, licenciements).
5. **Décisions d’agents** (Q(a), doctines, budget temps).
6. **Social** (griefs, grèves, reconnaissance, états, rituels).
7. **Journal** (compactage).

**Ordre canonique** :
`(jour, phase, priorité, id)` — **total** (pas d’ex æquo possible grâce à `id` unique).

---

## 🛡️ Invariants à respecter (I1-I9 applicables)

| ID | Invariant | Vérification | Criticité |
|----|-----------|--------------|-----------|
| I1 | **Déterminisme** : Même graine + mêmes données → même journal, bit à bit (y compris après snapshot). | Tests T1.2, T1.5, T1.6 | Bloquant |
| I9 | **Sérialisation canonique** : Deux états logiquement égaux → octets identiques. | Test T1.6 | Bloquant |
| I-Noyau-1 | **Clé totale** : Pas de collision possible (id unique + ordre lexicographique). | Test T1.1 | Bloquant |
| I-Noyau-2 | **Report strict** : Tout événement généré a une date **strictement postérieure** à l’instant courant. | Test T1.4 | Bloquant |
| I-Noyau-3 | **Pas de réentrance** : `exécuter` ne peut pas appeller `extraire` ou `insérer` directement. | Revue de code | Bloquant |

---

## ✅ Critères de réussite

### Fonctionnels
1. **Ordre canonique** : 10 000 événements aléatoires du même jour → extraction dans l’ordre strict `(phase, priorité, id)`.
2. **Permutation d’insertion** : Même ensemble d’événements insérés dans 10 ordres différents → **journaux identiques bit à bit** (I9).
3. **Report** : Un type d’événement auto-régénérant → **exactement un événement par instant**, avance d’un jour par cycle, arrêt propre à l’horizon.
4. **Date passée** : Générer un événement daté ≤ instant courant → **rejet par invariant**, arrêt du replai.
5. **Snapshot/reprise** : 100 jours simulés → snapshot → 100 jours → **journal de 200 jours identique bit à bit** à la simulation directe.
6. **Sérialisation canonique** : Même état logique construit dans deux ordres différents → **octets identiques**.
7. **Benchmark** : 1 million d’insertions + extractions en file chargée ; charge synthétique sur 300 ans → **temps sous seuil à mesurer**, taille de file bornée et stable.

### Non-fonctionnels
- **Déterminisme** : Respect des conventions 1-5 (entiers, pas de `HashMap`, clés stables, etc.).
- **Observabilité** : Journal causal complet (convention 27) ; tableau de bord textuel avec métriques de performance.
- **Tests headless** : Tous les tests T1.1 à T1.7 **verts en continu** (convention 26 : assertions actives en debug).

---

## 🏗️ Plan d’implémentation (6 étapes)

### Étape 1 : Types et structures de base
**Livrable** :
- `CléÉvénement` (u64, u8, i32, u64).
- `Événement` (clé + type + payload opaque).
- `ÉtatNoyau` (horloge, compteur_id, RNGs, file, registres).
- `TasMin<Événement>` (min-heap sur `CléÉvénement`).

**Tests** :
- T1.1 : Ordre canonique sur 10 000 événements.

**Estimation** : 1-2 sessions.

---

### Étape 2 : Boucle principale et report
**Livrable** :
- `fn avance_journée(&mut self, systèmes: &mut Systèmes) -> JournalDelta` (boucle 13.5).
- `fn générer(&mut self, type, payload, date, instant_courant) -> Résultat<(), Rejet>` (report 13.3).
- `fn prochain_instant((j, p)) -> (u64, u8)`.

**Comportement** :
- Extraire tous les événements de jour ≤ J dans l’ordre canonique.
- Exécuter chaque événement → système métier peut appeler `générer()`.
- `générer()` :
  - Si `date` absente → `prochain_instant(instant_courant)`.
  - Si `date` ≤ `instant_courant` → **Rejet** (invariant).
  - Sinon → allouer `id = compteur_id++`, insérer dans le tas.

**Tests** :
- T1.3 : Auto-régénération (1 événement/instant).
- T1.4 : Rejet des dates passées.

**Estimation** : 2-3 sessions.

---

### Étape 3 : RNG déterministes et allocation des IDs
**Livrable** :
- Implémentation des flux RNG (un par usage déclaré, convention 3).
- `compteur_id` jamais réinitialisé, jamais dérivé d’une taille de collection.

**Tests** :
- T1.2 : Permutation d’insertion → journal identique.

**Estimation** : 1 session.

---

### Étape 4 : Snapshot et sérialisation canonique
**Livrable** :
- `fn snapshot(&self) -> Bytes` : sérialisation canonique (ordre des champs fixé, boutisme fixé, file dans l’ordre de la clé).
- `fn restaurer(bytes: Bytes) -> ÉtatNoyau` : désérialisation + reconstruction du tas depuis la liste ordonnée.
- En-tête : version du format + version des données + graine.

**Tests** :
- T1.5 : Reprise bit à bit.
- T1.6 : Octets identiques pour états logiquement égaux.

**Estimation** : 2-3 sessions.

---

### Étape 5 : Harnais headless et benchmarks
**Livrable** :
- Générateurs de charge synthétique (1M événements, 300 ans).
- Mesures : temps d’exécution, taille de file, mémoire.
- Replai de référence (archivé en lecture seule).

**Tests** :
- T1.7 : Benchmark (1M événements en < seuil, file stable).

**Estimation** : 1-2 sessions.

---

### Étape 6 : Revue et validation
**Livrable** :
- Rapport de benchmark.
- Vérification que **aucune ligne de sémantique métier** n’est présente.
- Validation humaine : **toute divergence avec le replai de référence exige une validation explicite** (convention 6).

**Critères d’acceptation** :
- T1.1 à T1.7 verts.
- I9 vert (déterminisme).
- Replai de référence en lecture seule.

**Estimation** : 1 session.

---

## 📊 Données et calibrage

**Valeurs utilisées** (extrait du registre consolidé) :
| ID | Paramètre | Valeur | Origine | Statut |
|----|-----------|--------|---------|--------|
| T1.5 | Jours/mois | 30 | spec P1 §8 | **À calibrer** (impact rendu uniquement) |
| O10.2 | Plafond taille file (300 ans) | À mesurer | spec P1 §13.6 | **À calibrer** (benchmark) |
| NB_FLUX | Nombre de flux RNG | 1 (par défaut) | Conventions §3 | **À calibrer** (selon besoins) |

**Impact** :
- La tranche 1 **n’utilise pas** les paramètres économiques (prix, salaires, etc.) — ceux-ci sont gérés par les couches métiers (tranche 2+).
- Seuls les paramètres **temporels** et **de performance** sont pertinents.

---

## 🧪 Tests headless (T1.1 à T1.7)

### T1.1 : Ordre canonique
**Montage** : 10 000 événements aléatoires du même jour, insérés dans un ordre aléatoire.
**Critère** : Extraction dans l’ordre strict `(phase, priorité, id)`.

### T1.2 : Permutation d’insertion
**Montage** : Même ensemble d’événements, 10 ordres d’insertion différents.
**Critère** : Journaux identiques **bit à bit** (I9).

### T1.3 : Report
**Montage** : Un type d’événement qui se régénère lui-même.
**Critère** : Exactement un événement par instant, avance d’un jour par cycle, arrêt propre à l’horizon, **aucune boucle intra-instant**.

### T1.4 : Date passée
**Montage** : Générer un événement daté ≤ instant courant.
**Critère** : Rejet par invariant, **arrêt du replai**.

### T1.5 : Snapshot/reprise
**Montage** : 100 jours simulés → snapshot → 100 jours.
**Critère** : Journal de 200 jours **identique bit à bit** à la simulation directe.

### T1.6 : Sérialisation canonique
**Montage** : Même état logique construit dans deux ordres différents.
**Critère** : Octets **identiques**.

### T1.7 : Benchmark
**Montage** : 1 million d’insertions + extractions ; charge synthétique sur 300 ans.
**Critère** : Temps sous seuil à mesurer, taille de file **bornée et stable**.

---

## ⚠️ Pièges déclarés (à vérifier en revue de code)

| Piège | Description | Vérification | Impact |
|-------|-------------|--------------|--------|
| P1 | Itération sur `HashMap` | Revue de code | **Bloquant** (convention 2) |
| P2 | Source de temps réelle (`Date::now`) | Grep `time`, `now`, `clock` | **Bloquant** |
| P3 | Réentrance | Vérifier que `exécuter` n’appelle pas `extraire`/`insérer` | **Bloquant** |
| P4 | Compteur d’id réinitialisé | Revue de l’implémentation de `compteur_id` | **Bloquant** |
| P5 | Tas restauré depuis ordre arbitraire | Vérifier que la désérialisation reconstruit le tas depuis la liste ordonnée | **Bloquant** |
| P6 | Flottants dans l’état | Grep `f32`, `f64` dans l’état | **Bloquant** (convention 1) |

---

## 📌 Dépendances et intégration

### Dépendances externes
- **Aucune** : La tranche 1 est **autonome** et ne dépend d’aucune autre tranche.
- Les couches métiers (tranche 2+) **dépendent** de la tranche 1, mais pas l’inverse.

### Intégration avec les tranches suivantes
- **Tranche 2** (identités et journal) : Utilisera `générer()`, `avance_journée()`, et la sérialisation.
- **Tranche 3+** : Ajouteront des `IdType` et des systèmes métiers, **sans modifier le noyau**.

---

## 📝 Historique

| Date | Modification | Auteur |
|------|--------------|--------|
| 2024-11-13 | Création initiale | Agent |

---

## ✅ Validation

**À faire avant implémentation** :
- [x] Spécification validée par les conventions 9 et 25.
- [x] Toutes les références aux sections de la spec P1 vérifiées.
- [x] Invariants I1, I9, I-Noyau-1 à 3 identifiés.
- [x] Critères de réussite alignés sur les tests T1.1 à T1.7.
- [ ] **Validation humaine** : Aucune décision de design, donc pas de validation requise (ordre de mission §29).

**Prochaine étape** :
**Lancer l’implémentation de l’Étape 1 (Types et structures de base).**

---

## 📎 Annexes

- **Spécification source** : `Docs/spec P1 - quartier vivant.md` §13, §29.
- **Conventions applicables** : 1-5 (déterminisme), 7 (tests headless), 9-10 (méthode de travail), 25-27 (spécification et observabilité).
- **Registre des calibrages** : `Docs/registre-consolide-a-calibrer.md` (paramètres temporels uniquement pour cette tranche).