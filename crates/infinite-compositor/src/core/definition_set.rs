//! [`DefinitionSet`] — what [`crate::core::link`] is handed.

use std::collections::BTreeMap;

use crate::core::addr::Addr;
use crate::core::block::Block;
use crate::core::composition::Composition;

/// The definitions a link runs against: stored records, pending edits, or a mix.
///
/// This type is why D26 exists. The editor must be able to answer *"if I drop this
/// wire here, does it link?"* while the wire is still **pending** (D8) — that is,
/// while it is by definition not in the store. A compositor that read the store
/// directly could not answer that question at all. So the definition set is an
/// ordinary argument, resolved by the `Definitions` port and handed in, and a
/// speculative set is not a special case.
///
/// It is also why `link` must be cheap: it runs at interaction rate, on input that
/// changes with every keystroke, and a keystroke is not a write (D24).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DefinitionSet {
    /// Blocks by address.
    pub blocks: BTreeMap<Addr, Block>,
    /// Compositions by address, for bodies of kind `composed`.
    pub compositions: BTreeMap<Addr, Composition>,
}

impl DefinitionSet {
    /// The block at an address, if this set holds one.
    pub fn block(&self, at: &Addr) -> Option<&Block> {
        self.blocks.get(at)
    }

    /// The composition at an address, if this set holds one.
    pub fn composition(&self, at: &Addr) -> Option<&Composition> {
        self.compositions.get(at)
    }
}
