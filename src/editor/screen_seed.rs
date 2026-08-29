//! Screen Spec tree for genesis (E16) — kept beside genesis so `genesis.rs` stays short.

use crate::editor::addresses;
use crate::editor::spec::{build, Spec};
use crate::facade::SpaceRecord;

/// The authored screen under [`addresses::SCREEN_ROOT_KEY`].
pub fn screen_tree() -> Vec<Spec> {
    let canvas = Spec::leaf("canvas", 1, host("canvas", [1.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 0.0], true))
        .with_children(vec![
            build::area("node-a", 1, [0.4, 0.4, 0.0], [0.4, 0.4, 0.0], [0.0, 0.0], true).with_children(
                vec![
                    build::area("a1", 1, [0.15, 0.15, 0.0], [0.15, 0.15, 0.0], [0.05, 0.05], false),
                    build::area("a2", 2, [0.15, 0.15, 0.0], [0.15, 0.15, 0.0], [0.05, 0.22], false),
                ],
            ),
            build::area("node-b", 2, [0.4, 0.4, 0.0], [0.4, 0.4, 0.0], [0.5, 0.5], false),
            build::link_wire(
                "wire-ab",
                3,
                addresses::node_a_key().to_vec(),
                addresses::node_b_key().to_vec(),
                0.012,
            ),
        ]);
    let inspector = build::panel("inspector", 2, [0.25, 0.25, 0.0], [1.0, 1.0, 0.0], [0.75, 0.0])
        .with_children(inspector_rows());
    let palette = build::panel("palette", 3, [0.75, 0.75, 0.0], [0.15, 0.15, 0.0], [0.0, 0.85])
        .with_children(palette_rows());
    let toolbar = build::panel("toolbar", 4, [0.75, 0.75, 0.0], [0.08, 0.08, 0.0], [0.0, 0.0])
        .with_children(toolbar_rows());
    vec![canvas, inspector, palette, toolbar]
}

fn host(
    style: &str,
    across: [f64; 3],
    down: [f64; 3],
    origin: [f64; 2],
    accepts: bool,
) -> SpaceRecord {
    SpaceRecord {
        across,
        down,
        style: style.into(),
        detail_override: None,
        hosts_space: true,
        accepts,
        origin,
        primitive: String::new(),
    }
}

fn inspector_rows() -> Vec<Spec> {
    [1, 2, 3, 4, 5, 6]
        .into_iter()
        .zip(["addr", "style", "across", "down", "origin", "depth"])
        .zip([0.02, 0.08, 0.14, 0.20, 0.26, 0.32])
        .map(|((slot, label), y)| {
            build::text_run(
                label,
                slot,
                [0.0, 0.0, 0.0],
                [0.0, 0.025, 0.0],
                [0.02, y],
                format!("{label} —"),
            )
        })
        .collect()
}

fn palette_rows() -> Vec<Spec> {
    vec![
        template("plain", 1, "plain", [0.02, 0.03], "plain", false),
        template("total", 2, "total", [0.14, 0.02], "0", true),
        template("bump", 3, "bump", [0.26, 0.02], "+", true),
    ]
}

fn template(
    name: &str,
    slot: u32,
    style: &str,
    origin: [f64; 2],
    text: &str,
    as_text: bool,
) -> Spec {
    let rec = SpaceRecord {
        across: [0.08, 0.08, 0.0],
        down: [0.08, 0.08, 0.0],
        style: style.into(),
        detail_override: None,
        hosts_space: true,
        accepts: true,
        origin,
        primitive: if as_text {
            "text".into()
        } else {
            String::new()
        },
    };
    Spec::leaf(name, slot, rec)
        .with_payload(text.as_bytes().to_vec())
        .with_children(vec![build::text_run(
            format!("{name}-label"),
            1,
            [0.0, 0.0, 0.0],
            [0.0, 0.025, 0.0],
            [0.0, 1.05],
            name,
        )])
}

fn toolbar_rows() -> Vec<Spec> {
    vec![
        Spec::leaf(
            "history",
            1,
            SpaceRecord {
                across: [0.14, 0.14, 0.0],
                down: [0.04, 0.04, 0.0],
                style: "plain".into(),
                detail_override: None,
                hosts_space: false,
                accepts: true,
                origin: [0.02, 0.02],
                primitive: "text".into(),
            },
        )
        .with_payload(b"undo redo".to_vec()),
        build::text_run("zoom", 2, [0.0, 0.0, 0.0], [0.0, 0.025, 0.0], [0.18, 0.02], "zoom"),
        Spec::leaf(
            "run",
            3,
            SpaceRecord {
                across: [0.08, 0.08, 0.0],
                down: [0.04, 0.04, 0.0],
                style: "plain".into(),
                detail_override: None,
                hosts_space: false,
                accepts: true,
                origin: [0.34, 0.02],
                primitive: "text".into(),
            },
        )
        .with_payload(b"run".to_vec()),
    ]
}
