// Exemple: Génère un replai de référence
use std::path::PathBuf;
use clap::Parser;
use vestiges_horizons::{EtatNoyau, Systemes, simuler_headless, generer_charge_synthetique};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 42)]
    graine: u64,
    #[arg(long, default_value_t = 100)]
    jours: u64,
    #[arg(long, default_value_t = 1000)]
    evenements: usize,
    #[arg(long, default_value = "replai_ref.bin")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut graine_bytes = [0u8; 32];
    let seed_bytes = args.graine.to_le_bytes();
    graine_bytes[..seed_bytes.len()].copy_from_slice(&seed_bytes);
    
    let mut etat = EtatNoyau::avec_graine(graine_bytes);
    generer_charge_synthetique(&mut etat, args.evenements, graine_bytes);
    let mut systemes = Systemes::new();
    simuler_headless(&mut etat, &mut systemes, args.jours);
    
    let snapshot = etat.snapshot();
    std::fs::write(args.output, &snapshot)?;
    println!("Replai sauvegarde: {}", args.output.display());
    Ok(())
}
