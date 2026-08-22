//! [`Composition`] — a graph of blocks and wires, which is itself a block.

use std::collections::BTreeMap;

use crate::core::addr::Addr;
use crate::core::block::Block;
use crate::core::wire::Wire;

/// A graph (D20) seen as a block (D14.6).
///
/// Blocks live in a `BTreeMap` rather than a hash map, and that is a correctness
/// choice rather than a taste one. `Addr` is totally ordered, so iteration is address
/// order, so a plan built by walking a composition is **deterministic by
/// construction**. D19's equivalence law is exact rather than statistical *only*
/// because execution is deterministic; a plan whose order depended on hash iteration
/// would make the compile story unverifiable, and nothing would announce it.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Composition {
    /// The sibling spaces at this level, by address.
    pub blocks: BTreeMap<Addr, Block>,
    /// The hyperedges among them, in authored order.
    pub wires: Vec<Wire>,
    /// Whether this composition claims to be a pure function of its declared
    /// inputs (D19). A true value that then reads something else is `not-pure`.
    pub compilable: bool,
}
