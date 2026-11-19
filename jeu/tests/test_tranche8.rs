//! Tests de la Tranche 8 : Basculements (Grève)
//! 
//! Spécification : Docs/spec P1 - quartier vivant.md §20 et §35
//! Tests de fermeture : T8.1 à T8.5

use vestiges_horizons::{
    strike::*,
    entities::*,
    griefs::GriefsState,
    salary::SalaireReelEtGriefs,
    Event, EventData, EventQueue, Phase, Priority, Timestamp,
    EVENT_TYPE_INPUT_CHANGED, EVENT_TYPE_GRIEF_BOUND_REACHED,
    EVENT_TYPE_STRIKE_COOLDOWN_ENDED, EVENT_TYPE_STRIKE_TRIGGERED, EVENT_TYPE_STRIKE_ENDED,
};
use std::collections::BTreeMap;

// ============================================================================
// TRANCHE B1 — COMMIT 2 : automate événementiel (spec T7-T8 tranche B1 §4, §6.1)
// ============================================================================

/// Harnais de simulation événementielle minimal : GriefsState + StrikeManager + EventQueue.
struct Harness {
    queue: EventQueue,
    sm: StrikeManager,
    journal: CausalJournal,
    griefs: BTreeMap<EntityId, GriefsState>,
    /// o_mille par défaut par cohorte
    o: BTreeMap<EntityId, u16>,
    /// o_mille associé à chaque événement InputChanged (id événement -> o)
    o_at_event: BTreeMap<u64, u16>,
    events_processed: u64,
    events_at_first_strike: Option<u64>,
    /// (ts, cohorte, strike_index, G au déclenchement, durée tirée)
    strikes_done: Vec<(Timestamp, EntityId, u64, i64, i64)>,
    /// (ts, cohorte, strike_index, G à la fin)
    strikes_ended: Vec<(Timestamp, EntityId, u64, i64)>,
}

impl Harness {
    fn new(seed: u64) -> Self {
        Harness {
            queue: EventQueue::new(),
            sm: StrikeManager::new(seed),
            journal: CausalJournal::new(),
            griefs: BTreeMap::new(),
            o: BTreeMap::new(),
            o_at_event: BTreeMap::new(),
            events_processed: 0,
            events_at_first_strike: None,
            strikes_done: Vec::new(),
            strikes_ended: Vec::new(),
        }
    }

    fn add_cohort(&mut self, number: u64, o_mille: u16) {
        let id = EntityId::new(EntityType::Cohort, number);
        self.griefs.insert(id, GriefsState::new(id));
        self.o.insert(id, o_mille);
    }

    fn input_changed(&mut self, cohort: EntityId, u: u16, p: u16, q: u16, ts: Timestamp) {
        let o = self.o[&cohort];
        self.input_changed_o(cohort, u, p, q, ts, o);
    }

    fn input_changed_o(&mut self, cohort: EntityId, u: u16, p: u16, q: u16, ts: Timestamp, o: u16) {
        let id = self.queue.next_id();
        self.o_at_event.insert(id.as_u64(), o);
        self.queue.push(Event::new(
            id,
            ts.day,
            ts.seq,
            Phase::Comptes,
            Priority(0),
            EVENT_TYPE_INPUT_CHANGED,
            EventData::InputChanged { entity_id: cohort, u, p, q },
        ));
    }

