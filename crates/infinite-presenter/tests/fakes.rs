//! The only implementations of the ports this layer ever names (D29).

#![cfg(feature = "binding")]
#![allow(dead_code)]

use infinite_presenter::binding::ports::{Glyphs, Scene, Surface};
use infinite_presenter::core::{
    Addr, Camera, Placement, Point, Rect, Revision, SceneSet, SurfaceRect,
};

/// A scene the test builds by hand.
pub struct FakeScene {
    set: SceneSet,
    camera: Option<Camera>,
}

impl FakeScene {
    /// A scene at `at` with no camera of its own.
    pub fn new(at: Revision) -> Self {
        Self {
            set: SceneSet::new(at),
            camera: None,
        }
    }

    /// Adds a placeable.
    pub fn insert(&mut self, item: infinite_presenter::core::Placeable) {
        self.set.insert(item);
    }
}

impl Scene for FakeScene {
    fn placed_in(&self, start: &Addr, end: &Addr, _at: Revision) -> SceneSet {
        let mut out = SceneSet::new(self.set.at());
        for item in self.set.iter() {
            if item.at.in_range(start, end) {
                out.insert(item.clone());
            }
        }
        out
    }

    fn camera(&self, _of: &Addr, _at: Revision) -> Option<Camera> {
        self.camera
    }
}

/// A surface whose origin the test can put anywhere. That is the whole point of P1.
pub struct FakeSurface {
    geometry: SurfaceRect,
    submitted: Option<Placement>,
}

impl FakeSurface {
    /// A surface at `origin`.
    pub fn new(origin: Point, size: Point, scale: f64) -> Self {
        Self {
            geometry: SurfaceRect::new(origin, size, scale),
            submitted: None,
        }
    }
}

impl Surface for FakeSurface {
    fn geometry(&self) -> SurfaceRect {
        self.geometry
    }

    fn submit(&mut self, placement: &Placement) {
        self.submitted = Some(placement.clone());
    }
}

/// Glyphs that return a declared box, so layout tests assert on arithmetic they control.
pub struct FakeGlyphs;

impl Glyphs for FakeGlyphs {
    fn measure(&self, run: &str, size: f64) -> Rect {
        Rect::new(
            Point::ORIGIN,
            Point::new(size * run.chars().count() as f64, size),
        )
    }
}
