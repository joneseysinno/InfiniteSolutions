//! [`View`] — the argument to [`crate::core::place`] (spec §5.1).

use crate::core::camera::Camera;
use crate::core::surface_rect::SurfaceRect;
use crate::core::transform::Transform;

/// Everything about where you are looking from.
///
/// A `View` is to [`crate::core::place`] what a `DefinitionSet` is to the compositor's
/// `link`: the whole of the context, handed in, so the function is pure and the answer
/// is reproducible.
///
/// **There is no focus in a `View`**, and the absence is deliberate (R27). Focus
/// drives *priority* — which space is rebuilt first — and priority is the runtime's
/// (`RUNTIME.md` §5.1, breakage B6). Nothing this layer computes depends on where
/// attention is; detail depends on zoom and on a space's own override (spec §7), which
/// is what D20 means by *detail is per space, not per camera*.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// Where you are looking from.
    pub camera: Camera,
    /// What you are looking at it through.
    pub surface: SurfaceRect,
    /// How far outside the surface to keep things placed, in surface units.
    ///
    /// A margin trades a little wasted work for no popping at the edge. It is a
    /// property of the view rather than a constant, because the right value depends on
    /// how fast the camera is being moved, and only the caller knows that.
    pub margin: f64,
    /// How large a space must appear, in device pixels, before its interior is shown
    /// (D45).
    ///
    /// This is the whole of *"zoom crosses the node/space seam"*, as a number. A
    /// space whose apparent extent reaches it is entered; below it the same space is
    /// a node in its parent's graph. It is a property of the view and not a constant
    /// for the reason `margin` is: what counts as legible depends on what is being
    /// drawn, and only the caller knows.
    ///
    /// It is **not** a comparison of address bits. Address depth says who is inside
    /// whom; it does not say when you can see in. Conflating the two is finding 19.
    pub opening_extent: f64,
}

impl View {
    /// The opening extent a caller gets when it does not name one.
    ///
    /// Roughly the smallest square in which two nested things and the gap between
    /// them are still distinguishable at ordinary display densities. A number, stated
    /// once, rather than five call sites each picking their own.
    pub const DEFAULT_OPENING_EXTENT: f64 = 256.0;

    /// A view, opening spaces at [`Self::DEFAULT_OPENING_EXTENT`].
    pub const fn new(camera: Camera, surface: SurfaceRect, margin: f64) -> Self {
        Self {
            camera,
            surface,
            margin,
            opening_extent: Self::DEFAULT_OPENING_EXTENT,
        }
    }

    /// The same view, opening spaces at `extent` device pixels instead.
    pub const fn opening_at(self, extent: f64) -> Self {
        Self {
            opening_extent: extent,
            ..self
        }
    }

    /// **The** transform: root-space coordinates to surface coordinates.
    ///
    /// > The transform that culls is the transform that draws. One function, called
    /// > twice.
    ///
    /// [`crate::core::visible`] inverts this; [`crate::core::place`] applies it. There
    /// is no second derivation of the same relationship anywhere in the layer, which
    /// is what makes the agreement test (spec §6.4) a property rather than a hope.
    pub fn embedding(&self) -> Transform {
        let middle = self.surface.origin.add(self.surface.size.scale(0.5));
        let zoom = self.camera.zoom;
        Transform::new(zoom, middle.sub(self.camera.centre.scale(zoom)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Camera, SurfaceRect, View};
    use crate::core::Point;

    fn view(origin: Point) -> View {
        View::new(
            Camera::new(Point::new(100.0, 50.0), 2.0),
            SurfaceRect::new(origin, Point::new(800.0, 600.0), 1.0),
            0.0,
        )
    }

    #[test]
    fn the_camera_centre_lands_in_the_middle_of_the_surface() {
        let v = view(Point::ORIGIN);
        let middle = v.embedding().apply(v.camera.centre);
        assert!((middle.x - 400.0).abs() < 1e-12);
        assert!((middle.y - 300.0).abs() < 1e-12);
    }

    #[test]
    fn a_surface_that_is_not_at_the_corner_shifts_the_whole_embedding() {
        let shifted = view(Point::new(37.0, 11.0));
        let middle = shifted.embedding().apply(shifted.camera.centre);
        assert!((middle.x - 437.0).abs() < 1e-12);
        assert!((middle.y - 311.0).abs() < 1e-12);
    }
}
