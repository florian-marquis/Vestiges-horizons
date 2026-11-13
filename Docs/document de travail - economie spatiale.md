---
name: "economie-spatiale-travail"
title: "Document de travail · Économie spatiale : mondes, accès orbital, nœuds"
type: "text/markdown"
---

# Document de travail · Économie spatiale : mondes, accès orbital, nœuds

*Document de travail, hors cadrage. Consolidation de la réflexion de session (novembre 2026) sur les différences ville/station, le coût d'accès à l'orbite, les chantiers navals et les places d'échange. Statut : réflexion approfondie, entrée prévue P2-P3. Rien n'est tranché au sens du cadrage.*

---

## 1. Le principe : un seul système, des données d'environnement

Ville planétaire et station orbitale sont **le même objet de simulation** — le bassin de population (quartiers + marché commun + institutions + informations) — avec des données d'environnement différentes. Aucune règle « parce que c'est une station » : tout ce qui distingue les deux doit sortir de la physique du monde et des chaînes de production, jamais d'une branche spéciale de code.

**Rien n'est cher en soi.** Tout prix sort de trois facteurs en données : l'abondance de la ressource, la facilité à la rendre disponible, l'infrastructure de distribution. Une station qui capte un glacier cométaire a de l'eau bon marché ; une cité minière en désert paie chaque litre. Le prix de l'eau du quartier P1 est une donnée de biome, pas une loi du jeu.

## 2. Les quatre variables qui différencient planète et station

| Variable | Planète | Station |
| --- | --- | --- |
| **Le contenant** | surface extensible, frontière ouverte (atmosphère et sol gratuits) | volume fermé, plafond dur, frontière étanche |
| **Les chaînes de survie** | fournies par le monde (air, pression, gravité) : zéro infrastructure | **manufacturées** : air, eau, pression, gravité (rotation) sont des produits avec recettes, machines, usure, pannes |
| **L'accès au reste de l'économie** | coûteux (sortir du puits de gravité) | immédiat (déjà dans l'espace) |
| **L'identité collective** | diluée par l'extérieur | renforcée par la clôture (voir document révoltes, 3.1) |

Le point clé : une station n'est pas « là où c'est cher », c'est **là où la survie est une production industrielle**. L'air d'une station bien gérée peut être quasi gratuit ; la différence est qu'une panne de recycleur n'y est pas un inconfort mais une extinction en jours. La ville supporte la négligence, la station non. Cette asymétrie se modélise avec les mécanismes existants (usure, entretien, chaîne de production), sans système nouveau.

## 3. Le coût d'accès à l'orbite : la variable structurante

**Le coût de mise en orbite du kilogramme n'est pas une constante du jeu : c'est une conséquence des données physiques de chaque monde.**

- Gravité forte + atmosphère dense → fusées chimiques coûteuses (des centaines de fois le coût idéal au kg) : la planète reste agricole et industrielle mais **fermée**.
- Gravité faible ou pas d'atmosphère → canon électromagnétique, catapulte : export quasi gratuit.
- Planète terrestre bien dotée → **l'ascenseur spatial** : le seul moyen de faire chuter le coût d'un ou deux ordres de grandeur, parce qu'il remplace l'énergie d'escalade par du fret passif. Mais il exige un matériau parfait, un investissement colossal, des décennies de travaux — et il crée un **goulot géopolitique** : qui tient l'ascenseur tient le commerce de la planète entière.

La chaîne causale complète, entièrement émergente et en données :

**gravité × atmosphère → coût d'accès orbital → schéma commercial → richesse → poids politique.**

Aucune règle « les mondes du centre sont riches » : ils le deviennent parce que leur physique les a dotés d'ascenseurs tôt.

### 3.1 L'ascenseur comme mégastructure jouable

Objet de gameplay avec les mécaniques seules du projet :

- une **firme** (ou un état) propriétaire, avec recettes, postes, goulots ;
- une **construction de décennies** : investissement patient, action Q(a) d'organisation, risque d'abandon en cours de route ;
- une **rente de monopole** qui nourrit les griefs (le prix du transit comme externalité non consentie, route 12) ;
- une **cible de guerre** et un point de défaillance unique : sa perte coupe la planète de l'espace ;
- un **objet politique** : le taxer, le nationaliser, le concurrencer (un second ascenseur casse la rente).

### 3.2 La propriété des nœuds critiques

La propriété de l'ascenseur (et, par extension, de tout nœud de passage obligé : place d'échange, grand chantier, relais de communication) est un enjeu politique majeur, pour trois raisons qui sortent des mécaniques seules :

