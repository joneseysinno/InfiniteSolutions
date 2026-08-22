//! [`Wire`] — a hyperedge seen at a port.

use crate::core::port::PortRef;

/// A directed hyperedge between ports.
///
/// N-ary on both ends, because a hyperedge connects any number of spaces (D20), and
/// all four repositories in this corpus rejected the 1:1 edge for the same stated
/// reason.
///
/// Every wire is **drawn** (D22). Nothing in this layer may construct a wire that was
/// not in the definition set: tags validate, they never discover. That is a review
/// check rather than a grep, and it is the one most likely to be violated by a
/// well-meaning convenience — `bion`'s tag-matched automatic binding is precisely the
/// shape D22 rejected, and it arrives as a reasonable-sounding suggestion (R29).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Wire {
    /// Where values come from.
    pub sources: Vec<PortRef>,
    /// Where they go.
    pub sinks: Vec<PortRef>,
}
