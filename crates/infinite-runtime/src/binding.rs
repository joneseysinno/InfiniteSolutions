//! The binding — the part that knows a graph exists (D7).
//!
//! It declares the [`ports`] the runtime needs and drives them. **It names no crate
//! belonging to another layer** (D23): the platform facade supplies the port
//! implementations, and the only store this layer ever names is the fake in
//! `tests/fake_store.rs`.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

pub mod ports;

mod driver;
mod registry;

pub use driver::Driver;
pub use registry::{Artifact, ArtifactRegistry};