    /// Boucle événementielle : traite les événements de jour <= horizon_day.
    fn run(&mut self, horizon_day: i64) {
        loop {
            let day = match self.queue.peek() {
                Some(e) => e.timestamp.day,
                None => break,
            };
            if day > horizon_day {
                break;
            }
            let ev = self.queue.pop().unwrap();
            self.events_processed += 1;
            match ev.event_type {
                EVENT_TYPE_INPUT_CHANGED => {
                    if let EventData::InputChanged { entity_id, u, p, q } = ev.data {
                        let (g_now, new_slope) = {
                            let gs = self.griefs.get_mut(&entity_id).unwrap();
                            gs.advance_to(ev.timestamp).unwrap();
                            let g_now = gs.g_qmicro();
                            gs.set_inputs_at(u, p, q, ev.timestamp).unwrap();
                            (g_now, gs.slope_micro_points_per_day())
                        };
                        let o = self
                            .o_at_event
                            .get(&ev.id.as_u64())
                            .copied()
                            .unwrap_or(self.o[&entity_id]);
                        // on_input_changed d'abord : le segment [last_update, ts] est crédité
                        // à l'entrée InputChanged précédente (active sur ce segment) ;
                        // puis on journalise le nouvel InputChanged (devient l'entrée active).
                        self.sm.on_input_changed(entity_id, ev.timestamp, g_now, new_slope, o, &mut self.queue);
                        self.sm.journal_input_changed(entity_id, ev.timestamp, ev.id.as_u64(), u, p, q, &mut self.journal);
                    }
                }
                EVENT_TYPE_STRIKE_TRIGGERED => {
                    if let EventData::StrikeTriggered { entity_id, draw_index, strike_index, schedule_version } = ev.data {
                        let (g, sl) = {
                            let gs = self.griefs.get_mut(&entity_id).unwrap();
                            gs.advance_to(ev.timestamp).unwrap();
                            (gs.g_qmicro(), gs.slope_micro_points_per_day())
                        };
                        if self.sm.on_strike_triggered(entity_id, ev.timestamp, g, sl, draw_index, strike_index, schedule_version, &mut self.queue) {
                            let dur = self.sm.cohort_state(entity_id).unwrap().duration_days;
                            if self.strikes_done.is_empty() {
                                self.events_at_first_strike = Some(self.events_processed);
                            }
                            self.strikes_done.push((ev.timestamp, entity_id, strike_index + 1, g, dur));
                            self.sm.journal_strike_triggered(entity_id, ev.timestamp, ev.id.as_u64(), g, &mut self.journal);
                        }
                    }
                }
                EVENT_TYPE_STRIKE_ENDED => {
                    if let EventData::StrikeEnded { entity_id, strike_index, cause, schedule_version } = ev.data {
                        let (g, sl) = {
                            let gs = self.griefs.get_mut(&entity_id).unwrap();
                            gs.advance_to(ev.timestamp).unwrap();
                            (gs.g_qmicro(), gs.slope_micro_points_per_day())
                        };
                        if self.sm.on_strike_ended(entity_id, ev.timestamp, g, sl, strike_index, cause, schedule_version, &mut self.queue) {
                            self.strikes_ended.push((ev.timestamp, entity_id, strike_index, g));
                            self.sm.journal_strike_ended(entity_id, ev.timestamp, ev.id.as_u64(), strike_index, cause, &mut self.journal);
                        }
                    }
                }
                EVENT_TYPE_STRIKE_COOLDOWN_ENDED => {
                    if let EventData::StrikeCooldownEnded { entity_id, schedule_version } = ev.data {
                        let (g, sl) = {
                            let gs = self.griefs.get_mut(&entity_id).unwrap();
                            gs.advance_to(ev.timestamp).unwrap();
                            (gs.g_qmicro(), gs.slope_micro_points_per_day())
                        };
                        self.sm.on_cooldown_ended(entity_id, ev.timestamp, g, sl, schedule_version, &mut self.queue);
                    }
                }
                EVENT_TYPE_GRIEF_BOUND_REACHED => {
                    if let EventData::GriefBoundReached { entity_id, bound_qmicro, schedule_version } = ev.data {
                        let (g, sl) = {
                            let gs = self.griefs.get_mut(&entity_id).unwrap();
                            gs.advance_to(ev.timestamp).unwrap();
                            (gs.g_qmicro(), gs.slope_micro_points_per_day())
                        };
                        self.sm.on_grief_bound_reached(entity_id, ev.timestamp, bound_qmicro, schedule_version, g, sl, &mut self.queue);
                    }
                }
                _ => {}
            }
        }
    }

    /// Premier StrikeTriggered planifié (non encore traité) pour une cohorte.
    fn first_scheduled_trigger(&self, cohort: EntityId) -> Option<Timestamp> {
        self.queue
            .sorted_events()
            .iter()
            .find(|e| {
                e.event_type == EVENT_TYPE_STRIKE_TRIGGERED
                    && matches!(&e.data, EventData::StrikeTriggered { entity_id, .. } if *entity_id == cohort)
            })
            .map(|e| e.timestamp)
    }

