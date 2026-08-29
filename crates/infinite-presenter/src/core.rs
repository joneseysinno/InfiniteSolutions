//! The pure core. Depends on nothing, and does not know a graph exists (R3, D7).
//!
//! Everything here is data in, data out. There is no I/O shape, no clock, no window,
//! no device, and no mutable state: [`place`] takes `&SceneSet` and [`probe`] takes
//! `&Placement`, so *derived state never writes back into the definition it derives
//! from* (R5) is a property of the signatures rather than a rule anyone has to
//! remember. That is the specification's P4, made a compile error.
//!
//! **One scalar, and it is `f64`** (spec §3.3). Narrowing happens inside the `Surface`
//! implementation, in the facade, at the last possible moment. `hyper-ui` ran an f64
//! world through a 32-bit camera and narrowed and widened back across the same
//! boundary twice per frame, which is how an address space whose premise is unbounded
//! refinement came to be projected through 24 bits of mantissa.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod addr;
mod arrange;
mod camera;
mod detail;
mod extent;
mod level;
mod place;
mod placeable;
mod placed;
mod placement;
mod point;
mod probe;
mod rect;
mod revision;
mod scene_set;
mod surface_rect;
mod transform;
mod view;
mod visible;

pub use addr::Addr;
pub use arrange::arrange;
pub use camera::Camera;
pub use detail::detail;
pub use extent::Extent;
pub use level::level;
pub use place::place;
pub use placeable::{Placeable, AREA, TEXT};
pub use placed::Placed;
pub use placement::{Batch, Placement};
pub use point::Point;
pub use probe::{probe, Probe};
pub use rect::Rect;
pub use revision::Revision;
pub use scene_set::SceneSet;
pub use surface_rect::SurfaceRect;
pub use transform::Transform;
pub use view::View;
pub use visible::visible;
