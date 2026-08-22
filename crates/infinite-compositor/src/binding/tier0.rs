//! Tier 0 — the resolved plan, with lookups hoisted (D28, spec §10.4).
//!
//! No compiler, no toolchain, no new dependency. Sources are slots, invocations
//! are the primitives captured at compile, order is the plan's.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::binding::artifact::encode as encode_plan;
use crate::binding::backend::{Artifact, Backend, Cost};
use crate::binding::interpret::interpret;
use crate::binding::ports::{Blocks, Primitive, Provenance, Values};
use crate::core::{Addr, Plan, PortRef, Value};

/// The string key this backend registers under.
pub const KEY: &str = "tier0";

/// Encodes a tier-0 artifact. The form *is* the plan (lookups already decided).
pub fn encode(plan: &Plan) -> Vec<u8> {
    let mut out = Vec::from(&b"T0"[..]);
    out.extend_from_slice(&encode_plan(plan));
    out
}

/// The resolved-plan backend.
pub struct Tier0 {
    natives: Vec<(Box<str>, Arc<dyn Primitive>, Vec<Box<str>>)>,
}

impl Tier0 {
    /// Builds a backend from primitives and their output port names.
    pub fn new(natives: Vec<(Box<str>, Arc<dyn Primitive>, Vec<Box<str>>)>) -> Self {
        Self { natives }
    }

    fn primitive(&self, key: &str) -> Option<(Arc<dyn Primitive>, Vec<Box<str>>)> {
        self.natives.iter().find(|(k, _, _)| k.as_ref() == key).map(|(_, p, o)| (Arc::clone(p), o.clone()))
    }
}

impl Backend for Tier0 {
    fn accepts(&self, plan: &Plan) -> bool {
        accepts_plan(self, plan)
    }

    fn compile(&self, plan: &Plan) -> Option<Box<dyn Artifact>> {
        if !self.accepts(plan) {
            return None;
        }
        let steps = resolve(self, plan)?;
        let feed = feed_slots(plan);
        let sink: Vec<Addr> = steps
            .iter()
            .flat_map(|s| s.outputs.iter().map(PortRef::slot))
            .collect();
        Some(Box::new(Tier0Artifact {
            steps,
            feed,
            sink,
            bytes: encode(plan),
        }))
    }

    fn cost(&self) -> Cost {
        Cost {
            compile: 0,
            crossing: 0,
        }
    }
}

fn accepts_plan(backend: &Tier0, plan: &Plan) -> bool {
    plan.steps.iter().all(|step| {
        if let Some(inner) = &step.inner {
            return accepts_plan(backend, inner);
        }
        step.key.is_empty() || backend.primitive(&step.key).is_some()
    })
}

fn resolve(backend: &Tier0, plan: &Plan) -> Option<Vec<Resolved>> {
    let mut steps = Vec::new();
    for step in &plan.steps {
        if let Some(inner) = &step.inner {
            steps.extend(resolve(backend, inner)?);
            continue;
        }
        if step.key.is_empty() {
            continue;
        }
        let (primitive, out_names) = backend.primitive(&step.key)?;
        steps.push(Resolved {
            primitive,
            inputs: step.inputs.iter().map(PortRef::slot).collect(),
            outputs: out_names
                .into_iter()
                .map(|name| PortRef {
                    block: step.block.clone(),
                    port: name,
                })
                .collect(),
        });
    }
    Some(steps)
}

struct Resolved {
    primitive: Arc<dyn Primitive>,
    inputs: Vec<Addr>,
    outputs: Vec<PortRef>,
}

struct Tier0Artifact {
    steps: Vec<Resolved>,
    feed: Vec<Addr>,
    sink: Vec<Addr>,
    bytes: Vec<u8>,
}

