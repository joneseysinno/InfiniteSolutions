//! E9 — tier 0 registers by passing the equivalence harness on the editor's plan.
//!
//! No per-backend test code: the harness is [`infinite_compositor::binding::check`].
//! The compiled artifact is derived state and passes R12's generic discard harness.
//!
//! The behaviour composition contains store effects (`read` / `amend` / `commit`).
//! D19's bit-for-bit law is for a pure function of declared inputs, so the corpus
//! draws the editor's *pure* steps — `offset` then `displace` — from the linked plan.

use infinite_compositor::binding::ports::{Backends as BackendsPort, Values as ValuesPort};
use infinite_compositor::binding::{check, TIER0_KEY};
use infinite_compositor::core::{Plan, Tag, Value};
use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade;

fn point(x: f64, y: f64) -> Vec<u8> {
    let mut p = Vec::with_capacity(16);
    p.extend_from_slice(&x.to_le_bytes());
    p.extend_from_slice(&y.to_le_bytes());
    p
}

fn drag(store: &facade::Store) {
    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::node_a_key())
        .or_else(|| placement.placed.iter().find(|p| p.accepts))
        .expect("a genesis space");
    let mid_x = (target.rect.min.x + target.rect.max.x) * 0.5;
    let mid_y = (target.rect.min.y + target.rect.max.y) * 0.5;
    store.amend(
        addresses::drag_from_key(),
        &point(mid_x - 10.0, mid_y),
    );
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(mid_x, mid_y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
}

#[test]
fn tier0_registers_by_passing_the_editors_plan() {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
    drag(&store);

    let linked = store.link_at(addresses::BEHAVIOUR_ROOT_KEY);
    assert!(
        !linked.has_findings(),
        "the editor's plan must link: {:?}",
        linked.findings
    );
    let plan = linked.value;
    let blocks = store.blocks();
    let backends = store.backends();
    let backend = BackendsPort::backend(&backends, TIER0_KEY).expect("tier 0 is registered");

    assert!(backend.accepts(&plan), "tier 0 accepts the editor's plan");
    assert!(
        backend.compile(&plan).is_some(),
        "tier 0 compiles the editor's plan"
    );
    assert!(
        store.artifact_passes_discard(TIER0_KEY),
        "the compiled artifact passes R12 with no per-artifact test code"
    );

    let corpus: Plan = Plan {
        steps: plan
            .steps
            .iter()
            .filter(|s| s.key.as_ref() == "offset" || s.key.as_ref() == "displace")
            .cloned()
            .collect(),
    };
    assert_eq!(corpus.steps.len(), 2);

    let values = store.values();
    let mut seed = Vec::new();
    for step in &corpus.steps {
        for input in &step.inputs {
            let at = input.slot();
            if seed.iter().any(|(a, _)| a == &at) {
                continue;
            }
            if let Some(value) = ValuesPort::read(&values, &at) {
                seed.push((at, value));
            } else if let Some(bytes) = store.stored_at(addresses::node_a_key()) {
                seed.push((at, Value::new(Tag::new("value"), bytes)));
            }
        }
    }

    assert!(
        check(backend, &corpus, &blocks, &seed),
        "tier 0 must match interpret on the editor's offset/displace plan"
    );
    assert!(BackendsPort::backend(&backends, "native").is_none());
}
