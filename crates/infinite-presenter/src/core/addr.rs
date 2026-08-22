//! [`Addr`] — the store's identity, as the presenter sees it (D29, spec §3.2).

/// An opaque, totally-ordered byte key.
///
/// The pure core depends on nothing (R3), so it cannot use the store's key type. This
/// is the third definition in the workspace; the runtime's and the compositor's are
/// the other two, and O13 records the trigger for promoting it to a crate of its own.
/// Worth knowing before assuming they are interchangeable: **this one and the
/// runtime's want the same three operations, and the compositor's deliberately wants
/// fewer** (spec §3.2).
///
/// Three properties, all supplied by the store:
///
/// 1. **Total order, and that order is spatial locality.** A subtree is one contiguous
///    key range (`infinitedb-spatial-layer.md` §10), which is exactly the range a cull
///    asks for. It is also why [`crate::core::probe`] descends in O(depth) instead of
///    scanning: address order *is* spatial order, so the search is a walk.
/// 2. **Truncation is level.** Level is the key truncated to a number of bits, and
///    that is the whole of this layer's detail model (spec §7).
/// 3. **Permanence.** An address, once issued, stays valid under refinement. Relied
///    upon, never verified — it is the store's invariant.
///
/// Note what is **not** here: the chart dimension *D*. Level is conventionally ℓ·*D*
/// bits, but charts need not share a dimension (`infinitedb-spatial-layer.md` §2), so
/// this layer counts bits and never learns what a dimension is. A thing you never
/// learn is a thing you cannot be wrong about.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Addr(Box<[u8]>);

impl Addr {
    /// Wraps raw key bytes.
    pub fn new(bytes: impl Into<Box<[u8]>>) -> Self {
        Self(bytes.into())
    }

    /// The raw key bytes, for handing back to a port. Never for interpretation.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// How many bits long this address is.
    pub fn prefix_bits(&self) -> u32 {
        (self.0.len() as u32).saturating_mul(8)
    }

    /// This address truncated to `bits` — the ancestor at that level.
    ///
    /// Truncating past the end returns the address unchanged: an address is already
    /// its own ancestor at every level below its own. Bits that fall inside the last
    /// retained byte are cleared rather than left dangling, so two addresses that
    /// share a level truncate to the *same* value, which is the only property the
    /// detail model needs of this function.
    pub fn truncate(&self, bits: u32) -> Self {
        if bits >= self.prefix_bits() {
            return self.clone();
        }
        let whole = (bits / 8) as usize;
        let spare = bits % 8;
        let keep = if spare == 0 { whole } else { whole + 1 };
        let mut bytes = self.0[..keep].to_vec();
        if spare != 0 {
            let mask = 0xFFu8 << (8 - spare);
            let last = keep - 1;
            bytes[last] &= mask;
        }
        Self(bytes.into())
    }

    /// Whether this address falls inside `[start, end)`.
    pub fn in_range(&self, start: &Self, end: &Self) -> bool {
        self >= start && self < end
    }

    /// Whether `other` lies inside the subtree this address roots.
    ///
    /// Byte-prefix containment, which under property 1 is subtree containment.
    pub fn contains(&self, other: &Self) -> bool {
        other.0.len() >= self.0.len() && other.0[..self.0.len()] == *self.0
    }
}

impl std::fmt::Debug for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Addr(")?;
        for byte in self.0.iter() {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::Addr;

    #[test]
    fn truncating_to_a_byte_boundary_keeps_whole_bytes() {
        let a = Addr::new(vec![0xAB, 0xCD, 0xEF]);
        assert_eq!(a.truncate(16), Addr::new(vec![0xAB, 0xCD]));
    }

    #[test]
    fn truncating_inside_a_byte_clears_the_spare_bits() {
        let a = Addr::new(vec![0b1111_1111]);
        assert_eq!(a.truncate(3), Addr::new(vec![0b1110_0000]));
    }

    #[test]
    fn siblings_at_a_level_truncate_to_one_value() {
        let left = Addr::new(vec![0xAB, 0b0000_0001]);
        let right = Addr::new(vec![0xAB, 0b0000_0010]);
        assert_eq!(left.truncate(12), right.truncate(12));
    }

    #[test]
    fn truncating_past_the_end_is_the_identity() {
        let a = Addr::new(vec![0xAB]);
        assert_eq!(a.truncate(64), a);
    }

    #[test]
    fn a_subtree_is_a_prefix() {
        let parent = Addr::new(vec![0xAB]);
        let child = Addr::new(vec![0xAB, 0x01]);
        assert!(parent.contains(&child));
        assert!(!child.contains(&parent));
    }

    #[test]
    fn order_is_byte_order() {
        let mut addrs = [
            Addr::new(vec![0x02]),
            Addr::new(vec![0x01, 0xFF]),
            Addr::new(vec![0x01]),
        ];
        addrs.sort();
        assert_eq!(addrs[0], Addr::new(vec![0x01]));
        assert_eq!(addrs[1], Addr::new(vec![0x01, 0xFF]));
        assert_eq!(addrs[2], Addr::new(vec![0x02]));
    }
}
