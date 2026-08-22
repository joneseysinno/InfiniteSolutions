//! [`Finding`] — the error surface (D16, spec §6).
//!
//! Specified before the plan, because it is read more often. D16 — *a child should be
//! able to build a full stack app* — rules out stack traces as the error surface, and
//! a finding is the only part of this layer a person ever reads.

use crate::core::addr::Addr;

/// The linker's own finding kinds. This list is the editor's error surface, so it must
/// stay complete.
///
/// The kind is a `&'static str` rather than an enum even though the linker's own set
/// is closed, and the reason is the two-domain test (R32): a physics facade wants to
/// say *"this boundary is unconstrained"*, and Coach Assistant wants to say *"this
/// drill declares twelve players and the roster has ten"*. Both are findings, both
/// want the editor's existing rendering, and neither is the linker's. The finding
/// **channel** is platform; the finding **kinds** are open.
pub mod kind {
    /// A required input port has no wire. `biomimicry` M12's error, verbatim.
    pub const UNSATISFIED_IMPORT: &str = "unsatisfied-import";
    /// A wire connects tag A to a port wanting tag B (D13).
    pub const TAG_MISMATCH: &str = "tag-mismatch";
    /// A port is bound more times than it admits.
    pub const ARITY: &str = "arity";
    /// A wire closes a loop outside a region marked iterative (D21).
    pub const CYCLE: &str = "cycle";
    /// A body names a native key with no registration, or an address with no
    /// definition.
    pub const UNRESOLVED_BLOCK: &str = "unresolved-block";
    /// A composition marked compilable reads something outside its declared inputs
    /// (D19).
    pub const NOT_PURE: &str = "not-pure";
}

/// What the person reads.
///
/// Three constraints, each of them a test rather than a review note:
///
/// 1. **Every finding has a site.** So *"go to the error"* is not an editor feature —
///    it is a zoom (D20).
/// 2. **Every finding has a remedy.** D16 means the message says what to do next, not
///    only what went wrong. An empty remedy is a defect.
/// 3. **One cause yields one finding.** A single unsatisfied import must not produce a
///    cascade.
///
/// Constraint 2 is enforced as far as a type can enforce it: [`Finding::new`] takes
/// all four text fields, so a finding cannot be built without one being supplied. That
/// it is non-empty is `tests/findings.rs`'s job.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Finding {
    /// Where. The editor zooms here.
    pub site: Addr,
    /// Registry key — see [`kind`].
    pub kind: &'static str,
    /// What the composition says.
    pub said: String,
    /// What would have satisfied the linker.
    pub wanted: String,
    /// What to do next. Never empty.
    pub remedy: String,
}

impl Finding {
    /// Builds a finding.
    pub fn new(
        site: Addr,
        kind: &'static str,
        said: impl Into<String>,
        wanted: impl Into<String>,
        remedy: impl Into<String>,
    ) -> Self {
        Self {
            site,
            kind,
            said: said.into(),
            wanted: wanted.into(),
            remedy: remedy.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{kind, Finding};
    use crate::core::addr::Addr;

    #[test]
    fn a_finding_carries_a_site_and_a_remedy() {
        let f = Finding::new(
            Addr::new(vec![0x01]),
            kind::TAG_MISMATCH,
            "a roster",
            "a drill",
            "rewire this port, or change the block that feeds it",
        );
        assert!(!f.remedy.is_empty());
        assert_eq!(f.site, Addr::new(vec![0x01]));
    }
}
