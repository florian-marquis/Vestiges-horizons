# Vestiges Horizons — Proposition corrective T7-T8 v1.2

**Statut :** tranche A approuvée, implémentée et validée techniquement
**Base de design :** cadrage v7.8, révision v7.8.1
**Périmètre :** dynamique entière des griefs et temps logique ; aucun correctif du risque ou des grèves dans cette tranche.

## 1. Décisions retenues pour la tranche A

- L'équation de référence est celle du cadrage, chapitre 2 :
  `dG/dt = 0,09u + 0,02p − 0,03q` points/jour.
- `u`, `p` et `q` sont stockés en millièmes sur `[0, 1000]`. `G` est stocké en quarts de micro-point (`qµpt`) sur `[0, 400 000 000]`, soit `[0, 100 000 000]` µpt.
- La pente entière en µpt/jour est `s = 90u + 20p − 30q`. Il n'y a pas de terme constant de soulagement.
- Une journée comporte quatre créneaux de durée égale. `Timestamp.seq` identifie le créneau (0 à 3) pour les événements T7. L'ordre de la file est étendu à `(jour, créneau, phase, priorité, id)` ; les événements existants au créneau 0 gardent leur ordre relatif.
- Le nouveau moteur est la source canonique et est relié au chemin T7 dès cette tranche. Il ne doit pas coexister avec un second état de griefs utilisé par le jeu.

## 2. État canonique et calcul

Un état de griefs par cohorte contient au minimum :

- l'identifiant de cohorte existant (`EntityId`) ;
- `g_qmicro: i64`, en quarts de micro-point ;
- `u`, `p`, `q: u16`, en millièmes ;
- `last_update: Timestamp` ;
- `epoch: u64`.

La pente reste constante entre deux événements `InputChanged`. Un créneau vaut un quart de journée ; avec `s` en µpt/jour, l'incrément exact est `s` qµpt par créneau :

```text
delta_slots = 4 * (target.day - last_update.day)
              + target.seq - last_update.seq
g_qmicro = clamp(g_qmicro + s * delta_slots, 0, 400_000_000)
```

Le calcul des produits et sommes intermédiaires utilise une largeur suffisante (`i128` si nécessaire). Toute date antérieure à `last_update` et toute entrée hors `[0, 1000]` produisent une erreur explicite. La mise à jour des entrées avance d'abord l'état avec l'ancienne pente, applique les nouvelles valeurs, puis incrémente l'époque.

`EventQueue` conserve l'ordre de jour, créneau, phase, priorité et identifiant. Son API existante reste compatible et crée les événements au créneau 0 ; une API horodatée permet de planifier les événements T7 aux autres créneaux. La séquence n'est pas une durée arbitraire : seuls les quatre créneaux définis par P1b donnent une fraction de journée.

## 3. Événements et périmètre

La tranche ajoute `InputChanged`, qui transporte l'identifiant de cohorte et les nouvelles valeurs `u/p/q`. Un changement d'entrée est traité à son horodatage logique ; le moteur avance l'état jusqu'à cet instant avec les anciennes valeurs avant de les remplacer.

Cette tranche ne définit ni bandes de risque, ni taux `λ`, ni tirage de seuil, ni durée ou fin de grève. Elle ne modifie pas encore la représentation des griefs individuels, l'extraction/réinjection ou I8 : ces sujets restent des tranches dédiées. Elle n'inclut pas le correctif distinct de `labor.rs`.

## 4. Tests de fermeture

1. **Conversion exacte :** `u=1000, p=q=0` donne une pente de `90 000` µpt/jour.
2. **Critère de pénurie totale :** depuis `G=0`, `u=1000` donne `99 990 000` µpt après 1 111 jours, puis atteint le plafond après 1 112 jours.
3. **Critère de pénurie partielle :** depuis `G=0`, `u=300` atteint le plafond après 3 704 jours, dans la cible de 8 à 12 ans.
4. **Apaisement :** depuis le plafond, `q=1000, u=p=0` laisse `10 000` µpt après 3 333 jours et atteint zéro au jour 3 334, dans la cible de 6 à 10 ans.
5. **Temps intrajournalier :** une journée complète correspond à quatre créneaux ; intégrer séparément chaque créneau donne le même résultat qu'une avance directe sur les mêmes entrées.
6. **Changements d'entrée :** une séquence de changements à différents créneaux est identique à un oracle pas-à-pas indépendant, y compris quand plusieurs entrées changent le même jour.
7. **Bornes et erreurs :** `G` reste dans `[0, 400 000 000]` qµpt ; date rétrograde et entrées hors bornes sont rejetées explicitement.
8. **Ordre canonique :** les événements sont traités par `(jour, créneau, phase, priorité, id)`, indépendamment de leur ordre d'insertion.
9. **Non-régression :** `test_replais` passe sans modification des JSON de référence T1-T5.

## 5. Conditions de sortie et décisions reportées

Les conditions techniques de fermeture de la tranche A sont satisfaites : le chemin T7 utilise un seul état canonique, les tests passent et les replais T1-T5 restent identiques. Aucun replai de référence n'a été régénéré.

Le benchmark T7-T8 reste un signal à examiner avant de valider le comportement des grèves : il compte 18 250 déclenchements sur 18 250 vérifications (5 cohortes sur 3 650 jours). Ce résultat n'est pas une validation du calibrage du risque ou des grèves, qui reste hors périmètre de la tranche A.

### Décision à présenter avant la tranche B

La tranche B déterminera quand une grève devient plus probable. Avant de la coder, présenter la proposition en mots simples et demander si les effets correspondent à l'intention :

- quand les griefs augmentent, est-ce que le risque de grève évolue comme attendu ?
- quand le salaire permet moins bien de couvrir les besoins, est-ce que le risque évolue comme attendu ?
- les moyens de la cohorte sont-ils pris en compte sans compter deux fois la hausse des prix ?

Montrer quelques situations concrètes et leur résultat attendu, sans demander à la personne de vérifier la formule ou le code. Elle doit pouvoir répondre simplement : « oui, cela correspond », « non, voici ce qui ne convient pas » ou « je ne sais pas encore ». La formule exacte et ses preuves techniques restent dans la spécification.

Avant la tranche D, spécifier les transitions et bornes de la représentation `G + δ`, notamment la conservation du total pendant l'extraction/réinjection et la sémantique de la population totale. Présenter aussi cette décision à l'humain par ses effets concrets, selon la convention 34.
