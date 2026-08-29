//! OS event → amend at a well-known address. Never a write (D24).
//!
//! There is no other input path. Every event becomes an `amend` at an address
//! from `editor::addresses`.

use crate::editor::addresses;
use crate::facade::Store;

/// The input portal.
pub struct Input;

impl Input {
    /// Pointer moved. Amended every move.
    pub fn on_pointer_move(&mut self, store: &Store, x: f64, y: f64) {
        let mut payload = Vec::with_capacity(16);
        payload.extend_from_slice(&x.to_le_bytes());
        payload.extend_from_slice(&y.to_le_bytes());
        store.amend(addresses::POINTER_POSITION.as_bytes(), &payload);
    }

    /// Pointer button flags. Amended on transition only.
    pub fn on_pointer_button(&mut self, store: &Store, flags: u8) {
        store.amend(addresses::POINTER_BUTTON.as_bytes(), &[flags]);
        if flags == 0 {
            store.amend(addresses::RELEASE_PULSE_KEY, &[1]);
        }
    }

    /// Key event. Amended on transition only.
    pub fn on_key(&mut self, store: &Store, payload: &[u8]) {
        store.amend(addresses::KEY.as_bytes(), payload);
    }

    /// Surface size, scale factor, origin. Amended on resize.
    pub fn on_resize(&mut self, store: &Store, width: f64, height: f64, scale: f64) {
        let mut payload = Vec::with_capacity(24);
        payload.extend_from_slice(&width.to_le_bytes());
        payload.extend_from_slice(&height.to_le_bytes());
        payload.extend_from_slice(&scale.to_le_bytes());
        store.amend(addresses::SURFACE.as_bytes(), &payload);
    }
}
