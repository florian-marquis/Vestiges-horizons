# T7-T8 — Synthèse pour reprendre la discussion

## Contexte

Revue indépendante de la spec corrective T7-T8 v1.2, de l'implémentation et du benchmark. L'objectif de la suite est de vérifier simplement si la dynamique des griefs correspond à l'intention, sans confondre cette vérification avec la formule du risque ou des grèves.

La source de design actuelle est le cadrage v7.8 (révision v7.8.1). La spec corrective concernée est [T7-T8 correctif - spec v1.2.md](<../spec/T7-T8 correctif - spec v1.2.md>).

## Décisions et compréhension retenues

- La formule de référence pour la tranche A est `dG/dt = 0,09u + 0,02p - 0,03q` points par jour, avec `u`, `p`, `q` en `[0,1]` et `G` borné à `[0,100]`.
- Il n'y a pas de terme de soulagement constant. Si `u=p=q=0`, G reste stable.
- Les trois comportements numériques présentés ont été considérés conformes à l'intention : la pénurie fait monter G lentement; les services le font baisser plus lentement; en l'absence de ces facteurs, G ne change pas.
- Le critère de pénurie partielle doit préciser `p=q=0`. Le critère d'apaisement précise déjà `u=p=0`.
- Pour valider le calibrage réel du quartier, on veut observer les sorties avant de choisir un seuil. Aucun seuil d'acceptation n'est fixé à ce stade.
- Toute validation humaine doit être présentée avec des résultats concrets, en termes simples, sans demander de vérifier le code ou les calculs.

## Résultats techniques déjà vérifiés

`cargo test --lib griefs` a réussi : 8 tests, 0 échec.

| Situation | Résultat observé |
| --- | --- |
| Pénurie totale : `u=1000, p=q=0` | 99,99 après 1 111 jours; G atteint 100 au jour 1 112 |
| Pénurie partielle : `u=300, p=q=0` | G atteint 100 au jour 3 704 |
| Services au maximum : `q=1000, u=p=0`, depuis G=100 | 0,01 après 3 333 jours; G atteint 0 au jour 3 334 |
| Aucun facteur : `u=p=q=0` | pente nulle; G reste stable d'après la formule, sans test dédié spécifique |

Ces tests vérifient l'application de la formule et des bornes, pas la pertinence ludique de la formule dans le quartier.

## Mesure headless à produire

La spec v1.2 demande maintenant une simulation headless de 10 ans sur le quartier P1. Pour chaque cohorte, afficher :

| Cohorte | q min. | q moyen | q max. | % des jours G=0 | % des jours 0<G<100 | % des jours G=100 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |

Il s'agit d'un tableau d'observation, sans seuil de réussite prédéfini. La revue humaine pourra ensuite répondre simplement si la situation du quartier correspond à ce qu'elle imagine.

**Limite actuelle :** `benchmark_p1` simule le marché, pas le système de griefs. `benchmark_t7_t8` fait varier `u/p/q` artificiellement et ne constitue pas une mesure du quartier. Il faut relier `q` à l'état réel P1 avant de produire ce tableau; ne pas présenter les séries synthétiques comme des résultats de quartier.

## Anomalie du benchmark des grèves

Le résultat de 18 250 déclenchements sur 18 250 vérifications ne prouve pas que G a saturé. Dans le code actuel, le risque comprend un terme lié au salaire réel qui dépasse déjà le seuil par défaut de 50 pour les cohortes du benchmark, même si G vaut zéro. Le benchmark vérifie le seuil chaque jour; le gestionnaire peut alors créer une nouvelle grève et remplacer la précédente. Ce benchmark n'est donc pas une preuve de calibrage T7, ni une validation du comportement T8.

Le risque et les grèves restent hors du périmètre de la tranche A. Ne pas modifier leur formule pendant la mesure de G.

## Prochaine étape dans une nouvelle discussion

1. Reprendre le mode JEU et charger les instructions du dépôt selon [AGENTS.md](../../AGENTS.md), notamment `Agents/DEV-JEU.md` et `Agents/memoire-vive.md`.
2. Identifier comment calculer `q` à partir de l'état réel du quartier P1 et l'intégrer à un scénario headless de 10 ans.
3. Produire le tableau par cohorte ci-dessus, sans seuil d'acceptation.
4. Présenter le tableau à l'humain avec une question simple : « Est-ce que cette évolution des griefs ressemble à ce que tu voulais pour le quartier ? »
5. Traiter séparément, après cette revue, le benchmark, la formule de risque et les déclenchements répétés de grève.

## Fichiers et état de référence

- Cadrage : `Docs/cadrage vestiges horizons - v7.8.md` (révision v7.8.1).
- Spec corrective : `Docs/spec/T7-T8 correctif - spec v1.2.md`.
- Moteur des griefs : `src/griefs.rs`.
- Benchmark P1 actuel : `src/bin/benchmark_p1.rs`.
- Benchmark T7-T8 actuel : `src/bin/benchmark_t7_t8.rs`.
- Branche observée lors de la rédaction : `feat/t7-griefs-tranche-a`, commit `1991327` (`docs(validation): clarifie les demandes de revue humaine`).
