//! [`Addr`] — the store's identity, as the presenter sees it (D29, spec §3.2).

/// An opaque, totally-ordered byte key with a significant bit length.
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
///
/// # The significant length is carried, not inferred (D45)
///
/// An address is a **bit** prefix of a key, and the key it prefixes may be wider than
/// the prefix. Before D45 this type inferred the significant length from the byte
/// length, which asserted two things at once: that a level boundary always falls on a
/// byte, and that a key handed over at its storage width is significant to its storage
/// width. The second is false for any fixed-width key space, and it is false here:
/// `infinite-db`'s editor space is one dimension of 32 bits, so *every* key arrives
/// four bytes wide and `prefix_bits()` came back 32 for the screen root, the canvas
/// and a node alike. [`Self::contains`] was then satisfiable only by equality, and
/// finding 19 followed — `place_group`'s descend guard could never fire, for any
/// genesis, at any depth.
///
/// So the significant length is a second field, supplied by whoever knows the key
/// scheme, which is the facade. This layer still learns nothing about how the bits
/// are allotted. [`Self::new`] keeps the old meaning — an address significant to its
/// whole byte length — which is what the pure core's own tests and
/// [`crate::binding::ranges`] want.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Addr {
    /// The key bytes, at whatever width the store hands them over.
    bytes: Box<[u8]>,
    /// How many leading bits of `bytes` are significant. Never exceeds `8 * len`.
    ///
    /// Ordered after `bytes` so that an ancestor sorts before the descendants that
    /// extend it, which is what keeps a subtree one contiguous range (property 1).
    bits: u32,
}

impl Addr {
    /// Wraps raw key bytes, significant to their whole length.
    pub fn new(bytes: impl Into<Box<[u8]>>) -> Self {
        let bytes = bytes.into();
        let bits = (bytes.len() as u32).saturating_mul(8);
        Self { bytes, bits }
    }

    /// Wraps raw key bytes whose significant prefix is `bits` long (D45).
    ///
    /// `bits` is clamped to the bytes actually present: an address cannot be
    /// significant past its own key.
    pub fn with_bits(bytes: impl Into<Box<[u8]>>, bits: u32) -> Self {
        let bytes = bytes.into();
        let bits = bits.min((bytes.len() as u32).saturating_mul(8));
        Self { bytes, bits }
    }

    /// The raw key bytes, for handing back to a port. Never for interpretation.
    ///
    /// The **whole** key, not the significant prefix, so a probe answers with an
    /// address the store can be asked about without the caller reassembling it.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// How many bits of this address are significant.
    pub fn prefix_bits(&self) -> u32 {
        self.bits
    }

    /// This address truncated to `bits` — the ancestor at that level.
    ///
    /// Truncating past the end returns the address unchanged: an address is already
    /// its own ancestor at every level below its own. Bits that fall inside the last
    /// retained byte are cleared rather than left dangling, so two addresses that
    /// share a level truncate to the *same* value, which is the only property the
    /// detail model needs of this function.
    pub fn truncate(&self, bits: u32) -> Self {
        if bits >= self.bits {
            return self.clone();
        }
        let whole = (bits / 8) as usize;
        let spare = bits % 8;
        let keep = if spare == 0 { whole } else { whole + 1 };
        let mut bytes = self.bytes[..keep].to_vec();
        if spare != 0 {
            let mask = 0xFFu8 << (8 - spare);
            let last = keep - 1;
            bytes[last] &= mask;
        }
        Self {
            bytes: bytes.into(),
            bits,
        }
    }

    /// Whether this address falls inside `[start, end)`.
    pub fn in_range(&self, start: &Self, end: &Self) -> bool {
        self >= start && self < end
    }

    /// Whether `other` lies inside the subtree this address roots.
    ///
    /// **Bit**-prefix containment, which under property 1 is subtree containment. The
    /// comparison runs over this address's significant bits only, so a four-byte key
    /// significant to twelve bits contains every four-byte key that agrees with it in
    /// those twelve — which is what a node hosting a space actually means, and what
    /// byte-prefix containment could not express (D45).
    pub fn contains(&self, other: &Self) -> bool {
        if other.bits < self.bits {
            return false;
        }
        let whole = (self.bits / 8) as usize;
        let spare = self.bits % 8;
        if other.bytes.len() < whole || self.bytes.len() < whole {
            return false;
        }
        if other.bytes[..whole] != self.bytes[..whole] {
            return false;
        }
        if spare == 0 {
            return true;
        }
        let mask = 0xFFu8 << (8 - spare);
        match (self.bytes.get(whole), other.bytes.get(whole)) {
            (Some(mine), Some(theirs)) => mine & mask == theirs & mask,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Addr(")?;
        for byte in self.bytes.iter() {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "/{})", self.bits)
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
        assert_eq!(a.truncate(3), Addr::with_bits(vec![0b1110_0000], 3));
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
    fn containment_is_over_significant_bits_and_not_whole_bytes() {
        // D45, and finding 19's exact shape: three keys of one fixed width, related
        // as canvas → node → interior node. Under byte-prefix containment all three
        // are four bytes long and none contains another.
        let canvas = Addr::with_bits(vec![0x11, 0x00, 0x00, 0x00], 8);
        let node = Addr::with_bits(vec![0x11, 0x10, 0x00, 0x00], 12);
        let inner = Addr::with_bits(vec![0x11, 0x11, 0x00, 0x00], 16);
        let sibling = Addr::with_bits(vec![0x11, 0x20, 0x00, 0x00], 12);

        assert!(canvas.contains(&node) && canvas.contains(&inner));
        assert!(node.contains(&inner));
        assert!(!sibling.contains(&inner));
        assert!(!node.contains(&sibling));
        assert!(!inner.contains(&node), "containment is not symmetric");
    }

    #[test]
    fn an_ancestor_sorts_before_its_descendants_and_they_are_contiguous() {
        // What `SceneSet::subtree` relies on: `range(root..).take_while(contains)`
        // sees the whole subtree and stops at the first thing outside it.
        let mut addrs = [
            Addr::with_bits(vec![0x12, 0x00, 0x00, 0x00], 8),
            Addr::with_bits(vec![0x11, 0x20, 0x00, 0x00], 12),
            Addr::with_bits(vec![0x11, 0x11, 0x00, 0x00], 16),
            Addr::with_bits(vec![0x11, 0x00, 0x00, 0x00], 8),
            Addr::with_bits(vec![0x11, 0x10, 0x00, 0x00], 12),
        ];
        addrs.sort();
        let root = Addr::with_bits(vec![0x11, 0x00, 0x00, 0x00], 8);
        assert_eq!(addrs[0], root);
        let inside: Vec<bool> = addrs.iter().map(|a| root.contains(a)).collect();
        assert_eq!(inside, vec![true, true, true, true, false]);
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
