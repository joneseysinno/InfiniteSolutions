//! [`compose`] — resolve the scene, then place it (spec §9).
//!
//! # `frame` is retired, not recycled (O21, D47, R17)
//!
//! What lived here was `frame(scene, surface, view, at)`: it resolved a `SceneSet`
//! internally, placed it, submitted the placement, and **dropped the set**. That was
//! adequate until D44 made fill resolution need the very set the placement was built
//! from, and D46 made batching need it too — so `Store::draw_with` took the three
//! steps itself, `frame` lost its last caller, and R27 makes an uncalled binding
//! function a defect rather than a spare part. Finding 17 and O21.
//!
//! The name is retired rather than reused (R17). What replaces it is a function with
//! the signature `frame`'s could not have: **the set comes back out**, and submitting
//! is the caller's, because only the caller can resolve a fill — a style key is the
//! app's and this crate may not name the app. Two consumers now want *"a placement
//! and the scene it came from"*: the fill map and the batch walk. That is what the
//! function is for.

use crate::binding::ports::Scene;
use crate::core::{place, Addr, Placement, Revision, SceneSet, View};

/// Resolves everything the view might place, and places it.
///
/// Two steps, and the third — submitting — is deliberately not here. The caller owns
/// it because the caller owns the `Surface` and the meaning of a style key.
///
/// `prior` is the previous frame's placement, so [`place`] can thread last-drawn
/// levels into hysteresis. The first frame passes `None`.
pub fn compose(
    scene: &dyn Scene,
    view: &View,
    at: Revision,
    prior: Option<&Placement>,
) -> (SceneSet, Placement) {
    let start = Addr::new(Vec::new());
    let end = Addr::new(vec![0xFF; 8]);
    let set = scene.placed_in(&start, &end, at);
    let placement = place(&set, view, prior);
    (set, placement)
}
