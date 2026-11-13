---
name: "registre-completude"
title: "Vestige : Horizon · Registre de complétude"
type: "text/markdown"
---

# Vestige : Horizon · Registre de complétude

*Document maître de suivi. Chaque concept et mécanisme du jeu y figure avec son statut. Objectif : qu'aucun angle mort ne soit invisible. Mis à jour à chaque session.*

**Statuts** : ⬛ spécifié (spec complète : entités, événements, invariants, tests) · 🟦 cadré (décidé au cadrage, sans spec détaillée) · 🟨 réfléchi (document de travail, en attente de cadrage) · 🟥 ouvert (question posée, sans réponse) · ✅ calibré (valeurs mesurées sur simulation — aucun à ce jour)

**Règle de lecture** : rien ne se code sans ⬛ + valeurs de départ. Un palier n'est « prêt à coder » que si toutes ses lignes sont ⬛ ou ✅.

---

## 1. Noyau temporel (P1, tranche 1)

| Mécanisme | Statut | Où / prochaine étape |
| --- | --- | --- |
| Horloge logique, file d'événements, ordre canonique | ⬛ | Spec P1 sections 13 et 29 (ordre de mission prêt) |
| Phases d'événements (7, ensemble fermé) | ⬛ | Spec P1 section 5 |
| Report, rejet de date passée, anti-réentrance | ⬛ | Spec P1 13.3 |
| État sérialisable, snapshot/reprise | ⬛ | Spec P1 13.4 |
| Journal causal (hors état, append-only) | ⬛ | Spec P1 tranche 2 |
| RNG déterministe, flux par usage | ⬛ | Spec P1, conventions |
| Invariants I1-I9 | ⬛ | Spec P1 section 7 |
| Bots, suite headless, replais de référence | ⬛ | Spec P1 tranches 10-11 |

## 2. Individus et population (P1)

| Mécanisme | Statut | Où / prochaine étape |
| --- | --- | --- |
| Individus, cohortes, extraction, règle d'identité | ⬛ | Spec P1 sections 4 et 15 |
| Besoins fondamentaux, budget de temps | ⬛ | Spec P1 6.2, 15.1 |
| États déclenchés (faim, fatigue, stress, ennui, douleur, angoisse, dépression) | ⬛ | Spec P1 6.5 ; seuils 🟥 à calibrer (tranche 10) |
| Empathie, contagion de souffrance | ⬛ | Spec P1 6.6 |
| Mémoire saillante | ⬛ | Spec P1 section 4 |
| Doctrine personnelle (console portable) | ⬛ | Spec P1 6.7 |
| Vivacité PNJ (habitudes, variance, groupes informels, rituels) | ⬛ | Spec P1 tranche 9 (glissée P1.5) |
| Parole à intention (6 intentions, contextualisation) | ⬛ | Spec P1 section 24 (P1.5) ; taille du corpus de répliques 🟥 |
| Registres de langue par cohorte | 🟥 | Ouvert, à cadrer |
| Vieillissement, mort, deuil | ⬛ | Spec P1 (émigration, deuil partiel) ; deuil indemnisé 🟥 (P2) |
| Succession, héritier, plafonnement | 🟦 | Cadrage ch. 13 ; calibrage du mentorat 🟥 |
| Enfants contributeurs (corvées dès 8 ans) | ⬛ | Spec P1 6.23 |

## 3. Économie (P1)

