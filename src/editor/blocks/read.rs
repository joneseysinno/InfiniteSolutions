//! `read` — an address → a value.

/// The address whose value should be read. The facade Primitive fetches it.
pub fn read(address: &[u8]) -> Vec<u8> {
    address.to_vec()
}
