//! S3 — the five ports, against fakes. No other layer is named.

#![cfg(feature = "binding")]

#[path = "fakes.rs"]
mod fakes;

use fakes::{FakeBackends, FakeBlocks, FakeDefinitions, FakeProvenance, FakeValues};
use infinite_compositor::binding::ports::{
    Backends, Blocks, Definitions, Provenance, Values,
};
use infinite_compositor::core::{link, Addr, DefinitionSet};

#[test]
fn the_five_ports_answer_against_fakes() {
    let defs = FakeDefinitions {
        set: DefinitionSet::default(),
    };
    let set = defs.resolve(&Addr::new(vec![0]));
    let out = link(&set, &Addr::new(vec![0]));
    assert!(out.has_findings(), "an empty root is unresolved");

    let blocks = FakeBlocks::new();
    assert!(blocks.signature("none").is_none());
    assert!(blocks.primitive("none").is_none());

    let values = FakeValues {
        by_addr: Default::default(),
    };
    assert!(values.read(&Addr::new(vec![1])).is_none());

    let prov = FakeProvenance {
        inputs: Default::default(),
    };
    assert!(prov.inputs_of(&Addr::new(vec![1])).is_empty());

    let backends = FakeBackends;
    assert!(backends.backend("none").is_none());
}
