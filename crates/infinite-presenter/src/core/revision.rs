//! [`Revision`] — the store's logical clock, as the presenter sees it.

/// A store revision.
///
/// A [`crate::core::SceneSet`] is resolved *at* a revision, and a
/// [`crate::core::Placement`] is valid *through* one — which is the validity watermark
/// D25's artifact registry asks for. The presenter receives it, compares it, and hands
/// it back; no arithmetic beyond comparison is defined on it.
///
/// Defined here rather than shared, for R3's reason and no other: the pure core
/// depends on nothing. It is now the second copy in the workspace, and the
/// specification records that under O13 alongside `Addr` rather than pretending it is
/// free.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(u64);

impl Revision {
    /// The revision before any write.
    pub const ZERO: Revision = Revision(0);

    /// Wraps a raw revision number from a port.
    pub const fn new(n: u64) -> Self {
        Self(n)
    }

    /// The raw revision number, for handing back to a port.
    pub const fn get(self) -> u64 {
        self.0
    }
}
