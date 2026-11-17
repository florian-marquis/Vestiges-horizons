# Vestiges Horizons — Proposition corrective T7-T8 v1.3

**Date :** 17/11/2026
**Statut :** spécifié — garde-fou validé le 17/11/2026 (sous conditions : S0 préalable avant S1–S4) ; lots A, B, C, D en exécution (tranche A2 réaliste)
**Base de design :** `docs/cadrage vestiges horizons - v7.8.md` (révision `v7.8.2`), `jeu/Docs/Conventions du projet (pour agents).md` (v2.3)
**Remplace :** `jeu/Docs/spec/T7-T8 correctif - spec v1.2.md` sur l'équation de griefs, les règles réalistes de marché/consommation et les tests de fermeture 1 à 4 (les tests 5 à 9 et l'interface événementielle de la v1.2 sont conservés).

---

## Règles dures et périmètre

- **Replais de référence en lecture seule (règle 6) :** `jeu/src/replai/*.json` (T1–T5) ne doivent jamais être modifiés ni régénérés. Si un lot devait changer leur comportement : STOP immédiat, lister les replais touchés et attendre la validation humaine.
- **Exclusions strictes :** ne pas modifier `src/strike.rs`, le calcul du risque (`calculer_taux_risque`, `seuil_risque`, tranche B) dans `src/salary.rs`, `src/labor.rs`, ni I8 / extraction / réinjection.
  *(Note pour la tranche B uniquement : `calculer_taux_risque()` lit encore `salaire_reel` en direct et sera corrigé en tranche B.)*
- **Arithmétique entière et unicité (règle 26) :** zéro `f64`, zéro `exp()` dans le chemin de calcul. Chaque arrondi par division entière est explicitement déclaré (troncature vers zéro ici). Chaque grandeur possède une seule implémentation canonique.

---

## Lot A — Corrections techniques sans design

1. **Comptage exact des jours de griefs (`quartier_10ans`) :**
   Les compteurs `Jours G=0`, `Jours 0<G<100` et `Jours G=100` sont évalués sur `g_qmicro` exact (`g_qmicro == 0`, `0 < g_qmicro < 400_000_000`, `g_qmicro == 400_000_000`), jamais sur la valeur tronquée `griefs_points()`. Un grief positif de `0 < g_qmicro < 4_000_000` (sous 1 point entier) compte bien dans `0<G<100`.
2. **Seuils de reprice du marché alimentaire :**
   Supprimer l'initialisation factice `stock_thresholds = (0, u32::MAX)` qui neutralise `auto_reprice()`, et appliquer la règle de couverture du lot B.3.

---

## Lot B — Marché, reste à vivre et consommation réaliste

### B.1 Consommation partielle au foyer et calcul de `u`

- Un foyer de `N` personnes (`member_count()`) disposant de `R` rations (`R < N` en pénurie) partage à parts égales : chaque membre reçoit `R / N` ration ; aucune priorité n'est donnée aux enfants en P1. Fin du régime « tout ou rien ».
- Le taux de privation `u` d'une cohorte est calculé **après** la consommation effective du jour, sur ce qui a été réellement mangé (jamais sur le stock avant repas) :

```text
u = 0                                                   si besoin_total == 0
u = min(1000, ((besoin_total - rations_mangees) * 1000) / besoin_total)   (troncature entière)
```

### B.2 Budget nourriture canonique (Option 1 — Reste à vivre)

- Une fonction pure unique `budget_nourriture(cohorte, jour)` (non stockée) calcule en entiers le budget alimentaire journalier d'une cohorte ou personne :

```text
budget_nourriture = max(0, revenu - loyer - eau - impot)
```