impl Artifact for Tier0Artifact {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let mut mem = std::collections::BTreeMap::new();
        for (slot, value) in self.feed.iter().zip(inputs.iter()) {
            mem.insert(slot.clone(), value.clone());
        }
        for step in &self.steps {
            let mut args = Vec::new();
            let mut missing = false;
            for slot in &step.inputs {
                match mem.get(slot) {
                    Some(value) => args.push(value.clone()),
                    None => {
                        missing = true;
                        break;
                    }
                }
            }
            if missing {
                continue;
            }
            let produced = step.primitive.invoke(&args);
            for (port, value) in step.outputs.iter().zip(produced.into_iter()) {
                mem.insert(port.slot(), value);
            }
        }
        self.sink
            .iter()
            .filter_map(|slot| mem.get(slot).cloned())
            .collect()
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub(crate) fn feed_slots(plan: &Plan) -> Vec<Addr> {
    let mut produced_blocks = BTreeSet::new();
    let mut feed = Vec::new();
    walk_feed(plan, &mut produced_blocks, &mut feed);
    feed
}

fn walk_feed(plan: &Plan, produced_blocks: &mut BTreeSet<Addr>, feed: &mut Vec<Addr>) {
    for step in &plan.steps {
        if let Some(inner) = &step.inner {
            walk_feed(inner, produced_blocks, feed);
            continue;
        }
        for src in &step.inputs {
            let slot = src.slot();
            if !produced_blocks.contains(&src.block) && !feed.contains(&slot) {
                feed.push(slot);
            }
        }
        produced_blocks.insert(step.block.clone());
    }
}

fn is_port_of(slot: &Addr, block: &Addr) -> bool {
    let b = block.as_bytes();
    let s = slot.as_bytes();
    s.len() > b.len() + 1 && s[..b.len()] == *b && s[b.len()] == 0xFE
}

struct MapValues {
    by_addr: std::collections::BTreeMap<Addr, Value>,
}
impl Values for MapValues {
    fn read(&self, at: &Addr) -> Option<Value> {
        self.by_addr.get(at).cloned()
    }
    fn write(&mut self, at: &Addr, value: Value) {
        self.by_addr.insert(at.clone(), value);
    }
}
struct MapProv {
    inputs: std::collections::BTreeMap<Addr, Vec<Addr>>,
}
impl Provenance for MapProv {
    fn record(&mut self, outputs: &[Addr], inputs: &[Addr], _block: &Addr) {
        for out in outputs {
            self.inputs.insert(out.clone(), inputs.to_vec());
        }
    }
    fn inputs_of(&self, output: &Addr) -> Vec<Addr> {
        self.inputs.get(output).cloned().unwrap_or_default()
    }
}

/// Interpreted vs compiled: outputs bit-for-bit, provenance edge-for-edge.
///
/// One function. A backend registers by passing it. There is no per-backend test.
pub fn check(
    backend: &dyn Backend,
    plan: &Plan,
    blocks: &dyn Blocks,
    seed: &[(Addr, Value)],
) -> bool {
    let mut values = MapValues {
        by_addr: seed.iter().cloned().collect(),
    };
    let mut provenance = MapProv {
        inputs: Default::default(),
    };
    let out = interpret(plan, blocks, &mut values, &mut provenance);
    if out.has_findings() {
        return false;
    }

    let mut interp_out = Vec::new();
    collect_outputs(plan, &values, &mut interp_out);

    if !edges_match(plan, &provenance) {
        return false;
    }

    if !backend.accepts(plan) {
        return false;
    }
    let Some(artifact) = backend.compile(plan) else {
        return false;
    };
    let feed = feed_slots(plan);
    let compiled_in: Vec<Value> = feed
        .iter()
        .filter_map(|slot| seed.iter().find(|(a, _)| a == slot).map(|(_, v)| v.clone()))
        .collect();
    if compiled_in.len() != feed.len() {
        return false;
    }
    let compiled_out = artifact.invoke(&compiled_in);
    interp_out == compiled_out
}

fn collect_outputs(plan: &Plan, values: &MapValues, into: &mut Vec<Value>) {
    for step in &plan.steps {
        if let Some(inner) = &step.inner {
            collect_outputs(inner, values, into);
            continue;
        }
        for (at, value) in &values.by_addr {
            if is_port_of(at, &step.block)
                && !step.inputs.iter().any(|i| i.slot() == *at)
            {
                into.push(value.clone());
            }
        }
    }
}

fn edges_match(plan: &Plan, provenance: &MapProv) -> bool {
    for step in &plan.steps {
        if let Some(inner) = &step.inner {
            if !edges_match(inner, provenance) {
                return false;
            }
            continue;
        }
        let declared: Vec<Addr> = step.inputs.iter().map(PortRef::slot).collect();
        for (out, ins) in &provenance.inputs {
            if is_port_of(out, &step.block) && ins != &declared {
                return false;
            }
        }
    }
    true
}
