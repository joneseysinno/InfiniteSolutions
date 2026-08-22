//! `commit` — an address → committed.

/// The commit boundary, authored rather than implicit. The facade Primitive
/// crosses it.
pub fn commit(address: &[u8]) -> Vec<u8> {
    address.to_vec()
}
