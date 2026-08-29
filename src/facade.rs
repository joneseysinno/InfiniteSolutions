//! The platform facade (D10, D32).
//!
//! Specification: `docs/specs/FACADE.md`. The only module that may name a layer
//! crate. Implements the thirteen ports, converts the three `Addr` types and the
//! two `Revision` types, registers derived artifacts neither crate can see (D30),
//! and narrows `f64` to `f32` in exactly one file.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod addr;
mod artifacts;
mod finding;
mod open;
pub mod ports;
mod tick;
mod present;
mod record;
mod run;
mod undo;

pub use addr::{
    compositor_addr, presenter_addr, presenter_revision, runtime_addr, runtime_revision,
    significant_bits,
};
pub use artifacts::{register, register_plan};
pub use finding::{from_empty_screen, from_precision_floor};
pub use record::{
    decode_composition, decode_selection, decode_space, decode_style, encode_composition,
    encode_selection, encode_space, encode_style, BlockRecord, CompositionRecord, PortRecord,
    SpaceRecord, WireRecord,
};
pub use open::{open, open_with_options, Store};
pub use present::SelectionView;
pub use tick::TickReport;
pub use undo::UNDO_KEY;