| Mécanisme | Statut | Où / prochaine étape |
| --- | --- | --- |
| Firmes, recettes, gamme, investissement (action, pas palier) | ⬛ | Spec P1 6.4, 6.15 |
| Usure et entretien minimaux | ⬛ | Spec P1 6.4 (route 1 version minimale) |
| Recettes multi-postes à goulot | ⬛ | Spec P1 6.15, décision 66 |
| Marché des biens, prix affichés, reprice | ⬛ | Spec P1 6.5 ; bornes 🟥 à calibrer |
| Marché du travail (5 grandeurs), appariement événementiel | ⬛ | Spec P1 6.3, tranche 6 |
| Métiers (triptyque), 10 métiers P1, décisions ponctuelles | ⬛ | Spec P1 6.15, tranche 6 |
| Parrainage | ⬛ | Spec P1 6.11 |
| Crédit, prêts, saisie | ⬛ | Spec P1 6.8 |
| Logements différenciés, sous-location | ⬛ | Spec P1 6.14 ; taux d'acceptation 🟥 |
| Qualité des biens | ⬛ | Spec P1 |
| Maisonnée = firme | ⬛ | Spec P1 6.15, tranche 4 |
| Marché de la ville élastique (le mur) | ⬛ | Spec P1 6.2 ; pente/délais 🟥 |
| Plancher anti-impasse (propriété testée) | ⬛ | Spec P1 6.24 |
| Fiscalité forfaitaire (prélèvement, allocations) | ⬛ | Spec P1 6.16 |
| Santé-trajectoire, maladies, contagion d'atelier | ⬛ | Spec P1 6.9 ; incidence 🟥 |
| Engagement employé ↔ firme (prévention, sans bonus) | ⬛ | Spec P1 6.20, décision 64 ; amplitude 🟥 |
| Traits du décideur (empathie, patience, pérénité dure) | ⬛ | Spec P1 6.20 |
| Griefs (équation, intégration événementielle), grève | ⬛ | Spec P1 tranche 7-8 ; coefficients 🟥 |
| Reconnaissance, valuation des rôles par curseurs | ⬛/🟦 | Spec P1 ; calibrage du ch. 13 🟥 |
| Scalaire « diversité accessible » de la ville | 🟥 | Écarté sauf problème mesuré à P1b |

## 4. P1.5 (enrichissements du quartier)

| Mécanisme | Statut | Où / prochaine étape |
| --- | --- | --- |
| Couplage des créneaux (fréquentation, équipes tournantes, lits chauds) | 🟦 | Cadré v6.6 ; à spécifier |
| Estaminet | 🟦 | Cadré v6.9 ; à spécifier |
| Information perçue (un décideur ne voit que ce qui lui parvient) | 🟦/⬛ | Cadré v6.8, dates d'apprentissage au journal ; à spécifier |
| Rendu cosmétique des flux de métier | ⬛ | Spec P1 section 23 |
| Densité minimale de scènes (test P1b) | 🟥 | À mesurer à P1b |

## 5. Palier P2 — le bassin (cadré, non spécifié)

| Mécanisme | Statut | Où / prochaine étape |
| --- | --- | --- |
| Bassin de population (ville/station : un objet, deux environnements) | 🟨 | Doc économie spatiale ; à cadrer puis spécifier |
| Première instance de P2 : ville ou station | 🟥 | Question ouverte 1 du doc économie spatiale |
| Factions (formation, déclaration, schismes, dissolution) | 🟨 | Doc révoltes sections 4, 5, 8 |
| Révoltes, émeute, propagation, révolutions | 🟨 | Doc révoltes ; feuille de route 6 questions |
| Grain de la légitimité (point racine) | 🟥 | Doc révoltes, question ouverte 1 |
| Pouvoir central : organisation ou mur | 🟥 | Doc révoltes, question ouverte 3 |
| Justice, prison, peines par doctrine | 🟨 | Doc révoltes section 7 |
| Économie politique de la révolte (financement, taxes de guerre) | 🟥 | Doc révoltes, question ouverte 4 |
| Conséquences de la victoire (planète indépendante) | 🟥 | Doc révoltes, question ouverte 5 |
| Migrations inter-quartiers | 🟦 | Route 10 ; interactions inter-quartiers candidate v7.2 |
| Vol et milice | 🟦 | Route 7 |
| Solidarité, groupes de pression, externalités | 🟦 | Route 12 |
| Directives complètes (délégation) | 🟦 | Route 4 |
| Réseau de faveurs complet | 🟦 | Route 8 |
| Spéculation et revente individuelle | 🟦 | Route 9 |
| Engagements crédibles, traités, réputation-rumeur | 🟦 | Route 3 |
| Fiabilité des rapports, falsification symétrique | 🟦 | Route 2 |
| Combat abstrait C1-C4 | 🟦 | Cadrage ch. 11 |
| Consoles, modèle réseau, délégation | 🟦 | Cadrage ch. 15 |
| Incitation → organisation | 🟥 | Ouvert (P2) |

