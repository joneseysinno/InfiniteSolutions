//! [`Point`] — a position, in whichever space is under discussion.

/// A position in two dimensions.
///
/// Which space it is a position *in* is carried by the context, never by the type.
/// That is deliberate and it is L5: a `Point` that knew which space it belonged to
/// would be a `Point` carrying identity.
///
/// `f64`, like everything else in this layer (spec §3.3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// First coordinate.
    pub x: f64,
    /// Second coordinate.
    pub y: f64,
}

impl Point {
    /// The origin.
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    /// A point.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Componentwise sum.
    pub fn add(self, other: Point) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    /// Componentwise difference.
    pub fn sub(self, other: Point) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    /// Uniform scale.
    pub fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}
