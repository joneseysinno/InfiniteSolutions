//! [`Registry`] — the string-keyed registry (R4, R16).

use std::collections::BTreeMap;

/// A string-keyed registry.
///
/// R4: layers reach each other through string-keyed registries, never compile-time
/// names. R16: kind is a prop and dispatch is a registry — named as the same mistake
/// five times in this corpus's own plans (`kind_id`, `WorkspaceInstance`, `IoKind`,
/// `AppSignal`, and the `ParticleKind` that was proposed and rejected).
///
/// Entries live in a `BTreeMap` rather than a hash map for the same reason
/// `Composition`'s blocks do: iteration is deterministic, so anything built by walking
/// a registry is reproducible, and D19's equivalence law stays exact.
///
/// A registry is a **symbol table, not state** (L4): it is populated at startup, and
/// registration is not reachable from [`crate::binding::interpret`].
pub struct Registry<T: ?Sized> {
    entries: BTreeMap<String, Box<T>>,
}

impl<T: ?Sized> Default for Registry<T> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl<T: ?Sized> Registry<T> {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an entry under a key. Startup only.
    pub fn register(&mut self, key: impl Into<String>, entry: Box<T>) {
        self.entries.insert(key.into(), entry);
    }

    /// Looks an entry up.
    pub fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key).map(|entry| &**entry)
    }

    /// Every registered key, in order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(|key| key.as_str())
    }
}
