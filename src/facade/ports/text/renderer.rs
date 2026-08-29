//! glyphon text renderer with a content-keyed buffer cache.

mod clear_pending;
mod ensure_buffer;
mod make_buffer;
mod measure;
mod new;
mod prepare;
mod queue_text;
mod render_into;
mod resize;
mod trim;

use glyphon::{FontSystem, SwashCache, TextAtlas, Viewport};

use super::{PendingText, TextKey};

/// glyphon + cosmic-text rasteriser for the surface (E14).
pub struct TextRenderer {
    pub(crate) font_system: FontSystem,
    pub(crate) swash_cache: SwashCache,
    pub(crate) viewport: Viewport,
    pub(crate) atlas: TextAtlas,
    pub(crate) renderer: glyphon::TextRenderer,
    /// Content-keyed cache — a vec, not a map, so L5's address-keyed-map check
    /// stays honest (this is a glyph buffer cache, not an identity table).
    pub(crate) cache: Vec<(TextKey, glyphon::Buffer)>,
    /// Logical viewport size (matches UI layout coordinates).
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Window scale — sharpens glyphs while positions stay logical.
    pub(crate) scale_factor: f32,
    pub(crate) pending: Vec<PendingText>,
}
