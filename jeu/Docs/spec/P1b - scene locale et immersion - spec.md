# Vestige : Horizon · Spécification P1b : scène locale et immersion

Document de spécification du premier jouable immersif (tranche 11 scindée). Références : Cadrage v7.8 / v7.8.1 (chapitres 7, 9, 12, 13, 14, 17), Spec P1 (quartier vivant), Conventions v2.3. « À calibrer » désigne une valeur d'hypothèse, jamais une décision.

## 1. Question fermatoire

**P1b est réussi si**, sur une graine donnée : le joueur se trouve dans un lieu pendant un créneau, y voit des vignettes strictement causales (chaque geste visible référence un événement du journal ou un état), y agit avec les mêmes tables que les PNJ, et le contrat headless `scene(...)` prouve que le rendu n'écrit jamais dans l'état, que la présence canonique égale les sprites affichés, et que l'empreinte d'état est identique avec ou sans scène, avec ou sans halte du théâtre local.

## 2. Périmètre

**Inclus** : lieu comme entité de présence ; présence (lieu, créneau) des nommés et agrégats de cohorte ; horodatage logique (jour, créneau) ; régimes de vitesse (scène, accéléré, saut) ; halte du théâtre local ; composition du créneau par le joueur ; interaction P1 réduit (observer, demander, témoin direct) ; mise en scène causale (table événement → vignette) ; figurants de cohorte ; contrat headless `scene` ; plan du quartier en version minimale (structure / peau / marqueurs) ; calques de sprite et marqueurs d'état pilotés par variables existantes ; écrans atteints par la promenade.

