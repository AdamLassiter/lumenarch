use serde::Serialize;

use crate::protocol::SessionSnapshot;

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
    let hash = stable_hash_json(&snapshot);
    snapshot.state_hash = hash;
    snapshot
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
