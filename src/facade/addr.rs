//! The three `Addr` conversions and the two `Revision` conversions (O13).
//!
//! Two of the three are still a newtype wrap of the inner value. The third is not,
//! and that is D45: **the presenter's address carries a significant bit length, and
//! this file is the only place that knows how to compute one.**
//!
//! O13's trigger was *"the moment a conversion needs logic, promote `Addr` to a
//! zero-dependency crate."* It has fired, and the promotion is **deferred, with the
//! trigger restated** (D45): the logic is one function over the editor's key scheme,
//! the runtime's and the compositor's addresses do not want it, and moving three
//! types into a crate to share a function only one of them calls would be generality
//! without a consumer (R27). The trigger becomes: *when a second layer needs the
//! significant length.* Do not add a fourth kind of logic here without answering it.

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

/// Store bytes → presenter address, with the significant length the key scheme
/// implies (D45).
///
/// The whole key goes across — a probe must answer with bytes the store can be asked
/// about — and [`significant_bits`] says how much of it is address rather than
/// padding. Without this the presenter sees four bytes for the screen root, the
/// canvas and a node alike, `Addr::contains` is satisfiable only by equality, and
/// nesting is structurally unreachable. That was finding 19.
pub fn presenter_addr(bytes: &[u8]) -> PresenterAddr {
    PresenterAddr::with_bits(bytes.to_vec(), significant_bits(bytes))
}

/// How many leading bits of a store key are address rather than padding.
///
/// Four bits per level, and **no level's nibble is zero** — children are numbered
/// from one (`editor::addresses`). So the significant length is four times the
/// position of the last non-zero nibble, and an all-zero key is the root of
/// everything, significant to nothing.
///
/// The rule lives here rather than in `editor::addresses` because the facade is what
/// hands addresses to a layer, and a layer may not name the editor (R2). It is stated
/// in both places and checked in one: `a_well_known_key_is_a_hierarchy`.
pub fn significant_bits(bytes: &[u8]) -> u32 {
    let mut last = 0usize;
    for (i, byte) in bytes.iter().enumerate() {
        if byte >> 4 != 0 {
            last = i * 2 + 1;
        }
        if byte & 0x0F != 0 {
            last = i * 2 + 2;
        }
    }
    (last as u32).saturating_mul(4)
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
    use super::significant_bits;

    #[test]
    fn a_key_is_significant_to_its_last_non_zero_nibble() {
        assert_eq!(significant_bits(&[]), 0);
        assert_eq!(significant_bits(&[0x00, 0x00, 0x00, 0x00]), 0);
        assert_eq!(significant_bits(&[0x10, 0x00, 0x00, 0x00]), 4);
        assert_eq!(significant_bits(&[0x11, 0x00, 0x00, 0x00]), 8);
        assert_eq!(significant_bits(&[0x11, 0x10, 0x00, 0x00]), 12);
        assert_eq!(significant_bits(&[0x11, 0x11, 0x00, 0x00]), 16);
        assert_eq!(significant_bits(&[0xFF, 0xFF, 0xFF, 0xFF]), 32);
    }
}
