//! The `Provenance` port.

use crate::core::Addr;

/// Records what was computed from what, at which revision (D11).
///
/// Half of this exists in the store and has never been driven:
/// `infinitedb_core/computation.rs`, `provenance.rs`, hyperedge payload codec V4 =
/// `computation`, `check_hyperedge_freshness`, `query_stale_downstream`, and the
/// `engine/derivation/` bus. The store already knows an edge can carry a computation
/// and that changing an input makes downstream stale. **The compositor is what invokes
/// one and populates that provenance.**
///
/// One declaration, three payoffs — the charter says two:
///
/// | Payoff | Consumer |
/// |---|---|
/// | staleness | the store computes the downstream set (D11) |
/// | compilability | a composition is compilable iff it is a pure function of its declared inputs (D19) |
/// | audit | a stamped result is reproducible from its provenance |
///
/// That is why `not-pure` is a **link-time** finding rather than a compile-time one:
/// the declaration that would make a composition compilable is the one the store needs
/// whether or not anyone ever compiles it.
pub trait Provenance {
    /// One executed step: what it wrote, what it read, and which block did it.
    fn record(&mut self, outputs: &[Addr], inputs: &[Addr], block: &Addr);

    /// The exact declared input set of an output. The S6 green check compares this
    /// against the store's staleness query — no more addresses, and no fewer.
    fn inputs_of(&self, output: &Addr) -> Vec<Addr>;
}
