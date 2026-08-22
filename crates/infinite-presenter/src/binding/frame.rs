//! [`frame`] — place, then hand the work to the surface (spec §9).

use crate::binding::ports::{Scene, Surface};
use crate::core::{place, Addr, Revision, View};

/// Draws one frame.
///
/// Resolve the visible range through [`Scene`], place it, submit it through
/// [`Surface`]. Three steps, no fourth.
pub fn frame(scene: &dyn Scene, surface: &mut dyn Surface, view: &View, at: Revision) {
    let start = Addr::new(Vec::new());
    let end = Addr::new(vec![0xFF; 8]);
    let set = scene.placed_in(&start, &end, at);
    let placement = place(&set, view);
    surface.submit(&placement);
}
