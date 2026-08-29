//! Place, probe, and submit a frame. The portal never names the presenter.

use std::collections::BTreeMap;

use infinite_presenter::binding::ports::Scene as ScenePort;
use infinite_presenter::binding::ports::Surface as SurfacePort;
use infinite_presenter::binding::compose;
use infinite_presenter::core::{probe, Camera, Point, Revision, View};

use super::finding::{from_empty_screen, from_precision_floor};
use super::open::Store;
use super::ports::Surface;

fn default_camera() -> Camera {
    Camera::new(Point::new(0.5, 0.5), 400.0)
}

/// Well-known key matches `editor::addresses::CAMERA_KEY` (D34); the literal bytes
/// appear here rather than a cross-layer import, matching the `SCREEN_ROOT_KEY`
/// precedent in `record_findings` below (R2).
const CAMERA_START: &[u8] = &[0x51, 0x00, 0x00, 0x00];
const CAMERA_END: &[u8] = &[0x52, 0x00, 0x00, 0x00];
const SELECT_START: &[u8] = &[0x52, 0x00, 0x00, 0x00];
const SESSION_END: &[u8] = &[0x53, 0x00, 0x00, 0x00];
const SCREEN_START: &[u8] = &[0x10, 0x00, 0x00, 0x00];
const SCREEN_END: &[u8] = &[0x20, 0x00, 0x00, 0x00];

/// What the property inspector shows about the current selection (E13.2).
///
/// Built from the `Scene` port only — the same source the canvas uses.
pub struct SelectionView {
    /// The store key, as lowercase hex with no separators.
    pub address: String,
    /// Opaque style key on the placeable.
    pub style: String,
    /// Across extent: min, ideal, weight.
    pub across: [f64; 3],
    /// Down extent: min, ideal, weight.
    pub down: [f64; 3],
    /// Authored origin in the containing space.
    pub origin: [f64; 2],
    /// Depth in levels, from the address prefix (D45).
    pub depth: u32,
}

impl Store {
    /// Sets the drawable rectangle. Origin may be non-zero (P1).
    ///
    /// **This is the one place the surface's geometry is set** (E10.3, D43). The
    /// portal calls it on resize and on a scale-factor change; nothing else does, and
    /// `/input/surface` is not a second path to the same fact.
    pub fn set_surface(&self, origin_x: f64, origin_y: f64, width: f64, height: f64, scale: f64) {
        *self.inner.surface.lock().expect("surface lock") =
            infinite_presenter::core::SurfaceRect::new(
                Point::new(origin_x, origin_y),
                Point::new(width, height),
                scale,
            );
    }

    /// The camera this store will place with. Session-scoped (D5), authored at
    /// `CAMERA_START` (E10.5): resolved stored ∪ pending, exactly as `Definitions`
    /// resolves a composition, so a restart replays it from the journal instead of
    /// losing it to a field that was never a record.
    pub fn camera(&self) -> Camera {
        let at = self.inner.db.stable_revision().legacy_sequence();
        let mut rows = self
            .inner
            .records_in_range(CAMERA_START, CAMERA_END, at)
            .unwrap_or_default();
        for (bytes, payload) in self.inner.overlay_pending(CAMERA_START, CAMERA_END) {
            if let Some(existing) = rows.iter_mut().find(|(b, _)| b == &bytes) {
                existing.1 = payload;
            } else {
                rows.push((bytes, payload));
            }
        }
        rows.into_iter()
            .find(|(bytes, _)| bytes.as_slice() == CAMERA_START)
            .and_then(|(_, payload)| super::record::decode_camera(&payload))
            .map(|(x, y, zoom)| Camera::new(Point::new(x, y), zoom))
            .unwrap_or_else(default_camera)
    }

