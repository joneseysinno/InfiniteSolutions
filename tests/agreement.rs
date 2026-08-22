//! E3 — cull agrees with draw, against a real surface.
//!
//! The agreement test (`PRESENTER.md` §6.4) against a real surface with a
//! **non-zero origin**. Clicking the space returns its address. The placement
//! passes the runtime's generic discard harness with no per-artifact test code.

use infinite_presenter::binding::ports::Surface as SurfacePort;
use infinite_presenter::core::{visible, Camera, Point, View};
use infinite_runtime::binding::ArtifactRegistry;
use infinite_runtime::binding::ports::StoreRead;
use infinite_solutions::facade::{open, register, runtime_revision};

fn seeded() -> (tempfile::TempDir, infinite_solutions::facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = open(dir.path()).expect("open store");
    store.ensure_space();
    store.set_surface(37.0, 11.0, 800.0, 600.0, 1.0);
    (dir, store)
}

#[test]
fn cull_agrees_with_draw_on_a_real_surface() {
    let (_dir, store) = seeded();
    let geom = SurfacePort::geometry(&store.surface());
    assert!(
        geom.origin.x > 0.0 && geom.origin.y > 0.0,
        "the green check is a non-zero origin"
    );
    let view = View::new(Camera::new(Point::new(0.5, 0.5), 2.0), geom, 8.0);
    let embedding = view.embedding();
    let seen = visible(&view);
    let drawn = view.surface.rect().inflate(view.margin);
    for i in 0..80 {
        let t = f64::from(i) / 80.0;
        let world = Point::new(
            seen.min.x + (seen.max.x - seen.min.x) * t,
            seen.min.y + (seen.max.y - seen.min.y) * (1.0 - t),
        );
        let screen = embedding.apply(world);
        assert_eq!(
            seen.contains(world),
            drawn.contains(screen),
            "cull and draw disagreed on a real surface at origin {:?}",
            geom.origin
        );
    }
}

#[test]
fn clicking_the_space_returns_its_address() {
    let (_dir, store) = seeded();
    let placement = store.place_now();
    let space = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == [0, 0, 0, 1])
        .expect("the seeded space must be placed");
    let mid = Point::new(
        (space.rect.min.x + space.rect.max.x) * 0.5,
        (space.rect.min.y + space.rect.max.y) * 0.5,
    );
    assert_eq!(
        store.probe_at(mid.x, mid.y).as_deref(),
        Some(&[0, 0, 0, 1][..]),
        "a click on the space returns its address"
    );
}

#[test]
fn placement_passes_the_generic_discard_harness() {
    let (_dir, store) = seeded();
    let view = View::new(
        Camera::new(Point::new(0.5, 0.5), 2.0),
        SurfacePort::geometry(&store.surface()),
        0.0,
    );
    let mut registry = ArtifactRegistry::new();
    register(&mut registry, view);
    let reader = store.store_read();
    assert!(
        registry
            .audit(&reader, runtime_revision(reader.head().get()))
            .is_empty(),
        "the generic harness failed the placement — D25 is wrong if this needs per-artifact code"
    );
}