## 6. Palier P3 — monde, système, empire (cadré et réfléchi)

| Mécanisme | Statut | Où / prochaine étape |
| --- | --- | --- |
| Migrations interplanétaires, lignes de passagers, crédit de passage | 🟦 | Route 11 |
| Coût d'accès orbital, ascenseur, chantiers navals, places d'échange | 🟨 | Doc économie spatiale |
| Propriété des nœuds critiques | 🟨 | Doc économie spatiale 3.2 |
| Guerre civile, indépendances, guerre de nœuds | 🟨 | Doc révoltes + économie spatiale |
| Dérive culturelle (route 6) : vecteur, valuations, boucle valeurs-institutions | 🟦 | Cadrage ch. 17 (v6.1) ; consolidation culture (contenu, naissance par divergence, canaux) tranchée v7.5 ; influence mutuelle par contact et pureté culturelle tranchées v7.7 ; à spécifier P2-P3 |
| Rendu : grammaire = projection du vecteur culturel, origine, dérive à trois vitesses, chocs matériels | ⬛ (cadré) | Cadrage ch. 17 et ch. 12 (v7.6) ; tables de projection à spécifier au palier concerné |
| Seuil de scission culturelle (distance → deux cultures) | 🟥 | À calibrer en P2/P3, avec le seuil de déclaration de faction |
| Calibrage de rareté des crises | 🟥 | À écrire comme critère d'acceptation dès la spec |

## 7. Transversal

| Mécanisme | Statut | Où / prochaine étape |
| --- | --- | --- |
| Calibrage initial P1 (valeurs de départ) | ⬛ (valeurs non calibrées) | Spec P1 section 25 ; mesure tranche 10 |
| Registre consolidé des « à calibrer » | 🟥 | À créer (fusion des mentions dispersées) |
| Paliers au-delà de P3 (secteur, galaxie ?) | 🟥 | Jamais dessinés — vraie question de conception |
| FTL : jamais, même P4+ | 🟥 | À confirmer (question ouverte 3, doc économie spatiale) |
| Qui construit les stations / mégastructures | 🟥 | Cadrage ch. 17, question ouverte 2 du doc économie spatiale |
| Document de présentation non technique | ✅ (vivant) | Canvas + Drive, à mettre à jour à chaque version du cadrage |
| Plan de développement (processus, outillage, règle de gel) | 🟥 | Proposé en session, à rédiger avant tout code |
| Génération procédurale : cohérence structure/peau/marqueurs | ⬛ (cadrée) | Cadrage ch. 12 (v7.3) ; tests définis, à détailler dans les specs de palier concernées |
| Grammaire architecturale par culture, dialectes biome, stratigraphie | ⬛ (cadrée) | Cadrage ch. 12 (v7.4) ; à détailler dans les specs de palier |
| Outil de production d'assets (manifeste → prompts, recolorisation, surcouches) | ⬛ (cadré) | Cadrage ch. 12 (v7.4) ; à construire hors jeu, en préparation |
| Sources d'assets libres (Kenney CC0, LPC, OpenGameArt CC0) | 🟦/🟥 | Licence à trancher (CC0 partout recommandé) ; packs à évaluer sur le lot de 10 assets |
| Suivi du projet (todo) | ✅ | Document « Suivi du projet », tenu à jour à chaque session |

---

## 8. Comptage (à actualiser à chaque session)

- Spécifié ⬛ : le noyau complet, tout le P1 (24 mécanismes), P1b, une partie du P1.5.
- Cadré 🟦 : routes 1 à 12, paliers P2 et P3 en directions.
- Réfléchi 🟨 : révoltes/factions/révolutions, économie spatiale, bassin.
- Ouvert 🟥 : ~20 questions listées ci-dessus, chacune datée d'un palier.
- Calibré ✅ : rien — tout le calibrage attend la tranche 10 (P1) et les paliers suivants.

*Prochaines lignes à faire passer en ⬛ : P1.5 (créneaux couplés, estaminet, information perçue), puis le bassin P2 quand la feuille de route des révoltes sera tranchée.*