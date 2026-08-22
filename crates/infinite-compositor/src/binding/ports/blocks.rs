//! The `Blocks` port.

use crate::core::{Signature, Value};

/// A native primitive: opaque values in, opaque values out.
///
/// The compositor knows a block's **shape** and never what it computes (L3, R31).
/// Every number in the system is behind this trait, which is what keeps
/// Navier-Stokes, quadrature, and an ACI clause out of a wiring substrate.
pub trait Primitive {
    /// Invokes the primitive. Values are opaque both ways (D13).
    fn invoke(&self, inputs: &[Value]) -> Vec<Value>;
}

/// Resolves a native block key to its declared signature and its primitive.
///
/// String-keyed (R4, R16). A key with no registration is an `unresolved-block` finding
/// at link time rather than a panic at run time — the cost of a registry, paid where
/// the author can see it.
pub trait Blocks {
    /// The declared ports of a native block.
    fn signature(&self, key: &str) -> Option<Signature>;

    /// The primitive itself.
    fn primitive(&self, key: &str) -> Option<&dyn Primitive>;
}
