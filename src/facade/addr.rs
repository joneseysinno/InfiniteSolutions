//! The three `Addr` conversions and the two `Revision` conversions (O13).
//!
//! Each is a newtype wrap of the inner value. The moment a conversion needs logic,
//! O13's trigger has fired — promote `Addr` to a zero-dependency crate, with a
//! decision record. Do not quietly add the logic.

use infinite_compositor::core::Addr as CompositorAddr;
use infinite_presenter::core::{Addr as PresenterAddr, Revision as PresenterRevision};
use infinite_runtime::core::{Addr as RuntimeAddr, Revision as RuntimeRevision};

/// Store bytes → runtime address. A wrap, nothing else.
pub fn runtime_addr(bytes: &[u8]) -> RuntimeAddr {
    RuntimeAddr::new(bytes.to_vec())
}

/// Store bytes → compositor address. A wrap, nothing else.
pub fn compositor_addr(bytes: &[u8]) -> CompositorAddr {
    CompositorAddr::new(bytes.to_vec())
}

/// Store bytes → presenter address. A wrap, nothing else.
pub fn presenter_addr(bytes: &[u8]) -> PresenterAddr {
    PresenterAddr::new(bytes.to_vec())
}

/// Store revision sequence → runtime revision. A wrap, nothing else.
pub fn runtime_revision(n: u64) -> RuntimeRevision {
    RuntimeRevision::new(n)
}

/// Store revision sequence → presenter revision. A wrap, nothing else.
pub fn presenter_revision(n: u64) -> PresenterRevision {
    PresenterRevision::new(n)
}
