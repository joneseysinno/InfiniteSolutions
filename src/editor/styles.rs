//! Style keys, and the bootstrap default table.
//!
//! The descriptor is data; the facade's `Surface` turns it into GPU work. This
//! file names no graphics crate. Authored rows under the style root land in E4;
//! this function is the fallback so an emptied store still renders a default.

/// Fill colour as four unit intervals (red, green, blue, alpha).
#[derive(Clone, Copy, Debug)]
pub struct Descriptor {
    /// The fill.
    pub fill: [f64; 4],
}

/// Bootstrap default for the case where the store has no style space.
pub fn bootstrap_default(key: &str) -> Descriptor {
    match key {
        "plain" => Descriptor {
            fill: [0.22, 0.48, 0.82, 1.0],
        },
        "canvas" => Descriptor {
            fill: [0.12, 0.13, 0.16, 1.0],
        },
        // E11. Deliberately far from `plain` in every channel, so a readback that
        // finds wire pixels where node pixels should be cannot pass as a rounding
        // difference — `tests/wires.rs` leans on the gap being larger than tolerance.
        "wire" => Descriptor {
            fill: [0.95, 0.71, 0.20, 1.0],
        },
        // Visible, and deliberately not the background: a style key with no authored
        // row must not be indistinguishable from nothing being there
        // (`PRESENTER.md` §13 finding 8).
        _ => Descriptor {
            fill: [0.55, 0.55, 0.55, 1.0],
        },
    }
}
