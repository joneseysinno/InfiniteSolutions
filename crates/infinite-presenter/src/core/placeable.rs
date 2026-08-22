//! [`Placeable`] — what the `Scene` port says about one thing.

use crate::core::addr::Addr;
use crate::core::extent::Extent;
use crate::core::point::Point;

/// One thing the presenter could place.
///
/// Everything here arrives from the store through the `Scene` port and is **read**.
/// Nothing on this type is ever written back (L6, R5) — which is P4, the breakage
/// `hyper-ui` has live, where `arrange` writes a measured content extent and a clamped
/// scroll offset back into the very payload it is laying out, so running layout twice
/// is not the same as running it once.
#[derive(Clone, Debug, PartialEq)]
pub struct Placeable {
    /// The store's address. The only identity in this layer (L5).
    pub at: Addr,
    /// How much room it wants across.
    pub across: Extent,
    /// How much room it wants down.
    pub down: Extent,
    /// Authored position in the containing space.
    pub position: Point,
    /// An opaque key naming how it should look.
    ///
    /// Opaque in the sense D13 makes a tag opaque: the presenter carries it to the
    /// `Surface` and never interprets it. A colour, a corner radius or a border is an
    /// app's business, and putting one here is how `SceneNode` ended up with
    /// `selected: bool` baked into a geometry record.
    pub style: Box<str>,
    /// Held open or closed against the view's default level. Authored (D5).
    pub detail_override: Option<i64>,
    /// Whether this space has an interior to descend into.
    pub hosts_space: bool,
    /// Whether a probe may land here.
    ///
    /// Resolved once, at place time, from what the store says — so that
    /// [`crate::core::probe`] never needs a port to answer (spec §8.4). `hyper-ui`'s
    /// `viewport_at` takes the store as an argument in order to ask what kind of thing
    /// it just found, which is P3.
    pub accepts: bool,
}
