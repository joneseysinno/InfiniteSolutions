//! [`Tag`] — opaque, and the only operation on one is match (D13).

/// An opaque tag on a value or a port.
///
/// D13: *the platform's only operation on a tag is **match** — can this output connect
/// to that input. It compares tags; it never interprets one.*
///
/// So there is no `parse`, no `convert`, no `validate`, no ordering, and no `Display`.
/// What a tag *means* is the app's, and D27 records the consequence: the platform
/// cannot tell a compute port from an interface port, because it has no operation that
/// would reveal the difference. That is why there is one linker and not two.
///
/// [`Tag::label`] exists for exactly one purpose — building the `said` and `wanted`
/// text of a [`crate::core::Finding`] (spec §6). Echoing an authored string into an
/// error message is not interpretation. Anything else that calls it is interpreting a
/// tag, which is a D13 violation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tag(Box<str>);

impl Tag {
    /// Wraps an authored tag.
    pub fn new(label: impl Into<Box<str>>) -> Self {
        Self(label.into())
    }

    /// Whether a value carrying `self` may connect to a port wanting `other`.
    ///
    /// This is the whole of the platform's relationship with tags. It is deliberately
    /// a named method rather than `==` at the call site: naming it keeps D22's
    /// boundary visible — a tag **validates** a connection the author drew, and never
    /// **discovers** one.
    pub fn matches(&self, other: &Tag) -> bool {
        self.0 == other.0
    }

    /// The authored label. For finding text only — see the type documentation.
    pub fn label(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Tag;

    #[test]
    fn a_tag_matches_only_itself() {
        let drill = Tag::new("drill");
        assert!(drill.matches(&Tag::new("drill")));
        assert!(!drill.matches(&Tag::new("roster")));
    }

    #[test]
    fn matching_is_symmetric() {
        let a = Tag::new("load");
        let b = Tag::new("load");
        assert_eq!(a.matches(&b), b.matches(&a));
    }
}
