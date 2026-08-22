//! §5.2 — **the green check for R12 and D25.** One harness, every artifact.
//!
//! The point of these tests is what is *absent*: no artifact-specific assertion
//! anywhere. `audit` drops and rebuilds every registered artifact and compares bytes
//! without knowing what any of them is, so an artifact registered next year by a layer
//! that does not exist yet is covered the day it is registered.

#![cfg(feature = "binding")]

#[path = "fake_store.rs"]
mod fake_store;

use std::cell::Cell;
use std::rc::Rc;

use fake_store::FakeStore;
use infinite_runtime::binding::{ArtifactRegistry, Driver};
use infinite_runtime::core::{Addr, Budget, Instant, Revision};

fn addr(bytes: &[u8]) -> Addr {
    Addr::new(bytes.to_vec())
}

fn seeded() -> FakeStore {
    let store = FakeStore::new(16);
    store.seed(addr(&[0x10, 0x01]), b"one".to_vec());
    store.seed(addr(&[0x10, 0x02]), b"two".to_vec());
    store.seed(addr(&[0x90, 0x01]), b"elsewhere".to_vec());
    store
}

/// A pure artifact: concatenates whatever is in its input range.
fn register_pure(registry: &mut ArtifactRegistry, key: &str, start: Addr, end: Addr) {
    let (s, e) = (start.clone(), end.clone());
    registry.register(key, vec![(start, end)], move |store| {
        store
            .range(&s, &e, Revision::ZERO)
            .into_iter()
            .flat_map(|(_, payload)| payload)
            .collect()
    });
}

#[test]
fn a_pure_artifact_passes_the_discard_test() {
    let store = seeded();
    let mut registry = ArtifactRegistry::new();
    register_pure(&mut registry, "range/10", addr(&[0x10]), addr(&[0x11]));

    assert!(registry.passes_discard_test("range/10", &store, Revision::ZERO));
    assert_eq!(
        registry.get("range/10").unwrap().bytes().unwrap(),
        b"onetwo"
    );
}

#[test]
fn the_audit_covers_every_artifact_with_no_per_artifact_code() {
    let store = seeded();
    let mut registry = ArtifactRegistry::new();
    register_pure(&mut registry, "range/10", addr(&[0x10]), addr(&[0x11]));
    register_pure(&mut registry, "range/90", addr(&[0x90]), addr(&[0x91]));
    register_pure(&mut registry, "range/all", addr(&[0x00]), addr(&[0xFF]));

    assert_eq!(registry.keys().count(), 3);
    assert!(
        registry.audit(&store, Revision::ZERO).is_empty(),
        "a registered artifact failed the discard test"
    );
}

#[test]
fn an_artifact_that_carries_state_between_rebuilds_is_caught() {
    // A cache that is written to (F-7) looks exactly like this: the rebuild is not a
    // pure function of what it reads. R12 exists to catch it, and the harness does,
    // without being told what this artifact is.
    let store = seeded();
    let mut registry = ArtifactRegistry::new();
    let counter = Rc::new(Cell::new(0u8));
    let smuggled = Rc::clone(&counter);
    registry.register("impure", vec![(addr(&[0x00]), addr(&[0xFF]))], move |_| {
        smuggled.set(smuggled.get() + 1);
        vec![smuggled.get()]
    });

    assert!(
        !registry.passes_discard_test("impure", &store, Revision::ZERO),
        "the harness failed to notice an artifact carrying state between rebuilds"
    );
    assert_eq!(
        registry.audit(&store, Revision::ZERO),
        vec!["impure".to_string()]
    );
}

#[test]
fn staleness_invalidates_only_artifacts_deriving_from_the_changed_address() {
    // S6's green check in miniature: an input change yields *exactly* the downstream
    // set. B2 is the failure this prevents — one keystroke redrawing the page.
    let store = seeded();
    let mut writer = store.handle();
    let mut driver = Driver::new(8);

    register_pure(driver.artifacts(), "range/10", addr(&[0x10]), addr(&[0x11]));
    register_pure(driver.artifacts(), "range/90", addr(&[0x90]), addr(&[0x91]));

    driver
        .artifacts()
        .rebuild("range/10", &store, Revision::ZERO);
    driver
        .artifacts()
        .rebuild("range/90", &store, Revision::ZERO);
    assert!(!driver.artifacts().is_stale("range/10"));
    assert!(!driver.artifacts().is_stale("range/90"));

    store.seed(addr(&[0x10, 0x03]), b"three".to_vec());
    store.go_stale(addr(&[0x10, 0x03]));

    driver.tick(Instant::ZERO, Budget::units(8), &store, &mut writer, &store);

    assert_eq!(
        driver.artifacts().get("range/10").unwrap().bytes().unwrap(),
        b"onetwothree",
        "the artifact deriving from the changed address was not rebuilt"
    );
    assert!(
        !driver.artifacts().is_stale("range/90"),
        "an unrelated artifact was invalidated — this is B2"
    );
}
