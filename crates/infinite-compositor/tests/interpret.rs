//! S6 — interpreted execution and provenance.

#![cfg(feature = "binding")]

#[path = "fakes.rs"]
mod fakes;

use fakes::{FakeBlocks, FakeProvenance, FakeValues};
use infinite_compositor::binding::interpret;
use infinite_compositor::binding::ports::{Primitive, Provenance};
use infinite_compositor::core::{
    Addr, Direction, Plan, Port, PortRef, Signature, Step, Tag, Value,
};

struct Concat;

impl Primitive for Concat {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let mut payload = Vec::new();
        for input in inputs {
            payload.extend_from_slice(input.payload());
        }
        vec![Value::new(Tag::new("blob"), payload)]
    }
}

fn a(n: u8) -> Addr {
    Addr::new(vec![n])
}

fn port(block: u8, name: &str) -> PortRef {
    PortRef {
        block: a(block),
        port: name.into(),
    }
}

#[test]
fn interpret_writes_outputs_and_records_the_declared_inputs() {
    let mut blocks = FakeBlocks::new();
    blocks.register(
        "concat",
        Signature {
            ports: vec![
                Port::new("a", Direction::In, Tag::new("blob")),
                Port::new("b", Direction::In, Tag::new("blob")),
                Port::new("out", Direction::Out, Tag::new("blob")),
            ],
        },
        Box::new(Concat),
    );

    let src_a = port(1, "a");
    let src_b = port(1, "b");
    let out = PortRef {
        block: a(1),
        port: "out".into(),
    };
    let plan = Plan {
        steps: vec![Step {
            block: a(1),
            key: "concat".into(),
            inputs: vec![src_a.clone(), src_b.clone()],
            outputs: vec![out.clone()],
            inner: None,
        }],
    };

    let mut values = FakeValues {
        by_addr: Default::default(),
    };
    values
        .by_addr
        .insert(src_a.slot(), Value::new(Tag::new("blob"), b"one".to_vec()));
    values
        .by_addr
        .insert(src_b.slot(), Value::new(Tag::new("blob"), b"two".to_vec()));
    let mut provenance = FakeProvenance {
        inputs: Default::default(),
    };

    let out_run = interpret(&plan, &blocks, &mut values, &mut provenance);
    assert!(!out_run.has_findings(), "{:?}", out_run.findings);
    assert_eq!(
        values.by_addr.get(&out.slot()).map(Value::payload),
        Some(b"onetwo".as_ref())
    );

    let declared = provenance.inputs_of(&out.slot());
    assert_eq!(declared, vec![src_a.slot(), src_b.slot()]);

    // The S6 green check, identical in form to RUNTIME.md S6: an input yields
    // exactly the downstream set — no more, no fewer.
    let downstream: Vec<Addr> = provenance
        .inputs
        .iter()
        .filter(|(_, ins)| ins.iter().any(|i| i == &src_a.slot()))
        .map(|(o, _)| o.clone())
        .collect();
    assert_eq!(downstream, vec![out.slot()]);
}
