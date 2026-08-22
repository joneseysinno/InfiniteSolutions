//! [`Outcome`] — a value **and** findings, never one or the other.

use crate::core::finding::Finding;

/// The result of linking or ordering.
///
/// Not a `Result`. D21 is explicit that a drawn cycle is **judged, not refused** —
/// *"the edge may exist; a finding says 'this closes a loop — mark the region
/// iterative or remove the edge'; derivation runs as far as it can and no further."*
/// The same stance already exists in the corpus as `Status::NotImplemented`, which
/// withholds `all_passed` without refusing to run.
///
/// A composition with one bad wire still runs the other ninety. Encoding that in the
/// return type is what stops it from being an intention.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Outcome<T> {
    /// What was produced, however partial.
    pub value: T,
    /// Everything wrong with it, each at an address, each with a remedy.
    pub findings: Vec<Finding>,
}

impl<T> Outcome<T> {
    /// An outcome with nothing wrong.
    pub fn clean(value: T) -> Self {
        Self {
            value,
            findings: Vec::new(),
        }
    }

    /// An outcome with findings.
    pub fn with(value: T, findings: Vec<Finding>) -> Self {
        Self { value, findings }
    }

    /// Whether anything was found. Note this is **not** "whether it runs".
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Outcome;

    #[test]
    fn an_outcome_carries_its_value_even_when_it_has_findings() {
        let o = Outcome::with(42, vec![]);
        assert_eq!(o.value, 42);
        assert!(!o.has_findings());
    }
}
