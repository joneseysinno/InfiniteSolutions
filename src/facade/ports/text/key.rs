//! Content-keyed cache entry for a shaped text buffer.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextKey {
    pub text: String,
    pub size_milli: u32,
    pub weight: u16,
}
