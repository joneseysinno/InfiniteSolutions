//! Infinite Solutions: the platform facade, the OS portal, and the editor.
//!
//! Specifications: `docs/specs/FACADE.md`, `docs/specs/EDITOR.md`. Decision: D32.
//!
//! # Layout
//!
//! [`facade`] is the only module that may name a layer crate or a graphics crate
//! (R2, D29). [`portal`] is the operating-system boundary (D18). [`editor`] is the
//! app: it names neither.
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod editor;
pub mod facade;
pub mod portal;
