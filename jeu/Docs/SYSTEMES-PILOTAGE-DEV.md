# Systèmes de Pilotage pour le Développement

**Contexte** : Ces systèmes complètent le pilotage du mode **JEU** (développement du jeu « Vestiges horizons »).
**Règle économique** : Chaque système doit avoir un **gain > coût en tokens** à chaque session. Sinon, le supprimer.

---

## 📋 Registres de Solutions et Mémoire

| Fichier | Type | Contenu | Règle d'ajout |
|---------|------|---------|---------------|
| [`solutions-dev.md`](solutions-dev.md) | Registre froid | Problèmes transverses **résolus** | 1 problème = 1 entrée, transverse à ≥2 modules |
| [`memoire-dev.md`](memoire-dev.md) | Mémoire vive | Décisions architecturales **récurrentes** | 1 décision citée ≥2 fois/session |

**À éviter** : 
- Dupliquer ce qui est déjà dans la spec ou les conventions
- Ajouter des solutions spécifiques à un seul module (le commit Git suffit)

---

## 🏗️ Architecture Decision Records (ADR)

**Dossier** : [`Docs/adr/`](adr/)

| ADR | Titre | Statut | Date |
|-----|-------|--------|------|
| [ADR-001](adr/ADR-001.md) | Gestion des débordements dans les calculs économiques | ✅ Accepté | 2024-11-14 |

**Format** :
- 1 fichier = 1 décision architecturale majeure
- Structure : Contexte, Décision, Conséquences, Alternatives rejetées

---

## ✅ Checklists et Vérifications

| Fichier | Type | Usage |
|---------|------|-------|
| [`checklist-invariants.md`](checklist-invariants.md) | Checklist | Suivi des invariants (I1-I9) par module |
| [`template-test-invariant.md`](template-test-invariant.md) | Template | Créer de nouveaux tests d'invariants |
| [`matrice-spec-tests.md`](matrice-spec-tests.md) | Matrice | Couverture des sections de spec → tests |

---

## 📊 Tableau de bord des Systèmes

| Système | Type | Gain | Coût | Ratio | Statut |
|---------|------|------|------|-------|--------|
| solutions-dev.md | Registre froid | Évite de répéter les solutions | Maintenance légère | ⬆️ | ✅ Actif |
| memoire-dev.md | Mémoire vive | Accélère les décisions | Lecture en début de session | ⬆️ | ✅ Actif |
| ADR | Registre | Document les décisions majeures | 1 fichier/decision | ⬆️ | ✅ Actif |
| checklist-invariants.md | Checklist | Suivi centralisé des invariants | Mise à jour manuelle | ⬆️ | ✅ Actif |
| template-test-invariant.md | Template | Standardisation des tests | Maintenance légère | ⬆️ | ✅ Actif |
| matrice-spec-tests.md | Matrice | Visualisation de la couverture | Mise à jour périodique | ⬆️ | ✅ Actif |

---

## 🔄 Règles de Maintenance

### 1. **Test économique** (Critère d'inclusion)
Chaque entrée doit répondre à :
```
gain_en_evitement_de_duplication > coût_en_tokens_à_chaque_session
```

### 2. **Période de probation**
- Une nouvelle entrée dans `memoire-dev.md` doit être citée **au moins 2 fois dans 3 sessions** pour être conservée
- Une entrée non lue dans **3 sessions consécutives** doit être supprimée

### 3. **Pas de duplication**
- Ne **jamais** reproduire ce qui est déjà dans :
  - `Docs/spec P1 - quartier vivant.md` (source de vérité des spécifications P1)
  - `Docs/Conventions du projet (pour agents).md` (règles d'exécution)
  - Les commentaires de code
  - Les messages de commit

### 4. **Mises à jour**
- **`solutions-dev.md`** : Mise à jour quand un problème transverse est résolu
- **`memoire-dev.md`** : Mise à jour en début de session si une décision récurrente est prise
- **`adr/`** : Nouveau fichier à chaque décision architecturale majeure
- **`checklist-invariants.md`** : Mise à jour à chaque ajout/modification d'invariant
- **`matrice-spec-tests.md`** : Mise à jour à chaque nouveau test

---

## 🚀 Utilisation Pratique

### En début de session JEU :
1. Charger `memoire-dev.md` en contexte
2. Consulter `checklist-invariants.md` pour voir les invariants à vérifier
3. Utiliser `template-test-invariant.md` pour créer de nouveaux tests

### Après une session JEU :
1. Mettre à jour `matrice-spec-tests.md` avec les nouveaux tests
2. Ajouter les problèmes transverses résolus dans `solutions-dev.md`
3. Créer un nouvel ADR si une décision architecturale majeure a été prise

### À la fin d'un sprint :
1. Vérifier que tous les éléments de la matrice ont une couverture test
2. Supprimer les entrées non utilisées dans `memoire-dev.md`
3. Archiver les ADR obsolètes

---

## 📁 Structure des Fichiers

```
Docs/
├── SYSTEMES-PILOTAGE-DEV.md          # Ce fichier (index)
├── solutions-dev.md                  # Registre froid des solutions transverses
├── memoire-dev.md                    # Mémoire vive des décisions récurrentes
├── checklist-invariants.md           # Checklist des invariants par module
├── template-test-invariant.md       # Template pour créer des tests d'invariants
├── matrice-spec-tests.md            # Matrice spec → tests
└── adr/                              # Architecture Decision Records
    └── ADR-001.md                    # Gestion des débordements
```

---

## 🔗 Liens avec les autres documents

- **Instructions agents (dev)** : `AGENTS.md`, `Agents/DEV-JEU.md`, `Agents/PROTOCOLE-ERREURS.md`, `Agents/memoire-vive.md`, `Agents/solutions.md`
- **Spécifications et conventions** : `Docs/Conventions du projet (pour agents).md`, `Docs/spec P1 - quartier vivant.md`, `Docs/spec/*`, `Docs/tickets/*`
- **Hiérarchie** : Conventions > Specs actives / Tickets > Instructions Agents/

---

## 💡 Bonnes Pratiques

1. **Avant d'ajouter une entrée** : Se demander "Est-ce que ça passera le test économique ?"
2. **Avant de modifier un système** : Vérifier l'impact sur le ratio gain/coût
3. **Régulièrement** : Faire le ménage dans `memoire-dev.md` (supprimer les entrées inutilisées)
4. **Systématiquement** : Mettre à jour la matrice après chaque nouveau test

---

## 📝 Historique

| Date | Action | Auteur |
|------|--------|--------|
| 2024-11-14 | Création des systèmes de pilotage dev | Agent IA |
| 2024-11-14 | Ajout des vérifications I9 pour tous les modules T5 | Agent IA |
| 2024-11-14 | Correction de la formule de prix de référence | Agent IA |