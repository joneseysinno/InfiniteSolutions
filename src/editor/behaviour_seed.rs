//! Behaviour composition seed — kept out of [`super::genesis`] so that file stays
//! under the E16.1 line budget (authoring spaces, not wiring tables).

use crate::editor::addresses;
use crate::editor::tags;
use crate::facade::{BlockRecord, CompositionRecord, PortRecord, WireRecord};

/// The editor's behaviour composition.
pub fn behaviour() -> CompositionRecord {
    let probe = native(
        addresses::behaviour_probe_key(),
        b"probe-at",
        vec![
            port("at", true, tags::POINT, false),
            port("hit", false, tags::ADDRESS, true),
        ],
    );
    let read = native(
        addresses::behaviour_read_key(),
        b"read",
        vec![
            port("addr", true, tags::ADDRESS, true),
            port("val", false, tags::VALUE, false),
        ],
    );
    let amend = native(
        addresses::behaviour_amend_key(),
        b"amend",
        vec![
            port("addr", true, tags::ADDRESS, true),
            port("val", true, tags::VALUE, false),
            port("pending", false, tags::FLAG, false),
        ],
    );
    let commit = native(
        addresses::behaviour_commit_key(),
        b"commit",
        vec![
            port("addr", true, tags::ADDRESS, true),
            port("done", false, tags::FLAG, false),
        ],
    );
    let offset = mapped(addresses::behaviour_offset_key());
    let gate = native(
        addresses::behaviour_gate_key(),
        b"gate",
        vec![
            port("val", true, tags::VALUE, false),
            port("on", true, tags::FLAG, false),
            port("pass", false, tags::VALUE, false),
        ],
    );
    let displace = mapped(addresses::behaviour_displace_key());
    let select_gate = native(
        addresses::behaviour_select_gate_key(),
        b"gate",
        vec![
            port("val", true, tags::VALUE, false),
            port("on", true, tags::FLAG, false),
            port("pass", false, tags::VALUE, false),
        ],
    );
    let encode_selection = native(
        addresses::behaviour_encode_selection_key(),
        b"map",
        vec![
            port("fn", true, tags::VALUE, false),
            port("val", true, tags::ADDRESS, false),
            port("aux", true, tags::VALUE, false),
            port("out", false, tags::VALUE, false),
        ],
    );
    let select_amend = native(
        addresses::behaviour_select_amend_key(),
        b"amend",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", true, tags::VALUE, false),
            port("pending", false, tags::FLAG, false),
        ],
    );
    let select_commit = native(
        addresses::behaviour_select_commit_key(),
        b"commit",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("done", false, tags::FLAG, false),
        ],
    );
    let edit_read = native(
        addresses::behaviour_edit_read_key(),
        b"read",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", false, tags::VALUE, false),
        ],
    );
    let set_origin = mapped(addresses::behaviour_set_origin_key());
    let edit_gate = native(
        addresses::behaviour_edit_gate_key(),
        b"gate",
        vec![
            port("val", true, tags::VALUE, false),
            port("on", true, tags::FLAG, false),
            port("pass", false, tags::VALUE, false),
        ],
    );
    let edit_amend = native(
        addresses::behaviour_edit_amend_key(),
        b"amend",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", true, tags::VALUE, false),
            port("pending", false, tags::FLAG, false),
        ],
    );
    let edit_commit = native(
        addresses::behaviour_edit_commit_key(),
        b"commit",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("done", false, tags::FLAG, false),
        ],
    );
    let place_read = native(
        addresses::behaviour_place_read_key(),
        b"read",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", false, tags::VALUE, false),
        ],
    );
    let place_set_origin = mapped(addresses::behaviour_place_set_origin_key());
    let place_gate = native(
        addresses::behaviour_place_gate_key(),
        b"gate",
        vec![
            port("val", true, tags::VALUE, false),
            port("on", true, tags::FLAG, false),
            port("pass", false, tags::VALUE, false),
        ],
    );
    let place_amend = native(
        addresses::behaviour_place_amend_key(),
        b"amend",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", true, tags::VALUE, false),
            port("pending", false, tags::FLAG, false),
        ],
    );
    let place_commit = native(
        addresses::behaviour_place_commit_key(),
        b"commit",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("done", false, tags::FLAG, false),
        ],
    );
    let encode_wire = mapped(addresses::behaviour_encode_wire_key());
    let wire_gate = native(
        addresses::behaviour_wire_gate_key(),
        b"gate",
        vec![
            port("val", true, tags::VALUE, false),
            port("on", true, tags::FLAG, false),
            port("pass", false, tags::VALUE, false),
        ],
    );
    let wire_amend = native(
        addresses::behaviour_wire_amend_key(),
        b"amend",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", true, tags::VALUE, false),
            port("pending", false, tags::FLAG, false),
        ],
    );
    let wire_commit = native(
        addresses::behaviour_wire_commit_key(),
        b"commit",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("done", false, tags::FLAG, false),
        ],
    );
    let text_read = native(
        addresses::behaviour_text_read_key(),
        b"read",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", false, tags::VALUE, false),
        ],
    );
    let text_map = mapped(addresses::behaviour_text_map_key());
    let text_gate = native(
        addresses::behaviour_text_gate_key(),
        b"gate",
        vec![
            port("val", true, tags::VALUE, false),
            port("on", true, tags::FLAG, false),
            port("pass", false, tags::VALUE, false),
        ],
    );
    let text_amend = native(
        addresses::behaviour_text_amend_key(),
        b"amend",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("val", true, tags::VALUE, false),
            port("pending", false, tags::FLAG, false),
        ],
    );
    let text_commit = native(
        addresses::behaviour_text_commit_key(),
        b"commit",
        vec![
            port("addr", true, tags::ADDRESS, false),
            port("done", false, tags::FLAG, false),
        ],
    );
    CompositionRecord {
        compilable: false,
        blocks: vec![
            probe,
            read,
            amend,
            commit,
            offset,
            gate,
            displace,
            select_gate,
            encode_selection,
            select_amend,
            select_commit,
            edit_read,
            set_origin,
            edit_gate,
            edit_amend,
            edit_commit,
            place_read,
            place_set_origin,
            place_gate,
            place_amend,
            place_commit,
            encode_wire,
            wire_gate,
            wire_amend,
            wire_commit,
            text_read,
            text_map,
            text_gate,
            text_amend,
            text_commit,
        ],
        wires: vec![
            w(
                &[(addresses::behaviour_probe_key(), "hit")],
                &[
                    (addresses::behaviour_read_key(), "addr"),
                    (addresses::behaviour_amend_key(), "addr"),
                    (addresses::behaviour_commit_key(), "addr"),
                    (addresses::behaviour_encode_selection_key(), "val"),
                ],
            ),
            w(
                &[(addresses::behaviour_read_key(), "val")],
                &[(addresses::behaviour_displace_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_offset_key(), "out")],
                &[(addresses::behaviour_displace_key(), "aux")],
            ),
            w(
                &[(addresses::behaviour_displace_key(), "out")],
                &[(addresses::behaviour_gate_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_gate_key(), "pass")],
                &[(addresses::behaviour_amend_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_encode_selection_key(), "out")],
                &[(addresses::behaviour_select_gate_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_select_gate_key(), "pass")],
                &[(addresses::behaviour_select_amend_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_edit_read_key(), "val")],
                &[(addresses::behaviour_set_origin_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_set_origin_key(), "out")],
                &[(addresses::behaviour_edit_gate_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_edit_gate_key(), "pass")],
                &[(addresses::behaviour_edit_amend_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_place_read_key(), "val")],
                &[(addresses::behaviour_place_set_origin_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_place_set_origin_key(), "out")],
                &[(addresses::behaviour_place_gate_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_place_gate_key(), "pass")],
                &[(addresses::behaviour_place_amend_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_encode_wire_key(), "out")],
                &[(addresses::behaviour_wire_gate_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_wire_gate_key(), "pass")],
                &[(addresses::behaviour_wire_amend_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_text_read_key(), "val")],
                &[(addresses::behaviour_text_map_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_text_map_key(), "out")],
                &[(addresses::behaviour_text_gate_key(), "val")],
            ),
            w(
                &[(addresses::behaviour_text_gate_key(), "pass")],
                &[(addresses::behaviour_text_amend_key(), "val")],
            ),
        ],
    }
}

fn w(sources: &[(&[u8], &str)], sinks: &[(&[u8], &str)]) -> WireRecord {
    WireRecord {
        sources: sources
            .iter()
            .map(|(a, p)| (a.to_vec(), (*p).into()))
            .collect(),
        sinks: sinks
            .iter()
            .map(|(a, p)| (a.to_vec(), (*p).into()))
            .collect(),
    }
}

fn mapped(at: &[u8]) -> BlockRecord {
    native(
        at,
        b"map",
        vec![
            port("fn", true, tags::VALUE, false),
            port("val", true, tags::VALUE, false),
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
