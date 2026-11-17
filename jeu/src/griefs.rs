use crate::entities::EntityId;
use crate::Timestamp;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

const MAX_GRIEFS_QMICRO: i64 = 400_000_000;
const MAX_INPUT: i64 = 1_000;
const SLOTS_PER_DAY: i128 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GriefEntree {
    U,
    P,
    Q,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GriefsState {
    entity_id: EntityId,
    g_qmicro: i64,
    u: u16,
    p: u16,
    q: u16,
    last_update: Timestamp,
    epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GriefsError {
    InputOutOfRange {
        input: GriefEntree,
        value: i64,
    },
    SlotOutOfRange {
        seq: u32,
    },
    TimeWentBackwards {
        current: Timestamp,
        target: Timestamp,
    },
    EpochOverflow,
    UnknownEntity(EntityId),
    UnsupportedEvent,
}

impl fmt::Display for GriefsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GriefsError::InputOutOfRange { input, value } => {
                write!(f, "grief input {:?}={} is outside [0, 1000]", input, value)
            }
            GriefsError::SlotOutOfRange { seq } => {
                write!(f, "timestamp slot {} is outside [0, 3]", seq)
            }
            GriefsError::TimeWentBackwards { current, target } => write!(
                f,
                "grief state cannot move backwards from ({}, {}) to ({}, {})",
                current.day, current.seq, target.day, target.seq
            ),
            GriefsError::EpochOverflow => write!(f, "grief state epoch overflow"),
            GriefsError::UnknownEntity(entity) => {
                write!(f, "no grief state for entity {:?}", entity)
            }
            GriefsError::UnsupportedEvent => {
                write!(f, "event is not an InputChanged event")
            }
        }
    }
}

impl Error for GriefsError {}

