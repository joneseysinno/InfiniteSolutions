//! [`link`] — the function this layer exists for.

use std::collections::BTreeSet;

use crate::core::addr::Addr;
use crate::core::block::{Block, BodyKind};
use crate::core::composition::Composition;
use crate::core::definition_set::DefinitionSet;
use crate::core::finding::{kind, Finding};
use crate::core::order::order;
use crate::core::outcome::Outcome;
use crate::core::plan::{Plan, Step};
use crate::core::port::{Direction, Port, PortRef};
use crate::core::tag::Tag;
use crate::core::wire::Wire;

/// Links a composition into a plan.
///
/// > **Given a set of definitions at a revision, produce a plan that can be executed —
/// > or a finding that says precisely why it cannot.**
///
/// Pure: no I/O, no clock, no store. The definition set is an argument (D26), which is
/// what lets the editor link a **pending** wire that is not in the store.
///
/// Returns an [`Outcome`], never a `Result`: a composition with one bad wire still
/// runs the other ninety (D21).
pub fn link(defs: &DefinitionSet, root: &Addr) -> Outcome<Plan> {
    link_path(defs, root, &mut BTreeSet::new())
}

fn link_path(defs: &DefinitionSet, root: &Addr, path: &mut BTreeSet<Addr>) -> Outcome<Plan> {
    if !path.insert(root.clone()) {
        return Outcome::with(Plan::default(), vec![delegate_cycle(root)]);
    }
    let out = link_inner(defs, root, path);
    path.remove(root);
    out
}

fn link_inner(defs: &DefinitionSet, root: &Addr, path: &mut BTreeSet<Addr>) -> Outcome<Plan> {
    let Some(composition) = defs.composition(root) else {
        if let Some(block) = defs.block(root) {
            if block.body.kind.key() == BodyKind::COMPOSED
                || block.body.kind.key() == BodyKind::DELEGATE
                || block.body.kind.key() == BodyKind::REGION
            {
                return link_path(defs, &block.body.target, path);
            }
            return Outcome::clean(Plan {
                steps: vec![Step {
                    block: root.clone(),
                    key: native_key(block),
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                    inner: None,
                }],
            });
        }
        return Outcome::with(Plan::default(), vec![unresolved(root)]);
    };

    let mut findings = Vec::new();
    let mut bad: BTreeSet<Addr> = BTreeSet::new();
    for (addr, block) in &composition.blocks {
        if !body_resolves(defs, block) {
            findings.push(unresolved(addr));
            bad.insert(addr.clone());
        }
    }

    findings.extend(check_wires(composition, &bad));
    if composition.compilable {
        findings.extend(check_pure(composition));
    }

    let ordered = order(composition);
    findings.extend(ordered.findings);

    let mut steps = Vec::new();
    for addr in &ordered.value {
        let Some(block) = composition.blocks.get(addr) else {
            continue;
        };
        let key = block.body.kind.key();
        if key == BodyKind::COMPOSED || key == BodyKind::DELEGATE {
            if block.body.target == *root {
                continue;
            }
            let inner = link_path(defs, &block.body.target, path);
            findings.extend(inner.findings);
            steps.extend(inner.value.steps);
            continue;
        }
        if key == BodyKind::REGION {
            let inner = link_path(defs, &block.body.target, path);
            findings.extend(inner.findings);
            let mut step = step_for(addr, block, composition);
            step.inner = Some(inner.value);
            steps.push(step);
            continue;
        }
        steps.push(step_for(addr, block, composition));
    }

    Outcome::with(Plan { steps }, findings)
}

fn body_resolves(defs: &DefinitionSet, block: &Block) -> bool {
    let target = &block.body.target;
    let key = block.body.kind.key();
    if key == BodyKind::NATIVE || key == BodyKind::PORTAL {
        return defs.block(target).is_some();
    }
    defs.composition(target).is_some() || defs.block(target).is_some()
}

fn check_wires(composition: &Composition, bad: &BTreeSet<Addr>) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut counts: Vec<((Addr, Box<str>), u32)> = Vec::new();

    for wire in &composition.wires {
        if let Some(f) = tag_mismatch(composition, wire, bad) {
            findings.push(f);
        }
        for end in wire.sources.iter().chain(wire.sinks.iter()) {
            if bad.contains(&end.block) {
                continue;
            }
            bump(&mut counts, end);
        }
    }

    for (addr, block) in &composition.blocks {
        if bad.contains(addr) {
            continue;
        }
        for port in &block.signature.ports {
            let n = count_of(&counts, addr, &port.name);
            if let Some(max) = port.arity {
                if n > max {
                    findings.push(Finding::new(
                        addr.clone(),
                        kind::ARITY,
                        format!("{} wires on {}", n, port.name),
                        format!("at most {max}"),
                        "remove the extra wire, or raise this port's arity",
                    ));
                }
            }
            if port.required && port.direction == Direction::In && n == 0 {
                findings.push(Finding::new(
                    addr.clone(),
                    kind::UNSATISFIED_IMPORT,
                    format!("no wire on {}", port.name),
                    "a wire that feeds this input",
                    "draw a wire to this port, or mark the port optional",
                ));
            }
        }
    }
    findings
}

