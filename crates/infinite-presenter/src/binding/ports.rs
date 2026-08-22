//! The three ports (D29, spec §3.1).
//!
//! The presenter declares what it needs as traits; the platform facade supplies the
//! implementations. **A fourth port requires a decision record.**
//!
//! Three is fewer than either sibling layer's five, and R27 is the reason: a port is a
//! defect until a named consumer requires it. The count going 5 → 5 → 3 across store-
//! facing layers is worth noticing rather than smoothing over — D14 states the
//! platform's job in one sentence and that sentence says nothing about drawing.
//!
//! **There is no `Clock`**, stated positively so it stays true: the presenter has no
//! *now* (R10, D5). Hysteresis is a function of zoom, not of time; animation, when a
//! consumer asks for it, is the runtime driving [`crate::binding::frame`] with a
//! changing view. If this layer ever needs a clock, R10 has been violated.
//!
//! **There is no write port, and that is L6 made structural.** The presenter cannot
//! author anything because it has nothing to author with. Compare `RUNTIME.md` §3.1,
//! where `StoreWrite` exists and is the entire subject of D24; the absence here is the
//! check.
//!
//! O10 note: [`scene::Scene`] is where a *"may this viewer see that space"* check
//! would be inserted, and a placement that has already been built is too late. Do not
//! build it so that it cannot be.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod glyphs;
mod scene;
mod surface;

pub use glyphs::Glyphs;
pub use scene::Scene;
pub use surface::Surface;
