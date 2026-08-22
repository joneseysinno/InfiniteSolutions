//! [`Region`] — an area marked iterative (D21), structurally.

use crate::core::addr::Addr;

/// An area the author has marked iterative.
///
/// Three facts are this layer's, and in draft 1 of the spec they are all it specifies:
///
/// 1. **From outside, a region is a block.** Declared inputs and outputs; one step in
///    the enclosing plan. D14.6 holds unchanged.
/// 2. **A cycle outside a region is a `cycle` finding**, not a refusal.
/// 3. **A region is still a pure function of its declared inputs**, so D19 holds and a
///    region is compilable.
///
/// Convergence, damping, and non-convergence-as-a-finding are **out of scope** — spec
/// §2 and `RUNTIME.md` §2 make the same cut for the same reason, and the trigger to
/// extend both is the same: the first named consumer with a solve in it.
///
/// The maximum iteration count and the stopping test are **visible properties**, not
/// hidden configuration. D21 requires the loop be drawn on the canvas rather than
/// buried in a black box, because the crane mat's three-way coupling between bending,
/// settlement and contact patch *is* the physics and should be legible.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Region {
    /// The composition the region iterates.
    pub body: Addr,
    /// Termination is guaranteed; non-convergence is a finding, not a hang.
    pub max_iterations: u32,
    /// The block whose output decides whether to stop.
    ///
    /// A block, not an expression — L3, the compositor contains no math. It does not
    /// know *why* a particular loop converges, only that it did or did not.
    pub stopping_test: Addr,
}
