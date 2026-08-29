//! The three `Addr` conversions and the two `Revision` conversions (O13 / E15).
//!
//! Presenter addresses carry a significant bit length. Under O32 that length is
//! `8 × key.len()` for the slot encoding — structural from the byte length, never
//! inferred by scanning for a last non-zero nibble.

use infinite_compositor::core::Addr as CompositorAddr;
use infinite_presenter::core::{Addr as PresenterAddr, Revision as PresenterRevision};
use infinite_runtime::core::{Addr as RuntimeAddr, Revision as RuntimeRevision};

/// Store bytes → runtime address. A wrap, nothing else.
pub fn runtime_addr(bytes: &[u8]) -> RuntimeAddr {
    RuntimeAddr::new(bytes.to_vec())
}

/// Store bytes → compositor address. A wrap, nothing else.
pub fn compositor_addr(bytes: &[u8]) -> CompositorAddr {
    CompositorAddr::new(bytes.to_vec())
}

/// Store bytes → presenter address with carried significant length (E15 / O32).
///
/// Bits are `8 × bytes.len()` under the fixed-width slot encoding. Callers that
/// already know bits may use [`presenter_addr_with_bits`].
pub fn presenter_addr(bytes: &[u8]) -> PresenterAddr {
    PresenterAddr::with_bits(bytes.to_vec(), bits_of(bytes))
}

/// Presenter address with an explicit carried length (mint / bootstrap).
#[allow(dead_code)]
pub fn presenter_addr_with_bits(bytes: &[u8], bits: u32) -> PresenterAddr {
    PresenterAddr::with_bits(bytes.to_vec(), bits)
}

/// Significant bits for a key under the E15 slot encoding: eight per byte.
pub fn bits_of(bytes: &[u8]) -> u32 {
    (bytes.len() as u32).saturating_mul(8)
}

/// Deprecated name for [`bits_of`]. Kept so older tests compile until updated.
///
/// Must not be used to *infer* depth from zero nibbles — that path is gone (E15).
pub fn significant_bits(bytes: &[u8]) -> u32 {
    bits_of(bytes)
}

/// Store revision sequence → runtime revision. A wrap, nothing else.
pub fn runtime_revision(n: u64) -> RuntimeRevision {
    RuntimeRevision::new(n)
}

/// Store revision sequence → presenter revision. A wrap, nothing else.
pub fn presenter_revision(n: u64) -> PresenterRevision {
    PresenterRevision::new(n)
}

#[cfg(test)]
mod tests {
    use super::{bits_of, presenter_addr};

    #[test]
    fn bits_follow_byte_length() {
        assert_eq!(bits_of(&[]), 0);
        assert_eq!(bits_of(&[0x10]), 8);
        assert_eq!(bits_of(&[0x10, 0x00, 0x01]), 24);
        assert_eq!(bits_of(&[0x10, 0x00, 0x01, 0x00, 0x01]), 40);
    }

    #[test]
    fn presenter_addr_carries_length_bits() {
        let a = presenter_addr(&[0x10, 0x00, 0x01]);
        assert_eq!(a.prefix_bits(), 24);
        assert_eq!(a.as_bytes(), &[0x10, 0x00, 0x01]);
    }
}
