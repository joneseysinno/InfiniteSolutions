//! [`ArtifactRegistry`] — lifecycle without content (D25).

use std::collections::BTreeMap;

use super::ports::StoreRead;
use crate::core::{Addr, Revision};

/// How an artifact rebuilds itself: a pure function of what it reads.
///
/// Purity is not decoration here. It is what the discard test checks, and it is the
/// same purity D19 requires for compilability — one discipline, two payoffs.
type Rebuild = Box<dyn Fn(&dyn StoreRead) -> Vec<u8>>;

/// A registered derived artifact.
///
/// The runtime knows an artifact's **lifecycle** and never its **content**. It knows
/// what address ranges the artifact derives from, how to rebuild it, and whether it is
/// current. It does not know what it is.
///
/// `RenderList` is the first instance, registered by the presenter's binding. That
/// resolves the apparent tension between D5 (which places `RenderList` in the runtime
/// as a declared artifact) and D15 (which places visibility and culling in the
/// presenter): **the presenter owns the function, the runtime owns the schedule.**
pub struct Artifact {
    inputs: Vec<(Addr, Addr)>,
    rebuild: Rebuild,
    bytes: Option<Vec<u8>>,
    valid_at: Option<Revision>,
}

impl Artifact {
    /// The artifact's current bytes, if it is built.
    pub fn bytes(&self) -> Option<&[u8]> {
        self.bytes.as_deref()
    }

    /// The revision the current bytes were built at.
    pub fn valid_at(&self) -> Option<Revision> {
        self.valid_at
    }

    /// Whether `addr` falls in any declared input range.
    pub fn derives_from(&self, addr: &Addr) -> bool {
        self.inputs.iter().any(|(s, e)| addr.in_range(s, e))
    }
}

impl std::fmt::Debug for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Artifact")
            .field("inputs", &self.inputs.len())
            .field("built", &self.bytes.is_some())
            .field("valid_at", &self.valid_at)
            .finish()
    }
}

/// String-keyed registry of derived artifacts (R4, R16).
///
/// # Why a registry earns its place here
///
/// Not only F-1 avoidance. A registry makes **R12 free**: the runtime can drop and
/// rebuild any artifact and compare bytes *without knowing what it is*, so the discard
/// test is one generic harness that every artifact anyone ever registers is
/// automatically subject to. Under a closed enum the harness would be re-derived per
/// variant — and a check that must be rewritten per variant is a check that stops being
/// run, which is the mechanism behind F-7.
#[derive(Default)]
pub struct ArtifactRegistry {
    artifacts: BTreeMap<String, Artifact>,
}

impl ArtifactRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an artifact under `key`.
    ///
    /// `inputs` are half-open address ranges `[start, end)`. `rebuild` must be a pure
    /// function of what it reads through the reader — that is what the discard test
    /// checks, and it is the same purity D19 requires for compilability, so it is one
    /// discipline rather than two.
    ///
    /// Registering an existing key replaces it. A name is never reused for a second
    /// structure (R17); replacing an artifact under its own name is not reuse.
    pub fn register(
        &mut self,
        key: impl Into<String>,
        inputs: Vec<(Addr, Addr)>,
        rebuild: impl Fn(&dyn StoreRead) -> Vec<u8> + 'static,
    ) {
        self.artifacts.insert(
            key.into(),
            Artifact {
                inputs,
                rebuild: Box::new(rebuild),
                bytes: None,
                valid_at: None,
            },
        );
    }

    /// Reads an artifact.
    pub fn get(&self, key: &str) -> Option<&Artifact> {
        self.artifacts.get(key)
    }

    /// Every registered key.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.artifacts.keys().map(String::as_str)
    }

    /// Keys of artifacts deriving from `addr`.
    pub fn invalidated_by(&self, addr: &Addr) -> Vec<String> {
        self.artifacts
            .iter()
            .filter(|(_, a)| a.derives_from(addr))
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Marks an artifact out of date.
    pub fn invalidate(&mut self, key: &str) {
        if let Some(a) = self.artifacts.get_mut(key) {
            a.valid_at = None;
        }
    }

    /// Whether an artifact needs rebuilding.
    pub fn is_stale(&self, key: &str) -> bool {
        self.artifacts
            .get(key)
            .is_some_and(|a| a.valid_at.is_none())
    }

    /// Rebuilds one artifact at `at`. Returns whether the key exists.
    pub fn rebuild(&mut self, key: &str, store: &dyn StoreRead, at: Revision) -> bool {
        match self.artifacts.get_mut(key) {
            Some(a) => {
                a.bytes = Some((a.rebuild)(store));
                a.valid_at = Some(at);
                true
            }
            None => false,
        }
    }

    /// Drops an artifact's bytes without unregistering it. The discard half of R12.
    pub fn discard(&mut self, key: &str) {
        if let Some(a) = self.artifacts.get_mut(key) {
            a.bytes = None;
            a.valid_at = None;
        }
    }

    /// **R12's harness.** Rebuilds `key`, discards it, rebuilds again, and reports
    /// whether the two results are byte-identical.
    ///
    /// One function, no per-artifact code, every artifact automatically covered. This
    /// is the payoff D25 was chosen for.
    pub fn passes_discard_test(&mut self, key: &str, store: &dyn StoreRead, at: Revision) -> bool {
        if !self.rebuild(key, store, at) {
            return false;
        }
        let first = self.get(key).and_then(Artifact::bytes).map(<[u8]>::to_vec);
        self.discard(key);
        self.rebuild(key, store, at);
        let second = self.get(key).and_then(Artifact::bytes).map(<[u8]>::to_vec);
        first == second
    }

    /// Runs [`Self::passes_discard_test`] over every registered artifact, returning the
    /// keys that failed. An empty result is R12 satisfied for the whole layer.
    pub fn audit(&mut self, store: &dyn StoreRead, at: Revision) -> Vec<String> {
        let keys: Vec<String> = self.artifacts.keys().cloned().collect();
        keys.into_iter()
            .filter(|k| !self.passes_discard_test(k, store, at))
            .collect()
    }
}

impl std::fmt::Debug for ArtifactRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.artifacts.iter()).finish()
    }
}
