use glyphon::{Cache, FontSystem, SwashCache, TextAtlas, TextRenderer as GlyphonTextRenderer, Viewport};

use super::TextRenderer;
use crate::facade::ports::text::{FONT_BYTES, FONT_FAMILY};

impl TextRenderer {
    /// Build the atlas and renderer against the facade's device.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        font_system.db_mut().load_font_data(FONT_BYTES.to_vec());
        // Prefer the embedded face so measure and raster agree across machines.
        font_system.db_mut().set_sans_serif_family(FONT_FAMILY);

        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            GlyphonTextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            cache: Vec::new(),
            width: 1,
            height: 1,
            scale_factor: 1.0,
            pending: Vec::new(),
        }
    }
}
