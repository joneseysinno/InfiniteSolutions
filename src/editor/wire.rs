//! Wiring by pointer — preview at [`addresses::GRAPH_ROOT_KEY`], commit via composition.

use crate::editor::addresses;
use crate::editor::mint;
use crate::editor::tags;
use crate::facade::{
    decode_space, encode_composition, BlockRecord, CompositionRecord, PortRecord, Store,
    WireRecord,
};

/// Whether shift-wire mode is active (gesture address or portal shift+drag).
pub fn mode_active(store: &Store) -> bool {
    store
        .pending_at(addresses::wire_mode_key())
        .is_some_and(|b| b.first().copied().unwrap_or(0) != 0)
}

/// A draggable wire endpoint under the canvas — not an existing wire primitive.
pub fn is_endpoint(store: &Store, key: &[u8]) -> bool {
    if key.len() <= addresses::canvas_key().len() || !key.starts_with(addresses::canvas_key()) {
        return false;
    }
    let Some(payload) = store
        .pending_at(key)
        .or_else(|| store.stored_at(key))
    else {
        return false;
    };
    let Some(space) = decode_space(&payload) else {
        return false;
    };
    space.accepts && space.primitive != "wire"
}

/// Pending composition for C4 preview while the wire is in flight (D39).
pub fn preview_graph(from: &[u8], to: &[u8], mismatch: bool) -> Vec<u8> {
    encode_composition(&if mismatch {
        mismatch_graph(from, to)
    } else {
        valid_graph(from, to)
    })
}

fn valid_graph(from: &[u8], to: &[u8]) -> CompositionRecord {
    CompositionRecord {
        compilable: false,
        blocks: vec![
            mapped(from),
            mapped(to),
        ],
        wires: vec![],
    }
}

fn mismatch_graph(from: &[u8], to: &[u8]) -> CompositionRecord {
    CompositionRecord {
        compilable: false,
        blocks: vec![
            mapped(from),
            native(
                to,
                b"commit",
                vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("done", false, tags::FLAG, false),
                ],
            ),
        ],
        wires: vec![WireRecord {
            sources: vec![(from.to_vec(), "out".into())],
            sinks: vec![(to.to_vec(), "addr".into())],
        }],
    }
}

fn mapped(at: &[u8]) -> BlockRecord {
    native(
        at,
        b"map",
        vec![
            port("fn", true, tags::VALUE, true),
            port("val", true, tags::VALUE, true),
            port("aux", true, tags::VALUE, false),
            port("out", false, tags::VALUE, false),
        ],
    )
}

fn native(at: &[u8], key: &[u8], ports: Vec<PortRecord>) -> BlockRecord {
    BlockRecord {
        at: at.to_vec(),
        kind: "native".into(),
        target: key.to_vec(),
        ports,
    }
}

fn port(name: &str, incoming: bool, tag: &str, required: bool) -> PortRecord {
    PortRecord {
        name: name.into(),
        incoming,
        tag: tag.into(),
        arity: None,
        required,
    }
}

/// Latches [`addresses::wire_from_key()`] and mints [`addresses::wire_addr_key()`].
pub fn begin(store: &Store, from: &[u8]) -> bool {
    let parent = mint::parent_key(from);
    let Some(addr) = store.mint_under(&parent) else {
        return false;
    };
    store.amend(addresses::wire_from_key(), from);
    store.amend(addresses::wire_addr_key(), &addr);
    true
}

/// Updates preview graph and target while the pointer moves.
pub fn update(store: &Store, to: &[u8], mismatch: bool) {
    let Some(from) = store.pending_at(addresses::wire_from_key()) else {
        return;
    };
    if !is_endpoint(store, to) || from == to {
        return;
    }
    store.amend(addresses::wire_to_key(), to);
    store.amend(
        addresses::GRAPH_ROOT_KEY,
        &preview_graph(&from, to, mismatch),
    );
}

/// Queues commit of the minted wire record and the preview graph.
pub fn finish(store: &Store, to: &[u8], mismatch: bool) {
    let Some(from) = store.pending_at(addresses::wire_from_key()) else {
        return;
    };
    if is_endpoint(store, to) && from != to {
        store.amend(addresses::wire_to_key(), to);
        store.amend(
            addresses::GRAPH_ROOT_KEY,
            &preview_graph(&from, to, mismatch),
        );
        store.amend(addresses::wire_commit_key(), &[1]);
    }
}
