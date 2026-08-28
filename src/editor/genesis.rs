//! The seed. Writes graph data. Contains no policy (`docs/specs/EDITOR.md` §6).
//!
//! No layout algorithm, no behaviour, and no conditional beyond "does this space
//! already exist". If genesis ever grows a decision, E4's discard test is how
//! you find out.

use crate::editor::addresses;
use crate::editor::styles::bootstrap_default;
use crate::editor::tags;
use crate::facade::{
    encode_composition, encode_space, encode_style, BlockRecord, CompositionRecord, PortRecord,
    SpaceRecord, WireRecord,
};

/// Writes the editor's screen, the `plain` style row, and the behaviour composition.
pub fn seed(exists: impl Fn(&[u8]) -> bool, mut put: impl FnMut(&[u8], &[u8])) {
    let canvas = encode_space(&SpaceRecord {
        across: [1.0, 1.0, 0.0],
        down: [1.0, 1.0, 0.0],
        style: "canvas".into(),
        detail_override: None,
        hosts_space: true,
        accepts: true,
        origin: [0.0, 0.0],
        primitive: String::new(),
        link: None,
    });
    let node = encode_space(&SpaceRecord {
        across: [0.4, 0.4, 0.0],
        down: [0.4, 0.4, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: true,
        accepts: true,
        origin: [0.0, 0.0],
        primitive: String::new(),
        link: None,
    });
    // The two nodes inside node A's own space. This is the fixture D20's claim needs:
    // a space, containing a node, which itself hosts a space that is populated. Both
    // sit inside node A's own 0.4 × 0.4 box, in node A's coordinates.
    let node_a1 = encode_space(&SpaceRecord {
        across: [0.15, 0.15, 0.0],
        down: [0.15, 0.15, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
        origin: [0.05, 0.05],
        primitive: String::new(),
        link: None,
    });
    let node_a2 = encode_space(&SpaceRecord {
        across: [0.15, 0.15, 0.0],
        down: [0.15, 0.15, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
        origin: [0.05, 0.22],
        primitive: String::new(),
        link: None,
    });
    let node_b = encode_space(&SpaceRecord {
        across: [0.4, 0.4, 0.0],
        down: [0.4, 0.4, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
        // Down as well as across, so the wire E11 draws between the two nodes is a
        // diagonal. A horizontal wire is indistinguishable from its own bounding box,
        // and a check that cannot tell a line from a box is not a check that a line
        // was drawn (`tests/wires.rs`).
        origin: [0.5, 0.5],
        primitive: String::new(),
        link: None,
    });
    // E11. A hyperedge has no authored position: `across` is the stroke's width, in
    // the canvas's own units, and `origin` is unread. `accepts` is false because
    // nothing yet answers a click on a wire — E13's property inspector is the
    // consumer that turns it on, and turning it on before then would be R27's defect.
    let wire = encode_space(&SpaceRecord {
        across: [0.012, 0.012, 0.0],
        down: [0.012, 0.012, 0.0],
        style: "wire".into(),
        detail_override: None,
        hosts_space: false,
        accepts: false,
        origin: [0.0, 0.0],
        primitive: "wire".into(),
        link: Some((
            addresses::NODE_A_KEY.to_vec(),
            addresses::NODE_B_KEY.to_vec(),
        )),
    });
    let plain = encode_style("plain", bootstrap_default("plain").fill);
    let canvas_style = encode_style("canvas", bootstrap_default("canvas").fill);
    let wire_style = encode_style("wire", bootstrap_default("wire").fill);
    let behaviour = encode_composition(&behaviour());

    put_if(&exists, &mut put, addresses::CANVAS_KEY, &canvas);
    put_if(&exists, &mut put, addresses::NODE_A_KEY, &node);
    put_if(&exists, &mut put, addresses::NODE_A1_KEY, &node_a1);
    put_if(&exists, &mut put, addresses::NODE_A2_KEY, &node_a2);
    put_if(&exists, &mut put, addresses::NODE_B_KEY, &node_b);
    put_if(&exists, &mut put, addresses::WIRE_AB_KEY, &wire);
    put_if(&exists, &mut put, addresses::STYLE_PLAIN_KEY, &plain);
    put_if(&exists, &mut put, addresses::STYLE_CANVAS_KEY, &canvas_style);
    put_if(&exists, &mut put, addresses::STYLE_WIRE_KEY, &wire_style);
    put_if(&exists, &mut put, addresses::BEHAVIOUR_ROOT_KEY, &behaviour);
}

fn behaviour() -> CompositionRecord {
    let probe = native(
        addresses::BEHAVIOUR_PROBE_KEY,
        b"probe-at",
        vec![
            port("at", true, tags::POINT, false),
            port("hit", false, tags::ADDRESS, true),
        ],
    );
    let read = native(
        addresses::BEHAVIOUR_READ_KEY,
        b"read",
        vec![
            port("addr", true, tags::ADDRESS, true),
            port("val", false, tags::VALUE, false),
        ],
    );
    let amend = native(
        addresses::BEHAVIOUR_AMEND_KEY,
        b"amend",
        vec![
            port("addr", true, tags::ADDRESS, true),
            port("val", true, tags::VALUE, false),
            port("pending", false, tags::FLAG, false),
        ],
    );
    let commit = native(
        addresses::BEHAVIOUR_COMMIT_KEY,
        b"commit",
        vec![
            port("addr", true, tags::ADDRESS, true),
            port("done", false, tags::FLAG, false),
        ],
    );
    let offset = native(
        addresses::BEHAVIOUR_OFFSET_KEY,
        b"offset",
        vec![
            port("from", true, tags::POINT, false),
            port("to", true, tags::POINT, false),
            port("delta", false, tags::POINT, false),
        ],
    );
    let gate = native(
        addresses::BEHAVIOUR_GATE_KEY,
        b"gate",
        vec![
            port("val", true, tags::VALUE, false),
            port("on", true, tags::FLAG, false),
            port("pass", false, tags::VALUE, false),
        ],
    );
    let displace = native(
        addresses::BEHAVIOUR_DISPLACE_KEY,
        b"displace",
        vec![
            port("record", true, tags::VALUE, true),
            port("delta", true, tags::POINT, true),
            port("out", false, tags::VALUE, false),
        ],
    );
    CompositionRecord {
        compilable: false,
        blocks: vec![probe, read, amend, commit, offset, gate, displace],
        wires: vec![
            WireRecord {
                sources: vec![(addresses::BEHAVIOUR_PROBE_KEY.to_vec(), "hit".into())],
                sinks: vec![
                    (addresses::BEHAVIOUR_READ_KEY.to_vec(), "addr".into()),
                    (addresses::BEHAVIOUR_AMEND_KEY.to_vec(), "addr".into()),
                    (addresses::BEHAVIOUR_COMMIT_KEY.to_vec(), "addr".into()),
                ],
            },
            WireRecord {
                sources: vec![(addresses::BEHAVIOUR_READ_KEY.to_vec(), "val".into())],
                sinks: vec![(addresses::BEHAVIOUR_DISPLACE_KEY.to_vec(), "record".into())],
            },
            WireRecord {
                sources: vec![(addresses::BEHAVIOUR_OFFSET_KEY.to_vec(), "delta".into())],
                sinks: vec![(addresses::BEHAVIOUR_DISPLACE_KEY.to_vec(), "delta".into())],
            },
            WireRecord {
                sources: vec![(addresses::BEHAVIOUR_DISPLACE_KEY.to_vec(), "out".into())],
                sinks: vec![(addresses::BEHAVIOUR_GATE_KEY.to_vec(), "val".into())],
            },
            WireRecord {
                sources: vec![(addresses::BEHAVIOUR_GATE_KEY.to_vec(), "pass".into())],
                sinks: vec![(addresses::BEHAVIOUR_AMEND_KEY.to_vec(), "val".into())],
            },
        ],
    }
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

fn put_if(
    exists: &impl Fn(&[u8]) -> bool,
    put: &mut impl FnMut(&[u8], &[u8]),
    key: &[u8],
    payload: &[u8],
) {
    if !exists(key) {
        put(key, payload);
    }
}
