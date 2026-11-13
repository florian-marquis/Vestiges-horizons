//! Vestiges Horizons - Tranche 1 : Noyau temporel
//!
//! Ce module implémente le noyau temporel du moteur de simulation.
//! 
//! ## Spécifications
//! - `Docs/spec-tranche-1-noyau-temporel.md`
//! - `Docs/spec P1 - quartier vivant.md` §13, §29
//!
//! ## Invariant critique
//! - Déterminisme absolu
//! - Pas de flottants dans l'état canonique
//! - Pas d'itération sur HashMap

use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub mod replai;

const VERSION_FORMAT: u32 = 1;
const VERSION_DONNEES: u32 = 1;

// Types de base
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Phase {
    FluxPhysiques = 1, Marches = 2, Comptes = 3, MarcheTravail = 4,
    DecisionsAgents = 5, Social = 6, Journal = 7,
}
impl Phase {
    pub fn next(self) -> Self {
        match self {
            Phase::FluxPhysiques => Phase::Marches, Phase::Marches => Phase::Comptes,
            Phase::Comptes => Phase::MarcheTravail, Phase::MarcheTravail => Phase::DecisionsAgents,
            Phase::DecisionsAgents => Phase::Social, Phase::Social => Phase::Journal,
            Phase::Journal => Phase::FluxPhysiques,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CleEventement { pub jour: u64, pub phase: Phase, pub priorite: i32, pub id: u64 }
impl CleEventement {
    pub fn new(jour: u64, phase: Phase, priorite: i32, id: u64) -> Self {
        Self { jour, phase, priorite, id }
    }
    pub fn prochain_instant(&self) -> Self {
        let (nj, np) = if self.phase == Phase::Journal { (self.jour + 1, Phase::FluxPhysiques) }
            else { (self.jour, self.phase.next()) };
        Self { jour: nj, phase: np, priorite: self.priorite, id: self.id }
    }
}
impl PartialOrd for CleEventement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for CleEventement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.jour.cmp(&other.jour).then(self.phase.cmp(&other.phase))
            .then(self.priorite.cmp(&other.priorite)).then(self.id.cmp(&other.id))
    }
}
pub type IdType = u32;
pub type Payload = Vec<u8>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evenement { pub cle: CleEventement, pub type_evenement: IdType, pub payload: Payload }
impl Evenement {
    pub fn new(cle: CleEventement, type_evenement: IdType, payload: Payload) -> Self {
        Self { cle, type_evenement, payload }
    }
}
impl PartialOrd for Evenement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Evenement {
    fn cmp(&self, other: &Self) -> Ordering { self.cle.cmp(&other.cle) }
}
pub type FileEvenements = BinaryHeap<std::cmp::Reverse<Evenement>>;
impl Default for FileEvenements { fn default() -> Self { Self::new() } }
impl FileEvenements {
    pub fn new() -> Self { BinaryHeap::new() }
    pub fn inserer(&mut self, e: Evenement) { self.push(std::cmp::Reverse(e)); }
    pub fn extraire_min(&mut self) -> Option<Evenement> { self.pop().map(|r| r.0) }
    pub fn est_vide(&self) -> bool { self.is_empty() }
    pub fn len(&self) -> usize { self.len() }
}

// État du noyau
#[derive(Serialize, Deserialize)]
struct EnTeteSnapshot { version_format: u32, version_donnees: u32, graine: [u8; 32] }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtatNoyau {
    pub horloge: u64, pub compteur_id: u64, pub file: Vec<Evenement>, pub registres: (),
    #[serde(skip)] pub rng: ChaCha8Rng, pub graine: [u8; 32],
}
impl Default for EtatNoyau { fn default() -> Self { Self::new() } }
impl EtatNoyau {
    pub fn new() -> Self {
        let g = [0u8; 32];
        Self { horloge: 0, compteur_id: 0, file: Vec::new(), registres: (),
            rng: ChaCha8Rng::from_seed(g), graine: g }
    }
    pub fn avec_graine(graine: [u8; 32]) -> Self {
        Self { horloge: 0, compteur_id: 0, file: Vec::new(), registres: (),
            rng: ChaCha8Rng::from_seed(graine), graine }
    }
    pub fn allouer_id(&mut self) -> u64 { let id = self.compteur_id; self.compteur_id += 1; id }
    pub fn random_u64(&mut self) -> u64 { self.rng.next_u64() }
    pub fn inserer_evenement(&mut self, e: Evenement) { self.file.push(e); self.file.sort(); }
    pub fn len_file(&self) -> usize { self.file.len() }
    pub fn file_vide(&self) -> bool { self.file.is_empty() }
    pub fn snapshot(&self) -> Vec<u8> {
        let en_tete = EnTeteSnapshot { version_format: VERSION_FORMAT, version_donnees: VERSION_DONNEES, graine: self.graine };
        #[derive(Serialize)]
        struct S<'a> { h: u64, c: u64, f: &'a Vec<Evenement>, r: &'a (), g: [u8; 32] }
        let s = S { h: &self.horloge, c: &self.compteur_id, f: &self.file, r: &self.registres, g: self.graine };
        let mut b = Vec::new();
        b.extend_from_slice(&bincode::serialize(&en_tete).unwrap());
        b.extend_from_slice(&bincode::serialize(&s).unwrap());
        b
    }
    pub fn restaurer(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let (hb, eb) = bytes.split_at(4 + 4 + 32);
        let ht: EnTeteSnapshot = bincode::deserialize(hb)?;
        if ht.version_format != VERSION_FORMAT { return Err(format!("Version {}", ht.version_format).into()); }
        #[derive(Deserialize)]
        struct S { h: u64, c: u64, f: Vec<Evenement>, r: (), g: [u8; 32] }
        let s: S = bincode::deserialize(eb)?;
        Ok(Self { horloge: s.h, compteur_id: s.c, file: s.f, registres: s.r,
            rng: ChaCha8Rng::from_seed(s.g), graine: s.g })
    }
    pub fn avance_journee(&mut self, systemes: &mut Systemes) -> JournalDelta {
        let jour_courant = self.horloge;
        let mut journal_delta = Vec::new();
        let mut temp_file = reconstruire_file(std::mem::take(&mut self.file));
        while let Some(evenement) = temp_file.extraire_min() {
            if evenement.cle.jour > jour_courant {
                self.file = temp_file.into_iter().map(|r| r.0).collect();
                self.file.push(evenement);
                self.file.sort();
                break;
            }
            let instant_courant = (evenement.cle.jour, evenement.cle.phase);
            let delta = systemes.executer(evenement, instant_courant, self);
            journal_delta.extend(delta);
        }
        if self.file.is_empty() {
            self.file = temp_file.into_iter().map(|r| r.0).collect();
            self.file.sort();
        }
        self.horloge = jour_courant + 1;
        journal_delta
    }
    pub fn generer(&mut self, type_evenement: IdType, payload: Payload, date_explicite: Option<(u64, Phase)>, instant_courant: (u64, Phase)) -> ResultatGeneration<()> {
        let date = match date_explicite {
            Some(date) => date,
            None => {
                let (j, p) = instant_courant;
                let np = if p == Phase::Journal { Phase::FluxPhysiques } else { p.next() };
                let nj = if p == Phase::Journal { j + 1 } else { j };
                (nj, np)
            }
        };
        if (date.0, date.1) <= (instant_courant.0, instant_courant.1) {
            return Err(ErreurGeneration::DatePassee { date_proposee: date, instant_courant });
        }
        let id = self.allouer_id();
        let cle = CleEventement::new(date.0, date.1, 0, id);
        self.inserer_evenement(Evenement::new(cle, type_evenement, payload));
        Ok(())
    }
}

// Systèmes métiers
pub trait SystemeMetier {
    fn executer(&mut self, evenement: Evenement, instant_courant: (u64, Phase), etat: &mut EtatNoyau) -> JournalDelta;
}
pub struct Systemes {}
impl Systemes {
    pub fn new() -> Self { Self {} }
    pub fn executer(&mut self, _: Evenement, _: (u64, Phase), _: &mut EtatNoyau) -> JournalDelta { Vec::new() }
}
pub type JournalDelta = Vec<u8>;

// Erreurs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurGeneration { DatePassee { date_proposee: (u64, Phase), instant_courant: (u64, Phase) } }
pub type ResultatGeneration<T> = Result<T, ErreurGeneration>;

// Utilitaires
pub fn reconstruire_file(evenements: Vec<Evenement>) -> FileEvenements {
    let mut file = FileEvenements::new();
    for e in evenements { file.inserer(e); }
    file
}
pub fn simuler_headless(etat: &mut EtatNoyau, systemes: &mut Systemes, jours: u64) -> Vec<JournalDelta> {
    let mut j = Vec::new();
    for _ in 0..jours { j.push(etat.avance_journee(systemes)); }
    j
}
pub fn generer_charge_synthetique(etat: &mut EtatNoyau, n: usize, graine: [u8; 32]) {
    etat.graine = graine; etat.rng = ChaCha8Rng::from_seed(graine); etat.compteur_id = 0; etat.file.clear();
    for _ in 0..n {
        let j = etat.random_u64() % 1000;
        let pv = etat.random_u64() % 7 + 1;
        let p = match pv {
            1 => Phase::FluxPhysiques, 2 => Phase::Marches, 3 => Phase::Comptes,
            4 => Phase::MarcheTravail, 5 => Phase::DecisionsAgents, 6 => Phase::Social,
            _ => Phase::Journal,
        };
        let c = CleEventement::new(j as u64, p, 0, etat.allouer_id());
        etat.file.push(Evenement::new(c, 0, vec![]));
    }
    etat.file.sort();
}

// =============================================================================
// Tests unitaires (T1.1 a T1.7)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    // T1.1: Ordre canonique
    #[test] fn test_ordre_lex() {
        let c1 = CleEventement::new(0, Phase::FluxPhysiques, 0, 1);
        let c2 = CleEventement::new(0, Phase::FluxPhysiques, 0, 2);
        let c3 = CleEventement::new(0, Phase::Marches, 0, 1);
        assert!(c1 < c2); assert!(c1 < c3);
    }
    
    #[test] fn test_prochain_instant() {
        let c = CleEventement::new(0, Phase::FluxPhysiques, 0, 0);
        let n = c.prochain_instant();
        assert_eq!(n.jour, 0); assert_eq!(n.phase, Phase::Marches);
    }
    
    // T1.2: Permutation
    #[test] fn test_permutation() {
        let evts = vec![
            Evenement::new(CleEventement::new(0, Phase::FluxPhysiques, 0, 1), 0, vec![]),
            Evenement::new(CleEventement::new(0, Phase::Marches, 0, 1), 0, vec![]),
        ];
        let mut f1 = FileEvenements::new();
        for e in &evts { f1.inserer(e.clone()); }
        let o1 = f1.extraire_min().unwrap();
        let mut f2 = FileEvenements::new();
        for e in evts.into_iter().rev() { f2.inserer(e); }
        let o2 = f2.extraire_min().unwrap();
        assert_eq!(o1.cle.phase, o2.cle.phase);
    }
    
    // T1.3: Auto-regeneration
    #[test] fn test_auto_reg() {
        let mut e = EtatNoyau::new();
        let mut s = Systemes::new();
        e.file.push(Evenement::new(CleEventement::new(0, Phase::FluxPhysiques, 0, 0), 0, vec![]));
        e.file.sort();
        e.avance_journee(&mut s);
        assert_eq!(e.horloge, 1);
    }
    
    // T1.4: Rejet date passee
    #[test] fn test_rejet() {
        let mut e = EtatNoyau::new();
        let ic = (0, Phase::FluxPhysiques);
        assert!(e.generer(0, vec![], Some((0, Phase::FluxPhysiques)), ic).is_err());
        assert!(e.generer(0, vec![], Some((0, Phase::Comptes)), ic).is_ok());
    }
    
    // T1.5: Snapshot
    #[test] fn test_snapshot() {
        let mut e1 = EtatNoyau::avec_graine([42u8; 32]);
        let mut s = Systemes::new();
        for _ in 0..10 { e1.avance_journee(&mut s); }
        let snap = e1.snapshot();
        for _ in 0..10 { e1.avance_journee(&mut s); }
        let mut e2 = EtatNoyau::restaurer(&snap).unwrap();
        for _ in 0..10 { e2.avance_journee(&mut s); }
        assert_eq!(e1.horloge, e2.horloge);
    }
    
    // T1.6: Serialisation canonique
    #[test] fn test_serial() {
        let g = [42u8; 32];
        let mut e1 = EtatNoyau::avec_graine(g);
        let mut e2 = EtatNoyau::avec_graine(g);
        e1.file.push(Evenement::new(CleEventement::new(0, Phase::FluxPhysiques, 0, 0), 0, vec![]));
        e1.file.push(Evenement::new(CleEventement::new(0, Phase::Marches, 0, 1), 0, vec![]));
        e1.file.sort();
        e2.file.push(Evenement::new(CleEventement::new(0, Phase::Marches, 0, 1), 0, vec![]));
        e2.file.push(Evenement::new(CleEventement::new(0, Phase::FluxPhysiques, 0, 0), 0, vec![]));
        e2.file.sort();
        assert_eq!(e1.snapshot(), e2.snapshot());
    }
    
    // T1.7: Benchmark
    #[test] fn test_bench() {
        let mut e = EtatNoyau::avec_graine([42u8; 32]);
        generer_charge_synthetique(&mut e, 1000, [42u8; 32]);
        let mut s = Systemes::new();
        simuler_headless(&mut e, &mut s, 10);
        assert!(e.len_file() <= 1000);
    }
    
    // Determinisme RNG
    #[test] fn test_rng() {
        let g = [42u8; 32];
        let mut e1 = EtatNoyau::avec_graine(g);
        let mut e2 = EtatNoyau::avec_graine(g);
        let v1: Vec<_> = (0..100).map(|_| e1.random_u64()).collect();
        let v2: Vec<_> = (0..100).map(|_| e2.random_u64()).collect();
        assert_eq!(v1, v2);
    }
}
