//! E2 — saturation against the real store, and journal replay after a crash.
//!
//! `RUNTIME.md` §7.4 specifies the test against `FakeStore`. This is the same
//! test with the real queue behind it (D33), plus D8's crash-replay check.

use infinite_db::OpenOptions;
use infinite_runtime::binding::ports::{StoreWrite, Submission};
use infinite_solutions::editor::addresses;
use infinite_solutions::facade::{open, open_with_options, runtime_addr};

const TICKS: u64 = 600;
const BUDGET_UNITS: u32 = 4;
const COMMIT_EVERY: u64 = 10;

fn field() -> &'static [u8] {
    &[0x10, 0x20]
}

#[test]
fn a_full_write_queue_costs_no_keystroke_and_no_tick_overrun() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut options = OpenOptions::default();
    options.io_thread.write_queue_capacity = 2;
    let store = open_with_options(dir.path(), options).expect("open store");

    store.pause_write_drain(true);
    {
        let mut write = store.store_write();
        let filler = runtime_addr(&[0xFF, 0xFF]);
        loop {
            if write.submit(&filler, &[]) == Submission::Full {
                break;
            }
        }
    }

    let mut high_water = 0usize;

    for n in 0..TICKS {
        let payload = format!("keystroke-{n}").into_bytes();
        assert!(
            store.amend(field(), &payload),
            "tick {n}: a keystroke was refused while the queue was full"
        );
        assert_eq!(
            store.pending_at(field()).as_deref(),
            Some(payload.as_slice()),
            "tick {n}: the keystroke did not reach the pending set within one tick"
        );

        if n % COMMIT_EVERY == COMMIT_EVERY - 1 {
            assert!(store.commit_at(field()), "tick {n}: commit refused");
        }

        let report = store.tick_at(n, BUDGET_UNITS);
        assert_eq!(
            report.submitted, 0,
            "tick {n}: the full queue accepted a write"
        );
        assert!(
            !report.budget_exhausted || report.submitted + report.refused <= BUDGET_UNITS,
            "tick {n}: the tick exceeded its budget"
        );
        high_water = high_water.max(store.pending_len());
        assert!(
            store.pending_len() <= 1024,
            "tick {n}: the pending set exceeded its bound"
        );
    }

    assert!(
        high_water <= 3,
        "pending high-water mark was {high_water}; coalescing is not holding"
    );

    store.pause_write_drain(false);
    for n in 0..32 {
        store.tick_at(TICKS + n, BUDGET_UNITS);
        let _ = store.sync();
        if store.committed_len() == 0 {
            break;
        }
    }
    store.sync().expect("drain sync");

    assert_eq!(
        store.committed_len(),
        0,
        "committed entries did not reach the store after the queue drained"
    );
    let expected = format!(
        "keystroke-{}",
        TICKS - 1 - (TICKS - 1) % COMMIT_EVERY + COMMIT_EVERY - 1
    );
    assert_eq!(
        store.stored_at(field()).expect("the field was never written"),
        expected.into_bytes(),
        "the store's final value is not the last committed input"
    );
}

#[test]
fn a_crash_mid_drag_replays_the_pending_set_before_the_first_tick() {
    let dir = tempfile::TempDir::new().unwrap();
    let origin = addresses::POINTER_POSITION.as_bytes();
    {
        let store = open(dir.path()).expect("open store");
        assert!(store.amend(origin, b"drag-1"));
        assert!(store.amend(origin, b"drag-2"));
        store.flush_journal().expect("flush journal");
        assert_eq!(store.pending_at(origin).as_deref(), Some(&b"drag-2"[..]));
    }

    let store = open(dir.path()).expect("reopen store");
    assert_eq!(
        store.pending_at(origin).as_deref(),
        Some(&b"drag-2"[..]),
        "journal replay must restore the pending set before the first tick"
    );
    let _ = store.tick();
    assert_eq!(
        store.pending_at(origin).as_deref(),
        Some(&b"drag-2"[..]),
        "the first tick must not drop a restored drag"
    );
}
