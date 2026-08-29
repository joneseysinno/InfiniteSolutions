//! The placement's registration parts (D25, spec §4).

use crate::core::{place, Addr, Placement, SceneSet, View};

/// The string key the placement is registered under.
pub const KEY: &str = "placement";

/// The address ranges a placement derives from, given the view it was built for.
///
/// The visible region is a rectangle; turning it into a key range is the store's
/// order, which this crate does not interpret. One wide range is the honest
/// declaration: any address the scene can name may appear.
pub fn ranges(_view: &View) -> Vec<(Addr, Addr)> {
    vec![(Addr::new(Vec::new()), Addr::new(vec![0xFF; 8]))]
}

/// The rebuild function the facade hands to the runtime: [`place`].
///
/// Artifact rebuild has no frame history, so prior levels are `None`.
pub fn rebuild(scene: &SceneSet, view: &View) -> Placement {
    place(scene, view, None)
}
