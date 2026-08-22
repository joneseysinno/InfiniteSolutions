//! Renders a compositor [`Finding`](infinite_compositor::core::Finding) and turns
//! the presenter's precision-floor *fact* into one (`PRESENTER.md` §10).

use infinite_compositor::core::Finding;

use super::addr::compositor_addr;

/// Turns a precision-floor address into a finding with a remedy.
pub fn from_precision_floor(site: &[u8]) -> Finding {
    Finding::new(
        compositor_addr(site),
        "precision-floor",
        "this space is refined past what the screen can distinguish at this zoom",
        "a level the surface can still resolve",
        "zoom in to work in this space",
    )
}

/// An emptied screen is a finding, never a black frame (`PRESENTER.md` §13 finding 8).
pub fn from_empty_screen(site: &[u8]) -> Finding {
    Finding::new(
        compositor_addr(site),
        "empty-screen",
        "the editor's screen root has no spaces",
        "a canvas authored by genesis",
        "re-run genesis to restore the screen",
    )
}
