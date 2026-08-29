//! Stored component definitions (E18b / E20). Genesis writes these; tests re-seed
//! from the store without calling the builders.

use crate::editor::addresses;
use crate::editor::spec::{self, build};
use crate::editor::tags;
use crate::facade::{
    encode_composition, BlockRecord, CompositionRecord, PortRecord, WireRecord,
};

/// Graph-region compositions plus the Innovator screen flatten.
pub fn seed_records() -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut out = vec![
        (
            addresses::increment_def_key().to_vec(),
            encode_composition(&increment_def()),
        ),
        (
            addresses::app_root_key().to_vec(),
            encode_composition(&app_root()),
        ),
        (
            addresses::field_row_def_key().to_vec(),
            encode_composition(&field_row_def()),
        ),
        (
            addresses::panel_def_key().to_vec(),
            encode_composition(&panel_def()),
        ),
        (
            addresses::section_header_def_key().to_vec(),
            encode_composition(&section_header_def()),
        ),
        (
            addresses::action_bar_def_key().to_vec(),
            encode_composition(&action_bar_def()),
        ),
    ];
    for flat in spec::flatten(addresses::SCREEN_ROOT_KEY, &innovator_spaces()) {
        out.push((flat.key, flat.payload));
    }
    out
}

/// One Innovator screen: panel + header + two field rows + action bar.
pub fn innovator_spaces() -> Vec<spec::Spec> {
    let mut panel = build::panel(
        "innovator",
        5,
        [0.22, 0.22, 0.0],
        [0.18, 0.18, 0.0],
        [0.02, 0.68],
    );
    panel.record.accepts = true;
    // Stay open at this size so field rows place (0.22 × zoom 400 < 256px).
    panel.record.detail_override = Some(4);
    vec![panel
    .with_children(vec![
        build::text_run(
            "header",
            1,
            [0.18, 0.18, 0.0],
            [0.03, 0.03, 0.0],
            [0.02, 0.01],
            "Wall",
        ),
        build::field(
            "height",
            2,
            [0.18, 0.18, 0.0],
            [0.03, 0.03, 0.0],
            [0.02, 0.05],
            "height",
        ),
        build::field(
            "width",
            3,
            [0.18, 0.18, 0.0],
            [0.03, 0.03, 0.0],
            [0.02, 0.09],
            "width",
        ),
        {
            let mut bar = build::field(
                "commit",
                4,
                [0.10, 0.10, 0.0],
                [0.03, 0.03, 0.0],
                [0.02, 0.13],
                "commit",
            );
            bar.record.style = "commit".into();
            bar
        },
    ])]
}

fn increment_def() -> CompositionRecord {
    CompositionRecord {
        compilable: false,
        blocks: vec![
            native(
                addresses::increment_read_key(),
                b"read",
                vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("val", false, tags::VALUE, false),
                ],
            ),
            mapped(addresses::increment_map_key()),
            native(
                addresses::increment_amend_key(),
                b"amend",
                vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("val", true, tags::VALUE, false),
                    port("pending", false, tags::FLAG, false),
                ],
            ),
            native(
                addresses::increment_commit_key(),
                b"commit",
                vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("done", false, tags::FLAG, false),
                ],
            ),
        ],
        wires: vec![
            wire(
                &[(addresses::increment_read_key(), "val")],
                &[(addresses::increment_map_key(), "val")],
            ),
            wire(
                &[(addresses::increment_map_key(), "out")],
                &[(addresses::increment_amend_key(), "val")],
            ),
        ],
    }
}

fn app_root() -> CompositionRecord {
    CompositionRecord {
        compilable: false,
        blocks: vec![BlockRecord {
            at: addresses::app_use_key().to_vec(),
            kind: "delegate".into(),
            target: addresses::increment_def_key().to_vec(),
            ports: Vec::new(),
        }],
        wires: Vec::new(),
    }
}

fn field_row_def() -> CompositionRecord {
    CompositionRecord {
        compilable: true,
        blocks: vec![
            mapped(addresses::field_row_map_key()),
            folded(addresses::field_row_fold_key()),
        ],
        wires: vec![wire(
            &[(addresses::field_row_map_key(), "out")],
            &[(addresses::field_row_fold_key(), "left")],
        )],
    }
}

fn panel_def() -> CompositionRecord {
    CompositionRecord {
        compilable: true,
        blocks: vec![folded(addresses::panel_fold_key())],
        wires: Vec::new(),
    }
}

fn section_header_def() -> CompositionRecord {
    CompositionRecord {
        compilable: true,
        blocks: vec![mapped(addresses::section_header_map_key())],
        wires: Vec::new(),
    }
}

fn action_bar_def() -> CompositionRecord {
    CompositionRecord {
        compilable: true,
        blocks: vec![folded(addresses::action_bar_fold_key())],
        wires: Vec::new(),
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

fn folded(at: &[u8]) -> BlockRecord {
    native(
        at,
        b"fold",
        vec![
            port("fn", true, tags::VALUE, false),
            port("left", true, tags::VALUE, false),
            port("right", true, tags::VALUE, false),
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

fn wire(sources: &[(&[u8], &str)], sinks: &[(&[u8], &str)]) -> WireRecord {
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
