//! E14.1 — hysteresis reaches the running path through a real [`place`].
//!
//! Finding 20: `detail`'s dead band existed and was unit-tested, but both call
//! sites in `place.rs` passed `previous: None`. This test is the check that can
//! fail for that reason (D41): place twice across a level boundary and assert
//! the level does not change twice inside one dead band.

use infinite_presenter::core::{
    Addr, Camera, Extent, Placeable, Point, Revision, SceneSet, SurfaceRect, View, detail, place,
};

fn one_space() -> SceneSet {
    let mut scene = SceneSet::new(Revision::new(1));
    scene.insert(Placeable {
        at: Addr::new(vec![0x01]),
        across: Extent::fixed(1.0),
        down: Extent::fixed(1.0),
        position: Point::ORIGIN,
        style: "plain".into(),
        detail_override: None,
        primitive: infinite_presenter::core::AREA.into(),
        link: None,
        hosts_space: false,
        accepts: true,
        text: "".into(),
    });
    scene
}

fn view_at(zoom: f64) -> View {
    View::new(
        Camera::new(Point::new(0.5, 0.5), zoom),
        SurfaceRect::new(Point::ORIGIN, Point::new(800.0, 600.0), 1.0),
        0.0,
    )
}

fn level_of(placement: &infinite_presenter::core::Placement) -> u32 {
    placement
        .placed
        .iter()
        .find(|p| p.at == Addr::new(vec![0x01]))
        .expect("the space is placed")
        .level
}

#[test]
fn place_twice_across_a_boundary_holds_inside_the_dead_band() {
    let scene = one_space();

    // Level 1 with no prior: zoom in [2, 4).
    let view1 = view_at(2.0);
    let p1 = place(&scene, &view1, None);
    let level1 = level_of(&p1);
    assert_eq!(level1, 1, "cold start at zoom 2.0 is level 1");

    // Just inside the promote dead band toward level 2: log = 2.1, naive = 2,
    // but with prior = 1 the slop holds at 1 (detail unit test's just_inside).
    let inside = 2f64.powf(2.0 + 0.1);
    let view2 = view_at(inside);
    let p2 = place(&scene, &view2, Some(&p1));
    assert_eq!(
        level_of(&p2),
        level1,
        "with prior, the level must not promote inside the dead band"
    );

    // The same zoom with no prior promotes — proving None was the defect.
    let cold = place(&scene, &view2, None);
    assert_eq!(
        level_of(&cold),
        2,
        "without prior, naive level 2 must win — the bug that must stop passing"
    );
    assert_eq!(
        detail(inside, None, 32, Some(1)),
        1,
        "detail itself still holds; place must be wiring it"
    );
}