**Exclus (et c'est tranché)** : coordonnées continues, trajets simulés, pathfinding, objets et gestes de métier simulés (décision 66) ; vue détaillée du service (route candidate, entrée P2) ; parole à intention complète et écoute passive (P1.5, sections 24 et 27 de la spec P1) ; couplage des créneaux (fréquentation, primes, équipes tournantes, lits chauds : P1.5) ; estaminet (P1.5) ; portraits modulaires (P1.5) ; bibliothèque de rendu (choix humain, hors spec).

## 3. Entités canoniques

| Entité | Champs minimaux | Notes |
| --- | --- | --- |
| Lieu | id, type (firme, logement, rue, place), référence optionnelle (id firme ou logement), créneaux d'ouverture | Firmes et logements existants deviennent des lieux ; rue et place commune sont des lieux sans firme |
| PrésenceNamed | id individu, id lieu, créneau | Un nommé a exactement une présence par créneau ; dérivée de l'emploi du temps positionné |
| PrésenceCohorte | id cohorte, id lieu, créneau, effectif | Agrégat ; jamais d'individu caché |
| EmploiDuTemps | 32 demi-heures, chacune un Usage + id lieu | Positionné : chaque demi-heure porte un lieu ; les 32 demi-heures se regroupent en 4 créneaux de 8 |
| DescriptionDeScène | instant (jour, créneau), id lieu, observateur, sprites[], vignettes[], marqueurs[] | Produit dérivé, hors état canonique, recalculable |
| Vignette | id événement (ou référence d'état), type de geste, acteurs, durée relative | Jamais décorative ; chaque vignette porte l'id de sa cause |
| Figurant | clé stable, id cohorte, id lieu, créneau, calques dérivés | Tirés des PrésenceCohorte ; jamais des entités ; interaction durable → extraction (I1) |

**Variables partagées lues (jamais écrites par la scène)** : stocks, files de pénurie, dette d'entretien, créneaux d'ouverture, grève, états cachés des nommés (faim, fatigue, stress, ennui, douleur), niveau de logement, séquelle, peau du métier, présence.

## 4. Présence et déplacement

### 4.1 Grain canonique

L'état canonique ne connaît que « tel individu (ou tel effectif de cohorte) est dans tel lieu pendant tel créneau ». Aucune coordonnée, aucun trajet, aucun objet.

- Un lieu = firme, logement, ou rue/place commune.
- La marche, les trajectoires et les animations de déplacement sont du **rendu pur**, recalculables depuis (présences du créneau précédent, présences du créneau courant, graine), jamais sauvegardées.
- Changer de lieu pendant un créneau réserve des demi-heures de l'emploi du temps de ce créneau (coût temporel du trajet en données, typiquement 1 demi-heure). Ce que le joueur fait à l'écran est exactement ce que la simulation retient.

### 4.2 Symétrie joueur / PNJ

- Le personnage du joueur est un individu nommé standard.
- S'il n'est pas à l'atelier pendant le créneau de son poste, il n'a pas travaillé : mêmes conséquences (salaire, engagement, Q(a)) qu'un PNJ absent.
- Aucune table d'actions réservée au joueur.

### 4.3 Figurants

- Les non-nommés n'existent que comme effectifs de `PrésenceCohorte`.
- Le rendu tire jusqu'à `plafond_sprites` figurants par clé stable `(graine, id_cohorte, id_lieu, créneau, indice)`.
- Le nombre affiché égale `min(effectif_présence, plafond_sprites)`. L'écart (effectif − affiché) est signalé qualitativement (foule dense), jamais inventé.
- Une interaction durable avec un figurant déclenche l'extraction existante (I1) ; le figurant devient nommé.

## 5. Temps : le créneau comme instant logique

### 5.1 Horodatage

L'horodatage logique d'un événement est `(jour, créneau)`. C'est l'« index de séquence d'horloge » du cadrage ch. 9. Les quatre créneaux (matin=0, après-midi=1, soir=2, nuit=3) ne sont plus de simples étiquettes : ils ordonnaient l'exécution.

Ordre canonique : `(jour, créneau, phase, priorité, id)`.

- Aucune phase ajoutée.
- Aucun événement ajouté : les événements existants reçoivent une étiquette de créneau (créneau naturel en données par type ; défaut 0 pour compatibilité des replais).
- Le report : tout événement généré part à un instant strictement postérieur `(jour, créneau)` ; sans date explicite → prochain créneau (ou lendemain créneau 0 si nuit).

### 5.2 Résolution à la clôture

```
ouverture_créneau(J, C):
  si régime = scène et personnage dans le théâtre:
    halte : le joueur compose son créneau (lieux, actions, demandes)
  sinon:
    autopilote (doctrine)

clôture_créneau(J, C):
  résoudre les emplois du temps du créneau
  exécuter les phases 1 à 7 pour (J, C) dans l'ordre canonique
  produire DescriptionDeScène à la demande (hors état)
```

Ce que le joueur réserve pendant le créneau (demi-heures, lieu, demandes) est figé à la clôture. Les vignettes des événements produits s'affichent ensuite.

### 5.3 Régimes de vitesse

| Régime | Durée réelle (à calibrer) | Comportement |
| --- | --- | --- |
| Scène | 45 à 90 s par créneau | Halte entre créneaux ; pause active disponible |
| Accéléré | 5 à 10 s par jour | Autopilote ; pas de halte |
| Saut | jusqu'à la prochaine alerte | Autopilote ; saute les créneaux sans événement pertinent |

Retour automatique en régime scène sur : alerte à forte valeur d'intervention (cadrage ch. 1), décision ponctuelle de métier (~1/mois), événement touchant un proche (mémoire saillante, foyer, groupe informel).

### 5.4 Transparence du théâtre

La halte n'est qu'une pause d'horloge réelle. L'ordre d'exécution et le contenu des événements sont strictement identiques avec ou sans halte, avec ou sans joueur (autopilote). Test : empreinte d'état et journal bit à bit identiques.

## 6. Interaction avec les PNJ (P1 réduit)

### 6.1 Co-présence obligatoire

Parler, demander ou observer de près exige d'être dans le **même lieu** au **même créneau**. Toute interaction consomme des demi-heures (en données), comme pour un PNJ.

### 6.2 Surfaces disponibles en P1b

| Surface | Effet | Source |
| --- | --- | --- |
| Observer | états visibles sur le sprite ; fiche qualitative selon le niveau de relation (échelle symétrique) | états cachés, mémoire saillante |
| Demander | poste, essai, parrainage, ardoise, prêt | actions Q(a) existantes (tranches 5-6) |
| Témoin direct | ce qui se passe dans le lieu où l'on est s'apprend à l'instant `(J, C)` via `appris_par` du journal | journal causal (tranche 2) |

### 6.3 Reporté à P1.5

Les cinq autres intentions (se plaindre, partager, inciter, entretenir, transmettre), l'écoute passive, le corpus de répliques complet et l'écran de conversation élargi : sections 24 et 27 de la spec P1, inchangées. Elles deviennent vraies parce que « être au bon endroit au bon créneau » est désormais un fait d'état.

### 6.4 Non-fuite d'information

Le témoin direct n'apprend que les événements dont le lieu et le créneau coïncident avec sa présence. Aucun nœud d'apprentissage futur. Test : zéro fuite (aligné sur « explique », cadrage ch. 9).

## 7. Mise en scène causale

### 7.1 Principe

Le quartier vit parce que chaque chose a une cause visible, pas parce qu'il y a du bruit. Aucune animation ni réplique décorative (spec P1, 6.18).

### 7.2 Table événement → vignette

En données versionnées : `(type_événement, conditions) → type_geste`. Chaque vignette porte `id_événement` (ou référence d'état : usage du créneau, seuil d'état franchi).

Entre deux événements, un sprite joue son **usage du créneau** (travail, repas, loisir, sommeil, trajet rendu) : c'est un état, pas un décor.

### 7.3 Densité minimale de scènes

Mesure headless : nombre de vignettes par créneau dans le lieu du joueur (et, agrégé, par lieu du quartier).

- Si la densité est trop basse : signal sur la simulation (pas assez d'événements, routines trop uniformes), **jamais comblé par du bruit**.
- Cible de départ (à calibrer) : ≥ 1 vignette causale par créneau dans le lieu du joueur en régime non-choc ; variance journalière > 0.

### 7.4 Contrat headless

```
scene(état, journal, instant, observateur) -> DescriptionDeScène
  lecture seule : aucun écriture, aucun tirage nouveau, aucun événement émis
  sprites   : nommés présents + figurants tirés des PrésenceCohorte (plafond)
  vignettes : projection des événements de (instant, lieu) + usages d'état
  marqueurs : dérivés des variables existantes (stocks, usure, grève, …)
```

- Déterministe : même entrée → même description bit à bit.
- Coût : O(sprites à l'écran + événements du lieu), jamais O(population du monde).

## 8. Graphismes (application quartier du cadrage ch. 12)

### 8.1 Plan du quartier

Pipeline structure / peau / marqueurs en version minimale :

- un style (une culture), un biome ;
- une pièce préfabriquée par type de lieu (atelier, épicerie, restaurant, étuve, dispensaire, logement×3, rue, place) ;
- autotiling des sols et murs ;
- traversée du quartier en moins de 30 s réelles à pied en régime scène (à calibrer).

### 8.2 Marqueurs d'état (lecture seule)

| Marqueur visible | Variable source |
| --- | --- |
| Façades usées, saleté | dette d'entretien, état des actifs |
| Étals fournis / vides | stocks de la firme |
| File devant la porte | file de pénurie |
| Rideau baissé | créneaux d'ouverture, faillite |
| Piquet, atelier fermé | grève |

### 8.3 Calques de sprite (32×64, cadrage ch. 12)

| Calque | Variable source |
| --- | --- |
| Maigreur / corpulence | faim |
| Démarche | fatigue, douleur |
| Tenue | niveau de logement |
| Séquelle visible | santé-trajectoire |
| Peau du métier | pratique / poste (données) |
| Direction / idle | rendu pur (graine) |

Lumière et ambiance des quatre créneaux : recolorisation programmatique (charte ch. 12), hors état.

Portraits modulaires : P1.5.

### 8.4 Manifeste d'assets minimal (lot de validation)

Ancré sur le lot de 10 assets du cadrage ch. 12, étendu au quartier :

- 3 tuiles de sol (rue, intérieur atelier, intérieur logement) ;
- 1 jeu d'autotuiles mur ;
- 2 personnages (journalier, patron) × 4 directions ;
- 1 portrait (P1.5) ;
- 2 objets (étal, piquet de grève) ;
- surcouches d'état : usure façade, file, rideau.

Licence : ouverte (CC0 recommandé, à trancher hors de cette spec).

## 9. Écrans et promenade

Les écrans de la spec P1 §23 (doctrine, planification, vue quartier, journal) restent valides. Compléments P1b :

- La console portable et les postes fixes du quartier sont des **objets-lieux** : le joueur les atteint en étant présent dans leur lieu.
- La vue du quartier peut s'ouvrir depuis n'importe quel lieu (lecture), mais éditer la doctrine ou accepter un poste se fait à la console (présence requise) ou via planification déjà engagée.
- Alertes : classement par valeur d'intervention ; retour en régime scène.

## 10. Découpage d'implémentation (rappel, hors code de cette session)

1. **Clé d'ordre** : `(jour, créneau, phase, priorité, id)` avec tous les types au créneau 0 → journaux identiques aux replais actuels (pas de validation humaine pour les replais).
2. **Créneaux naturels** par type en données → change les replais de référence → **validation humaine obligatoire** (convention 6).
3. Entité Lieu, présence, emploi du temps positionné (32 demi-heures).
4. Contrat `scene` headless + tests de fermeture.
5. Rendu et écrans (dépend de la bibliothèque choisie par l'humain).

## 11. Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T11a.1 Équivalence d'ordre | tous les types au créneau 0, même graine | journaux identiques bit à bit aux replais de référence actuels |
| T11a.2 Transparence du théâtre | même graine, avec haltes + autopilote vs exécution directe | empreinte d'état et journal identiques |
| T11a.3 Rendu sans écriture | appels répétés à `scene(...)` pendant 10 ans | empreinte d'état identique avant/après ; aucun événement émis |
| T11a.4 Non-contradiction | un jour, tous les lieux | sprites comptés = présence canonique (nommés + min(effectif, plafond)) ; toute vignette référence un id d'événement ou un état existant |
| T11a.5 Symétrie joueur / bot | joueur absent de son poste vs bot identique absent | mêmes conséquences (salaire, journal) bit à bit |
| T11a.6 Témoin direct | événement dans le lieu du joueur ; événement ailleurs | appris_par contient le joueur seulement pour le premier ; zéro fuite |
| T11a.7 Coût indépendant de la population | quartier 150 vs charge synthétique 10 000 cohortes hors bulle | temps de `scene` borné par plafond_sprites, pas par la population |
| T11a.8 Benchmark théâtre | 300 ans, régime accéléré avec retours scène sur alertes | latence et taille de file sous les seuils de la tranche 10 ; pas de dérive mémoire |
| T11a.9 Densité | 1 an, lieu du joueur | ≥ 1 vignette causale / créneau en régime non-choc (à calibrer) ; aucune vignette sans cause |
| T11b.1 Critères humains | un humain joue 30 minutes | comprend pourquoi il s'en sort ou non ; co-présence ressentie ; aucune attente passive non signalée |

## 12. Valeurs à calibrer (hypothèses)

| Paramètre | Valeur de départ | Validé par |
| --- | --- | --- |
| Durée réelle d'un créneau (régime scène) | 45 à 90 s | T11b.1 |
| Durée réelle d'un jour (accéléré) | 5 à 10 s | T11a.8 |
| Plafond de sprites par lieu | 24 | T11a.4, T11a.7 |
| Densité minimale de vignettes / créneau (lieu joueur) | ≥ 1 en non-choc | T11a.9 |
| Traversée du quartier à pied | ≤ 30 s réelles | T11b.1 |
| Coût temporel d'un changement de lieu | 1 demi-heure | T11a.5 |
| Créneau naturel par type d'événement | table en données (défaut 0) | T11a.1 puis validation humaine |

## 13. Invariants ajoutés / précisés

- **I9 étendu** : même graine et mêmes données = même journal, y compris après introduction du champ créneau à 0 pour tous les types (T11a.1).
- **I-scène-1** : `scene` est une fonction pure de lecture ; empreinte d'état inchangée (T11a.3).
- **I-scène-2** : pour tout (lieu, créneau), nombre de sprites affichés = nommés présents + min(effectif cohorte, plafond) (T11a.4).
- **I-scène-3** : toute vignette référence un événement du journal de l'instant ou un champ d'état existant ; zéro vignette décorative (T11a.4, T11a.9).

## 14. Critères d'acceptation

- T11a.1 à T11a.9 verts en continu.
- Revue humaine T11b.1 avec la fiche de revue (convention 24).
- Aucune dépendance de rendu dans le cœur ; le cœur tourne headless sans `scene` appelée.
- Les sections 24 et 27 de la spec P1 restent le contrat de P1.5 ; P1b ne les anticipe pas au-delà d'observer / demander / témoin direct.