    /// Délai du premier déclenchement (traité ou planifié) d'une cohorte, relatif à `base_day`.
    fn first_trigger_delay(&self, cohort: EntityId, base_day: i64) -> Option<i64> {
        let scheduled = self
            .first_scheduled_trigger(cohort)
            .map(|ts| ts.day - base_day);
        let done = self
            .strikes_done
            .iter()
            .find(|&&(_, c, _, _, _)| c == cohort)
            .map(|(ts, _, _, _, _)| ts.day - base_day);
        match (scheduled, done) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
}

fn cohort(n: u64) -> EntityId {
    EntityId::new(EntityType::Cohort, n)
}

fn slot_of(ts: Timestamp) -> i64 {
    ts.day * 4 + i64::from(ts.seq)
}

/// B1 §6.1 item 2 — Déterminisme et permutations (I9, conv. 18)
#[test]
fn test_b1_determinisme_et_permutations() {
    let schedule = |h: &mut Harness| {
        for n in 1..=5u64 {
            h.input_changed(cohort(n), 1000, 0, 0, Timestamp::new(0, 0));
            h.input_changed(cohort(n), 0, 0, 500, Timestamp::new(50 + 10 * n as i64, (n % 4) as u32));
        }
    };
    let mut h1 = Harness::new(42);
    for n in 1..=5u64 {
        h1.add_cohort(n, 600);
    }
    schedule(&mut h1);
    h1.run(2_000);
    assert!(!h1.strikes_done.is_empty(), "au moins une grève attendue");

    // Même graine, ordre d'enregistrement des cohortes permuté
    let mut h2 = Harness::new(42);
    for n in (1..=5u64).rev() {
        h2.add_cohort(n, 600);
    }
    schedule(&mut h2);
    h2.run(2_000);
    assert_eq!(h1.strikes_done, h2.strikes_done, "permutation ordre d'enregistrement des cohortes");
    assert_eq!(h1.events_processed, h2.events_processed);

    // Ordre d'insertion des événements dans la file permuté
    let mut h3 = Harness::new(42);
    for n in 1..=5u64 {
        h3.add_cohort(n, 600);
    }
    for n in (1..=5u64).rev() {
        h3.input_changed(cohort(n), 0, 0, 500, Timestamp::new(50 + 10 * n as i64, (n % 4) as u32));
        h3.input_changed(cohort(n), 1000, 0, 0, Timestamp::new(0, 0));
    }
    h3.run(2_000);
    assert_eq!(h1.strikes_done, h3.strikes_done, "permutation ordre d'insertion EventQueue");

    // Rejeu bit à bit
    let mut h4 = Harness::new(42);
    for n in 1..=5u64 {
        h4.add_cohort(n, 600);
    }
    schedule(&mut h4);
    h4.run(2_000);
    assert_eq!(h1.strikes_done, h4.strikes_done, "déterminisme bit à bit");
    assert_eq!(h1.events_processed, h4.events_processed);
}

/// B1 §6.1 item 3 — G = 0 → zéro grève (100 ans) ; O = 0 → zéro grève
#[test]
fn test_b1_g0_et_o0_zero_greve() {
    for seed in 0..100 {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 1_000);
        h.input_changed(cohort(1), 0, 0, 500, Timestamp::new(0, 0));
        h.run(36_500);
        assert!(h.strikes_done.is_empty(), "G = 0 => aucune grève, seed={seed}");
        assert_eq!(h.events_processed, 1, "aucun balayage quotidien, seed={seed}");
        assert_eq!(h.sm.phase_of(cohort(1)), StrikePhaseState::Dormant);

        let mut h2 = Harness::new(seed);
        h2.add_cohort(1, 0);
        h2.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        h2.run(36_500);
        assert!(h2.strikes_done.is_empty(), "O = 0 => aucune grève, seed={seed}");
        assert_eq!(h2.sm.phase_of(cohort(1)), StrikePhaseState::Dormant);
    }
}

/// B1 §6.1 item 4 — Monotonie en G et en O (date de première grève planifiée)
#[test]
fn test_b1_monotonie_g_et_o() {
    // Montée à pente 840 000 qµpt/jour (u=1000,p=0,q=0), o=0 pendant la montée,
    // puis G constant (0,0,500) avec o fixé : la date planifiée du 1er trigger
    // est lue directement dans la file (pas de longue simulation).
    for seed in 0..50u64 {
        // délais relatifs T = jour du trigger - jour du palier (G constant)
        let mut delays_g = Vec::new();
        for &d in &[477i64, 381, 238, 96] {
            let mut h = Harness::new(seed);
            h.add_cohort(1, 0);
            h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
            h.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(d, 0), 600);
            h.run(d);
            delays_g.push(h.first_trigger_delay(cohort(1), d));
        }
        for w in delays_g.windows(2) {
            assert!(w[0].is_some() && w[1].is_some(), "trigger attendu seed={seed} {:?}", delays_g);
            assert!(w[0] <= w[1], "monotonie en G seed={seed} (delais) : {:?}", delays_g);
        }

        let mut delays_o = Vec::new();
        for &o in &[1_000u16, 800, 600, 400, 200] {
            let mut h = Harness::new(seed);
            h.add_cohort(1, 0);
            h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
            h.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(477, 0), o);
            h.run(477);
            delays_o.push(h.first_trigger_delay(cohort(1), 477));
        }
        for w in delays_o.windows(2) {
            assert!(w[0].is_some() && w[1].is_some(), "trigger attendu seed={seed} {:?}", delays_o);
            assert!(w[0] <= w[1], "monotonie en O seed={seed} (delais) : {:?}", delays_o);
        }
    }
}

/// B1 §6.1 item 5 — Loi exponentielle E[T] ≈ 1/λ à ±5 % sur N = 5 000 graines
/// (f64 autorisé ici uniquement pour la moyenne statistique, hors moteur).
#[test]
fn test_b1_loi_exponentielle() {
    // Config A : G = M (atteint au jour 477), O = 0,5 → E[T] = 364,75 j
    let mut t_a = Vec::new();
    for seed in 0..5_000u64 {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 0);
        h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        h.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(477, 0), 500);
        h.run(477 + 8_000);
        if let Some((ts, _, _, _, _)) = h.strikes_done.first() {
            t_a.push(ts.day - 477);
        }
    }
    assert!(t_a.len() >= 4_990, "strikes A: {}", t_a.len());
    let mean_a = t_a.iter().sum::<i64>() as f64 / t_a.len() as f64;
    assert!((mean_a - 364.75).abs() <= 0.05 * 364.75, "moyenne A = {mean_a} (attendu 364.75)");

    // Config B : G = 840 000 × 238 = 199 920 000 qµpt (≈ 50 pts), O = 0,6 → E[T] = 1217,39 j
    let mut t_b = Vec::new();
    for seed in 0..5_000u64 {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 0);
        h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        h.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(238, 0), 600);
        h.run(238 + 25_000);
        if let Some((ts, _, _, _, _)) = h.strikes_done.first() {
            t_b.push(ts.day - 238);
        }
    }
    assert!(t_b.len() >= 4_990, "strikes B: {}", t_b.len());
    let mean_b = t_b.iter().sum::<i64>() as f64 / t_b.len() as f64;
    assert!((mean_b - 1_217.39).abs() <= 0.05 * 1_217.39, "moyenne B = {mean_b} (attendu 1217.39)");

    // Config C : pente linéaire croissante s = 210 000/créneau depuis t=0, O = 0,5 → E[T] = 666,28 j
    let mut t_c = Vec::new();
    for seed in 0..5_000u64 {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 500);
        h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        h.run(5_000);
        if let Some((ts, _, _, _, _)) = h.strikes_done.first() {
            t_c.push(ts.day);
        }
    }
    assert!(t_c.len() >= 4_990, "strikes C: {}", t_c.len());
    let mean_c = t_c.iter().sum::<i64>() as f64 / t_c.len() as f64;
    assert!((mean_c - 666.28).abs() <= 0.05 * 666.28, "moyenne C = {mean_c} (attendu 666.28)");
}

