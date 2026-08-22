//! The closure test (S5) — this layer's equivalent of R12's discard test.

use infinite_compositor::core::{
    link, signature_of, Addr, Block, Body, BodyKind, Composition, DefinitionSet, Direction,
    Port, Signature, Tag, Wire,
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

#[test]
fn a_wrapped_composition_plans_identically() {
    let key = b"ok".as_ref();
    let mut defs = DefinitionSet::default();
    defs.blocks.insert(
        Addr::new(key.to_vec()),
        native(
            key,
            vec![
                Port::new("in", Direction::In, Tag::new("point")),
                Port::new("out", Direction::Out, Tag::new("point")),
            ],
        ),
    );

    let mut open_in = Port::new("in", Direction::In, Tag::new("point"));
    open_in.required = false;
    let mut open_out = Port::new("out", Direction::Out, Tag::new("point"));
    open_out.required = false;
    let mut blocks = std::collections::BTreeMap::new();
    blocks.insert(
        a(1),
        native(
            key,
            vec![
                open_in,
                Port::new("out", Direction::Out, Tag::new("point")),
            ],
        ),
    );
    blocks.insert(
        a(2),
        native(
            key,
            vec![
                Port::new("in", Direction::In, Tag::new("point")),
                open_out,
            ],
        ),
    );
    let c = Composition {
        blocks,
        wires: vec![Wire {
            sources: vec![infinite_compositor::core::PortRef {
                block: a(1),
                port: "out".into(),
            }],
            sinks: vec![infinite_compositor::core::PortRef {
                block: a(2),
                port: "in".into(),
            }],
        }],
        compilable: false,
    };
    let root_c = a(10);
    defs.compositions.insert(root_c.clone(), c.clone());

    let plan_c = link(&defs, &root_c);
    assert!(
        !plan_c.has_findings(),
        "C must link cleanly: {:?}",
        plan_c.findings
    );
    let sig = signature_of(&c, &defs).value;

    let wrapper = a(20);
    let b = Block {
        signature: sig,
        body: Body {
            kind: BodyKind::new(BodyKind::COMPOSED),
            target: root_c.clone(),
        },
    };
    let mut inner = std::collections::BTreeMap::new();
    inner.insert(wrapper.clone(), b);
    let c_prime = Composition {
        blocks: inner,
        wires: vec![],
        compilable: false,
    };
    let root_p = a(30);
    defs.compositions.insert(root_p.clone(), c_prime);
    defs.blocks.insert(
        wrapper,
        Block {
            signature: signature_of(&c, &defs).value,
            body: Body {
                kind: BodyKind::new(BodyKind::COMPOSED),
                target: root_c,
            },
        },
    );

    let plan_p = link(&defs, &root_p);
    assert!(
        !plan_p.has_findings(),
        "C' must link cleanly: {:?}",
        plan_p.findings
    );
    assert_eq!(
        plan_c.value, plan_p.value,
        "wrapping a composition is identity"
    );
}
