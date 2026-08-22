//! [`SurfaceRect`] — the shape of the thing being drawn into.

use crate::core::point::Point;
use crate::core::rect::Rect;

/// Where the drawable area is, how big it is, and how many device pixels a unit is.
///
/// Reported by the `Surface` port, never guessed and never read from a window: this
/// layer has no window, in the same way the runtime has no thread (L1). The origin is
/// carried explicitly and separately from the camera because the corpus's one live
/// embedding bug is exactly what happens when it is not (spec §6.2, §6.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceRect {
    /// Top-left of the drawable area, in the host's coordinates.
    pub origin: Point,
    /// Its size, in logical units.
    pub size: Point,
    /// Device pixels per logical unit. Used by the `Surface` implementation, and by
    /// the precision floor (spec §10). Never by the transform arithmetic.
    pub scale_factor: f64,
}

impl SurfaceRect {
    /// A surface rectangle.
    pub const fn new(origin: Point, size: Point, scale_factor: f64) -> Self {
        Self {
            origin,
            size,
            scale_factor,
        }
    }

    /// The drawable area as a rectangle.
    pub fn rect(&self) -> Rect {
        Rect::new(self.origin, self.origin.add(self.size))
    }
}
