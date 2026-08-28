//! The binding — the part that knows a graph exists (D7).
//!
//! It declares the [`ports`] the presenter needs and drives them. **It names no crate
//! belonging to another layer, and no graphics crate** (D29): the platform facade
//! supplies the port implementations, and the only ones this layer ever names are the
//! fakes in `tests/fakes.rs`.
//!
//! D15 gives this layer wgpu *resource organization*. The organization is here — what
//! is placed, in what order, at what detail, grouped how. The API is the facade's.
//! [`ports::Surface`] is the seam, and putting it there is what lets the embedding be
//! tested without standing up a device and a window, which is the one thing
//! `hyper-ui` could not do and the reason its renderer has no tests at all.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

pub mod ports;

mod artifact;
mod compose;

pub use artifact::{rebuild, ranges, KEY};
pub use compose::compose;
