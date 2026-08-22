//! [`order`] — deterministic step order, and cycle detection.

use std::collections::{BTreeMap, BTreeSet};

use crate::core::addr::Addr;
use crate::core::composition::Composition;
use crate::core::finding::{kind, Finding};
use crate::core::outcome::Outcome;

/// Orders the blocks of a composition, reporting any wire that closes a loop.
///
/// Cycle detection happens **here**, at link time, and not at run time. That is the
/// spec's C5: *"a spinning app with no explanation is the worst failure available to
/// someone learning"* — D21's stated reason for rejecting runtime fixed-point
/// iteration, and the failure mode `bion` and `biomimicry` both drift toward.
///
/// A cycle produces a `cycle` finding **and** an order covering everything reachable
/// without it. Judged, not refused.
///
/// The order is deterministic because [`Composition::blocks`] is a `BTreeMap` and
/// `Addr` is totally ordered: ties in the topological order break by address, never by
/// iteration accident. D19 needs that to be true, or its equivalence law is
/// statistical rather than exact.
pub fn order(composition: &Composition) -> Outcome<Vec<Addr>> {
    let mut adj: BTreeMap<Addr, BTreeSet<Addr>> = BTreeMap::new();
    let mut indeg: BTreeMap<Addr, u32> = BTreeMap::new();
    for addr in composition.blocks.keys() {
        adj.insert(addr.clone(), BTreeSet::new());
        indeg.insert(addr.clone(), 0);
    }

    let mut looped: BTreeSet<Addr> = BTreeSet::new();
    for wire in &composition.wires {
        for src in &wire.sources {
            for sink in &wire.sinks {
                if !composition.blocks.contains_key(&src.block)
                    || !composition.blocks.contains_key(&sink.block)
                {
                    continue;
                }
                if src.block == sink.block {
                    looped.insert(src.block.clone());
                    continue;
                }
                if adj
                    .get_mut(&src.block)
                    .expect("adj")
                    .insert(sink.block.clone())
                {
                    *indeg.get_mut(&sink.block).expect("indeg") += 1;
                }
            }
        }
    }

    let mut ready: BTreeSet<Addr> = indeg
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(a, _)| a.clone())
        .collect();
    let mut out = Vec::new();
    let mut remaining = indeg;
    while let Some(a) = ready.iter().next().cloned() {
        ready.remove(&a);
        out.push(a.clone());
        remaining.remove(&a);
        if let Some(ns) = adj.get(&a) {
            for n in ns {
                if let Some(d) = remaining.get_mut(n) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        ready.insert(n.clone());
                    }
                }
            }
        }
    }

    let mut findings = Vec::new();
    if !remaining.is_empty() || !looped.is_empty() {
        let site = looped
            .iter()
            .chain(remaining.keys())
            .min()
            .expect("cycle site")
            .clone();
        findings.push(Finding::new(
            site,
            kind::CYCLE,
            "a wire that closes a loop",
            "an acyclic graph, or a region marked iterative",
            "mark the region iterative, or remove the edge that closes the loop",
        ));
        let mut rest: Vec<Addr> = remaining.into_keys().collect();
        rest.sort();
        out.extend(rest);
    }
    Outcome::with(out, findings)
}
