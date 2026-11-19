# Tranche B1 — Grève par risque cumulé : rapport de validation

- **Branche :** `feat/t7-t8-greve-risque-cumule` (base `e10b60e`, code vérifié à `f2fe9c8`)
- **Spécification verrouillée :** `jeu/Docs/spec/T7-T8 tranche B1 - spec.md`
- **Ticket séparé (hors périmètre B1) :** `jeu/Docs/tickets/credit-repayment-default-fix.md`
- **Politique salariale provisoire :** Option A (aucune perte de salaire pendant la grève en B1 ; sera traitée en B2)
- **Statut :** Implémentation et vérifications terminées ; **en attente de validation humaine** (B2 non démarrée).

---

## 1. Rapport par commit (`e10b60e..f2fe9c8`)

| # | Commit | Fichiers | Contenu |
| --- | --- | --- | --- |
| 0 | `aa48601` | `Docs/suivi du projet - todo.md`, `jeu/Docs/spec/T7-T8 tranche B1 - spec.md`, `jeu/Docs/tickets/credit-repayment-default-fix.md` (+305/-4) | Verrouillage de la spec B1 avec les 9 arbitrages et ouverture du ticket séparé `credit.rs` (sans toucher au code de `credit.rs`). |
| 1 | `193d13b` | `jeu/src/strike.rs` (+506) | Module de calcul 100 % entier : table `LN1P_TABLE[257]`, `stable_u64`, `draw_exponential_threshold_scaled`, `delta_h_scaled`, `find_crossing_slot` (forme fermée + bissection entière), `draw_strike_duration_days` et 9 tests unitaires. |
| 2 | `9c120ae` | `jeu/src/griefs.rs`, `jeu/src/lib.rs`, `jeu/src/replai_gen.rs`, `jeu/src/strike.rs`, `jeu/tests/test_tranche8.rs` (+1259/-8) | Automate événementiel par cohorte (`Dormant`, `Accumulating`, `PendingNextSlot`, `OnStrike`, `Cooldown`), événements 71–74 à `Id` déterministe, remise à zéro à `G = 0`, répit de 60 j (`STRIKE_COOLDOWN_DAYS = 60`), règle 18 (jamais même-instant), `G` inchangé et tests B1. |
| 3 | `f8e6c69` | `jeu/src/entities.rs`, `jeu/src/strike.rs`, `jeu/tests/test_tranche8.rs` (+453/-12) | Journal causal (top-5 causes `InputChanged` par `(ΔH DESC, id ASC)`, `learned_by = jour`, décisions 80/81) et `verify_i7` / `add_entry` étendus à `(timestamp, phase)` sans casser T2. |
| 4 | `9fb20e1` | `jeu/Cargo.toml`, `jeu/src/bin/benchmark_t7_t8.rs` (supprimé, -844), `jeu/src/salary.rs`, `jeu/src/strike.rs`, `jeu/tests/test_tranche7.rs`, `jeu/tests/test_tranche8.rs` (+174/-1620) | Suppression de `taux_risque`, `seuil_risque`, `calculer_taux_risque`, `seuil_franchi`, `declencher_greve`, `fin_de_greve` ; retrait `git rm` de `benchmark_t7_t8` (motif documenté au commit) ; synchro `self.strikes` pendant `OnStrike` ; test garde-fou statique (`DISPLAY_MODULE_ALLOWLIST = &[]`). |
| 5 | `24ee5a2` | `jeu/src/bin/quartier_10ans.rs` (+454/-2) | Ajout des lectures de grève B1 (`S0b`, `S1`, `S2`, `S4` sur 3 650 j, graine 42), intégrale analytique entière `u128`, non-régression déterministe et invariance de `G` vs A2. |
| 6 | `37827f1` | `jeu/src/strike.rs` (+36/-7) | Correctif moteur : saturation `i128` de `add_slots` et `slots_between` empêchant tout débordement `i64` en jour négatif ou passé sur les grands `k: u64`, + test `test_b1_add_slots_saturation_sans_debordement`. |
| 7 | `f2fe9c8` | `jeu/src/bin/quartier_10ans.rs` (+60/-29) | Correctif lecture : émission d'`InputChanged` uniquement sur changement réel de `(u, p, q)`, capture exacte des jalons `365`/`1825`/`3650`, `assert!(day >= 0)`, assertion `events_processed <= 20` sur `S0b`, 0 warning. |

