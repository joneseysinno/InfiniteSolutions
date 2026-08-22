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
        let at = self.inner.db.stable_revision().legacy_sequence();
        let start = root.as_bytes();
        let end = successor(start);
        let mut rows = match self.inner.records_in_range(start, &end, at) {
            Ok(rows) => rows,
            Err(e) => panic!("definition read failed (not an empty set): {e}"),
        };
        for (bytes, payload) in self.inner.overlay_pending(start, &end) {
            if let Some(existing) = rows.iter_mut().find(|(b, _)| b == &bytes) {
                existing.1 = payload;
            } else {
                rows.push((bytes, payload));
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
        super::blocks::inject_natives(&mut set);
        set
    }
}

fn successor(bytes: &[u8]) -> Vec<u8> {
    let mut c = Inner::coord(bytes);
    c = c.saturating_add(1);
    Inner::bytes_of(c)
}
