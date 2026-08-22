//! [`Signature`] — a block's declared ports.

use crate::core::port::{Direction, Port};

/// The declared ports of a block.
///
/// A composition's signature is **derived**, not authored: it is the composition's
/// unbound ports ([`crate::core::signature_of`], D14.6). That one rule is what makes
/// composition close, and closure is what turns primitives into blocks into whole
/// applications — without it composition stops after one flat layer.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Signature {
    /// Declared ports, in authored order.
    pub ports: Vec<Port>,
}

impl Signature {
    /// Ports the block reads.
    pub fn inputs(&self) -> impl Iterator<Item = &Port> {
        self.ports.iter().filter(|p| p.direction == Direction::In)
    }

    /// Ports the block writes.
    pub fn outputs(&self) -> impl Iterator<Item = &Port> {
        self.ports.iter().filter(|p| p.direction == Direction::Out)
    }

    /// Looks a port up by name.
    pub fn port(&self, name: &str) -> Option<&Port> {
        self.ports.iter().find(|p| &*p.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::Signature;
    use crate::core::port::{Direction, Port};
    use crate::core::tag::Tag;

    fn sig() -> Signature {
        Signature {
            ports: vec![
                Port::new("roster", Direction::In, Tag::new("roster")),
                Port::new("minutes", Direction::In, Tag::new("minutes")),
                Port::new("plan", Direction::Out, Tag::new("practice-plan")),
            ],
        }
    }

    #[test]
    fn inputs_and_outputs_partition_the_ports() {
        let s = sig();
        assert_eq!(s.inputs().count(), 2);
        assert_eq!(s.outputs().count(), 1);
        assert_eq!(s.inputs().count() + s.outputs().count(), s.ports.len());
    }

    #[test]
    fn a_port_is_found_by_name() {
        assert!(sig().port("plan").is_some());
        assert!(sig().port("nonexistent").is_none());
    }
}
