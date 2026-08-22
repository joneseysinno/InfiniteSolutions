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
}

impl View {
    /// A view.
    pub const fn new(camera: Camera, surface: SurfaceRect, margin: f64) -> Self {
        Self {
            camera,
            surface,
            margin,
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
