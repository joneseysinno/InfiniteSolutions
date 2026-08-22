//! [`place`] — the first half of the function this layer exists for.

use crate::core::addr::Addr;
use crate::core::arrange::arrange;
use crate::core::detail::detail;
use crate::core::placed::Placed;
use crate::core::placement::Placement;
use crate::core::point::Point;
use crate::core::rect::Rect;
use crate::core::scene_set::SceneSet;
use crate::core::transform::Transform;
use crate::core::view::View;

/// Places a scene for a view.
///
/// Pure: no I/O, no clock, no store, no surface. Both arguments are shared references
/// and there is no `&mut` anywhere in the signature, so *derived state never writes
/// back into the definition it derives from* (R5) is enforced by the compiler.
pub fn place(scene: &SceneSet, view: &View) -> Placement {
    let mut out = Placement {
        placed: Vec::new(),
        spaces: Default::default(),
        through: scene.at(),
        precision_floor: None,
    };
    let floor = surface_floor(view);
    let embedding = view.embedding();
    let roots: Vec<Addr> = scene
        .iter()
        .filter(|item| !scene.iter().any(|other| other.at.contains(&item.at) && other.at != item.at))
        .map(|item| item.at.clone())
        .collect();
    place_group(scene, view, &roots, embedding, None, floor, true, &mut out);
    out
}

fn surface_floor(view: &View) -> u32 {
    let px = view.surface.size.x.max(view.surface.size.y) * view.surface.scale_factor;
    if !(px > 1.0) {
        return 0;
    }
    px.log2().floor() as u32
}

fn place_group(
    scene: &SceneSet,
    view: &View,
    addrs: &[Addr],
    parent_to_surface: Transform,
    clip: Option<Rect>,
    floor: u32,
    stack_at_origin: bool,
    out: &mut Placement,
) {
    if addrs.is_empty() {
        return;
    }
    let items: Vec<_> = addrs.iter().filter_map(|a| scene.get(a)).collect();
    let across: Vec<_> = items.iter().map(|i| i.across).collect();
    let widths = if stack_at_origin {
        items.iter().map(|i| i.across.ideal.max(i.across.min)).collect()
    } else {
        arrange(&across, 1.0)
    };
    let mut x = 0.0;
    for (i, item) in items.iter().enumerate() {
        let w = widths[i].max(1e-12);
        let h = item.down.ideal.max(item.down.min).max(1e-12);
        let origin_x = if stack_at_origin { 0.0 } else { x };
        let local = Rect::new(Point::new(origin_x, 0.0), Point::new(origin_x + w, h));
        let local_to_parent = Transform::new(1.0, Point::new(origin_x, 0.0));
        let local_to_surface = local_to_parent.then(&parent_to_surface);
        let rect = parent_to_surface.apply_rect(&local);
        let showing = match &clip {
            Some(c) => rect.intersect(c),
            None => rect,
        };
        let level = detail(view.camera.zoom, item.detail_override, floor, None);
        if showing.is_empty() || !overlaps_surface(&showing, view) {
            x += w;
            continue;
        }
        if parent_to_surface.scale * w.min(h) * view.surface.scale_factor < 1.0
            && out.precision_floor.is_none()
        {
            out.precision_floor = Some(item.at.clone());
        }
        out.spaces.insert(item.at.clone(), local_to_surface);
        out.placed.push(Placed {
            at: item.at.clone(),
            rect,
            level,
            clip,
            accepts: item.accepts,
        });
        if item.hosts_space && level > item.at.prefix_bits() {
            let kids = direct_children(scene, &item.at);
            if !kids.is_empty() {
                place_group(
                    scene,
                    view,
                    &kids,
                    local_to_surface,
                    Some(showing),
                    floor,
                    false,
                    out,
                );
            }
        }
        x += w;
    }
}

fn overlaps_surface(showing: &Rect, view: &View) -> bool {
    let surface = view.surface.rect().inflate(view.margin);
    !showing.intersect(&surface).is_empty()
}

fn direct_children(scene: &SceneSet, parent: &Addr) -> Vec<Addr> {
    scene
        .subtree(parent)
        .filter(|item| item.at != *parent)
        .filter(|item| {
            !scene.iter().any(|mid| {
                mid.at != *parent
                    && mid.at != item.at
                    && parent.contains(&mid.at)
                    && mid.at.contains(&item.at)
            })
        })
        .map(|item| item.at.clone())
        .collect()
}
