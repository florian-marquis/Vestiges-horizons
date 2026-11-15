# Registre consolidé des valeurs "à calibrer" — P1 Quartier vivant

**Source de vérité** : Ce registre fusionne toutes les valeurs marquées "à calibrer" dans `Docs/spec P1 - quartier vivant.md` et les documents annexes.
**Règle applicable** : Convention 9 — "Spécification d'abord : chaque tâche commence par un texte court (entités, événements, invariants, critères de réussite)".
**Priorité** : Ce registre doit être **complété et validé avant le lancement de la Tranche 1** (noyau temporel).
**Statut** : Vivant — à mettre à jour à chaque session de spécification.

---

## 📌 Méthodologie de calibrage

1. **Valeur par défaut** : Utiliser les hypothèses de la spec P1 section 8 (Données initiales) comme point de départ.
2. **Validation** : Chaque valeur doit être **testable** via les scénarios de choc (section 9 de la spec P1).
3. **Documentation** : Toute modification doit être justifiée par un **test headless** (convention 7) ou un **benchmark** (convention 21).
4. **Hiérarchie** : En cas de conflit, la spec P1 prime sur ce registre. Les décisions tranchées (ex. : décision 64, 66) **ne sont pas à calibrer**.

---

## 🏗️ 1. Structure temporelle et créneaux

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier de tranchage | Tests associés |
|----|-----------|------------------|---------|--------|----------------------|----------------|
| T1.1 | Nombre de demi-heures par jour | 32 | spec P1 §4 | **À calibrer** | P1 | Équilibre temps de survie (≤12h/jour) |
| T1.2 | Perte d'éveil par jeûne | 1 demi-heure/jour et demi de jeûne | spec P1 §4.3 | **À calibrer** | P1 | Scénario 7 (chute salaire réel) |
| T1.3 | Fatigue supplémentaire (travail de nuit) | +1 demi-heure de fatigue par créneau de nuit | spec P1 §4.3 | **À calibrer** | P1.5 | Scénario 8 (grève) |
| T1.4 | Nombre de créneaux par jour | 4 (matin, après-midi, soir, nuit) | spec P1 §4.3 ; v7.8.1 | **Validé** | P1 | Instant logique, pas simple étiquette |
| T1.5 | Jours par mois | 30 | spec P1 §8 | **À calibrer** | P1 | Calendrier pour le rendu (hors état canonique) |
| T1.6 | Durée réelle d'un créneau (régime scène) | 45 à 90 s | spec P1b §12 | **À calibrer** | P1b | T11b.1 |
| T1.7 | Durée réelle d'un jour (accéléré) | 5 à 10 s | spec P1b §12 | **À calibrer** | P1b | T11a.8 |
| T1.8 | Coût temporel d'un changement de lieu | 1 demi-heure | spec P1b §12 | **À calibrer** | P1b | T11a.5 |
| T1.9 | Créneau naturel par type d'événement | table en données (défaut 0) | spec P1b §12 ; décision 70 | **À calibrer** | P1b | T11a.1 puis validation humaine (convention 6) |

---

## 👥 2. Population et démographie

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| P2.1 | Individus initiaux | 150 (60 journaliers, 60 ouvriers, 30 patrons/retirés) | spec P1 §8 | **Validé** | P1 | Scénario 1 (équilibre initial) |
| P2.2 | Cohortes initiales | 3 | spec P1 §8 | **À calibrer** | P1 | Regroupement des cohortes proches |
| P2.3 | Âge d'extraction (individu nommé) | ~14 ans | spec P1 §8 | **À calibrer** | P1 | Critère d'extraction (seuils de richesse/griefs) |
| P2.4 | Plafond individus nommés simultanés | 30 | spec P1 §8 | **À calibrer** | P1 | Performance file d'événements |
| P2.5 | Contribution enfant (8 ans) | 0,5 demi-heure/jour | spec P1 §6.14 | **À calibrer** | P1 | Équilibre temps foyer |
| P2.6 | Contribution enfant (12 ans) | 1,5 demi-heure/jour | spec P1 §6.14 | **À calibrer** | P1 | Équilibre temps foyer |
| P2.7 | Contribution enfant (14 ans) | Plein régime domestique | spec P1 §6.14 | **À calibrer** | P1 | Équilibre temps foyer |
| P2.8 | Plafond contribution enfantine | 2 demi-heures/jour | spec P1 §6.14 | **À calibrer** | P1 | Éviter incitation démographique |

