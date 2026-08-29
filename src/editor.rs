//! The app (D32). Names no layer crate and no graphics crate (R2, D29).
//!
//! Specification: `docs/specs/EDITOR.md`. Appearance is authored spaces; behaviour
//! is an authored composition; primitives are the native blocks in [`blocks`].
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

pub mod addresses;
pub mod blocks;
pub mod mint;
pub mod tags;

mod genesis;
mod inspector;
mod palette;
mod run;
mod styles;

pub use genesis::seed;
pub use inspector::{apply_origin, refresh as refresh_inspector};
pub use run::{bind, run};
pub use styles::{bootstrap_default, Descriptor};
