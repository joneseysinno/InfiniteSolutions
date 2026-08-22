//! The `Surface` port.

use crate::core::{Placement, SurfaceRect};

/// The thing being drawn into.
///
/// **The only place a GPU could ever exist, and it does not exist here.** The facade
/// implements this over wgpu; this crate's manifest names no graphics crate, which is
/// D29 and which is what makes the embedding testable at all.
///
/// The presenter decides *what* is submitted, in what order, at what detail, grouped
/// how. The implementation decides *how*, and it is also where `f64` narrows to
/// whatever the device wants — at the last possible moment, once, in one place
/// (spec §3.3).
pub trait Surface {
    /// Where the drawable area is and how big it is.
    ///
    /// Reported, never guessed and never read from a window: this layer has no window,
    /// in the same way the runtime has no thread. Carrying the origin explicitly is
    /// not incidental — the corpus's one live embedding bug is what happens when the
    /// cull path and the draw path disagree about it (spec §6.4).
    fn geometry(&self) -> SurfaceRect;

    /// Accepts a frame's work.
    ///
    /// Takes the placement whole. Instancing, batching and buffer residency for large
    /// numeric geometry are out of scope until a consumer with a solve in it arrives
    /// (spec §2) — the same deferral both sibling specifications make, with the same
    /// trigger.
    fn submit(&mut self, placement: &Placement);
}
