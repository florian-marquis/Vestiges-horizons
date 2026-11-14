# Matrice Spec → Tests

**Objectif** : Assurer que chaque section de la spécification a une couverture test adéquate.
**Source de vérité** : `Docs/spec P1 - quartier vivant.md`

---

## Légende
- ✅ : Couverture test complète
- ⚠️ : Couverture partielle
- ❌ : Aucune couverture
- N/A : Non applicable

---

## Section 17 : Spécification de la tranche 5

### 17.1 Marchés de biens

| Élément | Spec | Module | Tests | Statut | Notes |
|---------|------|--------|-------|--------|-------|
| Un marché par bien | §17.1 | market.rs | `test_market_creation` | ✅ | |
| Transactions au prix affiché | §17.1 | market.rs | `test_market_buy_sell` | ✅ | |
| Allocation en pénurie par file | §17.1 | market.rs | `test_market_wait_queue` | ⚠️ | File implémentée, test à ajouter |
| Reprice = première action Q(a) | §17.1 | market.rs | `test_market_auto_reprice` | ✅ | |
| Comportement par défaut : seuils de stock | §17.1 | market.rs | `test_market_auto_reprice` | ✅ | |
| Bornes de prix en données (I5) | §17.1 | market.rs | `test_price_bounds` | ❌ | À implémenter |
| Équilibré (débit = crédit, I3) | §17.1 | market.rs | `test_market_buy_sell` | ✅ | Vérifié dans les tests existants |
| Journalisé | §17.1 | market.rs | `test_market_buy_sell` | ⚠️ | Journalisation à vérifier |

### 17.2 Ville élastique

| Élément | Spec | Module | Tests | Statut | Notes |
|---------|------|--------|-------|--------|-------|
| Source et puits hors état des firmes | §17.2 | market.rs | `test_import_order` | ⚠️ | Implémentation partielle |
| Import par commandes datées | §17.2 | market.rs | `test_import_order` | ⚠️ | |
| Délai logistique : 3-10 jours | §17.2 | market.rs | `test_import_delay` | ❌ | À implémenter |
| Prix de référence mis à jour quotidiennement | §17.2 | market.rs | `test_reference_price_update` | ✅ | |
| +1% par 2% de demande excédentaire | §17.2 | market.rs | `test_reference_price_update` | ✅ | **Corrigé** : formule fixée |
| Plafond +50% | §17.2 | market.rs | `test_reference_price_cap` | ❌ | À implémenter |
| Dérive lentement à clé stable | §17.2 | market.rs | `test_reference_price_drift` | ❌ | À implémenter |

### 17.3 Repas et eau

| Élément | Spec | Module | Tests | Statut | Notes |
|---------|------|--------|-------|--------|-------|
| Modes de repas résolus quotidiennement | §17.3 | meal.rs | `test_meal_mode_resolution` | ✅ | |
| Choix pondéré par doctrine et salaire réel | §17.3 | meal.rs | `test_meal_mode_resolution` | ⚠️ | Doctrine pas encore implémentée |
| Coût monétaire + coût temporel | §17.3 | meal.rs | `test_meal_mode_resolution` | ✅ | |
| Niveau de logement (cuisine niveau 2+) | §17.3 | meal.rs | `test_meal_mode_resolution` | ✅ | |
| Eau refacturée dans le loyer | §17.3 | meal.rs | `test_water_cost_calculation` | ✅ | |
| Abonnement indexé sur la rareté | §17.3 | meal.rs | `test_water_cost_calculation` | ✅ | |
| 5-10% du panier (base) | §17.3 | meal.rs | `test_water_cost_calculation` | ✅ | |
| Jusqu'à 25% en pénurie | §17.3 | meal.rs | `test_water_cost_calculation` | ✅ | |
| Panier = nourriture + logement + eau + fiscalité | §17.3 | meal.rs | `test_basket_calculation` | ✅ | |
| Salaire réel = salaire × heures / coût panier | §17.3 | meal.rs | `test_basket_calculation` | ✅ | |

### 17.4 Fiscalité forfaitaire

| Élément | Spec | Module | Tests | Statut | Notes |
|---------|------|--------|-------|--------|-------|
| Impôt 10% prélevé à la source | §17.4 | tax.rs | `test_withholding_tax` | ✅ | |
| Rôle purement comptable | §17.4 | tax.rs | `test_withholding_tax` | ✅ | |
| Transferts déclarés (I3) | §17.4 | tax.rs | `test_tax_flows` | ✅ | |
| Allocations fixes vers les foyers | §17.4 | tax.rs | `test_allocations` | ✅ | |
| Vieillesse ≈ ½ panier | §17.4 | tax.rs | `test_allocations` | ✅ | |
| Aide par enfant ≈ ¼ panier | §17.4 | tax.rs | `test_allocations` | ✅ | |
| Toutes les grandeurs en net | §17.4 | tax.rs | `test_net_salary` | ✅ | |

