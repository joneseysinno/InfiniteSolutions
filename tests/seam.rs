//! E1 — the three ports agree.
//!
//! Write one space through the facade; read it back through `StoreRead`, through
//! `Definitions`, and through `Scene`; assert all three resolve the same address
//! at the same revision. Assert every conversion in `facade/addr.rs` is an unwrap.
//! Assert `StoreWrite::submit` returns `Full` rather than blocking when the queue
//! is saturated.

use infinite_compositor::binding::ports::Definitions;
use infinite_db::OpenOptions;
use infinite_presenter::binding::ports::Scene;
use infinite_runtime::binding::ports::{StoreRead, StoreWrite, Submission};
use infinite_solutions::facade::{
    compositor_addr, open, open_with_options, presenter_addr, presenter_revision, runtime_addr,
    runtime_revision,
};

fn successor(bytes: &[u8]) -> Vec<u8> {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(bytes);
    let n = u32::from_be_bytes(buf).saturating_add(1);
    n.to_be_bytes().to_vec()
}

#[test]
fn conversions_are_unwraps() {
    let bytes = &[0x01u8, 0x02, 0x03, 0x04];
    assert_eq!(runtime_addr(bytes).as_bytes(), bytes);
    assert_eq!(compositor_addr(bytes).as_bytes(), bytes);
    assert_eq!(presenter_addr(bytes).as_bytes(), bytes);
    assert_eq!(runtime_revision(7).get(), 7);
    assert_eq!(presenter_revision(7).get(), 7);
}

#[test]
fn one_space_is_readable_through_three_ports() {
    let dir = tempfile::TempDir::new().unwrap();
    let store = open(dir.path()).expect("open store");
    let origin = runtime_addr(&[0, 0, 0, 1]);
    let payload = b"space".to_vec();

    {
        let mut write = store.store_write();
        assert_eq!(
            write.submit(&origin, &payload),
            Submission::Accepted,
            "first submit must be taken"
        );
    }
    store.sync().expect("sync");

    let read = store.store_read();
    let head = read.head();
    let end = runtime_addr(&successor(origin.as_bytes()));
    let records = read.range(&origin, &end, head);
    assert_eq!(records.len(), 1, "StoreRead must see the written space");
    assert_eq!(records[0].0.as_bytes(), origin.as_bytes());
    assert_eq!(records[0].1, payload);

    let defs = store.definitions();
    let set = defs.resolve(&compositor_addr(origin.as_bytes()));
    assert!(
        set.block(&compositor_addr(origin.as_bytes())).is_some(),
        "Definitions must resolve the same address"
    );

    let scene = store.scene();
    let placed = scene.placed_in(
        &presenter_addr(origin.as_bytes()),
        &presenter_addr(&successor(origin.as_bytes())),
        presenter_revision(head.get()),
    );
    assert_eq!(placed.at().get(), head.get(), "Scene is at the same revision");
    assert!(
        placed
            .get(&presenter_addr(origin.as_bytes()))
            .is_some(),
        "Scene must place the same address"
    );
}

#[test]
fn submit_returns_full_when_queue_is_saturated() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut options = OpenOptions::default();
    options.io_thread.write_queue_capacity = 2;
    let store = open_with_options(dir.path(), options).expect("open store");
    let origin = runtime_addr(&[0, 0, 0, 2]);

    store.pause_write_drain(true);
    let mut write = store.store_write();
    let mut saw_full = false;
    for i in 0..8 {
        match write.submit(&origin, &[i]) {
            Submission::Accepted => {}
            Submission::Full => {
                saw_full = true;
                break;
            }
        }
    }
    drop(write);
    store.pause_write_drain(false);

    assert!(
        saw_full,
        "StoreWrite::submit must return Full rather than blocking when the queue is saturated"
    );
}
