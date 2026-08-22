//! [`Extent`] — how much room a thing wants along one axis.

/// A one-dimensional size request: a floor, a preference, and a share of surplus.
///
/// Salvaged verbatim from `Innovator/crates/hyper-ui/src/container/extent.rs`, which
/// is 36 lines, holds no identity, and is right. It is one of three things this layer
/// takes from that crate wholesale — the others are the asymmetric hysteresis rule
/// (spec §7.3) and depth-first paint order.
///
/// Logical units, device-independent. **Never a ratio**, which is the note the
/// original carries and the reason `weight` is separate from `ideal`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Extent {
    /// Below this, the thing is not worth showing at all.
    pub min: f64,
    /// What it would like.
    pub ideal: f64,
    /// Its share of any surplus. Zero means fixed.
    pub weight: f64,
}

impl Extent {
    /// An extent.
    pub const fn new(min: f64, ideal: f64, weight: f64) -> Self {
        Self { min, ideal, weight }
    }

    /// A thing that takes exactly this much and never more.
    pub const fn fixed(size: f64) -> Self {
        Self::new(size, size, 0.0)
    }

    /// A thing that would like `ideal`, will accept `min`, and takes an even share of
    /// anything left over.
    pub const fn preferred(min: f64, ideal: f64) -> Self {
        Self::new(min, ideal, 1.0)
    }
}
