//! [`Placeable`] — what the `Scene` port says about one thing.

use crate::core::addr::Addr;
use crate::core::extent::Extent;
use crate::core::point::Point;

/// The primitive key an unauthored record draws as.
///
/// A string and not a variant: the set of shapes is open — a rectangle, a link, a
/// text run, whatever a block author needs next — and R16 makes a closed enum a
/// defect wherever the set is open. F-1 counts five occurrences of that mistake in
/// the corpus already.
pub const AREA: &str = "rect";

/// The primitive key for a text run (E13.0, D46).
pub const TEXT: &str = "text";

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
    /// An opaque key naming *what shape* draws it (D46). [`AREA`] by default.
    ///
    /// Opaque in the same sense as `style`, and for the same reason: this layer
    /// carries it to the `Surface` and never interprets it. It is here rather than on
    /// `Placed` because it is authored, and it is what [`crate::core::Placement`]
    /// groups by — the *"grouped how"* D15 and D29 give this layer and finding 16
    /// recorded it could not say.
    pub primitive: Box<str>,
    /// The two things this connects, when it connects things rather than occupying
    /// an area (D46). `None` for every area primitive.
    ///
    /// A hyperedge has no authored position: its geometry is wherever its ends
    /// landed. Two addresses and not two points is L5 — the presenter refers to
    /// identity the store already issued and mints none of its own.
    pub link: Option<(Addr, Addr)>,
    /// Whether this space has an interior to descend into.
    pub hosts_space: bool,
    /// Whether a probe may land here.
    ///
    /// Resolved once, at place time, from what the store says — so that
    /// [`crate::core::probe`] never needs a port to answer (spec §8.4). `hyper-ui`'s
    /// `viewport_at` takes the store as an argument in order to ask what kind of thing
    /// it just found, which is P3.
    pub accepts: bool,
    /// The run to draw when `primitive` is [`TEXT`]. Empty for every other shape.
    pub text: Box<str>,
}
