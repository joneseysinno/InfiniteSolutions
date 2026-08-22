//! Place, probe, and submit a frame. The portal never names the presenter.

use infinite_presenter::binding::frame;
use infinite_presenter::binding::ports::Scene as ScenePort;
use infinite_presenter::core::{place, probe, Camera, Point, Revision, View};

use super::finding::{from_empty_screen, from_precision_floor};
use super::open::Store;
use super::ports::Surface;

fn default_camera() -> Camera {
    Camera::new(Point::new(0.5, 0.5), 400.0)
}

impl Store {
    /// Sets the drawable rectangle. Origin may be non-zero (P1).
    pub fn set_surface(&self, origin_x: f64, origin_y: f64, width: f64, height: f64, scale: f64) {
        *self.inner.surface.lock().expect("surface lock") =
            infinite_presenter::core::SurfaceRect::new(
                Point::new(origin_x, origin_y),
                Point::new(width, height),
                scale,
            );
    }

    /// The camera this store will place with. Session-scoped (D5).
    pub fn camera(&self) -> Camera {
        self.inner
            .camera
            .lock()
            .expect("camera lock")
            .unwrap_or_else(default_camera)
    }

    /// Binds the graph composition `link` previews while a wire is pending (C4).
    pub fn bind_graph(&self, root: &[u8]) {
        *self.inner.graph_root.lock().expect("graph lock") = Some(root.to_vec());
    }

    /// Zooms the canvas to a finding's site (D20 — go-to-error is a zoom).
    pub fn zoom_to(&self, site: &[u8]) {
        let geometry = *self.inner.surface.lock().expect("surface lock");
        let current = self.camera();
        let view = View::new(current, geometry, 0.0);
        let placement = self.place_now();
        let mut centre = current.centre;
        if let Some(item) = placement.placed.iter().find(|p| p.at.as_bytes() == site) {
            let mid = Point::new(
                (item.rect.min.x + item.rect.max.x) * 0.5,
                (item.rect.min.y + item.rect.max.y) * 0.5,
            );
            centre = view.embedding().invert().apply(mid);
        }
        *self.inner.camera.lock().expect("camera lock") =
            Some(Camera::new(centre, current.zoom * 2.0));
    }

    /// Writes one 1×1 space if the editor space is empty, so the window has
    /// something to show. E4 replaces this with genesis.
    pub fn ensure_space(&self) {
        let origin = [0, 0, 0, 1];
        let mut write = self.store_write();
        use infinite_runtime::binding::ports::StoreWrite;
        let _ = write.submit(&crate::facade::runtime_addr(&origin), b"space");
        drop(write);
        let _ = self.sync();
    }

    /// Places the current scene and submits it. Remembers the placement for probe.
    pub fn draw(&self) {
        let geometry = *self.inner.surface.lock().expect("surface lock");
        let scene = self.scene();
        let camera = ScenePort::camera(
            &scene,
            &crate::facade::presenter_addr(&[0, 0, 0, 1]),
            Revision::new(self.inner.db.stable_revision().legacy_sequence()),
        )
        .unwrap_or_else(|| self.camera());
        let view = View::new(camera, geometry, 0.0);
        let at = Revision::new(self.inner.db.stable_revision().legacy_sequence());
        let mut surface = Surface::with_geometry(geometry);
        frame(&scene, &mut surface, &view, at);
        let start = crate::facade::presenter_addr(&[]);
        let end = crate::facade::presenter_addr(&[0xFF, 0xFF, 0xFF, 0xFF]);
        let set = ScenePort::placed_in(&scene, &start, &end, at);
        let placement = place(&set, &view);
        self.record_findings(&placement);
        *self.inner.last_placement.lock().expect("placement lock") = Some(placement);
    }

    /// Answers a surface point with an address. No port is named here beyond the
    /// stored placement — `probe` itself takes none.
    pub fn probe_at(&self, x: f64, y: f64) -> Option<Vec<u8>> {
        let placed = self.inner.last_placement.lock().expect("placement lock");
        let placement = placed.as_ref()?;
        probe(placement, Point::new(x, y)).map(|p| p.at.as_bytes().to_vec())
    }

    /// Links the composition at `root`. The editor never names [`link`].
    pub fn link_at(&self, root: &[u8]) -> infinite_compositor::core::Outcome<infinite_compositor::core::Plan> {
        use infinite_compositor::binding::ports::Definitions;
        use infinite_compositor::core::link;
        let defs = self.definitions().resolve(&crate::facade::compositor_addr(root));
        link(&defs, &crate::facade::compositor_addr(root))
    }

    /// Places without submitting, for tests that need the placement.
    pub fn place_now(&self) -> infinite_presenter::core::Placement {
        let geometry = *self.inner.surface.lock().expect("surface lock");
        let scene = self.scene();
        let at = Revision::new(self.inner.db.stable_revision().legacy_sequence());
        let start = crate::facade::presenter_addr(&[]);
        let end = crate::facade::presenter_addr(&[0xFF, 0xFF, 0xFF, 0xFF]);
        let set = ScenePort::placed_in(&scene, &start, &end, at);
        let view = View::new(self.camera(), geometry, 0.0);
        let placement = place(&set, &view);
        self.record_findings(&placement);
        *self.inner.last_placement.lock().expect("placement lock") = Some(placement.clone());
        placement
    }

    fn record_findings(&self, placement: &infinite_presenter::core::Placement) {
        let mut findings = Vec::new();
        if placement.placed.is_empty() {
            // Site bytes match `editor::addresses::SCREEN_ROOT_KEY`. The path
            // string must not appear here (D34).
            findings.push(from_empty_screen(&[0x10, 0x00, 0x00, 0x00]));
        }
        if let Some(addr) = &placement.precision_floor {
            findings.push(from_precision_floor(addr.as_bytes()));
        }
        if let Some(root) = self.inner.graph_root.lock().expect("graph lock").clone() {
            if self.pending_at(&root).is_some() || self.has(&root) {
                findings.extend(self.link_at(&root).findings);
            }
        }
        *self.inner.findings.lock().expect("findings lock") = findings;
    }
}
