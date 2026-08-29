//! The thirteen ports (D23, D26, D29).
//!
//! Trait definitions live in the layer crates. Implementations live here. A sixth
//! runtime port, a sixth compositor port, or a fourth presenter port requires a
//! decision record.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod backends;
mod blocks;
mod clock;
mod definitions;
pub(crate) mod pure_fn;
mod glyphs;
mod journal;
mod provenance;
mod scene;
mod stale_feed;
mod store_read;
mod store_write;
mod surface;
mod text;
mod values;

pub use backends::Backends;
pub use blocks::Blocks;
pub(crate) use blocks::inject_natives;
pub use clock::Clock;
pub use definitions::Definitions;
pub use glyphs::Glyphs;
pub use journal::Journal;
pub(crate) use journal::decode_entry;
pub use provenance::Provenance;
pub use scene::Scene;
pub use stale_feed::StaleFeed;
pub use store_read::StoreRead;
pub use store_write::StoreWrite;
pub use surface::Surface;
pub use text::TextRenderer;
pub use values::Values;
