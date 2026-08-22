//! The five ports (D23, spec §3.1).
//!
//! The runtime declares what it needs as traits; the platform facade supplies the
//! implementations. **A sixth port requires a decision record.**
//!
//! `bion` built this once and enforced it: CNS reaches the store only through
//! `&dyn PnsReader`, checked by a CI grep of `src/cns` for DB tokens — the pattern R11
//! already cites. A runtime that names the store cannot be tested without a database,
//! which means the editor's latency behaviour cannot be tested at all.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod clock;
mod journal;
mod stale_feed;
mod store_read;
mod store_write;

pub use clock::Clock;
pub use journal::{Journal, JournalEntry};
pub use stale_feed::StaleFeed;
pub use store_read::{Records, StoreRead};
pub use store_write::{StoreWrite, Submission};
