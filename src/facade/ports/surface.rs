//! [`Surface`] — presenter. wgpu.
//!
//! **The only f64 → f32 narrowing in the repository** (D29, `PRESENTER.md` §3.3),
//! and the only file in `src/` that may contain the token `f32`.

use infinite_presenter::binding::ports::Surface as Port;
use infinite_presenter::core::{Placement, Point, SurfaceRect};

/// The thing being drawn into.
pub struct Surface {
    geometry: SurfaceRect,
    /// Last frame, after narrowing. Kept so a test can see the f32 path ran.
    narrowed: usize,
}

impl Surface {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::with_geometry(SurfaceRect::new(
            Point::ORIGIN,
            Point::new(800.0, 600.0),
            1.0,
        ))
    }

    pub(crate) fn with_geometry(geometry: SurfaceRect) -> Self {
        Self {
            geometry,
            narrowed: 0,
        }
    }

    /// How many vertices were narrowed this submit. For the agreement test.
    pub fn narrowed_count(&self) -> usize {
        self.narrowed
    }
}

impl Port for Surface {
    fn geometry(&self) -> SurfaceRect {
        self.geometry
    }

    fn submit(&mut self, placement: &Placement) {
        // The narrowing point. f64 world → f32 device, once, here.
        let mut verts: Vec<[f32; 2]> = Vec::with_capacity(placement.placed.len() * 4);
        for item in &placement.placed {
            let min_x = item.rect.min.x as f32;
            let min_y = item.rect.min.y as f32;
            let max_x = item.rect.max.x as f32;
            let max_y = item.rect.max.y as f32;
            verts.push([min_x, min_y]);
            verts.push([max_x, min_y]);
            verts.push([max_x, max_y]);
            verts.push([min_x, max_y]);
        }
        self.narrowed = verts.len();
        let _format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let _ = (_format, verts);
    }
}
