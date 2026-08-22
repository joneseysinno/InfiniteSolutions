//! [`signature_of`] — the rule that makes composition close.

use std::collections::BTreeSet;

use crate::core::addr::Addr;
use crate::core::composition::Composition;
use crate::core::definition_set::DefinitionSet;
use crate::core::outcome::Outcome;
use crate::core::port::PortName;
use crate::core::signature::Signature;

/// Derives a composition's own signature.
///
/// One rule, and it is the load-bearing obligation of the whole platform (D14.6):
///
/// > **The signature of a composition is its unbound ports.** An input port with no
/// > wire inside becomes an input of the whole; an output port with no wire inside
/// > becomes an output.
///
/// Therefore there is no "top level". The editor is a composition, the app is a
/// composition, and the difference between them is only which one you have zoomed to
/// (D20). It is also D22's answer to the tedium objection: if closure works, wire count
/// does not scale with app size, because the wires inside a block are drawn once and
/// the block is reused. A forty-wire canvas is a symptom of blocks composing badly.
///
/// Verified by the **closure test** (`tests/closure.rs`), this layer's equivalent of
/// R12's discard test: link C, wrap it as a block B, build C' holding only B wired
/// straight through, link C', and require the plan to be identical to the plan for C.
/// Closure then fails mechanically rather than as a judgment about whether nesting
/// feels right.
pub fn signature_of(composition: &Composition, _defs: &DefinitionSet) -> Outcome<Signature> {
    let mut bound: BTreeSet<(Addr, PortName)> = BTreeSet::new();
    for wire in &composition.wires {
        for end in wire.sources.iter().chain(wire.sinks.iter()) {
            bound.insert((end.block.clone(), end.port.clone()));
        }
    }
    let mut ports = Vec::new();
    for (addr, block) in &composition.blocks {
        for port in &block.signature.ports {
            if !bound.contains(&(addr.clone(), port.name.clone())) {
                ports.push(port.clone());
            }
        }
    }
    Outcome::clean(Signature { ports })
}
