//! The pure core. Depends on nothing, and does not know a graph exists (R3, D7).
//!
//! Everything here is data in, data out. There is no I/O shape, no clock, and no
//! mutable state of any kind — no `static`, no interior mutability, no method that
//! retains across calls. That is L4, and it is what makes this layer's output derived
//! by construction rather than by assertion.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod addr;
mod block;
mod composition;
mod definition_set;
mod finding;
mod link;
mod order;
mod outcome;
mod plan;
mod port;
mod region;
mod signature;
mod signature_of;
mod tag;
mod value;
mod wire;

pub use addr::Addr;
pub use block::{Block, Body, BodyKind};
pub use composition::Composition;
pub use definition_set::DefinitionSet;
pub use finding::{kind, Finding};
pub use link::link;
pub use order::order;
pub use outcome::Outcome;
pub use plan::{Plan, Step};
pub use port::{Direction, Port, PortName, PortRef};
pub use region::Region;
pub use signature::Signature;
pub use signature_of::signature_of;
pub use tag::Tag;
pub use value::{Payload, Value};
pub use wire::Wire;