---

## 💰 3. Économie et marchés

### 3.1 Prix et coûts

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| E3.1.1 | Prix nourriture (haute qualité) | 400/j | spec P1 §8 | **À calibrer** | P1 | Scénario 2 (choc d'offre) |
| E3.1.2 | Prix nourriture (basse qualité) | 300/j | spec P1 §8 | **À calibrer** | P1 | Scénario 2 |
| E3.1.3 | Prix cantine | 350/j | spec P1 §8 | **À calibrer** | P1 | Scénario 7 |
| E3.1.4 | Prix restaurant | 600/j | spec P1 §8 | **À calibrer** | P1 | Scénario 7 |
| E3.1.5 | Prix étuve | 100/visite | spec P1 §8 | **À calibrer** | P1 | Équilibre initial |
| E3.1.6 | Loyer (niveau 2) | 500/j | spec P1 §8 | **À calibrer** | P1 | Scénario 6 (concurrent) |
| E3.1.7 | Loyer (niveau 1) | À moduler | spec P1 §8 | **À calibrer** | P1 | Accessibilité |
| E3.1.8 | Loyer (niveau 3) | À moduler | spec P1 §8 | **À calibrer** | P1 | Luxe |
| E3.1.9 | Consultation (dispensaire) | 200 | spec P1 §8 | **À calibrer** | P1 | Scénario 10 (crise crédit) |
| E3.1.10 | Coût du panier | nourriture ×1/j + logement ×1/j (adulte) | spec P1 §4.4 | **À calibrer** | P1 | Référence pour salaire réel |
| E3.1.11 | Réduction panier (enfant) | 50% | spec P1 §4.4 | **À calibrer** | P1 | Équilibre foyer |

### 3.2 Salaires

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| E3.2.1 | Salaire journalier | 60/demi-journée | spec P1 §8 | **À calibrer** | P1 | Scénario 8 (grève) |
| E3.2.2 | Salaire ouvrier | 90/demi-journée | spec P1 §8 | **À calibrer** | P1 | Scénario 8 |
| E3.2.3 | Salaire qualifié | 140/demi-journée | spec P1 §8 | **À calibrer** | P1 | Scénario 8 |
| E3.2.4 | Salaire vidangeur | 80/demi-journée (prime de risque) | spec P1 §8 | **À calibrer** | P1 | Attractivité |
| E3.2.5 | Référence de calibrage | Panier à 900/j | spec P1 §8 | **À calibrer** | P1 | Salaire réel cible |

### 3.3 Élasticité et reprice

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| E3.3.1 | Pente élasticité ville | +1% prix / +2% demande excédentaire | spec P1 §4.4 | **À calibrer** | P1 | Scénario 2 |
| E3.3.2 | Plafond élasticité | +50% | spec P1 §4.4 | **À calibrer** | P1 | Scénario 2 |
| E3.3.3 | Délai réapprovisionnement | 3-10 jours (selon bien) | spec P1 §4.4 | **À calibrer** | P1 | Scénario 2 |
| E3.3.4 | Seuil reprice (±) | 10% du prix courant | spec P1 §8 | **À calibrer** | P1 | Dynamique marché |
| E3.3.5 | Seuil stock bas | 25% du stock de référence | spec P1 §8 | **À calibrer** | P1 | Rupture |
| E3.3.6 | Seuil stock haut | 75% du stock de référence | spec P1 §8 | **À calibrer** | P1 | Surstock |

---

## 🏠 4. Logement et foyer

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| H4.1 | Foyers initiaux | 45 | spec P1 §8 | **Validé** | P1 | |
| H4.2 | Logements par niveau | 15 (niveau 1, 2, 3) | spec P1 §8 | **Validé** | P1 | |
| H4.3 | Niveau 1 (sanitaires communs) | Sans cuisine | spec P1 §8 | **Validé** | P1 | |
| H4.4 | Temps de survie max | ≤12 demi-heures/jour | spec P1 §8 | **À calibrer** | P1 | Garde-fou |
| H4.5 | Coût parrainage | 2 demi-heures/jour | spec P1 §8 | **À calibrer** | P1 | Essai +50% probabilité |
| H4.6 | Eau (fraction du loyer) | 5-10% du panier | spec P1 §8 | **À calibrer** | P1 | Pénurie jusqu'à 25% |

---

## 🏭 5. Firmes et production

### 5.1 Usure

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| F5.1.1 | Usure atelier | 80 points/millé/an | spec P1 §8 | **À calibrer** | P1 | Scénario 4 (hausse intrant) |
| F5.1.2 | Usure commerce | 40 points/millé/an | spec P1 §8 | **À calibrer** | P1 | Scénario 4 |
| F5.1.3 | Usure parc immobilier | 60 points/millé/an | spec P1 §8 | **À calibrer** | P1 | Scénario 4 |

### 5.2 Recettes multi-postes

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| F5.2.1 | Couverts/serveur/service | 8-15 | spec P1 §5.1 | **À calibrer** | P1 | Goulot production |
| F5.2.2 | Plats/cuisinier | 10-20 | spec P1 §5.1 | **À calibrer** | P1 | Goulot production |

---

## ⚖️ 6. Santé et états individuels

### 6.1 Maladie

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| S6.1.1 | Risque maladie (nourriture basse qualité) | ×2 | spec P1 §8 | **À calibrer** | P1 | Scénario 1 (équilibre) |
| S6.1.2 | Risque maladie (densité foyer élevée) | ×1,5 | spec P1 §8 | **À calibrer** | P1 | Scénario 1 |
| S6.1.3 | Contagion (collègue malade) | +5 à +15% de risque | spec P1 §5.2 | **À calibrer** | P1 | Tranche 6 |
| S6.1.4 | Travail malade (allongement) | +20 à +50% durée | spec P1 §5.2 | **À calibrer** | P1 | Présentéisme |
| S6.1.5 | Séquelle (capacité) | -10 à -30 points | spec P1 §8 | **À calibrer** | P1 | Impact long terme |

### 6.2 Stress et santé mentale

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| S6.2.1 | Stress (seuil rendement) | À calibrer | spec P1 §5.3 | **À calibrer** | P1 | Scénario 8 |
| S6.2.2 | Stress (réduction apprentissage) | À calibrer | spec P1 §5.3 | **À calibrer** | P1 | Scénario 8 |
| S6.2.3 | Catharsis (réduction stress/échange) | -5 à -15 points | spec P1 §13.4 | **À calibrer** | P1 | Boucle parole → stress |
| S6.2.4 | Catharsis (décroissance) | ×0,5 par répétition | spec P1 §13.4 | **À calibrer** | P1 | Équilibre social |
| S6.2.5 | Angoisse (seuil épargne) | < 2 semaines de panier | spec P1 §8 | **À calibrer** | P1 | Comportement économique |
| S6.2.6 | Dépression (seuil stress) | Stress > seuil haut pendant 30 jours | spec P1 §8 | **À calibrer** | P1 | Scénario 11 |
| S6.2.7 | Dépression (incidence) | Sous seuil (à calibrer) | spec P1 §13.4 | **À calibrer** | P1 | Santé mentale |

---

## 💳 7. Crédit et fiscalité

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| C7.1 | Taux crédit (normal) | 10%/an | spec P1 §8 | **À calibrer** | P1 | Scénario 10 |
| C7.2 | Taux crédit (après défaut) | 20%/an | spec P1 §8 | **À calibrer** | P1 | Scénario 10 |
| C7.3 | Fiscal (impôt) | 10% à la source | spec P1 §8 | **À calibrer** | P1 | Scénario 11 |
| C7.4 | Allocation vieillesse | ≈ moitié du coût du panier | spec P1 §8 | **À calibrer** | P1 | Équilibre |
| C7.5 | Aide par enfant | ≈ quart du coût du panier | spec P1 §8 | **À calibrer** | P1 | Équilibre |

---

## 🎭 8. Social et reconnaissance

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| S8.1 | Demi-vie événement (reconnaissance) | 5 ans | spec P1 §8 | **À calibrer** | P1 | Dynamique sociale |
| S8.2 | Effet reconnaissance (poids) | ±10 | spec P1 §8 | **À calibrer** | P1 | Décisions Q(a) |
| S8.3 | Empathie (foyer) | 1,0 | spec P1 §8 | **À calibrer** | P1 | Contagion souffrance |
| S8.4 | Empathie (groupe informel) | 0,5 | spec P1 §8 | **À calibrer** | P1 | Contagion |
| S8.5 | Empathie (immeuble) | 0,3 | spec P1 §8 | **À calibrer** | P1 | Contagion |
| S8.6 | Empathie (connaissance) | 0,1 | spec P1 §8 | **À calibrer** | P1 | Contagion |
| S8.7 | Vocations (salaires méprisés) | Correction partielle | spec P1 §6.7 | **À calibrer** | P1 | Équilibre marché |
| S8.8 | Groupes informels initiaux | 10-15 cercles de 3-6 membres | spec P1 §8 | **À calibrer** | P1 | Réseau social |

---

## 🔄 9. Dynamique des cohortes

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| C9.1 | Niveau maîtrise (rendement) | Modulateur | spec P1 §5.1 | **À calibrer** | P1 | Productivité |
| C9.2 | Grèves (taux de base) | 0 | spec P1 §8 | **À calibrer** | P1 | Scénario 8 |
| C9.2 | Grèves (formule) | (G/100)² × (1 − salaire réel cible) | spec P1 §8 | **À calibrer** | P1 | Scénario 8 |

---

## 📊 10. Observation et journal

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| O10.1 | Plafond mémoire saillante | 20 références | spec P1 §4 | **À calibrer** | P1 | Performance |
| O10.2 | Taille file d'événements (plafond vigilance) | 300 ans | spec P1 §8 | **À calibrer** | P1 | Benchmark tranche 10 |
| O10.3 | Corpus templates par intention | 10-20 | spec P1 §13.4 | **À calibrer** | P1 | Rendu |

---

## 🎯 11. Critères de réussite P1

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| R11.1 | Durée tests headless | 10 ans simulés | spec P1 §9 | **Validé** | P1 | Tous scénarios |
| R11.2 | Nombre de graines | 5 minimum | spec P1 §9 | **Validé** | P1 | Tous scénarios |
| R11.3 | Incidence mort par faim | Sous seuil (à calibrer) | spec P1 §6.1 | **À calibrer** | P1 | Scénario 7 |

---

## 11. Immersion P1b (scène locale)

| ID | Paramètre | Valeur actuelle | Origine | Statut | Palier | Tests |
|----|-----------|------------------|---------|--------|--------|-------|
| I11.1 | Plafond de sprites par lieu | 24 | spec P1b §12 | **À calibrer** | P1b | T11a.4, T11a.7 |
| I11.2 | Densité minimale de vignettes / créneau (lieu joueur) | ≥ 1 en non-choc | spec P1b §12 | **À calibrer** | P1b | T11a.9 |
| I11.3 | Traversée du quartier à pied | ≤ 30 s réelles | spec P1b §12 | **À calibrer** | P1b | T11b.1 |
| I11.4 | Licence packs d'assets | CC0 recommandé | cadrage ch. 12 | **Ouvert** | P1b | Lot de 10 assets |

---

## 📝 Historique des modifications

| Date | Modification | Auteur | Motif |
|------|--------------|--------|-------|
| 2024-11-13 | Création du registre consolidé | Agent | Fusion des valeurs de spec P1 et documents annexes |
| 2026-11-15 | Ajout T1.6-T1.9 et section 11 (immersion P1b) | Agent | Spec P1b / v7.8.1 |

---

## 🔍 Index par origine

- **spec P1 §4** : T1.1, T1.2, T1.3, T1.4, E3.1.10, E3.1.11, O10.1
- **spec P1 §5** : F5.2.1, F5.2.2, S6.1.3, S6.1.4, S6.2.1, S6.2.2, C9.1
- **spec P1 §6** : S6.1.3, S6.1.4, S8.7
- **spec P1 §8** : Tous les paramètres de données initiales
- **spec P1b** : T1.6, T1.7, T1.8, T1.9, I11.1, I11.2, I11.3, I11.4
- **À vérifier** : Documents de travail révoltes/factions et économie spatiale

---

## ✅ Prochaines étapes

1. **Valider les valeurs par défaut** (section 8 de spec P1) comme base de travail.
2. **Lancer les scénarios de choc** (spec P1 §9) pour calibrer les paramètres critiques (prix, salaires, usure).
3. **Documenter les décisions** : Une fois une valeur calibrée, la déplacer vers la spec P1 ou les données, et la marquer comme "Validé" dans ce registre.
4. **Ne jamais coder sans calibrage validé** (convention 9).