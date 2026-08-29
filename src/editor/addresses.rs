//! The well-known addresses. The bootstrap ABI, in one file (D34 / E15 O32 / E16.2).
#![allow(missing_docs)]
//!
//! A change to a well-known address is a migration, and there is no migration
//! machinery. This is the only file in the repository in which a literal
//! well-known address may appear (`scripts/check-rules.sh`).
//!
//! # Layout (O32 / E15)
//!
//! Region roots are one-byte keys (bootstrap ABI — six roots). Nested addresses
//! are `parent ‖ u16_be(slot)` with slot ∈ `1..=0xFFFF`. Significant length is
//! `8 × key.len()`. Content keys below are derived via [`child_key`], not hand
//! byte-literals.

use std::sync::OnceLock;

// ── Bootstrap ABI (six region roots) ─────────────────────────────────────────

/// Inclusive start of the screen range.
pub const SCREEN_ROOT_KEY: &[u8] = &[0x10];

/// Inclusive start of the style range.
pub const STYLE_ROOT_KEY: &[u8] = &[0x20];

/// Inclusive start of the behaviour range.
pub const BEHAVIOUR_ROOT_KEY: &[u8] = &[0x30];

/// Inclusive start of the gesture range.
pub const GESTURE_ROOT_KEY: &[u8] = &[0x40];

/// Inclusive start of the session range.
pub const SESSION_ROOT_KEY: &[u8] = &[0x50];

/// Inclusive start of the authored-graph range.
pub const GRAPH_ROOT_KEY: &[u8] = &[0x60];

/// Exclusive end of the screen range (= style root).
pub fn screen_end_key() -> &'static [u8] {
    STYLE_ROOT_KEY
}

/// Exclusive end of the style range (= behaviour root).
pub fn style_end_key() -> &'static [u8] {
    BEHAVIOUR_ROOT_KEY
}

/// Exclusive end of the behaviour range (= gesture root).
pub fn behaviour_end_key() -> &'static [u8] {
    GESTURE_ROOT_KEY
}

/// Exclusive end of gesture (= session root).
pub fn gesture_end_key() -> &'static [u8] {
    SESSION_ROOT_KEY
}

/// Exclusive end of the session range (= graph root).
pub fn session_end_key() -> &'static [u8] {
    GRAPH_ROOT_KEY
}

/// Exclusive end of the graph range (past last region).
pub fn graph_end_key() -> &'static [u8] {
    const PAST: &[u8] = &[0x70];
    PAST
}

/// Pointer position in surface coordinates.
pub const POINTER_POSITION: &str = "/input/pointer/position";
/// Pointer button flags.
pub const POINTER_BUTTON: &str = "/input/pointer/button";
/// Key event.
pub const KEY: &str = "/input/key";
/// Surface size, scale factor, origin.
pub const SURFACE: &str = "/input/surface";
/// Style-table root path string.
pub const STYLE_ROOT: &str = "/style/";
/// Screen root path string.
pub const SCREEN_ROOT: &str = "/screen/";

// ── Derivation (O32) ─────────────────────────────────────────────────────────

/// Append a big-endian `u16` slot under `parent` (slot `0` reserved).
pub fn child_key(parent: &[u8], slot: u32) -> Vec<u8> {
    assert!(slot >= 1 && slot <= 0xFFFF, "slot out of range");
    let mut out = parent.to_vec();
    out.push((slot >> 8) as u8);
    out.push((slot & 0xFF) as u8);
    out
}

fn memo(slots: &[u32], root: &[u8]) -> &'static [u8] {
    // Each call site uses a dedicated OnceLock via the `content_key!` macro.
    let mut k = root.to_vec();
    for &slot in slots {
        k = child_key(&k, slot);
    }
    Box::leak(k.into_boxed_slice())
}

macro_rules! content_key {
    ($name:ident, $root:expr, $($slot:expr),+ $(,)?) => {
        pub fn $name() -> &'static [u8] {
            static KEY: OnceLock<&'static [u8]> = OnceLock::new();
            *KEY.get_or_init(|| memo(&[$($slot),+], $root))
        }
    };
}

