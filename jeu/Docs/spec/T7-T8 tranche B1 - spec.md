# Vestiges Horizons — Spécification Tranche B1 : Grève par risque cumulé

**Date :** 19/11/2026  
**Statut :** spécification verrouillée (feux verts + 9 arbitrages intégrés) — prête pour implémentation en salon Duo  
**Branche cible :** `feat/t7-t8-greve-risque-cumule` (créée depuis `e10b60e`)  
**Base de design :** `docs/cadrage vestiges horizons - v7.8.md` (révision `v7.8.2`), `jeu/Docs/Conventions du projet (pour agents).md` (v2.3), `jeu/Docs/spec P1 - quartier vivant.md` (§20 et §35), `jeu/Docs/spec/T7-T8 correctif - spec v1.3.md`  
**Régime de salaire pendant la grève :** **Option A (pas de perte de salaire) marquée PROVISOIRE** — `jeu/src/credit.rs` n'est ni raccordé aux cohortes ni alimenté par une trésorerie de foyer dans ce palier (voir ticket séparé `jeu/Docs/tickets/credit-repayment-default-fix.md`).

---

## 1. Règles dures et périmètre

1. **Replais de référence en lecture seule (règles 6 et 16) :** `jeu/src/replai/*.json` (`T1–T5`) et `jeu/tests/replais_hashes.txt` ne doivent jamais être modifiés ni régénérés. Si `cargo test --test test_replais` échoue → **STOP immédiat**, lister les replais touchés et attendre validation humaine.
2. **Modules hors périmètre (interdiction de modification) :** ne toucher ni `jeu/src/labor.rs` ni `jeu/src/credit.rs`. Ne pas démarrer B2 (arrêt d'offre de travail dans `labor.rs`) ni B3 (réponses `Q(a)` des firmes).
3. **Arithmétique 100 % entière et unicité (règle 26) :** zéro `f64`, zéro `f32`, zéro `exp()`, zéro `ln()` dans le moteur de simulation (`strike.rs`, `griefs.rs`, `salary.rs`). Tout arrondi par division entière est explicitement déclaré (troncature vers zéro ou plafond entier).
4. **Moteur purement événementiel :** aucune boucle quotidienne appelant `declencher_greve` ou vérifiant la fin de grève jour par jour. Coût proportionnel au nombre d'événements, indépendant de l'horizon en jours.
5. **Retrait de `benchmark_t7_t8` :** supprimer `jeu/src/bin/benchmark_t7_t8.rs` du dépôt (`git rm`) et son entrée `[[bin]]` dans `jeu/Cargo.toml`.
   - *Motif consigné :* binaire hérité d'avant v1.3 utilisant des séries synthétiques `f64`/`.sin()`, l'ancienne règle dure `seuil_franchi()` et un appel quotidien à `declencher_greve()`, remplacé par les scénarios réalistes dans `jeu/src/bin/quartier_10ans.rs`.
6. **Tickets séparés à ne pas corriger en B1 :**
   - `jeu/src/credit.rs:103,113` (`RepaymentSchedule::new`) : double division par `1000`.
   - `jeu/src/credit.rs:277` (`Loan::check_default`) : passage en défaut sans vérifier si l'échéance a été payée.
   - Voir `jeu/Docs/tickets/credit-repayment-default-fix.md`.

---

## 2. Suppression de l'ancienne règle dure et test garde-fou

1. **Suppression complète dans `jeu/src/salary.rs` :**
   - Retirer les champs `taux_risque`, `seuil_risque` et `date_franchissement` de `SalaireReelEtGriefs` (L82–86).
   - Retirer `calculer_taux_risque()`, `seuil_franchi()`, `set_seuil_risque()` ainsi que leurs appels (L161, L173, L186, L192–216).
   - Le salaire réel (`salaire_horaire`, `salaire_reel`) n'entre dans le risque de grève **qu'à travers `calculate_p` → `InputChanged` → `GriefsState` (`G`) → `λ`**.
   - Mettre à jour `jeu/tests/test_tranche7.rs` (`test_t7_5_seuil_risque_cumule`, `test_t7_6_greve_seulement_si_salaire_reel_bas`) et le test interne de `salary.rs` (`test_t7_5_seuil_risque`) pour vérifier l'absence de règle dure et le canal unique par `p → G`.
2. **Suppression dans `jeu/src/strike.rs` :**
   - Retirer l'ancien `StableRng`, l'ancien `draw_counter` global partagé entre cohortes, `planifier_franchissement(..., taux_risque, seuil_risque)` et `declencher_greve(cohort, day)` appelable à la volée en boucle quotidienne.
3. **Test garde-fou statique (`jeu/tests/test_tranche8.rs`) :**
   - Scanne récursivement tous les fichiers `.rs` sous `jeu/src/` (en excluant uniquement les chemins figurant dans `const DISPLAY_MODULE_ALLOWLIST: &[&str] = &[];`, réservée à un futur dossier `jeu/src/display/`).
   - Échoue si l'un des identifiants `taux_risque`, `seuil_risque`, `calculer_taux_risque` ou `seuil_franchi` apparaît hors commentaires dans le code moteur.

---

## 3. Modèle mathématique entier et arrondis déclarés (règle 26)

### 3.1 Constantes et unités canoniques

| Grandeur | Symbole / Constante | Valeur | Unité |
| --- | --- | --- | --- |
| Échelle du risque cumulé | `HAZARD_SCALE: u128` | `1_000_000_000_000` ($10^{12}$) | picorisques (`1.0` = $10^{12}$) |
| Taux maximal annuel | `LAMBDA_MAX_MILLE: u64` | `2_000` | millièmes d'an⁻¹ (`LAMBDA_MAX = 2,0`) |
| Organisation par défaut | `DEFAULT_ORGANISATION_MILLE: [u16; 5]` | `[200, 400, 600, 600, 400]` | millièmes (`[0,2 ; 0,4 ; 0,6 ; 0,6 ; 0,4]`) |
| Répit post-grève | `STRIKE_COOLDOWN_DAYS: i64` | `60` (`240` créneaux) | jours |
| Créneaux par jour / par an | `SLOTS_PER_DAY = 4`, `SLOTS_PER_YEAR = 1_460` | `4` / `1_460` | créneaux (`365 × 4`) |
| Plafond de `G` | `MAX_GRIEFS_QMICRO: i64` (`M`) | `400_000_000` | quarts de micro-point (`100` pts) |
| Dénominateur analytique | `HAZARD_INTEGRAL_DENOM: u128` | `700_800_000_000_000` | $(3 \cdot 1460 \cdot 10^6 \cdot M^2) / 10^{12}$ |
| $\ln(2)$ à l'échelle | `LN2_SCALED: u64` | `693_147_180_560` | $\text{round}(\ln(2) \cdot 10^{12})$ |

### 3.2 Clé stable et tirage déterministe (SplitMix64)

- **Compteur unique par cohorte (`draw_index: u64`)** : propre à chaque cohorte (indépendant de l'ordre d'enregistrement ou de traitement des autres cohortes). Initialisé à `0`, il est incrémenté **à chaque nouveau tirage de seuil** (crise initiale, après remise à zéro à `G = 0`, ou en fin de grève pour préparer la crise suivante).
- Pour une même crise naissante, l'indice actif `active_draw_index: u64` alimente à la fois la clé du seuil (`TAG_GREVE_SEUIL`) et celle de la durée (`TAG_GREVE_DUREE`), sans jamais être réutilisé ni re-tiré lors d'un `InputChanged` ou d'une butée `G = 100`.
- **Fonction de hachage pure à clé stable (`stable_u64`)** : mélange par `SplitMix64` (zéro allocation de chaîne, bijection 64 bits) de :
  ```text
  (world_seed: u64, cohort_id.to_u64(): u64, domain_tag: u64, draw_index: u64)
  ```
  avec `TAG_GREVE_SEUIL: u64 = 0x4752_4556_455F_5345` et `TAG_GREVE_DUREE: u64 = 0x4752_4556_455F_4455`.

### 3.3 Seuil exponentiel $E = -\ln(u)$ sans `f64` ni `exp`/`ln`

1. Soit `raw: u64 = stable_u64(world_seed, cohort_id, TAG_GREVE_SEUIL, draw_index)`.
2. On forme l'entier 65 bits `raw_65: u128 = (u128::from(raw) << 1) | 1`, qui représente le point milieu $u = \text{raw\_65} / 2^{65} \in (0, 1)$ (strictement positif et strictement inférieur à `1`).
3. Réduction binaire exacte $u = 2^{-k}(1 + f)$ avec $f \in [0, 1)$ :
   - Si `raw == 0` : $k = 65$, `frac64 = 0` $\Rightarrow E_{\text{scaled}} = 65 \cdot \text{LN2\_SCALED}$.
   - Sinon : $k = \text{raw.leading\_zeros()} + 1 \in [1, 64]$, `shifted = raw_65 << (k - 1)`, et `frac64 = (shifted & 0xFFFF_FFFF_FFFF_FFFF) as u64`.
4. Calcul de $\ln(1 + f)$ par table statique `LN1P_TABLE: [u64; 257]` précalculée ($\text{LN1P\_TABLE}[i] = \text{round}(\ln(1 + i/256) \cdot 10^{12})$ pour $i \in 0..=256$, avec $\text{LN1P\_TABLE}[0] = 0$ et $\text{LN1P\_TABLE}[256] = \text{LN2\_SCALED}$) et interpolation linéaire entière :
   - `idx = (frac64 >> 56) as usize` (`0..=255`)
   - `rem = (frac64 >> 24) & 0xFFFF_FFFF` (`0..2^32`)
   - **Arrondi déclaré (règle 26 — troncature entière vers zéro) :**
     ```text
     delta = LN1P_TABLE[idx + 1] - LN1P_TABLE[idx]
     ln1p  = LN1P_TABLE[idx] + (((u128::from(delta) * u128::from(rem)) >> 32) as u64)
     E_scaled = (u64::from(k) * LN2_SCALED).saturating_sub(ln1p).max(1)
     ```
   - `E_scaled >= 1` garanti (aucun déclenchement à durée nulle).

### 3.4 Intégrale analytique entière du taux de risque et franchissement

Entre deux instants $t_0$ et $t_0 + k$ ($k \ge 0$ créneaux) sans saturation intermédiaire, $g(t) = g_0 + s \cdot t$ varie linéairement entre `0` et `M = 400_000_000` qµpt, avec pente $s \in [-30\,000, +230\,000]$ qµpt/créneau :

$$\lambda_{\text{slot}}(g) = \frac{\text{LAMBDA\_MAX\_MILLE}}{1000 \cdot 1460} \cdot \frac{o_{\text{mille}}}{1000} \cdot \left(\frac{g}{M}\right)^2$$

1. **Intégrale $I(k)$ sur $k$ créneaux :**
   - Si $s = 0$ : $I(k) = 3 \cdot k \cdot g_0^2$.
   - Si $s \neq 0$ et $k \le k_{\text{full}}$ (où $k_{\text{full}} = (M - g_0) / s$ si $s > 0$, ou $g_0 / |s|$ si $s < 0$, troncature entière ; aucun écrêtage) :
     $$g_1 = g_0 + k \cdot s, \qquad I(k) = k \cdot (g_0^2 + g_0 g_1 + g_1^2)$$
   - Si $s \neq 0$ et le segment atteint la butée `bound` ($M$ si $s > 0$, $0$ si $s < 0$) avec un dernier créneau partiellement écrêté ($k_{\text{bound}} = \lceil \text{distance} / |s| \rceil = k_{\text{full}} + 1$) :
     - Sur $[0, k_{\text{full}}]$ : $I(k_{\text{full}})$ avec $g_{\text{full}} = g_0 + k_{\text{full}} \cdot s$.
     - Sur le créneau final $k_{\text{full}} \to k_{\text{bound}}$ : ajout de $(g_{\text{full}}^2 + g_{\text{full}} \cdot \text{bound} + \text{bound}^2)$ (linéaire entre $g_{\text{full}}$ et `bound`).
     - Pour tout $k > k_{\text{bound}}$ : si `bound == 0`, contribution nulle après $k_{\text{bound}}$ ; si `bound == M`, ajout de $3 \cdot (k - k_{\text{bound}}) \cdot M^2$.
2. **Conversion à l'échelle `HAZARD_SCALE` — Arrondi déclaré (règle 26 — troncature entière vers zéro en `u128`) :**
   ```text
   delta_h_scaled(k) = (u128::from(LAMBDA_MAX_MILLE) * u128::from(o_mille) * I(k)) / HAZARD_INTEGRAL_DENOM
   ```
   *(Preuve sans débordement : $I(k) \le 3 \cdot k \cdot 1{,}6 \cdot 10^{17}$ ; jusqu'à $k = 10^9$ créneaux ($\approx 685\,000$ ans), le numérateur reste $\le 10^{33} \ll \texttt{u128::MAX} \approx 3{,}4 \cdot 10^{38}$ ; garde `checked_mul` saturant au franchissement au-delà.)*
3. **Calcul du plus petit créneau de franchissement $k \ge 1$ pour $H_{\text{rem}} = E_{\text{scaled}} - H_{\text{cum}} > 0$ :**
   - **Si $g_0 = 0$ et $s \le 0$**, ou **$o_{\text{mille}} = 0$** : aucun franchissement possible (`None`).
   - **Si $s = 0$ ($g_0 > 0, o_{\text{mille}} > 0$, incluant le plateau $G = 100$)** — formule fermée en $O(1)$, **arrondi déclaré : plafond entier** :
     $$A = \text{LAMBDA\_MAX\_MILLE} \cdot o_{\text{mille}} \cdot 3 \cdot g_0^2$$
     $$k_{\text{cross}} = \frac{H_{\text{rem}} \cdot \text{HAZARD\_INTEGRAL\_DENOM} + A - 1}{A} \ge 1$$
   - **Si $s < 0$ ($g_0 > 0$)** :
     - $k_{\text{bound}} = (g_0 + |s| - 1) / |s|$ (plafond entier, borné à $\le 4 \cdot 10^8$).
     - Si `delta_h_scaled(k_bound) < H_rem` : la cohorte n'atteint pas le seuil avant `G = 0` → planifier `GriefBoundReached(0)` à `t0 + k_bound`.
     - Sinon : **bissection entière** sur `[1, k_bound]` ($\le 29$ itérations) retournant le plus petit `k` tel que `delta_h_scaled(k) >= H_rem` → planifier `StrikeTriggered` à `t0 + k`.
   - **Si $s > 0$ ($g_0 < M$)** :
     - $k_{\text{bound}} = ((M - g_0) + s - 1) / s$ (plafond entier, borné à $\le 4 \cdot 10^8$).
     - Si `delta_h_scaled(k_bound) >= H_rem` : **bissection entière** sur `[1, k_bound]` ($\le 29$ itérations) retournant le plus petit `k` tel que `delta_h_scaled(k) >= H_rem` → planifier `StrikeTriggered` à `t0 + k`.
     - Sinon : planifier `GriefBoundReached(M)` à `t0 + k_bound`. Lorsque cet événement arrive, on crédite `H_cum += delta_h_scaled(k_bound)` puis on passe sur le cas $s = 0, g_0 = M$ qui planifie `StrikeTriggered` en $O(1)$ sans nouveau tirage.

### 3.5 Durée tirée à clé stable

- **Arrondi déclaré (règle 26 — troncature entière vers zéro) :**
  ```text
  max_duration_days = 1 + (20 * u64::from(o_mille)) / 1_000
  ```
  Soit pour $O = [200, 400, 600, 600, 400]$ : durées maximales `[5, 9, 13, 13, 9]` jours.
- **Tirage :**
  ```text
  raw_dur = stable_u64(world_seed, cohort_id.to_u64(), TAG_GREVE_DUREE, active_draw_index)
  duration_days = (1 + (raw_dur % max_duration_days)) as i64
  ```

---

## 4. Machine à états, événements, règle 18 et ordonnancement

### 4.1 États d'une cohorte (`StrikePhaseState`)

1. **`Dormant`** : aucun seuil actif (`active_threshold = None`, `h_cum_scaled = 0`), aucune accumulation (`G == 0` et `slope <= 0`, ou `o_mille == 0`).
2. **`Accumulating`** : seuil actif (`active_draw_index`, `active_threshold`), `h_cum_scaled < active_threshold`, soit `G > 0` soit `slope > 0`. Prochain événement planifié : `StrikeTriggered` ou `GriefBoundReached`.
3. **`PendingNextSlot`** : franchissement détecté pendant un `InputChanged` au créneau `ts` (arbitrage 4 / règle 18) ; `StrikeTriggered` déjà planifié à `next_slot(ts)` en `Phase::Social`. Aucun nouveau recalcul ne peut annuler ce déclenchement imminent.
4. **`OnStrike`** : grève en cours (`strike_index` active), fin planifiée à `(day + duration_days, seq)` par `StrikeEnded(DurationElapsed)` (ou fin anticipée via `StrikeEndCause::Concession`). Aucun tirage actif en attente (`verify_no_redraw` garanti vrai).
5. **`Cooldown`** : répit de **`STRIKE_COOLDOWN_DAYS = 60`** jours (`cooldown_end = end_ts + 240 créneaux`). Nouveau seuil immédiatement tiré à la fin de grève (`draw_index` incrémenté, `h_cum_scaled = 0`), mais l'intégration de `H_cum` reste gelée (`0`) jusqu'à `cooldown_end`. À `StrikeCooldownEnded` (si `G > 0` ou `slope > 0`), reprise de la planification sur le seuil déjà tiré ; sinon retour en `Dormant` ? *Précision : si `G == 0 && slope <= 0` à l'issue du répit, on garde le seuil tiré en `Cooldown` terminé ou on applique la règle de remise à zéro si `G == 0` sans grève ; concrètement on passe en `Accumulating` si `G > 0 || slope > 0`, sinon `Dormant` (remise à zéro car `G == 0` sans grève).*

### 4.2 Nouveaux événements et identifiants stables (arbitrage 9)

Dans `jeu/src/lib.rs`, ajouter en `Phase::Social` (priorités déterministes ordonnées par phase causale au sein d'un créneau) :
- `EVENT_TYPE_GRIEF_BOUND_REACHED = 71` (`Priority(0)`) → `EventData::GriefBoundReached { entity_id, bound_qmicro, schedule_version }`
- `EVENT_TYPE_STRIKE_COOLDOWN_ENDED = 72` (`Priority(1)`) → `EventData::StrikeCooldownEnded { entity_id, schedule_version }`
- `EVENT_TYPE_STRIKE_TRIGGERED = 73` (`Priority(2)`) → `EventData::StrikeTriggered { entity_id, draw_index, strike_index, schedule_version }`
- `EVENT_TYPE_STRIKE_ENDED = 74` (`Priority(3)`) → `EventData::StrikeEnded { entity_id, strike_index, cause, schedule_version }`

**Encodage déterministe d'`Id` (arbitrage 9 — zéro dépendance à l'ordre d'insertion ni compteur partagé) :**
```text
bit 63       : 1 (namespace événements grève/butée, disjoint des ids 0..2^63)
bits 60..62  : kind_code (1=Bound, 2=Cooldown, 3=Trigger, 4=End)
bits 28..59  : cohort_id.number & 0xFFFF_FFFF (tri canonique par cohorte croissante)
bits  0..27  : schedule_version & 0x0FFF_FFFF (unicité par replanification)
```
L'insertion passe par `EventQueue::push(...)` avec cet `Id` déterministe. Tout événement portant un `schedule_version` différent du `schedule_version` courant de la cohorte est ignoré en `O(1)` à la lecture (`noop` sûr).

### 4.3 Règle 18 — Pas d'exécution dans le même instant

- Lors du traitement d'`InputChanged` à `ts = (day, seq)` (`Phase::Comptes`) :
  - On avance `GriefsState` et on intègre `H_cum` sur `[last_update, ts]` avec les anciennes entrées ;
  - Si `H_cum >= E_scaled` est constaté pendant `InputChanged` (ou si la dernière évaluation vient d'atteindre `E_scaled` exactement à `ts`), on **ne déclenche pas** `Strike` dans `ts` : on passe en `PendingNextSlot` et on planifie `StrikeTriggered` à **`next_slot(ts)`** (`(day, seq + 1)` si `seq < 3`, sinon `(day + 1, 0)`), `Phase::Social`.
  - Sinon (`H_cum < E_scaled`), on applique les nouvelles entrées `(u, p, q)`, puis on planifie le prochain événement (`GriefBoundReached` ou `StrikeTriggered`) qui tombe toujours à $k \ge 1$ créneau(x) dans le futur ($\ge \text{next\_slot}(ts)$).
- Au traitement de `GriefBoundReached(0)` sans grève : si `G` vaut `0` à la fin du segment, `h_cum_scaled = 0`, le seuil actif est abandonné (`Dormant`), le suivi des causes est vidé ; un nouveau seuil (nouveau `draw_index`) sera tiré uniquement à la prochaine remontée (`G > 0` ou `slope > 0`).
- Au traitement de `StrikeTriggered` : `GriefsState` est inchangé (`G` conservé), `strike_index += 1`, passage à `OnStrike`, planification unique de `StrikeEnded(DurationElapsed)` à `(day + duration_days, seq)`, écriture dans `CausalJournal`.

---

## 5. Journal causal et invariant I7 (`jeu/src/entities.rs` & `strike.rs`)

### 5.1 Suivi borné des causes par cohorte (arbitrage 5)

- Chaque cohorte maintient un tampon borné d'au plus **5 entrées** `CauseContribution { journal_entry_id: u64, delta_h_scaled: u128 }` accumulées pendant la crise courante (vidé au déclenchement d'une grève et lors d'une remise à zéro à `G = 0`).
- Lorsqu'un segment clôturé par un `InputChanged` (ou jusqu'au déclenchement) attribue $\Delta H > 0$ à l'`entry_id` d'`InputChanged` actif sur ce segment :
  - Si `entry_id` est déjà présent dans le tampon, on additionne son `delta_h_scaled` ;
  - Sinon on l'insère, on trie par **`delta_h_scaled` décroissant, puis `journal_entry_id` croissant**, et on tronque à **5 éléments** (coût mémoire constant $O(1)$).
- Les événements `InputChanged` eux-mêmes sont enregistrés dans `CausalJournal` (`DECISION_INPUT_CHANGED = 70`, `Phase::Comptes as u8`) afin que leurs `entry_id` existent bien dans le journal et vérifient I7.

### 5.2 Contenu obligatoire de l'entrée `StrikeTriggered` (règle 14)

- `timestamp` : `(day, seq)` du déclenchement (`Phase::Social`)
- `event_id` : `event.id.as_u64()`
- `phase` : `Phase::Social as u8` (`6`)
- `actor` : `Some(cohort_id)`
- `decision_type` : `DECISION_STRIKE_TRIGGERED` (`80`) (et `DECISION_STRIKE_ENDED` `81` à la fin de grève)
- `modified_values` :
  - `strike_index` (`0 -> strike_index`)
  - `draw_index` (`0 -> active_draw_index`)
  - `g_qmicro` (`0 -> g_qmicro` au déclenchement)
  - `lambda_mille_per_year` (`0 -> ((LAMBDA_MAX_MILLE * o_mille * g_qmicro^2) / (1000 * M^2))` en `u128`, troncature entière)
  - `threshold_scaled` (`0 -> active_threshold as i64`, tient dans `i64` jusqu'à $9 \cdot 10^{18}$)
  - `planned_duration_days` (`0 -> duration_days`)
- `causes` : liste des `journal_entry_id` du tampon top-5 (au plus 5 IDs)
- `learned_by` : `vec![(cohort_id, day as u64)]` (jour de la cohorte).

### 5.3 Évolution de `verify_i7` (arbitrage 6)

Dans [`jeu/src/entities.rs`](file:///c:/Users/f.marquis/Nextcloud/perso/ia/Vestiges horizons/jeu/src/entities.rs#L620-L717) :
- Dans `CausalJournal::add_entry` et `CausalJournal::verify_i7`, remplacer la condition de rejet :
  `cause.timestamp >= entry.timestamp`
  par :
  `(cause.timestamp, cause.phase) >= (entry.timestamp, entry.phase)`
- Vérifier obligatoirement que `test_replais` (`T2` notamment) et `test_tranche2` restent 100 % verts sans altération de `reference_seed42_t2.json`.

---

## 6. Batterie de tests headless & scénarios de lecture

### 6.1 Tests headless obligatoires (`jeu/tests/test_tranche8.rs` & `strike.rs`)

1. **Calcul pur (Commit 1) :** bijection/déterminisme `stable_u64`, positivité/précision de `−ln(u)` contre points de contrôle connus (sans `f64` dans le moteur), exactitude de `delta_h_scaled` sur segments linéaires / plateau / butées `0` et `M`, bissection exacte (`delta_h_scaled(k) >= H_rem` et `delta_h_scaled(k-1) < H_rem`), durée dans `[1, 1 + 20·O]`.
2. **Déterminisme & permutations (I9, conv. 18) :** même graine → bits identiques ; permutation de l'ordre d'insertion des événements et permutation de l'ordre d'enregistrement des cohortes → dates, seuils et durées strictement identiques.
3. **`G = 0` → zéro grève :** 100 graines sur 100 ans (`36 500` j) à `G = 0`, `O = 1,0` → `0` grève ; idem `O = 0`, `G = 100` → `0` grève.
4. **Monotonie en `G` et en `O` :** sur 50 graines, `date(G=100) <= date(G=80) <= date(G=50) <= date(G=20)` et `date(O=1,0) <= date(O=0,8) <= date(O=0,6) <= date(O=0,4) <= date(O=0,2)`.
5. **Loi exponentielle `E[T] ≈ 1/λ` à `G` constant :** sur `N = 5 000` graines, moyenne empirique dans `1/λ ± 5 %` pour `(G=100, O=0,5)` et `(G=50, O=0,6)` ; idem sur pente linéaire croissante `G(t)`.
6. **Coût `O(nb_événements)` :** nombre d'événements et d'appels de planification identique entre 10 ans et 100 ans pour 1 seul `InputChanged` à $t = 0$.
7. **Durée dans la fourchette & fin planifiée :** 1 000 graines par niveau de `O` → 100 % dans `[1, 1 + floor(20·O)]`, bornes atteintes, fin par `StrikeEnded` sans boucle quotidienne + fin anticipée `Concession`.
8. **Salaire réel absent du déclenchement :** `salaire_horaire ×100` vs `÷10` à `(u, p, q)` identiques → résultats strictement identiques.
9. **Remise à zéro à `G = 0` :** après montée puis redescente complète `GriefBoundReached(0)` sans grève, une seconde montée produit un comportement identique à une crise vierge de même `draw_index` (zéro mémoire de `H_cum` passé).
10. **Répit de 60 jours :** aucune grève ne peut démarrer pendant les `60` jours (`240` créneaux) suivant une fin de grève (`DurationElapsed` ou `Concession`), même à `G = 100` et petit seuil ; vérification que `G` n'est pas modifié par la grève.
11. **Règle 18 (pas d'exécution même-instant) :** un franchissement détecté durant `InputChanged` à `(day, seq)` se déclenche uniquement à `next_slot(day, seq)` en `Phase::Social`, jamais dans le même créneau.
12. **Journal causal et I7 :** tous les champs règle 14 présents, `causes.len() <= 5`, causes triées par `(ΔH DESC, entry_id ASC)`, `learned_by = [(cohort, day)]`, `verify_i7()` vert, `test_replais` T1–T5 vert.
13. **Test garde-fou statique :** échec si `taux_risque`, `seuil_risque`, `calculer_taux_risque`, `seuil_franchi` subsistent hors `DISPLAY_MODULE_ALLOWLIST`.

### 6.2 Scénarios de lecture (`quartier_10ans.rs`, Commit 5)

Exécuter les **4 scénarios `S0b`, `S1`, `S2`, `S4`** sur `10 ans` (`3 650` j), graine `42`, avec le moteur de grève Tranche B1 (`O = [0,2 ; 0,4 ; 0,6 ; 0,6 ; 0,4]`, Option A provisoire, sans modifier les sorties existantes de A2).
- Afficher pour chaque scénario et chaque cohorte (`0..=4`) :
  - **Nombre de grèves observées**
  - **Nombre attendu** = intégrale analytique entière exacte $\sum \Delta H_{\text{scaled}}(k) / 10^{12}$ (formatée à 3 décimales par division entière)
  - **Liste des grèves** : `(jour, créneau)` de déclenchement, durée (jours), et `G` au déclenchement (points/millipoints).
- **Convention 24 :** aucun seuil d'acceptation n'est fixé avant observation humaine.

---

## 7. Ordre strict des 5 commits atomiques

1. **Commit 1 — `feat(t7-t8): ajoute le module de calcul entier du risque cumule`**
   - Fonctions pures dans `jeu/src/strike.rs` (`stable_u64`, `draw_exponential_threshold_scaled`, `LN1P_TABLE`, `delta_h_scaled`, `find_crossing_slot`, `draw_strike_duration_days`) + tests unitaires complets (les anciennes méthodes restent compilables jusqu'au Commit 4).
2. **Commit 2 — `feat(t7-t8): integre l'automate evenementiel de greve par risque cumule`**
   - `griefs.rs` (`next_bound_timestamp`, constantes publiques), `lib.rs` (`EventType`, `EventData`), `replai_gen.rs` (bras de match pour compilation), `strike.rs` (automate d'états, `draw_index`, répit 60 j, remise à zéro `G=0`, règle 18 `next_slot`, encodage `Id` déterministe) + tests `test_tranche8.rs` (tests 2–7, 9–11).
3. **Commit 3 — `feat(t7-t8): journalise les greves et etend verify_i7 a (timestamp, phase)`**
   - `entities.rs` (`verify_i7` + `add_entry`), `strike.rs` (suivi top-5 causes, `CausalJournal`, `learned_by`) + test I7/T2 + règle 14 dans `test_tranche8.rs`. Vérifier `test_replais` (T2 strictement inchangé).
4. **Commit 4 — `refactor(t7-t8): supprime taux_risque/seuil_risque et retire benchmark_t7_t8`**
   - Retrait de `taux_risque`, `seuil_risque`, `date_franchissement`, `calculer_taux_risque`, `seuil_franchi`, `set_seuil_risque` dans `salary.rs` et de l'ancienne API dans `strike.rs` ; mise à jour de `test_tranche7.rs` ; ajout du test garde-fou statique dans `test_tranche8.rs` ; `git rm jeu/src/bin/benchmark_t7_t8.rs` + retrait dans `jeu/Cargo.toml`.
5. **Commit 5 — `feat(t7-t8): ajoute les lectures de greve S0b, S1, S2 et S4 sur 10 ans`**
   - Ajout des lectures `S0b`, `S1`, `S2`, `S4` dans `jeu/src/bin/quartier_10ans.rs` et tests de non-régression déterministe des lectures.
