# Objectif
Branche `feat/t7-t8-griefs-realistes` — **Lot F terminé** (F.3 isolé `751ba87` + F.1/F.2 `eb56c45`, spec `jeu/Docs/spec/T7-T8 correctif - spec v1.3.md`). En attente de validation humaine (règle 34) avant ouverture de la tranche B.

# Décisions
- **F.3 (`@agent3` Mistral Vibe, `751ba87`) :** `FOOD_BUDGET_RENT` et `FOOD_BUDGET_WATER` supprimés dans `jeu/src/salary.rs` ; lecture directe sur `self.panier.logement` et `self.panier.eau` ; `241` tests verts et sorties `S0–S4` à `×3` identiques à l'octet près avant/après ce commit isolé.
- **F.1 + F.2 (`@agent2` GitHub Copilot Fusion, `eb56c45`) :** `MAX_FOOD_PRICE = 600` (`6 × P_base`, pas `+10`/j sous 5 j et `-10`/j à `2 500`, `50` hausses pour atteindre `600`). `S0` et `S3` strictement inchangés. `S1` atteint `P max = 600` (`34` j de rupture), `S2` atteint `P max = 600` (`86` j), `S4` s'équilibre à `P max = 390` (`0` rupture). `241` tests verts, `test_replais` T1–T5 intacts, zéro `f64`/`exp(`.
- **Points ouverts, non traités :** `500` hab. vs `150` cadrage ; pas d'épargne/crédit/arriérés de loyer ; `calculer_taux_risque()` lit encore `salaire_reel` (tranche B) ; épargne / matelas de sécurité (angoisse anticipatoire, ch. 14 : route candidate hors tranche, à ne pas fusionner avec `p`).

# TODO
- [x] **F.3 (`@agent3`)** : commit isolé `751ba87`, zéro changement de comportement.
- [x] **F.1 + F.2 (`@agent2`)** : commit isolé `eb56c45`, `S0`/`S3` inchangés, comparatif `×3` vs `×6` sur `S1`, `S2`, `S4`, `241` tests verts.
- [ ] **Humain** : validation en situations concrètes (règle 34) avant la tranche B.

# Rédacteur actuel
Aucun (Lot F terminé et salon Duo clôturé)

# Destinataire suivant
Humain (validation Lot F)
