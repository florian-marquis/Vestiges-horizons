---
name: "suivi-projet"
title: "Vestige : Horizon · Suivi du projet (todo)"
type: "text/markdown"
---

# Vestige : Horizon · Suivi du projet (todo)

*Document de pilotage. Fait / en cours / à faire. Mis à jour à chaque session. Statut au 12 novembre 2026.*

---

## 1. Ce qui est fait

### Cadrage et conventions
- Cadrage du 4X **v7.2** (historique v4 → v7.2) : 12 routes approuvées, ch. 17, principe d'information perçue (v6.8), engagement par la prévention (v7.1), gel des enrichissements du quartier + candidate inter-quartiers (v7.2). Sur Drive (à jour).
- Conventions v2.1 (règles 1-27). Sur Drive (à jour).

### Spécification P1 — le quartier vivant (complète)
- Entités, 7 phases d'événements, mécanismes 6.1 à 6.24, 13 scénarios de choc, invariants I1-I9, données initiales.
- 11 tranches spécifiées (sections 13-23) avec tests de fermeture.
- 66 décisions tranchées (dont 64 : engagement sans bonus ni temps donné ; 66 : recettes multi-postes à goulot).
- Ordres de mission tranches 1 à 11 (sections 29-36) : types, API, étapes, pièges, critères d'acceptation.
- Calibrage initial consolidé (section 25), rituels, écran de conversation, directives de firme.
- Sur Drive (à jour).

### Documents de présentation et de travail
- **Présentation non technique** : « le jeu expliqué à tout le monde » + 2 schémas (paliers, boucle de vie). Sur Drive.
- **Document de travail révoltes, factions et révolutions** : propagation sans jauge, cinq ingrédients du « nous », déclaration, majorité inactive, traitement des repérés (foule/institution/stratégie), schismes, rôle du joueur, 6 tests headless, feuille de route de 6 questions. Sur Drive.
- **Document de travail économie spatiale** : bassin (ville/station = un objet, deux environnements), coût d'accès orbital, ascenseur (mégastructure + propriété des nœuds critiques), chantiers navals, places d'échange, géographie de la civilisation. Sur Drive.
- **Registre de complétude** : carte de tous les mécanismes avec statuts. Sur Drive.

## 2. En cours (réflexion ouverte)

- **Révoltes/révolutions** : feuille de route de 6 questions à trancher avant cadrage — (1) grain de la légitimité [racine], (2) sortie de la révolte de quartier (émeute), (3) pouvoir central organisation ou mur, (4) économie politique de la révolte, (5) conséquences de la victoire, (6) calibrage de rareté.
- **Économie spatiale** : 6 questions ouvertes — (1) première instance de P2 : ville ou station [intuition : ville], (2) qui construit les mégastructures, (3) FTL jamais ? à confirmer, (4) économie des ressources (astéroïdes vs surfaces), (5) statut de la place d'échange [intuition : infrastructure-institution + firmes], (6) calibrage des coûts d'accès.

## 3. À faire — prochains points prévus

