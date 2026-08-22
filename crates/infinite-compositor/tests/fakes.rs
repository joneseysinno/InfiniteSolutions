//! The only implementations of the ports this layer ever names (D26).

use std::collections::BTreeMap;
use std::sync::Arc;

use infinite_compositor::binding::Backend;
use infinite_compositor::binding::ports::{
    Backends, Blocks, Definitions, Primitive, Provenance, Values,
};
use infinite_compositor::core::{Addr, DefinitionSet, Signature, Value};

/// In-memory definitions. `resolve` returns the set it was built with.
pub struct FakeDefinitions {
    /// The set handed to [`infinite_compositor::core::link`].
    pub set: DefinitionSet,
}

impl Definitions for FakeDefinitions {
    fn resolve(&self, _root: &Addr) -> DefinitionSet {
        self.set.clone()
    }
}

/// Native blocks keyed by the registry string.
pub struct FakeBlocks {
    entries: Vec<(Box<str>, Signature, Arc<dyn Primitive>)>,
}

impl FakeBlocks {
    /// An empty registry.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// The tier-0 backend over the primitives already registered here.
    pub fn tier0(&self) -> infinite_compositor::binding::Tier0 {
        infinite_compositor::binding::Tier0::new(
            self.entries
                .iter()
                .map(|(k, sig, p)| {
                    (
                        k.clone(),
                        Arc::clone(p),
                        sig.outputs().map(|port| port.name.clone()).collect(),
                    )
                })
                .collect(),
        )
    }

    /// Registers a primitive under `key`. Startup only.
    pub fn register(&mut self, key: &str, signature: Signature, primitive: Box<dyn Primitive>) {
        self.entries
            .push((key.into(), signature, Arc::from(primitive)));
    }
}

impl Blocks for FakeBlocks {
    fn signature(&self, key: &str) -> Option<Signature> {
        self.entries
            .iter()
            .find(|(k, _, _)| &**k == key)
            .map(|(_, s, _)| s.clone())
    }

    fn primitive(&self, key: &str) -> Option<&dyn Primitive> {
        self.entries
            .iter()
            .find(|(k, _, _)| &**k == key)
            .map(|(_, _, p)| p.as_ref())
    }
}

/// Values at addresses.
pub struct FakeValues {
    /// Stored values, keyed by address.
    pub by_addr: BTreeMap<Addr, Value>,
}

impl Values for FakeValues {
    fn read(&self, at: &Addr) -> Option<Value> {
        self.by_addr.get(at).cloned()
    }

    fn write(&mut self, at: &Addr, value: Value) {
        self.by_addr.insert(at.clone(), value);
    }
}

/// Provenance records, keyed by the output address.
pub struct FakeProvenance {
    /// output → inputs.
    pub inputs: BTreeMap<Addr, Vec<Addr>>,
}

impl Provenance for FakeProvenance {
    fn record(&mut self, outputs: &[Addr], inputs: &[Addr], _block: &Addr) {
        for out in outputs {
            self.inputs.insert(out.clone(), inputs.to_vec());
        }
    }

    fn inputs_of(&self, output: &Addr) -> Vec<Addr> {
        self.inputs.get(output).cloned().unwrap_or_default()
    }
}

/// No backends registered.
pub struct FakeBackends;

impl Backends for FakeBackends {
    fn backend(&self, _key: &str) -> Option<&dyn Backend> {
        None
    }
}
