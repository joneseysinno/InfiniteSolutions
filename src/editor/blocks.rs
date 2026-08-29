//! The native block set (`docs/specs/EDITOR.md` §5).
//!
//! Each file holds a plain Rust function over opaque payloads. Registration is
//! `src/facade/ports/blocks.rs`, which is what makes this an app under R2 despite
//! living in the same crate. A seventh block needs a line in `EDITOR.md` §5.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod amend;
mod commit;
mod displace;
mod encode_selection;
mod gate;
mod offset;
mod probe_at;
mod read;
mod set_origin;

pub use amend::amend;
pub use commit::commit;
pub use displace::displace;
pub use encode_selection::encode_selection;
pub use gate::gate;
pub use offset::offset;
pub use probe_at::probe_at;
pub use read::read;
pub use set_origin::set_origin;