fn tag_mismatch(composition: &Composition, wire: &Wire, bad: &BTreeSet<Addr>) -> Option<Finding> {
    let mut tags: Vec<(&Tag, &Addr)> = Vec::new();
    for end in wire.sources.iter().chain(wire.sinks.iter()) {
        if bad.contains(&end.block) {
            continue;
        }
        if let Some(port) = port_of(composition, end) {
            tags.push((&port.tag, &end.block));
        }
    }
    let Some((first, _)) = tags.first() else {
        return None;
    };
    for (tag, at) in &tags {
        if !first.matches(tag) {
            return Some(Finding::new(
                (*at).clone(),
                kind::TAG_MISMATCH,
                first.label().to_string(),
                tag.label().to_string(),
                "rewire this port, or change the block that feeds it",
            ));
        }
    }
    None
}

fn check_pure(composition: &Composition) -> Vec<Finding> {
    let mut findings = Vec::new();
    for wire in &composition.wires {
        for src in &wire.sources {
            if !composition.blocks.contains_key(&src.block) {
                let site = wire
                    .sinks
                    .first()
                    .map(|s| s.block.clone())
                    .unwrap_or_else(|| src.block.clone());
                findings.push(Finding::new(
                    site,
                    kind::NOT_PURE,
                    "a read that is not a declared input",
                    "only the composition's declared inputs",
                    "declare the extra input on the composition, or stop marking it compilable",
                ));
                return findings;
            }
        }
    }
    findings
}

fn step_for(addr: &Addr, block: &Block, composition: &Composition) -> Step {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for port in block.signature.inputs() {
        let mut found = false;
        for wire in &composition.wires {
            if wire
                .sinks
                .iter()
                .any(|s| s.block == *addr && *s.port == *port.name)
            {
                inputs.extend(wire.sources.iter().cloned());
                found = true;
            }
        }
        if !found {
            inputs.push(PortRef {
                block: addr.clone(),
                port: port.name.clone(),
            });
        }
    }
    for port in block.signature.outputs() {
        for wire in &composition.wires {
            if wire
                .sources
                .iter()
                .any(|s| s.block == *addr && *s.port == *port.name)
            {
                outputs.extend(wire.sinks.iter().cloned());
            }
        }
    }
    Step {
        block: addr.clone(),
        key: native_key(block),
        inputs,
        outputs,
        inner: None,
    }
}

fn native_key(block: &Block) -> Box<str> {
    if block.body.kind.key() == BodyKind::NATIVE || block.body.kind.key() == BodyKind::PORTAL {
        String::from_utf8_lossy(block.body.target.as_bytes())
            .into_owned()
            .into_boxed_str()
    } else {
        Box::from("")
    }
}

fn port_of<'a>(composition: &'a Composition, end: &PortRef) -> Option<&'a Port> {
    composition.blocks.get(&end.block)?.signature.port(&end.port)
}

fn bump(counts: &mut Vec<((Addr, Box<str>), u32)>, end: &PortRef) {
    if let Some((_, n)) = counts
        .iter_mut()
        .find(|((a, p), _)| a == &end.block && p == &end.port)
    {
        *n += 1;
        return;
    }
    counts.push(((end.block.clone(), end.port.clone()), 1));
}

fn count_of(counts: &[((Addr, Box<str>), u32)], addr: &Addr, port: &str) -> u32 {
    counts
        .iter()
        .find(|((a, p), _)| a == addr && p.as_ref() == port)
        .map(|(_, n)| *n)
        .unwrap_or(0)
}

fn delegate_cycle(site: &Addr) -> Finding {
    Finding::new(
        site.clone(),
        kind::CYCLE,
        "a composed or delegate body that names an address already on the link path",
        "an acyclic definition",
        "point this body at a definition that does not lead back here",
    )
}

fn unresolved(site: &Addr) -> Finding {
    Finding::new(
        site.clone(),
        kind::UNRESOLVED_BLOCK,
        "a body that names nothing this set holds",
        "a registered native key, or an address with a definition",
        "register this block, or point the body at a defined address",
    )
}
