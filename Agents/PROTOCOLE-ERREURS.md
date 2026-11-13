# PROTOCOLE ERREURS — journal des solutions et mémoire vive

Objectif : ne pas payer deux fois le même problème, au meilleur coût en tokens. Deux registres distincts, une règle économique unique.

## 1. Le journal des solutions (registre froid) — `Agents/solutions.md`

Une entrée par problème rencontré et résolu :

```
### P-<n> — <symptôme court>
- Symptôme : message d'erreur exact / observable
- Contexte : fichier, palier, couche, commande
- Cause : racine identifiée (pas seulement le contournement)
- Résolution : ce qui a résolu, appliqué tel quel
- Occurrences : <nombre> (dates)
- Statut MV : froid / promu (date) / rétrogradé (date)
```

Règles de manipulation :
- **Lecture : uniquement sur incident** (section 2). Jamais en début de session, jamais par défaut.
- **Écriture : uniquement à la résolution d'un problème** qui a coûté une vraie investigation ou une tentative ratée. Une erreur triviale (coquille relevée du premier coup) n'entre pas au journal.
- L'entrée est complète au moment où le problème est résolu, pas après coup de mémoire.

## 2. Conduite en cas d'erreur

1. L'erreur survient : **avant toute deuxième tentative de correction, consulter le journal** et chercher par symptôme (une recherche textuelle suffit).
2. Entrée correspondante trouvée : appliquer la résolution, incrémenter le compteur d'occurrences.
3. Problème résolu sans entrée existante : ajouter l'entrée (format ci-dessus).
4. **Deux échecs consécutifs malgré le journal : escalade.** On arrête de tentative, on écrit un diagnostic : symptôme, hypothèses testées, résultats, question précise pour l'humain.
5. Après résolution : incrémenter/ajouter l'entrée, puis évaluer la promotion (section 3).

## 3. Promotion en mémoire vive

La mémoire vive est **lue en entier à chaque session JEU** (voire `AGENTS.md` section 4). Elle n'accueille donc que des entrées rentables.

Évaluation pour chaque problème résolu :

- **Coût** C ≈ nombre de tokens de l'entrée, payés une fois par session.
- **Gain** G = tokens évités par incident (re-lectures de docs, investigation, tentatives ratées) × fréquence estimée d'incidents par session.
- **Promouvoir si G > C.** En pratique :
  - Problème récurrent (3 occurrences et plus) avec une résolution stable : promouvoir.
  - Problème grave non récurrent mais à très fort gain par incident (blocage long, erreur coûteuse) : promouvoir.
  - Problème résolu par un changement structurel qui l'empêche définitivement de revenir : rester au journal froid, voire retirer l'entrée de la MV.
- La promotion met à jour le champ `Statut MV` de l'entrée du journal.
- La mémoire vive peut aussi accueillir l'entrée sans attendre la 3e occurrence si l'entrée du journal est déjà longue à relire : la version MV est alors **condensée** (symptôme-clef + résolution en une ou deux lignes).

## 4. Rétrogradation et purge

- Une entrée de la MV sans incident constaté depuis un audit significatif de sessions : rétrogradée au journal froid, champ `Statut MV` mis à jour.
- Une entrée devenue fausse (API changée, code supprimé, règle modifiée) : corrigée ou supprimée dans les deux registres.
- Audit de la MV à chaque enrichissement : si le fichier total coûte plus qu'il ne rapporte, rétrogradation des entrées les moins rentables d'abord.

## 5. Généralisation : la mémoire économique pour toute information

Le principe s'étend au-delà des erreurs. **Toute information apprise pendant le travail** (emplacement d'un fichier, invariant déjà arbitré, résumé d'une section du cadrage, raccourci de navigation, décision d'architecture) est candidate à la mémoire vive, sous le même test :

> L'entrée entre en mémoire vive uniquement si son gain attendu en tokens dépasse son coût.

- Format de l'entrée d'information dans `memoire-vive.md` :
  ```
  ### <intitulé court>
  - Info : <l'information, condensée>
  - Gain : <tokens/session ou /incident évités, chiffrage bref>
  - Coût : ~<tokens>
  - Origine : <fichier ou session d'origine>
  ```
- Une information déjà dans un document court lu à chaque session (ce routeur, les conventions) n'est **pas** dupliquée : on ne paie jamais deux fois la même information.
- À l'inverse, un résumé ne remplace pas son document source : s'il rend le document source inutile, on ne conserve que le résumé ; sinon le résumé doit dire quand rouvrir la source.

## 6. Cohérence avec le reste du système

- Une règle globale candidate (pas une info, une règle) suit le mode PILOTAGE (test économique de `Agents/PILOTAGE.md` section 3), pas la mémoire vive.
- Une information utile à un seul palier va dans la spec du palier, pas dans la MV.
- Le journal et la MV ne sont pas des journaux d'activité : si une information n'économise rien à personne, elle n'y entre pas.