// Screen content
content_key!(canvas_key, SCREEN_ROOT_KEY, 1);
content_key!(inspector_key, SCREEN_ROOT_KEY, 2);
content_key!(palette_key, SCREEN_ROOT_KEY, 3);
content_key!(toolbar_key, SCREEN_ROOT_KEY, 4);

content_key!(node_a_key, SCREEN_ROOT_KEY, 1, 1);
content_key!(node_a1_key, SCREEN_ROOT_KEY, 1, 1, 1);
content_key!(node_a2_key, SCREEN_ROOT_KEY, 1, 1, 2);
content_key!(node_b_key, SCREEN_ROOT_KEY, 1, 2);
content_key!(wire_ab_key, SCREEN_ROOT_KEY, 1, 3);

content_key!(inspector_addr_key, SCREEN_ROOT_KEY, 2, 1);
content_key!(inspector_style_key, SCREEN_ROOT_KEY, 2, 2);
content_key!(inspector_across_key, SCREEN_ROOT_KEY, 2, 3);
content_key!(inspector_down_key, SCREEN_ROOT_KEY, 2, 4);
content_key!(inspector_origin_key, SCREEN_ROOT_KEY, 2, 5);
content_key!(inspector_depth_key, SCREEN_ROOT_KEY, 2, 6);

content_key!(palette_plain_key, SCREEN_ROOT_KEY, 3, 1);
content_key!(palette_plain_label_key, SCREEN_ROOT_KEY, 3, 1, 1);
content_key!(palette_total_key, SCREEN_ROOT_KEY, 3, 2);
content_key!(palette_total_label_key, SCREEN_ROOT_KEY, 3, 2, 1);
content_key!(palette_bump_key, SCREEN_ROOT_KEY, 3, 3);
content_key!(palette_bump_label_key, SCREEN_ROOT_KEY, 3, 3, 1);

content_key!(toolbar_history_key, SCREEN_ROOT_KEY, 4, 1);
content_key!(toolbar_zoom_key, SCREEN_ROOT_KEY, 4, 2);
content_key!(toolbar_run_key, SCREEN_ROOT_KEY, 4, 3);

// Styles
content_key!(style_plain_key, STYLE_ROOT_KEY, 1);
content_key!(style_canvas_key, STYLE_ROOT_KEY, 2);
content_key!(style_wire_key, STYLE_ROOT_KEY, 3);

// Behaviour
content_key!(behaviour_probe_key, BEHAVIOUR_ROOT_KEY, 1);
content_key!(behaviour_read_key, BEHAVIOUR_ROOT_KEY, 2);
content_key!(behaviour_amend_key, BEHAVIOUR_ROOT_KEY, 3);
content_key!(behaviour_commit_key, BEHAVIOUR_ROOT_KEY, 4);
content_key!(behaviour_offset_key, BEHAVIOUR_ROOT_KEY, 5);
content_key!(behaviour_gate_key, BEHAVIOUR_ROOT_KEY, 6);
content_key!(behaviour_displace_key, BEHAVIOUR_ROOT_KEY, 7);
content_key!(behaviour_select_gate_key, BEHAVIOUR_ROOT_KEY, 8);
content_key!(behaviour_encode_selection_key, BEHAVIOUR_ROOT_KEY, 9);
content_key!(behaviour_select_amend_key, BEHAVIOUR_ROOT_KEY, 10);
content_key!(behaviour_select_commit_key, BEHAVIOUR_ROOT_KEY, 11);
content_key!(behaviour_edit_read_key, BEHAVIOUR_ROOT_KEY, 12);
content_key!(behaviour_set_origin_key, BEHAVIOUR_ROOT_KEY, 13);
content_key!(behaviour_edit_gate_key, BEHAVIOUR_ROOT_KEY, 14);
content_key!(behaviour_edit_amend_key, BEHAVIOUR_ROOT_KEY, 15);
content_key!(behaviour_edit_commit_key, BEHAVIOUR_ROOT_KEY, 16);
content_key!(behaviour_place_read_key, BEHAVIOUR_ROOT_KEY, 17);
content_key!(behaviour_place_amend_key, BEHAVIOUR_ROOT_KEY, 18);
content_key!(behaviour_place_commit_key, BEHAVIOUR_ROOT_KEY, 19);
content_key!(behaviour_place_set_origin_key, BEHAVIOUR_ROOT_KEY, 20);
content_key!(behaviour_place_gate_key, BEHAVIOUR_ROOT_KEY, 21);
content_key!(behaviour_encode_wire_key, BEHAVIOUR_ROOT_KEY, 22);
content_key!(behaviour_wire_amend_key, BEHAVIOUR_ROOT_KEY, 23);
content_key!(behaviour_wire_commit_key, BEHAVIOUR_ROOT_KEY, 24);
content_key!(behaviour_wire_gate_key, BEHAVIOUR_ROOT_KEY, 25);

