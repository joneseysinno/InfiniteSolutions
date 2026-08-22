//! The runtime layer: **motion in time**.
//!
//! Specification: `docs/specs/RUNTIME.md`. Decisions: D2, D4, D5, D6, D8, D17, D23,
//! D24, D25. Rules: R8–R14.
//!
//! # The membership test (D5)
//!
//! > If it survives a restart it belongs to the store. If it only means anything while
//! > something is running it belongs to the runtime.
//!
//! # Two prohibitions (D4)
//!
//! - **L1** — the runtime owns no thread pool. It is driven; it does not drive.
//! - **L2** — the runtime owns no storage. Nothing is authored here.
//!
//! # Layout
//!
//! [`core`] depends on nothing and does not know a graph exists. [`binding`] declares
//! the ports and drives them (D7). The split is a module and feature boundary rather
//! than a crate boundary, following `bion`'s proven shape.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod core;

#[cfg(feature = "binding")]
pub mod binding;
