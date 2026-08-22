//! [`Block`] — a space with declared ports, and its body.

use crate::core::addr::Addr;
use crate::core::signature::Signature;

/// How a body is resolved. An **open** set, deliberately.
///
/// R16 and F-1 (five prior occurrences) would already argue against an enum. The
/// evidence here is stronger than usual: D18 added **portals** and D21 added
/// **iterative regions** to this system on the same day the model was drawn — two new
/// body kinds in two days. A closed set would have been wrong before it compiled.
///
/// Known kinds are associated constants rather than variants, so a facade may add one
/// without touching the platform.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BodyKind(Box<str>);

impl BodyKind {
    /// A native primitive, written in Rust by a block author (D16, the first of two
    /// tiers of user).
    pub const NATIVE: &'static str = "native";
    /// A composition — a graph of blocks and wires (D14.6).
    pub const COMPOSED: &'static str = "composed";
    /// A use of another block. Use is delegation (D27); there is no `Instance`.
    pub const DELEGATE: &'static str = "delegate";
    /// A platform boundary (D18). Not specified by draft 1 of the spec.
    pub const PORTAL: &'static str = "portal";
    /// An area marked iterative (D21). Structure only in draft 1 — see `region.rs`.
    pub const REGION: &'static str = "region";

    /// Names a body kind.
    pub fn new(key: impl Into<Box<str>>) -> Self {
        Self(key.into())
    }

    /// The registry key.
    pub fn key(&self) -> &str {
        &self.0
    }
}

/// What a block *is*, once its kind has been resolved.
///
/// One kind and one address, never a variant per kind. Every space has a permanent
/// address (D20), so the target of a body is always an address; what differs between
/// kinds is only what the resolver does with it — the `Blocks` port for a native
/// primitive, the `Definitions` port for anything authored.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Body {
    /// Which resolver handles this body.
    pub kind: BodyKind,
    /// What that resolver is handed.
    pub target: Addr,
}

/// A space with declared ports.
///
/// `block` is not a sixth noun beside D20's five: a block **is** a space, seen as a
/// unit of composition. The compositor adds exactly three words to the vocabulary —
/// **block**, **port**, **plan** — and `Instance` is not among them, because use is
/// delegation (D27, spec §5.2). That is `bion`'s `.node().delegate()` chain grammar,
/// which D6 salvaged from `hypernode` as runtime material and which turns out to be
/// compositor material.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Block {
    /// Declared ports. Derived rather than authored for a composition (D14.6).
    pub signature: Signature,
    /// What it is.
    pub body: Body,
}
