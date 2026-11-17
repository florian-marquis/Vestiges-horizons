# T7-T8 · Tranche A2 — sources réelles de u et q, mesure de 10 ans (ordre de mission)

**Statut :** Phase 1 terminée techniquement (commit `5758e5a`), en attente de la revue humaine du tableau de 10 ans avant toute suite sur la tranche B.
**Base :** cadrage v7.8.1, spec T7-T8 v1.2 (§5 « Observabilité du calibrage »), conventions v2.3.
**Niveau de difficulté :** moyen (touche le marché et plusieurs modules). Escalade vers l'agent suivant après deux échecs, avec le diagnostic de `PROTOCOLE-ERREURS` §2.
**Ordre :** cette tranche précède la tranche B (grève). Pas de modification du risque pendant la mesure de G.

## 1. But

Alimenter G avec les entrées réelles du quartier P1 (et non des séries synthétiques), puis produire le tableau de la spec v1.2 §5 sur 10 ans. Ne modifie ni la pente de G, ni les coefficients, ni le risque, ni les grèves.

## 2. Phase 0 — audit en lecture seule (aucune modification)

Rendre compte de :
1. où `u`, `p`, `q` sont calculés aujourd'hui (fichier, fonction, type, présence de `f64`) ;
2. pour chaque cohorte, les grandeurs réellement présentes dans le cœur : demande, stock, production, importations, couverture ;
3. ce qui existe dans le cœur comme état de service (soins, logement, entretien) pouvant alimenter `q` ;
4. les décalages posés à la main par cohorte (`u + 25·index`, `p + 15·index`, `q × (1 + 0,05·index)`) et leurs doublons ;
5. les sections de la spec P1 consultées, les fichiers à toucher et les tests à écrire.

**S'arrêter ici et attendre le retour de l'architecte.**

## 3. Phase 1 — implémentation (après validation de D1)

**Règles de calcul** (entiers, millièmes `[0, 1000]`, aucune virgule flottante) :
- `u` = besoins alimentaires non servis ÷ demande. Nul tant que la couverture est positive. Demande nulle → `u = 0` (pas de division par zéro). Arrondi déclaré : troncature (convention 26).
- `q` = moyenne pondérée des taux de service réellement rendus : **soins** et **état des logements (entretien)**, décision D1 du 16/11/2026 (option 2). Les composantes et leurs poids sont une table de données : ajouter une composante plus tard = ajouter une ligne, sans changer le code de calcul. Poids de départ à calibrer : soins 50 %, logements 50 %. Une composante sans donnée réelle n'entre pas dans la moyenne et n'est pas comptée comme zéro.
- `p` : calcul existant du cœur, conservé ; vérifier seulement qu'il est entier en millièmes.
- Les cohortes diffèrent par leur état réel. Aucun décalage posé à la main.

**Événements :** `InputChanged` est émis par le cœur (marché, services), jamais par un benchmark. Il est émis à un franchissement (rupture ou retour de stock, changement de capacité d'un service, changement de prix au millième), au plus un par cohorte et par créneau. Aucun appel quotidien.

**Invariants :**
- `u`, `p`, `q` restent dans `[0, 1000]`.
- Une seule implémentation canonique par grandeur (convention 26).
- Aucun `f64` dans le chemin de calcul.
- Le calcul est en lecture seule sur le reste de l'état.

**Interdits :** modifier `src/strike.rs` et le risque dans `src/salary.rs` ; régénérer un replai de référence ; réutiliser `benchmark_t7_t8` ; ajouter une dépendance.

## 4. Tests headless

| # | Test |
| --- | --- |
| 1 | `u = 0` tant que la couverture est positive ; après rupture, `u` égale non servis ÷ demande, exactement, sur un cas non divisible (par exemple 7 non servis sur 19) |
| 2 | Demande nulle → `u = 0` ; `u`, `p`, `q` toujours dans `[0, 1000]` |
| 3 | Choc 2 (import ÷ 2 pendant 90 jours) : `u` reste nul tant que le stock tient, monte après rupture, redescend après retour des livraisons |
| 4 | `q` suit l'état des services : tous servis → maximum, aucun servi → zéro, cas intermédiaire vérifié à la main |
| 5 | Deux cohortes de situations différentes donnent des `u` et `q` différents sans aucun décalage codé |
| 6 | Recherche statique : aucun `f64` ni `exp()` dans les fichiers de calcul de `u`, `q`, `p` |
| 7 | Même graine, ordres d'insertion permutés : état et journal identiques (convention 18) |
| 8 | `test_replais` passe sans modification des JSON de référence |

## 5. Livrable de mesure

Nouveau scénario `quartier_10ans` : graine 42, 3 650 jours, avec le choc 2 de la spec P1. Il produit :
- le tableau de la spec v1.2 §5 (q min / moyen / max, jours G=0, 0<G<100, G=100, par cohorte) ;
- le nombre d'`InputChanged` par cohorte et le nombre de jours en rupture de stock.

Aucun seuil de réussite (convention 21). Aucune série synthétique.

## 6. Sortie de tranche

Compte rendu court, puis « Prochaine étape suggérée » : revue humaine du tableau. **Ne pas enchaîner sur la tranche B.** Commits selon les conventions 28-33.

## 7. Décision D1 (humaine) : de quoi dépend q dans le quartier ? — TRANCHÉE : option 2 (16/11/2026)

**Enrichissement futur :** possible par ligne de données (par exemple la sécurité avec la milice en P2). Chaque ajout change les valeurs de `q`, donc les trajectoires de G : il passe par une comparaison avant/après (convention 8) et une revue humaine, et reste soumis au gel des enrichissements de P1 tant que le cœur réduit n'est pas vert (cadrage v7.2).

Options examinées :

| Option | Effet visible | Avis de l'architecte |
| --- | --- | --- |
| 1. Soins seulement | Un dispensaire mal servi nourrit la colère, le reste ne compte pas | Trop pauvre : q reste presque constant |
| 2. Soins + état des logements (entretien) | Un quartier délabré ou sans soins s'aggrave lentement, même sans faim | **Recommandée** : correspond à la route 1 du cadrage (dette d'entretien → q → griefs) |
| 3. Soins + logement + sécurité | Idem, avec la sécurité | Prématuré : la sécurité vient avec la milice (P2) |