### Vérification globale (`f2fe9c8`)
- `cargo test --manifest-path jeu\Cargo.toml` : **252/252 passés, 0 échec, 0 ignoré** (`lib` 131, `quartier_10ans` 10, `test_inputs` 32, `test_replais` 3, `test_tranche2` 8, `test_tranche3` 22, `test_tranche4` 5, `test_tranche5` 14, `test_tranche7` 12, `test_tranche8` 15 ; 0 warning sur `strike.rs` et `quartier_10ans.rs`).
- Zéro occurrence dans le code de `taux_risque`, `seuil_risque`, `calculer_taux_risque`, `seuil_franchi`, `declencher_greve`, `fin_de_greve` et zéro `f64`/`f32`/`.exp(`/`.ln(` hors commentaires dans `strike.rs`, `salary.rs`, `entities.rs`, `quartier_10ans.rs`.
- `jeu/src/credit.rs` et `jeu/src/labor.rs` strictement intacts.

---

## 2. Replais touchés

**Aucun.**
- `cargo test --manifest-path jeu\Cargo.toml --test test_replais` : **3/3 verts** (`test_replais_determinisme_triple`, `test_garde_src_replai_inchanges`, `test_replais_hashes_correspondent`).
- Les 5 replais `T1`–`T5` sous `jeu/src/replai/` et `jeu/tests/replais_hashes.txt` sont strictement intacts.

---

## 3. Résultats des lectures (`quartier_10ans`, graine 42, 3 650 j)

Pour chaque cohorte :
- `attendu actif` = intégrale analytique entière de λ(G(t)) hors phases de grève et de répit de 60 j ;
- `attendu brut` = intégrale sur tout l'horizon `[0, 3650]`.
- Aucun seuil d'acceptation posé avant observation (convention 24).

### `S0b` — Nominal sans choc, `q = 500` (`events_processed = 5`, `verify_i7 = true`)

| Cohorte | `O` | Observées | Attendu actif | Attendu brut | Détail |
| --- | --- | ---: | ---: | ---: | --- |
| 0 | 200 | 0 | 0.457 | 0.457 | — |
| 1 | 400 | 0 | 0.422 | 0.422 | — |
| 2 | 600 | 0 | 0.074 | 0.074 | — |
| 3 | 600 | 0 | 0.000 | 0.000 | — |
| 4 | 400 | 0 | 0.000 | 0.000 | — |

### `S1` — Imports divisés par deux (`events_processed = 742`, `verify_i7 = true`)

| Cohorte | `O` | Observées | Attendu actif | Attendu brut | Détail |
| --- | --- | ---: | ---: | ---: | --- |
| 0 | 200 | 0 | 0.075 | 0.075 | — |
| 1 | 400 | 0 | 0.026 | 0.026 | — |
| 2 | 600 | 0 | 0.009 | 0.009 | — |
| 3 | 600 | 0 | 0.003 | 0.003 | — |
| 4 | 400 | 0 | 0.000 | 0.000 | — |

### `S2` — Imports interrompus (`events_processed = 474`, `verify_i7 = true`)

| Cohorte | `O` | Observées | Attendu actif | Attendu brut | Détail |
| --- | --- | ---: | ---: | ---: | --- |
| 0 | 200 | 0 | 0.134 | 0.134 | — |
| 1 | 400 | 0 | 0.069 | 0.069 | — |
| 2 | 600 | 0 | 0.041 | 0.041 | — |
| 3 | 600 | 0 | 0.025 | 0.025 | — |
| 4 | 400 | 0 | 0.012 | 0.012 | — |

*Note de lecture : `S0b` présente un attendu supérieur à `S1`/`S2` parce qu'en `S0b`, les services neutres `q = 500` ne compensent pas la légère pression prix des cohortes modestes, si bien que `G` monte lentement pendant 10 ans jusqu'à ~58 pts pour la cohorte 0 ; en `S1`/`S2`, les services `q = 800..1000` font redescendre `G` après le choc de 90 j.*

### `S4` — Dégradation longue (`events_processed = 18044`, `verify_i7 = true`)

Total : **9 grèves observées** pour **8.622** attendues en fenêtres actives (**9.510** en brut).

