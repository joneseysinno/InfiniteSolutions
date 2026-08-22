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
    if key == "plain" {
        Descriptor {
            fill: [0.22, 0.48, 0.82, 1.0],
        }
    } else {
        Descriptor {
            fill: [0.55, 0.55, 0.55, 1.0],
        }
    }
}