// Gestures
content_key!(drag_from_key, GESTURE_ROOT_KEY, 1);
content_key!(release_pulse_key, GESTURE_ROOT_KEY, 2);
content_key!(edit_origin_key, GESTURE_ROOT_KEY, 3);
content_key!(edit_commit_key, GESTURE_ROOT_KEY, 4);
content_key!(palette_from_key, GESTURE_ROOT_KEY, 5);
content_key!(place_origin_key, GESTURE_ROOT_KEY, 6);
content_key!(place_addr_key, GESTURE_ROOT_KEY, 7);
content_key!(place_commit_key, GESTURE_ROOT_KEY, 8);
content_key!(wire_mode_key, GESTURE_ROOT_KEY, 9);
content_key!(wire_from_key, GESTURE_ROOT_KEY, 10);
content_key!(wire_to_key, GESTURE_ROOT_KEY, 11);
content_key!(wire_addr_key, GESTURE_ROOT_KEY, 12);
content_key!(wire_commit_key, GESTURE_ROOT_KEY, 13);
content_key!(wire_mismatch_key, GESTURE_ROOT_KEY, 14);

// Session
content_key!(camera_key, SESSION_ROOT_KEY, 1);
content_key!(select_key, SESSION_ROOT_KEY, 2);
content_key!(run_key, SESSION_ROOT_KEY, 3);

// Graph / app
content_key!(app_root_key, GRAPH_ROOT_KEY, 1);
content_key!(app_link_key, GRAPH_ROOT_KEY, 1, 1);
content_key!(app_read_key, GRAPH_ROOT_KEY, 1, 2);
content_key!(app_increment_key, GRAPH_ROOT_KEY, 1, 3);
content_key!(app_amend_key, GRAPH_ROOT_KEY, 1, 4);
content_key!(app_commit_key, GRAPH_ROOT_KEY, 1, 5);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_is_six_region_roots() {
        let roots = [
            SCREEN_ROOT_KEY,
            STYLE_ROOT_KEY,
            BEHAVIOUR_ROOT_KEY,
            GESTURE_ROOT_KEY,
            SESSION_ROOT_KEY,
            GRAPH_ROOT_KEY,
        ];
        assert_eq!(roots.len(), 6);
        assert_eq!(screen_end_key(), STYLE_ROOT_KEY);
        assert_eq!(graph_end_key(), &[0x70]);
    }

    #[test]
    fn content_keys_are_derived_not_literal_tables() {
        assert_eq!(canvas_key(), child_key(SCREEN_ROOT_KEY, 1).as_slice());
        assert_eq!(
            node_a_key(),
            child_key(canvas_key(), 1).as_slice()
        );
        assert_eq!(
            behaviour_probe_key(),
            child_key(BEHAVIOUR_ROOT_KEY, 1).as_slice()
        );
    }
}
