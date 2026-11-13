// Benchmarks pour la Tranche 1
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vestiges_horizons::{CleEventement, EtatNoyau, Evenement, Phase};

fn bench_insertion(c: &mut Criterion) {
    c.bench_function("insertion_10000", |b| {
        b.iter(|| {
            let mut etat = EtatNoyau::new();
            for i in 0..10_000 {
                let cle = CleEventement::new(i % 365, Phase::FluxPhysiques, 0, i as u64);
                etat.inserer_evenement(Evenement::new(cle, 0, vec![]));
            }
            black_box(etat);
        })
    });
}

fn bench_extraction(c: &mut Criterion) {
    let mut etat = EtatNoyau::new();
    for i in 0..10_000 {
        let cle = CleEventement::new(0, Phase::FluxPhysiques, 0, i as u64);
        etat.inserer_evenement(Evenement::new(cle, 0, vec![]));
    }
    c.bench_function("extraction_10000", |b| {
        b.iter(|| {
            let mut etat = etat.clone();
            for _ in 0..10_000 { let _ = etat.extraire_min(); }
            black_box(etat);
        })
    });
}

fn bench_serialisation(c: &mut Criterion) {
    let mut etat = EtatNoyau::new();
    for i in 0..10_000 {
        let cle = CleEventement::new(i % 365, Phase::FluxPhysiques, 0, i as u64);
        etat.file.push(Evenement::new(cle, 0, vec![]));
    }
    c.bench_function("serialisation", |b| {
        b.iter(|| {
            let s = etat.snapshot();
            black_box(s);
        })
    });
}

criterion_group!(benches, bench_insertion, bench_extraction, bench_serialisation);
criterion_main!(benches);
