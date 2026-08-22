//! [`probe`] — the second half, and the half everything got wrong.

use crate::core::addr::Addr;
use crate::core::placement::Placement;
use crate::core::point::Point;

/// What is under a point.
#[derive(Clone, Debug, PartialEq)]
pub struct Probe {
    /// The store's address.
    pub at: Addr,
    /// The point, in that space's own coordinates.
    pub local: Point,
}

/// Answers a surface point with an address.
///
/// Takes a placement and nothing else. **No port, no store, no scene.** That is the
/// self-sufficiency test (spec §8.4): if this function ever needs a port, P3 has
/// recurred.
pub fn probe(placement: &Placement, at: Point) -> Option<Probe> {
    let mut current: Option<&crate::core::placed::Placed> = None;
    loop {
        let parent = current.map(|p| &p.at);
        let next = placement
            .placed
            .iter()
            .filter(|p| p.accepts && p.covers(at) && is_direct(parent, &p.at, placement))
            .last();
        match next {
            Some(p) => current = Some(p),
            None => break,
        }
    }
    let hit = current?;
    let local = placement
        .spaces
        .get(&hit.at)
        .map(|t| t.invert().apply(at))
        .unwrap_or(at);
    Some(Probe {
        at: hit.at.clone(),
        local,
    })
}

fn is_direct(parent: Option<&Addr>, child: &Addr, placement: &Placement) -> bool {
    match parent {
        None => !placement
            .placed
            .iter()
            .any(|p| p.at.contains(child) && p.at != *child),
        Some(parent) => {
            parent.contains(child)
                && *child != *parent
                && !placement.placed.iter().any(|p| {
                    p.at != *parent
                        && p.at != *child
                        && parent.contains(&p.at)
                        && p.at.contains(child)
                })
        }
    }
}