1. **Qui capte la rente.** Tout ce qui entre ou sort paie un transit ; la différence entre coût réel et prix de monopole est un transfert continu de richesse des habitants vers le propriétaire — une rente de situation, pas une rémunération de travail, exactement la matière des griefs (route 12).
2. **Qui tient une arme politique.** Couper, ralentir, prioriser : embargo contre une région récalcitrante, tarifs préférentiels aux alliés, taxe de crise. Le propriétaire tient l'économie en otage sans un seul soldat — et un propriétaire qui abuse fabrique ses ennemis : « ils nous ont coupés du ciel » est un récit fondateur de faction prêt à l'emploi (document révoltes, section 4).
3. **Qui décide de l'avenir physique du monde.** La capacité de transit limite ce que la planète peut importer, exporter, devenir. Un propriétaire conservateur fige l'économie ; un propriétaire qui agrandit en change la structure sociale. Un pouvoir de planification sans décret.

**Modèles de propriété, et leurs pathologies émergentes** (aucun ne domine — critère du projet) :

| Modèle | Avantage | Pathologie émergente |
| --- | --- | --- |
| Monopole privé (firme) | efficace, audacieux | rente maximale, captation politique, cible de révolte |
| État / centrale impériale | tarif uniforme, outil de politique | tarification politique, files, entretien négligé, instrument de répression |
| Coopérative des usagers (firmes et bassins cotisants) | prix coûtant, légitimité forte | décisions lentes, sous-investissement, poids excessif des gros usagers |
| Concession mixte (construit par l'un, exploité par l'autre) | partage des risques | litiges de frontière, renégociations, corruption du régulateur |

Aucun de ces comportements n'est codé : ils sortent de la Q(a) du propriétaire (une organisation, avec les traits de son décideur, 6.20) et des griefs des usagers. Test headless : quatre planètes jumelles, quatre modèles, mêmes données physiques — les pathologies apparaissent seules, au journal, et aucun modèle ne domine sur 500 ans.

**La propriété se décide d'abord à la construction** : qui finance les décennies de travaux exige généralement la propriété ou la concession — sauf si la centrale, les factions ou un consortium rachètent la dette. Une planète peut se faire construire son ascenseur par un empire qui, ce faisant, l'annexe économiquement sans tirer un coup de feu : du colonialisme par infrastructure, entièrement émergent.

