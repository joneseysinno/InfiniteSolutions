//! [`Scene`] — presenter. Extents, style keys, detail overrides, hosts-a-space.
//!
//! O10: this is where *"may this viewer see that space"* would be inserted. Do
//! not build it so that the check cannot be.

use std::sync::Arc;

use infinite_presenter::binding::ports::Scene as Port;
use infinite_presenter::core::{
    Addr, Camera, Extent, Placeable, Revision, SceneSet,
};

use crate::facade::addr::presenter_addr;
use crate::facade::open::Inner;

/// Placeable records over the real store.
pub struct Scene {
    pub(crate) inner: Arc<Inner>,
}

impl Port for Scene {
    fn placed_in(&self, start: &Addr, end: &Addr, at: Revision) -> SceneSet {
        let mut rows = match self.inner.records_in_range(
            start.as_bytes(),
            end.as_bytes(),
            at.get(),
        ) {
            Ok(rows) => rows,
            Err(e) => panic!("scene read failed (not an empty screen): {e}"),
        };
        for (bytes, payload) in self
            .inner
            .overlay_pending(start.as_bytes(), end.as_bytes())
        {
            if let Some(existing) = rows.iter_mut().find(|(b, _)| b == &bytes) {
                existing.1 = payload;
            } else {
                rows.push((bytes, payload));
            }
        }
        let mut set = SceneSet::new(at);
        for (bytes, payload) in rows {
            let record = if let Some(r) = crate::facade::record::decode_space(&payload) {
                r
            } else if payload == b"space" {
                crate::facade::SpaceRecord {
                    across: [1.0, 1.0, 0.0],
                    down: [1.0, 1.0, 0.0],
                    style: "plain".into(),
                    detail_override: None,
                    hosts_space: false,
                    accepts: true,
                    origin: [0.0, 0.0],
                    primitive: String::new(),
                    link: None,
                }
            } else {
                continue;
            };
            // An unauthored primitive is an area, not an error: every record written
            // before D46 is one, and defaulting here is what keeps them decodable.
            let primitive = if record.primitive.is_empty() {
                infinite_presenter::core::AREA.into()
            } else {
                record.primitive.into_boxed_str()
            };
            set.insert(Placeable {
                at: presenter_addr(&bytes),
                across: Extent::new(record.across[0], record.across[1], record.across[2]),
                down: Extent::new(record.down[0], record.down[1], record.down[2]),
                style: record.style.into_boxed_str(),
                detail_override: record.detail_override,
                primitive,
                link: record
                    .link
                    .map(|(from, to)| (presenter_addr(&from), presenter_addr(&to))),
                hosts_space: record.hosts_space,
                accepts: record.accepts,
                position: infinite_presenter::core::Point::new(record.origin[0], record.origin[1]),
            });
        }
        set
    }

    fn camera(&self, _of: &Addr, at: Revision) -> Option<Camera> {
        // Well-known key matches `editor::addresses::CAMERA_KEY` (D34); the literal
        // bytes appear here rather than a cross-layer import, matching the
        // `SCREEN_ROOT_KEY` precedent in `facade::present::record_findings` (R2).
        let start: &[u8] = &[0x51, 0x00, 0x00, 0x00];
        let end: &[u8] = &[0x52, 0x00, 0x00, 0x00];
        let mut rows = match self.inner.records_in_range(start, end, at.get()) {
            Ok(rows) => rows,
            Err(_) => Vec::new(),
        };
        for (bytes, payload) in self.inner.overlay_pending(start, end) {
            if let Some(existing) = rows.iter_mut().find(|(b, _)| b == &bytes) {
                existing.1 = payload;
            } else {
                rows.push((bytes, payload));
            }
        }
        rows.into_iter()
            .find(|(bytes, _)| bytes.as_slice() == start)
            .and_then(|(_, payload)| crate::facade::record::decode_camera(&payload))
            .map(|(x, y, zoom)| Camera::new(infinite_presenter::core::Point::new(x, y), zoom))
    }
}