/// B1 §6.1 item 6 — Coût O(nb_événements) : 1 seul InputChanged, horizon 10 vs 100 ans
#[test]
fn test_b1_cout_proportionnel_aux_evenements() {
    // Choisir une graine avec une grève dans les 10 ans
    let mut seed_ok = None;
    for seed in 0..200u64 {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 600);
        h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        h.run(3_650);
        if !h.strikes_done.is_empty() {
            seed_ok = Some(seed);
            break;
        }
    }
    let seed = seed_ok.expect("graine avec grève < 10 ans");

    let mut runs = Vec::new();
    for horizon in [3_650i64, 36_500] {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 600);
        h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        h.run(horizon);
        runs.push((h.events_at_first_strike, h.strikes_done.first().copied()));
    }
    assert_eq!(runs[0], runs[1], "même coût jusqu'à la 1re grève, horizon 10 vs 100 ans");
    let ev = runs[0].0.expect("grève déclenchée");
    assert!(ev <= 3, "coût O(événements) : {ev} événements jusqu'au 1er trigger (InputChanged + Trigger)");
}

/// B1 §6.1 item 7 — Durées dans [1, 1+floor(20·O)], fin par StrikeEnded, concession
#[test]
fn test_b1_durees_fin_et_concession() {
    for seed in 0..1_000u64 {
        for &o in DEFAULT_ORGANISATION_MILLE.iter() {
            let horizon = 477 + 2_000;
            let mut h = Harness::new(seed);
            h.add_cohort(1, 0);
            h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
            h.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(477, 0), o);
            h.run(horizon);
            if let Some((ts, _, si, _, dur)) = h.strikes_done.first().copied() {
                let maxd = 1 + (20 * u64::from(o)) / 1_000;
                assert!(
                    (1..=maxd as i64).contains(&dur),
                    "durée {dur} dans [1, {maxd}] pour O={o} seed={seed}"
                );
                // fin par StrikeEnded(DurationElapsed) traitée dans le run
                if ts.day + dur <= horizon {
                    assert!(
                        h.strikes_ended.iter().any(|&(_, _, si2, _)| si2 == si),
                        "StrikeEnded traité pour la grève {si}"
                    );
                }
            }
        }
    }

    // Concession : fin anticipée via l'API automate
    let mut sm = StrikeManager::new(42);
    let mut queue = EventQueue::new();
    sm.on_input_changed(cohort(1), Timestamp::new(0, 0), MAX_GRIEFS_QMICRO, 0, 600, &mut queue);
    let ev = queue.pop().unwrap();
    assert_eq!(ev.event_type, EVENT_TYPE_STRIKE_TRIGGERED);
    let (di, si, sv) = match &ev.data {
        EventData::StrikeTriggered { draw_index, strike_index, schedule_version, .. } => {
            (*draw_index, *strike_index, *schedule_version)
        }
        _ => panic!("StrikeTriggered attendu"),
    };
    assert!(sm.on_strike_triggered(cohort(1), ev.timestamp, MAX_GRIEFS_QMICRO, 0, di, si, sv, &mut queue));
    assert_eq!(sm.phase_of(cohort(1)), StrikePhaseState::OnStrike);
    assert!(sm.is_on_strike(cohort(1), ev.timestamp.day));
    assert_eq!(sm.get_strike(cohort(1)).unwrap().start_day, ev.timestamp.day);
    let (sv_end, si_now) = {
        let st = sm.cohort_state(cohort(1)).unwrap();
        (st.schedule_version, st.strike_index)
    };
    let ts_end = Timestamp::new(ev.timestamp.day + 1, ev.timestamp.seq); // 1 jour après le début
    assert!(sm.on_strike_ended(cohort(1), ts_end, MAX_GRIEFS_QMICRO, 0, si_now, StrikeEndCause::Concession, sv_end, &mut queue));
    assert_eq!(sm.phase_of(cohort(1)), StrikePhaseState::Cooldown);
    assert!(!sm.is_on_strike(cohort(1), ts_end.day));
    assert_eq!(sm.cohort_state(cohort(1)).unwrap().cooldown_end, add_slots(ts_end, 240));
    assert_eq!(sm.get_strike(cohort(1)).unwrap().end_cause, Some(StrikeEndCause::Concession));
    let ev2_ts = queue
        .sorted_events()
        .into_iter()
        .find(|e| e.event_type == EVENT_TYPE_STRIKE_COOLDOWN_ENDED)
        .expect("événement cooldown planifié")
        .timestamp;
    assert_eq!(ev2_ts, add_slots(ts_end, 240));
}

