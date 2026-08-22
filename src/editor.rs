//! The app (D32). Names no layer crate and no graphics crate (R2, D29).
//!
//! Specification: `docs/specs/EDITOR.md`. Appearance is authored spaces; behaviour
//! is an authored composition; primitives are the native blocks in [`blocks`].
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

pub mod addresses;
pub mod blocks;
pub mod tags;

mod genesis;
mod run;
mod styles;

pub use genesis::seed;
pub use run::{bind, run};
pub use styles::{bootstrap_default, Descriptor};
