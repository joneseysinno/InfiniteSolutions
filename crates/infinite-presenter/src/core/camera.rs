//! [`Camera`] — where you are looking from, and nothing else (spec §6.2).

use crate::core::point::Point;

/// A centre and a magnification.
///
/// **That is the whole type, and the omissions are the design.**
/// `hyper-ui`'s `SceneCamera` also carries the surface's pixel size, the surface's
/// origin, and a `user_adjusted` policy flag. The first two belong to the surface, and
/// keeping them here is precisely how that crate's cull path and draw path came to
/// disagree: `screen_to_world` accounts for the surface origin and
/// `visible_world_rect` does not, so whenever the canvas is not at the window's corner
/// the culled region is offset from what is drawn. Its own doc comment describes that
/// bug being fixed — in the draw path only.
///
/// The third omission is different in kind: `user_adjusted` is documented as *"stops
/// auto-fit taking over"* and is enforced entirely at the call site, which makes it a
/// comment wearing a type. Policy belongs to whoever decides the camera, and under D5
/// that is the store — the camera is authored, session-scoped state, in the manner of
/// *"which pod has focus"*. This layer reads it (L6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// The point in root-space coordinates that sits at the middle of the surface.
    pub centre: Point,
    /// Surface units per root-space unit.
    pub zoom: f64,
}

impl Camera {
    /// A camera.
    pub const fn new(centre: Point, zoom: f64) -> Self {
        Self { centre, zoom }
    }
}