### Documentation (priorité selon la règle « tout documenter avant de coder »)
1. **Registre consolidé des « à calibrer »** : fusionner toutes les valeurs en attente (spec section 25 + mentions dispersées) en un registre unique avec palier de tranchage.
2. **Spécification P1.5** : créneaux couplés, estaminet, information perçue — faire passer ces lignes du registre en ⬛.
3. **Trancher la feuille de route des révoltes** (les 6 questions, dans l'ordre, en sessions dédiées), puis rédiger la route candidate au cadrage.
4. **Trancher les questions d'économie spatiale**, puis spécifier le bassin (P2) : factions, politique, solidarité, migrations, milice, directives complètes, justice.
5. **Spécification P3** : migrations interplanétaires, accès orbital, indépendances, guerre civile.
6. **Dessiner les paliers au-delà de P3** (secteur, galaxie ?) — jamais esquissés.
7. **Plan de développement** (court) : processus tranche par tranche, outillage (dépôt, harnais, replais, CI), règle de gel « pas de code sans spec ⬛ », suivi des à-calibrer.
8. **Tenir à jour** : présentation non technique à chaque version du cadrage ; registre de complétude et ce suivi à chaque session.

### Développement (quand la décision sera prise)
9. **Tranche 1 — noyau temporel** : ordre de mission prêt (spec section 13 + mission section 29). Rien à spécifier de plus ; à lancer quand la stratégie « tout documenter » le décidera.
10. Tranches 2 à 8 (cœur réductible), puis P1b minimal, puis tranches 9-11 (P1.5).

### Points ouverts plus lointains (rappel)
- Deuil comme arrêt indemnisé ou non (P2, fiscalité) ; registres de langue par cohorte ; taille du corpus de répliques par intention ; moment où l'incitation devient organisation (P2) ; partage d'idées et dérive culturelle (route 6) ; calibrage de la reconnaissance (ch. 13) ; tout le palier P2 et au-delà.

## 4. Décisions récentes (dernières sessions)

| Version/décision | Contenu |
| --- | --- |
| v7.1 | Engagement sans bonus ni temps donné : prévention seulement (décision 64 réécrite, T6.7 durci) |
| v7.2 | Gel des enrichissements du quartier jusqu'au cœur vert ; candidate inter-quartiers (extension route 10, P2) |
| Décision 66 | Recettes multi-postes à goulot (tranche 6), jamais de chorégraphie par objet |
| Rendu | Simulation par objet exclue ; rendu cosmétique des flux en P1b ; vue détaillée candidate P2 |
| Vocabulaire | quartier → bassin → monde → système → empire ; ville/station = un objet, deux environnements |
| Prix | Rien n'est cher en soi : abondance × facilité d'accès × infrastructure |
| v7.3 | Génération procédurale : structure / peau (feuille de style du propriétaire) / marqueurs d'état ; construit ≠ possédé ; changement de propriétaire par événements de reprise ; biome = contraintes et usures, jamais un style |
| v7.4 | Grammaire architecturale par culture (lignes, toits, vide et plein, symétrie, ornement, espace commun), dialectes culture × biome, stratigraphie, courbes au niveau du bâtiment ; outil manifeste → prompts (variation par recolorisation et surcouches, IA hors exécution, validation par lot) |
| Assets | Sources libres identifiées : Kenney (CC0, base recommandée), LPC (32×64 personnages, licences mixtes CC-BY-SA/GPL), collections CC0 OpenGameArt ; décision de licence à trancher |
| v7.5 | Consolidation route 6 : culture = paquet de données porté par les cohortes ; naissance par divergence (seuil de distance culturelle à calibrer, jumeau lent du seuil de faction) ; canaux tous existants (exposition, institutions κ, générations, migrations, chocs, dissonance morale) ; lecture avec retard, jamais de jauge |
| v7.6 | Rendu : grammaire architecturale = projection du vecteur culturel (tables en données) ; origine par fonction de fondation (contrainte devenue identité) ; dérive à trois vitesses (ornement/matériaux/masses) ; chocs matériels → boucle contrainte-tradition ; coût simulation nul |
| v7.7 | Influence mutuelle des cultures : uniquement par contact individuel pondéré au statut perçu (soft power émergent) ; dissonance morale séduit ou raidit ; pureté culturelle = position doctrinale (institutions κ, factions) ; convergence en gradient de vitesses |

## 5. Inventaire des documents (Drive, dossier « Vestige horizon »)

| Document | État |
| --- | --- |
| Cadrage v7.7 | À jour |
| Conventions v2.1 | À jour |
| Spec P1 (quartier vivant) | À jour |
| Présentation non technique | À jour (vivant) |
| Doc de travail révoltes/factions/révolutions | À jour (vivant) |
| Doc de travail économie spatiale | À jour (vivant) |
| Registre de complétude | À jour (à actualiser chaque session) |
| Suivi du projet (ce document) | À jour (à actualiser chaque session) |