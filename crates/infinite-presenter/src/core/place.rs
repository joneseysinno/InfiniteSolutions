//! [`place`] — the first half of the function this layer exists for.

use crate::core::addr::Addr;
use crate::core::arrange::arrange;
use crate::core::detail::detail;
use crate::core::placeable::Placeable;
use crate::core::placed::Placed;
use crate::core::placement::{Batch, Placement};
use crate::core::point::Point;
use crate::core::rect::Rect;
use crate::core::scene_set::SceneSet;
use crate::core::transform::Transform;
use crate::core::view::View;

/// Places a scene for a view.
///
/// Pure: no I/O, no clock, no store, no surface. Arguments are shared references
/// and there is no `&mut` anywhere in the signature, so *derived state never writes
/// back into the definition it derives from* (R5) is enforced by the compiler.
///
/// `prior` is the placement from the previous frame. When present, each address's
/// last drawn level is threaded into [`detail`] so the asymmetric dead band applies
/// (finding 20). The first frame passes `None`.
pub fn place(scene: &SceneSet, view: &View, prior: Option<&Placement>) -> Placement {
    let mut out = Placement {
        placed: Vec::new(),
        batches: Vec::new(),
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
    place_group(scene, view, &roots, embedding, None, floor, true, prior, &mut out);
    out
}

fn prior_level(prior: Option<&Placement>, at: &Addr) -> Option<u32> {
    prior?.placed.iter().find(|p| p.at == *at).map(|p| p.level)
}

fn surface_floor(view: &View) -> u32 {
    let px = view.surface.size.x.max(view.surface.size.y) * view.surface.scale_factor;
    if !(px > 1.0) {
        return 0;
    }
    px.log2().floor() as u32
}

/// Appends one placed thing, keeping [`Placement::batches`] a partition of
/// [`Placement::placed`] into runs that share a primitive (D46).
///
/// The presenter authors the grouping because D15 and D29 give this layer *"what is
/// uploaded, in what order, at what detail, **grouped how**"*. A facade that worked
/// the runs out for itself would be inventing the grouping, which is `hyper-ui`'s
/// failure relocated rather than avoided (finding 16).
fn push(out: &mut Placement, primitive: &str, placed: Placed) {
    match out.batches.last_mut() {
        Some(batch) if &*batch.primitive == primitive => batch.count += 1,
        _ => out.batches.push(Batch {
            primitive: primitive.into(),
            first: out.placed.len(),
            count: 1,
        }),
    }
    out.placed.push(placed);
}

/// Whether a space is open at this view — that is, whether its interior is shown.
///
/// **This is D45's second half, and it is not a bit comparison.** The address says
/// who is inside whom; how much of it you can see says when you get to look. Before
/// D45 the two were one test — `level > at.prefix_bits()` — which conflated a
/// structural fact with a perceptual one and was unsatisfiable besides (finding 19).
///
/// The quantity is the space's apparent extent in device pixels, against a threshold
/// the *caller* sets ([`View::opening_extent`]), for the reason `View::margin` gives:
/// the right value depends on what is being drawn and only the caller knows that.
/// Deriving it rather than authoring it per record is what keeps the plan's option
/// (b) liability away — nobody has to remember to set a depth, and a space that grows
/// on screen opens without anyone editing it.
///
/// `detail_override` still holds a space open or closed against that default, because
/// D20's *"detail is per space, not per camera"* is the reason the field exists. One
/// step is one doubling, in the log domain, exactly as [`detail`] reads the same
/// field — so the two never disagree about what an override means.
fn is_open(item: &Placeable, rect: &Rect, view: &View) -> bool {
    if !item.hosts_space {
        return false;
    }
    let across = (rect.max.x - rect.min.x).abs();
    let down = (rect.max.y - rect.min.y).abs();
    let apparent = across.max(down) * view.surface.scale_factor;
    let held = f64::from(item.detail_override.unwrap_or(0).clamp(-1024, 1024) as i32).exp2();
    apparent * held >= view.opening_extent
}

#[allow(clippy::too_many_arguments)]
fn place_group(
    scene: &SceneSet,
    view: &View,
    addrs: &[Addr],
    parent_to_surface: Transform,
    clip: Option<Rect>,
    floor: u32,
    stack_at_origin: bool,
    prior: Option<&Placement>,
    out: &mut Placement,
) {
    if addrs.is_empty() {
        return;
    }
    let all: Vec<_> = addrs.iter().filter_map(|a| scene.get(a)).collect();
    // A link has no extent of its own to allocate — it runs between two things that
    // do — so it takes no part in the arrangement and is placed after them, when
    // their rectangles exist.
    let items: Vec<_> = all.iter().copied().filter(|i| i.link.is_none()).collect();
    let links: Vec<_> = all.iter().copied().filter(|i| i.link.is_some()).collect();
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
        let position = if stack_at_origin {
            item.position
        } else if item.position == Point::ORIGIN {
            Point::new(x, 0.0)
        } else {
            item.position
        };
        let local = Rect::new(position, Point::new(position.x + w, position.y + h));
        let local_to_parent = Transform::new(1.0, position);
        let local_to_surface = local_to_parent.then(&parent_to_surface);
        let rect = parent_to_surface.apply_rect(&local);
        let showing = match &clip {
            Some(c) => rect.intersect(c),
            None => rect,
        };
        let level = detail(
            view.camera.zoom,
            item.detail_override,
            floor,
            prior_level(prior, &item.at),
        );
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
        push(
            out,
            &item.primitive,
            Placed {
                at: item.at.clone(),
                rect,
                span: None,
                level,
                clip,
                accepts: item.accepts,
            },
        );
        if is_open(item, &rect, view) {
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
                    prior,
                    out,
                );
            }
        }
        x += w;
    }
    for item in links {
        place_link(view, item, clip, floor, prior, out);
    }
}

