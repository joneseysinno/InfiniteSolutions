//! [`Rect`] — a half-open rectangle.

use crate::core::point::Point;

/// A rectangle, as a low corner and a high corner.
///
/// **Half-open: it owns its low edges and not its high ones.** This mirrors the
/// store's own cell convention — *"cell boundaries are half-open (each cell owns its
/// low faces) so that every spatial point has exactly one containing cell at every
/// order"* (`infinitedb-spatial-layer.md` §3) — and it is the reason
/// [`crate::core::probe`] can give one answer rather than two.
///
/// `hyper-ui`'s `Rect::contains` is inclusive on both edges, so adjacent rectangles
/// both claim the pixel between them and a hit test on a tight grid is ambiguous by
/// construction. It has no test.
///
/// Min/max rather than origin/size, for one reason: `hyper-ui` carries **both**
/// representations, in different precisions, with no conversion between them anywhere
/// in the crate. One name, one thing (R17) applies to shapes as much as to types.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Low corner. Owned.
    pub min: Point,
    /// High corner. Not owned.
    pub max: Point,
}

impl Rect {
    /// A rectangle from two corners.
    pub const fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    /// Whether the rectangle has no interior.
    pub fn is_empty(&self) -> bool {
        !(self.max.x > self.min.x && self.max.y > self.min.y)
    }

    /// Whether `p` is inside, low edges included and high edges excluded.
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    /// The overlap of two rectangles. Empty when they do not overlap.
    pub fn intersect(&self, other: &Rect) -> Rect {
        Rect::new(
            Point::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y)),
            Point::new(self.max.x.min(other.max.x), self.max.y.min(other.max.y)),
        )
    }

    /// The rectangle grown by `by` on every side.
    ///
    /// This is where a cull margin is applied, and it is applied to the **one**
    /// rectangle both the cull and the draw are derived from (spec §6.4).
    pub fn inflate(&self, by: f64) -> Rect {
        Rect::new(
            Point::new(self.min.x - by, self.min.y - by),
            Point::new(self.max.x + by, self.max.y + by),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, Rect};

    fn unit() -> Rect {
        Rect::new(Point::ORIGIN, Point::new(1.0, 1.0))
    }

    #[test]
    fn the_low_edge_is_owned_and_the_high_edge_is_not() {
        assert!(unit().contains(Point::ORIGIN));
        assert!(!unit().contains(Point::new(1.0, 1.0)));
    }

    #[test]
    fn adjacent_rectangles_do_not_both_claim_the_seam() {
        let left = unit();
        let right = Rect::new(Point::new(1.0, 0.0), Point::new(2.0, 1.0));
        let seam = Point::new(1.0, 0.5);
        assert!(!left.contains(seam));
        assert!(right.contains(seam));
    }

    #[test]
    fn disjoint_rectangles_intersect_to_nothing() {
        let far = Rect::new(Point::new(9.0, 9.0), Point::new(10.0, 10.0));
        assert!(unit().intersect(&far).is_empty());
    }
}