**Accès joueur** : posséder le nœud critique de son monde est un sommet de carrière atteignable en P3 (comme la firme-étuve l'est en P1) ; le perdre ou le voir nationalisé est un scénario de chute tout aussi jouable. Symétrie intégrale : les PNJ patrons visent la même chose.

## 4. Où construire les vaisseaux

Les **chantiers navals** se placent là où trois conditions coexistent (le même calcul d'implantation que toute firme) :

1. **masse bon marché en orbite** : pied d'ascenseur, astéroïde, lune sans atmosphère ;
2. **main-d'œuvre qualifiée dense** : les vétérans et les académies ; le savoir est local (chapitre 16) — une station-chantier perdue coûte des années de savoir-faire ;
3. **capital patient** : des recettes longues et chères, des commandes à années.

Un chantier naval est une firme comme les autres : recettes, postes, goulots multi-postes (décision 66 de la spec P1). Les coques s'accumulent en **stocks agrégés** jusqu'à leur engagement (cadrage, chaîne de production), puis s'instancient pour le combat.

## 5. Où échanger : les places émergent des flux

Les **places d'échange** apparaissent aux nœuds de la logistique, jamais par décret : orbite haute des ascenseurs, croisements de routes, stations-relais là où la latence impose des ruptures de charge. Le cadrage le pose déjà : une station est un nœud avec taux d'entrée/sortie et stocks.

- Les marchés **apparaissent là où les flux se croisent, et meurent si les flux détournent**.
- Une place commerciale déchue est une station qui s'appauvrit : griefs, départ de la population (route 11), identité blessée — matériau direct pour le document révoltes.
- La spécialisation des nœuds suit la règle du point distinct, appliquée entre mondes.

## 6. La géographie naturelle de la civilisation (et plus loin encore)

En enchaînant les variables précédentes, la structure d'un empire réaliste sort sans règle :

1. **Les mondes à accès orbital bon marché** deviennent manufactures et hubs : riches, peuplés, connectés.
2. **Les mondes fermés** (gravité forte, atmosphère épaisse) restent greniers et arsenaux : peuplés, peu connectés, gouvernés de loin.
3. **La latence** (chapitre 15, vitesse de la lumière) fait le reste : les périphéries reçoivent les ordres en retard, s'auto-gouvernent de fait, développent leur identité — le document révoltes en fait des factions sécessionnistes en puissance.
4. **Les nœuds logistiques deviennent les objets politiques centraux** : les posséder, les taxer, les couper. Une guerre civile à l'échelle d'un empire est d'abord une guerre de nœuds — le cadrage le dit pour la logistique militaire, il suffit de l'appliquer au civil.

La hiérarchie spatiale complète : **quartier → bassin (ville ou station) → monde → système → empire** — chaque étage est le même objet nœud/flux avec des données différentes.

## 7. Implications de conception

- **La physique du monde est une donnée d'environnement** : gravité, atmosphère, ressources, clôture, distance. Le moteur ne connaît que des nœuds, des flux, des recettes et des Q(a).
- **Le bassin est l'objet de la spec P2** : une seule spécification, deux environnements en données. La première instance de P2 (ville ou station) est un choix à trancher ; intuition : ville d'abord, pour prolonger naturellement le P1 (la ville-mur devient un vrai lieu), la station ensuite comme environnement dur (plafonds, dépendance, survie manufacturée).
- **Tout est dans le budget late game** : nœuds et flux sont agrégés ; les individus nommés ne s'extraient que près du joueur ou aux moments charnières.

## 8. Questions ouvertes

1. **La première instance de P2** : ville planétaire ou station ? (intuition : ville)
2. **Qui construit les mégastructures** (ascenseurs, grands chantiers) : joueur, factions, secteur privé, tous ? (question déjà ouverte au cadrage pour les stations)
3. **Le modèle de propulsion et de route** : sublight pur (latence en années, cohérent avec le chapitre 15) — confirmer qu'aucun FTL n'entre jamais, même en P4+.
4. **L'économie des ressources** : astroïdes vs surfaces planétaires, qui mine quoi, avec quelles recettes et quels goulots ?
5. **La propriété des nœuds** : partiellement traitée (3.2) pour les nœuds critiques ; reste à trancher la place d'échange ordinaire — firme, institution, ou les deux (intuition : l'infrastructure est une institution, les stands et entrepôts sont des firmes).
6. **Le calibrage** : coûts d'accès orbital par classe de monde, temps de construction des mégastructures, part du commerce intermondial dans une économie de bassin.

## 9. Lien avec les autres documents

- Cadrage (v7.7) : chaîne de production et logistique (chapitres 8 et 9), latence (ch. 15), routes 10, 11, 12, question ouverte « qui construit les stations » ; ch. 12 v7.3-v7.4 (cohérence de génération, grammaire architecturale, manifeste → prompts) ; consolidation culturelle v7.5-v7.7 (l'architecture comme projection de la culture, dialectes culture × biome).
- Spec P1 : 6.15 (métiers, recettes, goulots), décision 66, section 25 (calibrage).
- Document de travail « Révoltes, factions et révolutions » : la clôture comme amplificateur d'identité (3.1), la latence comme générateur d'identité (section 3), la guerre civile comme guerre de nœuds.