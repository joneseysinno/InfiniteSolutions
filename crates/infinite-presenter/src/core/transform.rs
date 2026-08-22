//! [`Transform`] — one similarity, embedding one space in its parent (spec §6.1).

use crate::core::point::Point;
use crate::core::rect::Rect;

/// A similarity: uniform scale, then translation.
///
/// > **The presenter holds one `Transform` per *visited space*. A thing's position is
/// > the fold of the transforms along its path. There is no per-thing transform,
/// > ever.**
///
/// That is not an optimization. `infinitedb-spatial-layer.md` §7 (invariant 7) states
/// it as a property of the store: *"its relationship to the outside world is a single
/// similarity transform … the only thing that changes is that one embedding transform;
/// the subtree moves rigidly."* Holding one per thing would be a map from identity to
/// geometry, which is F-2's shape in this layer, and it would make a pan O(nodes)
/// instead of O(1).
///
/// A similarity and not a matrix: rotation and shear have no named consumer (R27),
/// and two multiplications compose exactly where a general matrix accumulates error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Uniform scale. Never zero for a transform that is used.
    pub scale: f64,
    /// Translation, applied after the scale.
    pub offset: Point,
}

impl Transform {
    /// The transform that changes nothing.
    pub const IDENTITY: Transform = Transform {
        scale: 1.0,
        offset: Point::ORIGIN,
    };

    /// A transform.
    pub const fn new(scale: f64, offset: Point) -> Self {
        Self { scale, offset }
    }

    /// Maps a point outward — from this space's coordinates into its parent's.
    pub fn apply(&self, p: Point) -> Point {
        p.scale(self.scale).add(self.offset)
    }

    /// Maps a rectangle outward. Corner order survives because the scale is positive.
    pub fn apply_rect(&self, r: &Rect) -> Rect {
        Rect::new(self.apply(r.min), self.apply(r.max))
    }

    /// The transform that undoes this one.
    ///
    /// Returns [`Transform::IDENTITY`] for a degenerate scale rather than producing an
    /// infinity. A zero scale means a space compressed out of existence, which is a
    /// detail decision (spec §7) that should have culled it — so the arithmetic
    /// refuses to be the place that reports it.
    pub fn invert(&self) -> Transform {
        if self.scale == 0.0 || !self.scale.is_finite() {
            return Transform::IDENTITY;
        }
        let inv = 1.0 / self.scale;
        Transform::new(inv, self.offset.scale(-inv))
    }

    /// This transform followed by `outer` — the fold along a path.
    ///
    /// Applying the result is applying `self` and then `outer`, which is what makes a
    /// thing's screen position a fold over its ancestors rather than a stored value.
    pub fn then(&self, outer: &Transform) -> Transform {
        Transform::new(self.scale * outer.scale, outer.apply(self.offset))
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, Transform};

    fn sample() -> Transform {
        Transform::new(2.0, Point::new(10.0, -4.0))
    }

    #[test]
    fn inverting_round_trips() {
        let t = sample();
        let p = Point::new(3.0, 7.0);
        let back = t.invert().apply(t.apply(p));
        assert!((back.x - p.x).abs() < 1e-12);
        assert!((back.y - p.y).abs() < 1e-12);
    }

    #[test]
    fn folding_two_transforms_equals_applying_them_in_turn() {
        let inner = Transform::new(0.5, Point::new(1.0, 2.0));
        let outer = sample();
        let p = Point::new(-3.0, 4.0);
        let folded = inner.then(&outer).apply(p);
        let stepwise = outer.apply(inner.apply(p));
        assert!((folded.x - stepwise.x).abs() < 1e-12);
        assert!((folded.y - stepwise.y).abs() < 1e-12);
    }

    #[test]
    fn a_degenerate_scale_inverts_to_the_identity_rather_than_an_infinity() {
        let flat = Transform::new(0.0, Point::new(5.0, 5.0));
        assert_eq!(flat.invert(), Transform::IDENTITY);
    }
}