/// Places one link — a thing whose geometry is *where its ends landed*.
///
/// The endpoints are looked up in what has already been placed, which is why links
/// are placed after the areas in their group: a hyperedge has no position of its own
/// and asking the scene for one would be inventing geometry the author never wrote.
/// An end that is not on screen — culled, or inside a space that is closed — leaves
/// the link unplaced, which is the honest answer and not a line to nowhere.
fn place_link(
    view: &View,
    item: &Placeable,
    clip: Option<Rect>,
    floor: u32,
    prior: Option<&Placement>,
    out: &mut Placement,
) {
    let Some((from, to)) = item.link.as_ref() else {
        return;
    };
    let Some(a) = centre_of(out, from) else { return };
    let Some(b) = centre_of(out, to) else { return };
    // The stroke's half-width, in the same units the extents are authored in, mapped
    // out through whichever space each end sits in. Both ends share a scale in every
    // arrangement this layer produces, so taking the first end's is exact rather than
    // approximate.
    let scale = out
        .spaces
        .get(from)
        .map(|t| t.scale)
        .unwrap_or(view.camera.zoom);
    let half = (item.across.ideal.max(item.across.min).max(1e-12) * scale * 0.5).max(0.5);
    let rect = Rect::new(
        Point::new(a.x.min(b.x) - half, a.y.min(b.y) - half),
        Point::new(a.x.max(b.x) + half, a.y.max(b.y) + half),
    );
    let showing = match &clip {
        Some(c) => rect.intersect(c),
        None => rect,
    };
    if showing.is_empty() || !overlaps_surface(&showing, view) {
        return;
    }
    push(
        out,
        &item.primitive,
        Placed {
            at: item.at.clone(),
            rect,
            span: Some((a, b)),
            level: detail(
                view.camera.zoom,
                item.detail_override,
                floor,
                prior_level(prior, &item.at),
            ),
            clip,
            accepts: item.accepts,
        },
    );
}

fn centre_of(out: &Placement, at: &Addr) -> Option<Point> {
    let placed = out.placed.iter().rev().find(|p| p.at == *at)?;
    Some(Point::new(
        (placed.rect.min.x + placed.rect.max.x) * 0.5,
        (placed.rect.min.y + placed.rect.max.y) * 0.5,
    ))
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
