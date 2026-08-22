//! [`visible`] — what a view can see, in root-space coordinates (spec §6.3).

use crate::core::rect::Rect;
use crate::core::view::View;

/// The region a view can see, as a rectangle in root-space coordinates.
///
/// **The inverse image of the surface rectangle under the view's own embedding.** Not
/// a second derivation of the same relationship — [`View::embedding`] is called, and
/// inverted, and that is the whole function. This is what makes the agreement test
/// (spec §6.4) hold structurally rather than by inspection.
///
/// A cull is then a **range of addresses**, not a predicate evaluated per thing: under
/// the store's total order a subtree is one contiguous key range
/// (`infinitedb-spatial-layer.md` §10), so the binding turns this rectangle into a
/// range and asks the `Scene` port for it once.
///
/// The counter-example is `hyper-ui`'s `InMemorySpatial`, which despite the name holds
/// no index — it scans every node, clones each hit, allocates a fresh vector per
/// frame, and tests each node's **centre point** while ignoring its size, so a thing
/// straddling the edge is culled while visibly on screen. The camera's fixed 64-unit
/// margin is what has been covering for that.
pub fn visible(view: &View) -> Rect {
    let seen = view.surface.rect().inflate(view.margin);
    view.embedding().invert().apply_rect(&seen)
}

#[cfg(test)]
mod tests {
    use super::visible;
    use crate::core::{Camera, Point, SurfaceRect, View};

    fn view(origin: Point, margin: f64) -> View {
        View::new(
            Camera::new(Point::new(100.0, 50.0), 2.0),
            SurfaceRect::new(origin, Point::new(800.0, 600.0), 1.0),
            margin,
        )
    }

    #[test]
    fn the_visible_region_is_centred_on_the_camera() {
        let r = visible(&view(Point::ORIGIN, 0.0));
        assert!((r.min.x - (100.0 - 200.0)).abs() < 1e-12);
        assert!((r.max.x - (100.0 + 200.0)).abs() < 1e-12);
        assert!((r.min.y - (50.0 - 150.0)).abs() < 1e-12);
        assert!((r.max.y - (50.0 + 150.0)).abs() < 1e-12);
    }

    #[test]
    fn a_surface_origin_does_not_move_what_is_visible() {
        // The bug this test exists for: `hyper-ui` probes the cull rectangle at window
        // coordinates while the draw path accounts for the surface origin, so moving
        // the canvas within the window slides the culled region off the drawn one.
        let at_corner = visible(&view(Point::ORIGIN, 0.0));
        let inset = visible(&view(Point::new(37.0, 11.0), 0.0));
        assert!((at_corner.min.x - inset.min.x).abs() < 1e-12);
        assert!((at_corner.max.y - inset.max.y).abs() < 1e-12);
    }

    #[test]
    fn the_margin_grows_the_region_by_itself_over_the_zoom() {
        let plain = visible(&view(Point::ORIGIN, 0.0));
        let padded = visible(&view(Point::ORIGIN, 64.0));
        assert!((plain.min.x - padded.min.x - 32.0).abs() < 1e-12);
    }
}
