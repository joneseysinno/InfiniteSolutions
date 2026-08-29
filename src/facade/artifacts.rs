//! Registers `Placement` with the runtime's registry (D25, D30).
//!
//! Neither layer can do this: D23 forbids the runtime naming another layer, D29
//! forbids the presenter naming the runtime. This file hands the three parts over.

use infinite_compositor::binding::{encode_plan, encode_tier0, KEY as PLAN_KEY, TIER0_KEY};
use infinite_compositor::core::{link, Block, Body, BodyKind, DefinitionSet, Signature};
use infinite_presenter::binding::{rebuild, ranges, KEY as PLACEMENT_KEY};
use infinite_presenter::core::{SceneSet, View};
use infinite_runtime::binding::ArtifactRegistry;
use infinite_runtime::core::Addr as RuntimeAddr;

use super::addr::{compositor_addr, presenter_addr, runtime_addr};
use super::ports::inject_natives;
use super::record::decode_composition;

/// Encodes a placement as bytes so the generic discard harness can compare them.
pub fn encode_placement(placement: &infinite_presenter::core::Placement) -> Vec<u8> {
    let mut out = Vec::new();
    for item in &placement.placed {
        out.extend_from_slice(&(item.at.as_bytes().len() as u32).to_le_bytes());
        out.extend_from_slice(item.at.as_bytes());
        for n in [item.rect.min.x, item.rect.min.y, item.rect.max.x, item.rect.max.y] {
            out.extend_from_slice(&n.to_le_bytes());
        }
        out.extend_from_slice(&item.level.to_le_bytes());
        out.push(u8::from(item.accepts));
    }
    out
}

/// Registers the placement under its string key. The rebuild is `place`.
pub fn register(registry: &mut ArtifactRegistry, view: View) {
    let inputs: Vec<(RuntimeAddr, RuntimeAddr)> = ranges(&view)
        .into_iter()
        .map(|(s, e)| (runtime_addr(s.as_bytes()), runtime_addr(e.as_bytes())))
        .collect();
    registry.register(PLACEMENT_KEY, inputs, move |store| {
        let mut scene = SceneSet::new(infinite_presenter::core::Revision::new(
            store.head().get(),
        ));
        for (addr, _) in store.range(
            &runtime_addr(&[]),
            &runtime_addr(&[0xFF, 0xFF, 0xFF, 0xFF]),
            store.head(),
        ) {
            scene.insert(infinite_presenter::core::Placeable {
                at: presenter_addr(addr.as_bytes()),
                across: infinite_presenter::core::Extent::fixed(1.0),
                down: infinite_presenter::core::Extent::fixed(1.0),
                position: infinite_presenter::core::Point::ORIGIN,
                style: "plain".into(),
                detail_override: None,
                primitive: infinite_presenter::core::AREA.into(),
                link: None,
                hosts_space: false,
                accepts: true,
                text: "".into(),
            });
        }
        encode_placement(&rebuild(&scene, &view))
    });
}

/// Registers the linked plan under its string key. The rebuild is `link`.
pub fn register_plan(registry: &mut ArtifactRegistry, start: &[u8], end: &[u8], root: &[u8]) {
    let inputs = vec![(runtime_addr(start), runtime_addr(end))];
    let start = start.to_vec();
    let end = end.to_vec();
    let root = root.to_vec();
    {
        let start = start.clone();
        let end = end.clone();
        let root = root.clone();
        registry.register(PLAN_KEY, inputs.clone(), move |store| {
            encode_plan(&linked_plan(store, &start, &end, &root))
        });
    }
    registry.register(TIER0_KEY, inputs, move |store| {
        encode_tier0(&linked_plan(store, &start, &end, &root))
    });
}

fn linked_plan(
    store: &dyn infinite_runtime::binding::ports::StoreRead,
    start: &[u8],
    end: &[u8],
    root: &[u8],
) -> infinite_compositor::core::Plan {
    let mut set = DefinitionSet::default();
    for (addr, payload) in store.range(&runtime_addr(start), &runtime_addr(end), store.head()) {
        let at = compositor_addr(addr.as_bytes());
        if let Some(composition) = decode_composition(&payload) {
            set.compositions.insert(at.clone(), composition);
            set.blocks.insert(
                at.clone(),
                Block {
                    signature: Signature::default(),
                    body: Body {
                        kind: BodyKind::new(BodyKind::COMPOSED),
                        target: at,
                    },
                },
            );
        }
    }
    inject_natives(&mut set);
    link(&set, &compositor_addr(root)).value
}