impl GriefsState {
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            g_qmicro: 0,
            u: 0,
            p: 0,
            q: 0,
            last_update: Timestamp::new(0, 0),
            epoch: 0,
        }
    }

    pub fn slope_micro_points_per_day(&self) -> i64 {
        180 * i64::from(self.u) + 20 * i64::from(self.p) + 60 * (500 - i64::from(self.q))
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn g_qmicro(&self) -> i64 {
        self.g_qmicro
    }

    pub fn u(&self) -> u16 {
        self.u
    }

    pub fn p(&self) -> u16 {
        self.p
    }

    pub fn q(&self) -> u16 {
        self.q
    }

    pub fn last_update(&self) -> Timestamp {
        self.last_update
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn advance_to(&mut self, target: Timestamp) -> Result<(), GriefsError> {
        validate_timestamp(target)?;
        if target < self.last_update {
            return Err(GriefsError::TimeWentBackwards {
                current: self.last_update,
                target,
            });
        }

        let delta_slots = (i128::from(target.day) - i128::from(self.last_update.day))
            * SLOTS_PER_DAY
            + i128::from(target.seq)
            - i128::from(self.last_update.seq);
        let delta = i128::from(self.slope_micro_points_per_day()) * delta_slots;
        self.g_qmicro =
            (i128::from(self.g_qmicro) + delta).clamp(0, i128::from(MAX_GRIEFS_QMICRO)) as i64;
        self.last_update = target;
        Ok(())
    }

    pub fn set_inputs_at(
        &mut self,
        u: u16,
        p: u16,
        q: u16,
        timestamp: Timestamp,
    ) -> Result<(), GriefsError> {
        validate_input(GriefEntree::U, i64::from(u))?;
        validate_input(GriefEntree::P, i64::from(p))?;
        validate_input(GriefEntree::Q, i64::from(q))?;
        validate_timestamp(timestamp)?;
        let next_epoch = self
            .epoch
            .checked_add(1)
            .ok_or(GriefsError::EpochOverflow)?;

        self.advance_to(timestamp)?;
        self.u = u;
        self.p = p;
        self.q = q;
        self.epoch = next_epoch;
        Ok(())
    }

    pub fn set_input_at(
        &mut self,
        input: GriefEntree,
        value: i64,
        timestamp: Timestamp,
    ) -> Result<(), GriefsError> {
        validate_input(input, value)?;
        let value = value as u16;
        let (u, p, q) = match input {
            GriefEntree::U => (value, self.p, self.q),
            GriefEntree::P => (self.u, value, self.q),
            GriefEntree::Q => (self.u, self.p, value),
        };
        self.set_inputs_at(u, p, q, timestamp)
    }

    pub fn griefs_micro_points(&self) -> i64 {
        self.g_qmicro / 4
    }

    pub fn griefs_points(&self) -> i64 {
        self.g_qmicro / 4_000_000
    }
}

fn validate_input(input: GriefEntree, value: i64) -> Result<(), GriefsError> {
    if !(0..=MAX_INPUT).contains(&value) {
        return Err(GriefsError::InputOutOfRange { input, value });
    }
    Ok(())
}

fn validate_timestamp(timestamp: Timestamp) -> Result<(), GriefsError> {
    if timestamp.seq > 3 {
        return Err(GriefsError::SlotOutOfRange { seq: timestamp.seq });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityType;

    fn entity() -> EntityId {
        EntityId::new(EntityType::Cohort, 1)
    }

    fn set_inputs(state: &mut GriefsState, u: u16, p: u16, q: u16) {
        state.set_inputs_at(u, p, q, Timestamp::new(0, 0)).unwrap();
    }

    #[test]
    fn revised_slope_conversion_is_exact() {
        let cases = [
            ((1_000, 0, 0), 210_000),
            ((0, 0, 500), 0),
            ((0, 0, 0), 30_000),
            ((0, 0, 1_000), -30_000),
            ((1_000, 1_000, 0), 230_000),
        ];
        for ((u, p, q), expected) in cases {
            let mut state = GriefsState::new(entity());
            set_inputs(&mut state, u, p, q);
            assert_eq!(state.slope_micro_points_per_day(), expected);
        }
    }

    #[test]
    fn total_shortage_without_services_reaches_ceiling_on_day_477() {
        let mut state = GriefsState::new(entity());
        set_inputs(&mut state, 1_000, 0, 0);
        state.advance_to(Timestamp::new(476, 0)).unwrap();
        assert_eq!(state.griefs_micro_points(), 99_960_000);
        assert_eq!(state.g_qmicro(), 399_840_000);
        state.advance_to(Timestamp::new(477, 0)).unwrap();
        assert_eq!(state.g_qmicro(), MAX_GRIEFS_QMICRO);
    }

    #[test]
    fn total_shortage_with_neutral_services_reaches_ceiling_on_day_556() {
        let mut state = GriefsState::new(entity());
        set_inputs(&mut state, 1_000, 0, 500);
        state.advance_to(Timestamp::new(555, 0)).unwrap();
        assert_eq!(state.griefs_micro_points(), 99_900_000);
        assert_eq!(state.g_qmicro(), 399_600_000);
        state.advance_to(Timestamp::new(556, 0)).unwrap();
        assert_eq!(state.g_qmicro(), MAX_GRIEFS_QMICRO);
    }

    #[test]
    fn partial_shortage_reaches_ceiling_on_day_1852() {
        let mut state = GriefsState::new(entity());
        set_inputs(&mut state, 300, 0, 500);
        state.advance_to(Timestamp::new(1_851, 0)).unwrap();
        assert_eq!(state.griefs_micro_points(), 99_954_000);
        assert_eq!(state.g_qmicro(), 399_816_000);
        state.advance_to(Timestamp::new(1_852, 0)).unwrap();
        assert_eq!(state.g_qmicro(), MAX_GRIEFS_QMICRO);
    }

    #[test]
    fn no_hunger_and_zero_services_reach_ceiling_on_day_3334() {
        let mut state = GriefsState::new(entity());
        set_inputs(&mut state, 0, 0, 0);
        state.advance_to(Timestamp::new(3_333, 0)).unwrap();
        assert_eq!(state.griefs_micro_points(), 99_990_000);
        assert_eq!(state.g_qmicro(), 399_960_000);
        state.advance_to(Timestamp::new(3_334, 0)).unwrap();
        assert_eq!(state.g_qmicro(), MAX_GRIEFS_QMICRO);
    }

    #[test]
    fn neutral_inputs_keep_griefs_constant_for_ten_years() {
        let mut state = GriefsState::new(entity());
        state.g_qmicro = 123_456_789;
        set_inputs(&mut state, 0, 0, 500);
        state.advance_to(Timestamp::new(3_650, 0)).unwrap();
        assert_eq!(state.g_qmicro(), 123_456_789);
    }

    #[test]
    fn full_services_reduce_griefs_to_zero_on_day_3334() {
        let mut state = GriefsState::new(entity());
        state.g_qmicro = MAX_GRIEFS_QMICRO;
        set_inputs(&mut state, 0, 0, 1_000);
        state.advance_to(Timestamp::new(3_333, 0)).unwrap();
        assert_eq!(state.griefs_micro_points(), 10_000);
        assert_eq!(state.g_qmicro(), 40_000);
        state.advance_to(Timestamp::new(3_334, 0)).unwrap();
        assert_eq!(state.g_qmicro(), 0);
    }

    #[test]
    fn ninety_day_shock_retreats_to_zero_after_540_recovery_days() {
        let mut state = GriefsState::new(entity());
        set_inputs(&mut state, 1_000, 0, 500);
        state.advance_to(Timestamp::new(90, 0)).unwrap();
        assert_eq!(state.griefs_micro_points(), 16_200_000);
        assert_eq!(state.g_qmicro(), 64_800_000);
        state
            .set_inputs_at(0, 0, 1_000, Timestamp::new(90, 0))
            .unwrap();
        state.advance_to(Timestamp::new(629, 0)).unwrap();
        assert_eq!(state.g_qmicro(), 120_000);
        state.advance_to(Timestamp::new(630, 0)).unwrap();
        assert_eq!(state.g_qmicro(), 0);
    }

    #[test]
    fn four_quarter_slots_match_one_day() {
        let mut split = GriefsState::new(entity());
        let mut direct = GriefsState::new(entity());
        set_inputs(&mut split, 1_000, 250, 0);
        set_inputs(&mut direct, 1_000, 250, 0);
        for seq in 1..=3 {
            split.advance_to(Timestamp::new(0, seq)).unwrap();
        }
        split.advance_to(Timestamp::new(1, 0)).unwrap();
        direct.advance_to(Timestamp::new(1, 0)).unwrap();
        assert_eq!(split.g_qmicro(), direct.g_qmicro());
        assert_eq!(split.g_qmicro(), 860_000);
    }

    #[test]
    fn event_sequence_matches_a_slot_by_slot_oracle() {
        let changes = [
            (Timestamp::new(0, 0), 300, 100, 0),
            (Timestamp::new(1, 2), 100, 300, 0),
            (Timestamp::new(2, 3), 0, 300, 500),
        ];
        let target = Timestamp::new(3, 1);
        let mut state = GriefsState::new(entity());
        let mut oracle_g = 0i128;
        let mut oracle_inputs = (0i64, 0i64, 0i64);
        let mut oracle_slot = 0i128;

        for (timestamp, u, p, q) in changes {
            state.set_inputs_at(u, p, q, timestamp).unwrap();
            let target_slot = i128::from(timestamp.day) * 4 + i128::from(timestamp.seq);
            while oracle_slot < target_slot {
                let slope =
                    180 * oracle_inputs.0 + 20 * oracle_inputs.1 + 60 * (500 - oracle_inputs.2);
                oracle_g = (oracle_g + i128::from(slope)).clamp(0, 400_000_000);
                oracle_slot += 1;
            }
            oracle_inputs = (i64::from(u), i64::from(p), i64::from(q));
        }

        state.advance_to(target).unwrap();
        let target_slot = i128::from(target.day) * 4 + i128::from(target.seq);
        while oracle_slot < target_slot {
            let slope = 180 * oracle_inputs.0 + 20 * oracle_inputs.1 + 60 * (500 - oracle_inputs.2);
            oracle_g = (oracle_g + i128::from(slope)).clamp(0, 400_000_000);
            oracle_slot += 1;
        }

        assert_eq!(i128::from(state.g_qmicro()), oracle_g);
        assert_eq!(state.epoch(), changes.len() as u64);
    }

    #[test]
    fn input_bounds_timestamp_bounds_and_order_are_enforced() {
        let mut state = GriefsState::new(entity());
        assert!(matches!(
            state.set_input_at(GriefEntree::U, 1_001, Timestamp::new(0, 0)),
            Err(GriefsError::InputOutOfRange { .. })
        ));
        assert!(matches!(
            state.advance_to(Timestamp::new(0, 4)),
            Err(GriefsError::SlotOutOfRange { .. })
        ));
        set_inputs(&mut state, 1_000, 0, 0);
        state.advance_to(Timestamp::new(10, 0)).unwrap();
        assert!(matches!(
            state.advance_to(Timestamp::new(9, 3)),
            Err(GriefsError::TimeWentBackwards { .. })
        ));
        assert!(state.g_qmicro() <= MAX_GRIEFS_QMICRO);
    }
}
