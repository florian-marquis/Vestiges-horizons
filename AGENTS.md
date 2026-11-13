# AGENTS.md — Routeur des agents du dépôt « Vestiges horizons »

Ce fichier est le point d'entrée unique des agents IA travaillant dans ce dépôt. Il ne décrit pas comment développer : il dit où sont les instructions et quand les charger.

## 1. Les deux projets, deux modes

| Mode | Projet | Instructions à charger | Ne pas toucher |
|------|--------|------------------------|----------------|
| **PILOTAGE** | Le système d'instruction des agents eux-mêmes (protocoles, processus, conventions) | `Agents/PILOTAGE.md` | Le code, les specs et les docs du jeu |
| **JEU** | Le développement du jeu « Vestiges horizons » 100% en mode agent | `Agents/DEV-JEU.md`, puis `Agents/memoire-vive.md` | Les fichiers du dossier `Agents/` (sauf enrichissement prévu par protocole) |

## 2. Commutation de mode

1. **Le mode n'est choisi que par l'utilisateur.** L'agent ne bascule jamais de lui-même d'un projet à l'autre.
2. Basculer vers JEU : « on repasse sur le jeu », « reprend le jeu », ou toute tâche sur le code et les docs du jeu.
3. Basculer vers PILOTAGE : toute demande portant sur les instructions, processus ou protocoles des agents.
4. Une commutation explicite met à jour `Agents/etat-mode.md` (une ligne, voir ce fichier).
5. À l'ouverture d'une session sans mode déclaré : lire `Agents/etat-mode.md` et reprendre le mode indiqué, sans redemander confirmation.
6. En cas d'ambiguïté sur la frontière entre les deux projets : rester dans le mode courant et poser une seule question précise à l'utilisateur.

## 3. Hiérarchie des documents de référence (mode JEU)

En cas de conflit, dans l'ordre décroissant :
1. Le cadrage `Docs/cadrage vestiges horizons - v7.7.md` (source de vérité du design)
2. Les `Docs/Conventions du projet (pour agents).md` (règles d'exécution)
3. La spécification du palier en cours (`Docs/spec P1 - quartier vivant.md`, puis les suivantes)
4. Les instructions du dossier `Agents/`

## 4. Mémoire : les deux registres

- **Mémoire vive** : `Agents/memoire-vive.md` — chargée en début de chaque session JEU. Ne contient que des entrées dont le gain en tokens dépasse leur coût (règle économique dans `Agents/PROTOCOLE-ERREURS.md`, section 5).
- **Journal des solutions** : `Agents/solutions.md` — registre froid des problèmes résolus. **Jamais chargé en contexte**, sauf en cas d'erreur ou pour enrichissement (`Agents/PROTOCOLE-ERREURS.md`, sections 1 et 2).

## 5. Règles transverses (tous modes)

- Langue de travail et des documents : français.
- Toute instruction ajoutée aux fichiers de `Agents/` obéit au même test économique que la mémoire vive : son gain doit dépasser son coût en tokens à chaque session (détail dans `Agents/PILOTAGE.md`).
- Ne jamais créer de fichier résumant un autre fichier sans le test économique précédent ; sinon on duplique et on paie le double.