/// B1 §6.1 item 8 — Salaire réel absent du déclenchement
#[test]
fn test_b1_salaire_reel_absent_du_declenchement() {
    let mut results = Vec::new();
    let mut salaires_reels = Vec::new();
    let mut p_values = Vec::new();
    let mut grief_values = Vec::new();
    for salaire in [10u64, 1_000_000] {
        let mut sr = SalaireReelEtGriefs::new(cohort(1));
        sr.salaire_horaire = salaire;
        sr.heures_travaillees = 8;
        sr.cout_panier = 2_000;
        sr.calculer_salaire_reel();
        let p = sr.calculate_p(Some(2_500));
        sr.mettre_a_jour_entrees(1_000, p, 0, Timestamp::new(0, 0)).unwrap();
        sr.mettre_a_jour_griefs(Timestamp::new(1, 0)).unwrap();
        salaires_reels.push(sr.salaire_reel);
        p_values.push(p);
        grief_values.push(sr.griefs.g_qmicro());

        let mut h = Harness::new(42);
        h.add_cohort(1, 600);
        h.input_changed(cohort(1), 1_000, p, 0, Timestamp::new(0, 0));
        h.run(3_000);
        results.push(h.strikes_done.clone());
    }
    assert_ne!(salaires_reels[0], salaires_reels[1]);
    assert_eq!(p_values[0], p_values[1]);
    assert_eq!(grief_values[0], grief_values[1]);
    assert_eq!(results[0], results[1], "salaire réel sans effet sur les dates de grève");
    assert!(!results[0].is_empty());
}

/// B1 §6.1 item 9 — Remise à zéro à G = 0 (zéro mémoire de H_cum)
#[test]
fn test_b1_remise_a_zero_g0() {
    for seed in 0..200u64 {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 200);
        h.input_changed(cohort(1), 1_000, 0, 500, Timestamp::new(0, 0)); // montée, pente 180 000
        h.input_changed(cohort(1), 0, 0, 1_000, Timestamp::new(100, 0)); // descente, pente -30 000 -> G=0 vers jour 700
        h.run(701);
        if !h.strikes_done.is_empty() {
            continue; // grève avant la remise à zéro : on cherche une graine sans grève
        }
        let st = h.sm.cohort_state(cohort(1)).unwrap();
        assert_eq!(st.phase, StrikePhaseState::Dormant, "remise à zéro à G=0 seed={seed}");
        assert_eq!(st.h_cum_scaled, 0, "H_cum remis à zéro seed={seed}");
        assert_eq!(st.active_threshold, 0, "seuil abandonné seed={seed}");
        let di_before = st.draw_index;
        // Seconde montée : nouveau seuil = tirage pur pour le draw_index suivant
        h.input_changed_o(cohort(1), 1_000, 0, 500, Timestamp::new(700, 0), 600);
        h.run(700);
        let st = h.sm.cohort_state(cohort(1)).unwrap();
        assert_eq!(st.active_draw_index, di_before + 1);
        assert_eq!(
            st.active_threshold,
            draw_exponential_threshold_scaled(seed, cohort(1).to_u64(), st.active_draw_index),
            "zéro mémoire de H_cum passé seed={seed}"
        );
        return;
    }
    panic!("aucune graine sans grève avant le jour 701");
}

/// B1 §6.1 item 10 — Répit de 60 jours (240 créneaux) et G inchangé par la grève
#[test]
fn test_b1_repit_60_jours_et_g_inchange() {
    for seed in 0..50u64 {
        let mut h = Harness::new(seed);
        h.add_cohort(1, 0);
        h.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0)); // montée vers M
        h.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(477, 0), 600); // G = M constant, O = 0,6
        h.run(477 + 2_000);
        assert!(!h.strikes_done.is_empty(), "grève attendue seed={seed}");
        for &(te, c, si, g_end) in &h.strikes_ended {
            // G inchangé par la grève (pente nulle pendant la grève ici)
            let trig = h.strikes_done.iter().find(|&&(_, c2, si2, _, _)| c2 == c && si2 == si);
            if let Some(&(_, _, _, g_trig, _)) = trig {
                assert_eq!(g_trig, g_end, "G inchangé par la grève seed={seed} si={si}");
            }
            // aucune nouvelle grève dans les 240 créneaux suivant la fin
            for &(tt, c2, si2, _, _) in &h.strikes_done {
                if c2 == c && si2 > si {
                    assert!(
                        slot_of(tt) >= slot_of(te) + 240,
                        "répit 60 j respecté seed={seed} : fin {:?} -> grève {:?}",
                        te, tt
                    );
                }
            }
        }
    }
}

