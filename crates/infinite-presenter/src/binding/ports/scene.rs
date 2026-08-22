//! The `Scene` port.

use crate::core::{Addr, Revision, SceneSet};

/// What is placed in an address range, at a revision.
///
/// Reads a **set**, resolved against a revision — the same shape as the compositor's
/// `Definitions` (D26), and for the same reason: the editor must place and probe
/// geometry that is still **pending** (D8), which is by definition not in the store.
/// The binding resolves the stored set through this port and overlays the runtime's
/// pending set before handing the result to [`crate::core::place`].
///
/// It reads and does not write. There is no companion `SceneWrite`, by L6.
pub trait Scene {
    /// Everything placeable whose address lies in `[start, end)` at `at`.
    ///
    /// One range, not a predicate per thing: a subtree is contiguous in key order
    /// (`infinitedb-spatial-layer.md` §10), so a cull is a range scan.
    fn placed_in(&self, start: &Addr, end: &Addr, at: Revision) -> SceneSet;

    /// The camera, which is authored, session-scoped store state (D5, spec §6.2).
    ///
    /// Read here rather than owned, because it survives a restart if the person wants
    /// it to, and D5's membership test settles that.
    fn camera(&self, of: &Addr, at: Revision) -> Option<crate::core::Camera>;
}
