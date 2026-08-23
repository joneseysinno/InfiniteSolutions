//! E10.5 — pan and zoom are authored, and they survive a restart.
//!
//! The camera is a well-known record (D5, E10.5), not a field on `Inner`: amending
//! it makes the new value visible immediately through the same stored ∪ pending
//! overlay `Definitions` and `Scene` already use for everything else, and it
//! replays from the journal exactly as `saturation.rs`'s crash-mid-drag case
//! proves for `POINTER_POSITION`.

use infinite_solutions::facade::open;

#[test]
fn pan_and_zoom_are_visible_before_any_commit() {
    let dir = tempfile::TempDir::new().unwrap();
    let store = open(dir.path()).expect("open store");

    let start = store.camera();
    store.pan_by(40.0, 20.0);
    let panned = store.camera();
    assert_ne!(panned.centre, start.centre, "pan_by must move the camera");
    assert_eq!(panned.zoom, start.zoom, "pan_by must not change zoom");

    store.zoom_by(1.0);
    let zoomed = store.camera();
    assert!(
        zoomed.zoom > panned.zoom,
        "zoom_by must change magnification"
    );
    assert_eq!(
        zoomed.centre, panned.centre,
        "zoom_by must not move the centre"
    );
}

#[test]
fn a_crash_after_pan_and_zoom_replays_the_camera_before_the_first_tick() {
    let dir = tempfile::TempDir::new().unwrap();
    let moved;
    {
        let store = open(dir.path()).expect("open store");
        store.pan_by(40.0, 20.0);
        store.zoom_by(2.0);
        moved = store.camera();
        store.flush_journal().expect("flush journal");
    }

    let store = open(dir.path()).expect("reopen store");
    assert_eq!(
        store.camera(),
        moved,
        "journal replay must restore the camera before the first tick; it is a \
         record like a dragged node (E7), not a field that was never written"
    );
    let _ = store.tick();
    assert_eq!(
        store.camera(),
        moved,
        "the first tick must not drop a restored camera"
    );
}
