//! [`Definitions`] — compositor. A definition set at a revision, stored ∪ pending.
//!
//! That union is C4 and it is the port's whole point.

use std::sync::Arc;

use infinite_compositor::binding::ports::Definitions as Port;
use infinite_compositor::core::{
    Addr, Block, Body, BodyKind, DefinitionSet, Signature,
};

use crate::facade::addr::compositor_addr;
use crate::facade::open::Inner;

/// Stored records unioned with the runtime's pending set.
pub struct Definitions {
    pub(crate) inner: Arc<Inner>,
}

impl Port for Definitions {
    fn resolve(&self, root: &Addr) -> DefinitionSet {
        // Point reads only. A Hilbert range over `[root, successor)` is not
        // the same as key order and can scan the whole space (E18b).
        let mut set = ingest_point(self, root.as_bytes());
        let graph = self.inner.graph_root.lock().expect("graph lock").clone();
        if let Some(graph) = graph {
            merge(&mut set, ingest_slots(self, &graph, 16));
        }
        if root.as_bytes().len() == 1 {
            merge(&mut set, ingest_slots(self, root.as_bytes(), 16));
        }
        super::blocks::inject_natives(&mut set);
        set
    }
}

fn ingest_slots(defs: &Definitions, parent: &[u8], last: u32) -> DefinitionSet {
    let mut set = ingest_point(defs, parent);
    for slot in 1..=last {
        let mut key = parent.to_vec();
        key.push((slot >> 8) as u8);
        key.push((slot & 0xFF) as u8);
        merge(&mut set, ingest_point(defs, &key));
    }
    set
}

fn ingest_point(defs: &Definitions, key: &[u8]) -> DefinitionSet {
    let mut rows = Vec::new();
    if let Some(payload) = defs.inner.current_value(key) {
        rows.push((key.to_vec(), payload));
    }
    let end = Inner::successor_key(key);
    for (bytes, payload) in defs.inner.overlay_pending(key, &end) {
        if bytes.as_slice() == key {
            if let Some(existing) = rows.iter_mut().find(|(b, _)| b == key) {
                existing.1 = payload;
            } else {
                rows.push((bytes, payload));
            }
        }
    }
    let mut set = DefinitionSet::default();
    for (bytes, payload) in rows {
        let at = compositor_addr(&bytes);
        if let Some(composition) = crate::facade::decode_composition(&payload) {
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
        } else {
            set.blocks.insert(
                at.clone(),
                Block {
                    signature: Signature::default(),
                    body: Body {
                        kind: BodyKind::new(BodyKind::NATIVE),
                        target: at,
                    },
                },
            );
        }
    }
    set
}

fn merge(into: &mut DefinitionSet, extra: DefinitionSet) {
    for (at, composition) in extra.compositions {
        into.compositions.entry(at).or_insert(composition);
    }
    for (at, block) in extra.blocks {
        into.blocks.entry(at).or_insert(block);
    }
}
