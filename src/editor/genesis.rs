//! The seed. Writes graph data. Contains no policy (`docs/specs/EDITOR.md` §6).
//!
//! Spaces via [`crate::editor::spec`] + [`crate::editor::screen_seed`] (E16).
//! Behaviour via [`crate::editor::behaviour_seed`].

use crate::editor::addresses;
use crate::editor::behaviour_seed;
use crate::editor::component_seed;
use crate::editor::screen_seed;
use crate::editor::spec::{self, build};
use crate::editor::styles::bootstrap_default;
use crate::facade::{encode_composition, encode_selection, encode_style};

/// Writes the editor's screen, styles, and behaviour composition.
pub fn seed(exists: impl Fn(&[u8]) -> bool, mut put: impl FnMut(&[u8], &[u8])) {
    for flat in spec::flatten(addresses::SCREEN_ROOT_KEY, &screen_seed::screen_tree()) {
        put_if(&exists, &mut put, &flat.key, &flat.payload);
    }
    // Strictly more spaces than pre-E16 (extra leaf under the canvas).
    for flat in spec::flatten(
        addresses::canvas_key(),
        &[build::area(
            "extra-1",
            10,
            [0.05, 0.05, 0.0],
            [0.05, 0.05, 0.0],
            [0.9, 0.05],
            false,
        )],
    ) {
        put_if(&exists, &mut put, &flat.key, &flat.payload);
    }

    put_if(
        &exists,
        &mut put,
        addresses::style_plain_key(),
        &encode_style("plain", bootstrap_default("plain").fill),
    );
    put_if(
        &exists,
        &mut put,
        addresses::style_canvas_key(),
        &encode_style("canvas", bootstrap_default("canvas").fill),
    );
    put_if(
        &exists,
        &mut put,
        addresses::style_wire_key(),
        &encode_style("wire", bootstrap_default("wire").fill),
    );
    put_if(
        &exists,
        &mut put,
        addresses::BEHAVIOUR_ROOT_KEY,
        &encode_composition(&behaviour_seed::behaviour()),
    );
    put_if(
        &exists,
        &mut put,
        addresses::select_key(),
        &encode_selection(&[]),
    );
    put_if(&exists, &mut put, addresses::run_key(), &[1]);
    for (key, payload) in component_seed::seed_records() {
        put_if(&exists, &mut put, &key, &payload);
    }
}

fn put_if(
    exists: &impl Fn(&[u8]) -> bool,
    put: &mut impl FnMut(&[u8], &[u8]),
    key: &[u8],
    payload: &[u8],
) {
    if !exists(key) {
        put(key, payload);
    }
}
