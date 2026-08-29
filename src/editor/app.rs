//! User-authored apps — a wire from bump to total runs an increment graph (E13.7).

use crate::editor::addresses;
use crate::editor::tags;
use crate::facade::{
    decode_space, encode_composition, BlockRecord, CompositionRecord, PortRecord, Store,
    WireRecord,
};

/// Links a bump block to a total block and installs the increment composition.
pub fn connect(store: &Store, from: &[u8], to: &[u8]) {
    let Some((bump, total)) = classify(store, from, to) else {
        return;
    };
    let mut link = Vec::with_capacity(8);
    link.extend_from_slice(&bump);
    link.extend_from_slice(&total);
    store.put(addresses::APP_LINK_KEY, &link);
    store.put(addresses::APP_ROOT_KEY, &increment_graph());
}

fn classify(store: &Store, a: &[u8], b: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let sa = space_at(store, a)?;
    let sb = space_at(store, b)?;
    if is_bump(&sa) && is_total(&sb) {
        Some((a.to_vec(), b.to_vec()))
    } else if is_bump(&sb) && is_total(&sa) {
        Some((b.to_vec(), a.to_vec()))
    } else {
        None
    }
}

fn space_at(store: &Store, key: &[u8]) -> Option<crate::facade::SpaceRecord> {
    store
        .stored_at(key)
        .or_else(|| store.pending_at(key))
        .and_then(|b| decode_space(&b))
}

fn is_bump(space: &crate::facade::SpaceRecord) -> bool {
    space.style == "bump" || space.text == "+"
}

fn is_total(space: &crate::facade::SpaceRecord) -> bool {
    space.style == "total" || space.text.parse::<i64>().is_ok()
}

/// Runs the app graph when the bump block is clicked.
pub fn try_run(store: &Store) {
    let Some(link) = store.stored_at(addresses::APP_LINK_KEY) else {
        return;
    };
    if link.len() < 8 {
        return;
    }
    let bump = &link[0..4];
    let total = &link[4..8];
    let Some(pos) = store.pending_at(addresses::POINTER_POSITION.as_bytes()) else {
        return;
    };
    let Some(hit) = store.probe_at(
        f64::from_le_bytes(pos[0..8].try_into().unwrap_or([0; 8])),
        f64::from_le_bytes(pos[8..16].try_into().unwrap_or([0; 8])),
    ) else {
        return;
    };
    if hit != bump {
        return;
    }
    store.write_slot(addresses::APP_READ_KEY, "addr", total, tags::ADDRESS);
    store.write_slot(addresses::APP_AMEND_KEY, "addr", total, tags::ADDRESS);
    store.write_slot(addresses::APP_COMMIT_KEY, "addr", total, tags::ADDRESS);
    store.run_at(addresses::APP_ROOT_KEY);
}

fn increment_graph() -> Vec<u8> {
    encode_composition(&CompositionRecord {
        compilable: true,
        blocks: vec![
            native(
                addresses::APP_READ_KEY,
                b"read",
                vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("val", false, tags::VALUE, false),
                ],
            ),
            native(
                addresses::APP_INCREMENT_KEY,
                b"increment-text",
                vec![
                    port("val", true, tags::VALUE, true),
                    port("out", false, tags::VALUE, false),
                ],
            ),
            native(
                addresses::APP_AMEND_KEY,
                b"amend",
                vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("val", true, tags::VALUE, false),
                    port("pending", false, tags::FLAG, false),
                ],
            ),
            native(
                addresses::APP_COMMIT_KEY,
                b"commit",
                vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("done", false, tags::FLAG, false),
                ],
            ),
        ],
        wires: vec![
            WireRecord {
                sources: vec![(addresses::APP_READ_KEY.to_vec(), "val".into())],
                sinks: vec![(addresses::APP_INCREMENT_KEY.to_vec(), "val".into())],
            },
            WireRecord {
                sources: vec![(addresses::APP_INCREMENT_KEY.to_vec(), "out".into())],
                sinks: vec![(addresses::APP_AMEND_KEY.to_vec(), "val".into())],
            },
        ],
    })
}

fn native(at: &[u8], target: &[u8], ports: Vec<PortRecord>) -> BlockRecord {
    BlockRecord {
        at: at.to_vec(),
        kind: "native".into(),
        target: target.to_vec(),
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