/// B1 §6.1 item 11 — Règle 18 : aucun déclenchement dans le même instant logique
#[test]
fn test_b1_regle_18_pas_execution_meme_instant() {
    let mut sm = StrikeManager::new(42);
    let mut queue = EventQueue::new();
    let ts = Timestamp::new(10, 2);
    {
        let st = sm.cohort_state_mut(cohort(1));
        st.phase = StrikePhaseState::Accumulating;
        st.active_draw_index = 1;
        st.active_threshold = 1_000; // sera dépassé par l'intégration du segment
        st.h_cum_scaled = 0;
        st.active_o_mille = 600;
        st.schedule_version = 5;
        st.last_update = Timestamp::new(10, 0); // 2 créneaux avant ts
        st.g_last_qmicro = 200_000_000;
        st.slope_last = 0;
    }
    // delta_h(2e8, 0, 600, 2) >> 1000 : franchissement constaté à ts
    sm.on_input_changed(cohort(1), ts, 200_000_000, 0, 600, &mut queue);
    let st = sm.cohort_state(cohort(1)).unwrap();
    assert_eq!(st.phase, StrikePhaseState::PendingNextSlot, "règle 18 : PendingNextSlot");
    let ev = queue.peek().unwrap();
    assert_eq!(ev.event_type, EVENT_TYPE_STRIKE_TRIGGERED);
    assert_eq!(ev.timestamp, next_slot(ts), "jamais dans le même créneau");
    assert_eq!(ev.timestamp, Timestamp::new(10, 3));
    assert_eq!(ev.phase, Phase::Social);
    // seq = 3 -> next_slot = (day+1, 0)
    assert_eq!(next_slot(Timestamp::new(10, 3)), Timestamp::new(11, 0));
}

/// B1 §4.2 — Encodage déterministe des Id (bit 63 = 1, pas de compteur partagé)
#[test]
fn test_b1_encodage_id_deterministe() {
    let c1 = cohort(1);
    let c2 = cohort(2);
    let id1 = encode_strike_event_id(KIND_STRIKE_TRIGGER, c1, 7);
    let id2 = encode_strike_event_id(KIND_STRIKE_TRIGGER, c2, 7);
    let id3 = encode_strike_event_id(KIND_STRIKE_END, c1, 7);
    let id4 = encode_strike_event_id(KIND_STRIKE_TRIGGER, c1, 8);
    assert_ne!(id1, id2, "cohortes distinctes");
    assert_ne!(id1, id3, "kinds distincts");
    assert_ne!(id1, id4, "versions distinctes");
    assert_eq!(id1, encode_strike_event_id(KIND_STRIKE_TRIGGER, c1, 7), "déterministe");
    assert!(id1.as_u64() >> 63 == 1, "bit 63 = 1, disjoint des ids 0..2^63");
    assert_eq!((id1.as_u64() >> 60) & 0x7, KIND_STRIKE_TRIGGER);
    assert_eq!((id1.as_u64() >> 28) & 0xFFFF_FFFF, 1);
    assert_eq!(id1.as_u64() & 0x0FFF_FFFF, 7);
}

// ============================================================================
// TRANCHE B1 — COMMIT 3 : journal causal et I7 (spec §5, §6.1 item 12)
// ============================================================================

