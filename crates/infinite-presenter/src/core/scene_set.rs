//! [`SceneSet`] — what [`crate::core::place`] is handed (spec §8.1).

use std::collections::BTreeMap;

use crate::core::addr::Addr;
use crate::core::placeable::Placeable;
use crate::core::revision::Revision;

/// The things a view might place, resolved at a revision.
///
/// A **set**, handed in as an argument, exactly as the compositor's `DefinitionSet` is
/// (D26). The reason is the same and it is the editor: geometry that is still
/// **pending** (D8) — a shape being dragged, a wire half-drawn — is by definition not
/// in the store, so a presenter that read the store directly could not place it, and
/// the person would see their own gesture only after committing it.
///
/// A `BTreeMap` rather than a hash map, for the reason `Composition` gives: iteration
/// is address order, so anything built by walking it is deterministic. A placement
/// whose draw order depended on hash iteration would be non-reproducible, and nothing
/// would announce it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneSet {
    at: Revision,
    items: BTreeMap<Addr, Placeable>,
}

impl SceneSet {
    /// An empty set at a revision.
    pub fn new(at: Revision) -> Self {
        Self {
            at,
            items: BTreeMap::new(),
        }
    }

    /// The revision this set was resolved at.
    pub fn at(&self) -> Revision {
        self.at
    }

    /// Adds or replaces a thing.
    ///
    /// This is how a pending edit enters: the binding resolves the stored set, then
    /// overlays what the runtime's pending set holds, and hands the result here.
    pub fn insert(&mut self, item: Placeable) {
        self.items.insert(item.at.clone(), item);
    }

    /// What the set says about an address.
    pub fn get(&self, at: &Addr) -> Option<&Placeable> {
        self.items.get(at)
    }

    /// Everything in the set, in address order.
    pub fn iter(&self) -> impl Iterator<Item = &Placeable> {
        self.items.values()
    }

    /// Everything whose address lies inside the subtree `root` names, in address
    /// order.
    ///
    /// One contiguous range, because address order is spatial order (spec §3.2).
    pub fn subtree<'a>(&'a self, root: &'a Addr) -> impl Iterator<Item = &'a Placeable> + 'a {
        self.items
            .range(root.clone()..)
            .take_while(|(at, _)| root.contains(at))
            .map(|(_, item)| item)
    }

    /// How many things are in the set.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::SceneSet;
    use crate::core::{Addr, Extent, Placeable, Point, Revision};

    fn thing(bytes: Vec<u8>) -> Placeable {
        Placeable {
            at: Addr::new(bytes),
            across: Extent::fixed(10.0),
            down: Extent::fixed(10.0),
            position: Point::ORIGIN,
            style: "plain".into(),
            detail_override: None,
            primitive: crate::core::AREA.into(),
            link: None,
            hosts_space: false,
            accepts: true,
            text: "".into(),
        }
    }

    #[test]
    fn a_subtree_is_one_contiguous_range() {
        let mut set = SceneSet::new(Revision::new(7));
        set.insert(thing(vec![0x01]));
        set.insert(thing(vec![0x01, 0x00]));
        set.insert(thing(vec![0x01, 0xFF]));
        set.insert(thing(vec![0x02]));

        let inside: Vec<_> = set
            .subtree(&Addr::new(vec![0x01]))
            .map(|item| item.at.clone())
            .collect();
        assert_eq!(inside.len(), 3);
        assert!(!inside.contains(&Addr::new(vec![0x02])));
    }
}
