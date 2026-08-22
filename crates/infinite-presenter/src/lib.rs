//! The presenter layer: **the embedding**.
//!
//! Specification: `docs/specs/PRESENTER.md`. Decisions: D2, D5, D7, D15, D17, D20,
//! D25, D29, D30, D31. Rules: R3, R5, R11, R15, R23, R27.
//!
//! # The membership test (spec §1)
//!
//! > If it survives a restart it belongs to the store. If it is true of a program
//! > before it runs it belongs to the compositor. If it is only true while something
//! > is running it belongs to the runtime. **If it is only true of one view of one
//! > screen, it belongs here** — and it is discarded the moment the view changes.
//!
//! # Two laws
//!
//! - **L5** — the presenter *mints* no identity. Every reference to a thing is the
//!   store's address. No id of its own, no handle, no index standing in for a node,
//!   and no map keyed by anything but an [`core::Addr`].
//! - **L6** — the presenter authors nothing. There is no write port. Camera, collapse
//!   and selection are read; hover and a drag in progress are the runtime's pending
//!   set (D8).
//!
//! L5 is the sentence `infinitedb-spatial-layer.md` §6 wrote first — *nothing in the
//! embedding layer carries identity* — with the one correction the corpus forced.
//! `hyper-ui`'s `SceneNode` holds no address at all, and the result is a layer that
//! **cannot be hit-tested** and had to smuggle `selected: bool` into its geometry
//! record. Referring to the store's address is not minting identity; it is the only
//! thing that keeps this layer answerable.
//!
//! # Layout
//!
//! [`core`] depends on nothing and does not know a graph exists. [`binding`] declares
//! the ports and drives them (D7). The split is a module and feature boundary rather
//! than a crate boundary, following `bion`'s proven shape, `infinite-runtime` and
//! `infinite-compositor`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod core;

#[cfg(feature = "binding")]
pub mod binding;
