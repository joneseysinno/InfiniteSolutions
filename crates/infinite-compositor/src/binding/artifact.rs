//! The linked plan's registration parts (D25, spec §4).
//!
//! The compositor owns the function ([`crate::core::link`]); the runtime owns the
//! schedule. Neither crate can see the other's registry (D23, D26), so the facade
//! hands these three parts over.

use crate::core::{Plan, PortRef};

/// The string key the plan is registered under.
pub const KEY: &str = "plan";

/// Encodes a plan as bytes so the generic discard harness can compare them.
///
/// Deterministic: steps in plan order, addresses and port names as stored.
pub fn encode(plan: &Plan) -> Vec<u8> {
    let mut out = Vec::from(&b"PL1"[..]);
    put_plan(&mut out, plan);
    out
}

fn put_plan(out: &mut Vec<u8>, plan: &Plan) {
    out.extend_from_slice(&(plan.steps.len() as u32).to_le_bytes());
    for step in &plan.steps {
        put_bytes(out, step.block.as_bytes());
        put_bytes(out, step.key.as_bytes());
        put_refs(out, &step.inputs);
        put_refs(out, &step.outputs);
        match &step.inner {
            Some(inner) => {
                out.push(1);
                put_plan(out, inner);
            }
            None => out.push(0),
        }
    }
}

fn put_refs(out: &mut Vec<u8>, refs: &[PortRef]) {
    out.extend_from_slice(&(refs.len() as u16).to_le_bytes());
    for r in refs {
        put_bytes(out, r.block.as_bytes());
        put_bytes(out, r.port.as_bytes());
    }
}

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    out.extend_from_slice(bytes);
}
