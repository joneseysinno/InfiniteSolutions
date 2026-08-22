//! The binding — the part that knows a graph exists (D7).
//!
//! It declares the [`ports`] the compositor needs and drives them. **It names no crate
//! belonging to another layer** (D26): the platform facade supplies the port
//! implementations, and the only implementations this layer ever names are the fakes
//! in `tests/fakes.rs`.
//!
//! The two registries here are symbol tables, not state — populated at startup,
//! never authored into while a program runs — so L4 survives their existence.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

pub mod ports;

mod artifact;
mod backend;
mod interpret;
mod registry;
mod tier0;

pub use artifact::{encode as encode_plan, KEY};
pub use backend::{Artifact, Backend, Cost};
pub use interpret::interpret;
pub use registry::Registry;
pub use tier0::{check, encode as encode_tier0, Tier0, KEY as TIER0_KEY};
