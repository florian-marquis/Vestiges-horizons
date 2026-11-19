---
name: "cadrage-4x-v4"
title: "Vestige : Horizon · Cadrage du 4X v7.7"
type: "text/markdown"
---

# Vestige : Horizon · Cadrage du 4X v7.7

Oct 14, 2026 · @Florian

> **v4** : tranché le langage du prototype (Rust d'emblée), lié les unités de conquête aux cohortes de population, ajouté le levier positif de succession (mentorat jouable, académies), et cadré le développement : solo assisté par agents IA (Cursor).
> **v4.1** : tirage des basculements par seuil de risque cumulé unique (remplace les fenêtres glissantes), limite de recrutement (effectif de cohorte contre production d'équipement), plafonnement du bonus de formation de l'héritier, replais de référence en lecture seule pour l'agent, conventions de déterminisme Rust, sprite de personnage en 32x64 px à calques, conventions déplacées dans un document séparé.
> **v5** : premier palier jouable = quartier vivant (chapitre 14), hiérarchie des consoles et modèle réseau unifié (chapitre 15), progression par dézoom, salaire réel et offre de travail par cohorte, doctrine personnelle du personnage éditable via terminal.
> **v5.1** : ajout du chapitre 16 (savoir et pratiques : pas d'arbre de recherche, maîtrise portée par des gens et des organisations, transmission par cinq canaux, innovation sans tirage exogène, orientation par les doctrines et κ, recherche publique, perte de savoir, délégation).
> **v6** : ordonnancement canonique des événements simultanés, invariants d'indépendance du découpage et de continuité d'identité, invariant d'extraction des cohortes, sémantique de l'équation de griefs, hiérarchie décisionnelle, reconnaissance séparée des besoins fondamentaux, capacité/autorisation/priorité des consoles, dimensions de l'héritage, trajectoire de validation après P1.
> **v6.1** : chapitre 17 (routes approuvées) : usure et entretien (version minimale dès P1), fiabilité des rapports (falsification symétrique), engagements crédibles (garanties matérielles d'abord), directives (ex-doctrines programmables), obsolescence unifiée au savoir du chapitre 16, dérive culturelle en extension optionnelle.
> **v6.2** : trois routes approuvées supplémentaires (chapitre 17) : vol et milice (P2), réseau de faveurs complet, spéculation et revente individuelle ; enrichissements tranchés pour la spécification P1 : crédit et saisie, santé-trajectoire, qualité des biens, logements différenciés avec sous-location, vie vitale minimale, parrainage ; le reprice devient une action de la firme.
> **v6.3** : routes 10 et 11 (boucle de population) : migrations inter-quartiers (P2, action individuelle par Q(a) sur le rang atteignable) et migrations interplanétaires par lignes de passagers (P3, flux agrégés de cohortes, information falsifiable, crédit de passage) ; P1 reste explicitement fermé aux flux de personnes ; l'invariant d'extraction est étendu aux transferts de cohortes.
> **v6.4** : états individuels déclenchés (jauge cachées, préemption par urgence, stress collecteur unique de santé mentale, ennui → loisir → attractivité) et principe du luxe comme temps libéré, chaque bien portant (coût monétaire, coût temporel) (chapitre 14) ; le stress alimente le moral des unités, la panique est un basculement (chapitre 11) ; fiscalité forfaitaire en P1 sans acteur ; diversité des lieux réglée par l'espace des arbitrages.
> **v6.5** : vivacité des PNJ (hétérogénéité, habitudes, mémoire active, répliques, groupes informels, rituels) et échelle d'interaction symétrique ; empathie comme trait et canal de contagion, angoisse, dépression, dissonance morale ; traits du décideur modulant le Q(a) des organisations ; valuation des rôles par curseurs de valeurs (route 6 étendue) ; route 12 : institutions de solidarité et groupes de pression.

## 1. Le late game : interruptions et valeur d'intervention

Le temps s'écoule en continu, avec pause active : le monde change, des événements locaux se produisent, l'information arrive en retard, le joueur intervient, et ses ordres voyagent puis sont interprétés sur place. Il n'y a pas de phases.

**Ce que voit le joueur**, pour chaque région :

- l'état observé à la date du dernier rapport ;
- l'état actuel estimé, avec son incertitude ;
- les ordres en transit ;
- les initiatives locales déjà connues.

**Classement des alertes.** Le produit Impact × Urgence × Irréversibilité est abandonné : il écrase les crises lentes, qui sont les plus dangereuses au sommet. On classe par valeur d'intervention :

```latex
V = \text{perte évitée si j'agis maintenant} - \text{perte évitée si j'agis plus tard, compte tenu du délai d'arrivée de mon ordre}
```

Une menace lente, vue avec quatre ans de retard, garde une forte valeur si l'ordre arrive avant le point de non-retour. Les alertes de même cause sont regroupées, et le joueur peut toujours consulter n'importe quel secteur de sa propre initiative.

**Ordres.** Ce sont des intentions structurées : objectif mesurable, budget, contraintes, horizon, niveau d'autonomie, conditions de réévaluation.

**Échelle de temps.** Avec des délais de plusieurs années, le joueur attendrait trop. Trois règles :

- une horloge unique à vitesse variable, pour garder le déterminisme simple ;
- un théâtre local simultané : c'est là où se trouve le personnage du joueur, qui agit en détail pendant que les décisions lointaines mûrissent (chapitre 13) ;
- une accélération qui ralentit automatiquement quand une alerte à forte valeur d'intervention apparaît.

## 2. Les crises : un taux de risque continu

Une crise n'est pas un événement scripté mais un état du monde auquel plusieurs systèmes réagissent, avec une probabilité de basculement qui monte avec la pression. Le déclencheur par seuils durs (G ≥ 65, L ≤ 40, 30 jours) est remplacé : il produisait des crises qui n'arrivent jamais à 64 puis tout d'un coup à 65.

**Variables agrégées par région**

| Variable | Sens | Plage |
| --- | --- | --- |
| u | part des besoins alimentaires non satisfaits | 0 à 1 |
| G | griefs de la population | 0 à 100, borné |
| L | légitimité perçue du pouvoir local | 0 à 100 |
| p | pression des prix sur les ménages | 0 à 1 |
| q | qualité effective des services publics | 0 à 1 |

**Sémantique (v6).** Les coefficients s'expriment en points de griefs par jour, mais le calibrage se fait sur des horizons pluriannuels : une pénurie durable doit saturer G en années, pas en mois, pour laisser aux crises lentes leur danger. L'intégration est événementielle : G est mis à jour quand une entrée change (u, p, q sont constantes par morceaux), avec bornes 0 à 100. La même équation s'applique au niveau individuel avec des coefficients par clé stable, et la somme des griefs individuels d'une cohorte doit rester concordante avec le grief agrégé (à tester). Les coefficients actuels restent des références de test tant que leur interprétation n'est pas validée en headless sur des horizons courts et longs.

Point de départ pour l'évolution des griefs, à calibrer (points de griefs par jour) :

```latex
\frac{dG}{dt} = 0{,}8\,u + 0{,}2\,p - 0{,}25\,q
```

**Du risque au basculement**

- Ces variables alimentent un taux de risque pour chaque type de basculement (grève, désobéissance administrative, coalition contestataire, sécession), pondéré par les moyens et les motivations des acteurs locaux présents.
- **Tirage unique par seuil de risque cumulé (v4.1)** : pour chaque crise naissante, un seuil de risque cumulé est tiré une seule fois (uniforme, à clé stable). Le moteur intègre le taux de risque au fil du temps ; la crise bascule quand le risque cumulé franchit le seuil. Quand les entrées changent (boucle G → L → q → G), le taux futur change et la date de franchissement est recalculée, sans nouveau tirage. Tant que le taux est constant, c'est exactement le tirage exponentiel : ni fenêtre à régler, ni taux périmé, ni biais de re-tirage. Compatible avec la file d'événements (le prochain franchissement prévu est un événement planifié, réévalué à chaque changement d'entrées).
- Le tirage utilise une clé stable : graine du monde, identifiant de l'entité, événement, indice du tirage.
- Les seuils servent aux alertes et à l'interface, jamais à la causalité.

**Modèle économique minimal.** Pour chaque région et chaque ressource : stock, production, demande, livraisons arrivées et livraisons attendues avec leur date. Le déficit net est la demande moins la production moins les importations entrantes ; la couverture est le stock divisé par ce déficit. Le franchissement d'un seuil de stock est planifié comme événement, sans décrémenter les stocks à chaque image.

**Exemple de chaîne émergente.** Un transporteur réaffecte des cargos vers une route plus rentable, les livraisons vers une colonie baissent, les réserves diminuent, les griefs montent, la légitimité du gouverneur baisse, puis il choisit une réponse selon ses moyens. Le pouvoir central l'apprend des années plus tard. Si la colonie avait des réserves ou si le gouverneur avait déjà reçu de l'autonomie, la crise peut être évitée.

## 3. Loyauté et décisions autonomes

Deux valeurs suffisent pour les personnages nommés, et la compatibilité idéologique se calcule au lieu d'être stockée.

- **Personnages nommés** : P, l'attachement personnel au joueur, et I, l'attachement à l'organisation, chacun de 0 à 100. La compatibilité idéologique se déduit de la distance entre le vecteur de motivations de la personne et celui du régime :

```latex
A = 1 - \tfrac{1}{2}\sum_i \left| w_{\text{personne},i} - w_{\text{régime},i} \right|
```

- **Anonymes** : regroupés en cohortes (effectif, motivations dominantes, attachement moyen, griefs, dispersion). Quand un individu devient important, il est extrait de sa cohorte avec une identité persistante, en conservant les totaux.
- **Invariant d'extraction (v6)** : l'extraction d'un individu d'une cohorte conserve exactement les totaux (population, richesse, griefs, attachement), en égalité stricte en virgule fixe ; la réinjection d'un individu disparu fait de même. Aucune extraction ne crée ni ne détruit population, richesse, griefs ou attachement. C'est un invariant testé en continu, pas seulement une convention de conception.
- **Règle de décision commune** : chaque organisation a des actions autorisées, chacune avec conditions d'accès, coût, résultat estimé, risque et durée d'effet. Le score d'une action :

```latex
Q(a) = \sum_i w_{\text{eff},i}\,\Delta\text{objectif}_i(a) - \lambda_c\,\text{Coût}(a) - \lambda_r\,\text{Risque}(a) - \lambda_d\,\text{Délai}(a)
```

- **Poids effectifs** : `w_eff = normaliser((1 − κ) × w_local + κ × w_régime)`, où κ mesure la force avec laquelle les institutions imposent les priorités du régime.
- **Stabilité des plans** : une faction ne change de plan que si un événement significatif survient ou si un seuil d'amélioration est dépassé.
- **Hiérarchie décisionnelle (v6)** : les décisions s'empilent en quatre niveaux, du plus stable au plus réactif : doctrine (poids w, très stable), objectifs stratégiques (réévalués sur événement significatif), plan opérationnel (réévalué sur changement de contrainte ou échéance), actions locales (évaluées par Q(a) quand l'opportunité se présente). Pas de nouveau moteur : la même règle de décision à des échelles de stabilité différentes. Les cadences de réévaluation sont pilotées par les données.
- **Désobéir sans trahir** : l'obéissance est une comparaison entre actions possibles. La loyauté pèse sur le choix, mais ne rend pas exécutable un ordre impossible.
- **Scission** : chaque personnage évalue les camps selon son attachement personnel, son intérêt matériel, sa compatibilité avec chaque régime et le risque de son choix. Certains suivent le joueur, d'autres restent, d'autres attendent de voir qui l'emporte.

## 4. Inertie et régimes

Le joueur garde une grande liberté tactique, mais changer les règles du système est lourd : l'inertie vient des intérêts et des capacités du monde, pas d'un délai arbitraire.

| Couche | Comportement |
| --- | --- |
| Priorité locale | Modifiable immédiatement par le responsable compétent |
| Politique opérationnelle | Modifiable vite, effets limités par les ressources et les délais |
| Régime institutionnel | Soutien, légitimité et réorganisation nécessaires |
| Culture et préférences | Évolution lente, par l'expérience, les générations et les institutions |

**Réforme de régime** (valeurs de départ, à calibrer) :

- coalition d'au moins 60 % du pouvoir politique pondéré ;
- légitimité d'au moins 35/100 ;
- transition de six mois à cinq ans de simulation, ralentie par la résistance des institutions, accélérée par la coalition, avec des opposants actifs pendant ce temps ;
- changement culturel profond : dix à cinquante ans.

Une voie autoritaire (coup de force) reste possible, avec ses propres coûts et risques. Les priorités du joueur ne sont jamais verrouillées artificiellement. Une politique visant une colonie distante ne prend effet qu'après réception de l'ordre, et peut être inadaptée à ce moment-là.

## 5. Technique : solo, déterminisme, journal de causalité

Le premier objectif est un jeu solo dont le cœur de simulation tourne sans rendu, se sauvegarde, se rejoue et se teste de façon reproductible.

- **Séparation stricte** entre l'état canonique, les commandes du joueur, les événements produits par le moteur, et la représentation graphique.
- **Langage (v4, tranché)** : Rust d'emblée, cœur séparé du rendu. La réécriture d'un noyau Python en Rust changerait les identifiants, l'ordre des tirages et donc les replais de non-régression qui servent de tests ; écrire directement en Rust préserve la suite de replais dès le premier jour. Le surcoût est réel même en développement assisté par IA, notamment les pièges de déterminisme (ci-dessous), mais il est payé une seule fois.
- **Développement solo assisté par agents IA (Cursor, v4)**. Le projet est développé par une personne avec des agents IA. Cela change la méthode plus que l'architecture :
  - ce document et les spécifications de palier sont la source de vérité des agents : chaque tâche commence par une spécification écrite (entités, événements, invariants, critères de réussite) ;
  - les tests headless, les replais de non-régression et la simulation de référence sont les garde-fous principaux de l'agent, mais pas absolus : un agent peut faire passer un test en régénérant le replai de référence. **Règle (v4.1)** : les replais de référence sont en lecture seule pour l'agent ; toute modification d'un replai de référence exige une validation humaine explicite, motif documenté ;
  - **Conventions de déterminisme Rust (v4.1)** : jamais d'itération sur une HashMap (ordre non spécifié, graine aléatoire par défaut) : BTreeMap, IndexMap ou hasher à graine fixe ; fusion déterministe des résultats en parallélisme par région ; entiers ou virgule fixe pour tout calcul reproductible ; les règles complètes vivent dans le document de conventions, court et séparé, qui sert de référence à Cursor ;
  - petites tranches verticales : une entité, quelques événements, un test, livrés en une session ; jamais de grande refactoration sans replai de comparaison avant et après ;
  - conventions explicites (nommage, format des événements du journal de causalité, format des données pilotant recettes, actions et régimes) : les agents suivent les conventions écrites, pas les implicites ;
  - les révisions humaines portent sur le design, les résultats de simulation (courbes de griefs, stocks, équilibre des bots) et les changements de replais de référence, pas sur la lecture ligne à ligne du code.
- **Déterminisme** : entiers ou virgule fixe pour tout calcul reproductible, tirages à clés stables (graine, entité, événement, indice) plutôt que dépendants de l'ordre des threads.
- **État** sérialisable, historique utile conservé sous forme d'événements.
- **Multijoueur** : pas de lockstep imposé. Si le besoin apparaît, serveur autoritaire et coopération asymétrique entre responsables de sous-factions ou de secteurs.
- **Journal de causalité** : chaque événement important enregistre identifiant, date, acteur, décision, valeurs modifiées, événements antérieurs impliqués, et date à laquelle le joueur peut l'apprendre. Il distingue ce qui est observé de ce qui a seulement rendu l'événement probable.
- **Rétention du journal** : sur plusieurs siècles, chaînes récentes détaillées, chaînes anciennes compactées, seuls les événements marquants conservés.

## 6. Secteur privé

Les capacités du secteur privé sont des actions de la table d'actions de chaque type d'acteur, pas des systèmes indépendants, et elles s'ajoutent dans cet ordre :

1. **Investir** : affecter du capital à de la production, du stockage ou du transport, avec coût initial, délai de construction, capacité finale, entretien, demande prévisionnelle et risque. Une firme peut investir trop tôt, trop tard ou dans une région que ses concurrents abandonneront.
2. **Réorganiser la logistique** : répartir les capacités de transport entre routes et contrats selon revenus, coûts, risques et délais. Le joueur agit indirectement, par les contrats, les subventions, la sécurité et les infrastructures.
3. **Faire du lobbying** : peser sur une réglementation ou un contrat, selon la puissance économique, les relations et la légitimité des institutions. Cela agit sur les coalitions, les coûts de réforme et la distribution de l'influence, sans nouvelle simulation sociale complète.

Cartels, contrebande et autres formes de contournement viennent beaucoup plus tard, une fois que l'économie de base produit déjà des comportements intéressants.

**Un décideur ne voit que ce qui lui parvient** : la table d'actions d'un décideur nommé raisonne sur son information perçue (ce qu'il a appris, daté au journal de causalité), jamais sur l'état réel du monde. L'asymétrie d'information est un fait politique, pas un bug : un patron dont les employés évitent sa table découvre le mécontentement en retard ; un rapport falsifié trompe son lecteur ; une information périmée fait décider trop tard. Application progressive : information sociale dès le palier quartier (P1.5), rapports et falsification aux paliers suivants.

## 7. Plan de prototypage par paliers

Le prototype avance par paliers, chacun fermé par une question à trancher avant d'ajouter la couche suivante.

&#91;embedded content: plan de prototypage · 5 paliers, une question par palier\]

P1 s'écrit en premier : si l'économie seule ne produit pas de situations intéressantes, ajouter du politique n'y changera rien.

**Propriétés testées en headless** : émergence, diversité, lisibilité, contrôlabilité, stabilité.

**Critère de réussite** : sur une graine donnée, sans scénario, savoir pourquoi la crise a émergé, quelles décisions auraient pu la changer, et pourquoi le joueur ne pouvait pas la régler par un ordre instantané.

## 8. Qui est le joueur sur plusieurs siècles

Le joueur contrôle toujours son personnage (chapitre 13). La continuité sur plusieurs siècles repose donc sur trois éléments : l'organisation que le personnage fonde et qui porte sa doctrine, ses successeurs (héritier ou protégé qui partage ses valeurs, qui devient le personnage du joueur à sa mort), et le personnage initial, qui reste une entité persistante du monde (conseiller ou symbole) une fois remplacé. Les trajets de plusieurs années sont traités au chapitre 13.

La durée de vie, la longévité graduée et la place de la succession sont précisées au chapitre 13.

## 9. Contraintes et principes d'architecture

Le projet vise une profondeur maximale sans économie de temps de développement, un jeu fluide en très late game et des couches ajoutables au fil des années. Cela se traduit par six règles.

**Ajouter des couches sans casser le noyau**

- Les couches ne s'appellent pas entre elles : elles communiquent par les événements et par un petit nombre de variables agrégées partagées (stocks, griefs, légitimité, sécurité, prix).
- Tout est piloté par les données : actions, ressources, recettes de production, régimes, grades.
- Sauvegardes versionnées avec migration, et suite de replays déterministes servant de test de non-régression (une graine, 200 ans, comparaison des indicateurs avant et après chaque ajout).

**Rester fluide en late late game**

- Chaque couche déclare un budget de coût : comment elle reste proportionnelle à ce que le joueur regarde (événementielle, agrégée, instanciée à la demande).
- Plafond de mémoire : entités mortes ou dormantes archivées, journal compacté par âge, historiques résumés.
- Test de performance en intégration continue : simulation headless de 1 000 ans sur quelques graines, avec un seuil de temps par jour simulé. Une couche qui le dépasse est retravaillée.
- Simulation parallélisée par région, de façon déterministe (tirages à clés stables).

**Se déplacer dans une station, un vaisseau, et piloter**

- Trois échelles d'espace : intérieur, espace local, carte stratégique, reliées par des transitions (amarrage, décollage, entrée dans une bulle locale).
- Une seule source de vérité : l'intérieur d'une station est une vue dérivée de son état agrégé (modules, stocks, cohorte d'équipage). Un achat à un terminal est une vraie transaction de marché.
- Les vaisseaux de la bulle locale sont instanciés depuis les flottes agrégées et reversés à l'agrégé quand on s'éloigne : le vaisseau piloté est le même objet que dans la simulation globale.
- Le contexte fixe la vitesse du temps : dans une bulle active (combat, amarrage), l'horloge revient à 1× ; sur la carte ou en croisière, elle accélère.
- Décision révisée : le personnage étant toujours contrôlé, la marche, les véhicules, le décollage et le voyage existent en version minimale dès le socle (une promenade locale, un cockpit avec une console, un trajet sur rails). Seuls les intérieurs détaillés progressent ensuite, à partir de plans générés depuis les modules.

La génération du terrain et la taille des sauvegardes sont traitées au chapitre 13 : seuls la graine, la version du générateur et les écarts sont sauvegardés, le reste se régénère.

**Une liberté qui ne soit pas illusoire**

1. Le joueur n'a aucun pouvoir que les PNJ n'ont pas : même table d'actions.
2. Les limites viennent de la physique et de l'information (ressources, distances, délais, connaissances, légitimité), pas de portes scriptées.
3. Chaque action laisse des traces persistantes (chronique, réputation, rivalités).
4. Plusieurs chemins viables vers un même but, chacun avec son coût : fonder, acheter, conquérir, saboter, corrompre.

Vérification : des bots de stratégie (marchand, pirate, conquérant, bâtisseur) jouent des centaines de parties en headless. Si une stratégie domine toutes les autres, ou si l'une est impossible, la liberté est illusoire et les chiffres le montrent.

**Règles de déterminisme temporel (v6)**

- **Ordonnancement des événements simultanés** : deux événements ne partagent jamais par accident le même rang d'exécution. Ordre canonique : horodatage logique (avec numéro de séquence d'horloge pour distinguer les instants), puis phase justifiée par le recensement des événements de P1 (aucune phase inventée avant ce recensement), puis priorité définie en données si nécessaire, puis identifiant stable de l'événement. Les événements générés pendant le traitement d'un événement partent au prochain instant logique, jamais en exécution immédiate réentrante : les boucles au même instant sont interdites par construction.
- **Indépendance du découpage (invariant)** : le résultat d'une simulation est strictement indépendant du découpage en régions, du nombre de threads et du temps réel de calcul. Le mécanisme de synchronisation (frontières d'échange, visibilité des événements interrégionaux, zones prêtes, événements en transit) sera spécifié au premier palier multi-régions ; P1, qui n'a qu'un quartier, s'exécute séquentiellement. Le temps de calcul disponible décide seulement de quand une zone est prête, jamais du résultat. Retards de calcul internes et délais de communication vécus par les personnages restent des choses distinctes.
- **Protocole de continuité d'identité** : tout passage agrégé ↔ détaillé conserve les identifiants, les quantités et ressources (ni création ni destruction), les relations et contrats nécessaires ; les variables agrégées restent réconciliables avec l'état détaillé selon une règle définie. La liste exacte des champs conservés par type d'entité (ressources, propriété, blessures, griefs, mémoire saillante, contrats, ordres en transit) est spécifiée dans les spécifications de palier. Les cycles agrégé → détaillé → agrégé sont testés sur la même graine, sans action supplémentaire, jusqu'à conservation stricte.
- **Trajectoire de validation après P1** : quartier vivant, puis quartier relié à une station (une organisation économique au-dessus), puis petit réseau de quelques marchés, puis vérification de l'agrégation, des communications et de la délégation. Hypothèses de travail pour l'étude, à confirmer en spécification : 150 individus, 5 à 10 entreprises, 3 à 5 cohortes, 10 ans simulés, plusieurs graines. P1 ne valide que le modèle local ; cette trajectoire vérifie que ses fondations n'empêchent pas l'échelle suivante, sans élargir le premier jouable.

**Ordre de développement.** Chaque couche doit être jouable seule avant la suivante :

1. noyau de simulation ;
2. économie et chaîne de production (extraction, traitement, composants, unités), le combat y restant abstrait ;
3. logistique ;
4. politique et crises ;
5. personnage au sol, véhicule, cockpit avec console, décollage et trajet sur rails, en version minimale (chapitre 13) ;
6. noyau tactique commun, d'abord en combat au sol abstrait puis local ;
7. postes de commande et écrans tactiques (accès, retard, capacité) ;
8. carrière de l'individu, en commençant par la voie du soldat ;
9. combat spatial et pilotage assisté ;
10. abordage et intérieurs de vaisseaux ;
11. intérieurs de stations détaillés.

## 10. Chaîne de production et logistique

Le gameplay de base repose sur une seule chaîne continue, de l'extraction à la production d'unités, avec le combat comme consommateur et la logistique comme liant. Chaque échelon de la carrière en est un maillon : le joueur commence par miner, transporter ou combattre à bord d'un vaisseau, puis délègue des routes, des stations, des chaînes entières.

```
extraction → traitement → composants → unités / stations
        ↘ commerce ↗         ↘ logistique ↗        combat (consomme et détruit)
```

**Chaîne minimale pour le palier P1.** Les valeurs sont des points de départ à calibrer.

| Étape | Entrées | Sortie | Durée |
| --- | --- | --- | --- |
| Extraction (minerai) | mine | minerai, taux fixe par jour | continue |
| Extraction (carburant) | puits ou collecteur | carburant, taux fixe par jour | continue |
| Raffinage | 3 minerai | 1 alliage | 2 jours |
| Assemblage | 2 alliage | 1 composant | 3 jours |
| Chantier | 4 composants | 1 chasseur | 10 jours |

Le carburant sert aussi à déplacer les cargos : c'est lui qui rend la distance coûteuse et relie la chaîne à la logistique. Le transport coûte aussi de la monnaie et du temps de trajet. La nourriture s'ajoute avec les populations au palier P2.

**Principes de modélisation**

- **Des flux, pas des objets.** Une station est un nœud avec des taux d'entrée et de sortie et des stocks. Tant que rien ne change, rien n'est recalculé. Les événements sont des ruptures de flux : stock vide, capacité atteinte, fin d'un lot, arrivée d'un cargo.
- **Trois niveaux de transformation** au départ : brut, matériau, composant, puis unité. Chaque niveau ajouté multiplie les liens à gérer et à comprendre.
- **Peu de ressources, bien différenciées.** Une ressource rare sert plus le jeu que dix ressources interchangeables.
- **Recettes en données** : ajouter une chaîne ne demande pas de code.
- **Unités produites en stocks agrégés** (« 12 chasseurs en réserve à la station X ») jusqu'à leur engagement, puis instanciées pour le combat.

**Logistique**

- **Nœuds** : stations et colonies, avec stocks et taux d'entrée et de sortie.
- **Arcs** : routes avec temps de trajet et capacité. Sans FTL, les distances sont la contrainte principale.
- Un contrat de transport relie une source, une destination, une ressource, un volume par période, une échéance et un prix. C'est la forme de coordination entre acteurs et le levier d'action indirect du joueur.
- Un transporteur est d'abord une capacité allouée à des contrats, et ne devient un objet individuel que dans la bulle locale du joueur.
- Chaque route a un délai (distance, vitesse) et un risque (piraterie, conflit). Le transporteur répartit sa capacité selon revenus, coûts, risques et délais, comme décrit pour le secteur privé.
- **Événements** : arrivée d'un cargo, fin de production, franchissement d'un seuil de stock, fin de contrat.
- Une rupture à un maillon (carburant, composant) remonte jusqu'aux unités et à la capacité militaire, avec les mêmes mécanismes de stocks et de retards que pour les crises du chapitre 2.

**Le combat dans la chaîne**

- Au niveau abstrait, un engagement se résout par rapport de forces avec pertes : les unités détruites créent une demande de remplacement qui empêche l'économie de saturer.
- Le combat est aussi une cause de crise : route coupée, station perdue, flotte détruite.
- La piraterie réduit la capacité effective des routes et crée un risque que les transporteurs intègrent dans leurs décisions.
- Le combat détaillé (tactique locale, pilotage assisté, abordage) est traité au chapitre 11 ; le combat abstrait suffit aux paliers P1 et P2.
- **Unités de conquête liées aux cohortes (v4, tranché)** : les troupes ne sont pas de simples unités produites. Chaque formation recrutée draine une cohorte de population d'origine (effectif, attachement, griefs). Les pertes remontent à la cohorte : deuil, griefs, baisse d'attachement dans la région d'origine, demande de soins et de remplacement. Une guerre impopulaire nourrit donc les crises du chapitre 2, une armée de métier bien traitée devient un pilier de légitimité, et conquérir coûte politiquement même quand tout se passe bien militairement. La cohorte d'origine des unités est inscrite dans le journal de causalité, ce qui permet d'expliquer une crise par les pertes d'un régiment trois ans plus tôt. **Limite de recrutement (v4.1)** : le recrutement est borné par le plus contraignant des deux goulots, l'effectif disponible de la cohorte d'origine et la production d'équipement de la chaîne. Un recrutement massif vide les cohortes (griefs, baisse d'attachement) même si l'équipement abonde ; une pénurie de composants fige les effectifs même si la population est disponible. La même règle s'applique au joueur et aux PNJ.
- La conquête de stations et de planètes concerne le sommet de l'échelle (populations, légitimité) et n'arrive pas avant P2.

**Ce que P1 doit montrer.** Pas de stock négatif, pas de goulot permanent sur un seul maillon, des prix qui restent finis, et des pénuries qu'on peut expliquer en remontant la chaîne depuis le journal de causalité.

## 11. Combat et conduite tactique

Le jeu privilégie la profondeur, la cohérence de la simulation et les choix tactiques, pas la qualité graphique ni le pilotage spectaculaire. Ce n'est pas un space shooter : le réalisme vient d'un modèle assez détaillé pour produire des décisions, des conséquences et des contre-stratégies, pas d'une physique exhaustive. Un modèle n'est complexifié que lorsqu'une approximation change sensiblement les décisions du joueur.

**Un seul noyau tactique.** Combat spatial, combat au sol et abordage partagent le même pipeline (détection, engagement, interception, dommages, soutien). Seuls l'environnement et les paramètres changent, ce qui évite de maintenir trois moteurs. Précision (v6) : le noyau partagé porte les étapes du pipeline ; les modèles de résolution, paramètres et contraintes restent spécialisés par environnement chaque fois que l'approximation commune changerait réellement les décisions.

| Système | Combat spatial | Combat au sol | Abordage |
| --- | --- | --- | --- |
| Détection | capteurs, signature, portée, qualité et fraîcheur des pistes | observation, reconnaissance, capteurs | capteurs locaux, visibilité des compartiments, alertes |
| Engagement | choix des cibles, fenêtres de tir, priorités, capacité à traiter plusieurs menaces | tir, posture, couverts, objectifs | ordres et règles d'engagement en espace confiné |
| Interception | trajectoires, délais, saturation, ressources consommées | tirs indirects et drones, quand présents | blocage d'accès, portes, verrouillages |
| Dommages | sous-systèmes dégradés, conséquences opérationnelles | blessures, pertes de spécialistes, matériel | combattants, portes, cloisons, consoles locaux |
| Soutien | stocks, rechargement, communications, guerre électronique, drones | munitions, ravitaillement, liaisons, renforts | munitions, charges de brèche, renforts |

**Simulation événementielle.** Le temps est continu mais piloté par les événements pertinents : nouvelle détection, changement d'ordre, lancement, interception probable, perte de piste, impact. Les trajectoires se calculent par segments et sont réévaluées quand un événement survient.

**Trois niveaux de résolution**

1. **Agrégé** : formations, détachements, flottes ; progression, détection, pertes estimées, cohésion et contrôle des objectifs, sans représenter chaque combattant. Pour les opérations lointaines et secondaires.
2. **Tactique détaillée** : positions, couverts, lignes de vue, escouades et objectifs. Pour une bataille importante.
3. **Incident local décisif** : une zone précise (brèche, couloir, poste de contrôle) sans détailler tout le champ de bataille ou tout le vaisseau. Pour un point de résistance déterminant.

On n'ouvre une situation tactique que si l'intervention du joueur peut changer le résultat ; sinon le combat se résout au niveau agrégé.

**Règle d'identité.** Une unité agrégée qui devient détaillée reste la même unité : pertes, moral, expérience, équipement et ordres sont conservés. À la fin du combat détaillé, survivants, blessés, munitions consommées, positions et objectifs atteints sont réinjectés dans le modèle agrégé. Le passage d'un niveau à l'autre ne recrée pas de soldats, n'efface pas de pertes et ne permet pas de rejouer un combat résolu : le détail est une autre représentation de l'état existant. Les combattants individuels suivent la règle des cohortes : anonymes et agrégés, extraits avec une identité persistante quand ils sont nommés ou engagés en détail.

### Combat spatial

L'objectif est qu'aucune arme ni défense ne domine : les doctrines évoluent par contre-mesures.

- **Armes à énergie** : pas l'armement principal contre les vaisseaux. Elles servent à éblouir ou dégrader des capteurs, perturber la détection et, dans de bonnes conditions, neutraliser de petits drones. Leur efficacité dépend des conditions d'engagement.
- **Missiles** : coûteux et performants, capables d'emporter des systèmes plus complexes.
- **Drones d'attaque consommables** : une catégorie distincte, pas un missile au rabais. Ils répondent à une autre doctrine, avec des compromis de vitesse, de charge utile, d'autonomie, de coordination et de dépendance aux communications. Leur coût réel (conception, fabrication, transport, stockage, coordination, pertes) pèse sur la capacité à soutenir une campagne, ce qui les relie à la chaîne de production.
- **Saturation** : la défense est limitée par le nombre de pistes suivies, de cibles engageables simultanément, les stocks d'intercepteurs, les délais de rechargement et la capacité à discriminer les menaces.
- **Contre-doctrines** : missiles intercepteurs, défense cinétique rapprochée, guerre électronique et leurres, drones défensifs, meilleure détection et meilleur suivi des pistes, dispersion des forces, gestion des stocks.
- **Ce qui compte** : stocks, cadence, portée, qualité des pistes et capacité de traitement des engagements, plus que des caractéristiques abstraites de dégâts.
- **Commandement** : ordres de haut niveau, doctrines et règles d'autonomie, sans microgestion permanente. Le pilotage direct d'un vaisseau reste possible mais assisté (ordres de manœuvre), et la résolution passe par le modèle tactique.
- **Vue** : 2D de dessus, centrée sur la décision (contacts, pistes incertaines, trajectoires estimées, fenêtres d'interception, zones de menace, état des défenses, stocks).

### Combat au sol

**Trois objets, pas des soldats commandés un par un**

| Objet | Fonction | Détail simulé |
| --- | --- | --- |
| Combattant | Individu avec compétences, expérience, blessures, équipement | Individuel seulement si cela influence le résultat |
| Escouade | Groupe tactique qui reçoit les ordres | Position, cohésion, munitions, moral, état des membres |
| Formation | Regroupement d'escouades et d'unités spécialisées | Progression, couverture, soutien, pertes, capacité opérationnelle |

Les rôles (fusilier, spécialiste antichar ou explosifs, médecin, opérateur de drones) donnent accès à des capacités, pas seulement des bonus : perdre le spécialiste d'explosifs peut empêcher de franchir un obstacle, perdre un chef expérimenté dégrade la coordination.

**Ordres et exécution**

- Le joueur donne une intention, un objectif et des contraintes : déplacement, posture, engagement, coordination. L'escouade choisit comment l'exécuter selon ses capacités et ses informations.
- Les ordres ne garantissent pas un comportement parfait : une unité mal entraînée, isolée ou privée de communications peut réagir tard ou renoncer à une manœuvre trop dangereuse. Chaque comportement reste explicable par l'entraînement, le moral, l'information et les ordres reçus.
- Les échanges de tir dépendent de la détection, de la portée, du terrain, de la posture, de la cohésion, de la fatigue, des munitions, du soutien et des communications. Ils se calculent par salves ; seuls les impacts qui changent l'état opérationnel sont détaillés individuellement.

**Terrain.** Il porte des propriétés fonctionnelles : couverts, élévation, bâtiments, décombres, zones ouvertes, obstacles, installations capturables ou désactivables. Les déplacements sont continus ou semi-continus, pas bloqués sur une grille ; les étages sont des niveaux séparés reliés par escaliers, ascenseurs ou rampes.

**Objectifs et capacité opérationnelle.** La mission définit la victoire (prendre ou défendre une installation, sécuriser une zone d'atterrissage, extraire des personnes, tenir jusqu'aux renforts, détruire une capacité précise, forcer un retrait). Une unité nombreuse mais privée de spécialistes, de ravitaillement ou d'accès à l'objectif n'est plus capable de sa mission : la capacité opérationnelle compte plus que le nombre de survivants.

**Moral, reddition et pertes persistantes.** On suit au minimum l'effectif disponible, la cohésion, le moral, la fatigue, les munitions et les compétences critiques. Le moral n'est pas une simple barre : il peut conduire à ralentir, se replier, refuser une action risquée ou se rendre. Le stress individuel et de cohorte (spécification P1 : états cachés) est une entrée du moral : la panique est un basculement du moral, tiré par seuil de risque cumulé (chapitre 2). Les états individuels (faim, fatigue, stress, douleur) traversent la règle d'identité : une unité issue d'une cohorte misérable, en santé et salaire réel bas, panique plus tôt qu'une unité équivalente issue d'une cohorte aisée. La qualité de vie est une variable militaire. Pertes et blessures se répercutent dans la simulation globale : un spécialiste perdu n'est pas remplacé instantanément, les blessés demandent des soins et une récupération.

### Abordage

L'abordage a une identité propre : un environnement confiné, des accès limités, une coque et des systèmes locaux qui peuvent être endommagés, et le plus souvent l'objectif de prendre un bâtiment que l'on veut conserver. Il réutilise le noyau tactique du combat au sol.

**Cinq phases**, résolues à l'échelle que justifie l'importance du vaisseau :

1. **Préparation** : évaluer l'état du vaisseau, les accès, les défenses connues et l'effectif, avec des informations possiblement incomplètes ou obsolètes.
2. **Approche et neutralisation locale** : atteindre un point d'accès malgré les défenses, qui peuvent retarder l'opération ou forcer à changer d'entrée.
3. **Création de la brèche** : accès forcé avec du matériel spécialisé et du temps.
4. **Progression intérieure** : sécuriser les compartiments utiles ; les défenseurs se regroupent, bloquent des accès, abandonnent des secteurs ou contre-attaquent.
5. **Contrôle et sécurisation** : le contrôle n'est acquis que si les objectifs nécessaires sont tenus et que les défenseurs ne peuvent plus reprendre la situation.

Ces phases ne sont pas un scénario rigide : un petit vaisseau faiblement défendu se prend vite, un transport militaire préparé à l'abordage devient une opération longue, coûteuse et incertaine.

**La brèche forcée est une décision.** Elle fixe la situation tactique initiale :

- un délai d'ouverture et une consommation de matériel ;
- une exposition temporaire des assaillants et un passage étroit qui limite les combattants engagés ;
- des dégâts locaux à la porte, à la cloison ou à des équipements ;
- un état de compartiment simplifié (pression normale, dégradée, dépressurisée, inaccessible), sans simuler les fluides ;
- une alerte donnée aux défenseurs, qui peuvent se repositionner.

Une brèche rapide n'est pas toujours préférable à une brèche préparée : le matériel, le temps, les protections et la connaissance du vaisseau créent des choix.

**Intérieur en graphe fonctionnel.** Chaque vaisseau possède une représentation abstraite de ses compartiments (fonction, accessibilité, état local, occupants, dégâts) et de leurs connexions (porte, cloison, échelle, sas ou passage, ouvert, fermé, bloqué ou détruit). Ce graphe est dérivé de l'état du vaisseau, selon la règle de la source de vérité unique, et suffit à résoudre un abordage ordinaire. Quand un combat important survient, le jeu génère ou charge une représentation détaillée des seuls compartiments concernés. Cela permet aussi des vaisseaux procéduraux crédibles sans plan intérieur dessiné à la main.

**Objectifs.** Il n'y a pas de règle universelle du type « capturer la passerelle = gagner ». Les objectifs pertinents (point de commandement local, communications internes, poste de contrôle des accès, local technique, zone de rassemblement) dépendent de la classe du vaisseau : un cargo automatisé, un transport militaire et un croiseur n'ont ni les mêmes points de contrôle ni les mêmes défenses.

**Dégâts locaux seulement.** L'abordage endommage les combattants, les portes et verrous, les cloisons, les capteurs et consoles locaux, les communications internes, le contrôle d'accès et les équipements techniques d'un compartiment. Il ne déclenche pas de simulation complète de la propulsion, de la navigation, du réacteur ou des réseaux. Une extension pourra être envisagée plus tard si le gameplay l'exige.

**Capturer n'est pas exploiter.** Après la résistance, il faut évaluer le contrôle de l'intérieur, l'état des accès, la disponibilité de l'équipage, la présence des spécialistes, le matériel consommé et le temps de remise en service. Résultats possibles : capture exploitable immédiatement, capture nécessitant des réparations ou un équipage de remplacement, bâtiment immobilisé, objectif partiellement atteint. On distingue aussi les défenseurs neutralisés, capturés, évacués et restés à bord, ce qui donne à la victoire des conséquences politiques, économiques et humaines.

### Le joueur soldat et ses écrans

Le joueur contrôle toujours son personnage, qui commande son escouade sur le terrain. Les vues de commandement (carte, tactique) ne sont pas des modes de jeu : ce sont des écrans que le personnage consulte dans le monde (chapitre 13).

- **Un agent comme les autres.** Aucun pouvoir spécial. Il peut mettre ses propres actions en autopilote (trajets longs), pas ses décisions.
- **Contrôle par décisions fines à pause active** (déplacement, couvert, cible prioritaire, usage d'un objet), la résolution restant celle du modèle tactique, sans visée manuelle. C'est déterministe et testable en headless.
- **Un vrai coût d'opportunité.** Sur le terrain : information rapide, bonus d'exemple (moral, cohésion), risque de mort. À un poste de commande : vue plus large, mais ordres retardés par les liaisons, et la capacité du poste limite ce que l'on peut gérer. Certaines actions exigent la présence et les compétences du personnage (poser une charge, par exemple).
- **L'écran montre ce que sa liaison a reçu** (pistes, incertitude), pas une vue omnisciente.
- **Mort du personnage** : succession par un adjoint ou un protégé, comme au chapitre 8.
- **La proportion change avec le grade** : surtout le terrain pour un soldat, un mélange pour un chef d'escouade, surtout les postes de commande ensuite. Les autres voies (pilote, marchand, pirate) auront leur équivalent, à définir.

### Vues

Le prototype tactique utilise la vue de dessus partout. La vue isométrique du combat au sol est tranchée plus tard : la simulation étant découplée du rendu, ce choix ne l'affecte pas, et l'isométrique coûte cher en art (directions, occlusion). Le zoom permet d'inspecter un vaisseau, ses modules et son équipage sans basculer vers un simulateur de pilotage.

### Paliers de développement du combat

1. **C1 · Combat au sol agrégé** : ordres, déplacement, détection, pertes, moral, reddition, contrôle d'objectifs. Figer d'abord ordres, moral et reddition : ce sont eux qui font d'un combat une opération militaire et pas une comparaison de dégâts.
2. **C2 · Carte tactique locale** en vue de dessus : couverts, lignes de vue, obstacles, escouades, exécution des ordres.
3. **C3 · Persistance des combattants** : blessures, compétences, pertes de spécialistes, récupération, remplacements ; le joueur possède ou relâche son personnage.
4. **C4 · Combat spatial** : détection, engagement, interception et saturation, avec stocks et capacité de traitement.
5. **C5 · Abordage abstrait** (graphe de compartiments, portes, brèches, conditions de capture), puis détail local à la demande.
6. **C6 · Conséquences stratégiques** : équipage manquant, vaisseau capturé mais indisponible, réparations, prisonniers, matériel consommé.

**Vérification.** Des bots de doctrine jouent des centaines de combats en headless pour vérifier que les doctrines produisent des choix et des contre-choix (saturation contre défense en profondeur, missiles contre intercepteurs, guerre électronique contre qualité des pistes) et qu'aucune stratégie unique n'est systématiquement optimale. Chaque palier doit prouver qu'une même opération peut être résolue à plusieurs échelles, que les ordres produisent des résultats compréhensibles et que les conséquences persistent après le combat.

## 12. Graphismes et pipeline d'assets

Le rendu est au service de la décision et le budget va à la simulation. Les tuiles (tilesets) servent là où l'espace est discret, et pas ailleurs.

**Où utiliser des tuiles**

- **Combat au sol** : terrain, bâtiments, couverts, étages.
- **Intérieurs de vaisseaux et de stations** : un plan est un assemblage de tuiles (sols, murs, portes, modules), généré depuis le graphe de compartiments et l'état de l'objet, sans plan dessiné à la main.
- **Autotiling** : murs et jonctions choisis automatiquement selon les voisins, pour réduire fortement la quantité d'art à produire.

**Où ne pas en utiliser**

- **Espace** : l'espace est continu, donc des sprites (vaisseaux, drones, stations) sur fond de carte, avec un rendu vectoriel simple pour les trajectoires, zones de menace et pistes.
- **Carte stratégique et interface** : pictogrammes et vectoriel.

**Principes**

1. **Un style par culture ou faction, obtenu par variation de palette et de modules**, plutôt que des tilesets entiers par faction.
2. **Vue de dessus d'abord.** L'isométrique demande plus de variantes et la gestion des hauteurs ; la simulation étant découplée du rendu, il peut être testé plus tard.
3. **Pipeline piloté par les données** : un tileset est une table (tuile, propriétés fonctionnelles, règles d'assemblage), comme les recettes de production. Les propriétés (couvert, obstacle, porte) viennent de la simulation et l'image n'en est que l'apparence.
4. **Le niveau de détail graphique suit celui de la simulation** : agrégé sur la carte, tuiles et sprites seulement quand on ouvre une situation tactique.

**Cohérence de la génération : structure, peau, marqueurs (v7.3)**

La génération d'un plan (intérieur, base, terrain bâti) sépare trois couches, consommées par une seule pipeline déterministe : `plan = f(graphe, feuille_de_style, environnement, état, graine)` — reproductible, mise en cache par (empreinte du graphe, version de style, empreinte d'état), conformément à la sauvegarde L0-L3.

1. **Structure** : le graphe de compartiments et ses fonctions, dérivé de l'état de simulation ; neutre, aucune couleur de faction.
2. **Peau** : la **feuille de style du propriétaire** en données versionnées (palette dérivée de la palette maîtresse, matériaux, densité décorative, formes de modules, signes) appliquée comme **poids et contraintes sur le choix de tuiles**, jamais comme un générateur séparé — même logique que la peau des métiers. Les feuilles de style dérivent des cultures et dérivent avec elles (route 6).
3. **Marqueurs d'état** : usure, réparations, appropriations successives — la mémoire visible du lieu.

**Construit ≠ possédé** : un vaisseau ou un bâtiment garde le style de sa naissance (chantier, époque, culture d'origine) ; le propriétaire actuel n'ajoute qu'une couche de marque (livrée, symboles, réaménagements). Un achat d'occasion à une faction rivale se voit. **Un changement de propriétaire ne régénère rien** : la conversion visuelle est un processus d'**événements de reprise** journalisés (repeintures, modules ajoutés, graffiti, réparations hétérogènes), lent et partiel — les murs ont une mémoire.

**Le biome n'a pas de style, il a des contraintes et des usures** : filtre de tuiles possibles en données par biome (pas de bois en atmosphère corrosive), matériaux locaux issus de l'économie (recettes existantes), skin d'usure caractéristique (rouille, sable, givre) sur la couche 3, et jeux d'autotuiles de transition reliant biome et structure quel que soit le style du propriétaire.

Tests : même graine et mêmes données → même plan bit à bit ; aucun plan ne contient de tuile interdite par son biome (validation du lot) ; la conquête ne modifie aucune tuile sans événement de reprise ; vue agrégée et vue en tuiles d'un même lieu ne se contredisent jamais (même source de vérité) ; tout assemblage style A / style B / biome passe les autotuiles sans coupure.

**Grammaire architecturale par culture (v7.4)**

La feuille de style du propriétaire se généralise en **grammaire architecturale en données** : modules autorisés, proportions, symétries, densité d'ornement, placement du vide, règles d'orientation. Dimensions portées par la grammaire : lignes (courbes contre orthogonal), toits, verticalité, vide et plein (cours intérieures contre façades ouvertes), symétrie, répétition, ornement, ouvertures, espace commun. Le générateur consomme la grammaire comme le graphe fonctionnel : aucune branche « culture » dans le code.

- **Culture × biome = dialecte** : la même grammaire s'adapte aux contraintes du climat (toits plats qui prennent des pentes sur monde enneigé, cours qui se ferment sous les vents de sable) ; reconnaissable comme la même famille, adaptée au lieu. Une matrice de règles, pas un tileset par combinaison.
- **L'architecture suit la dérive culturelle (route 6)** : les constructions neuves glissent avec les valeurs, les anciennes persistent (construit ≠ possédé) ; un quartier se lit en stratigraphie, et l'archéologie devient un mode de lecture du journal.
- **Pont vers la simulation, en variables agrégées** : densité, intimité moyenne, espace commun par habitant — consommées par les mécanismes existants (seuil d'intimité des conversations, candidats au point distinct, fréquence des interactions). Jamais de règle de gameplay dédiée.
- **Les courbes au niveau du bâtiment, pas de la tuile** : tuiles 32 px orthogonales ; les courbes vivent dans les empreintes multi-tuiles et les préfabriqués des lieux clés (déjà prévus), complétées par un petit jeu de tuiles d'angle arrondi et par les masses du graphe (bâtiments ronds, rues sinueuses décidées par le générateur de graphe).

**Charte de style pour la génération d'assets par IA**

Première approche : produire les assets 100 % par IA, avec une charte qui verrouille le style. Le plus dur n'est pas la qualité d'une image mais la cohérence sur des centaines d'assets. Deux choix de départ, à corriger si besoin : pixel art sur grille de 32 px ; terrain en vue de dessus stricte, personnages et objets en trois-quarts légèrement surélevé (un personnage en vue strictement de dessus est illisible et mal généré).

Bloc de style, placé au début de chaque requête (en anglais, la plupart des outils suivent mieux les consignes dans cette langue) :

```
2D pixel art game asset, sober industrial science-fiction, realistic lived-in world
(working logistics, no neon, no fantasy). 32x32 pixel tile grid, crisp pixels, no
anti-aliasing, no blur, no gradients, no dithering except subtle texture.
Limited cohesive palette of muted desaturated colors: olive green, slate grey,
rust brown, steel blue, with an amber accent reserved for interactive objects.
Light from top-left, shadows fall bottom-right, three-tone shading per material,
selective 1 px outline in a darker shade of the fill (never pure black).
Terrain seen straight top-down; characters, vehicles and props seen from a slightly
raised three-quarter top-down angle. Semi-realistic proportions (head about 1/5 of
body height), not chibi. Transparent background for sprites.
No text, no logo, no watermark, no signature, no photorealism, no 3D render look,
no glow effects, no existing franchise or character.
```

Palette maîtresse (reprise des maquettes ; trois tons par couleur générés automatiquement) :

| Rôle | Couleur |
| --- | --- |
| Herbe, herbe sombre, arbre | #456b3a, #3f6540, #2a4e2c |
| Terre, bois | #76624a, #8a6a3b |
| Asphalte, sol métal, mur | #5b6168, #3b4148, #808994 |
| Rouille, accent ambre, accent jaune | #8d3b2f, #b88a35, #f0c75e |
| Équipe (bleu, bleu clair) | #365f8c, #9ad0ff |
| Fond | #0f141a |

Suffixes par type d'asset :

- Tuile : seamless tileable 32x32 ground tile, \[matière\].
- Jeu d'autotuiles : autotile transition set between \[A\] and \[B\], 16 pieces, edges align on a 32x32 grid.
- Personnage : character sprite, \[rôle\], standing idle, 4 directions on one sheet, 32x64 px, helmet and weapon on separate layers, same proportions as the reference.
- Portrait : bust portrait 64x64, \[âge\], neutral expression, same face structure as the reference, hair and skin layers on separate colors.

Protocole de cohérence :

1. Générer d'abord une ancre de style : un petit jeu de tuiles, un personnage et un objet.

**Production d'assets : manifeste → prompts (v7.4)**

L'outil de production ne contient aucun prompt en dur ; il **lit les données et émet les prompts** :

- **Entrées** : le manifeste des tuiles de base (id, catégorie, propriétés fonctionnelles, jeux d'autotiles requis — la même table que le pipeline), les grammaires architecturales et feuilles de style des cultures, les contraintes de biome, la présente charte.
- **Sorties** : les prompts versionnés (un par tuile de base, un par surcouche), les scripts de recolorisation, et le lot de validation (10 assets).
- **Variation par paramètres, pas par prompts** : la variation culturelle est produite par **recolorisation programmatique** (palette swap déterministe) et par de petites **surcouches** générées par IA (~20-40 sprites par culture : frises, symboles, tissus, graffitis ; ~10 par biome d'usure). Une nouvelle culture coûte une palette et une trentaine de surcouches, pas un tileset.
- **Porte de validation humaine** : chaque asset passe le lot (cohérence, raccords, lisibilité à 32 px, respect de la palette) avant d'entrer au dépôt ; le prompt est archivé avec la version de l'asset (reproductibilité).
- **L'IA est dans la chaîne de production de contenu, jamais dans le jeu en exécution** (même règle que les répliques).
- **Le manifeste est la source de vérité** : une tuile absente du manifeste n'existe pas pour la génération ; pas d'asset orphelin. Les packs libres CC0 peuvent peupler directement le manifeste ; l'IA ne génère que ce qui manque.
2. La fournir comme image de référence à chaque requête suivante, avec une graine fixe si l'outil le permet.
3. Repasser chaque image à la palette maîtresse et à la grille de 32 px (beaucoup d'outils produisent en haute résolution puis réduisent, ce qui donne des pixels flous), et vérifier que les tuiles se raccordent sans coupure.
4. Tester sur un lot de 10 assets avant de s'engager : 3 tuiles, 1 jeu d'autotuiles, 2 personnages dans 4 directions, 1 portrait à deux âges, 2 objets. Critères : cohérence, raccords, lisibilité à 32 px, respect de la palette.

Points juridiques à vérifier auprès de sources officielles : Steam demande de déclarer le contenu généré par IA livré dans le jeu ; une image purement générée peut ne pas être protégeable par le droit d'auteur aux États-Unis ; il faut éviter de reproduire des personnages existants.

## 13. Le personnage, interface du jeu

Le joueur contrôle toujours son personnage. Il se promène sur les planètes, monte dans les véhicules, y compris les vaisseaux, décolle et voyage vers d'autres planètes. Les autres vues (carte stratégique, écrans tactiques, logistique) sont accessibles par ce personnage, par des écrans situés dans le monde. Le personnage est donc l'interface : le joueur ne peut être qu'à un endroit à la fois, et le reste de l'empire tourne sur ses doctrines et ses délégués.

**Ce que le personnage peut faire**

- Marcher dans des zones locales (colonie, site, intérieur) instanciées de façon déterministe depuis l'état agrégé de la planète. Les planètes ne se parcourent pas en continu : les grands déplacements se font en véhicule, sur rails.
- Monter dans un véhicule ou un vaisseau, le piloter de façon assistée (ordres de manœuvre, pas de pilotage manuel), décoller, entrer dans l'espace local, puis voyager sur rails vers une autre planète.
- Mettre ses propres actions en autopilote pour les trajets longs.
- Utiliser des écrans pour commander, selon ce que son accès et sa liaison permettent.

**Les écrans sont des objets du monde, pas des vues omniscientes**

|  | Terminal portable | Poste fixe (passerelle, centre de commandement) |
| --- | --- | --- |
| Disponibilité | partout | seulement sur place |
| Capacité | faible : quelques pistes, ordres simples | élevée, améliorable comme un module |
| Liaison | selon l'équipement, fort retard | dédiée, retard minimal selon la distance physique |
| Écrans | carte stratégique sommaire | carte, tactique, logistique, abordage |

- Un écran affiche ce que sa liaison a reçu, avec sa date d'observation et l'incertitude de l'état actuel estimé. Si l'action est proche, l'écran est en temps réel ; si elle est lointaine, il est retardé de la distance.
- L'accès dépend du grade, du matériel et de la présence. Le centre de commandement du chapitre 11 est un lieu, relié à la chaîne de production.
- Le monde continue quand le joueur ouvre un écran. La pause active reste possible, ce qui rend les écrans jouables sans ralentir le reste.

**Temps et vitesse**

- À pied ou aux commandes d'une bulle active : 1×. En transit : accélération automatique, avec retour à 1× pour une alerte à forte valeur d'intervention.
- Une même simulation, trois bulles (surface, orbite, système) : le véhicule instancié est le même objet que dans l'état agrégé, et il y retourne quand on s'éloigne.

**Durée de vie, voyages et succession**

- **Voyages.** Les trajets courants restent de quelques jours à quelques semaines (rails, bulles locales). Les voyages de plusieurs années sont réservés aux expéditions exceptionnelles. En transit, le temps s'accélère automatiquement, des événements de bord (panne, rencontre, tension d'équipage, décision) ramènent à 1×, et les ordres à bord sont immédiats alors que les ordres vers l'empire arrivent avec le retard de la liaison. Les doctrines préparées avant le départ portent l'essentiel.
- **Longévité graduée.** Espérance de vie de base d'environ 80 ans. Un rajeunissement ou un vieillissement ralenti (150 à 250 ans, valeurs à calibrer) existe, dont l'accès dépend du grade et de la richesse, pour le joueur comme pour les PNJ. C'est un enjeu politique (inégalités, légitimité) et il faut que les PNJ longévifs puissent toujours être renversés, pour éviter la sclérose (chapitre 4).
- **Mort réelle.** Elle reste possible (combat, accident, maladie). La succession est un filet de sécurité, préparée par un mentorat : le joueur choisit et forme un protégé pendant la partie, ce qui transfère l'investissement avant la mort. Critère de validité : la succession n'est retenue que si ce qu'elle apporte en gameplay (renouvellement, crises de transmission, jeu de lignée) dépasse ce que le joueur perd en investissement dans son personnage.
- **Mentorat jouable et succession désirable (v4)**. La succession n'est pas qu'un filet de sécurité : le joueur peut la vouloir. Trois leviers :
  - **Héritier jouable par intermittence** : pendant la formation, le joueur peut, s'il le veut, incarner son protégé lors de certaines missions et responsabilités qu'il lui confie (commander une escouade, gérer une station, mener une négociation). Jouer l'héritier accélère sa formation et crée de l'attachement ; le délaisser est possible, mais le protégé sera alors moins bon et moins lié.
  - **Académie et transmission** : la formation commence tôt (académie, tutorat), et les compétences du personnage du joueur influencent la facilité d'apprentissage de l'héritier, avec l'appui du réseau et de la doctrine de l'organisation. Bien mené, l'héritier est souvent meilleur que le joueur ne l'était au même âge : la succession est une progression, pas une régression. **Plafonnement (v4.1)** : le bonus de formation est borné ; l'héritier ne dépasse pas le niveau du prédécesseur augmenté d'une marge limitée, et les gains ne se cumulent pas indéfiniment d'une génération à l'autre, sans quoi des surhommes en 200 ans rendraient caduques les règles anti-sclérose du chapitre 4 (borne à calibrer en headless).
  - **Héritage et légende** : à la succession, le nouveau personnage hérite du réseau, de la doctrine et d'une partie des compétences ; l'ancien reste conseiller ou symbole (chapitre 8) et sa réputation ouvre des portes que l'héritier n'aurait pas pu ouvrir seul. Bien préparée, la succession donne plus de moyens qu'elle n'en retire : c'est ce qui rend acceptable l'investissement perdu.
  - **Dimensions de l'héritage (v6)** : la succession ne transmet pas un bonus global mais des dimensions distinctes, chacune selon sa règle : compétences personnelles (en partie, sous le plafond déjà décidé), savoirs documentés (chapitre 16), réseau relationnel (selon la confiance de chacun, pas automatique), autorité organisationnelle (selon les institutions, chapitre 4), légitimité (selon la reconnaissance), réputation et mémoire collective, patrimoine (selon le droit).
- **Retraite volontaire.** Le joueur peut passer la main au moment de son choix ; l'ancien personnage reste conseiller (chapitre 8).
- **Cryostase.** Option réservée aux trajets, à condition que les PNJ y aient accès au même coût (équipement rare, risque d'échec).
- **Transfert de conscience dans un corps robotique.** Couche tardive, optionnelle, hors du socle. Si elle est ajoutée : une seule instance active, sauvegarde datée (on perd ce qui s'est passé depuis), coût élevé, accès symétrique pour les PNJ, et lien avec les drones et exosquelettes.

**Calibrage initial**

Valeurs de départ, à ajuster par simulation headless puis par tests de jeu. Cible : 300 ans de jeu, fin non obligatoire (mode bac à sable).

| Phase | Années de jeu | Part du temps réel (150 à 200 h) |
| --- | --- | --- |
| Début (individu, soldat, première équipe) | 0 à 15 | ≈ 8 % |
| Milieu (grades, sous-faction, première faction) | 15 à 60 | ≈ 22 % |
| Late game (faction, empire) | 60 à 300 | ≈ 70 % |

La vitesse moyenne est d'environ 30 à 50 minutes réelles par année de jeu, soit quelques secondes par jour de jeu, compatible avec la chaîne du chapitre 10.

| Paramètre de durée de vie | Valeur de départ |
| --- | --- |
| Âge de départ du joueur | 20 ans |
| Espérance de base | 80 ans, mortalité naturelle croissante à partir de 55 ans |
| Niveau 1 (vieillissement ralenti) | environ 140 ans, accessible au grade intermédiaire (année 20 à 25) |
| Niveau 2 (rajeunissement périodique) | environ 220 à 250 ans, accessible en late game (année 45 et au-delà) |
| Part des PNJ concernés | niveau 1 : environ 10 % de l'élite ; niveau 2 : moins de 1 % |
| Coût | part élevée du budget, seuils de grade et de richesse |

Le personnage du joueur couvre ainsi environ 200 ans de partie : une seule succession est typique, deux en cas de mort accidentelle.

- **Mort accidentelle :** 1 à 3 % par an en début de partie, 0,1 à 0,3 % par an en late game (poste de commande plus fréquent). Sur 200 ans, la probabilité cumulée est de l'ordre d'un tiers : la succession reste un vrai filet de sécurité.
- **Mentorat :** protégé choisi vers l'année 30 à 40, avec 10 à 20 ans de formation avant la succession attendue.
- **Validité de la succession :** au moins 60 à 70 % de l'investissement transféré (réseau, doctrine, compétences en partie). En dessous, elle est vécue comme une perte. Seuil à tester en jeu, pas par bots.
- **Sclérose des élites longévives :** pas plus de 15 % des postes de pouvoir tenus par des PNJ de niveau 2 ; la légitimité s'use avec la durée au pouvoir, ce qui garde le renversement possible (chapitre 4).
- **Voyages :** trajets courants de 2 à 30 jours ; interstellaire de 1 à 10 ans ; expéditions de plus de 10 ans exceptionnelles (au plus une toutes les 20 à 30 ans de jeu pour le joueur). En transit : environ 4 à 8 minutes réelles par année de trajet, avec un événement de bord tous les 2 à 4 mois de jeu en moyenne. Retard de liaison : distance divisée par la vitesse de la lumière.
- **Validation par bots en headless :** sur 300 ans, mesurer le nombre de successions et l'âge à la mort, la part de pouvoir tenue par les longévifs, la répartition du temps réel par phase et le nombre d'événements de bord par trajet.

**Planètes, déplacements et génération**

**Pourquoi se poser.** Test à appliquer : un écran ne pourrait-il pas faire la même chose ? Si oui, se poser est du décor. La présence apporte quelque chose d'unique dans cinq cas :

- prospection et extraction (trouver et évaluer un site, installer une mine) ;
- population et politique (voir les cohortes, les griefs et les gouverneurs, avec des bonus de confiance et de loyauté qu'une télécommande ne donne pas) ;
- conflit au sol (conquête, sabotage, infiltration) ;
- infrastructure (rails, lanceurs, postes de commande) ;
- information sans retard sur l'état réel d'un lieu.

Les lieux d'intérêt viennent du modèle agrégé, jamais du hasard : une mine existe parce que la simulation dit qu'il y en a une, une manifestation apparaît parce que les griefs sont élevés. La zone locale est une vue du monde simulé.

**Modes de déplacement.** Chaque mode a une niche et une limite.

| Mode | Niche | Limite à imposer |
| --- | --- | --- |
| À pied | zone locale | lent |
| Véhicule terrestre libre | région, hors réseau (prospection, convois, embuscades) | plus lent que le rail sur longue distance, carburant, terrain |
| Aéronef atmosphérique | région, repérage, insertion | détection et défense anti-aérienne, météo, carburant |
| Rail (train sous vide) | entre hubs | réseau fixe, horaires, sabotage possible |
| Suborbital | entre continents | coût élevé, vaisseau et zone d'atterrissage requis |

Le rail est rapide mais contraint, donc il ne remplace pas le véhicule libre, seul à aller hors réseau. Tout vol est détectable par la simulation (le joueur n'a pas de pouvoir que les PNJ n'ont pas). Le véhicule du joueur est le même objet que dans la logistique (règle d'identité) : un convoi peut être conduit par le joueur ou rester agrégé.

**Niveau de détail selon la vitesse.** À pied, une case fait environ 1 m ; à 100 km/h on traverse l'écran en une seconde, donc le détail du terrain suit la vitesse.

| Niveau | Contexte | Résolution du terrain |
| --- | --- | --- |
| L0 | à pied | environ 1 m |
| L1 | véhicule terrestre | environ 8 m |
| L2 | vol bas | environ 64 m |
| L3 | rail, suborbital | pas de terrain, seulement route et événements |

**Génération et sauvegarde.**

- **Ne jamais sauvegarder ce qui peut se régénérer.** Le terrain est une fonction pure de la graine, des coordonnées et du niveau de détail. La sauvegarde contient la graine, la version du générateur et les écarts : ce que la simulation ou le joueur a modifié.
- **Le temps de calcul disponible ne change jamais le résultat**, sinon le déterminisme est cassé. Il décide seulement de quand une zone est prête (rayon de préchargement, ordre de génération). Si elle ne l'est pas, on affiche un niveau grossier, ou le véhicule ralentit pour une raison en jeu (terrain accidenté, capteurs limités).
- **Déterminisme et versions.** Le générateur donne des résultats identiques sur toutes les machines (entiers ou flottants contrôlés) et sa version est inscrite dans la sauvegarde, car en changer déplacerait tout ce qui a été posé dessus. Un test d'intégration compare des empreintes de zones entre machines.
- **Du grossier au fin.** Les niveaux grossiers contraignent les niveaux fins, jamais l'inverse. Les modifications remontent par l'état agrégé : une mine vide un pourcentage de la région, et le détail fin se déduit de ce pourcentage et de la graine.
- **Ce qui persiste.** Les objets réellement posés (bases, épaves, cratères) sont des entités de la simulation, pas des cases modifiées : la taille dépend du nombre d'entités, pas de la surface. Les écarts anciens et non visités sont repliés dans l'état agrégé (compaction), avec un plafond de mémoire.
- **Ordre de grandeur de la taille.** Un morceau de 64×64 cases modifié coûte de 0,2 à 8 Ko compressés. Avec 5 000 à 50 000 morceaux modifiés sur une longue partie, on reste à quelques ou quelques dizaines de Mo (à vérifier). Les zones générées vont dans un cache disque facultatif et jetable, hors sauvegarde.
- **Zone locale.** Rayon de 500 m à 1 km en L0 (environ un million de cases), généré par morceaux chargés en flux.
- **Monde mixte.** Terrain procédural, avec des modules préfabriqués à la main pour les lieux clés (intérieurs, bases), comme pour les plans générés depuis les modules.

**Personnages : attachement et immersion**

L'attachement vient de ce que le joueur a vécu avec un personnage et de ce qu'il peut perdre, bien plus que du rendu graphique. Les leviers sont d'abord dans la simulation.

- **Mémoire par événements saillants.** Chaque personnage nommé garde les événements qui l'ont marqué (blessé, sauvé, promu, trompé, perte d'un proche), avec un poids. On ne stocke que des références d'événements du journal causal, le texte est généré à la demande. Plafond d'une vingtaine de souvenirs par personnage, les moins saillants s'effaçant, ce qui borne la taille des sauvegardes.
- **Naissance par les actes.** Les foules restent des cohortes anonymes (règle d'identité). Un individu devient nommé quand il fait quelque chose de marquant (survit à un assaut, sauve le joueur, désobéit pour de bonnes raisons). L'attachement est gagné, et il ne se dilue pas dans trop de noms.
- **Personnalité lisible.** Le vecteur de valeurs et les loyautés du chapitre 3 se voient dans le comportement : le joueur doit pouvoir deviner ce que fera quelqu'un.
- **Relations entre personnages** (amitiés, rivalités, couples, mentorat) : une perte a des effets sur les autres.
- **Conséquences réelles.** Mort permanente et blessures durables (niveau de détail à décider), croissance visible, rancune et confiance qui laissent une trace.
- **Rendu à peu de frais.** État visible sur le sprite (blessure, fatigue, équipement), silhouettes distinctes, portraits modulaires qui vieillissent avec la longévité, répliques courtes générées à partir de l'état (peu de modèles, bien écrits, pour éviter la répétition), journal et lettres.
- **Immersion.** Interface dans le monde, informations limitées et datées, moments ordinaires (repas, repos, trajets), son et ambiance.
- **Selon la phase.** Début : l'escouade (c'est là que l'attachement individuel est le plus fort, à la manière de Band of Brothers). Milieu : les protégés et délégués. Late game : l'héritier, les conseillers, l'organisation.
- **Ordre de construction.** Identité et traits lisibles, mémoire saillante, état visible sur le sprite, répliques et journal générés, relations, puis portraits modulaires avec vieillissement. Les trois premières couches suffisent pour le palier du soldat.

**Risques à traiter**

- **Voyages de plusieurs années.** Traités plus haut : accélération en transit, événements de bord et commandement à distance, avec des distances courantes courtes. Risque restant : l'ennui des longs transits, d'où l'importance des événements de bord.
- **Mort du personnage** : succession par mentorat (chapitre 8), avec la règle de validité de la sous-partie « Durée de vie, voyages et succession ».
- **Quantité de contenu** : les intérieurs praticables de chaque véhicule ou station coûtent cher. Plans générés depuis les modules, et une seule pièce (le poste de pilotage) pour les petits engins.
- **Coût de simulation** : seule la bulle autour du personnage est instanciée, le reste est agrégé (chapitre 9).

## 14. Le premier palier : le quartier vivant

Le jeu se développe et se joue par paliers de dézoom : chaque niveau de dézoom doit être jouable avant d'ouvrir le suivant, conformément à la règle « chaque couche jouable seule ». Le premier palier est un quartier miséreux d'une ville ; l'objectif du joueur est de s'extraire de sa misère. La simulation des besoins est le moteur de ce palier, et c'est l'instanciation à l'échelle individuelle des variables du chapitre 2, pas un système séparé.

**Les besoins triviaux comme moteur**

- 4 besoins fondamentaux de départ : nourriture, logement, santé, sécurité. La **reconnaissance** (v6) n'est pas un besoin matériel mais une variable sociale séparée : position relative dans un groupe de référence borné, mise à jour sur événements (promotion, déclassement, honneur, affront). Elle pèse sur les motivations (poids dans Q(a)) plus que sur les griefs, pour éviter le double comptage avec le chapitre 2.
- Chaque individu est un échantillonnage déterministe de sa cohorte : mêmes variables (u, p, q, G), coefficients individuels tirés par clé stable. Les griefs individuels suivent la même équation que le chapitre 2.
- Les besoins consomment du temps, pas seulement de l'argent : survivre prend du temps, ce temps n'est pas disponible pour travailler ou se former. Le temps est la ressource rare, sans jauges visibles ; le personnage vit « je n'ai pas le temps », pas « fatigue à 70 % ».
- **Salaire réel** : la variable ménage n'est pas p seule mais le salaire nominal divisé par le coût du panier de besoins (p en est une composante). Quand le salaire réel ne suffit plus, l'offre de travail de la cohorte baisse, ce qui pousse les salaires ; les firmes (chapitre 6) contiennent les salaires via leur table d'actions (Q(a), chapitre 3). La grève existe déjà comme basculement (chapitre 2) : la boucle salaires ↔ griefs ↔ grèves est complète et émergente, sans sous-système de négociation.
- L'offre de travail est agrégée par cohorte (heures disponibles, salaire réel cible), pas par individu, sauf dans la bulle du joueur.
- **États déclenchés, jauges cachées** : cinq variables individuelles (faim, fatigue, stress, ennui, douleur), dérivées des variables existantes (fatigue ← budget d'éveil ; stress ← griefs, dette, ennui, douleur ; ennui ← déficit de loisir), jamais affichées en nombre. Les seuils en données produisent des états qualitatifs journalisés ; un état critique préempte l'emploi du temps, sinon la doctrine pondère Q(a). Le stress est le collecteur unique de santé mentale. L'ennui pousse au loisir (économie du loisir : étuve, restaurant, occasions sociales) et, agrégé par cohorte, nourrit l'attractivité du lieu (salaire réel, loisir, sécurité, griefs bas), consommée plus tard par les migrations (routes 10 et 11) et la dérive culturelle (route 6).
- **Le luxe est le temps libéré** : chaque bien et service porte (coût monétaire, coût temporel) ; la richesse convertit l'argent en temps, la pauvreté se paie en temps. La part du budget de temps consacrée à la survie décroît avec la richesse (environ 20 % de temps libre pour le miséreux, 70 % et plus pour le notable) : le contraste du trivial de début au palais de fin est la courbe émotionnelle des 300 ans. Le serviteur du notable est le journalier du début : la maisonnée est une firme (recettes, employés, pratiques), le luxe des uns crée les emplois des autres.
- **Diversité des lieux** : un type de lieu n'entre que s'il occupe un point distinct de l'espace (coût monétaire, coût temporel, valeur sociale) et joue un rôle dans une boucle (métier, besoin, occasion sociale) ; deux lieux au même point fusionnent en données. La diversité croît avec la richesse du lieu, par types en données, jamais semée pour l'esthétique.
- **Créneaux, pas horloge** : le jour du quartier porte quatre créneaux (matin, après-midi, soir, nuit), de simples étiquettes sur les demi-heures ; postes, lieux et emplois du temps les déclarent en données. Le couplage (fréquentation par recouvrement, primes de créneau, équipes tournantes, lits chauds) entre en P1.5 ; le rythme du quartier est une émergence de la distribution des créneaux, jamais une règle.
- **La ville amortit, elle ne neutralise pas** : le marché de la ville hors quartier porte une courbe d'offre en données (prix de référence montant avec le volume demandé) et des délais logistiques ; un choc local produit des hausses de prix et des ruptures temporaires, amorties avec retard. Deux autres protections de validité du cœur P1 : les enfants contributeurs (participation domestique progressive dès 8 ans, corvées seulement, jamais d'emploi salarié) et le plancher anti-impasse (quatre voies assemblées : entraide, servitude, maisonnée, sous-location précaire ; propriété testée en headless : taux de sortie de misère post-faillite strictement positif, jamais une règle dédiée).
- **Fiscalité de P1, forfaitaire** : prélèvement à la source et allocations fixes, sans acteur fiscal ; toutes les grandeurs s'expriment en net. La politique fiscale (taux vivants, incitations, conditionnalité) entre quand l'État devient une organisation jouable (P2), les incitations au palier réseau (P3) ; le consentement à l'impôt sera une source de griefs et de légitimité.
- **Vivacité des PNJ** : hétérogénéité par clé stable (doctrine, seuils, préférences), habitudes persistantes et visibles, mémoire active qui biaise Q(a), variance quotidienne causale, répliques générées depuis le journal, groupes informels de 3 à 6, rituels sociaux sur les événements de vie, expression qualitative des états. Le quartier vit parce que chaque chose a une cause visible, pas parce qu'il y a du bruit. L'échelle d'interaction avec les PNJ est symétrique : masse, témoins, transactions, conversations, relations, intime, commandement.
- **Empathie et souffrance d'autrui** : l'empathie est un trait individuel et un canal de contagion (souffrance perçue = empathie × affinité × souffrance visible, sur l'ensemble de conscience borné par la mémoire) ; elle alimente le stress puis les griefs de cohorte par les groupes, sans second registre. L'angoisse est un état anticipatoire (épargne, stocks, tendance) ; la dépression un état persistant du stress, rare et réversible par causes ; la dissonance morale (efficacité contre doctrine) mène à révision de doctrine ou transgression.
- **Traits du décideur** : la table Q(a) d'une organisation est modulée par les traits de son décideur nommé (l'empathie d'abord) : patience, tolérance, dissonance du dirigeant, contrainte de pérénité dure ; la rétention des vétérans fait la compétitivité par la transmission (chapitre 16) ; deux régimes de gestion coexistent, aucun ne domine. **L'engagement des employés ferme la boucle** : dérivé des décisions observées de la firme, il pondère leurs décisions d'arrêt maladie, de soin et de démission, et agit par la prévention (moins d'incidents, usure ralentie, options de résolution dans les décisions ponctuelles) ; jamais de temps de travail supplémentaire ni de multiplicateur de recette, la productivité supérieure étant une mesure émergente ; la contagion de maladie à l'atelier fait du présentéisme imposé une pathologie émergente.
- **Valuation des rôles** : la reconnaissance et les salaires cibles sont pondérés par des curseurs de valeurs en données (réussite contre utilité, héritage contre mérite, statut du soin, temps), jamais par le rang de revenu codé en dur ; l'écart de valuation (salaires contre utilités visibles au journal) est une mesure ; les vocations suivent la reconnaissance, avec correction partielle par pénurie.

**Boucle de progression du quartier**

travailler → acquérir des compétences → promotion → meilleur salaire réel → meilleur logement et conditions → besoins mieux satisfaits → temps libéré → plus efficace au travail et plus apte à acquérir des compétences.

Deux garde-fous bornent ce cercle vertueux, sans règle artificielle :

- les prix locaux montent avec la demande (mieux se loger coûte plus cher parce que d'autres le veulent aussi) : le panier s'endort chèrement avec le niveau de vie ;
- rendements décroissants sur chaque usage du temps : pas de bonus multiplicatifs abstraits.

La boucle est symétrique : un PNJ peut la parcourir avec les mêmes leviers, et le taux de sortie de misère du joueur doit être identique à celui d'un PNJ à investissement égal.

**Doctrine personnelle et immersion sans corvée**

Le personnage du joueur, quand il n'est pas dirigé finement, satisfait ses besoins de lui-même selon sa doctrine personnelle : une petite table de poids (le même format que les motivations du chapitre 3), éditée via son terminal portable branché sur une console (chapitre 15). C'est le même mécanisme d'autopilote que le chapitre 13 et la même règle de décision Q(a) que tous les agents : aucune jauges obligatoire, seulement des choix de profondeur. Diriger finement son personnage est possible (décisions fines à pause active), jamais requis.

**Valeurs de départ, à calibrer**

| Paramètre | Valeur de départ |
| --- | --- |
| Individus du quartier | 150 (cohorte + nommés extraits par leurs actes) |
| Besoins | 4 fondamentaux (nourriture, logement, santé, sécurité) + reconnaissance |
| Latence maximale des décisions fines | quelques secondes par jour de jeu |
| Premier achat structurant | console fixe (la portable ne donne que la planification basique) |
| Réévaluation de doctrine | sur événement ou seuil, comme la stabilité des plans (chapitre 3) |

**Critères de réussite en headless** : distribution de richesse et de salaires plausibles sur 10 ans ; des PNJ s'extraient et d'autres s'enfoncent ; taux de sortie de misère du joueur = celui d'un PNJ à investissement égal ; fréquence des grèves corrélée au salaire réel ; la misère résiduelle vient de la structure des prix, pas d'une trappe artificielle ; chaque trajectoire s'explique par le journal de causalité. Si le quartier est vivant sans le joueur, chaque dézoom suivant n'est qu'un changement de résolution.

## 15. Consoles, réseaux et liaison

**Règle unique.** Toute communication est un message dont la latence est la distance divisée par la vitesse de la lumière, plus les relais traversés. Il n'y a pas de zones « avec/sans internet » en booléen : la couverture vient d'infrastructures réelles, pour le joueur comme pour les PNJ.

**Consoles : capacité = coût + encombrement.** Une console est un objet produit, transporté et installé (chaîne du chapitre 10), améliorable comme un module.

| Console | Place | Coût relatif | Capacité | Communication autonome |
| --- | --- | --- | --- | --- |
| Portable | poche | 1 | planification basique, quelques pistes | portée locale (≈ un quartier) |
| Fixe (appartement, bureau) | une pièce | 5 | planification complète, écrans de base | locale, plus ou moins loin selon la qualité |
| Véhicule / vaisseau | un poste | 20 | écrans tactiques embarqués | selon la liaison embarquée |
| Poste de commandement | une salle | 100+ | tout | dédiée |

- **Emprunt de capacité** : le terminal portable se connecte à la meilleure console à portée et hérite d'une partie de ses capacités, avec le retard de la liaison. Dans un quartier sans réseau, la portée est celle de la console fixe voisine.
- **Capacité, autorisation et priorité sont séparées (v6)** : la capacité est technique (coût + encombrement) ; l'autorisation est une propriété (grade, propriété du matériel, droits d'accès, chiffrement) ; la priorité est une file (bande et rang d'abonnement). Une console puissante inaccessible ne sert à rien ; une console accédée sans priorité attend son tour. Les trois axes sont pilotés par les données et symétriques pour tous les acteurs.
- **Doctrine personnelle et planification** : la portable suffit au quartier pour la planification basique ; la console fixe de l'appartement débloque la planification complète du quartier (chapitre 14). À trancher : jusqu'où la portable connectée à une console fixe hérite-t-elle à distance ?

**Réseaux : des infrastructures, pas des zones.**

- **Constellations de satellites** : la couverture planétaire vient de constellations, entités produites et possédées (le secteur privé du chapitre 6 peut en être propriétaire), destructibles, brouillables (guerre électronique du chapitre 11). Une constellation bloquée par un conflit crée un blackout régional ; la censure et le contrôle des opérateurs deviennent des enjeux.
- **Relais interplanétaires** : latence de quelques minutes à quelques heures selon la distance.
- **Interstellaire** : store-and-forward, latence de la distance divisée par c, en années. Aucun temps réel, jamais.

**Abonnements = allocation de capacité, pas d'accès instantané.** Un abonnement (planète, système, univers) est un contrat comme un contrat de transport : bande garantie et priorité sur les relais d'une zone. L'abonnement « univers » garantit de la bande sur les relais jusqu'au système d'origine, pas du temps réel à distance. Les abonnements sont accessibles aux PNJ au même coût, et le marché des relais est une application du chapitre 6.

| Zone | Latence typique | Valeur de départ |
| --- | --- | --- |
| Quartier (portée console) | quasi nulle | < 0,1 s |
| Planète avec constellation | petite | < 1 s |
| Système (relais) | minutes à heures | distance/c + relais |
| Interstellaire | années | distance/c |

**Coût de simulation.** Les messages sont des entités avec des événements d'arrivée ; les consoles sont des entités produites ; les constellations des nœuds de couverture. Rien de continu, tout événementiel, compatible avec la file d'événements et le late game. Test headless : un message émis arrive avec la latence exacte attendue sur toute la grille des distances ; un blackout de constellation coupe bien toutes les liaisons qui la traversent et aucune autre.



## 16. Savoir et pratiques : pas d'arbre de recherche

Le savoir n'est pas une liste déroulante débloquée par des points : c'est une chose possédée par des gens et des organisations. Il n'y a pas d'arbre de recherche ; il y a des pratiques, avec des porteurs, des niveaux de maîtrise et des canaux de transmission. Symétrie totale joueur/PNJ, déterminisme total, diffusion inégale dans un univers sans FTL.

**Ontologie**

| Type | Exemple | Portée par | Effet |
| --- | --- | --- | --- |
| Recette | raffinage 3 minerai → 1 alliage | un site + une cohorte qui l'exécute | débloque un flux |
| Technique | forage en veine profonde, pilotage de drone | une cohorte, une escouade | améliore le rendement d'une action |
| Doctrine | saturation par missiles | une organisation, une armée | change les poids des actions dans Q(a) |

- La maîtrise (0 à 10) est attachée au couple (pratique, porteur), jamais au monde : le savoir est local. Une station perdue peut coûter des années de savoir-faire.
- **Immersion par les noms** : les pratiques sont définies en données avec un nom propre ; le niveau de maîtrise est interne et l'interface n'affiche que du qualitatif (« maîtrise confirmée », « encore approximative »). Les variantes notables portent le nom de leur praticien, extrait de sa cohorte par ses actes (chapitre 13). La technologie a des auteurs.

**Transmission, du plus lent au plus rapide**

1. Apprentissage sur le tas, borné par les vétérans disponibles (1 vétéran forme 3 élèves au plus).
2. Documentation : écrite au-dessus d'un seuil de maîtrise, transmise comme un message (latence distance/c, bande consommée), plafond de maîtrise inférieur à l'expérience pratique.
3. Embauche et migration des porteurs.
4. Enseignement institutionnel : académies, réutilisant le mécanisme du mentorat (chapitre 13).
5. Espionnage et reverse engineering, reliés à l'abordage (chapitre 11).

**Innovation, sans tirage exogène**

- **Courbe d'apprentissage** : chaque exécution a une probabilité d'incrément (seuil de risque cumulé du chapitre 2, clé stable). Les régions qui raffinent beaucoup raffinent mieux : le progrès est la conséquence de l'activité réelle.
- **Combinaisons** : des pratiques nouvelles sont définies en données comme des assemblages (maîtriser A niveau 4 et B niveau 3 débloque la recette C). Piloté par les tables, pas par une IA de conception.
- **Ruptures** : une combinaison de second ordre, un praticien exceptionnel (profil hors norme par clé stable, qui devient nommé), ou une diffusion retardée qui paraît une rupture vue de loin. Jamais de « eureka » inexpliqué : toute rupture est traçable dans le journal de causalité.
- **Perte de savoir** : une cohorte dispersée, une académie fermée, un site détruit fait retomber la maîtrise ; la documentation en protège partiellement. Des âges sombres locaux peuvent émerger sur 300 ans.

**Orientation du progrès : les doctrines et le régime**

Le joueur valorise le savoir dans ses doctrines (poids w). Le canal d'influence dépend du régime, via κ (chapitre 3) :

- **Économie de marché** (κ bas) : les firmes suivent leurs poids locaux ; le joueur agit indirectement, par subventions, commandes, crédits de manque à gagner (actions de la table des firmes).
- **Économie dirigée** (κ haut) : les poids effectifs collent aux priorités du régime ; action directe, au prix d'un coût d'équilibrage des ressources des firmes.

**Recherche publique** : académie, observatoire, mission scientifique sont des organisations avec budget, doctrine et table d'actions, comme une firme. Leur produit est le savoir (maîtrise, documentation, praticiens formés). Les PNJ et les puissances rivales y ont accès au même titre.

**Délégation et budget d'attention.** Le savoir n'est pas une liste à gérer : les postes compétents (conservateur des savoirs, responsable d'académie) sont tenus par des personnages nommés avec leurs poids et leurs griefs (chapitre 3), qui remontent les décisions profondes selon la valeur d'intervention (chapitre 1) et gèrent le reste en autonomie. Le joueur apprend la délégation par inversion au quartier : son patron lui délègue avant qu'il ne délègue.

**Valeurs de départ, à calibrer**

| Paramètre | Valeur de départ |
| --- | --- |
| Niveaux de maîtrise | 0 à 10 |
| Incrément par exécution | ~1 gain pour 30 exécutions en niveau bas, via seuil cumulé |
| Documentation | écrite dès niveau 4, transmet jusqu'au niveau 3 |
| Formation accélérée | plafonnée : 1 vétéran → 3 élèves |
| Coût mémoire | (pratiques × porteurs actifs), porteurs dormants archivés |

**Critères de réussite en headless** : sur 100 ans, la carte des maîtrises diverge selon l'activité réelle des régions ; un site perdu fait perdre de la maîtrise ; une documentation relève la destination sous son plafond ; aucune région ne maîtrise tout sans dépendance réelle ; aucune stratégie unique (tout-documenter, tout-centraliser, tout-embaucher) ne domine les bots.

**Dans le palier quartier (P1)** : l'apprentissage en faisant est inclus dès le départ, car la boucle de compétences du chapitre 14 en dépend : les compétences montent par l'exécution, plafonnées par les vétérans disponibles, à l'échelle observable de 150 individus.



## 17. Routes approuvées : pistes de profondeur différées

Douze pistes de profondeur sont approuvées comme directions, détaillées dans le document de travail « Pistes de profondeur » ; les routes 10 et 11 (v6.3) forment la boucle de population, la route 12 (v6.5) porte les institutions de solidarité et les groupes de pression. Elles ne sont pas des chapitres du socle : chacune entre au palier indiqué, fermée par ses tests headless, et satisfait les cinq critères d'adoption (dilemme à conséquence retardée, symétrie, coût borné, lisibilité par le journal, testabilité par bots).

1. **Usure et entretien** (version minimale dès P1) : tout actif construit porte un état de service calculé paresseusement (état initial, date, taux), avec franchissements de seuils planifiés comme événements ; l'entretien est une action de l'organisation propriétaire, consommant des composants et de la main-d'œuvre ; la dette d'entretien par région alimente la qualité des services q puis les griefs. Garde-fou contre la saturation du late game : s'étendre vite contre tenir ce qu'on a. Valeurs de départ : usure 1 à 3 % par an, entretien 2 à 4 % du coût de construction par an, part de l'économie en régime établi 10 à 25 %. Test : un bâtisseur qui étend sans consolider entre en crise avant l'an 100 ; un consolideur excessif est dépassé en influence.
2. **Fiabilité des rapports** (avec la politique et les crises) : les sources sont humaines, avec biais, bruit et mensonge selon les attachements P et I et l'intérêt matériel ; la vérification (second avis, audit, inspection, présence) a un coût, un délai et une probabilité de détection ; chaque source a une cote de fiabilité apprise et visible. Le bruit des PNJ secondaires est appliqué au moment de la décision, par clé stable, sans croyance stockée. Le joueur peut falsifier des rapports : même table d'actions, mêmes coûts et mêmes risques que les PNJ, symétrie intégrale.
3. **Engagements crédibles** (avec les factions) : traités, garanties, gages, menaces conditionnelles ; registre bilatéral creux des engagements tenus et rompus (demi-vie 20 à 40 ans), servant de terme de risque dans Q(a) ; réputation propagée comme rumeur, avec retard et atténuation par la distance. Les garanties matérielles et les gages viennent d'abord ; l'otage-personnage est une variante tardive branchée sur le chapitre 13.
4. **Directives** (avec la délégation à plusieurs échelons) : règles de délégation ordonnées (condition sur l'état observé par le délégué, action de la table, niveau d'autonomie, déclencheur de réévaluation, expiration), évaluées par abonnement aux variables qu'elles mentionnent, jamais à chaque tick. Modes d'échec : condition obsolète, conflit entre échelons, ambiguïté d'interprétation selon κ, contexte imprévu. Nommées « directives » pour ne pas les confondre avec les doctrines de motivation du chapitre 3. Modèles de règles prédéfinis, pas de script libre. Ce sont des données versionnées, transmises à l'héritier.
5. **Obsolescence et contre-doctrines** (avec le combat, C1 à C4) : matrice d'efficacité relative en données, mélange de forces par faction, adoption différée de la contre-doctrine par retour d'expérience, coût et délai de reconversion industrielle. Unifiée au chapitre 16 : les doctrines tactiques sont des pratiques de type doctrine, l'adoption est une transmission, la reconversion est un changement de recettes. Aucune doctrine ne doit dépasser environ 60 % de victoires de façon durable.
6. **Dérive culturelle** (extension tardive, optionnelle) : le vecteur de motivations des cohortes évolue lentement, à pas annuel ou sur événement significatif, sous exposition aux événements, amortissement vers la base et influence des institutions via κ, avec plafond par décennie et carte culturelle lisible avec retard. Un régime stable ne doit pas devenir incompatible avec sa population avant 80 à 120 ans, sauf s'il ignore la légitimité. S'y rattachent les **curseurs de valuation** (réussite contre utilité, héritage contre mérite, statut du soin, temps) : ils pondèrent la reconnaissance et les salaires cibles, jamais le rang de revenu codé en dur ; l'utilité visible se calcule depuis le journal, l'écart de valuation (salaires contre utilités) est une mesure ; la boucle valeurs → récompenses → élites financières → académies et lobbies → doctrines transmises → valeurs se ferme par les organes existants ; aucun pôle ne domine durablement ; la valuation est data-driven dès P1 (curseurs fixes), façonnée par les institutions en P2, divergente entre planètes en P3.

   **Consolidation (v7.5) — ce qu'est une culture et comment elle naît.** Une culture est un **paquet de données porté par les cohortes**, jamais par les individus ni par le décor : vecteur de motivations, curseurs de valuation, doctrines transmises, rituels et fêtes, grammaire architecturale et feuille de style (ch. 12, v7.4), registres de langue (à cadrer). Un individu hérite des poids par défaut de ses cohortes (famille, quartier, métier) ; sa doctrine personnelle et ses affinités peuvent s'en écarter, au prix de la dissonance morale. La culture est un **agrégat** au sens de la règle d'identité : la somme pondérée de ses porteurs, jamais une entité qui flotte. **Elle naît par divergence, pas par décret** : deux groupes séparés (autre bassin, autre planète, latence) voient leurs expositions diverger, leurs vecteurs dériver en sens différents, et au-delà d'un seuil de distance culturelle (distance entre vecteurs et curseurs, à calibrer — le jumeau lent du seuil de déclaration de faction) ils constituent deux cultures ; le moment où les intéressés se nomment différents est la déclaration de faction (processus commun, deux vitesses). **Canaux de modification, tous existants** : exposition (événements vécus et racontés), institutions via κ, générations (transmission avec bruit — le moteur lent), migrations (import de poids, mélange), chocs (les scénarios de choc sont aussi des chocs culturels), dissonance morale (convertisseur individuel). **La lecture avec retard est la règle d'or** : personne ne voit un pourcentage de changement, seulement les manifestations datées (stratigraphie architecturale, rituels qui changent de forme, ton des répliques, valuations dans les salaires cibles) ; la carte culturelle est un produit dérivé consultable, jamais une jauge active. Tests : deux cohortes sœurs séparées divergent au-delà du seuil, au journal ; aucun pôle ne domine sur 500 ans ; une institution via κ déplace la culture à la vitesse plafonnée, jamais plus ; une culture importée par migration se mélange ou se maintient selon taille relative et densité de réseau, jamais d'assimilation magique. **Influence mutuelle (v7.7)** : les cultures ne s'influencent jamais entre elles directement — l'influence ne circule que par le **contact individuel** (commerce, migrations, déplacements, conquête, médias), pondéré par le statut perçu : le soft power (diffusion asymétrique des cultures prestigieuses) émerge de la reconnaissance, sans règle dédiée. La dissonance morale peut **séduire ou raidir** : l'exposition à l'étranger dévie vers lui ou raidit vers la base (amortissement renforcé quand l'exposition est vécue comme menace) ; la **pureté culturelle** est une position doctrinale disponible, portée par des institutions (κ contre l'étranger) et des factions possibles — les cycles hybridation → purification → hybridation sortent de la dynamique de boucle. La convergence mutuelle est **en gradient** : rapide sur les couches superficielles (mode, mots, plats), lente sur les profondes (masses, valeurs) ; si les couches profondes convergent, la fusion est réelle. Test : deux cultures en contact convergent partiellement, symétriques sauf écart de prestige, gradient de vitesses respecté, au journal.

   **Extension rendu (v7.6) — origine et dérive du style graphique.** La grammaire architecturale n'est pas un stock indépendant : c'est une **projection du vecteur culturel**, en tables de données (dimension culturelle → paramètre de grammaire, avec inertie par dimension) — symétrie ↔ hiérarchie, ornement ↔ valuation « réussite », cours contre façades ↔ intimité, matériaux durables contre éphémères ↔ valuation « héritage », proéminence du soin et du savoir ↔ statuts correspondants. **Origine** : `grammaire_initiale = f(biome, économie à la fondation, culture mère, technologie disponible)` — la tradition est une contrainte devenue identité (les premières contraintes se figent en style ; quand la contrainte disparaît, le style persiste par inertie et par référence au bâti existant) ; toute origine est racontable au journal. **Dérive à trois vitesses** : ornement et surcouches (années — la mode), matériaux et palettes (décennies — l'économie), masses et architecture (générations — la culture profonde) ; ce gradient rend la stratigraphie datable. **Chocs matériels** : une pénurie longue dévie la projection par substitution de matériaux (les recettes parlent) et la génération qui grandit dedans l'intègre en norme — la boucle contrainte → adaptation → tradition → identité se referme ; une contrainte brève laisse un vestige, rien d'autre. Technique : les bâtiments échantillonnent la grammaire à leur construction (version archivée, un entier dans l'état) ; la palette culturelle dérive par la même projection, appliquée par recolorisation programmatique (outil v7.4 inchangé) ; coût simulation nul, tout est présentation et données dérivées. Tests : projection déterministe bit à bit ; bot archéologue qui date les bâtiments par couches avec erreur bornée par les vitesses ; origine reconstruisible au journal ; choc long → matériau signature changé en ≤ 2 générations, choc bref → vestige ; dialectes culture × biome apparentés (distance de style bornée).
7. **Vol et milice** (entrée P2, avec la propriété et la richesse visible) : voler est une action désespérée et symétrique, disponible pour tout individu (joueur compris) quand la misère ou l'opportunité le justifient dans Q(a), avec détection, sanction et réputation ; la milice est une force embauchée par les firmes et les propriétaires pour protéger les biens, pas un pouvoir public gratuit. Le vol reste rare si P1 est bien calibré : il est le symptôme mesurable d'une misère réelle, donc un indicateur de test.
8. **Réseau de faveurs complet** (après le parrainage de P1) : graphe de relations entre individus nommés, construit par les actes (prêts, parrainages, services, trahisons), chaque lien portant sa nature, sa force et son âge ; les faveurs circulent le long des liens, avec mémoire des dettes. Nécessite l'extraction d'individus par leurs actes (chapitre 3) ; coût borné par le plafond d'individus nommés actifs.
9. **Spéculation et revente individuelle** (avec le marché parallèle) : un individu peut constituer un stock pour le revendre en pénurie, action symétrique évaluée par Q(a), bornée par sa trésorerie et son espace de stockage ; les profiteurs de crise deviennent visibles et nourrissent les griefs collectifs. Différée comme le marché parallèle du chapitre 6, car elle suppose des prix affichés volatils et une asymétrie d'information crédible.
10. **Migrations inter-quartiers** (entrée P2, palier station) : migrer est une action individuelle évaluée annuellement par Q(a), déclenchée par l'écart entre le rang atteignable ailleurs et le rang actuel (mécanisme de reconnaissance, chapitre 14) ; coûts : transport, reconnaissance remise à zéro, perte du réseau local (parrainage, faveurs). Le départ et l'arrivée sont des événements qui réinjectent l'individu dans les cohortes du quartier cible. Le quartier gagne une rétroaction démographique (embourgeoisement ou exode des ambitieux), sans levier direct : personne ne déplace des gens. P1 reste explicitement fermé : la ville hors quartier est un mur d'import/export et de rumeurs, pas de flux de personnes. Valeurs de départ : seuil = écart de rang ≥ 2 quantiles, coût = 10 jours de salaire, cooldown 3 ans, flux net attendu ≤ 2 %/an entre quartiers contrastés. Test : deux quartiers à prix contrastés produisent des flux corrélés au salaire réel, et une spirale locale (si elle survient) est entièrement expliquée par le journal. **Extension candidate (v7.2, à approuver à l'entrée de P2) — interactions inter-quartiers** : les quartiers sont spécialisés (tous n'ont pas tous les services, placement en données, règle du point distinct entre quartiers) et les habitants se déplacent pour travailler, se soigner ou varier (diversité alimentaire et de lieux pesant sur le loisir et le moral). Déplacement = décision Q(a) sur information perçue, coût en temps (créneaux) et monnaie ; flux agrégés par cohorte (règle d'identité), individus nommés détaillés seulement dans le rayon d'interaction du joueur. Valeurs de départ : trajet = 15-30 min + 2-5 % du salaire journalier par frontière de quartier ; seuil de déplacement = gain de reconnaissance ou de diversité ≥ coût. Test headless : deux quartiers dont un avec hôpital et diversité de restauration ; les flux suivent l'écart de services et le salaire réel, le moral des deux quartiers converge partiellement. P1 et P1.5 restent fermés : à un seul quartier instancié, la variété s'achète dans le quartier ou pas du tout (le luxe est la conversion d'argent en temps).
11. **Migrations interplanétaires** (entrée P3, petit réseau) : une ligne de passagers est une firme (navires avec capacité et usure, prix du passage, latence distance/c du chapitre 15) ; les flux sont agrégés par cohorte (âge, pratiques, richesse), évalués annuellement par paire origine-destination, avec extraction d'individus nommés à l'arrivée. La décision individuelle se prend sur des informations perçues, rumeurs comprises, parfois fausses (route 2) ; le passage se paie cash ou par crédit remboursé en servitude (spéc P1, 6.8). La pression foncière et la qualité de vie deviennent les moteurs de la colonisation ; l'émigration est une soupape politique qui saigne aussi les porteurs de savoir (chapitre 16). Garde-fous contre l'homogénéisation : frictions fortes (latence en années, coût, attachement, perte de reconnaissance et de réseau) et capacité des navires plafonnée ; la convergence des loyers réels doit prendre des décennies. Interdépendance à spécifier : recrutement militaire (chapitre 10) et immigration pèsent sur les mêmes cohortes. Valeurs de départ : passage = 6 mois de salaire ou indenture 3 ans, capacité 500 à 5 000 passagers, flux total ≤ 1 %/an d'une planète, jeunes (14-35 ans) ×2, attachement I et liens de faveurs divisent la probabilité de départ ; l'attractivité (salaire réel, loisir, sécurité, griefs bas), enregistrée par cohorte dès P1, pèse comme facteur d'arrivée. Tests : convergence lente entre planètes contrastées ; fausse rumeur d'Eldorado → vague, correction, griefs traçables contre l'émetteur ; invariant I1 étendu : tout transfert de cohorte conserve strictement effectif, monnaie, griefs, attachement.
12. **Institutions de solidarité et groupes de pression** (entrée P2, avec la politique) : les systèmes de solidarité (assurance maladie privée ou publique universelle, assurance chômage, retraites) sont des institutions en données (financement, couverture, prestataire, qualité) qui redistribuent la variance des chocs déjà simulés ; aucune pathologie n'est inscrite par idéologie : la sélection adverse, l'aléa moral, les files d'attente et la ruine médicale émergent des mécaniques de mutualisation. Les groupes de pression sont des organisations comme les autres : le capital dépense de la monnaie, la société civile des heures militantes (le coût existe des deux côtés) ; l'influence est plafonnée par décennie et sa transparence dépend de la fiabilité des rapports (route 2) ; le joueur peut financer l'un ou l'autre camp, symétrie intégrale. Les externalités (pollution, dérèglement climatique) sont des transferts de coût non consentis qui atterrissent sur la qualité des services, la santé et les paramètres du biome (le prix de l'eau indexé sur la rareté en est le thermomètre) ; « acheter la mise à l'abri » déplace ces coûts, et la révolte reste la conséquence non achetable : un état capturé perd en légitimité ce qu'il gagne en recettes, les basculements finissent par parler. En P1, le régime « rien » (dispensaire privé payé à l'acte) est le régime par défaut, et son coût est le matériau que les factions argumenteront ensuite. Tests : deux villes à économie identique et régimes opposés ; la sélection adverse apparaît seule dans le privé non régulé ; aucune pathologie sans cause au journal.

À éviter, déjà tranché : davantage de ressources ou de couches économiques avant que P1 ait prouvé l'économie ; un arbre technologique classique ; de la narration scriptée ; la simulation de métier par objet et par trajet à la RimWorld (coût late game, aucun dilemme ajouté — le rendu des flux est cosmétique, P1b, décision 66 de la spec).

Deuxième route candidate (v7.2) : **vue détaillée du service** (entrée P2, avec la présence physique) : quand le personnage du joueur est présent dans une firme, une vue détaillée du travail et de la consommation s'ouvre sur le principe agrégé/détaillé du combat (un seul pipeline, paramètres par environnement) ; le joueur y agit par décisions et intentions, jamais en manipulant les objets ; la vue détaillée lit l'état détaillé extrait selon le protocole de continuité d'identité et n'ajoute aucune règle propre.

## Où nous en sommes

Les fondations du design sont posées, le combat est cadré (chapitre 11), le personnage est l'interface du jeu et ses voyages, sa durée de vie et la durée de partie (300 ans) sont tranchés (chapitre 13). La spécification du palier P1 est complète (entités, événements, mécanismes 6.1 à 6.24, tranches 1 à 11 détaillées avec leurs tests, calibrage initial consolidé, vie sociale et parole des PNJ).

**Décidé**

- 4X temps réel 2D, profondeur avant graphismes, parties très longues sans ralentissement.
- Échelle de délégation individu → empire, un seul modèle d'entité, carrières différenciées par type d'organisation.
- Factions et sous-factions (firmes, associations, pirates), moteur de motivation interchangeable avec forte inertie.
- Simulation événementielle, déterministe, à niveaux de détail, solo d'abord.
- Crises émergentes, journal de causalité, loyauté à deux valeurs, secteur privé en actions génériques.
- Univers sans FTL, qui justifie la délégation et la simulation paresseuse des zones lointaines.

**Tranché dans cette version**

- Taux de risque continu à la place des seuils durs.
- Alertes classées par valeur d'intervention.
- Prototype découpé en paliers, avec interface jouable dès P1b.
- Horloge unique et théâtre local pour les échelles de temps.
- Continuité du joueur par son personnage et ses successeurs ; le personnage initial reste une entité persistante (conseiller, symbole) une fois remplacé.
- Un seul noyau tactique paramétré pour le combat spatial, au sol et l'abordage, avec trois niveaux de résolution et une identité des unités conservée d'un niveau à l'autre.
- Jeu non shooter : pilotage assisté, la résolution passe par le modèle tactique.
- Le joueur contrôle toujours son personnage (décisions fines à pause active, sans visée manuelle). Les vues de commandement sont des écrans du monde : terminal portable pour commander de partout, postes fixes pour la vraie capacité. Le monde continue quand un écran est ouvert, avec pause active possible.
- Prototype tactique en vue de dessus ; l'isométrique est tranché plus tard.
- Rendu en tuiles pour le sol et les intérieurs, sprites pour l'espace.
- Intérieurs de vaisseaux en graphe de compartiments, détail local à la demande ; abordage par brèche forcée.
- Marche, véhicules, décollage et voyage en version minimale dès le socle ; intérieurs détaillés progressifs.
- Durée de vie : longévité graduée à accès limité par grade et richesse (symétrique pour les PNJ), mort réelle possible, succession par mentorat comme filet de sécurité, retraite volontaire ; transfert de conscience en couche tardive.
- Planètes : vol libre en atmosphère autorisé mais détectable et coûteux ; rail contraint aux hubs ; lieux d'intérêt issus uniquement du modèle agrégé ; terrain procédural avec préfabriqués pour les lieux clés ; niveaux de détail L0 à L3 suivant la vitesse ; seuls la graine, la version du générateur et les écarts sont sauvegardés.
- Personnages : mémoire par événements saillants (plafonnée), individus nommés par leurs actes, état visible sur le sprite, répliques et journal générés, portraits modulaires qui vieillissent.

**Tranché en v6**

- Ordonnancement canonique des événements simultanés : horodatage logique à séquence, phases justifiées par le recensement P1, priorités en données, identifiant stable ; événements générés reportés au prochain instant (chapitre 9).
- Invariant d'indépendance du découpage : résultat strictement indépendant des régions, threads et temps réel ; mécanisme de synchronisation reporté au premier palier multi-régions ; P1 séquentiel (chapitre 9).
- Protocole de continuité d'identité : conservation des identifiants, quantités, relations et contrats aux passages agrégé ↔ détaillé ; champs conservés spécifiés par type d'entité dans les specs de palier (chapitre 9).
- Invariant d'extraction des cohortes : conservation stricte des totaux (population, richesse, griefs, attachement) à l'extraction et à la réinjection, testée en continu (chapitre 3).
- Sémantique de l'équation de griefs : coefficients par jour, calibrage pluriannuel, intégration événementielle, concordance agrégé/individuel à tester (chapitre 2).
- Hiérarchie décisionnelle doctrine → objectifs → plan → actions, sans nouveau moteur, cadences pilotées par les données (chapitre 3).
- Reconnaissance : variable sociale séparée des 4 besoins fondamentaux, position relative dans un groupe de référence, mise à jour événementielle, pèse sur les motivations plus que sur les griefs (chapitre 14).
- Consoles : capacité, autorisation et priorité sont trois axes séparés (chapitre 15).
- Héritage : transmission par dimensions distinctes, pas de bonus global (chapitre 13).
- Noyau tactique : pipeline partagé, modèles de résolution spécialisés quand l'approximation changerait les décisions (chapitre 11).
- Trajectoire de validation après P1 : quartier → station → petit réseau → vérification de l'agrégation, sans élargir le premier jouable (chapitre 9).
- Test transversal : l'absence de FTL doit produire des divergences locales, des initiatives, des informations périmées et des conflits autonomie/contrôle dans les scénarios headless ; critère de cohérence, pas de nouveau mécanisme.
- **Tranché en v6.1** : les six pistes de profondeur sont des routes approuvées (chapitre 17), chacune liée à son palier d'entrée et à ses tests ; les règles de délégation s'appellent « directives » ; le joueur peut falsifier des rapports (symétrie intégrale) ; les engagements commencent par des garanties matérielles ; la piste obsolescence est unifiée au chapitre 16 ; la dérive culturelle est une extension optionnelle ; la version minimale de l'usure entre dans la spécification P1.
- **Tranché en v6.2** : trois routes approuvées de plus (vol et milice en P2, réseau de faveurs complet, spéculation individuelle) ; la spécification P1 intègre le crédit et la saisie, la santé-trajectoire, la qualité des biens, les logements différenciés avec sous-location, la vie vitale minimale et le parrainage ; le reprice des prix affichés est une action de la firme évaluée par Q(a), l'heuristique par seuils de stock n'est que son comportement par défaut en données.
- **Tranché en v6.3** : la migration est une action, pas un système : inter-quartiers par Q(a) sur le rang atteignable (P2), interplanétaire par flux agrégés de cohortes sur lignes de passagers (P3) ; la ville hors quartier est fermée en P1 ; personne ne déplace des populations ; l'invariant I1 s'étend aux transferts de cohortes entre quartiers et entre planètes.
- **Tranché en v6.4** : cinq états individuels cachés (faim, fatigue, stress, ennui, douleur) avec préemption par urgence ; le stress est le collecteur unique de santé mentale et une entrée du moral, la panique un basculement ; le luxe est la conversion d'argent en temps, chaque bien porte (coût monétaire, coût temporel) ; la diversité des lieux obéit à la règle du point distinct ; la fiscalité de P1 est forfaitaire, sans acteur ; l'attractivité des lieux est enregistrée dès P1.
- **Tranché en v6.5** : la vivacité des PNJ vient de la causalité visible (hétérogénéité, habitudes, mémoire active, répliques traçables, groupes informels, rituels), jamais du bruit décoratif ; l'échelle d'interaction avec les PNJ est symétrique ; l'empathie est un trait et un canal de contagion, la dépression un état réversible du stress, la dissonance morale un pont vers la transgression ; les traits du décideur modulent le Q(a) des organisations ; la valuation des rôles est pondérée par des curseurs de valeurs en données ; la solidarité et le lobbying entrent en P2 (route 12), P1 garde le régime « rien ».
- **Tranché en v6.6** : le temps du quartier est découpé en quatre créneaux étiquetés, sans horloge intra-journalière (couplage en P1.5) ; le marché de la ville est élastique (courbe d'offre en données, délais logistiques) : il amortit les chocs locaux avec retard au lieu de les neutraliser ; les enfants contribuent au foyer dès 8 ans (corvées domestiques, jamais d'emploi salarié) ; le plancher anti-impasse est une propriété statistique testée en headless (entraide, servitude, maisonnée, sous-location précaire), jamais une règle dédiée.
- **Tranché en v6.7** : la spécification P1 est complète, tranches 1 à 11 détaillées avec leurs tests de fermeture (sections 13 à 23 de la spec) ; le cœur réductible (tranches 1 à 8 plus P1b minimal) est livrable seul, la tranche 9 et les enrichissements glissent en P1.5.
- **Tranché en v6.8** : la parole des PNJ est une action comme les autres, une action Q(a) à intention (se plaindre, partager, inciter, demander, entretenir, transmettre), contextualisée par la relation, le lieu et le créneau ; les répliques sont des templates rendus hors état (messages entiers localisables), jamais un LLM en exécution ; l'inquiétude d'autrui est une action bornée par la doctrine, dont le seuil d'intimité est un seuil d'information ; un décideur ne voit que ce qui lui parvient (chapitre 6).
- **Tranché en v6.9** : le calibrage initial de P1 est consolidé dans la spécification (section 25) ; les rituels sont un catalogue en données déclenché par les événements de vie, la fête de quartier portant le futur pain-et-jeux des factions (P2) ; l'écran de conversation propose des intentions filtrées, sans texte libre ni minigame ; l'estaminet entre en P1.5 (point distinct du restaurant) ; l'émigration produit un deuil partiel, sans griefs résiduels sauf abandon perçu ; l'allocation vieillesse reste structurellement sous le seuil de trappe.
- **Tranché en v7.0** : l'engagement employé ↔ firme (dérivé des décisions observées de la firme, jamais lu par le décideur) pondère les décisions d'arrêt, de soin et de démission des salariés ; la maladie devient contagieuse à l'atelier (exposition aux malades présents) et le présentéisme imposé produit contagion, maladies allongées et séquelles ; aucun régime de gestion ne domine, la firme au churn gardant l'avantage d'adaptation en choc de demande.
- **Tranché en v7.1** : l'engagement n'agit que par la prévention (probabilité d'incidents réduite, usure ralentie, options de résolution dans les décisions ponctuelles) ; ni temps de travail supplémentaire ni bonus de production, même dérivé d'actes journalisés (double comptage du travail déjà payé en heures) ; la productivité supérieure de la firme patiente est une mesure émergente (production par heure travaillée), jamais un coefficient.
- **Tranché en v7.2** : pas d'enrichissement nouveau du palier quartier tant que le cœur réductible (tranches 1 à 8 + P1b minimal) n'est pas vert en headless ; toute idée nouvelle est une route candidate passant les cinq critères d'adoption. Première candidate enregistrée : interactions inter-quartiers (extension de la route 10, entrée P2) — spécialisation des services entre quartiers et déplacements individuels pour travailler, se soigner et varier, en flux agrégés par cohorte.
- **Tranché en v7.3** : la génération procédurale des plans sépare structure (état de simulation), peau (feuille de style du propriétaire en données, appliquée en poids et contraintes) et marqueurs d'état (usure, reprises) ; construit ≠ possédé (le style de naissance persiste, le propriétaire ajoute une couche) ; un changement de propriétaire ne régénère rien, il journalise des événements de reprise ; le biome porte des contraintes et des usures, jamais un style.
- **Tranché en v7.4** : la feuille de style devient une **grammaire architecturale en données** par culture (lignes, toits, vide et plein, symétrie, ornement, espace commun), adaptée au biome en dialectes, évoluant avec la dérive culturelle, lisible en stratigraphie, branchée à la simulation par trois variables agrégées (densité, intimité, espace commun) ; les courbes vivent au niveau du bâtiment (empreintes multi-tuiles, préfabriqués), pas de la tuile. Production d'assets par un outil **manifeste → prompts** : variation par recolorisation programmatique et surcouches, jamais un prompt par tuile et par culture ; validation humaine par lot ; l'IA hors exécution ; le manifeste est la source de vérité.
- **Tranché en v7.5** : consolidation de la route 6 — une culture est un paquet de données porté par les cohortes (motivations, valuations, doctrines, rituels, grammaire architecturale, registres de langue) ; elle naît par divergence entre groupes séparés, au-delà d'un seuil de distance culturelle à calibrer (jumeau lent du seuil de faction) ; canaux de modification tous existants (exposition, institutions κ, générations, migrations, chocs, dissonance morale) ; lecture avec retard : manifestations datées, jamais de jauge.
- **Tranché en v7.6** : au niveau rendu, la grammaire architecturale est une **projection du vecteur culturel** (tables en données, inertie par dimension), jamais un stock indépendant ; **origine** par fonction de fondation (biome, économie, culture mère, technologie) — la tradition est une contrainte devenue identité ; **dérive à trois vitesses** (ornement : années ; matériaux : décennies ; masses : générations) rendant la stratigraphie datable ; les chocs matériels dévient la projection et se figent en norme (boucle contrainte → tradition), une contrainte brève laisse un vestige ; coût simulation nul, version de grammaire archivée à la construction.
- **Tranché en v7.7** : l'influence mutuelle des cultures ne circule que par le contact individuel pondéré au statut perçu (soft power émergent, jamais de mélange atmosphérique) ; la dissonance morale peut séduire ou raidir ; la pureté culturelle est une position doctrinale disponible (institutions κ contre l'étranger, factions possibles) ; convergence en gradient de vitesses, testée headless.

**Tranché en v5**

- Progression par dézoom : chaque niveau de dézoom jouable avant le suivant ; premier palier jouable = le quartier vivant (chapitre 14).
- Besoins triviaux = instanciation individuelle des variables du chapitre 2 (mêmes variables, coefficients par clé stable), pas un système séparé.
- Salaire réel (salaire nominal / coût du panier) et offre de travail par cohorte ; fixation des salaires par la table d'actions des firmes ; grèves par le taux de basculement du chapitre 2.
- Boucle de progression du quartier bornée par les prix locaux et les rendements décroissants sur l'usage du temps, pas par des règles artificielles.
- Doctrine personnelle du personnage, éditable via terminal ; autopilote = même mécanisme et même règle de décision que tous les agents.
- Hiérarchie des consoles (capacité = coût + encombrement), terminal portable empruntant la meilleure console à portée, modèle réseau unifié : latence = distance/c + relais, couverture par infrastructures réelles (constellations destructibles), abonnements comme contrats de bande et priorité, jamais de temps réel interstellaire (chapitre 15).

**Tranché en v4 et v4.1**

- Langage du prototype : Rust d'emblée, cœur séparé du rendu ; pas de réécriture, les replais de non-régression sont validés dès P1.
- Développement solo assisté par agents IA (Cursor) : spécifications écrites comme source de vérité, tests headless et replais comme garde-fous, tranches verticales courtes (chapitre 5).
- Taux de basculement tirés une seule fois par seuil de risque cumulé, recalcul de la date de franchissement quand les entrées changent : exact, déterministe, sans fenêtre à régler (chapitre 2).
- Unités de conquête liées aux cohortes de population : les pertes remontent à la cohorte d'origine, nourrissent les griefs et pèsent sur la légitimité ; recrutement borné par le plus contraignant de l'effectif de cohorte et de la production d'équipement (chapitre 10).
- Mentorat jouable et succession désirable : héritier incarnable par intermittence, académie et transmission influencée par les compétences du joueur, héritage de réseau, doctrine et légende ; bonus de formation plafonné pour éviter la dérive de surhommes (chapitre 13).
- Règles pour le développement assisté : replais de référence en lecture seule pour l'agent (toute modification exige une validation humaine), conventions de déterminisme Rust (pas d'itération de HashMap, fusion déterministe), conventions dans un document court séparé (chapitre 5).
- Sprites de personnages en 32x64 px, casque et arme en calques séparés (chapitre 12).

**Encore ouvert**

1. Valider ou ajuster le calibrage initial du chapitre 13 (durée de partie, longévité, fréquence des successions, voyages).
2. Gameplay concret des deux ou trois premiers grades de chaque voie, et équivalent des postes de commande pour les voies non militaires (pilote, marchand, pirate).
3. Règles de passage entre niveaux de détail (formation, escouade, incident local) : règles adaptatives plutôt que seuils fixes.
4. Rythme exact entre temps calme et interruptions.
5. Valider en prototype la taille des zones locales, les résolutions L0 à L3 et la liste des lieux préfabriqués à la main.
6. Qui construit les stations : le joueur, les factions, le secteur privé, ou tous ?
7. Place des drones terrestres, robots et exosquelettes ; niveau de détail des blessures, soins et évacuations.
8. Calibrer le mentorat jouable du chapitre 13 : fréquence et durée des missions de l'héritier, bonus de formation selon les compétences du personnage, plafond de progression par génération, ce que l'héritier peut faire seul.
9. Calibrer la limite de recrutement du chapitre 10 : arbitrage entre effectif de cohorte et production d'équipement, vitesse de vidage d'une cohorte, délai de reconstitution.
10. Consoles (chapitre 15) : jusqu'où la portable connectée à une console fixe hérite-t-elle à distance ; calibrage des coûts, portées et capacités par palier ; marché des abonnements (prix, priorité, concurrence des opérateurs).
11. Quartier (chapitre 14) : vitesse de sortie de misère visée ; calibrer la reconnaissance (groupes de référence, événements de mise à jour, poids dans Q(a)) ; arbitrer le salaire réel contre p pour éviter le double emploi.
12. Constellations (chapitre 15) : qui les construit et les possède, coût de lancement, détail de la couverture, lien avec le secteur privé du chapitre 6.

**Prochaine étape** : écrire la tranche 1 (noyau temporel, spec section 13), puis les tranches suivantes dans l'ordre du découpage.