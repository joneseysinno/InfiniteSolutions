use crate::facade::ports::text::TextKey;

use super::TextRenderer;

impl TextRenderer {
    pub(crate) fn ensure_buffer(&mut self, key: &TextKey) {
        if self.cache.iter().any(|(k, _)| k == key) {
            return;
        }
        let buffer = self.make_buffer(key);
        self.cache.push((key.clone(), buffer));
    }

    pub(crate) fn take_buffer(&mut self, key: &TextKey) -> glyphon::Buffer {
        if let Some(i) = self.cache.iter().position(|(k, _)| k == key) {
            return self.cache.swap_remove(i).1;
        }
        self.make_buffer(key)
    }

    pub(crate) fn insert_buffer(&mut self, key: TextKey, buffer: glyphon::Buffer) {
        if let Some((_, slot)) = self.cache.iter_mut().find(|(k, _)| k == &key) {
            *slot = buffer;
        } else {
            self.cache.push((key, buffer));
        }
    }

    pub(crate) fn buffer(&self, key: &TextKey) -> Option<&glyphon::Buffer> {
        self.cache.iter().find(|(k, _)| k == key).map(|(_, b)| b)
    }
}
