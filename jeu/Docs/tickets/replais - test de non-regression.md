# Ticket : Tests de non-régression des replais

## Contexte

Le cadrage (chapitre 9) et les conventions du projet (règles 6 et 8) reposent sur une suite de replais déterministes servant de tests de non-régression. Cependant, l'analyse révèle que :

1. **Les replais existants ne sont pas branchés** : Aucun test automatisé ne charge ni ne compare les fichiers `src/replai/reference_seed42_t*.json`.
2. **Les replais ne couvrent pas tout le code** : Par exemple, T3-T5 ne couvrent pas la production avec recettes (`recipe.rs`), et labor.rs n'est utilisé par aucun replai.
3. **`generer_replai.rs` écrit en dur dans `src/replai/`** : Impossible de générer vers un répertoire temporaire pour comparaison sans modifier le code.
4. **`main.rs` écrit directement dans `src/replai/`** : Les fonctions `generate_reference_replay_t1` et `generate_reference_replay_t2` (lignes 121, 125) écrasent les fichiers de référence lors de l'exécution de `cargo run`.

## Problème

Sans tests de comparaison automatisés, corriger les défauts de type (f32/f64 dans l'état canonique) est **invérifiable**. On ne peut pas garantir que les corrections ne cassent pas la déterminisme ou ne modifient pas le comportement attendu.

## Exigences supplémentaires

### Sérialisation déterministe

- **Interdiction des HashMap** : Toutes les structures sérialisées doivent utiliser `BTreeMap` ou `Vec` pour garantir un ordre déterministe. La sérialisation JSON dépend de l'ordre des champs.
- **Ordre des champs struct** : Les structs doivent être sérialisées avec `#[derive(Serialize)]` qui garantit l'ordre alphabétique des champs.
- **Validation** : Ajouter un test qui vérifie que la sérialisation est déterministe (deux sérialisations du même état → même JSON).

### Protection de src/replai/

- **`main.rs`** : Supprimer ou commenter les lignes 121-126 qui générent T1 et T2 dans `src/replai/`. La génération de replais doit être explicite et contrôlée.
- **`generer_replai.rs`** : Ne **jamais** écrire dans `src/replai/` par défaut. L'option `--output-dir` doit être obligatoire pour écrire dans ce répertoire.

### Replais de couverture

Créer **avant les corrections** des replais qui couvrent explicitement :

1. **recipe.rs** :
   - Production avec `RecipeProductionManager::produce()`
   - Calcul de `final_quantity` avec base_yield f32
   - Appel à `add_output_stock()`

2. **labor.rs** :
   - Appel à `Engagement::mise_a_jour()` avec décroissance exponentielle
   - Simulation sur 100+ jours pour observer la décroissance

Ces replais doivent être générés **avec le code actuel (f32)** pour servir de référence avant correction.

## Spécification des tests de non-régression

### Entités

- **Replai de référence** : Fichier JSON dans `src/replai/reference_seed<seed>_t<X>.json` contenant la séquence d'événements attendue.
- **Replai généré** : Fichier JSON généré en mémoire ou dans un répertoire temporaire par `generer_replai.rs`.
- **Hash d'état** : SHA-256 du contenu JSON (pas textuel, mais binaire pour éviter les problèmes d'encodage).

### Comparaison

1. Pour chaque tranche T1-T5 (et futur T6+) :
   - Exécuter `generer_replai --output-dir /tmp/replai_test -- <seed> <tranche>`
   - Calculer le hash SHA-256 du fichier généré
   - Comparer avec le hash du fichier de référence `src/replai/reference_seed<seed>_t<X>.json`
   - **Si différent** : signaler un changement de comportement (à valider manuellement)

2. Pour les tranches futures (T7-T8, etc.) :
   - Créer un replai de référence **avant** la première implémentation
   - Ajouter à la suite de tests

3. **Test de déterminisme** : Exécuter la génération 3 fois avec la même seed, vérifier que les 3 hash sont identiques.

### Option de répertoire de sortie

