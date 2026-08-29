//! Shaped text via glyphon + cosmic-text (E14, R-F).
//!
//! Ported from Innovator's `hyper-ui/src/text/`. Graphics crates stay in the
//! facade (D29). The presenter asks only for extents through the `Glyphs` port;
//! rasterisation is queued here and drawn by [`Surface`](super::Surface).

mod key;
mod pending;
mod renderer;

pub use key::TextKey;
pub use renderer::TextRenderer;

pub(crate) use pending::PendingText;

/// Embedded Inter Regular (OFL). Loaded once so CI readbacks are machine-stable.
pub(crate) const FONT_BYTES: &[u8] = include_bytes!("../assets/Inter-Regular.ttf");

/// The family name registered for [`FONT_BYTES`].
pub(crate) const FONT_FAMILY: &str = "Inter";