    /// The authored selection. Resolved stored ∪ pending (E13.1).
    pub fn selection(&self) -> Option<Vec<u8>> {
        let at = self.inner.db.stable_revision().legacy_sequence();
        let mut rows = self
            .inner
            .records_in_range(SELECT_START, SESSION_END, at)
            .unwrap_or_default();
        for (bytes, payload) in self.inner.overlay_pending(SELECT_START, SESSION_END) {
            if let Some(existing) = rows.iter_mut().find(|(b, _)| b == &bytes) {
                existing.1 = payload;
            } else {
                rows.push((bytes, payload));
            }
        }
        rows.into_iter()
            .find(|(bytes, _)| bytes.as_slice() == SELECT_START)
            .and_then(|(_, payload)| super::record::decode_selection(&payload))
            .flatten()
    }

    /// The selected placeable as the scene port sees it (E13.2).
    pub fn selection_view(&self) -> Option<SelectionView> {
        let key = self.selection()?;
        let scene = self.scene();
        let at = Revision::new(self.inner.db.stable_revision().legacy_sequence());
        let set = ScenePort::placed_in(
            &scene,
            &crate::facade::presenter_addr(SCREEN_START),
            &crate::facade::presenter_addr(SCREEN_END),
            at,
        );
        let addr = crate::facade::presenter_addr(&key);
        let item = set.get(&addr)?;
        Some(SelectionView {
            address: key
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            style: item.style.to_string(),
            across: [item.across.min, item.across.ideal, item.across.weight],
            down: [item.down.min, item.down.ideal, item.down.weight],
            origin: [item.position.x, item.position.y],
            depth: addr.prefix_bits() / 4,
        })
    }

    /// Moves the session camera by a logical surface delta.
    pub fn pan_by(&self, delta_x: f64, delta_y: f64) {
        let current = self.camera();
        let zoom = current.zoom.max(f64::MIN_POSITIVE);
        let centre = current.centre.sub(Point::new(delta_x / zoom, delta_y / zoom));
        self.amend(
            CAMERA_START,
            &super::record::encode_camera(centre.x, centre.y, zoom),
        );
    }

    /// Changes the session camera magnification while keeping it in a usable range.
    pub fn zoom_by(&self, steps: f64) {
        let current = self.camera();
        let zoom = (current.zoom * 1.1_f64.powf(steps)).clamp(1.0, 1.0e9);
        self.amend(
            CAMERA_START,
            &super::record::encode_camera(current.centre.x, current.centre.y, zoom),
        );
    }

    /// Binds the graph composition `link` previews while a wire is pending (C4).
    pub fn bind_graph(&self, root: &[u8]) {
        *self.inner.graph_root.lock().expect("graph lock") = Some(root.to_vec());
    }

    /// Binds the authored style table (D44). The app owns the addresses (D34), so the
    /// app hands them over, exactly as it does for the plan and the graph.
    pub fn bind_styles(&self, start: &[u8], end: &[u8]) {
        *self.inner.style_range.lock().expect("style lock") = Some((start.to_vec(), end.to_vec()));
    }

    /// Binds the space whose fill is the background (E10.2).
    ///
    /// The clear colour is authored, not a constant: edit that space's style row and
    /// the background changes, which makes the very first pixel drawn a proof of the
    /// whole chain — store, scene, place, surface, screen.
    pub fn bind_background(&self, at: &[u8]) {
        *self.inner.background.lock().expect("background lock") = Some(at.to_vec());
    }