Modifier `generer_replai.rs` pour accepter un argument optionnel `--output-dir` ou `-o` :

```rust
// Dans main()
let args: Vec<String> = env::args().collect();
let mut output_dir = None;
let mut arg_index = 1;

while arg_index < args.len() {
    match args[arg_index].as_str() {
        "--output-dir" | "-o" => {
            arg_index += 1;
            if arg_index < args.len() {
                output_dir = Some(args[arg_index].clone());
            }
        }
        _ => break,
    }
    arg_index += 1;
}

let output_dir = output_dir.unwrap_or_else(|| "src/replai/".to_string());

// Puis dans chaque fonction de génération :
let filename = format!("{}/reference_seed{}_t{}.json", output_dir, seed, tranche);
```

**Règle** : Par défaut, **ne jamais écrire dans `src/replai/`** sans drapeau explicite `--output-dir src/replai/`.

### Critères de réussite

1. Tous les replais de référence (T1-T5) passent le test de comparaison avec leur hash attendu.
2. Un nouveau replai peut être généré et comparé sans modifier `src/replai/`.
3. Le test est déterministe : exécution multiple avec même seed → même hash.
4. Le test est intégré à la CI (à terme).
5. Les replais de couverture pour recipe.rs et labor.rs existent et sont validés.

### Protection en lecture seule

- Les fichiers `src/replai/*.json` sont **en lecture seule** pour les agents.
- Toute correction qui modifie un replai doit :
  1. Être validée par un test de comparaison qui **échoue avant** et **passe après**
  2. Avoir une justification écrite dans le commit
  3. Être revu par un humain

## Travail à faire

### Priorité maximale (blocage pour T7-T8)

1. **Créer le test de comparaison** : Script ou test Rust qui implémente la logique ci-dessus.
2. **Ajouter l'option `--output-dir`** à `generer_replai.rs`.
3. **Corriger main.rs** : Supprimer ou désactiver les appels à `generate_reference_replay_t1` et `generate_reference_replay_t2` qui écrivent dans `src/replai/`.

### Avant toute correction f32/f64

4. **Créer replais de couverture** :
   - Un replai T6 (ou un nouveau fichier) qui appelle `RecipeProductionManager::produce()` avec des paramètres réalistes
   - Un replai T7 qui appelle `Engagement::mise_a_jour()` sur 100+ jours
5. **Valider la sérialisation** : Vérifier que tous les replais utilisent BTreeMap et pas HashMap.

### Après validation des replais

6. **Appliquer les corrections f32/f64** une par une, en vérifiant que les replais de couverture changent comme attendu.
7. **Mettre à jour les replais de référence** si nécessaire, avec validation humaine.

## Preuves d'exécution (à intégrer dans le ticket une fois les tests implantés)

### Déterminisme actuel
```
T1: 20bf028e7412347c9fc8f26cca7554c5d11c4dde2cb2482fca6b9d23c658cb1a
T2: c1526d99cb74e742f92a0439a9e292e38667c44f0658cbbcb220aff65c6c3cd9
T3: d8482acdbf47cad02c4fd9f09cbad4d5cb1cfa5865f69c4866626b674d709b8c
T4: 643aa447ae9536f78853701eb66fa79fddc0f3b2bdb2bf34f4214d965de7e3de
T5: 9b39ceb4d843a77ba6f81f5228278f46a19af5b06d3e97be786c2ab8aaec4bfe
```

### Couverture actuelle
- **recipe.rs** : T5 ne couvre PAS la production (pas d'appel à `produce` ou `produce_all`)
- **labor.rs** : Aucun replai T1-T5 ne couvre la décroissance exponentielle (pas d'appel à `Engagement::mise_a_jour`)

## Notes

- Les replais actuels (T1-T5) sont déterministes (hash identiques entre générations).
- Le problème principal n'est pas la déterminisme, mais **l'absence de tests de comparaison** et la **couverture incomplète**.
- La correction des f32/f64 doit attendre que les replais de couverture soient créés et validés.
