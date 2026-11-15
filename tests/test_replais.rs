//! Tests de non-régression des replais T1–T5.
//!
//! Compare le SHA-256 des octets générés aux hashes figés dans
//! `tests/replais_hashes.txt`. Ne lit / ne modifie jamais les JSON
//! de `src/replai/` pour la comparaison (garde séparée avant/après).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use vestiges_horizons::replai_gen::{
    generate_and_write, generate_bytes, replai_filename, DEFAULT_SEED,
};

const TRANCHES: &[&str] = &["t1", "t2", "t3", "t4", "t5"];
const HASHES_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/replais_hashes.txt");
const REPLAI_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/replai");

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Répertoire temporaire unique hors du dépôt (nettoyé à la fin du scope).
fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "vestiges_{}_{}_{}",
        prefix,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).expect("création répertoire temporaire");
    dir
}

fn load_expected_hashes() -> BTreeMap<String, String> {
    let content = fs::read_to_string(HASHES_FILE)
        .unwrap_or_else(|e| panic!("Impossible de lire {} : {}", HASHES_FILE, e));
    let mut map = BTreeMap::new();
    for (lineno, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, hash)) = line.split_once(':') else {
            panic!(
                "Format invalide ligne {} de {} : {:?}",
                lineno + 1,
                HASHES_FILE,
                line
            );
        };
        map.insert(name.trim().to_string(), hash.trim().to_lowercase());
    }
    map
}

fn hash_reference_jsons(dir: &Path) -> BTreeMap<PathBuf, String> {
    let mut map = BTreeMap::new();
    let entries = fs::read_dir(dir).unwrap_or_else(|e| {
        panic!("Impossible de lire {} : {}", dir.display(), e);
    });
    for entry in entries {
        let entry = entry.expect("entrée src/replai");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path).expect("lecture JSON référence");
        map.insert(path, sha256_hex(&bytes));
    }
    map
}

#[test]
fn test_replais_hashes_correspondent() {
    let expected = load_expected_hashes();
    let dir = unique_temp_dir("hashes");

    for tranche in TRANCHES {
        let filename = replai_filename(DEFAULT_SEED, tranche);
        let Some(expected_hash) = expected.get(&filename) else {
            panic!("Hash manquant dans replais_hashes.txt pour {}", filename);
        };

        let path = generate_and_write(&dir, DEFAULT_SEED, tranche)
            .unwrap_or_else(|e| panic!("génération {} : {}", filename, e));
        let bytes = fs::read(&path).expect("lecture générée");
        let got = sha256_hex(&bytes);
        assert_eq!(
            &got, expected_hash,
            "Régression détectée pour {} : hash obtenu {}, attendu {}",
            filename, got, expected_hash
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_replais_determinisme_triple() {
    for tranche in TRANCHES {
        let h1 = sha256_hex(&generate_bytes(DEFAULT_SEED, tranche).unwrap());
        let h2 = sha256_hex(&generate_bytes(DEFAULT_SEED, tranche).unwrap());
        let h3 = sha256_hex(&generate_bytes(DEFAULT_SEED, tranche).unwrap());
        assert_eq!(
            h1, h2,
            "déterminisme cassé pour {} (génération 1 vs 2)",
            tranche
        );
        assert_eq!(
            h2, h3,
            "déterminisme cassé pour {} (génération 2 vs 3)",
            tranche
        );
    }
}

#[test]
fn test_garde_src_replai_inchanges() {
    let before = hash_reference_jsons(Path::new(REPLAI_DIR));
    assert!(
        !before.is_empty(),
        "aucun JSON trouvé dans {}",
        REPLAI_DIR
    );

    let dir = unique_temp_dir("garde");
    for tranche in TRANCHES {
        generate_and_write(&dir, DEFAULT_SEED, tranche).unwrap();
    }
    let _ = fs::remove_dir_all(&dir);

    let after = hash_reference_jsons(Path::new(REPLAI_DIR));
    assert_eq!(
        before, after,
        "les fichiers src/replai/*.json ont été modifiés pendant les tests"
    );
}
