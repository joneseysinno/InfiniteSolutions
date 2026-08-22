//! [`Addr`] — the only thing the core knows about identity (D23, spec §3.2).

/// An opaque, totally-ordered byte key.
///
/// The runtime **compares** it, takes **ranges** of it, and measures **shared
/// prefixes**. It never interprets it. The store's key type is not named here: the
/// pure core depends on nothing (R3), and a generic parameter would thread through
/// every type in the layer to buy an abstraction with one instantiation (R27).
///
/// Three properties are relied upon, all supplied by the store:
///
/// 1. **Total order, and that order is spatial locality.** This is what makes a range
///    scan of a subtree affordable — O2's placement policy, stated as a requirement.
/// 2. **Prefix truncation is level.** Level ℓ is the key truncated to ℓ·D bits. The
///    core does not know D and does not need it: the one use is scheduling priority by
///    distance from focus, which needs only *shared prefix length*.
/// 3. **Permanence.** An address, once issued, stays valid under refinement. Relied
///    upon, never verified — it is the store's invariant.
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

    /// Number of leading bits this address shares with `other`.
    ///
    /// The single input to scheduling priority (spec §5.1). Property 2 above is why
    /// this means "how close, spatially" rather than "how similar, textually".
    pub fn shared_prefix_bits(&self, other: &Self) -> u32 {
        let mut bits = 0;
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            if a == b {
                bits += 8;
            } else {
                return bits + (a ^ b).leading_zeros();
            }
        }
        bits
    }

    /// Whether this address falls inside `[start, end)`.
    pub fn in_range(&self, start: &Self, end: &Self) -> bool {
        self >= start && self < end
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
    fn shared_prefix_is_zero_for_differing_first_bit() {
        let a = Addr::new(vec![0b0000_0000]);
        let b = Addr::new(vec![0b1000_0000]);
        assert_eq!(a.shared_prefix_bits(&b), 0);
    }

    #[test]
    fn shared_prefix_counts_whole_matching_bytes() {
        let a = Addr::new(vec![0xAB, 0xCD, 0b0000_0000]);
        let b = Addr::new(vec![0xAB, 0xCD, 0b0000_1111]);
        assert_eq!(a.shared_prefix_bits(&b), 20);
    }

    #[test]
    fn a_prefix_shares_all_of_its_own_bits() {
        let parent = Addr::new(vec![0xAB]);
        let child = Addr::new(vec![0xAB, 0x01]);
        assert_eq!(parent.shared_prefix_bits(&child), 8);
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
