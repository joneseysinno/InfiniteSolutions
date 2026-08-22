//! The compositor layer: **the static structure of a program**.
//!
//! Specification: `docs/specs/COMPOSITOR.md`. Decisions: D2, D7, D11, D13, D14, D17,
//! D19, D21, D22, D26, D27, D28. Rules: R31, R4, R16, R27.
//!
//! # The membership test (§1)
//!
//! > If it is true of a program **before it runs** it belongs here. If it is only true
//! > **while it is running** it belongs to the runtime. If it **survives a restart** it
//! > belongs to the store.
//!
//! The store has logical time and the runtime has *now*. This layer has neither:
//! linking a composition at revision N is a pure function of the definitions at
//! revision N.
//!
//! # Two laws
//!
//! - **L3** — the compositor contains no math (R31). It owns the contract and
//!   lifecycle of a computation; every number is inside a block.
//! - **L4** — the compositor is a function, not a place. It holds nothing across a
//!   call, so everything it produces is derived (R12) and passes the discard test by
//!   construction.
//!
//! # Layout
//!
//! [`core`] depends on nothing and does not know a graph exists. [`binding`] declares
//! the ports and drives them (D7). The split is a module and feature boundary rather
//! than a crate boundary, following `bion`'s proven shape and `infinite-runtime`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod core;

#[cfg(feature = "binding")]
pub mod binding;
