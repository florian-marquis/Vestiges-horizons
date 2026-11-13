
# Vestige : Horizon · Spécification P1 : le quartier vivant

Document de spécification du premier palier jouable, destiné à être exécuté par des agents (Cursor). Références : Cadrage v6.5 (chapitres 2, 3, 9, 10, 11, 14, 16, 17), Conventions v2. Chaque section se termine par ce qui la valide ; « à calibrer » désigne une valeur d'hypothèse, jamais une décision.

## 1. Question fermatoire

**P1 est réussi si**, sur une graine donnée, sans scénario ni intervention du joueur : le quartier vit (les 150 individus mangent, travaillent, s'endettent, déménagent, s'extraient ou s'enfoncent), le taux de sortie de misère du personnage est égal à celui d'un PNJ à investissement égal, chaque trajectoire s'explique par le journal causal, tous les invariants de la section 7 restent verts, et l'ensemble tourne headless sur 10 ans en un temps mesuré (pas de seuil a priori, benchmark obligatoire à la tranche 10).

## 2. Périmètre

**Inclus** : noyau de simulation (horloge, file d'événements, ordonnancement), individus et cohortes, besoins fondamentaux et budget de temps, états individuels déclenchés (faim, fatigue, stress, ennui, douleur, angoisse, dépression), empathie et contagion de souffrance, vivacité des PNJ (hétérogénéité, habitudes, mémoire active, répliques, groupes informels, rituels), traits du décideur, valuation des rôles pondérée en données, repas différenciés et eau refacturée au prix du biome, firmes de bouche et de loisir (épicerie, restaurant, étuve), métiers et maisonnées, marché des biens à prix affichés fixés par les firmes, marché du travail à cinq grandeurs, salaire réel et griefs, basculements (grève), usure et entretien minimaux, pratiques et apprentissage, reconnaissance, crédit et saisie, santé-trajectoire, qualité des biens, logements différenciés avec sous-location, vie vitale minimale, parrainage, fiscalité forfaitaire (prélèvement à la source, allocations fixes), attractivité enregistrée, journal causal, bots et suite headless, écrans minimaux (P1b).

**Exclus (et c'est tranché)** : politique et légitimité (P2), politique fiscale (taux vivants, incitations, allocations conditionnelles : P2-P3), institutions de solidarité et groupes de pression (P2, route 12 du cadrage ; la graine en P1 est le régime « rien » : dispensaire privé payé à l'acte), migrations (routes 10 et 11 du cadrage), ville hors quartier (le quartier a des frontières commerciales : import/export à prix de référence), terrain praticable et marche (P2), combat, logistique interrégionale, réseaux et constellations (le terminal du joueur est local), succession, voyages. Le seul acteur nonhumain est le « marché de la ville », une source et un puits à prix de référence, pas un agent.

## 3. Unités et arithmétique

- Tout est entier. Monnaie : centimes (1 unité = 100). Probabilités : 1/10 000. Temps : le jour est l'unité d'ancrage ; les durées en jours ; les heures de budget de temps en demi-heures. Le jour porte quatre **créneaux** étiquetés (matin, après-midi, soir, nuit) : des étiquettes sur les demi-heures, jamais une horloge intra-journalière.
- Un événement porte un horodatage logique (jour, index de séquence d'horloge). Deux événements du même jour ne partagent jamais le même rang d'exécution (cadrage ch. 9).
- Aucun flottant dans l'état canonique ; les taux (usure, griefs) sont des entiers avec échelle déclarée.

## 4. Entités canoniques

| Entité | Champs minimaux | Notes |
| --- | --- | --- |
| Individu | id, cohorte d'origine, âge, rôle (journalier, ouvrier, patron, retiré), pratiques maîtrisées (id pratique → niveau), solde (centimes), logement (id ou aucun), pratique en cours, états cachés (faim, fatigue, stress, ennui, douleur : 0-1000), empathie (0-1000), routine (emploi du temps par défaut), mémoire saillante (références, plafond 20) | 150 au départ ; extraction d'un individu nommé selon l'invariant I1 |
| Cohorte | id, effectif, motivations dominantes (w), attachement moyen, griefs (0-100), heures disponibles, salaire réel cible | 3 à 5 au départ ; porte des totaux conservés (monnaie, somme des griefs pondérée, attachement), moyennes dérivées (14.2) ; regroupement des cohortes proches autorisé, fusion conservative |
| Foyer | id, membres, logement, dette | Décisions d'achat au niveau foyer pour les biens domestiques |
| Groupe informel | id, membres (3 à 6 individus nommés), lieu habituel, affinités | Embryon du réseau de faveurs (P2) ; canal de contagion de la souffrance |
| Firme | id, type (atelier, épicerie, restaurant, étuve, propriétaire foncier, crédit, dispensaire, maisonnée), décideur (id individu), cohorte employée, actifs, stock d'intrants, stock de produits, prix affiché, masse salariale, trésorerie, dette d'entretien, créneaux d'ouverture | 8 à 12 au départ ; la faillite est possible (transfert des actifs au créancier principal, jamais destruction de monnaie) |
| Bien | id, unité, qualité, prix affiché (centimes), stock | 3 biens au départ : nourriture (2 qualités), logement (3 niveaux), soins |
| Logement | id, niveau (1 à 3), taille, occupants, sous-locataires, propriétaire | Parc fixe en quantité, différencié en qualité |
| Prêt | id, prêteur, emprunteur, principal, taux, garantie, échéancier, état (sain, en défaut, saisi) | Prêteur = firme ; prêts entre individus via la même table |
| Actif | id, firme propriétaire, type, état (0-1000), (état initial, date, taux) | Usure paresseuse (cadrage ch. 17, piste 1) |
| Pratique | id, type (recette, technique), nom, exigences | Piloté par les données (ch. 16) |
| Maîtrise | (id pratique, porteur), niveau 0-10, expérience cumulée, seuil de risque tiré | Un porteur est un individu ou une cohorte |
| Emploi | id, firme, cohorte visée, heures, salaire horaire, créneau (matin, après-midi, soir, nuit), état (ouvert, pourvu, rompu) | Support des cinq grandeurs du marché du travail |
| Marché | id, bien, prix affiché, seuils de stock, file d'attente | Un marché par bien ; pas d'enchère, prix affichés |
| Journal (événement causal) | id, date, phase, acteur, décision, valeurs modifiées, événements antérieurs, date d'apprentissage | Format unique, contrôle par la convention 23 |

**Variables partagées du palier** (agrégées par cohorte, instanceées pour la bulle du joueur et les individus nommés) : u (part des besoins alimentaires non satisfaits), p (pression des prix), q (qualité effective des services : santé, entretien du quartier), G (griefs), salaire réel (salaire horaire × heures / coût du panier), reconnaissance (position relative dans le groupe de référence).

## 5. Recensement des événements et phases

Le tri des événements d'un même jour suit : horodatage (jour), puis phase, puis priorité en données, puis id. Phases du recensement P1 (aucune autre ne sera ajoutée sans justification) :

| Phase | Événements |
| --- | --- |
| 1 · Flux physiques | arrivée d'import, fin de production, consommation du foyer, franchissement de seuil de stock, franchissement de seuil d'usure |
| 2 · Marchés | transaction, reprice (ajustement du prix affiché), rupture de stock, faillite |
| 3 · Comptes | versement des salaires, loyer, remboursement, intérêts, échéance de prêt, défaut, saisie, servitude, prélèvement à la source, allocation publique |
| 4 · Marché du travail | publication d'un emploi, embauche, licenciement, démission, fin d'essai, parrainage |
| 5 · Décisions d'agents | évaluation d'action (Q(a)), réévaluation de doctrine (dissonance), arbitrage du budget de temps |
| 6 · Social | mise à jour des griefs, franchissement du seuil de risque cumulé (grève), reconnaissance (promotion, affront), franchissement de seuil d'état (faim, fatigue, stress, ennui, douleur), interruption d'urgence, activité de loisir, contagion de souffrance, rituel social, maladie, blessure, guérison, séquelle, décès, naissance, héritage, extraction d'un individu nommé |
| 7 · Journal | compactage, rétention |

Règle : les événements générés pendant le traitement partent au jour suivant (ou à l'instant logique suivant intra-jour si la phase n'est pas encore passée) ; jamais d'exécution immédiate réentrante.

## 6. Mécanismes

### 6.1 Budget de temps (par individu, par jour)

Le jour se répartit en 32 demi-heures : 16 d'éveil utile.

```
budget_éveil = 32
temps_besoins = repas (2) + trajet (1) + corvées (2, réduites par le logement)
si soin en attente : temps_soin selon file
si recherche d'emploi : temps_recherche (2 à 4)
si insécurité élevée : temps_sécurité (1 à 2)
temps_restant = budget_éveil − temps_besoins − temps_soin − ...
travail effectué = min(heures_poste, temps_restant)
formation = temps_restant − travail, si pratique choisie et vétéran disponible
```

- Un individu privé de nourriture perd 1 demi-heure d'éveil par jour et demi de jeûne (à calibrer).
- L'insatisfaction des besoins ne produit pas de jauge visible : elle consomme du temps et nourrit u et G.
- **Garde-fou de calibrage** : le temps de survie d'un travailleur à plein temps au niveau de vie de référence reste ≤ 12 demi-heures (6 h). Au-delà, la boucle de progression du palier est impossible.
- Les 32 demi-heures se regroupent en quatre **créneaux** (matin, après-midi, soir, nuit) ; un poste occupe un créneau, un emploi du temps remplit des créneaux. Le travail nocturne ajoute une demi-heure de fatigue par créneau de nuit (à calibrer) ; la prime de nuit est un comportement de firme (P1.5, 6.22).

### 6.2 Marché des biens (prix affichés, pas d'enchère)

- Chaque marché affiche un prix. **Le reprice est une action de la firme**, évaluée par Q(a) dans sa table d'actions (monter, baisser, tenir). Le comportement par défaut, fourni en données, est l'heuristique par seuils de stock : stock < seuil bas → prix +pas ; stock > seuil haut → prix −pas. Un commerçant peut donc s'en écarter : maintenir un prix haut en pénurie pour conserver son stock, ou casser les prix pour évincer un concurrent.
- **Qualité des biens** : la nourriture existe en deux qualités ; la basse nourrit mais use la santé (risque de maladie accru). Les qualités sont des recettes distinctes en données, achetables séparément sur le même marché.
- Transactions au prix affiché. En pénurie (stock insuffisant), allocation par file : arrivée, puis rang (abonnement d'achat du foyer, plus cher, passe devant), puis id. Les enfants du quartier mangent avant les adultes (priorité en données).
- Coût du panier = nourriture ×1/jour + logement ×1/jour par adulte (réduit de moitié par enfant, à calibrer).
- Marché de la ville : source et puits à prix de référence dérivés de la graine, avec variations lentes ; aucune règle d'équilibre, les déséquilibres persistants sont des résultats légitimes.
- **Élasticité de la ville (protection de validité)** : la ville n'est pas un amortisseur infini. Sa courbe d'offre est en données : le prix de référence monte avec le volume demandé au-dessus de la capacité nominale (pente de départ : +1 % de prix par 2 % de demande excédentaire, plafond +50 %, à calibrer), et le réapprovisionnement porte un délai logistique (3 à 10 jours selon le bien, à calibrer). Un choc local (choc 2) produit donc une hausse du prix de référence et des ruptures temporaires, amorties avec retard, jamais neutralisées instantanément. La ville suit le quartier, elle ne le sauve pas ; l'élasticité est une donnée de la source, pas une règle d'équilibre du moteur.

```
u_foyer = nourriture_non_achetée / besoin_total   (0 si stock foyer > 0)
p_foyer = max(0, coût_panier − revenu_disponible) / coût_panier
q_individuel = f(santé accessible, dette d'entretien du quartier)
dG = 0,8·u + 0,2·p − 0,25·q   (points/jour, bornés [0, 100], intégrés quand une entrée change)
```

### 6.3 Marché du travail (cinq grandeurs distinctes)

| Grandeur | Définition | Mise à jour |
| --- | --- | --- |
| Disponibilité | heures d'éveil moins le temps des besoins, par cohorte | quotidienne |
| Volonté | heures offertes au salaire courant (fonction du salaire réel cible) | événementielle |
| Demande | postes publiés par les firmes | décision des firmes |
| Postes pourvus | appariement volontés/demande par cohorte et pratique | événement embauche |
| Heures travaillées | min(poste, temps restant), effectivement payées | quotidienne |

- Les firmes arbitrent par Q(a) : embaucher, licencier, monter le salaire, baisser le salaire, fermer, entretenir, investir (P1 : recette unique d'atelier). Coûts et risques réels ; aucune convergence garantie.
- Grève : quand le seuil de risque cumulé d'une cohorte est franchi (taux fonction de G, du salaire réel, des moyens de la cohorte), la cohorte cesse d'offrir du travail pour une durée tirée ; l'événement est dans le journal avant, pendant, après.

### 6.4 Usure et entretien (version minimale)

- Actifs des firmes : état 0-1000, usure 1 à 3 % par an (entiers : 30 à 100 points/millé). L'état est calculé paresseusement ; le franchissement des seuils (capacité réduite sous 400, critique sous 100) est un événement planifié.
- Entretien : action de la firme, consomme composants (import) et heures de cohorte ; politique par firme (minimale, normale, renforcée).
- La dette d'entretien du quartier alimente q.

### 6.5 Pratiques et apprentissage

- Pratiques initiales : 4 à 6 (poste d'ouvrier, commerce de détail, tenue de comptes, prospection de clientèle, deux techniques d'atelier).
- Chaque exécution d'une demi-journée de travail ajoute de l'expérience ; le franchissement du seuil de risque cumulé (tiré à clé stable) monte le niveau ; le plafond de formation est 1 vétéran pour 3 élèves.
- Le niveau de maîtrise module le rendement de l'atelier et le salaire horaire (à calibrer).

### 6.6 Reconnaissance

- Groupe de référence : cohorte + immeuble. Position relative = rang du revenu du foyer dans le groupe, borné.
- Mise à jour sur événements : promotion, embauche, licenciement, acte marquant, affront. Elle pèse dans Q(a) des individus (peser sur l'ambition : accepter un poste risqué, migrer), pas sur G directement.

### 6.7 Écran du joueur (P1b, minimal)

- Le personnage du joueur est un individu standard de la cohorte des journaliers, sans pouvoir spécial.
- Console portable : doctrine personnelle éditable (poids entre besoins, travail, formation, reconnaissance ; modèles prédéfinis, pas de script libre), planification basique (accepter un poste, choisir une pratique, abonnement d'achat).
- Vue du quartier : liste des foyers et individus nommés, états qualitatifs (jamais de jauge numérique de grief), journal causal consultable avec dates d'apprentissage.
- Marche et terrain : hors périmètre P1, tranché pour P2.

### 6.8 Crédit et saisie

- Un prêteur local, firme parmi les autres. Offres de prêt en données : montant maximal, taux annuel, garantie exigée, échéancier. Intérêts composés en virgule fixe, capitalisés aux échéances.
- Garanties, dans l'ordre : le logement, puis la servitude (heures de travail dues au prêteur, rémunérées en dessous du salaire du marché). Le défaut est un événement ; la saisie transfère le logement au prêteur, qui peut le relouer.
- Le crédit est symétrique : le joueur peut emprunter et prêter, dans les mêmes conditions qu'un PNJ (prêts entre individus via la même table d'actions, plafonnés par la trésorerie du prêteur).
- Valeurs de départ, à calibrer : taux 10 % par an, 20 % après défaut.

### 6.9 Santé-trajectoire

- Maladie et blessure : événements à clé stable (graine, individu, date). Fréquence liée à la densité du logement, à la qualité de la nourriture et à l'usure des ateliers ; gravité bénigne ou grave.
- Soins : le dispensaire (firme) traite une file ; se soigner coûte du temps (attente) et de l'argent. Ne pas se soigner allonge la maladie et augmente le risque de séquelle.
- **Contagion de maladie (entre en tranche 6, avec les collègues)** : l'exposition aux malades présents (même lieu, même créneau) est un facteur de risque en données (+5 à +15 % de risque par collègue malade présent, à calibrer). **Travailler malade allonge la maladie** (non-repos : +20 à +50 % de durée, à calibrer) et monte le risque de séquelle. Le choix de s'arrêter est une décision Q(a) pondérée par l'engagement (6.20) : le présentéisme est une décision, jamais une règle.
- Séquelle : réduction permanente de la capacité de travail (en points, en données), visible dans l'état qualitatif de l'individu.
- La santé moyenne de la cohorte alimente la qualité des services `q`, donc les griefs.

### 6.10 Vie vitale minimale

- Vieillissement par anniversaire, un événement annuel par cohorte. Décès liés à l'âge et à la santé, par tirage à clé stable. Naissances agrégées par cohorte : les enfants sont des effectifs, des individus nommés ne sont créés qu'à l'extraction, vers 14 ans.
- Un décès transfère le patrimoine (logement, épargne) selon une règle en données : conjoint, descendants, créanciers. Chaque héritage est un événement du journal.

### 6.11 Parrainage

- Un employé dont la pratique est ≥ niveau 3 peut parrainer un journalier : coût en temps régulier, effet concret sur l'embauche (la firme accorde un essai), lien social durable dans la mémoire saillante des deux individus.
- C'est la version minimale du capital social du chapitre 13. Le graphe complet de relations est hors périmètre P1 (route approuvée, voir cadrage ch. 17).

### 6.12 États individuels déclenchés

- Cinq variables cachées par individu nommé (0-1000) : **faim, fatigue, stress, ennui, douleur**. Accumulation quotidienne déterministe, à clé stable. Jamais affichées en nombre : seuls des états qualitatifs apparaissent (sur le sprite, dans le journal).
- Les états sont des **dérivés des variables existantes**, pas de nouveaux besoins : faim ← consommation manquante (u) ; fatigue ← usage du budget d'éveil ; stress ← collecteur unique (griefs, dette, ennui, douleur, précarité) ; ennui ← déficit de loisir ; douleur ← santé (maladie, blessure, séquelle).
- Seuils en données → événement d'état (journalisé). Un état **critique** produit un événement d'**interruption d'urgence** qui préempte l'emploi du temps ; sinon la doctrine pondère Q(a) et l'autopilote tient le planning. Le joueur n'intervient jamais obligatoirement ; l'emploi du temps est sa surface d'édition (P1b).
- Le stress est l'**unique variable de santé mentale** : rendement et apprentissage réduits au-delà du seuil, à calibrer.
- L'ennui pèse dans Q(a) vers les actions de loisir (étuve, restaurant, occasions sociales) ; une activité de loisir tous les 3 à 5 jours évite le déficit. L'ennui alimente le stress.
- Cohortes : coefficients agrégés, aucune variable individuelle hors des nommés. L'autopilote gère les urgences seul : un bot sans doctrine doit survivre.
- **Règle pour P2 et plus (rien à implémenter en P1)** : le stress des individus et des cohortes sera une entrée du moral des unités (cadrage ch. 11) ; la panique sera un basculement du moral par seuil de risque cumulé. Les états sont conçus pour traverser la règle d'identité.

### 6.13 Eau, hygiène et niveaux de logement

- L'eau courante est dans tous les logements ; c'est un **import de la ville refacturé dans le loyer** par le propriétaire (une seule facture). L'abonnement est indexé sur la rareté de l'eau (paramètre du biome, en données) : 5 à 10 % du panier en situation normale, jusqu'à 25 % en pénurie. Élasticité quasi nulle : chaque hausse se traduit en salaire réel en baisse, donc en misère et en griefs, sans baisse de consommation.
- Niveaux de logement : **niveau 1** = sanitaires communs sur le palier (densité haute, risque maladie ×1,5, déjà en 6.9) et **pas de cuisine** : manger dehors est structurel (cantine, vente de rue) ; **niveau 2** = eau courante individuelle, cuisine, densité basse ; **niveau 3** = confort complet.
- L'hygiène n'est ni un acte ni un service : c'est un **attribut du niveau de logement**. La coupure d'eau (usure des adductions, entretien par le fontainier-vidangeur) est un événement qui monte densité et risque maladie.
- L'étuve (bains de détente) n'est **pas** un sanitaire : c'est une firme de loisir (6.14). Aucune firme de l'eau en P1.

### 6.14 Repas, loisir et gradient du luxe

- Chaque bien et service porte un couple **(coût monétaire, coût temporel)** en données. La richesse convertit l'argent en temps ; la pauvreté se paie en temps. C'est le gradient du luxe : la part du budget d'éveil consacrée à la survie décroît avec la richesse (hypothèses : ~20 % de temps libre pour le miséreux, 70 % et plus pour le notable, décroissance continue à mesurer par décade).

| Mode de repas | Coût monétaire | Coût temporel | Social | Condition |
| --- | --- | --- | --- | --- |
| Cantine, vente de rue | bas-moyen | faible (1 demi-heure) | faible | aucune |
| Cuisine maison | bas (épicerie) | haut (2 à 3) | foyer | cuisine au logement (niveau 2 et +) |
| Restaurant | moyen-élevé | faible (1) | oui, + loisir | aucune |
| Cuisinier de maisonnée | élevé (un salaire) | quasi nul | au foyer | richesse |

- **Règle de diversité des firmes** : une firme n'entre que si elle occupe un point distinct de l'espace (coût monétaire, coût temporel, valeur sociale) et joue un rôle dans une boucle (métier, besoin, occasion sociale). Deux firmes au même point fusionnent en données. **L'estaminet entre en P1.5** (décision 58) : comptoir debout, boisson et plat simple à prix bas, conversation longue à petit coût ; point distinct du restaurant (repas complet, assis, plus cher) ; c'est le lieu naturel des conversations politiques.
- Le restaurant : plats préparés, files, reprice par Q(a) ; l'ardoise est du **crédit informel** (table des prêts entre individus, 6.8) ; contribution au loisir contre l'ennui. Métier jouable (voir 6.15).
- Le loisir ne satisfait aucun besoin fondamental : il réduit l'ennui, crée des occasions sociales (mémoire saillante, graines du réseau de faveurs P2) et alimente l'attractivité (6.17).

### 6.15 Métiers du quartier

- Un métier est un triptyque en données : **pratique × recette × poste** (offre d'une firme). **Jamais de minigame ni de QTE** (symétrie joueur/PNJ, incalibrable sur 300 ans). L'expérience tient dans trois niveaux : **résolution** (temps alloué + recette → production et salaire, par événements) ; **décisions ponctuelles** (propositions d'événement au travail : incident de machine, raccourci risqué, place disputée, élève à parrainer ; ~1/mois, à clé stable ; même table pour les PNJ, résolue par Q(a)) ; **vécu incarné** (P1b : emploi du temps et journal ; P2 : présence physique).
- **Pas de niveaux discrets par firme** : la différenciation tient à trois dimensions continues : la taille (postes, capital, effectif), l'état des actifs (usure, entretien, 6.4) et la **gamme de recette** (recette de base contre recette améliorée, en données). L'**amélioration est une action, pas un palier** : « investir » (déjà dans la table d'actions Q(a), 6.3) achète des actifs, et les actifs possédés débloquent les recettes supérieures ; le franchissement est un événement journalisé, symétrique (un PNJ investit pareil). Une usine n'est pas un « atelier niveau 5 » : c'est la même firme avec plus de capital et de postes, résolue en cohortes. Valeurs de départ : actif de gamme supérieure ≈ 6 à 12 mois de la marge de la firme ; recette améliorée vendue 30 à 60 % plus cher, plus d'intrants et d'heures qualifiées ; une firme qui n'entretient pas dégrade sa gamme faute d'actifs en état.
- **Recettes multi-postes à goulot** : une recette peut déclarer plusieurs étages de postes (pratique requise × capacité en unités/demi-journée, ex. cuisine et service au restaurant) ; la production de la firme est le **min des capacités des étages pourvus** (le goulot), recalculée par événement (embauche, départ, incident), jamais par tick. La division du travail devient une contrainte économique visible : poste manquant → couverts perdus → revenu en baisse → embauche prioritaire, tout au journal. Aucune chorégraphie par objet : on simule la contrainte, pas les gestes (jamais de préparer/cuire/dresser instancié). Valeurs de départ : 8-15 couverts par serveur et par service, 10-20 plats par cuisinier, à calibrer.
- La peau des métiers (noms, intitulés, sprites) est en **données versionnées**, jamais dans le moteur : un métier est défini par sa recette et sa place dans l'économie, pas par son époque.
- Dix métiers P1 : manœuvre (chantiers de la ville, risque de blessure, plancher) ; ouvrier d'atelier (apprentissage, usure, incidents) ; épicerier (reprice, stocks) ; restaurateur (cuisine, file, ardoise) ; fontainier-vidangeur (entretien des adductions et latrines publiques : son travail pèse sur la santé du quartier ; stigmatisé, payé en prime de risque) ; étuvier ; soigneur (tri de la file : règle morale en données) ; scribe (lettres et contrats : documentation du ch. 16 à l'échelle du quartier, nœud d'information et de crédit informel) ; prêteur (6.8) ; propriétaire foncier (rentes, entretien, allocation, refacturation de l'eau).
- Pas d'agriculteur (la nourriture vient de la ville) ni de milicien (vol et milice : route 7, P2).
- **Maisonnée = firme** : recettes (entretien, cuisine, soins), employés, pratiques (service domestique), usure de la maison. Les patrons et retraités du quartier peuvent employer des journaliers : le luxe des uns crée les emplois des autres. Le notable du late game est patron de sa propre firme domestique (même code que l'atelier).
- **Joueur-patron (P1b)** : progression par capital requis : service ambulant (barrière quasi nulle) → boutique, restaurant ou étuve (le plafond réaliste du palier) → atelier (hors de portée de P1, il reste aux PNJ). Pilotage par **directives minimales** (embaucher au-dessus d'un seuil, reprice conditionnel, stocks, crédit client) : mêmes tables que l'autopilote des firmes PNJ, aucune gestion en temps réel.

### 6.16 Fiscalité forfaitaire et transferts

- **Impôt sur salaires et bénéfices : 10 %, prélevé à la source par la firme** (rôle purement comptable, aucun choix) ; sort vers la ville (condition aux limites).
- **Allocation vieillesse ≈ la moitié du coût du panier** et **aide par enfant ≈ le quart** : transferts forfaitaires fixes de la ville vers les foyers, journalisés, comptés par I3 comme transferts déclarés.
- Toutes les grandeurs économiques du palier s'expriment **en net** (salaire réel, panier, Q(a) des firmes).
- **Aucun acteur fiscal en P1** : taux fixes, pas de politique. La trappe est un défaut de calibrage : le taux de sortie de misère doit être insensible à l'existence des allocations. La politique fiscale (taux vivants, incitations, conditionnalité) entre quand l'État est une organisation jouable (P2), les incitations au palier réseau (P3).

### 6.17 Attractivité (variable enregistrée)

- Composite par cohorte : salaire réel, loisir disponible, sécurité, griefs bas ; coefficients en données.
- **Enregistrée dès P1, consommée par rien en P1** : préparée pour les routes 10 et 11 (migrations) et la route 6 (dérive culturelle) du cadrage. Coût quasi nul, décision riche ensuite.

### 6.18 Vivacité des PNJ

- **Hétérogénéité par clé stable** : chaque nommé tire sa variation de doctrine (lève-tôt ou couche-tard, économe ou dépensier, sociable ou solitaire), ses seuils d'état et ses préférences de lieu. La cohorte donne la moyenne, jamais l'uniformité.
- **Habitudes et inertie** : un PNJ tient une routine (emploi du temps par défaut) jusqu'à réévaluation sur événement ou seuil ; la routine est visible (même place à l'estaminet, même jour de marché, même trajet).
- **Mémoire active** : chaque souvenir saillant pèse comme un terme de préférence dans Q(a) : un affront détourne de l'offenseur, un bon souvenir fait revenir.
- **Variance quotidienne** : les coûts de temps varient (files plus longues, petits incidents, disputes) ; chaque variance a une cause journalisée ; deux jours ne se ressemblent jamais.
- **Répliques générées** : gabarits en données remplis par le dernier événement saillant de l'individu ; déterministes, à clé stable, résolus à la consultation, jamais stockés massivement ; chaque réplique traçable au journal, jamais décorative.
- **Groupes informels** : cercles de 3 à 6 nommés qui fréquentent le même lieu (collègues, voisins d'immeuble, habitués) ; invitations, entraide, commérage différé (graine de la route 2) ; bornés par le plafond de nommés actifs.
- **Rituels sociaux** : les événements de vie (6.10) deviennent des occasions : funérailles (le travail ralentit ce jour-là, coût de temps pour les proches), naissance (cadeaux, visites), jour de paie, jour de marché.
- **Expression des états** : fatigue, stress, ennui, douleur se montrent (démarche, tenue, réplique), jamais en nombre.
- **Principe pour les agents** : le quartier vit parce que chaque chose a une cause visible, pas parce qu'il y a du bruit. Aucune animation ni réplique décorative.
- **Échelle d'interaction avec les PNJ, symétrique** : la masse (cohorte), les témoins (nommés observés), les transactions (marchés et files), les conversations (lieux partagés), les relations (parrainage, prêts), l'intime (foyer), le commandement (patron). Le joueur ne dispose d'aucun niveau fermé aux PNJ.

### 6.19 Empathie, souffrance d'autrui et santé mentale

- **Empathie** : trait individuel (0-1000, clé stable) ; moyenne de cohorte en agrégé ; modulable plus tard par la culture (route 6).
- **Affinité** : poids des liens existants : foyer 1,0 ; groupe informel 0,5 ; immeuble 0,3 ; simple connaissance 0,1. Ensemble de conscience borné par la mémoire (plafond 20).
- **Souffrance perçue** = Σ (empathie × affinité × souffrance visible) sur l'ensemble de conscience ; elle alimente le stress de l'individu, puis remonte vers les griefs de la cohorte par les groupes (**contagion**). L'empathie est un canal, pas un second registre : aucune nouvelle jauge de souffrance, les souffrances sont les états et variables existants (u, douleur, deuil, dette, endettement).
- **Angoisse** : état anticipatoire dérivé (tendance de u, stocks du foyer, épargne) ; déclenchée si épargne < 2 semaines de panier et tendance haussière ; alimente le stress.
- **Dépression** : état persistant de l'axe stress (stress au-delà du seuil haut pendant X jours consécutifs), avec hystérésis (lente à entrer, lente à sortir) ; effets : initiative réduite (moins de recherches d'emploi, plus de routine), apprentissage ralenti ; sortie par causes : soins, lien social, stress bas soutenu ; jamais sans cause journalisée ; rare et réversible (la trappe est interdite, critère du palier).
- **Dissonance morale** : quand l'action la plus efficace contredit la doctrine, dissonance accumulée → stress ; au-delà du seuil : révision de doctrine (réévaluation sur événement, déjà tranchée) ou transgression (le pont vers le vol, la contrebande, la révolution aux paliers suivants). La séquence complète est journalisée.
- **Interaction moral/physique** : douleur → stress → manque de volonté → moins de soins et de travail → douleur et misère persistent. La boucle sort du collecteur, sans nouveau mécanisme.

### 6.20 Traits du décideur et firme

- La firme décide par sa table Q(a) ; le **décideur nommé** (patron) module les poids de la table par ses traits (l'empathie d'abord ; les autres traits viendront avec les factions). C'est le même mécanisme partout : une organisation pense avec le caractère de qui la dirige.
- Empathie du décideur = **patience et tolérance** : seuil de licenciement pour sous-productivité passagère multiplié par (1 + empathie/1000) ; salaires soutenus tant que la trésorerie projetée reste au-dessus de X mois de charges.
- La contrainte de **pérénité est dure** : en situation de faillite imminente, l'empathie s'efface, la trésorerie gagne.
- **Dissonance du décideur** : licencier un employé en difficulté connue (présent dans la mémoire saillante) coûte de la dissonance, donc du stress ; proportionnelle à l'affinité et à la souffrance visible (6.19).
- **Employabilité** : un employé malade est un coût mort pour une firme au churn, une parenthèse pour une firme patiente ; la santé-trajectoire (6.9) devient une variable d'employabilité modulée par la tolérance du décideur.
- **Rétention → savoir → compétitivité** : le licencié emporte sa maîtrise (le savoir est local, ch. 16) ; l'employé maintenu la garde et transmet (1 vétéran pour 3 élèves) ; la maîtrise moyenne de la firme fait le rendement et la compétitivité.
- **Engagement (employé ↔ firme)** : un scalaire par couple (0-1000), mis à jour **uniquement par les décisions observées de la firme** (soutien à un malade, licenciement pour maladie, salaires soutenus), pondérées par l'empathie et la proximité de l'observateur (mémoire saillante) ; demi-vie 1 à 2 ans. Le décideur ne le lit jamais (décision 57) : il perçoit l'effort, le churn et les arrêts, pas le scalaire.
- **Effets de l'engagement** : côté décisions, il pondère l'arrêt maladie, le soin et la démission (engagement bas → présentéisme et churn volontaire ; engagement haut → repos choisi, fidélité). Côté production, **prévention, jamais bonus** : l'engagement ne produit ni temps de travail supplémentaire ni multiplicateur de recette. Il réduit la probabilité d'incidents (usure légèrement ralentie sur les postes tenus par des salariés engagés, en données) et débloque des options de résolution dans les décisions ponctuelles (chercher une solution au lieu de l'arrêt d'atelier). La productivité supérieure de la firme patiente est une **mesure émergente** (production par heure travaillée, comparée dans les tests), pas un paramètre.
- **La boucle complète** : patron sanctionneur → engagement en baisse chez les témoins → présentéisme (ne pas s'arrêter) → contagion à l'atelier et maladies allongées → séquelles → productivité durablement en baisse ; patron patient → repos, soin, savoir conservé. En choc de demande, la firme au churn garde l'avantage d'adaptation : aucun régime ne domine.
- **Deux régimes viables, aucun dominant** : la firme patiente gagne en savoir accumulé en régime stable, la firme au churn s'adapte plus vite en choc de demande ; les cultures d'entreprise divergent visiblement (vétérans et prix hauts contre renouvellement et prix bas).

### 6.21 Valuation des rôles (curseurs, fixes en P1)

- La reconnaissance cesse d'être le rang de revenu : elle devient **f(revenu, utilité visible, conformité aux valeurs)**, pondérée par des **curseurs de valeurs en données** : réussite contre utilité, héritage contre mérite, statut du soin, temps. En P1, curseurs fixes au pôle « valeur individuelle » (le régime par défaut), jamais codés en dur dans le moteur.
- Le **salaire cible** de chaque rôle est modulé par la valuation culturelle : les normes pèsent dans la négociation.
- L'**utilité visible** est calculable depuis le journal (l'hygiène attribue l'incidence de maladie, 6.9) ; l'**écart de valuation** (rapport des salaires contre rapport des utilités) est une mesure des tests.
- **Vocations** : les choix de formation suivent la reconnaissance ; un rôle méprisé voit ses salaires monter par pénurie (correction partielle, à calibrer).
- P2 et plus : les institutions (académies, presse, lobbies) façonnent les curseurs (route 12) ; divergence entre planètes (route 6) ; l'attractivité migratoire devient culturelle (routes 10 et 11).

### 6.22 Créneaux et rythme social (schéma en P1, couplage en P1.5)

- **En P1, le schéma seulement** : les postes déclarent leur créneau, les lieux leurs heures d'ouverture, les emplois du temps remplissent des créneaux. Pas d'horloge intra-journalière : le jour reste l'unité d'ancrage, les créneaux sont des étiquettes sur les demi-heures existantes.
- **Le couplage entre en P1.5** (avec la vie sociale, tranche 9) :
  - **Fréquentation par recouvrement** : la fréquentation d'un lieu = les individus dont le temps libre chevauche son créneau d'ouverture ; les files ont des heures de pointe ; le rythme majoritaire du quartier est une émergence de la distribution des créneaux des postes, pas une règle.
  - **Prime de créneau impopulaire** : si les postes d'un créneau sont rares, les firmes paient une prime (différentiel compensateur, comme le vidangeur) ; contrepartie : pénalité de fatigue et de stress pour le travail nocturne.
  - **Équipes tournantes** : une firme à actifs contraints peut organiser 2 à 3 équipes sur les mêmes machines (action Q(a) de plus) ; arbitrage capital contre travail émergent.
  - **Lits chauds** : partage d'un lit sous condition de complémentarité des créneaux de travail (le lit est libre quand l'autre travaille) ; 2 à 3 occupants, densité au niveau 1 maximum ; le coût du logement baisse, le risque santé monte.
- Valeurs de départ : prime de nuit +20 à +40 % du salaire du créneau ; pénalité nocturne +1 demi-heure de fatigue par créneau de nuit ; rotations de 2 à 3 équipes ; lits chauds 2 à 3 occupants.
- Tests (P1.5) : distribution de fréquentation par créneau (pointe du soir à l'estaminet) ; la prime de nuit émerge seule quand les postes de nuit sont rares ; un atelier contraint par ses machines adopte les rotations ; l'occupation des lits chauds est complémentaire, jamais conflictuelle ; tout individu garde au moins un créneau libre par jour.

### 6.23 Enfants contributeurs (protection de validité)

- **Participation domestique progressive** : dès 8 ans, un enfant peut effectuer des corvées adaptées (portage d'eau, ménage, garde des plus petits). La contribution monte avec l'âge : 0,5 demi-heure/jour à 8 ans, 1,5 à 12 ans, plein régime domestique à 14 ans (à calibrer). Elle réduit d'autant le temps de survie du foyer, sans toucher au budget de temps de survie des adultes qu'elle remplace.
- **Aucun emploi salarié, aucune pratique marchande avant l'âge adulte** : l'enfant contribue au foyer, pas au marché. Son temps de contribution reste plafonné (2 demi-heures/jour, à calibrer) pour ne pas créer d'incitation démographique à la main-d'œuvre enfantine.
- **Aide par enfant (6.16)** : l'allocation amortit le coût du panier, la contribution domestique amortit le temps ; les deux sont forfaitaires et indépendantes.
- Test : un foyer avec enfants en bas âge consomme plus de temps de survie par adulte qu'un foyer sans enfant, mais l'écart est amorti (allocation + corvées) ; un foyer mono-parental avec trois enfants survit sans transfert informel, plus difficilement.

### 6.24 Plancher anti-impasse (protection de validité)

Garantie de design : après une faillite (saisie du logement, perte de l'atelier), un individu doit toujours avoir au moins une voie de sortie de misère, sans règle dédiée, par assemblage des mécanismes existants :

| Voie | Mécanisme porteur | Condition |
| --- | --- | --- |
| Entraide | prêts entre individus (entité Prêt de 6.8 avec prêteur = individu, plafonné par la richesse du prêteur) | un proche solvable existe (parenté, groupe informel) |
| Servitude | 6.8 : servitude pour dette, logement et repas garantis | un créancier accepte (Q(a)) |
| Maisonnée | 6.15 : embauche comme domestique, logement et repas inclus | un patron embauche (Q(a)) |
| Sous-location précaire | après saisie, la famille expulsée redevient sous-locataire de son ancien logement (décision 4) : loyer moindre, densité en hausse | le nouveau propriétaire accepte (Q(a)) |

- Aucune voie n'est garantie individuellement ; le plancher est une **propriété statistique du système** : au moins une voie praticable pour presque tous. Test obligatoire : un bot après faillite a un taux de sortie de misère strictement positif à 5 ans (tranche 10), sur toutes les graines. Un taux nul est un défaut de calibrage (6.16, 6.2), jamais une règle ajoutée.
- La seule issue hors système reste la mort par faim, possible et expliquée au journal ; elle doit rester rare (incidence sous un seuil à calibrer).

## 7. Invariants (testés en continu, échec = arrêt du replai)

1. **I1 Extraction** : toute extraction/réinjection conserve exactement effectif, monnaie, griefs, attachement (égalité stricte).
2. **I2 Stocks** : aucun stock négatif, jamais.
3. **I3 Monnaie** : la somme des soldes ne change que par les transferts déclarés (imports, salaires de la ville) ; chaque transaction est équilibrée (débit = crédit).
4. **I4 Temps** : pour chaque individu et chaque jour, la somme des usages = budget d'éveil.
5. **I5 Prix** : tout prix affiché reste dans ses bornes en données ; jamais infini ni nul.
6. **I6 État** : tout état d'actif dans [0, 1000] ; toute maîtrise dans [0, 10] ; tout G dans [0, 100].
7. **I7 Journal** : tout événement référence des ids existants à sa date ; l'ordre causal est valide ; production et apprentissage distincts.
8. **I8 Concordance** : G d'une cohorte = somme des griefs individuels pondérés par les effectifs, tolérance zéro.
9. **I9 Replai** : même graine et mêmes données = même journal, bit à bit.

## 8. Données initiales (hypothèses à calibrer)

| Paramètre | Valeur de départ |
| --- | --- |
| Individus | 150 (cohorte journaliers 60, ouvriers 60, patrons/retirés 30) |
| Cohortes | 3 |
| Firmes | 3 ateliers, 1 épicerie (2 qualités), 1 restaurant, 1 étuve, 1 propriétaire foncier, 1 dispensaire, 1 prêteur, 2 maisonnées (patrons/retirés) |
| Logements | 45 foyers, parc fixe, 15 par niveau (1 à 3), loyers par marché ; niveau 1 : sanitaires communs, sans cuisine |
| Prix initiaux | nourriture 400/j (qualité haute) et 300/j (basse) à l'épicerie, cantine 350/j, restaurant 600/j, étuve 100/visite, loyer 500/j (niveau 2 ; à moduler par niveau), consultation 200 |
| Salaires | journalier 60/demi-journée, ouvrier 90, qualifié 140 (à calibrer sur le panier : 900/j) |
| Panier | 1 nourriture + 1 logement par adulte et par jour |
| Pas de reprice | ±10 % du prix courant, seuils de stock à 25 % et 75 % du stock de référence |
| Usure | atelier : 80 points/millé/an ; commerce : 40 ; parc immobilier : 60 |
| Pratiques initiales | 6, niveaux de départ 1 à 4 selon cohorte |
| Reconnaissance | demi-vie de l'événement : 5 ans ; effet borné à ±10 sur les poids effectifs |
| Grèves | taux de base nul ; multiplié par (G/100)² × (1 − salaire réel cible), seuil tiré à clé stable |
| Crédit | taux 10 %/an, 20 % après défaut ; garantie logement puis servitude (heures à demi-salaire) |
| Santé | risque de maladie ×2 si nourriture basse qualité, ×1,5 si densité du foyer élevée ; séquelle −10 à −30 points de capacité |
| Vie vitale | décès liés à l'âge et à la santé ; individus créés à l'extraction vers 14 ans |
| Parrainage | coût 2 demi-heures/jour pour le parrain ; essai accordé avec +50 % de probabilité |
| Eau | import refacturé dans le loyer : 5 à 10 % du panier (pénurie : jusqu'à 25 %) ; élasticité quasi nulle |
| Fiscal | impôt 10 % à la source ; allocation vieillesse ≈ la moitié du coût du panier ; aide par enfant ≈ le quart |
| États | 5 variables cachées (faim, fatigue, stress, ennui, douleur), seuils en données ; une activité de loisir tous les 3 à 5 jours évite le déficit |
| Métiers | 10 (manœuvre, ouvrier, épicerier, restaurateur, fontainier-vidangeur, étuvier, soigneur, scribe, prêteur, propriétaire) ; vidangeur 80/demi-journée (prime de risque), incident d'atelier ~1/mois |
| Temps de survie | ≤ 12 demi-heures/jour pour un travailleur à plein temps au niveau de vie de référence (garde-fou) |
| Empathie | 0-1000 par nommé, clé stable ; affinités : foyer 1,0, groupe informel 0,5, immeuble 0,3, connaissance 0,1 |
| Angoisse et dépression | angoisse si épargne < 2 semaines de panier et tendance haussière ; dépression si stress > seuil haut pendant 30 jours ; sortie par soins ou lien social |
| Dissonance | par action contradictoire ; réévaluation de doctrine au-delà du seuil ; transgression si la dissonance dépasse le poids de la règle violée |
| Groupes informels | 10 à 15 cercles de 3 à 6 membres au départ |
| Rituels | jour de paie, jour de marché hebdomadaire ; funérailles et visites de naissance sur événement |
| Valuation | curseurs fixes en P1 (pôle « valeur individuelle ») ; pondérations de la reconnaissance en données |
| Créneaux | 4 étiquettes par jour (matin, après-midi, soir, nuit) sur postes, lieux et emplois du temps ; prime de nuit, rotations d'équipes, lits chauds : P1.5 (6.22) |
| Durée des tests | 10 ans simulés, 5 graines minimum |

## 9. Suite de tests headless

**Scénarios de choc** (chacun doit être explicable de bout en bout par le journal) :

1. Équilibre initial : 1 an sans choc, distribution stable.
2. Choc d'offre : l'import de nourriture baisse de moitié pendant 90 jours ; la ville amortit avec retard (hausse du prix de référence par élasticité, ruptures temporaires, 6.2), jamais instantanément.
3. Choc de demande : une commande de la ville double pendant 60 jours.
4. Hausse d'intrant : le prix de référence des composants +30 %.
5. Faillite : un atelier ferme (dette d'entretien et de salaires).
6. Concurrent : une firme de la ville ouvre un commerce dans le quartier.
7. Chute du salaire réel : le prix de la nourriture +50 %.
8. Grève : un choc combiné (chocs 2 et 7) doit produire une grève explicable.
9. Rétablissement : fin du choc 2, vérifier la reprise sans réinitialisation artificielle.
10. Crise de crédit : le prêteur durcit (taux doublé, garanties exigées) ; vérifier que les saisies et servitudes sont explicables et que les trajectoires divergent sans piège inéluctable.
11. Choc fiscal : la ville double l'impôt pendant un an ; griefs traçables, grève seulement si combiné à un choc de prix ; aucune trappe (taux de sortie de misère insensible aux allocations).
12. Coupure d'eau : les adductions cèdent (entretien négligé) ; crête de maladie, consommation d'eau inchangée (élasticité nulle), rétablissement par le fontainier-vidangeur.
13. Contagion : un décès dans un groupe informel monte le stress des proches par empathie et affinité ; l'offre de travail du groupe baisse ce jour-là ; toute la chaîne est traçable au journal.

**Bots** (5 stratégies d'individus, plusieurs centaines d'instances sur 5 graines) : le journalier (besoins d'abord), l'ambitieux (formation, mobilité), l'économe (épargne, abonnements), le dépendant (dette), le gréviste (conflit). Critères : aucun ne domine systématiquement ; aucun n'est impossible ; le taux de sortie de misère du joueur = celui des PNJ à investissement égal ; un bot après faillite a un taux de sortie de misère strictement positif à 5 ans, sur toutes les graines (plancher anti-impasse, 6.24).

**Mesures** : distribution de richesse, salaire réel médian, taux de grève, taux de sortie de misère à 5 et 10 ans, part du temps passé aux besoins, distribution des repas par mode (corrélée au salaire réel et au niveau de logement), temps libre par décade (décroissant avec la richesse), taux d'assistanat (part des foyers dont les transferts couvrent plus de la moitié du panier), taux d'endettement des foyers, incidence des séquelles, variance des emplois du temps (aucune journée identique deux fois), churn des firmes par empathie du décideur, écart de valuation (salaires contre utilités visibles), incidence d'angoisse et de dépression, chaque réplique traçable au journal, concordance agrégé/individuel, temps réel de simulation (benchmark, tranche 10).

**Observabilité (conventions 26 et 27)** : séries synthétiques à pas fixe (quotidien pour la bulle chaude, décennal pour la tendance) des agrégats : effectifs par cohorte, G, salaire réel, prix affichés, états moyens, trésoreries des firmes ; tableau de bord textuel avec diff entre deux dates ; reprise depuis un snapshot (itérer sur un bug sans resimuler 10 ans) ; requête « explique » qui liste les contributions terme à terme à une variation (pourquoi G a monté à l'an 7 : u, p, q, contagion). Le journal causal explique un événement ; les séries disent quand regarder.

## 10. Découpage en tranches Cursor

Chaque tranche suit la convention 25 et se ferme par ses tests. Rien n'est ajouté à une tranche ouverte.

**Noyau réductible** : les tranches 1 à 8 plus P1b minimal forment le P1 réduit, livrable seul. La tranche 9 (vie sociale : empathie, contagion, rituels, valuation, groupes informels) et les enrichissements peuvent glisser à un P1.5 si le temps presse, sans casser aucun test des tranches 1 à 8. Cette réduction est une décision de gestion de périmètre, pas une coupure de design.

**Contenu de P1.5** (tampon, dans l'ordre d'entrée) : couplage des créneaux (fréquentation par recouvrement, primes de créneau, équipes tournantes, lits chauds, 6.22) ; colportage de rumeurs (graine de la route 2) ; troc et débrouille informelle ; tensions d'affinité (micro-affronts) ; habitudes spatiales (stress à la fermeture d'un lieu familier). Le cœur P1 reçoit en revanche les trois protections de validité : élasticité et délais du marché de la ville (6.2), enfants contributeurs, plancher anti-impasse.

| # | Tranche | Contenu | Tests de fermeture |
| --- | --- | --- | --- |
| 1 | Noyau temporel | horloge, séquences, file d'événements, tri par (jour, phase, priorité, id), état sérialisable (spec détaillée en section 13) | événements d'un même jour traités dans l'ordre canonique ; permutation d'insertion → même journal (I9) |
| 2 | Identités et journal | ids persistants, individus et cohortes, extraction, journal causal (spec détaillée en section 14) | I1, I7 ; extraction répétée 100 fois sans dérive |
| 3 | Besoins, temps, santé et états | budget de temps, consommation, foyer, enfants contributeurs (6.23), maladie et soins, séquelles, états déclenchés (5 variables, préemption par urgence), angoisse et dépression (spec détaillée en section 15) | I4 ; un individu riche libère du temps, un individu affamé en perd ; une blessure réduit la capacité de façon traçable ; un bot sans doctrine survit ; un état critique interrompt l'emploi du temps ; la dépression est rare et réversible par causes ; les corvées d'enfants amortissent le temps de survie du foyer |
| 4 | Firmes et production | recettes d'atelier, stocks, actifs et usure paresseuse, maisonnées des patrons (spec détaillée en section 16) | I2, I6 ; usure calcule paresseusement = usure calculée naïvement |
| 5 | Marché, prix, repas et crédit | prix affichés, reprice par Q(a), allocation en pénurie, élasticité et délais du marché de la ville (6.2), modes de repas (cantine, épicerie, restaurant), eau refacturée, fiscalité forfaitaire, prêts (firme et entre individus), saisie, sous-location post-saisie (plancher, 6.24) (spec détaillée en section 17) | I2, I3, I5 ; chocs 1, 2, 10, 11 et 12 ; le choc 2 produit une hausse de prix de référence différée, pas une neutralisation ; une saisie est explicable de bout en bout ; la distribution des repas suit le salaire réel |
| 6 | Marché du travail, métiers et parrainage | cinq grandeurs, Q(a) des firmes modulée par les traits du décideur, métiers (triptyque, décisions ponctuelles), étuve et loisir, parrainage (spec détaillée en section 18) | embauche/licenciement expliqués ; chocs 3 et 4 ; un parrainé obtient un essai plus vite qu'un isolé ; deux firmes identiques sauf l'empathie du décideur divergent en churn et en maîtrise |
| 7 | Salaires réels et griefs | panier, p, q, G, concordance (spec détaillée en section 19) | I8 ; grève seulement si salaire réel bas (choc 8) |
| 8 | Basculements | seuil de risque cumulé, grève (spec détaillée en section 20) | même graine → même date de grève ; re-tirage impossible |
| 9 | Pratiques, reconnaissance et vie sociale | maîtrise, apprentissage, plafond vétérans, reconnaissance, valuation des rôles, empathie et contagion, groupes informels, rituels, naissances et décès, héritage (spec détaillée en section 21) | montée de niveau par exécution ; un élève isolé progresse moins vite ; un deuil baisse l'offre de travail du groupe (scénario 13) ; naissance/décès conservent I1 |
| 10 | Suite headless et bots | 13 scénarios (équilibre + 12 chocs), 5 bots, 5 graines, benchmark (spec détaillée en section 22) | tous les invariants verts ; rapport de mesures ; un bot après faillite sort de la misère à 5 ans sur toutes les graines (6.24) ; le choc 2 est amorti avec retard et signifiant (6.2) |
| 11 | P1b, écrans minimaux | doctrine personnelle, planification, vue quartier, journal (spec détaillée en section 23) | un humain joue 30 minutes et comprend pourquoi il s'en sort ou non ; aucune attente passive non signalée ; le classement par valeur d'intervention suffit à tenir le joueur informé sans surveillance continue |

## 11. Décisions tranchées (session « profondeur P1 »)

1. **Reprice = action de la firme** via Q(a) ; l'heuristique par seuils de stock est le comportement par défaut en données, pas une règle du moteur.
2. **Allocation en pénurie par file** (arrivée, rang, id), pas par rationnement proportionnel.
3. **P1b limité aux écrans** : la marche et le terrain sont tranchés pour P2.
4. **Parc immobilier fixe et différencié** : 45 logements en 3 niveaux, sous-location permise (le sous-locataire paie moins mais sa densité monte), construction hors périmètre P1.
5. **Trois cohortes au départ** ; le regroupement des journaliers et ouvriers en une seule cohorte est testé en tranche 10.
6. **Cinq enrichissements inclus dans P1** : crédit et saisie, santé-trajectoire, qualité des biens, vie vitale minimale (naissance, décès, héritage), parrainage.
7. **Eau courante refacturée au prix du biome** (5-10 % du panier, import de la ville, une seule facture) ; hygiène = attribut du niveau de logement (niveau 1 : sanitaires communs, sans cuisine).
8. **États individuels cachés** (faim, fatigue, stress, ennui, douleur), états déclenchés, préemption par urgence ; le stress est le collecteur unique de santé mentale ; l'ennui alimente le stress et pèse dans Q(a) vers le loisir.
9. **Chaque bien et service porte (coût monétaire, coût temporel)** : le luxe convertit l'argent en temps ; gradient du repas de la cantine au cuisinier de maisonnée.
10. **Règle de diversité** : point distinct dans (monétaire, temporel, social) + rôle dans une boucle ; restaurant en P1 ; estaminet en réserve.
11. **Métiers** : triptyque pratique × recette × poste, jamais de minigame ; 10 métiers P1 ; maisonnée = firme ; joueur-patron limité aux firmes de service abordables, piloté par directives ; **pas de niveaux discrets par firme** : taille, état des actifs et gamme de recette suffisent, l'investissement (action Q(a)) débloque les recettes supérieures.
12. **Fiscalité forfaitaire, sans acteur** : impôt 10 % à la source, allocation vieillesse, aide par enfant ; grandeurs en net ; politique fiscale reportée à P2, incitations à P3.
13. **Attractivité enregistrée dès P1**, consommée par les routes 10, 11 et 6 (migrations, dérive culturelle).
14. **Vivacité des PNJ** : hétérogénéité par clé stable, habitudes persistantes et visibles, mémoire active, variance causale, répliques générées depuis le journal, groupes informels de 3 à 6, rituels sur les événements de vie, expression qualitative des états ; jamais de décor.
15. **Empathie = trait et canal de contagion** (pas de second registre de souffrance) ; angoisse anticipatoire ; dépression = état persistant du stress, rare et réversible ; dissonance morale → révision de doctrine ou transgression.
16. **Traits du décideur modulent Q(a) de la firme** ; empathie = patience et tolérance, bornée par la pérénité ; employabilité modulée par la tolérance ; deux régimes de gestion viables, aucun dominant.
17. **Valuation des rôles pondérée en données dès P1** (curseurs fixes, jamais le rang de revenu codé en dur) ; l'écart de valuation est une mesure ; les vocations suivent la reconnaissance.
18. **Solidarité et lobbying : P1 = régime « rien »** (dispensaire privé à l'acte, aucune assurance) ; les institutions de solidarité et les groupes de pression entrent en P2 (cadrage, route 12).
19. **Créneaux, pas horloge** : quatre étiquettes par jour sur postes, lieux et emplois du temps dès le cœur P1 ; le couplage (fréquentation, primes de créneau, équipes tournantes, lits chauds) entre en P1.5 ; le rythme du quartier est une émergence, jamais une règle.
20. **Élasticité de la ville** : la courbe d'offre du marché de la ville est en données (prix de référence montant avec le volume demandé, délais logistiques 3 à 10 jours) ; la ville amortit les chocs locaux avec retard, elle ne les neutralise pas ; aucune règle d'équilibre du moteur.
21. **Enfants contributeurs** : participation domestique progressive dès 8 ans (corvées, garde des petits, plafond 2 demi-heures/jour), jamais d'emploi salarié avant l'âge adulte ; l'aide par enfant (6.16) amortit le reste.
22. **Plancher anti-impasse = propriété testée, pas règle** : quatre voies assemblées (entraide, servitude, maisonnée, sous-location précaire) ; critère : taux de sortie de misère post-faillite strictement positif à 5 ans sur toutes les graines ; un échec est un défaut de calibrage.
23. **Noyau temporel (section 13)** : id = compteur global monotone, jamais réutilisé, inclus dans l'état sérialisé ; report par défaut au prochain instant (phase suivante du même jour, sinon lendemain phase 1) ; toute date explicite strictement postérieure à l'instant courant, sinon rejet ; jamais d'exécution réentrante.
24. **Journal et calendrier hors état** : le journal est un produit dérivé append-only, reconstituable par rejeu ; le calendrier d'affichage (mois, années) est une conversion en données, hors état canonique.
25. **Identité (section 14)** : un compteur par type d'entité, dans l'état, jamais réutilisé ; morts et firmes fermées archivés, pas supprimés ; références toujours par id ; noms dérivés de (graine, id), hors état canonique.
26. **Cohorte = totaux, moyennes dérivées** : l'extraction déduit exactement les valeurs de l'individu, la réinjection les absorbe ; fusion de cohortes strictement conservatrice ; toute future grandeur agrégée suit la même règle.
27. **Journal causal = graphe acyclique à liens explicites** : chaque entrée référence ses causes strictement antérieures ; la requête « explique » remonte les liens ; le compactage résume sans casser la résolubilité.
28. **Bouchons de tranches précoces** : la tranche 3 tourne avec un approvisionnement à prix fixe et un dispensaire à capacité fixe, en données ; remplacés par les tranches 4 et 5 ; aucun test de tranche 3 ne dépend d'un bouchon.
29. **Entrées actives du stress en tranche 3** : faim, fatigue, ennui, douleur ; l'entrée griefs reste à zéro jusqu'à la tranche 7, la précarité entre avec la tranche 5.
30. **Interruption d'urgence = replanification par événements normaux** : l'urgence annule et replanifie les actions restantes du jour via le report du noyau (13.3), jamais par manipulation directe de la file.
31. **Bouchons de tranche 4** : débouché synthétique (demande à prix fixe) et dotation de travail synthétique (heures par poste), en données ; remplacés par les tranches 5 et 6 ; aucun test de tranche 4 ne dépend d'un bouchon.
32. **Usure paresseuse = (état initial, date, taux)** : jamais d'état courant stocké ; franchissements de seuil planifiés à la date exacte et recalculés à chaque changement d'entrée, même mécanisme que le taux de risque continu (cadrage ch. 2).
33. **Politiques avant Q(a)** : en tranche 4, production et entretien sont des politiques en données, la table Q(a) complète entre avec les tranches 5 et 6 ; la maîtrise est une entrée du rendement dès la tranche 4, son accumulation attend la tranche 9.
34. **Q(a) économique en tranche 5** : la table d'actions reçoit reprice, achats d'intrants, vente à la ville et prêt ; les actions d'emploi attendent la tranche 6.
35. **Ville hors état des firmes** : l'import passe par des commandes datées (arrivée à J + délai), le prix de référence est mis à jour par un événement quotidien de demande agrégée ; aucune règle d'équilibre.
36. **Choc 11 en deux temps** : en tranche 5, flux fiscaux et réponse de u/p ; l'assertion griefs s'ajoute en tranche 7 quand G existe.
37. **Sous-location post-saisie = voie par défaut proposée** : jamais automatique, acceptée par Q(a) du nouveau propriétaire ; l'expulsion ne produit jamais de sans-abri sans voie praticable ; test partiel en tranche 5, statistique en tranche 10.
38. **Appariement événementiel** : le marché du travail apparie volontés et demande par cohorte et pratique, événement par événement ; jamais un marché clearing, aucune convergence garantie.
39. **Décisions ponctuelles de métier** : même table Q(a) pour le joueur et les PNJ (~1/mois, à clé stable) ; aucune règle spéciale au joueur.
40. **Transition bouchon → marché sans saut** : les heures de la dotation de travail sont reprises par les postes pourvus au fil des embauches ; l'état agrégé est continu à la bascule.
41. **G intégré événementiellement** : mise à jour au changement d'une entrée, jamais par pas fixe ; identique à l'intégration continue de référence (T7.4).
42. **Choc 8 en deux temps** : entrées du seuil de risque calculées en tranche 7, basculement en tranche 8.
43. **Basculement = date exacte planifiée** : le franchissement du seuil de risque réutilise le mécanisme de l'usure paresseuse (16.3) ; le tirage a lieu une seule fois au franchissement, les dates sont recalculées quand les entrées changent.
44. **Durée de grève tirée une fois** : journalisée, jamais re-tirable ; la fin de grève a toujours une cause journalisée.
45. **Apprentissage événementiel** : l'expérience vient de l'exécution, jamais d'un timer ; le plafond 1 vétéran / 3 élèves est une règle de transmission, pas de progression.
46. **Contagion par les groupes** : la souffrance perçue alimente le stress puis remonte aux griefs via les groupes informels ; aucun second registre de souffrance.
47. **Naissances agrégées** : les enfants sont des effectifs de cohorte ; les individus nommés n'existent qu'à l'extraction, vers 14 ans ; naissance et décès conservent I1.
48. **Tranche 10 sans mécanisme** : elle joue la suite et produit le rapport ; tout échec remonte à la tranche fautive, jamais corrigé ici.
49. **Benchmark 300 ans = critère late game du palier** : latence et taille de file sous les seuils fixés en 13.6 et 13.7.
50. **P1b sans mécanisme** : les écrans lisent l'état et le journal, éditent doctrine et planification (déjà des données) ; aucun pouvoir que les PNJ n'ont pas.
51. **Sélection et rendu séparés** : choisir de parler est une décision Q(a) journalisée ; le texte est un rendu hors état, à la demande ; messages Fluent entiers, attributs grammaticaux en données ; la locale ne touche jamais la simulation ; le LLM n'est qu'un outil d'écriture du corpus.
52. **Parler = action Q(a) à intention** : six intentions (se plaindre, partager, inciter, demander, entretenir, transmettre), portées par des variables existantes ; aucun moteur d'intentions parallèle.
53. **Catharsis dégressive, G fermé** : la répétition décroît (×0,5) ; la contagion passe par le stress ; l'équation de griefs ne dépend que de u, p, q.
54. **L'agitation amplifie, elle ne crée pas** : un agitateur accélère la chaîne existante ; T8.3 (aucune grève en régime sain) reste le garde-fou ; la démagogie organisée est une route P2.
55. **Contextualisation = trois filtres en données** (relation, lieu, créneau) : ils filtrent la sélection et modulent effets et risques, sans multiplier les corpus ; conséquences émergentes par propagation normale, aucune règle de dénonciation.
56. **Le seuil d'intimité est un seuil d'information** : les états cachés d'autrui ne sont visibles qu'aux niveaux relations et intime, plus les indices visibles et les témoignages ; aider est une action Q(a) via les transferts existants, bornée par la doctrine.
57. **Un décideur ne voit que ce qui lui parvient** : la Q(a) d'un décideur nommé raisonne sur son information perçue (dates d'apprentissage), jamais sur l'état réel ; information sociale en P1.5, rapports falsifiables en P2.
58. **L'estaminet entre en P1.5** : point distinct validé (comptoir debout, prix bas, conversation longue ; le lieu des conversations politiques) contre le restaurant (repas complet, assis, plus cher) ; confirmé par le test de diversité.
59. **Émigration et griefs (P2, route 10)** : le départ d'un proche est un événement de vie (mémoire, reconnaissance) ; l'émigré sort des comptes par transfert déclaré (I1) ; griefs résiduels seulement si le départ est perçu comme abandon (affront, en données).
60. **Pain-et-jeux (P2)** : le levier des factions est le financement d'occasions sociales et de rituels (fête de quartier) ; coût réel, rendement décroissant, transparence dépend de la fiabilité des rapports (route 2).
61. **Trappe des allocations** : l'allocation vieillesse (½ panier) est structurellement sous le seuil de survie ; critère : taux de sortie de misère insensible aux allocations (choc 11) ; aucune conditionnalité en P1.
62. **Écran de conversation** : intentions filtrées par le contexte, cible choisie, jamais de texte libre ni de minigame ; l'écoute passive est une source d'information datée ; les intentions indisponibles sont grisées avec leur raison.
63. **Engagement employé ↔ firme** : scalaire par couple (0-1000), mis à jour uniquement par les décisions observées de la firme (× empathie × proximité, mémoire saillante), demi-vie 1 à 2 ans ; jamais lu par le décideur (décision 57) ; il pondère les décisions d'arrêt, de soin et de démission des salariés.
64. **Prévention au lieu de bonus** : l'engagement ne produit ni temps de travail supplémentaire ni bonus de production ; il réduit la probabilité d'incidents (usure ralentie, en données) et débloque des options de résolution dans les décisions ponctuelles ; la productivité supérieure est une mesure émergente (production par heure travaillée), jamais un coefficient. Un temps donné ou un multiplicateur dérivé de l'engagement est une faute de conception (double comptage du travail déjà payé en heures).
65. **Contagion de maladie à l'atelier** : l'exposition aux malades présents (même lieu, même créneau) est un facteur de risque de 6.9 ; travailler malade allonge la maladie et monte le risque de séquelle ; le présentéisme est une décision Q(a) pondérée par l'engagement, jamais une règle ; aucun régime de gestion ne devient dominant (choc de demande).
66. **Recettes multi-postes à goulot** : une recette déclare des étages (pratique requise × capacité), la production de la firme est le min des capacités pourvues, recalculée par événement (embauche, départ, incident), goulot journalisé ; aucune chorégraphie par objet ni sous-action de métier instanciée (préparer, cuire, dresser) : on simule la contrainte, jamais les gestes.

Routes approuvées pour P2 et plus (voir cadrage, ch. 17) : vol et milice, réseau de faveurs complet, spéculation et revente individuelle, migrations inter-quartiers (route 10) et interplanétaires (route 11), institutions de solidarité et groupes de pression (route 12). Pour le combat (P2+) : le stress individuel et de cohorte sera une entrée du moral des unités, la panique un basculement (cadrage ch. 11).

## 12. Critères d'acceptation humaine (revue de palier)

- Le quartier sans joueur pendant 10 ans produit des trajectoires variées et toutes expliquées par le journal.
- La misère résiduelle vient des prix et des évènements, jamais d'une trappe artificielle.
- Aucun individu ne sort de la misère sans cause identifiable (embauche, héritage, chance explicite).
- Les chiffres de la tranche 10 sont revus à la main, avec la fiche de revue (convention 24).

## 13. Spécification de la tranche 1 : le noyau temporel

**Objet** : la tranche 1 ne livre que l'ossature : horloge, file d'événements, ordre canonique, report des événements générés, état sérialisable. **Aucune sémantique métier** : le noyau ne connaît ni individu ni firme, seulement des événements typés en données. Toutes les tranches suivantes s'insèrent dans ce cadre sans le modifier.

### 13.1 Horloge

- Compteur de jours, entier non signé, démarrage J0. **Aucune heure du jour dans le noyau** : les demi-heures et créneaux (6.1, 6.22) sont des attributs de planification des entités, jamais de l'horloge.
- Calendrier d'affichage (mois, années ; 30 jours/mois à calibrer) : pure conversion pour le rendu et les séries, **hors état canonique**.
- L'horodatage logique d'un événement est sa clé de tri complète : (jour, phase, priorité, id).

### 13.2 Événement

| Champ | Type | Note |
| --- | --- | --- |
| jour | u64 | unité d'ancrage |
| phase | u8 | 1 à 7, recensement de la section 5, ensemble fermé |
| priorité | i32 | par type d'événement, en données, défaut 0 |
| id | u64 | compteur global monotone, jamais réutilisé |
| type | id | type d'événement en données |
| payload | données du type | références par id, sérialisable |

La clé (jour, phase, priorité, id) est **totale** : l'id unique interdit tout ex æquo, l'ordre canonique est sans ambiguïté.

### 13.3 File, journée et report

- File de priorité (min-heap) sur la clé. Traiter un jour J = extraire tous les événements de jour ≤ J dans l'ordre canonique ; la journée est finie quand la file ne contient plus d'événement de jour ≤ J.
- **Report (règle de la section 5, précisée)** : tout événement généré pendant le traitement est daté d'un instant **strictement postérieur** à l'instant courant (jour, phase en cours). Sans date explicite, le noyau le date au prochain instant : phase suivante du même jour, sinon lendemain phase 1. Avec date explicite (échéance de prêt, fin de choc), elle doit être strictement postérieure à l'instant courant, sinon rejet (invariant, arrêt du replai). Jamais d'exécution réentrante : aucune boucle causale ne peut boucler intra-instant, et le générateur ne choisit jamais la date de ses conséquences immédiates.

### 13.4 État sérialisable

- État du noyau = { horloge, compteur d'id, état du RNG (flux déterministes, un par usage), file sérialisée dans l'ordre de la clé, registres des systèmes (vides en tranche 1) }.
- Sérialisation canonique : champs dans un ordre fixé par le code, entiers à boutisme fixé, aucune itération de table de hachage (conventions) ; deux états logiquement égaux se sérialisent en octets identiques.
- Snapshot = état + version du format + version des données. Reprise : charger et continuer, aucun recalcul (convention 27).
- **Le journal est hors état** : produit dérivé append-only, reconstituable par rejeu ; le compactage (phase 7) ne touche jamais l'état canonique.

### 13.5 Pseudo-code de la boucle

```
boucle principale:
  J = horloge
  tant que la file contient un événement de jour ≤ J:
    e = extraire_min()                    // clé (jour, phase, priorité, id)
    instant_courant = (e.jour, e.phase)
    exécuter(e)                           // peut appeler générer()
  horloge = J + 1                         // fin de journée

générer(type, payload, date_explicite?):
  date = date_explicite ? date_explicite
                        : prochain_instant(instant_courant)
  invariant: (date.jour, date.phase) > instant_courant   // sinon rejet
  insérer { date.jour, date.phase, priorité(type), id: compteur++, type, payload }

prochain_instant((j, p)) = (j, p + 1) si p < 7 sinon (j + 1, 1)
```

### 13.6 Coût et effet sur les autres couches

- Coût : O(log n) par événement, n = taille de la file, la seule structure globale chaude du moteur. Mesure de suivi : taille de la file à 300 ans (plafond de vigilance, à calibrer avec la charge de la tranche 10).
- Effet : les couches ne datent ni ne trient rien elles-mêmes ; elles déclarent des types d'événements (priorités en données) et des dates explicites. Le report garantit la lisibilité causale du journal : causes avant effets, jamais entrelacées dans un même instant.

### 13.7 Tests de fermeture (headless, sans métier)

| Test | Montage | Critère |
| --- | --- | --- |
| T1.1 Ordre canonique | 10 000 événements aléatoires du même jour, insérés dans un ordre aléatoire | extraction dans l'ordre strict (phase, priorité, id) |
| T1.2 Permutation d'insertion | même ensemble, 10 ordres d'insertion différents | journaux identiques bit à bit (I9) |
| T1.3 Report | un type d'événement qui se régénère lui-même | exactement un événement par instant, avance d'un jour par cycle, arrêt propre à l'horizon, aucune boucle intra-instant |
| T1.4 Date passée | générer un événement daté ≤ instant courant | rejet par invariant, arrêt du replai |
| T1.5 Snapshot/reprise | 100 jours, snapshot, 100 jours ; reprise du snapshot | journal de 200 jours identique bit à bit à la simulation directe |
| T1.6 Sérialisation canonique | même état logique construit dans deux ordres différents | octets identiques |
| T1.7 Benchmark | 1 million d'insertions + extractions en file chargée ; charge synthétique sur 300 ans | temps sous seuil à mesurer ; taille de file bornée et stable |

## 14. Spécification de la tranche 2 : identités et journal

**Objet** : le régime d'identité (ids persistants, individus et cohortes, extraction/réinjection) et le journal causal. Toujours sans sémantique économique : aucune transaction, seulement des identités, des totaux conservés et des traces.

### 14.1 Ids persistants

- **Un compteur par type d'entité**, inclus dans l'état sérialisé ; id interne = (type, numéro). Jamais réutilisé : les morts et les firmes fermées sont **archivés** (champs figés, retirés des index actifs), pas supprimés ; toute référence reste résoluble.
- Les références entre entités sont toujours par id, jamais par pointeur ni par nom. Les noms (PNJ, firmes) sont **hors état canonique** : dérivés de façon déterministe de (graine, id) pour l'affichage.
- L'id d'événement reste le compteur global monotone du noyau (13.2) ; les compteurs d'entités sont distincts.

### 14.2 Individus et cohortes : la règle d'identité

- **La cohorte porte des totaux** : effectif, monnaie totale, somme des griefs pondérée, attachement total. Les moyennes (griefs 0-100, attachement moyen) sont **dérivées** à l'affichage et pour I8, jamais stockées. Un individu est un échantillonnage déterministe de sa cohorte (clé stable), cohérent avec I8 par construction.
- **Extraction** (individu nommé par ses actes, événement de phase 6) : l'individu sort de la cohorte avec ses valeurs propres ; la cohorte déduit exactement (effectif − 1, monnaie − solde, somme des griefs − contribution, attachement − contribution). **Réinjection** : addition réciproque ; l'individu ramène ses valeurs devenues divergentes, la cohorte les absorbe. Critère d'extraction en données (seuils de richesse, de griefs, événement saillant, à calibrer) ; plafond d'individus nommés simultanés (30, à calibrer).
- **Fusion de cohortes** (regroupement de proches) : strictement conservatrice, somme des totaux, moyennes recalculées ; jamais de perte.
- Aucun autre individu ni cohorte n'est modifié par une extraction : l'opération est locale et réversible.

### 14.3 Journal causal

- Une entrée par événement traité (produit dérivé, décision 24) : id d'événement, date, phase, acteur (id), décision, valeurs modifiées, **liens causaux explicites** (ids des événements antérieurs qui causent celle-ci), date d'apprentissage (quand l'information devient connue d'un acteur ; usage minimal en P1, préparatoire au chapitre 16).
- Le journal forme un **graphe causal acyclique** : les liens pointent toujours vers des événements strictement antérieurs (garanti par le report du noyau, 13.3).
- La requête « explique » (convention 27) remonte les liens depuis un événement jusqu'aux racines ; c'est le seul outil d'explication, aucune trace parallèle.
- Compactage (phase 7) : les entrées vieilles au-delà de la rétention sont résumées ; leurs liens restent résolubles via les résumés. Le compactage ne touche jamais l'état canonique.

### 14.4 Coût et effet sur les autres couches

- Extraction/réinjection : O(1) par opération. Journal : append-only, écriture O(1), lecture par requête uniquement.
- Effet : toutes les couches suivantes référencent par id et journalisent par liens causaux ; la règle d'identité (totaux en cohorte, moyennes dérivées) s'applique à toute future grandeur agrégée (u, p, q, G, salaire réel de la section 4).

### 14.5 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T2.1 Extraction répétée | extraire/réinjecter le même individu 100 fois, alternées avec d'autres extractions | totaux de la cohorte strictement identiques après chaque cycle (I1), zéro dérive |
| T2.2 Isolation | extraction d'un individu | aucun autre individu ni cohorte modifié ; diff d'état limitée à l'individu et à sa cohorte |
| T2.3 Validité du journal | 1 an de simulation à événements aléatoires | chaque entrée référence des ids existants à sa date, liens strictement antérieurs (I7) |
| T2.4 Requête « explique » | chaîne causale d'un événement profond | remonte jusqu'aux racines, sans cycle, sans trou ; le résumé d'une entrée compactée reste résoluble |
| T2.5 Non-réutilisation | 100 000 événements, morts et fermetures d'entités | aucun id réutilisé ; compteurs monotones dans l'état |
| T2.6 Fusion conservatrice | fusion de deux cohortes proches | totaux conservés, moyennes dérivées correctes, I8 vert avant et après |
| T2.7 Snapshot avec extraits | snapshot avec 10 individus nommés, reprise | identités, références et totaux identiques bit à bit |

## 15. Spécification de la tranche 3 : besoins, temps, santé et états

**Objet** : première couche à sémantique : budget de temps, consommation du foyer, enfants contributeurs, santé-trajectoire, états déclenchés, angoisse et dépression. Les firmes et les marchés n'existent pas encore : **l'approvisionnement est un bouchon** (source à prix fixe en données, stock infini) remplacé par les tranches 4 et 5 ; le dispensaire est un bouchon à capacité fixe. Aucun test de la tranche 3 ne dépend du comportement d'un bouchon.

### 15.1 Budget de temps (exécution de 6.1)

- Chaque individu nommé résout chaque jour l'arbitrage des 32 demi-heures : besoins d'abord (repas, trajet, corvées), puis travail, puis formation et loisir ; l'ordre de résolution est la doctrine, en données. Les non-nommés sont résolus **en agrégé par cohorte** (mêmes équations, heures moyennes par poste), conformément aux niveaux de détail ; la bulle du joueur est la seule résolution individuelle étendue.
- I4 vérifié chaque jour pour chaque nommé et chaque cohorte. Jeûne : −1 demi-heure d'éveil par jour et demi sans nourriture. Garde-fou de calibrage : temps de survie ≤ 12 demi-heures au niveau de vie de référence.

### 15.2 Consommation et foyer

- Le foyer est l'unité d'achat domestique : stock de nourriture, consommation quotidienne par membre, deux qualités (la basse use la santé, 6.9). u du foyer → u des individus et de la cohorte par moyenne pondérée (règle d'identité, 14.2).
- Le bouchon vend au prix fixe en données ; les files, prix affichés et reprice attendent la tranche 5. Les chocs d'offre ne sont jouables qu'à partir de la tranche 5.

### 15.3 Enfants contributeurs (exécution de 6.23)

- Corvées des enfants (effectifs de cohorte et nommés de 8 à 14 ans) : 0,5 à 1,5 demi-heures/jour selon l'âge, plafond 2 ; elles réduisent les corvées du foyer, jamais le temps de travail des adultes directement. Aucun emploi salarié.
- L'aide par enfant (6.16) est créditée au foyer comme transfert déclaré (I3), même en régime bouchon.

### 15.4 Santé-trajectoire (exécution de 6.9)

- Maladie et blessure : événements à clé stable (graine, individu, date), fréquence liée à la densité du logement, à la qualité de la nourriture et à l'usure ; gravité bénigne ou grave.
- Soins : file du dispensaire (bouchon à capacité fixe en données jusqu'aux tranches 4 et 6) ; soigner coûte du temps et de l'argent ; ne pas se soigner allonge la maladie et monte le risque de séquelle ; la séquelle est une réduction permanente de capacité, en points en données.
- La santé moyenne de la cohorte alimente q (consommée par les griefs en tranche 7).

### 15.5 États déclenchés, angoisse, dépression (exécution de 6.12 et 6.19)

- Cinq variables cachées par nommé (0-1000), accumulation quotidienne déterministe à clé stable, dérivées des variables existantes. **Entrées actives en tranche 3** : faim ← u ; fatigue ← budget d'éveil (majoration nocturne, 6.1) ; ennui ← déficit de loisir ; douleur ← santé. L'entrée griefs du stress reste à zéro jusqu'à la tranche 7 ; la précarité (dette, logement) entre avec la tranche 5.
- Seuils en données → événement d'état journalisé (phase 6). Un état **critique** déclenche une **interruption d'urgence** : les actions non encore exécutées du jour sont annulées et replanifiées **par des événements normaux via le report du noyau (13.3)**, jamais par manipulation directe de la file ; l'urgence consomme le temps qu'il faut et l'emploi du temps reprend le lendemain.
- Stress = collecteur unique de santé mentale : rendement et apprentissage réduits au-delà du seuil (à calibrer). L'ennui pèse dans Q(a) vers le loisir (consommé en tranches 4 et 6) et alimente le stress.
- **Angoisse** : état anticipatoire (épargne, stocks, tendance), à clé stable. **Dépression** : état persistant du stress, rare et réversible par causes ; rémission = disparition prolongée des causes, sans réinitialisation artificielle.

### 15.6 Coût et effet sur les autres couches

- Coût : O(1) par individu et par jour ; les états sont mis à jour en phase 6, un seul passage. Les états sont conçus pour traverser la règle d'identité (agrégation par cohorte, extraction des nommés) et la future entrée du moral (P2, rien à implémenter).
- Effet : Q(a) des firmes (tranches 4 et 6) consomme fatigue, stress et ennui ; les griefs (tranche 7) consomment u, p, q et le stress ; l'emploi du temps devient la surface d'édition du joueur (P1b).

### 15.7 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T3.1 Budget d'éveil | 1 an de simulation, 150 individus | I4 vert chaque jour pour chaque nommé et cohorte ; échec = arrêt du replai |
| T3.2 Richesse libère du temps | deux individus identiques sauf richesse | le riche a plus de temps libre (repas servis, logement équipé), mesuré par décade, décroissance continue |
| T3.3 Jeûne traçable | un individu privé de nourriture | perte de demi-heures d'éveil journalisée, chaîne causale complète |
| T3.4 Blessure → séquelle | blessure grave, soin ou non | capacité réduite de façon traçable ; le non-soin allonge la maladie et produit la séquelle |
| T3.5 Bot sans doctrine | un bot sans doctrine, 1 an | il survit par l'autopilote des besoins ; aucune attente passive non signalée |
| T3.6 Interruption d'urgence | état critique en pleine journée | actions restantes annulées et replanifiées par événements normaux, journalisées ; reprise normale le lendemain |
| T3.7 Dépression | stress élevé prolongé puis causes supprimées | incidence sous le seuil (à calibrer) ; rémission observée par disparition des causes, sans réinitialisation |
| T3.8 Corvées d'enfants | foyers avec et sans enfants de 8 à 14 ans | temps de survie par adulte amorti (corvées + allocation) ; foyer mono-parental à trois enfants : survie positive |
| T3.9 Concordance de u | u cohorte contre u individuels | moyenne pondérée exacte (règle d'identité, 14.2), tolérance zéro |

## 16. Spécification de la tranche 4 : firmes et production

**Objet** : la firme comme unité de production : recettes, stocks, actifs, usure paresseuse, entretien, maisonnées. La vente et l'embauche n'existent pas encore : **deux bouchons**, un débouché synthétique (demande à prix fixe en données) et une dotation de travail synthétique (heures par poste en données), remplacés par les tranches 5 et 6. Aucun test de la tranche 4 ne dépend d'un bouchon.

### 16.1 Structure et décisions

- La firme porte les champs de la section 4 ; son décideur est un individu nommé (les traits de 6.20 moduleront Q(a) en tranche 6). En tranche 4, les décisions de production sont **des politiques en données** (politique d'entretien, plan de production) ; la table Q(a) complète (embaucher, licencier, investir, fermer) entre avec les tranches 5 et 6.
- **Maisonnée = firme** (6.15) : type maisonnée, recettes domestiques (entretien, cuisine, soins), patron = individu nommé ; strictement les mêmes structures et le même code que l'atelier, aucun cas spécial.

### 16.2 Recettes et production

- Une recette est en données : (intrants, heures par poste, production, qualité). Les intrants sont un import (bouchon à prix fixe en tranche 4).
- Exécution quotidienne en phase 1 : **production = min(capacité des actifs, heures disponibles, intrants disponibles)** ; chaque facteur contraignant est identifiable au journal. Les intrants sont consommés avant production, jamais de stock négatif (I2).
- Le niveau de maîtrise des porteurs module le rendement (coefficients en données) ; l'accumulation d'expérience et l'apprentissage entrent en tranche 9, la maîtrise n'est ici qu'une entrée de calcul.

### 16.3 Usure paresseuse (6.4)

- Un actif stocke **(état initial, date, taux)** et jamais l'état courant : état(t) = état_initial − taux × (t − date), en entiers. Le franchissement des seuils (capacité réduite sous 400, critique sous 100) est un **événement planifié à la date exacte de franchissement**, recalculée quand les entrées changent : le même mécanisme que le taux de risque continu du cadrage (ch. 2), appliqué à la matière.
- L'entretien (politique minimale, normale, renforcée en données) consomme composants et heures et modifie le taux ou remonte l'état ; chaque changement replanifie les dates de seuil. La dette d'entretien du quartier alimente q (consommée en tranche 7).
- Coût : O(1) par actif et par jour (une soustraction), aucun recalcul de masse ; le parc d'actifs est borné par le parc immobilier et les ateliers, donc le coût est stable en late game.

### 16.4 Coût et effet sur les autres couches

- La production journalisée en phase 1 nourrit les stocks que les marchés de la tranche 5 vendront ; les heures consommées s'imputent au budget de temps (15.1) via la dotation bouchon puis aux emplois réels (tranche 6). Les états d'actifs et la gamme de recette (6.15) portent toute la différenciation des firmes, sans niveaux discrets.

### 16.5 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T4.1 Stocks | 1 an de production agressive sous bouchons | I2 vert : aucun stock négatif, jamais |
| T4.2 Bornes | tous les actifs et porteurs | I6 vert : états dans [0, 1000], maîtrises dans [0, 10] |
| T4.3 Paresseux = naïf | 10 ans avec changements de politique d'entretien | état paresseux identique bit à bit à un oracle naïf recalculé chaque jour |
| T4.4 Seuils planifiés | usure traversant 400 puis 100, entretien en cours de route | événements de franchissement à la date exacte, replanifiés après chaque changement d'entrée |
| T4.5 Facteur contraignant | ateliers bridés tour à tour par capacité, heures, intrants | production = min des trois, facteur contraignant nommé au journal |
| T4.6 Maisonnée = firme | une maisonnée et un atelier aux données égales | trajectoires identiques bit à bit par le même code |
| T4.7 Maîtrise en entrée | deux ateliers identiques sauf maîtrise des porteurs | rendements proportionnels aux coefficients en données |
| T4.8 Snapshot avec usure | snapshot avec actifs à mi-vie, reprise | états, dates de seuil et production identiques bit à bit |

## 17. Spécification de la tranche 5 : marché, prix, repas et crédit

**Objet** : les marchés de biens et le crédit : prix affichés, reprice, allocation en pénurie, ville élastique, modes de repas, eau, fiscalité, prêts, saisie, sous-location post-saisie. Le marché du travail reste un bouchon (dotation de travail de la tranche 4, remplacée en tranche 6). C'est la tranche la plus dense du cœur : toutes les grandeurs y deviennent monétaires.

### 17.1 Marchés de biens (exécution de 6.2)

- Un marché par bien ; transactions au prix affiché ; en pénurie, allocation par file : arrivée, rang (abonnement d'achat), id ; les enfants du quartier passent devant (priorité en données). Les qualités sont des recettes distinctes sur le même marché.
- **Reprice = première action Q(a) introduite** : la table d'actions de la firme reçoit ici ses actions économiques (reprice, acheter des intrants, vendre à la ville, prêter pour le prêteur) ; les actions d'emploi attendent la tranche 6. Le comportement par défaut est l'heuristique par seuils de stock, en données ; bornes de prix en données (I5).
- Chaque transaction est un événement de phase 2, équilibrée (débit = crédit, I3), journalisée.

### 17.2 Ville élastique (exécution de 6.2)

- La ville est une **source et un puits hors état des firmes** : l'import passe par des commandes datées (événement d'arrivée en phase 1, à J + délai logistique du bien, 3 à 10 jours) ; l'export vend au prix de référence.
- Le prix de référence est mis à jour par un événement quotidien de demande agrégée : il monte avec le volume demandé au-dessus de la capacité nominale (+1 % par 2 %, plafond +50 %, en données) et dérive lentement à clé stable. Aucune règle d'équilibre : les déséquilibres persistants sont des résultats légitimes.
- Effet attendu : un choc local se traduit par des ruptures temporaires puis une hausse de prix différée ; la ville amortit avec retard (test T5.5).

### 17.3 Repas et eau (exécution de 6.14 et 6.13)

- Les modes de repas sont résolus quotidiennement par le foyer selon le couple (coût monétaire, coût temporel) et le niveau de logement (cuisine seulement au niveau 2 et plus) ; le choix est pondéré par la doctrine et le salaire réel. L'ardoise du restaurant est du crédit informel via la table des prêts entre individus.
- L'eau est refacturée dans le loyer (une seule facture), abonnement indexé sur la rareté (5 à 10 % du panier, jusqu'à 25 % en pénurie, en données), élasticité nulle. La coupure d'eau est un événement d'usure des adductions, rétabli par le fontainier-vidangeur ; elle monte densité et risque maladie (15.4).

### 17.4 Fiscalité forfaitaire (exécution de 6.16)

- Impôt 10 % prélevé à la source par la firme (rôle purement comptable), sort vers la ville ; allocations fixes vers les foyers (vieillesse ≈ ½ panier, aide par enfant ≈ ¼), comptées comme transferts déclarés (I3). Toutes les grandeurs s'expriment en net.

### 17.5 Crédit, saisie, sous-location (exécution de 6.8 et 6.24)

- Prêteur firme et prêts entre individus par la même table (plafonnée par la trésorerie du prêteur) ; offres en données (montant, taux 10 %/an, 20 % après défaut, garantie, échéancier) ; intérêts composés en virgule fixe, capitalisés aux échéances.
- Garanties dans l'ordre : logement, puis servitude (heures dues, rémunérées sous le salaire du marché). Le défaut est un événement ; la saisie transfère le logement au prêteur, qui peut le relouer.
- **Sous-location précaire** : après saisie, la famille expulsée se voit proposer de redevenir sous-locataire de son ancien logement (décision 4) : loyer moindre, densité en hausse ; l'acceptation est une décision Q(a) du nouveau propriétaire, jamais automatique. L'expulsion ne produit jamais de sans-abri sans voie praticable : les quatre voies du plancher (6.24) sont toutes actives ici (la maisonnée via le bouchon de travail jusqu'à la tranche 6).

### 17.6 Coût et effet sur les autres couches

- Coût : O(1) par transaction et par bien ; le nombre de marchés est celui des biens en données, donc borné. La ville ajoute un événement quotidien par bien et une file de commandes datées.
- Effet : les salaires réels (tranche 7) consomment panier et fiscalité ; les griefs (tranche 7) consomment u, p, q ; les modes de repas nourrissent le loisir et l'attractivité ; les prêts entre individus sont l'embryon du réseau de faveurs (P2).

### 17.7 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T5.1 Monnaie | 1 an de transactions tous marchés | I3 vert : chaque transaction équilibrée, somme des soldes constante hors transferts déclarés |
| T5.2 Bornes de prix | chocs de prix combinés | I5 vert : prix dans les bornes en données, jamais infini ni nul |
| T5.3 Stocks | pénuries volontaires | I2 vert : stocks de marchés et de foyers jamais négatifs ; files d'attente résolues par (arrivée, rang, id) |
| T5.4 Choc 1 | 1 an sans choc | distribution des prix et des repas stable, explicable |
| T5.5 Choc 2 | import ÷2 pendant 90 jours | ruptures temporaires puis hausse de prix de référence différée du délai logistique ; rétablissement (choc 9) sans réinitialisation |
| T5.6 Choc 10 | taux doublé, garanties exigées | saisies et servitudes explicables de bout en bout ; trajectoires divergentes sans piège inéluctable |
| T5.7 Choc 11 | impôt ×2 pendant un an | flux fiscaux traçables (I3), réponse de u et p correcte ; l'assertion griefs s'ajoute en tranche 7 |
| T5.8 Choc 12 | adductions cédées | consommation d'eau inchangée (élasticité nulle), crête de maladie (15.4), rétablissement par le fontainier-vidangeur |
| T5.9 Saisie explicable | défaut suivi jusqu'à la sous-location | chaîne complète prêt → défaut → saisie → proposition de sous-location au journal, chaque maillon causalement lié |
| T5.10 Repas et salaire réel | distribution des modes de repas | corrélée au salaire réel et au niveau de logement ; cuisine maison uniquement au niveau 2 et plus |
| T5.11 Plancher partiel | chaque individu saisi de l'année | au moins une voie praticable (entraide, servitude, maisonnée, sous-location) ; le test statistique complet (bots, 5 graines) reste en tranche 10 |

## 18. Spécification de la tranche 6 : marché du travail, métiers et parrainage

**Objet** : les cinq grandeurs du marché du travail (6.3), la Q(a) d'emploi des firmes modulée par les traits du décideur (6.20), les métiers (6.15) et le parrainage (6.11). La dotation de travail bouchon de la tranche 4 disparaît ici.

### 18.1 Cinq grandeurs

- Disponibilité (quotidienne, par cohorte, 15.1), volonté (heures offertes au salaire courant, événementielle, fonction du salaire réel cible), demande (postes publiés, décision des firmes), postes pourvus (appariement volontés/demande par cohorte et pratique), heures travaillées (min(poste, temps restant)).
- **L'appariement est événementiel** (phase 4), jamais un marché clearing : aucune convergence garantie, les déséquilibres persistants sont des résultats légitimes. L'entité Emploi porte le créneau (6.22).
- **Transition sans saut** : les heures de la dotation bouchon sont reprises par les postes pourvus au fil des embauches ; l'état agrégé (heures travaillées par cohorte) est continu à la bascule.

### 18.2 Q(a) d'emploi et traits du décideur (6.20)

- Actions d'emploi ajoutées à la table : embaucher, licencier, monter le salaire, baisser le salaire. Coûts et risques réels.
- Empathie du décideur = patience (seuil de licenciement pour sous-productivité passagère ×(1 + empathie/1000)) et tolérance (employabilité des malades) ; contrainte de pérénité dure (en faillite imminente, l'empathie s'efface) ; dissonance du décideur au licenciement d'un employé connu (mémoire saillante). Deux régimes viables, aucun dominant : la firme patiente accumule du savoir en régime stable, la firme au churn s'adapte plus vite en choc de demande.
- **Engagement employé ↔ firme** (6.20) : scalaire 0-1000 par couple, mis à jour par les décisions observées de la firme (× empathie × proximité), demi-vie 1-2 ans, jamais lu par le décideur (décision 57). Effets : pondération de l'arrêt maladie, du soin et de la démission ; prévention (probabilité d'incidents réduite, usure ralentie) ; options de résolution dans les décisions ponctuelles. Ni heures supplémentaires, ni multiplicateur de recette (décision 64).

### 18.3 Métiers et parrainage

- **Décisions ponctuelles** (~1/mois, à clé stable) : incident de machine, raccourci risqué, place disputée, élève à parrainer ; même table pour les PNJ et le joueur, résolue par Q(a).
- **Parrainage** : un employé de pratique ≥ 3 parraine un journalier ; coût en temps régulier, effet : essai accordé plus vite, lien durable dans la mémoire saillante des deux.
- **Recettes multi-postes** (6.15) : production de la firme = min des capacités des étages pourvus, recalculée par événement d'embauche, de départ ou d'incident ; une recette mono-poste est le cas dégénéré à un étage. Le goulot est journalisé (quel étage plafonne, de combien).

### 18.4 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T6.1 Décisions d'emploi expliquées | embauche et licenciement quelconques | chaîne Q(a) complète au journal (entrées, poids, décision) |
| T6.2 Choc 3 | commande de la ville ×2 pendant 60 jours | embauches explicables, postes pourvus montent puis redescendent |
| T6.3 Choc 4 | prix des composants +30 % | chaque firme répond par Q(a) (prix, licenciement, entretien différé), choix explicables |
| T6.4 Parrainage | un parrainé et un isolé à qualifications égales | le parrainé obtient un essai plus vite, chaîne journalisée |
| T6.5 Deux régimes | deux firmes identiques sauf l'empathie du décideur | churn et maîtrise moyenne divergent, aucune ne domine sur les deux régimes |
| T6.6 Transition du bouchon | bascule dotation → marché réel | heures travaillées par cohorte continues, aucun saut d'état, I4 vert |
| T6.7 Présentéisme et contagion | deux firmes identiques sauf l'empathie du décideur | la firme sanctionneuse produit plus de jours travaillés malades, de contagion chez les collègues, de séquelles et de churn volontaire ; la patiente l'inverse ; en choc de demande, la churn garde l'avantage d'adaptation ; l'engagement ne bouge jamais sans événement ; l'engagement ne crée jamais d'heures ni de multiplicateur de recette ; la production par heure travaillée est supérieure dans la firme patiente, traçable à moins d'incidents (usure ralentie) et à plus d'incidents résolus |
| T6.8 Goulot multi-postes | restaurant avec cuisinier seul, puis cuisinier + serveur | la production plafonne au min des capacités d'étage ; l'embauche du serveur débloque les couverts ; le goulot est nommé au journal (étage, amplitude) ; aucun recalcul hors événement |

## 19. Spécification de la tranche 7 : salaires réels et griefs

**Objet** : la boucle économique se referme sur elle-même : panier, p, q, G, concordance. La grève attend la tranche 8 ; ici le seuil de risque calcule ses entrées.

### 19.1 Panier et salaire réel

- Panier = nourriture + logement + eau refacturée + fiscalité, exprimé en net (17.4). Salaire réel = salaire horaire × heures travaillées / coût du panier. p est la composante pression des prix du panier ; u vient de 15.2, q de 15.4 et 16.3 (santé, dette d'entretien du quartier).

### 19.2 Griefs (équation du cadrage, ch. 2 et 14)

- dG = 0,8·u + 0,2·p − 0,25·q points/jour, bornés [0, 100], **intégration événementielle** : G est mis à jour quand une entrée change, jamais par pas fixe ; coefficients par jour, calibrage pluriannuel.
- Chaque individu nommé porte son grief individuel (mêmes coefficients, échantillonnage déterministe) ; les cohortes portent la somme pondérée (règle d'identité, 14.2). **I8 à tolérance zéro.**
- L'entrée griefs du stress (15.5) s'active ici ; le seuil de risque cumulé calcule ses entrées (G, salaire réel, moyens de la cohorte) mais le basculement attend la tranche 8.
- **Choc 11 complété** (décision 36) : l'assertion griefs traçables s'ajoute aux flux fiscaux déjà testés.

### 19.3 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T7.1 Concordance | 10 ans, toutes cohortes | I8 vert à tolérance zéro, y compris après extractions et fusions |
| T7.2 Choc 7 | prix de la nourriture +50 % | salaire réel en baisse, G monte, l'offre de travail de la cohorte baisse ; chaîne traçable |
| T7.3 Choc 11 complet | impôt ×2 pendant un an | griefs traçables terme à terme (u, p, q) ; aucune trappe : taux de sortie de misère insensible aux allocations |
| T7.4 Intégration événementielle | entrée changeant plusieurs fois par jour | G mis à jour à chaque changement, résultat identique à l'intégration continue de référence |
| T7.5 Seuil de risque (entrées) | choc 8 en préparation | le taux de risque cumulé ne franchit le seuil que si le salaire réel est bas ; les dates de franchissement sont planifiées et recalculées |

## 20. Spécification de la tranche 8 : basculements

**Objet** : le seuil de risque cumulé et la grève (6.3, cadrage ch. 2). Le dernier mécanisme du cœur économique.

### 20.1 Seuil de risque cumulé

- Chaque cohorte accumule un taux de risque (fonction de G, du salaire réel, des moyens de la cohorte, coefficients en données). La date de franchissement est **calculée exactement et planifiée**, recalculée quand les entrées changent : le même mécanisme que l'usure paresseuse (16.3). Le tirage a lieu **une seule fois** au franchissement.

### 20.2 Grève

- Au franchissement : tirage unique d'une durée, la cohorte cesse d'offrir du travail pour cette durée ; événements avant (montée du risque), pendant (refus d'offrir, postes vacants), après (reprise) au journal. Les firmes répondent par Q(a) (monter les salaires, attendre, licencier) ; la fin de grève a toujours une cause journalisée (concession, échéance de la durée tirée), jamais une horloge arbitraire.

### 20.3 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T8.1 Déterminisme | même graine, même données | même date de grève, bit à bit (I9) |
| T8.2 Tirage unique | rejeu du franchissement | le tirage consommé est journalisé, jamais re-tiré |
| T8.3 Choc 8 complet | chocs 2 et 7 combinés | grève produite et explicable seulement si le salaire réel est bas ; aucune grève en régime sain |
| T8.4 Fin de grève | toutes les grèves d'un an | chaque fin a une cause journalisée (concession ou durée écoulée) |
| T8.5 Recalcul | entrée changeant avant le franchissement | date replanifiée, aucun tirage anticipé |

## 21. Spécification de la tranche 9 : pratiques, reconnaissance et vie sociale

**Objet** : la couche sociale qui fait du quartier un lieu vivant : maîtrise et apprentissage (6.5), reconnaissance et valuation (6.6, 6.21), empathie et contagion (6.19), groupes informels et rituels (6.18), vie vitale (6.10). C'est la tranche glissable en P1.5 sans casser les tranches 1 à 8.

### 21.1 Maîtrise et apprentissage

- Chaque exécution d'une demi-journée de travail ajoute de l'expérience (**événementielle, jamais un timer**) ; le franchissement du seuil de risque tiré à clé stable monte le niveau (0-10) ; plafond de formation : 1 vétéran pour 3 élèves ; l'élève isolé progresse moins vite. La maîtrise module rendement et salaire (entrée active depuis 16.2).

### 21.2 Reconnaissance et valuation des rôles

- Groupe de référence : cohorte + immeuble ; position = rang du revenu du foyer dans le groupe, borné ; mise à jour événementielle (promotion, embauche, licenciement, acte marquant, affront).
- **Valuation (6.21)** : reconnaissance = f(revenu, utilité visible, conformité aux valeurs), pondérée par des curseurs en données (fixes en P1) ; les salaires cibles suivent la valuation ; les vocations suivent la reconnaissance, corrigées partiellement par la pénurie. L'utilité visible est calculée depuis le journal ; l'écart de valuation est une mesure.

### 21.3 Empathie et contagion (6.19)

- Souffrance perçue = Σ (empathie × affinité × souffrance visible) sur l'ensemble de conscience (mémoire saillante, plafond 20 ; affinités : foyer 1,0, groupe 0,5, immeuble 0,3, connaissance 0,1). Elle alimente le stress (collecteur unique), puis remonte aux griefs de cohorte **par les groupes** : aucun second registre de souffrance.

### 21.4 Groupes informels, rituels, vivacité (6.18)

- Groupes de 3 à 6 individus nommés, lieu habituel, affinités ; rituels sur les événements de vie (phase 6) ; répliques générées depuis le journal ; mémoire active qui biaise Q(a) ; habitudes persistantes et visibles. Tout est traçable : chaque réplique référence les événements qui la motivent.

### 21.5 Vie vitale (6.10)

- Vieillissement par anniversaire (événement annuel par cohorte) ; décès liés à l'âge et à la santé, par tirage à clé stable ; **naissances agrégées** : les enfants sont des effectifs de cohorte, les individus nommés ne sont créés qu'à l'extraction, vers 14 ans ; héritage selon une règle en données (conjoint, descendants, créanciers), chaque héritage journalisé. Naissance et décès conservent I1 (transferts déclarés).

### 21.6 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T9.1 Apprentissage | exécutions répétées | montée de niveau par franchissement de seuil tiré, journalisée ; aucun timer |
| T9.2 Élève isolé | un élève sans vétéran et un élève parrainé | le premier progresse moins vite, écart conforme aux coefficients |
| T9.3 Deuil (scénario 13) | décès dans un groupe informel | stress des proches monte par empathie et affinité, l'offre de travail du groupe baisse ce jour-là, chaîne traçable |
| T9.4 Symétrie d'empathie | deux individus identiques sauf empathie | stress divergent après un deuil voisin, conforme aux coefficients |
| T9.5 Valuation | rôles à utilités contrastées | écart de valuation mesuré ; les vocations suivent la reconnaissance, les salaires corrigent par pénurie |
| T9.6 Vie vitale | 10 ans de démographie | I1 vert à chaque naissance et décès ; héritages journalisés et causalement liés |
| T9.7 Vivacité | répliques d'un an | chaque réplique traçable aux événements qui la motivent ; aucune réplique décorative |

## 22. Spécification de la tranche 10 : suite headless et bots

**Objet** : aucune mécanique nouvelle. La tranche joue la suite complète (section 9) et produit le rapport de mesures ; tout échec remonte à la tranche fautive, jamais corrigé ici.

### 22.1 Contenu

- Les **13 scénarios** de la section 9 (équilibre + 12 chocs), chacun explicable de bout en bout par le journal.
- Les **5 bots** (journalier, ambitieux, économe, dépendant, gréviste), plusieurs centaines d'instances, **5 graines**, 10 ans de simulation ; **benchmark** à 300 ans.

### 22.2 Critères transversaux

- Aucun bot ne domine systématiquement, aucun n'est impossible ; taux de sortie de misère du joueur = celui des PNJ à investissement égal ; **plancher anti-impasse** : un bot après faillite sort de la misère à 5 ans sur toutes les graines (6.24) ; le choc 2 est amorti avec retard et signifiant (6.2) ; les chocs 11 et 12 ne créent aucune trappe.

### 22.3 Observabilité (conventions 26 et 27)

- Séries synthétiques à pas fixe (quotidien pour la bulle chaude, décennal pour la tendance) ; tableau de bord avec diff entre deux dates ; reprise depuis snapshot ; requête « explique » terme à terme. Le rapport de mesures (liste de la section 9) est relu à la main avec la fiche de revue (convention 24).

### 22.4 Tests de fermeture

| Test | Montage | Critère |
| --- | --- | --- |
| T10.1 Invariants | toutes les graines, tous les scénarios | I1 à I9 verts en continu, échec = arrêt du replai |
| T10.2 Rapport | 5 graines, 10 ans | rapport de mesures complet, revu à la main |
| T10.3 Plancher | bots après faillite, 5 graines | taux de sortie de misère strictement positif à 5 ans, partout |
| T10.4 Benchmark | 300 ans simulés, charge réelle | latence et taille de file sous les seuils fixés en 13.6 et 13.7 ; pas de dérive de mémoire |

## 23. Spécification de la tranche 11 : P1b, écrans minimaux

**Objet** : les écrans du premier jouable (6.7). **Aucun mécanisme nouveau** : les écrans lisent l'état et le journal, et éditent des données déjà existantes (doctrine, planification). Le joueur est un individu standard de la cohorte des journaliers, sans pouvoir spécial.

### 23.1 Écrans

- **Doctrine personnelle** : poids entre besoins, travail, formation, reconnaissance ; modèles prédéfinis, pas de script libre ; éditée sur la console portable.
- **Planification basique** : accepter un poste, choisir une pratique, abonnement d'achat ; décisions fines à pause active, jamais requises.
- **Vue du quartier** : foyers et individus nommés, états qualitatifs (jamais de jauge numérique de grief), journal causal consultable avec dates d'apprentissage.
- **Rendu cosmétique des flux de métier** : les trajets et gestes du travail (aller chercher, cuisiner, servir) sont une **peau de présentation** qui lit l'état agrégé, jamais un simulé : présence crénière aux postes, débit de la firme (couverts/demi-journée, goulot de la décision 66) pilote l'animation de flux ; un poste vacant se rend par l'absence, jamais par un sprite animé. L'animation ne lit que l'état, ne l'écrit jamais et ne le contredit pas (même séparation que les répliques : rendu hors état). Aucune simulation par objet ni par trajet.
- **Alertes** : classées par valeur d'intervention (cadrage ch. 1) ; le monde continue pendant la consultation, avec pause active possible.

### 23.2 Critères de fermeture (humains, section 12)

- Un humain joue 30 minutes et comprend pourquoi il s'en sort ou non ; aucune attente passive non signalée ; le classement par valeur d'intervention suffit à tenir le joueur informé sans surveillance continue ; latence maximale de quelques secondes par jour de jeu respectée ; revue de palier avec la fiche de revue (convention 24).

## 24. Répliques, intentions de parole et colportage (P1.5)

**Objet** : comment les PNJ parlent. Principe directeur : une réplique est le **rendu d'une décision**, jamais un ornement ; parler est une action comme les autres, évaluée par Q(a), avec un coût (temps) et des effets (stress, information, relations). Toute la section entre en P1.5 avec la vie sociale (tranche 9 glissée) ; le cœur P1 n'en dépend pas.

### 24.1 Répliques : sélection et rendu séparés

- **La sélection est de la simulation** : choisir de parler, à qui, et quoi exprimer, est une décision Q(a) journalisée (phase 6). **Le texte est du rendu** : hors état canonique, produit à la demande (LOD du texte : on ne rend que ce qui est observé, près du joueur ou dans le journal consulté). Aucune chaîne de caractères dans l'état ; les noms et les textes sont dérivés de façon déterministe.
- **Templates connectés à l'état** : squelettes de phrases à slots, filtrés par conditions strictes (états cachés, événements de la mémoire saillante, contexte). Corpus par intention, borné (10 à 20 templates par intention au départ, à calibrer) ; le silence est une réponse valide : pas de réplique sans événement causal à exprimer.
- **Localisation (Fluent, messages entiers)** : un message Fluent par template, slots référencés par variables, sélecteurs pour genre et nombre ; les attributs grammaticaux (genre, nombre) des entités sont en données. **L'unité de traduction est la phrase entière**, pas la brique sémantique : l'ordre des mots, les elisions et les idiomes ne se recomposent pas d'une langue à l'autre. La locale ne touche jamais la simulation : même graine → mêmes sélections, quelle que soit la langue.
- Un LLM peut servir **d'outil d'écriture du corpus** en développement (validé à la main, figé en données versionnées) ; jamais en exécution.

### 24.2 Intentions de parole

Parler est une **action Q(a) à intention** ; les intentions sont portées par des variables existantes, aucun moteur d'intentions parallèle :

| Intention | Variable portante | Effet | Coût |
| --- | --- | --- | --- |
| Se plaindre (catharsis) | stress (collecteur unique) | stress du locuteur en baisse ; stress de l'auditeur en hausse via empathie × affinité (contagion, 6.19) | temps |
| Partager du positif | reconnaissance, affinité, mémoire saillante | lien renforcé, occasion sociale, rituel possible | temps |
| Inciter (agitation) | taux de risque cumulé de la cohorte | le stress transmis remonte aux griefs par les groupes ; le taux de risque des auditeurs monte plus vite | temps + risque (repérage par le décideur) |
| Demander | temps_recherche (6.1), ardoise (6.14), parrainage (6.11) | surface de l'action existante, pas une action nouvelle | déjà compté |
| Entretenir une relation | affinité + mémoire saillante | rafraîchit la mémoire de l'autre ; sans contact, la relation s'estompe en sortant de la mémoire, sans décroissance artificielle | temps |
| Transmettre une connaissance | pratiques, colportage (6.5, ch. 16) | fait **savoir que** (existence d'une pratique, d'un poste, d'un prix) ; le savoir-faire reste vétéran → élève au travail | temps |

- **Catharsis dégressive** : parler du même sujet au même auditeur a un rendement dégressif (×0,5 par répétition, à calibrer) ; catharsis de départ −5 à −15 points de stress par échange. Sans cela, la boucle parole → contagion → stress → parole produit des épidémies de dépression sans cause économique.
- **G reste fermé** : l'équation de griefs ne dépend que de u, p, q ; la contagion passe par le stress (6.19), jamais par un ajout direct à G.
- **L'agitation amplifie, elle ne crée pas** : un agitateur accélère la chaîne existante (parole → stress → griefs → seuil de risque) ; le test T8.3 reste le garde-fou : aucune grève en régime sain, même avec agitateurs. La présence d'agitateurs est le contenu concret des « moyens de la cohorte ». La démagogie organisée (agitateurs payés) est une route P2 (factions).

### 24.3 Contextualisation : trois filtres en données

On ne parle pas pareil à tout le monde ni partout. Trois dimensions filtrent les intentions disponibles et modulent effets et risques, **sans multiplier les corpus** :

1. **La relation** : chaque intention exige un niveau minimal de l'échelle symétrique (7 niveaux, ch. 14). Parler au-dessus de son niveau coûte de la reconnaissance ou crée un affront.
2. **Le lieu** : chaque type de lieu déclare qui entend (témoins) et qui contrôle. À l'atelier, le décideur entend ; à l'estaminet, les témoins propagent plus loin mais personne ne contrôle ; au foyer, intime mais audience réduite. Les témoins reçoivent par le même mécanisme que l'auditeur direct, affinité moindre.
3. **Le créneau** (6.22) : les conversations occupent le temps libre ; parler pendant le travail coûte en productivité et en risque ; le soir au lieu habituel est le créneau naturel.

**Conséquences émergentes, pas de règles spéciales** : aucune règle de médisance ou de dénonciation ; le décideur apprend ce qui lui parvient par la propagation normale, et sa Q(a) réagit. Une plainte qui remonte, un agitateur dénoncé : tout est émergent et traçable.

### 24.4 Le repas comme scène sociale et politique

- Qui mange où et quand détermine l'audience : le repas est la principale occasion de conversation du quartier (6.14, colonne sociale ; pauses et repas du midi pour les salariés).
- **L'évitement est une stratégie** : un salarié mécontent évite la table du patron (parler de ses griefs à portée du décideur est risqué) ; il choisit la cantine entre collègues ou le créneau du soir. L'évitement aveugle le décideur, ce qui est un fait politique, pas un bug.
- **Un décideur ne voit que ce qui lui parvient** (principe acté au cadrage v6.8) : la Q(a) d'un décideur nommé raisonne sur son **information perçue** (ce qu'il a appris, dates d'apprentissage au journal), jamais sur l'état réel. Application progressive : information sociale en P1.5, rapports falsifiables en P2. Mesure : l'écart entre le G réel de la cohorte et le G perçu du décideur.

### 24.5 Inquiétude d'autrui et entraide

- **Le seuil d'intimité est un seuil d'information** : on ne s'inquiète que des besoins qu'on perçoit. Les états cachés d'autrui ne sont visibles qu'aux niveaux « relations » et « intime » de l'échelle, plus les indices visibles (maigreur, fatigue sur le sprite, ch. 13) et les témoignages. L'intimité est la permission de voir.
- **Aider est une façon de faire baisser son propre stress** : la souffrance perçue alimente le stress (6.19) ; agir pour la réduire est une action Q(a) comme les autres, pas un altruisme en boîte noire. Aider = des transferts existants (nourriture, argent, temps de garde) ; aucune nouvelle entité ; effet sur la reconnaissance (acte marquant) et l'affinité.
- **Le garde-fou est la doctrine** : aider entre en concurrence avec ses propres besoins dans Q(a) ; un individu dont u est élevé ne donne pas ; le « bot saint » qui se ruine est exclu par les poids, pas par une règle.
- **Arc de long terme** : l'entraide individuelle (P1.5) donne au plancher anti-impasse (6.24) son moteur causal (« un proche solvable existe » devient un comportement) ; elle est le matériau que les institutions de solidarité formaliseront en P2 (route 12), avec la sélection adverse qui va avec.

### 24.6 Coût

- Sélection : O(templates filtrés par intention), petit ; journalisation d'un événement de parole par échange. Rendu : à la demande seulement. Aucun coût dans la boucle chaude du noyau ; le corpus est en données versionnées.

### 24.7 Tests de fermeture (P1.5)

| Test | Montage | Critère |
| --- | --- | --- |
| T24.1 Déterminisme inter-langues | même graine, deux locales | mêmes événements de sélection bit à bit ; seul le rendu diffère |
| T24.2 Traçabilité | toutes les répliques d'un an | chaque réplique référence son intention (poids Q(a)) et l'événement exprimé ; zéro réplique décorative |
| T24.3 Répétition | un an de conversations | fréquence d'un même template sous le plafond en données |
| T24.4 Couverture | matrice intention × relation × lieu × créneau | toute intention indisponible dans un contexte n'est jamais sélectionnée ; corpus complet pour les contextes autorisés |
| T24.5 Catharsis dégressive | répétitions du même sujet au même auditeur | effet décroissant conforme ; pas d'épidémie de stress sans cause économique |
| T24.6 Agitateur inoffensif | agitateur actif, quartier sain | aucune grève (T8.3 respecté) |
| T24.7 Agitateur en choc | même agitateur + choc 7 | grève plus précoce qu'avec le choc seul ; chaîne parole → stress → griefs → seuil traçable |
| T24.8 Évitement du patron | salariés mécontents, patron présent à la cantine | les mécontents choisissent d'autres tables ou créneaux ; le patron perçoit les griefs avec retard ; écart G réel/perçu mesuré |
| T24.9 Entraide | voisin affamé, individu empathique intime | don si surplus, pas de don si u propre élevé ; acte marquant journalisé ; survie du receveur améliorée |
| T24.10 Plancher causalement expliqué | bots après faillite | le taux de sortie de misère positif (6.24) s'explique par des actes d'entraide journalisés |

## 25. Calibrage consolidé (paramètres introduits depuis la section 8)

Complément de la section 8 ; chaque ligne porte la valeur de départ et le test qui la valide. Toute valeur est une hypothèse : la modifier est un calibrage, jamais une règle.

| Paramètre | Valeur de départ | Validé par |
| --- | --- | --- |
| Seuils des états cachés (0-1000) | élevé ≥ 400, critique ≥ 700 (qualitatif journalisé) | T3.5, T3.6 |
| Dépression | stress ≥ 700 pendant 30 jours consécutifs ; sortie après 30 jours sous 500 (hystérésis) | T3.7 |
| Angoisse | épargne < 2 semaines de panier et tendance de u haussière | T3.5 |
| Jeûne | −1 demi-heure d'éveil par 1,5 jour sans nourriture | T3.3 |
| Corvées d'enfants | 0,5 demi-heure/jour à 8 ans, 1,5 à 12, plein régime à 14 ; plafond 2 | T3.8 |
| Ville : pente d'élasticité | +1 % de prix par 2 % de demande excédentaire, plafond +50 % | T5.5 |
| Ville : délais logistiques | 3 à 10 jours selon le bien | T5.5 |
| Eau | 5-10 % du panier, jusqu'à 25 % en pénurie | T5.8 |
| Sous-location post-saisie | loyer −30 à −50 %, densité +1 | T5.9 |
| Prime de nuit | +20 à +40 % du salaire du créneau ; pénalité +1 demi-heure de fatigue | T6.5, P1.5 |
| Catharsis | −5 à −15 points de stress par échange ; dégressivité ×0,5 par répétition | T24.5 |
| Contagion | formule 6.19 ; affinités foyer 1,0, groupe 0,5, immeuble 0,3, connaissance 0,1 | T9.3, T9.4 |
| Corpus de répliques | 10 à 20 templates par intention, variantes de ton par contexte | T24.3, T24.4 |
| Individus nommés | plafond 30 simultanés ; extraction vers 14 ans | T2.1, T9.6 |
| Groupes informels | 3 à 6 membres ; mémoire saillante plafond 20 | T9.3 |
| Décisions ponctuelles de métier | ~1/mois par individu, à clé stable | T6.1 |
| Grève | taux de risque = f(G, salaire réel, moyens) ; durée tirée une fois | T8.1 à T8.5 |
| Benchmark 300 ans | hypothèse : simulation complète en moins d'une minute headless, file < 10⁵ événements | T10.4 |
| Mortalité par faim | incidence < 1 % de la population par an en régime non-choc | T10.2 |

## 26. Rituels (catalogue en données, P1.5)

Un rituel est déclenché par un événement de vie, jamais planifié décorativement. Chaque type est en données : déclencheur, participants (foyer, groupe, quartier), durée, coût (monétaire, temporel), effets (stress, reconnaissance, mémoire saillante, affinité).

| Rituel | Déclencheur | Participants | Effets principaux |
| --- | --- | --- | --- |
| Funérailles | décès | groupe, immeuble | deuil (stress des proches, contagion encadrée), reconnaissance du défunt, mémoire |
| Nommage | naissance | foyer | reconnaissance des parents, lien foyer |
| Première embauche | embauche d'un jeune | foyer, parrain | reconnaissance, vocation |
| Entrée dans un groupe | admission | groupe | affinité, appartenance |
| Départ à la retraite | fin de rôle | cohorte, apprentis | transmission, reconnaissance |
| Fête de quartier | périodique (saisonnière, en données) | tout le quartier | stress en baisse global, occasions sociales, attractivité |

- La fête de quartier est le vecteur du **pain-et-jeux** en P2 (décision 60) : financer une fête achète de la légitimité, à coût réel et rendement décroissant.
- Test : chaque rituel est traçable à son déclencheur ; les effets sont mesurés (stress des proches après funérailles, retour des occasions sociales après fête) ; aucun rituel sans effet.

## 27. Écran de conversation (P1b)

- Le joueur voit l'intention qualitative de l'interlocuteur, l'historique de la conversation, et choisit parmi **les intentions disponibles**, filtrées par le contexte (24.3 : relation, lieu, créneau). La conversation consomme du temps réel de jeu.
- **Pas de texte libre, pas de minigame** : le déterminisme, la traduction et la symétrie joueur/PNJ l'interdisent. Le joueur choisit une intention et une cible ; le système rend la réplique comme pour un PNJ.
- **L'écoute passive** : ce qui se dit à portée parvient au joueur avec sa propre date d'apprentissage ; être au bon endroit au bon créneau est une source d'information, pas une corvée.
- Test : un humain tient une conversation utile en moins de dix secondes d'interface ; aucune option non signalée ; les intentions indisponibles sont grisées avec leur raison.

## 28. Directives de firme (tranche 11)

Le joueur-patron édite des directives ; l'autopilote les exécute ; ce sont les mêmes tables que les firmes PNJ (aucune gestion en temps réel).

| Directive | Paramètres | Défaut |
| --- | --- | --- |
| Embauche | seuil de charge, maîtrise visée | embaucher si charge > 80 % pendant 5 jours |
| Salaire | suivre le marché, retenir, serrer | suivre le marché |
| Prix (reprice) | conditionnel : seuils de stock, concurrence | heuristique par défaut (6.2) |
| Stocks | seuil bas, seuil haut | 25 % / 75 % du stock de référence |
| Crédit client (ardoise) | plafond, durée | plafond 1 semaine de panier |
| Entretien | minimale, normale, renforcée | normale |
| Investissement | seuil de trésorerie | hors de portée en P1b (l'atelier reste aux PNJ) |

Test : un joueur-patron qui ne touche à rien obtient les mêmes trajectoires qu'une firme PNJ identique ; chaque directive modifiée change une trajectoire explicable.

## 29. Ordre de mission : tranche 1 (noyau temporel)

Document de travail pour l'agent de code. Source de vérité : sections 13 (spécification), 5 (phases), 7 (invariants), conventions 1 à 27. L'agent n'ajoute rien hors périmètre (convention 25) et ne prend aucune décision de design : toute ambiguïté remonte.

### 29.1 Objectif et interdits

- **Livrer** : l'horloge, la file d'événements, l'ordre canonique, le report, le snapshot/reprise, le harnais de tests T1.1 à T1.7.
- **Interdits** : toute sémantique métier (aucun individu, firme, bien) ; toute horloge réelle (l'horloge est logique) ; tout flottant dans l'état ; toute itération de table de hachage dans l'état canonique ; toute exécution réentrante ; tout parallélisme (P1 séquentiel) ; tout rendu.

### 29.2 Types (pseudo-code)

```
struct CléÉvénement { jour: u64, phase: u8, priorité: i32, id: u64 }   // ordre lexicographique total
struct Événement { clé: CléÉvénement, type: IdType, payload: Payload } // types et priorités en données

struct ÉtatNoyau {
  horloge: u64,                    // jour courant
  compteur_id: u64,                // monotone, jamais réutilisé
  rng: [ÉtatRng; NB_FLUX],         // un flux déterministe par usage déclaré
  file: TasMin<Événement>,         // ordonné par CléÉvénement
  registres: RegistresSystèmes,    // vide en tranche 1
}
```

La clé est **totale** : l'id unique garantit un ordre sans ex æquo ; le tas n'a jamais besoin de critère de départage supplémentaire.

### 29.3 API publique

```
fn avance_journée(&mut self, systèmes: &mut Systèmes) -> JournalDelta
    // exécute la boucle de 13.5 pour le jour courant ; retourne les entrées de journal produites

fn générer(&mut self, type: IdType, payload: Payload,
           date: Option<(u64, u8)>, instant_courant: (u64, u8)) -> Résultat<(), Rejet>
    // date absente → prochain_instant(instant_courant) ;
    // date présente ≤ instant_courant → Rejet (invariant, arrêt du replai) ;
    // alloue id = compteur_id++, insère dans le tas

fn prochain_instant((j, p)) -> (u64, u8)   // (j, p+1) si p < 7 sinon (j+1, 1)

fn snapshot(&self) -> Bytes                // sérialisation canonique + versions
fn restaurer(bytes: Bytes) -> ÉtatNoyau    // aucun recalcul
```

Les systèmes métier reçoivent l'événement, le traitent, appellent `générer` ; ils ne voient jamais le tas, ne datent rien eux-mêmes et ne peuvent pas extraire.

### 29.4 Sérialisation canonique

- Champs dans un ordre fixé par le code, entiers à boutisme fixé, file émise **dans l'ordre de la clé** (pas l'ordre mémoire du tas) ; en-tête = version du format + version des données + graine.
- Deux états logiquement égaux → octets identiques, quelle que soit leur histoire de construction (T1.6).

### 29.5 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Contenu | Tests de fermeture |
| --- | --- | --- |
| 1 | Types, clé, tas min, insertion/extraction | T1.1 (ordre canonique sur 10 000 événements) |
| 2 | Boucle `avance_journée`, report, rejet de date passée | T1.3 (auto-régénération), T1.4 (rejet) |
| 3 | Flux RNG déterministes, allocation des ids | T1.2 (permutation d'insertion → journal identique) |
| 4 | Snapshot, restauration, sérialisation canonique | T1.5 (reprise bit à bit), T1.6 (octets identiques) |
| 5 | Harnais headless, générateurs de charge, mesures | T1.7 (benchmark 10⁶ événements, charge 300 ans) |
| 6 | Replai de référence (lecture seule pour l'agent) | journal de référence archivé, comparé à chaque exécution |

### 29.6 Pièges déclarés (à vérifier en revue de code)

- Itération de table de hachage : interdite dans l'état et la sérialisation ; si un index est nécessaire, ordre canonique déclaré.
- Source de temps réelle (`Date::now`, etc.) : interdite partout ; le temps est l'horloge logique.
- Réentrance : `exécuter` ne peut ni extraire ni insérer directement ; seul `générer` insère, et il reporte.
- Compteur d'id : jamais réinitialisé, jamais dérivé d'une taille de collection ; il vit dans l'état sérialisé.
- Tas : la restauration doit reconstruire le tas depuis la sérialisation ordonnée, jamais depuis un ordre arbitraire.
- Arrondis : aucun dans le noyau (entiers purs) ; toute future arithmétique suit la convention 26 (implémentation canonique unique, arrondi déclaré).

### 29.7 Critères d'acceptation

- T1.1 à T1.7 verts en continu (convention 26 : assertions d'invariants actives en debug, échec = arrêt du replai).
- I9 vert : même graine et mêmes données → même journal, bit à bit, y compris après reprise snapshot.
- Le replai de référence passe en lecture seule ; toute divergence exige une validation humaine.
- Rapport de benchmark produit (temps, taille de file) et archivé avec la tranche.
- Aucune ligne de sémantique métier : la revue vérifie que le noyau est réutilisable tel quel pour les tranches 2 et suivantes.

## 30. Ordre de mission : tranche 2 (identités et journal)

Document de travail pour l'agent de code. Source de vérité : section 14 (spécification), 7 (invariants I1, I7, I8), conventions 1 à 27. Mêmes interdits que la tranche 1 ; l'agent ne décide rien.

### 30.1 Objectif et interdits

- **Livrer** : le régime d'identité (compteurs par type, archivage), les cohortes à totaux avec extraction/réinjection et fusion, le journal causal à liens explicites, la requête « explique », le compactage.
- **Interdits** : toute sémantique économique (aucune transaction, aucun marché) ; tout graphe de relations complet (hors périmètre P1, 6.11) ; tout nom stocké dans l'état ; toute suppression d'entité (archivage seulement) ; toute écriture dans le journal hors `journaliser`.

### 30.2 Types (pseudo-code)

```
struct IdEntité { type: IdType, numéro: u64 }        // compteur par type, dans l'état

struct Cohorte {
  id: IdEntité,
  effectif: u64,
  monnaie_totale: u64,            // totaux, jamais de moyenne stockée
  somme_griefs: i64,              // somme des griefs des membres
  attachement_total: i64,
  // + motivations, heures disponibles, salaire réel cible (section 4)
}
// G affiché = somme_griefs / effectif, arrondi déclaré (convention 26)

struct IndividuNommé {
  id: IdEntité, cohorte_origine: IdEntité,
  grief: i64, solde: u64, attachement: i64,
  // + champs de la section 4
}

struct EntréeJournal {
  id_événement: u64,              // id du noyau (29.2)
  jour: u64, phase: u8,
  acteur: Option<IdEntité>,
  type_décision: IdType,
  valeurs_modifiées: [Modification],
  causes: [u64],                  // ids d'événements strictement antérieurs
  appris_par: [(acteur: IdEntité, date: u64)],   // dates d'apprentissage
}

struct Résumé { ids_compactés: [u64], synthèse: Données }  // compactage, hors état canonique
```

### 30.3 API publique

```
fn extraire(&mut self, cohorte: IdEntité, échantillon: CléStable) -> IndividuNommé
    // crée l'individu nommé, déduit exactement (effectif−1, monnaie−solde, somme_griefs−grief, attachement−attachement)

fn réinjecter(&mut self, individu: IndividuNommé, cohorte: IdEntité)
    // addition réciproque ; l'individu est archivé après réinjection

fn fusionner(&mut self, a: IdEntité, b: IdEntité) -> IdEntité
    // somme des totaux, a absorbé b (archivé) ; jamais de moyenne additionnée

fn journaliser(&mut self, entrée: EntréeJournal) -> u64
    // append-only ; vérifie en debug que causes ⊂ événements strictement antérieurs (I7)

fn expliquer(&self, id_événement: u64) -> ChaîneCausale
    // remonte les liens jusqu'aux racines, résout les résumés ; sans cycle par construction

fn compacter(&mut self, avant_jour: u64) -> [Résumé]
    // résume les entrées vieilles ; les liens restent résolubles via les résumés ; ne touche jamais l'état canonique
```

Les noms (PNJ, firmes) sont dérivés de (graine, id) au rendu, jamais stockés ni comparés.

### 30.4 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Contenu | Tests de fermeture |
| --- | --- | --- |
| 1 | Registre d'entités, compteurs par type, archivage | T2.5 (100 000 événements, aucun id réutilisé) |
| 2 | Cohortes à totaux, extraction/réinjection | T2.1 (100 cycles sans dérive), T2.2 (isolation) |
| 3 | Fusion conservatrice | T2.6 (totaux conservés, I8 vert) |
| 4 | Journal causal (entrées, liens, dates d'apprentissage) | T2.3 (validité I7 sur 1 an) |
| 5 | Requête « explique » | T2.4 (chaîne complète, sans trou) |
| 6 | Compactage | T2.4 (résumés résolubles) |
| 7 | Snapshot avec extraits, replai de référence étendu | T2.7 (reprise bit à bit) |

### 30.5 Pièges déclarés (à vérifier en revue de code)

- **Moyennes** : jamais stockées, jamais additionnées ; seule la moyenne dérivée est affichée, avec arrondi déclaré (convention 26). Additionner deux moyennes est l'erreur classique de la fusion.
- **Conservation exacte** : l'extraction déduit des entiers ; si une valeur de l'individu n'est pas déductible de la cohorte, c'est un défaut de spécification, pas un arrondi à masquer.
- **Ids** : les compteurs vivent dans l'état sérialisé ; l'archivage ne libère aucun numéro ; aucune dérivation depuis une taille de collection.
- **Liens causaux** : strictement antérieurs, vérifiés en debug ; un lien vers le présent ou le futur est un rejet, pas un avertissement.
- **Journal** : append-only ; le compactage produit des résumés hors état, il ne réécrit ni ne supprime l'historique brut servi au rejeu.
- **Noms** : toute comparaison ou tri d'entités se fait par id, jamais par nom.
- **Isolation** : l'extraction ne touche que la cohorte d'origine et l'individu créé ; le diff d'état est un critère de test, pas une impression.

### 30.6 Critères d'acceptation

- T2.1 à T2.7 verts en continu ; I1, I7, I8, I9 verts, y compris après compactage et reprise snapshot.
- Le replai de référence de la tranche 1 reste bit à bit ; le nouveau replai (avec entités) est archivé en lecture seule.
- La requête « explique » produit une chaîne lisible pour tout événement d'un an de simulation de charge.
- Aucune sémantique économique : la revue vérifie que les tranches 3 et suivantes peuvent consommer identités et journal tels quels.

## 31. Ordre de mission : tranche 3 (besoins, temps, santé et états)

Document de travail pour l'agent de code. Source de vérité : section 15 (spécification), 6.1, 6.9, 6.12, 6.19, 6.23, invariants I4, calibrage des sections 8 et 25. Mêmes interdits généraux ; l'agent ne décide rien.

### 31.1 Objectif et interdits

- **Livrer** : la résolution quotidienne du budget de temps (nommés et agrégé par cohorte), le foyer et la consommation, les enfants contributeurs, la santé-trajectoire (maladie, soins, séquelles), les cinq états déclenchés avec préemption par urgence, l'angoisse et la dépression, l'autopilote des besoins (doctrine).
- **Interdits** : marchés, prix et firmes réelles (bouchons uniquement, décision 28) ; toute mise à jour de G (l'entrée griefs du stress reste à zéro, décision 29 ; la précarité attend la tranche 5) ; toute jauge numérique exposée ; tout timer d'apprentissage ou de rémission.

### 31.2 Types (pseudo-code)

```
struct EmploiDuTemps { usages: [Usage; 32] }        // une valeur par demi-heure, I4 par construction
enum Usage { Sommeil, Repas, Trajet, Corvée, Travail, Soin, Recherche, Loisir, Urgence, Libre }

struct Foyer { id: IdEntité, membres: [IdEntité], stock: u32, stock_basse_qualité: u32 }

struct ÉtatsCachés { faim: u16, fatigue: u16, stress: u16, ennui: u16, douleur: u16 }  // 0-1000

struct SantéTrajectoire {
  capacité: u16,                    // réduite par les séquelles, jamais sous 0
  séquelles: [Séquelle],            // (origine: IdÉvénement, points: u16) — permanent
  maladie_active: Option<Maladie>,  // (gravité, jour_début, soignée: bool)
}

struct Doctrine { poids: PoidsBesoins }   // en données, modèles prédéfinis, éditable en P1b
```

**Bouchons** (types dédiés, remplaçables sans toucher aux appelants) : `SourcePrixFixe` (approvisionnement du foyer, prix en données) et `DispensaireCapacitéFixe` (file de soins, capacité en données).

### 31.3 API publique

```
fn résoudre_jour(individu, doctrine, besoin_courant) -> EmploiDuTemps
    // allocation exacte des 32 demi-heures : besoins d'abord, puis travail, puis formation/loisir
    // version agrégée par cohorte : mêmes équations, heures moyennes, aucun individu caché simulé

fn consommer(foyer: &mut Foyer, jour: u64) -> u_foyer
    // consommation quotidienne par membre, deux qualités ; u du foyer → individus → cohorte (moyenne pondérée exacte)

fn accumuler_états(individu, entrées_du_jour) -> [ÉvénementÉtat]
    // faim ← u ; fatigue ← budget d'éveil (majoration nocturne) ; ennui ← déficit de loisir ;
    // douleur ← santé ; stress ← collecteur (griefs = 0 en tranche 3) ; franchissements de seuil journalisés

fn préempter(emploi_du_temps, urgence) -> ()
    // annule les usages restants du jour et les replanifie par générer() du noyau (décision 30) ;
    // ne manipule jamais la file directement

fn tirer_maladie(individu, jour, facteurs_risque) -> Option<Maladie>
    // tirage à clé stable (graine, id, jour) ; facteurs : densité, qualité de nourriture, usure

fn soigner(dispensaire: &mut DispensaireCapacitéFixe, individu) -> RésultatSoin
    // file à capacité fixe ; coûte temps + argent (bouchon) ; non-soin → maladie allongée, risque de séquelle
```

La dépression est un état de l'axe stress : entrée après 30 jours ≥ 700, sortie après 30 jours < 500, chaque transition causée et journalisée (calibrage, section 25).

### 31.4 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Contenu | Tests de fermeture |
| --- | --- | --- |
| 1 | Budget de temps, résolution quotidienne, agrégé cohorte | T3.1 (I4 quotidien, 150 individus) |
| 2 | Foyer, consommation, u, concordance | T3.9 (tolérance zéro) |
| 3 | Bouchons source et dispensaire | T3.2 (richesse libère du temps, via coûts en données) |
| 4 | Enfants contributeurs | T3.8 (amortissement, mono-parental) |
| 5 | Santé-trajectoire : maladie, file de soins, séquelles | T3.4 (blessure → séquelle traçable) |
| 6 | États déclenchés, urgence préemptive | T3.5 (bot sans doctrine), T3.6 (interruption propre) |
| 7 | Angoisse, dépression | T3.7 (rare, réversible par causes) |
| 8 | Replai étendu (entités + états) | I9 bit à bit, replais 1 et 2 inchangés |

### 31.5 Pièges déclarés (à vérifier en revue de code)

- **I4 par construction** : l'emploi du temps est une allocation exacte des 32 demi-heures en entiers ; toute construction par soustractions successives qui peut perdre ou doubler une demi-heure est un défaut. L'égalité est stricte, jamais à tolérance.
- **L'urgence ne touche pas à la file** : elle annule et replanifie par `générer()` ; insérer ou supprimer directement dans le tas est une violation de la décision 30.
- **Les états sont des dérivés** : aucune variable qui s'auto-alimente ; l'entrée griefs reste à zéro, la précarité n'existe pas encore ; les ajouter « en avance » est un hors-périmètre.
- **Hystérésis de la dépression** : entrée et sortie par causes journalisées et durées en données ; aucun timer magique, aucune réinitialisation.
- **Tirages** : tous à clé stable (graine, id, jour), jamais depuis un RNG global consommé hors flux déclarés ; l'ordre des tirages ne doit dépendre d'aucun ordre d'itération.
- **Agrégé** : les non-nommés ne sont jamais instanciés ; la concordance de u (T3.9) est à tolérance zéro ; l'individu nommé extrait suit ses propres valeurs, la cohorte les absorbe à la réinjection.
- **Bouchons** : types dédiés derrière une interface ; les appeler autrement qu'en données (conditions codées) les rend non remplaçables en tranche 5.
- **Garde-fou de calibrage** : temps de survie ≤ 12 demi-heures au niveau de vie de référence est un critère de test, jamais une règle du moteur.

### 31.6 Critères d'acceptation

- T3.1 à T3.9 verts en continu ; I4 vert chaque jour pour chaque nommé et cohorte ; I9 vert, replais 1 et 2 inchangés.
- Un bot sans doctrine survit un an par l'autopilote ; un état critique préempte proprement et le lendemain reprend normalement.
- Aucune jauge numérique exposée dans les données de sortie : seuls des états qualitatifs journalisés.
- Les bouchons sont remplaçables par les tranches 4 et 5 sans modification d'une ligne des appelants : la revue le vérifie sur l'interface.
- Aucune sémantique de marché, de firme ou de griefs : la revue vérifie le périmètre.

## 32. Ordre de mission : tranche 4 (firmes et production)

Document de travail pour l'agent de code. Source de vérité : section 16 (spécification), 6.4, 6.15, invariants I2 et I6, calibrage des sections 8 et 25. L'agent ne décide rien.

### 32.1 Objectif et interdits

- **Livrer** : la firme comme unité de production (recettes, stocks), les actifs avec usure paresseuse (triplet + dates de seuil planifiées), l'entretien par politiques en données, les maisonnées, la maîtrise comme entrée du rendement.
- **Interdits** : vente et prix affichés (bouchon de débouché ; le reprice est la tranche 5) ; embauche réelle (dotation de travail bouchon, tranche 6) ; table Q(a) complète (politiques en données, décision 33) ; faillite (tranche 5, choc 5) ; tout niveau discret par firme ; toute accumulation d'expérience (tranche 9).

### 32.2 Types (pseudo-code)

```
struct Firme {
  id: IdEntité, type: TypeFirme, décideur: IdEntité,
  actifs: [IdEntité], stock_intrants: u64, stock_produits: u64,
  politique_entretien: PolitiqueEntretien,     // minimale | normale | renforcée (données)
  recette_courante: IdRecette, trésorerie: u64, dette_entretien_quartier: u64,
}
struct Recette { id, intrants: [(BienId, u64)], heures_par_poste: u16, production: u64, qualité: u8 }

struct Actif { id, firme: IdEntité, type: IdType,
  état_initial: u16, date_référence: u64, taux: u16 }
// état(t) = état_initial − taux × (t − date_référence)  — JAMAIS d'état courant stocké

struct DateSeuil { actif: IdEntité, seuil: u16, date: u64 }   // événement planifié, recalculé
```

`TypeFirme::Maisonnée` utilise la même structure et le même code que l'atelier : zéro branche spéciale.

### 32.3 API publique

```
fn produire(firme, heures_disponibles, intrants_disponibles) -> RésultatProduction
    // production = min(capacité des actifs, heures, intrants) ; facteur contraignant nommé au journal ;
    // intrants consommés avant production (I2)

fn état_actif(actif, jour) -> u16                    // paresseux, O(1), une soustraction

fn planifier_seuils(actif) -> [DateSeuil]
    // date exacte de franchissement de 400 puis 100, en entiers, division déclarée ;
    // recalculée à chaque changement de taux ou d'état (entretien)

fn entretenir(firme, politique) -> ()
    // consomme composants + heures ; modifie le taux ou remonte l'état ; replanifie les seuils

fn créer_maisonnée(patron: IdEntité) -> Firme        // même code que l'atelier
```

Le rendement multiplie la production par les coefficients de maîtrise des porteurs (entrée en données) ; l'expérience ne s'accumule pas ici.

### 32.4 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Contenu | Tests de fermeture |
| --- | --- | --- |
| 1 | Firme, recettes, production bornée | T4.5 (facteur contraignant nommé) |
| 2 | Stocks et conservation | T4.1 (I2), T4.2 (I6) |
| 3 | Usure paresseuse + dates de seuil | T4.3 (paresseux = naïf), T4.4 (dates exactes) |
| 4 | Entretien par politiques, replanification | T4.3 et T4.4 avec changements de politique |
| 5 | Maisonnées | T4.6 (équivalence bit à bit avec l'atelier) |
| 6 | Maîtrise en entrée du rendement | T4.7 (proportionnalité) |
| 7 | Snapshot avec actifs à mi-vie, replai étendu | T4.8, replais 1 à 3 inchangés |

### 32.5 Pièges déclarés (à vérifier en revue de code)

- **Ne jamais stocker l'état courant d'un actif** : le triplet (état initial, date, taux) suffit ; stocker l'état courant casse le calcul paresseux et le recalcul des dates.
- **Dates de seuil exactes** : calcul en entiers avec division déclarée (convention 26) ; replanifier à chaque changement d'entrée, jamais « au prochain passage quotidien ».
- **Production** : min des trois facteurs, facteur contraignant nommé au journal ; aucune production partielle inexpliquée.
- **I2** : vérifier la disponibilité des intrants avant d'engager les heures ; consommer avant de produire, jamais l'inverse.
- **Maisonnée** : si un `if type == Maisonnée` apparaît dans le code de production, c'est un défaut de conception à remonter.
- **Bouchons** : débouché (demande à prix fixe) et dotation de travail (heures par poste) derrière interface dédiée, remplaçables en tranches 5 et 6 sans toucher aux appelants.
- **Dette d'entretien du quartier** : compteur accumulé, consommé par q en tranche 7 ; jamais recalculé rétroactivement.
- **Maîtrise** : entrée de calcul seulement ; accumuler de l'expérience ici est un hors-périmètre (tranche 9).

### 32.6 Critères d'acceptation

- T4.1 à T4.8 verts ; I2 et I6 verts en continu ; usure paresseuse identique bit à bit à l'oracle naïf sur 10 ans avec changements de politique.
- Replais 1 à 3 inchangés ; replai 4 archivé en lecture seule.
- Aucune branche spéciale maisonnée ; bouchons remplaçables sans modification des appelants ; aucune vente, aucun prix, aucune embauche réelle.

## 33. Ordre de mission : tranche 5 (marché, prix, repas et crédit)

Document de travail pour l'agent de code. Source de vérité : section 17 (spécification), 6.2, 6.8, 6.13, 6.14, 6.16, 6.24, invariants I2, I3, I5, chocs 1, 2, 10, 11, 12. La tranche la plus dense : l'agent procède strictement étape par étape.

### 33.1 Objectif et interdits

- **Livrer** : les marchés de biens (prix affichés, files en pénurie), le reprice comme action Q(a) (actions économiques), la ville élastique (commandes datées, prix de référence), les modes de repas, l'eau refacturée, la fiscalité forfaitaire, les prêts (firme et entre individus), le défaut, la saisie, la servitude, la sous-location post-saisie.
- **Interdits** : marché du travail réel (bouchon de la tranche 4 inchangé jusqu'à la tranche 6) ; toute mise à jour de G (tranche 7) ; toute grève (tranche 8) ; toute règle d'équilibre ; tout prix hors bornes ; toute destruction de monnaie.

### 33.2 Types (pseudo-code)

```
struct Marché { bien: BienId, prix_affiché: u64, bornes: (u64, u64),
  file_pénurie: File<PrioritéPénurie>, seuils: (u64, u64) }
struct PrioritéPénurie { jour_arrivée: u64, rang: Rang, id: u64 }   // clé totale (arrivée, rang, id)
// rang : abonnement d'achat > standard ; enfants prioritaire en données

struct CommandeVille { bien: BienId, volume: u64, date_arrivée: u64 }  // J + délai, événement phase 1
struct PrixRéférence { bien: BienId, prix: u64, capacité_nominale: u64 }  // événement quotidien de demande

enum ActeurÉco { Firme(IdEntité), Individu(IdEntité) }   // prêts entre individus = même table

struct Prêt {
  id: IdEntité, prêteur: ActeurÉco, emprunteur: ActeurÉco,
  principal: u64, taux: u16,                    // 10 %/an, 20 % après défaut (données)
  garantie: Garantie, échéancier: [Échéance], état: ÉtatPrêt,
}
enum Garantie { Aucune, Logement(IdEntité), Servitude }   // servitude : heures à demi-salaire
enum ÉtatPrêt { Sain, EnDéfaut, Saisi, Soldé }
```

### 33.3 API publique

```
fn transacter(marché, acheteur, quantité) -> RésultatTransaction
    // au prix affiché ; si stock insuffisant → file de pénurie (allocation par (arrivée, rang, id)) ;
    // chaque transaction équilibrée (I3) et journalisée (phase 2)

fn reprice(firme, marché) -> ()   // action Q(a) ; heuristique par seuils de stock par défaut, en données ; bornes I5

fn commander_ville(bien, volume, jour) -> ()      // arrivée à J + délai (3 à 10 jours, données)
fn mettre_à_jour_prix_référence(bien, demande_agrégée) -> ()
    // +1 % par 2 % d'excédent, plafond +50 %, dérive lente à clé stable ; aucune règle d'équilibre

fn choisir_mode_repas(foyer, doctrine, salaire_réel) -> ModeRepas
    // pondère (coût monétaire, coût temporel, social) ; cuisine seulement niveau 2+ ; consomme du temps (I4)

fn prélever_fiscalité(firme) -> TransfertDéclaré    // 10 % à la source, sort vers la ville
fn verser_allocations(foyer) -> TransfertDéclaré    // vieillesse ½ panier, aide par enfant ¼

fn octroyer_prêt / fn échéance_prêt / fn constater_défaut / fn saisir / fn proposer_sous_location
    // intérêts composés en virgule fixe, capitalisés aux échéances ;
    // garanties dans l'ordre : logement, puis servitude ;
    // la saisie transfère le logement (jamais de destruction de monnaie) ;
    // la sous-location est proposée au nouveau propriétaire (Q(a)), jamais automatique
```

L'ardoise du restaurant crée un prêt dans la table des prêts entre individus, jamais un solde négatif.

### 33.4 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Contenu | Tests de fermeture |
| --- | --- | --- |
| 1 | Marchés, transactions, files en pénurie | T5.3 (I2, files), T5.4 (choc 1) |
| 2 | Équilibrage monnaie partout | T5.1 (I3 sur un an) |
| 3 | Reprice Q(a) + bornes | T5.2 (I5 sous chocs combinés) |
| 4 | Ville élastique : commandes datées, prix de référence | T5.5 (choc 2 + rétablissement, choc 9) |
| 5 | Modes de repas, eau refacturée | T5.10 (repas/salaire réel), T5.8 (choc 12) |
| 6 | Fiscalité forfaitaire | T5.7 (flux et u/p ; G attend la tranche 7) |
| 7 | Prêts, échéances, défaut, saisie, servitude | T5.6 (choc 10) |
| 8 | Sous-location post-saisie, plancher partiel | T5.9 (chaîne complète), T5.11 |
| 9 | Replai étendu, chocs rejoués | I9 ; chocs 1, 2, 10, 11, 12 explicables de bout en bout |

### 33.5 Pièges déclarés (à vérifier en revue de code)

- **I3 partout** : chaque mouvement de monnaie est une transaction équilibrée ou un transfert déclaré ; allocations et impôt sont des transferts déclarés, jamais des créations ou destructions.
- **La ville n'est pas une firme** : aucune entité firme pour elle ; seulement des commandes datées et un événement quotidien de prix de référence. Toute convergence vers un équilibre est une faute de conception.
- **Prix de référence ≠ prix affiché** : le premier est la donnée de la source ville, le second est la décision de la firme ; ne jamais les fusionner ni dériver l'un de l'autre automatiquement.
- **Files de pénurie** : clé totale (arrivée, rang, id) ; la priorité enfants est un rang en données, pas un tri parallèle.
- **Intérêts** : composés en virgule fixe, capitalisés aux échéances uniquement ; tout flottant ou capitalisation quotidienne implicite est un défaut.
- **Saisie** : transfert de propriété entre acteurs, monnaie conservée ; la sous-location est une proposition Q(a), jamais un effet automatique de la saisie.
- **Repas** : le mode choisi consomme du temps réel du budget (I4) ; l'ardoise est un prêt, pas un découvert.
- **Choc 11** : flux fiscaux et réponse de u/p seulement ; brancher G ici est un hors-périmètre (décision 36).
- **Élasticité** : pente, plafond et délais en données ; un dépassement de plafond est un défaut de calibrage à remonter, pas une exception à masquer.

### 33.6 Critères d'acceptation

- T5.1 à T5.11 verts ; I2, I3, I5 verts en continu ; chocs 1, 2, 10, 11, 12 explicables de bout en bout par le journal.
- Une saisie complète est traçable : prêt → défaut → saisie → proposition de sous-location, chaque maillon lié causalement.
- Replais 1 à 4 inchangés ; replai 5 archivé en lecture seule.
- Le bouchon de travail est intact et remplaçable en tranche 6 sans toucher aux appelants ; aucune grandeur en G, aucune grève.

## 34. Ordre de mission : tranche 6 (marché du travail, métiers et parrainage)

Document de travail pour l'agent de code. Source de vérité : section 18 (spécification), 6.3, 6.11, 6.15, 6.20, 6.22. L'agent ne décide rien.

### 34.1 Objectif et interdits

- **Livrer** : les cinq grandeurs du marché du travail, l'appariement événementiel, la Q(a) d'emploi modulée par les traits du décideur, les décisions ponctuelles de métier, les recettes multi-postes à goulot (décision 66), le parrainage, la bascule du bouchon de travail, l'engagement employé ↔ firme avec présentéisme et contagion de maladie à l'atelier (6.9, 6.20).
- **Interdits** : toute mise à jour de G (tranche 7), toute grève (tranche 8), toute accumulation d'expérience (tranche 9), tout marché clearing, toute règle spéciale au joueur, tout niveau discret par firme.

### 34.2 Types (pseudo-code)

```
struct Emploi { id: IdEntité, firme: IdEntité, cohorte_visée: IdEntité,
  heures: u16, salaire_horaire: u64, créneau: Créneau, état: Ouvert | Pourvu | Rompu }

struct MarchéTravail {
  disponibilité: ParCohorte<u32>,       // quotidienne (15.1)
  volonté: ParCohorte<u32>,             // événementielle, fonction du salaire réel cible
  demande: [Emploi],                    // postes publiés (décision des firmes)
  postes_pourvus: ParCohorte<u32>, heures_travaillées: ParCohorte<u32>,
}

struct TraitsDécideur { empathie: u16 }   // 0-1000 ; les autres traits viendront avec les factions

struct DécisionPonctuelle { type: IdType, options: [IdActionQ], clé_stable }  // ~1/mois

struct Engagement { employé: IdEntité, firme: IdEntité, valeur: u16 }  // 0-1000, événementiel, demi-vie 1-2 ans
// mis à jour par les décisions observées de la firme, pondérées par empathie × proximité (mémoire saillante)
```

### 34.3 API publique

```
fn publier_poste / fn embaucher / fn licencier / fn ajuster_salaire
    // actions Q(a) ; la chaîne (entrées, poids, décision) est journalisée ; coût et risques réels

fn apparier(jour: u64) -> [Embauche]
    // volontés × demande par cohorte et pratique, événement par événement (phase 4) ;
    // aucune convergence recherchée, les déséquilibres persistants sont légitimes

fn décision_ponctuelle(individu, événement_travail) -> IdActionQ
    // ~1/mois, à clé stable ; la même table résout joueur et PNJ

fn parrainer(employé: IdEntité, journalier: IdEntité) -> LienParrainage
    // condition : pratique ≥ 3 ; coût en temps régulier ; effet : essai accordé plus vite ;
    // lien durable dans la mémoire saillante des deux

fn basculer_dotation() -> ()
    // les heures du bouchon de travail sont reprises par les postes pourvus au fil des embauches ;
    // heures travaillées par cohorte continues, aucun saut d'état (T6.6)

fn mettre_à_jour_engagement(observateurs: [IdEntité], décision_firme: IdÉvénement) -> ()
    // événementiel : soutien +5 à +15, licenciement pour maladie −20 à −40 (× empathie × proximité) ;
    // aucune mise à jour sans événement journalisé ; jamais lu par le décideur (décision 57)

fn pondérer_arrêt_maladie(employé, engagement) -> Décision
    // engagement bas → présentéisme (travailler malade) ; engagement haut → repos et soin choisis ;
    // le présentéisme est une décision, jamais une règle

fn prévention_engagement(employé, poste) -> FacteurUsure
    // l'engagement réduit la probabilité d'incidents (usure légèrement ralentie, en données) ;
    // aucun temps de travail supplémentaire, aucun multiplicateur de recette (décision 64)

fn options_résolution(employé, incident) -> [OptionDécision]
    // engagement haut débloque « chercher une solution » en plus de « arrêter l'atelier »
    // dans les décisions ponctuelles ; la production préservée vient du résultat de la
    // décision, jamais d'un coefficient

fn production_goulot(firme) -> Production
    // min des capacités des étages pourvus de la recette (6.15, décision 66) ;
    // recalcul déclenché par embauche, départ ou incident, jamais par tick ;
    // le goulot (étage, amplitude) est journalisé

fn exposer_aux_malades(présents: [IdEntité], créneau) -> FacteurRisque
    // +5 à +15 % de risque de maladie par collègue malade présent (même lieu, même créneau) ;
    // travailler malade allonge la maladie (+20 à +50 % de durée) et monte le risque de séquelle
```

Modulation par les traits : seuil de licenciement ×(1 + empathie/1000), employabilité des malades selon la tolérance, pérénité dure (en faillite imminente l'empathie s'efface), dissonance au licenciement d'un employé connu (passe par le stress, collecteur unique).

### 34.4 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Contenu | Tests de fermeture |
| --- | --- | --- |
| 1 | Cinq grandeurs, entité Emploi, publication | T6.1 (partie publication) |
| 2 | Appariement événementiel par cohorte et pratique | T6.2 (choc 3) |
| 3 | Q(a) d'emploi + traits du décideur | T6.1 (chaînes complètes), T6.3 (choc 4) |
| 4 | Décisions ponctuelles de métier | T6.1 (table unique joueur/PNJ) |
| 4b | Recettes multi-postes à goulot | T6.8 |
| 5 | Parrainage | T6.4 |
| 6 | Deux régimes de gestion | T6.5 (divergence, aucun dominant) |
| 7 | Bascule du bouchon, suppression du bouchon | T6.6 (continuité des heures) |
| 8 | Engagement, présentéisme, contagion d'atelier | T6.7 (deux régimes divergent par les salariés) |
| 9 | Replai étendu | I9 ; replais 1 à 5 inchangés |

### 34.5 Pièges déclarés (à vérifier en revue de code)

- **Pas de clearing** : l'appariement ne cherche aucun équilibre ; ajouter une convergence salariale ou de plein emploi est une faute de conception.
- **Volonté événementielle** : recalculée quand le salaire réel ou G change, jamais par routine quotidienne.
- **Traits** : l'empathie module des poids de la table, elle ne court-circuite jamais la décision ; la pérénité dure est une condition, pas un trait.
- **Table unique** : toute branche spéciale au joueur dans les décisions ponctuelles est une faute (décision 39).
- **Savoir local** : le licencié emporte sa maîtrise ; la firme ne conserve pas les niveaux des partis.
- **Dissonance du décideur** : elle passe par le stress (collecteur), jamais par un nouveau registre.
- **Bascule** : le bouchon n'est supprimé qu'après le vert de T6.6 ; l'état agrégé doit être continu à la bascule.
- **Pas de bonus d'engagement** : l'engagement n'ajoute ni heures ni multiplicateur de recette ; il agit par la prévention (usure ralentie) et les options de résolution ; un temps donné ou un coefficient dérivé de l'engagement est une faute (décision 64).
- **Goulot par événement** : la production multi-postes ne se recalcule qu'à l'embauche, au départ ou à l'incident ; un recalcul par tick est une faute de coût (décision 66). Aucune sous-action de métier instanciée : la chorégraphie par objet est exclue.
- **Engagement caché** : le décideur ne lit jamais le scalaire ; il perçoit effort, churn et arrêts (décision 57). Le brancher dans sa Q(a) est une faute.

### 34.6 Critères d'acceptation

- T6.1 à T6.6 verts ; replais 1 à 5 inchangés ; replai 6 archivé en lecture seule.
- Bouchon de travail supprimé du code actif (archivé) après T6.6 ; aucune mise à jour de G ; aucune grève.

## 35. Ordre de mission : tranches 7 et 8 (griefs et basculements)

Document de travail pour l'agent de code. Source de vérité : sections 19 et 20 (spécifications), cadrage ch. 2, invariant I8, décision 43. Les deux tranches sont couplées : les entrées du seuil (7) précèdent le basculement (8) ; l'agent livre la 7 fermée avant d'ouvrir la 8.

### 35.1 Objectif et interdits

- **Tranche 7** : panier en net, salaire réel, G intégré événementiellement, concordance I8, entrée griefs du stress activée, entrées du taux de risque avec dates de franchissement planifiées.
- **Tranche 8** : basculement au franchissement (tirage unique), grève, réponses Q(a) des firmes, fins de grève causées.
- **Interdits** : toute intégration de G à pas fixe ; toute modification de l'équation dG = 0,8u + 0,2p − 0,25q (cadrage) ; tout re-tirage ; toute grève ou fin de grève sans cause journalisée ; tout flottant.

### 35.2 Types (pseudo-code)

```
struct Griefs { valeur: u16 }   // 0-100 ; cohorte = somme pondérée des individus (I8, tolérance zéro)

struct IntégrateurG {
  entrées: (u, p, q), date_dernière: u64, valeur: u16,
  // G(t) = valeur + (0,8u + 0,2p − 0,25q) × (t − date_dernière), entiers à échelle déclarée
}

struct TauxDeRisque { valeur: u32, date_franchissement: Option<u64>, tirage: Option<TirageGrève> }
struct Grève { cohorte: IdEntité, début: u64, durée: u64, cause_fin: CauseFin }
enum CauseFin { Concession(Q(a)), DuréeÉcoulée }
```

### 35.3 API publique

```
fn intégrer_g(entité, nouvelles_entrées, jour) -> ()
    // événementiel : mise à jour au changement d'une entrée, équivalente à l'intégration continue (T7.4)

fn calculer_taux_de_risque(cohorte, G, salaire_réel, moyens) -> u32   // coefficients en données
fn planifier_franchissement(cohorte, taux) -> DateSeuil
    // même mécanisme que l'usure paresseuse (16.3) : date exacte, recalculée à chaque changement d'entrée

fn basculer(cohorte) -> Grève
    // tirage unique au franchissement, journalisé, jamais re-tirable (décision 43) ;
    // la cohorte cesse d'offrir du travail pour la durée tirée

fn fin_de_grève(grève, cause) -> ()
    // concession (Q(a) des firmes) ou durée écoulée ; toujours causée, jamais par horloge arbitraire
```

### 35.4 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Tranche | Contenu | Tests de fermeture |
| --- | --- | --- | --- |
| 1 | 7 | Panier en net, salaire réel | T7.2 (choc 7) |
| 2 | 7 | G événementiel + concordance | T7.1 (I8), T7.4 (équivalence continue) |
| 3 | 7 | Entrée griefs du stress ; choc 11 complété | T7.3 |
| 4 | 7 | Taux de risque : entrées et dates planifiées | T7.5 |
| 5 | 8 | Basculement, tirage unique, grève | T8.1, T8.2, T8.5 |
| 6 | 8 | Réponses Q(a), fins de grève | T8.3 (choc 8 complet), T8.4 |
| 7 | 8 | Replai étendu, chocs 7 et 8 rejoués | I9 ; replais 1 à 6 inchangés |

### 35.5 Pièges déclarés (à vérifier en revue de code)

- **Intégration événementielle** : G se calcule depuis la dernière mise à jour ; un pas quotidien est une faute ; l'équivalence avec l'intégration continue de référence est un test, pas une opinion (T7.4).
- **Échelle entière** : 0,8, 0,2 et 0,25 sont des coefficients par jour à échelle déclarée (convention 26) ; les convertir en flottants est une faute.
- **I8 à tolérance zéro** : la somme pondérée passe par les totaux de cohorte (14.2), jamais par des moyennes recomposées.
- **Tirage unique** : consommé au franchissement et journalisé ; replanifier la date n'autorise jamais un second tirage (T8.2).
- **Fins de grève** : concession ou durée écoulée, toujours journalisées ; toute autre voie est une faute.
- **Entrée griefs du stress** : activée seule, sans toucher aux autres entrées (décision 29).

### 35.6 Critères d'acceptation

- T7.1 à T7.5 et T8.1 à T8.5 verts ; I8 à tolérance zéro en continu ; I9 vert, replais 1 à 6 inchangés.
- Aucune grève en régime sain, même avec agitateurs (T8.3, garde-fou de la section 24) ; le choc 8 combiné produit une grève explicable seulement si le salaire réel est bas.

## 36. Ordre de mission : tranches 9 à 11 (vie sociale, suite headless, P1b)

Document de travail pour l'agent de code. Source de vérité : sections 21, 22, 23 (spécifications), 24 (répliques et colportage), 26 (rituels), 27 (écran de conversation), 28 (directives), 12 (critères humains). La tranche 9 est glissable en P1.5 sans casser les tranches 1 à 8 ; l'agent la livre comme un lot autonome.

### 36.1 Objectif et interdits

- **Tranche 9** : maîtrise et apprentissage événementiels, reconnaissance et valuation des rôles, empathie et contagion, groupes informels, rituels (catalogue 26), vie vitale (naissances agrégées, décès, héritage), répliques et colportage (24).
- **Tranche 10** : aucun mécanisme nouveau ; suite complète (13 scénarios, 5 bots, 5 graines), observabilité (conventions 26 et 27), benchmark 300 ans.
- **Tranche 11** : écrans (doctrine, planification, vue quartier, journal, conversation), directives de firme (28).
- **Interdits** : tout timer d'apprentissage ; tout second registre de souffrance ; toute naissance individuelle (effectifs seulement) ; tout mécanisme dans les tranches 10 et 11 ; tout texte libre ; toute jauge numérique de grief ; la locale dans la simulation.

### 36.2 Types (sélection)

```
struct Maîtrise { niveau: u8, expérience: u32, seuil_risque: u32 }   // 0-10 ; tirage à clé stable
struct Reconnaissance { rang_groupe: u8, événements: [IdÉvénement] } // demi-vie 5 ans (section 8)
struct GroupeInformel { id: IdEntité, membres: [IdEntité], lieu_habituel: IdEntité, affinités }
struct Rituel { type: IdRituel, déclencheur: IdÉvénement, participants: [IdEntité] }
struct SélectionRéplique { intention: IdIntention, cible: IdEntité, événement: IdÉvénement, contexte: Contexte }
// Contexte = (niveau_relation, lieu, créneau) : les trois filtres de 24.3, en données
```

### 36.3 API publique (sélection)

```
fn accumuler_expérience(pratique, demi_journée_travaillée) -> ()   // franchissement de seuil journalisé
fn transmettre(vétéran, élèves) -> ()                              // plafond 1 vétéran / 3 élèves
fn mettre_à_jour_reconnaissance(individu, événement) -> ()
fn percevoir_souffrance(individu) -> u32
    // Σ empathie × affinité × souffrance visible, ensemble de conscience borné par la mémoire (20)
fn propager_par_groupes(stress) -> ()   // remontée vers les griefs de cohorte ; aucun second registre
fn déclencher_rituel(événement_de_vie) -> Rituel
fn naissance(cohorte) / fn décès(...) / fn hériter(...)   // I1, transferts déclarés, journalisés
fn sélectionner_réplique(individu, Q(a)) -> SélectionRéplique      // action Q(a) ; texte rendu hors état
fn expliquer(id) / fn séries_synthétiques / fn diff / fn reprise_snapshot   // observabilité, tranche 10
```

### 36.4 Étapes de construction (dans l'ordre, chacune fermée par ses tests)

| Étape | Tranche | Contenu | Tests de fermeture |
| --- | --- | --- | --- |
| 1 | 9 | Maîtrise, apprentissage, plafond vétérans | T9.1, T9.2 |
| 2 | 9 | Reconnaissance, valuation des rôles | T9.5 |
| 3 | 9 | Empathie, contagion, groupes informels | T9.3, T9.4 |
| 4 | 9 | Rituels (catalogue 26) | traçabilité des effets |
| 5 | 9 | Vie vitale : naissances, décès, héritage | T9.6 |
| 6 | 9 | Répliques, intentions, colportage (24) | T24.1 à T24.7 |
| 7 | 10 | Suite : 13 scénarios, 5 bots, 5 graines | T10.1 à T10.3 |
| 8 | 10 | Observabilité + benchmark 300 ans | T10.4 |
| 9 | 11 | Écrans : doctrine, planification, vue, journal, conversation | critères humains (section 12) |
| 10 | 11 | Directives de firme (28) | équivalence avec une firme PNJ |

### 36.5 Pièges déclarés (à vérifier en revue de code)

- **Apprentissage** : l'expérience vient de l'exécution ; tout timer est une faute (décision 45).
- **Contagion** : un seul chemin, par le stress puis les groupes ; un second registre de souffrance est une faute (décision 46).
- **Naissances** : effectifs de cohorte ; instancier un enfant comme individu est une faute (décision 47) ; les nommés naissent à l'extraction, vers 14 ans.
- **Répliques** : la sélection est journalisée, le texte est hors état ; toute chaîne de caractères dans l'état canonique est une faute (décision 51) ; la locale ne touche jamais la simulation (T24.1).
- **Tranche 10** : tout échec remonte à la tranche fautive ; corriger un mécanisme dans la suite est une faute (décision 48).
- **Tranche 11** : les écrans ne lisent que l'état et le journal, et éditent des données existantes ; toute règle de gameplay côté écran est une faute (décision 50).
- **Vie vitale** : chaque naissance et décès passe par un transfert déclaré (I1) ; l'héritage suit la règle en données, jamais une distribution par défaut codée.

### 36.6 Critères d'acceptation

- T9.x, T10.x et T24.x verts ; tous les invariants verts sur 5 graines et 13 scénarios ; benchmark 300 ans sous les seuils de la section 25.
- Critères humains de la section 12 : un humain joue 30 minutes et comprend pourquoi il s'en sort ou non ; revue de palier avec la fiche de revue (convention 24).
- Le joueur n'a aucun pouvoir que les PNJ n'ont pas : la revue vérifie la symétrie des tables, des écrans et des directives.