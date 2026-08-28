//! **Retired.** This file is dead and should be deleted from the working tree.
//!
//! `frame(scene, surface, view, at)` lived here. D47 retired it: it resolved its own
//! `SceneSet`, submitted, and dropped the set, and both D44's fill resolution and
//! D46's batching need the set the placement was built from. Its replacement is
//! [`crate::binding::compose`] in `compose.rs`, which returns `(SceneSet, Placement)`
//! and leaves submitting to the caller.
//!
//! `binding.rs` no longer declares `mod frame;`, so nothing here is compiled. The file
//! remains only because the change that retired it was written across a bridge that
//! cannot delete. **Delete it.** The name is retired, not recycled (R17) — do not put
//! anything else in this file.
