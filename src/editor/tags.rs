//! The editor's tag convention (D13 — tags are the app's).
//!
//! The platform's only operation on a tag is match. A new tag needs a line in
//! `docs/specs/EDITOR.md` §7 saying which of §1's six interactions requires it.

/// A 2-space position in surface coordinates.
pub const POINT: &str = "point";

/// An address's bytes.
pub const ADDRESS: &str = "address";

/// An opaque payload. `read` / `amend` / `gate` carry this.
pub const VALUE: &str = "value";

/// A byte, zero or not.
pub const FLAG: &str = "flag";

/// A key event.
pub const KEY: &str = "key";

/// Size, scale factor, origin.
pub const SURFACE: &str = "surface";

/// Authored geometry, read through `Scene` not through a block.
pub const EXTENT: &str = "extent";
