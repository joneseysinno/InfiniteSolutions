//! [`Placed`] — one thing, on the screen.

use crate::core::addr::Addr;
use crate::core::point::Point;
use crate::core::rect::Rect;

/// Where one thing is, and how much of it is showing.
///
/// **An address and a rectangle.** Not an id and a rectangle: the address is the
/// store's, which is L5's whole distinction. The presenter mints no identity; it
/// refers to identity that already exists.
///
/// What is deliberately **not** here: a colour, a selection flag, a hover flag, a
/// style resolved to values. The counter-example is `hyper-ui`'s `SceneNode`, an
/// eleven-line struct that correctly holds no identity and therefore had nowhere to
/// put selection except `selected: bool` inside the geometry — so selecting a thing
/// means re-deriving and re-uploading its geometry, and a click has no path back to a
/// node at all. Holding an address is what avoids both.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    /// The store's address.
    pub at: Addr,
    /// Where it is, in surface coordinates.
    ///
    /// For a link primitive this is the segment's bounding box, grown by the stroke's
    /// half-width — so culling, clipping and [`Self::covers`] keep working on one
    /// shape for every primitive, and only the shader reads [`Self::span`].
    pub rect: Rect,
    /// The two surface points a link runs between (D46). `None` for an area.
    ///
    /// A bounding box cannot say which diagonal a line takes, so the two points are
    /// carried rather than recovered. One `Option` and not a second parallel list of
    /// links: a list per primitive is F-1 wearing a different hat — the set of
    /// primitives is open, and a `Placement` with a field per kind closes it.
    pub span: Option<(Point, Point)>,
    /// How many address bits are significant here (spec §7).
    pub level: u32,
    /// The clip imposed by an enclosing space, if any.
    pub clip: Option<Rect>,
    /// Whether a probe may land here (spec §8.2).
    pub accepts: bool,
}

impl Placed {
    /// The part of this thing that is actually showing.
    pub fn showing(&self) -> Rect {
        match &self.clip {
            Some(clip) => self.rect.intersect(clip),
            None => self.rect,
        }
    }

    /// Whether a surface point lands on the showing part.
    pub fn covers(&self, at: crate::core::point::Point) -> bool {
        self.showing().contains(at)
    }
}
