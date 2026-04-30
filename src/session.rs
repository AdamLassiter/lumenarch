use serde::Serialize;

use crate::protocol::{EncounterRegisterState, SessionAppState, SessionSnapshot, ShipSnapshot};

#[derive(Serialize)]
struct CanonicalSessionHash<'a> {
    app_state: SessionAppState,
    ship: &'a ShipSnapshot,
    progression: &'a crate::state::DemoProgression,
    sector: &'a crate::state::SectorState,
    last_mission_report: &'a crate::state::LastMissionReport,
    encounter_registers: &'a EncounterRegisterState,
}

pub(crate) fn stable_hash_json<T: Serialize>(value: &T) -> u64 {
    let encoded = serde_json::to_vec(value).unwrap_or_default();
    stable_hash_bytes(&encoded)
}

pub(crate) fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn snapshot_with_hash(mut snapshot: SessionSnapshot) -> SessionSnapshot {
    snapshot.state_hash = 0;
    let hash = stable_hash_json(&CanonicalSessionHash {
        app_state: snapshot.app_state,
        ship: &snapshot.ship,
        progression: &snapshot.progression,
        sector: &snapshot.sector,
        last_mission_report: &snapshot.last_mission_report,
        encounter_registers: &snapshot.encounter_registers,
    });
    snapshot.state_hash = hash;
    snapshot
}

pub(crate) fn canonical_session_hash(
    app_state: SessionAppState,
    ship: &ShipSnapshot,
    progression: &crate::state::DemoProgression,
    sector: &crate::state::SectorState,
    last_mission_report: &crate::state::LastMissionReport,
    encounter_registers: &EncounterRegisterState,
) -> u64 {
    stable_hash_json(&CanonicalSessionHash {
        app_state,
        ship,
        progression,
        sector,
        last_mission_report,
        encounter_registers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hash_is_repeatable() {
        let a = stable_hash_bytes(b"lumenarch");
        let b = stable_hash_bytes(b"lumenarch");
        assert_eq!(a, b);
    }
}
