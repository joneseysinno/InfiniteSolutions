//! [`Value`] — opaque payload plus opaque tag (D13).

use crate::core::tag::Tag;

/// An opaque payload.
///
/// Bytes, and bytes on purpose. The platform may store a value, move it along a wire,
/// match its tag at wire time, and detect that it changed. It may not parse, validate,
/// render, convert, evaluate, or check units (D13).
///
/// A second consequence worth having: bytes give D19's equivalence law an exact
/// meaning. *"Observationally identical"* becomes byte equality of every output, with
/// nothing to argue about — which is why `tests/equivalence.rs` can be one generic
/// harness rather than one comparison per value shape.
pub type Payload = Box<[u8]>;

/// A tagged, opaque value travelling along a wire.
///
/// The fifth `Value` model in this corpus — `bion`'s enum, `biomimicry`'s integer
/// millis plus records, `PropValue`, and a map of names to numbers — and intended to
/// be the last. It is not an enum, which is the point: `bion/soma`'s closed set would
/// have needed a seventh variant for video and an eighth for mesh, which is F-1 in the
/// one type that touches everything.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Value {
    tag: Tag,
    payload: Payload,
}

impl Value {
    /// Builds a value from an authored tag and opaque bytes.
    pub fn new(tag: Tag, payload: impl Into<Payload>) -> Self {
        Self {
            tag,
            payload: payload.into(),
        }
    }

    /// The tag, for matching.
    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    /// The bytes, uninterpreted.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}