    /// The authored style table, as name → fill, in address order.
    ///
    /// A `Vec` rather than a map keyed by the name: L5 forbids a map keyed by
    /// anything but an address, the table is a handful of rows, and dodging the rule
    /// with a lookup structure would be the letter against the spirit.
    pub fn styles(&self) -> Vec<(String, [f64; 4])> {
        let Some((start, end)) = self.inner.style_range.lock().expect("style lock").clone() else {
            return Vec::new();
        };
        self.records(&start, &end)
            .into_iter()
            .filter_map(|(_, payload)| super::record::decode_style(&payload))
            .collect()
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
        self.amend(
            CAMERA_START,
            &super::record::encode_camera(centre.x, centre.y, current.zoom * 2.0),
        );
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

    /// Places the current scene and submits it through `surface`.
    ///
    /// **O21 is closed here.** `infinite_presenter::binding::frame` resolved its own
    /// `SceneSet`, submitted, and dropped the set — and D44's fill resolution needs
    /// the set the placement was built from, so this function took the steps itself
    /// and left `frame` with no caller, which R27 makes a defect. D47 retires the
    /// name and gives the binding [`infinite_presenter::binding::compose`], which
    /// hands the set back and leaves submitting to whoever knows what a style key
    /// means. That is this function, and it is the caller.
    pub fn draw_with(&self, surface: &mut Surface) {
        let geometry = *self.inner.surface.lock().expect("surface lock");
        surface.set_geometry(geometry);
        let scene = self.scene();
        let at = Revision::new(self.inner.db.stable_revision().legacy_sequence());
        let camera = ScenePort::camera(&scene, &crate::facade::presenter_addr(&[0, 0, 0, 1]), at)
            .unwrap_or_else(|| self.camera());
        let view = View::new(camera, geometry, 0.0);
        let (set, placement) = compose(&scene, &view, at);

        let styles = self.styles();
        let mut fills = BTreeMap::new();
        let mut text_runs = BTreeMap::new();
        for item in set.iter() {
            fills.insert(item.at.clone(), fill_of(&styles, &item.style));
            if &*item.primitive == infinite_presenter::core::TEXT {
                text_runs.insert(item.at.clone(), item.text.clone());
            }
        }
        if let Some(background) = self.inner.background.lock().expect("background lock").clone() {
            let key = crate::facade::presenter_addr(&background);
            if let Some(fill) = fills.get(&key) {
                surface.set_clear(*fill);
            }
        }
        surface.set_fills(fills);
        surface.set_text_runs(text_runs);

        SurfacePort::submit(surface, &placement);
        self.record_findings(&placement);
        *self.inner.last_placement.lock().expect("placement lock") = Some(placement);
    }

    /// Places and submits into a surface with no device. What the pre-E10 tests used.
    pub fn draw(&self) {
        let geometry = *self.inner.surface.lock().expect("surface lock");
        let mut surface = Surface::with_geometry(geometry);
        self.draw_with(&mut surface);
    }

    /// Answers a surface point with an address. No port is named here beyond the
    /// stored placement — `probe` itself takes none.
    pub fn probe_at(&self, x: f64, y: f64) -> Option<Vec<u8>> {
        let placed = self.inner.last_placement.lock().expect("placement lock");
        let placement = placed.as_ref()?;
        probe(placement, Point::new(x, y)).map(|p| p.at.as_bytes().to_vec())
    }

    /// Links the composition at `root`. The editor never names [`link`].
    pub fn link_at(
        &self,
        root: &[u8],
    ) -> infinite_compositor::core::Outcome<infinite_compositor::core::Plan> {
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
        let view = View::new(self.camera(), geometry, 0.0);
        let (_set, placement) = compose(&scene, &view, at);
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

/// Resolves one style key against the authored table.
///
/// An unknown key gets a visible grey rather than nothing, because
/// `PRESENTER.md` §13 finding 8 is that a failed lookup and an empty screen must
/// never be indistinguishable.
fn fill_of(styles: &[(String, [f64; 4])], key: &str) -> [f64; 4] {
    styles
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, fill)| *fill)
        .unwrap_or([0.55, 0.55, 0.55, 1.0])
}

#[cfg(test)]
mod tests {
    use crate::facade;
    use infinite_presenter::core::Point;

    #[test]
    fn session_camera_pan_and_zoom_are_stable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = facade::open(dir.path()).expect("open");
        store.pan_by(40.0, 20.0);
        let camera = store.camera();
        assert_eq!(camera.centre, Point::new(0.4, 0.45));
        store.zoom_by(1.0);
        assert!((store.camera().zoom - 440.0).abs() < 1e-12);
    }
}
