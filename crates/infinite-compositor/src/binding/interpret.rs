//! [`interpret`] — walk the plan.

use crate::binding::ports::{Blocks, Provenance, Values};
use crate::core::{kind, Finding, Outcome, Plan, PortRef};

/// Executes a plan, interpreted.
///
/// A graph runs the moment it is drawn; there is no build step between composing and
/// seeing it work (D19). This path is **always available**, and under D19's equivalence
/// law it is the **specification** every compiled form is measured against — which is
/// why `tests/equivalence.rs` compares against this function rather than against a
/// reference implementation written for the purpose.
///
/// The whole of it: for each step, read inputs through [`Values`], invoke through
/// [`Blocks`], write outputs through [`Values`], record through [`Provenance`].
///
/// Note what is absent, and keep it absent. No math (L3) — every number is inside a
/// block. No scheduling (R10) — every *when* is the runtime's; this function is called,
/// it does not decide that it should be. No transport, no allocation policy, no clock.
///
/// *Lands in stage **S6**.*
pub fn interpret(
    plan: &Plan,
    blocks: &dyn Blocks,
    values: &mut dyn Values,
    provenance: &mut dyn Provenance,
) -> Outcome<()> {
    let mut findings = Vec::new();
    for step in &plan.steps {
        if let Some(inner) = &step.inner {
            let nested = interpret(inner, blocks, values, provenance);
            findings.extend(nested.findings);
            continue;
        }
        if step.key.is_empty() {
            continue;
        }
        let Some(sig) = blocks.signature(&step.key) else {
            findings.push(unresolved(&step.block, &step.key));
            continue;
        };
        let Some(primitive) = blocks.primitive(&step.key) else {
            findings.push(unresolved(&step.block, &step.key));
            continue;
        };

        let mut inputs = Vec::new();
        let mut input_addrs = Vec::new();
        let mut missing = false;
        for src in &step.inputs {
            let at = src.slot();
            input_addrs.push(at.clone());
            match values.read(&at) {
                Some(value) => inputs.push(value),
                None => {
                    missing = true;
                    break;
                }
            }
        }
        if missing {
            continue;
        }

        let produced = primitive.invoke(&inputs);
        let mut output_addrs = Vec::new();
        for (port, value) in sig.outputs().zip(produced.into_iter()) {
            let at = PortRef {
                block: step.block.clone(),
                port: port.name.clone(),
            }
            .slot();
            values.write(&at, value);
            output_addrs.push(at);
        }
        provenance.record(&output_addrs, &input_addrs, &step.block);
    }
    Outcome::with((), findings)
}

fn unresolved(site: &crate::core::Addr, key: &str) -> Finding {
    Finding::new(
        site.clone(),
        kind::UNRESOLVED_BLOCK,
        key,
        "a registered primitive",
        "register the block, or change the body to name one that exists",
    )
}