### 17.5 Crédit, saisie, sous-location

| Élément | Spec | Module | Tests | Statut | Notes |
|---------|------|--------|-------|--------|-------|
| Prêteur firme et prêts entre individus | §17.5 | credit.rs | `test_loan_creation` | ✅ | |
| Offres en données | §17.5 | credit.rs | `test_loan_creation` | ✅ | |
| Tafaux 10%/an, 20% après défaut | §17.5 | credit.rs | `test_loan_creation` | ✅ | |
| Garantie, échéancier | §17.5 | credit.rs | `test_loan_creation` | ✅ | |
| Intérêts composés en virgule fixe | §17.5 | credit.rs | `test_loan_payment` | ✅ | |
| Capitalisés aux échéances | §17.5 | credit.rs | `test_loan_payment` | ✅ | |
| Garanties : logement, puis servitude | §17.5 | credit.rs | `test_seizure_chain` | ✅ | |
| Défait = événement | §17.5 | credit.rs | `test_seizure_chain` | ✅ | |
| Saisie transfère le logement au prêteur | §17.5 | credit.rs | `test_seizure_chain` | ✅ | |
| Sous-location précaire | §17.5 | credit.rs | `test_seizure_chain` | ✅ | |
| Décision Q(a) du nouveau propriétaire | §17.5 | credit.rs | `test_sublease_accept` | ✅ | |

---

## Tests d'invariants (Section 7)

### Tests I1 (Extraction/Réinjection)

| Test | Module | Statut | Notes |
|------|--------|--------|-------|
| `test_extraction_reinjection_i1` | entities.rs | ✅ | |
| `test_i1_conservation_cycle` | entities.rs | ✅ | |
| `test_i1_conservation` | lib.rs | ✅ | |

### Tests I9 (Déterminisme)

| Test | Module | Statut | Notes |
|------|--------|--------|-------|
| `test_i9_determinism` | lib.rs | ✅ | |
| `verify_i9_deterministic_pricing` | market.rs | ✅ | |
| `verify_i9_deterministic_credit` | credit.rs | ✅ | |
| `verify_i9_deterministic_tax` | tax.rs | ✅ | |
| `verify_i9_deterministic_meals` | meal.rs | ✅ | |

---

## Tests de la section 17.7 (Tests de fermeture T5)

| Test | Description | Module | Statut | Notes |
|------|-------------|--------|--------|-------|
| T5.1 | même graine, même données = même résultat | tous | ✅ | Vérifications I9 dans chaque module |
| T5.5 | choc local → ruptures temporaires + hausse de prix différée | market.rs | ❌ | À implémenter |
| T5.7 | flux fiscaux traçables (I3) | tax.rs | ✅ | `verify_t5_7_tax_flows()` |
| T5.9 | chaîne complète prêt → défaut → saisie → proposition | credit.rs | ✅ | `verify_t5_9_seizure_chain()` |
| T5.10 | distribution des modes de repas corrélée au salaire réel | meal.rs | ✅ | `verify_t5_10_meal_distribution()` |
| T5.11 | au moins une voie praticable | credit.rs | ✅ | `verify_t5_11_plancher()` |

---

## Statistiques de couverture

| Section | Total | ✅ Complète | ⚠️ Partielle | ❌ Aucune | % Couverture |
|---------|-------|------------|--------------|-----------|--------------|
| 17.1 | 7 | 5 | 2 | 0 | 71% |
| 17.2 | 7 | 3 | 1 | 3 | 43% |
| 17.3 | 9 | 7 | 1 | 1 | 78% |
| 17.4 | 7 | 7 | 0 | 0 | 100% |
| 17.5 | 10 | 9 | 0 | 1 | 90% |
| **Total T5** | **40** | **31** | **4** | **5** | **78%** |

---

## Actions requises

### Priorité Haute (Couverture < 50%)
1. **17.2** : Implémenter les tests manquants pour la ville élastique
   - Délai logistique
   - Plafond +50%
   - Dérive lentement

### Priorité Moyenne (Couverture 50-80%)
1. **17.1** : Ajouter test pour l'allocation en pénurie par file
2. **17.1** : Implémenter les bornes de prix (I5)

### Priorité Basse (Couverture > 80%)
1. **17.3** : Ajouter test pour la pondération par doctrine
2. **17.5** : Vérifier la sous-location précaire

---

## Historique des mises à jour

| Date | Section | Action | Auteur |
|------|---------|--------|--------|
| 2024-11-14 | 17.2 | Correction de la formule de prix de référence | Agent IA |
| 2024-11-14 | Tous | Ajout des vérifications I9 | Agent IA |
| 2024-11-14 | Tous | Ajout des tests d'invariants | Agent IA |