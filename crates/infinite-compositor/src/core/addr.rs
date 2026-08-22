//! [`Addr`] — the only thing this layer knows about identity (D26, spec §3.2).

/// An opaque, totally-ordered byte key.
///
/// The compositor **compares** it and takes **ranges** of it. It never interprets it,
/// and it never measures a shared prefix. The store's key type is not named here: the
/// pure core depends on nothing (R3), and a generic parameter would thread through
/// every type in the layer to buy an abstraction with one instantiation (R27).
///
/// This layer needs strictly **less** of an address than the runtime does:
///
/// | Property | Runtime | Compositor |
/// |---|---|---|
/// | equality | yes | **yes** — a wire names two ports; two blocks are the same or they are not |
/// | total order | yes | **yes** — plan order must be deterministic, or D19's equivalence law is statistical rather than exact |
/// | prefix truncation is level | yes | **no** — that is priority by distance from focus, which is the runtime's |
/// | permanence | relied on | relied on — the store's invariant, never verified here |
///
/// **The absence of `shared_prefix_bits` is deliberate and load-bearing.**
/// `infinite-runtime`'s `Addr` has it, because scheduling priority needs it. Adding it
/// here would give the compositor a reason to care how deep a block sits, and no
/// consumer of this layer requires that (R27). If one ever does, the want belongs in a
/// decision record before it belongs in this file.
///
/// Defined separately from the runtime's `Addr` because each pure core depends on
/// nothing. The duplication is recorded as **O13**, with a trigger, rather than solved.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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

    #[test]
    fn equal_bytes_are_the_same_address() {
        assert_eq!(Addr::new(vec![0xAB, 0xCD]), Addr::new(vec![0xAB, 0xCD]));
    }
}
