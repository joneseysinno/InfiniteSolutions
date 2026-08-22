//! [`StoreRead`] — records in, and straight back out.

use crate::core::{Addr, Revision};

/// Records as they cross the boundary: address plus opaque payload (D13).
///
/// Named so the shape is one thing with one name (R17) rather than an inline tuple
/// repeated at every call site — and so the R11 lint has something to grep for.
pub type Records = Vec<(Addr, Vec<u8>)>;

/// Reads a range of records from the store.
///
/// # The R11 obligation
///
/// A record may pass **through** the runtime within a single `tick`. It may not be
/// **retained** across ticks, except inside a declared derived artifact — which is why
/// [`ArtifactRegistry`](crate::binding::ArtifactRegistry) rebuild functions take a
/// `&dyn StoreRead` and return bytes rather than holding the reader.
///
/// Checked by a lint over the layer's struct fields: no field whose type is or contains
/// a record. Locals and arguments are unrestricted. This is the field-level form of
/// `bion`'s CI grep, and being mechanical is the entire point.
pub trait StoreRead {
    /// Records in `[start, end)` at `at`, in key order.
    ///
    /// Key order is address order is spatial locality (`Addr` property 1), which is
    /// what makes a subtree scan affordable — O2 stated as a requirement.
    ///
    /// Payloads are opaque (D13). The runtime does not parse, validate, convert or
    /// render them.
    fn range(&self, start: &Addr, end: &Addr, at: Revision) -> Records;

    /// The newest revision this reader can serve.
    fn head(&self) -> Revision;
}
