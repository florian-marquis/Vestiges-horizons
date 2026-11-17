# Objectif
Branche `feat/t7-t8-griefs-realistes` — **Lot H** (lot G clos : G1 `bf6f41b`, G2 `6501e43`, suivi `89a4ffd`).
- **H1 (Pilote, lecture seule)** : rapport complet S1/S2/S4 après G1+G2 ; explication de l'écart « jours de rupture marché » S2.
- **H2 (code, Duo)** : scénario **S0b** dans `jeu/src/bin/quartier_10ans.rs` — 10 ans, graine 42, aucun choc, mêmes données figées que S0, `q = 500` constant pour toutes les cohortes (soins et logement fixés à 500). S0 inchangé. Règle d'arrêt : une cohorte à `G = 100` en moins de 10 ans sans choc → STOP, ne rien retoucher.
- **H3 (documents)** : `etat.md` à jour (ce fichier), puis annexe de `jeu/Docs/spec/T7-T8 correctif - spec v1.3.md` (S0b + résultats du lot G).

# Décisions
- H2 est une micro-tâche unique, un seul commit : `feat(t7-t8): ajoute le scenario S0b (q=500 constant)`.
- Rédacteur unique : `@agent2` (Copilot Fusion). `@agent3` en audit lecture seule ; `@agent1` (Cursor) en secours uniquement.
- Règles dures inchangées : replais `jeu/src/replai/*.json` en lecture seule, pas de tranche B, entiers uniquement, ne pas toucher `strike.rs`, `labor.rs`, `meal.rs`, `calculer_taux_risque`, I8, `Market::auto_reprice()`, ni les sorties S0–S4 existantes.

# TODO
- [x] Lot G clos (G0, G1, G2).
- [x] H1 — rapport complet (Pilote) ; écart S2 = erreur de relevé (86 jours partout).
- [x] H2 — S0b commité (`2bbc49f`) après relecture positive d'Agent 3 ; 242 tests verts et sorties S0–S4 identiques octet pour octet.
- [x] H3 — annexe spec v1.3 + etat.md.

# Rédacteur actuel
Inactif sur le code — H2 terminé, en attente de validation du Pilote.

# Destinataire suivant
@pilote (validation/clôture H2, puis H3)

