//! `amend` — (address, value) → pending.

/// The write path, and the only one (D24). The facade Primitive amends pending.
pub fn amend(address: &[u8], value: &[u8]) -> (Vec<u8>, Vec<u8>) {
    (address.to_vec(), value.to_vec())
}
