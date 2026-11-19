//! Générateur de replai de référence pour Vestiges Horizons
//!
//! Usage :
//!   cargo run --bin generer_replai -- --output-dir <chemin> <seed> <tranche>
//!
//! `--output-dir` est obligatoire. Aucune écriture par défaut dans src/replai/.

use std::env;
use std::path::Path;
use std::process;

use vestiges_horizons::replai_gen;

fn print_usage() {
    eprintln!(
        "Usage : cargo run --bin generer_replai -- --output-dir <chemin> <seed> <tranche>"
    );
    eprintln!("Exemple : cargo run --bin generer_replai -- --output-dir ./out 42 t3");
    eprintln!("Tranches : t1, t2, t3, t4, t5");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut output_dir: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output-dir" | "-o" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Erreur : --output-dir exige un chemin.");
                    print_usage();
                    process::exit(1);
                }
                output_dir = Some(args[i].clone());
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            other if other.starts_with('-') => {
                eprintln!("Option inconnue : {}", other);
                print_usage();
                process::exit(1);
            }
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }

    let Some(output_dir) = output_dir else {
        eprintln!("Erreur : --output-dir <chemin> est obligatoire.");
        print_usage();
        process::exit(1);
    };

    if positional.len() != 2 {
        eprintln!("Erreur : seed et tranche sont obligatoires.");
        print_usage();
        process::exit(1);
    }

    let seed: u64 = match positional[0].parse() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Seed doit être un entier : {}", positional[0]);
            process::exit(1);
        }
    };
    let tranche = positional[1].as_str();

    match replai_gen::generate_and_write(Path::new(&output_dir), seed, tranche) {
        Ok(path) => {
            println!("Replai généré : {}", path.display());
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