/// B1 §6.1 item 12 — Journal causal : champs règle 14, top-5, verify_i7 vert
#[test]
fn test_b1_journal_causal_et_i7() {
    // Chercher une graine dont la 1re grève tombe après le jour 490
    // (pour avoir > 5 InputChanged contributeurs avant le déclenchement).
    let mut h = None;
    for seed in 0..300u64 {
        let mut cand = Harness::new(seed);
        cand.add_cohort(1, 0);
        cand.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        cand.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(477, 0), 600);
        for d in 478..=484i64 {
            cand.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(d, 0), 600);
        }
        cand.run(477 + 3_000);
        if cand.strikes_done.first().map(|(ts, _, _, _, _)| ts.day) > Some(490) {
            h = Some(cand);
            break;
        }
    }
    let h = h.expect("graine avec 1re grève après le jour 490");
    assert!(!h.strikes_done.is_empty());
    assert!(h.journal.verify_i7(), "I7 vert sur tout le run");

    // Entrées InputChanged journalisées en Phase::Comptes (phase 3)
    let input_entries: Vec<_> = h
        .journal
        .all_entries()
        .filter(|e| e.decision_type == DECISION_INPUT_CHANGED)
        .collect();
    assert!(input_entries.len() >= 7, "au moins 7 InputChanged journalisés");
    assert!(input_entries.iter().all(|e| e.phase == Phase::Comptes as u8));
    assert!(input_entries.iter().all(|e| e.actor == Some(cohort(1))));

    // Entrée StrikeTriggered : tous les champs règle 14 présents
    let trig = h
        .journal
        .all_entries()
        .find(|e| e.decision_type == DECISION_STRIKE_TRIGGERED)
        .expect("entrée StrikeTriggered journalisée");
    assert_eq!(trig.phase, Phase::Social as u8);
    assert_eq!(trig.actor, Some(cohort(1)));
    let fields: Vec<&str> = trig.modified_values.iter().map(|m| m.field.as_str()).collect();
    for f in [
        "strike_index",
        "draw_index",
        "g_qmicro",
        "lambda_mille_per_year",
        "threshold_scaled",
        "planned_duration_days",
    ] {
        assert!(fields.contains(&f), "champ règle 14 '{f}' présent");
    }
    assert_eq!(trig.learned_by, vec![(cohort(1), trig.timestamp.day as u64)]);
    // Top-5 strict : reconstruire les deltas crédités à chaque entrée InputChanged
    // (segment [ts_i, ts_{i+1}] avec o_i actif, G = M constant) et comparer.
    assert_eq!(trig.causes.len(), 5, "tampon top-5 strict, obtenu {}", trig.causes.len());
    let mut ic: Vec<&JournalEntry> = h
        .journal
        .all_entries()
        .filter(|e| e.decision_type == DECISION_INPUT_CHANGED)
        .collect();
    ic.sort_by_key(|e| e.id);
    let mut deltas: Vec<(u64, u128)> = Vec::new();
    for (i, e) in ic.iter().enumerate() {
        let next_ts = if i + 1 < ic.len() { ic[i + 1].timestamp } else { trig.timestamp };
        let k = slots_between(e.timestamp, next_ts);
        let o_i = h.o_at_event.get(&e.event_id).copied().unwrap_or(0);
        let d = if k > 0 {
            delta_h_scaled(MAX_GRIEFS_QMICRO, 0, u64::from(o_i), k)
        } else {
            0
        };
        deltas.push((e.id, d));
    }
    deltas.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    deltas.truncate(5);
    let expected: Vec<u64> = deltas.iter().map(|(id, _)| *id).collect();
    assert_eq!(trig.causes, expected, "causes = top-5 (delta DESC, entry_id ASC)");
    for &c in &trig.causes {
        let ce = h.journal.all_entries().find(|e| e.id == c).expect("cause existe");
        assert_eq!(ce.decision_type, DECISION_INPUT_CHANGED, "cause = entrée InputChanged");
    }
    // lambda_mille_per_year cohérent : LAMBDA_MAX_MILLE · o · g² / (1000 · M²), troncature
    let g = trig.modified_values.iter().find(|m| m.field == "g_qmicro").unwrap().new_value as i128;
    let lam = trig.modified_values.iter().find(|m| m.field == "lambda_mille_per_year").unwrap().new_value as u128;
    let attendu = (LAMBDA_MAX_MILLE as u128 * 600u128 * (g as u128).pow(2)) / (1000u128 * (MAX_GRIEFS_QMICRO as u128).pow(2));
    assert_eq!(lam as u128, attendu, "lambda_mille_per_year = troncature entière");

    // Entrée StrikeEnded journalisée (DECISION_STRIKE_ENDED = 81)
    let ended = h.journal.all_entries().find(|e| e.decision_type == DECISION_STRIKE_ENDED);
    assert!(ended.is_some(), "StrikeEnded journalisé");
    let ended = ended.unwrap();
    assert_eq!(ended.phase, Phase::Social as u8);
    assert_eq!(ended.learned_by, vec![(cohort(1), ended.timestamp.day as u64)]);

    // Tampon vidé au déclenchement : plus de causes sur l'état courant après la grève
    // (re-rempli seulement par de nouveaux segments post-grève)
    let st = h.sm.cohort_state(cohort(1)).unwrap();
    assert!(st.causes.len() <= 5);
}

/// B1 §5.3 — I7 sur (timestamp, phase) : même ts, phase antérieure acceptée ;
/// même (timestamp, phase) ou postérieure rejetée.
#[test]
fn test_b1_i7_tuple_timestamp_phase() {
    let mut j = CausalJournal::new();
    let mk = |day: i64, seq: u32, phase: u8, causes: Vec<u64>| JournalEntry {
        id: 0,
        timestamp: Timestamp::new(day, seq),
        event_id: (day * 100 + i64::from(seq) * 10 + i64::from(phase)) as u64,
        phase,
        actor: None,
        decision_type: 0,
        modified_values: vec![],
        causes,
        learned_by: vec![],
    };
    // A en (10, 0) phase Comptes (3)
    let a = j.add_entry(mk(10, 0, 3, vec![])).expect("A acceptée");
    // B en (10, 0) phase Social (6), cause A : même timestamp, phase antérieure -> ACCEPTÉ (arbitrage 6)
    let b = j.add_entry(mk(10, 0, 6, vec![a])).expect("même ts, phase antérieure acceptée");
    assert!(j.verify_i7());
    // même (timestamp, phase) que B comme cause de B -> rejeté
    assert!(j.add_entry(mk(10, 0, 6, vec![b])).is_none(), "même (ts, phase) rejeté");
    // cause postérieure en phase (B=(10,0,6)) pour entrée (10,0,2) -> rejeté
    assert!(j.add_entry(mk(10, 0, 2, vec![b])).is_none(), "cause postérieure rejetée");
    // cause strictement antérieure en timestamp -> acceptée
    let _c = j.add_entry(mk(10, 1, 1, vec![b])).expect("cause antérieure en ts acceptée");
    let _d = j.add_entry(mk(11, 3, 7, vec![a])).expect("cause antérieure (jour précédent) acceptée");
    assert!(j.verify_i7());
    // cause inexistante : add_entry tolère (journalisation dans l'ordre), verify_i7 non
    assert!(j.add_entry(mk(11, 0, 1, vec![999])).is_some(), "cause future tolérée à l'ajout");
    assert!(!j.verify_i7(), "verify_i7 exige que la cause existe");
}

