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
            let (across, down, style, detail_override, hosts_space, accepts, position) =
                if let Some(r) = crate::facade::record::decode_space(&payload) {
                    (
                        infinite_presenter::core::Extent::new(r.across[0], r.across[1], r.across[2]),
                        infinite_presenter::core::Extent::new(r.down[0], r.down[1], r.down[2]),
                        r.style.into_boxed_str(),
                        r.detail_override,
                        r.hosts_space,
                        r.accepts,
                        infinite_presenter::core::Point::new(r.origin[0], r.origin[1]),
                    )
                } else if payload == b"space" {
                    (
                        Extent::fixed(1.0),
                        Extent::fixed(1.0),
                        "plain".into(),
                        None,
                        false,
                        true,
                        infinite_presenter::core::Point::ORIGIN,
                    )
                } else {
                    continue;
                };
            set.insert(Placeable {
                at: presenter_addr(&bytes),
                across,
                down,
                style,
                detail_override,
                hosts_space,
                accepts,
                position,
            });
        }
        set
    }

    fn camera(&self, _of: &Addr, _at: Revision) -> Option<Camera> {
        *self.inner.camera.lock().expect("camera lock")
    }
}
