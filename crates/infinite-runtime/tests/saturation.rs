//! §7.4 — **the green check for D24.** Store backpressure must not reach the input
//! path.
//!
//! Verification method (R23): an automated test against a fake whose write queue can be
//! saturated on demand. Budget is asserted in *work units* rather than wall time —
//! wall time is not reproducible and would make this a flaky test rather than a proof.
//! The claim being verified is "a full queue costs no keystroke and no tick overrun",
//! and work units are exactly what that claim is about.

#![cfg(feature = "binding")]

#[path = "fake_store.rs"]
mod fake_store;

use fake_store::{FakeJournal, FakeStore};
use infinite_runtime::binding::Driver;
use infinite_runtime::core::{Addr, Budget, Instant};

const TICKS: u64 = 600; // 60 Hz for 10 seconds.
const BUDGET_UNITS: u32 = 4;
const PENDING_CAPACITY: usize = 64;
const COMMIT_EVERY: u64 = 10;

fn field() -> Addr {
    Addr::new(vec![0x10, 0x20])
}

#[test]
fn a_full_write_queue_costs_no_keystroke_and_no_tick_overrun() {
    let store = FakeStore::new(2);
    let mut writer = store.handle();
    let mut journal = FakeJournal::default();
    let mut driver = Driver::new(PENDING_CAPACITY);

    store.saturate();
    assert!(store.is_saturated(), "precondition: the queue is full");

    let mut open = None;
    let mut last_payload: Vec<u8>;
    let mut high_water = 0usize;

    for n in 0..TICKS {
        // --- the input path -------------------------------------------------
        let payload = format!("keystroke-{n}").into_bytes();
        let seq = match open {
            Some(seq) => {
                assert!(
                    driver.pending().amend(seq, payload.clone()),
                    "tick {n}: a keystroke was refused while the queue was full"
                );
                seq
            }
            None => driver
                .pending()
                .open(field(), payload.clone())
                .expect("tick {n}: pending set overflowed under backpressure"),
        };
        driver.journal(seq, &mut journal);
        last_payload = payload;

        assert!(
            driver
                .pending()
                .list()
                .any(|e| e.seq() == seq && e.payload() == last_payload),
            "tick {n}: the keystroke did not reach the pending set within one tick"
        );

        open = Some(seq);
        if n % COMMIT_EVERY == COMMIT_EVERY - 1 {
            driver.pending().commit(seq);
            open = None;
        }

        // --- the runtime ----------------------------------------------------
        let outcome = driver.tick(
            Instant::from_nanos(n),
            Budget::units(BUDGET_UNITS),
            &store,
            &mut writer,
            &store,
        );

        assert_eq!(
            outcome.submitted, 0,
            "tick {n}: the full queue accepted a write"
        );
        assert!(
            outcome.rebuilt + outcome.submitted <= BUDGET_UNITS,
            "tick {n}: the tick exceeded its budget"
        );

        high_water = high_water.max(driver.pending().len());
        assert!(
            driver.pending().len() <= PENDING_CAPACITY,
            "tick {n}: the pending set exceeded its bound"
        );
    }

    // Coalescing is what keeps the bound from mattering: 600 keystrokes and 60
    // commits against one address, under a queue that never accepted anything, is a
    // handful of entries — not 600. (D24.3)
    assert!(
        high_water <= 3,
        "pending high-water mark was {high_water}; coalescing is not holding"
    );

    // --- the queue drains ---------------------------------------------------
    store.drain();
    for n in 0..8 {
        driver.tick(
            Instant::from_nanos(TICKS + n),
            Budget::units(BUDGET_UNITS),
            &store,
            &mut writer,
            &store,
        );
        store.drain();
    }

    assert!(
        driver.pending().committed().count() == 0,
        "committed entries did not reach the store after the queue drained"
    );
    let final_value = store.get(&field()).expect("the field was never written");
    let expected = format!(
        "keystroke-{}",
        TICKS - 1 - (TICKS - 1) % COMMIT_EVERY + COMMIT_EVERY - 1
    );
    assert_eq!(
        final_value,
        expected.into_bytes(),
        "the store's final value is not the last committed input"
    );
}

#[test]
fn nothing_pending_is_ever_dropped_silently() {
    let store = FakeStore::new(1);
    let mut writer = store.handle();
    let mut driver = Driver::new(2);

    store.saturate();

    let a = driver
        .pending()
        .open(Addr::new(vec![1]), b"a".to_vec())
        .unwrap();
    let b = driver
        .pending()
        .open(Addr::new(vec![2]), b"b".to_vec())
        .unwrap();
    driver.pending().commit(a);
    driver.pending().commit(b);

    let overflow = driver
        .pending()
        .open(Addr::new(vec![3]), b"c".to_vec())
        .unwrap_err();
    assert_eq!(overflow.oldest, a, "overflow must name the oldest entry");
    assert_eq!(driver.pending().len(), 2, "overflow dropped an entry");

    driver.tick(Instant::ZERO, Budget::units(4), &store, &mut writer, &store);
    assert_eq!(driver.pending().len(), 2, "a refused submit lost an entry");
}
