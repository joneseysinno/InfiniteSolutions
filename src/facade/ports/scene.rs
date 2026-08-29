//! [`Scene`] — presenter. Extents, style keys, detail overrides, hosts-a-space.
//!
//! O10: this is where *"may this viewer see that space"* would be inserted. Do
//! not build it so that the check cannot be.

use std::sync::Arc;

use infinite_presenter::binding::ports::Scene as Port;
use infinite_presenter::binding::ports::Glyphs as GlyphsPort;
use infinite_presenter::core::{
    Addr, Camera, Extent, Placeable, Revision, SceneSet, TEXT,
};

use crate::facade::addr::presenter_addr;
use crate::facade::open::Inner;
use crate::facade::ports::Glyphs;
use crate::facade::record::{decode_link_payload, payload_key};

/// Placeable records over the real store.
pub struct Scene {
    pub(crate) inner: Arc<Inner>,
}

fn shape_payload(inner: &Inner, space: &[u8]) -> Vec<u8> {
    let key = payload_key(space);
    for (bytes, payload) in inner.overlay_pending(&key, &inner_successor(&key)) {
        if bytes.as_slice() == key.as_slice() {
            return payload;
        }
    }
    inner.current_value(&key).unwrap_or_default()
}

fn inner_successor(key: &[u8]) -> Vec<u8> {
    crate::facade::open::Inner::successor_key(key)
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
        let glyphs = Glyphs::new();
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
                }
            } else {
                continue;
            };
            let shape = shape_payload(&self.inner, &bytes);
            let text = String::from_utf8_lossy(&shape).into_owned();
            let link = decode_link_payload(&shape)
                .map(|(from, to)| (presenter_addr(&from), presenter_addr(&to)));
            // An unauthored primitive is an area, not an error: every record written
            // before D46 is one, and defaulting here is what keeps them decodable.
            let primitive = if record.primitive.is_empty() {
                infinite_presenter::core::AREA.into()
            } else {
                record.primitive.into_boxed_str()
            };
            let em = record.down[1].max(record.down[0]).max(1e-12);
            let (across, down) = if &*primitive == TEXT {
                let ink = GlyphsPort::measure(&glyphs, &text, em);
                (
                    Extent::fixed((ink.max.x - ink.min.x).max(1e-12)),
                    Extent::fixed(em),
                )
            } else {
                (
                    Extent::new(record.across[0], record.across[1], record.across[2]),
                    Extent::new(record.down[0], record.down[1], record.down[2]),
                )
            };
            set.insert(Placeable {
                at: presenter_addr(&bytes),
                across,
                down,
                style: record.style.into_boxed_str(),
                detail_override: record.detail_override,
                primitive,
                link,
                hosts_space: record.hosts_space,
                accepts: record.accepts,
                position: infinite_presenter::core::Point::new(record.origin[0], record.origin[1]),
                text: text.into_boxed_str(),
            });
        }
        set
    }

    fn camera(&self, _of: &Addr, at: Revision) -> Option<Camera> {
        // Well-known key matches `editor::addresses::camera_key()` (D34); the literal
        // bytes appear here rather than a cross-layer import, matching the
        // `SCREEN_ROOT_KEY` precedent in `facade::present::record_findings` (R2).
        let start: &[u8] = &[0x50, 0x00, 0x01];
        let end: &[u8] = &[0x50, 0x00, 0x02];
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