/// B1 §5.1 — Tampon top-5 via l'automate : plusieurs InputChanged contributeurs
/// à G constant (deltas égaux) -> tronqué à 5, tri delta DESC puis id ASC.
/// (Le test unitaire fin de add_cause est dans strike.rs::tests::test_b1_tampon_causes_top5.)
#[test]
fn test_b1_tampon_causes_top5() {
    // Chercher une graine dont le trigger tombe après le jour 486 (tampon non vidé)
    let mut h = None;
    for seed in 0..300u64 {
        let mut cand = Harness::new(seed);
        cand.add_cohort(1, 0);
        cand.input_changed(cohort(1), 1_000, 0, 0, Timestamp::new(0, 0));
        cand.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(477, 0), 600);
        for d in 478..=486i64 {
            cand.input_changed_o(cohort(1), 0, 0, 500, Timestamp::new(d, 0), 600);
        }
        cand.run(486);
        let st = cand.sm.cohort_state(cohort(1)).unwrap();
        let trigger_apres = cand
            .strikes_done
            .first()
            .map(|(ts, _, _, _, _)| ts.day)
            .unwrap_or(i64::MAX)
            > 486
            && cand.first_scheduled_trigger(cohort(1)).map(|ts| ts.day).unwrap_or(i64::MAX) > 486;
        if st.causes.len() == 5 && trigger_apres {
            h = Some(cand);
            break;
        }
    }

    let h = h.expect("graine avec tampon plein et trigger après 486");
    let st = h.sm.cohort_state(cohort(1)).unwrap();
    assert_eq!(st.causes.len(), 5, "9 segments contributeurs -> 5 retenus, obtenu {}", st.causes.len());
    // tri : delta DESC puis id ASC (deltas égaux ici -> ids croissants)
    for w in st.causes.windows(2) {
        assert!(
            w[0].delta_h_scaled > w[1].delta_h_scaled
                || (w[0].delta_h_scaled == w[1].delta_h_scaled && w[0].journal_entry_id < w[1].journal_entry_id),
            "tri delta DESC, id ASC : {:?}",
            st.causes
        );
    }
}

const DISPLAY_MODULE_ALLOWLIST: &[&str] = &[];

#[test]
fn test_b1_ancienne_regle_absente_du_moteur() {
    fn collect_rust_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("lecture du répertoire source") {
            let path = entry.expect("entrée du répertoire source").path();
            if path.is_dir() {
                collect_rust_files(&path, files);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    fn strip_non_code(source: &str) -> String {
        let chars: Vec<char> = source.chars().collect();
        let mut code = String::with_capacity(source.len());
        let mut i = 0;
        let mut block_depth = 0usize;
        while i < chars.len() {
            if block_depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    block_depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    block_depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            } else if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            } else if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                block_depth = 1;
                i += 2;
            } else if chars[i] == 'r' {
                let mut quote = i + 1;
                while chars.get(quote) == Some(&'#') {
                    quote += 1;
                }
                if chars.get(quote) == Some(&'"') {
                    let hashes = quote - i - 1;
                    i = quote + 1;
                    while i < chars.len() {
                        if chars[i] == '"'
                            && (0..hashes).all(|offset| chars.get(i + 1 + offset) == Some(&'#'))
                        {
                            i += hashes + 1;
                            break;
                        }
                        i += 1;
                    }
                } else {
                    code.push(chars[i]);
                    i += 1;
                }
            } else if chars[i] == '"' {
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 2;
                    } else if chars[i] == '"' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
            } else if chars[i] == '\''
                && (chars.get(i + 2) == Some(&'\'')
                    || (chars.get(i + 1) == Some(&'\\')
                        && chars.get(i + 3) == Some(&'\'')))
            {
                i += if chars.get(i + 1) == Some(&'\\') { 4 } else { 3 };
            } else {
                code.push(chars[i]);
                i += 1;
            }
        }
        code
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let guard_test = root.join(file!()).canonicalize().expect("chemin du garde-fou");
    let forbidden = [
        "taux_risque",
        "seuil_risque",
        "calculer_taux_risque",
        "seuil_franchi",
    ];

    for (tree, dir) in [("src", root.join("src")), ("tests", root.join("tests"))] {
        let mut files = Vec::new();
        collect_rust_files(&dir, &mut files);
        for path in files {
            if path.canonicalize().expect("chemin du fichier Rust") == guard_test {
                continue;
            }
            let relative = path.strip_prefix(&dir).expect("chemin sous le répertoire");
            let relative = relative.to_string_lossy().replace('\\', "/");
            if tree == "src" && DISPLAY_MODULE_ALLOWLIST.contains(&relative.as_str()) {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("lecture du fichier source");
            let code = strip_non_code(&source);
            for identifier in forbidden {
                let found = code
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .any(|token| token == identifier);
                assert!(!found, "{identifier} subsiste dans {tree}/{relative}");
            }
        }
    }
}
