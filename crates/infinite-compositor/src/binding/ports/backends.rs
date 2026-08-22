//! The `Backends` port.

use crate::binding::backend::Backend;

/// Resolves a compiled-form key to a registered backend (D28).
///
/// String-keyed for the same reason as `Blocks`, and for a sharper one: the set of
/// compiled forms is **open**. Beyond O9's three candidates, this system's own roadmap
/// already holds two more that neither `bion` nor `biomimicry` considered — a GPU
/// kernel (the presenter is wgpu per D15, so a fused numeric composition compiled to
/// WGSL is a short walk) and a pushdown into the store (a composition of pure reads
/// over an address range is a range scan, and the index already exists).
pub trait Backends {
    /// A registered backend, by key.
    fn backend(&self, key: &str) -> Option<&dyn Backend>;
}
