//! [`Revision`] — the store's logical clock, as the runtime sees it.

/// A store revision.
///
/// The store has logical time; the runtime has *now* (D5). A `Revision` crosses the
/// boundary in one direction only: the runtime receives it, orders by it, and hands it
/// back. It is never interpreted, and no arithmetic beyond comparison is defined on it.
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
