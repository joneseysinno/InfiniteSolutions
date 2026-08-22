//! The `Values` port.

use crate::core::{Addr, Value};

/// Reads an input at an address; writes an output at an address.
///
/// Payload and tag are both opaque (D13).
pub trait Values {
    /// The value at an address, if there is one.
    fn read(&self, at: &Addr) -> Option<Value>;

    /// Writes a computed value.
    ///
    /// Note this is not the input path. A keystroke is never a write (D24); anything
    /// arriving here is a derivation result, not a gesture.
    fn write(&mut self, at: &Addr, value: Value);
}