| Cohorte | `O` | Obs. | Attendu actif | Attendu brut | Grèves déclenchées (début `(j, créneau)`, durée, fin, `G` au déclenchement) |
| --- | --- | ---: | ---: | ---: | --- |
| 0 | 200 | 1 | 1.622 | 1.689 | #1 : `(3433, 2)`, 1 j, fin `(3434, 2)` `DurationElapsed`, `G = 400 000 000 qµpt` (100 pts) |
| 1 | 400 | 3 | 2.420 | 2.811 | #1 : `(2593, 1)`, 6 j, fin `(2599, 1)`, `G = 337 395 860 qµpt` (84 pts) ; #2 : `(2813, 1)`, 2 j, fin `(2815, 1)`, `G = 400 000 000 qµpt` (100 pts) ; #3 : `(2948, 1)`, 5 j, fin `(2953, 1)`, `G = 400 000 000 qµpt` (100 pts) |
| 2 | 600 | 2 | 2.433 | 2.759 | #1 : `(2973, 2)`, 1 j, fin `(2974, 2)`, `G = 303 862 640 qµpt` (75 pts) ; #2 : `(3514, 2)`, 1 j, fin `(3515, 2)`, `G = 400 000 000 qµpt` (100 pts) |
| 3 | 600 | 1 | 1.542 | 1.584 | #1 : `(2875, 3)`, 1 j, fin `(2876, 3)`, `G = 176 992 680 qµpt` (44 pts) |
| 4 | 400 | 2 | 0.605 | 0.667 | #1 : `(2602, 1)`, 2 j, fin `(2604, 1)`, `G = 113 656 440 qµpt` (28 pts) ; #2 : `(3323, 3)`, 7 j, fin `(3330, 3)`, `G = 228 447 840 qµpt` (57 pts) |

---

## 4. Anomalies relevées et traitées

1. **`self.strikes` désynchronisé pendant une grève active (Commit 2 → corrigé au Commit 4 `9fb20e1`)** : `is_on_strike` / `get_strike` renvoyaient `false`/`None` pendant `OnStrike`. Corrigé et couvert par test.
2. **Débordement `i64` dans `add_slots` sur les très grands `k` (détecté au Commit 5 → corrigé au Commit 6 `37827f1`)** : quand un groupe avait une tension très faible et stable, le délai calculé pouvait dépasser `i64::MAX` et se replier en date négative ou passée. Corrigé en `i128` saturé avec test dédié.
3. **Émission quotidienne redondante d'`InputChanged` dans `quartier_10ans.rs` (détecté au Commit 5 → corrigé au Commit 7 `f2fe9c8`)** : `S0b` traitait 18 250 événements (un par jour et par groupe) au lieu de 5, polluant le journal et les causes. Corrigé : `S0b` ne traite plus que 5 événements sur 10 ans.
4. **Hors périmètre B1 (consigné sans toucher au code)** : les deux anomalies de `credit.rs` (double division par 1000 dans `RepaymentSchedule::new` L103/L113 et absence de vérification des échéances payées dans `Loan::check_default` L277) sont ouvertes dans `jeu/Docs/tickets/credit-repayment-default-fix.md`.

---

## 5. Demande de validation humaine (règle 34 — situations concrètes)

Sur ce qui se passe concrètement dans le quartier pendant 10 ans avec ce nouveau mécanisme :

1. **Un quartier sans crise mais aux services tout juste moyens (`S0b`)** : aucun groupe ne manque de nourriture, mais les familles les plus modestes peinent lentement sur les prix et rien ne vient les apaiser. Sur 10 ans, aucune grève n'éclate dans cet exemple (même si un risque modéré finit par s'accumuler en fin de décennie : environ une chance sur deux d'en voir une pour les deux groupes les plus pauvres). **Ce calme relatif sur 10 ans sans pénurie te semble-t-il juste, ou est-ce encore trop calme / trop nerveux ?**
2. **Une pénurie sévère mais courte de 3 mois dans un quartier par ailleurs bien équipé (`S1`, `S2`)** : les gens manquent de nourriture pendant l'hiver, la colère monte nettement, puis les approvisionnements reviennent et les services publics calment progressivement la tension jusqu'à ce qu'elle retombe à zéro (ce qui efface aussi la mémoire du risque accumulé). Sur 10 ans, cette crise isolée de 3 mois ne suffit pas à déclencher de grève dans l'exemple observé (risque global d'environ 10 à 15 % pour le groupe le plus touché). **Est-ce cohérent avec le rythme voulu (une crise isolée de 3 mois peut inquiéter, mais reste rarement suivie d'une grève si les services tiennent), ou faudrait-il qu'une rupture totale de 3 mois provoque plus souvent une grève ?**
3. **Une dégradation lente et durable de tout le quartier (`S4`)** : les prix montent, le rationnement s'installe et les services s'effondrent année après année. Pendant les 7 premières années, la colère enfle sans grève ; puis, entre la 7e et la 10e année, **9 grèves** éclatent au total sur les 5 groupes (de 1 à 3 grèves par groupe), durant entre **1 et 7 jours** selon l'organisation du groupe, toujours suivies d'au moins **2 mois de répit** avant qu'un même groupe puisse repartir en grève — sans que la grève ne fasse disparaître la colère elle-même (elle reste au même niveau tant que les conditions ne changent pas). **Ce rythme (délai de plusieurs années avant les premières grèves, 1 à 3 épisodes par groupe dans les années de crise profonde, durées d'1 à 7 jours, répit de 2 mois) correspond-il à ce que tu veux ressentir en jeu ?**
