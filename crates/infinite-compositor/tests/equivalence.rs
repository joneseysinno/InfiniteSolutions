//! The equivalence harness (S7) — and the backend registration procedure (D28).
//!
//! This file is not only a test. **A backend is registered by passing it.**
//!
//! D19: *the interpreted execution is the specification; a compiled block must be
//! observationally identical to it.* For every plan in a maintained corpus: run
//! interpreted, run compiled, compare outputs bit-for-bit and provenance
//! edge-for-edge. Exact rather than statistical.
//!
//! **Green check:** tier 0 registers by passing this harness over the corpus, with no
//! per-backend test code.
//!
//! The corpus is drawn from the editor's plan *shapes*: a native, a chain (drag is
//! offset then displace), and a fan-out (probe hits several sinks). The editor's
//! own linked plan is the other half of the corpus (`tests/tier0.rs` at the root).

#![cfg(feature = "binding")]

#[path = "fakes.rs"]
mod fakes;

use fakes::FakeBlocks;
use infinite_compositor::binding::ports::Primitive;
use infinite_compositor::binding::{check, TIER0_KEY};
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

fn concat_sig() -> Signature {
    Signature {
        ports: vec![
            Port::new("a", Direction::In, Tag::new("blob")),
            Port::new("b", Direction::In, Tag::new("blob")),
            Port::new("out", Direction::Out, Tag::new("blob")),
        ],
    }
}

fn blob(bytes: &[u8]) -> Value {
    Value::new(Tag::new("blob"), bytes.to_vec())
}

fn corpus(blocks: &FakeBlocks) -> Vec<(Plan, Vec<(Addr, Value)>)> {
    let native = {
        let src_a = port(1, "a");
        let src_b = port(1, "b");
        let out = port(1, "out");
        (
            Plan {
                steps: vec![Step {
                    block: a(1),
                    key: "concat".into(),
                    inputs: vec![src_a.clone(), src_b.clone()],
                    outputs: vec![out],
                    inner: None,
                }],
            },
            vec![(src_a.slot(), blob(b"one")), (src_b.slot(), blob(b"two"))],
        )
    };

    let chain = {
        let a1 = port(1, "a");
        let b1 = port(1, "b");
        let out1 = port(1, "out");
        let b2 = port(2, "b");
        let out2 = port(2, "out");
        (
            Plan {
                steps: vec![
                    Step {
                        block: a(1),
                        key: "concat".into(),
                        inputs: vec![a1.clone(), b1.clone()],
                        outputs: vec![out1.clone()],
                        inner: None,
                    },
                    Step {
                        block: a(2),
                        key: "concat".into(),
                        inputs: vec![out1, b2.clone()],
                        outputs: vec![out2],
                        inner: None,
                    },
                ],
            },
            vec![
                (a1.slot(), blob(b"x")),
                (b1.slot(), blob(b"y")),
                (b2.slot(), blob(b"z")),
            ],
        )
    };

    let fan_out = {
        let a1 = port(1, "a");
        let b1 = port(1, "b");
        let out1 = port(1, "out");
        let b2 = port(2, "b");
        let out2 = port(2, "out");
        let b3 = port(3, "b");
        let out3 = port(3, "out");
        (
            Plan {
                steps: vec![
                    Step {
                        block: a(1),
                        key: "concat".into(),
                        inputs: vec![a1.clone(), b1.clone()],
                        outputs: vec![out1.clone()],
                        inner: None,
                    },
                    Step {
                        block: a(2),
                        key: "concat".into(),
                        inputs: vec![out1.clone(), b2.clone()],
                        outputs: vec![out2],
                        inner: None,
                    },
                    Step {
                        block: a(3),
                        key: "concat".into(),
                        inputs: vec![out1, b3.clone()],
                        outputs: vec![out3],
                        inner: None,
                    },
                ],
            },
            vec![
                (a1.slot(), blob(b"p")),
                (b1.slot(), blob(b"q")),
                (b2.slot(), blob(b"r")),
                (b3.slot(), blob(b"s")),
            ],
        )
    };

    let _ = blocks;
    vec![native, chain, fan_out]
}

#[test]
fn a_backend_is_registered_by_passing_this() {
    let mut blocks = FakeBlocks::new();
    blocks.register("concat", concat_sig(), Box::new(Concat));
    let backend = blocks.tier0();
    for (plan, seed) in corpus(&blocks) {
        assert!(
            check(&backend, &plan, &blocks, &seed),
            "tier 0 must match interpret on every corpus plan"
        );
    }
    assert_eq!(TIER0_KEY, "tier0");
}
