//! The `Definitions` port.

use crate::core::{Addr, DefinitionSet};

/// Resolves definitions into a set that [`crate::core::link`] can run against.
///
/// The implementation may draw on stored records, on the runtime's pending set (D8),
/// or on both — and the compositor cannot tell which, deliberately.
pub trait Definitions {
    /// Everything reachable from `root`, at whatever revision the facade resolved.
    fn resolve(&self, root: &Addr) -> DefinitionSet;
}
