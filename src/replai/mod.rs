// Module de gestion des replais de reference
use crate::{EtatNoyau, Systemes, simuler_headless, generer_charge_synthetique};

pub fn generer_replai_de_reference(graine: [u8; 32], jours: u64, n: usize) -> Vec<u8> {
    let mut etat = EtatNoyau::avec_graine(graine);
    let mut systemes = Systemes::new();
    generer_charge_synthetique(&mut etat, n, graine);
    simuler_headless(&mut etat, &mut systemes, jours);
    etat.snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_replai_deterministe() {
        let g = [42u8; 32];
        let r1 = generer_replai_de_reference(g, 50, 100);
        let r2 = generer_replai_de_reference(g, 50, 100);
        assert_eq!(r1, r2);
    }
}