- **Source unique du revenu :** `revenu` est lu depuis la **même source canonique** que celle utilisée pour le calcul de `p` (`SalaireReelEtGriefs` dans `src/salary.rs`). Aucun second calcul du revenu ne doit exister. Le salaire n'entre jamais directement dans `G`.
- **Cohérence du panier utilisé pour `p` :** le coût du panier par personne utilisé pour calculer `p` contient la ration au prix affiché `P` (`100` au prix de base, jamais l'ancien `×10 = 1 000`) et exactement les mêmes `loyer (110)`, `eau (20)` et `impôt (10 % du revenu)` que `budget_nourriture`, soit `cout_panier(c, P) = P + 110 + 20 + impot(c)`.
- **Impôt :** l'impôt forfaitaire de `10 %` utilise exclusivement le mécanisme forfaitaire existant de P1 (`src/tax.rs`, `TAX_RATE = 100` millièmes, §6.16 et §17.4 de `Docs/spec P1 - quartier vivant.md`) et `PanierComposante::Fiscalite` — aucun nouveau mécanisme fiscal n'est créé.
- **Neutralité I1 :** aucune écriture dans `Cohort.total_money` (pas d'épargne ni de crédit dans cette tranche) ; les tests de l'invariant I1 restent intacts.
- **Note `meal.rs` :** la constante `let budget = 10000;` dans `MealManager::resolve_all_meals` (`jeu/src/meal.rs:330`) ne se trouve pas sur le chemin alimentaire de `quartier_10ans.rs` et reste donc inchangée.

### B.3 Prix affiché de la nourriture

- Le prix affiché `P` du jour est fixé **avant** la résolution des achats du jour, uniquement à partir de la couverture de stock de la veille (`stock_veille / demande_nominale_jour`, aucun recalcul réentrant dans le même instant, règle 18).
- Lorsque la couverture tombe sous `5` jours de stock, `P` augmente progressivement en entiers jusqu'au plafond `3 × P_base` (`300` centimes pour `P_base = 100`). Lorsque le stock se reconstitue au-dessus des seuils de reprise, `P` redescend progressivement vers `P_base`. `P` reste toujours strictement positif et fini.

### B.4 Achat partiel et rationnement (Mode A)

- Suppression complète de l'ordre d'itération par index de cohorte (`enumerate`) et de tout privilège joueur.
- Pour chaque cohorte `c` :

```text
rations_payables(c) = min(besoin(c), budget_nourriture(c) / P)   (division entière par troncature)
```

- Si `sum(rations_payables) <= stock_disponible` : chaque cohorte est servie à hauteur de `rations_payables(c)`.
- Sinon (`sum(rations_payables) > stock_disponible`) :
  1. Part de base entière par cohorte : `floor(stock_disponible * rations_payables(c) / sum_rations_payables)` (calcul sur largeur `u128` / `u64` sans débordement, troncature entière).
  2. Répartition du reliquat `stock_disponible - sum_part_base` ration par ration selon le **plus grand reste** `(stock_disponible * rations_payables(c)) % sum_rations_payables` décroissant, puis en cas d'égalité par **identifiant stable croissant** (`EntityId`).
- Les cohortes dont `budget_nourriture(c) < P` obtiennent `rations_payables = 0` et décrochent en premier (les plus pauvres avant les plus aisées).

### B.5 Scénario `quartier_10ans` et Garde-fou de départ

- Paramètres physiques du marché alimentaire (fixés par la mission) :
  - Stock initial : `2 000` rations (≈ 4 jours pour 500 habitants).
  - Importations nominales : `500` rations/jour (zéro excédent structurel).
  - Capacité de stockage maximale : `5` jours (`2 500` rations).
  - Populations des 5 cohortes : `[60, 80, 100, 120, 140]` (total `500`, graine `42`, horizon `3 650` jours).
- **Constat du Garde-fou sur les anciennes données :** avec `HOURLY_WAGES = [100, 200, 300, 400, 500]`, `8 h/j` et `Logement 2 000 + Eau 500 + Fiscalité 300 = 2 800`, les cohortes 0 à 2 ont `budget_nourriture = 0` (`ratio = 0,00 < 1,0`) dès le premier jour sans choc → **STOP obligatoire**.
- **Données réalistes validées le 17/11/2026 (figées, ne jamais modifier après exécution) — voir Annexe ci-dessous :**
  - Prix de base `P_base = 100` (plafond `300`), charges fixes par personne/jour : `loyer = 110` (fixe même quand le logement s'use), `eau = 20`, `impôt = 10%` du revenu brut journalier (troncature entière, mécanisme forfaitaire P1).
  - Salaires horaires : `[35, 40, 50, 60, 70]` sur `8 h/j` → revenus moyens par habitant `[280, 320, 400, 480, 560]` (enfants et inactifs compris), impôts `[28, 32, 40, 48, 56]`, `budget_nourriture`/pers. `[122, 158, 230, 302, 374]`.
  - Ratio `budget_nourriture / P_base` sans choc : **`[1,22 ; 1,58 ; 2,30 ; 3,02 ; 3,74]`** (coût du panier au jour 0 : `[258, 262, 270, 278, 286]`, donc `p = 0` au jour 0 pour toutes les cohortes). Totaux par cohorte agrégés selon les tailles `[60, 80, 100, 120, 140]`.

### Tests de fermeture du Lot B

1. **Stock et conservation :** stock du marché toujours `>= 0` ; `rations achetées = sorties de stock = rations mangées + variation du stock foyer`.
2. **Invariance d'ordre (convention 18) :** permuter l'ordre des cohortes produit un état final et un journal d'événements strictement identiques.
3. **Contrainte budgétaire :** aucune cohorte ne dépense plus que `budget_nourriture`.
4. **Prix fini et réversible :** `P` reste fini, `<= 3 * P_base`, monte sous 5 jours de couverture puis redescend lors du retour du stock.
5. **Service complet au prix de base :** sans choc et à `P = P_base`, toutes les cohortes sont servies intégralement.
6. **Décrochage progressif à prix ×1,5 :** à `P = 150`, la cohorte la plus pauvre est servie partiellement tandis que la cohorte la plus aisée reste servie intégralement.
7. **Neutralité `total_money` :** zéro écriture dans `Cohort.total_money`, suite de tests I1 inchangée.
8. **Égalité de source revenu :** `budget_nourriture` et `p` lisent exactement la même source de revenu.
9. **Calcul de `u` post-repas :** `u` reflète le manque réellement subi après partage `R/N`, et diffère entre deux foyers recevant des rations différentes.

---

## Lot C — Équation de griefs révisée (cadrage v7.8.2)

### C.1 Équation et échelle entière

- Équation continue validée :

```text
dG/dt = 0,18 u + 0,02 p - 0,06 (q - 0,5)    (points/jour)
```

  *(Le double emploi de `p` et du salaire réel reste tranché : le salaire réel n'entre dans `G` que via `p`.)*
- Avec `u, p, q` en millièmes sur `[0, 1000]`, la pente entière en µpt/jour vaut :

```text
s = 180 * u + 20 * p + 60 * (500 - q)
```

- Un créneau (quart de journée) avance `g_qmicro` de `s` qµpt, borné dans `[0, 400_000_000]` qµpt (`i128` pour les produits intermédiaires, même gestion d'horodatage, d'ordre canonique et d'erreurs explicites que la v1.2).

### C.2 Tests de fermeture du Lot C (13 tests)

**Tests 1 à 8 (recalculés pour la v7.8.2, remplacent les tests 1 à 4 de la v1.2) :**

1. **Conversion exacte :** `(u=1000, p=q=0)` → `+210 000` µpt/j ; `(u=p=0, q=500)` → `0` µpt/j ; `(u=p=0, q=0)` → `+30 000` µpt/j ; `(u=p=0, q=1000)` → `-30 000` µpt/j ; `(u=1000, p=1000, q=0)` → `+230 000` µpt/j.
2. **Pénurie totale sans services :** `G=0`, `u=1000, p=0, q=0` → `99 960 000` µpt (`399 840 000` qµpt) après `476` jours ; plafond `400 000 000` qµpt au jour `477`.
3. **Pénurie totale avec services neutres :** `G=0`, `u=1000, p=0, q=500` → `99 900 000` µpt après `555` jours ; plafond au jour `556`.
4. **Pénurie partielle :** `G=0`, `u=300, p=0, q=500` → `99 954 000` µpt après `1 851` jours ; plafond au jour `1 852`.
5. **Sans faim ni services :** `G=0`, `u=p=0, q=0` → `99 990 000` µpt après `3 333` jours ; plafond au jour `3 334`.
6. **État neutre :** `u=p=0, q=500` → `G` reste strictement constant sur `3 650` jours.
7. **Apaisement :** depuis le plafond (`400 000 000` qµpt), `u=p=0, q=1000` → `10 000` µpt après `3 333` jours ; zéro au jour `3 334`.
8. **Choc de 90 jours et retombée :** depuis `G=0`, `u=1000, p=0, q=500` pendant `90` jours → `16 200 000` µpt (`16,2` points) ; puis `u=0, q=1000` → retour exact à zéro après `540` jours.

**Tests 9 à 13 (conservés à l'identique des tests 5 à 9 de la v1.2) :**

9. **Temps intrajournalier :** 4 créneaux séparés identiques à 1 avance directe d'1 jour.
10. **Changements d'entrée :** séquence horodatée identique à un oracle pas-à-pas indépendant.
11. **Bornes et erreurs :** `[0, 400 000 000]` qµpt, rejet explicite date rétrograde / entrée `> 1000`.
12. **Ordre canonique :** `(jour, créneau, phase, priorité, id)` indépendant de l'ordre d'insertion.
13. **Non-régression T1–T5 :** `test_replais` vert sans aucune modification des JSON de référence.

---

## Lot D — Scénarios de référence (headless, graine 42, 10 ans)

*Données figées :* celles de l'Annexe datée ci-dessous (validées le `17/11/2026`). Ne modifier ni les données ni les coefficients pour « faire joli ».

### Scénario S0 — Témoin nominal sans choc (OBLIGATOIRE avant S1–S4)

- **Paramétrage :** `10 ans` (`3 650` jours), graine `42`, aucun choc, stock initial `2 000` rations, importations `500`/j (avec pipeline d'approvisionnement initial pré-amorcé à `t=0` sur le délai logistique pour éviter une rupture artificielle aux jours 0–2), capacité de stockage max `5` jours (`2 500` rations).
- **Sorties obligatoires par cohorte :** `p` au jour 0 ; rapport `budget / prix de la ration` ; `u max` ; `G` à `1`, `5` et `10` ans (`jours 365, 1825, 3650`) ; jours `G=0`, `0<G<100`, `G=100` (sur `g_qmicro` exact) ; `q` min/moyen/max.
- **Règle d'arrêt S0 :** si une cohorte atteint `G=100` en moins de `3` ans (`1 095` jours) sans aucun choc → **STOP immédiat**. Ne retoucher ni les données ni les coefficients ; rapporter les mesures et poser à l'humain la question de la définition de `p` en situations concrètes (règle 34). Si S0 passe, enchaîner sur S1–S4.

### Scénarios S1 à S4 (si S0 validé sans déclenchement d'arrêt)

Pour chacun des 4 scénarios ci-dessous, produire le tableau complet de la spec v1.2 §5 (sur `g_qmicro` exact) enrichi de `u max`, `jours de rationnement`, `prix maximum de la nourriture` et `G maximum` :

| Scénario | Paramétrage | Sorties additionnelles exigées |
| --- | --- | --- |
| **S1** | Pénurie partielle : importations `÷2` pendant 90 j (`jours 200–289`), puis normales (`500`/j) | Série `u` jour par jour pendant le choc (cohorte pauvre + cohorte moyenne) |
| **S2** | Pénurie totale : importations à `0` pendant 90 j (`jours 200–289`), puis normales | Série `u` jour par jour pendant le choc (cohorte pauvre + cohorte moyenne) |
| **S3** | Dispensaire sous-dimensionné (soins `≈ 0,2` pour la cohorte la plus pauvre), sans faim | Tableau §5 + métriques étendues |
| **S4** | Logements non entretenus et soins qui baissent sur 10 ans, puis salaire gelé avec prix qui montent | Tableau §5 + métriques étendues |

---

## Annexe — Données figées du scénario (validées le 17/11/2026)

*Paramètres définitifs, consignés datés pour référence et interdits de retouche après lecture des résultats :*

- **Horloge travail et population :** `8 h/j` ; tailles de cohortes `[60, 80, 100, 120, 140]` (total `500` habitants).
- **Salaires horaires :** `[35, 40, 50, 60, 70]` (soit revenus journaliers par habitant `[280, 320, 400, 480, 560]`).
- **Nature du revenu :** « revenu par personne » = **revenu moyen PAR HABITANT** (enfants et inactifs compris), pas par travailleur.
- **Prix et charges par habitant/jour :** prix de base de la ration `P_base = 100` (plafond `300`) ; `loyer = 110` (fixe même quand le logement s'use — choix de scénario assumé) ; `eau = 20` ; `impôt = 10 %` du revenu (`tax::TAX_RATE = 100` millièmes, mécanisme forfaitaire P1 existant §6.16 / §17.4).
- **Cohérence `p` / `budget_nourriture` :** `cout_panier(c, P) = P + 110 + 20 + impot(c)` (ration au prix `P = 100` au jour 0, pas `1 000`) et `budget_nourriture(c) = max(0, revenu(c) - 110 - 20 - impot(c))`, tous deux branchés sur la même source de revenu dans `SalaireReelEtGriefs`.

## Points ouverts, non traités (à reporter dans le rapport final)

- Quartier de `500` habitants dans ce scénario contre `150` individus dans le cadrage.
- Pas d'épargne, pas de crédit, pas d'arriérés de loyer dans cette tranche.
- `calculer_taux_risque()` utilise `salaire_reel` en direct : à retirer en tranche B.
