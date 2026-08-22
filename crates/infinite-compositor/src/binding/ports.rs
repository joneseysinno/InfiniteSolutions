//! The five ports (D26, spec §3.1).
//!
//! The compositor declares what it needs as traits; the platform facade supplies the
//! implementations. **A sixth port requires a decision record.**
//!
//! D3 established that a computation must not depend on the runtime, or the two become
//! mutual hostages. D23 ran the argument for the runtime. The third run is sharper
//! than either: the editor must answer *"if I drop this wire here, does it link?"*
//! about a wire that is still **pending** and therefore by definition not in the store.
//! A compositor that read the store directly could not answer it at all.
//!
//! **There is no `Clock`**, and the absence is stated positively so it stays true: the
//! compositor has no *now* (R10, D5). If this layer ever needs one, the item belongs in
//! the runtime.
//!
//! O10 note: [`blocks::Blocks`] is where a *"may this composition use that block"*
//! check would be inserted. Do not build it so that it cannot be.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod backends;
mod blocks;
mod definitions;
mod provenance;
mod values;

pub use backends::Backends;
pub use blocks::{Blocks, Primitive};
pub use definitions::Definitions;
pub use provenance::Provenance;
pub use values::Values;
