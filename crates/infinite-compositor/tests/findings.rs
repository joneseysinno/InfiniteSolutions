//! The S4 corpus: the error surface (D16, spec §6).
//!
//! **Green check:** a corpus of malformed compositions, each yielding **exactly one**
//! finding, at an address, carrying said / wanted / remedy.

use infinite_compositor::core::{
    kind, link, Addr, Block, Body, BodyKind, Composition, DefinitionSet, Direction, Port,
    PortRef, Signature, Tag, Wire,
};

fn a(n: u8) -> Addr {
    Addr::new(vec![n])
}

fn native(key: &[u8], ports: Vec<Port>) -> Block {
    Block {
        signature: Signature { ports },
        body: Body {
            kind: BodyKind::new(BodyKind::NATIVE),
            target: Addr::new(key.to_vec()),
        },
    }
}

fn port(name: &str, dir: Direction, tag: &str) -> Port {
    Port::new(name, dir, Tag::new(tag))
}

fn pref(block: u8, name: &str) -> PortRef {
    PortRef {
        block: a(block),
        port: name.into(),
    }
}

fn wire(from: (u8, &str), to: (u8, &str)) -> Wire {
    Wire {
        sources: vec![pref(from.0, from.1)],
        sinks: vec![pref(to.0, to.1)],
    }
}

fn registered(defs: &mut DefinitionSet, key: &[u8], tag_in: &str, tag_out: &str) {
    defs.blocks.insert(
        Addr::new(key.to_vec()),
        native(
            key,
            vec![
                port("in", Direction::In, tag_in),
                port("out", Direction::Out, tag_out),
            ],
        ),
    );
}

struct Case {
    name: &'static str,
    kind: &'static str,
    defs: DefinitionSet,
    root: Addr,
}

fn corpus() -> Vec<Case> {
    let ok = b"ok".as_ref();

    let unsatisfied = {
        let mut defs = DefinitionSet::default();
        registered(&mut defs, ok, "point", "point");
        let mut blocks = std::collections::BTreeMap::new();
        blocks.insert(a(1), native(ok, vec![port("in", Direction::In, "point")]));
        defs.compositions.insert(
            a(0),
            Composition {
                blocks,
                wires: vec![],
                compilable: false,
            },
        );
        Case {
            name: "unsatisfied-import",
            kind: kind::UNSATISFIED_IMPORT,
            defs,
            root: a(0),
        }
    };

    let mismatch = {
        let mut defs = DefinitionSet::default();
        registered(&mut defs, ok, "point", "point");
        let mut blocks = std::collections::BTreeMap::new();
        blocks.insert(
            a(1),
            native(ok, vec![port("out", Direction::Out, "roster")]),
        );
        blocks.insert(
            a(2),
            native(ok, vec![port("in", Direction::In, "drill")]),
        );
        defs.compositions.insert(
            a(0),
            Composition {
                blocks,
                wires: vec![wire((1, "out"), (2, "in"))],
                compilable: false,
            },
        );
        Case {
            name: "tag-mismatch",
            kind: kind::TAG_MISMATCH,
            defs,
            root: a(0),
        }
    };

    let arity = {
        let mut defs = DefinitionSet::default();
        registered(&mut defs, ok, "point", "point");
        let mut sink = port("in", Direction::In, "point");
        sink.arity = Some(1);
        let mut blocks = std::collections::BTreeMap::new();
        blocks.insert(a(1), native(ok, vec![port("out", Direction::Out, "point")]));
        blocks.insert(a(2), native(ok, vec![port("out", Direction::Out, "point")]));
        blocks.insert(a(3), native(ok, vec![sink]));
        defs.compositions.insert(
            a(0),
            Composition {
                blocks,
                wires: vec![wire((1, "out"), (3, "in")), wire((2, "out"), (3, "in"))],
                compilable: false,
            },
        );
        Case {
            name: "arity",
            kind: kind::ARITY,
            defs,
            root: a(0),
        }
    };

    let cycle = {
        let mut defs = DefinitionSet::default();
        registered(&mut defs, ok, "point", "point");
        let mut blocks = std::collections::BTreeMap::new();
        blocks.insert(
            a(1),
            native(
                ok,
                vec![
                    port("in", Direction::In, "point"),
                    port("out", Direction::Out, "point"),
                ],
            ),
        );
        blocks.insert(
            a(2),
            native(
                ok,
                vec![
                    port("in", Direction::In, "point"),
                    port("out", Direction::Out, "point"),
                ],
            ),
        );
        defs.compositions.insert(
            a(0),
            Composition {
                blocks,
                wires: vec![wire((1, "out"), (2, "in")), wire((2, "out"), (1, "in"))],
                compilable: false,
            },
        );
        Case {
            name: "cycle",
            kind: kind::CYCLE,
            defs,
            root: a(0),
        }
    };

    let unresolved = {
        let mut defs = DefinitionSet::default();
        let mut blocks = std::collections::BTreeMap::new();
        blocks.insert(
            a(1),
            native(b"missing", vec![port("in", Direction::In, "point")]),
        );
        defs.compositions.insert(
            a(0),
            Composition {
                blocks,
                wires: vec![],
                compilable: false,
            },
        );
        Case {
            name: "unresolved-block",
            kind: kind::UNRESOLVED_BLOCK,
            defs,
            root: a(0),
        }
    };

    let not_pure = {
        let mut defs = DefinitionSet::default();
        registered(&mut defs, ok, "point", "point");
        let mut blocks = std::collections::BTreeMap::new();
        blocks.insert(a(1), native(ok, vec![port("in", Direction::In, "point")]));
        defs.compositions.insert(
            a(0),
            Composition {
                blocks,
                wires: vec![Wire {
                    sources: vec![PortRef {
                        block: a(9),
                        port: "secret".into(),
                    }],
                    sinks: vec![pref(1, "in")],
                }],
                compilable: true,
            },
        );
        Case {
            name: "not-pure",
            kind: kind::NOT_PURE,
            defs,
            root: a(0),
        }
    };

    vec![unsatisfied, mismatch, arity, cycle, unresolved, not_pure]
}

#[test]
fn every_finding_has_a_site_and_a_remedy() {
    for case in corpus() {
        let out = link(&case.defs, &case.root);
        assert_eq!(
            out.findings.len(),
            1,
            "{}: expected one finding, got {:?}",
            case.name,
            out.findings
        );
        let f = &out.findings[0];
        assert_eq!(f.kind, case.kind, "{}", case.name);
        assert!(!f.site.as_bytes().is_empty(), "{}: site", case.name);
        assert!(!f.said.is_empty(), "{}: said", case.name);
        assert!(!f.wanted.is_empty(), "{}: wanted", case.name);
        assert!(!f.remedy.is_empty(), "{}: remedy", case.name);
    }
}

#[test]
fn one_cause_yields_one_finding() {
    for case in corpus() {
        let out = link(&case.defs, &case.root);
        assert_eq!(
            out.findings.len(),
            1,
            "{} cascaded: {:?}",
            case.name,
            out.findings
        );
    }
}

#[test]
fn a_cycle_yields_both_a_plan_and_a_finding() {
    let case = corpus()
        .into_iter()
        .find(|c| c.kind == kind::CYCLE)
        .expect("cycle case");
    let out = link(&case.defs, &case.root);
    assert_eq!(out.findings.len(), 1);
    assert_eq!(out.findings[0].kind, kind::CYCLE);
    assert!(
        !out.value.steps.is_empty(),
        "a drawn cycle is judged, not refused"
    );
}
