//! The self-sufficiency test (S6).
//!
//! `probe` is called with **no port in scope at all**.

use infinite_presenter::core::{
    Addr, Camera, Extent, Placeable, Point, Revision, SceneSet, SurfaceRect, View, place, probe,
};

fn thing(bytes: Vec<u8>, hosts: bool, accepts: bool, override_steps: Option<i64>) -> Placeable {
    Placeable {
        at: Addr::new(bytes),
        across: Extent::fixed(1.0),
        down: Extent::fixed(1.0),
        style: "plain".into(),
        detail_override: override_steps,
        hosts_space: hosts,
        accepts,
    }
}

fn view() -> View {
    View::new(
        Camera::new(Point::new(0.5, 0.5), 1000.0),
        SurfaceRect::new(Point::new(37.0, 11.0), Point::new(800.0, 600.0), 1.0),
        0.0,
    )
}

fn centred(placement: &infinite_presenter::core::Placement, addr: &Addr) -> Point {
    let p = placement
        .placed
        .iter()
        .find(|p| p.at == *addr)
        .expect("address was not placed");
    Point::new(
        (p.rect.min.x + p.rect.max.x) * 0.5,
        (p.rect.min.y + p.rect.max.y) * 0.5,
    )
}

#[test]
fn a_point_answers_without_a_port() {
    let v = view();

    let mut overlap = SceneSet::new(Revision::new(1));
    overlap.insert(thing(vec![0x01], false, true, None));
    overlap.insert(thing(vec![0x02], false, true, None));
    let placed = place(&overlap, &v);
    let at = centred(&placed, &Addr::new(vec![0x01]));
    let hit = probe(&placed, at).expect("overlap should hit");
    assert_eq!(hit.at, Addr::new(vec![0x02]), "the later sibling must win");

    let mut clipped = SceneSet::new(Revision::new(1));
    clipped.insert(thing(vec![0x10], true, true, None));
    clipped.insert(Placeable {
        at: Addr::new(vec![0x10, 0x01]),
        across: Extent::fixed(0.4),
        down: Extent::fixed(1.0),
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
    });
    clipped.insert(Placeable {
        at: Addr::new(vec![0x10, 0x02]),
        across: Extent::fixed(0.4),
        down: Extent::fixed(1.0),
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
    });
    let placed = place(&clipped, &v);
    let parent = centred(&placed, &Addr::new(vec![0x10]));
    let hit = probe(&placed, parent).expect("parent should hit");
    assert!(
        hit.at == Addr::new(vec![0x10]) || hit.at.as_bytes().starts_with(&[0x10]),
        "a point on the parent lands in the subtree"
    );

    let mut collapsed = SceneSet::new(Revision::new(1));
    collapsed.insert(thing(vec![0x20], true, true, Some(-64)));
    collapsed.insert(thing(vec![0x20, 0x01], false, true, None));
    let placed = place(&collapsed, &v);
    assert!(
        placed
            .placed
            .iter()
            .all(|p| p.at != Addr::new(vec![0x20, 0x01])),
        "a collapsed space must not show its interior"
    );
    let at = centred(&placed, &Addr::new(vec![0x20]));
    assert_eq!(
        probe(&placed, at).map(|p| p.at),
        Some(Addr::new(vec![0x20]))
    );

    let mut gutter = SceneSet::new(Revision::new(1));
    gutter.insert(thing(vec![0x30], true, true, None));
    gutter.insert(Placeable {
        at: Addr::new(vec![0x30, 0x01]),
        across: Extent::new(0.2, 0.2, 0.0),
        down: Extent::fixed(1.0),
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
    });
    gutter.insert(Placeable {
        at: Addr::new(vec![0x30, 0x02]),
        across: Extent::new(0.2, 0.2, 0.0),
        down: Extent::fixed(1.0),
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
    });
    let placed = place(&gutter, &v);
    let parent_rect = placed
        .placed
        .iter()
        .find(|p| p.at == Addr::new(vec![0x30]))
        .unwrap()
        .rect;
    let last_kid = placed
        .placed
        .iter()
        .filter(|p| p.at.as_bytes().starts_with(&[0x30]) && p.at.as_bytes().len() > 1)
        .last()
        .expect("children should be placed");
    let gap = Point::new(
        (last_kid.rect.max.x + parent_rect.max.x) * 0.5,
        centred(&placed, &Addr::new(vec![0x30])).y,
    );
    assert!(
        parent_rect.contains(gap),
        "the leftover after arranged children should still be on the parent"
    );
    assert!(
        placed
            .placed
            .iter()
            .filter(|p| p.at != Addr::new(vec![0x30]))
            .all(|p| !p.covers(gap)),
        "the leftover is a gutter, not a child"
    );
    assert_eq!(
        probe(&placed, gap).map(|p| p.at),
        Some(Addr::new(vec![0x30])),
        "a gutter belongs to the parent"
    );

    let miss = Point::new(-10_000.0, -10_000.0);
    assert!(
        probe(&placed, miss).is_none(),
        "a point outside every space answers none"
    );
}
