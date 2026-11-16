# Index des Spécifications Techniques (`docs/specs/`)

Ce dossier regroupe les spécifications formalisées issues de la traduction des intentions de gameplay (Game Design Intent) par l'Agent Architecte avant découpage en tickets dans `jeu/Docs/tickets/`.

---

## 1. Gabarit de Spécification d'Intention

Chaque nouvelle intention de gameplay traduite en spec suit cette structure :

```markdown
# SPEC-<ID> — <Titre court>

- **Statut** : Proposition | Validée | Découpée en tickets
- **Palier cible** : P1 | P1b | P1.5 | P2 | P3
- **Intention brute** : « <citation de l'utilisateur> »
- **Références Cadrage** : `docs/cadrage vestiges horizons - v7.8.md` (§...)

## 1. Évaluation de cohérence
- Alignement avec le GDD, les routes approuvées et le gel du palier courant.
- Décisions ouvertes ou arbitrages requis (convention 34 si décision humaine nécessaire).

## 2. Règles de calcul (déterminisme)
- Arithmétique entière / virgule fixe, unités, échelles et politique d'arrondi déclarée (convention 26).
- Clés stables des tirages et bornes.

## 3. Impacts sur l'état canonique et les événements
- Entités et champs modifiés ou ajoutés.
- Événements entrants / sortants, phase (1–7) et horodatage `(jour, créneau)`.
- Variables agrégées partagées lues / écrites.

## 4. Audit technique (lecture seule `jeu/src/`)
- Modules existants concernés et contraintes constatées dans le code.
- Invariants impactés (I1–I9).

## 5. User Stories et critères d'acceptation headless
- US-1 : ... (critères mesurables, tests headless requis).

## 6. Découpage en tickets cibles (après validation)
- Références vers les tickets créés dans `jeu/Docs/tickets/` ou `jeu/Docs/spec/`.
```

---

## 2. Spécifications actives et tickets dans `jeu/` (chemins uniquement)

Les spécifications d'exécution et tickets destinés aux agents de développement restent dans `jeu/Docs/` (ne pas dupliquer ici) :

- Spécification P1 (quartier vivant) : `jeu/Docs/spec P1 - quartier vivant.md`
- Spécification P1b (scène locale et immersion) : `jeu/Docs/spec/P1b - scene locale et immersion - spec.md`
- Correctif T7–T8 (tranche A validée techniquement) : `jeu/Docs/spec/T7-T8 correctif - spec v1.2.md`
- Ordre de mission T7–T8 tranche A2 : `jeu/Docs/spec/T7-T8 tranche A2 - ordre de mission.md`
- Ticket correctif flottants `labor.rs` : `jeu/Docs/tickets/labor-f64-fix.md`
- Ticket tests de non-régression replais : `jeu/Docs/tickets/replais - test de non-regression.md`
