//! The pure core. Depends on nothing, and does not know a graph exists (R3, D7).
//!
//! Everything here is data in, data out. There is no I/O shape, no trait object, and
//! no clock — [`Instant`] is a monotonic count handed in from outside, which is what
//! makes the schedule deterministic and therefore what makes D19's equivalence law
//! checkable.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod addr;
mod budget;
mod coalesce;
mod frontier;
mod instant;
mod outcome;
mod pending;
mod priority;
mod revision;

pub use addr::Addr;
pub use budget::Budget;
pub use coalesce::{coalesce, Coalesced};
pub use frontier::Frontier;
pub use instant::Instant;
pub use outcome::Outcome;
pub use pending::{Overflow, Pending, PendingSet, Seq};
pub use priority::{priority, Priority};
pub use revision::Revision;
