# Objectif
Tranche B1 — Grève par risque cumulé sur la branche `feat/t7-t8-greve-risque-cumule` (depuis `e10b60e`, commit spec/docs `aa48601`).
Spec verrouillée : `jeu/Docs/spec/T7-T8 tranche B1 - spec.md`. Ticket séparé (ne pas corriger en B1) : `jeu/Docs/tickets/credit-repayment-default-fix.md`.

# Décisions
- **Salon Duo :** clôturé (`room_id = 47d5aa23-9b73-4ba6-9e4a-fef79658c5a6`, `last_seq = 20`) — Tranche B1 livrée (`f2fe9c8`) et prête pour validation humaine.
- **Spécialisation stable de session (`session_profile`) :**
  - `@pilote` (Antigravity `Orchestrator (Neon)`) : lecture seule sur `jeu/src/`, découpage, contrôle, maintien de `etat.md`.
  - `@agent3` (Mistral Vibe `mistral-large-4-0-3`) : tâches complexes `N2–N3` (`Rédacteur actuel` sur Commits 1, 2, 3).
  - `@agent2` (GitHub Copilot `gpt-4.1` `0x`) : tâches simples/audits `N1–N2`, relecture lecture seule, puis Commit 4.
  - `@agent1` (Cursor `auto`) : en veille / escalade `N4` uniquement.
- **Règles dures B1 :** replais `jeu/src/replai/*.json` (`T1–T5`) en lecture seule (STOP immédiat si `test_replais` échoue) ; ne toucher ni `jeu/src/labor.rs` ni `jeu/src/credit.rs` ; Option A (pas de perte de salaire) provisoire ; zéro `f64`/`f32`/`exp`/`ln` ; ne pas démarrer B2 ni B3 ; passer `-c user.name="florian Marquis" -c user.email="florianmarquis91@gmail.com"` si besoin à `git commit` et ne jamais toucher `etat.md` côté agents.

# TODO
- [x] Branche `feat/t7-t8-greve-risque-cumule` créée depuis `e10b60e` + spec verrouillée `jeu/Docs/spec/T7-T8 tranche B1 - spec.md` + ticket `credit.rs` (`aa48601`).
- [x] **Commit 1 (`193d13b`, validé Pilote & relu `@agent2`)** : module de calcul pur dans `jeu/src/strike.rs` + 9 tests B1 (`cargo test` et `test_replais` verts, `LN1P_TABLE[257]` exacte).
- [x] **Commit 2 (`9c120ae`, validé Pilote)** : état canonique, événements 71–74 et planification (`griefs.rs`, `lib.rs`, `replai_gen.rs`, `strike.rs`, `tests/test_tranche8.rs` 20/20).
- [x] **Commit 3 (`f8e6c69`, validé Pilote)** : journal causal top-5 causes et `verify_i7` sur `(timestamp, phase)` (`entities.rs`, `strike.rs`, `tests/test_tranche8.rs`, `cargo test` & `test_replais` 3/3 verts, T2 inchangé).
- [x] **Commit 4 (`9fb20e1`, validé Pilote)** : suppression `taux_risque`/`seuil_risque`/`calculer_taux_risque`/`seuil_franchi` (`salary.rs`, `strike.rs`, `tests/test_tranche7.rs`), synchro `is_on_strike`/`self.strikes`, test garde-fou + compléments §6.1 (`tests/test_tranche8.rs` 15/15), `git rm jeu/src/bin/benchmark_t7_t8.rs` + retrait `Cargo.toml` (`test_replais` 3/3 vert).
- [x] **Commit 5 (`24ee5a2`, livré `@agent3`, audité Pilote)** : scénarios de lecture `S0b`, `S1`, `S2`, `S4` sur 10 ans (`quartier_10ans.rs`) + non-régression déterministe (`10/10`, `test_replais` 3/3).
- [x] **Commit 6 (`37827f1`, validé Pilote)** : saturation `i128` de `add_slots` et `slots_between` contre le débordement (`jeu/src/strike.rs` + test `test_b1_add_slots_saturation_sans_debordement`).
- [x] **Commit 7 (`f2fe9c8`, validé Pilote)** : émission d'`InputChanged` uniquement sur changement réel dans `quartier_10ans.rs`, capture exacte des jalons `365`/`1825`/`DAYS`, `assert!(day >= 0)`, 0 warning (`252/252` tests verts, `test_replais` 3/3).

# Rédacteur actuel
Aucun (Tranche B1 prête pour validation humaine)

# Destinataire suivant
Utilisateur (validation humaine B1 selon règle 34)

# Configuration active (Pilote & Agents)
- `room_id` : `47d5aa23-9b73-4ba6-9e4a-fef79658c5a6` (clôturé à `last_seq = 20`)
- `@pilote` : `antigravity` (`gemini-pro-orchestrator-neon` / `Orchestrator (Neon)`)
- `@agent1` : `cursor` (`auto` — veille/escalade N4)
- `@agent2` : `copilot` (`gpt-4.1` `0x` — relecture lecture seule / Commit 4)
- `@agent3` : `vibe` (`mistral-large-4-0-3` — complexe N2–N3)
