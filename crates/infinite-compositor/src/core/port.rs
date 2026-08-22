//! [`Port`] — name, direction, tag. Item 1 of what the substrate owes an app (D14).

use crate::core::addr::Addr;
use crate::core::tag::Tag;

/// A port's name, unique within its block's signature.
pub type PortName = Box<str>;

/// Which way a value travels through a port.
///
/// The set is closed (D35). A value crosses a boundary one way; a third variant
/// would be a different concept, not a third direction. `scripts/check-rules.sh`
/// still pins the core's enum count at one so a second enum cannot arrive quietly.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    /// The block reads a value here.
    In,
    /// The block writes a value here.
    Out,
}

/// A named, directed, tagged attachment point on a block.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Port {
    /// Unique within the block's signature.
    pub name: PortName,
    /// In or out.
    pub direction: Direction,
    /// Matched, never interpreted (D13).
    pub tag: Tag,
    /// How many wires may bind here. `None` is unbounded.
    ///
    /// Hyperedges are n-ary (D20), so arity is per-port policy rather than a global
    /// rule. Exceeding it is the `arity` finding.
    pub arity: Option<u32>,
    /// Whether a link with no wire bound here is an `unsatisfied-import` finding.
    pub required: bool,
}

impl Port {
    /// A required port with unbounded arity.
    pub fn new(name: impl Into<PortName>, direction: Direction, tag: Tag) -> Self {
        Self {
            name: name.into(),
            direction,
            tag,
            arity: None,
            required: true,
        }
    }
}

/// One end of a wire: a port on a block, named by the block's address.
///
/// Not an index into a vector and not a key into a side map. F-2 — a map keyed by id
/// standing in for an edge — has roughly thirty instances in one codebase in this
/// corpus, and a type like this one is where that starts.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PortRef {
    /// The block's address within its enclosing composition.
    pub block: Addr,
    /// The port's name on that block.
    pub port: PortName,
}

impl PortRef {
    /// The address [`crate::binding::interpret`] reads and writes for this port.
    ///
    /// A value travels by address (D13). The slot is the block's address, a
    /// separator, and the port name — compared, never interpreted.
    pub fn slot(&self) -> Addr {
        let mut bytes = Vec::with_capacity(self.block.as_bytes().len() + 1 + self.port.len());
        bytes.extend_from_slice(self.block.as_bytes());
        bytes.push(0xFE);
        bytes.extend_from_slice(self.port.as_bytes());
        Addr::new(bytes)
    }
}
